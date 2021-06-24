use image::{ImageBuffer, Pixel, Rgba, RgbaImage, RgbImage};

use crate::skin::{self, Skin};

pub fn rescale(image: &RgbImage, scale: u32) -> RgbImage {
    let (width, height) = image.dimensions();
    let (scaled_width, scaled_height) = (width << scale, height << scale);

    ImageBuffer::from_fn(scaled_width, scaled_height, |scaled_x, scaled_y| {
        let x = scaled_x >> scale;
        let y = scaled_y >> scale;
        *image.get_pixel(x, y)
    })
}

pub fn render_face(skin: &Skin) -> RgbImage {
    let format = skin.format;

    let face = TexView::of(format.head.front, &skin.image);
    let hat = TexView::of(format.hat.front, &skin.image);

    let mut result = ImageBuffer::new(face.width, face.height);

    for y in 0..face.height {
        for x in 0..face.width {
            let mut face = *face.get_pixel(x, y);
            face.blend(hat.get_pixel(x, y));

            let [r, g, b, _] = face.0;
            result.put_pixel(x, y, image::Rgb([r, g, b]));
        }
    }

    result
}

struct TexView<'a> {
    offset: (u32, u32),
    width: u32,
    height: u32,
    image: &'a RgbaImage,
}

impl<'a> TexView<'a> {
    #[inline]
    fn of(region: skin::TexRegion, image: &image::RgbaImage) -> TexView {
        TexView {
            offset: region.origin,
            width: region.size.0,
            height: region.size.1,
            image,
        }
    }

    #[inline]
    fn get_pixel(&self, x: u32, y: u32) -> &Rgba<u8> {
        if x >= self.width || y >= self.height {
            panic!("tried to access pixel at ({}; {}) which is out of bounds for {}x{} view", x, y, self.width, self.height);
        }

        let (ox, oy) = self.offset;
        self.image.get_pixel(x + ox, y + oy)
    }
}
