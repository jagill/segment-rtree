use crate::errors::ValidationError;
use crate::SegmentPath;

pub struct Polygon {
    shell: SegmentPath,
    holes: Vec<SegmentPath>,
}

impl Polygon {
    pub fn new(shell: SegmentPath, holes: Vec<SegmentPath>) -> Self {
        Polygon { shell, holes }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }

    pub fn shell(&self) -> &SegmentPath {
        &self.shell
    }

    pub fn holes(&self) -> &[SegmentPath] {
        &self.holes
    }
}
