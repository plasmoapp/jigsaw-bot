use image::{DynamicImage, RgbaImage};
use tiny_skia::{IntSize, Mask, Pixmap};

pub trait MaskApply {
    fn apply_to_image(&self, image: &DynamicImage) -> DynamicImage;
}

impl MaskApply for Mask {
    fn apply_to_image(&self, image: &DynamicImage) -> DynamicImage {
        let image_bytes = image.to_rgba8().to_vec();

        let mut pixmap = Pixmap::from_vec(
            image_bytes,
            IntSize::from_wh(image.width(), image.height())
                .expect("width and height should be positive"),
        ).expect("image should be valid");

        pixmap.apply_mask(self);

        DynamicImage::from(
            RgbaImage::from_raw(
                image.width(),
                image.height(),
                pixmap.data().to_vec(),
            ).expect("image should be valid"),
        )
    }
}
