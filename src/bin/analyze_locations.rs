#![feature(slice_partition_dedup)]
use debs2021::gen::challenger::*;
use bytes::Bytes;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use prost::Message;

async fn load_locations() -> Locations {
    let mut f = File::open("/run/media/m/PUBLIC/Thesis/locations_dump.bin").await.expect("Failed to open file");
    let mut data = vec![];
    f.read_to_end(&mut data).await.expect("I/O read fail");
    let b = Bytes::from(data);
    Message::decode(b).expect("Failed to decode locations")
}

#[tokio::main]
pub async fn main() {
    let load_start = tokio::time::Instant::now();
    let locations = load_locations().await.locations;
    let load_duration = load_start.elapsed();
    println!("Loading of locations took {}ms", load_duration.as_millis());

    let zipcode_dups = {
        let mut locations = locations.clone();
        locations.sort_unstable_by_key(|location| location.zipcode.clone());
        let (_, dups) = locations.partition_dedup_by_key(|location| location.zipcode.clone());
        dups.to_vec()
    };

    if zipcode_dups.len() > 0 {
        println!("Zipcode dups: {}, ex:", zipcode_dups.len());
        let location = &zipcode_dups[0];
        println!(r#"Zip: "{}", city: "{}", qkm: {}, population: {}"#, &location.zipcode, &location.city, location.qkm, location.population);
    } else {
        println!("Zip codes are unique");
    }

    if let Some(no_polys) = locations.iter().find(|loc| loc.polygons.is_empty()) {
        println!("At least one location has no polygons: {:?}", no_polys);
    }
}
