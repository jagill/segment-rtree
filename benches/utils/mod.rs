pub mod other_impls;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::fs;
use std::path::Path;

use seg_rtree::from_wkt::{parse_wkt, Geometry};
use seg_rtree::{Coordinate, Rectangle};

//// Utility functions

pub(crate) fn read_test_case(name: &str) -> Vec<Vec<Geometry>> {
    let filename = format!("benches/testdata/{}.wkt", name);
    let filepath = Path::new("/Users/jagill/dev/seg_rtree").join(Path::new(&filename));
    let contents = fs::read_to_string(Path::new(&filepath)).unwrap();

    contents
        .split("\n\n")
        .map(|f| parse_wkt(f).unwrap())
        .collect()
}

pub(crate) fn get_positions_list(name: &str) -> Vec<Vec<Coordinate>> {
    let positions_list: Vec<Vec<Coordinate>> = read_test_case(name)
        .into_iter()
        .take(20)
        .map(|mut geoms| geoms.remove(0))
        .filter_map(|geom| match geom {
            Geometry::Polygon(poly) => Some(poly.shell),
            _ => None,
        })
        .collect();
    positions_list
}

pub(crate) fn make_rectangles_list(positions_list: &[Vec<Coordinate>]) -> Vec<Vec<Rectangle>> {
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

pub(crate) fn get_rectangles_list(name: &str) -> Vec<Vec<Rectangle>> {
    let positions_list = get_positions_list(name);
    make_rectangles_list(&positions_list)
}

pub(crate) fn get_random_points(rect: Rectangle, n: usize, seed: u64) -> Vec<Coordinate> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut results = Vec::new();
    for _i in 0..n {
        results.push(Coordinate::new(
            rng.gen_range(rect.x_min, rect.x_max),
            rng.gen_range(rect.y_min, rect.y_max),
        ));
    }

    results
}

pub(crate) fn get_random_rects(rect: Rectangle, n: usize, seed: u64) -> Vec<Rectangle> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut results = Vec::new();
    for _i in 0..n {
        results.push(Rectangle::new(
            Coordinate::new(
                rng.gen_range(rect.x_min, rect.x_max),
                rng.gen_range(rect.y_min, rect.y_max),
            ),
            Coordinate::new(
                rng.gen_range(rect.x_min, rect.x_max),
                rng.gen_range(rect.y_min, rect.y_max),
            ),
        ));
    }

    results
}
