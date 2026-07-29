use std::ffi::OsString;
use std::path::PathBuf;

use crate::complex::point::Point;
use crate::hps::data_storer::data_storer::DataStorer;
use crate::puzzle::puzzle::*;
use crate::puzzle::super_data::SetOrAll;
use crate::ui::render::{CoordinateConverter, OutlineStyle, draw_circle};
use crate::{DEF_PATH, DEFAULT_PUZZLE};
use egui::*;

///default scale factor
const SCALE_FACTOR: f32 = 500.0;
///default animation speed
const ANIMATION_SPEED: f64 = 5.0;
///credits string
const CREDITS: &str = "Created by Henry Pickle,
with major help from:
Luna Harran (sonicpineapple)
Andrew Farkas (HactarCE)
cryofractal
Milo Jacquet";

#[derive(Debug)]
///used for running the app. contains all puzzle and view data at runtime
pub struct App {
    data_storer: Option<DataStorer>, //stores the data for the puzzles (on the right panel)
    puzzle: Option<Puzzle>,          //stores the puzzle
    log_path: String,                //stores the path log files are loaded from/saved to
    curr_msg: String,                //current message (usually for errors)
    animation_speed: f64,            //speed at which animations happen
    last_frame_time: web_time::Instant, //the absolute time at which the last frame happened
    outline_width: f32,              //the width of the outlines
    scale_factor: f32,               //the scale factor (zoom)
    offset: Vec2,                    //the offset of the puzzle from the center of the screen (pan)
    cut_on_turn: bool,               //whether or not turns should cut the puzzle
    preview: bool,                   //whether the solved state is being previewed
    mouse_function: MouseFunction,   // Current hover function
    hovered_piece: Option<usize>,    // Index of the hovered piece from the previous frame
    inserting_on_drag: Option<bool>, // Whether the drag is inserting or removing from the set in SuperData.
}
impl App {
    ///initialize a new app, using some default settings (from the constants)
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut data_storer = DataStorer::new(false).ok(); //initialize a new data storer
        let p = if let Some(ref mut ds) = data_storer {
            let _ = ds.load_puzzles(
                DEF_PATH,
                //"Configs/Keybinds/Puzzles/",
                //"Configs/Keybinds/groups.kdl",
            );
            let _ = ds.load_keybinds("Configs/keybinds.kdl");
            let p_data = &ds
                .puzzles
                .lock()
                .unwrap()
                .get(&PathBuf::from(DEFAULT_PUZZLE));
            if let Some(in_data) = p_data.clone() {
                in_data
                    .load(
                        &mut ds.rt,
                        ds.keybinds
                            .get_keybinds_for_puzzle(&in_data.path.file_name().unwrap()),
                    )
                    .ok()
            } else {
                None
            }
        } else {
            None
        };
        Self {
            //return the default app
            data_storer,
            puzzle: p.map(|p| Puzzle::new(p, false)),
            log_path: String::from("logfile"),
            curr_msg: String::new(),
            animation_speed: ANIMATION_SPEED,
            last_frame_time: web_time::Instant::now(),
            outline_width: 5.0,
            scale_factor: SCALE_FACTOR,
            offset: vec2(0.0, 0.0),
            cut_on_turn: false,
            preview: false,
            mouse_function: MouseFunction::Normal,
            hovered_piece: None,
            inserting_on_drag: None,
            // keybinds: if let Some(kb) = &p_data.keybinds
            //     && let Some(gr) = &p_data.keybind_groups
            //     && let Some(keybinds) = load_keybinds(&kb, &gr)
            // {
            //     Some(keybinds.clone())
            // } else {
            //     None
            // },
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MouseFunction {
    Normal,
    Super(SuperMouseFunction),
}

#[derive(Debug, Copy, Clone)]
pub enum SuperMouseFunction {
    OrientationColor,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //run the ui of the program on a central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap(); //the space the program has to work with
            let cc = CoordinateConverter {
                rect,
                scale_factor: self.scale_factor,
                offset_pos: self.offset,
            };

            // Render the puzzle on the bottom layer
            if let Some(puzzle) = self.puzzle.as_mut() {
                if let Err(x) = puzzle.render(ui, cc, self.outline_width, self.preview) {
                    self.curr_msg = x;
                };

                // Render the hovered outline on top of the normal outline
                if let Some(hovered_piece) = self.hovered_piece {
                    if let Err(x) = puzzle.render_piece(
                        hovered_piece,
                        ui,
                        cc,
                        self.outline_width,
                        OutlineStyle::Hovered,
                        self.preview,
                    ) {
                        self.curr_msg = x;
                    }

                    if puzzle.super_data.is_some() {
                        if let Err(x) = puzzle.render_piece(
                            hovered_piece,
                            ui,
                            cc,
                            self.outline_width,
                            OutlineStyle::HoveredSecondary,
                            !self.preview,
                        ) {
                            self.curr_msg = x;
                        }
                    }
                }
            }

            // Manage puzzle animation
            let delta_time = self.last_frame_time.elapsed(); //the time since the last frame
            self.last_frame_time = web_time::Instant::now(); //reset the time tracker
            if let Some(ref mut p) = self.puzzle
                && p.position.anim_left >= 0.0
            {
                //if the animation is still running, advance it according to delta_time and the animation speed
                p.position.anim_left = f32::max(
                    p.position.anim_left - (delta_time.as_secs_f32() * self.animation_speed as f32),
                    0.0,
                );

                ui.ctx().request_repaint();
            }
            if 24.9 < self.animation_speed
                && let Some(ref mut p) = self.puzzle
            {
                //if the animation speed is fast enough, remove animations entirely
                p.position.animation_offset = None;
            }

            //render the data storer panel -- this stores all of the puzzles that you can load
            if let Some(ref mut ds) = self.data_storer {
                match ds.render_panel(ctx) {
                    Err(()) => {
                        self.curr_msg =
                            String::from("Failed to render side panel or failed to create puzzle!")
                    }
                    Ok(Some(puzzle_data)) => {
                        //if a puzzle is returned (a button is clicked), load it
                        match puzzle_data.data.load(
                            &mut ds.rt,
                            ds.keybinds.get_keybinds_for_puzzle(&OsString::from(
                                puzzle_data.data.path.file_name().unwrap(),
                            )),
                        ) {
                            Ok(puz_data) => {
                                self.puzzle = Some(Puzzle::new(puz_data, puzzle_data.is_super))
                            }
                            Err(diag) => self.curr_msg = diag.msg.to_string(),
                        }
                        // if let Some(kb) = puzzle_data.keybinds
                        //     && let Some(gr) = puzzle_data.keybind_groups
                        //     && let Some(keybinds) = load_keybinds(&kb, &gr)
                        // {
                        //     self.keybinds = Some(keybinds);
                        // } else {
                        //     self.keybinds = None;
                        // }
                    }
                    _ => {}
                }
            } else {
                self.curr_msg = String::from("Error loading data storer!");
            }

            //self.curr_msg = String::from("HI");
            //UI Section: menu bar
            egui::MenuBar::new().ui(ui, |ui| {
                //file menu controls save/loading logs
                let file_button = default_menu_button("File");
                file_button.ui(ui, |ui| {
                    //field for adding log path
                    ui.label("Log File Path");
                    ui.add(egui::TextEdit::singleline(&mut self.log_path));
                    //saving, does not work on web
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.add(egui::Button::new("SAVE")).clicked() {
                        self.curr_msg = if let Some(ref mut ds) = self.data_storer
                            && let Some(ref mut p) = self.puzzle
                        {
                            match ds.save(&self.log_path, p) {
                                Ok(()) => String::from("Saved successfully!"),
                                Err(err) => err.to_string(),
                            }
                        } else {
                            String::from("Cannot save due to missing puzzle or data storer!")
                        }
                    }
                    // //loading, does not work on web
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.add(egui::Button::new("LOAD LOG")).clicked() {
                        if let Some(ref mut ds) = self.data_storer {
                            self.puzzle = ds.load_save(&self.log_path);
                        } else {
                            self.curr_msg =
                                String::from("Cannot load log due to missing data storer!")
                        }
                    }
                });
                //view menu controls view graphics
                let view_button = default_menu_button("View");
                view_button.ui(ui, |ui| {
                    //outline width slider
                    ui.add(
                        egui::Slider::new(&mut self.outline_width, (0.0)..=10.0)
                            .text("Outline Width"),
                    );
                    //animation speed slider
                    ui.add(
                        egui::Slider::new(&mut self.animation_speed, (1.0)..=25.0)
                            .text("Animation Speed"),
                    );
                    //rending size (zoom) slider
                    ui.add(
                        egui::Slider::new(&mut self.scale_factor, (10.0)..=5000.0)
                            .text("Rendering Size"),
                    );
                    //panning sliders
                    ui.add(egui::Slider::new(&mut self.offset.y, (-2.0)..=2.0).text("Move Y"));
                    ui.add(egui::Slider::new(&mut self.offset.x, (-2.0)..=2.0).text("Move X"));
                    //preview solved state toggle
                    ui.checkbox(&mut self.preview, "Preview solved state?");
                    //cut on turn toggle
                    //reset view button
                    if ui.add(egui::Button::new("Reset View")).clicked() {
                        (self.scale_factor, self.offset) = (SCALE_FACTOR, vec2(0.0, 0.0))
                    }
                });
                //scramble menu controls scrambling
                let scramble_button = default_menu_button("Scramble");
                scramble_button.ui(ui, |ui| {
                    //scramble button
                    if ui.add(egui::Button::new("Scramble")).clicked()
                        && !self.preview
                        && let Some(ref mut p) = self.puzzle
                    {
                        let _ = p.scramble(self.cut_on_turn);
                    }
                    //reset button
                    if ui.add(egui::Button::new("Reset")).clicked()
                        && !self.preview
                        && let Some(ref mut p) = self.puzzle
                        && p.reset().is_err()
                    {
                        self.curr_msg = String::from("Reset failed!")
                    };
                });
                //puzzle menu controls puzzle operations
                let puzzle_button = default_menu_button("Puzzle");
                puzzle_button.ui(ui, |ui| {
                    //undo button, also performed using the z key
                    if (ui.add(egui::Button::new("Undo Move")).clicked())
                        && !self.preview
                        && let Some(ref mut p) = self.puzzle
                    {
                        let _ = p.undo();
                    }
                    ui.checkbox(&mut self.cut_on_turn, "Cut on turn?");
                    if ui.add(egui::Button::new("Check Solved")).clicked()
                        && let Some(ref mut p) = self.puzzle
                    {
                        p.check();
                    }
                });
                //credits menu displays credits (bugged?)
                let credits_button = default_menu_button("Credits");
                credits_button.ui(ui, |ui| {
                    //display the credits
                    ui.label(CREDITS);
                    ui.separator();
                    //display the puzzle contributors
                    // ui.label(RichText::new("Top puzzle contributors:").color(egui::Color32::WHITE));
                    // match self
                    //     .data_storer
                    //     .get_top_authors::<{ crate::ui::data_storer::TOP }>()
                    // {
                    //     //add the labels for the top 5 puzzle contributors
                    //     Ok(top) => {
                    //         for t in top {
                    //             ui.label(format!("{}: {}", t.0, t.1));
                    //         }
                    //     }
                    //     Err(_) => {}
                    // }
                });
            });

            //UI Section: display puzzle info
            if let Some(ref mut p) = self.puzzle {
                Window::new("Puzzle Info")
                    .default_pos((10.0, 40.0))
                    .auto_sized()
                    .show(ctx, |ui| {
                        ui.label(String::from("Name: ") + &p.name);
                        ui.label(String::from("Authors: ") + &p.authors.join(", "));
                        ui.label(p.position.pieces.len().to_string() + " pieces");
                    });

                if let Some(super_data) = &mut p.super_data {
                    Window::new("Super Data")
                        .default_pos((10.0, 200.0))
                        .auto_sized()
                        .show(ctx, |ui| {
                            if matches!(
                                self.mouse_function,
                                MouseFunction::Super(SuperMouseFunction::OrientationColor)
                            ) {
                                if ui.button(String::from("Move mode")).clicked() {
                                    self.mouse_function = MouseFunction::Normal;
                                };
                                if super_data.orientation_colored.is_all() {
                                    if ui.button(String::from("Deselect all")).clicked() {
                                        super_data.orientation_colored = SetOrAll::empty();
                                    };
                                } else {
                                    if ui.button(String::from("Select all")).clicked() {
                                        super_data.orientation_colored = SetOrAll::All;
                                    };
                                }
                            } else {
                                if ui
                                    .button(String::from("Orientation color toggle mode"))
                                    .clicked()
                                {
                                    self.mouse_function =
                                        MouseFunction::Super(SuperMouseFunction::OrientationColor);
                                };
                            }
                        });
                }
            }

            //UI Section: Bottom left area
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::Frame::popup(ui.style())
                    .stroke(Stroke::NONE)
                    .shadow(Shadow::NONE)
                    .show(ui, |ui| {
                        ui.set_max_width(200.0);
                        ui.separator();
                        if let Some(ref p) = self.puzzle {
                            //displays move count
                            ui.label(p.position.stack.len().to_string() + " ETM");
                            //if the puzzle is solved, display as much (this is currently not working)
                            if p.position.solved {
                                ui.label("Solved!");
                            }
                        }
                        //display the current message if it isn't empty
                        if !self.curr_msg.is_empty() {
                            ui.label(&self.curr_msg);
                        }
                    });
            });

            //gets the rect for interaction with the puzzle (so that ui elements like buttons dont conflict with puzzle input)
            let cor_rect = Rect {
                min: pos2(180.0, 30.0),
                max: pos2(rect.width() - 180.0, rect.height()),
            };

            //get the interactor
            let r = ui.interact(cor_rect, egui::Id::new(19), egui::Sense::all());
            let _ = self.process_event(&r, ctx, ui, cc);
        });
    }
}

impl App {
    fn set_message_err<T>(&mut self, value: Result<T, String>) -> Option<T> {
        match value {
            Ok(value) => Some(value),
            Err(msg) => {
                self.curr_msg = msg;
                None
            }
        }
    }

    fn process_event(
        &mut self,
        r: &Response,
        ctx: &Context,
        ui: &mut Ui,
        cc: CoordinateConverter,
    ) -> Option<()> {
        let puzzle = self.puzzle.as_mut()?;

        // Interactions that work regardless of mode

        if ui.input(|i: &InputState| i.key_pressed(egui::Key::Z)) {
            let _ = puzzle.undo();
            return Some(());
        }

        let mouse = mouse_interaction(&r, ctx, ui, cc)?;

        if let MouseInteractionType::Scroll(scroll) = mouse.typ
            && mouse.ctrl
        {
            self.scale_factor += 10.0 * scroll as f32;
            return Some(());
        }

        //if the middle mouse button is being pressed, pan the camera
        if r.dragged_by(egui::PointerButton::Middle) {
            let delta = r.drag_delta();
            let good_delta = vec2(delta.x / self.scale_factor, -(delta.y / self.scale_factor));
            self.offset += good_delta;
        }

        //keybinds
        if ui.ctx().memory(|x| x.focused().is_none()) {
            let ev = ctx.input(|i| i.events.clone());
            for event in ev {
                if let Event::Key {
                    key,
                    physical_key,
                    pressed,
                    repeat: _,
                    modifiers: _,
                } = event
                {
                    let b = if let Some(p) = physical_key { p } else { key };
                    if pressed
                        && let Some((t, m)) = puzzle.keybinds.get(&b).cloned()
                        && puzzle.turns.contains_key(&t)
                    {
                        if let Err(x) = puzzle.turn_id(&t, self.cut_on_turn, m) {
                            self.curr_msg = x;
                        }

                        return Some(());
                    }
                }
            }
        }

        self.hovered_piece = None; // it will be set below if necessary

        // Mode-dependent interactions

        // let mut error_message = None;
        match self.mouse_function {
            MouseFunction::Normal => match mouse.typ {
                MouseInteractionType::Click => {
                    if let Some((turn_id, _)) = puzzle.turn_at_point(mouse.position) {
                        let turn_result = puzzle.turn_id(&turn_id, self.cut_on_turn, -1);
                        self.set_message_err(turn_result);
                    }
                }
                MouseInteractionType::SecondaryClick => {
                    if let Some((turn_id, _)) = puzzle.turn_at_point(mouse.position) {
                        let turn_result = puzzle.turn_id(&turn_id, self.cut_on_turn, 1);
                        self.set_message_err(turn_result);
                    }
                }
                MouseInteractionType::Hover => {
                    if mouse.shift {
                        if !puzzle.in_animation() {
                            // Block hovering if animation is active
                            // Shift held, highlight a piece
                            self.hovered_piece = puzzle.piece_at_point(mouse.position, false);
                        }
                    } else {
                        if let Some((_, turn)) = puzzle.turn_at_point(mouse.position) {
                            draw_circle(turn.turn.circle, ui, cc);
                        }
                    }
                }
                MouseInteractionType::DragOver => {}
                MouseInteractionType::Scroll(scroll) => {
                    if let Some((turn_id, _)) = puzzle.turn_at_point(mouse.position) {
                        let turn_result =
                            puzzle.turn_id(&turn_id, self.cut_on_turn, scroll as isize);
                        self.set_message_err(turn_result);
                    }
                }
            },
            MouseFunction::Super(mf) => {
                let hovered_piece = puzzle.piece_at_point(mouse.position, false);
                let super_data = puzzle.super_data.as_mut()?;

                match mf {
                    SuperMouseFunction::OrientationColor => match mouse.typ {
                        MouseInteractionType::Click => {
                            if let Some(hovered_piece) = hovered_piece {
                                super_data.orientation_colored.toggle(hovered_piece);
                            }
                        }
                        MouseInteractionType::SecondaryClick => {}
                        MouseInteractionType::Hover => {
                            self.hovered_piece = puzzle.piece_at_point(mouse.position, false);
                        }
                        MouseInteractionType::DragOver => {
                            if let Some(hovered_piece) = hovered_piece {
                                match self.inserting_on_drag {
                                    Some(insert) => {
                                        super_data
                                            .orientation_colored
                                            .toggle_to(hovered_piece, insert);
                                    }
                                    None => {
                                        self.inserting_on_drag = Some(
                                            super_data.orientation_colored.toggle(hovered_piece),
                                        )
                                    }
                                }
                            }
                        }
                        MouseInteractionType::Scroll(_) => {}
                    },
                }
            }
        }

        if !matches!(mouse.typ, MouseInteractionType::DragOver) {
            // Done dragging
            self.inserting_on_drag = None;
        }

        Some(())
    }
}

//make a menu button that isn't trash
fn default_menu_button<'a>(text: &'a str) -> egui::containers::menu::MenuButton<'a> {
    let button = egui::containers::menu::MenuButton::new(text);
    let config = egui::containers::menu::MenuConfig::new();
    button.config(config.close_behavior(PopupCloseBehavior::CloseOnClickOutside))
}

#[derive(Debug, Clone, Copy)]
struct MouseInteraction {
    position: Point,
    typ: MouseInteractionType,
    shift: bool,
    ctrl: bool,
}

#[derive(Debug, Clone, Copy)]
enum MouseInteractionType {
    Click,
    SecondaryClick,
    Hover,
    DragOver,
    Scroll(i32),
}

fn mouse_interaction(
    r: &Response,
    ctx: &Context,
    ui: &mut Ui,
    cc: CoordinateConverter,
) -> Option<MouseInteraction> {
    let shift = ctx.input(|i| i.modifiers.shift);
    let ctrl = ctx.input(|i| i.modifiers.shift);
    let dragging = ctx.input(|i| i.pointer.is_decidedly_dragging());

    let scroll = ui.input(|input| {
        input
            .raw
            .events
            .iter()
            .filter_map(|ev| match ev {
                Event::MouseWheel {
                    unit: MouseWheelUnit::Line | MouseWheelUnit::Page,
                    delta,
                    modifiers: _,
                } => Some((delta.x + delta.y).signum() as i32),
                _ => None,
            })
            .sum::<i32>()
    });

    let typ;
    let raw_position;
    if scroll != 0 {
        raw_position = r.hover_pos();
        typ = MouseInteractionType::Scroll(scroll);
    } else {
        if dragging {
            raw_position = r.hover_pos();
            typ = MouseInteractionType::DragOver;
        } else if r.clicked() {
            raw_position = r.interact_pointer_pos();
            typ = MouseInteractionType::Click;
        } else if r.secondary_clicked() {
            raw_position = r.interact_pointer_pos();
            typ = MouseInteractionType::SecondaryClick;
        } else {
            raw_position = r.hover_pos();
            typ = MouseInteractionType::Hover;
        }
    }

    let position = Point::from_pos2(&raw_position?, cc);

    Some(MouseInteraction {
        position,
        typ,
        shift,
        ctrl,
    })
}
