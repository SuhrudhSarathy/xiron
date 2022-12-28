use parry2d::shape::Polyline;
use parry2d::math::Point;
use macroquad::prelude::*;

use crate::behaviour::traits::Drawable;


#[derive(Clone)]
pub struct Wall
{
    pub coords: Vec<(f32, f32)>,
    pub shape: Polyline
}

impl Wall
{
    pub fn new(coords: Vec<(f32, f32)>) -> Wall
    {
        let mut points = Vec::new();
        for coord in coords.iter()
        {
            points.push(Point::new(coord.0, coord.1));
        }
        Wall {
            coords: coords,
            shape: Polyline::new(points, Option::None),
        }
    }
}

impl Drawable for Wall
{
    fn draw(&self, tf: fn((f32, f32))->(f32, f32)) 
    {
        for i in 0..self.coords.len()-1
        {
            let c1 = self.coords[i];
            let c2 = self.coords[i+1];

            let c1_tfed = tf(c1);
            let c2_tfed = tf(c2);

            draw_line(c1_tfed.0, c1_tfed.1, c2_tfed.0, c2_tfed.1, 3.0, BLACK);

        }
    }
}