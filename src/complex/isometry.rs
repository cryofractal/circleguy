use crate::complex::{c64::Scalar, rotation::Rotation};
use std::ops::Mul;

use approx_collections::ApproxEq;

use crate::complex::{c64::C64, point::Point, vector::Vector};

/// A transformation that maps z to a z + b. It acts on the right.
#[derive(Clone, Debug, Copy)]
pub struct Isometry {
    pub a: C64,
    pub b: C64,
}

impl Isometry {
    /// Identity isometry.
    pub fn identity() -> Self {
        Self {
            a: C64 { re: 1.0, im: 0.0 },
            b: C64 { re: 0.0, im: 0.0 },
        }
    }

    /// Inverse isometry.
    pub fn inverse(self) -> Self {
        // if z' = a z + b, then z = z'/a - b/a
        Self {
            a: self.a.recip(),
            b: -self.b * self.a.recip(),
        }
    }

    /// An isometry that rotates by the angle `angle` around the point `cent`.
    pub fn rotate_about(cent: Point, angle: Scalar) -> Self {
        Self::rotate_with(cent, Rotation::from_angle(angle))
    }

    /// An isometry that performs the rotation `phi` around the point `cent`.
    pub fn rotate_with(cent: Point, phi: Rotation) -> Self {
        // z' = (z - cent) * phi + cent = phi * z + cent (1 - phi)
        Self {
            a: phi.0,
            b: cent.0 * (C64 { re: 1.0, im: 0.0 } - phi.0),
        }
    }
}

impl ApproxEq for Isometry {
    fn approx_eq(&self, other: &Self, prec: approx_collections::Precision) -> bool {
        self.a.approx_eq(&other.a, prec) && self.b.approx_eq(&other.b, prec)
    }
}

impl Mul for Isometry {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        // if z' = a z + b and z'' = a' z' + b', then z = a' a z + a' b + b'
        Self {
            a: self.a * rhs.a,
            b: self.b * rhs.a + rhs.b,
        }
    }
}

impl Mul<Isometry> for Point {
    type Output = Point;
    fn mul(self, rhs: Isometry) -> Self::Output {
        Point(self.0 * rhs.a + rhs.b)
    }
}

impl Mul<Isometry> for Vector {
    type Output = Vector;
    fn mul(self, rhs: Isometry) -> Self::Output {
        Vector(self.0 * rhs.a)
    }
}
