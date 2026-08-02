use crate::complex::{arc::Arc, c64::Scalar, complex_circle::Circle, rotation::Rotation};
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
            a: C64::one(),
            b: C64::zero(),
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
            b: cent.0 * (C64::one() - phi.0),
        }
    }

    /// The angle of rotation of this isometry in the interval [-π, π].
    pub fn rotation_angle(self) -> Scalar {
        self.a.angle()
    }

    /// Right-multiply by an isometry in place.
    pub fn right_mul_mut(&mut self, other: Self) {
        *self = *self * other
    }

    /// Left-multiply by an isometry in place.
    pub fn left_mul_mut(&mut self, other: Self) {
        *self = other * *self
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

impl Mul<Isometry> for Circle {
    type Output = Circle;
    fn mul(self, rhs: Isometry) -> Self::Output {
        Circle {
            center: self.center * rhs,
            r_sq: self.r_sq,
        }
    }
}

impl Mul<Isometry> for Arc {
    type Output = Arc;
    fn mul(self, rhs: Isometry) -> Self::Output {
        Arc {
            circle: self.circle * rhs,
            start: self.start * rhs,
            angle: self.angle,
        }
    }
}
