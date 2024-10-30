use std::{
    str::{self, FromStr},
    thread,
    time::Duration,
};

use macroquad::prelude::error;
use serde::{Deserialize, Serialize};
use zmq::{Context, Message, Socket};

use std::sync::mpsc::Sender;

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
    pub robot_id: String,
    pub angle_min: f32,
    pub angle_max: f32,
    pub num_readings: i32,
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
        self._publisher
            .set_conflate(true)
            .expect("Could not set conflate");
        self._publisher
            .set_immediate(true)
            .expect("Could not set immediate");
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
    spin_every: Duration,
    sender: Sender<String>,
    topic_name: String,
}

impl Subscriber {
    pub fn new(
        context: &Context,
        topic_name: String,
        spin_every: Duration,
        sender: Sender<String>,
    ) -> Self {
        let _subscriber = context.socket(zmq::SUB).unwrap();
        Self {
            _subscriber: _subscriber,
            spin_every: spin_every,
            sender: sender,
            topic_name: topic_name,
        }
    }

    pub fn bind(&self, address: String) {
        self._subscriber.connect(&address).expect("Could not bind");
        self._subscriber.set_rcvhwm(10).expect("Could not set HWM");
        self._subscriber
            .set_rcvtimeo(1000)
            .expect("Could not set rcvtimeout");
        let _out = self
            ._subscriber
            .set_subscribe(self.topic_name.as_bytes())
            .unwrap();
    }

    pub fn recv(&self) -> Option<String> {
        let mut message1 = Message::new();
        let _res = self
            ._subscriber
            .recv(&mut message1, 0)
            .expect("Failed to get recv message");

        let mut message2 = Message::new();
        let _res = self
            ._subscriber
            .recv(&mut message2, 0)
            .expect("Failed to get recv message");

        let _topic = message1.as_str().unwrap();
        let message = message2.as_str();

        match message {
            Some(message) => {
                return Some(String::from_str(message).unwrap());
            }
            None => {
                return None;
            }
        }
    }

    pub fn spin(self) {
        thread::spawn(move || loop {
            let out = self.recv();

            match out {
                Some(data) => self.sender.send(data).unwrap(),
                None => {}
            }

            thread::sleep(self.spin_every);
        });
    }
}
