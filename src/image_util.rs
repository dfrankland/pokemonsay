use image::{DynamicImage, GenericImageView, imageops::FilterType};

pub fn crop_transparent_pixels(img: &DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();

    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;

    // Find the bounding box
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            // Check if the alpha channel is not fully transparent (e.g., > 0)
            if pixel[3] > 0 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }

    // If no non-transparent pixels are found, return an empty image or handle as appropriate
    if min_x > max_x || min_y > max_y {
        return DynamicImage::new_rgba8(0, 0); // Or handle empty image case
    }

    // Calculate the new dimensions and crop
    let crop_width = max_x - min_x + 1;
    let crop_height = max_y - min_y + 1;

    img.crop_imm(min_x, min_y, crop_width, crop_height)
}

pub fn scale_by_integer(img: DynamicImage, factor: u8) -> DynamicImage {
    let (width, height) = img.dimensions();
    if factor <= 1 || width == 0 || height == 0 {
        return img;
    }

    img.resize_exact(
        width.saturating_mul(u32::from(factor)),
        height.saturating_mul(u32::from(factor)),
        FilterType::Nearest,
    )
}
