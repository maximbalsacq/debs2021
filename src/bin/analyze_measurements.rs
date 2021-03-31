use debs2021::gen::challenger::*;
use debs2021::io::load_batch;


use prost_types::Timestamp;
use chrono::naive::NaiveDateTime;
#[derive(Debug)]
struct TimeInfo {
    havetimestamps: bool,
    timestampssorted: bool,
    min: Option<NaiveDateTime>,
    max: Option<NaiveDateTime>,
    last_invalid: Option<Timestamp>
}

fn analyze_timestamps<'a>(timestamps: impl Iterator<Item=&'a Timestamp>) -> TimeInfo {
    use std::convert::TryInto;
    let mut havetimestamps = false;
    let mut min = None;
    let mut max = None;
    let mut last_invalid = None;
    let mut timestampssorted = true;
    for ts in timestamps {
        let datetime = match NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos.try_into().unwrap()) {
            None => {
                last_invalid = Some(ts);
                continue;
            }
            Some(x) => x,
        };

        if !havetimestamps {
            // first iteration only
            min = Some(datetime);
            max = Some(datetime);
            havetimestamps = true;
        } else if datetime < min.unwrap() {
            // timestamps are not monotonically increasing
            timestampssorted = false;
            min = Some(datetime);
        } else if datetime > max.unwrap() {
            max = Some(datetime);
        }
    }
    TimeInfo {
        havetimestamps,
        timestampssorted,
        min,
        max,
        last_invalid: last_invalid.cloned()
    }
}

#[derive(Debug)]
struct MeasurementInfo {
    current_tsinfo: TimeInfo,
    current_complete: bool,
    last_tsinfo: TimeInfo,
    last_complete: bool,
    outside_germany: usize,
}

fn analyze_one(batch: &Batch) -> MeasurementInfo {
    let current_timestamps = batch.current
            .iter()
            .flat_map(|x| x.timestamp.clone())
            .collect::<Vec<Timestamp>>();
    let current_complete = batch.current.len() == current_timestamps.len();
    let current_tsinfo = analyze_timestamps(current_timestamps.iter());

    let last_timestamps = batch.lastyear
            .iter()
            .flat_map(|x| x.timestamp.clone())
            .collect::<Vec<Timestamp>>();
    let last_complete = batch.lastyear.len() == last_timestamps.len();
    let last_tsinfo = analyze_timestamps(last_timestamps.iter());
    let outside_germany = batch.lastyear.iter().chain(batch.current.iter()) 
        .filter(|measurement| measurement.latitude < 47.40724 || measurement.latitude > 54.9079 || measurement.longitude < 5.98815 || measurement.longitude > 14.98853)
        .count();
    MeasurementInfo {
        current_tsinfo,
        current_complete,
        last_tsinfo,
        last_complete,
        outside_germany,
    }
}

#[tokio::main]
pub async fn main() {
    let root = std::env::var("DEBS_DATA_ROOT").expect("DEBS_DATA_ROOT not set!");
    for i in 0..100000 {
        let batch = load_batch(&root, i).await.expect("Loading of batch failed");
        let analyzed = analyze_one(&batch);
        println!("Batch {}: {:?}", i, analyzed);
    }
}
