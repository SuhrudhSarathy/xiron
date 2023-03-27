use xiron::prelude::*;
use std::thread::sleep;
use std::time::Duration;
use std::collections::HashMap;

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
        Err(e)=> {
            println!("Could not connect to control publisher at tcp://localhost:8081. Will keep retrying");
            println!("Error: {}", e);
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
                    Err(e) => {
                        controller_connected = false;
                        println!("Error: {}", e);
                    }
                }
            }
            else {
                // Get the control input from the socket
                let topic = controller.recv_msg(1).unwrap();
                println!("Got Data: {:?}", topic) ;
                let data = controller.recv_msg(1).unwrap();

                

                // Convert the message to string
                let jsonified_string = String::from(std::str::from_utf8(&data).unwrap());
                let vel_config: TwistArray = serde_json::from_str(&jsonified_string).unwrap();

                let mut data_vec: Vec<(u8, f32, f32)> = Vec::new();

                for config in vel_config.twists.iter()
                {
                    // Get the id from hashmap
                    let rhandler = robot_map.get(&config.id);
                    match rhandler
                    {
                        Some(rhandler) => {
                            data_vec.push((rhandler.id as u8, config.vel.0, config.vel.1));
                        }
                        None => {
                            println!("Did not find the robot controller");
                        }
                    }
                }

                // Send the data over via the channel
                tx.send(data_vec).await.unwrap();
            }
            
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

            sleep(Duration::from_millis(10));
        }
    });

    control_send_task.await.unwrap();
    simulator_task.await.unwrap();

}