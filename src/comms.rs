use crossbeam::channel::{unbounded, Receiver, Sender};
use prost::Message;
use prost_types::Any;

use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use tungstenite::accept_hdr;
use tungstenite::handshake::server::{Request, Response};

use crate::object::LiDARMsg;

include!(concat!(env!("OUT_DIR"), "/_.rs"));

pub struct CommResponseError {
    pub reason: String,
}

#[derive(Debug)]
pub enum CommResponse {
    LaserScan(LaserScanMsg),
    Pose(PoseMsg),
    Reset(ResetMsg),
    Twist(TwistMsg),
}

impl From<(LiDARMsg, String)> for CommResponse {
    fn from(scan: (LiDARMsg, String)) -> Self {
        let (scan, robot_name) = scan;
        let scan_msg = LaserScanMsg {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
            robot_id: robot_name.clone(),
            angle_min: scan.angle_min,
            angle_max: scan.angle_max,
            num_readings: scan.num_readings,
            values: scan.values,
        };

        return Self::LaserScan(scan_msg);
    }
}

impl CommResponse {
    pub fn from_bytes(data: Vec<u8>) -> Result<CommResponse, CommResponseError> {
        let any_message = Any::decode(data.as_slice());
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        match any_message {
            Ok(msg) => {
                if msg.type_url == "vel" {
                    let twist_msg = TwistMsg::decode(msg.value.as_slice());
                    match twist_msg {
                        Ok(msg) => {
                            return Ok(CommResponse::Twist(msg));
                        }
                        Err(_) => {
                            return Err(CommResponseError {
                                reason: "Could not decode TwistMsg".to_string(),
                            });
                        }
                    }
                } else if msg.type_url == "reset" {
                    return Ok(CommResponse::Reset(ResetMsg { timestamp }));
                } else if msg.type_url == "pose" {
                    let pose_msg = PoseMsg::decode(msg.value.as_slice());
                    match pose_msg {
                        Ok(msg) => {
                            return Ok(CommResponse::Pose(msg));
                        }
                        Err(_) => {
                            return Err(CommResponseError {
                                reason: "Could not decode PoseMsg".to_string(),
                            });
                        }
                    }
                } else if msg.type_url == "scan" {
                    let scan_msg = LaserScanMsg::decode(msg.value.as_slice());
                    match scan_msg {
                        Ok(msg) => {
                            return Ok(CommResponse::LaserScan(msg));
                        }
                        Err(_) => {
                            return Err(CommResponseError {
                                reason: "Could not decode TwistMsg".to_string(),
                            });
                        }
                    }
                } else {
                    return Err(CommResponseError {
                        reason: format!("Unknown msg_type in Protobuf message: {}", msg.type_url)
                            .to_string(),
                    });
                }
            }
            Err(_error) => {
                return Err(CommResponseError {
                    reason: "Could not decode Protobuf".to_string(),
                });
            }
        }
    }
    pub fn to_bytes(self) -> Vec<u8> {
        let current_msg_type;
        let value_vec;
        match self {
            CommResponse::LaserScan(laser_scan_msg) => {
                current_msg_type = "scan";
                value_vec = laser_scan_msg.encode_to_vec();
            }
            CommResponse::Pose(pose_msg) => {
                current_msg_type = "pose";
                value_vec = pose_msg.encode_to_vec();
            }
            CommResponse::Reset(reset_msg) => {
                current_msg_type = "reset";
                value_vec = reset_msg.encode_to_vec();
            }
            CommResponse::Twist(twist_msg) => {
                current_msg_type = "twist";
                value_vec = twist_msg.encode_to_vec();
            }
        }

        let any_msg = Any {
            type_url: current_msg_type.to_string(),
            value: value_vec,
        };

        return any_msg.encode_to_vec();
    }
}

#[derive(Clone, Copy)]
pub struct XironCommServer {
    url: &'static str,
    sim_to_client_port: u16,
    client_to_sim_port: u16,
}

impl XironCommServer {
    pub fn new(
        url: &'static str,
        sim_to_client_port: u16,
        client_to_sim_port: u16,
    ) -> XironCommServer {
        XironCommServer {
            url,
            sim_to_client_port,
            client_to_sim_port,
        }
    }

    pub fn start(
        self,
    ) -> (
        Sender<Result<CommResponse, CommResponseError>>,
        Receiver<Result<CommResponse, CommResponseError>>,
    ) {
        let url = self.url;
        let s2c_port = self.sim_to_client_port.clone();
        let c2s_port = self.client_to_sim_port.clone();

        // Websocket incoming message Sender and Reciver. Websocket incoming message reciever should be returned
        let (sim_to_client_sender, sim_to_client_receiver) = unbounded();

        // Websocket outgoing message Sender and Reciver. Websocket outgoing message sender should be returned
        let (client_to_sim_sender, client_to_sim_reciever) = unbounded();

        let sim_to_client_receiver_clone = sim_to_client_receiver.clone();
        let client_to_sim_sender_clone = client_to_sim_sender.clone();

        let handler = Arc::new(self);
        let sim_to_client_url = format!("{}:{}", url, s2c_port);
        let client_to_sim_url = format!("{}:{}", url, c2s_port);

        thread::spawn(move || {
            let handler_clone = Arc::clone(&handler);
            let sim_to_client_server = TcpListener::bind(&sim_to_client_url).expect(
                format!(
                    "Could not bind Sim2Client server at url: {}",
                    sim_to_client_url
                )
                .as_str(),
            );
            println!("Successfull binded to {}", sim_to_client_url);

            let client_to_sim_server = TcpListener::bind(&client_to_sim_url).expect(
                format!(
                    "Could not bind Client2Sim server at url: {}",
                    client_to_sim_url
                )
                .as_str(),
            );
            println!("Successfull binded to {}", client_to_sim_url);

            let sim_to_client_thread_builder =
                thread::Builder::new().name("sim2clientServer".to_string());
            let client_to_sim_thread_builder =
                thread::Builder::new().name("client2simServer".to_string());

            let s2c_thread = sim_to_client_thread_builder
                .spawn(move || {
                    for stream in sim_to_client_server.incoming() {
                        let stream = stream.unwrap();

                        let sim_to_client_receiver_clone = sim_to_client_receiver_clone.clone();

                        // Handle incoming messages from WebSocket
                        thread::spawn({
                            let handler = handler.clone();
                            move || {
                                handler
                                    .handle_sending_messages(stream, sim_to_client_receiver_clone);
                            }
                        });
                    }
                })
                .unwrap();

            let c2s_thread = client_to_sim_thread_builder
                .spawn(move || {
                    for stream in client_to_sim_server.incoming() {
                        let stream = stream.unwrap();

                        let client_to_sim_sender_clone = client_to_sim_sender_clone.clone();

                        // Handle incoming messages from WebSocket
                        thread::spawn({
                            let handler = handler_clone.clone();
                            move || {
                                handler
                                    .handle_receiving_messages(stream, client_to_sim_sender_clone);
                            }
                        });
                    }
                })
                .unwrap();

            // Wait for the threads to join
            s2c_thread.join().unwrap();
            c2s_thread.join().unwrap();
        });
        return (sim_to_client_sender, client_to_sim_reciever);
    }

    fn handle_receiving_messages(
        &self,
        stream: TcpStream,
        sender: Sender<Result<CommResponse, CommResponseError>>,
    ) {
        let callback = |req: &Request, response: Response| {
            println!(
                "New connection: {} for receiving messages",
                req.uri().path()
            );
            Ok(response)
        };

        println!("Receiving Messages...");
        let websocket = accept_hdr(stream, callback);
        match websocket {
            Ok(mut websocket) => loop {
                let new_message = websocket.read();
                match new_message {
                    Ok(msg) => {
                        let data = msg.into_data();
                        let output = CommResponse::from_bytes(data);
                        let sent_result = sender.send(output);
                        match sent_result {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Could not send via sender: {}", e);
                            }
                        }
                    }
                    Err(error) => {
                        println!("Error when recieving message: {}", error);
                        break;
                    }
                }
            },
            Err(e) => {
                println!("Recieved Server handshake error while trying to recieve data from Websocket: {}", e);
            }
        }
    }

    fn handle_sending_messages(
        &self,
        stream: TcpStream,
        reciever: Receiver<Result<CommResponse, CommResponseError>>,
    ) {
        let callback = |req: &Request, response: Response| {
            println!(
                "New connection recieved from path: {} for Sending Messages",
                req.uri().path()
            );
            Ok(response)
        };

        println!("Sending Messages...");
        let websocket = accept_hdr(stream, callback);
        match websocket {
            Ok(mut websocket) => loop {
                let new_message = reciever.recv();
                match new_message {
                    Ok(msg) => match msg {
                        Ok(msg) => {
                            let data = msg.to_bytes();
                            let message = tungstenite::Message::Binary(data);
                            let sent_result = websocket.send(message);
                            match sent_result {
                                Ok(_) => {}
                                Err(e) => {
                                    println!("Error in Sending message: {}", e);
                                    break;
                                }
                            }
                        }
                        Err(_) => {}
                    },
                    Err(error) => {
                        println!("Error when recieving message: {}", error);
                        break;
                    }
                }
            },
            Err(e) => {
                println!(
                    "Recieved Server handshake error while trying to send data: {}",
                    e
                );
            }
        }
    }
}
