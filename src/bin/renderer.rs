

use xiron::prelude::*;

#[macroquad::main(xiron)]
async fn main()
{
    let context = zmq::Context::new();
    let mut render_handler = RenderingHandler::new();
    render_handler.from_file("./examples/path_tracking/config.yaml".to_owned());

    let subscriber = context.socket(zmq::SUB).unwrap();

    subscriber.connect("tcp://localhost:8080").expect("Couldnt connect to Publisher");
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