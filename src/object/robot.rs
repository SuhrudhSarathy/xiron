use std::f32::consts::PI;
use parry2d::shape::Ball;
use macroquad::prelude::*;

use crate::behaviour::traits::{Drawable};
use crate::parameter::{RESOLUTION, DT};

pub fn normalise(mut theta: f32) -> f32 {
    if theta > PI {
        while theta > PI {
            theta -= 2.0 * PI;
        }

        return theta;
    } else if theta < -PI {
        while theta < -PI {
            theta += 2.0 * PI;
        }

        return theta;
    }

    return theta;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Robot {
    pub id: String,
    pub pose: (f32, f32, f32),
    pub vel: (f32, f32),

    // Collision stuff
    pub shape: Ball,
    pub radius: f32
}

impl Robot {
    pub fn new(id: String, pose: (f32, f32, f32), vel: (f32, f32)) -> Robot {
        Robot {
            id: id,
            pose: pose,
            vel: vel,
            shape: Ball::new(0.5),
            radius: 0.5,
        }
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
    }
}

impl Drawable for Robot
{
    fn draw(&self, tf: fn((f32, f32))->(f32, f32))
    {
        let tf_pos = tf((self.pose.0, self.pose.1));
        draw_circle(tf_pos.0, tf_pos.1, self.radius/RESOLUTION, BLACK);
        
        let x2: f32 = self.pose.0 + self.radius * self.pose.2.cos();
        let y2: f32 = self.pose.1 + self.radius * self.pose.2.sin();

        let tf2_pos = tf((x2, y2));

        draw_line(tf_pos.0, tf_pos.1, tf2_pos.0, tf2_pos.1, 2.0, RED);

    }
}