use sqlx::{Sqlite, Pool};
use chrono::Utc;
use crate::types::{Player};

pub async fn setRosterDatabase(player: &Player, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT OR REPLACE INTO account (
            allyCode, name, level, playerId,
            guildId, guildName, guildLogoBackground, guildBannerColor, guildBannerLogo,
            selectedPlayerTitleId, selectedPlayerPortraitId, skillRating, leagueId, divisionId, last_updated
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&player.allyCode)
    .bind(&player.name)
    .bind(player.level as i64)
    .bind(&player.playerId)
    .bind(&player.guildId)
    .bind(&player.guildName)
    .bind(&player.guildLogoBackground)
    .bind(&player.guildBannerColor)
    .bind(&player.guildBannerLogo)
    .bind(&player.selectedPlayerTitle.id)
    .bind(&player.selectedPlayerPortrait.id)
    .bind(player.playerRating.playerSkillRating.skillRating as i64)
    .bind(&player.playerRating.playerRankStatus.leagueId)
    .bind(player.playerRating.playerRankStatus.divisionId as i64)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;


    for unit in &player.rosterUnit {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO rosterUnit (
            definitionId, currentRarity, currentLevel, currentTier, relicTier, allyCode
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&unit.definitionId)
        .bind(unit.currentRarity)
        .bind(unit.currentLevel)
        .bind(unit.currentTier)
        .bind(unit.relic.as_ref().map(|r| r.currentTier as i64))
        .bind(&player.allyCode)
        .execute(pool)
        .await?;
    }
    println!("set into database");
    println!("{}", Utc::now());

    Ok(())
}


use crate::types::*;

pub async fn get_player_from_db(ally_code: &str, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<Player, sqlx::Error> {
    // Fetch account
    let account = sqlx::query_as::<_, AccountRow>(
        r#"SELECT * FROM account WHERE allyCode = ?"#,
    )
    .bind(ally_code)
    .fetch_one(pool)
    .await?;

    // Fetch roster
    let roster = sqlx::query_as::<_, RosterUnitRow>(
        r#"SELECT * FROM rosterUnit WHERE allyCode = ?"#,
    )
    .bind(ally_code)
    .fetch_all(pool)
    .await?;

    // Convert roster rows into your game structs
    let roster_units: Vec<RosterUnit> = roster
        .into_iter()
        .map(|row| RosterUnit {
            definitionId: row.definitionId,
            currentRarity: row.currentRarity as u32,
            currentLevel: row.currentLevel as u32,
            currentTier: row.currentTier as u32,
            relic: row.relicTier.map(|t| Relic { currentTier: t as u32 }),
        })
        .collect();

    // Build Player
    let player = Player {
        rosterUnit: roster_units,
        name: account.name,
        level: account.level as u32,
        allyCode: account.allyCode,
        playerId: account.playerId,
        guildId: account.guildId,
        guildName: account.guildName,
        guildLogoBackground: account.guildLogoBackground,
        guildBannerColor: account.guildBannerColor,
        guildBannerLogo: account.guildBannerLogo,
        selectedPlayerTitle: SelectedPlayerThing {
            id: account.selectedPlayerTitleId,
        },
        selectedPlayerPortrait: SelectedPlayerThing {
            id: account.selectedPlayerPortraitId,
        },
        playerRating: PlayerRating {
            playerSkillRating: PlayerSkillRating {
                skillRating: account.skillRating as u32,
            },
            playerRankStatus: PlayerRankStatus {
                leagueId: account.leagueId,
                divisionId: account.divisionId as u32,
            },
        },
        last_updated: account.last_updated
    };

    println!("{}", player.name);

    Ok(player)
}


use sqlx::FromRow;

#[derive(FromRow)]
struct AccountRow {
    allyCode: String,
    name: String,
    level: i64,
    playerId: String,
    guildId: String,
    guildName: String,
    guildLogoBackground: String,
    guildBannerColor: String,
    guildBannerLogo: String,
    selectedPlayerTitleId: String,
    selectedPlayerPortraitId: String,
    skillRating: i64,
    leagueId: String,
    divisionId: i64,
    last_updated: String,
}

#[derive(FromRow)]
struct RosterUnitRow {
    definitionId: String,
    currentRarity: i64,
    currentLevel: i64,
    currentTier: i64,
    relicTier: Option<i64>,
    allyCode: String,
}
