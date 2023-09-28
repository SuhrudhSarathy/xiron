use serde::{Deserialize, Serialize};
use zmq::{Context, Socket};

#[derive(Debug, Deserialize, Default, Serialize)]
pub struct Twist {
    pub robot_id: String,
    pub linear: (f32, f32),
    pub angular: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pose {
    pub robot_id: String,
    pub position: (f32, f32),
    pub orientation: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LaserScan {
    pub angle_min: f32,
    pub angle_max: f32,
    pub num_readings: u32,
    pub values: Vec<f32>,
}

pub struct Publisher {
    _publisher: Socket,
    topic_name: String,
}

impl Publisher {
    pub fn new(context: &Context, topic_name: String) -> Self {
        let _publisher = context.socket(zmq::PUB).unwrap();
        Publisher {
            _publisher: _publisher,
            topic_name: topic_name,
        }
    }

    pub fn bind(&self, address: String) {
        self._publisher.bind(&address).expect("Could not bind");
    }

    pub fn send<T: Serialize>(&self, message: &T) {
        let message_as_string = serde_json::to_string(&message).expect("Could not convert");

        self._publisher
            .send(&format!("{}", self.topic_name), zmq::SNDMORE)
            .unwrap();
        self._publisher
            .send(&message_as_string.to_owned(), 0)
            .unwrap();
    }
}

pub struct Subscriber {
    _subscriber: Socket,
    topic_name: String,
}

impl Subscriber {
    pub fn new(context: &Context, topic_name: String) -> Self {
        let _subscriber = context.socket(zmq::PUB).unwrap();
        Self {
            _subscriber: _subscriber,
            topic_name: topic_name,
        }
    }

    pub fn bind(&self, address: String) {
        self._subscriber.bind(&address).expect("Could not bind");
        let subscription = format!("{:03}", self.topic_name).into_bytes();
        self._subscriber.set_subscribe(&subscription).unwrap();
    }

    pub fn recv(&self) -> String {
        let _topic = self._subscriber.recv_msg(0).unwrap();
        let data = self._subscriber.recv_msg(0).unwrap();

        let data_as_str = std::str::from_utf8(&data).unwrap();

        return data_as_str.to_string();
    }
}
