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

    pub fn toggle_orientation_colored(&mut self, i: usize) {
        if !self.orientation_colored.insert(i) {
            self.orientation_colored.remove(&i);
        }
    }
}
