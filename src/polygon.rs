use crate::errors::ValidationError;
use crate::geometry_state::Validated;
use crate::LinearRing;

pub struct Polygon {
    shell: LinearRing<Validated>,
    holes: Vec<LinearRing<Validated>>,
}

impl Polygon {
    pub fn new(shell: LinearRing<Validated>, holes: Vec<LinearRing<Validated>>) -> Self {
        Polygon { shell, holes }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }

    pub fn shell(&self) -> &LinearRing<Validated> {
        &self.shell
    }

    pub fn holes(&self) -> &[LinearRing<Validated>] {
        &self.holes
    }
}
