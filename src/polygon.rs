use crate::algorithms::validate_polygon;
use crate::errors::ValidationError;
use crate::geometry_state::{HasRTree, Prepared, Raw, Validated};
use crate::{HasEnvelope, LineString, LinearRing, Rectangle};

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

impl<S: HasRTree> HasEnvelope for Polygon<S> {
    fn envelope(&self) -> Rectangle {
        self.shell.envelope()
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
        Polygon::try_new(shell, holes)
    }
}

impl Polygon<Validated> {
    pub fn try_new(
        shell: LinearRing<Validated>,
        holes: Vec<LinearRing<Validated>>,
    ) -> Result<Self, ValidationError> {
        validate_polygon(&shell, &holes)?;
        Ok(Polygon { shell, holes })
    }

    pub fn clone_to_raw(&self) -> Polygon<Raw> {
        Polygon {
            shell: LineString::new(self.shell.coords().clone())
                .into_ring()
                .unwrap(),
            holes: self
                .holes
                .iter()
                .map(|hole| LineString::new(hole.coords().clone()))
                .map(|ls| ls.into_ring().unwrap())
                .collect(),
        }
    }
}
