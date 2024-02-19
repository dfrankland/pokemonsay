use anyhow::Result;
use reqwest::{
    blocking::{Client as BlockingClient, RequestBuilder},
    Url,
};
use sha1_smol::Sha1;
use std::{env, fs, path::PathBuf};

mod mediawiki_api {
    use anyhow::Result;
    use reqwest::Url;
    use serde::Deserialize;
    use std::collections::HashMap;

    // https://archives.bulbagarden.net/wiki/Category:HOME_menu_sprites
    const DEFAULT_URL: &str = "https://archives.bulbagarden.net/w/api.php";
    const DEFAULT_URL_PARAMS: &[(&str, &str)] = &[
        ("action", "query"),
        ("generator", "categorymembers"),
        ("gcmtitle", "Category:HOME_menu_sprites"),
        ("gcmlimit", "500"), // 500 is the max that can be queried at one time
        ("prop", "imageinfo"),
        ("iiprop", "url|sha1"),
        ("format", "json"),
    ];

    pub fn paginated_url(pagination_params: &Option<HashMap<String, String>>) -> Result<String> {
        let mut combined_params = Vec::from(DEFAULT_URL_PARAMS);
        if let Some(params) = pagination_params {
            combined_params.extend(params.iter().map(|(k, v)| (k.as_ref(), v.as_ref())));
        }

        Ok(Url::parse_with_params(DEFAULT_URL, combined_params)?.to_string())
    }

    #[derive(Deserialize)]
    pub struct ImageInfo {
        pub url: String,
        pub sha1: String,
    }

    #[derive(Deserialize)]
    pub struct Image {
        pub title: String,
        pub imageinfo: (ImageInfo,),
    }

    #[derive(Deserialize)]
    pub struct QueryPages {
        pub pages: HashMap<String, Image>,
    }

    #[derive(Deserialize)]
    pub struct ParseQuery {
        pub r#continue: Option<HashMap<String, String>>,
        pub query: QueryPages,
    }
}

// needed to prevent Cloudflare from blocking requests
fn unblock_cloudflare(request_builder: RequestBuilder) -> RequestBuilder {
    request_builder.header("User-Agent", "curl/8.1.1")
}

pub fn update_file(
    base_bath: &PathBuf,
    image: &mediawiki_api::Image,
    blocking_client: &BlockingClient,
) -> Result<()> {
    let file_url = Url::parse(&image.imageinfo.0.url)?;
    let file_name = file_url
        .path_segments()
        .expect("Should not fail")
        .last()
        .unwrap_or_else(|| image.title.as_ref());
    let file_path = base_bath.join(file_name);

    if file_path.try_exists()? {
        let mut sha1 = Sha1::new();
        sha1.update(&fs::read(&file_path)?);
        let sha1_digest = sha1.digest().to_string();
        if sha1_digest == image.imageinfo.0.sha1 {
            println!("\tFile is up to date: {}", file_path.display());
            return Ok(());
        }
    }

    println!("\tQuerying {file_url}");
    let bytes = unblock_cloudflare(blocking_client.get(file_url))
        .send()?
        .bytes()?;
    println!("\t\tUpdating {}", file_path.display());
    fs::write(&file_path, bytes)?;

    Ok(())
}

fn main() -> Result<()> {
    let base_path = env::args_os().skip(1).next().map_or_else(
        || env::current_dir().map(|p| p.join("files")),
        |p| Ok(PathBuf::from(p)),
    )?;
    fs::create_dir_all(&base_path)?;

    let blocking_client = BlockingClient::new();
    let mut pagination_params = None;
    loop {
        let url = mediawiki_api::paginated_url(&pagination_params)?;
        println!("Querying {url}");
        let parse_query = unblock_cloudflare(blocking_client.get(url))
            .send()?
            .json::<mediawiki_api::ParseQuery>()?;

        for image in parse_query.query.pages.values() {
            update_file(&base_path, image, &blocking_client)?;
        }

        pagination_params = parse_query.r#continue;
        if pagination_params.is_none() {
            break;
        }
    }

    Ok(())
}
