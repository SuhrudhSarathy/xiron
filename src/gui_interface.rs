use egui_macroquad::egui::{self, Button, Layout, TopBottomPanel, Window};
use egui_macroquad::egui::{Context, Visuals};
use macroquad::prelude::*;
use std::collections::HashMap;
use std::future::Future;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::prelude::{
    ObjectParameterType, Robot, RobotHandler, SelectedObjectType, SimulationHandler, StaticObj,
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

pub struct EguiInterface {
    pub clicked_mode: Mode,
    pub nearest_object_index: (Option<SelectedObjectType>, i32),
    pub play: PlayMode,
    pub object_select_mode: ObjectSelectMode,

    // Sender for filebox
    pub open_file_path_sender: Sender<String>,
    pub save_file_path_sender: Sender<String>,

    sim_handler: Arc<Mutex<SimulationHandler>>,
    robot_handlers: Vec<RobotHandler>,
    robot_name_map: HashMap<String, RobotHandler>,
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
        }
    }

    pub fn show_elements(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("FileEditViewBar").show(ctx, |ui| self.draw_file_top_bar(ctx, ui));
        TopBottomPanel::top("MainTopBar").show(ctx, |ui| self.draw_top_bar_elements(ui));
        TopBottomPanel::bottom("Play-Pause Button").show(ctx, |ui| self.handle_bottom_bar(ui));

        // draw stuff
        self.add_objects_from_top_bar();

        // Deal with clicks on Objects
        self.deal_with_click_on_objects(ctx);
    }

    fn draw_file_top_bar(&mut self, ctx: &Context, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                let save_config_button = ui.button("Save Config");
                let open_config_button = ui.button("Open Config");
                ui.separator();
                let close_button = ui.button("Close Simulator");

                if close_button.clicked() {
                    std::process::exit(0);
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
                    // TODO: This is buggy
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
        });
    }

    fn draw_top_bar_elements(&mut self, ui: &mut egui::Ui) {
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

                if is_mouse_button_down(MouseButton::Right) {
                    self.object_select_mode = ObjectSelectMode::None;
                }
            });
        });
    }

    fn add_objects_from_top_bar(&mut self) {
        let (mx, my) = mouse_position();

        if self.clicked_mode == Mode::Robot {
            draw_circle(mx, my, 25.0, BLACK);
        } else if self.clicked_mode == Mode::StaticObj {
            draw_rectangle(mx - 25.0, my - 50.0, 50.0, 100.0, BLACK);
        }

        let mut sh = self.sim_handler.lock().unwrap();

        if is_mouse_button_down(MouseButton::Left) {
            let (x, y) = SimulationHandler::get_world_from_pixel(mx, my);
            if self.clicked_mode == Mode::Robot {
                let robot_id = format!("robot{}", self.robot_handlers.len());
                println!("Radius: {:?}", SimulationHandler::scale_function(25.0));
                let (_, robot_handler) = sh.add_robot(Robot::from_id_and_pose(
                    robot_id.clone(),
                    (x, y, 0.0),
                    SimulationHandler::scale_function(25.0),
                ));

                self.robot_handlers.push(robot_handler);
                self.robot_name_map.insert(robot_id, robot_handler);
            } else if self.clicked_mode == Mode::StaticObj {
                sh.add_static_obj(StaticObj::new(
                    (x, y),
                    SimulationHandler::scale_function(50.0),
                    SimulationHandler::scale_function(100.0),
                    0.0,
                ))
            }

            self.clicked_mode = Mode::None;
        } else if is_mouse_button_down(MouseButton::Right) {
            self.clicked_mode = Mode::None;
        }
    }

    fn deal_with_click_on_objects(&mut self, ctx: &egui::Context) {
        let (mx, my) = mouse_position();
        let (x, y) = SimulationHandler::get_world_from_pixel(mx, my);

        let mut sh = self.sim_handler.lock().unwrap();

        // We are doing this in order toupdate the index only when we click near some object.
        let did_we_get_nearest_object = sh.get_nearest_object(x, y);
        match did_we_get_nearest_object.0 {
            Some(_r) => {
                if is_mouse_button_down(MouseButton::Left) {
                    self.nearest_object_index = did_we_get_nearest_object;
                    println!("Got nearest object");
                }
            }
            None => {}
        }

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
        if is_mouse_button_down(MouseButton::Right) {
            self.nearest_object_index = (None, -1);
        }
    }

    fn handle_bottom_bar(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            ui.with_layout(
                egui::Layout::left_to_right(egui::Align::TOP),
                |ui| match self.play {
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
                },
            );
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
