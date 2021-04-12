type CityId = u32;

#[derive(Debug, Copy, Clone)]
pub struct LocalizedMeasurement {
    pub batch_seq_id: i64,
    pub cityid: CityId,
    pub timestamp_seconds: i64,
    ///Particles < 10µm (particulate matter)
    pub p1: f32,
    ///Particles < 2.5µm (ultrafine particles)
    pub p2: f32,
}

pub struct TimePartitionIterator<T, F>
where
    T: Iterator<Item = LocalizedMeasurement>,
    F: Fn(Option<i64>, &LocalizedMeasurement) -> i64,
{
    currrent_partition_end_seconds: i64,
    values: Vec<LocalizedMeasurement>,
    iter: std::iter::Peekable<T>,
    next_timestamp_fn: F,
}

impl<T, F> TimePartitionIterator<T, F>
where
    T: Iterator<Item = LocalizedMeasurement>,
    F: Fn(Option<i64>, &LocalizedMeasurement) -> i64,
{
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

fn partition_5min(
    current_window_end: Option<i64>,
    non_fitting_measurement: &LocalizedMeasurement,
) -> i64 {
    current_window_end
        .map(|x| x + 300)
        .unwrap_or_else(|| non_fitting_measurement.timestamp_seconds + 300)
}

pub trait Partition5Min: Iterator<Item = LocalizedMeasurement> {
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

impl<T: Sized> Partition5Min for T where T: Iterator<Item = LocalizedMeasurement> {}

use std::collections::VecDeque;
pub type AnalysisWindowIter<'a, T> = std::collections::vec_deque::Iter<'a, T>;

pub struct AnalysisWindow<'lcur, 'llast, TCur, TLast> {
    pub current: AnalysisWindowIter<'lcur, TCur>,
    pub lastyear: AnalysisWindowIter<'llast, TLast>,
}

pub struct AnalysisWindowsMap<ICur, TCur, ILast, TLast, F, TOut>
where
    ICur: Iterator<Item = TCur>,
    ILast: Iterator<Item = TLast>,
    TCur: 'static,
    TLast: 'static,
    F: for<'a> Fn(AnalysisWindow<'a, 'a, TCur, TLast>) -> TOut,
{
    current_iter: ICur,
    current_queue: VecDeque<TCur>,

    lastyear_iter: ILast,
    lastyear_queue: VecDeque<TLast>,

    map_func: F,
    completed: bool,
}

impl<ICur, TCur, ILast, TLast, F, TOut> AnalysisWindowsMap<ICur, TCur, ILast, TLast, F, TOut>
where
    ICur: Iterator<Item = TCur>,
    ILast: Iterator<Item = TLast>,
    TOut: 'static,
    F: for<'a> Fn(AnalysisWindow<'a, 'a, TCur, TLast>) -> TOut,
{

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
            completed
        }
    }
}


impl<ICur, TCur, ILast, TLast, F, TOut> Iterator for AnalysisWindowsMap<ICur, TCur, ILast, TLast, F, TOut>
where
    ICur: Iterator<Item = TCur>,
    ILast: Iterator<Item = TLast>,
    TCur: 'static,
    TLast: 'static,
    TOut: 'static,
    F: for<'a> Fn(AnalysisWindow<'a, 'a, TCur, TLast>) -> TOut,
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
        let res = Some((self.map_func)(window));

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

        res
    }
}

use crate::spliter::IterPair;

impl<ICur, TCur, ILast, TLast> IterPair<ICur, ILast>
where
    ICur: Iterator<Item = TCur>,
    ILast: Iterator<Item = TLast>,
    TCur: 'static,
    TLast: 'static,
    {
    pub fn with_analysis_windows<F, TOut>(self, current_window_size: usize, lastyear_window_size: usize, map_func: F) -> AnalysisWindowsMap<ICur, TCur, ILast, TLast, F, TOut>
    where
    TOut: 'static,
    F: for<'a> Fn(AnalysisWindow<'a, 'a, TCur, TLast>) -> TOut {
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
            .with_analysis_windows(2, 2, |window| {
                let current_total : i32 = window.current.sum();
                let lastyear_total : i32 = window.lastyear.sum();
                assert_eq!(current_total+2, lastyear_total);
                current_total + lastyear_total
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
            .with_analysis_windows(2, 2, |window| {
                let current_total : i32 = window.current.sum();
                let lastyear_total : i32 = window.lastyear.sum();
                assert_eq!(current_total+2, lastyear_total);
                current_total + lastyear_total
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
            .with_analysis_windows(1, 3, |window| {
                let current_total : i32 = window.current.sum();
                let lastyear_total : i32 = window.lastyear.sum();
                current_total + lastyear_total
            })
            .collect::<Vec<i32>>();
        assert_eq!(x, vec![
            3+(2+3+4),
            4+(3+4+5),
        ]);
    }
}


#[derive(Debug,Copy,Clone)]
pub struct ParticleAggregate {
    sum_p1: f32,
    sum_p2: f32,
    denom: usize,
}

impl ParticleAggregate {
    pub fn new(init: (f32, f32)) -> Self {
        Self {
            sum_p1: init.0,
            sum_p2: init.1,
            denom: 1,
        }
    }

    pub fn add(&mut self, val: (f32, f32)) {
        self.sum_p1 += val.0;
        self.sum_p2 += val.1;
        self.denom += 1;
    }

    pub fn p1(&self) -> f32 {
        self.sum_p1 / (self.denom as f32)
    }

    pub fn p2(&self) -> f32 {
        self.sum_p2 / (self.denom as f32)
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
            .fold((0.0, 0.0, 0), |acc, (p1, p2)| (acc.0 + p1, acc.1 + p2, acc.2 + 1));
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

#[derive(Debug,Default,Clone)]
struct ActiveCities {
    inner: HashSet<CityId>,
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

type CityParticleMap = std::collections::HashMap<CityId, ParticleAggregate>;
// type CityParticleMap = std::collections::BTreeMap<CityId, ParticleAggregate>;

#[derive(Clone)]
struct PreAggregate<I>
where
    I : Iterator<Item = Vec<LocalizedMeasurement>> {
    inner: I
}

impl<I> Iterator for PreAggregate<I> 
where
    I : Iterator<Item = Vec<LocalizedMeasurement>> {
    type Item = CityParticleMap;

    fn next(&mut self) -> Option<Self::Item> {
        let mut map : CityParticleMap = Default::default();
        let measurements = self.inner.next()?;
        for m in measurements {
            match map.get_mut(&m.cityid) {
                Some(aggregate) => aggregate.add((m.p1, m.p2)),
                None => assert!(map.insert(m.cityid, ParticleAggregate::new((m.p1, m.p2))).is_none()),
            }
        }
        Some(map)
    }
}

#[allow(unreachable_code)]
fn partition_test() {
    struct TestBatch {
        current: Vec<LocalizedMeasurement>,
        lastyear: Vec<LocalizedMeasurement>,
    }
    struct SegmentInfo {
        measurements: Vec<LocalizedMeasurement>,
        active: ActiveCities,
    }
    impl SegmentInfo {
        fn from_vec(measurements: Vec<LocalizedMeasurement>) -> Self {
            let active = measurements.iter().map(|m| m.cityid).collect();
            Self {
                measurements,
                active,
            }
        }
    }
    let mut tpi : TimePartitionIterator<_, _> = Vec::<LocalizedMeasurement>::new()
        .into_iter()
        .partition_5min();

    let segs = tpi
        .map(|current| SegmentInfo::from_vec(current));
    /* let batches : Vec<TestBatch> = vec![];
    batches
        .into_iter()
        .map(|batch| (batch.current.into_iter(), batch.lastyear.into_iter()))
        .map(|(current, lastyear)| (current.partition_5min(), lastyear.partition_5min())); */
    
    let window : (AnalysisWindowIter<SegmentInfo>, AnalysisWindowIter<SegmentInfo>) = todo!();
    std::iter::once(window)
        .map(|(current, lastyear)| {
            let active = current.fold(ActiveCities::default(), |acc: ActiveCities, x| acc | x.active.clone());
        });
}
