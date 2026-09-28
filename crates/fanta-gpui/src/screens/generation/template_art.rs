//! Bundled sample artwork for hosts that provide prompt templates without a
//! thumbnail. The artwork is illustrative; generated output previews remain
//! entirely host supplied.

use std::sync::{Arc, OnceLock};

use gpui::{Image, ImageFormat};

use super::{GenerationKind, GenerationTemplate};

static EDITORIAL: OnceLock<Arc<Image>> = OnceLock::new();
static PRODUCT: OnceLock<Arc<Image>> = OnceLock::new();
static VIDEO_REVEAL: OnceLock<Arc<Image>> = OnceLock::new();
static VIDEO_NATURE: OnceLock<Arc<Image>> = OnceLock::new();
static AUDIO_VOICE: OnceLock<Arc<Image>> = OnceLock::new();
static AUDIO_MUSIC: OnceLock<Arc<Image>> = OnceLock::new();
static SVG_COMPASS: OnceLock<Arc<Image>> = OnceLock::new();
static SVG_MONOGRAM: OnceLock<Arc<Image>> = OnceLock::new();

fn cached_image(
    cache: &'static OnceLock<Arc<Image>>,
    format: ImageFormat,
    bytes: &'static [u8],
) -> Arc<Image> {
    cache
        .get_or_init(|| Arc::new(Image::from_bytes(format, bytes.to_vec())))
        .clone()
}

pub(super) fn for_template(template: &GenerationTemplate) -> Arc<Image> {
    match template.id.as_ref() {
        "image-editorial" => cached_image(
            &EDITORIAL,
            ImageFormat::Png,
            include_bytes!("template_art/editorial-portrait.png"),
        ),
        "image-product" => cached_image(
            &PRODUCT,
            ImageFormat::Png,
            include_bytes!("template_art/product-study.png"),
        ),
        "video-orbit" | "video-product" | "video-animate-product" => cached_image(
            &VIDEO_REVEAL,
            ImageFormat::Svg,
            include_bytes!("template_art/video-reveal.svg"),
        ),
        "video-atmosphere" | "video-nature" | "video-animate-landscape" => cached_image(
            &VIDEO_NATURE,
            ImageFormat::Svg,
            include_bytes!("template_art/video-nature.svg"),
        ),
        "audio-music" => cached_image(
            &AUDIO_MUSIC,
            ImageFormat::Svg,
            include_bytes!("template_art/audio-music.svg"),
        ),
        "svg-monogram" | "svg-mark" => cached_image(
            &SVG_MONOGRAM,
            ImageFormat::Svg,
            include_bytes!("template_art/svg-monogram.svg"),
        ),
        _ => match template.kind {
            GenerationKind::Image => cached_image(
                &EDITORIAL,
                ImageFormat::Png,
                include_bytes!("template_art/editorial-portrait.png"),
            ),
            GenerationKind::Video => cached_image(
                &VIDEO_NATURE,
                ImageFormat::Svg,
                include_bytes!("template_art/video-nature.svg"),
            ),
            GenerationKind::Audio => cached_image(
                &AUDIO_VOICE,
                ImageFormat::Svg,
                include_bytes!("template_art/audio-voice.svg"),
            ),
            GenerationKind::Svg => cached_image(
                &SVG_COMPASS,
                ImageFormat::Svg,
                include_bytes!("template_art/svg-compass.svg"),
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_previews_are_distinct_and_reused() {
        let editorial =
            GenerationTemplate::new("image-editorial", "Editorial", GenerationKind::Image, "");
        let product =
            GenerationTemplate::new("image-product", "Product", GenerationKind::Image, "");
        let first = for_template(&editorial);
        assert!(Arc::ptr_eq(&first, &for_template(&editorial)));
        assert!(!Arc::ptr_eq(&first, &for_template(&product)));
    }
}
