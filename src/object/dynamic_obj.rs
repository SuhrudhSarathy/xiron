use macroquad::prelude::*;
use parry2d::shape::Ball;

use crate::behaviour::traits::Drawable;
use crate::parameter::RESOLUTION;

#[derive(Debug, Clone)]
pub struct DynamicObj
{
    pub radius: f32,
    pub path_to_trace: Vec<(f32, f32, f32)>,
    
    pub shape: Ball,
    pub current_pose: (f32, f32, f32),
}

impl DynamicObj
{
    pub fn new(radius: f32, path_to_trace: Vec<(f32, f32, f32)>) -> DynamicObj
    {
        DynamicObj
        {
            current_pose: path_to_trace.clone()[0],
            radius,
            path_to_trace,
            shape: Ball::new(radius),
        }
    }
}

impl Drawable for DynamicObj
{
    fn draw(&self, tf: fn((f32, f32)) -> (f32, f32)) 
    {
        let xl = self.current_pose.0;
        let yl = self.current_pose.1;

        let tf_ed = tf((xl, yl));
        
        draw_circle(tf_ed.0, tf_ed.1, self.radius/RESOLUTION, GREEN);
    }
}