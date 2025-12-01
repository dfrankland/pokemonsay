mod db;
mod http;
mod image_util;
#[allow(clippy::all, dead_code)]
mod pokeapi_db;

use std::{
    io::{self, BufRead, IsTerminal},
    path::PathBuf,
};

#[cfg(not(feature = "embed-db"))]
use clap::builder::ArgPredicate;
use clap::{Parser, ValueEnum};
use image::GenericImageView;
use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::{
    db::{DEFAULT_POKEMON_QUERY, DEFAULT_SPECIES_NAME_QUERY, DEFAULT_SPRITES_QUERY, Db},
    http::{DEFAULT_GRAPHQL_QUERY, Http},
};

const DEFAULT_POKEMONSAY_TEMPLATE: &str = "Wild {pokemon} appeared!";

#[derive(Debug, Clone, Serialize)]
struct PokemonsayTemplateContext {
    pokemon: String,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Opt {
    /// Specifies the method to use for fetching Pokemon data
    ///
    /// `db`: Uses the database (embedded in the CLI or provided via `--db-path`).
    ///       Defaults to embedded database if available.
    ///
    /// `http`: Queries the PokeAPI GraphQL endpoint at `https://graphql.pokeapi.co/v1beta2`.
    ///         Has a rate limit of 200 queries per hour.
    #[cfg(feature = "embed-db")]
    #[arg(long, default_value = "db")]
    query_method: QueryMethod,

    /// Specifies the method to use for fetching Pokemon data
    ///
    /// `db`: Requires a database path to be provided via `--db-path`.
    ///
    /// `http`: Queries the PokeAPI GraphQL endpoint at `https://graphql.pokeapi.co/v1beta2`.
    ///         Has a rate limit of 200 queries per hour.
    #[cfg(not(feature = "embed-db"))]
    #[arg(
        long,
        default_value = "http",
        default_value_if("db_path", ArgPredicate::IsPresent, Some("db"))
    )]
    query_method: QueryMethod,

    /// Path to the PokeAPI SQLite database
    ///
    /// Can also be set via the `POKEMONSAY_DB_PATH` environment variable.
    ///
    /// Uses the database embedded within the CLI by default.
    #[cfg(feature = "embed-db")]
    #[arg(long, env = "POKEMONSAY_DB_PATH")]
    db_path: Option<PathBuf>,

    /// Path to the PokeAPI SQLite database
    ///
    /// Can also be set via the `POKEMONSAY_DB_PATH` environment variable.
    ///
    /// Required if the CLI queries the database with `query-method db`.
    #[cfg(not(feature = "embed-db"))]
    #[arg(long, env = "POKEMONSAY_DB_PATH", required_if_eq("query_method", "db"))]
    db_path: Option<PathBuf>,

    /// Specifies the method to use for retrieving Pokemon sprite images
    ///
    /// `embedded`: Uses sprite images embedded within the CLI.
    ///
    /// `http`: Downloads sprites from `https://raw.githubusercontent.com/PokeAPI/sprites/master`.
    #[cfg(feature = "embed-sprites")]
    #[arg(long, default_value = "embedded")]
    sprites_retrieval_method: SpriteRetrievalMethod,

    /// Specifies the method to use for retrieving Pokemon sprite images
    ///
    /// `http`: Downloads sprites from `https://raw.githubusercontent.com/PokeAPI/sprites/master`.
    #[cfg(not(feature = "embed-sprites"))]
    #[arg(long, default_value = "http")]
    sprites_retrieval_method: SpriteRetrievalMethod,

    #[arg(long, default_value = DEFAULT_POKEMON_QUERY, hide_default_value = true, help = format!("Custom SQL query to fetch Pokemon data from the database\n\nOnly used when `--query-method db` is set.\n\nDefault value:\n```sql{}```", DEFAULT_POKEMON_QUERY))]
    db_pokemon_query: String,

    #[arg(long, default_value = DEFAULT_SPRITES_QUERY, hide_default_value = true, help = format!("Custom SQL query to fetch Pokemon sprite URLs from the database\n\nOnly used when `--query-method db` is set.\n\nDefault value:\n```sql{}```", DEFAULT_SPRITES_QUERY))]
    db_sprites_query: String,

    #[arg(long, default_value = DEFAULT_SPECIES_NAME_QUERY, hide_default_value = true, help = format!("Custom SQL query to fetch Pokemon species names from the database\n\nOnly used when `--query-method db` is set.\n\nDefault value:\n```sql{}```", DEFAULT_SPECIES_NAME_QUERY))]
    db_species_name_query: String,

    #[arg(long, default_value = DEFAULT_GRAPHQL_QUERY, hide_default_value = true, help = format!("Custom GraphQL query to fetch Pokemon data\n\nOnly used when `--query-method http` is set.\n\nDefault value:\n```graphql{}```", DEFAULT_GRAPHQL_QUERY))]
    http_graphql_query: String,

    /// Template for the message displayed below the Pokemon sprite
    ///
    /// Uses TinyTemplate syntax with `{pokemon}` as the Pokemon name placeholder.
    ///
    /// Can be overridden by piping text to stdin.
    #[arg(long, default_value = DEFAULT_POKEMONSAY_TEMPLATE)]
    pokemonsay_template: String,

    /// Whether to crop transparent pixels from the Pokemon sprite background
    ///
    /// When enabled, removes transparent padding around the sprite image for a tighter display.
    #[arg(long)]
    crop_sprite_transparent_bg: bool,

    /// Maximum dimension (width or height) for displaying the Pokemon sprite in the terminal
    ///
    /// A value of `0` will disable setting a max dimension.
    ///
    /// For terminals that don't support graphics protocols, defaults to `0`.
    ///
    /// For terminals that support Kitty, iTerm2, or Sixel graphics protocols, defaults to `30`.
    #[arg(long, default_value = {if viuer::get_kitty_support() != viuer::KittySupport::None || viuer::is_iterm_supported() || viuer::is_sixel_supported() { "30" } else { "0" }})]
    max_sprite_dimension: u32,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum QueryMethod {
    Db,
    Http,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SpriteRetrievalMethod {
    #[cfg(feature = "embed-sprites")]
    Embedded,
    Http,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    let pokemon = match opt.query_method {
        QueryMethod::Db => {
            let db = Db::new(&opt.db_path).await?;
            db.get_pokemon(
                &opt.db_pokemon_query,
                &opt.db_species_name_query,
                &opt.db_sprites_query,
            )
            .await?
        }
        QueryMethod::Http => {
            let http = Http::new();
            http.get_pokemon(opt.http_graphql_query).await?
        }
    };

    let sprite_bytes = match opt.sprites_retrieval_method {
        #[cfg(feature = "embed-sprites")]
        SpriteRetrievalMethod::Embedded => Db::get_sprites(&pokemon.sprite_url)?,
        SpriteRetrievalMethod::Http => {
            let http = Http::new();
            http.get_sprite(&pokemon.sprite_url).await?
        }
    };

    let mut sprite_image = image::load_from_memory(sprite_bytes.as_ref())?;

    if opt.crop_sprite_transparent_bg {
        sprite_image = crate::image_util::crop_transparent_pixels(&sprite_image);
    }

    let mut viuer_config = viuer::Config {
        transparent: true,
        absolute_offset: false,
        premultiplied_alpha: true,
        ..Default::default()
    };

    // TODO: Check if PNG DPI needs reading to figure out how to set max size appropriately
    let (width, height) = sprite_image.dimensions();
    if opt.max_sprite_dimension > 0 {
        if width >= height {
            viuer_config.width = Some(width.min(opt.max_sprite_dimension));
        } else if height >= width {
            viuer_config.height = Some(height.min(opt.max_sprite_dimension));
        }
    }

    viuer::print(&sprite_image, &viuer_config)?;

    let mut tt = TinyTemplate::new();
    const TEMPLATE_NAME: &str = "pokemonsay";
    let piped_in_template = {
        let stdin = io::stdin();
        if stdin.is_terminal() {
            None
        } else {
            let mut line = String::new();
            let mut handle = stdin.lock();
            handle.read_line(&mut line)?;
            Some(String::from(line.trim()))
        }
    };
    let template = piped_in_template.unwrap_or(opt.pokemonsay_template);
    tt.add_template(TEMPLATE_NAME, &template)?;
    let context = PokemonsayTemplateContext {
        pokemon: pokemon.name.to_uppercase(),
    };
    let rendered = tt.render(TEMPLATE_NAME, &context)?;

    // TODO: Render the box based on the size of the rendered sprite,
    // centering the rendered text
    let rendered_len = rendered.len();
    const PADDING: usize = 8;
    assert!(PADDING.is_multiple_of(2));
    println!("◓{}◓", "═".repeat(rendered_len + PADDING));
    println!("‖{}‖", " ".repeat(rendered_len + PADDING));
    println!(
        "‖{padding}{rendered}{padding}‖",
        padding = " ".repeat(PADDING / 2)
    );
    println!("‖{}‖", " ".repeat(rendered_len + PADDING));
    println!("◓{}◓", "═".repeat(rendered_len + PADDING));

    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct Pokemon {
    pub name: String,
    pub sprite_url: String,
}
