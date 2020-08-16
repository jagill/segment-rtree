use crate::geometry_state::{HasRTree, Validated};
use crate::utils::winding_number;
use crate::{Coordinate, LinearRing, Rectangle};

pub fn point_in_loop(point: Coordinate, path: &LinearRing<Validated>) -> bool {
    let coords = path.coords();
    let rtree = path.rtree();

    let mut wn: i32 = 0;

    // Stack entries: (level, offset)
    let mut stack = Vec::with_capacity(rtree.height() * rtree.degree());
    if check_point_rect(point, rtree.get_rectangle(rtree.height(), 0)) {
        stack.push((rtree.height(), 0));
    }
    while let Some((level, offset)) = stack.pop() {
        let rect = rtree.get_rectangle(level, offset);
        if rect.x_min > point.x {
            let (low, high) = rtree.get_low_high(level, offset);
            wn += winding_number(point, coords[low], coords[high]);
            continue;
        }
        if level == 0 {
            wn += winding_number(point, coords[offset], coords[offset + 1]);
        } else {
            let child_level = level - 1;
            let first_child_offset = rtree.degree() * offset;
            for child_offset in first_child_offset..(first_child_offset + rtree.degree()) {
                if check_point_rect(point, rtree.get_rectangle(child_level, child_offset)) {
                    stack.push((child_level, child_offset));
                }
            }
        }
    }
    wn != 0
}

// Check if a point is in the rectangle, or to its left
fn check_point_rect(point: Coordinate, rect: Rectangle) -> bool {
    point.x <= rect.x_max && point.y >= rect.y_min && point.y <= rect.y_max
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn check_containment() {
        let loop_a =
            LinearRing::try_from(vec![(0., 0.), (0., 1.), (1., 1.), (1., 0.), (0., 0.)]).unwrap();
        assert!(point_in_loop((0.5, 0.5).into(), &loop_a));
        assert!(point_in_loop((0.0, 0.0).into(), &loop_a));
        assert!(point_in_loop((0.5, 0.0).into(), &loop_a));
        assert!(point_in_loop((0.0, 0.5).into(), &loop_a));
        assert!(!point_in_loop((1.1, 0.0).into(), &loop_a));
    }
}
