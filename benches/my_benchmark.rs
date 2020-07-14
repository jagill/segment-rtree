use criterion::{black_box, criterion_group, criterion_main, Criterion};
use seg_rtree::{Coordinate, Flatbush, Rectangle, SegRTree, LineString};

use seg_rtree::from_wkt::{parse_wkt, Geometry};
use std::fs;
use std::path::Path;

fn read_test_case(name: &str) -> Vec<Vec<Geometry>> {
    let filename = format!("benches/testdata/{}.wkt", name);
    let filepath = Path::new("/Users/jagill/dev/seg_rtree").join(Path::new(&filename));
    let contents = fs::read_to_string(Path::new(&filepath)).unwrap();

    contents
        .split("\n\n")
        .map(|f| parse_wkt(f).unwrap())
        .collect()
}

fn get_rectangles_list(name: &str) -> Vec<Vec<Rectangle>> {
    let positions_list: Vec<Vec<Coordinate>> = read_test_case(name)
        .into_iter()
        .map(|mut geoms| geoms.remove(0))
        .filter_map(|geom| match geom {
            Geometry::Polygon(poly) => Some(poly.shell),
            _ => None,
        })
        .collect();
    let rectangles_list: Vec<Vec<Rectangle>> = positions_list
        .iter()
        .map(|positions| {
            positions
                .windows(2)
                .map(|c| Rectangle::new(c[0], c[1]))
                .collect()
        })
        .collect();
    rectangles_list
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let rectangles_list = get_rectangles_list("africa");
    let mut group = c.benchmark_group("africa");
    group.bench_function("seg_rtree_inc", |b| {
        b.iter(|| {
            for rectangles in &rectangles_list {
                let mut rtree = SegRTree::new(8, rectangles.len());
                for rect in rectangles {
                    rtree.add(*rect).unwrap();
                }
            }
        })
    });
    group.bench_function("seg_rtree_bulk", |b| {
        b.iter(|| {
            for rectangles in &rectangles_list {
                SegRTree::new_loaded(8, rectangles);
            }
        })
    });
    group.bench_function("flatbush", |b| {
        b.iter(|| {
            for rectangles in &rectangles_list {
                Flatbush::new(8, rectangles);
            }
        })
    });
    group.bench_function("flatbush_unsorted", |b| {
        b.iter(|| {
            for rectangles in &rectangles_list {
                Flatbush::new_unsorted(8, rectangles);
            }
        })
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
