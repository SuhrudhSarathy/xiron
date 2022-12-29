extern crate parry2d;

use parry2d::math::{Isometry, Point, Vector};
use parry2d::query::{Ray, RayCast};
use parry2d::shape::Ball;

use std::thread;
use std::time::Duration;

fn main() {
    let ball = Ball::new(5.0);
    let mut dx = 0.0;

    loop {
        let ray = Ray::new(Point::new(dx, 0.0), Vector::new(1.0, 0.0));
        // let ray = ray.transform_by(&Isometry::new(
        //     Vector::new(0.0, 0.0),
        //     0.1,
        // ));
        let result = ball.cast_ray(
            &Isometry::new(Vector::new(15.0, 0.0), 0.0),
            &ray,
            10.0,
            true,
        );
        println!("{:?}", result);

        thread::sleep(Duration::from_millis(100));
        dx += 0.01;
    }
}
