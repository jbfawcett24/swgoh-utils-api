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
    let pool = SqlitePool::connect("sqlite:////data/mydb.sqlite").await.unwrap();
    match payload.charId.as_deref() {
        None | Some("") => {
            return Ok(Json(getAllChar(&pool).await))
            //Err((StatusCode::BAD_REQUEST, "charId required for now".to_string()))
        }
        Some(charId) => {
            let char_data = getSingleChar(charId.to_string()).await;
            let mut returnList:Vec<Character> = Vec::new();
            returnList.push(char_data);
            Ok(Json(returnList))
        }
    }
}

/// Retrieves all characters from the database with their associated data
/// Returns a Vec of Character structs, each containing unit info, categories, 
/// crew members, skills, and equipment tiers

async fn getAllChar(pool: &SqlitePool) -> Vec<Character> {
    println!("getting all characters");
    // Step 1: Fetch ALL base unit information in one query
    let all_units: Vec<UnitRow> = sqlx::query_as("SELECT * FROM unit")
        .fetch_all(pool)
        .await
        .unwrap();

    // Step 2: Fetch ALL related data in bulk queries (not per-unit)
    // This dramatically reduces the number of database round-trips
    
    // Get all category associations at once
    let all_categories: Vec<CategoryRow> = sqlx::query_as(
        "SELECT * FROM unit_has_trait"
    )
        .fetch_all(pool)
        .await
        .unwrap();

    // Get all skills at once
    let all_skills: Vec<SkillRow> = sqlx::query_as(
        "SELECT * FROM skill"
    )
        .fetch_all(pool)
        .await
        .unwrap();

    // Get all crew members at once
    let all_crew: Vec<CrewRow> = sqlx::query_as(
        "SELECT * FROM crew"
    )
        .fetch_all(pool)
        .await
        .unwrap();

    // Get all tier information at once
    let all_tiers: Vec<TierRow> = sqlx::query_as(
        "SELECT * FROM unitTier ORDER BY baseId, tier"
    )
        .fetch_all(pool)
        .await
        .unwrap();

    // Get all equipment at once
    let all_equipment: Vec<EquipmentRow> = sqlx::query_as(
        "SELECT * FROM equipment"
    )
        .fetch_all(pool)
        .await
        .unwrap();

    // Step 3: Build lookup maps for O(1) access instead of linear searches
    // This organizes data by baseId for fast retrieval
    use std::collections::HashMap;

    // Map: baseId -> Vec<category_name>
    let mut categories_map: HashMap<String, Vec<String>> = HashMap::new();
    for cat in all_categories {
        categories_map
            .entry(cat.baseId.clone())
            .or_insert_with(Vec::new)
            .push(cat.category_name);
    }

    // Map: baseId -> Vec<skillId>
    let mut skills_map: HashMap<String, Vec<String>> = HashMap::new();
    for skill in all_skills {
        skills_map
            .entry(skill.baseId.clone())
            .or_insert_with(Vec::new)
            .push(skill.skillId);
    }

    // Map: baseId -> Vec<unitId>
    let mut crew_map: HashMap<String, Vec<String>> = HashMap::new();
    for crew in all_crew {
        crew_map
            .entry(crew.baseId.clone())
            .or_insert_with(Vec::new)
            .push(crew.unitId);
    }

    // Map: baseId -> Vec<TierRow>
    let mut tiers_map: HashMap<String, Vec<TierRow>> = HashMap::new();
    for tier in all_tiers {
        tiers_map
            .entry(tier.baseId.clone())
            .or_insert_with(Vec::new)
            .push(tier);
    }

    // Map: (baseId, tier) -> Vec<equipmentId>
    let mut equipment_map: HashMap<(String, i64), Vec<String>> = HashMap::new();
    for equip in all_equipment {
        equipment_map
            .entry((equip.baseId.clone(), equip.tier))
            .or_insert_with(Vec::new)
            .push(equip.equipmentId);
    }

    // Step 4: Build Character structs using the lookup maps
    // This is now O(n) instead of O(n²) or worse
    let mut characters: Vec<Character> = Vec::new();

    for unit_info in all_units {
        // Look up categories for this unit (O(1) lookup)
        let category_names = categories_map
            .get(&unit_info.baseId)
            .cloned()
            .unwrap_or_default();

        // Look up crew for this unit (O(1) lookup)
        let crew_ids = crew_map
            .get(&unit_info.baseId)
            .cloned()
            .unwrap_or_default();

        // Look up skills for this unit (O(1) lookup)
        let skill_ids = skills_map
            .get(&unit_info.baseId)
            .cloned()
            .unwrap_or_default();

        // Build tiers with equipment for this unit
        let mut tiers: Vec<Tier> = Vec::new();
        
        if let Some(tier_rows) = tiers_map.get(&unit_info.baseId) {
            for tier_row in tier_rows {
                // Look up equipment for this specific tier (O(1) lookup)
                let equipment_ids = equipment_map
                    .get(&(unit_info.baseId.clone(), tier_row.tier as i64))
                    .cloned()
                    .unwrap_or_default();

                tiers.push(Tier {
                    tier: tier_row.tier as u32,
                    equipmentSet: equipment_ids,
                });
            }
        }

        // Construct the complete Character object
        let character = Character {
            baseId: unit_info.baseId,
            categoryId: category_names,
            crew: crew_ids,
            iconPath: unit_info.iconPath,
            relicDefinition: unit_info.relicDefinition.unwrap_or_default(),
            skillReference: skill_ids,
            thumbnainName: unit_info.thumbnailName,
            unitTier: tiers,
        };

        characters.push(character);
    }

    // Return all constructed characters
    characters
}

// async fn getAllChar(pool: &SqlitePool) -> Vec<Character> {
//     // Fetch all base unit information from the unit table
//     let all_units: Vec<UnitRow> = sqlx::query_as("SELECT * FROM unit")
//         .fetch_all(pool)
//         .await
//         .unwrap();

//     // Initialize vector to store all constructed Character objects
//     let mut characters: Vec<Character> = Vec::new();

//     // Iterate through each unit and build its complete Character struct
//     for unit_info in all_units {
//         // Get all category/trait associations for this unit
//         let categories: Vec<CategoryRow> = sqlx::query_as(
//             "SELECT * FROM unit_has_trait WHERE baseId = ?"
//         )
//             .bind(&unit_info.baseId)
//             .fetch_all(pool)
//             .await
//             .unwrap();

//         // Get all skills associated with this unit
//         let skills: Vec<SkillRow> = sqlx::query_as(
//             "SELECT * FROM skill WHERE baseId = ?"
//         )
//             .bind(&unit_info.baseId)
//             .fetch_all(pool)
//             .await
//             .unwrap();

//         // Get all crew members for this unit
//         let crew: Vec<CrewRow> = sqlx::query_as(
//             "SELECT * FROM crew WHERE baseId = ?"
//         )
//             .bind(&unit_info.baseId)
//             .fetch_all(pool)
//             .await
//             .unwrap();

//         // Get all tier information for this unit
//         let unit_tiers: Vec<TierRow> = sqlx::query_as(
//             "SELECT * FROM unitTier WHERE baseId = ?"
//         )
//             .bind(&unit_info.baseId)
//             .fetch_all(pool)
//             .await
//             .unwrap();

//         // Extract just the category names from the CategoryRow structs
//         let category_names: Vec<String> = categories
//             .into_iter()
//             .map(|c| c.category_name)
//             .collect();

//         // Extract just the crew unit IDs from the CrewRow structs
//         let crew_ids: Vec<String> = crew
//             .into_iter()
//             .map(|c| c.unitId)
//             .collect();

//         // Extract just the skill IDs from the SkillRow structs
//         let skill_ids: Vec<String> = skills
//             .into_iter()
//             .map(|s| s.skillId)
//             .collect();

//         // Build the tiers with their associated equipment
//         let mut tiers: Vec<Tier> = Vec::new();

//         for tier_row in unit_tiers {
//             // For each tier, fetch all equipment items
//             let equipment_rows_result: Result<Vec<EquipmentRow>, sqlx::Error> = 
//                 sqlx::query_as(
//                     "SELECT * FROM equipment WHERE tier = ? AND baseId = ?"
//                 )
//                 .bind(tier_row.tier)
//                 .bind(&tier_row.baseId)
//                 .fetch_all(pool)
//                 .await;

//             // Handle potential errors when fetching equipment
//             let equipment_rows = match equipment_rows_result {
//                 Ok(rows) => rows,
//                 Err(e) => {
//                     eprintln!(
//                         "Failed to fetch equipment for unit {} tier {}: {}",
//                         &tier_row.baseId, tier_row.tier, e
//                     );
//                     // Skip this tier if equipment fetch fails
//                     continue;
//                 }
//             };

//             // Extract equipment IDs from the equipment rows
//             let equipment_ids: Vec<String> = equipment_rows
//                 .into_iter()
//                 .map(|e| e.equipmentId)
//                 .collect();

//             // Create Tier struct with tier number and equipment list
//             tiers.push(Tier {
//                 tier: tier_row.tier as u32,
//                 equipmentSet: equipment_ids,
//             });
//         }

//         // Construct the complete Character object with all gathered data
//         let character = Character {
//             baseId: unit_info.baseId,
//             categoryId: category_names,
//             crew: crew_ids,
//             iconPath: unit_info.iconPath,
//             // Use empty string if relicDefinition is NULL
//             relicDefinition: unit_info.relicDefinition.unwrap_or_default(),
//             skillReference: skill_ids,
//             thumbnainName: unit_info.thumbnailName,
//             unitTier: tiers,
//         };

//         // Add completed character to the results vector
//         characters.push(character);
//     }

//     // Return all constructed characters
//     characters
// }

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