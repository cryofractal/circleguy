use crate::puzzle::color::golden_ratio_color;
use egui::Color32;
use std::collections::HashSet;

/// Data on puzzle control for all puzzles
#[derive(Debug, Clone)]
pub struct ControlData {
    pub tape_sets: Vec<TapeSet>,
}

/// One set of pieces taped together
#[derive(Debug, Clone)]
pub struct TapeSet {
    pub pieces: HashSet<usize>,
    pub outline_color: Color32,
}

impl ControlData {
    pub fn new() -> Self {
        Self {
            tape_sets: Vec::new(),
        }
    }

    pub fn add_new_tape_set(&mut self) {
        self.tape_sets.push(TapeSet {
            pieces: HashSet::new(),
            outline_color: golden_ratio_color(self.tape_sets.len()),
        })
    }

    pub fn toggle_tape_group(&mut self, index: usize, piece: usize) -> bool {
        if let Some(tape_group) = self.tape_sets.get_mut(index) {
            if tape_group.pieces.contains(&piece) {
                tape_group.pieces.remove(&piece);
                false
            } else {
                tape_group.pieces.insert(piece);
                true
            }
        } else {
            false // dummy value
        }
    }

    pub fn toggle_tape_group_to(&mut self, index: usize, piece: usize, insert: bool) {
        if let Some(tape_group) = self.tape_sets.get_mut(index) {
            if insert {
                tape_group.pieces.insert(piece);
            } else {
                tape_group.pieces.remove(&piece);
            }
        }
    }
}
