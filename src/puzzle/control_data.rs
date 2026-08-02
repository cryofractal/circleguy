use std::collections::HashSet;

/// Data on puzzle control for all puzzles
#[derive(Debug, Clone)]
pub struct ControlData {
    pub tape_groups: Vec<HashSet<usize>>,
}

impl ControlData {
    pub fn new() -> Self {
        Self {
            tape_groups: Vec::new(),
        }
    }

    pub fn toggle_tape_group(&mut self, index: usize, piece: usize) -> bool {
        if let Some(tape_group) = self.tape_groups.get_mut(index) {
            if tape_group.contains(&piece) {
                tape_group.remove(&piece);
                false
            } else {
                tape_group.insert(piece);
                true
            }
        } else {
            false // dummy value
        }
    }

    pub fn toggle_tape_group_to(&mut self, index: usize, piece: usize, insert: bool) {
        if let Some(tape_group) = self.tape_groups.get_mut(index) {
            if insert {
                tape_group.insert(piece);
            } else {
                tape_group.remove(&piece);
            }
        }
    }
}
