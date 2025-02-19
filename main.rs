use sha2::{Sha256, Digest};
use reqwest::blocking::Client;
use serde_json::Value;
use std::time::Instant;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio::net::TcpStream;
use futures_util::{StreamExt, SinkExt};
use serde_json::json;
use url::Url;


/// Retrieves the latest block hash from a Bitcoin Core node
fn get_latest_block_hash() -> Option<String> {
    let client = Client::new();
    let url = "http://127.0.0.1:8332"; // Bitcoin Core RPC URL

    let body = serde_json::json!({
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
            return json["result"]["previousblockhash"].as_str().map(|s| s.to_string());
        }
    }

    None
}

/// Submits the mined block to Bitcoin Core
fn submit_block(block_data: &str) {
    let client = Client::new();
    let url = "http://127.0.0.1:8332";

    let body = serde_json::json!({
        "jsonrpc": "1.0",
        "id": "submitblock",
        "method": "submitblock",
        "params": [block_data]
    });

    let _ = client.post(url)
        .header("Content-Type", "application/json")
        .basic_auth("your_rpc_user", Some("your_rpc_password"))
        .json(&body)
        .send();
}

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

fn test_run() {
    let start_time = Instant::now();
    let data = "00000000000000000000cf03b5053b2fd56201405c84e8a873cb119ed013c63f";
    let mut nonce: u64 = 1725217284;
    let mut difficulty_prefix = "114167270716407.60";
    let input = format!("{}{}", data, nonce);
    
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    let hex_hash = format!("{:x}", result);

    // looking for 1425943111
    loop {
        let input = format!("{}{}", data, nonce);
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        let hex_hash = format!("{:x}", result);

        if hex_hash.starts_with(difficulty_prefix) {
            let duration = start_time.elapsed();
            println!("✅ Block Mined! Nonce: {}", nonce);
            println!("🔑 Hash: {}", hex_hash);
            println!("⏱️ Time Taken: {:.2?}", duration);
            submit_block(&hex_hash);
            break;
        }
        nonce += 1;
        if nonce % 1000000 == 0 {
            println!("Attempts: {} | Last hash: {}", nonce, hex_hash);
            break;
        }
    }

    println!("🔑 Test Mode - Computed Hash: {}", hex_hash);
}



#[tokio::main]
async fn main() {
    let mut nonce: u64 = 0;
    let start_time = Instant::now();
    test_run();
 
    let url = Url::parse("wss://ws.blockchain.info/inv").unwrap();
    
    println!("\n🔌 Connecting to Blockchain WebSocket API...");
    
    let (mut ws_stream, _) = connect_async(url).await.expect("\n🔴 WebSocket connection failed\n");
    println!("\n✅ Connected to WebSocket!\n");

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
                    //proof_of_work(prev_hash, merkle_root, difficulty);
                }
            }
        }
    }


    // 🔥 TEST MODE (COMMENT THIS OUT AFTER TESTING)
    
    // let test_data = "00000000000000000000dd7f073dbfb3b1af97416484b7ac961f530b5b830523";
    // let test_nonce: u64 = 3926389902;

    // let test_input = format!("{}{}", test_data, test_nonce);
    // let mut test_hasher = Sha256::new();
    // test_hasher.update(test_input.as_bytes());
    // let test_result = test_hasher.finalize();
    // let test_hex_hash = format!("{:x}", test_result);

    // println!("🔑 Test Mode - Computed Hash: {}", test_hex_hash);
    // println!("❌ Test Mode Disabled. Uncomment the block to enable it.");
    

    // Fetch real blockchain data
    // let data = match get_latest_block_hash() {
    //     Some(hash) => hash,
    //     None => {
    //         eprintln!("Failed to fetch latest block data.");
    //         return;
    //     }
    // };

    // println!("⛏️ Mining block with base hash: {}", data);

    // loop {
    //     let input = format!("{}{}", data, nonce);
    //     let mut hasher = Sha256::new();
    //     hasher.update(input.as_bytes());
    //     let result = hasher.finalize();
    //     let hex_hash = format!("{:x}", result);

    //     if hex_hash.starts_with(DIFFICULTY_PREFIX) {
    //         let duration = start_time.elapsed();
    //         println!("✅ Block Mined! Nonce: {}", nonce);
    //         println!("🔑 Hash: {}", hex_hash);
    //         println!("⏱️ Time Taken: {:.2?}", duration);

    //         submit_block(&hex_hash);
    //         break;
    //     }

    //     nonce += 1;

    //     if nonce % 1_000_000 == 0 {
    //         println!("Attempts: {} | Last hash: {}", nonce, hex_hash);
    //     }
    // }
}
