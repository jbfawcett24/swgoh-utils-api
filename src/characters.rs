#![allow(non_snake_case)]

use std::{sync::Arc};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use axum::{
    extract::{Json, State}, http::StatusCode
};

use sqlx::{SqlitePool, Row};

use crate::types::GameData;

#[derive(Deserialize, Serialize)]
pub struct CharPayload {
    charId: Option<String>
}

// pub async fn characters(gamedata: State<Arc<GameData>>, Json(payload): Json<CharPayload>) -> Result<Json<Value>, (StatusCode, String)> {
//     println!("We been pinged");
    
//     match payload.charId.as_deref() {
//         None | Some("") => {
//             return Ok(Json(json!(**gamedata)));
//         }
//         Some(charId) => {
//             match gamedata.units.iter().find(|u| u.baseId == charId) {
//                 Some(unit) => Ok(Json(json!(unit))),
//                 None => Err((StatusCode::NOT_FOUND, format!("Character '{}' not found", charId)))
//             }
//         }
//     }
// }

pub async fn characters(Json(payload): Json<CharPayload>) -> Result<Json<Vec<Character>>, (StatusCode, String)> {
    match payload.charId.as_deref() {
        None | Some("") => {
            //return Ok(Json(getAllChar()))
            Err((StatusCode::BAD_REQUEST, "charId required for now".to_string()))
        }
        Some(charId) => {
            let char_data = getSingleChar(charId.to_string()).await;
            let mut returnList:Vec<Character> = Vec::new();
            returnList.push(char_data);
            Ok(Json(returnList))
        }
    }
}

async fn getSingleChar(baseId: String) -> Character {
    let pool = SqlitePool::connect("sqlite:////data/mydb.sqlite").await.unwrap();
    let unitInfo:UnitRow = sqlx::query_as("SELECT * FROM unit WHERE baseId = ?")
        .bind(&baseId).fetch_one(&pool).await.unwrap();

    let categories:Vec<CategoryRow> = sqlx::query_as(
        "SELECT * FROM unit_has_trait WHERE baseId = ?"
    )
        .bind(&baseId).fetch_all(&pool).await.unwrap();

    let skills:Vec<SkillRow> = sqlx::query_as("SELECT * FROM skill WHERE baseId = ?")
        .bind(&baseId).fetch_all(&pool).await.unwrap();

    let crew:Vec<CrewRow> = sqlx::query_as("Select * FROM crew WHERE baseId = ?")
        .bind(&baseId).fetch_all(&pool).await.unwrap();

    let unitTier:Vec<TierRow> = sqlx::query_as("SELECT * FROM unitTier WHERE baseId = ?")
        .bind(&baseId).fetch_all(&pool).await.unwrap();

    let category_names:Vec<String>  = categories.into_iter().map(|c| c.category_name).collect();
    let crew_ids:Vec<String> = crew.into_iter().map(|c| c.unitId).collect();
    let skill_ids:Vec<String> = skills.into_iter().map(|s| s.skillId).collect();

    let mut tiers: Vec<Tier> = Vec::new();

for tier in unitTier {
    let equipment_rows_result: Result<Vec<EquipmentRow>, sqlx::Error> = sqlx::query_as(
        "SELECT * FROM equipment WHERE tier = ? AND baseId = ?"
    )
    .bind(tier.tier)
    .bind(&tier.baseId)
    .fetch_all(&pool)
    .await;

    let equipment_rows = match equipment_rows_result {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!(
                "Failed to fetch equipment for unit {} tier {}: {}",
                &tier.baseId, tier.tier, e
            );
            continue; // skip this tier if it fails
        }
    };

    let equipment_ids: Vec<String> = equipment_rows.into_iter()
        .map(|e| e.equipmentId)
        .collect();

    tiers.push(Tier {
        tier: tier.tier as u32,
        equipmentSet: equipment_ids,
    });
}



    let character = Character {
        baseId: unitInfo.baseId,
        categoryId: category_names,
        crew: crew_ids,
        iconPath: unitInfo.iconPath,
        relicDefinition: unitInfo.relicDefinition.unwrap_or_default(),
        skillReference: skill_ids,
        thumbnainName: unitInfo.thumbnailName,
        unitTier: tiers
    };

    return character
}

// async fn getAllChar() {
//     return "HI"
// }

pub async fn setCharactersToDB(gamedata: &Arc<GameData>) {
    let pool = SqlitePool::connect("sqlite:////data/mydb.sqlite").await.unwrap();

    let existing_ids: Vec<(String,)> = sqlx::query_as("SELECT baseId FROM unit")
        .fetch_all(&pool)
        .await
        .unwrap();
    let existing_set: std::collections::HashSet<String> = 
        existing_ids.into_iter().map(|(id,)| id).collect();
    
    // Filter to only new units
    let new_units: Vec<_> = gamedata.units.iter()
        .filter(|u| !existing_set.contains(&u.baseId))
        .collect();
    
    if new_units.is_empty() {
        println!("No new characters to add");
        return;
    }


    for unit in new_units {
        println!("setting unit {}", &unit.baseId);
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO unit (
            baseId, iconPath, thumbnailName, relicDefinition
            ) VALUES (?, ?, ?, ?);
            "#    
        ).bind(&unit.baseId)
        .bind(&unit.iconPath)
        .bind(&unit.thumbnailName)
        .bind(&unit.relicDefinition.as_ref().map(|r| &r.texture))
        .execute(&pool)
        .await
        .unwrap();
        
        for category in &unit.categoryId {
            sqlx::query(
                r#"
                INSERT OR IGNORE INTO category (category_name)
                VALUES (?);
                "#
            ).bind(&category)
            .execute(&pool)
            .await
            .unwrap();

            sqlx::query(
                r#"
                INSERT OR IGNORE INTO unit_has_trait (baseId, category_name)
                VALUES (?, ?);
                "#
            ).bind(&unit.baseId)
            .bind(&category)
            .execute(&pool)
            .await
            .unwrap();
        }

        for crew in &unit.crew {
            sqlx::query(
                r#"
                INSERT OR IGNORE INTO crew (unitId, baseId)
                VALUES (?, ?);
                "#
            ).bind(&crew.unitId)
            .bind(&unit.baseId)
            .execute(&pool)
            .await
            .unwrap();
        }

        for skill in &unit.skillReference {
            sqlx::query(
                r#"
                INSERT OR IGNORE INTO skill (skillId, baseId)
                VALUES (?, ?);
                "#
            ).bind(&skill.skillId)
            .bind(&unit.baseId)
            .execute(&pool)
            .await
            .unwrap();
        }

        for unitTier in &unit.unitTier {
            sqlx::query(
                r#"
                INSERT OR IGNORE INTO unitTier (baseId, tier)
                VALUES (?, ?)
                "#
            )
            .bind(&unit.baseId)
            .bind(&unitTier.tier)
            .execute(&pool)
            .await
            .unwrap();

            for equipment in &unitTier.equipmentSet {
                sqlx::query(
                r#"
                    INSERT OR IGNORE INTO equipment (equipmentId, tier, baseId)
                    VALUES (?, ?, ?);
                    "#
                )
                .bind(&equipment)
                .bind(&unitTier.tier)
                .bind(&unit.baseId)
                .execute(&pool)
                .await
                .unwrap();
            }
        }

    }
    println!("all units added");
}

#[derive(Deserialize, Serialize)]
pub struct Character {
    pub baseId: String,
    pub categoryId: Vec<String>,
    pub crew: Vec<String>,
    pub iconPath: String,
    pub relicDefinition: String,
    pub skillReference: Vec<String>,
    pub thumbnainName: String,
    pub unitTier:  Vec<Tier>    
}

#[derive(Deserialize, Serialize)]
pub struct Tier {
    pub tier: u32,
    pub equipmentSet: Vec<String>
}

//Table Row Definitions
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
struct UnitRow {
    baseId: String,
    iconPath: String,
    thumbnailName: String,
    relicDefinition: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
struct CategoryRow {
    baseId: String,
    category_name: String,
}

#[derive(Debug, Serialize, FromRow)]
struct CrewRow {
    baseId: String,
    unitId: String,
}

#[derive(Debug, Serialize, FromRow)]
struct SkillRow {
    skillId: String,
    baseId: String,
}

#[derive(Debug, Serialize, FromRow)]
struct TierRow {
    baseId: String,
    tier: i32,
}

#[derive(Debug, Serialize, FromRow)]
struct EquipmentRow {
    equipmentId: String,
    tier: i64,
    baseId: String,
}


//curl -X POST localhost:7474/characters -H "Content-Type: application/json" -d '{"charId":"BADBATCHECHO"}'