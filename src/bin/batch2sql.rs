use debs2021::io::load_batch;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt,BufWriter};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::var("DEBS_DATA_ROOT").expect("DEBS_DATA_ROOT not set!");
    let batchnum = 0;
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
