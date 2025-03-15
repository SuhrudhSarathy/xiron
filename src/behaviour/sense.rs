use crate::behaviour::traits::{Collidable, Sensable};
use crate::object::sensors::*;
use rand::prelude::*;
use rayon::prelude::*;

impl Sensable for LiDAR {
    type OutputMsg = LiDARMsg;

    fn get_pose(&self) -> (f32, f32, f32) {
        return self.pose;
    }

    fn sense(&self, collidables: &Vec<Box<dyn Collidable>>) -> Self::OutputMsg {
        let values: Vec<f32> = self
            .rays
            .par_iter()
            .map(|ray| {
                let mut min_dist = 20.0;
                for obj in collidables.iter() {
                    let dist = obj.raycast(ray);
                    if dist < min_dist && dist > 0.0 {
                        min_dist = dist;
                    }
                }
                let mut rng = thread_rng(); // Each thread gets its own RNG
                let random_val = rng.gen_range(-0.05..0.05);
                min_dist + random_val
            })
            .collect();

        LiDARMsg {
            angle_min: self.angle_min,
            angle_max: self.angle_max,
            num_readings: self.num_readings,
            values,
        }
    }
}
