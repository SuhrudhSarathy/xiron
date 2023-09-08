use egui_macroquad::egui::{self, panel::Side, Button, ImageButton, SidePanel, TopBottomPanel};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::mpsc::{Receiver, Sender};
use xiron::prelude::*;

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

fn send_data(publisher: &zmq::Socket) {
    let vel = VelocityQuery {
        robot_id: "robot0".to_string(),
        velocity: (0.1, 0.1),
    };
    let vel_as_string = serde_json::to_string(&vel).expect("Could not convert");

    publisher.send(&format!("{}", 10011), zmq::SNDMORE).unwrap();
    publisher.send(&vel_as_string.to_owned(), 0).unwrap();
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
    pub object_vector: Vec<(f32, f32, Mode)>,
    pub nearest_object: (f32, f32, Mode),
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
            nearest_object: (0.0, 0.0, Mode::None),
            play: PlayMode::Play,

            open_file_path_sender: open_sender,
            save_file_path_sender: save_sender,
        }
    }

    fn show_elements(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("FileEditViewBar").show(ctx, |ui| self.draw_file_top_bar(ui));
        TopBottomPanel::top("MainTopBar").show(ctx, |ui| self.draw_top_bar_elements(ui));
        TopBottomPanel::bottom("Play-Pause Button").show(ctx, |ui| self.handle_play_pause(ui));

        // draw stuff
        self.deal_with_object_clicks();

        // Deal with clicks on Objects
        self.deal_with_click_on_objects(ctx);
    }

    fn draw_file_top_bar(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                let save_config_button = ui.button("Save Config");
                let open_config_button = ui.button("Open Config");

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
            })
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

    fn deal_with_object_clicks(&mut self) {
        let (mx, my) = mouse_position();

        if self.clicked_mode == Mode::Circle {
            draw_circle(mx, my, 25.0, RED);
        } else if self.clicked_mode == Mode::Rectangle {
            draw_rectangle(mx, my, 50.0, 100.0, GREEN);
        }

        if is_mouse_button_down(MouseButton::Left) {
            if self.clicked_mode == Mode::Circle {
                self.object_vector.push((mx, my, Mode::Circle));
            } else if self.clicked_mode == Mode::Rectangle {
                self.object_vector.push((mx, my, Mode::Rectangle));
            }

            self.clicked_mode = Mode::None;
        } else if is_mouse_button_down(MouseButton::Right) {
            self.clicked_mode = Mode::None;
        }

        for obj in self.object_vector.iter() {
            match obj.2 {
                Mode::None => {}
                Mode::Circle => {
                    draw_circle(obj.0, obj.1, 25.0, RED);
                }
                Mode::Rectangle => {
                    draw_rectangle(obj.0, obj.1, 50.0, 100.0, GREEN);
                }
            }
        }
    }

    fn deal_with_click_on_objects(&mut self, ctx: &egui::Context) {
        let (mx, my) = mouse_position();

        for obj in self.object_vector.iter() {
            if ((mx - obj.0).powf(2.0) + (my - obj.1).powf(2.0)).sqrt() < 10.0 {
                self.nearest_object = *obj;
            }
        }

        match self.nearest_object.2 {
            Mode::None => {
                SidePanel::right("Circle Configuration").show(ctx, |ui| {});
            }
            Mode::Circle => {
                SidePanel::right("Circle Configuration").show(ctx, |ui| {
                    ui.label("Selected Circle");
                    ui.label(format!(
                        "Position: {}, {}",
                        self.nearest_object.0, self.nearest_object.1
                    ));
                });
            }

            Mode::Rectangle => {
                SidePanel::right("Rectangle Configuration").show(ctx, |ui| {
                    ui.label("Selected Rectangle");
                    ui.label(format!(
                        "Position: {}, {}",
                        self.nearest_object.0, self.nearest_object.1
                    ));
                });
            }
        }

        if is_mouse_button_down(MouseButton::Right) {
            self.nearest_object = (0.0, 0.0, Mode::None);
        }
    }

    fn handle_play_pause(&mut self, ui: &mut egui::Ui) {
        match self.play {
            PlayMode::Pause => {
                let button = ui.add(Button::new("Play |>"));
                if button.clicked() {
                    self.play = PlayMode::Play;
                }
            }

            PlayMode::Play => {
                let button = ui.add(Button::new("Pause ||"));
                if button.clicked() {
                    self.play = PlayMode::Pause;
                }
            }
        }
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

    let mut object_vector: Vec<(f32, f32, Mode)> = Vec::new();

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
