mod coordinate;
pub mod errors;
mod flatbush;
pub mod from_wkt;
mod polygon;
mod rectangle;
mod seg_rtree;
mod segment_path;
mod utils;

pub use crate::seg_rtree::{SegRTree, SegmentUnion};
pub use coordinate::Coordinate;
pub use flatbush::Flatbush;
pub use polygon::Polygon;
pub use rectangle::Rectangle;
pub use segment_path::SegmentPath;

pub mod algorithms;
