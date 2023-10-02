/* This implements a Path Controller in Rust using a PD controller */

use crate::handler::RobotHandler;
use crate::utils::normalise;

pub enum ProportionalControllerState {
    Init,
    Angular,
    Linear,
    Step,
    Idle,
}

pub struct ProportionalController {
    pub robot: RobotHandler,
    pub dtolerance: f32,
    pub atolerance: f32,

    pub current_path_index: usize,
    pub kp: f32,
    pub kd: f32,

    pub max_vel: (f32, f32),
    pub path: Vec<(f32, f32, f32)>,

    i: usize,
    state: ProportionalControllerState,
    prev_dist_error: f32,
    prev_angle_error: f32,
}

impl ProportionalController {
    pub fn new(robot: RobotHandler, dtolerance: f32, atolerance: f32) -> ProportionalController {
        ProportionalController {
            robot: robot,
            dtolerance: dtolerance,
            atolerance: atolerance,
            current_path_index: 0,
            kp: 0.1,
            kd: 10.0,
            max_vel: (0.75, 1.0),
            path: Vec::new(),
            i: 0,
            state: ProportionalControllerState::Init,
            prev_dist_error: 0.0,
            prev_angle_error: 0.0,
        }
    }

    pub fn set_path(&mut self, path: Vec<(f32, f32, f32)>) {
        self.path = path;
    }

    pub fn set_gains(&mut self, kp: f32, kd: f32) {
        self.kp = kp;
        self.kd = kd;
    }

    pub fn set_max_vel(&mut self, max_vel: (f32, f32)) {
        self.max_vel = max_vel;
    }

    /* Main controller function.
    Given the current pose and a tolerance, returns the control input for that time
    */
    pub fn control(&mut self, current_pose: &(f32, f32, f32)) -> (f32, f32) {
        if self.i == self.path.len() {
            self.state = ProportionalControllerState::Idle;
        } else {
            let dx = self.path[self.i].0 - current_pose.0;
            let dy = self.path[self.i].1 - current_pose.1;

            let ddist = (dx * dx + dy * dy).sqrt();

            let dtheta = normalise(dy.atan2(dx) - current_pose.2);

            // Select state based on the current condition.
            if dtheta.abs() < self.atolerance {
                if ddist.abs() < self.dtolerance {
                    self.state = ProportionalControllerState::Step;
                } else if ddist.abs() >= self.dtolerance {
                    self.state = ProportionalControllerState::Linear;
                }
            } else if dtheta.abs() >= self.atolerance {
                self.state = ProportionalControllerState::Angular;
            }

            /* Return velocities based on the state of the robot */
            match self.state {
                ProportionalControllerState::Init => return (0.0, 0.0),
                ProportionalControllerState::Step => {
                    self.i += 1;
                    return (0.0, 0.0);
                }
                ProportionalControllerState::Idle => return (0.0, 0.0),
                ProportionalControllerState::Angular => {
                    let mut vel = self.kp * dtheta + self.kd * self.prev_angle_error;
                    if vel < -self.max_vel.1 {
                        vel = -self.max_vel.1;
                    } else if vel > self.max_vel.1 {
                        vel = self.max_vel.1;
                    }

                    self.prev_angle_error = dtheta;
                    return (0.0, vel);
                }
                ProportionalControllerState::Linear => {
                    let mut vel = self.kp * ddist + self.kd * self.prev_dist_error;
                    if vel < -self.max_vel.0 {
                        vel = -self.max_vel.0;
                    } else if vel > self.max_vel.0 {
                        vel = self.max_vel.0;
                    }

                    self.prev_dist_error = ddist;
                    return (vel, 0.0);
                }
            }
        }

        return (0.0, 0.0);
    }
}
