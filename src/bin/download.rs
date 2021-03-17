use debs2021::gen::challenger::*;
use debs2021::gen::challenger::challenger_client::*;
use debs2021::gen::challenger::benchmark_configuration::*;

use tonic::transport::Channel;
use bytes::BytesMut;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://challenge.msrg.in.tum.de:5023")
        .connect()
        .await?;
    let mut client = ChallengerClient::new(channel);
    let benchmark_conf = BenchmarkConfiguration {
        token: "elyadzboygfstsicxvyenztgyludxcve".to_string(),
        batch_size: 10000,
        benchmark_name: "connection test".to_string(),
        benchmark_type: "test".to_string(),
        queries: vec![Query::Q1 as i32, Query::Q2 as i32],
        ..Default::default()
    };
    let benchmark_id = client.create_new_benchmark(benchmark_conf).await?.into_inner();
    // let locations = client.get_locations(benchmark_id).await?.into_inner();
    // let mut buf = BytesMut::with_capacity(locations.encoded_len());
    // locations.encode(&mut buf)?;
    // let mut file = BufWriter::new(File::create("/run/media/maxim/PUBLIC/locations_dump.bin").await?);
    // file.write_all(&buf).await?;
    // file.flush().await?;
    // drop(file);
    use prost::Message;
    use tokio::fs::File;
    use tokio::io::{AsyncWriteExt,BufWriter};

    client.start_benchmark(benchmark_id.clone()).await?;
    for i in 0..100000 {
        let dir = i/1000;
        let batch = client.next_batch(benchmark_id.clone()).await?.into_inner();
        let mut buf = BytesMut::with_capacity(batch.encoded_len());
        batch.encode(&mut buf).unwrap();
        let mut file = BufWriter::new(File::create(format!("/run/media/maxim/PUBLIC/Thesis/messages/{}/batch_{}.bin", dir, i)).await?);
        file.write_all(&buf).await?;
        file.flush().await?;
        drop(file);
    }
    client.end_benchmark(benchmark_id).await?;

    Ok(())
}
