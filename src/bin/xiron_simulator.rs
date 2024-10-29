use macroquad::prelude::*;
use serde_json::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use xiron::comms::Twist;
use xiron::prelude::*;

use std::env::args;

#[macroquad::main(xiron)]
async fn main() {
    println!("Xiron Simulator!");

    let context = zmq::Context::new();

    let pose_publisher = Publisher::new(&context, "pose".to_string());
    pose_publisher.bind("tcp://127.0.0.1:5555".to_string());

    let scan_publisher = Publisher::new(&context, "scan".to_string());
    scan_publisher.bind("tcp://127.0.0.1:5858".to_string());

    let (string_sender, string_reciever) = std::sync::mpsc::channel();
    let vel_subscriber = Subscriber::new(
        &context,
        "vel".to_string(),
        Duration::from_millis(25),
        string_sender,
    );

    vel_subscriber.bind("tcp://127.0.0.1:5556".to_string());
    vel_subscriber.spin();

    let (reset_sender, reset_reciever) = std::sync::mpsc::channel();
    let reset_subscriber = Subscriber::new(
        &context,
        "reset".to_string(),
        Duration::from_millis(25),
        reset_sender,
    );

    reset_subscriber.bind("tcp://127.0.0.1:5956".to_string());
    reset_subscriber.spin();

    let (sender, reciever) = std::sync::mpsc::channel();
    let (save_sender, save_reciever) = std::sync::mpsc::channel();

    let sim_handler = SimulationHandler::new();
    let sim_handler_mutex = Arc::new(Mutex::new(sim_handler));
    let sim_handler_mutex_clone = Arc::clone(&sim_handler_mutex);
    let mut egui_handler = EguiInterface::new(sender, save_sender, sim_handler_mutex);
    let mut last_sent_time: Option<f64> = None;

    // Parse the CLI args for file path
    let file_path_arg = std::env::args().nth(1);
    match file_path_arg {
        Some(file_path) => {
            println!("Starting simulator with input path: {}", file_path);
            let mut sh = sim_handler_mutex_clone.lock().unwrap();
            sh.load_file_path(file_path);
            let robot_handlers = sh.reset();
            egui_handler.reset_robot_handlers(robot_handlers);
        }
        None => {
            println!("No file passed as argument. Continuing without loading file");
        }
    }

    // Main simulation Loop
    loop {
        clear_background(WHITE);

        match reciever.try_recv() {
            Ok(message) => {
                println!("Got Open message here: {}", message);
                let mut sh = sim_handler_mutex_clone.lock().unwrap();
                sh.load_file_path(message);
                let robot_handlers = sh.reset();
                egui_handler.reset_robot_handlers(robot_handlers);
            }
            Err(_) => {}
        }

        match save_reciever.try_recv() {
            Ok(message) => {
                println!("Got Save message here: {}", message);
                let sh = sim_handler_mutex_clone.lock().unwrap();
                let config = sh.to_config();

                let f = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(message)
                    .expect("Couldn't open file");
                serde_yaml::to_writer(f, &config).unwrap();
            }
            Err(_) => {}
        }

        match egui_handler.play {
            PlayMode::Pause => {}
            PlayMode::Play => {
                let mut sh = sim_handler_mutex_clone.lock().unwrap();
                sh.step();
            }
        }
        {
            let sh = sim_handler_mutex_clone.lock().unwrap();
            sh.draw_lines();
        }

        // draw the Egui stuff and macroquad stuff also here only
        egui_macroquad::ui(|egui_ctx| egui_handler.show_elements(egui_ctx));

        {
            let sh = sim_handler_mutex_clone.lock().unwrap();
            sh.draw();
        }

        egui_macroquad::draw();

        // Check if there were any velocity messages to process.
        {
            let mut sh = sim_handler_mutex_clone.lock().unwrap();
            let output = string_reciever.try_recv();
            match output {
                Ok(output) => {
                    let twist_command_result: Result<Twist, Error> =
                        serde_json::from_str(&output.to_string());
                    match twist_command_result {
                        Ok(twist_command) => {
                            let robot_handler =
                                egui_handler.get_robot_handler(&twist_command.robot_id);
                            match robot_handler {
                                None => {}
                                Some(handler) => {
                                    sh.control(
                                        &handler,
                                        (twist_command.linear.0, twist_command.angular),
                                    );
                                }
                            }
                        }
                        Err(err) => {
                            println!("Got Error in parsing velocities: {err}");
                        }
                    }
                }
                Err(_error) => {}
            }
        }

        // Check for reset request
        let reset_output = reset_reciever.try_recv();
        match reset_output {
            Ok(_output) => {
                println!("Resetting Environment");
                egui_handler.reset();
            }
            Err(_error) => {}
        }

        let time_now = get_time();
        match last_sent_time {
            None => {
                last_sent_time = Some(get_time());
                let sh = sim_handler_mutex_clone.lock().unwrap();
                for robot_name in egui_handler.robot_name_map.keys() {
                    let handler = egui_handler.get_robot_handler(robot_name);
                    match handler {
                        Some(robot) => {
                            let pose = sh.get_pose(&robot);
                            let pose_msg = Pose {
                                robot_id: robot_name.clone(),
                                position: (pose.0, pose.1),
                                orientation: pose.2,
                            };
                            pose_publisher.send(&pose_msg);

                            let scan = sh.sense(&robot);
                            let scan_msg = LaserScan {
                                robot_id: robot_name.clone(),
                                angle_min: scan.angle_min,
                                angle_max: scan.angle_max,
                                num_readings: scan.num_readings,
                                values: scan.values,
                            };
                            scan_publisher.send(&scan_msg);
                        }
                        None => {}
                    }
                }
            }

            // Publish the Scan data and Pose data for each robot present.
            Some(t_last) => {
                if (time_now - t_last) > (1.0 / DATA_SEND_FREQ) {
                    last_sent_time = Some(get_time());
                    let sh = sim_handler_mutex_clone.lock().unwrap();
                    for robot_name in egui_handler.robot_name_map.keys() {
                        let handler = egui_handler.get_robot_handler(robot_name);
                        match handler {
                            Some(robot) => {
                                let pose = sh.get_pose(&robot);
                                let pose_msg = Pose {
                                    robot_id: robot_name.clone(),
                                    position: (pose.0, pose.1),
                                    orientation: pose.2,
                                };
                                pose_publisher.send(&pose_msg);

                                let scan = sh.sense(&robot);
                                let scan_msg = LaserScan {
                                    robot_id: robot_name.clone(),
                                    angle_min: scan.angle_min,
                                    angle_max: scan.angle_max,
                                    num_readings: scan.num_readings,
                                    values: scan.values,
                                };
                                scan_publisher.send(&scan_msg);
                            }
                            None => {}
                        }
                    }
                }
            }
        }

        next_frame().await;
    }
}
