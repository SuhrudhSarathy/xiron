/*In this file, we try to simulate the environment without the Animation.
This can be used to speed up simulations where the animation is not required */

extern crate xiron;

use xiron::prelude::*;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};


fn main() {
    let (simh, roboth) = SimulationHandler::from_file("/Users/suhrudh/programming/rust/xiron/examples/config.yaml".to_owned());
    let simh = Arc::new(Mutex::new(simh));
    let simh2 = Arc::clone(&simh);
    let t1 = thread::spawn(move || {
        loop {
            simh.lock().unwrap().step();
            thread::sleep(Duration::from_millis(100));
            println!("Sleeping");
        }
    });
    
    let t2 = thread::spawn(move || {
        loop {
            let mut s = simh2.lock().unwrap();
            s.control(&roboth[0], (0.1, 0.0));
            thread::sleep(Duration::from_millis(1000));
            println!("Sleeping2");
        }
    });

    t1.join().unwrap();
    t2.join().unwrap();
}