use serde::{Deserialize, Serialize};
use serde_yaml;

use crate::prelude::DriveType;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WallConfig {
    pub endpoints: Vec<(f32, f32)>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StaticObjConfig {
    pub center: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RobotConfig {
    pub id: String,
    pub pose: (f32, f32, f32),
    pub vel: (f32, f32, f32),
    pub lidar: bool,
    pub footprint: Vec<f32>,
    pub drive_type: DriveType,
    pub add_noise: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub robots: Vec<RobotConfig>,
    pub walls: Vec<WallConfig>,
    pub static_objects: Vec<StaticObjConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Twist {
    pub id: String,
    pub vel: (f32, f32, f32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TwistArray {
    pub twists: Vec<Twist>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub pose: (f32, f32, f32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PositionArray {
    pub positions: Vec<Position>,
}

pub fn get_config_from_file(path: String) -> Option<Config> {
    let file_result = std::fs::File::open(path);
    match file_result {
        Ok(file) => {
            let config: Config =
                serde_yaml::from_reader(file).expect("Couldn't read config file. Rewrite properly");

            return Some(config);
        }
        Err(e) => {
            println!("Error in opening file: {}", e);
            return None;
        }
    }
}
