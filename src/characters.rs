#![allow(non_snake_case)]

use serde::{Serialize, Deserialize};
use axum::{
    extract::Json, response::IntoResponse, http::StatusCode
};

#[derive(Deserialize, Serialize)]
pub struct CharPayload {
    charId: String
}

pub async fn characters(Json(payload): Json<CharPayload>) -> Result<impl IntoResponse, impl IntoResponse> {
    println!("We been pinged");
    println!("charId: {}", payload.charId);
    if payload.charId.trim().is_empty() {
        let body = Json(serde_json::json!({
            "error": "charId cannot be empty"
        }));
        return Err((StatusCode::BAD_REQUEST, body));
    }

    Ok(Json(payload))
}