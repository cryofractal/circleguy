use crate::puzzle::render_piece::RenderPiece;
use approx_collections::ApproxEq;

use crate::{
    PRECISION,
    puzzle::{piece_shape::PieceShape, puzzle::Puzzle},
};

impl ApproxEq for PieceShape {
    fn approx_eq(&self, other: &Self, prec: approx_collections::Precision) -> bool {
        compare_vecs(&self.border, &other.border, |x, y| x.approx_eq(y, prec))
    }
}

pub fn same_pieces(first: &Vec<RenderPiece>, second: &Vec<RenderPiece>) -> bool {
    compare_vecs(first, second, |x, y| {
        x.piece.color == y.piece.color && x.piece.shape.approx_eq(&y.piece.shape, PRECISION)
    })
}

impl Puzzle {
    pub fn is_solved(&self) -> bool {
        // TODO: this doesn't work
        same_pieces(&self.data.pieces, &self.solved_data.pieces)
    }
}

pub fn compare_vecs<T: Clone, F: Fn(&T, &T) -> bool>(
    first: &Vec<T>,
    second: &Vec<T>,
    eq: F,
) -> bool {
    if first.len() != second.len() {
        return false;
    }
    let (mut one, mut two) = (first.clone(), second.clone());
    loop {
        let mut index = None;
        for i in 0..two.len() {
            if eq(&one[0], &two[i]) {
                index = Some(i);
                break;
            }
        }
        if let Some(i) = index {
            two.remove(i);
            one.remove(0);
            if one.len() == 0 {
                break true;
            }
        } else {
            break false;
        }
    }
}
