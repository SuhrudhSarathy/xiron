/* This implements a Path Controller in Rust using a PD controller */

use crate::object::handler::RobotHandler;
use crate::utils::normalise;


pub enum PathControllerState
{
    Init,
    Angular,
    Linear,
    Step,
    Idle,
}

pub struct PathController
{
    pub robot: RobotHandler,
    pub dtolerance: f32,
    pub atolerance: f32,

    pub current_path_index: usize,
    pub kp: f32,
    pub kd: f32,

    pub max_vel: (f32, f32),
    pub path: Vec<(f32, f32, f32)>,

    i: usize,
    state: PathControllerState,
}

impl PathController
{
    pub fn new(robot: RobotHandler, dtolerance: f32, atolerance: f32) -> PathController
    {
        PathController {
            robot: robot,
            dtolerance: dtolerance,
            atolerance: atolerance,
            current_path_index: 0,
            kp: 0.1,
            kd: 10.0,
            max_vel: (0.5, 1.0),
            path: Vec::new(),
            i: 0,
            state: PathControllerState::Init
        }
    }

    pub fn set_path(&mut self, path: Vec<(f32, f32, f32)>)
    {
        self.path = path;
    }

    pub fn set_gains(&mut self, kp: f32, kd: f32)
    {
        self.kp = kp;
        self.kd = kd;
    }

    pub fn set_max_vel(&mut self, max_vel: (f32, f32))
    {
        self.max_vel = max_vel;
    }

    /* Main controller function. 
    Given the current pose and a tolerance, returns the control input for that time 
    */
    pub fn control(&mut self, current_pose: (f32, f32, f32)) -> (f32, f32)
    {
        if self.i == self.path.len()
        {
            self.state = PathControllerState::Idle;
        }

        else 
        {
            let dx = current_pose.0 - self.path[self.i].0;
            let dy = current_pose.1 - self.path[self.i].1;

            let ddist = (dx * dx + dy * dy).sqrt();

            let dtheta = normalise(current_pose.2 - self.path[self.i].2);

            // Select state based on the current condition.
            if dtheta.abs() < self.atolerance
            {
                if ddist.abs() < self.dtolerance
                {
                    self.state = PathControllerState::Step;
                }

                else if ddist.abs() >= self.dtolerance
                {
                    self.state = PathControllerState::Linear;
                }
                
            }
            else if dtheta.abs() >= self.atolerance {
                self.state = PathControllerState::Angular;
            }

            /* Return velocities based on the state of the robot */
            match self.state
            {
                PathControllerState::Init => {return (0.0, 0.0)},
                PathControllerState::Step => {
                    self.i += 1;
                    return (0.0, 0.0);
                }
                PathControllerState::Idle => {return (0.0, 0.0)},
                PathControllerState::Angular =>
                {
                    let mut vel = self.kp * dtheta;
                    if vel < -self.max_vel.1
                    {
                        vel = -self.max_vel.1;
                    }
                    else if vel > self.max_vel.1 {
                        vel = self.max_vel.1;
                    }

                    return (0.0, vel);
                }
                PathControllerState::Linear =>
                {
                    let mut vel = self.kp * ddist;
                    if vel < -self.max_vel.0
                    {
                        vel = -self.max_vel.0;
                    }
                    else if vel > self.max_vel.0 {
                        vel = self.max_vel.0;
                    }

                    return (vel, 0.0);
                }
            }

        }

        return (0.0, 0.0)
    }
}