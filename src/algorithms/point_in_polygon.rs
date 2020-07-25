use crate::errors::ValidationError;
use crate::utils::winding_number;
use crate::{Coordinate, LineString, Rectangle};

pub fn point_in_polygon(point: Coordinate, path: &LineString) -> Result<bool, ValidationError> {
    let coords = path.coords();
    let rtree = path.rtree();
    if coords.len() < 4 || coords[0] != coords[coords.len() - 1] {
        return Err(ValidationError::NotARing);
    }

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
    Ok(wn != 0)
}

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
            LineString::try_from(vec![(0., 0.), (0., 1.), (1., 1.), (1., 0.), (0., 0.)]).unwrap();
        assert!(point_in_polygon((0.5, 0.5).into(), &loop_a).unwrap());
        assert!(point_in_polygon((0.0, 0.0).into(), &loop_a).unwrap());
        assert!(point_in_polygon((0.5, 0.0).into(), &loop_a).unwrap());
        assert!(point_in_polygon((0.0, 0.5).into(), &loop_a).unwrap());
        assert!(!point_in_polygon((1.1, 0.0).into(), &loop_a).unwrap());
    }
}
