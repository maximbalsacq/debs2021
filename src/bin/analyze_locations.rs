#![feature(slice_partition_dedup)]
use debs2021::io::load_locations;

#[tokio::main]
pub async fn main() {
    let root = std::env::var("DEBS_DATA_ROOT").expect("DEBS_DATA_ROOT not set!");
    let load_start = tokio::time::Instant::now();
    let locations = load_locations(&root)
        .await
        .expect("Failed to load locations")
        .locations;
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

    let location_count = locations.len();
    let poly_count : usize = locations.iter().map(|l| l.polygons.len()).sum();
    let points_count : usize = locations
        .iter()
        .map(|l| l.polygons.iter()
            .map(|p| p.points.len())
            .sum::<usize>())
        .sum();
    println!(
        "{} polygons for {} locations ({} per location)",
        poly_count,
        location_count,
        poly_count as f64 / location_count as f64
    );

    println!(
        "{} points for {} polygons ({} per polgon/{} per location)",
        points_count,
        poly_count,
        points_count as f64 / poly_count as f64,
        points_count as f64 / location_count as f64
    );


    use std::iter::FromIterator;
    let convex_poly_count : usize = locations.iter()
        .map(|l| l.polygons.iter())
        .flatten()
        .map(|poly| {
            let p = geo::Polygon::new(
                // exterior ring
                geo::LineString::from_iter(
                    poly.points
                    .iter()
                    .map(|point| (point.longitude, point.latitude))),
                    // interior ring (unused)
                    vec![]
            );
            // the specified alternative does not exist
            #[allow(deprecated)]
            usize::from(p.is_convex())
        })
        .sum();
    
    println!("Convex: {}/{} ({:.03}%)",
        convex_poly_count,
        poly_count,
        (convex_poly_count as f64 / poly_count as f64) * 100.0
    );
}
