use crate::errors::ValidationError;
use crate::geometry_state::Validated;
use crate::LineString;

pub struct Polygon {
    shell: LineString<Validated>,
    holes: Vec<LineString<Validated>>,
}

impl Polygon {
    pub fn new(shell: LineString<Validated>, holes: Vec<LineString<Validated>>) -> Self {
        Polygon { shell, holes }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }

    pub fn shell(&self) -> &LineString<Validated> {
        &self.shell
    }

    pub fn holes(&self) -> &[LineString<Validated>] {
        &self.holes
    }
}
