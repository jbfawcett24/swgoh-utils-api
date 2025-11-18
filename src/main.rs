#![allow(non_snake_case)]
use std::{io::Write, path::{Path, PathBuf}, vec};

use reqwest::{self, Client, header};
use serde_json::{self, json};
use axum::{
    Router, extract::{FromRequestParts, Json}, http::StatusCode, response::IntoResponse, routing::{get, get_service, post}
};
use sqlx::{SqlitePool, Row, Error as SqlxError};
use chrono::{Utc, Duration};

use tokio::{fs::{self, File}, io::AsyncWriteExt};
use tower_http::services::ServeDir;

use argon2::{
    Argon2, password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng
    }
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, encode, decode};

use tower_http::cors::{CorsLayer};
use axum::http::request::Parts;

mod types;
use types::{GameMetadata, GameData, Player};

mod characters;
use characters::{characters, setCharactersToDB};
mod roster;
use roster::{setRosterDatabase};
mod setup;
use setup::{dbSetup};

const COMLINK:&str = "http://comlink:3000";
const ASSET_EXTRACTOR:&str = "http://asset_extractor:8080";

#[tokio::main]
//endpoints - 
//characters - all character names, skills, image, id - charId just sends one
//account - account information, all character gear, star, relic
//guild - accounts guild, with number of each character unlocked, each user fleet, squad, and GAC rank - charId returns dictionary of all characters at every level
//journey - journey guide information

async fn main() {

        std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("PANIC: {:?}", panic_info);
        if let Some(location) = panic_info.location() {
            eprintln!(
                "Panic occurred in file '{}' at line {}",
                location.file(),
                location.line()
            );
        }
    }));

    println!("Starting up...");
    std::io::stdout().flush().unwrap();

    println!("creating database");
    fs::create_dir_all("/data").await.unwrap();

    dbSetup().await;

    let gamedata = get_game_data().await.unwrap();
    setCharactersToDB(&gamedata).await;

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(24*60*60));

        loop {
            interval.tick().await;

            println!("checking for game data updates");

            match get_game_data().await {
                Ok(new_game_data) => {
                    setCharactersToDB(&new_game_data).await;
                    println!("game data updated");
                }
                Err(e) => {
                    eprintln!("Failed to update game data : {}", e)
                }
            }
        }
    });

    let image_dir = PathBuf::from("./assets");

    // let cors = CorsLayer::new()
    // .allow_origin(tower_http::cors::Any)
    // .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
    // .allow_headers(Any);

    let cors = CorsLayer::permissive();
    
    let app = Router::new()
        .route("/", get(root))
        .route("/characters", post(characters))
        .route("/account", get(account))
        .route("/guild", get(guild))
        .route("/refreshAccount", get(refresh_account_handler))
        .route("/signUp", post(signUp))
        .route("/signIn", post(signIn))
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
        .layer(cors);

    let listener  = tokio::net::TcpListener::bind("0.0.0.0:7474").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str{
    "Hello World"
}
async fn guild() -> impl IntoResponse {
    "hi"
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

use crate::roster::get_player_from_db;
#[derive(Deserialize, Serialize)]
pub struct PlayerPayload {
    pub allyCode: Option<String>
}

async fn account( AuthBearer(claims): AuthBearer) -> Result<Json<Player>, StatusCode>{
    let pool = SqlitePool::connect("sqlite:////data/mydb.sqlite").await.unwrap();
    // let ally_code = match payload.allyCode.as_deref() {
    //     Some(code) => code,
    //     None => return Err(StatusCode::BAD_REQUEST),
    // };
    let ally_code = &claims.sub;

    // Try loading from DB first
    if let Ok(player) = get_player_from_db(ally_code, &pool).await {
        println!("from database");
        println!("{}", &player.name);
        return Ok(Json(player));
    }

    return refreshAccount(ally_code.to_string()).await;
}

async fn refreshAccount(ally_code: String) -> Result<Json<Player>, StatusCode> {


    let pool = SqlitePool::connect("sqlite:////data/mydb.sqlite").await.unwrap();
    println!("player time {}", ally_code);
    let client = Client::new();
    let data_url = format!("{COMLINK}/player");
    //println!("{:?}", payload.allyCode);
    let request_body = json!({
        "payload": {
            "allyCode": ally_code
        },
        "enums": false
    });

    let response = client
        .post(data_url)
        .json(&request_body)
        .send()
        .await
        .map_err(|_| StatusCode::NOT_ACCEPTABLE)?;

    println!("response recieved");
    let player = response
        .json::<Player>()
        .await
        .map_err(|e| {println!("error {}", e); return StatusCode::NOT_IMPLEMENTED;})?;

    sqlx::query(r#"DELETE FROM rosterUnit WHERE allycode = ?"#).bind(ally_code).execute(&pool).await.unwrap();

    println!("adding to database {}", player.name);
    setRosterDatabase(&player, &pool).await.unwrap();

    Ok(Json(player))
}

//#[derive(Deserialize)]
// struct RefreshPayload {
//     allyCode: String,
// }

async fn refresh_account_handler(
    AuthBearer(claims): AuthBearer
) -> Result<Json<Player>, StatusCode> {
    refreshAccount(claims.sub).await
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

#[derive(Deserialize)]
struct SignInPayload {
    username: String,
    password: String
}

#[derive(Deserialize, Serialize)]
pub struct Claims {
    sub: String,
    exp: usize
}

async fn signIn(Json(payload): Json<SignInPayload>) -> Result<Json<serde_json::Value>, StatusCode>{
    println!("we signing in");
    let pool = SqlitePool::connect("sqlite:////data/mydb.sqlite").await.unwrap();
    let account_info = sqlx::query(
        r#"
            SELECT * FROM user WHERE username = ?
        "#
    )
    .bind(&payload.username)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let password_hash: String = account_info.try_get("password").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let allyCode: String = account_info.try_get("allyCode").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let parsed_hash = PasswordHash::new(&password_hash)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Argon2::default()
    .verify_password(payload.password.as_bytes(), &parsed_hash)
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: allyCode.clone(),
        exp: expiration
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "mysecret".into());
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    println!("{}", token);

    Ok(Json(json!({ "token": token })))
}

#[derive(Deserialize)]
struct SignUpPayload {
    username: String,
    password: String,
    allyCode: String,
    email: String
}


async fn signUp(Json(payload): Json<SignUpPayload>) -> Result<StatusCode, StatusCode> {

    // Generate a random salt
    let salt = SaltString::generate(&mut OsRng);

    // Create an Argon2 instance
    let argon2 = Argon2::default();

    // Hash the password
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    println!("Password hash: {}", password_hash);
    let _ = refreshAccount(payload.allyCode.clone()).await.unwrap();
    println!("setting plater to db");

    // Connect to database
    let pool = SqlitePool::connect("sqlite:////data/mydb.sqlite")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Insert user into database
    let result = sqlx::query::<sqlx::Sqlite>(
        r#"
        INSERT INTO user (
            username, password, createdAt, allyCode, email
        ) VALUES (?, ?, ?, ?, ?)
        "#
    )
    .bind(&payload.username)
    .bind(&password_hash)
    .bind(Utc::now().to_rfc3339())
    .bind(&payload.allyCode)
    .bind(&payload.email)
    .execute(&pool) // <-- note the &pool
    .await;


    match result {
        Ok(_) => return Ok(StatusCode::OK),
        Err(SqlxError::Database(db_err)) if db_err.message().contains("UNIQUE constraint failed") => {
            return Err(StatusCode::CONFLICT) // 409 Conflict is appropriate here
        }
        Err(e) => {
            eprintln!("Other SQLx error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub struct AuthBearer(pub Claims);

impl<S> FromRequestParts<S> for AuthBearer
where
    S: Send + Sync,

{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Try to extract the Authorization header
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing Authorization header".to_string(),
            ))?;

        // Check for the correct "Bearer " prefix
        if !auth_header.starts_with("Bearer ") {
            return Err((StatusCode::UNAUTHORIZED, "Invalid token format".into()));
        }

        // Extract the token part (after "Bearer ")
        let token = auth_header.trim_start_matches("Bearer ").trim();

        // Decode and validate the JWT
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "mysecret".into());

        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "Invalid or expired token".to_string(),
            )
        })?;

        // If we reach here, the token is valid!
        Ok(AuthBearer(decoded.claims))
    }
}

//curl -i -X POST http://localhost:7474/account -H "Content-Type: application/json" -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI0ODI4NDEyMzUiLCJleHAiOjE3NjI1NTA3OTJ9.nHDNNWiEcj8bD2mGNcmF7T4rqnkVYCJNaG7RmiD9S3Q" -d '{"allyCode": "482841235"}'