use std::collections::HashSet;

/// Data on color scheme for super puzzles
#[derive(Debug, Clone)]
pub struct SuperData {
    pub orientation_colored: HashSet<usize>,
}

impl SuperData {
    pub fn new() -> Self {
        Self {
            orientation_colored: HashSet::new(),
        }
    }

    /// Split the piece at index `index` into itself and another piece at index `new_index`.
    pub fn split_index(&mut self, index: usize, new_index: usize) {
        let SuperData {
            orientation_colored,
        } = self;
        if orientation_colored.contains(&index) {
            orientation_colored.insert(new_index);
        }
    }

    pub fn toggle_orientation_colored(&mut self, i: usize) {
        if !self.orientation_colored.insert(i) {
            self.orientation_colored.remove(&i);
        }
    }
}
