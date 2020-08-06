use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub struct MinHeap<T> {
    max_heap: BinaryHeap<Reverse<T>>,
}

impl<T: Ord + Copy> MinHeap<T> {
    pub fn new() -> Self {
        MinHeap {
            max_heap: BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, t: T) {
        self.max_heap.push(Reverse(t));
    }

    pub fn pop(&mut self) -> Option<T> {
        Some(self.max_heap.pop()?.0)
    }

    pub fn peek(&self) -> Option<T> {
        Some(self.max_heap.peek()?.0)
    }

    pub fn is_empty(&self) -> bool {
        self.max_heap.is_empty()
    }

    pub fn len(&self) -> usize {
        self.max_heap.len()
    }
}
