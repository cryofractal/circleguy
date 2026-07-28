use crate::{
    complex::{
        arc::Arc,
        c64::Scalar,
        complex_circle::{Circle, Contains, OrientedCircle},
        isometry::Isometry,
        point::Point,
        rotation::Rotation,
    },
    puzzle::{piece::Piece, piece_shape::PieceShape},
};

#[derive(Clone, Debug, Copy)]
///turn of a certain angle around a circle. only points within the circle should be affected.
pub struct Turn {
    pub circle: Circle,
    pub rot: Rotation, //rotation is stored as a mag-1 complex number
}

/// The inside and outside halves of something that may have been cut.
#[derive(Clone, Debug)]
pub struct CutResult<P> {
    pub inside: Option<P>,
    pub outside: Option<P>,
}

impl<P> CutResult<P> {
    /// A `CutResult` with references to the original's data.
    pub fn as_refs(&self) -> CutResult<&P> {
        CutResult {
            inside: self.inside.as_ref(),
            outside: self.outside.as_ref(),
        }
    }

    /// Apply a function to both halves of the object.
    pub fn map<Q>(self, f: impl Fn(P) -> Q) -> CutResult<Q> {
        CutResult {
            inside: self.inside.map(&f),
            outside: self.outside.map(&f),
        }
    }

    /// Apply a function to both halves of the object.
    pub fn map_inside<Q>(self, f: impl Fn(P, bool) -> Q) -> CutResult<Q> {
        CutResult {
            inside: self.inside.map(|p| f(p, true)),
            outside: self.outside.map(|p| f(p, false)),
        }
    }
}

impl<P> IntoIterator for CutResult<P> {
    type Item = P;

    type IntoIter = std::iter::Flatten<std::array::IntoIter<Option<P>, 2>>;

    fn into_iter(self) -> Self::IntoIter {
        [self.inside, self.outside].into_iter().flatten()
    }
}

impl Turn {
    ///take the inverse of a turn, which is around the same circle but with flipped sign on the angle
    pub fn inverse(&self) -> Self {
        Self {
            circle: self.circle,
            rot: self.rot.conj(),
        }
    }
    ///multiply a turn by an integer.
    pub fn mult(&self, mult: Scalar) -> Self {
        Self {
            circle: self.circle,
            rot: (Rotation::from_angle(self.rot.angle() * (mult as Scalar))), //multiply the angle by the scalar and recalculate the number
        }
    }
    ///rotate a point according to the turn. does not care whether the point is in/out of the circle
    pub fn rot_point(&self, point: Point) -> Point {
        self.circle.center + (self.rot * (point - self.circle.center))
    }
    ///rotate a circle according to the turn. does not care whether the circle is in/out of the circle
    pub fn rot_circle(&self, circle: Circle) -> Circle {
        Circle {
            center: self.rot_point(circle.center),
            r_sq: circle.r_sq,
        }
    }
    ///rotate an arc according to the turn. does not care whether the arc is in/out of the circle
    pub fn rot_arc(&self, arc: Arc) -> Arc {
        Arc {
            circle: self.rot_circle(arc.circle),
            start: self.rot_point(arc.start),
            angle: arc.angle,
        }
    }
    ///rotate a shape according to the turn. does not care whether the shape is in/out of the circle
    pub fn rot_pieceshape(&self, shape: &PieceShape) -> PieceShape {
        PieceShape {
            bounds: shape
                .bounds
                .iter()
                .map(|x| OrientedCircle {
                    circ: self.rot_circle(x.circ),
                    ori: x.ori,
                })
                .collect(),
            border: shape.border.iter().map(|x| self.rot_arc(*x)).collect(),
        }
    }
    ///turn a pieceshape according to the turn, without cutting. returns None if the turn is blocked by the piece.
    ///pieces outside turn.circle are unaffected
    pub fn turn_pieceshape(&self, shape: &PieceShape) -> Option<PieceShape> {
        match shape.in_circle(self.circle) {
            None => None,
            Some(Contains::Inside) | Some(Contains::Border) => Some(self.rot_pieceshape(shape)),
            Some(Contains::Outside) => Some(shape.clone()),
        }
    }
    ///turn a pieceshape according to the turn, with cutting. returns 1 or 2 pieces.
    ///if two shapes are returned, a cut was made and exactly one of the two shape was rotated.
    pub fn turn_cut_pieceshape(&self, shape: &PieceShape) -> Result<CutResult<PieceShape>, String> {
        match shape.in_circle(self.circle) {
            None => {
                let (i, o) = shape
                    .cut_by_circle(self.circle)
                    .ok_or("Turn.turn_cut_pieceshape failed: shape crossed cut but was not cut!")?;
                Ok(CutResult {
                    inside: Some(self.rot_pieceshape(&i)),
                    outside: Some(o),
                })
            }
            Some(Contains::Inside) | Some(Contains::Border) => Ok(CutResult {
                inside: Some(self.rot_pieceshape(shape)),
                outside: None,
            }),
            Some(Contains::Outside) => Ok(CutResult {
                inside: None,
                outside: Some(shape.clone()),
            }),
        }
    }
    ///turn a piece according to the turn, without cutting. see turn_pieceshape().
    pub fn turn_piece(&self, piece: &Piece) -> Option<Piece> {
        Some(Piece {
            shape: self.turn_pieceshape(&piece.shape)?,
            color: piece.color,
        })
    }
    ///turn a piece according to the turn, with cutting. see turn_cut_pieceshape().
    pub fn turn_cut_piece(&self, piece: &Piece) -> Result<CutResult<Piece>, String> {
        Ok(self.turn_cut_pieceshape(&piece.shape)?.map(|x| Piece {
            shape: x.clone(),
            color: piece.color,
        }))
    }

    // The isometry corresponding to the turn.
    pub fn isometry(&self) -> Isometry {
        Isometry::rotate_with(self.circle.center, self.rot)
    }
}
#[derive(Debug, Clone, Copy)]
//turn that stores its order
pub struct OrderedTurn {
    pub turn: Turn,
    pub order: usize,
}

impl OrderedTurn {
    pub fn inverse(&self) -> Self {
        Self {
            turn: self.turn.inverse(),
            order: self.order,
        }
    }
    pub fn mult(&self, mult: isize) -> Self {
        Self {
            turn: self.turn.mult(mult as Scalar),
            order: if mult == 0 || self.order == 0 {
                0
            } else {
                self.order / (num::integer::gcd(self.order, mult as usize))
            },
        }
    }
    pub fn turn_piece(&self, piece: &Piece) -> Option<Piece> {
        self.turn.turn_piece(piece)
    }
    pub fn turn_cut_piece(&self, piece: &Piece) -> Result<CutResult<Piece>, String> {
        self.turn.turn_cut_piece(piece)
    }
}
