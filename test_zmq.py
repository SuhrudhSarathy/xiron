import zmq
import json
from dataclasses import dataclass


@dataclass
class VelocityRequest:
    robot_id: str
    velocity: tuple[float, float]


ctx = zmq.Context()


if __name__ == "__main__":
    s = ctx.socket(zmq.SUB)
    s.connect("tcp://localhost:8080")
    val = "1000"
    s.subscribe(b"pose")

    while True:
        output = s.recv_string(0)
        print(output)
