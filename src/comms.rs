use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Implementation of the gRPC server interfaces
use tonic::{Request, Response, Status};

pub mod xiron_interfaces {
    tonic::include_proto!("xiron_interfaces"); // The string specified here must match the proto package name
}

pub use xiron_interfaces::xiron_interface_server::{XironInterface, XironInterfaceServer};
pub use xiron_interfaces::{
    LidarRequest, LidarResponse, PoseRequest, PoseResponse, VelocityRequest, VelocityResponse,
};

use tokio::sync::mpsc::UnboundedSender;

use crate::handler::{RobotHandler, SimulationHandler};

pub struct XironInterfaceServerImpl {
    sim_handler: Arc<Mutex<SimulationHandler>>,
    tx: UnboundedSender<Vec<(u8, f32, f32)>>,
    robot_map: Arc<HashMap<String, RobotHandler>>,
}

impl XironInterfaceServerImpl {
    pub fn new(
        sim_handler: Arc<Mutex<SimulationHandler>>,
        tx: UnboundedSender<Vec<(u8, f32, f32)>>,
        robot_map: Arc<HashMap<String, RobotHandler>>,
    ) -> XironInterfaceServerImpl {
        XironInterfaceServerImpl {
            sim_handler,
            tx,
            robot_map,
        }
    }
}
#[tonic::async_trait]
impl XironInterface for XironInterfaceServerImpl {
    async fn set_velocity(
        &self,
        request: Request<VelocityRequest>,
    ) -> Result<Response<VelocityResponse>, Status> {
        let req = request.into_inner();
        let robot_id = req.id;

        let robot_handler = self.robot_map.get(&robot_id).unwrap();
        let v = req.v;
        let w = req.w;

        let result = self
            .tx
            .send(vec![(robot_handler.id as u8, v as f32, w as f32)]);

        // Send the reponse async
        match result {
            Ok(_) => {
                let response = VelocityResponse { ack: true };
                Ok(Response::new(response))
            }

            Err(_) => {
                let response = VelocityResponse { ack: false };
                Ok(Response::new(response))
            }
        }
    }

    async fn get_pose(
        &self,
        request: Request<PoseRequest>,
    ) -> Result<Response<PoseResponse>, Status> {
        let req = request.into_inner();
        let robot_id = req.id;

        let robot_handler = self.robot_map.get(&robot_id).unwrap();

        // Get lock on sim handler
        let sh = self.sim_handler.lock().unwrap();
        let pose = sh.get_pose(robot_handler);

        // // drop the mutex immediately as the later part is async
        // drop(sh);

        // Make the reponse
        let resp = PoseResponse {
            x: pose.0 as f64,
            y: pose.1 as f64,
            theta: pose.2 as f64,
        };

        Ok(Response::new(resp))
    }

    async fn get_lidar_scan(
        &self,
        request: Request<LidarRequest>,
    ) -> Result<Response<LidarResponse>, Status> {
        let req: LidarRequest = request.into_inner();
        let robot_id = req.id;

        let robot_handler = self.robot_map.get(&robot_id).unwrap();
        let sh = self.sim_handler.lock().unwrap();

        let lidar_msg = sh.sense(robot_handler);

        let mut vals: Vec<f64> = Vec::new();

        for val in lidar_msg.values.iter() {
            vals.push(*val as f64);
        }

        let resp: LidarResponse = LidarResponse {
            min_angle: lidar_msg.angle_min.clone() as f64,
            max_angle: lidar_msg.angle_max.clone() as f64,
            num_readings: lidar_msg.num_readings.clone() as i32,
            values: vals,
        };
        Ok(Response::new(resp))
    }
}
