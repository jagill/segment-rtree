use crate::SegRTree;
use crate::{Coordinate, Rectangle};

#[derive(Debug)]
struct ValidationError;
#[derive(Debug)]
struct RingError;

// STATE

struct Raw;

struct Prepared {
    rtree: SegRTree,
}

struct Validated {
    rtree: SegRTree,
}

// LINESTRING

struct LineString<S> {
    coords: Vec<Coordinate>,
    state: S,
}

impl LineString<Raw> {
    pub fn new(coords: Vec<Coordinate>) -> Self {
        LineString {
            coords,
            state: Raw {},
        }
    }

    pub fn prepare(self) -> LineString<Prepared> {
        let rtree = if self.coords.is_empty() {
            SegRTree::new_empty()
        } else {
            let rectangles: Vec<Rectangle> = self
                .coords
                .windows(2)
                .map(|c| Rectangle::new(c[0], c[1]))
                .collect();

            SegRTree::new_loaded(16, &rectangles)
        };
        LineString {
            coords: self.coords,
            state: Prepared { rtree },
        }
    }

    pub fn validate(self) -> Result<LineString<Validated>, ValidationError> {
        self.prepare().validate()
    }
}

impl LineString<Prepared> {
    pub fn rtree(&self) -> &SegRTree {
        &self.state.rtree
    }

    pub fn validate(self) -> Result<LineString<Validated>, ValidationError> {
        if self.coords.len() == 1 {
            return Err(ValidationError);
        }
        for (index, range) in self.coords.windows(2).enumerate() {
            if range[0] == range[1] {
                return Err(ValidationError);
            }
        }

        for (index_a, index_b) in self.rtree().query_self_intersections() {
            check_intersection(index_a, index_b, &self.coords)?;
        }
        Ok(LineString {
            coords: self.coords,
            state: Validated {
                rtree: self.state.rtree,
            },
        })
    }
}

// LinearRing

struct LinearRing<S> {
    coords: Vec<Coordinate>,
    state: S,
}

impl<S> LineString<S> {
    pub fn is_ring(&self) -> bool {
        self.coords.len() > 3 || self.coords.first() == self.coords.last()
    }

    pub fn into_ring(self) -> Result<LinearRing<S>, RingError> {
        if !self.is_ring() {
            Err(RingError)
        } else {
            Ok(LinearRing {
                coords: self.coords,
                state: self.state,
            })
        }
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

// POLYGON

#[allow(dead_code)]
struct Polygon<S> {
    shell: LinearRing<S>,
    holes: Vec<LinearRing<S>>,
    // Should Prepared polygons have an RTree of their rings?
}

#[allow(dead_code)]
impl Polygon<Raw> {
    pub fn new(shell: LinearRing<Raw>, holes: Vec<LinearRing<Raw>>) -> Self {
        Polygon { shell, holes }
    }

    pub fn prepare(self) -> Polygon<Prepared> {
        Polygon {
            shell: self.shell.prepare(),
            holes: self.holes.into_iter().map(|ls| ls.prepare()).collect(),
        }
    }

    pub fn validate(self) -> Result<Polygon<Validated>, ValidationError> {
        self.prepare().validate()
    }
}

#[allow(dead_code)]
impl Polygon<Prepared> {
    pub fn new(shell: LinearRing<Prepared>, holes: Vec<LinearRing<Prepared>>) -> Self {
        Polygon { shell, holes }
    }

    pub fn validate(self) -> Result<Polygon<Validated>, ValidationError> {
        let shell = self.shell.validate()?;
        let holes_result: Result<Vec<LinearRing<Validated>>, ValidationError> =
            self.holes.into_iter().map(|ls| ls.validate()).collect();
        let holes = holes_result?;
        check_polygon_validity(&shell, &holes)?;

        Ok(Polygon { shell, holes })
    }
}

#[allow(dead_code)]
impl Polygon<Validated> {
    pub fn try_new(
        shell: LinearRing<Validated>,
        holes: Vec<LinearRing<Validated>>,
    ) -> Result<Self, ValidationError> {
        check_polygon_validity(&shell, &holes)?;
        Ok(Polygon { shell, holes })
    }
}

fn check_intersection(
    index: usize,
    other_index: usize,
    coords: &[Coordinate],
) -> Result<(), ValidationError> {
    // STUB
    Ok(())
}

fn check_polygon_validity(
    shell: &LinearRing<Validated>,
    holes: &[LinearRing<Validated>],
) -> Result<(), ValidationError> {
    // STUB
    Ok(())
}
