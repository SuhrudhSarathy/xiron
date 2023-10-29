use egui_macroquad::egui::{self, Button, TopBottomPanel};
use egui_macroquad::egui::{Context, Visuals};
use macroquad::prelude::*;
use std::collections::HashMap;
use std::future::Future;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::prelude::{
    ObjectParameterType, Robot, RobotHandler, SelectedObjectType, SimulationHandler, StaticObj,
    Wall,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mode {
    Robot,
    StaticObj,
    Wall,
    None,
}

pub enum PlayMode {
    Play,
    Pause,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ObjectSelectMode {
    Rotate,
    Bound,
    Center,
    None,
}

#[derive(Debug, PartialEq, Clone)]
pub enum WallDrawStatus {
    WallStart,
    WallMid(Vec<(f32, f32)>),
    Idle,
}

pub struct EguiInterface {
    pub clicked_mode: Mode,
    pub nearest_object_index: (Option<SelectedObjectType>, i32),
    pub play: PlayMode,
    pub object_select_mode: ObjectSelectMode,

    // Sender for filebox
    pub open_file_path_sender: Sender<String>,
    pub save_file_path_sender: Sender<String>,

    sim_handler: Arc<Mutex<SimulationHandler>>,
    pub robot_handlers: Vec<RobotHandler>,
    pub robot_name_map: HashMap<String, RobotHandler>,

    // local variables
    wall_draw_status: WallDrawStatus,
}

impl EguiInterface {
    pub fn new(
        open_sender: Sender<String>,
        save_sender: Sender<String>,
        sim_handler_mutex: Arc<Mutex<SimulationHandler>>,
    ) -> EguiInterface {
        EguiInterface {
            clicked_mode: Mode::None,
            nearest_object_index: (None, -1),
            play: PlayMode::Play,
            sim_handler: sim_handler_mutex,

            open_file_path_sender: open_sender,
            save_file_path_sender: save_sender,
            object_select_mode: ObjectSelectMode::None,

            robot_handlers: Vec::new(),
            robot_name_map: HashMap::new(),

            wall_draw_status: WallDrawStatus::Idle,
        }
    }
    /// Returns the Robot Handler of a given robot string
    pub fn get_robot_handler(&self, robot_id: &String) -> Option<RobotHandler> {
        let handler = self.robot_name_map.get(robot_id);

        match handler {
            Some(h) => return Some(h.clone()),
            None => return None,
        }
    }

    pub fn reset_robot_handlers(&mut self, robot_handlers: Vec<(String, RobotHandler)>) {
        for (name, handler) in robot_handlers.iter() {
            self.robot_handlers.push(handler.clone());
            self.robot_name_map.insert(name.clone(), handler.clone());
        }
    }

    /// Main function for rendinering Egui Elements on the screen
    pub fn show_elements(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("FileEditViewBar")
            .show(ctx, |ui| self.draw_file_edit_view_help_bar(ctx, ui));
        TopBottomPanel::top("MainTopBar")
            .show(ctx, |ui| self.draw_adding_and_modifying_objects_bar(ui));
        TopBottomPanel::bottom("Play-Pause Button")
            .show(ctx, |ui| self.draw_bottom_play_pause_bar(ui));

        // draw stuff
        self.add_selected_objects_to_canvas();

        // Deal with clicks on Objects
        self.deal_with_click_on_objects(ctx);
    }

    /// Draws and handles all Egui elemts for the Top bar containing
    fn draw_file_edit_view_help_bar(&mut self, ctx: &Context, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                let save_config_button = ui.button("Save Config");
                let open_config_button = ui.button("Open Config");
                let reset_simulation_button = ui.button("Reset Simulation");
                ui.separator();
                let close_button = ui.button("Close Simulator");

                if close_button.clicked() {
                    std::process::exit(0);
                }

                if reset_simulation_button.clicked() {
                    // Reset all variables
                    self.nearest_object_index = (None, -1);
                    self.clicked_mode = Mode::None;
                    self.object_select_mode = ObjectSelectMode::None;
                    self.robot_handlers.clear();
                    self.robot_name_map.clear();

                    let mut sh = self.sim_handler.lock().unwrap();
                    let robot_handlers = sh.reset();

                    // Drop sh to make take ownership of self
                    drop(sh);

                    self.reset_robot_handlers(robot_handlers);
                }

                if save_config_button.clicked() {
                    // Spawn dialog on main thread
                    let task = rfd::AsyncFileDialog::new().save_file();
                    let sender = self.save_file_path_sender.clone();
                    // Await somewhere else
                    execute(async move {
                        let file = task.await;

                        if let Some(file) = file {
                            // If you are on native platform you can just get the path
                            #[cfg(not(target_arch = "wasm32"))]
                            println!("{:?}", file.path());

                            let file_path = file.path().to_str();

                            match file_path {
                                None => {}
                                Some(path) => {
                                    sender.send(path.to_string()).ok();
                                }
                            }
                        }
                    });
                }

                if open_config_button.clicked() {
                    // Spawn dialog on main thread
                    let task = rfd::AsyncFileDialog::new().pick_file();
                    let sender = self.open_file_path_sender.clone();
                    // Await somewhere else
                    execute(async move {
                        let file = task.await;

                        if let Some(file) = file {
                            // If you are on native platform you can just get the path
                            #[cfg(not(target_arch = "wasm32"))]
                            println!("{:?}", file.path());

                            let file_path = file.path().to_str();

                            match file_path {
                                None => {}
                                Some(path) => {
                                    sender.send(path.to_string()).ok();
                                }
                            }
                        }
                    });
                }
            });
            ui.menu_button("View", |ui| {
                let dark_mode = ui.visuals().dark_mode;
                let mut mode_string = "Dark Mode";
                if dark_mode {
                    mode_string = "Light Mode";
                }

                let light_mode_button = ui.button(mode_string);

                if light_mode_button.clicked() {
                    let visuals = if ui.visuals().dark_mode {
                        Visuals::light()
                    } else {
                        Visuals::dark()
                    };
                    ctx.set_visuals(visuals);
                }
            });

            ui.menu_button("Help", |ui| {
                let _general_help =
                    ui.hyperlink_to("General", "https://suhrudhsarathy.github.io/xiron/");
                let _interface_help = ui.hyperlink_to(
                    "Python Interface",
                    "https://suhrudhsarathy.github.io/xiron/user_guide/python_interface/",
                );
                ui.separator();
                let _documentation_button =
                    ui.hyperlink_to("Code ", "https://github.com/SuhrudhSarathy/xiron");
            });
        });
    }

    /// Draws buttons for adding Robot, Static Object and Wall to the screen.
    /// Also draws the buttons for Modifying Rotation, Bounds and Position.
    fn draw_adding_and_modifying_objects_bar(&mut self, ui: &mut egui::Ui) {
        // Add a menu bar
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let circle_button = ui.add(egui::Button::new("Add Robot ⏺"));
                if circle_button.clicked() {
                    self.clicked_mode = Mode::Robot;
                }

                let rectangle_button = ui.add(egui::Button::new("Add Static Obj ▭"));
                if rectangle_button.clicked() {
                    self.clicked_mode = Mode::StaticObj;
                }

                let wall_button = ui.add(egui::Button::new("Add Wall ||"));
                if wall_button.clicked() {
                    self.clicked_mode = Mode::Wall;
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_enabled(
                        self.object_select_mode != ObjectSelectMode::Rotate,
                        egui::Button::new("Modify Rotation"),
                    )
                    .clicked()
                {
                    self.object_select_mode = ObjectSelectMode::Rotate;
                }

                if ui
                    .add_enabled(
                        self.object_select_mode != ObjectSelectMode::Bound,
                        egui::Button::new("Modify Bounds"),
                    )
                    .clicked()
                {
                    self.object_select_mode = ObjectSelectMode::Bound;
                }

                if ui
                    .add_enabled(
                        self.object_select_mode != ObjectSelectMode::Center,
                        egui::Button::new("Modify Center"),
                    )
                    .clicked()
                {
                    self.object_select_mode = ObjectSelectMode::Center;
                }

                if is_key_pressed(KeyCode::Escape) {
                    self.object_select_mode = ObjectSelectMode::None;
                }
            });
        });
    }

    /// Draws the selected objects to screen and also updates the Simulation handler when clicked.
    fn add_selected_objects_to_canvas(&mut self) {
        let (mx, my) = mouse_position();
        let mut sh = self.sim_handler.lock().unwrap();

        if self.clicked_mode == Mode::Robot {
            draw_circle(mx, my, 10.0, BLACK);
        } else if self.clicked_mode == Mode::StaticObj {
            draw_rectangle(mx - 12.5, my - 12.5, 25.0, 25.0, GRAY);
        } else if self.clicked_mode == Mode::Wall {
            match &mut self.wall_draw_status {
                WallDrawStatus::Idle => {
                    // This is the first time Wall was selected
                    self.wall_draw_status = WallDrawStatus::WallStart;
                }
                WallDrawStatus::WallStart => {
                    if is_mouse_button_pressed(MouseButton::Left) {
                        let (mx, my) = mouse_position();
                        let wall_coords_vector: Vec<(f32, f32)> = vec![(mx, my)];
                        self.wall_draw_status = WallDrawStatus::WallMid(wall_coords_vector);
                    }
                }
                WallDrawStatus::WallMid(vector) => {
                    for i in 0..vector.len() - 1 {
                        let p0 = vector[i];
                        let p1 = vector[i + 1];

                        // Draw a line for this
                        draw_line(p0.0, p0.1, p1.0, p1.1, 8.0, BLACK);
                    }
                    let p0 = vector[vector.len() - 1];
                    draw_line(p0.0, p0.1, mx, my, 8.0, BLACK);

                    if is_mouse_button_pressed(MouseButton::Left) {
                        vector.push((mx, my));
                    } else if is_mouse_button_pressed(MouseButton::Right) {
                        let mut tfed_pts = Vec::new();
                        for pt in vector.iter() {
                            let out_pt = SimulationHandler::get_world_from_pixel(pt.0, pt.1);
                            tfed_pts.push(out_pt);
                        }

                        let wall = Wall::new(tfed_pts);
                        sh.add_wall(wall);
                        self.wall_draw_status = WallDrawStatus::Idle;
                        self.clicked_mode = Mode::None;
                    } else if is_key_pressed(KeyCode::Escape) {
                        self.wall_draw_status = WallDrawStatus::Idle;
                        self.clicked_mode = Mode::None;
                    }
                }
            }
        }

        if is_mouse_button_down(MouseButton::Left) {
            let (x, y) = SimulationHandler::get_world_from_pixel(mx, my);
            if self.clicked_mode == Mode::Robot {
                let robot_id = format!("robot{}", self.robot_handlers.len());
                let (_, robot_handler) = sh.add_robot(Robot::from_id_and_pose(
                    robot_id.clone(),
                    (x, y, 0.0),
                    SimulationHandler::scale_function(10.0),
                ));

                self.robot_handlers.push(robot_handler);
                self.robot_name_map.insert(robot_id, robot_handler);
                self.clicked_mode = Mode::None;
            } else if self.clicked_mode == Mode::StaticObj {
                sh.add_static_obj(StaticObj::new(
                    (x, y),
                    SimulationHandler::scale_function(25.0),
                    SimulationHandler::scale_function(25.0),
                    0.0,
                ));
                self.clicked_mode = Mode::None;
            }
        } else if is_key_pressed(KeyCode::Escape) {
            self.clicked_mode = Mode::None;
        }
    }

    /// Fucntion to deal when an object is clicked
    fn deal_with_click_on_objects(&mut self, _ctx: &egui::Context) {
        let (mx, my) = mouse_position();
        let (x, y) = SimulationHandler::get_world_from_pixel(mx, my);

        let mut sh = self.sim_handler.lock().unwrap();

        // We are doing this in order to update the index only when we click near some object.
        let did_we_get_nearest_object = sh.get_nearest_object(x, y);
        match did_we_get_nearest_object.0 {
            Some(_r) => {
                if is_mouse_button_down(MouseButton::Left) {
                    self.nearest_object_index = did_we_get_nearest_object;
                    // println!("Got nearest object");
                }
            }
            None => {}
        }

        /*
        Piece of code to draw a green boundary on the selected object
        */
        sh.draw_bounds_of_selected_object(self.nearest_object_index);

        let (object_type, index) = (self.nearest_object_index.0, self.nearest_object_index.1);

        // Now here we will have the object type and index saved together
        match object_type {
            None => {}
            Some(_obj) => {
                if is_mouse_button_pressed(MouseButton::Left) {
                    match self.object_select_mode {
                        ObjectSelectMode::None => {}
                        ObjectSelectMode::Bound => {
                            let (mx, my) = mouse_position();
                            let (wx, wy) = SimulationHandler::get_world_from_pixel(mx, my);
                            let center = sh.get_parameters_of_selected_object(
                                self.nearest_object_index,
                                ObjectParameterType::Position(0.0, 0.0),
                            );
                            match center {
                                ObjectParameterType::Position(x, y) => {
                                    let width = (wx - x).abs();
                                    let heigh = (wy - y).abs();

                                    sh.change_parameters_of_selected_object(
                                        self.nearest_object_index,
                                        ObjectParameterType::Bounds(width, heigh),
                                    );
                                }
                                _ => {}
                            }
                        }
                        ObjectSelectMode::Center => {
                            let (mx, my) = mouse_position();
                            let (wx, wy) = SimulationHandler::get_world_from_pixel(mx, my);
                            sh.change_parameters_of_selected_object(
                                (object_type, index),
                                ObjectParameterType::Position(wx, wy),
                            );
                        }
                        ObjectSelectMode::Rotate => {
                            let (mx, my) = mouse_position();
                            let (wx, wy) = SimulationHandler::get_world_from_pixel(mx, my);
                            let center = sh.get_parameters_of_selected_object(
                                self.nearest_object_index,
                                ObjectParameterType::Position(0.0, 0.0),
                            );
                            match center {
                                ObjectParameterType::Position(x, y) => {
                                    let rotation = (wy - y).atan2(wx - x);
                                    sh.change_parameters_of_selected_object(
                                        self.nearest_object_index,
                                        ObjectParameterType::Rotation(rotation),
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        if is_key_pressed(KeyCode::Escape) {
            self.nearest_object_index = (None, -1);
        }
    }

    /// Function to draw the bottom bar
    fn draw_bottom_play_pause_bar(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                match self.play {
                    PlayMode::Pause => {
                        let button = ui.add(Button::new("Play ▶"));
                        if button.clicked() {
                            self.play = PlayMode::Play;
                        }
                    }

                    PlayMode::Play => {
                        let button = ui.add(Button::new("Pause ⏸"));
                        if button.clicked() {
                            self.play = PlayMode::Pause;
                        }
                    }
                };
            });
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    ui.label(format!("Made with {} by Suhrudh", "♡"));
                },
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ui.label(format!("FPS: {}", get_fps()));
                ui.separator();
                ui.label(format!("Elapsed Time: {:.3}s", get_time()));
            });
        });

        // Put text on the left about the current FPS
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || futures::executor::block_on(f));
}
#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}
