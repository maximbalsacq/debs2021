#![feature(is_sorted)]
use debs2021::io::*;
use geo::prelude::*;

#[tokio::main]
pub async fn main() {
    let root = std::env::var("DEBS_DATA_ROOT").expect("DEBS_DATA_ROOT not set!");
    let load_start = tokio::time::Instant::now();
    let locations = load_locations(&root).await.expect("Loading of locations failed");
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

    let batch = load_batch(&root, 0).await.expect("Loading of batch failed");
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
