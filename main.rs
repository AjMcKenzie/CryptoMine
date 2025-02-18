use sha2::{Sha256, Digest};
use reqwest::blocking::Client;
use serde_json::Value;
use std::time::Instant;

const DIFFICULTY_PREFIX: &str = "0000"; // Adjust for real mining

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

fn main() {
    let mut nonce: u64 = 0;
    let start_time = Instant::now();

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
