use crate::algorithms::validate_polygon;
use crate::errors::ValidationError;
use crate::geometry_state::{Prepared, Raw, Validated};
use crate::LinearRing;

pub struct Polygon<S> {
    shell: LinearRing<S>,
    holes: Vec<LinearRing<S>>,
}

impl<S> Polygon<S> {
    pub fn shell(&self) -> &LinearRing<S> {
        &self.shell
    }

    pub fn holes(&self) -> &[LinearRing<S>] {
        &self.holes
    }
}

impl Polygon<Raw> {
    pub fn new(shell: LinearRing<Raw>, holes: Vec<LinearRing<Raw>>) -> Self {
        Polygon { shell, holes }
    }

    pub fn prepare(self) -> Polygon<Prepared> {
        let shell = self.shell.prepare();
        let holes = self.holes.into_iter().map(|hole| hole.prepare()).collect();
        Polygon { shell, holes }
    }
}

impl Polygon<Prepared> {
    pub fn new(shell: LinearRing<Prepared>, holes: Vec<LinearRing<Prepared>>) -> Self {
        Polygon { shell, holes }
    }

    pub fn validate(self) -> Result<Polygon<Validated>, ValidationError> {
        let shell = self.shell.validate()?;
        let try_holes: Result<Vec<_>, _> =
            self.holes.into_iter().map(|hole| hole.validate()).collect();
        let holes = try_holes?;
        validate_polygon(&shell, &holes)?;
        Ok(Polygon { shell, holes })
    }
}
