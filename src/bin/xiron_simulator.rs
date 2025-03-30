use macroquad::prelude::*;
use pose_msg::PositionMsg;
use std::sync::{Arc, Mutex};

use xiron::prelude::*;

use std::time::{SystemTime, UNIX_EPOCH};

#[macroquad::main(xiron)]
async fn main() {
    println!("Xiron Simulator!");

    static XIRON_COMM_SERVER_ADDR: &str = "localhost";
    let s2c_port = 9000;
    let c2s_port = 9001;
    let xiron_comm_server = XironCommServer::new(XIRON_COMM_SERVER_ADDR, s2c_port, c2s_port);

    let (xiron_comm_server_tx, xiron_comm_server_rx) = xiron_comm_server.start();

    let (open_sender, open_reciever) = std::sync::mpsc::channel();
    let (save_sender, save_reciever) = std::sync::mpsc::channel();

    let sim_handler = SimulationHandler::new();
    let sim_handler_mutex = Arc::new(Mutex::new(sim_handler));
    let sim_handler_mutex_clone = Arc::clone(&sim_handler_mutex);
    let mut egui_handler = EguiInterface::new(open_sender, save_sender, sim_handler_mutex);
    let mut last_sent_time: Option<f64> = None;

    // Parse the CLI args for file path
    let file_path_arg = std::env::args().nth(1);
    match file_path_arg {
        Some(file_path) => {
            if file_path == "" {
                println!("Empty File path. Not opening");
            } else {
                println!("Starting simulator with input path: {}", file_path);
                let mut sh = sim_handler_mutex_clone.lock().unwrap();
                sh.load_file_path(file_path);
                let robot_handlers = sh.reset();
                egui_handler.reset_robot_handlers(robot_handlers);
            }
        }
        None => {
            println!("No file passed as argument. Continuing without loading file");
        }
    }

    let mut rate = LoopRateHandler::new(1.0 / DT as f64);
    rate.sleep();

    // Main simulation Loop
    loop {
        clear_background(WHITE);

        match open_reciever.try_recv() {
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

        egui_handler.set_and_update_camera();
        egui_macroquad::draw();

        // Check if there were any velocity messages to process.
        let try_recieving_message = xiron_comm_server_rx.try_recv();
        if let Ok(message) = try_recieving_message {
            match message {
                Ok(comm_resp) => {
                    match comm_resp {
                        CommResponse::Reset(_reset_msg) => {
                            println!("Resetting the simulation");

                            // This resets the simulation handler also.
                            egui_handler.reset();
                        }
                        CommResponse::Twist(twist_msg) => {
                            let robot_handler = egui_handler.get_robot_handler(&twist_msg.robot_id);
                            match robot_handler {
                                Some(handler) => {
                                    let mut sh = sim_handler_mutex_clone.lock().unwrap();
                                    let linear = twist_msg.linear.unwrap();
                                    let angular = twist_msg.angular;

                                    // Set the control value
                                    sh.control(&handler, (linear.x, linear.y, angular));
                                }
                                None => {
                                    println!(
                                        "Robot: {} does not exist in simulation",
                                        twist_msg.robot_id
                                    );
                                }
                            }
                        }
                        _ => {
                            // Ignore any other type.
                        }
                    }
                }
                Err(e) => {
                    println!("Error in recieving from Websocket: {}", e.reason);
                }
            }
        }

        let time_now = get_time();
        let mut send_message: bool = false;
        match last_sent_time {
            None => {
                send_message = true;
            }

            // Publish the Scan data and Pose data for each robot present.
            Some(t_last) => {
                if (time_now - t_last) > (1.0 / DATA_SEND_FREQ) {
                    send_message = true;
                }
            }
        }

        if send_message {
            last_sent_time = Some(get_time());
            let sh = sim_handler_mutex_clone.lock().unwrap();
            for robot_name in egui_handler.robot_name_map.keys() {
                let handler = egui_handler.get_robot_handler(robot_name);
                match handler {
                    Some(robot) => {
                        let pose = sh.get_pose(&robot);
                        let pose_msg = PoseMsg {
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs_f64(),
                            robot_id: robot_name.clone(),
                            position: Some(PositionMsg {
                                x: pose.0,
                                y: pose.1,
                            }),
                            orientation: pose.2,
                        };

                        let resp = CommResponse::Pose(pose_msg);
                        match xiron_comm_server_tx.send(Ok(resp)) {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Got error when sending pose via channel {}", e);
                            }
                        }
                        let scan = sh.sense(&robot);
                        let scan_resp = CommResponse::from((scan, robot_name.clone()));
                        match xiron_comm_server_tx.send(Ok(scan_resp)) {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Got error when sending scan via channel {}", e);
                            }
                        }
                    }
                    None => {}
                }
            }
        }

        next_frame().await;
        rate.sleep();
    }
}
