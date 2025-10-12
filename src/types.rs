#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GameMetadata {
    pub assetVersion: u32,
    pub latestGamedataVersion: String
}
 
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameData {
    pub units: Vec<Unit>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]pub struct Unit {
    pub baseId: String,
    pub categoryId: Vec<String>,
    pub relicDefinition: Option<RelicDefinition>,
    pub skillReference: Vec<Skill>,
    pub thumbnailName: String,
    pub unitTier: Vec<Tier>,
    pub crew: Vec<Crew>,
    pub iconPath: Option<String>
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RelicDefinition {
    pub texture: String
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Skill {
    pub skillId: String
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tier {
    pub tier: u32,
    pub equipmentSet: Vec<String>
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Crew {
    pub unitId: String
}