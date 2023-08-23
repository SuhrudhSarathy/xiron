# Xiron Python Bindings
This module implements Python bindings for the gRPC clients. The installation instructions are given below

# Installation
1. Install Protobufs (if not installed already)
### Ubuntu
```bash
sudo apt update && sudo apt upgrade -y
sudo apt install -y protobuf-compiler libprotobuf-dev
```
### Mac
Assuming [Homebrew](https://brew.sh) is already installed. (If not, see instructions for installing Homebrew on the Homebrew website.)
```bash
brew install protobuf
```

2. Install `grpcio` and `grpcio-tools`
```bash
python -m pip install --upgrade pip
python -m pip install grpcio
python -m pip install grpcio-tools
```

3. Run `setup.py` to install the bindings system wide
```bash
pip install .
```

# Structure
The gRPC call bindings is implemented in the [XironPythonInterface](./xiron_py/bindings.py). This opens up a channel and call the respective services. For now, the following functions are available:

1. `set_velocity`: Sets the velocity of a robot with a given Robot Id.
2. `get_pose`: Gets the pose of a robot with a given Robot Id.
3. `add_pose_subscriber`: Repeatedly calls the `GetPose` RPC to mimic a subscriber to the pose of a Robot.

# Example
```python
from xiron_py import XironPythonInterface

def pose_callback_funtion(msg):
    # Do something
    print(msg)

def lidar_callback_funtion(msg):
    # Do something
    print(msg)

if __name__ == '__main__':

    # Create an object
    xip = XironPythonInterface()

    # Set velocity
    xip.set_velocity("robot0", 0.25, 0.3)

    # Add subscriber to the Pose
    xip.add_pose_subscriber("robot0", pose_callback_funtion, 10)
    xip.add_lidar_subscriber("robot0", lidar_callback_funtion, 10)

    # Spin so that the thread is alive
    xip.spin()
```