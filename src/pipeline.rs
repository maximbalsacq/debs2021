use crate::AnalysisLocations;
use crate::aggregate::*;
use crate::aqi::AQIValue;
use crate::spliter::*;
use crate::CityId;
use crate::gen::challenger::{Batch,Measurement};

pub fn get_final_aggregate<'a>(active_cities: &ActiveCities, preaggregated: impl Iterator<Item = &'a CityParticleMap>) -> CityParticleMap {
    let mut result = CityParticleMap::default();
    preaggregated
        .flatten()
        .filter(|(cityid, _)| active_cities.is_active(**cityid))
        .for_each(|(cityid, preaggregate)| {
            match result.get_mut(cityid) {
                Some(x) => *x += *preaggregate,
                None => assert!(result.insert(*cityid, *preaggregate).is_none()),
            };
        });
    result
}

pub fn run_pipeline(locations: AnalysisLocations, batches_iter: impl Iterator<Item=Batch>) {
    let localize = |meas : Measurement, batch_seq_id: i64| {
        let location = locations.localize(meas.latitude, meas.longitude).next()?;
        Some(LocalizedMeasurement {
            batch_seq_id,
            cityid: location.cityid,
            timestamp_seconds: meas.timestamp.unwrap().seconds,
            p1: meas.p1,
            p2: meas.p2,
        })
    };
    let IterPair(batch_current_iter, batch_lastyear_iter) = batches_iter.map(|batch| {
        let batch_seq_id = batch.seq_id;

        let current_iter = batch.current
            .into_iter()
            .filter_map(move |m| localize(m, batch_seq_id));
        let lastyear_iter = batch.lastyear
            .into_iter()
            .filter_map(move |m| localize(m, batch_seq_id));
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
    let resiter = IterPair(current_iter, lastyear_iter)
        .with_analysis_windows(5*24*(60/5), 5*24*(60/5), |window| {
            let active_cities = window.current
                .clone() // only clones the iterator, not the underlying values
                .flat_map(|city_aggregate_map : &CityParticleMap| city_aggregate_map.keys())
                .cloned() // clone city ids. since this is a u32, it should be a simple copy
                .collect::<ActiveCities>();
            
            let last_day_aqi = {
                let last_day_windows = window.current
                    .clone()
                    .skip(4*24*(60/5)); // skip to lasy day
                get_final_aggregate(&active_cities, last_day_windows)
            };

            let current_aggregates = get_final_aggregate(&active_cities, window.current);
            let lastyear_aggregates = get_final_aggregate(&active_cities, window.lastyear);

            let mut improvements = current_aggregates
                .into_iter()
                .filter_map(|(cityid, aggregate)| {
                    // Get this year's 5 day AQI
                    let current_aqip1 = AQIValue::from_pm10(aggregate.p1()).expect("Average current p1 invalid");
                    let current_aqip2 = AQIValue::from_pm25(aggregate.p2()).expect("Average current p2 invalid");
                    let current_aqi = current_aqip1.get_asdebs().max(current_aqip2.get_asdebs());

                    // Get last year's 5 day AQI
                    // If no sensor data was available in given city for lastyear window period,
                    // calculation of improvement is impossible. Skip those cases.
                    let lastyear_aggregate = lastyear_aggregates.get(&cityid)?;
                    let lastyear_aqip1 = AQIValue::from_pm10(lastyear_aggregate.p1()).expect("Average lastyear p1 invalid");
                    let lastyear_aqip2 = AQIValue::from_pm25(lastyear_aggregate.p2()).expect("Average lastyear p2 invalid");
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
                    let lastday_aggregate = last_day_aqi
                        .get(&city.cityid)
                        .expect("Know city is contained in last 10 min");
                    let current_aqip1 = AQIValue::from_pm10(lastday_aggregate.p1())
                        .expect("Average last 24h p1 invalid")
                        .get_asdebs();
                    let current_aqip2 = AQIValue::from_pm25(lastday_aggregate.p2())
                        .expect("Average last 24h p2 invalid")
                        .get_asdebs();

                    crate::gen::challenger::TopKCities {
                        position: position as i32,
                        city: locations.lookup(city.cityid).to_owned(),
                        current_aqip1,
                        current_aqip2,
                        average_aqi_improvement: city.improvement,
                    }
                })
                .collect::<Vec<_>>();
            crate::gen::challenger::ResultQ1 {
                benchmark_id: 0,
                batch_seq_id: 0,
                topkimproved: result,
            }
        });

    for res in resiter.take(100) {
        dbg!(res.topkimproved);
    }
}
