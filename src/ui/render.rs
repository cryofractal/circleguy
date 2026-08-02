use crate::DEF_PATH;
use crate::DETAIL;
use crate::PRECISION;
use crate::complex::arc::Arc;
use crate::complex::c64::C64;
use crate::complex::complex_circle::Circle;
use crate::complex::complex_circle::Contains;
use crate::complex::complex_circle::OrientedCircle;
use crate::complex::isometry::Isometry;
use crate::complex::point::Point;
use crate::complex::vector::Vector;
use crate::hps::data_storer::data_storer::DataStorer;
use crate::hps::data_storer::data_storer::PuzzleLoadingData;
use crate::hps::data_storer::def_entry::DefEntry;
use crate::puzzle::color::Color;
use crate::puzzle::color::rainbow_float;
use crate::puzzle::color::rainbow_rational;
use crate::puzzle::piece::Piece;
use crate::puzzle::puzzle::*;
use crate::puzzle::render_piece::Triangulation;
use crate::puzzle::super_data::Annotation;
use crate::puzzle::super_data::SuperStyle;
use approx_collections::ApproxEq;
use approx_collections::FloatPool;
use core::f64;
use egui::FontId;
use egui::Popup;
use egui::RichText;
use egui::{
    Color32, Pos2, Rect, Stroke, Ui, Vec2,
    epaint::{self, PathShape},
    pos2,
};
use std::cmp::*;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::ffi::OsString;
use std::hash::Hash;
use std::hash::Hasher;

const STARBURST_SIZE: usize = 20;
const STARBURST_CUT_RADIUS: f64 = 1000.0;
const ARROW_LENGTH: f32 = 5.0;
const ARROW_COLOR: egui::Color32 = Color32::BLACK;

pub struct RenderingCircle {
    pub cent: Pos2,
    pub rad: f32,
}

///draws a the circumference of a circle given the coordinates
pub fn draw_circle(real_circle: Circle, ui: &mut Ui, cc: CoordinateConverter) {
    {
        ui.painter().circle_stroke(
            real_circle.center.to_pos2(cc),
            real_circle.r() as f32 * cc.scale_factor * (cc.rect.width() / 1920.0),
            (10.0, Color32::WHITE),
        );
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CoordinateConverter {
    pub rect: Rect,
    pub scale_factor: f32,
    pub offset_pos: Vec2,
}

impl Point {
    ///translates from cga2d coords to egui coords
    fn to_pos2(&self, cc: CoordinateConverter) -> Pos2 {
        pos2(
            (self.0.re as f32 + cc.offset_pos.x) * (cc.scale_factor * cc.rect.width() / 1920.0)
                + (cc.rect.width() / 2.0)
                + cc.rect.min.x,
            -(self.0.im as f32 + cc.offset_pos.y) * (cc.scale_factor * cc.rect.width() / 1920.0)
                + (cc.rect.height() / 2.0)
                + cc.rect.min.y,
        )
    }
    ///translates from egui coords to cga2d coords
    pub fn from_pos2(pos: &Pos2, cc: CoordinateConverter) -> Self {
        Self(C64 {
            re: (((pos.x - (cc.rect.width() / 2.0))
                * (1920.0 / (cc.scale_factor * cc.rect.width())))
                - cc.offset_pos.x) as f64,
            im: (((pos.y - (cc.rect.height() / 2.0))
                * (-1920.0 / (cc.scale_factor * cc.rect.width())))
                - cc.offset_pos.y) as f64,
        })
    }
}

impl Triangulation {
    ///render the triangulation, according to a detail and a color.
    pub fn render_fill(
        &self,
        ui: &mut Ui,
        cc: CoordinateConverter,
        isometry: Isometry,
        color: Color32,
    ) {
        let mut triangle_vertices: Vec<epaint::Vertex> = Vec::new(); //make a new vector of epaint vertices
        for triangle in &self.inside {
            //iterate over the triangles
            for point in triangle {
                let point = *point * isometry;
                let vertex = epaint::Vertex {
                    pos: point.to_pos2(cc),
                    uv: pos2(0.0, 0.0),
                    color,
                };
                triangle_vertices.push(vertex); //add the nondegenerate triangle vertices
            }
        }
        let mut mesh = epaint::Mesh::default(); //make a new mesh
        mesh.indices = (0..(triangle_vertices.len() as u32)).collect();
        mesh.vertices = triangle_vertices; //add all the vertices
        ui.painter().add(egui::Shape::Mesh(mesh.into())); //paint the triangles
    }

    ///render the outlines of the triangulation, according to a detail and a color.
    pub fn render_outlines(
        &self,
        ui: &mut Ui,
        cc: CoordinateConverter,
        isometry: Isometry,
        width: f32,
        color: Color32,
    ) {
        //now we render the outlines
        for arc in &self.border {
            ui.painter().add(PathShape::line(
                arc.iter().map(|x| (*x * isometry).to_pos2(cc)).collect(),
                Stroke::new(width, color),
            ));
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutlineStyle {
    Normal,
    Taped,
    Hovered,
    HoveredSecondary,
    Debug,
}

impl OutlineStyle {
    pub fn width(self) -> f32 {
        match self {
            OutlineStyle::Normal => 1.0,
            OutlineStyle::Taped => 0.5,
            OutlineStyle::Hovered => 2.0,
            OutlineStyle::HoveredSecondary => 1.5,
            OutlineStyle::Debug => 3.0,
        }
    }

    pub fn color(self) -> Color32 {
        match self {
            OutlineStyle::Normal => Color32::BLACK,
            OutlineStyle::Taped => Color32::BLACK,
            OutlineStyle::Hovered => Color32::from_rgb(210, 210, 210),
            OutlineStyle::HoveredSecondary => Color32::from_rgb(168, 168, 168),
            OutlineStyle::Debug => Color32::MAGENTA,
        }
    }
}

fn starburst_circle(i: usize, n: usize, center: Point) -> Circle {
    Circle {
        center: Point(
            STARBURST_CUT_RADIUS * C64::from_angle(2.0 * PI * i as f64 / n as f64) + center.0,
        ),
        r_sq: STARBURST_CUT_RADIUS.powi(2),
    }
}

///render a piece, with an outline
impl Puzzle {
    pub fn render_piece(
        &self,
        index: usize,
        ui: &mut Ui,
        cc: CoordinateConverter,
        solved: bool,
    ) -> Result<(), String> {
        let Some(piece) = self.position.pieces.get(index) else {
            return Ok(());
        };

        let isometry = if solved {
            Isometry::identity()
        } else {
            piece.attitude
                * if let Some(offset) = self.position.animation_offset
                    && piece.in_circle(offset.circle)
                        == Some(crate::complex::complex_circle::Contains::Inside)
                {
                    //get the offset of the piece, base on if its in the animation_offset circle
                    offset.mult(self.position.anim_left as f64).isometry()
                } else {
                    Isometry::identity()
                }
        };

        let pieces = match &self.super_data {
            Some(super_data) => {
                match super_data.get_style(index) {
                    None => {
                        vec![(piece.clone(), Color32::DARK_GRAY)]
                    }
                    Some(SuperStyle::OrientationColor) => {
                        let angle = isometry.rotation_angle();
                        // angle is in [-π, π]
                        let color = if isometry.approx_eq(&Isometry::identity(), PRECISION) {
                            Color::White.to_egui()
                        } else if angle.approx_eq(&0.0, PRECISION) {
                            Color::Gray.to_egui()
                        } else {
                            rainbow_float(angle / (2.0 * PI))
                        };
                        vec![(piece.clone(), color)]
                    }
                    Some(SuperStyle::Starburst) => {
                        let starburst_center = self
                            .position
                            .pieces
                            .get(super_data.starburst_center)
                            .ok_or("Starburst center does not exist")?
                            .barycenter();
                        let mut pieces = Vec::new();
                        for i in 0..STARBURST_SIZE {
                            // TODO: hack because no straight lines: cut by huge circles
                            let shape = &piece.piece.shape;
                            let Some(shape) = shape.intersect_by_circle(OrientedCircle {
                                circ: starburst_circle(i, STARBURST_SIZE, starburst_center),
                                ori: Contains::Inside,
                            }) else {
                                continue;
                            };
                            let Some(shape) = shape.intersect_by_circle(OrientedCircle {
                                circ: starburst_circle(i - 1, STARBURST_SIZE, starburst_center),
                                ori: Contains::Outside,
                            }) else {
                                continue;
                            };

                            let color = rainbow_rational(i, STARBURST_SIZE);
                            pieces.push((
                                Piece {
                                    shape,
                                    color: Color::Black,
                                }
                                .triangulate(DETAIL),
                                color,
                            )) // TODO: dummy color
                        }
                        pieces
                    }
                    Some(SuperStyle::SolidColor(i)) => {
                        vec![(
                            piece.clone(),
                            super_data
                                .solid_colors
                                .get(i)
                                .cloned()
                                .unwrap_or(Color::None.to_egui()),
                        )]
                    }
                }
            }
            None => {
                let color = piece.piece.color.to_egui();
                vec![(piece.clone(), color)]
            }
        };

        for (new_piece, color) in pieces {
            for triangle in &new_piece.triangulations {
                //iterate over the triangles
                triangle.render_fill(ui, cc, isometry, color);
            }
        }

        Ok(())
    }

    pub fn render_piece_outline(
        &self,
        index: usize,
        ui: &mut Ui,
        cc: CoordinateConverter,
        outline_size: f32,
        outline_style: OutlineStyle,
        solved: bool,
    ) -> Result<(), String> {
        let Some(piece) = self.position.pieces.get(index) else {
            return Ok(());
        };

        let isometry = if solved {
            Isometry::identity()
        } else {
            piece.attitude
                * if let Some(offset) = self.position.animation_offset
                    && piece.in_circle(offset.circle)
                        == Some(crate::complex::complex_circle::Contains::Inside)
                {
                    //get the offset of the piece, base on if its in the animation_offset circle
                    offset.mult(self.position.anim_left as f64).isometry()
                } else {
                    Isometry::identity()
                }
        };

        for triangle in &piece.triangulations {
            triangle.render_outlines(
                ui,
                cc,
                isometry,
                outline_size * outline_style.width(),
                outline_style.color(),
            );
        }

        Ok(())
    }

    pub fn render_piece_annotation(
        &self,
        index: usize,
        ui: &mut Ui,
        cc: CoordinateConverter,
        solved: bool,
    ) -> Result<(), String> {
        let Some(piece) = self.position.pieces.get(index) else {
            return Ok(());
        };

        let isometry = if solved {
            Isometry::identity()
        } else {
            piece.attitude
                * if let Some(offset) = self.position.animation_offset
                    && piece.in_circle(offset.circle)
                        == Some(crate::complex::complex_circle::Contains::Inside)
                {
                    //get the offset of the piece, base on if its in the animation_offset circle
                    offset.mult(self.position.anim_left as f64).isometry()
                } else {
                    Isometry::identity()
                }
        };

        let Some(super_data) = self.super_data.as_ref() else {
            return Ok(());
        };

        let Some(annotation) = super_data.get_annotation(index) else {
            return Ok(());
        };

        match annotation {
            Annotation::Arrow => {
                let angle = isometry.rotation_angle();
                let barycenter = piece.barycenter() * isometry;
                let mut mesh = epaint::Mesh::default(); //make a new mesh
                mesh.indices = vec![0, 1, 2, 0, 2, 3];
                let zeta = C64::from_angle(PI * 0.4);
                mesh.vertices = vec![
                    zeta.conj(),
                    C64::one(),
                    zeta,
                    ((zeta * zeta).re * 2.0 - 1.0) * C64::one(),
                ]
                .into_iter()
                .map(|z| z * C64::from_angle(angle))
                .map(|z| epaint::Vertex {
                    pos: barycenter.to_pos2(cc)
                        + Vec2 {
                            x: z.im as f32,
                            y: z.re as f32,
                        } * ARROW_LENGTH,
                    uv: pos2(0.0, 0.0),
                    color: ARROW_COLOR,
                })
                .collect(); //add all the vertices
                ui.painter().add(egui::Shape::Mesh(mesh.into())); //paint the triangles
            }
        }

        Ok(())
    }
}

impl Puzzle {
    ///render the puzzle, including outlines
    pub fn render(
        &self,
        ui: &mut Ui,
        cc: CoordinateConverter,
        outline_width: f32,
        solved: bool,
    ) -> Result<(), String> {
        for i in 0..self.position.pieces.len() {
            //render each piece
            self.render_piece(i, ui, cc, solved)?;

            let outline_style = if self
                .control_data
                .tape_groups
                .iter()
                .any(|tape_group| tape_group.contains(&i))
            {
                OutlineStyle::Taped
            } else {
                OutlineStyle::Normal
            };

            self.render_piece_outline(i, ui, cc, outline_width, outline_style, solved)?;
        }

        for tape_group in &self.control_data.tape_groups {
            let mut arc_endpoints = HashMap::new(); // circles as keys is sound because of interning
            let outline_style = OutlineStyle::Normal;

            for i in tape_group {
                let Some(piece) = self.position.pieces.get(*i) else {
                    continue;
                };

                let isometry = if solved {
                    Isometry::identity()
                } else {
                    piece.attitude
                        * if let Some(offset) = self.position.animation_offset
                            && piece.in_circle(offset.circle)
                                == Some(crate::complex::complex_circle::Contains::Inside)
                        {
                            //get the offset of the piece, base on if its in the animation_offset circle
                            offset.mult(self.position.anim_left as f64).isometry()
                        } else {
                            Isometry::identity()
                        }
                };

                for arc in &piece.piece.shape.border {
                    arc_endpoints
                        .entry((
                            FloatOrd(arc.circle.center.0.re),
                            FloatOrd(arc.circle.center.0.im),
                            FloatOrd(arc.circle.r_sq),
                        ))
                        .or_insert(ArcEndpoints::new())
                        .add_arc(*arc * isometry);
                }
            }

            for ae in arc_endpoints.into_values() {
                for arc in ae.read_arcs() {
                    ui.painter().add(PathShape::line(
                        arc.get_polygon(DETAIL)
                            .iter()
                            .map(|p| p.to_pos2(cc))
                            .collect(),
                        Stroke::new(outline_width * outline_style.width(), outline_style.color()),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Render only puzzle annotations
    pub fn render_annotations(
        &self,
        ui: &mut Ui,
        cc: CoordinateConverter,
        solved: bool,
    ) -> Result<(), String> {
        for i in 0..self.position.pieces.len() {
            //render each piece
            self.render_piece_annotation(i, ui, cc, solved)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PuzzleLoadingDataWithMode {
    pub data: PuzzleLoadingData,
    pub is_super: bool,
}

impl DataStorer {
    ///render the data panel on the screen and read input for which button is clicked
    pub fn render_panel(
        &mut self,
        ctx: &egui::Context,
    ) -> Result<Option<PuzzleLoadingDataWithMode>, ()> {
        fn cmp_entries(a: &DefEntry, b: &DefEntry) -> Ordering {
            match (a, b) {
                (DefEntry::Def(data_a), DefEntry::Def(data_b)) => {
                    String::cmp(&data_a.name, &data_b.name)
                }
                (DefEntry::Def(_), DefEntry::Folder(_)) => Ordering::Less,
                (DefEntry::Folder(_), DefEntry::Def(_)) => Ordering::Greater,
                (DefEntry::Folder((na, _)), DefEntry::Folder((nb, _))) => OsString::cmp(na, nb),
            }
        }
        fn render_def_entry(entry: &DefEntry, ui: &mut Ui) -> Option<PuzzleLoadingDataWithMode> {
            match entry {
                DefEntry::Def(data) => {
                    let response = ui.add(egui::Button::new(data.name.clone()));

                    let context_response = Popup::context_menu(&response)
                        .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
                        .show(|ui| {
                            if ui.button("Standard mode").clicked() {
                                return Some(false);
                            }
                            if ui.button("Super mode").clicked() {
                                return Some(true);
                            }
                            None
                        });

                    let is_super = if let Some(context_response) = context_response {
                        context_response.inner?
                    } else if response.clicked() {
                        false
                    } else {
                        return None;
                    };

                    Some(PuzzleLoadingDataWithMode {
                        data: data.clone(),
                        is_super,
                    })
                }
                DefEntry::Folder((name, dirs)) => {
                    if let Some(x) = ui
                        .collapsing(name.to_string_lossy(), |inner_ui| {
                            let mut ret = None;
                            let mut sorted_dirs =
                                dirs.clone().into_values().collect::<Vec<DefEntry>>();
                            sorted_dirs.sort_by(|a, b| cmp_entries(a, b));
                            for v in &sorted_dirs {
                                if let Some(x) = render_def_entry(v, inner_ui) {
                                    ret = Some(x);
                                }
                            }
                            ret
                        })
                        .body_returned
                    {
                        x
                    } else {
                        None
                    }
                }
            }
        }
        let panel = egui::SidePanel::new(egui::panel::Side::Right, "data_panel").resizable(false); //make the new panel
        Ok(panel
            .show(ctx, |ui| {
                ui.label(RichText::new("Puzzles").font(FontId::proportional(20.0)));
                //button to reload the puzzles into the data_storer if they were modifed (doing this every frame is too costly)
                if ui.add(egui::Button::new("Reload Puzzle List")).clicked() {
                    if let Err(x) = self.reset(false) {
                        return Err(x);
                    }
                    let _ = self.load_puzzles(DEF_PATH);
                    let _ = self.load_keybinds("Configs/keybinds.kdl");
                }
                if ui
                    .add(egui::Button::new("Load Experimental Puzzles"))
                    .clicked()
                {
                    if let Err(x) = self.reset(true) {
                        return Err(x);
                    }
                    let _ = self.load_puzzles(DEF_PATH);
                    let _ = self.load_keybinds("Configs/keybinds.kdl");
                }
                ui.separator();
                Ok(egui::ScrollArea::vertical()
                    .show(ui, |ui| {
                        let puzzles_real = self.puzzles.lock().unwrap();
                        render_def_entry(&puzzles_real, ui)
                    })
                    .inner)
            })
            .inner
            .or(Err(()))?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct FloatOrd(f64);

impl Eq for FloatOrd {}

impl Ord for FloatOrd {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Hash for FloatOrd {
    fn hash<H>(&self, h: &mut H)
    where
        H: Hasher,
    {
        h.write_u64(self.0.to_bits());
    }
}

struct ArcEndpoints {
    circle: Circle,
    intern: FloatPool,
    endpoints: BinaryHeap<FloatOrd>, // [-π, π]
}

impl ArcEndpoints {
    fn new() -> Self {
        Self {
            circle: Circle {
                center: Point(C64::zero()),
                r_sq: 0.0,
            }, // dummy circle
            intern: FloatPool::new(PRECISION),
            endpoints: BinaryHeap::new(),
        }
    }

    fn add_arc(&mut self, arc: Arc) {
        self.circle = arc.circle;

        let mut start = (arc.start - arc.circle.center).angle(); // [-π, π]
        let mut end = start + arc.angle;
        if arc.angle > 0.0 {
            std::mem::swap(&mut start, &mut end);
        }

        loop {
            let midpoint = (start + end) / 2.0;
            if midpoint < -PI - 1e-8 {
                start += 2.0 * PI;
                end += 2.0 * PI;
                continue;
            }
            if midpoint > PI + 1e-8 {
                start -= 2.0 * PI;
                end -= 2.0 * PI;
                continue;
            }
            break;
        }

        if start < -PI {
            self.add_endpoint(start + 2.0 * PI);
            self.add_endpoint(PI);
        } else {
            self.add_endpoint(start);
        }

        if end > PI {
            self.add_endpoint(end - 2.0 * PI);
            self.add_endpoint(-PI);
        } else {
            self.add_endpoint(end);
        }
    }

    fn add_endpoint(&mut self, endpoint: f64) {
        let endpoint = self.intern.intern(endpoint);
        self.endpoints.push(FloatOrd(endpoint));
    }

    fn read_arcs(self) -> Vec<Arc> {
        // A better language would make doing this without allocation easy
        let endpoints = self.endpoints.into_sorted_vec();

        let mut dedup = Vec::new();
        for endpoint in endpoints {
            if dedup.first().is_some_and(|ep| *ep == endpoint) {
                dedup.pop();
            } else {
                dedup.push(endpoint);
            }
        }

        dedup
            .as_chunks()
            .0
            .into_iter()
            .map(|[FloatOrd(start), FloatOrd(end)]| Arc {
                circle: self.circle,
                angle: end - start,
                start: self.circle.center
                    + self.circle.r_sq.sqrt() * Vector(C64::from_angle(*start)),
            })
            .collect()
    }
}
