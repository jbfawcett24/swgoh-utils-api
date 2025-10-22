use std::{io::Write, path::{Path, PathBuf}, sync::Arc, vec};

use reqwest::{self, Client};
use serde_json::{self, json};
use axum::{
    response::IntoResponse, routing::{get, get_service, post}, Router, extract::{State, Json}, http::StatusCode
};
use tokio::{fs::{self, File}, io::AsyncWriteExt};
use tower_http::services::ServeDir;

use tower_http::cors::{CorsLayer, Any};
use axum::http::{Method};

mod types;
use types::{GameMetadata, GameData, Player};

mod characters;
use characters::characters;

const COMLINK:&str = "http://comlink:3000";
const ASSET_EXTRACTOR:&str = "http://asset_extractor:8080";

#[tokio::main]
//endpoints - 
//characters - all character names, skills, image, id - charId just sends one
//account - account information, all character gear, star, relic
//guild - accounts guild, with number of each character unlocked, each user fleet, squad, and GAC rank - charId returns dictionary of all characters at every level
//journey - journey guide information

async fn main() {

    println!("Starting up...");
    std::io::stdout().flush().unwrap();

    let gamedata = get_game_data().await.unwrap();
    let gamedata = Arc::new(gamedata);

    let image_dir = PathBuf::from("./assets");

    let cors = CorsLayer::new()
    .allow_origin(tower_http::cors::Any)
    .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
    .allow_headers(Any);
    
    let app = Router::new()
        .route("/", get(root))
        .route("/characters", post(characters))
        .route("/account", post(account))
        .route("/guild", get(guild))
        .nest(
            "/assets",
            Router::new()
                .fallback_service(
                    get_service(ServeDir::new(image_dir))
                        .handle_error(|error| async move {
                            (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Unhandled internal error: {}", error),
                            )
                        }),
                ),
        )
        .with_state(gamedata)
        .layer(cors);

    let listener  = tokio::net::TcpListener::bind("0.0.0.0:7474").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str{
    "Hello World"
}
async fn guild(State(gamedata): State<Arc<GameData>>) -> impl IntoResponse {
    Json((*gamedata).clone())
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
    println!("Asset Version: {}", metadata.assetVersion);
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

    let gamedata = splice_game_data(gamedata);
    save(&gamedata).await;

    get_assets(&gamedata, &metadata.assetVersion).await;
    let gamedata = add_images_gamedata(gamedata);
    println!("Setup complete");
    Ok(gamedata)
}

async fn save(data: &GameData) {
    let mut save_file = File::create("data.json").await.unwrap();
    save_file.write_all(serde_json::to_string_pretty(data).unwrap().as_bytes()).await.unwrap();
}

use std::collections::HashSet;

fn splice_game_data(mut gamedata: GameData) -> GameData {
    let filter_keywords = ["_GLE", "SPEEDERBIKE", "MARQUEE", "EVENT"];
    let mut seen_base_ids:HashSet<String> = HashSet::new();

    gamedata.units.retain(|unit| {
        // Check if baseId contains any unwanted substrings
        let contains_blocked = filter_keywords.iter().any(|keyword| unit.baseId.contains(keyword));
        
        // If it's blocked, we remove it
        if contains_blocked {
            return false;
        }

        // If it's a duplicate, we remove it
        if !seen_base_ids.insert(unit.baseId.clone()) {
            return false;
        }

        true // Otherwise, keep it
    });

    gamedata
}


async fn get_assets(data:&GameData, asset_version:&u32) {
    println!("{}", asset_version);
    let mut asset_list:Vec<String> = vec![];

    if !Path::new("assets").exists() {
        fs::create_dir_all("assets").await.expect("Failed to create directory");
    }

    let asset_files: HashSet<String> = std::fs::read_dir("assets")
    .unwrap()
    .filter_map(|entry| {
        entry.ok().and_then(|e| {
            e.file_name()
                .into_string()
                .ok()
        })
    })
    .collect();

    for unit in &data.units {
        let check_name = format!("{}.png", unit.thumbnailName);
        if !asset_files.contains(&check_name) {
            let asset_name = unit.thumbnailName.trim_start_matches("tex.").to_string();
            asset_list.push(asset_name);
        }
    }

    let mut cur_num = 0;
    for asset in &asset_list {
        let number = &asset_list.len();
        println!("Downloading asset: {} ({}/{})", asset, cur_num, number);
        let url = format!("{}/Asset/single?assetName={}&version={}&forceReDownload=false", ASSET_EXTRACTOR, &asset, &asset_version);
        download_asset(url, &asset).await.unwrap();
        cur_num+=1;
    }
}

async fn download_asset(url: String, filename: &str) -> Result<(), Box<dyn std::error::Error>>{
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    let directory = "assets";
    let download_path = Path::new(directory);
    let full_filename = format!("tex.{}.png", filename);
    let full_path = download_path.join(full_filename);

    if !Path::new(directory).exists() {
        fs::create_dir_all(directory).await?;
    }

    let mut file = File::create(full_path).await?;
    let bytes = response.bytes().await?;
    
    file.write_all(&bytes).await?;

    Ok(())
}

fn add_images_gamedata(mut gamedata:GameData) -> GameData {
    for unit in &mut gamedata.units {
        let filepath = format!("assets/{}.png", unit.thumbnailName);
        unit.iconPath = Some(filepath);
    }

    gamedata
}
use serde::{Serialize, Deserialize};
#[derive(Deserialize, Serialize)]
pub struct PlayerPayload {
    pub allyCode: Option<String>
}

async fn account(Json(payload): Json<PlayerPayload>) -> Result<Json<Player>, StatusCode>{
    println!("player time");
    let client = Client::new();
    let data_url = format!("{COMLINK}/player");
    println!("{:?}", payload.allyCode);
    let request_body = json!({
        "payload": {
            "allyCode": payload.allyCode
        },
        "enums": false
    });

    let response = client
        .post(data_url)
        .json(&request_body)
        .send()
        .await
        .map_err(|_| StatusCode::NOT_ACCEPTABLE)?;

    let player = response
        .json::<Player>()
        .await
        .map_err(|_| StatusCode::NOT_IMPLEMENTED)?;

    Ok(Json(player))
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

//curl -X POST localhost:3000/player -H "Content-Type: application/json" -d '{"payload": {"allyCode": "482841235"}}' -o player.json
