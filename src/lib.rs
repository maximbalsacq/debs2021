#![feature(or_patterns)]
pub mod gen;
pub mod io;
pub mod aqi;
pub mod aggregate;
pub mod spliter;
pub mod pipeline;

use crate::gen::challenger::Locations;
use geo::{MultiPolygon,point,prelude::{Contains,BoundingRect}};

type CityId = u32;

#[derive(Debug, PartialEq, Eq)]
pub struct AnalysisLocation {
    zipcode: String,
    cityid: CityId,
}


use std::sync::atomic::AtomicUsize;
#[derive(Debug)]
pub struct AnalysisLocations {
    bboxtree: RTree<RTreeLocation>,
    locations: Vec<(AnalysisLocation, MultiPolygon<f64>)>,
    insidecache: Vec<LocationCache>,
    outsidecache: Vec<LocationCache>,
    /// a map from CityId to city name, where CityId is simply an index
    /// into the array
    known_cities: Vec<String>,
    pub cachehits: AtomicUsize,
    pub cachemisses: AtomicUsize,
    pub outsidecachehits: AtomicUsize,
}

use rstar::{AABB,RTree,RTreeObject,Point,PointDistance};

#[derive(Debug)]
struct RTreeLocation {
    bbox: AABB<[f64; 2]>,   
    /// index into locations array
    idx: u32,
}

impl RTreeObject for RTreeLocation {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.bbox.clone()
    }
}

impl PointDistance for RTreeLocation {
    fn distance_2(
        &self,
        point: &[f64; 2],
    ) -> <[f64;2] as Point>::Scalar {
        self.bbox.distance_2(point)
    }
}

#[derive(Debug,Default)]
struct LocationCacheItem {
    x: f32,
    y: f32,
    rsquare: f32,
}

impl PartialEq for LocationCacheItem {
    fn eq(&self, other: &Self) -> bool {
        self.rsquare == other.rsquare && self.x == other.x && self.y == other.y
    }
}
impl Eq for LocationCacheItem {}

impl PartialOrd for LocationCacheItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        match self.rsquare.partial_cmp(&other.rsquare) {
            o @ Some(Ordering::Less | Ordering::Greater) => o,
            Some(Ordering::Equal) => match self.x.partial_cmp(&other.x) {
                o @ Some(Ordering::Less | Ordering::Greater) => o,
                Some(Ordering::Equal) => self.y.partial_cmp(&other.y),
                None => None
            }
            None => None,
        }
    }
}

impl Ord for LocationCacheItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(&other).unwrap()
    }
}

impl RTreeObject for LocationCacheItem {
    type Envelope = AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.x, self.y])
    }
}

impl PointDistance for LocationCacheItem {
    fn distance_2(
        &self,
        point: &[f32; 2],
    ) -> <[f32;2] as Point>::Scalar {
        self.envelope().distance_2(point)
    }
}

use std::sync::RwLock;

const CACHE_SIZE: usize = 32;

use arrayvec::ArrayVec;
use rstar::primitives::Line;
#[derive(Debug,Default)]
struct LocationCache {
    lines: RTree<Line<[f32; 2]>>,
    // items: RwLock<RTree<LocationCacheItem, LocationRTreeParams>>,
    items: RwLock<ArrayVec<LocationCacheItem, CACHE_SIZE>>,
}

impl LocationCache {
    fn new(locations: &crate::gen::challenger::Location) -> Self {
        let points = locations.polygons
            .iter()
            .flat_map(|poly| poly.points.iter())
            .map(|point| [point.longitude as f32, point.latitude as f32])
            .collect::<Vec<_>>();
        let points_len = points.len();
        assert!(points_len > 0);
        let lines = (0..points_len)
            .map(|idx| {
                let first_point = points[idx];
                let second_point = if idx != points_len - 1 {
                    points[idx+1]
                } else {
                    points[0] // last point connects to first
                };
                rstar::primitives::Line::new(first_point, second_point)
            })
            .collect::<Vec<_>>();
        let lines = RTree::bulk_load(lines);
        LocationCache {
            lines,
            // items: RwLock::new(RTree::new_with_params()),
            // items: RwLock::new(Vec::with_capacity(<LocationRTreeParams as rstar::RTreeParams>::MAX_SIZE)),
            items: RwLock::new(ArrayVec::new()),
        }
    }

    fn contains(&self, latitude: f32, longitude: f32) -> bool {
        self.items
            .read()
            .unwrap()
            .iter()
            .map(|item| (item, [item.x, item.y].distance_2(&[longitude, latitude])))
            .any(|(item, distance)| distance < item.rsquare)
    }

    fn add(&self, latitude: f32, longitude: f32) {
        if self.items.read().unwrap().len() >= CACHE_SIZE {
            return;
        }

        let rsquare = self.lines.nearest_neighbor_iter_with_distance_2(&[longitude, latitude])
            .map(|(_line, distance)| distance)
            .next()
            .expect("Should have at least two points");
        if rsquare < 0.00001 {
            // don't keep items very close to the border
            return;
        }
        
        let mut items = self.items.write().unwrap();
        if items.len() >= CACHE_SIZE {
            // can happen since read lock was dropped previously
            // just abort
            return;
        }
        items.push(LocationCacheItem {
            x: longitude,
            y: latitude,
            rsquare,
        });
        items.sort();
    }
}

impl AnalysisLocations {
    pub fn new(locations: Locations) -> Self {
        use std::iter::FromIterator;
        use std::collections::BTreeMap;
        let mut known_cities_map = BTreeMap::new();
        let mut known_cities = vec![];
        let mut next_id = 0u32;
        let cache : Vec<LocationCache> = locations.locations.iter()
            .map(|location| LocationCache::new(location))
            .collect();
        let outsidecache : Vec<LocationCache> = locations.locations.iter()
            .map(|location| LocationCache::new(location))
            .collect();
        let locations : Vec<(AnalysisLocation, MultiPolygon<f64>)> = locations.locations.into_iter()
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
                let cityid = match known_cities_map.get(&location.city) {
                    Some(id) => *id,
                    None => {
                        known_cities_map.insert(location.city.clone(), next_id);
                        known_cities.push(location.city);
                        next_id += 1;
                        next_id - 1
                    }
                };
                let location = AnalysisLocation {
                    zipcode: location.zipcode,
                    cityid,
                };
                (location, multipoly)
            })
            .collect();
        let bboxitems = locations
            .iter()
            .enumerate()
            .map(|(idx, (_, multipoly))| {
                let boundingrect = multipoly.bounding_rect().expect("Location should contain >= 1 polygon");
                let (minx, miny) = boundingrect.min().x_y();
                let (maxx, maxy) = boundingrect.max().x_y();
                let bbox = AABB::from_corners([minx, miny], [maxx, maxy]);
                RTreeLocation {
                    bbox,
                    idx: idx as u32,
                }
            })
            .collect::<Vec<_>>();
        let bboxtree = RTree::bulk_load(bboxitems);
        Self {
            bboxtree,
            locations,
            known_cities,
            insidecache: cache,
            outsidecache: outsidecache,
            cachehits: AtomicUsize::from(0),
            cachemisses: AtomicUsize::from(0),
            outsidecachehits: AtomicUsize::from(0),
        }
    }

    pub fn localize(&self, latitude: f32, longitude: f32) -> impl Iterator<Item=&AnalysisLocation> {
        self.bboxtree
            .locate_all_at_point(&[longitude as f64, latitude as f64])
            .filter_map(move |treeobject| {
                /* 
                // same code w/o cache. uncomment if you want to compare
                let (location, bounding_poly) = &self.locations[treeobject.idx as usize];
                let p = point!(x: f64::from(longitude), y: f64::from(latitude));
                if bounding_poly.contains(&p) {
                    Some(location)
                } else {
                    None
                }
                */
                if self.outsidecache[treeobject.idx as usize].contains(latitude, longitude) {
                    debug_assert!({
                        let (_, bounding_poly) = &self.locations[treeobject.idx as usize];
                        let p = point!(x: f64::from(longitude), y: f64::from(latitude));
                        !bounding_poly.contains(&p)
                    });
                    self.outsidecachehits.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
                    None
                } else if self.insidecache[treeobject.idx as usize].contains(latitude, longitude) {
                    debug_assert!({
                        let (_, bounding_poly) = &self.locations[treeobject.idx as usize];
                        let p = point!(x: f64::from(longitude), y: f64::from(latitude));
                        bounding_poly.contains(&p)
                    });
                    self.cachehits.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
                    Some(&self.locations[treeobject.idx as usize].0)
                } else {
                    let (location, bounding_poly) = &self.locations[treeobject.idx as usize];
                    let p = point!(x: f64::from(longitude), y: f64::from(latitude));
                    self.cachemisses.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
                    if bounding_poly.contains(&p) {
                        self.insidecache[treeobject.idx as usize].add(latitude, longitude);
                        Some(location)
                    } else {
                        self.outsidecache[treeobject.idx as usize].add(latitude, longitude);
                        None
                    }
                }
            })
    }

    pub fn lookup(&self, cityid: CityId) -> &str {
        &self.known_cities[cityid as usize]
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
        assert_eq!(locations.lookup(freiburg.unwrap().cityid), "Freiburg im Breisgau");
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
