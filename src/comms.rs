use crossbeam::channel::{unbounded, Receiver, Sender};
use futures::stream;
use std::net::TcpListener;
use std::thread;

use serde::{Deserialize, Serialize};
use tungstenite::accept_hdr;
use tungstenite::handshake::server::{Request, Response};

#[derive(Debug, Deserialize, Default, Serialize)]
pub struct Twist {
    pub timestamp: f64,
    pub robot_id: String,
    pub linear: (f32, f32),
    pub angular: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pose {
    pub timestamp: f64,
    pub robot_id: String,
    pub position: (f32, f32),
    pub orientation: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LaserScan {
    pub timestamp: f64,
    pub robot_id: String,
    pub angle_min: f32,
    pub angle_max: f32,
    pub num_readings: i32,
    pub values: Vec<f32>,
}

pub struct WebsocketPublisher {
    url: String,
}

impl WebsocketPublisher {
    pub fn new(url: String) -> WebsocketPublisher {
        WebsocketPublisher { url }
    }

    pub fn start(&self) -> Sender<String> {
        let url = self.url.clone();
        let (tx, rx): (Sender<String>, Receiver<String>) = unbounded();

        thread::spawn(move || {
            let server = TcpListener::bind(&url).unwrap();
            println!("WebSocket Publisher on {} initialized", url);

            for stream in server.incoming() {
                let rx_cloned = rx.clone();
                thread::spawn(move || {
                    let callback = |req: &Request, response: Response| {
                        println!("New connection: {}", req.uri().path());
                        Ok(response)
                    };

                    println!("New connection established");
                    let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

                    loop {
                        match rx_cloned.recv() {
                            Ok(message) => {
                                if let Err(e) = websocket.send(tungstenite::Message::Text(message))
                                {
                                    println!("Error sending over WebSocket: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                println!("Error receiving from channel: {}", e);
                                break;
                            }
                        }
                    }
                });
            }
        });

        return tx;
    }
}

pub struct WebsocketSubscriber {
    url: String,
}

impl WebsocketSubscriber {
    pub fn new(url: String) -> WebsocketSubscriber {
        WebsocketSubscriber { url }
    }

    pub fn start(self) -> Receiver<String> {
        let url = self.url.clone();
        let (tx, rx): (Sender<String>, Receiver<String>) = unbounded();

        let tx_clone = tx.clone();

        thread::spawn(move || {
            let server = TcpListener::bind(&url).unwrap();
            println!("WebSocket Subscriber on {} initialized", url);

            for stream in server.incoming() {
                let tx_cloned = tx_clone.clone();

                thread::spawn(move || {
                    let callback = |req: &Request, response: Response| {
                        println!("New connection: {}", req.uri().path());
                        Ok(response)
                    };

                    println!("New connection established");
                    let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

                    loop {
                        let ret = websocket.read();
                        match ret {
                            Ok(ret_msg) => {
                                if let Ok(message) = ret_msg.into_text() {
                                    let _ret = tx_cloned.send(message);
                                    match _ret {
                                        Ok(_) => {}
                                        Err(e) => {
                                            println!("Got Send Error: {}. Breaking comms", e);
                                            break;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Error when recieving message: {}", e);
                                break;
                            }
                        }
                    }
                });
            }
        });

        return rx;
    }
}
