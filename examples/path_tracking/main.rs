extern crate xiron;

use macroquad::prelude::*;
use xiron::prelude::{controller::PathController, *};

#[macroquad::main(xiron)]
async fn main() {
    let (mut sim_handler, robot_handlers) =
        SimulationHandler::from_file("examples/path_tracking/config.yaml".to_owned());
    let (_, robot0_handle) = robot_handlers[0].clone();

    let path = vec![
        (2.0, 0.0, 0.0),
        (2.0, 4.0, 0.0),
        (0.0, 4.0, 0.0),
        (0.0, 0.0, 0.0),
    ];

    let mut path_controller = PathController::new(robot0_handle, 0.1, 0.1);
    path_controller.set_path(path);

    loop {
        clear_background(WHITE);

        sim_handler.step();
        sim_handler.draw();

        let current_pose = sim_handler.get_pose(&robot0_handle);
        let vel = path_controller.control(&current_pose);

        // println!("Current pose: {}, {}, {}\nCurrent Vel: {}, {}", current_pose.0, current_pose.1, current_pose.2, vel.0, vel.1);
        sim_handler.control(&robot0_handle, vel);

        next_frame().await
    }
}
