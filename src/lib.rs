pub mod gen;
pub mod io;
pub mod aqi;
pub mod aggregate;
pub mod spliter;

use crate::gen::challenger::Locations;
use geo::{MultiPolygon,Rect,point,prelude::{Contains,BoundingRect}};

#[derive(Debug, PartialEq, Eq)]
pub struct AnalysisLocation {
    zipcode: String,
    city: String,
}

impl From<crate::gen::challenger::Location> for AnalysisLocation {
    fn from(loc: crate::gen::challenger::Location) -> Self {
        Self {
            zipcode: loc.zipcode,
            city: loc.city,
        }
    }
}

#[derive(Debug)]
pub struct AnalysisLocations {
    locations: Vec<(AnalysisLocation, Rect<f64>, MultiPolygon<f64>)>
}

impl AnalysisLocations {
    pub fn new(locations: Locations) -> Self {
        use std::iter::FromIterator;
        let locations = locations.locations.into_iter()
            .map(|location| {
                let multipoly = geo::MultiPolygon::from_iter(
                    location
                        .polygons
                        .iter()
                        .map(|poly| geo::Polygon::new(
                            // exterior ring
                            geo::LineString::from_iter(
                                poly.points
                                .iter()
                                .map(|point| (point.longitude, point.latitude))),
                            // interior ring (unused)
                            vec![]
                        ))
                );
                let boundingrect = multipoly.bounding_rect().expect("Location should contain >= 1 polygon");
                (location.into(), boundingrect, multipoly)
            })
            .collect();
        Self {
            locations
        }
    }

    pub fn localize(&self, latitude: f32, longitude: f32) -> impl Iterator<Item=&AnalysisLocation> {
        self.locations
            .iter()
            .filter(move |(_, bounding_rect, bounding_poly)| {
                let p = point!(x: f64::from(longitude), y: f64::from(latitude));
                bounding_rect.contains(&p) && bounding_poly.contains(&p)
            })
            .map(|(location, _, _)| location)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn check_locating_works() {
        use super::AnalysisLocations;
        use super::io::load_locations;
        let root = std::env::var("DEBS_DATA_ROOT").expect("DEBS_DATA_ROOT not set!");
        let locations = AnalysisLocations::new(load_locations(&root)
            .await
            .expect("Failed to load locations"));
        let mut outside_germany_iter = locations.localize(46.0, 20.0);
        assert_eq!(outside_germany_iter.next(), None);
        let mut freiburg_iter = locations.localize(47.99422, 7.849722);
        let freiburg = freiburg_iter.next();
        assert!(freiburg.is_some());
        assert_eq!(&freiburg.unwrap().city, "Freiburg im Breisgau");
        assert_eq!(freiburg_iter.next(), None);
    }

    #[tokio::test]
    async fn check_locating_samples_works() {
        use super::AnalysisLocations;
        use super::io::{load_locations,load_batch_from};
        let root = std::env::var("DEBS_DATA_ROOT").expect("DEBS_DATA_ROOT not set!");
        let locations = AnalysisLocations::new(load_locations(&root)
            .await
            .expect("Failed to load locations"));
        let batch = load_batch_from(&format!("{}/test_batch.bin", root))
            .await
            .expect("Failed to load batch");
        for measurement in batch.lastyear.iter().chain(batch.current.iter()) {
            if measurement.latitude < 47.40724 || measurement.latitude > 54.9079 || measurement.longitude < 5.98815 || measurement.longitude > 14.98853 {
                // skip measurements outside germany
                continue;
            }
            let mut loc = locations.localize(measurement.latitude, measurement.longitude);
            let matching_loc = loc.next();
            matching_loc.expect(&format!("Did not find a matching location for latitude {}, longitude {}", measurement.latitude, measurement.longitude)); // Fail test if no location found
            assert_eq!(loc.next(), None);
        }
    }
}
