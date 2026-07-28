use crate::DETAIL;
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
    pub data: PuzzleData,
    pub stack: Vec<(String, isize)>,
    pub scramble: Option<Vec<String>>,
    pub animation_offset: Option<Turn>, //the turn of the puzzle that the animation is currently doing
    pub solved: bool,
    pub anim_left: f32, //the amount of animation left
    pub solved_data: PuzzleData,
    pub super_data: Option<SuperData>,
}
#[derive(Debug, Clone)]
pub struct PuzzleData {
    pub name: String,
    pub path: PathBuf,
    pub authors: Vec<String>,
    pub pieces: Vec<RenderPiece>, // Make sure these aren't reordered
    pub turns: HashMap<String, OrderedTurn>,
    pub intern: FloatPool,
    pub depth: usize,
    pub keybinds: HashMap<egui::Key, (String, isize)>,
}

impl Puzzle {
    pub fn new(data: PuzzleData, is_super: bool) -> Self {
        Self {
            data: data.clone(),
            stack: vec![],
            scramble: None,
            animation_offset: None,
            solved: true,
            anim_left: 0.0,
            solved_data: data,
            super_data: is_super.then_some(SuperData::new()),
        }
    }
    ///checks if self is solved and updates self.is_solved accordingly
    pub fn check(&mut self) {
        self.solved = self.is_solved();
    }
    ///turns the puzzle around a turn. cuts along the turn first if cut is true.
    ///if the turn was completed, returns Ok(true)
    ///if the turn was bandaged (and cut was false), returns Ok(false)
    ///if an error was encountered, returns Err(e) where e was the error
    pub fn turn(&mut self, turn: OrderedTurn, cut: bool) -> Result<bool, String> {
        let mut new_pieces = Vec::new(); //make a list of new pieces to populate
        if cut {
            let mut cut_pieces = Vec::new(); // Pieces that go on the end of the list
            //if cut is true, cut
            for piece in &self.data.pieces {
                //cut each piece
                let mut turned_iter = turn.turn.turn_cut_render_piece(piece, DETAIL)?.into_iter();
                if let Some(turned) = turned_iter.next() {
                    new_pieces.push(turned); //add it to the list
                    if let Some(turned) = turned_iter.next() {
                        cut_pieces.push(turned); //add it to the list
                    }
                }
            }
            new_pieces.extend(cut_pieces);
        } else {
            for piece in &self.data.pieces {
                new_pieces.push(match turn.turn.turn_render_piece(piece) {
                    None => return Ok(false),
                    Some(x) => x,
                }); //otherwise, just turn each piece
            }
        }
        self.data.pieces = new_pieces;
        self.anim_left = 1.0; //set the animation to run
        self.animation_offset = Some(turn.turn.inverse());
        self.intern_all(); //intern everything
        self.solved = false;
        Ok(true)
    }
    ///turns the puzzle around a turn, given by an id. cuts along the turn first if cut is true.
    ///if the turn was completed, returns Ok(true).
    ///if the turn was bandaged (and cut was false), returns Ok(false).
    ///if an error was encountered, returns Err(e) where e was the error
    pub fn turn_id(&mut self, id: &str, cut: bool, mult: isize) -> Result<bool, String> {
        let turn = self
            .data
            .turns
            .get(id)
            .ok_or("No turn found with ID!".to_string())?
            .mult(mult);
        if !self.turn(turn, cut)? {
            return Ok(false);
        }
        self.stack.push((id.to_string(), mult));
        Ok(true)
    }
    ///undoes the last turn.
    ///Ok(true) means that the move was undone successfully
    ///Ok(false) means that the stack was empty
    ///Err(e) means that an error was encountered
    pub fn undo(&mut self) -> Result<bool, String> {
        if let Some(last) = &self.stack.pop() {
            let last_turn = self.data.turns[&last.0]; //try to find the last turn
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
        for _ in 0..self.data.depth {
            //choose a random turn and do it
            let key = self
                .data
                .turns
                .keys()
                .choose(&mut rng)
                .ok_or("Puzzle.scramble failed: rng choosing a turn failed!".to_string())?
                .clone();
            self.turn(self.data.turns[&key], cut)?;
            scramble.push(key);
        }
        self.animation_offset = None;
        self.scramble = Some(scramble); //set the scramble to Some
        Ok(())
    }
    ///reset the puzzle, using the stored definition
    pub fn reset(&mut self) -> Result<(), String> {
        *self = Puzzle::new(self.solved_data.clone(), self.is_super());
        Ok(())
    }

    pub fn is_super(&self) -> bool {
        self.super_data.is_some()
    }

    /// Is the puzzle in an animation right now
    pub fn in_animation(&self) -> bool {
        self.animation_offset.is_some() && self.anim_left > 0.0
    }
}
