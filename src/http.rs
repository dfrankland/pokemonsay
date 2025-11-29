use serde::{Deserialize, Serialize};

use crate::Pokemon;

pub const DEFAULT_GRAPHQL_QUERY: &str = r#"
  query ($random_offset: Int!) {
    pokemon(
      offset: $random_offset
      order_by: [{id: asc}]
      limit: 1
      where: {pokemonsprites: {sprites: {_has_key: "front_default", _is_null: false}}}
    ) {
      pokemonspecy {
        pokemonspeciesnames(where: {language: {name: {_eq: "en"}}}) {
          name
        }
      }
      pokemonsprites {
        sprites(path: "front_default")
      }
    }
  }
"#;

#[derive(Debug, Clone)]
pub struct Http {
    client: reqwest::Client,
}

impl Http {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    // This API has a rate limit of 200 calls per hour
    // TODO: Add some caching to prevent repeated calls?
    const POKEAPI_GRAPHQL_API: &str = "https://graphql.pokeapi.co/v1beta2";

    async fn get_last_pokemon(&self) -> anyhow::Result<i64> {
        let body = serde_json::json!({
          "query": r#"
        {
          pokemon(limit: 1, order_by: [{order: desc}]) {
            id
          }
        }
      "#
        });
        let res = self
            .client
            .post(Self::POKEAPI_GRAPHQL_API)
            .body(serde_json::to_vec(&body)?)
            .send()
            .await?;
        let query: GraphQLQueryResponse<PokemonQueryResponse<LastPokemonQueryResponseFields>> =
            res.json().await?;
        Ok(query.data.pokemon.0.id)
    }

    pub async fn get_pokemon(&self, graphql_query: impl AsRef<str>) -> anyhow::Result<Pokemon> {
        let last_pokemon = self.get_last_pokemon().await?;
        let random_offset = rand::random_range(0..=last_pokemon);
        let body = serde_json::json!({
          "query": graphql_query.as_ref(),
          "variables": { "random_offset": random_offset }
        });
        let res = self
            .client
            .post(Self::POKEAPI_GRAPHQL_API)
            .body(serde_json::to_vec(&body)?)
            .send()
            .await?;
        let query: GraphQLQueryResponse<
            PokemonQueryResponse<PokemonSpecyAndSpritesQueryResponseFields>,
        > = res.json().await?;
        Ok(Pokemon {
            name: query.data.pokemon.0.pokemonspecy.pokemonspeciesnames.0.name,
            sprite_url: query.data.pokemon.0.pokemonsprites.0.sprites,
        })
    }

    pub async fn get_sprite(&self, url: &str) -> anyhow::Result<bytes::Bytes> {
        let res = self.client.get(url).send().await?;
        Ok(res.bytes().await?)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct GraphQLQueryResponse<T> {
    data: T,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
struct PokemonQueryResponse<T> {
    pokemon: (T,),
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
struct LastPokemonQueryResponseFields {
    id: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PokemonSpecyAndSpritesQueryResponseFields {
    pokemonspecy: PokemonSpecyQueryResponseFields,
    pokemonsprites: (PokemonSpritesQueryResponseFields,),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PokemonSpecyQueryResponseFields {
    pokemonspeciesnames: (PokemonSpeciesNamesQueryResponseFields,),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PokemonSpeciesNamesQueryResponseFields {
    name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PokemonSpritesQueryResponseFields {
    sprites: String,
}
