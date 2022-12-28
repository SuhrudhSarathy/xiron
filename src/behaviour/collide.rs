use crate::object::*;
use crate::behaviour::traits::{Collidable};
use parry2d::shape::Shape;

impl Collidable for Robot
{
    fn get_pose(&self) -> (f32, f32, f32) {
        return self.pose;
    }

    fn get_shape(&self) -> Box<dyn Shape> {
        
        return Box::new(self.shape.clone());
    }

    fn get_max_extent(&self) -> f32 
    {
        return self.radius;
    }
}

impl Collidable for Wall
{
    fn get_pose(&self) -> (f32, f32, f32) {
        return (0.0, 0.0, 0.0);
    }

    fn get_shape(&self) -> Box<dyn Shape> {
        return Box::new(self.shape.clone());
    }

    fn get_max_extent(&self) -> f32 {
        return 0.0;
    }
}