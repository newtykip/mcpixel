mod average;

use average::Blocks;
use color_space::{CompareCie1976, CompareCie2000, CompareCmc, CompareEuclidean, Rgb};
use derive_builder::Builder;
pub use image::imageops::FilterType as DownsamplingMethod;
use image::{imageops, DynamicImage, RgbaImage};
use include_dir::{include_dir, Dir};
use rayon::prelude::*;
use std::io::{Error, ErrorKind, Result};
use std::ops::{Div, Mul};
use strum::IntoEnumIterator;

const BLOCKS: Dir = include_dir!("assets/blocks");
const ASSET_SIZE: usize = 16;
const BUILD_LIMIT: u16 = 320;

#[derive(Builder)]
pub struct Options {
    colour_space: ColourSpace,
    downsampling: DownsamplingMethod,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            colour_space: ColourSpace::Cie2000,
            downsampling: DownsamplingMethod::Nearest,
        }
    }
}

#[derive(Clone)]
pub enum ColourSpace {
    Cie2000,
    Cie1976,
    Cmc,
    Euclidean,
}

pub struct Design {
    blocks: Vec<Vec<Option<Blocks>>>,
}

impl Design {
    pub fn new(image: &DynamicImage, build_height: u16, options: Option<Options>) -> Result<Self> {
        // ensure that the build height is not greater than the build limit
        if build_height > BUILD_LIMIT {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Height is greater than the build limit",
            ));
        }

        // resize the image to the build height
        let options = options.unwrap_or_default();
        let image = image.to_rgba8();
        let image = imageops::resize(
            &image,
            (image.width().mul(build_height as u32) + image.height() - 1).div(image.height()),
            build_height as u32,
            options.downsampling,
        );
        let (width, height) = image.dimensions();

        // create the output list
        let mut blocks = vec![vec![None; width as usize]; height as usize];

        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);

                // ignore transparent pixels
                if pixel.0[3] == 0 {
                    continue;
                }

                // find the most similar block
                let colour = Rgb::new(pixel.0[0] as f64, pixel.0[1] as f64, pixel.0[2] as f64);

                let block = Blocks::iter()
                    .par_bridge()
                    .map(|block| {
                        let avg: Rgb = block.clone().into();
                        let difference = match options.colour_space {
                            ColourSpace::Cie2000 => avg.compare_cie2000(&colour),
                            ColourSpace::Cie1976 => avg.compare_cie1976(&colour),
                            ColourSpace::Cmc => avg.compare_cmc(&colour),
                            ColourSpace::Euclidean => avg.compare_euclidean(&colour),
                        };

                        (block, avg, difference.floor() as usize)
                    })
                    .min_by_key(|x| x.2)
                    .unwrap()
                    .0;

                blocks[y as usize][x as usize] = Some(block);
            }
        }

        Ok(Self { blocks })
    }

    pub fn draw_image(&self) -> Result<RgbaImage> {
        let mut output = RgbaImage::new(
            (self.blocks[0].len() * ASSET_SIZE) as u32,
            (self.blocks.len() * ASSET_SIZE) as u32,
        );

        for (y, row) in self.blocks.iter().enumerate() {
            for (x, col) in row.iter().enumerate() {
                if let Some(path) = col {
                    let asset = BLOCKS.get_file(path);

                    if let Some(asset) = asset {
                        let asset = image::load_from_memory(asset.contents())
                            .map_err(|err| Error::new(ErrorKind::InvalidData, err))?
                            .to_rgba8();

                        imageops::overlay(&mut output, &asset, (x * 16) as i64, (y * 16) as i64);
                    }
                }
            }
        }

        Ok(output)
    }
}
