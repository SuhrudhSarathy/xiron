use std::time::Duration;
use std::thread::sleep;

fn main()
{
    let context = zmq::Context::new();
    let subscriber = context.socket(zmq::SUB).unwrap();

    subscriber.connect("tcp://localhost:5557").expect("Couldnt connect to Publisher");
    let subscription = format!("{:03}", 1).into_bytes();
    subscriber.set_subscribe(&subscription).unwrap();
    
    loop {
        let topic = subscriber.recv_msg(0).unwrap();
        let data = subscriber.recv_msg(0).unwrap();
        assert_eq!(&topic[..], &subscription[..]);
        println!("{}", std::str::from_utf8(&data).unwrap());

        sleep(Duration::from_millis(100));
    }
}