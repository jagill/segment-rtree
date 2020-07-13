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
