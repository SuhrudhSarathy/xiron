use xiron::prelude::*;

#[test]
fn check_jsonifying_from_config()
{
    let robot_config = RobotConfig {
        id: "robot0".to_string(),
        pose: (0.0, 0.0, 0.0),
        vel: (0.0, 0.0),
        lidar: false
    };

    let config = Config{
        robots: vec![robot_config.clone()],
        walls: Vec::new(),
        static_objects: Vec::new(),
    };

    let json_string = get_config_to_string(config);
    let config_reborn = get_config_from_string(json_string);
    
    let robot_from_config = config_reborn.robots[0].clone();
    assert_eq!(robot_from_config.id, robot_config.id);
    assert_eq!(robot_from_config.pose, robot_config.pose);
    assert_eq!(robot_from_config.vel, robot_config.vel);
    assert_eq!(robot_from_config.lidar, robot_config.lidar);

}