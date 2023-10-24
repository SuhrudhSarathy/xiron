# Python Interface

To interact with the simulation and send control commands to the robot(s), the Python Interface `xiron_py` can be used.

### Dependencies
1. Make sure you have python installed on your system. Python can be installed from [here](https://www.python.org/downloads/)
2. Install pip to manage dependencies. Instructions are available [here](https://pip.pypa.io/en/stable/installation/)

### Installation
1. Use pip to install the module. To install it directly from github:
```
pip install git+https://github.com/SuhrudhSarathy/xironpy.git
```
2. To install from source, clone the repository and use pip to install
```
git clone https://github.com/SuhrudhSarathy/xironpy.git
```
```
cd xironpy
pip install -e .
```

## Usage
A simple example demonstating the use of callback function to gather scan and pose data and use velocity publisher to send control commands is shown below.

```python
from xiron_py.comms import XironContext
from xiron_py.data import Twist
from time import sleep


def scan_callback(msg):
    print(f"Recieved Scan message: {msg}")


def pose_callback(msg):
    print(f"Recieved Pose message: {msg}")


if __name__ == "__main__":
    # Create a context object
    ctx = XironContext()

    # Create the Velocity publisher for robot0
    vel_pub = ctx.create_vel_publisher("robot0")

    # Create the Scan Subscriber and add callback function
    ctx.create_scan_subscriber("robot0", scan_callback)

    # Create the Pose Subscriber and add callback function
    ctx.create_pose_subscriber("robot0", pose_callback)

    twist_message = Twist("robot0", [0.1, 0.0], 0.1)
    for i in range(100):
        vel_pub.publish(twist_message)
        print("Publihsed vel: ", i)
        sleep(0.1)

    twist_message = Twist("robot0", [0.0, 0.0], 0.0)
    vel_pub.publish(twist_message)

    print("Done!")

```

## Datatypes
Information about `Pose`, `LaserScan` data and `Twist` (velocity) of the robot are passed via the python dataclasses below.

```python

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

```

The source code for `xiron_py` is hosted [here](https://github.com/suhrudhSarathy/xironpy).