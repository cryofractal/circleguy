use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum SetOrAll {
    Set(HashSet<usize>),
    All,
}

impl SetOrAll {
    pub fn empty() -> Self {
        Self::Set(HashSet::new())
    }

    pub fn contains(&self, index: usize) -> bool {
        match self {
            SetOrAll::Set(set) => set.contains(&index),
            SetOrAll::All => true,
        }
    }

    pub fn insert(&mut self, index: usize) {
        match self {
            SetOrAll::Set(set) => set.insert(index),
            SetOrAll::All => true,
        };
    }

    pub fn remove(&mut self, index: usize) {
        match self {
            SetOrAll::Set(set) => set.remove(&index),
            SetOrAll::All => true,
        };
    }

    pub fn toggle(&mut self, index: usize) {
        if self.contains(index) {
            self.remove(index);
        } else {
            self.insert(index);
        }
    }

    fn split_index(&mut self, index: usize, new_index: usize) {
        if self.contains(index) {
            self.insert(new_index);
        }
    }

    pub fn is_all(&self) -> bool {
        match self {
            SetOrAll::Set(_) => false,
            SetOrAll::All => true,
        }
    }
}

/// Data on color scheme for super puzzles
#[derive(Debug, Clone)]
pub struct SuperData {
    pub orientation_colored: SetOrAll,
}

impl SuperData {
    pub fn new() -> Self {
        Self {
            orientation_colored: SetOrAll::empty(),
        }
    }

    /// Split the piece at index `index` into itself and another piece at index `new_index`.
    pub fn split_index(&mut self, index: usize, new_index: usize) {
        let SuperData {
            orientation_colored,
        } = self;
        orientation_colored.split_index(index, new_index);
    }
}
