use crate::Coordinate;

use self::Side::*;
#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    pub x_min: f64,
    pub y_min: f64,
    pub x_max: f64,
    pub y_max: f64,
}

impl PartialEq for Rectangle {
    fn eq(&self, other: &Self) -> bool {
        if self.is_empty() {
            other.is_empty()
        } else {
            self.x_min == other.x_min
                && self.y_min == other.y_min
                && self.x_max == other.x_max
                && self.y_max == other.y_max
        }
    }
}

pub trait HasEnvelope {
    fn envelope(&self) -> Rectangle;
}

impl HasEnvelope for Coordinate {
    fn envelope(&self) -> Rectangle {
        Rectangle {
            x_min: self.x,
            y_min: self.y,
            x_max: self.x,
            y_max: self.y,
        }
    }
}

impl HasEnvelope for Rectangle {
    fn envelope(&self) -> Rectangle {
        *self
    }
}

impl Rectangle {
    pub fn new(p1: Coordinate, p2: Coordinate) -> Self {
        Rectangle {
            x_min: p1.x.min(p2.x),
            y_min: p1.y.min(p2.y),
            x_max: p1.x.max(p2.x),
            y_max: p1.y.max(p2.y),
        }
    }

    pub fn new_empty() -> Self {
        Rectangle {
            x_min: f64::NAN,
            y_min: f64::NAN,
            x_max: f64::NAN,
            y_max: f64::NAN,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.x_min.is_nan() || self.y_min.is_nan() || self.x_max.is_nan() || self.y_max.is_nan()
    }

    pub fn of<T: HasEnvelope>(items: &[T]) -> Self {
        items.iter().fold(Rectangle::new_empty(), |mut s, r| {
            s.expand(r.envelope());
            s
        })
    }

    pub fn center(&self) -> Coordinate {
        Coordinate {
            x: (self.x_max + self.x_min) / 2.,
            y: (self.y_max + self.y_min) / 2.,
        }
    }

    pub fn intersects<T: HasEnvelope>(&self, item: T) -> bool {
        let other = item.envelope();
        self.x_min <= other.x_max
            && self.x_max >= other.x_min
            && self.y_min <= other.y_max
            && self.y_max >= other.y_min
    }

    pub fn contains<T: HasEnvelope>(&self, item: T) -> bool {
        let other = item.envelope();
        self.x_min <= other.x_min
            && self.x_max >= other.x_max
            && self.y_min <= other.y_min
            && self.y_max >= other.y_max
    }

    pub fn merge<T: HasEnvelope>(&self, item: T) -> Self {
        let rect = item.envelope();
        Rectangle {
            x_min: self.x_min.min(rect.x_min),
            y_min: self.y_min.min(rect.y_min),
            x_max: self.x_max.max(rect.x_max),
            y_max: self.y_max.max(rect.y_max),
        }
    }

    pub fn expand<T: HasEnvelope>(&mut self, item: T) {
        let rect = item.envelope();
        self.x_min = self.x_min.min(rect.x_min);
        self.y_min = self.y_min.min(rect.y_min);
        self.x_max = self.x_max.max(rect.x_max);
        self.y_max = self.y_max.max(rect.y_max);
    }

    /// Return the intersection of the segment defined by start and end.
    /// Uses the Liang-Barsky algorithm:
    /// https://www.skytopia.com/project/articles/compsci/clipping.html
    pub fn intersect_segment(
        &self,
        start: Coordinate,
        end: Coordinate,
    ) -> Option<(Coordinate, Coordinate)> {
        if self.contains(start) && self.contains(end) {
            return Some((start, end));
        } else if start == end {
            return None;
        }

        let mut t0 = 0.;
        let mut t1 = 1.;
        let x_delta = end.x - start.x;
        let y_delta = end.y - start.y;

        for side in SIDES.iter() {
            let (p, q) = match side {
                Left => (-x_delta, -(self.x_min - start.x)),
                Right => (x_delta, (self.x_max - start.x)),
                Top => (-y_delta, -(self.y_min - start.y)),
                Bottom => (y_delta, (self.y_max - start.y)),
            };
            let r = q / p;
            if p == 0. && q < 0. {
                return None;
            }
            if p < 0. {
                if r > t1 {
                    return None;
                } else if r > t0 {
                    t0 = r;
                }
            } else if p > 0. {
                if r < t0 {
                    return None;
                } else if r < t1 {
                    t1 = r;
                }
            }
        }
        Some((
            Coordinate::new(start.x + t0 * x_delta, start.y + t0 * y_delta),
            Coordinate::new(start.x + t1 * x_delta, start.y + t1 * y_delta),
        ))
    }
}

enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

static SIDES: [Side; 4] = [Left, Right, Top, Bottom];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip() {
        let rect = Rectangle::new((0., 0.).into(), (1., 1.).into());
        assert_eq!(
            rect.intersect_segment((0.2, -0.2).into(), (0.1, -0.1).into()),
            None
        );
        assert_eq!(
            rect.intersect_segment((0.2, -0.2).into(), (0.2, 0.2).into()),
            Some(((0.2, 0.0).into(), (0.2, 0.2).into()))
        );
        assert_eq!(
            rect.intersect_segment((-0.2, -0.2).into(), (1.2, 1.2).into()),
            Some(((0.0, 0.0).into(), (1.0, 1.0).into()))
        );
        assert_eq!(
            rect.intersect_segment((0.2, 0.2).into(), (0.8, 0.8).into()),
            Some(((0.2, 0.2).into(), (0.8, 0.8).into()))
        );
        assert_eq!(
            rect.intersect_segment((0.0, -1.0).into(), (0.0, 0.0).into()),
            Some(((0.0, 0.0).into(), (0.0, 0.0).into()))
        );
    }
}
