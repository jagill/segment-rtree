use crate::errors::ValidationError;
use crate::LineString;

pub struct Polygon {
    shell: LineString,
    holes: Vec<LineString>,
}

impl Polygon {
    pub fn new(shell: LineString, holes: Vec<LineString>) -> Self {
        Polygon { shell, holes }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }

    pub fn shell(&self) -> &LineString {
        &self.shell
    }

    pub fn holes(&self) -> &[LineString] {
        &self.holes
    }
}
