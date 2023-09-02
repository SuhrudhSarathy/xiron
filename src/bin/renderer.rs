use argparse::{ArgumentParser, Store};
use xiron::prelude::*;

#[macroquad::main(xiron)]
async fn main() {
    let mut config_file_path: String = "".to_owned();
    let mut host_ip = "127.0.0.1:8081".to_owned();

    // this block limits scope of borrows by ap.refer() method as mutable reference is used
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Run Xiron Simulator.");
        ap.refer(&mut config_file_path).add_option(
            &["-c", "--config"],
            Store,
            "Path to Config file",
        );
        ap.refer(&mut host_ip).add_option(
            &["-h", "--name"],
            Store,
            "IP of the host. By default, it is set to localhost",
        );
        ap.parse_args_or_exit();
    }

    if &config_file_path.to_string() == "" {
        !panic!("Configuration file cannot be empty");
    }

    let context = zmq::Context::new();
    let mut render_handler = RenderingHandler::new();
    render_handler.from_file(config_file_path.to_owned());

    let subscriber = context.socket(zmq::SUB).unwrap();

    subscriber
        .connect("tcp://localhost:8080")
        .expect("Couldnt connect to Publisher");
    let subscription = format!("{:03}", 1).into_bytes();
    subscriber.set_subscribe(&subscription).unwrap();

    loop {
        clear_background(WHITE);
        let _ = subscriber.recv_msg(0).unwrap();
        let data = subscriber.recv_msg(0).unwrap();
        render_handler.render(&data);

        next_frame().await;
    }
}
