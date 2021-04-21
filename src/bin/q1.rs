use debs2021::io::load_locations;
use debs2021::pipeline::run_pipeline;
use debs2021::AnalysisLocations;

use debs2021::io::LoadError;
use std::fs::File;
use debs2021::gen::challenger::Batch;
use std::io::Read;
use bytes::Bytes;
use prost::Message;
fn load_batch(root: &str, num: usize) -> Result<Batch, LoadError> {
    let dir = num/1000;
    let filename = format!("{}/messages/{}/batch_{}.bin", root, dir, num);
    load_batch_from(&filename)
}

fn load_batch_from(filename: &str) -> Result<Batch, LoadError> {
    let mut f = File::open(filename)
        .map_err(|source| LoadError::FileOpenError { filename: filename.to_owned(), source })?;
    let mut data = vec![];
    f.read_to_end(&mut data)
        .map_err(|source| LoadError::FileReadError { filename: filename.to_owned(), source })?;
    let b = Bytes::from(data);
    Batch::decode(b).map_err(|source| LoadError::FileDecodeError { filename: filename.to_owned(), source })
}

use std::sync::mpsc::*;
use std::thread;
fn start_batch_loader_thread(root: &str, batches: std::ops::Range<usize>) -> Receiver<Result<Batch, LoadError>> {
    let (sender, receiver) = sync_channel(20); // load up to X batches in advance
    let root = root.to_owned();
    thread::spawn(move || {
        for i in batches {
           sender.send(load_batch(&root, i)).unwrap();
        }
    });
    receiver
}

#[tokio::main]
pub async fn main() {
    let root = std::env::var("DEBS_DATA_ROOT").expect("DEBS_DATA_ROOT not set!");
    /*
    let batch_iter = (0..5000)
        .map(|i| { load_batch(&root, i).expect("Loading of batch failed") });
    */
    let batch_iter = start_batch_loader_thread(&root, 0..5000)
        .into_iter()
        .map(|b| b.expect("Loading of batch failed"));

    let locations = load_locations(&root)
        .await
        .expect("Failed to load locations");
    let locations = AnalysisLocations::new(locations);
    run_pipeline(locations, batch_iter);
}
