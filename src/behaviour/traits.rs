use parry2d::math::{Isometry, Vector};
use parry2d::query::contact::contact;
use parry2d::query::Ray;
use parry2d::shape::Shape;

pub trait Drawable {
    fn draw(&self, tf: fn((f32, f32)) -> (f32, f32));

    fn draw_bounds(&self, tf: fn((f32, f32)) -> (f32, f32));
}

pub trait Collidable: Send + Sync {
    fn get_pose(&self) -> (f32, f32, f32);
    fn get_shape(&self) -> Box<dyn Shape + Send + Sync>;
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

        let result = contact(&pos1, &*c1, &pos2, &*c2, 1.0);
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

    fn collision_check_at(
        &self,
        other: &dyn Collidable,
        pose: &(f32, f32, f32),
        other_pose: Option<(f32, f32, f32)>,
    ) -> bool {
        let pos1 = Isometry::new(Vector::new(pose.0, pose.1), pose.2);
        let mut pos2 = Isometry::new(
            Vector::new(other.get_pose().0, other.get_pose().1),
            other.get_pose().2,
        );

        match other_pose {
            Some(other_pose) => {
                pos2 = Isometry::new(Vector::new(other_pose.0, other_pose.1), other_pose.2);
            }
            None => {}
        }

        let c1 = self.get_shape();
        let c2 = other.get_shape();

        let result = contact(&pos1, &*c1, &pos2, &*c2, 1.0);
        match result {
            Ok(result) => match result {
                None => false,
                Some(dist) => {
                    if dist.dist < 0.05 {
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

    fn collision_check_at_toi(
        &self,
        other: &dyn Collidable,
        start_pose: &(f32, f32, f32),
        end_pose: &(f32, f32, f32),
        other_start_pose: Option<(f32, f32, f32)>,
        other_end_pose: Option<(f32, f32, f32)>,
    ) -> Option<f32> {
        let start_pos1 = Isometry::new(Vector::new(start_pose.0, start_pose.1), start_pose.2);
        let end_pos1 = Isometry::new(Vector::new(end_pose.0, end_pose.1), end_pose.2);

        let start_pos2 = match other_start_pose {
            Some(pose) => Isometry::new(Vector::new(pose.0, pose.1), pose.2),
            None => Isometry::new(
                Vector::new(other.get_pose().0, other.get_pose().1),
                other.get_pose().2,
            ),
        };

        let end_pos2 = match other_end_pose {
            Some(pose) => Isometry::new(Vector::new(pose.0, pose.1), pose.2),
            None => start_pos2,
        };

        let c1 = self.get_shape();
        let c2 = other.get_shape();

        let motion1 = end_pos1.translation.vector - start_pos1.translation.vector;
        let motion2 = end_pos2.translation.vector - start_pos2.translation.vector;

        let result = parry2d::query::time_of_impact(
            &start_pos1,
            &motion1,
            &*c1,
            &start_pos2,
            &motion2,
            &*c2,
            1.0,
            true,
        );

        match result {
            Ok(toi) => match toi {
                Some(toi) => {
                    return Some(toi.toi);
                }
                None => {
                    return None;
                }
            },
            Err(_) => None,
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
            10.0,
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
