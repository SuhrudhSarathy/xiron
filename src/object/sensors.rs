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
    pub angle_increment: f32,
}

impl LiDAR {
    pub fn new(pose: (f32, f32, f32)) -> LiDAR {
        let mut rays = Vec::new();
        let angle_min = -PI;
        let angle_max = PI;
        let angle_increment = PI / 10.0;
        for theta in ((angle_min as i32 * 100)..(angle_max as i32 * 100))
            .step_by((angle_increment * 100.0) as usize)
        {
            let angle = pose.2 + theta as f32 * 0.01;
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
            angle_increment: angle_increment,
        }
    }

    pub fn translate_to(&mut self, new_pose: (f32, f32, f32)) {
        let mut rays = Vec::new();
        for theta in ((self.angle_min as i32 * 100)..(self.angle_max as i32 * 100))
            .step_by((self.angle_increment * 100.0) as usize)
        {
            let angle = self.pose.2 + theta as f32 * 0.01;
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
    pub angle_increment: f32,
    pub values: Vec<f32>,
}
