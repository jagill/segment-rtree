use std::collections::BTreeSet;

#[derive(Default)]
pub struct SegmentUnion {
    set: BTreeSet<usize>,
}

impl SegmentUnion {
    pub fn new() -> Self {
        SegmentUnion {
            set: BTreeSet::new(),
        }
    }

    pub fn add(&mut self, low: usize, high: usize) {
        self._add(low);
        self._add(high);
    }

    fn _add(&mut self, entry: usize) {
        if self.set.contains(&entry) {
            self.set.remove(&entry);
        } else {
            self.set.insert(entry);
        }
    }

    pub fn peek(&self) -> Option<usize> {
        // Really?  This is a little ridiculous. https://github.com/rust-lang/rust/issues/62924
        Some(*self.set.iter().next()?)
    }

    fn _pop(&mut self) -> Option<usize> {
        let ret = self.peek()?;
        self.set.remove(&ret);
        Some(ret)
    }

    /// Pop two elements, as a low-high pair.
    pub fn pop(&mut self) -> Option<(usize, usize)> {
        Some((self._pop()?, self._pop()?))
    }

    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    /// Number of contained indicies.  This is twice the number of low-high pairs.
    pub fn len(&self) -> usize {
        self.set.len()
    }
}
