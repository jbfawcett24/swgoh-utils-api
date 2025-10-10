use std::{
    io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}
};
use reqwest::{self, Client};
use serde::{Serialize};
use serde_json::{self, json};

mod types;
use types::{GameMetadata, GameData};

const COMLINK:&str = "http://localhost:3000";
const ASSET_EXTRACTOR:&str = "http://localhost:3001";

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7474").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream).await;
    }
}

async fn handle_connection(stream: TcpStream){
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    let status_line = "HTTP/1.1 200 OK";
    println!("{request_line}");

    if request_line == "GET / HTTP/1.1" {
        let content = match check_comlink().await {
            Ok(body) => body.to_string(), 
            Err(_) => "Error".to_string()
        };
        println!("{}", content);

        return_data(status_line, &content, "application/json", stream);
    } else if request_line == "GET /metadata HTTP/1.1" {
        let content = get_game_data().await.unwrap();

        return_data(status_line, &content, "application/json", stream);
    }
    else {
        let status_line = "HTTP/1.1 404 NOT FOUND";
        let content = "404 Not Found :(";

        return_data(status_line, &content, "text/html", stream);
    }
}


fn return_data<T: Serialize>(status: &str, content: &T, content_type: &str, mut stream: TcpStream,) {
    let content = serde_json::to_string(content).unwrap();
    let response = format!("{status}\r\nContent-Type: {content_type}\r\n\r\n{content}");
    println!("{}", response);
    stream.write_all(response.as_bytes()).unwrap();
}

async fn check_comlink() -> Result<String, reqwest::Error> {
    let url = format!("{COMLINK}/readyz");
    let body = reqwest::get(url)
        .await?
        .text()
        .await?;

    println!("body = {body:?}");

    Ok(body)
}


async fn get_game_data() -> Result<GameData, reqwest::Error> {
    let client = Client::new();
    let meta_url = format!("{COMLINK}/metadata");
    println!("Getting game metadata...");
    let metadata = client
        .post(&meta_url)
        .send()
        .await?
        .json::<GameMetadata>()
        .await?;

    println!("Getting game data... (Version: {})", metadata.latestGamedataVersion);
    let data_url = format!("{COMLINK}/data");
    let request_body = json!({
        "payload": {
            "version": metadata.latestGamedataVersion.to_string(),
            "includePveUnits": false,
            "requestSegment": 3,
        },
        "enums": false
    });
    let gamedata = client.post(data_url).json(&request_body).send().await?
        .json::<GameData>()
        .await?;


    Ok(gamedata)
}

// curl -X POST "https://localhost:3000/data" \
//      -H "Content-Type: application/json" \
//      -d '{
//            "payload": {
//              "version": "0.37.0:siY-7g7ETs6TYA1Vqs19iA",
//              "includePveUnits": false,
//              "devicePlatform": "Android",
//              "requestSegment": 0,
//              "items": "string"
//            },
//            "enums": false
//          }'
