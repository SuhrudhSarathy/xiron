extern crate xiron;

use macroquad::{input, prelude::*};
use xiron::prelude::*;

#[macroquad::main(xiron)]
async fn main() {
    let (mut sim_handler, robot_handlers) =
        SimulationHandler::from_file("./examples/config.yaml".to_owned());
    let robot0_handle = robot_handlers[0].clone();
    
    loop {
        clear_background(WHITE);

        sim_handler.step();
        sim_handler.draw();

        // Take input
        let mut vel = (0.0, 0.0);

        if input::is_key_down(KeyCode::Left) {
            vel.1 = 0.5;
        } else if input::is_key_down(KeyCode::Right) {
            vel.1 = -0.5;
        }

        if input::is_key_down(KeyCode::Up) {
            vel.0 = 0.5;
        } else if input::is_key_down(KeyCode::Down) {
            vel.0 = -0.5;
        }
        sim_handler.control(&robot0_handle, vel);

        let lidar_msg = sim_handler.sense(&robot0_handle);

        next_frame().await
    }
}
