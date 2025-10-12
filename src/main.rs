use std::{fmt::format, os::unix::fs::FileExt, path::Path, vec};

use reqwest::{self, Client, Error};
use serde::{Serialize, Deserialize};
use serde_json::{self, json};
use axum::{
    body::Bytes, extract::Json, http::StatusCode, response::IntoResponse, routing::{get, post}, Form, Router
};
use tokio::{fs::{self, File}, io::AsyncWriteExt};

mod types;
use types::{GameMetadata, GameData};

mod characters;
use characters::characters;

const COMLINK:&str = "http://localhost:3000";
const ASSET_EXTRACTOR:&str = "http://localhost:3001";

#[tokio::main]
//endpoints - 
//characters - all character names, skills, image, id - charId just sends one
//account - account information, all character gear, star, relic
//guild - accounts guild, with number of each character unlocked, each user fleet, squad, and GAC rank - charId returns dictionary of all characters at every level
async fn main() {
    get_game_data().await.unwrap();

    let app = Router::new()
        .route("/", get(root))
        .route("/characters", post(characters))
        .route("/account", post(account))
        .route("/guild", post(guild));

    let listener  = tokio::net::TcpListener::bind("0.0.0.0:7474").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str{
    println!("We Be Request");
    "Hello World"
}

async fn account() -> &'static str{
    "Hello account"
}

async fn guild() -> &'static str {
    "Hello guild"
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
    get_assets(&gamedata, &metadata.assetVersion).await;
    Ok(gamedata)
}

async fn get_assets(data:&GameData, asset_version:&u32) {
    println!("{}", asset_version);
    let mut asset_list:Vec<String> = vec![];
    for unit in &data.units {
        let asset_name = unit.thumbnailName.trim_start_matches("tex.").to_string();
        asset_list.push(asset_name);
    }

    for asset in asset_list {
        let url = format!("{}/Asset/single?assetName={}&version={}&forceReDownload=false", ASSET_EXTRACTOR, &asset, &asset_version);
        download_asset(url, &asset).await.unwrap();
    }
}


async fn download_asset(url: String, filename: &str) -> Result<(), Box<dyn std::error::Error>>{
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    let directory = "./src/assets";
    let download_path = Path::new(directory);
    let full_filename = format!("{}.png", filename);
    let full_path = download_path.join(full_filename);

    if !Path::new(directory).exists() {
        fs::create_dir_all(directory).await?;
    }

    let mut file = File::create(full_path).await?;
    let bytes = response.bytes().await?;
    
    file.write_all(&bytes).await?;

    Ok(())
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
