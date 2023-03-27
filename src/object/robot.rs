use macroquad::miniquad::conf;
use macroquad::prelude::*;
use parry2d::shape::Ball;

use crate::behaviour::traits::Drawable;
use crate::parameter::{DT, RESOLUTION};
use crate::prelude::RobotConfig;
use crate::prelude::traits::{Collidable, Sensable};
use crate::utils::normalise;

use super::sensors::{LiDAR, LiDARMsg};

#[derive(Debug, Clone)]
pub struct Robot {
    pub id: String,
    pub pose: (f32, f32, f32),
    pub vel: (f32, f32),

    // Collision stuff
    pub shape: Ball,
    pub radius: f32,

    // Robot shall have a Vector of Sensables
    pub lidar: Vec<LiDAR>,
}

impl Robot {
    pub fn new(id: String, pose: (f32, f32, f32), vel: (f32, f32), lidar_present: bool) -> Robot {
        if lidar_present {
            return Robot {
                id: id,
                pose: pose,
                vel: vel,
                shape: Ball::new(0.5),
                radius: 0.5,
                lidar: vec![LiDAR::new(pose)],
            };
        }

        return Robot {
            id: id,
            pose: pose,
            vel: vel,
            shape: Ball::new(0.5),
            radius: 0.5,
            lidar: Vec::new(),
        };
    }

    pub fn control(&mut self, vel: (f32, f32)) {
        self.vel = vel;
    }

    pub fn next(&mut self) -> (f32, f32, f32) {
        let theta = normalise(self.pose.2 + self.vel.1 * DT);
        let x = self.pose.0 + self.vel.0 * self.pose.2.cos() * DT;
        let y = self.pose.1 + self.vel.0 * self.pose.2.sin() * DT;

        return (x, y, theta);
    }

    pub fn step(&mut self, next: &(f32, f32, f32)) {
        self.pose = *next;

        for lidar in self.lidar.iter_mut() {
            lidar.translate_to(*next)
        }
    }

    pub fn sense(&self, collidables: &Vec<Box<dyn Collidable>>) -> LiDARMsg {
        return self.lidar[0].sense(collidables);
    }

    pub fn update_from_config(&mut self, config: &RobotConfig)
    {
        self.pose = config.pose;
        
        // Stepping will set the LiDar position also
        self.step(&(self.pose.0, self.pose.1, self.pose.2));
    }

    pub fn into_config(&self)->RobotConfig 
    {
        let mut lidar = false;
        if self.lidar.len() > 0 { lidar = true;}

        RobotConfig
        {
            id: self.id.clone(),
            pose: self.pose,
            vel: self.vel,
            lidar: lidar
        }
    }
}

impl Drawable for Robot {
    fn draw(&self, tf: fn((f32, f32)) -> (f32, f32)) {
        let tf_pos = tf((self.pose.0, self.pose.1));
        draw_circle(tf_pos.0, tf_pos.1, self.radius / RESOLUTION, BLACK);

        let x2: f32 = self.pose.0 + self.radius * self.pose.2.cos();
        let y2: f32 = self.pose.1 + self.radius * self.pose.2.sin();

        let tf2_pos = tf((x2, y2));

        draw_line(tf_pos.0, tf_pos.1, tf2_pos.0, tf2_pos.1, 2.0, RED);
    }
}