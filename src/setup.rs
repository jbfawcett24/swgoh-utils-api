use sqlx::{SqlitePool};

pub async fn dbSetup() {
    let pool = SqlitePool::connect("sqlite:////data/mydb.sqlite").await.unwrap();

    sqlx::query(
        r#"
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;

        CREATE TABLE IF NOT EXISTS unit (
            baseId TEXT PRIMARY KEY,
            iconPath TEXT NOT NULL,
            thumbnailName TEXT NOT NULL,
            relicDefinition TEXT
        );

        CREATE TABLE IF NOT EXISTS category (
            category_name TEXT PRIMARY KEY NOT NULL
        );

        CREATE TABLE IF NOT EXISTS unit_has_trait (
            baseId TEXT NOT NULL,
            category_name TEXT NOT NULL,
            FOREIGN KEY (baseId) REFERENCES unit(baseId),
            FOREIGN KEY (category_name) REFERENCES category(category_name),
            PRIMARY KEY (baseId, category_name)
        );

        CREATE TABLE IF NOT EXISTS crew (
            unitId TEXT NOT NULL PRIMARY KEY,
            baseId TEXT NOT NULL,
            FOREIGN KEY (baseId) REFERENCES unit(baseId)
        );

        CREATE TABLE IF NOT EXISTS skill (
            skillId TEXT PRIMARY KEY,
            baseId TEXT NOT NULL,
            FOREIGN KEY (baseId) REFERENCES unit(baseId)
        );

        CREATE TABLE IF NOT EXISTS unitTier (
            baseId TEXT NOT NULL,
            tier INTEGER NOT NULL,
            FOREIGN KEY (baseId) REFERENCES unit(baseId),
            PRIMARY KEY (tier, baseId)
        );

        CREATE TABLE IF NOT EXISTS equipment (
            equipmentId TEXT NOT NULL,
            tier INTEGER NOT NULL,
            baseId TEXT NOT NULL,
            PRIMARY KEY (equipmentId, tier, baseId),
            FOREIGN KEY (tier, baseId) REFERENCES unitTier(tier, baseId)
        );
        "#
    ).execute(&pool).await.unwrap();

    println!("databases made");
}