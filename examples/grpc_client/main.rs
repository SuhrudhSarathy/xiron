pub mod xiron_interfaces {
    tonic::include_proto!("xiron_interfaces");
}

use xiron_interfaces::{xiron_interface_client::XironInterfaceClient, VelocityRequest};
use xiron_interfaces::{EmptyRequest, LidarRequest, PoseRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = XironInterfaceClient::connect("http://[::1]:8081").await?;

    println!("Connected to client successfully");

    // Get pose
    let mut pose = client
        .get_pose(PoseRequest {
            id: "robot0".into(),
        })
        .await?;
    let pinto = pose.into_inner();
    println!("{}, {}, {}", pinto.x, pinto.y, pinto.theta);

    for i in 0..10 {
        let response = client
            .set_velocity(VelocityRequest {
                id: "robot0".into(),
                v: i as f64 * 0.1,
                w: 0.5,
            })
            .await?;

        let lidar_scan_response = client
            .get_lidar_scan(LidarRequest {
                id: "robot0".into(),
            })
            .await?;

        println!("{}", lidar_scan_response.into_inner().num_readings);

        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    pose = client
        .get_pose(PoseRequest {
            id: "robot0".into(),
        })
        .await?;
    let pinto = pose.into_inner();
    println!("{}, {}, {}", pinto.x, pinto.y, pinto.theta);

    let _resp = client.reset_env(EmptyRequest {}).await?;

    pose = client
        .get_pose(PoseRequest {
            id: "robot0".into(),
        })
        .await?;
    let pinto = pose.into_inner();
    println!("Pose after reset {}, {}, {}", pinto.x, pinto.y, pinto.theta);

    for i in 0..10 {
        let response = client
            .set_velocity(VelocityRequest {
                id: "robot0".into(),
                v: i as f64 * 0.1,
                w: 0.5,
            })
            .await?;

        let lidar_scan_response = client
            .get_lidar_scan(LidarRequest {
                id: "robot0".into(),
            })
            .await?;

        println!("{}", lidar_scan_response.into_inner().num_readings);

        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    Ok(())
}
