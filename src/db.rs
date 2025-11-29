use std::path::Path;

use sea_orm::{ConnectOptions, Database, DbBackend, Statement, prelude::*};

use crate::{Pokemon, pokeapi_db::prelude::*};

pub const DEFAULT_POKEMON_QUERY: &str = r#"
SELECT
    "pokemon_v2_pokemon"."id",
    "pokemon_v2_pokemon"."name",
    "pokemon_v2_pokemon"."pokemon_species_id"
FROM "pokemon_v2_pokemon"
JOIN
    "pokemon_v2_pokemonsprites" ON "pokemon_v2_pokemonsprites"."pokemon_id" = "pokemon_v2_pokemon"."id"
WHERE 1=1
    AND JSON_EXTRACT("pokemon_v2_pokemonsprites"."sprites", '$.front_default') IS NOT NULL
ORDER BY RANDOM()
LIMIT 1
"#;

pub const DEFAULT_SPRITES_QUERY: &str = r#"
SELECT
    "pokemon_v2_pokemonsprites"."id",
    COALESCE(JSON_EXTRACT("pokemon_v2_pokemonsprites"."sprites", '$.front_default'), '') AS "sprites"
FROM "pokemon_v2_pokemonsprites"
WHERE 1=1
    AND "pokemon_v2_pokemonsprites"."pokemon_id" = $1
ORDER BY RANDOM()
LIMIT 1
"#;

pub const DEFAULT_SPECIES_NAME_QUERY: &str = r#"
SELECT
    "pokemon_v2_pokemonspeciesname"."id",
    "pokemon_v2_pokemonspeciesname"."genus",
    "pokemon_v2_pokemonspeciesname"."name"
FROM "pokemon_v2_pokemonspeciesname"
WHERE 1=1
    AND "pokemon_v2_pokemonspeciesname"."language_id" = (
        SELECT
            "pokemon_v2_language"."id"
        FROM "pokemon_v2_language"
        WHERE 1=1
            AND "pokemon_v2_language"."name" = 'en'
        LIMIT 1
    )
    AND "pokemon_v2_pokemonspeciesname"."pokemon_species_id" = $1
ORDER BY RANDOM()
LIMIT 1
"#;

pub struct Db {
    db: DatabaseConnection,
}

impl Db {
    #[cfg(feature = "embed-db")]
    pub async fn new(db_path: &Option<impl AsRef<Path>>) -> anyhow::Result<Self> {
        let db = Database::connect(
            db_path
                .as_ref()
                .map(|p| path_to_connect_options(p.as_ref()))
                .unwrap_or_else(|| ConnectOptions::new("sqlite::memory:")),
        )
        .await?;

        {
            use sea_orm::sqlx::sqlite::SqliteOwnedBuf;
            let pool = db.get_sqlite_connection_pool();
            let mut conn = pool.acquire().await?;
            const DB_BYTES: &[u8] =
                include_bytes!(include_str!(concat!(env!("OUT_DIR"), "/embed_db_path")));
            conn.deserialize(None, SqliteOwnedBuf::try_from(DB_BYTES)?, true)
                .await?;
        }

        Ok(Self { db })
    }

    #[cfg(not(feature = "embed-db"))]
    pub async fn new(db_path: &Option<impl AsRef<Path>>) -> anyhow::Result<Self> {
        let db = Database::connect(path_to_connect_options(
            db_path
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Database not provided!"))?
                .as_ref(),
        ))
        .await?;
        Ok(Self { db })
    }

    pub async fn get_pokemon(
        &self,
        pokemon_query: impl AsRef<str>,
        species_name_query: impl AsRef<str>,
        sprites_query: impl AsRef<str>,
    ) -> anyhow::Result<Pokemon> {
        let pokemon = PokemonV2Pokemon::find()
            .from_raw_sql(Statement::from_string(
                DbBackend::Sqlite,
                pokemon_query.as_ref(),
            ))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Pokemon not found!"))?;

        let species_name = PokemonV2Pokemonspeciesname::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                species_name_query.as_ref(),
                vec![pokemon.pokemon_species_id.into()],
            ))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Pokemon species name not found!"))?;

        let sprites = PokemonV2Pokemonsprites::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Sqlite,
                sprites_query.as_ref(),
                vec![pokemon.id.into()],
            ))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Pokemon sprites not found!"))?;

        if sprites.sprites.is_empty() {
            return Err(anyhow::anyhow!("Pokemon sprite not found!"));
        } else if let Ok(sprites_json) = serde_json::from_str::<serde_json::Value>(&sprites.sprites)
            && !sprites_json.is_string()
        {
            return Err(anyhow::anyhow!("Pokemon sprite not a string!"));
        }

        Ok(Pokemon {
            name: species_name.name,
            sprite_url: sprites.sprites,
        })
    }

    #[cfg(feature = "embed-sprites")]
    pub fn get_sprites(url: &str) -> anyhow::Result<bytes::Bytes> {
        include!(concat!(env!("OUT_DIR"), "/embed_sprites_paths"))
            .get(
                url.replace(
                    "https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon/",
                    "",
                )
                .as_str(),
            )
            .ok_or_else(|| anyhow::anyhow!("Embedded Pokemon sprite is missing!"))
            .cloned()
    }
}

fn path_to_connect_options(path: &Path) -> ConnectOptions {
    ConnectOptions::new(format!("sqlite://{}", path.to_string_lossy()))
}
