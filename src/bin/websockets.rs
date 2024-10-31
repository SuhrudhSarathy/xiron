use macroquad::prelude::*;
use std::net::TcpListener;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use tungstenite::accept_hdr;
use tungstenite::handshake::server::{Request, Response};

fn window_conf() -> Conf {
    Conf {
        window_title: "WebSocket Server".to_owned(),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let (tx, rx): (Sender<String>, Receiver<String>) = channel();

    // Spawn WebSocket server in a separate thread
    thread::spawn(move || {
        let server = TcpListener::bind("127.0.0.1:3012").unwrap();
        println!("WebSocket server listening on ws://127.0.0.1:3012");

        for stream in server.incoming() {
            let tx_clone = tx.clone();
            thread::spawn(move || {
                let callback = |req: &Request, response: Response| {
                    println!("New WebSocket connection: {}", req.uri().path());
                    Ok(response)
                };

                let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

                loop {
                    let msg = websocket.read().unwrap();
                    if msg.is_text() {
                        if let Ok(text) = msg.into_text() {
                            tx_clone.send(text).unwrap();
                        }
                    }
                }
            });
        }
    });

    let mut messages = Vec::new();

    loop {
        clear_background(WHITE);

        // Check for new WebSocket messages
        while let Ok(msg) = rx.try_recv() {
            messages.push(msg);
        }

        // Display received messages
        for (i, msg) in messages.iter().enumerate() {
            draw_text(msg, 20.0, 20.0 + i as f32 * 20.0, 20.0, BLACK);
        }

        next_frame().await
    }
}