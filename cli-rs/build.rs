use quote::{quote, ToTokens};
use std::{env, fs, path::PathBuf};

fn codegen(sprites: Vec<PathBuf>) -> String {
    let inserts = sprites
        .iter()
        .map(|path| {
            let file_name = path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_token_stream();
            let file_path = path.to_str().unwrap().to_token_stream();
            quote! {
                m.insert(#file_name, include_bytes!(#file_path).as_slice());
            }
        })
        .collect::<Vec<_>>();

    quote! {
        use std::collections::HashMap;
        use lazy_static::lazy_static;

        lazy_static! {
            pub static ref SPRITES: HashMap<&'static str, &'static [u8]> = {
                let mut m = HashMap::new();
                #(#inserts)*
                m
            };
        }
    }
    .to_string()
}

fn main() {
    let files_path = fs::canonicalize("../data/files").unwrap();
    let mut files = fs::read_dir(files_path).unwrap();
    let mut sprites: Vec<PathBuf> = Vec::new();

    while let Some(Ok(dir_entry)) = files.next() {
        let file = dir_entry.path();
        sprites.push(file.clone());
        println!("cargo:rerun-if-changed={}", file.display());
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out_dir.join("files.rs"), codegen(sprites)).unwrap();
}
