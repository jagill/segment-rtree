use crate::{Coordinate, Rectangle};

pub(crate) fn rectangles_from_coordinates(coords: &[Coordinate]) -> Vec<Rectangle> {
    coords
        .windows(2)
        .map(|c| Rectangle::new(c[0], c[1]))
        .collect()
}

pub(crate) fn calculate_level_indices(degree: usize, num_items: usize) -> Vec<usize> {
    let mut level_indices: Vec<usize> = vec![0];

    let mut level = 0;
    let mut level_size = num_items;

    while level_size > 1 {
        let level_buffer = if level_size % degree > 0 { 1 } else { 0 };
        // least multiple of degree >= level_size
        let level_capacity = degree * (level_size / degree + level_buffer);
        level_indices.push(level_indices[level] + level_capacity);
        level += 1;
        level_size = level_capacity / degree;
        assert_eq!(level_indices.len(), level + 1);
    }
    level_indices
}

pub(crate) fn winding_number(point: Coordinate, start: Coordinate, end: Coordinate) -> i32 {
    // Calculate the two halves of the cross-product (= lx - rx)
    let lx = (end.x - start.x) * (point.y - start.y);
    let rx = (end.y - start.y) * (point.x - start.x);

    if start.y <= point.y {
        // Upward crossing
        if end.y > point.y && lx > rx {
            return 1;
        }
    } else {
        // Downward crossing
        if end.y <= point.y && lx < rx {
            return -1;
        }
    }
    0
}

/**
 * Check the intersection of two segments A and B.
 *
 * NB: This does not do an initial check with Envelopes; the caller should do that.
 */
pub(crate) fn intersect_segments(
    start_a: Coordinate,
    end_a: Coordinate,
    start_b: Coordinate,
    end_b: Coordinate,
) -> Option<(Coordinate, Coordinate)> {
    if (start_a == start_b && end_a == end_b) || (start_a == end_b && end_a == start_b) {
        return Some((start_a, end_a));
    }

    let da = end_a - start_a; // The vector for segment A
    let db = end_b - start_b; // The vector for segment B
    let offset = start_b - start_a; // The offset between segments (starts)

    let da_x_db = da.cross(db);
    let offset_x_da = offset.cross(da);

    if da_x_db == 0. {
        // This means the two segments are parallel.
        // If the offset is not also parallel, they must be disjoint.
        if offset_x_da != 0. {
            return None;
        } else {
            // If the offset is also parallel, check for overlap.
            let da_2 = da.dot(da);
            // Offset, in units of da.
            let t0 = offset.dot(da) / da_2;
            // start_a to end_b, in units of da.
            let t1 = t0 + da.dot(db) / da_2;
            let t_min = t0.min(t1);
            let t_max = t0.max(t1);
            if t_min > 1. || t_max < 0. {
                // if min(t0, t1) > 1 or max(t0, t1) < 0, they don't intersect.
                return None;
            } else {
                // Else, the intersect
                let start = start_a + da * t_min.max(0.);
                let end = start_a + da * t_max.min(1.);
                return Some((start, end));
            }
        }
    } else {
        // The segments are not parallel, so they are disjoint or intersect at a point
        // Calculate where the infinite lines would intersect; if these are on the segments
        // then the segments intersect.
        let ta = offset.cross(db) / da_x_db;
        let tb = offset_x_da / da_x_db;
        if 0. <= ta && ta <= 1. && 0. <= tb && tb <= 1. {
            let intersection = start_a + da * ta;
            return Some((intersection, intersection));
        }
    }
    None
}
