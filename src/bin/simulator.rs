use tonic::transport::Server;
use xiron::prelude::*;
use std::thread::sleep;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::env;

use tokio::sync::mpsc;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> 
{
    let args: Vec<String> = env::args().collect();

    if args.len() < 2
    {
        panic!("Pass the configuration file as an argument");
    }
    
    let file_path = &args[1];

    let context = zmq::Context::new();
    
    // Simulation publisher
    let publisher = context.socket(zmq::PUB).unwrap();
    publisher.bind("tcp://*:8080").expect("Could not bind publisher socket");

    let (sim_handler, robot_handlers) = SimulationHandler::from_file(file_path.to_owned());
    
    // Make Arc Mutex of Sim handler
    let simh_loop = Arc::new(Mutex::new(sim_handler));
    let simh_server = Arc::clone(&simh_loop);


    let mut robot_map: HashMap<String, RobotHandler> = HashMap::new();

    for (name, robot_handler) in robot_handlers
    {
        robot_map.insert(name, robot_handler);
    }

    let robot_map_arc = Arc::new(robot_map);

    // create the channel for recieving control commands
    let (tx, mut rx) = mpsc::unbounded_channel();
    let addr = "[::1]:8081".parse()?;
    let xserver = XironInterfaceServerImpl::new(simh_server, tx.clone(), robot_map_arc.clone());

    let simulator_task = tokio::spawn(async move {
        println!("Started loop Task");
        loop {
            let mut sim_handler = simh_loop.lock().unwrap();
            // recieve the data from sender
            let recieved_data = rx.try_recv();

            match recieved_data {
                Ok(vel_commands) => {
                    for data in vel_commands.iter()
                    {
                        let robot_handler = RobotHandler::new(data.0 as usize);
                        sim_handler.control(&robot_handler, (data.1, data.2));
                    }
                }
                Err(_) =>
                {
                }
            }

            sim_handler.step();
            let current_config = sim_handler.to_config();

            // Send out the data for the renderer
            publisher
                .send(&format!("{:03}", 1), zmq::SNDMORE)
                .unwrap();
            publisher.send(&get_config_to_string(current_config), 1).unwrap();

            drop(sim_handler);

            sleep(Duration::from_millis((DT * 1000.0) as u64));
        }
    });

    println!("Starting Server");
    Server::builder().add_service(XironInterfaceServer::new(xserver)).serve(addr).await?;
    
    simulator_task.await.unwrap();

    Ok(())
}