use xiron::prelude::*;
use std::thread::sleep;
use std::time::Duration;

fn main()
{
    let context = zmq::Context::new();
    let publisher = context.socket(zmq::PUB).unwrap();
    publisher.bind("tcp://*:8080").expect("Could not bind publisher socket");

    let (mut sim_handler, robot_handlers) = SimulationHandler::from_file("./examples/path_tracking/config.yaml".to_owned());
    let robot_handler = robot_handlers[0];

    loop {
        sim_handler.control(&robot_handler, (0.3, 0.1));
        sim_handler.step();

        let current_config = sim_handler.to_config();
        // Send out the data for the renderer
        publisher
            .send(&format!("{:03}", 1), zmq::SNDMORE)
            .unwrap();
        publisher.send(&get_config_to_string(current_config), 1).unwrap();

        sleep(Duration::from_millis(10));
    }
}