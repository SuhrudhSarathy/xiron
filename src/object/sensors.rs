use parry2d::math::{Point, Vector};
use parry2d::query::Ray;
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct LiDAR {
    pub pose: (f32, f32, f32),
    pub rays: Vec<Ray>,

    // data for sending
    pub angle_min: f32,
    pub angle_max: f32,
    pub num_readings: i32,
}

impl LiDAR {
    pub fn new(pose: (f32, f32, f32)) -> LiDAR {
        let mut rays = Vec::new();
        let angle_min = -PI;
        let angle_max = PI;
        let num_readings: i32 = 180;
        let dtheta = (angle_max - angle_min) / num_readings as f32;

        for dt in 0..num_readings {
            let theta = angle_min + dt as f32 * dtheta;
            let angle = pose.2 + theta as f32;
            let ray = Ray::new(
                Point::new(pose.0, pose.1),
                Vector::new(angle.cos(), angle.sin()),
            );

            rays.push(ray);
        }

        LiDAR {
            pose: pose,
            rays: rays,
            angle_min: angle_min,
            angle_max: angle_max,
            num_readings: num_readings,
        }
    }

    pub fn translate_to(&mut self, new_pose: (f32, f32, f32)) {
        let mut rays = Vec::new();
        let dtheta = (self.angle_max - self.angle_min) / self.num_readings as f32;
        for dt in 0..self.num_readings {
            let theta = self.angle_min + dt as f32 * dtheta;
            let angle = self.pose.2 + theta as f32;
            let ray = Ray::new(
                Point::new(self.pose.0, self.pose.1),
                Vector::new(angle.cos(), angle.sin()),
            );

            rays.push(ray);
        }
        self.pose = new_pose;
        self.rays = rays;
    }
}

pub struct LiDARMsg {
    pub angle_min: f32,
    pub angle_max: f32,
    pub num_readings: i32,
    pub values: Vec<f32>,
}
