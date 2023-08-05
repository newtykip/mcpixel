mod average;

use average::Blocks;
#[cfg(cie1976)]
use color_space::CompareCie1976;
#[cfg(cie2000)]
use color_space::CompareCie2000;
#[cfg(cmc)]
use color_space::CompareCmc;
#[cfg(euclidean)]
use color_space::CompareEuclidean;
use color_space::Rgb;
use image::{imageops, DynamicImage, RgbaImage};
use include_dir::{include_dir, Dir};
use rayon::prelude::*;
use std::ops::{Div, Mul};
use strum::IntoEnumIterator;

const BLOCKS: Dir = include_dir!("assets/blocks");
const ASSET_SIZE: usize = 16;
const BUILD_LIMIT: u16 = 320;

fn asset_list(image: &DynamicImage, height: u16) -> Result<Vec<Vec<Option<Blocks>>>, &str> {
    // ensure that the build height is not greater than the build limit
    if height > BUILD_LIMIT {
        return Err("Height is greater than the build limit");
    }

    // compute the width of the build
    let image = image.to_rgba8();
    let width = (image.width().mul(height as u32) + image.height() - 1).div(image.height());

    // create the block list
    let mut blocks = vec![vec![None; width as usize]; height as usize];

    let ratio = (image.height()).div(height as u32);

    for y in 0..(height as u32) {
        for x in 0..width {
            let pixels = (0..ratio)
                .zip(0..ratio)
                .map(|(dx, dy)| image.get_pixel(x * ratio + dx, y * ratio + dy));

            let pixel_count = pixels.clone().count() as f64;

            let red = pixels
                .clone()
                .map(|pixel| pixel.0[0] as f64)
                .sum::<f64>()
                .div(pixel_count);

            let green = pixels
                .clone()
                .map(|pixel| pixel.0[1] as f64)
                .sum::<f64>()
                .div(pixel_count);

            let blue = pixels
                .clone()
                .map(|pixel| pixel.0[2] as f64)
                .sum::<f64>()
                .div(pixel_count);

            println!("{:?} {:?} {:?}", red, green, blue);

            let colour = Rgb::new(red, green, blue);

            // // ignore transparent pixels
            // if pixel.0[3] == 0 {
            //     continue;
            // }

            // find the most similar block
            let block = Blocks::iter()
                .par_bridge()
                .map(|block| {
                    let avg: Rgb = block.clone().into();

                    #[cfg(cie2000)]
                    let difference = avg.compare_cie2000(&colour);
                    #[cfg(cie1976)]
                    let difference = avg.compare_cie1976(&colour);
                    #[cfg(cmc)]
                    let difference = avg.compare_cmc(&colour);
                    #[cfg(euclidean)]
                    let difference = avg.compare_euclidean(&colour);

                    (block, avg, difference.floor() as usize)
                })
                .min_by_key(|x| x.2)
                .unwrap()
                .0;

            blocks[y as usize][x as usize] = Some(block);
        }
    }

    Ok(blocks)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kylo = image::open("cat.png")?;
    let assets = asset_list(&kylo, 57)?;
    let mut output = RgbaImage::new(
        (assets[0].len() * ASSET_SIZE) as u32,
        (assets.len() * ASSET_SIZE) as u32,
    );

    for (y, row) in assets.iter().enumerate() {
        for (x, col) in row.iter().enumerate() {
            if let Some(path) = col {
                let asset = BLOCKS.get_file(path);

                if let Some(asset) = asset {
                    let asset = image::load_from_memory(asset.contents())?.to_rgba8();

                    imageops::overlay(&mut output, &asset, (x * 16) as i64, (y * 16) as i64);
                }
            }
        }
    }

    output.save("output.png")?;

    Ok(())
}
