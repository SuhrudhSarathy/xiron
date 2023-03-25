extern crate zmq;

use std::thread::sleep;
use std::time::Duration;

fn main()
{
    let context = zmq::Context::new();
    let publisher =  context.socket(zmq::PUB).unwrap();

    publisher.bind("tcp://*:5557").expect("Could not bind publisher socket");

    println!("Waiting for subscriber to connect");
    sleep(Duration::from_millis(1000));
    println!("Sending Data");

    let mut i = 0;
    loop {
        publisher
            .send(&format!("{:03}", 1), zmq::SNDMORE)
            .unwrap();
        publisher.send(&format!("Hello World. {}", i), 0).unwrap();
        sleep(Duration::from_millis(1000));
        i += 1;
    }
    
}