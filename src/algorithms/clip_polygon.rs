use super::clip::Clipper;
use crate::geometry_state::{HasRTree, Raw, Validated};
use crate::rectangle::Side;
use crate::{Coordinate, HasEnvelope, LinearRing, Polygon, Rectangle};
use std::cmp::Ordering;
use std::collections::BTreeSet;

pub fn clip_polygon(clip_rect: Rectangle, polygon: &Polygon<Validated>) -> Vec<Polygon<Raw>> {
    if clip_rect.contains(polygon.envelope()) {
        return vec![polygon.clone_to_raw()];
    }
    let (sections, holes, crossings) = find_sections_holes_crossings(clip_rect, polygon);
    let mut output = Vec::new();
    // Fill output

    // TODO: Determine where holes go.  Consider case of polygon fully enclosed.
    output
}

fn find_sections_holes_crossings(
    clip_rect: Rectangle,
    polygon: &Polygon<Validated>,
) -> (
    Vec<Vec<Coordinate>>,
    Vec<Vec<Coordinate>>,
    BTreeSet<Crossing>,
) {
    let mut sections = Vec::with_capacity(polygon.holes().len() + 1);
    let mut holes = Vec::new();
    let mut crossings = BTreeSet::new();

    let mut add_crossing = |coord: Coordinate, index: usize, position: Position| {
        let side = Side::find_side(coord, clip_rect).expect(&format!(
            "Coordinate {} not on side of rect {:?}",
            coord, clip_rect,
        ));
        crossings.insert(Crossing {
            side,
            coord,
            index,
            position,
        });
    };

    let mut process_ring = |ring: &LinearRing<Validated>| {
        let clipper = Clipper::new(clip_rect, ring.coords(), ring.rtree());
        for section in clipper.clip() {
            if section.first() == section.last() {
                holes.push(section);
            } else {
                let section_index = sections.len();
                add_crossing(section[0], section_index, Position::First);
                add_crossing(section[section.len() - 1], section_index, Position::Last);
                sections.push(section);
            }
        }
    };
    process_ring(polygon.shell());
    polygon.holes().iter().for_each(process_ring);
    (sections, holes, crossings)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Position {
    First,
    Last,
}

#[derive(Debug)]
struct Crossing {
    side: Side,
    coord: Coordinate,
    index: usize,
    position: Position,
}

// We should have only finite coordinates
impl PartialEq for Crossing {
    fn eq(&self, other: &Self) -> bool {
        if !self.coord.is_finite() {
            panic!("Found non-finite coordinate.");
        }
        self.side == other.side
            && self.coord == other.coord
            && self.index == other.index
            && self.position == other.position
    }
}

impl Eq for Crossing {}

/// Order Crossings clockwise, from Top Left corner.  Break ties by section index.
impl PartialOrd for Crossing {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.side
                .cmp(&other.side)
                .then_with(|| match self.side {
                    Side::Top => _cmp(self.coord.x, other.coord.x),
                    Side::Right => _cmp(self.coord.y, other.coord.y).reverse(),
                    Side::Bottom => _cmp(self.coord.x, other.coord.x).reverse(),
                    Side::Left => _cmp(self.coord.y, other.coord.y),
                })
                .then(self.index.cmp(&other.index))
                .then(self.position.cmp(&other.position)),
        )
    }
}

// Since our geometries are validated, we should only have finite coordinates here
fn _cmp(x: f64, y: f64) -> Ordering {
    x.partial_cmp(&y).expect("Can't order NaN coordinate.")
}

impl Ord for Crossing {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    mod sections_holes_crossings {
        use super::*;
        use crate::rectangle::Side::*;

        fn assert_result(
            poly: &Polygon<Validated>,
            sections: &Vec<Vec<Coordinate>>,
            holes: &Vec<Vec<Coordinate>>,
            crossings: &Vec<Crossing>,
        ) {
            let clip_rect = Rectangle {
                x_min: 0.,
                y_min: 0.,
                x_max: 10.,
                y_max: 10.,
            };
            let (actual_sections, actual_holes, actual_crossings) =
                find_sections_holes_crossings(clip_rect, poly);
            let actual_crossings: Vec<Crossing> = actual_crossings.into_iter().collect();
            assert_eq!(sections, &actual_sections);
            assert_eq!(holes, &actual_holes);
            assert_eq!(crossings, &actual_crossings);
        }

        #[test]
        fn test_jag() {
            let poly = Polygon::try_new(
                LinearRing::try_from(vec![(1., 1.), (1., 20.), (9., 20.), (9., 1.), (1., 1.)])
                    .unwrap(),
                vec![],
            )
            .unwrap();
            assert_result(
                &poly,
                &vec![Coordinate::vec_from(&[
                    (9., 10.),
                    (9., 1.),
                    (1., 1.),
                    (1., 10.),
                ])],
                &vec![],
                // &vec![],
                &vec![
                    Crossing {
                        side: Top,
                        coord: Coordinate::from((1., 10.)),
                        index: 0,
                        position: Position::Last,
                    },
                    Crossing {
                        side: Top,
                        coord: Coordinate::from((9., 10.)),
                        index: 0,
                        position: Position::First,
                    },
                ],
            );
        }

        #[test]
        fn test_basic() {
            let ring_23_coords =
                Coordinate::vec_from(&vec![(2., 2.), (3., 2.), (3., 3.), (2., 3.), (2., 2.)]);
            let ring_23 = LinearRing::try_from(ring_23_coords.clone()).unwrap();
            let ring_14_coords =
                Coordinate::vec_from(&vec![(1., 1.), (4., 1.), (4., 4.), (1., 4.), (1., 1.)]);
            let ring_14 = LinearRing::try_from(ring_14_coords.clone()).unwrap();

            let poly = Polygon::try_new(ring_23.clone(), vec![]).unwrap();
            assert_result(&poly, &vec![], &vec![ring_23_coords.clone()], &vec![]);

            let poly = Polygon::try_new(ring_14.clone(), vec![ring_23.clone()]).unwrap();
            assert_result(
                &poly,
                &vec![],
                &vec![ring_14_coords.clone(), ring_23_coords.clone()],
                &vec![],
            );
        }

        #[test]
        fn test_shell_clipping() {
            let poly = Polygon::try_new(
                LinearRing::try_from(vec![(1., 1.), (1., 20.), (9., 20.), (9., 1.), (1., 1.)])
                    .unwrap(),
                vec![],
            )
            .unwrap();
            assert_result(
                &poly,
                &vec![Coordinate::vec_from(&[
                    (9., 10.),
                    (9., 1.),
                    (1., 1.),
                    (1., 10.),
                ])],
                &vec![],
                &vec![
                    Crossing {
                        side: Top,
                        coord: Coordinate::from((1., 10.)),
                        index: 0,
                        position: Position::Last,
                    },
                    Crossing {
                        side: Top,
                        coord: Coordinate::from((9., 10.)),
                        index: 0,
                        position: Position::First,
                    },
                ],
            );

            let poly = Polygon::try_new(
                LinearRing::try_from(vec![(1., -1.), (1., 20.), (9., 20.), (9., -1.), (1., -1.)])
                    .unwrap(),
                vec![],
            )
            .unwrap();
            assert_result(
                &poly,
                &vec![
                    Coordinate::vec_from(&[(1., 0.), (1., 10.)]),
                    Coordinate::vec_from(&[(9., 10.), (9., 0.)]),
                ],
                &vec![],
                &vec![
                    Crossing {
                        side: Top,
                        coord: Coordinate::from((1., 10.)),
                        index: 0,
                        position: Position::Last,
                    },
                    Crossing {
                        side: Top,
                        coord: Coordinate::from((9., 10.)),
                        index: 1,
                        position: Position::First,
                    },
                    Crossing {
                        side: Bottom,
                        coord: Coordinate::from((9., 0.)),
                        index: 1,
                        position: Position::Last,
                    },
                    Crossing {
                        side: Bottom,
                        coord: Coordinate::from((1., 0.)),
                        index: 0,
                        position: Position::First,
                    },
                ],
            );
        }
    }
}
