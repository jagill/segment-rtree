use crate::utils::{calculate_level_indices, winding_number};
use crate::{Coordinate, Rectangle};

#[allow(dead_code)]
#[derive(Debug)]
pub struct SegRTree {
    degree: usize,
    max_size: usize,
    current_size: usize,
    current_level: usize,
    level_indices: Vec<usize>,
    tree: Vec<Rectangle>,
}

#[allow(dead_code)]
impl SegRTree {
    pub(crate) fn len(&self) -> usize {
        self.current_size
    }

    pub fn height(&self) -> usize {
        self.current_level
    }
    pub fn degree(&self) -> usize {
        self.degree
    }

    pub fn new_empty() -> Self {
        SegRTree {
            degree: 2,
            max_size: 0,
            current_size: 0,
            current_level: 0,
            level_indices: vec![0],
            tree: vec![Rectangle::new_empty()],
        }
    }

    pub fn new(mut degree: usize, max_size: usize) -> Self {
        degree = degree.max(2);
        let level_indices = calculate_level_indices(degree, max_size);
        let tree_size = level_indices[level_indices.len() - 1] + 1;
        let empty_rect = Rectangle::new_empty();
        SegRTree {
            degree,
            max_size,
            current_size: 0,
            current_level: 0,
            level_indices,
            tree: vec![empty_rect; tree_size],
        }
    }

    pub fn new_loaded(mut degree: usize, rects: &[Rectangle]) -> Self {
        degree = degree.max(2);
        let max_size = rects.len();
        let level_indices = calculate_level_indices(degree, max_size);
        let tree_size = level_indices[level_indices.len() - 1] + 1;
        let empty_rect = Rectangle::new_empty();
        let mut tree = vec![empty_rect; tree_size];
        copy_into_tree(&mut tree, 0, rects);

        for level in 1..level_indices.len() {
            let level_index = level_indices[level];
            let previous_items = &tree[level_indices[level - 1]..level_index];
            let next_items: Vec<Rectangle> = previous_items
                .chunks(degree)
                .map(|items| Rectangle::of(items))
                .collect();
            copy_into_tree(&mut tree, level_index, &next_items);
        }

        tree.shrink_to_fit();
        SegRTree {
            degree,
            max_size,
            current_size: max_size,
            current_level: level_indices.len() - 1,
            level_indices,
            tree,
        }
    }

    pub fn envelope(&self) -> Rectangle {
        self.get_rectangle(self.height() - 1, 0)
    }

    pub fn add(&mut self, mut rect: Rectangle) -> Result<(), String> {
        if self.current_size >= self.max_size {
            return Err("Exceeded capacity".to_owned());
        }

        let mut level = 0;
        let mut offset = self.current_size;
        loop {
            let index = self.level_indices[level] + offset;
            rect.expand(self.tree[index]);
            self.tree[index] = rect;
            if offset == 0 {
                break;
            } else if offset == 1 {
                // The parent needs the other child
                rect.expand(self.tree[index - 1]);
            }
            offset /= self.degree;
            level += 1;
        }

        self.current_level = level;
        self.current_size += 1;
        Ok(())
    }

    pub fn query_rect(&self, rect: Rectangle) -> Vec<usize> {
        self.query(|level, offset| self.get_rectangle(level, offset).intersects(rect))
    }

    pub fn query_point(&self, point: Coordinate) -> Vec<usize> {
        self.query(|level, offset| self.get_rectangle(level, offset).contains(point))
    }

    fn query<P>(&self, predicate: P) -> Vec<usize>
    where
        P: Fn(usize, usize) -> bool,
    {
        let mut results = Vec::new();
        if self.current_size == 0 {
            return results;
        }

        // Stack entries: (level, offset)
        let mut stack = vec![(self.height(), 0)];
        while let Some((level, offset)) = stack.pop() {
            if !predicate(level, offset) {
                continue;
            }
            if level == 0 {
                results.push(offset);
            } else {
                let child_level = level - 1;
                let first_child_offset = self.degree * offset;
                for child_offset in first_child_offset..(first_child_offset + self.degree) {
                    stack.push((child_level, child_offset));
                }
            }
        }

        results
    }

    pub fn query_self_intersections(&self) -> Vec<(usize, usize)> {
        let mut results = Vec::new();
        if self.current_size == 0 {
            return results;
        }

        // Stack entries: (level, offset)
        let mut stack = vec![(self.height(), 0, self.height(), 0)];

        while let Some((level_a, offset_a, level_b, offset_b)) = stack.pop() {
            let rect_a = self.get_rectangle(level_a, offset_a);
            let rect_b = self.get_rectangle(level_b, offset_b);
            if !rect_a.intersects(rect_b) {
                continue;
            }

            if level_a == 0 && level_b == 0 {
                if offset_a < offset_b {
                    results.push((offset_a, offset_b));
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

    pub fn check_containment(
        &self,
        point: Coordinate,
        coords: &[Coordinate],
    ) -> Result<bool, String> {
        if coords.len() - 1 != self.current_size {
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
        let mut stack = vec![(self.current_level, 0)];
        while let Some((level, offset)) = stack.pop() {
            let rect = self.get_rectangle(level, offset);
            if rect.x_min > point.x {
                let (low, high) = self.get_low_high(level, offset);
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
                let first_child_offset = self.degree * offset;
                for child_offset in first_child_offset..(first_child_offset + self.degree) {
                    stack.push((child_level, child_offset));
                }
            }
        }

        Ok(wn != 0)
    }

    pub(crate) fn get_rectangle(&self, level: usize, offset: usize) -> Rectangle {
        self.tree[self.level_indices[level] + offset]
    }

    pub(crate) fn get_low_high(&self, level: usize, offset: usize) -> (usize, usize) {
        let width = self.degree.pow(level as u32);
        (width * offset, width * (offset + 1))
    }

    pub(crate) fn root(&self) -> (usize, usize) {
        (self.height(), 0)
    }
}

fn copy_into_tree(tree: &mut [Rectangle], index: usize, rects: &[Rectangle]) {
    let (_, subtree) = tree.split_at_mut(index);
    let (subtree, _) = subtree.split_at_mut(rects.len());
    subtree.copy_from_slice(rects);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_seg_rtree() {
        let p1 = Coordinate { x: 0., y: 0. };
        let r = Rectangle {
            x_min: -10.,
            y_min: -5.,
            x_max: 1.,
            y_max: 5.,
        };
        let mut tree = SegRTree::new(2, 0);
        assert_eq!(tree.current_size, 0);
        assert_eq!(tree.current_level, 0);
        assert_eq!(tree.query_point(p1), Vec::new());
        assert_eq!(tree.query_rect(r), Vec::new());
        assert!(tree.add(r).is_err());
    }

    fn _assert_add(i: usize, tree: &mut SegRTree, rects: &[Rectangle]) {
        tree.add(rects[i]).unwrap();
        assert_eq!(tree.current_size, i + 1);
        assert_eq!(tree.tree[i], rects[i]);
        assert_eq!(tree.query_rect(rects[i]), vec![i]);
    }

    fn _assert_queries(max_index: usize, tree: &SegRTree, rects: &[Rectangle]) {
        #[allow(clippy::needless_range_loop)]
        for i in 0..=max_index {
            assert_eq!(tree.query_rect(rects[i]), vec![i]);
        }
    }

    #[test]
    fn test_build_seg_rtree() {
        let mut tree = SegRTree::new(2, 6);
        assert_eq!(tree.current_size, 0);
        assert_eq!(tree.current_level, 0);
        let rects: Vec<Rectangle> = (0..6)
            .map(|i| Rectangle {
                x_min: i as f64,
                y_min: i as f64,
                x_max: i as f64,
                y_max: i as f64,
            })
            .collect();

        _assert_add(0, &mut tree, &rects);
        assert_eq!(tree.current_level, 0);
        _assert_queries(0, &tree, &rects);

        _assert_add(1, &mut tree, &rects);
        assert_eq!(tree.current_level, 1);
        _assert_queries(1, &tree, &rects);

        _assert_add(2, &mut tree, &rects);
        assert_eq!(tree.current_level, 2);
        _assert_queries(2, &tree, &rects);

        _assert_add(3, &mut tree, &rects);
        assert_eq!(tree.current_level, 2);
        _assert_queries(3, &tree, &rects);

        _assert_add(4, &mut tree, &rects);
        assert_eq!(tree.current_level, 3);
        _assert_queries(4, &tree, &rects);

        _assert_add(5, &mut tree, &rects);
        assert_eq!(tree.current_level, 3);
        _assert_queries(5, &tree, &rects);

        let rect = Rectangle {
            x_min: 0.,
            y_min: 0.,
            x_max: 5.,
            y_max: 5.,
        };
        let mut results = tree.query_rect(rect);
        results.sort_unstable();
        assert_eq!(results, vec![0, 1, 2, 3, 4, 5]);

        let rect = Rectangle {
            x_min: 1.,
            y_min: 1.,
            x_max: 3.,
            y_max: 3.,
        };
        let mut results = tree.query_rect(rect);
        results.sort_unstable();
        assert_eq!(results, vec![1, 2, 3]);
    }
}
