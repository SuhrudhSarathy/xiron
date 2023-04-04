<p align="center">
    <img src="images/logo.png" 
        alt="Picture" 
        width="200" 
        height="200" 
        style="display: block; margin: 0 auto" />
</p>

# Xiron
A lightweight 2D robot simulator written in Rust.

# Dependencies
1. Install the Rust Programming language : https://www.rust-lang.org/tools/install
2. Install Protobufs
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

# Installation
1. Clone the repository
```
git clone https://github.com/SuhrudhSarathy/xiron.git
```
2. Use `cargo` to build
```
cd xiron/
cargo build
```
or
```
cargo build --release
```

# Simulator
There are multiple ways to use Xiron as a simulator. You can simulate your configuration using a simple loop where you update the kinematics and render sequentially or use two different processes to go completely async. The example [path_tracking](examples/path_tracking) gives an example of the first method.

The simulator can also be run asynchronously with or without rendering. The [Simulator](src/bin/simulator.rs) updates the kinematics, performs collision checks etc. and the [Renderer](src/bin/renderer.rs) renders the scene asynchronously. This allows us to run the simulation in headless mode without having to render the scene.

The __Simulator__ communicates with the __Renderer__ about the current scene using ZeroMQ messaging queue. The __Simulator__ implements gRPC server communication. The services `GetPose` and `SetVelocity` get the pose of a robot and set the velocity of the robot respectively. An example Rust gRPC client has been written as an example.

## Instructions
1. Run the simulation server using cargo
```bash
cargo run --bin simulator /path/to/config
```

2. Run the rendered using cargo
```bash
cargo run --bin renderer /path/to/config
```

3. Run the client
```bash
cargo run --example grpc_client
```
The client sends velocity commands to the server to command the robot to move in circles. You can use the config file from [here](examples/websocket_client/config.yaml)

## Export Simulator and Renderer
Since rust provides completely self-contained binaries, exporting the _simulator_ and _renderer_ binaries and running them standalone should work fine.

# Bindings
Since the simulation exposes gRPC services, it is easy to write clients in different languages. The bindings to the respective gRPC clients are in the [bindings](bindings) folder. The installation instructions are given in the particular folder of the bindings.

# Examples
Examples are in the [examples](examples) directory.

<p align="center">
    <img src="images/screen.png" 
        alt="Picture" 
        width="500" 
        height="500" 
        style="display: block; margin: 0 auto" />
</p>

# World Editor
You can use the [World Editor](src/bin/world_editor.rs) to create a config file using a GUI. That config file can be then used to load the simulation world.

<p align="center">
    <img src="images/world_editor.png" 
        alt="Picture" 
        width="500" 
        height="500" 
        style="display: block; margin: 0 auto" />
</p>

# Roadmap
The Roadmap is mentioned in the [Projects](https://github.com/SuhrudhSarathy/xiron/projects) section.

# References
1. Rust Language Reference: https://www.rust-lang.org
2. gRPC: https://grpc.io