use macroquad::prelude::*;
use parry2d::math::Vector;
use parry2d::shape::{Ball, Cuboid};

use crate::behaviour::traits::Drawable;
use crate::parameter::{DT, RESOLUTION};
use crate::prelude::traits::{Collidable, Sensable};
use crate::prelude::RobotConfig;
use crate::utils::normalise;

use super::sensors::{LiDAR, LiDARMsg};

#[derive(Debug, Clone)]
pub enum Footprint {
    Circular(Ball),
    Rectangular(Cuboid),
}

#[derive(Debug, Clone)]
pub struct Robot {
    pub id: String,
    pub pose: (f32, f32, f32),
    pub vel: (f32, f32),

    // Collision stuff
    pub shape: Footprint,

    // Robot shall have a Vector of Sensables
    pub lidar: Vec<LiDAR>,
}

impl Robot {
    pub fn new(
        id: String,
        pose: (f32, f32, f32),
        vel: (f32, f32),
        lidar_present: bool,
        footprint: Vec<f32>,
    ) -> Robot {
        if footprint.len() == 1 {
            let fshape = Footprint::Circular(Ball {
                radius: (footprint[0]),
            });

            if lidar_present {
                return Robot {
                    id: id,
                    pose: pose,
                    vel: vel,
                    shape: fshape,
                    lidar: vec![LiDAR::new(pose)],
                };
            }

            return Robot {
                id: id,
                pose: pose,
                vel: vel,
                shape: fshape,
                lidar: Vec::new(),
            };
        } else {
            let width = footprint[0] * 0.5;
            let height = footprint[1] * 0.5;
            let fshape = Footprint::Rectangular(Cuboid {
                half_extents: Vector::new(width, height),
            });

            if lidar_present {
                return Robot {
                    id: id,
                    pose: pose,
                    vel: vel,
                    shape: fshape,
                    lidar: vec![LiDAR::new(pose)],
                };
            }

            return Robot {
                id: id,
                pose: pose,
                vel: vel,
                shape: fshape,
                lidar: Vec::new(),
            };
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

        for lidar in self.lidar.iter_mut() {
            lidar.translate_to(*next)
        }
    }

    pub fn sense(&self, collidables: &Vec<Box<dyn Collidable>>) -> LiDARMsg {
        return self.lidar[0].sense(collidables);
    }

    pub fn update_from_config(&mut self, config: &RobotConfig) {
        self.pose = config.pose;

        // Stepping will set the LiDar position also
        self.step(&(self.pose.0, self.pose.1, self.pose.2));
    }

    pub fn into_config(&self) -> RobotConfig {
        let mut lidar = false;
        if self.lidar.len() > 0 {
            lidar = true;
        }

        let mut extents: Vec<f32> = Vec::new();

        match self.shape {
            Footprint::Circular(s) => {
                extents.push(s.radius);
            }
            Footprint::Rectangular(s) => {
                extents.push(s.half_extents[0] * 2.0);
                extents.push(s.half_extents[1] * 2.0);
            }
        }

        RobotConfig {
            id: self.id.clone(),
            pose: self.pose,
            vel: self.vel,
            lidar: lidar,
            footprint: extents,
        }
    }
}

impl Drawable for Robot {
    fn draw(&self, tf: fn((f32, f32)) -> (f32, f32)) {
        match self.shape {
            Footprint::Circular(b) => {
                let r = b.radius;
                let tf_pos = tf((self.pose.0, self.pose.1));
                draw_circle(tf_pos.0, tf_pos.1, r / RESOLUTION, BLACK);

                let x2: f32 = self.pose.0 + r * self.pose.2.cos();
                let y2: f32 = self.pose.1 + r * self.pose.2.sin();

                let tf2_pos = tf((x2, y2));

                draw_line(tf_pos.0, tf_pos.1, tf2_pos.0, tf2_pos.1, 2.0, RED);
            }

            Footprint::Rectangular(r) => {
                let extents = r.half_extents;
                let w = extents[0];
                let h = extents[1];

                let c = self.pose.2.cos();
                let s = self.pose.2.sin();

                let x1 = self.pose.0 + w * c - h * s;
                let y1 = self.pose.1 + w * s + h * c;

                let x2 = self.pose.0 - w * c - h * s;
                let y2 = self.pose.1 - w * s + h * c;

                let x3 = self.pose.0 - w * c + h * s;
                let y3 = self.pose.1 - w * s - h * c;

                let x4 = self.pose.0 + w * c + h * s;
                let y4 = self.pose.1 + w * s - h * c;

                let tf_p1 = tf((x1, y1));
                let tf_p2 = tf((x2, y2));
                let tf_p3 = tf((x3, y3));
                let tf_p4 = tf((x4, y4));

                // Draw the body
                draw_triangle(
                    Vec2 {
                        x: tf_p1.0,
                        y: tf_p1.1,
                    },
                    Vec2 {
                        x: tf_p2.0,
                        y: tf_p2.1,
                    },
                    Vec2 {
                        x: tf_p3.0,
                        y: tf_p3.1,
                    },
                    BLACK,
                );
                draw_triangle(
                    Vec2 {
                        x: tf_p1.0,
                        y: tf_p1.1,
                    },
                    Vec2 {
                        x: tf_p3.0,
                        y: tf_p3.1,
                    },
                    Vec2 {
                        x: tf_p4.0,
                        y: tf_p4.1,
                    },
                    BLACK,
                );

                // Draw the angle
                let r = (w * w + h * h).sqrt();
                let x2: f32 = self.pose.0 + r * self.pose.2.cos();
                let y2: f32 = self.pose.1 + r * self.pose.2.sin();

                let tf_pos = tf((self.pose.0, self.pose.1));
                let tf2_pos = tf((x2, y2));

                draw_line(tf_pos.0, tf_pos.1, tf2_pos.0, tf2_pos.1, 2.0, RED);
            }
        }
    }
}
