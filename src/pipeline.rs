use crate::AnalysisLocations;
use crate::aggregate::*;
use crate::aqi::AQIValue;
use crate::spliter::*;
use crate::CityId;
use crate::gen::challenger::{Batch,Measurement};

pub fn get_final_aggregate<'a>(preaggregated: impl Iterator<Item = &'a CityParticleMap>) -> CityParticleMap {
    preaggregated
        .flatten()
        .fold(
            CityParticleMap::default(),
            |mut result, (cityid, preaggregate)| {
            match result.get_mut(cityid) {
                Some(x) => *x += *preaggregate,
                None => assert!(result.insert(*cityid, *preaggregate).is_none()),
            };
            result
        })
}

pub fn run_pipeline(locations: AnalysisLocations, batches_iter: impl Iterator<Item=Batch> + Send) {
    let localize = |meas : Measurement, batch_seq_id: i64| {
        if meas.latitude < 47.40724 || meas.latitude > 54.9079 || meas.longitude < 5.98815 || meas.longitude > 14.98853 {
            // outside germany, don't bother searching
            return None;
        }
        let location = locations.localize(meas.latitude, meas.longitude).next()?;
        Some(LocalizedMeasurement {
            batch_seq_id,
            cityid: location.cityid,
            timestamp_seconds: meas.timestamp.unwrap().seconds,
            p1: meas.p1,
            p2: meas.p2,
        })
    };
    use rayon::prelude::*;
    let IterPair(batch_current_iter, batch_lastyear_iter) = batches_iter
        .map(|batch| {
        let batch_seq_id = batch.seq_id;

        let current_iter = batch.current
            .into_par_iter()
            .filter(|m| m.p1 >= 0.0 && m.p2 >= 0.0)
            .filter_map(move |m| localize(m, batch_seq_id))
            .collect::<Vec<_>>()
            .into_iter();
        let lastyear_iter = batch.lastyear
            .into_par_iter()
            .filter(|m| m.p1 >= 0.0 && m.p2 >= 0.0)
            .filter_map(move |m| localize(m, batch_seq_id))
            .collect::<Vec<_>>()
            .into_iter();
        (current_iter, lastyear_iter)
    }).spliter();

    let current_iter = batch_current_iter
        .flatten()
        .partition_5min()
        .inspect(|x| {
            println!("Have current batch from {} to {}",
                x.first().map(|first| first.timestamp_seconds).unwrap_or(0),
                x.last().map(|first| first.timestamp_seconds).unwrap_or(0));
        })
        .preaggregate();
    let lastyear_iter = batch_lastyear_iter
        .flatten()
        .partition_5min()
        .preaggregate();

    struct TopKCity {
        improvement: i32,
        cityid: CityId,
    }
    struct SlidingWindowContents {
        current_1d_aggregates: CityParticleMap,
        current_5d_aggregates: CityParticleMap,
        lastyear_5d_aggregates: CityParticleMap,
    }
    let resiter = IterPair(current_iter, lastyear_iter)
        .with_analysis_windows(5*24*(60/5), 5*24*(60/5), |window, cache: Option<SlidingWindowContents>| {
            debug_assert_eq!(window.current.len(), 5*24*(60/5));
            debug_assert_eq!(window.lastyear.len(), 5*24*(60/5));
            let active_cities = window.current
                .clone() // only clones the iterator, not the underlying values
                .rev()
                .take(2) // get last 2 batches of 5 minutes = 10 minute window
                .flat_map(|city_aggregate_map : &CityParticleMap| city_aggregate_map.into_iter().map(|(k,_v)| k))
                .cloned() // clone city ids. since this is a u32, it should be a simple copy
                .collect::<ActiveCities>();
            /* {
                let current_samples = window.current.clone().map(|x| x.len()).sum::<usize>();
                let lastyear_samples = window.lastyear.clone().map(|x| x.len()).sum::<usize>();
                println!("window has {} current/{} lastyear samples ({} total)", current_samples, lastyear_samples, current_samples + lastyear_samples);
            } */
            
            let (last_day_aqi, current_aggregates, lastyear_aggregates, cache) = {
                let (current_1d_aggregates, current_5d_aggregates, lastyear_5d_aggregates) = if let Some(SlidingWindowContents { current_1d_aggregates, current_5d_aggregates, lastyear_5d_aggregates }) = cache {
                    // cache exists add values from cache to newest value
                    // to obtain the value for this window
                    let newest : &CityParticleMap = window.current.clone().last().unwrap();
                    let newest_lastyear : &CityParticleMap = window.lastyear.clone().last().unwrap();
                    let mut new_current_1d_aggregates = newest.clone();
                    let mut new_current_5d_aggregates = newest.clone();
                    let mut new_lastyear_5d_aggregates = newest_lastyear.clone();
                    map_add(&mut new_current_1d_aggregates, &current_1d_aggregates);
                    map_add(&mut new_current_5d_aggregates, &current_5d_aggregates);
                    map_add(&mut new_lastyear_5d_aggregates, &lastyear_5d_aggregates);
                    (new_current_1d_aggregates, new_current_5d_aggregates, new_lastyear_5d_aggregates) 
                } else {
                    // generate the aggreagate from scrath using the 5 day window
                    let last_day_windows = window.current
                        .clone()
                        .rev()
                        .take(24*(60/5)); // take values from last day
                    let current_1d_aggregates = get_final_aggregate(last_day_windows);
                    let current_aggregates = get_final_aggregate(window.current.clone());
                    let lastyear_aggregates = get_final_aggregate(window.lastyear.clone());
                    (current_1d_aggregates, current_aggregates, lastyear_aggregates)
                };

                // create the cache by removing dropping the now obsolete last element from each window
                // to be examined
                // remove the first CityParticleMap from one day ago
                let oldest_current_1d = window.current.clone().rev().nth(24*(60/5)-1).unwrap();
                let mut current_1d_aggregates_tocache = current_1d_aggregates.clone();
                map_sub(&mut current_1d_aggregates_tocache, oldest_current_1d);


                // remove the first CityParticleMap from 5 days ago
                // since the window is exactly 5 days big, this should be the first item
                let oldest_current_5d = window.current.clone().next().unwrap();
                let mut current_5d_aggregates_tocache = current_5d_aggregates.clone();
                map_sub(&mut current_5d_aggregates_tocache, oldest_current_5d);

                // remove the first CityParticleMap from 1 year 5 days ago
                // since the window is exactly 5 days big, this should be the first item
                let oldest_lastyear_5d = window.lastyear.clone().next().unwrap();
                let mut lastyear_5d_aggregates_tocache = lastyear_5d_aggregates.clone();
                map_sub(&mut lastyear_5d_aggregates_tocache, oldest_lastyear_5d);

                let newcache = SlidingWindowContents {
                    current_1d_aggregates: current_1d_aggregates_tocache,
                    current_5d_aggregates: current_5d_aggregates_tocache,
                    lastyear_5d_aggregates: lastyear_5d_aggregates_tocache,
                };
                (current_1d_aggregates, current_5d_aggregates, lastyear_5d_aggregates, newcache)
            };


            let mut improvements = current_aggregates
                .into_par_iter()
                .filter(|(cityid, _)| active_cities.is_active(*cityid))
                .filter_map(|(cityid, aggregate)| {
                    // Get this year's 5 day AQI
                    let current_aqip1 = AQIValue::from_pm10(aggregate.p1())?;//.expect("Average current p1 invalid");
                    let current_aqip2 = AQIValue::from_pm25(aggregate.p2())?;//.expect("Average current p2 invalid");
                    let current_aqi = current_aqip1.get_asdebs().max(current_aqip2.get_asdebs());

                    // Get last year's 5 day AQI
                    // If no sensor data was available in given city for lastyear window period,
                    // calculation of improvement is impossible. Skip those cases.
                    let lastyear_aggregate = lastyear_aggregates.get(&cityid)?;
                    let lastyear_aqip1 = AQIValue::from_pm10(lastyear_aggregate.p1())?;//.expect("Average lastyear p1 invalid");
                    let lastyear_aqip2 = AQIValue::from_pm25(lastyear_aggregate.p2())?;//.expect("Average lastyear p2 invalid");
                    let lastyear_aqi = lastyear_aqip1.get_asdebs().max(lastyear_aqip2.get_asdebs());

                    let improvement = current_aqi - lastyear_aqi;

                    Some(TopKCity {
                        improvement,
                        cityid
                    })
                })
                .collect::<Vec<TopKCity>>();
            improvements.sort_by_key(|x| x.improvement);
            let result = improvements.into_iter()
                .take(50) // top 50
                .enumerate()
                .map(|(position, city)| {

                    // Calculate the aggregate for the lasy day (only for the top 50)
                    assert!(active_cities.is_active(city.cityid));
                    let lastday_aggregate = last_day_aqi
                        .get(&city.cityid)
                        .expect("Know city is contained in last 10 min");
                    let current_aqip1 = AQIValue::from_pm10(lastday_aggregate.p1());
                    #[cfg(debug_assertions)]
                    if current_aqip1.is_none() {
                        println!(
                            "24h p1 for city {} (cityid {}) outside scale",
                            locations.lookup(city.cityid).to_owned(),
                            city.cityid
                        );
                        let last24h_iter = window
                            .current
                            .clone()
                            .rev()
                            .take(24*(60/5) - 1)
                            .filter_map(|city_aggregate_map| city_aggregate_map.get(&city.cityid));
                        let last24h_values = last24h_iter
                            .clone()
                            .copied()
                            .collect::<Vec<ParticleAggregate>>();
                        let last24h_actual = last24h_iter.copied().collect::<ParticleAggregate>();
                        println!(
                            " Cached 24h p1: {:?}, actual: {:?}, values:\n{:?}",
                            cache.current_1d_aggregates.get(&city.cityid),
                            last24h_actual,
                            last24h_values);
                    }
                    let current_aqip1 = current_aqip1
                        .map(|v| v.get_asdebs())
                        .unwrap_or(i32::MAX); // FIXME: Handle Average last 24h p1 outside scale
                    let current_aqip2 = AQIValue::from_pm25(lastday_aggregate.p2())
                        .map(|v| v.get_asdebs())
                        .unwrap_or(i32::MAX); // FIXME: Handle Average last 24h p2 outside scale

                    crate::gen::challenger::TopKCities {
                        position: position as i32,
                        city: locations.lookup(city.cityid).to_owned(),
                        current_aqip1,
                        current_aqip2,
                        average_aqi_improvement: -city.improvement,
                    }
                })
                .collect::<Vec<_>>();
            let res = crate::gen::challenger::ResultQ1 {
                benchmark_id: 0,
                batch_seq_id: 0,
                topkimproved: result,
            };
            (res, cache)
        });

    for _res in resiter {
        // dbg!(res.topkimproved);
    }
    use std::sync::atomic::Ordering;
    println!("Cache hits/misses/outside: {}/{}/{}", locations.cachehits.load(Ordering::SeqCst), locations.cachemisses.load(Ordering::SeqCst), locations.outsidecachehits.load(Ordering::SeqCst));
}
