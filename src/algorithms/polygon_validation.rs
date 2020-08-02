use super::point_in_polygon::point_in_polygon;
use crate::errors::ValidationError;
use crate::errors::ValidationError::*;
use crate::geometry_state::{HasRTree, Validated};
use crate::utils::intersect_segments;
use crate::{Coordinate, HasEnvelope, LineString, Polygon};
use std::collections::{HashMap, HashSet};

type Intersections = HashSet<(usize, usize)>;

/// Validate a polygon, on the assumption its component paths are valid.
pub fn validate_polygon(polygon: &Polygon) -> Result<(), ValidationError> {
    let shell = polygon.shell();
    if shell.coords().first() != shell.coords().last() {
        return Err(NotARing);
    }
    let holes = polygon.holes();
    let mut intersections: Intersections = Intersections::new();
    for (i, hole) in holes.iter().enumerate() {
        if hole.coords().first() != hole.coords().last() {
            return Err(NotARing);
        }
        if shell.envelope() == hole.envelope() || !shell.envelope().contains(hole.envelope()) {
            return Err(HoleNotValid);
        }

        let intersection = find_intersecting_point(hole, shell)?;
        if intersection.is_some() {
            intersections.insert((0, i + 1));
        }

        if !point_in_polygon(find_nonequal_point(hole.coords(), intersection), shell)? {
            return Err(HoleNotValid);
        }

        // Check existing holes for intersections.
        // For large number of holes, might be faster to build an Rtree of the
        // holes and restrict candidates that way?
        for (j, other_hole) in holes[0..i].iter().enumerate() {
            if !hole.envelope().intersects(other_hole.envelope()) {
                continue;
            }
            let intersection = find_intersecting_point(hole, other_hole)?;
            if intersection.is_some() {
                intersections.insert((i + 1, j + 1));
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

    if has_cycle(&intersections) {
        Err(InteriorDisconnected)
    } else {
        Ok(())
    }
}

/// Find 0 or 1 intersecting points.  If there are 2 or more points, the
/// intersection is invalid, and return a ValidationError.
fn find_intersecting_point(
    ring_a: &LineString<Validated>,
    ring_b: &LineString<Validated>,
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

fn has_cycle(intersections: &Intersections) -> bool {
    if intersections.is_empty() {
        return false;
    }

    let mut edges: HashMap<usize, Vec<usize>> = HashMap::with_capacity(intersections.len() * 2);
    for (v1, v2) in intersections {
        edges.entry(*v1).or_default().push(*v2);
        edges.entry(*v2).or_default().push(*v1);
    }

    let mut seen: HashSet<usize> = HashSet::with_capacity(edges.len());
    // Vec<(node, parent)>
    let mut stack: Vec<(usize, usize)> = Vec::with_capacity(edges.len());

    for &base_node in edges.keys() {
        if seen.contains(&base_node) {
            continue;
        }
        stack.push((base_node, base_node));

        while let Some((node, parent)) = stack.pop() {
            seen.insert(node);
            for &next_node in &edges[&node] {
                if !seen.contains(&next_node) {
                    stack.push((next_node, node));
                } else if next_node != parent {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_cycle() {
        let mut map: Intersections = Intersections::new();
        assert!(!has_cycle(&map));
        map.insert((0, 1));
        assert!(!has_cycle(&map));
        map.insert((1, 2));
        assert!(!has_cycle(&map));
        map.insert((2, 3));
        assert!(!has_cycle(&map));
        map.insert((4, 5));
        assert!(!has_cycle(&map));
    }

    #[test]
    fn test_cycle() {
        let mut map: Intersections = Intersections::new();
        map.insert((0, 1));
        map.insert((1, 2));
        map.insert((2, 3));
        map.insert((0, 2));
        assert!(has_cycle(&map));
        map.insert((0, 3));
        assert!(has_cycle(&map));
        map.insert((1, 3));
        assert!(has_cycle(&map));
    }
}
