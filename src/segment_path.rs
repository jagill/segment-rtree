use crate::seg_rtree::SegRTree;
use crate::utils::intersect_segments;
use crate::{Coordinate, Rectangle};
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug)]
pub struct SegmentPath {
    coords: Vec<Coordinate>,
    rtree: SegRTree,
}

#[derive(Error, Debug, PartialEq)]
pub enum ValidationError {
    #[error("Path has only 1 coordinate")]
    SinglePathCoordinate,

    #[error("Degenenerate Segment {index} at {position:?}")]
    DegenerateSegment { index: usize, position: Coordinate },

    #[error("Overlapping segments {first_index} {second_index} between {start:?} and {end:?}")]
    OverlappingSegments {
        first_index: usize,
        second_index: usize,
        start: Coordinate,
        end: Coordinate,
    },

    #[error("Self-intersection for segments {first_index} {second_index} at {position:?}")]
    SelfIntersection {
        first_index: usize,
        second_index: usize,
        position: Coordinate,
    },

    #[error("Path is not a loop: first and last coordinates are not equal.")]
    NotALoop,
}

impl<IP: Into<Coordinate>> TryFrom<Vec<IP>> for SegmentPath {
    type Error = ValidationError;

    fn try_from(coords: Vec<IP>) -> Result<Self, Self::Error> {
        SegmentPath::new_validated(coords.into_iter().map(|ip| ip.into()).collect())
    }
}

impl SegmentPath {
    pub fn new(coords: Vec<Coordinate>) -> Self {
        let rtree = if coords.is_empty() {
            SegRTree::new_empty()
        } else {
            let rectangles: Vec<Rectangle> = coords
                .windows(2)
                .map(|c| Rectangle::new(c[0], c[1]))
                .collect();

            SegRTree::new_loaded(16, &rectangles)
        };

        SegmentPath { coords, rtree }
    }

    /// Create a SegmentPath and validate it during construction.
    pub fn new_validated(coords: Vec<Coordinate>) -> Result<Self, ValidationError> {
        if coords.is_empty() {
            return Ok(SegmentPath {
                coords,
                rtree: SegRTree::new_empty(),
            });
        }
        if coords.len() == 1 {
            return Err(ValidationError::SinglePathCoordinate);
        }
        let mut rtree = SegRTree::new(16, coords.len() - 1);
        for (index, range) in coords.windows(2).enumerate() {
            let start = range[0];
            let end = range[1];
            if start == end {
                return Err(ValidationError::DegenerateSegment {
                    index,
                    position: start,
                });
            }

            for other_index in rtree.query_rect(Rectangle::new(start, end)) {
                check_intersection(index, other_index, &coords)?;
            }
            rtree
                .add(Rectangle::new(start, end))
                .expect("Incorrect rtree size in construction");
        }
        Ok(SegmentPath { coords, rtree })
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
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

        for (index_a, index_b) in self.rtree.query_self_intersections() {
            check_intersection(index_a, index_b, &self.coords)?;
        }
        Ok(())
    }

    pub fn contains<IP: Into<Coordinate>>(&self, point: IP) -> Result<bool, String> {
        self.rtree.check_containment(point.into(), &self.coords)
    }

    pub fn coords(&self) -> &[Coordinate] {
        &self.coords
    }

    pub fn rtree(&self) -> &SegRTree {
        &self.rtree
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
        let path = SegmentPath::new_validated(Vec::new()).expect("Construction should not fail.");
        assert_eq!(path.coords, Vec::new());
    }

    fn assert_path_ok(coords: Vec<(f64, f64)>) {
        let positions: Vec<Coordinate> = coords.clone().into_iter().map(|c| c.into()).collect();
        let path = SegmentPath::try_from(coords).expect("Construction should not fail");
        assert_eq!(path.coords, positions);
        assert_eq!(path.rtree.len(), positions.len() - 1);
        let path = SegmentPath::new(positions.clone());
        path.validate().unwrap();
        assert_eq!(path.coords, positions);
        assert_eq!(path.rtree.len(), positions.len() - 1);
    }

    fn assert_invalid_path(coords: Vec<(f64, f64)>, expected: ValidationError) {
        let positions: Vec<Coordinate> = coords.clone().into_iter().map(|c| c.into()).collect();
        let err = SegmentPath::try_from(coords).expect_err("Expected validation to fail");
        assert_eq!(err, expected);
        let path = SegmentPath::new(positions);
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

    #[test]
    fn check_containment() {
        let loop_a =
            SegmentPath::try_from(vec![(0., 0.), (0., 1.), (1., 1.), (1., 0.), (0., 0.)]).unwrap();
        assert!(loop_a.contains((0.5, 0.5)).unwrap());
        assert!(loop_a.contains((0.0, 0.0)).unwrap());
        assert!(loop_a.contains((0.5, 0.0)).unwrap());
        assert!(loop_a.contains((0.0, 0.5)).unwrap());
        assert!(!loop_a.contains((1.1, 0.0)).unwrap());
    }
}
