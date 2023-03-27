use xiron::prelude::*;
use std::thread::sleep;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;

#[tokio::main]
async fn main()
{
    let context = zmq::Context::new();
    
    // Simulation publisher
    let publisher = context.socket(zmq::PUB).unwrap();
    publisher.bind("tcp://*:8080").expect("Could not bind publisher socket");

    // Use this to check if we can connect to the control publisher
    let mut controller_connected = false;

    let controller = context.socket(zmq::SUB).unwrap();
    match controller.connect("tcp://*:8081")
    {
        Ok(_) =>
        {
            let subscription = format!("{:03}", 1).into_bytes();
            controller.set_subscribe(&subscription).unwrap();
            println!("Connected to control publisher at tcp://localhost:8081");
            controller_connected = true;
        }
        Err(_)=> {
            println!("Could not connect to control publisher at tcp://localhost:8081. Will keep retrying");
            controller_connected = false;}
    }
                
    let (mut sim_handler, robot_handlers) = SimulationHandler::from_file("./examples/path_tracking/config.yaml".to_owned());
    let mut robot_map: HashMap<String, RobotHandler> = HashMap::new();

    for (name, robot_handler) in robot_handlers
    {
        robot_map.insert(name, robot_handler);
    }

    // create the channel for recieving control commands
    let (tx, mut rx) = mpsc::channel(100);

    let robot_map_mutex: Arc<Mutex<HashMap<String, RobotHandler>>> = Arc::new(Mutex::new(robot_map));

    let control_send_task = tokio::spawn(async move {
        loop {

            if !controller_connected
            {
                // Try connecting the socket until its online
                match controller.connect("tcp://*:8081")
                {
                    Ok(_) => {
                        controller_connected = true;
                        let subscription = format!("{:03}", 1).into_bytes();
                        controller.set_subscribe(&subscription).unwrap();
                    }
                    Err(_) => {controller_connected = false;}
                }
            }
            else {
                // Get the control input from the socket
                let _ = controller.recv_msg(0).unwrap();
                let data = controller.recv_msg(0).unwrap();

            }
            let data = vec![(0, 0.1, 0.5), (1, 0.2, -0.5), (2, -0.25, -0.25)];
            tx.send(data).await.unwrap();

            // sleep for a while
            sleep(Duration::from_millis(100));
        }
    });
    
    let simulator_task = tokio::spawn(async move {
        loop {
            // recieve the data from sender
            let recieved_data = rx.try_recv();

            match recieved_data {
                Ok(vel_commands) => {
                    for data in vel_commands.iter()
                    {
                        let robot_handler = RobotHandler::new(data.0);
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

            sleep(Duration::from_millis(10));
        }
    });

    control_send_task.await.unwrap();
    simulator_task.await.unwrap();

}