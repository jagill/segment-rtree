use crate::errors::ValidationError;
use crate::geometry_state::{HasRTree, Prepared, Raw, Validated};
use crate::seg_rtree::SegRTree;
use crate::utils::{intersect_segments, rectangles_from_coordinates};
use crate::Coordinate;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct LineString<S> {
    coords: Vec<Coordinate>,
    state: S,
}

impl<S: HasRTree> HasRTree for LineString<S> {
    fn rtree(&self) -> &SegRTree {
        self.state.rtree()
    }
}

impl<S> LineString<S> {
    pub fn coords(&self) -> &[Coordinate] {
        &self.coords
    }
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
            SegRTree::new_loaded(16, &rectangles_from_coordinates(&self.coords))
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
    pub fn validate(self) -> Result<LineString<Validated>, ValidationError> {
        if self.coords.len() == 1 {
            return Err(ValidationError::SinglePathCoordinate);
        }
        for (index, range) in self.coords.windows(2).enumerate() {
            if range[0] == range[1] {
                return Err(ValidationError::DegenerateSegment {
                    index,
                    position: range[0],
                });
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

impl LineString<Validated> {}

impl<IP: Into<Coordinate>> TryFrom<Vec<IP>> for LineString<Validated> {
    type Error = ValidationError;

    fn try_from(coords: Vec<IP>) -> Result<Self, Self::Error> {
        LineString::new(coords.into_iter().map(|ip| ip.into()).collect())
            .prepare()
            .validate()
    }
}

fn check_intersection(
    index: usize,
    other_index: usize,
    coords: &[Coordinate],
) -> Result<(), ValidationError> {
    let first_index = index.min(other_index);
    let second_index = index.max(other_index);
    let first_start = coords[first_index];
    let first_end = coords[first_index + 1];
    let second_start = coords[second_index];
    let second_end = coords[second_index + 1];
    match intersect_segments(first_start, first_end, second_start, second_end) {
        None => Ok(()),
        Some((isxn_start, isxn_end)) => {
            if isxn_start != isxn_end {
                Err(ValidationError::OverlappingSegments {
                    first_index,
                    second_index,
                    start: isxn_start,
                    end: isxn_end,
                })
            } else if first_index == second_index - 1 {
                if isxn_start == second_start {
                    Ok(())
                } else {
                    Err(ValidationError::SelfIntersection {
                        first_index,
                        second_index,
                        position: isxn_start,
                    })
                }
            } else if first_index == 0 && second_index == coords.len() - 2 {
                if isxn_start == first_start && isxn_start == second_end {
                    Ok(())
                } else {
                    Err(ValidationError::SelfIntersection {
                        first_index,
                        second_index,
                        position: isxn_start,
                    })
                }
            } else {
                Err(ValidationError::SelfIntersection {
                    first_index,
                    second_index,
                    position: isxn_start,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_path() {
        let path =
            LineString::try_from(Vec::<Coordinate>::new()).expect("Construction should not fail.");
        assert_eq!(path.coords, Vec::new());
    }

    fn assert_path_ok(coords: Vec<(f64, f64)>) {
        let positions: Vec<Coordinate> = coords.clone().into_iter().map(|c| c.into()).collect();
        let path = LineString::try_from(coords).expect("Construction should not fail");
        assert_eq!(path.coords, positions);
        assert_eq!(path.rtree().len(), positions.len() - 1);
        let path = LineString::try_from(positions.clone()).expect("Construction should not fail");
        assert_eq!(path.coords, positions);
        assert_eq!(path.rtree().len(), positions.len() - 1);
    }

    fn assert_invalid_path(coords: Vec<(f64, f64)>, expected: ValidationError) {
        let positions: Vec<Coordinate> = coords.clone().into_iter().map(|c| c.into()).collect();
        let err = LineString::try_from(coords).expect_err("Expected validation to fail");
        assert_eq!(err, expected);
        let path = LineString::new(positions);
        let err = path.validate().expect_err("Expected validation to fail");
        assert_eq!(err, expected);
    }

    #[test]
    fn test_basic_paths() {
        assert_path_ok(vec![(0., 0.), (1., 1.)]);
        assert_path_ok(vec![(0., 0.), (1., 1.), (2., 2.)]);
        assert_path_ok(vec![(0., 0.), (1., 0.), (0., 1.), (0., 0.)]);
    }

    #[test]
    fn test_invalid_paths() {
        assert_invalid_path(vec![(0., 0.)], ValidationError::SinglePathCoordinate);
        assert_invalid_path(
            vec![(0., 0.), (1., 1.), (1., 0.), (0., 1.)],
            ValidationError::SelfIntersection {
                first_index: 0,
                second_index: 2,
                position: (0.5, 0.5).into(),
            },
        );
        assert_invalid_path(
            vec![(0., 0.), (0., 1.), (0., 0.5)],
            ValidationError::OverlappingSegments {
                first_index: 0,
                second_index: 1,
                // start: (0.0, 1.0).into(),
                // end: (0.0, 0.5).into(),
                end: (0.0, 1.0).into(),
                start: (0.0, 0.5).into(),
            },
        );
        assert_invalid_path(
            vec![(0., 0.), (0., 1.), (0.5, 0.), (1., 1.), (1., 0.), (0., 0.)],
            ValidationError::SelfIntersection {
                first_index: 2,
                second_index: 4,
                position: (0.5, 0.).into(),
            },
        );
        assert_invalid_path(
            vec![
                (0., 0.),
                (0., 1.),
                (0.5, 0.5),
                (1., 1.),
                (1., 0.),
                (0.5, 0.5),
            ],
            ValidationError::SelfIntersection {
                first_index: 2,
                second_index: 4,
                position: (0.5, 0.5).into(),
            },
        );
    }
}
