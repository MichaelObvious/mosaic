extern crate image;

use image::io::Reader as ImageReader;
use image::GenericImageView;
use image::GenericImage;
use image::imageops::FilterType::Triangle;

use std::io::{Write};
use std::collections::HashMap;



const TILE_SIZE: u32 = 1;
const OUTPUT_WIDTH: u32 = 512;
const DITHERING: bool = true;
const TILES_FOLDER: &str = "./tiles/";
const INPUT_IMAGE: &str = "./input.png";
const OUTPUT_PATH: &str = "./output.png";



fn find_images(dir: &str) -> Vec<std::path::PathBuf> {
    let mut images: Vec<std::path::PathBuf> = Vec::new();
    for element in std::path::Path::new(dir).read_dir().unwrap() {
        let path = element.unwrap().path();
        if let Some(extension) = path.extension() {
            if extension == "jpeg" || extension == "jpg" || extension == "png" || extension == "gif" {
                images.push(path);
            }
        }
    }
    
    images.sort();

    return images;
}

// fn find_files(dir: &str) -> Vec<std::path::PathBuf> {
//     let mut paths: Vec<std::path::PathBuf> = Vec::new();
//     for element in std::path::Path::new(dir).read_dir().unwrap() {
//         let path = element.unwrap().path();
//         paths.push(path);
//     }
    
//     paths.sort();

//     return paths;
// }

fn print_progress(title: &str, progress: f32) {
    const PROGRESS_BAR_SIZE: u32 = 50;
    print!("\r{} [", title);
    if progress >= 1.0 / PROGRESS_BAR_SIZE as f32 {
        for _ in 0..((progress * PROGRESS_BAR_SIZE as f32) as u32 - 1) {
            print!("=");
         }
    }
    print!(">");
    for _ in ((progress * PROGRESS_BAR_SIZE as f32) as u32)..PROGRESS_BAR_SIZE {
        print!(" ");
    }
    print!("] {:0>4.2}% ", progress * 100.0);
    std::io::stdout().flush().unwrap();
}

fn distance(a: &(u8, u8, u8), b: &(u8, u8, u8)) -> f32 {
    let a1 = (a.0 as f32, a.1 as f32, a.2 as f32);
    let b1 = (b.0 as f32, b.1 as f32, b.2 as f32);

    // return (a1.0.max(a1.1).max(a1.2) - b1.0.max(b1.1).max(b1.2)).abs();

    // return ((a1.0 + a1.1 + a1.2) / 3.0 - (b1.0 + b1.1 + b1.2) / 3.0).abs(); Ew

    return ((a1.0 - b1.0).powf(2.0) + (a1.1 - b1.1).powf(2.0) + (a1.2 - b1.2).powf(2.0)).sqrt();
}

fn find_nearest_tile(c: (u8, u8, u8), tiles: &HashMap<(u8, u8, u8), image::DynamicImage>) -> ((u8, u8, u8), (f32, f32, f32)) {
    let mut nearest: (u8, u8, u8) = (0, 0, 0);
    let mut d = f32::INFINITY;
    for k in tiles.keys() {
        let new_d = distance(&c, &k);
        if new_d < d {
            d = new_d;
            nearest = *k;
        }
    }
    let error = (c.0 as f32 - nearest.0 as f32, c.1 as f32 - nearest.1 as f32, c.2 as f32 - nearest.2 as f32);
    return (nearest, error);
}

fn main() {
    // Create HashMap of images
    let mut tiles: HashMap<(u8, u8, u8), image::DynamicImage> = HashMap::new();
    let mut tiles_used: HashMap<(u8, u8, u8), bool> = HashMap::new();

    let image_files = find_images(TILES_FOLDER);

    print_progress("Parsing tiles", 0.0);
    for (n, path) in image_files.iter().enumerate() {
        if let Ok(image) = ImageReader::open(path).unwrap().decode() {
            let tile = image.resize_to_fill(TILE_SIZE, TILE_SIZE, Triangle);
            let colour = image.resize_to_fill(1, 1, Triangle).get_pixel(0, 0);

            let col = (colour.0[0], colour.0[1], colour.0[2]);
            tiles.insert(col, tile);
        }
        print_progress("Parsing tiles", (n+1) as f32 / image_files.len() as f32);
    }
    println!();
    println!("\t Collected {} tiles", tiles.keys().len());

    // Pick image and downscale
    let input = ImageReader::open(INPUT_IMAGE).unwrap().decode().unwrap();
    let size = (OUTPUT_WIDTH, (OUTPUT_WIDTH as f32 / (input.width() as f32 / input.height() as f32)) as u32);
    let mut scaled_input = input.resize_to_fill(size.0, size.1, Triangle);
    // scaled_input.save("scaled-input.png").unwrap();

    // Create output image
    let mut output = image::DynamicImage::new_rgba8(size.0 * TILE_SIZE, size.1 * TILE_SIZE);

    print_progress("Placing tiles", 0.0);
    for y in 0..scaled_input.height() {
        for x in 0..scaled_input.width() {
            let colour = scaled_input.get_pixel(x, y).0;
            let (key, error) = find_nearest_tile((colour[0], colour[1], colour[2]), &tiles);
            let tile = &tiles[&key];
            tiles_used.insert(key, true);

            // Dither
            if DITHERING {
                if x + 1 < scaled_input.width() {
                    let mut new_col = scaled_input.get_pixel(x + 1, y);
                    new_col.0[0] = (new_col.0[0] as i32 + (error.0 * 7.0 / 16.0) as i32).max(0).min(255) as u8;
                    new_col.0[1] = (new_col.0[1] as i32 + (error.1 * 7.0 / 16.0) as i32).max(0).min(255) as u8;
                    new_col.0[2] = (new_col.0[2] as i32 + (error.2 * 7.0 / 16.0) as i32).max(0).min(255) as u8;
                    scaled_input.put_pixel(x + 1, y, new_col);
                }

                if y + 1 < scaled_input.height() && x >= 1 {
                    let mut new_col = scaled_input.get_pixel(x - 1, y + 1);
                    new_col.0[0] = (new_col.0[0] as i32 + (error.0 * 3.0 / 16.0) as i32).max(0).min(255) as u8;
                    new_col.0[1] = (new_col.0[1] as i32 + (error.1 * 3.0 / 16.0) as i32).max(0).min(255) as u8;
                    new_col.0[2] = (new_col.0[2] as i32 + (error.2 * 3.0 / 16.0) as i32).max(0).min(255) as u8;
                    scaled_input.put_pixel(x - 1, y + 1, new_col);
                }

                if y + 1 < scaled_input.height() {
                    let mut new_col = scaled_input.get_pixel(x, y + 1);
                    new_col.0[0] = (new_col.0[0] as i32 + (error.0 * 5.0 / 16.0) as i32).max(0).min(255) as u8;
                    new_col.0[1] = (new_col.0[1] as i32 + (error.1 * 5.0 / 16.0) as i32).max(0).min(255) as u8;
                    new_col.0[2] = (new_col.0[2] as i32 + (error.2 * 5.0 / 16.0) as i32).max(0).min(255) as u8;
                    scaled_input.put_pixel(x, y + 1, new_col);
                }

                if y + 1 < scaled_input.height() && x + 1 < scaled_input.width() {
                    let mut new_col = scaled_input.get_pixel(x + 1, y + 1);
                    new_col.0[0] = (new_col.0[0] as i32 + (error.0 * 1.0 / 16.0) as i32).max(0).min(255) as u8;
                    new_col.0[1] = (new_col.0[1] as i32 + (error.1 * 1.0 / 16.0) as i32).max(0).min(255) as u8;
                    new_col.0[2] = (new_col.0[2] as i32 + (error.2 * 1.0 / 16.0) as i32).max(0).min(255) as u8;
                    scaled_input.put_pixel(x + 1, y + 1, new_col);
                }
            }

            output.copy_from(&(*tile).view(0, 0, tile.width(), tile.height()), x * TILE_SIZE, y * TILE_SIZE).unwrap();
            if (y * scaled_input.width() + x + 1) % (scaled_input.width() * scaled_input.height() / 1000) == 0 {
                print_progress("Placing tiles", (y * scaled_input.width() + x + 1) as f32 / (scaled_input.width() * scaled_input.height()) as f32);
            }
        }
    }
    output.save(OUTPUT_PATH).unwrap();
    print_progress("Placing tiles", 1.0);
    println!();

    let mut used = 0;
    for _ in tiles_used.keys() {
        used += 1;
    }
    println!("\t{}/{} tiles used", used, tiles.keys().len());

}