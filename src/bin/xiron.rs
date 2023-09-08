use egui_macroquad::egui::{
    self, Button, Layout, RichText, SidePanel, TextBuffer, TopBottomPanel, Window,
};
use egui_macroquad::egui::{Context, Visuals};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::mpsc::Sender;
use xiron::prelude::*;

trait Object {
    fn modify_bounds(&mut self, width: f32, height: f32);
    fn modify_rotation(&mut self, angle: f32);
    fn modify_position(&mut self, x: f32, y: f32);

    fn draw(&self);

    fn get_center(&self) -> (f32, f32);
    fn get_rotation(&self) -> f32;
    fn get_bounds(&self) -> (f32, f32);
}

#[derive(Deserialize, Serialize)]
struct VelocityQuery {
    robot_id: String,
    velocity: (f32, f32),
}

#[derive(Deserialize, Serialize)]
struct PoseQuery {
    robot_id: String,
    pose: (f32, f32, f32),
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct CircleObj {
    pub radius: f32,
    pub center: (f32, f32),
}

impl CircleObj {
    pub fn new(radius: f32, center: (f32, f32)) -> Self {
        Self {
            radius: radius,
            center: center,
        }
    }
}

impl Object for CircleObj {
    fn modify_bounds(&mut self, width: f32, _height: f32) {
        self.radius = width;
    }
    fn modify_position(&mut self, x: f32, y: f32) {
        self.center = (x, y);
    }
    fn modify_rotation(&mut self, _angle: f32) {}

    fn draw(&self) {
        draw_circle(self.center.0, self.center.1, self.radius, RED);
    }

    fn get_center(&self) -> (f32, f32) {
        (self.center.0, self.center.1)
    }

    fn get_rotation(&self) -> f32 {
        0.0
    }

    fn get_bounds(&self) -> (f32, f32) {
        (self.radius, self.radius)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct RectangleObj {
    pub width: f32,
    pub height: f32,
    pub center: (f32, f32),
    pub rotation_angle: f32,
}

impl RectangleObj {
    pub fn new(width: f32, height: f32, center: (f32, f32)) -> Self {
        Self {
            width: width,
            height: height,
            center: center,
            rotation_angle: 0.0,
        }
    }
}

impl Object for RectangleObj {
    fn modify_bounds(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    fn modify_position(&mut self, x: f32, y: f32) {
        self.center = (x, y);
    }

    fn modify_rotation(&mut self, angle: f32) {
        self.rotation_angle = angle;
    }

    fn draw(&self) {
        let w = self.width * 0.5;
        let h = self.height * 0.5;

        let c = self.rotation_angle.cos();
        let s = self.rotation_angle.sin();

        let x1 = self.center.0 + w * c - h * s;
        let y1 = self.center.1 + w * s + h * c;

        let x2 = self.center.0 - w * c - h * s;
        let y2 = self.center.1 - w * s + h * c;

        let x3 = self.center.0 - w * c + h * s;
        let y3 = self.center.1 - w * s - h * c;

        let x4 = self.center.0 + w * c + h * s;
        let y4 = self.center.1 + w * s - h * c;

        // Draw the body
        draw_triangle(
            Vec2 { x: x1, y: y1 },
            Vec2 { x: x2, y: y2 },
            Vec2 { x: x3, y: y3 },
            GREEN,
        );
        draw_triangle(
            Vec2 { x: x1, y: y1 },
            Vec2 { x: x3, y: y3 },
            Vec2 { x: x4, y: y4 },
            GREEN,
        );
    }

    fn get_center(&self) -> (f32, f32) {
        self.center
    }

    fn get_bounds(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    fn get_rotation(&self) -> f32 {
        self.rotation_angle
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Mode {
    Circle,
    Rectangle,
    None,
}

enum PlayMode {
    Play,
    Pause,
}

struct EguiInterface {
    pub clicked_mode: Mode,
    pub object_vector: Vec<Box<dyn Object>>,
    pub nearest_object_index: Option<usize>,
    pub play: PlayMode,

    // Sender for filebox
    pub open_file_path_sender: Sender<String>,
    pub save_file_path_sender: Sender<String>,
}

impl EguiInterface {
    fn new(open_sender: Sender<String>, save_sender: Sender<String>) -> EguiInterface {
        EguiInterface {
            clicked_mode: Mode::None,
            object_vector: Vec::new(),
            nearest_object_index: None,
            play: PlayMode::Play,

            open_file_path_sender: open_sender,
            save_file_path_sender: save_sender,
        }
    }

    fn show_elements(&mut self, ctx: &egui::Context) {
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
            let circle_button = ui.add(egui::Button::new("Add Circle ⏺"));
            if circle_button.clicked() {
                self.clicked_mode = Mode::Circle;
            }

            let rectangle_button = ui.add(egui::Button::new("Add Rectangle ▭"));
            if rectangle_button.clicked() {
                self.clicked_mode = Mode::Rectangle;
            }
        });
    }

    fn add_objects_from_top_bar(&mut self) {
        let (mx, my) = mouse_position();

        if self.clicked_mode == Mode::Circle {
            draw_circle(mx, my, 25.0, RED);
        } else if self.clicked_mode == Mode::Rectangle {
            draw_rectangle(mx - 25.0, my - 50.0, 50.0, 100.0, GREEN);
        }

        if is_mouse_button_down(MouseButton::Left) {
            if self.clicked_mode == Mode::Circle {
                self.object_vector
                    .push(Box::new(CircleObj::new(25.0, (mx, my))));
            } else if self.clicked_mode == Mode::Rectangle {
                self.object_vector
                    .push(Box::new(RectangleObj::new(50.0, 100.0, (mx, my))));
            }

            self.clicked_mode = Mode::None;
        } else if is_mouse_button_down(MouseButton::Right) {
            self.clicked_mode = Mode::None;
        }

        for obj in self.object_vector.iter() {
            obj.draw();
        }
    }

    fn deal_with_click_on_objects(&mut self, ctx: &egui::Context) {
        let (mx, my) = mouse_position();

        for obj in self.object_vector.iter().enumerate() {
            let center = obj.1.get_center();
            let (width, height) = obj.1.get_bounds();
            if (mx - center.0).abs() < 0.25 * width && (my - center.1).abs() < 0.25 * height {
                self.nearest_object_index = Some(obj.0);
            }
        }

        match self.nearest_object_index {
            None => {}
            Some(obj) => {
                Window::new("Configuration")
                    .min_width(250.0)
                    .collapsible(true)
                    .show(ctx, |ui| {
                        ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
                            ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                                ui.label("Bounds: ");
                                let (mut width, mut height) = self.object_vector[obj].get_bounds();
                                ui.add(egui::DragValue::new(&mut width));
                                ui.add(egui::DragValue::new(&mut height));

                                self.object_vector[obj].modify_bounds(width, height);
                            });

                            ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                                let mut rotation = self.object_vector[obj].get_rotation();
                                ui.label("Rotation: ");
                                ui.add(egui::Slider::new(&mut rotation, -3.14..=3.14));
                                self.object_vector[obj].modify_rotation(rotation);
                            });

                            ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                                ui.label("Center: ");
                                let (mut cx, mut cy) = self.object_vector[obj].get_center();
                                ui.add(egui::DragValue::new(&mut cx));
                                ui.add(egui::DragValue::new(&mut cy));

                                self.object_vector[obj].modify_position(cx, cy);
                            });

                            ui.separator();

                            ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                                ui.label("Delete object Permanently: ");
                                let delete_button = ui.button("Delete");

                                if delete_button.clicked() {
                                    self.object_vector.remove(obj);

                                    self.nearest_object_index = None;
                                }
                            });
                        });
                    });
            }
        }

        if is_mouse_button_down(MouseButton::Right) {
            self.nearest_object_index = None;
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

#[macroquad::main(xiron)]
async fn main() {
    println!("This will be the new simulator");

    let context = zmq::Context::new();
    let publisher = context.socket(zmq::PUB).unwrap();
    publisher
        .bind("tcp://*:8080")
        .expect("Could not bind to the socket");

    let (sender, reciever) = std::sync::mpsc::channel();
    let (save_sender, save_reciever) = std::sync::mpsc::channel();

    let mut sim_handler = SimulationHandler::new();
    let mut egui_handler = EguiInterface::new(sender, save_sender);

    loop {
        clear_background(WHITE);

        send_data(&publisher);

        match reciever.try_recv() {
            Ok(message) => {
                println!("Got Open message here: {}", message);
                sim_handler.load_file_path(message);
                sim_handler.reset();
            }
            Err(_) => {}
        }

        match save_reciever.try_recv() {
            Ok(message) => {
                println!("Got Save message here: {}", message);
                let config = sim_handler.to_config();

                let f = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(message)
                    .expect("Couldn't open file");
                serde_yaml::to_writer(f, &config).unwrap();
            }
            Err(_) => {}
        }

        match egui_handler.play {
            PlayMode::Pause => {}
            PlayMode::Play => {
                sim_handler.step();
            }
        }

        sim_handler.draw();

        // draw the Egui stuff
        egui_macroquad::ui(|egui_ctx| egui_handler.show_elements(egui_ctx));

        egui_macroquad::draw();

        next_frame().await;
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

fn send_data(publisher: &zmq::Socket) {
    let vel = VelocityQuery {
        robot_id: "robot0".to_string(),
        velocity: (0.1, 0.1),
    };
    let vel_as_string = serde_json::to_string(&vel).expect("Could not convert");

    publisher.send(&format!("{}", 10011), zmq::SNDMORE).unwrap();
    publisher.send(&vel_as_string.to_owned(), 0).unwrap();
}
