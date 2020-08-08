use crate::errors::ValidationError;
use crate::geometry_state::{HasRTree, Prepared, Raw, Validated};
use crate::{Coordinate, LineString, SegRTree};
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct LinearRing<S> {
    coords: Vec<Coordinate>,
    state: S,
}

impl<S> LineString<S> {
    pub fn validate_ring(&self) -> Result<(), ValidationError> {
        if self.coords().len() < 3 {
            Err(ValidationError::TooFewCoordinates)
        } else if self.coords().first() != self.coords().last() {
            Err(ValidationError::NotClosed)
        } else {
            Ok(())
        }
    }

    pub fn is_ring(&self) -> bool {
        self.validate_ring().is_ok()
    }

    pub fn into_ring(self) -> Result<LinearRing<S>, ValidationError> {
        self.validate_ring()?;
        Ok(LinearRing {
            coords: self.coords,
            state: self.state,
        })
    }
}

impl<S> LinearRing<S> {
    pub fn coords(&self) -> &Vec<Coordinate> {
        &self.coords
    }
}

impl<S: HasRTree> HasRTree for LinearRing<S> {
    fn rtree(&self) -> &SegRTree {
        self.state.rtree()
    }
}

impl LinearRing<Raw> {
    pub fn prepare(self) -> LinearRing<Prepared> {
        LineString {
            coords: self.coords,
            state: self.state,
        }
        .prepare()
        .into_ring()
        .unwrap()
    }

    pub fn validate(self) -> Result<LinearRing<Validated>, ValidationError> {
        self.prepare().validate()
    }
}

impl LinearRing<Prepared> {
    pub fn validate(self) -> Result<LinearRing<Validated>, ValidationError> {
        Ok(LineString {
            coords: self.coords,
            state: self.state,
        }
        .validate()?
        .into_ring()
        .unwrap())
    }
}

impl<IP: Into<Coordinate>> TryFrom<Vec<IP>> for LinearRing<Validated> {
    type Error = ValidationError;

    fn try_from(coords: Vec<IP>) -> Result<Self, Self::Error> {
        LineString::new(coords.into_iter().map(|ip| ip.into()).collect())
            .prepare()
            .validate()?
            .into_ring()
    }
}
