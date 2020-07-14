mod coordinate;
pub mod errors;
mod flatbush;
pub mod from_wkt;
mod line_string;
mod polygon;
mod rectangle;
mod seg_rtree;
mod utils;

pub use crate::seg_rtree::{SegRTree, SegmentUnion};
pub use coordinate::Coordinate;
pub use flatbush::Flatbush;
pub use line_string::LineString;
pub use polygon::Polygon;
pub use rectangle::Rectangle;

pub mod algorithms;
