use macroquad::prelude::*;
use parry2d::math::Vector;
use parry2d::shape::Cuboid;

use crate::{
    behaviour::traits::{Collidable, Drawable, GuiObject},
    prelude::traits::Genericbject,
};

#[derive(Debug, Clone)]
pub struct StaticObj {
    pub center: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub rotation: f32,

    // collision
    pub shape: Cuboid,
}

impl StaticObj {
    pub fn new(center: (f32, f32), width: f32, height: f32, rotation: f32) -> StaticObj {
        StaticObj {
            center,
            width,
            height,
            rotation,

            shape: Cuboid::new(Vector::new(width * 0.5, height * 0.5)),
        }
    }
}

impl Drawable for StaticObj {
    fn draw(&self, tf: fn((f32, f32)) -> (f32, f32)) {
        let w = self.width * 0.5;
        let h = self.height * 0.5;

        let c = self.rotation.cos();
        let s = self.rotation.sin();

        let x1 = self.center.0 + w * c - h * s;
        let y1 = self.center.1 + w * s + h * c;

        let x2 = self.center.0 - w * c - h * s;
        let y2 = self.center.1 - w * s + h * c;

        let x3 = self.center.0 - w * c + h * s;
        let y3 = self.center.1 - w * s - h * c;

        let x4 = self.center.0 + w * c + h * s;
        let y4 = self.center.1 + w * s - h * c;

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
            GRAY,
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
            GRAY,
        );
    }

    fn draw_bounds(&self, tf: fn((f32, f32)) -> (f32, f32)) {
        let w = self.width * 0.5 + 0.25;
        let h = self.height * 0.5 + 0.25;

        let c = self.rotation.cos();
        let s = self.rotation.sin();

        let x1 = self.center.0 + w * c - h * s;
        let y1 = self.center.1 + w * s + h * c;

        let x2 = self.center.0 - w * c - h * s;
        let y2 = self.center.1 - w * s + h * c;

        let x3 = self.center.0 - w * c + h * s;
        let y3 = self.center.1 - w * s - h * c;

        let x4 = self.center.0 + w * c + h * s;
        let y4 = self.center.1 + w * s - h * c;

        let tf_p1 = tf((x1, y1));
        let tf_p2 = tf((x2, y2));
        let tf_p3 = tf((x3, y3));
        let tf_p4 = tf((x4, y4));

        // Draw the body
        draw_triangle_lines(
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
            5.0,
            GREEN,
        );
        draw_triangle_lines(
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
            5.0,
            GREEN,
        );
    }
}

impl GuiObject for StaticObj {
    fn modify_bounds(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;

        println!("Set bounds: {}, {}", self.width, self.height);
    }

    fn modify_position(&mut self, x: f32, y: f32) {
        self.center.0 = x;
        self.center.1 = y;
    }

    fn modify_rotation(&mut self, angle: f32) {
        self.rotation = angle;
    }

    fn get_bounds(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    fn get_center(&self) -> (f32, f32) {
        self.center
    }

    fn get_rotation(&self) -> f32 {
        self.rotation
    }
}

impl Genericbject for StaticObj {
    fn get_collidable(&self) -> Box<dyn Collidable> {
        return Box::new(Self::clone(&self));
    }
}
