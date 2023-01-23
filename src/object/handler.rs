use super::robot::Robot;
use super::sensors::LiDARMsg;
use super::static_obj::StaticObj;
use super::wall::Wall;
use crate::behaviour::traits::{Collidable, Drawable};
use crate::parameter::*;
use crate::parser::get_config_from_file;

#[derive(Debug, Clone, Copy)]
pub struct RobotHandler {
    id: usize,
}

pub struct SimulationHandler {
    robots: Vec<Robot>,
    objects: Vec<Box<dyn Collidable>>,
    artists: Vec<Box<dyn Drawable>>,
}

impl SimulationHandler {
    pub fn new() -> SimulationHandler {
        return SimulationHandler {
            robots: Vec::new(),
            objects: Vec::new(),
            artists: Vec::new(),
        };
    }

    pub fn from_file(filepath: String) -> (SimulationHandler, Vec<RobotHandler>) {
        let config = get_config_from_file(filepath);

        let mut sim_handle = SimulationHandler::new();

        let mut robot_handles = Vec::new();

        for robot in config.robots.iter() {
            let handle = sim_handle.add_robot(Robot::new(
                robot.id.clone(),
                robot.pose,
                robot.vel,
                robot.lidar,
            ));
            robot_handles.push(handle);
        }

        for wall in config.walls.iter() {
            sim_handle.add_wall(Wall::new(wall.endpoints.clone()));
        }

        for obj in config.static_objects.iter() {
            sim_handle.add_static_obj(StaticObj::new(obj.center, obj.width, obj.height));
        }

        return (sim_handle, robot_handles);
    }

    pub fn add_robot(&mut self, robot: Robot) -> RobotHandler {
        self.robots.push(robot);

        return RobotHandler {
            id: self.robots.len() - 1,
        };
    }

    pub fn add_wall(&mut self, wall: Wall) {
        self.objects.push(Box::new(wall.clone()));
        self.artists.push(Box::new(wall.clone()));
    }

    pub fn add_static_obj(&mut self, obj: StaticObj) {
        self.objects.push(Box::new(obj.clone()));
        self.artists.push(Box::new(obj.clone()));
    }

    pub fn control(&mut self, robot: &RobotHandler, control: (f32, f32)) {
        self.robots[robot.id].control(control);
    }

    pub fn sense(&self, robot: &RobotHandler) -> LiDARMsg {
        return self.robots[robot.id].sense(&self.objects);
    }
    
    pub fn get_pose(&self, robot: &RobotHandler) -> (f32, f32, f32)
    {
        return self.robots[robot.id].pose;
    }

    pub fn step(&mut self) {
        // For each robot, perform collision check and then step
        for robot in self.robots.iter_mut() {
            let next_pose = robot.next();
            let mut collision: bool = false;
            for object in self.objects.iter() {
                collision = robot.collision_check_at(object.as_ref(), &next_pose);
                if collision {
                    break;
                }
            }
            if !collision {
                robot.step(&next_pose);
                // println!("{}: {}, {}, {}", robot.id, robot.pose.0, robot.pose.1, robot.pose.2);
            }
        }
    }

    pub fn draw(&self) {
        for robot in self.robots.iter() {
            robot.draw(Self::tf_function);
        }

        for artist in self.artists.iter() {
            artist.draw(Self::tf_function);
        }
    }

    fn tf_function(pos: (f32, f32)) -> (f32, f32) {
        let i = (pos.0 - XLIMS.0) / RESOLUTION;
        let j = (YLIMS.1 - pos.1) / RESOLUTION;

        return (i, j);
    }
}

unsafe impl Sync for SimulationHandler {}

unsafe impl Send for SimulationHandler {}