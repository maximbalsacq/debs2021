pub mod gen;
pub mod io;

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
                                .map(|point| (point.latitude, point.longitude))),
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
