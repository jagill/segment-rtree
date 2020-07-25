use rstar::{RTree, RTreeObject, AABB};
use seg_rtree::utils::winding_number;
use seg_rtree::Coordinate;

pub struct Segment {
    start: Coordinate,
    end: Coordinate,
}

impl RTreeObject for Segment {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners([self.start.x, self.start.y], [self.end.x, self.end.y])
    }
}

pub(crate) fn build_rstar(coords: &[Coordinate]) -> RTree<Segment> {
    RTree::bulk_load(
        coords
            .windows(2)
            .map(|w| Segment {
                start: w[0],
                end: w[1],
            })
            .collect(),
    )
}

pub(crate) fn point_in_polygon_rstar(point: Coordinate, rtree: &RTree<Segment>) -> bool {
    let mut wn: i32 = 0;

    let ray = AABB::from_corners([point.x, point.y], [f64::INFINITY, point.y]);
    for seg in rtree.locate_in_envelope_intersecting(&ray) {
        wn += winding_number(point, seg.start, seg.end);
    }

    wn != 0
}
