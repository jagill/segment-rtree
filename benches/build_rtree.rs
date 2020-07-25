mod utils;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use seg_rtree::SegRTree;
use utils::{get_positions_list, make_rectangles_list};

pub fn construction_benchmark(c: &mut Criterion) {
    let name = "africa";
    let positions_list = get_positions_list(name);
    let rectangles_list = make_rectangles_list(&positions_list);
    println!("Benchmarking {} polygons", rectangles_list.len());
    let mut group = c.benchmark_group(format!("build_rtree_{}", name));
    for degree in [8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("seg_rtree_bulk", degree),
            degree,
            |b, &d| {
                for rectangles in &rectangles_list {
                    b.iter(|| {
                        SegRTree::new_loaded(d, rectangles);
                    });
                }
            },
        );
    }
    group.bench_function("build_rstar", |b| {
        for coords in &positions_list {
            b.iter(|| utils::other_impls::build_rstar(coords));
        }
    });

    group.finish();
}

criterion_group!(benches, construction_benchmark);
criterion_main!(benches);
