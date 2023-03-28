use tokio_tungstenite::connect_async;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use std::thread::sleep;
use std::time::Duration;

use xiron::prelude::*;

#[tokio::main]
async fn main()
{
    // Address of the Webscoket server
    let address = "ws://localhost:8081";

    // Connect to the Webscoket server
    let (ws_stream, _) = connect_async(address).await.expect("Couldnt connect to the websocket");

    let (mut writer, _) = ws_stream.split();

    // Construct the Twist Msg to send to Robot0
    let vel1 = Twist{id: String::from("robot0"), vel: (0.3, 0.2)};

    // Put it in the TwistArray container
    let twist_array = TwistArray{twists: vec![vel1]};

    // Send the velocity 10 times
    for _ in 0..10
    {
        let msg = twist_array.clone();
        let msg_as_string = serde_json::to_string(&msg).unwrap();
        writer.send(Message::Text(msg_as_string)).await.unwrap();

        sleep(Duration::from_secs(2));
    }
    
    // Send Zero velocity to stop the Robot
    let vel1 = Twist{id: String::from("robot0"), vel: (0.0, 0.0)};
    
    let twist_array = TwistArray{twists: vec![vel1]};
    let msg_as_string = serde_json::to_string(&twist_array).unwrap();
    
    writer.send(Message::Text(msg_as_string)).await.unwrap();

    // Close the websocket connection
    writer.send(Message::Close(None)).await.unwrap();
}