use crate::DETAIL;
use crate::complex::complex_circle::Contains;
use crate::hps::custom_values::hpspuzzledata::HPSPuzzleData;
use crate::hps::data_storer::data_storer::PuzzleData;
use crate::puzzle::piece::Piece;
use crate::puzzle::render_piece::RenderPiece;
use crate::puzzle::super_data::SuperData;
use crate::puzzle::turn::*;
use approx_collections::FloatPool;
use rand::SeedableRng;
use rand::prelude::IteratorRandom;
use std::collections::HashMap;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::path::PathBuf;
#[derive(Debug, Clone)]
pub struct Puzzle {
    pub name: String,
    pub path: PathBuf,
    pub authors: Vec<String>,
    pub turns: HashMap<String, OrderedTurn>,
    pub intern: FloatPool,
    pub depth: usize,
    pub keybinds: HashMap<egui::Key, (String, isize)>,
    pub solved_pieces: Vec<RenderPiece>, // This is only used for resetting the puzzle
    pub super_data: Option<SuperData>,
    pub position: PuzzlePosition,
}

#[derive(Debug, Clone)]
pub struct PuzzlePosition {
    pub pieces: Vec<RenderPiece>, // Make sure these aren't reordered
    pub stack: Vec<(String, isize)>,
    pub scramble: Option<Vec<String>>,
    pub animation_offset: Option<Turn>, //the turn of the puzzle that the animation is currently doing
    pub solved: bool,
    pub anim_left: f32, //the amount of animation left
}

impl PuzzlePosition {
    pub fn new(pieces: Vec<RenderPiece>) -> Self {
        Self {
            pieces,
            stack: vec![],
            scramble: None,
            animation_offset: None,
            solved: true,
            anim_left: 0.0,
        }
    }
}

impl Puzzle {
    pub fn new(data: PuzzleData, is_super: bool) -> Self {
        let pieces: Vec<_> = data
            .data
            .pieces
            .iter()
            .map(|x| x.clone().triangulate(DETAIL))
            .collect();
        Self {
            name: data.data.name.clone(),
            authors: data.data.authors.clone(),
            turns: data.data.turns.clone(),
            intern: data.data.intern.clone(),
            depth: data.data.scramble,
            keybinds: data.keybinds,
            path: data.path,
            solved_pieces: pieces.clone(),
            super_data: is_super.then_some(SuperData::new()),
            position: PuzzlePosition::new(pieces),
        }
    }
    ///checks if self is solved and updates self.is_solved accordingly
    pub fn check(&mut self) {
        self.position.solved = self.is_solved();
    }
    ///turns the puzzle around a turn. cuts along the turn first if cut is true.
    ///if the turn was completed, returns Ok(true)
    ///if the turn was bandaged (and cut was false), returns Ok(false)
    ///if an error was encountered, returns Err(e) where e was the error
    pub fn turn(&mut self, turn: OrderedTurn, cut: bool) -> Result<bool, String> {
        let mut new_pieces = Vec::new();
        let mut cut_pieces = Vec::new(); // Pieces that go on the end of the list
        for piece in &self.position.pieces {
            let circle = turn.turn.circle * turn.turn.isometry().inverse();
            match piece.in_circle(turn.turn.circle) {
                None => {
                    if cut {
                        // Cut the piece
                        let (shape_in, shape_out) = piece
                            .piece
                            .shape
                            .cut_by_circle(circle)
                            .ok_or("Cut failed: shape crossed cut but was not cut!")?;

                        let mut piece_in = Piece {
                            shape: shape_in,
                            color: piece.piece.color,
                        }
                        .triangulate(DETAIL);
                        piece_in.isometry.right_mul_mut(turn.turn.isometry());
                        new_pieces.push(piece_in);

                        let piece_out = Piece {
                            shape: shape_out,
                            color: piece.piece.color,
                        }
                        .triangulate(DETAIL);
                        cut_pieces.push(piece_out);
                    } else {
                        return Ok(false);
                    }
                }
                Some(Contains::Inside | Contains::Border) => {
                    let mut piece = piece.clone();
                    piece.isometry.right_mul_mut(turn.turn.isometry());
                    new_pieces.push(piece);
                }
                Some(Contains::Outside) => {
                    new_pieces.push(piece.clone());
                }
            }
        }
        new_pieces.extend(cut_pieces);
        self.position.pieces = new_pieces;

        self.position.anim_left = 1.0; //set the animation to run
        self.position.animation_offset = Some(turn.turn.inverse());
        self.intern_all(); //intern everything
        self.position.solved = false;
        Ok(true)
    }
    ///turns the puzzle around a turn, given by an id. cuts along the turn first if cut is true.
    ///if the turn was completed, returns Ok(true).
    ///if the turn was bandaged (and cut was false), returns Ok(false).
    ///if an error was encountered, returns Err(e) where e was the error
    pub fn turn_id(&mut self, id: &str, cut: bool, mult: isize) -> Result<bool, String> {
        let turn = self
            .turns
            .get(id)
            .ok_or("No turn found with ID!".to_string())?
            .mult(mult);
        if !self.turn(turn, cut)? {
            return Ok(false);
        }
        self.position.stack.push((id.to_string(), mult));
        Ok(true)
    }
    ///undoes the last turn.
    ///Ok(true) means that the move was undone successfully
    ///Ok(false) means that the stack was empty
    ///Err(e) means that an error was encountered
    pub fn undo(&mut self) -> Result<bool, String> {
        if let Some(last) = &self.position.stack.pop() {
            let last_turn = self.turns[&last.0]; //try to find the last turn
            if !self.turn(last_turn.inverse().mult(last.1), false)? {
                return Err(String::from("Puzzle.undo failed: undo turn was bandaged!"));
            };
            Ok(true)
        } else {
            Ok(false)
        }
    }
    ///scramble the puzzle self.depth moves
    pub fn scramble(&mut self, cut: bool) -> Result<(), String> {
        self.reset()?;
        let mut scramble = Vec::new();
        let mut h = DefaultHasher::new();
        web_time::Instant::now().hash(&mut h);
        let bytes = h.finish().to_ne_bytes();
        let mut rng = rand::rngs::StdRng::from_seed(
            //initialize the rng from a seed. this is needed for web reasons
            [bytes; 4]
                .as_flattened()
                .try_into()
                .expect("error casting [[u8; 8]; 4] to [u8; 32]"),
        );
        for _ in 0..self.depth {
            //choose a random turn and do it
            let key = self
                .turns
                .keys()
                .choose(&mut rng)
                .ok_or("Puzzle.scramble failed: rng choosing a turn failed!".to_string())?
                .clone();
            self.turn(self.turns[&key], cut)?;
            scramble.push(key);
        }
        self.position.animation_offset = None;
        self.position.scramble = Some(scramble); //set the scramble to Some
        Ok(())
    }
    ///reset the puzzle, using the stored definition
    pub fn reset(&mut self) -> Result<(), String> {
        self.position = PuzzlePosition::new(self.solved_pieces.clone());
        Ok(())
    }

    pub fn is_super(&self) -> bool {
        self.super_data.is_some()
    }

    /// Is the puzzle in an animation right now
    pub fn in_animation(&self) -> bool {
        self.position.animation_offset.is_some() && self.position.anim_left > 0.0
    }
}
