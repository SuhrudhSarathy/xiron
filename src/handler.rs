use std::iter::zip;


use crate::object::robot::Robot;
use crate::object::sensors::LiDARMsg;
use crate::object::static_obj::StaticObj;
use crate::object::wall::Wall;
use crate::behaviour::traits::{Collidable, Drawable};
use crate::parameter::*;
use crate::parser::*;

#[derive(Debug, Clone, Copy)]
pub struct RobotHandler {
    pub id: usize,
}

impl RobotHandler
{
    pub fn new(id: usize) -> RobotHandler
    {
        RobotHandler { id }
    }
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

    pub fn from_file(filepath: String) -> (SimulationHandler, Vec<(String, RobotHandler)>) {
        let config = get_config_from_file(filepath);

        let mut sim_handle = SimulationHandler::new();

        let mut robot_handles = Vec::new();

        for robot in config.robots.iter() {
            let handle = sim_handle.add_robot(Robot::new(
                robot.id.clone(),
                robot.pose,
                robot.vel,
                robot.lidar,
                robot.footprint.clone(),
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

    pub fn add_robot(&mut self, robot: Robot) -> (String, RobotHandler) {
        let name = robot.id.clone();
        self.robots.push(robot);

        return (name, RobotHandler {
            id: self.robots.len() - 1,
        });
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

    pub fn get_pose(&self, robot: &RobotHandler) -> (f32, f32, f32) {
        return self.robots[robot.id].pose;
    }

    // TODO: Check if this can be simplified
    pub fn step(&mut self) {
        // For each robot, perform collision check and then step
        let mut next_poses: Vec<(f32, f32, f32)> = Vec::with_capacity(self.robots.len());
        let mut collisions: Vec<bool> = Vec::with_capacity(self.robots.len());

        for robot in &mut self.robots
        {
            let next_pose = robot.next();
            next_poses.push(next_pose);
        }

        let iter = zip(&self.robots, &next_poses);
        for (robot, next_pose) in iter.into_iter()
        {
            let mut object_collision: bool = false;
            
            // Object Collisions
            for object in self.objects.iter() {
                object_collision = robot.collision_check_at(object.as_ref(), &next_pose);
                if object_collision {
                    break;
                }
            }
            
            let mut robot_collision: bool = false;
            for robot2 in self.robots.iter()
            {
                if robot.id != robot2.id {
                    robot_collision = robot.collision_check_at(robot2, &next_pose);
                }
            }
            if !object_collision && !robot_collision {
                collisions.push(false);
            }
            else {
                collisions.push(true);
            }
        }
        
        for i in 0..self.robots.len()
        {
            let robot = &mut self.robots[i];
            let next_pose = next_poses[i];
            let collision = collisions[i];
            if !collision {
                robot.step(&next_pose);
            }
        }

    }
    
    pub fn collision_status_at(&self, roboth: &RobotHandler, pose: &(f32, f32, f32)) -> bool
    {
        let robot = self.robots[roboth.id].clone();
        for object in self.objects.iter()
        {
            let collision = robot.collision_check_at(object.as_ref(), pose);
            if collision{
                return true;
            }
        }
        return false;
    }

    pub fn to_config(&self) -> Config
    {
        let mut robot_config_vectors: Vec<RobotConfig> = Vec::new();
        for robot in self.robots.iter()
        {
            robot_config_vectors.push(robot.into_config());
        }

        Config {robots: robot_config_vectors, walls: Vec::new(), static_objects: Vec::new()}
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


pub struct RenderingHandler
{
    robots: Vec<Robot>,
    artists: Vec<Box<dyn Drawable>>,
}

impl RenderingHandler
{
    fn tf_function(pos: (f32, f32)) -> (f32, f32) {
        let i = (pos.0 - XLIMS.0) / RESOLUTION;
        let j = (YLIMS.1 - pos.1) / RESOLUTION;

        return (i, j);
    }

    pub fn new() -> RenderingHandler {

        return RenderingHandler {
            robots: Vec::new(),
            artists: Vec::new(),
        };
    }

    pub fn from_file(&mut self, filepath: String)
    {
        let config = get_config_from_file(filepath);

        for robot in config.robots.iter() {
            self.robots.push(
                Robot::new(
                robot.id.clone(),
                robot.pose,
                robot.vel,
                robot.lidar,
                robot.footprint.clone(),
            ));
        }

        for wall in config.walls.iter() {
            self.artists.push(Box::new(Wall::new(wall.endpoints.clone())));
        }

        for obj in config.static_objects.iter() {
            self.artists.push(Box::new(StaticObj::new(obj.center, obj.width, obj.height)));
        }
    }

    pub fn from_config(&mut self, config: &Config)
    {
        // Here we are assuming that the robots are in the same order
        // and hence, we go with a iterator without any search.
        // Later on, we can solve this using some hashmap

        for i in 0..config.robots.len()
        {
            let robot_config = &config.robots[i];
            let robot = &mut self.robots[i];

            robot.update_from_config(robot_config);
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

    pub fn render(&mut self, data: &zmq::Message)
    {

        let config = get_config_from_string(String::from(std::str::from_utf8(&data).unwrap()));

        self.from_config(&config);

        self.draw();
    }

}