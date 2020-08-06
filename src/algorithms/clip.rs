use super::min_heap::MinHeap;
use crate::geometry_state::{HasRTree, Validated};
use crate::{Coordinate, LineString, Rectangle, SegRTree, SegmentUnion};

type Heap = MinHeap<(usize, usize)>;

struct SectionBuilder {
    coordinates: Vec<Coordinate>,
    indices: Vec<usize>,
}

impl SectionBuilder {
    pub fn with_capacity(capacity: usize) -> Self {
        SectionBuilder {
            coordinates: Vec::with_capacity(capacity),
            indices: Vec::with_capacity(16),
        }
    }

    pub fn push(&mut self, coord: Coordinate) {
        self.coordinates.push(coord);
    }

    pub fn extend(&mut self, coords: &[Coordinate]) {
        self.coordinates.extend_from_slice(coords);
    }

    pub fn flush(&mut self) {
        self.indices.push(self.coordinates.len());
    }

    /// Flush if there's unflushed coordinates
    fn maybe_flush(&mut self) {
        let num_coords = self.coordinates.len();
        if num_coords > 0 {
            match self.indices.last() {
                Some(i) if *i == num_coords => (),
                _ => self.flush(),
            }
        }
    }

    pub fn to_vec(mut self) -> Vec<Vec<Coordinate>> {
        self.maybe_flush();
        let mut results = Vec::with_capacity(self.indices.len());

        let mut remaining;
        for range in self.indices.windows(2) {
            remaining = self.coordinates.split_off(range[1] - range[0]);
            results.push(self.coordinates);
            self.coordinates = remaining;
        }
        results
    }
}

struct Clipper<'a> {
    clip_rect: Rectangle,
    coords: &'a [Coordinate],
    rtree: &'a SegRTree,
    last_index: Option<usize>,
}

impl<'a> Clipper<'a> {
    pub fn new(clip_rect: Rectangle, path: &'a LineString<Validated>) -> Self {
        Clipper {
            clip_rect,
            coords: path.coords(),
            rtree: path.rtree(),
            last_index: None,
        }
    }

    pub fn clip(mut self) -> Vec<Vec<Coordinate>> {
        let (contained, intersects) = self.find_relevant_segments();
        let mut output = self.build_output(contained, intersects).to_vec();
        self.reconnect_loop(&mut output);
        output
    }

    fn find_relevant_segments(&self) -> (SegmentUnion, Heap) {
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
            if self.clip_rect.contains(rect) {
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

    fn build_output(
        &mut self,
        mut contained: SegmentUnion,
        mut intersects: Heap,
    ) -> SectionBuilder {
        let mut sections = SectionBuilder::with_capacity(contained.len() + 2 * intersects.len());

        while !(contained.is_empty() || intersects.is_empty()) {
            if contained.peek().unwrap() < intersects.peek().unwrap().0 {
                self.push_contained(&mut contained, &mut sections);
            } else {
                self.push_intersects(&mut intersects, &mut sections);
            }
        }

        while !(contained.is_empty()) {
            self.push_contained(&mut contained, &mut sections);
        }

        while !(intersects.is_empty()) {
            self.push_intersects(&mut intersects, &mut sections);
        }

        sections.flush();
        sections
    }

    fn push_contained(&mut self, contained: &mut SegmentUnion, sections: &mut SectionBuilder) {
        let (mut low, high) = contained.pop().unwrap();
        if Some(low) == self.last_index {
            low += 1;
        } else {
            sections.flush();
        }
        sections.extend(&self.coords[low..=high]);
        self.last_index = Some(high);
    }

    fn push_intersects(&mut self, intersects: &mut Heap, sections: &mut SectionBuilder) {
        let (low, high) = intersects.pop().unwrap();
        let seg_start = self.coords[low];
        let seg_end = self.coords[high];
        if let Some((isxn_start, isxn_end)) = self.clip_rect.intersect_segment(seg_start, seg_end) {
            if Some(low) != self.last_index {
                sections.flush();
                sections.push(isxn_start);
            }
            if isxn_end != isxn_start {
                sections.push(isxn_end);
            }
            if isxn_end == seg_end {
                self.last_index = Some(high);
            }
        }
    }

    fn reconnect_loop(&self, output: &mut Vec<Vec<Coordinate>>) {
        // Check if we have a loop that starts and ends in the rectangle, but
        // was clipped into two pieces
        if output.len() > 1
            && output.first().and_then(|ls| ls.first()) == output.last().and_then(|ls| ls.last())
        {
            let mut last_piece = output.pop().unwrap();
            last_piece.pop();
            last_piece.extend_from_slice(output.first().unwrap());
            output.push(last_piece);
            output.swap_remove(0);
        }
    }
}

/// Clip a path by intersecting with a rectangle
pub fn clip_path(clip_rect: Rectangle, path: &LineString<Validated>) -> Vec<Vec<Coordinate>> {
    let clipper = Clipper::new(clip_rect, path);
    clipper.clip()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    fn floats_to_coords(input: Vec<(f64, f64)>) -> Vec<Coordinate> {
        input.into_iter().map(|c| c.into()).collect()
    }

    fn assert_clip(rect: Rectangle, input: Vec<(f64, f64)>, output: Vec<Vec<(f64, f64)>>) {
        let input = floats_to_coords(input);
        let output: Vec<Vec<Coordinate>> = output.into_iter().map(floats_to_coords).collect();
        assert_eq!(
            clip_path(rect, &LineString::try_from(input).unwrap()),
            output
        );
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

    #[test]
    fn test_loops() {
        let rect = Rectangle::new((0., 0.).into(), (1., 1.).into());
        assert_clip(
            rect,
            vec![
                (0.25, 0.25),
                (0.75, 0.25),
                (0.75, 0.75),
                (0.25, 0.75),
                (0.25, 0.25),
            ],
            vec![vec![
                (0.25, 0.25),
                (0.75, 0.25),
                (0.75, 0.75),
                (0.25, 0.75),
                (0.25, 0.25),
            ]],
        );
        assert_clip(
            rect,
            vec![(0.5, 0.5), (1.5, 0.5), (1.5, 1.5), (0.5, 1.5), (0.5, 0.5)],
            vec![vec![(0.5, 1.0), (0.5, 0.5), (1.0, 0.5)]],
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
