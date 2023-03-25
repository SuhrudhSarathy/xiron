extern crate zmq;

struct ZMQPublisher
{
    port: String,
    topic: i8,
    context: zmq::Context,
    publisher: zmq::Socket
}

impl ZMQPublisher
{
    pub fn new(port: &str, topic: i8) -> ZMQPublisher
    {
        let context = zmq::Context::new();
        let publisher = context.socket(zmq::PUB).unwrap();

        publisher.bind(port).expect("Couldn't bind to the Port as it was already bound");

        ZMQPublisher { port: String::from(port), topic: topic, context: context, publisher: publisher }
    }

    pub fn send(&self, msg: &str)
    {
        let send_init = self.publisher.send(&format!("{:03}", 1), zmq::SNDMORE);
        match send_init
        {
            Ok(_) =>
            {
                self.publisher.send(msg, 0).expect("Failed to send data");
            }
            Err(e) =>
            {
                panic!("Failed to send");
            }
        }
    }
}

struct ZMQSubscriber
{
    port: String,
    topic: i8,
    context: zmq::Context,
    subscriber: zmq::Socket
}

impl ZMQSubscriber
{
    pub fn new(port: &str, topic: i8) -> ZMQSubscriber
    {
        let context = zmq::Context::new();
        let subsriber = context.socket(zmq::PUB).unwrap();

        subsriber.connect(port).expect("Couldn't bind to the Port as it was already bound");
        subsriber.set_subscribe(&format!("{:03}", topic).into_bytes()).unwrap();

        ZMQSubscriber { port: String::from(port), topic: topic, context: context, subscriber: subsriber }
    }

    pub fn recv(&self) -> String
    {
        let _ = self.subscriber.recv_msg(0).unwrap();
        let data = self.subscriber.recv_msg(0).unwrap();

        String::from(std::str::from_utf8(&data).unwrap())
    }
}
