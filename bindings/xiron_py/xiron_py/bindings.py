from xiron_py.parser import *
import zmq

class XironBinding:
    def __init__(self, port, topic):
        self.context = zmq.Context()
        self.control_publisher = self.context.socket(zmq.PUB)

        self.control_publisher.bind(port)
        self.control_topic = topic

        self.sensor_subscriber = self.context.socket(zmq.SUB)

    def publish_control(self, msg: TwistArray):
        data_as_string = msg.to_json()
        
        if self.control_topic is not None:
            self.control_publisher.send_string(str(self.control_topic))
            self.control_publisher.send_string(data_as_string)

            print("Sent string")



    def add_sensor_subscriber(self, port, topic, callback_functions):
        self.sensor_subscriber.connect(port)

        # And register the callback function


if __name__ == '__main__':
    from time import sleep

    bindings = XironBinding("tcp://*:8081", 1)
    twist1 = Twist("robot0", (0.1, -0.5))
    twist2 = Twist("robot1", (-0.1, 0.5))
    twist3 = Twist("robot2", (0.2, 0.2))

    twist_array = TwistArray([twist1, twist2, twist3])

    while True:
        bindings.publish_control(twist_array)

        sleep(0.1)