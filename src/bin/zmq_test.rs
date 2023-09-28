extern crate zmq;

use std::str;

fn main() {
    // Create a ZeroMQ context
    let context = zmq::Context::new();

    // Create a SUB socket
    let subscriber = context.socket(zmq::SUB).unwrap();

    // Connect the socket to the publisher's address
    assert!(subscriber.connect("tcp://127.0.0.1:5556").is_ok());

    // Define the topic you want to subscribe to as a byte string
    let topic_to_subscribe = "vel".to_string();

    // Subscribe to the specified topic
    assert!(subscriber
        .set_subscribe(&topic_to_subscribe.as_bytes())
        .is_ok());

    loop {
        // Receive and print messages
        let topic = subscriber.recv_bytes(0).unwrap();
        let message = subscriber.recv_string(0).unwrap().unwrap();
        println!(
            "Received: Topic='{}', Message='{}'",
            str::from_utf8(&topic).unwrap(),
            message
        );
    }
}
