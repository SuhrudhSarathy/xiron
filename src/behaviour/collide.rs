use crate::behaviour::traits::Collidable;
use crate::object::static_obj::StaticObj;
use crate::object::*;
use parry2d::shape::Shape;

impl Collidable for Robot {
    fn get_pose(&self) -> (f32, f32, f32) {
        return self.pose;
    }

    fn get_shape(&self) -> Box<dyn Shape> {
        match self.shape {
            Footprint::Circular(b) => {
                return Box::new(b.clone());
            }

            Footprint::Rectangular(p) => {
                return Box::new(p.clone());
            }
        }
    }

    fn get_max_extent(&self) -> f32 {
        match self.shape {
            Footprint::Circular(s) => {
                return 2.0 * s.radius;
            }
            Footprint::Rectangular(s) => {
                return f32::min(s.half_extents[0] * 2.0, s.half_extents[1] * 2.0);
            }
        }
    }
}

impl Collidable for Wall {
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

impl Collidable for StaticObj {
    fn get_pose(&self) -> (f32, f32, f32) {
        return (self.center.0, self.center.1, 0.0);
    }

    fn get_shape(&self) -> Box<dyn Shape> {
        return Box::new(self.shape);
    }

    fn get_max_extent(&self) -> f32 {
        return f32::min(self.height, self.width);
    }
}

impl Collidable for DynamicObj {
    fn get_pose(&self) -> (f32, f32, f32) {
        return (
            self.current_pose.0,
            self.current_pose.1,
            self.current_pose.2,
        );
    }

    fn get_shape(&self) -> Box<dyn Shape> {
        return Box::new(self.shape);
    }

    fn get_max_extent(&self) -> f32 {
        return self.radius;
    }
}
