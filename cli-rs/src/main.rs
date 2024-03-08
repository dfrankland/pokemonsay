use image::{codecs::png::PngDecoder, DynamicImage};
use rand::{thread_rng, Rng};
use viuer::{print, Config};

include!(concat!(env!("OUT_DIR"), "/files.rs"));

fn main() {
    let mut rng = thread_rng();
    let nth = rng.gen_range(0..SPRITES.len());
    let sprite_bytes = SPRITES.values().nth(nth).unwrap();
    let sprite_image = DynamicImage::from_decoder(PngDecoder::new(*sprite_bytes).unwrap()).unwrap();
    print(
        &sprite_image,
        &Config {
            absolute_offset: false,
            width: Some(20),
            ..Default::default()
        },
    )
    .unwrap();
}
