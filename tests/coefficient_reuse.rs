//! A `Resizer` keeps the coefficients of each axis for its next resize (photo-gallery's change, see
//! vendor/README.md): a resize with the same source size, crop, output size and filter as the last one on an axis reuses
//! them. Its result must be the same as a fresh resizer's, whatever came before it.

use fast_image_resize::images::Image;
use fast_image_resize::pixels::PixelType;
use fast_image_resize::{ResizeOptions, Resizer};

fn source(width: u32, height: u32) -> Image<'static> {
    let mut buffer = Vec::with_capacity((width * height * 3) as usize);
    for y in 0..height {
        for x in 0..width {
            buffer.push(((x * 7 + y * 3) % 256) as u8);
            buffer.push(((x * 11 + y * 5) % 256) as u8);
            buffer.push(((x * 13 + y * 17) % 256) as u8);
        }
    }
    Image::from_vec_u8(width, height, buffer, PixelType::U8x3).unwrap()
}

/// The result of resizing `src` to `width` × `height` with `options`, by a resizer of its own.
fn fresh(src: &Image<'_>, width: u32, height: u32, options: &ResizeOptions) -> Vec<u8> {
    let mut dst = Image::new(width, height, PixelType::U8x3);
    Resizer::new().resize(src, &mut dst, options).unwrap();
    dst.buffer().to_vec()
}

#[test]
fn a_repeated_resize_gives_the_fresh_result() {
    let src = source(120, 90);
    let options = ResizeOptions::new();
    let expected = fresh(&src, 37, 29, &options);

    let mut resizer = Resizer::new();
    for _ in 0..3 {
        let mut dst = Image::new(37, 29, PixelType::U8x3);
        resizer.resize(&src, &mut dst, &options).unwrap();
        assert_eq!(dst.buffer(), expected.as_slice());
    }
}

#[test]
fn a_sequence_of_sizes_and_crops_gives_the_fresh_results() {
    let src = source(120, 90);
    let cases = [
        (37, 29, ResizeOptions::new()),
        (50, 40, ResizeOptions::new()),
        (37, 29, ResizeOptions::new().crop(1.5, 0.0, 60.0, 40.0)),
        (37, 29, ResizeOptions::new().crop(0.0, 2.25, 90.0, 60.0)),
        (37, 29, ResizeOptions::new()),
        (50, 40, ResizeOptions::new().crop(1.5, 0.0, 60.0, 40.0)),
        (64, 48, ResizeOptions::new()),
    ];

    let mut resizer = Resizer::new();
    for (width, height, options) in &cases {
        let expected = fresh(&src, *width, *height, options);
        let mut dst = Image::new(*width, *height, PixelType::U8x3);
        resizer.resize(&src, &mut dst, options).unwrap();
        assert_eq!(dst.buffer(), expected.as_slice(), "{width}x{height}");
    }
}

#[test]
fn a_shifted_second_pass_after_a_reused_axis_gives_the_fresh_result() {
    // A crop that starts past the first pixel makes the second pass read a temporary image that
    // starts at the first bound (the shift). Resize it twice, so the second run reuses the shifted
    // coefficients' axes from the first.
    let src = source(200, 150);
    let options = ResizeOptions::new().crop(17.25, 9.5, 120.0, 100.0);
    let expected = fresh(&src, 45, 40, &options);

    let mut resizer = Resizer::new();
    for _ in 0..2 {
        let mut dst = Image::new(45, 40, PixelType::U8x3);
        resizer.resize(&src, &mut dst, &options).unwrap();
        assert_eq!(dst.buffer(), expected.as_slice());
    }
}
