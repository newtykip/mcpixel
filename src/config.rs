use derive_builder::Builder;
pub use image::imageops::FilterType as DownsamplingMethod;

#[derive(Builder)]
pub struct Config {
    pub colour_space: ColourSpace,
    pub downsampling: DownsamplingMethod,
}

impl Default for Config {
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
