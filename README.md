<p align="center">
    <img src="images/xiron.png" 
        alt="Picture" 
        style="display: block; margin: 0 auto" />
</p>

# Xiron
A lightweight 2D robot simulator written in Rust. The main documentation and How to Use guide can be found [here](https://suhrudhsarathy.github.io/xiron/)

# Dependencies
1. Install the Rust Programming language : https://www.rust-lang.org/tools/install

# Installation
1. Clone the repository
```
git clone https://github.com/SuhrudhSarathy/xiron.git
```
2. Use `cargo` to build the library along with the binary
```
cargo build --release
```

# Xiron-Simulator
The main simulator is a binary. To run the binary
```
cargo run --bin xiron_simulator
```
Also, to install the Simulator and use it globally:
```
cargo install --path .   
```
Make sure you are inside the `xiron` directory. This builds the binary and places it in `$HOME/.cargo/bin` folder. Make sure that `PATH` is set properly. After this, open up a new terminal and run the simulator using the command:
```
xiron_simulator
```

# Roadmap
The Roadmap is mentioned in the [Projects](https://github.com/SuhrudhSarathy/xiron/projects) section.
