use macroquad::prelude::*;
use pose_msg::PositionMsg;
use prost::Message;
use prost_types::Any;
use std::sync::{Arc, Mutex};

use xiron::prelude::*;

use std::time::{SystemTime, UNIX_EPOCH};

#[macroquad::main(xiron)]
async fn main() {
    println!("Xiron Simulator!");

    let ws_subscriber_addr = "localhost:9000";
    let ws_publisher_addr = "localhost:9001";

    let ws_subcriber = WebsocketSubscriber::new(ws_subscriber_addr.to_string());
    let ws_publisher = WebsocketPublisher::new(ws_publisher_addr.to_string());

    let pub_tx = ws_publisher.start();
    let sub_rx = ws_subcriber.start();

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

        egui_macroquad::draw();

        // Check if there were any velocity messages to process.
        let recieved_msg = sub_rx.try_recv();
        match recieved_msg {
            Ok(msg) => {
                let protobuf_message = Any::decode(msg.as_slice());
                match protobuf_message {
                    Ok(msg) => {
                        if msg.type_url == "vel" {
                            let twist_command_result = TwistMsg::decode(msg.value.as_slice());
                            match twist_command_result {
                                Ok(twist_command) => {
                                    let robot_handler =
                                        egui_handler.get_robot_handler(&twist_command.robot_id);
                                    match robot_handler {
                                        None => {}
                                        Some(handler) => {
                                            let mut sh = sim_handler_mutex_clone.lock().unwrap();
                                            let twist = twist_command.linear.unwrap();
                                            sh.control(
                                                &handler,
                                                (twist.x, twist_command.angular),
                                            );
                                        }
                                    }
                                }
                                Err(err) => {
                                    println!("Got Error in parsing velocities: {err}");
                                }
                            }
                            
                        } else if msg.type_url == "reset" {
                            egui_handler.reset();
                        }
                    }
                    Err(err) => {
                        error!("Error in decoding json message: {}", err)
                    }
                }
            }
            Err(_error) => {}
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
                            position: Some(PositionMsg{x: pose.0, y: pose.1}),
                            orientation: pose.2,
                        };

                        let packed_msg = Any{
                            type_url: "pose".to_string(),
                            value: pose_msg.encode_to_vec(),

                        };
                        match pub_tx.send(packed_msg.encode_to_vec()) {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Got error when sending pose via channel {}", e);
                            }
                        }
                        let scan = sh.sense(&robot);
                        let scan_msg = LaserScanMsg {
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs_f64(),
                            robot_id: robot_name.clone(),
                            angle_min: scan.angle_min,
                            angle_max: scan.angle_max,
                            num_readings: scan.num_readings,
                            values: scan.values,
                        };
                        let packed_msg = Any { type_url: "scan".to_string(), value: scan_msg.encode_to_vec() };
                        match pub_tx.send(packed_msg.encode_to_vec()) {
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
