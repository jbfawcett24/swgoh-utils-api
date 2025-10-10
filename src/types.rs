use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GameMetadata {
    pub assetVersion: u32,
    pub latestGamedataVersion: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GameData {
    units: Vec<Unit>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Unit {
    baseId: String,
    categoryId: Vec<String>,
    relicDefinition: Option<RelicDefinition>,
    skillReference: Vec<Skill>,
    thumbnailName: String,
    unitTier: Vec<Tier>,
    crew: Vec<Crew>
}
#[derive(Debug, Deserialize, Serialize)]
struct RelicDefinition {
    texture: String
}
#[derive(Debug, Deserialize, Serialize)]
struct Skill {
    skillId: String
}
#[derive(Debug, Deserialize, Serialize)]
struct Tier {
    tier: u32,
    equipmentSet: Vec<String>
}
#[derive(Debug, Deserialize, Serialize)]
struct Crew {
    unitId: String
}