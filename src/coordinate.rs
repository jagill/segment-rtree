use std::fmt;
use std::ops::{Add, Mul, Sub};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl From<(f64, f64)> for Coordinate {
    fn from(coord: (f64, f64)) -> Self {
        Coordinate {
            x: coord.0,
            y: coord.1,
        }
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Coordinate {
    pub fn new(x: f64, y: f64) -> Self {
        Coordinate { x, y }
    }

    /// Cross product of the vector self x rhs
    pub fn cross(&self, rhs: Coordinate) -> f64 {
        self.x * rhs.y - self.y * rhs.x
    }

    /// Dot product of the vector self . rhs
    pub fn dot(&self, rhs: Coordinate) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }

    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

impl Add for Coordinate {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Coordinate {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Coordinate {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Coordinate {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f64> for Coordinate {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Coordinate {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Coordinate> for f64 {
    type Output = Coordinate;

    fn mul(self, rhs: Coordinate) -> Self::Output {
        Coordinate {
            x: rhs.x * self,
            y: rhs.y * self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mul() {
        let p = Coordinate::new(1., 2.);
        let expected = Coordinate::new(0.2, 0.4);
        assert_eq!(p * 0.2, expected);
        assert_eq!(0.2 * p, expected);
    }

    #[allow(clippy::float_cmp)]
    #[test]
    fn test_cross() {
        let p = Coordinate::new(1., 2.);
        assert_eq!(p.cross(p), 0.);

        let p1 = Coordinate::new(1., 0.);
        let p2 = Coordinate::new(0., 1.);
        assert_eq!(p1.cross(p2), 1.);
        assert_eq!(p2.cross(p1), -1.);
        assert_eq!((3. * p1).cross(5. * p2), 3. * 5.);
    }
}
