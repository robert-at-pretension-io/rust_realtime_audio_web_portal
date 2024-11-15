use futures::{SinkExt, StreamExt};
use http::Request;
use base64;
use rand::{Rng, thread_rng};
use serde_json::json;
use std::env;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use warp::ws::{Message as WarpMessage, WebSocket, Ws};
use warp::Filter;

fn generate_key() -> String {
    let mut key = [0u8; 16];
    thread_rng().fill(&mut key[..]);
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, key)
}

async fn handle_browser_client(browser_ws: WebSocket) {
    println!("Browser connected!");

    // Connect to OpenAI
    let openai_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let url = "wss://api.openai.com/v1/realtime?model=gpt-4o-realtime-preview-2024-10-01";
    
    let request = Request::builder()
        .uri(url)
        .header("Host", "api.openai.com")
        .header("Authorization", format!("Bearer {}", openai_key))
        .header("OpenAI-Beta", "realtime=v1")
        .header("Sec-WebSocket-Key", generate_key())
        .header("Sec-WebSocket-Version", "13")
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .body(())
        .expect("Failed to build request");

    let ws_stream = connect_async(request)
        .await
        .expect("Failed to connect to OpenAI");
    println!("Connected to OpenAI!");

    let (mut openai_write, mut openai_read) = ws_stream.0.split();
    let (mut browser_write, mut browser_read) = browser_ws.split();

    // Initialize OpenAI session
    let init_msg = json!({
        "type": "session.update",
        "session": {
            "modalities": ["text", "audio"],
            "instructions": "You are a helpful assistant.",
            "voice": "alloy"
        }
    });
    openai_write
        .send(TungsteniteMessage::Text(init_msg.to_string()))
        .await
        .expect("Failed to send init message");
    println!("Sent initialization message to OpenAI");

    // Forward browser messages to OpenAI
    tokio::spawn(async move {
        while let Some(msg) = browser_read.next().await {
            match msg {
                Ok(browser_msg) => {
                    println!("→ Browser to OpenAI: {}", browser_msg.to_str().unwrap_or("binary data"));
                    // Convert warp message to tungstenite message
                    let openai_msg = if browser_msg.is_text() {
                        TungsteniteMessage::Text(browser_msg.to_str().unwrap().to_string())
                    } else if browser_msg.is_binary() {
                        TungsteniteMessage::Binary(browser_msg.as_bytes().to_vec())
                    } else {
                        continue;
                    };
                    if let Err(e) = openai_write.send(openai_msg).await {
                        println!("Error sending to OpenAI: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    println!("Error reading from browser: {}", e);
                    break;
                }
            }
        }
        println!("Browser to OpenAI forwarding ended");
    });

    // Forward OpenAI messages to browser
    while let Some(msg) = openai_read.next().await {
        match msg {
            Ok(openai_msg) => {
                println!("← OpenAI to Browser: {}", openai_msg.to_string());
                // Convert tungstenite message to warp message 
                let browser_msg = match openai_msg {
                    TungsteniteMessage::Text(t) => WarpMessage::text(t),
                    TungsteniteMessage::Binary(b) => WarpMessage::binary(b),
                    _ => continue,
                };
                if let Err(e) = browser_write.send(browser_msg).await {
                    println!("Error sending to browser: {}", e);
                    break;
                }
            }
            Err(e) => {
                println!("Error reading from OpenAI: {}", e);
                break;
            }
        }
    }
    println!("OpenAI to Browser forwarding ended");
}

#[tokio::main]
async fn main() {
    env_logger::init();
    println!("Starting server...");

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: Ws| {
            ws.on_upgrade(|socket| handle_browser_client(socket))
        });

    println!("Server ready at ws://localhost:3000/ws");
    warp::serve(ws_route).run(([127, 0, 0, 1], 3000)).await;
}
