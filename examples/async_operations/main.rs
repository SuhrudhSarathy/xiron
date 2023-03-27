extern crate tokio;
extern crate xiron;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use xiron::prelude::*;

async fn step(sh: Arc<Mutex<SimulationHandler>>) {
    loop {
        let mut sh = sh.lock().await;
        sh.step();

        // Drop this so that other functions can use this while you sleep
        drop(sh);

        tokio::time::sleep(Duration::from_millis((1000.0 / 60.0) as u64)).await;
    }
}

async fn control(sh: Arc<Mutex<SimulationHandler>>, r: &RobotHandler, mut p: PathController) {
    loop {
        let mut sh = sh.lock().await;
        let current_pose = sh.get_pose(r);

        println!("Current Pose: {}, {}, {}", current_pose.0, current_pose.1, current_pose.2);
        let vel = p.control(&current_pose);

        sh.control(r, vel);

        // Drop this so that others can use this while you sleep
        drop(sh);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
#[tokio::main]
async fn main() {
    // Initialise the simulation handler and robot handlers
    let (sim_handler, robot_handlers) =
        SimulationHandler::from_file("examples/keyboard_input/congif.yaml".to_owned());

    // Get the handler for a single robot
    let (_, robot0_handle) = robot_handlers[0].clone();

    // Make the mutex
    let simhandler_mutex = Arc::new(Mutex::new(sim_handler));
    let simhandler_mutex2 = Arc::clone(&simhandler_mutex);
    // Path controller
    let path = vec![
        (2.0, 0.0, 0.0),
        (2.0, 4.0, 0.0),
        (0.0, 4.0, 0.0),
        (0.0, 0.0, 0.0),
    ];
    let mut path_controller = PathController::new(robot0_handle, 0.1, 0.1);

    path_controller.set_path(path);

    // Spawn tokio tasks
    let step_task = tokio::spawn(async move {
        step(simhandler_mutex).await;
    });

    // Task to control the robot. This can be an IO intensive task where you take
    // some input from a Websocket etc.
    let control_task = tokio::spawn(async move {
        control(simhandler_mutex2, &robot0_handle, path_controller).await;
    });
    
    // wait for the tasks
    step_task.await.unwrap();
    control_task.await.unwrap();
}
