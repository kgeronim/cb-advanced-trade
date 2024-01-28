use chrono::{Duration, Utc};
use jwt_compact::{alg::Es256, AlgorithmExt, Claims, Header, TimeOptions};
use p256::SecretKey;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;

#[tokio::main]
async fn main() {
    let data = fs::read_to_string("./coinbase_cloud_api_key.json").expect("Unable to read file");
    let json: serde_json::Value = serde_json::from_str(&data).unwrap();

    let private_key = json["privateKey"].to_string();
    let private_key: String = serde_json::from_str(&private_key).unwrap();

    let name = json["name"].to_string();
    let name: String = serde_json::from_str(&name).unwrap();

    // println!("{}", private_key);
    let mut bytes = [0; 16];
    rand::thread_rng().fill_bytes(&mut bytes);

    let time_options = TimeOptions::default();

    let secret_key = SecretKey::from_sec1_pem(&private_key).unwrap();
    let header = Header::new(PrivateHeader {
        nonce: hex::encode(&bytes),
    })
    .with_key_id(name.clone());

    let claims = Claims::new(PrivateClaims {
        uri: "GET api.coinbase.com/api/v3/brokerage/accounts".to_string(),
        sub: name,
        aud: "retail_rest_api_proxy".to_string(),
        iss: "coinbase-cloud".to_string(),
    })
    .set_not_before(Utc::now())
    .set_duration(&time_options, Duration::seconds(120));

    let key = Es256.token(&header, &claims, &secret_key.into()).unwrap();

    // println!("{}", key);

    let client = reqwest::Client::new();
    let resp = client.get("https://api.coinbase.com/api/v3/brokerage/accounts")
        .bearer_auth(key)
        .send()
        .await.unwrap().text().await.unwrap();

    println!("{}", resp);
}

// Define our own private claims
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct PrivateClaims {
    uri: String,
    sub: String,
    aud: String,
    iss: String,
}

// Define our own private claims
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct PrivateHeader {
    nonce: String,
}
