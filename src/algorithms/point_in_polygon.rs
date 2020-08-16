use crate::geometry_state::{HasRTree, Validated};
use crate::utils::{winding_number, WindingPosition};
use crate::{Coordinate, HasEnvelope, LinearRing, Polygon, Rectangle};

#[derive(Debug, PartialEq, Eq)]
pub enum ContainRelation {
    Exterior,
    Boundary,
    Interior,
}

pub fn point_in_polygon(point: Coordinate, polygon: &Polygon<Validated>) -> ContainRelation {
    let shell_relation = point_in_loop(point, polygon.shell());
    if shell_relation == ContainRelation::Interior {
        for hole in polygon.holes() {
            match point_in_loop(point, hole) {
                ContainRelation::Interior => return ContainRelation::Exterior,
                ContainRelation::Boundary => return ContainRelation::Boundary,
                ContainRelation::Exterior => (),
            }
        }
    }
    shell_relation
}

pub fn point_in_loop(point: Coordinate, path: &LinearRing<Validated>) -> ContainRelation {
    if !path.envelope().contains(point) {
        return ContainRelation::Exterior;
    }
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
            match winding_number(point, coords[low], coords[high]) {
                WindingPosition::On => unreachable!(),
                WindingPosition::Left => wn += 1,
                WindingPosition::Right => wn -= 1,
                WindingPosition::Off => (),
            }
            continue;
        }
        if level == 0 {
            match winding_number(point, coords[offset], coords[offset + 1]) {
                WindingPosition::On => return ContainRelation::Boundary,
                WindingPosition::Left => wn += 1,
                WindingPosition::Right => wn -= 1,
                WindingPosition::Off => (),
            }
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
    if wn != 0 {
        ContainRelation::Interior
    } else {
        ContainRelation::Exterior
    }
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
    fn check_containment_loop() {
        let loop_a =
            LinearRing::try_from(vec![(0., 0.), (0., 1.), (1., 1.), (1., 0.), (0., 0.)]).unwrap();
        assert_eq!(
            point_in_loop((0.5, 0.5).into(), &loop_a),
            ContainRelation::Interior
        );
        assert_eq!(
            point_in_loop((0.0, 0.0).into(), &loop_a),
            ContainRelation::Boundary
        );
        assert_eq!(
            point_in_loop((0.5, 0.0).into(), &loop_a),
            ContainRelation::Boundary
        );
        assert_eq!(
            point_in_loop((0.0, 0.5).into(), &loop_a),
            ContainRelation::Boundary
        );
        assert_eq!(
            point_in_loop((1.1, 0.0).into(), &loop_a),
            ContainRelation::Exterior
        );
    }
    #[test]
    fn check_containment_polygon() {
        let shell =
            LinearRing::try_from(vec![(0., 0.), (0., 10.), (10., 10.), (10., 0.), (0., 0.)])
                .unwrap();
        let holes = vec![
            LinearRing::try_from(vec![(1., 1.), (1., 8.), (3., 8.), (3., 1.), (1., 1.)]).unwrap(),
            LinearRing::try_from(vec![(6., 6.), (6., 9.), (9., 10.), (9., 6.), (6., 6.)]).unwrap(),
        ];
        let polygon = Polygon::try_new(shell, holes).unwrap();
        assert_eq!(
            point_in_polygon((0.5, 0.5).into(), &polygon),
            ContainRelation::Interior
        );
        assert_eq!(
            point_in_polygon((0.0, 0.0).into(), &polygon),
            ContainRelation::Boundary
        );
        assert_eq!(
            point_in_polygon((1.0, 1.0).into(), &polygon),
            ContainRelation::Boundary
        );
        assert_eq!(
            point_in_polygon((1.5, 1.5).into(), &polygon),
            ContainRelation::Exterior
        );
        assert_eq!(
            point_in_polygon((9.0, 10.).into(), &polygon),
            ContainRelation::Boundary
        );
        assert_eq!(
            point_in_polygon((10.1, 0.0).into(), &polygon),
            ContainRelation::Exterior
        );
    }
}
