use crate::Coordinate;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ValidationError {
    #[error("Coordinate has a non-finite component")]
    NonFiniteCoordinate,

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

    #[error("Hole is not contained in the shell or intersects too many times.")]
    HoleNotValid,

    #[error("Polygon rings have >1 intersection.")]
    MultipleIntersections,

    #[error("Polygon interior is disconneted.")]
    InteriorDisconnected,

    #[error("Rings must have at least 4 coordinates.")]
    TooFewCoordinates,

    #[error("Rings must have their first and last coordinate equal.")]
    NotClosed,
}
