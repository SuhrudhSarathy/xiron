use macroquad::prelude::*;
use parry2d::math::Point;
use parry2d::shape::Polyline;

use crate::behaviour::traits::Drawable;
use crate::prelude::traits::{Collidable, Genericbject, GuiObject};

#[derive(Clone)]
pub struct Wall {
    pub coords: Vec<(f32, f32)>,
    pub shape: Polyline,
}

impl Wall {
    pub fn new(coords: Vec<(f32, f32)>) -> Wall {
        let mut points = Vec::new();
        for coord in coords.iter() {
            points.push(Point::new(coord.0, coord.1));
        }
        Wall {
            coords: coords,
            shape: Polyline::new(points, Option::None),
        }
    }
}

impl Drawable for Wall {
    fn draw(&self, tf: fn((f32, f32)) -> (f32, f32)) {
        for i in 0..self.coords.len() - 1 {
            let c1 = self.coords[i];
            let c2 = self.coords[i + 1];

            let c1_tfed = tf(c1);
            let c2_tfed = tf(c2);

            draw_line(c1_tfed.0, c1_tfed.1, c2_tfed.0, c2_tfed.1, 3.0, BLACK);
        }
    }
    fn draw_bounds(&self, _tf: fn((f32, f32)) -> (f32, f32)) {}
}

impl GuiObject for Wall {
    fn get_bounds(&self) -> (f32, f32) {
        (0.0, 0.0)
    }

    fn get_center(&self) -> (f32, f32) {
        (0.0, 0.0)
    }

    fn get_rotation(&self) -> f32 {
        0.0
    }

    fn modify_bounds(&mut self, _width: f32, _height: f32) {}

    fn modify_position(&mut self, _x: f32, _y: f32) {}

    fn modify_rotation(&mut self, _angle: f32) {}
}

impl Genericbject for Wall {
    fn get_collidable(&self) -> Box<dyn Collidable> {
        return Box::new(Self::clone(&self));
    }
}
