use macroquad::prelude::*;
use xiron::ws_comms::{WebsocketPublisher, WebsocketSubscriber};

fn window_conf() -> Conf {
    Conf {
        window_title: "WebSocket Server".to_owned(),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let string_sub = WebsocketSubscriber::new("localhost:8765".to_string());
    let rx = string_sub.start();

    let status_pub = WebsocketPublisher::new("localhost:8766".to_string());
    let tx = status_pub.start();

    let mut messages = Vec::new();

    loop {
        clear_background(WHITE);

        // Check for new WebSocket messages
        while let Ok(msg) = rx.try_recv() {
            messages.push(msg);
        }

        tx.send("hello world 2".to_string()).unwrap();

        // Display received messages
        for (i, msg) in messages.iter().enumerate() {
            draw_text(msg, 20.0, 20.0 + i as f32 * 20.0, 20.0, BLACK);
        }

        next_frame().await
    }
}
