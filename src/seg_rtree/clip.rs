use crate::geometries::SegmentPath;
use crate::seg_rtree::min_heap::MinHeap;
use crate::seg_rtree::segment_union::SegmentUnion;
use crate::seg_rtree::SegRTree;
use crate::{Coordinate, Rectangle};

type Heap = MinHeap<(usize, usize)>;

struct Clipper<'a> {
    clip_rect: Rectangle,
    coords: &'a [Coordinate],
    rtree: &'a SegRTree,
    output: Vec<Vec<Coordinate>>,
    last_index: Option<usize>,
}

impl<'a> Clipper<'a> {
    pub fn new(clip_rect: Rectangle, path: &'a SegmentPath) -> Self {
        Clipper {
            clip_rect,
            coords: path.coords(),
            rtree: path.rtree(),
            output: Vec::new(),
            last_index: None,
        }
    }

    pub fn clip(mut self) -> Vec<Vec<Coordinate>> {
        let (contained, intersects) = self.find_relevant_segments();
        self.build_output(contained, intersects);
        self.output
    }

    fn find_relevant_segments(&mut self) -> (SegmentUnion, Heap) {
        let mut contained = SegmentUnion::new();
        let mut intersects = Heap::new();
        let degree = self.rtree.degree();

        let mut stack = vec![self.rtree.root()];
        while let Some((level, offset)) = stack.pop() {
            let rect = self.rtree.get_rectangle(level, offset);
            if !self.clip_rect.intersects(rect) {
                continue;
            }
            let (low, high) = self.rtree.get_low_high(level, offset);
            if self.clip_rect.contains_rect(rect) {
                contained.add(low, high);
            } else if level == 0 {
                intersects.push((low, high));
            } else {
                let child_level = level - 1;
                let first_child_offset = degree * offset;
                for child_offset in first_child_offset..(first_child_offset + degree) {
                    stack.push((child_level, child_offset));
                }
            }
        }

        (contained, intersects)
    }

    fn build_output(&mut self, mut contained: SegmentUnion, mut intersects: Heap) {
        // TODO: Pre-allocate a vector, and memcpy into it.
        let mut out_coords = Vec::<Coordinate>::new();
        while !(contained.is_empty() || intersects.is_empty()) {
            if contained.peek().unwrap() < intersects.peek().unwrap().0 {
                self.push_contained(&mut contained, &mut out_coords);
            } else {
                self.push_intersects(&mut intersects, &mut out_coords);
            }
        }

        while !(contained.is_empty()) {
            self.push_contained(&mut contained, &mut out_coords);
        }

        while !(intersects.is_empty()) {
            self.push_intersects(&mut intersects, &mut out_coords);
        }

        self.flush_output(&mut out_coords);
    }

    fn push_contained(&mut self, contained: &mut SegmentUnion, out_coords: &mut Vec<Coordinate>) {
        let (mut low, high) = contained.pop().unwrap();
        if Some(low) == self.last_index {
            low += 1;
        } else {
            self.flush_output(out_coords);
        }
        out_coords.extend(&self.coords[low..=high]);
        self.last_index = Some(high);
    }

    fn push_intersects(&mut self, intersects: &mut Heap, out_coords: &mut Vec<Coordinate>) {
        let (low, high) = intersects.pop().unwrap();
        let seg_start = self.coords[low];
        let seg_end = self.coords[high];
        if let Some((isxn_start, isxn_end)) = self.clip_rect.intersect_segment(seg_start, seg_end) {
            if Some(low) != self.last_index {
                self.flush_output(out_coords);
                out_coords.push(isxn_start);
            }
            if isxn_end != isxn_start {
                out_coords.push(isxn_end);
            }
            if isxn_end == seg_end {
                self.last_index = Some(high);
            }
        }
    }

    fn flush_output(&mut self, out_coords: &mut Vec<Coordinate>) {
        if !out_coords.is_empty() {
            self.output.push(out_coords.clone());
            out_coords.clear();
        }
    }
}

/// Clip a path by intersecting with a rectangle
pub fn clip_path(clip_rect: Rectangle, path: &SegmentPath) -> Vec<Vec<Coordinate>> {
    let clipper = Clipper::new(clip_rect, path);
    clipper.clip()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floats_to_coords(input: Vec<(f64, f64)>) -> Vec<Coordinate> {
        input.into_iter().map(|c| c.into()).collect()
    }

    fn assert_clip(rect: Rectangle, input: Vec<(f64, f64)>, output: Vec<Vec<(f64, f64)>>) {
        let input = floats_to_coords(input);
        let output: Vec<Vec<Coordinate>> = output.into_iter().map(floats_to_coords).collect();
        assert_eq!(clip_path(rect, &SegmentPath::new(input)), output);
    }

    #[test]
    fn test_basic_linestring_clips() {
        // Test 1-segment clips
        let rect = Rectangle::new((0., 0.).into(), (1., 1.).into());
        // Completely contained
        assert_clip(
            rect,
            vec![(0., 0.), (1., 1.)],
            vec![vec![(0., 0.), (1., 1.)]],
        );
        assert_clip(
            rect,
            vec![(0.1, 0.7), (0.5, 0.2)],
            vec![vec![(0.1, 0.7), (0.5, 0.2)]],
        );
        // outside to in
        assert_clip(
            rect,
            vec![(-1.0, 0.5), (0.5, 0.5)],
            vec![vec![(0., 0.5), (0.5, 0.5)]],
        );
        assert_clip(rect, vec![(-1.0, 0.5), (0.0, 0.5)], vec![vec![(0.0, 0.5)]]);
        // inside to out
        assert_clip(
            rect,
            vec![(0.5, 0.5), (1.5, 0.5)],
            vec![vec![(0.5, 0.5), (1.0, 0.5)]],
        );
        assert_clip(rect, vec![(1.0, 0.5), (1.5, 0.5)], vec![vec![(1.0, 0.5)]]);
        // start, end outside
        assert_clip(rect, vec![(-1.5, 0.), (1., 2.)], vec![]);
        assert_clip(rect, vec![(-1., 0.), (1., 2.)], vec![vec![(0., 1.)]]);
        assert_clip(
            rect,
            vec![(-1., -1.), (1., 1.)],
            vec![vec![(0., 0.), (1., 1.)]],
        );
    }

    #[test]
    fn test_small_linestring_clips() {
        // Test 1-segment clips
        let rect = Rectangle::new((0., 0.).into(), (1., 1.).into());
        assert_clip(
            rect,
            vec![(-1., 0.25), (0.25, 0.25), (0.5, 0.75), (0.5, 2.0)],
            vec![vec![(0., 0.25), (0.25, 0.25), (0.5, 0.75), (0.5, 1.0)]],
        );
        assert_clip(
            rect,
            vec![(-0.25, 0.5), (0.5, 1.25), (1.25, 0.5)],
            vec![vec![(0., 0.75), (0.25, 1.)], vec![(0.75, 1.0), (1.0, 0.75)]],
        );
    }

    #[allow(dead_code)]
    fn test_numerical_precision() {
        let rect = Rectangle::new((0., 0.).into(), (1., 1.).into());
        assert_clip(
            rect,
            vec![(-1., 0.2), (0.2, 0.2), (0.4, 0.8), (0.4, 2.0)],
            vec![vec![(0., 0.2), (0.2, 0.2), (0.4, 0.8), (0.4, 1.0)]],
        );
    }
}
