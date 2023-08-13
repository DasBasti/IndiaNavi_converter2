// Setup warnings/errors:
#![forbid(unsafe_code)]
#![deny(bare_trait_objects, unused_doc_comments, unused_import_braces)]
// Clippy:
#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
#![allow(clippy::non_ascii_literal)]

use colored::*;
use image::io::Reader as ImageReader;
use image::ImageEncoder;
use image::{GenericImageView, ImageBuffer, ImageError, Pixel, Rgb, RgbImage};
use lab::Lab;
use num_integer::Roots;
use std::cmp::{max, min};
use std::io::Cursor;

const BLACK: Rgb<u8> = Rgb([0, 0, 0]);
const WHITE: Rgb<u8> = Rgb([255, 255, 255]);
const RED: Rgb<u8> = Rgb([255, 0, 0]);
const BLUE: Rgb<u8> = Rgb([0, 0, 255]);
const GREEN: Rgb<u8> = Rgb([0, 255, 0]);
const YELLOW: Rgb<u8> = Rgb([255, 255, 50]);
const ORANGE: Rgb<u8> = Rgb([255, 127, 0]);

pub fn color_to_raw(c: Rgb<u8>) -> u8 {
    match c {
        BLACK => 0,
        WHITE => 1,
        RED => 4,
        BLUE => 3,
        GREEN => 2,
        YELLOW => 5,
        ORANGE => 6,
        _ => 7,
    }
}

fn full_color(_: u32, _: u32, c: Vec<Rgb<u8>>) -> Rgb<u8> {
    c[0]
}

fn fiddyfiddy(x: u32, y: u32, c: Vec<Rgb<u8>>) -> Rgb<u8> {
    if (x % 2) == 1 {
        if (y % 2) == 1 {
            return c[0];
        } else {
            return c[1];
        }
    } else {
        if (y % 2) == 1 {
            return c[1];
        }
    }

    c[0]
}
/*/
fn threefourth(x: u32, y: u32, c: Vec<Rgb<u8>>) -> Rgb<u8> {
    if (x % 4) == 0 {
        if (y % 4) == 0 {
            return c[1];
        } else {
            return c[0];
        }
    }

    c[0]
}
*/

pub fn generic_map_color(x: u32, y: u32, pixel: Rgb<u8>) -> Rgb<u8> {
    let color_map: [(Lab, fn(u32, u32, Vec<Rgb<u8>>) -> Rgb<u8>, Vec<Rgb<u8>>); 14] = [
        (Lab::from_rgb(&[255, 255, 255]), full_color, vec![WHITE]),
        (Lab::from_rgb(&[0, 0, 0]), full_color, vec![BLACK]),
        (Lab::from_rgb(&[90, 90, 90]), full_color, vec![BLACK]),
        (Lab::from_rgb(&[0, 0, 255]), full_color, vec![BLUE]),
        (Lab::from_rgb(&[255, 0, 0]), full_color, vec![RED]),
        (Lab::from_rgb(&[0, 255, 0]), full_color, vec![GREEN]),
        (Lab::from_rgb(&[255, 127, 0]), full_color, vec![ORANGE]),
        (Lab::from_rgb(&[255, 255, 0]), full_color, vec![YELLOW]),
        (
            Lab::from_rgb(&[127, 127, 127]),
            fiddyfiddy,
            vec![BLACK, WHITE],
        ),
        (
            Lab::from_rgb(&[255, 255, 155]),
            fiddyfiddy,
            vec![YELLOW, WHITE],
        ),
        (
            Lab::from_rgb(&[127, 255, 127]),
            fiddyfiddy,
            vec![GREEN, WHITE],
        ),
        (
            Lab::from_rgb(&[212, 250, 212]),
            fiddyfiddy,
            vec![GREEN, WHITE],
        ),
        (
            Lab::from_rgb(&[251, 212, 157]),
            fiddyfiddy,
            vec![RED, WHITE],
        ),
        (Lab::from_rgb(&[127, 0, 255]), fiddyfiddy, vec![RED, BLUE]),
    ];

    let mut most_fitting_dE = f32::MAX;
    let mut most_fitting_idx = 0;
    for (idx, color) in color_map.iter().enumerate() {
        let lab = Lab::from_rgb(&pixel.0);
        let dE_squared = (lab.l - color.0.l).powf(2.0)
            + (lab.a - color.0.a).powf(2.0)
            + (lab.b - color.0.b).powf(2.0);
        if dE_squared < most_fitting_dE {
            most_fitting_idx = idx;
            most_fitting_dE = dE_squared;
        }
    }
    let rgb = color_map[most_fitting_idx].1(x, y, color_map[most_fitting_idx].2.clone());
    println!(
        "{} {}",
        format!("{:02x}{:02x}{:02x}", pixel[0], pixel[1], pixel[2])
            .on_truecolor(pixel[0], pixel[1], pixel[2]),
        format!(
            "{:02x}{:02x}{:02x} = {}",
            rgb[0], rgb[1], rgb[2], most_fitting_idx
        )
        .on_truecolor(rgb[0], rgb[1], rgb[2])
    );

    return rgb;
}

pub fn outdoor_map_color(x: u32, y: u32, pixel: Rgb<u8>) -> Rgb<u8> {
    let color_map: [(Rgb<u8>, fn(u32, u32, Vec<Rgb<u8>>) -> Rgb<u8>, Vec<Rgb<u8>>); 113] = [
        (Rgb([255, 255, 255]), full_color, vec![WHITE]),
        (Rgb([0, 0, 0]), full_color, vec![BLACK]),
        (Rgb([0, 0, 255]), full_color, vec![BLUE]),
        (Rgb([255, 0, 0]), full_color, vec![RED]),
        (Rgb([0, 255, 0]), full_color, vec![GREEN]),
        (Rgb([255, 127, 0]), full_color, vec![ORANGE]),
        (Rgb([255, 255, 0]), full_color, vec![YELLOW]),
        (Rgb([0xff, 0xff, 0xfb]), full_color, vec![WHITE]),
        (Rgb([0xec, 0xf3, 0xc4]), fiddyfiddy, vec![YELLOW, WHITE]),
        (Rgb([0xf1, 0xf2, 0xd9]), fiddyfiddy, vec![YELLOW, WHITE]),
        (Rgb([0xd3, 0xd3, 0xce]), fiddyfiddy, vec![BLACK, WHITE]),
        (Rgb([0xd2, 0xd3, 0xce]), fiddyfiddy, vec![BLACK, WHITE]),
        (Rgb([0xd2, 0xd3, 0xce]), fiddyfiddy, vec![BLACK, WHITE]),
        (Rgb([0xd7, 0xd9, 0xc5]), fiddyfiddy, vec![BLACK, WHITE]),
        (Rgb([0xe5, 0xf0, 0xd4]), fiddyfiddy, vec![GREEN, WHITE]),
        (Rgb([0xce, 0xe7, 0xc3]), fiddyfiddy, vec![GREEN, WHITE]),
        (Rgb([0xd1, 0xea, 0xc6]), fiddyfiddy, vec![GREEN, WHITE]),
        (Rgb([0x97, 0xcb, 0x8d]), fiddyfiddy, vec![GREEN, BLACK]),
        (Rgb([0xe6, 0xe9, 0xd4]), fiddyfiddy, vec![YELLOW, WHITE]),
        (Rgb([0xf0, 0xf3, 0xd1]), fiddyfiddy, vec![YELLOW, WHITE]),
        (Rgb([0xeb, 0xf4, 0xe9]), fiddyfiddy, vec![YELLOW, WHITE]),
        (Rgb([0xee, 0xf2, 0xd2]), fiddyfiddy, vec![YELLOW, WHITE]),
        (Rgb([0xef, 0xf0, 0xdc]), full_color, vec![WHITE]),
        (Rgb([0xff, 0xff, 0xda]), full_color, vec![ORANGE]),
        (Rgb([0x44, 0x44, 0x44]), full_color, vec![BLACK]),
        (Rgb([0x67, 0x66, 0xd9]), full_color, vec![BLUE]),
        (Rgb([0x86, 0xab, 0x84]), full_color, vec![BLACK]),
        (Rgb([0xad, 0xad, 0xaa]), full_color, vec![BLACK]),
        (Rgb([0x9d, 0x9d, 0x9b]), full_color, vec![BLACK]),
        (Rgb([0xef, 0xf2, 0xd2]), full_color, vec![YELLOW]),
        (Rgb([0xef, 0xf2, 0xd2]), fiddyfiddy, vec![GREEN, YELLOW]),
        (Rgb([0xda, 0xe7, 0xc5]), fiddyfiddy, vec![GREEN, WHITE]),
        (Rgb([0xa7, 0xcd, 0x92]), full_color, vec![GREEN]),
        (Rgb([0x55, 0xa6, 0xd8]), full_color, vec![BLUE]),
        (Rgb([0x6a, 0x69, 0xdc]), full_color, vec![BLUE]),
        (Rgb([0x97, 0xc6, 0xd6]), fiddyfiddy, vec![BLUE, WHITE]),
        (Rgb([0x8d, 0xb2, 0xb4]), fiddyfiddy, vec![BLUE, BLACK]),
        (Rgb([0xd9, 0xea, 0xa8]), full_color, vec![GREEN]),
        (Rgb([0xcc, 0xe3, 0x96]), full_color, vec![GREEN]),
        (Rgb([0xcc, 0xe3, 0x96]), full_color, vec![GREEN]),
        (Rgb([0xd3, 0xd4, 0xd1]), fiddyfiddy, vec![BLACK, WHITE]),
        (Rgb([0xe2, 0xe2, 0xdd]), fiddyfiddy, vec![BLACK, WHITE]),
        (Rgb([0xd5, 0xd5, 0xd2]), fiddyfiddy, vec![BLACK, WHITE]),
        (Rgb([0xd7, 0xd8, 0xd2]), fiddyfiddy, vec![BLACK, WHITE]),
        (Rgb([0xcc, 0xcd, 0xba]), fiddyfiddy, vec![BLACK, WHITE]),
        (Rgb([0xf8, 0xf9, 0xe0]), fiddyfiddy, vec![YELLOW, WHITE]),
        (Rgb([0xf5, 0xf6, 0xdd]), fiddyfiddy, vec![YELLOW, WHITE]),
        (Rgb([0xe3, 0xe4, 0xd4]), fiddyfiddy, vec![YELLOW, WHITE]),
        (Rgb([0xe9, 0xea, 0xd6]), fiddyfiddy, vec![YELLOW, WHITE]),
        (Rgb([0xb3, 0xb3, 0xac]), full_color, vec![BLACK]),
        (Rgb([0xdf, 0xe0, 0xdc]), full_color, vec![BLACK]),
        (Rgb([0xdd, 0xdd, 0xd9]), fiddyfiddy, vec![BLACK, WHITE]),
        (Rgb([0xdf, 0xe0, 0xdc]), fiddyfiddy, vec![BLACK, WHITE]),
        (Rgb([0xdd, 0xdd, 0xd9]), fiddyfiddy, vec![BLACK, WHITE]),
        (Rgb([0xf6, 0xf6, 0xf2]), full_color, vec![WHITE]),
        (Rgb([0xa3, 0xa4, 0x9e]), full_color, vec![BLACK]),
        (Rgb([0xb5, 0xb6, 0xa9]), full_color, vec![BLACK]),
        (Rgb([0xef, 0xf0, 0xd7]), full_color, vec![WHITE]),
        (Rgb([0xf1, 0xf2, 0xce]), fiddyfiddy, vec![YELLOW, WHITE]),
        (Rgb([0xf2, 0xf5, 0xd3]), fiddyfiddy, vec![YELLOW, WHITE]),
        (Rgb([0xc5, 0xc5, 0xc2]), full_color, vec![BLACK]),
        (Rgb([0xe9, 0xea, 0xe6]), full_color, vec![WHITE]),
        (Rgb([0x88, 0x89, 0x84]), full_color, vec![BLACK]),
        (Rgb([0x78, 0x79, 0x75]), full_color, vec![BLACK]),
        (Rgb([0xf5, 0xf5, 0xf2]), full_color, vec![WHITE]),
        (Rgb([0x4e, 0x52, 0xc4]), full_color, vec![BLUE]),
        (Rgb([0x66, 0x68, 0xca]), full_color, vec![BLUE]),
        (Rgb([0xa9, 0x74, 0xc6]), fiddyfiddy, vec![BLUE, RED]),
        (Rgb([0x73, 0xb9, 0x6d]), full_color, vec![GREEN]),
        (Rgb([0xa4, 0xd2, 0x9b]), full_color, vec![GREEN]),
        (Rgb([0xd6, 0xef, 0xca]), fiddyfiddy, vec![GREEN, WHITE]),
        (Rgb([0xd1, 0xea, 0xc5]), fiddyfiddy, vec![GREEN, WHITE]),
        (Rgb([0xbd, 0xe1, 0xb2]), fiddyfiddy, vec![GREEN, WHITE]),
        (Rgb([0xa9, 0xa9, 0xa6]), full_color, vec![BLACK]),
        (Rgb([0xf7, 0xf8, 0xdf]), full_color, vec![WHITE]),
        (Rgb([0xdb, 0xdc, 0xc7]), full_color, vec![WHITE]),
        (Rgb([0xe1, 0xe2, 0xd5]), full_color, vec![WHITE]),
        (Rgb([0xd7, 0xd7, 0xd4]), full_color, vec![WHITE]),
        (Rgb([0xe2, 0xe3, 0xdd]), full_color, vec![WHITE]),
        (Rgb([0x6a, 0x6a, 0x66]), full_color, vec![BLACK]),
        (Rgb([0xbb, 0xbc, 0xaa]), full_color, vec![BLACK]),
        (Rgb([0xd8, 0xd8, 0xba]), fiddyfiddy, vec![BLACK, YELLOW]),
        (Rgb([0x8a, 0x8a, 0x7d]), full_color, vec![BLACK]),
        (Rgb([0xe2, 0xe3, 0xdd]), full_color, vec![WHITE]),
        (Rgb([0xff, 0xf0, 0xce]), fiddyfiddy, vec![ORANGE, WHITE]),
        (Rgb([0x77, 0x78, 0x6b]), full_color, vec![BLACK]),
        (Rgb([0x86, 0x7b, 0x6e]), full_color, vec![BLACK]),
        (Rgb([0xfe, 0xfe, 0xfb]), full_color, vec![WHITE]),
        (Rgb([0xde, 0xea, 0xce]), fiddyfiddy, vec![GREEN, WHITE]),
        (Rgb([0xef, 0xf2, 0xd1]), fiddyfiddy, vec![YELLOW, WHITE]),
        (Rgb([0x94, 0x94, 0x92]), full_color, vec![BLACK]),
        (Rgb([0x96, 0x99, 0x8d]), full_color, vec![BLACK]),
        (Rgb([0x8f, 0x23, 0x31]), full_color, vec![RED]),
        (Rgb([0xc8, 0x1c, 0x33]), full_color, vec![RED]),
        (Rgb([0x69, 0x68, 0xe0]), fiddyfiddy, vec![BLUE, RED]),
        (Rgb([0x87, 0x87, 0xb8]), fiddyfiddy, vec![BLACK, RED]),
        (Rgb([0xe7, 0xea, 0xca]), fiddyfiddy, vec![WHITE, YELLOW]),
        (Rgb([0xf6, 0xf9, 0xd7]), fiddyfiddy, vec![WHITE, YELLOW]),
        (Rgb([0xd6, 0xe4, 0xac]), full_color, vec![GREEN]),
        (Rgb([0xc5, 0xdc, 0xb1]), full_color, vec![GREEN]),
        (Rgb([0xd2, 0xe4, 0x9a]), full_color, vec![GREEN]),
        (Rgb([0xf2, 0xf0, 0xc6]), full_color, vec![YELLOW]),
        (Rgb([0xf1, 0xf2, 0xdd]), full_color, vec![WHITE]),
        (Rgb([0xb1, 0x6a, 0xcb]), full_color, vec![RED]),
        (Rgb([0xbc, 0xbc, 0xa4]), full_color, vec![BLACK]),
        (Rgb([0xb4, 0xc5, 0xac]), full_color, vec![BLACK]),
        (Rgb([0xb3, 0xb3, 0x9c]), full_color, vec![BLACK]),
        (Rgb([0xb0, 0xd2, 0xd6]), full_color, vec![BLUE]),
        (Rgb([0x93, 0xc5, 0xd7]), full_color, vec![BLUE]),
        (Rgb([0xf1, 0xf5, 0xd2]), fiddyfiddy, vec![YELLOW, WHITE]),
        (Rgb([0xd3, 0xe3, 0xcd]), fiddyfiddy, vec![BLACK, WHITE]),
        (Rgb([0xd3, 0xe7, 0x99]), fiddyfiddy, vec![GREEN, YELLOW]),
        (Rgb([0xc8, 0xd9, 0xa6]), full_color, vec![GREEN]),
    ];

    // check for whiteish and blackish colors
    let max = [pixel[0], pixel[1], pixel[2]]
        .iter()
        .copied()
        .max()
        .unwrap();
    let min = [pixel[0], pixel[1], pixel[2]]
        .iter()
        .copied()
        .min()
        .unwrap();
    if max - min < 21 {
        if max > 80 {
            return WHITE;
        } else if max > 200 {
            return BLACK;
        } else {
            fiddyfiddy(x, y, vec![BLACK, WHITE]);
        }
    }

    // check if we have this color
    for (_idx, color) in color_map.iter().enumerate() {
        if pixel[0] == color.0[0] && pixel[1] == color.0[1] && pixel[2] == color.0[2] {
            return color.1(x, y, color.2.clone());
        }
    }
    println!(
        "{}",
        format!(
            "(Rgb([{:#04x},{:#04x},{:#04x}]), full_color, vec![BLACK]), ",
            pixel[0], pixel[1], pixel[2],
        )
        .on_truecolor(pixel[0], pixel[1], pixel[2])
    );

    return RED;
}

/*
fn encode_png<P, Container>(img: &ImageBuffer<P, Container>) -> Result<Vec<u8>, ImageError>
where
    P: Pixel<Subpixel = u8> + 'static,
    Container: Deref<Target = [P::Subpixel]>,
{
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    encoder.write_image(img, img.width(), img.height(), image::ColorType::Rgb8)?;
    Ok(buf)
}
*/
pub fn convert_image(image_data: &[u8]) -> Result<Vec<u8>, ImageError> {
    let in_img = ImageReader::new(Cursor::new(image_data))
        .with_guessed_format()?
        .decode()?;

    let (w, h) = in_img.dimensions();
    let mut output = RgbImage::new(w, h); // create a new buffer for our output

    let mut pxl: u8 = 0;
    let mut high = true;
    let mut raw = Vec::new();
    for (x, y, pixel) in in_img.pixels() {
        let rgb_pixel = image::Rgb([pixel[0], pixel[1], pixel[2]]);
        let png_pixel = generic_map_color(x, y, rgb_pixel);
        if high == true {
            pxl = color_to_raw(png_pixel) << 4;
        } else {
            pxl += color_to_raw(png_pixel);
            raw.push(pxl);
        }

        high = !high;

        output.put_pixel(x, y, png_pixel);
    }

    Ok(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_is_black() {
        let result = outdoor_map_color(0, 0, Rgb([0, 0, 0]));
        assert_eq!(result, Rgb([0, 0, 0]));
    }
}
