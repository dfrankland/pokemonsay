use std::{env, fs};
use std::path::Path;

use glob::glob;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    {
        let dest_path = Path::new(&out_dir).join("embed_db_path");
        let embed_db_path = env::var("EMBED_DB_PATH").unwrap_or_else(|_| {
          String::from(fs::canonicalize("./result/db.sqlite3").unwrap().to_string_lossy())
        });
        fs::write(dest_path, embed_db_path).unwrap();
    }

    {
      let dest_path = Path::new(&out_dir).join("embed_sprites_paths");
      let embed_sprites_path = env::var("EMBED_SPRITES_PATH").unwrap_or_else(|_| String::from("./result/sprites"));

      let sprite_entries_source = glob(&format!("{embed_sprites_path}/**/*.png")).unwrap().filter_map(|entry| entry.ok()).map(|path| {
        let key = path.file_name().unwrap().to_string_lossy();
        let value = fs::canonicalize(&path).unwrap();
        let value = value.to_string_lossy();
        format!(r#"("{key}", bytes::Bytes::from_static(include_bytes!("{value}")))"#)
      }).collect::<Vec<_>>().join(",\n");
      let sprites_source = format!(r#"
        std::collections::HashMap::<&str, bytes::Bytes>::from([
          {sprite_entries_source}
        ])
      "#);

      fs::write(dest_path, sprites_source).unwrap();
    }
}
