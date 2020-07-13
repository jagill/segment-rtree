/**
 * A fast, low memory footprint static Rtree.
 *
 * Original implementation in Javascript: https://github.com/mourner/flatbush
 * Initial conversion to rust by Jacob Wasserman @jwass
 */
use super::hilbert::Hilbert;
use crate::utils::calculate_level_indices;
use crate::{Coordinate, Rectangle};

pub const FLATBUSH_DEFAULT_DEGREE: usize = 16;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Flatbush {
    pub degree: usize,
    // nodes in level i are (level_indices[i] .. level_indices[i + 1] - 1)
    level_indices: Vec<usize>,
    tree: Vec<Rectangle>,
    node_indices: Vec<usize>,
}

#[allow(dead_code)]
impl Flatbush {
    pub fn new_empty() -> Flatbush {
        Flatbush {
            degree: FLATBUSH_DEFAULT_DEGREE,
            level_indices: vec![0],
            tree: vec![Rectangle::new_empty()],
            node_indices: vec![0],
        }
    }

    pub fn new(degree: usize, items: &[Rectangle]) -> Flatbush {
        let total_envelope = Rectangle::of(items);
        let hilbert_square: Hilbert;
        if total_envelope.is_empty() {
            // The list of items are empty, or all items are empty.
            return Flatbush::new_unsorted(degree, items);
        } else {
            hilbert_square = Hilbert::new(total_envelope);
        }

        let mut entries: Vec<(u32, usize, Rectangle)> = items
            .iter()
            .copied()
            .enumerate()
            .map(|(i, e)| (hilbert_square.hilbert(e.center()), i, e))
            .collect();

        entries.sort_unstable_by_key(|&(h, _, _)| h);

        Flatbush::_new_unsorted(
            degree,
            entries.into_iter().map(|(_, i, e)| (i, e)).collect(),
        )
    }

    pub fn new_unsorted(degree: usize, items: &[Rectangle]) -> Flatbush {
        let entries = items.iter().copied().enumerate().collect();
        Flatbush::_new_unsorted(degree, entries)
    }

    fn _new_unsorted(degree: usize, entries: Vec<(usize, Rectangle)>) -> Flatbush {
        if entries.is_empty() {
            return Flatbush::new_empty();
        }
        let degree = degree.max(2);
        let level_indices = calculate_level_indices(degree, entries.len());
        let tree_size = level_indices[level_indices.len() - 1] + 1;

        let node_indices: Vec<usize> = entries.iter().map(|(i, _e)| i).copied().collect();
        let mut tree: Vec<Rectangle> = Vec::with_capacity(tree_size);
        tree.extend(entries.iter().map(|(_i, e)| e));

        tree.extend(vec![Rectangle::new_empty(); level_indices[1] - tree.len()]);

        for level in 1..level_indices.len() {
            let level_index = level_indices[level];
            tree.extend(vec![Rectangle::new_empty(); level_index - tree.len()]);
            assert_eq!(tree.len(), level_index);

            let level_items = &tree[level_indices[level - 1]..level_indices[level]];
            let next_items: Vec<Rectangle> = level_items
                .chunks(degree)
                .map(|items| Rectangle::of(items))
                .collect();
            tree.extend(next_items);
        }

        tree.shrink_to_fit();

        Flatbush {
            degree,
            level_indices,
            tree,
            node_indices,
        }
    }

    fn height(&self) -> usize {
        self.level_indices.len() - 1
    }

    fn get_rectangle(&self, level: usize, offset: usize) -> Rectangle {
        self.tree[self.level_indices[level] + offset]
    }

    fn envelope(&self) -> Rectangle {
        self.get_rectangle(self.height(), 0)
    }

    /**
     * Find geometries that might intersect the query_rect.
     *
     * This only checks bounding-box intersection, so the candidates must be
     * checked by the caller.
     */
    pub fn query_rect(&self, query: Rectangle) -> Vec<usize> {
        let mut results = Vec::new();
        let mut stack: Vec<(usize, usize)> = vec![(self.height(), 0)];

        // The todo_list will keep a LIFO stack of nodes to be processed.
        // The invariant is that everything in todo_list (envelope) intersects
        // query_rect, and is level > 0 (leaves are yielded).
        while let Some((level, offset)) = stack.pop() {
            let rect = self.get_rectangle(level, offset);
            if !query.intersects(rect) {
                continue;
            }
            if level == 0 {
                results.push(self.node_indices[offset]);
            } else {
                let child_level = level - 1;
                let first_child_offset = self.degree * offset;
                let last_child_offset = first_child_offset + self.degree;
                for child_offset in first_child_offset..last_child_offset {
                    stack.push((child_level, child_offset));
                }
            }
        }

        results
    }

    /**
     * Find geometries that might be within `distance` of `position`.
     *
     * This only checks bounding-box distance, so the candidates must be
     * checked by the caller.
     */
    pub fn query_within(&self, position: Coordinate, distance: f64) -> Vec<usize> {
        let delta = Coordinate::new(distance, distance);
        self.query_rect(Rectangle::new(position - delta, position + delta))
    }

    /**
     * Find all distinct elements of the Rtree that might intersect each other.
     *
     * This will only return each candidate pair once; the element with the
     * smaller index will be the first element of the pair.  It will not return
     * the degenerate pair of two of the same elements.
     *
     * This only checks bounding-box intersection, so the candidates must be
     * checked by the caller.
     */
    pub fn query_self_intersections(&self) -> Vec<(usize, usize)> {
        let mut results = Vec::new();

        // The stack will keep pairs of nodes to be processed.
        // The invariants for the todo_list are:
        // * level_a <= level_b
        // * if level_a == level_b, offset_a <= offset_b (to handle duplicates)
        // * if level_a == 0 == level_b, offset_a < offset_b (to handle duplicates)
        let mut stack: Vec<(usize, usize, usize, usize)> =
            vec![(self.height(), 0, self.height(), 0)];

        while let Some((level_a, offset_a, level_b, offset_b)) = stack.pop() {
            let rect_a = self.get_rectangle(level_a, offset_a);
            let rect_b = self.get_rectangle(level_b, offset_b);
            if !rect_a.intersects(rect_b) {
                continue;
            }

            if level_a == 0 && level_b == 0 {
                let index_a = self.node_indices[offset_a];
                let index_b = self.node_indices[offset_b];
                if index_a < index_b {
                    results.push((index_a, index_b));
                }
            } else if level_a == level_b {
                let child_level = level_a - 1;
                let first_child_offset = self.degree * offset_a;
                for child_offset in first_child_offset..(first_child_offset + self.degree) {
                    stack.push((child_level, child_offset, level_b, offset_b));
                }
            } else {
                assert_eq!(level_a + 1, level_b);
                let child_level = level_b - 1;
                let first_child_offset = self.degree * offset_b;
                let last_child_offset = first_child_offset + self.degree;
                for child_offset in first_child_offset..last_child_offset {
                    stack.push((level_a, offset_a, child_level, child_offset));
                }
            }
        }

        results
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FlatbushNode {
    // Level in tree, 0 is leaf, max is root.
    pub level: usize,
    // The index within the tree
    pub tree_index: usize,
    // Index of node in a level
    pub sibling_index: usize,
    pub envelope: Rectangle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tree() {
        let empty = Flatbush::new_empty();
        let query_rect = Rectangle::new((0., 0.).into(), (1., 1.).into());
        assert_eq!(empty.query_rect(query_rect), Vec::<usize>::new());
        assert_eq!(empty.query_self_intersections(), Vec::new());
    }

    #[test]
    fn test_build_tree_unsorted() {
        let degree = 4;
        let e0 = Rectangle::new((7.0, 44.).into(), (8., 48.).into());
        let e1 = Rectangle::new((25., 48.).into(), (35., 55.).into());
        let e2 = Rectangle::new((98., 46.).into(), (99., 56.).into());
        let e3 = Rectangle::new((58., 65.).into(), (73., 79.).into());
        let e4 = Rectangle::new((43., 40.).into(), (44., 45.).into());
        let e5 = Rectangle::new((97., 87.).into(), (100., 91.).into());
        let e6 = Rectangle::new((92., 46.).into(), (108., 57.).into());
        let e7 = Rectangle::new((7.1, 48.).into(), (10., 56.).into());
        let envs = vec![e0, e1, e2, e3, e4, e5, e6, e7];

        let flatbush = Flatbush::new_unsorted(degree, &envs);

        // This is unsorted, so the order should be:
        // [e0..e3, e4..e7, p1=parent(e0..e3), p2=parent(e4..e7), root=parent(p1, p2)]

        assert_eq!(flatbush.degree, degree);
        assert_eq!(flatbush.level_indices, vec![0, 8, 12]);
        assert_eq!(flatbush.tree[0..8], envs[..]);
        assert_eq!(
            flatbush.tree[8..12],
            vec![
                Rectangle::new((7.0, 44.).into(), (99., 79.).into()),
                Rectangle::new((7.1, 40.).into(), (108., 91.).into()),
                Rectangle::new_empty(),
                Rectangle::new_empty(),
            ][..]
        );
        assert_eq!(
            flatbush.tree[12],
            Rectangle::new((7., 40.).into(), (108., 91.).into())
        );
    }

    fn get_envelopes() -> Vec<Rectangle> {
        #[rustfmt::skip]
        let rects: Vec<f64> = vec![
             8, 62, 11, 66,
            57, 17, 57, 19,
            76, 26, 79, 29,
            36, 56, 38, 56,
            92, 77, 96, 80,
            87, 70, 90, 74,
            43, 41, 47, 43,
             0, 58,  2, 62,
            76, 86, 80, 89,
            27, 13, 27, 15,
            71, 63, 75, 67,
            25,  2, 27,  2,
            87,  6, 88,  6,
            22, 90, 23, 93,
            22, 89, 22, 93,
            57, 11, 61, 13,
            61, 55, 63, 56,
            17, 85, 21, 87,
            33, 43, 37, 43,
             6,  1,  7,  3,
            80, 87, 80, 87,
            23, 50, 26, 52,
            58, 89, 58, 89,
            12, 30, 15, 34,
            32, 58, 36, 61,
            41, 84, 44, 87,
            44, 18, 44, 19,
            13, 63, 15, 67,
            52, 70, 54, 74,
            57, 59, 58, 59,
            17, 90, 20, 92,
            48, 53, 52, 56,
             2, 68, 92, 72,
            26, 52, 30, 52,
            56, 23, 57, 26,
            88, 48, 88, 48,
            66, 13, 67, 15,
             7, 82,  8, 86,
            46, 68, 50, 68,
            37, 33, 38, 36,
             6, 15,  8, 18,
            85, 36, 89, 38,
            82, 45, 84, 48,
            12,  2, 16,  3,
            26, 15, 26, 16,
            55, 23, 59, 26,
            76, 37, 79, 39,
            86, 74, 90, 77,
            16, 75, 18, 78,
            44, 18, 45, 21,
            52, 67, 54, 71,
            59, 78, 62, 78,
            24,  5, 24,  8,
            64, 80, 64, 83,
            66, 55, 70, 55,
             0, 17,  2, 19,
            15, 71, 18, 74,
            87, 57, 87, 59,
             6, 34,  7, 37,
            34, 30, 37, 32,
            51, 19, 53, 19,
            72, 51, 73, 55,
            29, 45, 30, 45,
            94, 94, 96, 95,
             7, 22, 11, 24,
            86, 45, 87, 48,
            33, 62, 34, 65,
            18, 10, 21, 14,
            64, 66, 67, 67,
            64, 25, 65, 28,
            27,  4, 31,  6,
            84,  4, 85,  5,
            48, 80, 50, 81,
             1, 61,  3, 61,
            71, 89, 74, 92,
            40, 42, 43, 43,
            27, 64, 28, 66,
            46, 26, 50, 26,
            53, 83, 57, 87,
            14, 75, 15, 79,
            31, 45, 34, 45,
            89, 84, 92, 88,
            84, 51, 85, 53,
            67, 87, 67, 89,
            39, 26, 43, 27,
            47, 61, 47, 63,
            23, 49, 25, 53,
            12,  3, 14,  5,
            16, 50, 19, 53,
            63, 80, 64, 84,
            22, 63, 22, 64,
            26, 66, 29, 66,
             2, 15,  3, 15,
            74, 77, 77, 79,
            64, 11, 68, 11,
            38,  4, 39,  8,
            83, 73, 87, 77,
            85, 52, 89, 56,
            74, 60, 76, 63,
            62, 66, 65, 67,
        ]
        .into_iter()
        .map(|v| v as f64)
        .collect();
        rects
            .chunks(4)
            .map(|r| Rectangle::new((r[0], r[1]).into(), (r[2], r[3]).into()))
            .collect()
    }

    #[test]
    fn test_intersection_candidates_unsorted() {
        let envelopes = get_envelopes();
        let f = Flatbush::new_unsorted(16, &envelopes);
        let query_rect = Rectangle::new((40., 40.).into(), (60., 60.).into());

        let brute_results = find_brute_intersections(query_rect, &envelopes);
        let mut rtree_results = f.query_rect(query_rect);
        rtree_results.sort();
        assert_eq!(rtree_results, brute_results);
    }

    #[test]
    fn test_intersection_candidates_hilbert() {
        let envelopes = get_envelopes();
        let f = Flatbush::new(16, &envelopes);
        let query_rect = Rectangle::new((40., 40.).into(), (60., 60.).into());

        let brute_results = find_brute_intersections(query_rect, &envelopes);
        let mut rtree_results = f.query_rect(query_rect);
        rtree_results.sort();
        assert_eq!(rtree_results, brute_results);
    }

    #[test]
    fn test_self_intersection_unsorted() {
        let envelopes: Vec<Rectangle> = get_envelopes();
        let f = Flatbush::new_unsorted(16, &envelopes);

        let brute_results = find_brute_self_intersections(&envelopes);
        let mut rtree_results = f.query_self_intersections();
        rtree_results.sort();
        assert_eq!(rtree_results, brute_results);
    }

    #[test]
    fn test_self_intersection_hilbert() {
        let envelopes: Vec<Rectangle> = get_envelopes();
        let f = Flatbush::new(16, &envelopes);

        let brute_results = find_brute_self_intersections(&envelopes);
        let mut rtree_results = f.query_self_intersections();
        rtree_results.sort();
        assert_eq!(rtree_results, brute_results);
    }

    // #[test]
    // fn test_rtree_intersection_unsorted() {
    //     let mut envelopes1 = get_envelopes();
    //     let n_envs = envelopes1.len();
    //     let envelopes2 = envelopes1.split_off(2 * envelopes1.len() / 3);
    //     assert_eq!(envelopes1.len() + envelopes2.len(), n_envs);

    //     let f1 = Flatbush::new_unsorted(&envelopes1, 16);
    //     let f2 = Flatbush::new_unsorted(&envelopes2, 16);
    //     let mut rtree_results = f1.find_other_rtree_intersection_candidates(&f2);
    //     rtree_results.sort();
    //     let brute_results = find_brute_cross_intersections(&envelopes1, &envelopes2);
    //     assert_eq!(rtree_results, brute_results);
    // }

    // #[test]
    // fn test_rtree_intersection_hilbert() {
    //     let mut envelopes1 = get_envelopes();
    //     let n_envs = envelopes1.len();
    //     let envelopes2 = envelopes1.split_off(2 * envelopes1.len() / 3);
    //     assert_eq!(envelopes1.len() + envelopes2.len(), n_envs);

    //     let f1 = Flatbush::new(&envelopes1, 16);
    //     let f2 = Flatbush::new(&envelopes2, 16);
    //     let mut rtree_results = f1.find_other_rtree_intersection_candidates(&f2);
    //     rtree_results.sort();
    //     let brute_results = find_brute_cross_intersections(&envelopes1, &envelopes2);
    //     assert_eq!(rtree_results, brute_results);
    // }

    // #[test]
    // fn test_rtree_intersection_with_empty() {
    //     let envelopes1 = get_envelopes();
    //     let f1 = Flatbush::new(&envelopes1, 16);
    //     let f2 = Flatbush::new_empty();
    //     let rtree_results = f1.find_other_rtree_intersection_candidates(&f2);
    //     assert_eq!(rtree_results, vec![]);
    // }

    fn find_brute_intersections(query_rect: Rectangle, envelopes: &[Rectangle]) -> Vec<usize> {
        envelopes
            .iter()
            .enumerate()
            .filter(|(_, e)| e.intersects(query_rect))
            .map(|(i, _)| i)
            .collect()
    }

    fn find_brute_self_intersections(envelopes: &[Rectangle]) -> Vec<(usize, usize)> {
        let mut results = Vec::new();
        for (i1, e1) in envelopes.iter().copied().enumerate() {
            for (i2, e2) in envelopes.iter().copied().enumerate() {
                if i1 >= i2 {
                    continue;
                }
                if !e1.intersects(e2) {
                    continue;
                }
                results.push((i1, i2))
            }
        }
        results
    }

    // fn find_brute_cross_intersections(
    //     envelopes1: &[Rectangle],
    //     envelopes2: &[Rectangle],
    // ) -> Vec<(usize, usize)> {
    //     type EnumEnv = (usize, Rectangle);
    //     let envelopes1: Vec<EnumEnv> = envelopes1.iter().copied().enumerate().collect();
    //     let envelopes2: Vec<EnumEnv> = envelopes2.iter().copied().enumerate().collect();
    //     iproduct!(envelopes1, envelopes2)
    //         .filter(|((_, e1), (_, e2))| e1.intersects(*e2))
    //         .map(|((i1, _), (i2, _))| (i1, i2))
    //         .collect()
    // }
}
