import zmq
import json
import time
from dataclasses import dataclass, asdict
from threading import Thread
from typing import List
import time


@dataclass
class Twist:
    robot_id: str
    linear: tuple[float, float]
    angular: float


@dataclass
class Pose:
    robot_id: str
    position: tuple[float, float]
    orientation: float


@dataclass
class LaserScan:
    robot_id: str
    angle_min: float
    angle_max: float
    num_readings: int
    values: List[float]


ctx = zmq.Context()


def recv_stuff(s: zmq.Socket):
    while True:
        output = s.recv_string(0)
        if output != "pose":
            json_obj = json.loads(output)
            data = Pose(
                json_obj["robot_id"], json_obj["position"], json_obj["orientation"]
            )

            # print(data)


def recv_scan(s: zmq.Socket):
    while True:
        output = s.recv_string(0)
        if output != "scan":
            json_obj = json.loads(output)
            data = LaserScan(
                json_obj["robot_id"],
                json_obj["angle_min"],
                json_obj["angle_max"],
                json_obj["num_readings"],
                json_obj["values"],
            )

            print(time.time())


if __name__ == "__main__":
    s = ctx.socket(zmq.SUB)
    s.connect("tcp://127.0.0.1:5555")
    s.subscribe(b"pose")

    socket = ctx.socket(zmq.PUB)
    socket.bind("tcp://127.0.0.1:5556")

    s2 = ctx.socket(zmq.SUB)
    s2.connect("tcp://127.0.0.1:5858")
    s2.subscribe(b"scan")

    send_thread = Thread(target=recv_stuff, args=(s,), daemon=True)
    send_thread.start()

    send_thread2 = Thread(target=recv_scan, args=(s2,), daemon=True)
    send_thread2.start()

    while True:
        topic = b"vel"  # The topic for the message (can be any bytes-like object)
        vel_obj = Twist("robot0", [0.2, 0.0], 0.2)
        vel_string = json.dumps(asdict(vel_obj))
        message = vel_string.encode(
            "utf-8"
        )  # The message to send (can be any bytes-like object)

        # Send the message with the specified topic
        socket.send_multipart([topic, message])

        vel_obj = Twist("robot1", [0.3, 0.0], 0.6)
        vel_string = json.dumps(asdict(vel_obj))
        message = vel_string.encode("utf-8")

        # Send the message with the specified topic
        socket.send_multipart([topic, message])

        time.sleep(1)
