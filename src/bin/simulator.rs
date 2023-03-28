use xiron::prelude::*;
use std::thread::sleep;
use std::time::Duration;
use std::collections::HashMap;

use tokio_tungstenite::tungstenite::Message;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use futures::{StreamExt, TryStreamExt};
use futures_util::future;
use tokio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;
use std::sync::Arc;
use std::env;


async fn handle_connection(robot_map: Arc<HashMap<String, RobotHandler>>, tx: mpsc::UnboundedSender<Vec<(u8, f32, f32)>>, stream: TcpStream, addr: SocketAddr)
{
    let ws_stream = accept_async(stream).await.expect("Error during Handshake");

    let (_, reader) = ws_stream.split();

    let broadcast_incoming = reader.try_for_each(|msg|{
        // Only handling if its a Text. Not handling otherwise.
        match msg
        {
            Message::Text(message) => {
                let vel_config: TwistArray= serde_json::from_str(&message).unwrap();
                let mut twist_configs: Vec<(u8, f32, f32)> = Vec::with_capacity(vel_config.twists.len());

                for twist in vel_config.twists.iter()
                {
                    let robot_handler = robot_map.get(&twist.id).unwrap();
                    twist_configs.push((robot_handler.id as u8, twist.vel.0, twist.vel.1));
                }

                // send over the channel
                tx.send(twist_configs).unwrap();
            }

            Message::Binary(_) => {}
            Message::Ping(_) => {}
            Message::Pong(_) => {}
            Message::Close(_) => {
                println!("Closing the connection with addr: {}", addr);
            }
            Message::Frame(_) => {}

        }

        // Process the message 
        future::ok(())
    });

    let output = broadcast_incoming.await;

    match output
    {
        Ok(_) => {}
        Err(e) => {println!("Shutdown happened at addr: {} with error : {}", addr, e);}
    }
}

#[tokio::main]
async fn main()
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

    // Create a websocket server to get the control inputs
    let server = TcpListener::bind(&"localhost:8081").await.expect("Failed to bind to port localhost:8081");
    println!("Controller Server connection activated at localhost:8081");
                
    let (mut sim_handler, robot_handlers) = SimulationHandler::from_file(file_path.to_owned());
    let mut robot_map: HashMap<String, RobotHandler> = HashMap::new();

    for (name, robot_handler) in robot_handlers
    {
        robot_map.insert(name, robot_handler);
    }

    let robot_map_arc = Arc::new(robot_map);

    // create the channel for recieving control commands
    let (tx, mut rx) = mpsc::unbounded_channel();

    let controller_task = tokio::spawn(async move
    {
        while let Ok((stream, addr)) = server.accept().await
        {
            println!("Got request from address: {:?}", addr);
            tokio::spawn(handle_connection(robot_map_arc.clone(), tx.clone(), stream, addr));
            
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

    controller_task.await.unwrap();
    simulator_task.await.unwrap();

}