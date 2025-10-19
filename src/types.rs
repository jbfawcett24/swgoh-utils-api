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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Unit {
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


//Player class 
#[derive(Deserialize, Serialize)]
pub struct Player {
    pub rosterUnit: Vec<RosterUnit>,
    pub name: String,
    pub level: u32,
    pub allyCode: String,
    pub playerId: String,
    pub guildId: String,
    pub guildName: String,
    pub guildLogoBackground: String,
    pub guildBannerColor: String,
    pub guildBannerLogo: String,
    pub selectedPlayerTitle: SelectedPlayerThing,
    pub selectedPlayerPortrait: SelectedPlayerThing,
    pub playerRating: PlayerRating
}
#[derive(Deserialize, Serialize)]
pub struct SelectedPlayerThing {
    pub id: String
}
#[derive(Deserialize, Serialize)]
pub struct PlayerRating {
    pub playerSkillRating: PlayerSkillRating,
    pub playerRankStatus: PlayerRankStatus
}
#[derive(Deserialize, Serialize)]
pub struct PlayerSkillRating {
    skillRating: u32
}
#[derive(Deserialize, Serialize)]
pub struct PlayerRankStatus {
    pub leagueId: String,
    pub divisionId: u32
}
#[derive(Deserialize, Serialize)]
pub struct RosterUnit {
    pub definitionId: String,
    pub currentRarity: u32,
    pub currentLevel: u32,
    pub currentTier: u32,
    pub relic: Option<Relic>
}
#[derive(Deserialize, Serialize)]
pub struct Relic {
    pub currentTier: u32
}