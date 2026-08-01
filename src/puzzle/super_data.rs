use egui::Color32;
use std::collections::HashMap;

use crate::puzzle::color::rainbow_float;

/// Data on color scheme for super puzzles
#[derive(Debug, Clone)]
pub struct SuperData {
    piece_styles: HashMap<usize, SuperStyle>,
    pub piece_style_all: Option<SuperStyle>,
    pub starburst_center: usize,
    pub solid_colors: Vec<Color32>,
    piece_annotations: HashMap<usize, Annotation>,
    pub piece_annotation_all: Option<Annotation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuperStyle {
    OrientationColor,
    Starburst,
    SolidColor(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Annotation {
    Arrow,
}

impl SuperData {
    pub fn new() -> Self {
        Self {
            piece_styles: HashMap::new(),
            piece_style_all: None,
            starburst_center: 0,
            solid_colors: Vec::new(),
            piece_annotations: HashMap::new(),
            piece_annotation_all: None,
        }
    }

    /// Split the piece at index `index` into itself and another piece at index `new_index`.
    pub fn split_index(&mut self, index: usize, new_index: usize) {
        if let Some(style) = self.piece_styles.get(&index) {
            self.piece_styles.insert(new_index, *style);
        }
        if let Some(annotation) = self.piece_annotations.get(&index) {
            self.piece_annotations.insert(new_index, *annotation);
        }
    }

    pub fn set_style(&mut self, index: usize, style: SuperStyle) {
        if style == SuperStyle::Starburst && self.styled_count(style) == 0 {
            self.starburst_center = index;
        }
        self.piece_styles.insert(index, style);
    }

    pub fn remove_style(&mut self, index: usize) {
        self.piece_styles.remove(&index);
    }

    pub fn styled_count(&self, style: SuperStyle) -> usize {
        self.piece_styles.values().filter(|s| **s == style).count()
    }

    pub fn style_selected_string(&self, style: SuperStyle) -> String {
        if self.piece_style_all == Some(style) {
            "All selected".to_string()
        } else {
            format!("{} selected", self.styled_count(style))
        }
    }

    pub fn toggle_style_all(&mut self, style: SuperStyle) {
        if self.piece_style_all == Some(style) {
            self.piece_style_all = None;
        } else {
            self.piece_style_all = Some(style);
        }
    }

    pub fn toggle_style(&mut self, index: usize, style: SuperStyle) -> bool {
        if self.piece_styles.get(&index) == Some(&style) {
            self.remove_style(index);
            false
        } else {
            self.set_style(index, style);
            true
        }
    }

    pub fn toggle_style_to(&mut self, index: usize, style: SuperStyle, insert: bool) {
        if insert {
            self.set_style(index, style);
        } else if self.piece_styles.get(&index) == Some(&style) {
            self.remove_style(index);
        }
    }

    pub fn get_style(&self, index: usize) -> Option<SuperStyle> {
        self.piece_styles
            .get(&index)
            .copied()
            .or(self.piece_style_all)
    }

    pub fn set_annotation(&mut self, index: usize, annotation: Annotation) {
        self.piece_annotations.insert(index, annotation);
    }

    pub fn remove_annotation(&mut self, index: usize) {
        self.piece_annotations.remove(&index);
    }

    pub fn annotationd_count(&self, annotation: Annotation) -> usize {
        self.piece_annotations
            .values()
            .filter(|s| **s == annotation)
            .count()
    }

    pub fn annotation_selected_string(&self, annotation: Annotation) -> String {
        if self.piece_annotation_all == Some(annotation) {
            "All selected".to_string()
        } else {
            format!("{} selected", self.annotationd_count(annotation))
        }
    }

    pub fn toggle_annotation_all(&mut self, annotation: Annotation) {
        if self.piece_annotation_all == Some(annotation) {
            self.piece_annotation_all = None;
        } else {
            self.piece_annotation_all = Some(annotation);
        }
    }

    pub fn toggle_annotation(&mut self, index: usize, annotation: Annotation) -> bool {
        if self.piece_annotations.get(&index) == Some(&annotation) {
            self.remove_annotation(index);
            false
        } else {
            self.set_annotation(index, annotation);
            true
        }
    }

    pub fn toggle_annotation_to(&mut self, index: usize, annotation: Annotation, insert: bool) {
        if insert {
            self.set_annotation(index, annotation);
        } else if self.piece_annotations.get(&index) == Some(&annotation) {
            self.remove_annotation(index);
        }
    }

    pub fn get_annotation(&self, index: usize) -> Option<Annotation> {
        self.piece_annotations
            .get(&index)
            .copied()
            .or(self.piece_annotation_all)
    }

    pub fn add_color(&mut self) {
        let next_unused = rainbow_float(0.61803398874989484 * self.solid_colors.len() as f64);
        self.solid_colors.push(next_unused);
    }

    pub fn remove_color(&mut self, i: usize) {
        if i < self.solid_colors.len() {
            self.solid_colors.remove(i);
            self.piece_styles = self
                .piece_styles
                .iter()
                .filter_map(|(p, style)| match style {
                    SuperStyle::SolidColor(j) if *j > i => {
                        Some((*p, SuperStyle::SolidColor(j - 1)))
                    }
                    SuperStyle::SolidColor(j) if *j == i => None,
                    _ => Some((*p, *style)),
                })
                .collect();
        }
    }

    pub fn vec_solid_colors(&self) -> Vec<(usize, SuperStyle, Color32)> {
        // Cannot borrow self so cannot return iterator
        self.solid_colors
            .iter()
            .enumerate()
            .map(|(i, c)| (i, SuperStyle::SolidColor(i), *c))
            .collect()
    }
}
