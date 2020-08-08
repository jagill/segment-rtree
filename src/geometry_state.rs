use crate::{HasEnvelope, Rectangle, SegRTree};

#[derive(Debug, Clone, Copy)]
pub struct Raw;

#[derive(Debug, Clone)]
pub struct Prepared {
    pub(crate) rtree: SegRTree,
}

#[derive(Debug, Clone)]
pub struct Validated {
    pub(crate) rtree: SegRTree,
}

pub trait HasRTree: HasEnvelope {
    fn rtree(&self) -> &SegRTree;
}

impl<C: HasRTree> HasEnvelope for C {
    fn envelope(&self) -> Rectangle {
        self.rtree().envelope()
    }
}

impl HasRTree for Prepared {
    fn rtree(&self) -> &SegRTree {
        &self.rtree
    }
}

impl HasRTree for Validated {
    fn rtree(&self) -> &SegRTree {
        &self.rtree
    }
}
