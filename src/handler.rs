use std::iter::zip;

use egui_macroquad::egui::*;
use macroquad::prelude::*;

use crate::behaviour::traits::{Collidable, Drawable};
use crate::object::robot::Robot;
use crate::object::sensors::LiDARMsg;
use crate::object::static_obj::StaticObj;
use crate::object::wall::Wall;
use crate::parameter::*;
use crate::parser::*;

#[derive(Debug, Clone, Copy)]
pub struct RobotHandler {
    pub id: usize,
}

impl RobotHandler {
    pub fn new(id: usize) -> RobotHandler {
        RobotHandler { id }
    }
}

pub struct SimulationHandler {
    robots: Vec<Robot>,
    objects: Vec<Box<dyn Collidable>>,
    artists: Vec<Box<dyn Drawable>>,

    walls: Vec<Wall>,
    static_objects: Vec<StaticObj>,

    filepath: String,
}

impl SimulationHandler {
    pub fn new() -> SimulationHandler {
        return SimulationHandler {
            robots: Vec::new(),
            objects: Vec::new(),
            artists: Vec::new(),

            walls: Vec::new(),
            static_objects: Vec::new(),
            filepath: "".to_string(),
        };
    }

    pub fn from_file(filepath: String) -> (SimulationHandler, Vec<(String, RobotHandler)>) {
        let file_path_copy = filepath.clone();
        let config = get_config_from_file(filepath);

        let mut sim_handle = SimulationHandler::new();
        sim_handle.load_file_path(file_path_copy.to_owned());

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
    pub fn load_file_path(&mut self, path: String) {
        self.filepath = path.clone();
    }

    pub fn reset(&mut self) {
        self.robots.clear();
        self.objects.clear();
        self.artists.clear();

        self.walls.clear();
        self.static_objects.clear();

        let config = get_config_from_file(self.filepath.to_owned());
        for robot in config.robots.iter() {
            let _handle = self.add_robot(Robot::new(
                robot.id.clone(),
                robot.pose,
                robot.vel,
                robot.lidar,
                robot.footprint.clone(),
            ));
        }

        for wall in config.walls.iter() {
            self.add_wall(Wall::new(wall.endpoints.clone()));
        }

        for obj in config.static_objects.iter() {
            self.add_static_obj(StaticObj::new(obj.center, obj.width, obj.height));
        }
    }

    pub fn add_robot(&mut self, robot: Robot) -> (String, RobotHandler) {
        let name = robot.id.clone();
        self.robots.push(robot);

        return (
            name,
            RobotHandler {
                id: self.robots.len() - 1,
            },
        );
    }

    pub fn add_wall(&mut self, wall: Wall) {
        self.objects.push(Box::new(wall.clone()));
        self.artists.push(Box::new(wall.clone()));

        self.walls.push(wall.clone());
    }

    pub fn add_static_obj(&mut self, obj: StaticObj) {
        self.objects.push(Box::new(obj.clone()));
        self.artists.push(Box::new(obj.clone()));

        self.static_objects.push(obj.clone());
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

        for robot in &mut self.robots {
            let next_pose = robot.next();
            next_poses.push(next_pose);
        }

        let iter = zip(&self.robots, &next_poses);
        for (robot, next_pose) in iter.into_iter() {
            let mut object_collision: bool = false;

            // Object Collisions
            for object in self.objects.iter() {
                object_collision = robot.collision_check_at(object.as_ref(), &next_pose);
                if object_collision {
                    break;
                }
            }

            let mut robot_collision: bool = false;
            for robot2 in self.robots.iter() {
                if robot.id != robot2.id {
                    robot_collision = robot.collision_check_at(robot2, &next_pose);
                }
            }
            if !object_collision && !robot_collision {
                collisions.push(false);
            } else {
                collisions.push(true);
            }
        }

        for i in 0..self.robots.len() {
            let robot = &mut self.robots[i];
            let next_pose = next_poses[i];
            let collision = collisions[i];
            if !collision {
                robot.step(&next_pose);
            }
        }
    }

    pub fn collision_status_at(&self, roboth: &RobotHandler, pose: &(f32, f32, f32)) -> bool {
        let robot = self.robots[roboth.id].clone();
        for object in self.objects.iter() {
            let collision = robot.collision_check_at(object.as_ref(), pose);
            if collision {
                return true;
            }
        }
        return false;
    }

    pub fn to_config(&self) -> Config {
        let mut robot_config_vectors: Vec<RobotConfig> = Vec::new();
        for robot in self.robots.iter() {
            robot_config_vectors.push(robot.into_config());
        }

        let mut wall_config_vector: Vec<WallConfig> = Vec::new();

        for wall in self.walls.iter() {
            wall_config_vector.push(WallConfig {
                endpoints: wall.coords.clone(),
            });
        }

        let mut static_objects_config: Vec<StaticObjConfig> = Vec::new();

        for obj in self.static_objects.iter() {
            static_objects_config.push(StaticObjConfig {
                center: obj.center.clone(),
                width: obj.width,
                height: obj.height,
            });
        }

        Config {
            robots: robot_config_vectors,
            walls: wall_config_vector,
            static_objects: static_objects_config,
        }
    }

    pub fn draw(&self) {
        for x in (1..screen_width() as i32).step_by((1.0 / RESOLUTION) as usize) {
            draw_line(x as f32, 0.0, x as f32, screen_height(), 1.0, LIGHTGRAY)
        }

        for y in (1..screen_height() as i32).step_by((1.0 / RESOLUTION) as usize) {
            draw_line(
                0.0 as f32,
                y as f32,
                screen_width(),
                y as f32,
                1.0,
                LIGHTGRAY,
            )
        }

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

pub struct RenderingHandler {
    robots: Vec<Robot>,
    artists: Vec<Box<dyn Drawable>>,
}

impl RenderingHandler {
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

    pub fn from_file(&mut self, filepath: String) {
        let config = get_config_from_file(filepath);

        for robot in config.robots.iter() {
            self.robots.push(Robot::new(
                robot.id.clone(),
                robot.pose,
                robot.vel,
                robot.lidar,
                robot.footprint.clone(),
            ));
        }

        for wall in config.walls.iter() {
            self.artists
                .push(Box::new(Wall::new(wall.endpoints.clone())));
        }

        for obj in config.static_objects.iter() {
            self.artists
                .push(Box::new(StaticObj::new(obj.center, obj.width, obj.height)));
        }
    }

    pub fn from_config(&mut self, config: &Config) {
        // Here we are assuming that the robots are in the same order
        // and hence, we go with a iterator without any search.
        // Later on, we can solve this using some hashmap

        for i in 0..config.robots.len() {
            let robot_config = &config.robots[i];
            let robot = &mut self.robots[i];

            robot.update_from_config(robot_config);
        }
    }

    pub fn draw(&self) {
        // Draw vertical lines throughout the location
        for x in (1..WIDTH as i32).step_by((1.0 / RESOLUTION) as usize) {
            draw_line(x as f32, 0.0, x as f32, HEIGHT, 1.0, LIGHTGRAY)
        }

        for y in (1..HEIGHT as i32).step_by((1.0 / RESOLUTION) as usize) {
            draw_line(0.0 as f32, y as f32, WIDTH, y as f32, 1.0, LIGHTGRAY)
        }

        for robot in self.robots.iter() {
            robot.draw(Self::tf_function);
        }

        for artist in self.artists.iter() {
            artist.draw(Self::tf_function);
        }
    }

    pub fn draw_equi_comps(&self) {
        egui_macroquad::ui(|egui_ctx| {
            Window::new("Robot Information").show(egui_ctx, |ui| {
                for robot in self.robots.iter() {
                    ui.label(format!("Robot: {}", robot.id.to_owned()));
                    ui.label(format!(
                        "Pose: {x}, {y}, {t}",
                        x = robot.pose.0,
                        y = robot.pose.1,
                        t = robot.pose.2
                    ));
                }
            });
        });

        // Draw things before egui

        egui_macroquad::draw();
    }

    pub fn render(&mut self, data: &zmq::Message) {
        let config = get_config_from_string(String::from(std::str::from_utf8(&data).unwrap()));

        self.from_config(&config);

        self.draw();

        self.draw_equi_comps();

        // Draw Egui stuff here
    }
}
