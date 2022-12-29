use macroquad::prelude::*;
use parry2d::math::Vector;
use parry2d::shape::Cuboid;

use crate::behaviour::traits::Drawable;
use crate::parameter::RESOLUTION;

#[derive(Debug, Clone)]
pub struct StaticObj {
    pub center: (f32, f32),
    pub width: f32,
    pub height: f32,

    // collision
    pub shape: Cuboid,
}

impl StaticObj {
    pub fn new(center: (f32, f32), width: f32, height: f32) -> StaticObj {
        StaticObj {
            center: center,
            width: width,
            height: height,

            shape: Cuboid::new(Vector::new(width * 0.5, height * 0.5)),
        }
    }
}

impl Drawable for StaticObj {
    fn draw(&self, tf: fn((f32, f32)) -> (f32, f32)) {
        let xl = self.center.0 - self.width * 0.5;
        let yl = self.center.1 + self.height * 0.5;

        let tf_ed = tf((xl, yl));

        draw_rectangle(
            tf_ed.0,
            tf_ed.1,
            self.width / RESOLUTION,
            self.height / RESOLUTION,
            GRAY,
        );
    }
}
