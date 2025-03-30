use crate::parameter::{HEIGHT, WIDTH};
use macroquad::prelude::*;
use std::f32::consts::PI;

use std::thread;
use std::time::{Duration, Instant};

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
        fullscreen: false,
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

pub fn interpolate_pose(start: &(f32, f32, f32), end: &(f32, f32, f32), t: f32) -> (f32, f32, f32) {
    (
        start.0 + (end.0 - start.0) * t,
        start.1 + (end.1 - start.1) * t,
        start.2 + (end.2 - start.2) * t,
    )
}

pub fn draw_rotated_rectangle(
    center: (f32, f32),
    half_extents: (f32, f32),
    rotation: f32,
    color: Color,
    tf: fn((f32, f32)) -> (f32, f32),
) {
    let (x, y) = center;
    let (w, h) = half_extents;

    let c = rotation.cos();
    let s = rotation.sin();

    let x1 = x + w * c - h * s;
    let y1 = y + w * s + h * c;

    let x2 = x - w * c - h * s;
    let y2 = y - w * s + h * c;

    let x3 = x - w * c + h * s;
    let y3 = y - w * s - h * c;

    let x4 = x + w * c + h * s;
    let y4 = y + w * s - h * c;

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
        color,
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
        color,
    );
}

pub struct LoopRateHandler {
    sleep_duration: Duration,
    _last_slept_time: Option<Instant>,
}

impl LoopRateHandler {
    pub fn new(rate: f64) -> LoopRateHandler {
        return LoopRateHandler {
            sleep_duration: Duration::from_secs_f64(1.0 / rate),
            _last_slept_time: None,
        };
    }

    pub fn sleep(&mut self) {
        let now = Instant::now();

        if let Some(last_time) = self._last_slept_time {
            let elapsed = now.duration_since(last_time);
            if elapsed < self.sleep_duration {
                let sleep_time = self.sleep_duration - elapsed;
                thread::sleep(sleep_time);
            }
        }

        self._last_slept_time = Some(Instant::now());
    }

    pub fn set_rate(&mut self, rate: f64) {
        self.sleep_duration = Duration::from_secs_f64(1.0 / rate);
    }

    pub fn get_rate(&self) -> f64 {
        1.0 / self.sleep_duration.as_secs_f64()
    }

    pub fn reset(&mut self) {
        self._last_slept_time = None;
    }
}
