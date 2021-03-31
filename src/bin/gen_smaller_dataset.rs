use debs2021::gen::challenger::*;
use debs2021::io::load_batch;
use prost::Message;

use tokio::fs::File;
use tokio::io::{AsyncWriteExt,BufWriter};
use bytes::BytesMut;

async fn save_batch(root : &str, batch: &Batch) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = BytesMut::with_capacity(batch.encoded_len());
    batch.encode(&mut buf).unwrap();
    let mut file = BufWriter::new(File::create(format!("{}/test_batch.bin", root)).await?);
    file.write_all(&buf).await?;
    file.flush().await?;
    Ok(())
}

fn area_filter(m: &Measurement) -> bool {
    m.latitude < (47.40724 + 54.9079) / 4.0
        && m.longitude < (5.98815 + 14.98853) / 4.0
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::var("DEBS_DATA_ROOT").expect("DEBS_DATA_ROOT not set!");
    let mut new_batch = load_batch(&root, 0).await.expect("Loading of batch failed");
    new_batch.current.clear();
    new_batch.lastyear.clear();
    for i in 0..2000 {
        let batch = load_batch(&root, i).await.expect("Loading of batch failed");
        let mut additional_current : Vec<Measurement> = batch.current
            .into_iter()
            .filter(area_filter)
            .collect();
        let mut additional_lastyear : Vec<Measurement> = batch.lastyear
            .into_iter()
            .filter(area_filter)
            .collect();
        new_batch.current.append(&mut additional_current);
        new_batch.lastyear.append(&mut additional_lastyear);
    }
    use chrono::NaiveDateTime;
    use std::convert::TryInto;
    let first_ts = new_batch.current.first().unwrap().timestamp.clone().unwrap();
    let first_time = NaiveDateTime::from_timestamp(first_ts.seconds, first_ts.nanos.try_into().unwrap());
    let last_ts = new_batch.current.last().unwrap().timestamp.clone().unwrap();
    let last_time = NaiveDateTime::from_timestamp(last_ts.seconds, last_ts.nanos.try_into().unwrap());
    println!("Saved {}/{} samples in range from {} to {} ({})",
        new_batch.current.len(), new_batch.lastyear.len(),
        first_time, last_time, (last_time - first_time)
        );
    save_batch(&root, &new_batch).await?;
    Ok(())
}
