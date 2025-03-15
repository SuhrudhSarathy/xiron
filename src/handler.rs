use crate::behaviour::traits::{Collidable, Drawable};
use crate::object::robot::Robot;
use crate::object::sensors::LiDARMsg;
use crate::object::static_obj::StaticObj;
use crate::object::wall::Wall;
use crate::parameter::*;
use crate::parser::*;
use crate::prelude::traits::{Genericbject, GuiObject};
use crate::prelude::Footprint;
use crate::utils::interpolate_pose;
use macroquad::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct RobotHandler {
    pub id: usize,
}

impl RobotHandler {
    pub fn new(id: usize) -> RobotHandler {
        RobotHandler { id }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum SelectedObjectType {
    Robot,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ObjectParameterType {
    Position(f32, f32),
    Rotation(f32),
    Bounds(f32, f32),
}

pub struct SimulationHandler {
    robots: Vec<Robot>,
    objects: Vec<Box<dyn Genericbject>>,

    walls: Vec<Wall>,
    static_objects: Vec<StaticObj>,

    filepath: String,
}

impl SimulationHandler {
    pub fn new() -> SimulationHandler {
        return SimulationHandler {
            robots: Vec::new(),
            objects: Vec::new(),

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

        match config {
            Some(config) => {
                for robot in config.robots.iter() {
                    let handle = sim_handle.add_robot(Robot::new(
                        robot.id.clone(),
                        robot.pose,
                        robot.vel,
                        robot.lidar,
                        robot.footprint.clone(),
                        robot.drive_type,
                        robot.add_noise,
                    ));
                    robot_handles.push(handle);
                }

                for wall in config.walls.iter() {
                    sim_handle.add_wall(Wall::new(wall.endpoints.clone()));
                }

                for obj in config.static_objects.iter() {
                    sim_handle.add_static_obj(StaticObj::new(
                        obj.center,
                        obj.width,
                        obj.height,
                        obj.rotation,
                    ));
                }
            }
            None => {}
        }

        return (sim_handle, robot_handles);
    }
    pub fn load_file_path(&mut self, path: String) {
        self.filepath = path.clone();
    }

    pub fn reset(&mut self) -> Vec<(String, RobotHandler)> {
        self.robots.clear();
        self.objects.clear();

        self.walls.clear();
        self.static_objects.clear();

        let config_return = get_config_from_file(self.filepath.to_owned());
        match config_return {
            Some(config) => {
                let mut all_robot_handlers: Vec<(String, RobotHandler)> = Vec::new();
                for robot in config.robots.iter() {
                    let handle = self.add_robot(Robot::new(
                        robot.id.clone(),
                        robot.pose,
                        robot.vel,
                        robot.lidar,
                        robot.footprint.clone(),
                        robot.drive_type,
                        robot.add_noise,
                    ));

                    all_robot_handlers.push(handle);
                    for wall in config.walls.iter() {
                        self.add_wall(Wall::new(wall.endpoints.clone()));
                    }

                    for obj in config.static_objects.iter() {
                        self.add_static_obj(StaticObj::new(
                            obj.center,
                            obj.width,
                            obj.height,
                            obj.rotation,
                        ));
                    }
                }

                return all_robot_handlers;
            }
            None => {
                return Vec::new();
            }
        }
    }

    pub fn add_robot(&mut self, robot: Robot) -> (String, RobotHandler) {
        let name = robot.id.clone();

        println!(
            "
        Added robot with name : {}. Radius: {}, Center: {}, {}",
            name,
            robot.get_bounds().0,
            robot.get_center().0,
            robot.get_center().1
        );

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

        println!("\n{:?}\n added", wall.coords);

        self.walls.push(wall.clone());
    }

    pub fn add_static_obj(&mut self, obj: StaticObj) {
        self.objects.push(Box::new(obj.clone()));

        self.static_objects.push(obj.clone());
    }

    pub fn control(&mut self, robot: &RobotHandler, control: (f32, f32, f32)) {
        self.robots[robot.id].control(control);
    }

    pub fn sense(&self, robot: &RobotHandler) -> LiDARMsg {
        let mut collidables_vector = Vec::new();
        for obj in self.objects.iter() {
            collidables_vector.push(obj.get_collidable());
        }
        for obj in self.robots.iter() {
            collidables_vector.push(obj.get_collidable());
        }
        return self.robots[robot.id].sense(&collidables_vector);
    }

    pub fn get_pose(&self, robot: &RobotHandler) -> (f32, f32, f32) {
        return self.robots[robot.id].pose;
    }

    pub fn get_nearest_object(&self, x: f32, y: f32) -> (Option<SelectedObjectType>, i32) {
        // First check all robots
        let mut selected_object_type = None;
        let mut nearest_index: i32 = -1;

        for i in 0..self.robots.len() {
            let robot = self.robots[i].clone();
            let (rx, ry) = (robot.pose.0, robot.pose.1);

            match robot.shape {
                Footprint::Circular(r) => {
                    if (rx - x).abs() < r.radius && (ry - y).abs() < r.radius {
                        selected_object_type = Some(SelectedObjectType::Robot);
                        nearest_index = i as i32;

                        return (selected_object_type, nearest_index);
                    }
                }

                Footprint::Rectangular(c) => {
                    if (rx - x).abs() < c.half_extents.x && (ry - y).abs() < c.half_extents.y {
                        selected_object_type = Some(SelectedObjectType::Robot);
                        nearest_index = i as i32;

                        return (selected_object_type, nearest_index);
                    }
                }
            }
        }

        // Now check for generic objects
        // This might not work really well for Walls. We dont what to do anything for walls as of now.
        for i in 0..self.objects.len() {
            let object = self.objects[i].as_ref();

            if (x - object.get_pose().0).abs() < object.get_bounds().0
                && (y - object.get_pose().1).abs() < object.get_bounds().1
            {
                selected_object_type = Some(SelectedObjectType::Other);
                nearest_index = i as i32;

                return (selected_object_type, nearest_index);
            }
        }

        return (selected_object_type, nearest_index);
    }

    pub fn get_parameters_of_selected_object(
        &self,
        selected_object: (Option<SelectedObjectType>, i32),
        parameter_type: ObjectParameterType,
    ) -> ObjectParameterType {
        let (selected_type, index) = selected_object;

        match selected_type {
            Some(t) => match t {
                SelectedObjectType::Robot => match parameter_type {
                    ObjectParameterType::Bounds(_w, _h) => {
                        let bounds = self.robots[index as usize].get_bounds();
                        return ObjectParameterType::Bounds(bounds.0, bounds.1);
                    }
                    ObjectParameterType::Rotation(_angle) => {
                        let angle = self.robots[index as usize].get_rotation();
                        return ObjectParameterType::Rotation(angle);
                    }
                    ObjectParameterType::Position(_x, _y) => {
                        let position = self.robots[index as usize].get_center();

                        return ObjectParameterType::Position(position.0, position.1);
                    }
                },
                SelectedObjectType::Other => match parameter_type {
                    ObjectParameterType::Bounds(_w, _h) => {
                        let bounds = self.objects[index as usize].get_bounds();

                        return ObjectParameterType::Bounds(bounds.0, bounds.1);
                    }
                    ObjectParameterType::Rotation(_angle) => {
                        let angle = self.objects[index as usize].get_rotation();
                        return ObjectParameterType::Rotation(angle);
                    }
                    ObjectParameterType::Position(_x, _y) => {
                        let position = self.objects[index as usize].get_center();
                        return ObjectParameterType::Position(position.0, position.1);
                    }
                },
            },
            None => {
                return ObjectParameterType::Rotation(0.0);
            }
        }
    }

    pub fn change_parameters_of_selected_object(
        &mut self,
        selected_object: (Option<SelectedObjectType>, i32),
        parameter_type: ObjectParameterType,
    ) {
        let (selected_type, index) = selected_object;

        match selected_type {
            Some(t) => match t {
                SelectedObjectType::Robot => match parameter_type {
                    ObjectParameterType::Bounds(w, h) => {
                        self.robots[index as usize].modify_bounds(w, h)
                    }
                    ObjectParameterType::Rotation(angle) => {
                        self.robots[index as usize].modify_rotation(angle)
                    }
                    ObjectParameterType::Position(x, y) => {
                        self.robots[index as usize].modify_position(x, y)
                    }
                },
                SelectedObjectType::Other => match parameter_type {
                    ObjectParameterType::Bounds(w, h) => {
                        self.objects[index as usize].modify_bounds(w, h)
                    }
                    ObjectParameterType::Rotation(angle) => {
                        self.objects[index as usize].modify_rotation(angle)
                    }
                    ObjectParameterType::Position(x, y) => {
                        self.objects[index as usize].modify_position(x, y)
                    }
                },
            },
            None => {}
        }
    }

    pub fn delete_selected_object(&mut self, selected_object: (Option<SelectedObjectType>, i32)) {
        let (selected_type, index) = selected_object;

        match selected_type {
            Some(t) => match t {
                SelectedObjectType::Other => {
                    let _val = self.objects.remove(index as usize);
                }
                SelectedObjectType::Robot => {
                    let _val = self.robots.remove(index as usize);
                }
            },
            None => {}
        }
    }

    // TODO: Check if this can be simplified
    pub fn step(&mut self) {
        let mut next_poses: Vec<(f32, f32, f32)> = Vec::with_capacity(self.robots.len());

        for robot in &mut self.robots {
            let next_pose = robot.next();
            next_poses.push(next_pose);
        }
        let mut collisions: Vec<Option<f32>> = vec![None; self.robots.len()];

        // Check collisions with objects and other robots
        for i in 0..self.robots.len() {
            let robot = &self.robots[i];
            let start_pose = robot.get_pose();
            let end_pose = next_poses[i];

            // Object Collisions
            for object in &self.objects {
                if let Some(toi) = robot.collision_check_at_toi(
                    &*object.get_collidable(),
                    &start_pose,
                    &end_pose,
                    None,
                    None,
                ) {
                    collisions[i] = Some(collisions[i].map_or(toi, |t| t.min(toi)));
                }
            }

            // Robot Collisions
            for j in 0..self.robots.len() {
                if i != j {
                    let robot2 = &self.robots[j];
                    let start_pose2 = robot2.get_pose();
                    let end_pose2 = next_poses[j];
                    if let Some(toi) = robot.collision_check_at_toi(
                        robot2,
                        &start_pose,
                        &end_pose,
                        Some(start_pose2),
                        Some(end_pose2),
                    ) {
                        collisions[i] = Some(collisions[i].map_or(toi, |t| t.min(toi)));
                        collisions[j] = Some(collisions[j].map_or(toi, |t| t.min(toi)));
                    }
                }
            }
        }

        // Update robot positions and handle inelastic collisions
        for i in 0..self.robots.len() {
            let robot = &mut self.robots[i];
            let start_pose = robot.get_pose();
            let end_pose = next_poses[i];

            if let Some(toi) = collisions[i] {
                // Collision occurred, move robot to collision point and stop
                let collision_pose = interpolate_pose(&start_pose, &end_pose, toi);
                robot.step(&collision_pose);
                robot.control((0.0, 0.0, 0.0));
            } else {
                // No collision, move to next pose
                robot.step(&end_pose);
            }
        }
    }

    pub fn collision_status_at(&self, roboth: &RobotHandler, pose: &(f32, f32, f32)) -> bool {
        let robot = self.robots[roboth.id].clone();
        for object in self.objects.iter() {
            let collision = robot.collision_check_at(&*object.get_collidable(), pose, None);
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
                rotation: obj.rotation,
            });
        }

        Config {
            robots: robot_config_vectors,
            walls: wall_config_vector,
            static_objects: static_objects_config,
        }
    }

    pub fn draw_lines(&self) {
        let _one_meter_step = Self::inverse_scale_function(1.0);

        let mut x = XLIMS.0;
        let mut y = YLIMS.0;

        while x < screen_width() {
            let init_coord = Self::tf_function((x, YLIMS.0));
            let final_coord = Self::tf_function((x, screen_height()));
            draw_line(
                init_coord.0,
                init_coord.1,
                final_coord.0 as f32,
                final_coord.1,
                1.0,
                LIGHTGRAY,
            );

            x += 1.0;
        }

        while y < screen_height() {
            let init_coord = Self::tf_function((XLIMS.0, y));
            let final_coord = Self::tf_function((screen_width(), y));
            draw_line(
                init_coord.0,
                init_coord.1,
                final_coord.0 as f32,
                final_coord.1,
                1.0,
                LIGHTGRAY,
            );

            y += 1.0;
        }

        // Draw origin
        let origin_coord = (0.0, 0.0);
        let one_meter_in_x = (1.0, 0.0);
        let one_meter_in_y = (0.0, 1.0);

        let origin_in_pixel_frame = Self::tf_function(origin_coord);
        let one_meterx_in_pixel_frame = Self::tf_function(one_meter_in_x);
        let one_metery_in_pixel_frame = Self::tf_function(one_meter_in_y);

        draw_line(
            origin_in_pixel_frame.0,
            origin_in_pixel_frame.1,
            one_meterx_in_pixel_frame.0,
            one_meterx_in_pixel_frame.1,
            2.0,
            RED,
        );

        draw_line(
            origin_in_pixel_frame.0,
            origin_in_pixel_frame.1,
            one_metery_in_pixel_frame.0,
            one_metery_in_pixel_frame.1,
            2.0,
            GREEN,
        );
    }
    pub fn draw(&self) {
        for robot in self.robots.iter() {
            robot.draw(Self::tf_function);
        }

        for object in self.objects.iter() {
            object.draw(Self::tf_function);
        }
    }

    pub fn draw_bounds_of_selected_object(
        &self,
        selected_object: (Option<SelectedObjectType>, i32),
    ) {
        let (selected_object_type, index) = selected_object;
        match selected_object_type {
            Some(object) => match object {
                SelectedObjectType::Robot => {
                    self.robots[index as usize].draw_bounds(Self::tf_function);
                }
                SelectedObjectType::Other => {
                    self.objects[index as usize].draw_bounds(Self::tf_function);
                }
            },
            None => {}
        }
    }

    /// get pixel coordinate from World
    pub fn tf_function(pos: (f32, f32)) -> (f32, f32) {
        let i = (pos.0 - XLIMS.0) / RESOLUTION;
        let j = (YLIMS.1 - pos.1) / RESOLUTION;

        return (i, j);
    }

    /// Get World coordinate from Pixel
    pub fn get_world_from_pixel(px: f32, py: f32) -> (f32, f32) {
        let wx = XLIMS.0 + px * RESOLUTION;
        let wy = YLIMS.1 - py * RESOLUTION;

        return (wx, wy);
    }

    /// Gives scale in world frame
    pub fn scale_function(value: f32) -> f32 {
        return value * RESOLUTION;
    }

    /// Gives scale in pixel map frame
    pub fn inverse_scale_function(value: f32) -> f32 {
        return value / RESOLUTION;
    }
}

unsafe impl Sync for SimulationHandler {}

unsafe impl Send for SimulationHandler {}
