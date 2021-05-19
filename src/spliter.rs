use std::collections::VecDeque;

// Code in this file based on
// https://stackoverflow.com/a/25588440

/// Temporarily stores the items
/// obtained from iterator iter
/// to make it possible to access them
/// from the different iterators
struct SharedInner<A, B, It>
where
    It: Iterator<Item = (A, B)>,
{
    /// Temporary store for the first items
    /// in the tuple yielded by the iterator
    first: VecDeque<A>,
    /// Temporary store for the second items
    /// in the tuple yielded by the iterator
    second: VecDeque<B>,
    /// An iterator yielding a 2-tuple
    iter: It,
}

impl<A, B, It> SharedInner<A, B, It>
where
    It: Iterator<Item = (A, B)>,
{
    /// Returns the next first item of the tuples
    /// yielded by the inner iterator
    #[inline]
    fn next_first(&mut self) -> Option<A> {
        self.first.pop_front().or_else(|| {
            let (first, second) = self.iter.next()?;
            self.second.push_back(second);
            Some(first)
        })
    }

    /// Returns the next second item of the tuples
    /// yielded by the inner iterator
    #[inline]
    fn next_second(&mut self) -> Option<B> {
        self.second.pop_front().or_else(|| {
            let (first, second) = self.iter.next()?;
            self.first.push_back(first);
            Some(second)
        })
    }
}

use std::cell::RefCell;
use std::rc::Rc;
/// The Shared type allows to keep a reference to the SharedInner type,
/// which makes it possible to create two different struct ([First]
/// and [Second]), which implement Iterator for the different items
/// in the shared inner iterator.
type Shared<A, B, It> = Rc<RefCell<SharedInner<A, B, It>>>;

/// An iterator yielding all the items in the first position of the
/// tuples yielded by the iterator of type It.
pub struct First<A, B, It>
where
    It: Iterator<Item = (A, B)>,
{
    data: Shared<A, B, It>,
}

impl<A, B, It> Iterator for First<A, B, It>
where
    It: Iterator<Item = (A, B)>,
{
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        let mut inner = self.data.borrow_mut();
        inner.next_first()
    }
}

/// An iterator yielding all the items in the second position of the
/// tuples yielded by the iterator of type It.
pub struct Second<A, B, It>
where
    It: Iterator<Item = (A, B)>,
{
    data: Shared<A, B, It>,
}

impl<A, B, It> Iterator for Second<A, B, It>
where
    It: Iterator<Item = (A, B)>,
{
    type Item = B;

    fn next(&mut self) -> Option<Self::Item> {
        let mut inner = self.data.borrow_mut();
        inner.next_second()
    }
}


/// Splits an iterator which yields a 2-tuple into a
/// pair of two iterators, each iterator returns the corresponding
/// item of the 2-tuple.
fn split<A, B, It>(it: It) -> IterPair<First<A, B, It>, Second<A, B, It>>
where
    It: Iterator<Item = (A, B)>,
{
    let data = Rc::new(RefCell::new(SharedInner { 
        iter: it,
        first: VecDeque::new(),
        second: VecDeque::new(),
    }));

    IterPair(First { data: data.clone() }, Second { data })
}

/// A pair of iterators.
/// This is used to be able to implement methods/traits on a tuple of iterators.
pub struct IterPair<A,B>(pub A, pub B);

/// Convenience trait to be able to split an iterator yielding 2-tuples.
pub trait Spliter<A,B> : Iterator<Item=(A,B)> {
    /// Splits an iterator yielding tuples. See [split]
    fn spliter(self) -> IterPair<First<A, B, Self>, Second<A, B, Self>> where Self: Sized {
        split(self)
    }
}

/// Implement the trait on each Iterator yielding a 2-tuple.
impl<A,B,T: Sized> Spliter<A,B> for T where T: Iterator<Item = (A, B)> {}

#[cfg(test)]
mod spliter_test {
    use super::{IterPair,Spliter};
    #[test]
    fn simple_spliter_test() {
        let IterPair(first, second) = (1..5)
            .map(|x| (x, 6.0-x as f32/2.0))
            .spliter();
        assert_eq!(first.collect::<Vec<_>>(), vec![1, 2, 3, 4]);
        assert_eq!(second.collect::<Vec<_>>(), vec![5.5, 5.0, 4.5, 4.0]);
    }
}
