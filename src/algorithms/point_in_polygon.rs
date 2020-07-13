use crate::utils::winding_number;
use crate::Coordinate;
use crate::SegmentPath;

pub fn point_in_polygon(point: Coordinate, path: &SegmentPath) -> Result<bool, String> {
    let coords = path.coords();
    let rtree = path.rtree();
    if coords.len() - 1 != rtree.len() {
        return Err("Supplied coordinates don't match SegRTree size.".to_owned());
    }
    if coords.len() < 4 {
        return Err(format!(
            "Only supplied {} coordinates; can't be a loop.",
            coords.len()
        ));
    }
    if coords[0] != coords[coords.len() - 1] {
        return Err("Coordinates are not a loop.".to_owned());
    }

    let mut wn: i32 = 0;

    // Stack entries: (level, offset)
    let mut stack = vec![(rtree.height(), 0)];
    while let Some((level, offset)) = stack.pop() {
        let rect = rtree.get_rectangle(level, offset);
        if rect.x_min > point.x {
            let (low, high) = rtree.get_low_high(level, offset);
            wn += winding_number(point, coords[low], coords[high]);
            continue;
        }
        if !rect.contains(point) {
            continue;
        }
        if level == 0 {
            wn += winding_number(point, coords[offset], coords[offset + 1]);
        } else {
            let child_level = level - 1;
            let first_child_offset = rtree.degree() * offset;
            for child_offset in first_child_offset..(first_child_offset + rtree.degree()) {
                stack.push((child_level, child_offset));
            }
        }
    }

    Ok(wn != 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn check_containment() {
        let loop_a =
            SegmentPath::try_from(vec![(0., 0.), (0., 1.), (1., 1.), (1., 0.), (0., 0.)]).unwrap();
        assert!(point_in_polygon((0.5, 0.5).into(), &loop_a).unwrap());
        assert!(point_in_polygon((0.0, 0.0).into(), &loop_a).unwrap());
        assert!(point_in_polygon((0.5, 0.0).into(), &loop_a).unwrap());
        assert!(point_in_polygon((0.0, 0.5).into(), &loop_a).unwrap());
        assert!(!point_in_polygon((1.1, 0.0).into(), &loop_a).unwrap());
    }
}
