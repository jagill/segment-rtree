use crate::Coordinate;
use wkt::types;
use wkt::types::Coord;

#[derive(PartialEq, Debug)]
pub struct Polygon {
    pub shell: Vec<Coordinate>,
    pub holes: Vec<Vec<Coordinate>>,
}

#[derive(PartialEq, Debug)]
pub enum Geometry {
    Empty,
    Point(Coordinate),
    MultiPoint(Vec<Coordinate>),
    LineString(Vec<Coordinate>),
    MultiLineString(Vec<Vec<Coordinate>>),
    Polygon(Polygon),
    MultiPolygon(Vec<Polygon>),
}

impl From<Coord<f64>> for Coordinate {
    fn from(coord: Coord<f64>) -> Self {
        Coordinate {
            x: coord.x,
            y: coord.y,
        }
    }
}

fn point_to_position(point: types::Point<f64>) -> Option<Coordinate> {
    Some(point.0?.into())
}

fn coords_to_positions(coords: Vec<Coord<f64>>) -> Vec<Coordinate> {
    coords.into_iter().map(Coordinate::from).collect()
}

fn linestring_to_positions(linestring: types::LineString<f64>) -> Vec<Coordinate> {
    coords_to_positions(linestring.0)
}

#[allow(dead_code)]
pub fn parse_wkt(wkt_str: &str) -> Result<Vec<Geometry>, &str> {
    let wkt_geoms = wkt::Wkt::from_str(wkt_str)?;
    let geoms = wkt_geoms.items.into_iter().map(from_wkt_geometry).collect();
    Ok(geoms)
}

fn from_wkt_geometry(geom: wkt::Geometry<f64>) -> Geometry {
    match geom {
        wkt::Geometry::Point(p) => from_wkt_point(p),
        wkt::Geometry::LineString(ls) => from_wkt_linestring(ls),
        wkt::Geometry::Polygon(p) => from_wkt_polygon(p),
        wkt::Geometry::MultiPoint(mp) => from_wkt_multi_point(mp),
        wkt::Geometry::MultiLineString(mls) => from_wkt_multi_linestring(mls),
        wkt::Geometry::MultiPolygon(mpoly) => from_wkt_multi_polygon(mpoly),
        _ => unimplemented!(),
    }
}

fn from_wkt_point(pt: types::Point<f64>) -> Geometry {
    match point_to_position(pt) {
        None => Geometry::Empty,
        Some(pos) => Geometry::Point(pos),
    }
}

fn from_wkt_linestring(ls: types::LineString<f64>) -> Geometry {
    Geometry::LineString(linestring_to_positions(ls))
}

fn _from_wkt_polygon(poly: wkt::types::Polygon<f64>) -> Option<Polygon> {
    let mut linestrings = poly.0;
    if linestrings.is_empty() {
        return None;
    }
    let shell: Vec<Coordinate> = linestring_to_positions(linestrings.remove(0));
    let holes: Vec<Vec<Coordinate>> = linestrings
        .into_iter()
        .map(linestring_to_positions)
        .collect();
    Some(Polygon { shell, holes })
}

fn from_wkt_polygon(poly: wkt::types::Polygon<f64>) -> Geometry {
    match _from_wkt_polygon(poly) {
        None => Geometry::Empty,
        Some(p) => Geometry::Polygon(p),
    }
}

fn from_wkt_multi_point(mp: wkt::types::MultiPoint<f64>) -> Geometry {
    Geometry::MultiPoint(mp.0.into_iter().filter_map(point_to_position).collect())
}

fn from_wkt_multi_linestring(mls: wkt::types::MultiLineString<f64>) -> Geometry {
    Geometry::MultiLineString(mls.0.into_iter().map(linestring_to_positions).collect())
}

fn from_wkt_multi_polygon(mpoly: wkt::types::MultiPolygon<f64>) -> Geometry {
    Geometry::MultiPolygon(mpoly.0.into_iter().filter_map(_from_wkt_polygon).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_positions(coords: Vec<(f64, f64)>) -> Vec<Coordinate> {
        coords.into_iter().map(|c| c.into()).collect()
    }

    fn get_single_geom(wkt_str: &str) -> Geometry {
        let mut geoms = parse_wkt(wkt_str).unwrap();
        assert_eq!(geoms.len(), 1);
        geoms.remove(0)
    }

    fn assert_equals_point(wkt_str: &str, x: f64, y: f64) {
        if let Geometry::Point(pos) = get_single_geom(wkt_str) {
            assert_eq!(pos, Coordinate { x, y });
        } else {
            panic!("Geometry is not point");
        }
    }

    fn assert_equals_linestring(wkt_str: &str, coords: Vec<(f64, f64)>) {
        if let Geometry::LineString(linestring) = get_single_geom(wkt_str) {
            assert_eq!(linestring, make_positions(coords));
        } else {
            panic!("Geometry is not linestring");
        }
    }

    fn assert_equals_polygon(
        wkt_str: &str,
        exterior: Vec<(f64, f64)>,
        interiors: Vec<Vec<(f64, f64)>>,
    ) {
        if let Geometry::Polygon(polygon) = get_single_geom(wkt_str) {
            assert_eq!(
                polygon,
                Polygon {
                    shell: make_positions(exterior),
                    holes: interiors.into_iter().map(make_positions).collect()
                }
            );
        } else {
            panic!("Geometry is not polygon");
        }
    }

    fn assert_equals_multipoint(wkt_str: &str, coords: Vec<(f64, f64)>) {
        if let Geometry::MultiPoint(positions) = get_single_geom(wkt_str) {
            assert_eq!(positions, make_positions(coords));
        } else {
            panic!("Geometry is not multipoint");
        }
    }

    fn assert_equals_multilinestring(wkt_str: &str, coords_list: Vec<Vec<(f64, f64)>>) {
        let expected_list: Vec<Vec<Coordinate>> =
            coords_list.into_iter().map(make_positions).collect();
        if let Geometry::MultiLineString(positions_list) = get_single_geom(wkt_str) {
            assert_eq!(positions_list, expected_list);
        } else {
            panic!("Geometry is not multilinestring");
        }
    }

    fn assert_equals_multipolygon(wkt_str: &str, polys: Vec<Vec<(f64, f64)>>) {
        let expected_list: Vec<Polygon> = polys
            .into_iter()
            .map(make_positions)
            .map(|shell| Polygon {
                shell,
                holes: Vec::new(),
            })
            .collect();
        if let Geometry::MultiPolygon(polygons) = get_single_geom(wkt_str) {
            assert_eq!(polygons, expected_list);
        } else {
            panic!("Geometry is not multipolygon");
        }
    }

    #[test]
    fn check_empty_str() {
        assert_eq!(parse_wkt("").unwrap(), Vec::new());
    }

    #[test]
    fn check_bad_str() {
        assert!(parse_wkt("xyz").is_err());
    }

    #[test]
    fn check_point() {
        assert_equals_point("POINT(1.0 1.0)", 1.0, 1.0);
    }

    #[test]
    fn check_integer_point() {
        assert_equals_point("POINT (3 4)", 3.0, 4.0);
    }

    #[test]
    fn check_linestring_empty() {
        assert_equals_linestring("LINESTRING EMPTY", Vec::new());
    }

    #[test]
    fn check_linestring_single_point() {
        assert_equals_linestring("LINESTRING(1 1)", vec![(1.0, 1.0)]);
    }

    #[test]
    fn check_linestring_four_point() {
        assert_equals_linestring(
            "LINESTRING(1 1,2 3,4 8, -6 3)",
            vec![(1.0, 1.0), (2., 3.), (4., 8.), (-6., 3.)],
        );
    }

    #[test]
    fn check_linestring_duplicate() {
        assert_equals_linestring("LINESTRING(1 1, 1 1)", vec![(1.0, 1.0), (1., 1.)]);
    }

    // The wkt library doesn't deserialize these correctly
    // #[test]
    // fn check_polygon_empty() {
    //    assert_equals_polygon("POLYGON EMPTY", Vec::new(), Vec::new());
    // }

    #[test]
    fn check_polygon_simple() {
        assert_equals_polygon(
            "POLYGON((1 1, 3 3, 3 1, 1 1))",
            vec![(1., 1.), (3., 3.), (3., 1.), (1., 1.)],
            Vec::new(),
        );
    }

    #[test]
    fn check_polygon_interior() {
        assert_equals_polygon(
            "POLYGON((-5 -5, -5 5, 5 5, 5 -5, -5 -5),(0 0, 3 0, 3 3, 0 3, 0 0))",
            vec![(-5., -5.), (-5., 5.), (5., 5.), (5., -5.), (-5., -5.)],
            vec![vec![(0., 0.), (3., 0.), (3., 3.), (0., 3.), (0., 0.)]],
        );
    }

    #[test]
    fn check_polygon_two_interiors() {
        assert_equals_polygon(
            "POLYGON((-20 -20, -20 20, 20 20, 20 -20, -20 -20), (10 0, 0 10, 0 -10, 10 0), (-10 0, 0 10, -5 -10, -10 0))",
            vec![(-20., -20.), (-20., 20.), (20., 20.), (20., -20.), (-20., -20.)],
            vec![
                vec![(10., 0.), (0., 10.), (0., -10.), (10., 0.)],
                vec![(-10., 0.), (0., 10.), (-5., -10.), (-10., 0.)]
            ],
        );
    }

    #[test]
    fn check_multipoint() {
        assert_equals_multipoint("MULTIPOINT((2 3), (7 8))", vec![(2., 3.), (7., 8.)]);
    }

    #[test]
    fn check_multilinestring() {
        assert_equals_multilinestring(
            "MULTILINESTRING((1 1, 5 5), (1 3, 3 1))",
            vec![vec![(1., 1.), (5., 5.)], vec![(1., 3.), (3., 1.)]],
        )
    }

    #[test]
    fn check_multipolygon() {
        assert_equals_multipolygon(
            "MULTIPOLYGON(((1 1, 1 -1, -1 -1, -1 1, 1 1)),((1 1, 3 1, 3 3, 1 3, 1 1)))",
            vec![
                vec![(1., 1.), (1., -1.), (-1., -1.), (-1., 1.), (1., 1.)],
                vec![(1., 1.), (3., 1.), (3., 3.), (1., 3.), (1., 1.)],
            ],
        )
    }
}
