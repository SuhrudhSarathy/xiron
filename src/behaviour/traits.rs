use parry2d::math::{Isometry, Vector};
use parry2d::query::contact::contact;
use parry2d::query::Ray;
use parry2d::shape::Shape;

pub trait Drawable {
    fn draw(&self, tf: fn((f32, f32)) -> (f32, f32));

    fn draw_bounds(&self, tf: fn((f32, f32)) -> (f32, f32));
}

pub trait Collidable {
    fn get_pose(&self) -> (f32, f32, f32);
    fn get_shape(&self) -> Box<dyn Shape>;
    fn get_max_extent(&self) -> f32;

    fn collision_check(&self, other: &dyn Collidable) -> bool {
        let pos1 = Isometry::new(
            Vector::new(self.get_pose().0, self.get_pose().1),
            self.get_pose().2,
        );
        let pos2 = Isometry::new(
            Vector::new(other.get_pose().0, other.get_pose().1),
            other.get_pose().2,
        );
        let c1 = self.get_shape();
        let c2 = other.get_shape();

        let result = contact(&pos1, &*c1, &pos2, &*c2, 5.0);
        match result {
            Ok(result) => match result {
                None => false,
                Some(dist) => {
                    if dist.dist < 0.1 {
                        return true;
                    } else {
                        return false;
                    }
                }
            },
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    fn collision_check_at(&self, other: &dyn Collidable, pose: &(f32, f32, f32)) -> bool {
        let pos1 = Isometry::new(Vector::new(pose.0, pose.1), pose.2);
        let pos2 = Isometry::new(
            Vector::new(other.get_pose().0, other.get_pose().1),
            other.get_pose().2,
        );
        let c1 = self.get_shape();
        let c2 = other.get_shape();

        let result = contact(&pos1, &*c1, &pos2, &*c2, 5.0);
        match result {
            Ok(result) => match result {
                None => false,
                Some(dist) => {
                    if dist.dist < 0.1 {
                        return true;
                    } else {
                        return false;
                    }
                }
            },
            Err(error) => {
                panic!("{}", error);
            }
        }
    }
    fn raycast(&self, ray: &Ray) -> f32 {
        // println!("{:?}", ray);
        let shape = self.get_shape();
        let ray_result = shape.cast_ray(
            &Isometry::new(
                Vector::new(self.get_pose().0, self.get_pose().1),
                self.get_pose().2,
            ),
            ray,
            100.0,
            true,
        );
        match ray_result {
            None => return -10.0,
            Some(r) => return r,
        }
    }
}

pub trait Sensable {
    type OutputMsg;

    fn get_pose(&self) -> (f32, f32, f32);
    fn sense(&self, collidables: &Vec<Box<dyn Collidable>>) -> Self::OutputMsg;
}

pub trait GuiObject {
    fn modify_bounds(&mut self, width: f32, height: f32);
    fn modify_rotation(&mut self, angle: f32);
    fn modify_position(&mut self, x: f32, y: f32);

    fn get_center(&self) -> (f32, f32);
    fn get_rotation(&self) -> f32;
    fn get_bounds(&self) -> (f32, f32);
}

pub trait Genericbject: Collidable + Drawable + GuiObject {
    fn get_collidable(&self) -> Box<dyn Collidable>;
}
