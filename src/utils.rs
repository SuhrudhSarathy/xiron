use crate::parameter::{HEIGHT, WIDTH};
use macroquad::prelude::*;
use std::f32::consts::PI;

use std::thread;
use std::time::Duration;

pub fn normalise(mut theta: f32) -> f32 {
    if theta > PI {
        while theta > PI {
            theta -= 2.0 * PI;
        }

        return theta;
    } else if theta < -PI {
        while theta < -PI {
            theta += 2.0 * PI;
        }

        return theta;
    }

    return theta;
}

pub fn xiron() -> Conf {
    Conf {
        window_title: "Xiron".to_owned(),
        window_height: HEIGHT as i32,
        window_width: WIDTH as i32,
        ..Default::default()
    }
}

pub fn sleep(time_ms: u64) {
    thread::sleep(Duration::from_millis(time_ms));
}

pub fn spin() {
    loop {
        sleep(10000);
    }
}
