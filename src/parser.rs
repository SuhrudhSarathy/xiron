use serde::{Serialize, Deserialize};
use serde_yaml;


#[derive(Debug, Deserialize, Serialize)]
pub struct WallConfig
{
    pub endpoints: Vec<(f32, f32)>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RobotConfig
{
    pub id: String,
    pub pose: (f32, f32, f32),
    pub vel: (f32, f32),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config
{
    pub robots: Vec<RobotConfig>,
    pub walls: Vec<WallConfig>
}

pub fn get_config_from_file(path: String) -> Config
{
    let file = std::fs::File::open(path).expect("File not openble");
    let config: Config = serde_yaml::from_reader(file).expect("Couldn't read config file. Rewrite properly");

    return config;
}