# Installation
There are two modes of Installation.
### 1. Binaries
You can directly download the simulator from the binaries and execute them. The binaries are available [here]()

### 2. Source Installation
#### a. Dependencies Installation
* Install the Rust Programming language from their [official website](https://www.rust-lang.org/learn/get-started)

* On Linux, you need to first run:
```
sudo apt-get install -y libclang-dev libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```

* On Fedora Rawhide, you need to run
```
dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel
```
There are no dependencies apart from Rust for `Windows` and `Mac`.

To install from source, you will need to install the Rust Programming language.

#### b. Source Code
Clone the source code from Github
```
git clone https://github.com/SuhrudhSarathy/xiron.git
```

#### c. Build Library and Simulator
1. To build the library and simulator run:
```
cargo build
```
By default, this will build the library and simulator in debug mode. To build in release mode,
```
cargo build --release
```

2. The built simulator can be found as a binary in the `target/debug` or `target/release` folder depending on whether the build was in debug mode or release mode.

3. You can also run the simulator using cargo with
```
cargo run --bin xiron_simulator
```
This runs the simulator in debug mode. To run in release mode,
```
cargo run --bin xiron_simulator --release
```
