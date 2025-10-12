#![allow(non_snake_case)]

use std::{path::PathBuf, sync::Arc};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use axum::{
    extract::{Json, State}, http::StatusCode, response::{IntoResponse, Response}
};

use crate::types::GameData;

#[derive(Deserialize, Serialize)]
pub struct CharPayload {
    charId: Option<String>
}

pub async fn characters(gamedata: State<Arc<GameData>>, Json(payload): Json<CharPayload>) -> Result<Json<Value>, (StatusCode, String)> {
    println!("We been pinged");
    match payload.charId.as_deref() {
        None | Some("") => {
            return Ok(Json(json!(**gamedata)));
        }
        Some(charId) => {
            match gamedata.units.iter().find(|u| u.baseId == charId) {
                Some(unit) => Ok(Json(json!(unit))),
                None => Err((StatusCode::NOT_FOUND, format!("Character '{}' not found", charId)))
            }
        }
    }
}

//curl -X POST localhost:7474/characters -H "Content-Type: application/json" -d '{"charId":"beans"}'