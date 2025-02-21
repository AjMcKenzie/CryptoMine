use sha2::{Sha256, Digest};
use reqwest::blocking::Client;
use serde_json::Value;
use std::time::Instant;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json::json;
use url::Url;

/// Retrieves the latest block hash from a Bitcoin Core node
fn get_latest_block_hash() -> Option<String> {
    let client = Client::new();
    let url = "http://127.0.0.1:8332"; // Bitcoin Core RPC URL

    let body = json!({
        "jsonrpc": "1.0",
        "id": "getblocktemplate",
        "method": "getblocktemplate",
        "params": [{"rules": ["segwit"]}]
    });

    if let Ok(response) = client.post(url)
        .header("Content-Type", "application/json")
        .basic_auth("your_rpc_user", Some("your_rpc_password")) // Replace with your credentials
        .json(&body)
        .send()
    {
        if let Ok(json) = response.json::<Value>() {
            return json.get("result")
                .and_then(|r| r.get("previousblockhash"))
                .and_then(|h| h.as_str())
                .map(|s| s.to_string());
        }
    }

    None
}

/// Submits the mined block to Bitcoin Core
fn submit_block(block_data: &str) {
    let client = Client::new();
    let url = "http://127.0.0.1:8332";

    let body = json!({
        "jsonrpc": "1.0",
        "id": "submitblock",
        "method": "submitblock",
        "params": [block_data]
    });

    match client.post(url)
        .header("Content-Type", "application/json")
        .basic_auth("your_rpc_user", Some("your_rpc_password"))
        .json(&body)
        .send()
    {
        Ok(res) => println!("✔️ Block submission response: {:?}", res.text().unwrap_or_default()),
        Err(e) => eprintln!("❌ Error submitting block: {:?}", e),
    }
}

/// Performs proof-of-work mining with real difficulty comparison
fn proof_of_work(prev_hash: &str, merkle_root: &str, target_difficulty: &str) {
    let mut nonce = 0;
    let difficulty_num = u128::from_str_radix(target_difficulty, 16).unwrap_or(u128::MAX);

    loop {
        let input = format!("{}{}{}", prev_hash, merkle_root, nonce);
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let hash_result = hasher.finalize();
        let hash_hex = hex::encode(hash_result);

        let hash_num = u128::from_str_radix(&hash_hex[..32], 16).unwrap_or(u128::MAX);

        if hash_num <= difficulty_num {
            println!("✅ Block Mined! Nonce: {}", nonce);
            println!("🔗 Hash: {}", hash_hex);
            submit_block(&hash_hex);
            break;
        }

        nonce += 1;
    }
}

/// Test mining with preset values
fn test_run() {
    let start_time = Instant::now();
    let data = "00000000000000000000cf03b5053b2fd56201405c84e8a873cb119ed013c63f";
    let mut nonce: u64 = 1725217284;
    let difficulty_prefix = "000000000000000000000000000000000000";

    loop {
        let input = format!("{}{}", data, nonce);
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        let hex_hash = format!("{:x}", result);

        if hex_hash.starts_with(&difficulty_prefix) {
            let duration = start_time.elapsed();
            println!("✅ Block Mined! Nonce: {}", nonce);
            println!("🔑 Hash: {}", hex_hash);
            println!("⏱️ Time Taken: {:.2?}", duration);
            submit_block(&hex_hash);
            break;
        }
        nonce += 1;

        if nonce % 1_000_000 == 0 {
            println!("Attempts: {} | Last hash: {}", nonce, hex_hash);
        }
    }
}

/// Handles WebSocket connection with auto-reconnect logic
async fn connect_websocket() {
    loop {
        let url = Url::parse("wss://ws.blockchain.info/inv").unwrap();
        println!("\n🔌 Connecting to Blockchain WebSocket API...");

        match connect_async(url).await {
            Ok((mut ws_stream, _)) => {
                println!("\n✅ Connected to WebSocket!\n");

                // Subscribe to new block notifications
                let subscribe_msg = json!({ "op": "ping_block" }).to_string();
                if let Err(err) = ws_stream.send(Message::Text(subscribe_msg)).await {
                    eprintln!("❌ Subscription error: {:?}", err);
                    continue;
                }
                println!("📡 Subscribed to new Bitcoin blocks...");

                // Handle incoming messages
                while let Some(msg) = ws_stream.next().await {
                    if let Ok(msg) = msg {
                        if let Ok(text) = msg.to_text() {
                            if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                                if let Some(op) = parsed.get("op").and_then(|o| o.as_str()) {
                                    if op == "block" {
                                        if let Some(block) = parsed.get("x") {
                                            let prev_hash = block.get("hash").and_then(|h| h.as_str()).unwrap_or("UNKNOWN");
                                            let merkle_root = block.get("mrklRoot").and_then(|m| m.as_str()).unwrap_or("UNKNOWN");
                                            let difficulty = "0000"; // Simulated difficulty

                                            println!("🔹 New Block Found!");
                                            println!("📌 Prev Hash: {}", prev_hash);
                                            println!("🌿 Merkle Root: {}", merkle_root);

                                            // Start mining
                                            // proof_of_work(prev_hash, merkle_root, difficulty);
                                        }
                                    }
                                }
                            } else {
                                eprintln!("❌ Failed to parse WebSocket message!");
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("🔴 WebSocket connection failed: {:?}", e);
            }
        }

        println!("🔄 Reconnecting in 5 seconds...");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

/// Entry point
#[tokio::main]
async fn main() {
    println!("Starting test mining...");
    test_run();

    println!("Starting WebSocket connection...");
    connect_websocket().await;
}
