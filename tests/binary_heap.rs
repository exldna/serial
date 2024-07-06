use std::cmp::Ordering;
use utils::rand::RngCore;

#[test]
fn test_basic() {
    let mut heap = serial::BinaryHeap::new();
    assert_eq!(heap.len(), 0);
    heap.push(3);
    heap.push(4);
    heap.push(1);
    heap.push(2);
    heap.push(5);
    assert_eq!(heap.len(), 5);
    assert_eq!(heap.pop(), Some(5));
    assert_eq!(heap.pop(), Some(4));
    assert_eq!(heap.pop(), Some(3));
    assert_eq!(heap.pop(), Some(2));
    assert_eq!(heap.pop(), Some(1));
    assert_eq!(heap.pop(), None);
    assert_eq!(heap.len(), 0);
}

#[test]
#[allow(dead_code)]
fn assert_covariance() {
    fn a<'a>(x: serial::BinaryHeap<&'static str>) -> serial::BinaryHeap<&'a str> {
        x
    }
}

#[test]
fn test_dropck() {
    let mock = utils::DropMock::new();
    {
        let mut heap = serial::BinaryHeap::<OrderedDropMock>::new();
        let mut random_range = utils::rand::thread_rng();
        for _ in 0..10 {
            heap.push(OrderedDropMock {
                0: random_range.next_u32(),
                1: mock.clone(),
            });
        }
    }
    assert_eq!(mock.drop_cnt(), 10);
}

/// OrderedDropMock counts drops and ordered like u32.
#[allow(dead_code)] // Compiler don't assume that dropping is use.
struct OrderedDropMock(u32, utils::DropMock);

impl PartialEq for OrderedDropMock {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for OrderedDropMock {}

impl PartialOrd for OrderedDropMock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for OrderedDropMock {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
