use axum::{
    extract::{Json, State}, http::StatusCode
};
use serde::{Serialize, Deserialize};
use sqlx::{SqlitePool, prelude::FromRow};
use crate::AuthBearer;

#[derive(Deserialize, Serialize)]
pub struct PlanPayload {
    name: String,
    icon: String,
    characters: Vec<CharPlanPayload>
}

#[derive(Deserialize, Serialize)]
pub struct CharPlanPayload {
    baseId: String,
    name: String,
    goalStars: i64,
    goalGear: i64,
    goalRelic: i64
}

pub async fn set_plan(    State(pool): State<SqlitePool>,
    AuthBearer(claims): AuthBearer,
    Json(payload): Json<PlanPayload>,) -> (StatusCode, String) {

    let allyCode = &claims.sub;

    //Set into database

    let rec = sqlx::query(r#"
    INSERT INTO plan
        (planName, icon, allyCode)
    VALUES (?, ?, ?);
    "#)
    .bind(&payload.name)
    .bind(&payload.icon)
    .bind(allyCode)
    .execute(&pool)
    .await;

    match rec {
    Ok(result) => {
        let id = result.last_insert_rowid();
        for character in payload.characters {
            let charRec = sqlx::query(r#"
            INSERT INTO charPlan
                (charName, goalStars, goalGear, goalRelic, baseId, planId)
                VALUES (?, ?, ?, ?, ?, ?)
            "#)
            .bind(&character.name)
            .bind(&character.goalStars)
            .bind(&character.goalGear)
            .bind(&character.goalRelic)
            .bind(&character.baseId)
            .bind(id)
            .execute(&pool)
            .await;
            
            match charRec {
                Ok(_) => {
                    println!("We inserted {} Ok", &character.baseId);
                }
                Err(code) => {
                    println!("Error: {}", code);
                }
            }
        }
        return (StatusCode::ACCEPTED, format!("Plan {} inserted successfully", payload.name));
    
    }
    Err(sqlx::Error::Database(db_err)) => {
        // Check for unique constraint violation
        if db_err.code().as_deref() == Some("2067") {
            return (StatusCode::CONFLICT, "Plan name already exists".to_string());
        }
        // Handle other database errors
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", db_err));
    }
    Err(e) => {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e));
    }
}
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Plan {
    pub name: String,
    pub icon: String,
    pub characters: Vec<CharPlan>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CharPlan {
    pub baseId: String,
    pub name: String,
    pub icon: String,
    pub goalGear: i32,
    pub goalRelic: i32,
    pub goalStars: i32,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct PlanRow {
    pub name: String,
    pub icon: String,
    pub id: i64
}
pub async fn get_plan(State(pool): State<SqlitePool>, AuthBearer(claims): AuthBearer) -> Result<Json<Vec<Plan>>, (StatusCode, String)> {

    let allyCode = claims.sub;

    let plans: Vec<PlanRow> = sqlx::query_as(r#"
        SELECT planName AS name, icon, id
        FROM plan
        WHERE allyCode = ?;
    "#)
    .bind(&allyCode)
    .fetch_all(&pool)
    .await
    .unwrap();

    let mut plansList: Vec<Plan> = Vec::new();

    for plan in plans {
        let chars: Vec<CharPlan> = sqlx::query_as(r#"
            SELECT charPlan.charName AS name, unit.iconPath AS icon, charPlan.goalStars, charPlan.goalGear, charPlan.goalRelic, charPlan.baseId
            FROM charPlan INNER JOIN plan ON charPlan.planId = plan.id
                INNER JOIN unit ON unit.baseId = charPlan.baseId
            WHERE plan.allyCode = ? AND plan.id = ?;
        "#)
        .bind(&allyCode)
        .bind(plan.id)
        .fetch_all(&pool)
        .await
        .unwrap();

        plansList.push(Plan {
            name: plan.name,
            icon: plan.icon,
            characters: chars
        });
    }

    return Ok(Json(plansList));
    //return Err((StatusCode::NOT_IMPLEMENTED, "Not done yet".to_string()));
}

// eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI0ODI4NDEyMzUiLCJleHAiOjE3NjM4MzIzNzh9.HWhGgf3-2JISldlivY9zbsARdiODLXHLhY_kjSyD6pQ

// curl -X POST http://localhost:7474/set_plan \
//   -H "Content-Type: application/json" \
//   -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI3Mzc2NjcxNDQiLCJleHAiOjE3NjQyMDU5NTd9.0wl4IXNVNHfcMR6vElsuTyU-fzv08m9RHIBm5otgLLY" \
//   -d '{
//     "name": "Jedi Master Kenobi", 
//     "icon": "assets/tex.charui_globiwan.png",
//     "characters": [
//         {
//             "baseId": "GENERALKENOBI",
//             "name": "General Kenobi",
//             "goalGear": 13,
//             "goalRelic": 8,
//             "goalStars": 7
//         },
//         {
//             "baseId": "CAPITALNEGOTIATOR",
//             "name": "Negotiator",
//             "goalGear": 0,
//             "goalRelic": 0,
//             "goalStars": 6
//         },
//                 {
//             "baseId": "MACEWINDU",
//             "name": "Mace Windu",
//             "goalGear": 13,
//             "goalRelic": 3,
//             "goalStars": 7
//         },
//                 {
//             "baseId": "AAYLASECURA",
//             "name": "Aayla Secura",
//             "goalGear": 13,
//             "goalRelic": 3,
//             "goalStars": 7
//         },
//                 {
//             "baseId": "BOKATAN",
//             "name": "Bo-Katan Kryze",
//             "goalGear": 13,
//             "goalRelic": 5,
//             "goalStars": 7
//         },
//                 {
//             "baseId": "QUIGONJINN",
//             "name": "Qui-Gon Jinn",
//             "goalGear": 13,
//             "goalRelic": 3,
//             "goalStars": 7
//         },
//                 {
//             "baseId": "MAGNAGUARD",
//             "name": "IG-100 MagnaGuard",
//             "goalGear": 13,
//             "goalRelic": 5,
//             "goalStars": 7
//         },
//                 {
//             "baseId": "CLONESERGEANTPHASEI",
//             "name": "Clone Sergeant - Phase I",
//             "goalGear": 13,
//             "goalRelic": 5,
//             "goalStars": 7
//         },
//                 {
//             "baseId": "WATTAMBOR",
//             "name": "Wat Tambor",
//             "goalGear": 13,
//             "goalRelic": 7,
//             "goalStars": 7
//         },
//                 {
//             "baseId": "GRIEVOUS",
//             "name": "General Grievous",
//             "goalGear": 13,
//             "goalRelic": 7,
//             "goalStars": 7
//         },
//                 {
//             "baseId": "",
//             "name": "Cad Bane",
//             "goalGear": 13,
//             "goalRelic": 5,
//             "goalStars": 7
//         },
//                 {
//             "baseId": "CC2224",
//             "name": "CC-2224 \"Cody\"",
//             "goalGear": 13,
//             "goalRelic": 5,
//             "goalStars": 7
//         },
//                 {
//             "baseId": "JANGOFETT",
//             "name": "Jango Fett",
//             "goalGear": 13,
//             "goalRelic": 7,
//             "goalStars": 7
//         },
//                 {
//             "baseId": "SHAAKTI",
//             "name": "Shaak Ti",
//             "goalGear": 13,
//             "goalRelic": 7,
//             "goalStars": 7
//         },
//                 {
//             "baseId": "GRANDMASTERYODA",
//             "name": "Grand Master Yoda",
//             "goalGear": 13,
//             "goalRelic": 8,
//             "goalStars": 7
//         }
//     ]
// }'

// curl http://localhost:7474/get_plan \
//  -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI3Mzc2NjcxNDQiLCJleHAiOjE3NjM4NDc2OTd9.MsCRD3uLIa4jUEkt77nTh2dvM-Inhd-GrqXs_Z-yWBk"