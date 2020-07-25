mod utils;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use seg_rtree::algorithms::{point_in_polygon, point_in_polygon2};
use seg_rtree::{LineString, Rectangle};
use utils::{get_positions_list, get_random_points, make_rectangles_list};

pub fn point_in_polygon_benchmark(c: &mut Criterion) {
    let name = "africa";
    let positions_list = get_positions_list(name);
    let rectangles_list = make_rectangles_list(&positions_list);
    println!("Benchmarking {} polygons", rectangles_list.len());
    let mut group = c.benchmark_group(format!("point_in_polygon_{}", name));

    let degree = 8;
    group.bench_function(BenchmarkId::new("seg_rtree_pip", degree), |b| {
        // for coords in &positions_list {
        let coords = &positions_list[0];
        let shell = LineString::new(coords.clone());
        let query_points = get_random_points(shell.envelope(), 1000, 342);
        b.iter(|| {
            for &point in &query_points {
                black_box(point_in_polygon(point, &shell)).unwrap();
            }
        })
        // }
    });
    // group.bench_function(BenchmarkId::new("seg_rtree_pip2", degree), |b| {
    //     for coords in &positions_list {
    //         let shell = LineString::new(coords.clone());
    //         let query_points = get_random_points(shell.envelope(), 1000, 342);
    //         b.iter(|| {
    //             for &point in &query_points {
    //                 black_box(point_in_polygon2(point, &shell)).unwrap();
    //             }
    //         });
    //     }
    // });
    // group.bench_function(BenchmarkId::new("rstar_rtree_pip", degree), |b| {
    //     for coords in &positions_list {
    //         let rstar = utils::other_impls::build_rstar(coords);
    //         let query_points = get_random_points(Rectangle::of(coords), 1000, 342);
    //         b.iter(|| {
    //             for &point in &query_points {
    //                 black_box(utils::other_impls::point_in_polygon_rstar(point, &rstar));
    //             }
    //         });
    //     }
    // });

    group.finish();
}

criterion_group!(benches, point_in_polygon_benchmark);

criterion_main!(benches);
