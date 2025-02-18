use tokio_tungstenite::connect_async;
use tokio::net::TcpStream;
use futures_util::{StreamExt, SinkExt};
use serde_json::{json, Value};
use sha2::{Sha256, Digest};
use url::Url;

/// Performs proof-of-work mining
fn proof_of_work(prev_hash: &str, merkle_root: &str, target_difficulty: &str) {
    let mut nonce = 0;
    let difficulty_prefix = "0".repeat(target_difficulty.len());

    loop {
        let input = format!("{}{}{}", prev_hash, merkle_root, nonce);
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let hash_result = hasher.finalize();
        let hash_hex = hex::encode(hash_result);

        if hash_hex.starts_with(&difficulty_prefix) {
            println!("✅ Block Mined! Nonce: {}", nonce);
            println!("🔗 Hash: {}", hash_hex);
            break;
        }

        nonce += 1;
    }
}

#[tokio::main]
async fn main() {
    let url = Url::parse("wss://ws.blockchain.info/inv").unwrap();
    
    println!("🔌 Connecting to Blockchain WebSocket API...");
    
    let (mut ws_stream, _) = connect_async(url).await.expect("🔴 WebSocket connection failed");
    println!("✅ Connected to WebSocket!");

    // Subscribe to new block notifications
    let subscribe_msg = json!({ "op": "blocks_sub" }).to_string();
    ws_stream.send(subscribe_msg.into()).await.unwrap();
    println!("📡 Subscribed to new Bitcoin blocks...");

    while let Some(msg) = ws_stream.next().await {
        if let Ok(msg) = msg {
            if let Ok(text) = msg.to_text() {
                let parsed: Value = serde_json::from_str(text).unwrap();

                if parsed["op"] == "block" {
                    let block = &parsed["x"];
                    let prev_hash = block["hash"].as_str().unwrap_or("");
                    let merkle_root = block["mrklRoot"].as_str().unwrap_or("");
                    let difficulty = "0000"; // Simulated difficulty

                    println!("🔹 New Block Found!");
                    println!("📌 Prev Hash: {}", prev_hash);
                    println!("🌿 Merkle Root: {}", merkle_root);

                    // Start mining
                    proof_of_work(prev_hash, merkle_root, difficulty);
                }
            }
        }
    }
}
