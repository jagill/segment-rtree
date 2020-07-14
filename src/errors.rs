use crate::Coordinate;
use thiserror::Error;

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
    NotARing,
}
