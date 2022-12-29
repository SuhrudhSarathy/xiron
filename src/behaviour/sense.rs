use crate::behaviour::traits::{Collidable, Sensable};
use crate::object::sensors::*;

impl Sensable for LiDAR {
    type OutputMsg = LiDARMsg;

    fn get_pose(&self) -> (f32, f32, f32) {
        return self.pose;
    }

    fn sense(&self, collidables: &Vec<Box<dyn Collidable>>) -> Self::OutputMsg {
        let mut values = Vec::new();

        // Ray cast and get data
        for ray in self.rays.iter() {
            let mut min_dist: f32 = 1000.0;
            for obj in collidables.iter() {
                let dist = obj.raycast(ray);
                if dist < min_dist && dist > 0.0 {
                    min_dist = dist;
                }
            }

            values.push(min_dist);
        }

        LiDARMsg {
            angle_min: self.angle_min,
            angle_max: self.angle_max,
            angle_increment: self.angle_increment,
            values: values,
        }
    }
}
