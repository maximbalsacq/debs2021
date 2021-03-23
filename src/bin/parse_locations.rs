#![feature(is_sorted)]
use debs2021::gen::challenger::*;
use bytes::Bytes;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use geo::prelude::*;

use prost::Message;

async fn load_locations() -> Locations {
    let mut f = File::open("/run/media/m/PUBLIC/Thesis/locations_dump.bin").await.expect("Failed to open file");
    let mut data = vec![];
    f.read_to_end(&mut data).await.expect("I/O read fail");
    let b = Bytes::from(data);
    Message::decode(b).expect("Failed to decode locations")
}

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

#[tokio::main]
pub async fn main() {
    let load_start = tokio::time::Instant::now();
    let locations = load_locations().await;
    let load_duration = load_start.elapsed();
    println!("Loading of locations took {}ms", load_duration.as_millis());

    let bbcalc_start = tokio::time::Instant::now();
    use std::iter::FromIterator;
    let multipolys = locations.locations.iter()
        .map(|location| {
            let multipoly = geo::MultiPolygon::from_iter(
                location
                .polygons
                .iter()
                .map(|poly| geo::Polygon::new(
                    geo::LineString::from_iter(
                        poly.points
                        .iter()
                        .map(|point| (point.latitude, point.longitude))),
                    vec![]
                ))
            );
            ((), multipoly)
        })
        .collect::<Vec<_>>();
    let bboxes = multipolys
        .iter()
        .map(|(location, multipoly)| (location, multipoly.bounding_rect()))
        .collect::<Vec<_>>();
    dbg!(&bboxes[0]);
    let bbcalc_duration = bbcalc_start.elapsed();
    println!("Generation of {} bboxes took {}ms", bboxes.len(), bbcalc_duration.as_millis());

    let batch = load_batch(0).await.expect("Loading of batch failed");
    let timestamps = batch.current
        .iter()
        .filter_map(|m| Some(m.timestamp.as_ref()?.seconds as i128 * 1_000_000_000 + m.timestamp.as_ref()?.nanos as i128))
        .collect::<Vec<_>>();
    println!("Is batch sorted by time? {}", timestamps.is_sorted());
    let measurement = batch.current.iter().next().expect("Missing measurement");

    let betterfind_start = tokio::time::Instant::now();
    let candidatecount = bboxes
        .iter()
        // .filter_map(|&(location, opt_bb)| opt_bb.map(|bb| (location, bb)))
        .filter(|&(_location, bb)| bb.expect("At least one poly").contains(&geo::Coordinate::from((measurement.latitude.into(), measurement.longitude.into()))))
        .count();
    let betterfind_duration = betterfind_start.elapsed();
    println!("Found {} candidates in {}ns", candidatecount, betterfind_duration.as_nanos());
}
