use super::point_in_polygon::point_in_polygon;
use crate::errors::ValidationError;
use crate::errors::ValidationError::*;
use crate::utils::intersect_segments;
use crate::{Coordinate, Polygon, LineString};
use std::collections::HashMap;

type IntersectionMap = HashMap<(usize, usize), Coordinate>;

/// Validate a polygon, on the assumption its component paths are valid.
pub fn validate_polygon(polygon: &Polygon) -> Result<(), ValidationError> {
    let shell = polygon.shell();
    if shell.coords().first() != shell.coords().last() {
        return Err(NotARing);
    }
    let holes = polygon.holes();
    let mut intersection_matrix: IntersectionMap = HashMap::new();
    for (i, hole) in holes.iter().enumerate() {
        if hole.coords().first() != hole.coords().last() {
            return Err(NotARing);
        }
        if shell.envelope() == hole.envelope() || !shell.envelope().contains_rect(hole.envelope()) {
            return Err(HoleNotValid);
        }

        let intersection = find_intersecting_point(hole, shell)?;
        if let Some(p) = intersection {
            intersection_matrix.insert((0, i + 1), p);
        }

        if !point_in_polygon(find_nonequal_point(hole.coords(), intersection), shell)? {
            return Err(HoleNotValid);
        }

        // Check existing holes for intersections.
        for (j, other_hole) in holes[0..i].iter().enumerate() {
            if !hole.envelope().intersects(other_hole.envelope()) {
                continue;
            }
            let intersection = find_intersecting_point(hole, other_hole)?;
            if let Some(p) = intersection {
                intersection_matrix.insert((i + 1, j + 1), p);
            }
            // Check that each hole is not in the other
            if point_in_polygon(find_nonequal_point(hole.coords(), intersection), other_hole)? {
                return Err(HoleNotValid);
            }
            if point_in_polygon(find_nonequal_point(other_hole.coords(), intersection), hole)? {
                return Err(HoleNotValid);
            }
        }
    }

    // TODO: Make sure interior is connected, using intersection matrix.

    Ok(())
}

/// Find 0 or 1 intersecting points.  If there are 2 or more points, the
/// intersection is invalid, and return a ValidationError.
fn find_intersecting_point(
    ring_a: &LineString,
    ring_b: &LineString,
) -> Result<Option<Coordinate>, ValidationError> {
    let mut final_intersection = None;
    for (index_a, index_b) in ring_a.rtree().query_other_intersections(ring_b.rtree()) {
        let start_a = ring_a.coords()[index_a];
        let end_a = ring_a.coords()[index_a + 1];
        let start_b = ring_b.coords()[index_b];
        let end_b = ring_b.coords()[index_b + 1];

        let seg_intersection = intersect_segments(start_a, end_a, start_b, end_b);
        if seg_intersection.is_none() {
            continue;
        }
        let (isxn_start, isxn_end) = seg_intersection.unwrap();
        if isxn_start != isxn_end {
            return Err(OverlappingSegments {
                first_index: index_a,
                second_index: index_b,
                start: isxn_start,
                end: isxn_end,
            });
        }
        match final_intersection {
            None => final_intersection = Some(isxn_start),
            Some(_) => return Err(MultipleIntersections),
        }
    }
    Ok(final_intersection)
}

/// Find a point in coords that is not the needle.  We are only using this for
/// rings which have >=3 non-equal points, so this will always find a match.
fn find_nonequal_point(coords: &[Coordinate], needle: Option<Coordinate>) -> Coordinate {
    assert!(coords.len() > 3);
    for &coord in coords {
        if needle != Some(coord) {
            return coord;
        }
    }
    unreachable!();
}
