pub mod object;
pub mod behaviour;
pub mod parameter;
pub mod parser;

use crate::object::handler::{SimulationHandler};
use crate::parameter::*;

use macroquad::{prelude::*, input};


// Config
fn window_config() -> Conf
{
    Conf
    {
        window_title: "hummer".to_owned(),
        window_height: HEIGHT as i32,
        window_width: WIDTH as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_config)]
async fn main() {
    let (mut sim_handler, robot_handlers) = SimulationHandler::from_file("/Users/suhrudh/programming/rust/hummer/config.yaml".to_owned());
    let robot0_handle = robot_handlers[0].clone();
    loop {
        clear_background(WHITE);

        sim_handler.step();
        sim_handler.draw();
        
        // Take input
        let mut vel = (0.0, 0.0);

        if input::is_key_down(KeyCode::Left)
        {
            vel.1 = 0.5;
        }
        else if input::is_key_down(KeyCode::Right)
        {
            vel.1 = -0.5;
        }

        if input::is_key_down(KeyCode::Up)
        {
            vel.0 = 0.5;
        }
        else if input::is_key_down(KeyCode::Down)
        {
            vel.0 = -0.5;
        }
        sim_handler.control(&robot0_handle, vel);

        next_frame().await
    }
}