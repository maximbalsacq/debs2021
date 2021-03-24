use debs2021::gen::challenger::*;
use bytes::Bytes;
use prost::Message;

use tokio::fs::File;
use tokio::io::AsyncReadExt;

async fn load_batch(num: usize) -> Result<Batch, &'static str> {
    let dir = num/1000;
    let mut f = File::open(format!("/run/media/m/PUBLIC/Thesis/messages/{}/batch_{}.bin", dir, num))
        .await
        .map_err(|_| "Failed to open file")?;
    let mut data = vec![];
    f.read_to_end(&mut data)
        .await
        .map_err(|_| "I/O read fail")?;
    let b = Bytes::from(data);
    Ok(Batch::decode(b).map_err(|_| "Failed to decode batch")?)
}

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
    last_complete: bool
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
    MeasurementInfo {
        current_tsinfo,
        current_complete,
        last_tsinfo,
        last_complete,
    }
}

#[tokio::main]
pub async fn main() {
    for i in 0..100000 {
        let batch = load_batch(i).await.expect("Loading of batch failed");
        let analyzed = analyze_one(&batch);
        println!("Batch {}: {:?}", i, analyzed);
    }
}
