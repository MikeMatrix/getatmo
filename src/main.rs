use regex::Regex;
use reqwest::{self, Client};
use serde_json::Value;
use std::{collections::HashMap, env};

async fn request_token(client: &Client) -> Option<String> {
    let Ok(response) = client.get("https://weathermap.netatmo.com").send().await else {
        return None;
    };
    let Ok(body) = response.text().await else {
        return None;
    };
    let regex = Regex::new(r#"(?m)accessToken:\s"(?<token>([a-z0-9]+)\|([a-z0-9]+))","#).unwrap();
    let result = regex.captures(&body);
    let Some(catpure) = result else {
        return None;
    };

    Some(catpure["token"].to_string())
}

async fn request_data(client: &Client, token: String, device_id: String) -> HashMap<String, f64> {
    let mut result_map = HashMap::new();
    let Ok(repsonse) = client
        .post("https://app.netatmo.net/api/getpublicmeasure")
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "device_id": device_id,
        }))
        .send()
        .await else {
            return result_map;
        };

    let Ok(data) = repsonse.text().await else {
        return result_map;
    };

    let Ok(value) = serde_json::from_str::<Value>(data.as_str()) else {
        return result_map;
    };

    let Some(measures) = &value["body"][0]["measures"].as_object() else {
        return result_map;
    };

    let empty_map = serde_json::map::Map::new();
    let empty_vec = Vec::new();

    for (_, measure) in measures.iter() {
        let res = measure["res"].as_object().unwrap_or(&empty_map);
        let data_types = measure["type"].as_array().unwrap_or(&empty_vec);

        for (_, data) in res.iter() {
            let data = data.as_array().unwrap_or(&empty_vec);

            for (i, v) in data.iter().enumerate() {
                let data_type = data_types.get(i).and_then(Value::as_str).unwrap_or("");
                let value = v.as_f64().unwrap_or(0.0);
                result_map.insert(data_type.to_string(), value);
            }
        }
    }
    return result_map;
}

#[tokio::main]
async fn main() {
    let mut args = env::args();

    let Some(program) = args.next() else {
        println!("Error: Something went very wrong!");
        return;
    };

    let Some(device_id) = args.next() else {
        println!("Error: {} <device_id>", program);
        return;
    };

    let client = reqwest::Client::new();
    let Some(token) = request_token(&client).await else {
        println!("Error: No token found");
        return;
    };
    let result_map = request_data(&client, token, device_id).await;
    let output = serde_json::to_string(&result_map).unwrap_or("{}".to_string());
    println!("{}", output);
}
