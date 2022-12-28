pub mod object;
pub mod behaviour;
pub mod parameter;


use crate::object::robot::Robot;
use crate::object::wall::Wall;
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
    let robot0 = Robot::new(String::from("robot0"), (0.0, 1.0, 0.0), (0.0, 0.0));

    let mut wall_coords: Vec<(f32, f32)> = Vec::new();

    wall_coords.push((1.5, 0.0));
    wall_coords.push((1.5, 3.0));
    wall_coords.push((5.5, 3.0));

    let wall = Wall::new(wall_coords);

    let mut sim_handler = SimulationHandler::new();
    let robot0_handle = sim_handler.add_robot(robot0);
    sim_handler.add_wall(wall);

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
