import zmq
import json
import time
from dataclasses import dataclass, asdict
from threading import Thread


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


ctx = zmq.Context()


def recv_stuff(s: zmq.Socket):
    while True:
        output = s.recv_string(0)
        if output != "pose":
            json_obj = json.loads(output)
            data = Pose(
                json_obj["robot_id"], json_obj["position"], json_obj["orientation"]
            )

            print(data)


if __name__ == "__main__":
    s = ctx.socket(zmq.SUB)
    s.connect("tcp://127.0.0.1:5555")
    val = "1000"
    s.subscribe(b"pose")

    socket = ctx.socket(zmq.PUB)
    socket.bind("tcp://127.0.0.1:5556")

    send_thread = Thread(target=recv_stuff, args=(s,), daemon=True)
    send_thread.start()

    while True:
        topic = b"vel"  # The topic for the message (can be any bytes-like object)
        vel_obj = Twist("robot0", [0.2, 0.0], 0.2)
        vel_string = json.dumps(asdict(vel_obj))
        message = vel_string.encode(
            "utf-8"
        )  # The message to send (can be any bytes-like object)

        # Send the message with the specified topic
        socket.send_multipart([topic, message])

        time.sleep(1)
