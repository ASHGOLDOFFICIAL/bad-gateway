use image::{DynamicImage, GenericImageView, Pixel, Rgb, RgbImage, imageops::FilterType};

use crate::{ArtCell, AsciiArt};

/// Default charset from darkest to brightest.
pub const DEFAULT_CHARSET: &[char] = &[' ', '░', '▒', '▓', '█'];

/// The default cell aspect ratio.
pub const DEFAULT_CELL_ASPECT_RATIO: f64 = 1.0;

/// Config for [`AsciiArtist`].
#[must_use]
pub struct AsciiArtistConfig {
    charset: Vec<char>,
    cell_aspect_ratio: f64,
}

impl Default for AsciiArtistConfig {
    #[inline(always)]
    fn default() -> Self {
        Self {
            charset: DEFAULT_CHARSET.to_vec(),
            cell_aspect_ratio: DEFAULT_CELL_ASPECT_RATIO,
        }
    }
}

impl AsciiArtistConfig {
    /// Sets charset used in [`AsciiArt`].
    ///
    /// Defaults to [`DEFAULT_CHARSET`].
    #[inline]
    pub fn with_charset(mut self, charset: &[char]) -> Result<Self, &'static str> {
        let charset = charset.to_vec();
        if charset.is_empty() {
            Err("charset must not be empty")
        } else {
            self.charset = charset;
            Ok(self)
        }
    }

    /// Height to width ratio, sets how many source rows should be
    /// sampled per output row relative to columns per output column.
    /// Useful for terminals where cells aren't squares.
    ///
    /// Must be in (0, 100]
    ///
    /// Defaults to [`DEFAULT_CELL_ASPECT_RATIO`] (square).
    #[inline]
    pub fn with_cell_aspect_ratio(mut self, cell_aspect_ratio: f64) -> Result<Self, &'static str> {
        if !cell_aspect_ratio.is_finite() || cell_aspect_ratio <= 0.0 || cell_aspect_ratio > 100.0 {
            Err("cell_aspect_ratio must be finite and within (0, 100]")
        } else {
            self.cell_aspect_ratio = cell_aspect_ratio;
            Ok(self)
        }
    }
}

/// Configures and performs image-to-[`AsciiArt`] renders.
#[must_use]
pub struct AsciiArtist {
    config: AsciiArtistConfig,
}

impl AsciiArtist {
    /// Makes new `AsciiArtist` for the given config.
    #[inline(always)]
    pub const fn new(config: AsciiArtistConfig) -> Self {
        Self { config }
    }

    /// Renders `image` to fit exactly `width` cells wide,
    /// deriving the height from the image's own aspect ratio.
    #[inline(always)]
    pub fn render_with_width(&self, image: &DynamicImage, width: u16) -> AsciiArt {
        let height = self.height_for_width(image, width);
        self.render(image, width, height)
    }

    /// Renders `image` to fit exactly `height` cells tall,
    /// deriving the width from the image's own aspect ratio.
    #[inline(always)]
    pub fn render_with_height(&self, image: &DynamicImage, height: u16) -> AsciiArt {
        let width = self.width_for_height(image, height);
        self.render(image, width, height)
    }

    /// Renders `image` to fit exactly `width` x `height` cells.
    pub fn render(&self, image: &DynamicImage, width: u16, height: u16) -> AsciiArt {
        if width == 0 || height == 0 {
            return AsciiArt::EMPTY;
        }

        let sample_height_float = (f64::from(height) * self.config.cell_aspect_ratio).round();
        let sample_height = (sample_height_float as u32).max(1);
        let oversampled = image
            .resize_exact(u32::from(width), sample_height, FilterType::Triangle)
            .to_rgb8();

        let cells = (0..height)
            .flat_map(|row| (0..width).map(move |col| (row, col)))
            .map(|(row, col)| {
                let start = u32::from(row) * sample_height / u32::from(height);
                let end = ((u32::from(row) + 1) * sample_height / u32::from(height))
                    .clamp(start + 1, sample_height);
                let pixel = Self::averaged_pixel(&oversampled, u32::from(col), start, end);
                Self::pixel_to_cell(&self.config.charset, pixel)
            })
            .collect();

        AsciiArt {
            width,
            height,
            cells,
        }
    }

    /// Averages oversampled rows `start..end` of column `col` into one pixel.
    fn averaged_pixel(image: &RgbImage, col: u32, start: u32, end: u32) -> Rgb<u8> {
        let count = end - start;
        let (r, g, b) = (start..end)
            .map(|row| image.get_pixel(col, row).0)
            .fold((0u32, 0u32, 0u32), |(r, g, b), [cr, cg, cb]| {
                (r + u32::from(cr), g + u32::from(cg), b + u32::from(cb))
            });

        Rgb([(r / count) as u8, (g / count) as u8, (b / count) as u8])
    }

    fn height_for_width(&self, image: &DynamicImage, width: u16) -> u16 {
        let (image_width, image_height) = image.dimensions();
        if image_width == 0 {
            return 0;
        }
        let ratio = f64::from(width) / f64::from(image_width) / self.config.cell_aspect_ratio;
        let height = ratio * f64::from(image_height);
        height.round().clamp(0.0, f64::from(u16::MAX)) as u16
    }

    fn width_for_height(&self, image: &DynamicImage, height: u16) -> u16 {
        let (image_width, image_height) = image.dimensions();
        if image_height == 0 {
            return 0;
        }
        let ratio = f64::from(height) / f64::from(image_height) * self.config.cell_aspect_ratio;
        let width = ratio * f64::from(image_width);
        width.round().clamp(0.0, f64::from(u16::MAX)) as u16
    }

    fn pixel_to_cell(charset: &[char], pixel: Rgb<u8>) -> ArtCell {
        let luminance = f32::from(pixel.to_luma().0[0]);
        let index = ((luminance / 255.0) * (charset.len() - 1) as f32).round() as usize;
        (charset[index], (pixel[0], pixel[1], pixel[2]))
    }
}

#[cfg(test)]
mod tests {
    use image::Rgb as TestRgb;

    use super::*;

    #[test]
    fn config_rejects_an_empty_charset() {
        assert!(AsciiArtistConfig::default().with_charset(&[]).is_err());
    }

    #[test]
    fn config_rejects_a_non_positive_cell_aspect_ratio() {
        assert!(
            AsciiArtistConfig::default()
                .with_cell_aspect_ratio(0.0)
                .is_err()
        );
        assert!(
            AsciiArtistConfig::default()
                .with_cell_aspect_ratio(-1.0)
                .is_err()
        );
    }

    #[test]
    fn config_rejects_an_excessively_large_cell_aspect_ratio() {
        assert!(
            AsciiArtistConfig::default()
                .with_cell_aspect_ratio(1_000_000.0)
                .is_err()
        );
    }

    #[test]
    fn config_rejects_a_non_finite_cell_aspect_ratio() {
        assert!(
            AsciiArtistConfig::default()
                .with_cell_aspect_ratio(f64::NAN)
                .is_err()
        );
        assert!(
            AsciiArtistConfig::default()
                .with_cell_aspect_ratio(f64::INFINITY)
                .is_err()
        );
    }

    fn solid(width: u32, height: u32, rgb: [u8; 3]) -> DynamicImage {
        DynamicImage::ImageRgb8(RgbImage::from_pixel(width, height, TestRgb(rgb)))
    }

    fn artist() -> AsciiArtist {
        AsciiArtist::new(AsciiArtistConfig::default())
    }

    #[test]
    fn black_renders_as_the_darkest_glyph() {
        let image = solid(4, 4, [0, 0, 0]);
        let art = artist().render(&image, 2, 2);
        assert_eq!(art.get(0, 0), Some((DEFAULT_CHARSET[0], (0, 0, 0))));
    }

    #[test]
    fn white_renders_as_the_brightest_glyph() {
        let image = solid(4, 4, [255, 255, 255]);
        let art = artist().render(&image, 2, 2);
        assert_eq!(
            art.get(0, 0).unwrap().0,
            DEFAULT_CHARSET[DEFAULT_CHARSET.len() - 1]
        );
    }

    #[test]
    fn zero_sized_area_yields_no_cells() {
        let image = solid(4, 4, [10, 10, 10]);
        assert_eq!(artist().render(&image, 0, 3).size(), (0, 0));
        assert!(artist().render(&image, 0, 3).get(0, 0).is_none());
        assert!(artist().render(&image, 3, 0).get(0, 0).is_none());
    }

    #[test]
    fn get_is_none_on_out_of_bounds() {
        let image = solid(4, 4, [1, 2, 3]);
        let art = artist().render(&image, 2, 2);
        assert!(art.get(2, 0).is_none());
        assert!(art.get(0, 2).is_none());
    }

    #[test]
    fn custom_charset_is_used_instead_of_the_default() {
        let image = solid(4, 4, [0, 0, 0]);
        let charset = ['x', 'y', 'z'];
        let config = AsciiArtistConfig::default().with_charset(&charset).unwrap();
        let art = AsciiArtist::new(config).render(&image, 2, 2);
        assert_eq!(art.get(0, 0).unwrap().0, 'x');
    }

    #[test]
    fn render_with_width_derives_height_from_aspect_ratio() {
        let image = solid(8, 4, [1, 2, 3]);
        let art = artist().render_with_width(&image, 4);
        assert_eq!(art.size(), (4, 2));
    }

    #[test]
    fn render_with_height_derives_width_from_aspect_ratio() {
        let image = solid(8, 4, [1, 2, 3]);
        let art = artist().render_with_height(&image, 2);
        assert_eq!(art.size(), (4, 2));
    }

    #[test]
    fn custom_cell_aspect_ratio_changes_the_derived_height() {
        let image = solid(8, 4, [1, 2, 3]);
        let config = AsciiArtistConfig::default()
            .with_cell_aspect_ratio(2.0)
            .unwrap();
        let art = AsciiArtist::new(config).render_with_width(&image, 4);
        assert_eq!(art.size(), (4, 1));
    }

    #[test]
    fn non_default_cell_aspect_ratio_samples_more_vertical_detail() {
        let mut image = RgbImage::new(1, 4);
        for y in 0..2 {
            image.put_pixel(0, y, TestRgb([0, 0, 0]));
        }
        for y in 2..4 {
            image.put_pixel(0, y, TestRgb([255, 255, 255]));
        }
        let image = DynamicImage::ImageRgb8(image);

        let square = AsciiArtistConfig::default();
        let square_art = AsciiArtist::new(square).render(&image, 1, 1);
        assert_eq!(square_art.get(0, 0).unwrap().1, (128, 128, 128));

        let tall = AsciiArtistConfig::default()
            .with_cell_aspect_ratio(2.0)
            .unwrap();
        let tall_art = AsciiArtist::new(tall).render(&image, 1, 2);
        assert_eq!(tall_art.get(0, 0).unwrap().1, (0, 0, 0));
        assert_eq!(tall_art.get(0, 1).unwrap().1, (255, 255, 255));
    }
}
