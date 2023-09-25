use std::sync::{Arc, Mutex};
use xiron::prelude::*;

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

    let sim_handler = SimulationHandler::new();
    let sim_handler_mutex = Arc::new(Mutex::new(sim_handler));
    let sim_handler_mutex_clone = Arc::clone(&sim_handler_mutex);
    let mut egui_handler = EguiInterface::new(sender, save_sender, sim_handler_mutex);

    loop {
        clear_background(WHITE);

        // send_data(&publisher);

        // match reciever.try_recv() {
        //     Ok(message) => {
        //         println!("Got Open message here: {}", message);
        //         sim_handler.load_file_path(message);
        //         sim_handler.reset();
        //     }
        //     Err(_) => {}
        // }

        match save_reciever.try_recv() {
            Ok(message) => {
                println!("Got Save message here: {}", message);
                let sh = sim_handler_mutex_clone.lock().unwrap();
                let config = sh.to_config();

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
                let mut sh = sim_handler_mutex_clone.lock().unwrap();
                sh.step();
            }
        }
        {
            let sh = sim_handler_mutex_clone.lock().unwrap();
            sh.draw_lines();
        }

        // draw the Egui stuff and macroquad stuff also here only
        egui_macroquad::ui(|egui_ctx| egui_handler.show_elements(egui_ctx));

        {
            let sh = sim_handler_mutex_clone.lock().unwrap();
            sh.draw();
        }

        egui_macroquad::draw();

        next_frame().await;
    }
}
