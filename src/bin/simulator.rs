extern crate argparse;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use tonic::transport::Server;
use xiron::prelude::*;

use argparse::{ArgumentParser, Store};

use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config_file_path: String = "".to_owned();
    let mut host_ip = "[::1]:50051".to_owned();

    // this block limits scope of borrows by ap.refer() method as mutable reference is used
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Run Xiron Simulator.");
        ap.refer(&mut config_file_path).add_option(
            &["-c", "--config"],
            Store,
            "Path to Config file",
        );
        ap.refer(&mut host_ip).add_option(
            &["-i", "--ip"],
            Store,
            "IP of the host. By default, it is set to localhost",
        );
        ap.parse_args_or_exit();
    }

    if &config_file_path.to_string() == "" {
        !panic!("Configuration file cannot be empty");
    }

    // ZMQ context
    let context = zmq::Context::new();

    // Simulation publisher
    let publisher = context.socket(zmq::PUB).unwrap();
    publisher
        .bind("tcp://*:8080")
        .expect("Could not bind publisher socket");

    let (sim_handler, robot_handlers) = SimulationHandler::from_file(config_file_path.to_owned());

    // Make Arc Mutex of Sim handler
    // One of them is required for to serve the gRPC server and the other to step the simulation
    let simh_loop = Arc::new(Mutex::new(sim_handler));

    let simh_server = Arc::clone(&simh_loop);

    let mut robot_map: HashMap<String, RobotHandler> = HashMap::new();

    for (name, robot_handler) in robot_handlers {
        robot_map.insert(name, robot_handler);
    }

    let robot_map_arc = Arc::new(robot_map);

    // create the channel for recieving control commands
    let (tx, mut rx) = mpsc::unbounded_channel();
    let addr = "[::1]:8081".parse().unwrap();
    let xserver = XironInterfaceServerImpl::new(simh_server, tx.clone(), robot_map_arc.clone());

    let simulator_task = tokio::spawn(async move {
        println!("Started loop Task");
        loop {
            let mut sim_handler = simh_loop.lock().unwrap();
            // recieve the data from sender
            let recieved_data = rx.try_recv();

            match recieved_data {
                Ok(vel_commands) => {
                    for data in vel_commands.iter() {
                        let robot_handler = RobotHandler::new(data.0 as usize);
                        sim_handler.control(&robot_handler, (data.1, data.2));
                    }
                }
                Err(_) => {}
            }

            sim_handler.step();
            let current_config = sim_handler.to_config();

            // Send out the data for the renderer
            publisher.send(&format!("{:03}", 1), zmq::SNDMORE).unwrap();
            publisher
                .send(&get_config_to_string(current_config), 1)
                .unwrap();

            drop(sim_handler);

            sleep(Duration::from_millis((DT * 1000.0) as u64));
        }
    });

    println!("Starting Server: {addr}");
    Server::builder()
        .add_service(XironInterfaceServer::new(xserver))
        .serve(addr)
        .await?;

    simulator_task.await.unwrap();

    Ok(())
}
