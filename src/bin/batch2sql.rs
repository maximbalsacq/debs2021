use debs2021::io::load_batch;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt,BufWriter};

async fn batch2sql(root: &str, batchnum: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = BufWriter::new(File::create(format!("{}/sql/{}/{}.sql", &root, batchnum / 1000, batchnum % 1000)).await?);
    let batch = load_batch(&root, batchnum).await?;
    out.write_all("BEGIN;\n".as_bytes()).await?;
    for m in &batch.current {
        let s = format!(
            "INSERT INTO meas_current VALUES ({}, to_timestamp({}) AT TIME ZONE 'GMT', ST_Point({}, {}), {}, {});\n",
            batchnum,
            m.timestamp.as_ref().cloned().unwrap().seconds,
            m.latitude,
            m.longitude,
            m.p1,
            m.p2,
        );
        out.write_all(s.as_bytes()).await?;
    }
    for m in &batch.lastyear {
        let s = format!(
            "INSERT INTO meas_lastyear VALUES ({}, to_timestamp({}) AT TIME ZONE 'GMT', ST_Point({}, {}), {}, {});\n",
            batchnum,
            m.timestamp.as_ref().cloned().unwrap().seconds,
            m.latitude,
            m.longitude,
            m.p1,
            m.p2,
        );
        out.write_all(s.as_bytes()).await?;
    }
    out.write_all("COMMIT;\n".as_bytes()).await?;
    out.flush().await?;
    Ok(())
}

async fn guarded_batch2sql(root: &str, batchnum: usize, file_limiting_sem: std::sync::Arc<tokio::sync::Semaphore>) -> Result<(), Box<dyn std::error::Error>> {
    let lock = file_limiting_sem.acquire().await.expect("Failed to acquire file semaphore");
    let res = batch2sql(&root, batchnum).await;
    drop(lock);
    res
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::Arc;
    let root = std::env::var("DEBS_DATA_ROOT").expect("DEBS_DATA_ROOT not set!");
    let file_limiting_sem = Arc::new(tokio::sync::Semaphore::new(400));
    let futures = (0..10000)
        .map(|batchnum| guarded_batch2sql(&root, batchnum, Arc::clone(&file_limiting_sem)))
        .collect::<Vec<_>>();
    use futures::future::join_all;
    join_all(futures).await.into_iter().collect()
}
