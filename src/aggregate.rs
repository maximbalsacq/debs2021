use crate::CityId;

/// Contains the information of a measurement,
/// but with city id and batch seq id instead
/// of coordinates.
#[derive(Debug, Copy, Clone)]
pub struct LocalizedMeasurement {
    /// The sequence number of the source batch
    pub batch_seq_id: i64,
    /// The city id which can be used to look up
    /// the city name using [AnalysisLocations::lookup](crate::AnalysisLocations::lookup)
    pub cityid: CityId,
    /// The timestamp of the measurement.
    /// Contrary to the generated source,
    /// 1. it is not an Option since
    ///    the timestamp is always present
    /// 2. only the seconds are used
    pub timestamp_seconds: i64,
    ///Particles < 10µm (particulate matter)
    pub p1: f32,
    ///Particles < 2.5µm (ultrafine particles)
    pub p2: f32,
}

/// An Iterator which partitions [LocalizedMeasurement]s by their timestamp.
/// The order of the [LocalizedMeasurement]s is preserved
/// and verified to be monotonically increasing.
pub struct TimePartitionIterator<T, F>
where
    T: Iterator<Item = LocalizedMeasurement>,
    F: Fn(Option<i64>, &LocalizedMeasurement) -> i64,
{
    /// The timestamp at which the current partition ends.
    currrent_partition_end_seconds: i64,
    /// The values for the current partition
    values: Vec<LocalizedMeasurement>,
    /// The iterator, wrapped in Peekable to make it possible
    /// temporarily store the next value
    iter: std::iter::Peekable<T>,
    /// A function which returns the next timestamp based on the
    /// current partition end and the next measurement not
    /// fitting into the current window.
    next_timestamp_fn: F,
}

impl<T, F> TimePartitionIterator<T, F>
where
    T: Iterator<Item = LocalizedMeasurement>,
    F: Fn(Option<i64>, &LocalizedMeasurement) -> i64,
{
    /// Creates a new TimePartitionIterator using the iterator iter
    /// and a function to determine the window of timestamps.
    /// The return value of the function determining the timestamps
    /// MUST be monotonically increasing.
    pub fn new(iter: T, next_timestamp_fn: F) -> Self {
        let mut iter = iter.peekable();
        let currrent_partition_end_seconds = match iter.peek() {
            Some(m) => (next_timestamp_fn)(None, m),
            None => 0, // doesn't matter, don't have any measurements anyway
        };
        Self {
            iter,
            values: vec![],
            currrent_partition_end_seconds,
            next_timestamp_fn,
        }
    }
}

impl<T, F> Iterator for TimePartitionIterator<T, F>
where
    T: Iterator<Item = LocalizedMeasurement>,
    F: Fn(Option<i64>, &LocalizedMeasurement) -> i64,
{
    type Item = Vec<LocalizedMeasurement>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.peek() {
                None if !self.values.is_empty() => {
                    // just reached the end of measurements,
                    // yield aggregated values
                    break Some(std::mem::take(&mut self.values));
                }
                None => {
                    // previously reached end of measurements and don't
                    // have any values left to return
                    break None;
                }
                Some(m) if m.timestamp_seconds < self.currrent_partition_end_seconds => {
                    // measurement fits into current time window
                    debug_assert!(self
                        .values
                        .last()
                        .map(|last| last.timestamp_seconds <= m.timestamp_seconds)
                        .unwrap_or(true));
                    self.values.push(self.iter.next().unwrap());
                    // no break here, continue until either a measurement does not fit
                    // into the current window or we run out of values
                }
                Some(m) => {
                    // measurement does not fit into current window
                    debug_assert!(self
                        .values
                        .last()
                        .map(|last| last.timestamp_seconds <= m.timestamp_seconds)
                        .unwrap_or(true));
                    
                    let next_partition_end_seconds =
                        (self.next_timestamp_fn)(Some(self.currrent_partition_end_seconds), m);

                    assert!(self.currrent_partition_end_seconds <= next_partition_end_seconds);
                    self.currrent_partition_end_seconds = next_partition_end_seconds;

                    if m.timestamp_seconds < self.currrent_partition_end_seconds {
                        // fits into new window
                        break Some(std::mem::replace(
                            &mut self.values,
                            vec![self.iter.next().unwrap()],
                        ));
                    } else {
                        // does not fit into ether the current or the next window, which means the next window is empty.
                        // the current window needs to emitted, then the next empty window becomes the current window.
                        // On the next next() call this match branch will re-run and try to fit the measurement
                        // into the next window
                        break Some(std::mem::replace(&mut self.values, vec![]));
                    }
                }
            }
        }
    }
}

/// A naive timestamp window generation function,
/// which starts with the current non_fitting_measurement's
/// timestamp and simply continues to add 300 seconds.
/// It does not take account things like leap seconds.
fn partition_5min(
    current_window_end: Option<i64>,
    non_fitting_measurement: &LocalizedMeasurement,
) -> i64 {
    current_window_end
        .map(|x| x + 300)
        .unwrap_or_else(|| non_fitting_measurement.timestamp_seconds + 300)
}

/// A convenience trait which makes it possible to partition an iterator
/// of [LocalizedMeasurement]s into 5 minute batches.
pub trait Partition5Min: Iterator<Item = LocalizedMeasurement> {
    /// Partitions the Iterator of LocalizedMeasurements into batches of 5 minutes
    fn partition_5min(
        self,
    ) -> TimePartitionIterator<
        Self,
        for<'r> fn(std::option::Option<i64>, &'r LocalizedMeasurement) -> i64,
    >
    where
        Self: Sized,
    {
        TimePartitionIterator::new(self, partition_5min)
    }
}

// Implement this trait on all iterators outputting LocalizedMeasurements
impl<T: Sized> Partition5Min for T where T: Iterator<Item = LocalizedMeasurement> {}

use std::collections::VecDeque;
/// An iterator over all values belonging to a specific time period.
pub type AnalysisWindowIter<'a, T> = std::collections::vec_deque::Iter<'a, T>;

/// A window providing two iterators over the same period in the current and the last year.
pub struct AnalysisWindow<'lcur, 'llast, TCur: Sync, TLast: Sync> {
    /// The iterator over the values of the current year
    pub current: AnalysisWindowIter<'lcur, TCur>,
    /// The iterator over values of the the last year
    pub lastyear: AnalysisWindowIter<'llast, TLast>,
}

/// A struct which allows joining two iterators (one for the current, one for last year)
/// and mapping each AnalysisWindow (optionally with a cache) generated by these two iterators
/// to an outupt value (and optionally a new cache value)
pub struct AnalysisWindowsMap<ICur, TCur, ILast, TLast, F, TOut, TCache=()>
where
    ICur: Iterator<Item = TCur>,
    ILast: Iterator<Item = TLast>,
    F: for<'a> Fn(AnalysisWindow<'a, 'a, TCur, TLast>, Option<TCache>) -> (TOut, TCache),
    TCur: Sync,
    TLast: Sync,
{
    /// The iterator yielding values for the current year.
    current_iter: ICur,
    /// A temporary store for values in the current AnalysisWindow
    /// (e.g. all values in the last 5 days)
    /// to make iteration possible
    current_queue: VecDeque<TCur>,

    /// The iterator yielding values for the last year.
    lastyear_iter: ILast,
    /// A temporary store for values in the last AnalysisWindow
    /// (e.g. all values in the last 5 days 1 year ago)
    /// to make iteration possible
    lastyear_queue: VecDeque<TLast>,

    /// A cache value which may be used by the mapping function.
    /// The mapping function must always return a value for the cache
    /// (which, if not necessary, can simply be an empty tuple).
    cache: Option<TCache>,

    /// The mapping function which maps each AnalysisWindow to an owned value.
    map_func: F,
    /// Used to detect when either iterator is exhausted,
    /// to avoid calling the mapping function with incorrectly sized values.
    completed: bool,
}

impl<ICur, TCur, ILast, TLast, F, TOut, TCache> AnalysisWindowsMap<ICur, TCur, ILast, TLast, F, TOut, TCache>
where
    ICur: Iterator<Item = TCur>,
    ILast: Iterator<Item = TLast>,
    TOut: 'static,
    F: for<'a> Fn(AnalysisWindow<'a, 'a, TCur, TLast>, Option<TCache>) -> (TOut, TCache),
    TCur: Sync,
    TLast: Sync,
{

    /// Creates a new AnalysisWindowsMap instance using the iterators current_iter and lastyear_iter.
    /// Each [AnalysisWindow] will be mapped to a TOut value by the mapping function map_func.
    /// It will receive windows with a size of current_window_size and lastyear_window_size until
    /// either iterator is exhausted. If the window sizes differ, `(current_window_size - lastyear_window_size).abs()`
    /// initial windows will be skipped on the iterator with a smaller size.
    pub fn new(mut current_iter: ICur, mut lastyear_iter: ILast, current_window_size: usize, lastyear_window_size: usize, map_func: F) -> Self {
        let mut current_queue = VecDeque::with_capacity(current_window_size+1);
        let mut lastyear_queue = VecDeque::with_capacity(lastyear_window_size+1);

        for idx in 0..usize::max(current_window_size, lastyear_window_size) {
            current_queue.push_back(match current_iter.next() {
                Some(x) => x,
                None => break,
            });
            if idx >= current_window_size {
                current_queue.pop_front();
            }

            lastyear_queue.push_back(match lastyear_iter.next() {
                Some(x) => x,
                None => break,
            });

            if idx >= lastyear_window_size {
                lastyear_queue.pop_front();
            }
        }
        let completed = {
            current_queue.len() != current_window_size 
            || lastyear_queue.len() != lastyear_window_size
        };
        // don't need *_window_size anymore, as window shifs 1 at a time

        Self {
            current_iter,
            current_queue,
            lastyear_iter,
            lastyear_queue,
            map_func,
            completed,
            cache: None,
        }
    }
}


impl<ICur, TCur, ILast, TLast, F, TOut, TCache> Iterator for AnalysisWindowsMap<ICur, TCur, ILast, TLast, F, TOut, TCache>
where
    ICur: Iterator<Item = TCur>,
    ILast: Iterator<Item = TLast>,
    TOut: 'static,
    F: for<'a> Fn(AnalysisWindow<'a, 'a, TCur, TLast>, Option<TCache>) -> (TOut, TCache),
    TCur: Sync,
    TLast: Sync,
{
    type Item=TOut;

    fn next(&mut self) -> Option<Self::Item> {
        if self.completed {
            return None;
        }

        let window = AnalysisWindow {
            current: self.current_queue.iter(),
            lastyear: self.lastyear_queue.iter(),
        };
        let (res, cache) = (self.map_func)(window, self.cache.take());
        self.cache = Some(cache); // update cache

        match (self.current_iter.next(), self.lastyear_iter.next()) {
            (Some(current), Some(lastyear)) => {
                self.current_queue.push_back(current);
                self.lastyear_queue.push_back(lastyear);
            },
            _ => {
                self.completed = true;
            }
        }
        self.current_queue.pop_front();
        self.lastyear_queue.pop_front();

        Some(res)
    }
}

use crate::spliter::IterPair;

impl<ICur, TCur, ILast, TLast> IterPair<ICur, ILast>
where
    ICur: Iterator<Item = TCur>,
    ILast: Iterator<Item = TLast>,
    TCur: Sync,
    TLast: Sync,
    {
    
    /// Convenience function to be able to call AnalysisWindowsMap::new on an iterator
    pub fn with_analysis_windows<F, TOut, TCache>(self, current_window_size: usize, lastyear_window_size: usize, map_func: F) -> AnalysisWindowsMap<ICur, TCur, ILast, TLast, F, TOut, TCache>
    where
    TOut: 'static,
    F: for<'a> Fn(AnalysisWindow<'a, 'a, TCur, TLast>, Option<TCache>) -> (TOut, TCache) {
        AnalysisWindowsMap::new(self.0, self.1, current_window_size, lastyear_window_size, map_func)
    }
}

#[cfg(test)]
mod window_test {
    use crate::spliter::Spliter;
    // use crate::aggregate::IterPair;
    use crate::aggregate::IterPair;
    #[test]
    fn join_test() {
        let x = IterPair(1..5, 2..6)
            .with_analysis_windows(2, 2, |window, _cache: Option<()>| {
                let current_total : i32 = window.current.sum();
                let lastyear_total : i32 = window.lastyear.sum();
                assert_eq!(current_total+2, lastyear_total);
                (current_total + lastyear_total, ())
            })
            .collect::<Vec<i32>>();
        assert_eq!(x, vec![
            1+2+2+3,
            2+3+3+4,
            3+4+4+5,
        ]);
    }

    #[test]
    fn split_join_test() {
        let x = (1..5)
            .map(|x| (x, x+1))
            .spliter()
            .with_analysis_windows(2, 2, |window, _cache: Option<()>| {
                let current_total : i32 = window.current.sum();
                let lastyear_total : i32 = window.lastyear.sum();
                assert_eq!(current_total+2, lastyear_total);
                (current_total + lastyear_total, ())
            })
            .collect::<Vec<i32>>();
        assert_eq!(x, vec![
            1+2+2+3,
            2+3+3+4,
            3+4+4+5,
        ]);
    }

    #[test]
    fn different_windowsizes_test() {
        let x = (1..5)
            .map(|x| (x, x+1))
            .spliter()
            .with_analysis_windows(1, 3, |window, _cache: Option<()>| {
                let current_total : i32 = window.current.sum();
                let lastyear_total : i32 = window.lastyear.sum();
                (current_total + lastyear_total, ())
            })
            .collect::<Vec<i32>>();
        assert_eq!(x, vec![
            3+(2+3+4),
            4+(3+4+5),
        ]);
    }
}


/// ParticleAggregate is used to aggregate multiple measurements
/// and calculate the mean p1/p2. It provides some additional
/// integrity checks, such as checking that input values are
/// positive and p1 and p2 are added at the same time.
/// To avoid rounding errors, values are internally stored as f64.
#[derive(Debug,Default,Copy,Clone)]
pub struct ParticleAggregate {
    /// The sum of all p1 values added to this aggregate
    sum_p1: f64,
    /// The sum of all p2 values added to this aggregate
    sum_p2: f64,
    /// The count of all p1 and p2 values added to this aggregate
    denom: usize,
}

impl ParticleAggregate {
    /// Creates a new aggregate based on a single measurement.
    pub fn new(init: (f32, f32)) -> Self {
        debug_assert!(init.0 >= 0.0 && init.1 >= 0.0);
        Self {
            sum_p1: init.0 as f64,
            sum_p2: init.1 as f64,
            denom: 1,
        }
    }

    /// Adds a single measurement to the aggregate.
    pub fn add(&mut self, val: (f32, f32)) {
        debug_assert!(val.0 >= 0.0 && val.1 >= 0.0);
        self.sum_p1 += val.0 as f64;
        self.sum_p2 += val.1 as f64;
        self.denom += 1;
    }

    /// Calculates the final (mean) p1 value of the aggregate.
    pub fn p1(&self) -> f32 {
        (self.sum_p1 / (self.denom as f64)) as f32
    }

    /// Calculates the final (mean) p2 value of the aggregate.
    pub fn p2(&self) -> f32 {
        (self.sum_p2 / (self.denom as f64)) as f32
    }
}

impl std::ops::Add for ParticleAggregate {
    type Output=Self;
    fn add(self, rhs: ParticleAggregate) -> Self {
        Self {
            sum_p1: self.sum_p1 + rhs.sum_p1,
            sum_p2: self.sum_p2 + rhs.sum_p2,
            denom: self.denom + rhs.denom,
        }
    }
}

impl std::ops::AddAssign for ParticleAggregate {
    fn add_assign(&mut self, rhs: ParticleAggregate) {
        *self = Self {
            sum_p1: self.sum_p1 + rhs.sum_p1,
            sum_p2: self.sum_p2 + rhs.sum_p2,
            denom: self.denom + rhs.denom,
        };
    }
}

impl std::ops::Sub for ParticleAggregate {
    type Output=Self;
    fn sub(self, rhs: ParticleAggregate) -> Self {
        debug_assert!(self.denom >= rhs.denom, "Removing {} units from aggregate with only {} added (self: {:#?}, rhs: {:#?})", rhs.denom, self.denom, self, rhs);
        let new_denom = self.denom - rhs.denom;
        if new_denom == 0 {
            // when no values are left, use all-zeroes instead
            // off subtracting to avoid/reduce rounding errors
            Self::default()   
        } else {
            debug_assert!((self.sum_p1 - rhs.sum_p1) >= -f64::EPSILON, "Removing {} p1 from aggregate with only {} p1 added (self: {:#?}, rhs: {:#?})", rhs.sum_p1, self.sum_p1, self, rhs);
            debug_assert!((self.sum_p2 - rhs.sum_p2) >= -f64::EPSILON, "Removing {} p2 from aggregate with only {} p2 added (self: {:#?}, rhs: {:#?})", rhs.sum_p2, self.sum_p2, self, rhs);
            Self {
                sum_p1: self.sum_p1 - rhs.sum_p1,
                sum_p2: self.sum_p2 - rhs.sum_p2,
                denom: new_denom,
            }
        }
    }
}

impl std::ops::SubAssign for ParticleAggregate {
    fn sub_assign(&mut self, rhs: ParticleAggregate) {
        debug_assert!(self.denom >= rhs.denom, "Removing {} units from aggregate with only {} added (self: {:#?}, rhs: {:#?})", rhs.denom, self.denom, self, rhs);
        *self = {
            let new_denom = self.denom - rhs.denom;
            if new_denom == 0 {
                // when no values are left, use all-zeroes instead
                // off subtracting to avoid/reduce rounding errors
                Self::default()   
            } else {
                debug_assert!((self.sum_p1 - rhs.sum_p1) >= -f64::EPSILON, "Removing {} p1 from aggregate with only {} p1 added (self: {:#?}, rhs: {:#?})", rhs.sum_p1, self.sum_p1, self, rhs);
                debug_assert!((self.sum_p2 - rhs.sum_p2) >= -f64::EPSILON, "Removing {} p2 from aggregate with only {} p2 added (self: {:#?}, rhs: {:#?})", rhs.sum_p2, self.sum_p2, self, rhs);
                Self {
                    sum_p1: self.sum_p1 - rhs.sum_p1,
                    sum_p2: self.sum_p2 - rhs.sum_p2,
                    denom: new_denom,
                }
            }
        }
    }
}

impl std::iter::FromIterator<ParticleAggregate> for ParticleAggregate {
    fn from_iter<T: IntoIterator<Item = ParticleAggregate>>(iter: T) -> Self {
        let (sum_p1, sum_p2, denom) = iter
            .into_iter()
            .fold((0.0, 0.0, 0), |acc, x| (acc.0 + x.sum_p1, acc.1 + x.sum_p2, acc.2 + x.denom));
        Self {
            sum_p1, sum_p2, denom
        }
    }
}

impl std::iter::FromIterator<(f32, f32)> for ParticleAggregate {
    fn from_iter<T: IntoIterator<Item = (f32, f32)>>(iter: T) -> Self {
        let (sum_p1, sum_p2, denom) = iter
            .into_iter()
            .fold((0.0, 0.0, 0), |acc, (p1, p2)| (acc.0 + p1 as f64, acc.1 + p2 as f64, acc.2 + 1));
        Self {
            sum_p1, sum_p2, denom
        }
    }
}

#[cfg(test)]
mod particle_aggregate_test {
    use super::ParticleAggregate;
    #[test]
    fn test_from_f32() {
        let agg : ParticleAggregate = vec![(0.1f32, 0.3f32), (0.2, 0.2)].into_iter().collect();
        assert_eq!(agg.p1(), 0.15); // (0.1 + 0.2) / 2
        assert_eq!(agg.p2(), 0.25); // (0.3 + 0.2) / 2
    }

    #[test]
    fn test_aggregate_multiple() {
        let agg1 : ParticleAggregate = vec![(0.1f32, 0.3f32), (0.2, 0.2)].into_iter().collect();
        let agg2 : ParticleAggregate = vec![(0.1f32, 0.3), (0.2, 1.6), (0.3, 0.2)].into_iter().collect();
        assert_eq!(agg1.p1(), 0.15); // (0.1 + 0.2) / 2
        assert_eq!(agg2.p1(), 0.2); // (0.1 + 0.2 + 0.3) / 3
        assert_eq!(agg1.p2(), 0.25); // (0.3 + 0.2) / 2
        assert!((agg2.p2() - 0.7).abs() < 0.0000001); // (0.3 + 1.6 + 0.2) / 3

        let agg : ParticleAggregate = vec![agg1, agg2].into_iter().collect();
        // (0.1 + 0.2 + 0.3 + 0.1 + 0.2) / (2 + 3)
        // = 0.9 / 5
        assert_eq!(agg.p1(), 0.18);
        // (0.3 + 0.2 + 0.3 + 1.6 + 0.2) / (2 + 3)
        // = 2.6 / 5
        assert!((agg.p2() - 0.52).abs() < 0.0000001);
    }
}

use std::collections::HashSet;

/// ActiveCities is used to keep track of which cities
/// has been active in the last 10 minutes.
#[derive(Debug,Default,Clone)]
pub struct ActiveCities {
    inner: HashSet<CityId>,
}

impl ActiveCities {
    /// Returns true if the city identified by the id
    /// is known to have been active in the last 10 minutes.
    pub fn is_active(&self, city: CityId) -> bool {
        self.inner.contains(&city)
    }
}

impl std::iter::FromIterator<CityId> for ActiveCities {
    fn from_iter<T: IntoIterator<Item = CityId>>(iter: T) -> Self {
        Self {
            inner: std::iter::FromIterator::from_iter(iter)
        }
    }
}

impl std::ops::BitOr for ActiveCities {
    type Output=ActiveCities;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        self.inner = &self.inner | &rhs.inner;
        self
    }
}

/// A map, mapping each CityId to the corresponding ParticleAggregate
pub type CityParticleMap = std::collections::HashMap<CityId, ParticleAggregate>;
// type CityParticleMap = std::collections::BTreeMap<CityId, ParticleAggregate>;

/// Calculates a\[k\] += b\[k]\ for every key k in b
pub(crate) fn map_add(a: &mut CityParticleMap, b: &CityParticleMap) {
    for (k, v2) in b {
        match a.get_mut(&k) {
            Some(v) => *v += *v2,
            None => assert!(a.insert(*k, *v2).is_none()),
        }
    }
}

/// Calculates a\[k\] -= b\[k\] for every key k in b
pub(crate) fn map_sub(a: &mut CityParticleMap, b: &CityParticleMap) {
    for (k, v2) in b {
        match a.get_mut(k) {
            Some(v) => *v -= *v2,
            None => panic!("Removing city w/ cityid {} without having added it previously", *k),
        }
    }
}

/// A struct which holds data describing a preaggregate (Fünf-Minuten-Aggregat).
/// Generated by [PreAggregate]
#[derive(Clone)]
pub struct PreAggregateData {
    /// The values of each city
    pub values: CityParticleMap,
    /// The maximum batch id of data used.
    /// None if the window contains no measurements
    pub maxbatch: Option<i64>,
}

/// An iterator which preaggregates a Vec<[LocalizedMeasurement]>
/// into a [CityParticleMap].
#[derive(Clone)]
pub struct PreAggregate<I>
where
    I : Iterator<Item = Vec<LocalizedMeasurement>> {
    inner: I
}

impl<I> Iterator for PreAggregate<I> 
where
    I : Iterator<Item = Vec<LocalizedMeasurement>> {
    type Item = PreAggregateData;

    fn next(&mut self) -> Option<Self::Item> {
        let mut map : CityParticleMap = Default::default();
        let measurements = self.inner.next()?;
        let maxbatch = measurements
            .last()
            .map(|x| x.batch_seq_id);
        for m in measurements {
            match map.get_mut(&m.cityid) {
                Some(aggregate) => aggregate.add((m.p1, m.p2)),
                None => assert!(map.insert(m.cityid, ParticleAggregate::new((m.p1, m.p2))).is_none()),
            }
        }
        Some(PreAggregateData {
            values: map,
            maxbatch,
        })
    }
}

/// Convenience trait which allows to generate a preaggregate for every
/// Iterator yielding Vec<LocalizedMeasurement>.
pub trait WithPreAggregate : Iterator<Item = Vec<LocalizedMeasurement>> {
    /// (Pre)aggregrates Vec<[LocalizedMeasurement]>s into a CityParticleMap.
    fn preaggregate(self) -> PreAggregate<Self> where Self: Sized {
        PreAggregate { inner: self }
    }
}

impl<T: Sized> WithPreAggregate for T where T: Iterator<Item = Vec<LocalizedMeasurement>> {}
