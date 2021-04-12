use std::collections::VecDeque;

struct SharedInner<A, B, It>
where
    It: Iterator<Item = (A, B)>,
{
    first: VecDeque<A>,
    second: VecDeque<B>,
    iter: It,
}

impl<A, B, It> SharedInner<A, B, It>
where
    It: Iterator<Item = (A, B)>,
{
    #[inline]
    fn next_first(&mut self) -> Option<A> {
        self.first.pop_front().or_else(|| {
            let (first, second) = self.iter.next()?;
            self.second.push_back(second);
            Some(first)
        })
    }

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
type Shared<A, B, It> = Rc<RefCell<SharedInner<A, B, It>>>;

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

pub struct IterPair<A,B>(pub A, pub B);

pub trait Spliter<A,B> : Iterator<Item=(A,B)> {
    fn spliter(self) -> IterPair<First<A, B, Self>, Second<A, B, Self>> where Self: Sized {
        split(self)
    }
}
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
