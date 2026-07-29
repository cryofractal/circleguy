use std::collections::HashMap;

/// Data on color scheme for super puzzles
#[derive(Debug, Clone)]
pub struct SuperData {
    pub piece_styles: HashMap<usize, SuperStyle>,
    pub piece_style_all: Option<SuperStyle>,
    pub starburst_center: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuperStyle {
    OrientationColor,
    Starburst,
}

impl SuperData {
    pub fn new() -> Self {
        Self {
            piece_styles: HashMap::new(),
            piece_style_all: None,
            starburst_center: 0,
        }
    }

    /// Split the piece at index `index` into itself and another piece at index `new_index`.
    pub fn split_index(&mut self, index: usize, new_index: usize) {
        if let Some(style) = self.piece_styles.get(&index) {
            self.piece_styles.insert(new_index, *style);
        }
    }

    pub fn styled_count(&self, style: SuperStyle) -> usize {
        self.piece_styles.values().filter(|s| **s == style).count()
    }

    pub fn selected_string(&self, style: SuperStyle) -> String {
        if self.piece_style_all == Some(style) {
            "All selected".to_string()
        } else {
            format!("{} selected", self.styled_count(style))
        }
    }

    pub fn toggle_all(&mut self, style: SuperStyle) {
        if self.piece_style_all == Some(style) {
            self.piece_style_all = None;
        } else {
            self.piece_style_all = Some(style);
        }
    }

    pub fn toggle(&mut self, index: usize, style: SuperStyle) -> bool {
        if self.piece_styles.get(&index) == Some(&style) {
            self.piece_styles.remove(&index);
            false
        } else {
            self.piece_styles.insert(index, style);
            true
        }
    }

    pub fn toggle_to(&mut self, index: usize, style: SuperStyle, insert: bool) {
        if insert {
            self.piece_styles.insert(index, style);
        } else if self.piece_styles.get(&index) == Some(&style) {
            self.piece_styles.remove(&index);
        }
    }

    pub fn get(&self, index: usize) -> Option<SuperStyle> {
        self.piece_styles
            .get(&index)
            .copied()
            .or(self.piece_style_all)
    }
}
