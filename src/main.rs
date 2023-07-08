extern crate regex;
extern crate serde;
extern crate serde_json;

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

async fn request_data(
    client: &Client,
    token: String,
    device_id: String,
) -> Result<HashMap<String, f64>, String> {
    let Ok(repsonse) = client
        .post("https://app.netatmo.net/api/getpublicmeasure")
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "device_id": device_id,
        }))
        .send()
        .await else {
            return Err(String::from("Request Failed"));
        };

    let Ok(data) = repsonse.text().await else {
        return Err(String::from("Body fetching failed"));
    };

    let Ok(value) = serde_json::from_str::<Value>(data.as_str()) else {
        return Err(String::from("Deserialize failed"));
    };

    let Some(measures) = &value["body"][0]["measures"].as_object() else {
        return Err(String::from("Measures failed"));
    };

    let mut result_map = HashMap::new();

    // What even is this hairball...
    for key in measures.keys() {
        let res = &measures[key]["res"].as_object().expect("");
        let data_types = &measures[key]["type"].as_array().expect("data types");
        for res_key in res.keys() {
            let data = res[res_key].as_array().expect("res array");
            for (i, v) in data.iter().enumerate() {
                result_map.insert(
                    data_types[i].as_str().expect("String").to_string(),
                    v.as_f64().expect("f64"),
                );
            }
        }
    }

    return Ok(result_map);
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

    match request_data(&client, token, device_id).await {
        Ok(output) => println!("{}", serde_json::to_string_pretty(&output).expect("Fuck")),
        Err(output) => println!("Error: {}", output),
    }
}
