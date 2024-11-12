use crossbeam::channel::{unbounded, Receiver, Sender};

use std::net::TcpListener;
use std::thread;

use tungstenite::accept_hdr;
use tungstenite::handshake::server::{Request, Response};

include!(concat!(env!("OUT_DIR"), "/_.rs"));

pub struct WebsocketPublisher {
    url: String,
}

impl WebsocketPublisher {
    pub fn new(url: String) -> WebsocketPublisher {
        WebsocketPublisher { url }
    }

    pub fn start(&self) -> Sender<Vec<u8>> {
        let url = self.url.clone();
        let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = unbounded();

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
                                if let Err(e) =
                                    websocket.send(tungstenite::Message::Binary(message))
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

    pub fn start(self) -> Receiver<Vec<u8>> {
        let url = self.url.clone();
        let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = unbounded();

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
                                let message = ret_msg.into_data();
                                let _ret = tx_cloned.send(message);
                                match _ret {
                                    Ok(_) => {}
                                    Err(e) => {
                                        println!("Got Send Error: {}. Breaking comms", e);
                                        break;
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
