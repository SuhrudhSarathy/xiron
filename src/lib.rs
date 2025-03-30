//! `xiron` is a easy to use 2D robot simulator written in Rust.
//!
//! There are two goals for xiron. By offering an intuitive Simulator, it aims to lower the entrance barrier for anyone interested in robots.
//! The other goal is to assist roboticists in simulating algorithms and behaviors without the need of resource-intensive simulation tools.
//!
//! ## Features
//! 1. There is a single bianry to run a simulator with the GUI.
//! 2. A simple YAML based configuration settings.
//! 3. A simple Python interface is provided to communicate with the simulator.

pub mod algorithms;
pub mod behaviour;
pub mod camera_handler;
pub mod comms;
pub mod gui_interface;
pub mod handler;
pub mod object;
pub mod parameter;
pub mod parser;
pub mod prelude;
pub mod utils;
