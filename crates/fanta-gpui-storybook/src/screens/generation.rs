//! Mock host for the four generation screens.
//!
//! The reusable screen receives a catalog and gallery snapshot and emits
//! typed requests. This story owns sample results and echoes accepted
//! selection and generation requests back into the screen.

use crate::*;
use std::{collections::HashMap, sync::Arc};

use super::generation_media::{GenerationMediaClip, GenerationMediaView};

use fanta_gpui::generation::{
    GenerationAction, GenerationChoice, GenerationKind, GenerationModel, GenerationOptionGroup,
    GenerationOutput, GenerationOutputStatus, GenerationRecipe, GenerationScreen, GenerationSource,
    GenerationTemplate, GenerationViewData, GenerationVoiceReference,
};
use gpui::{Image, ImageFormat};

const EDITORIAL_PORTRAIT: &[u8] = include_bytes!("../../assets/generation/editorial-portrait.png");
const PRODUCT_STUDY: &[u8] = include_bytes!("../../assets/generation/product-study.png");
const VIDEO_NATURE: &[u8] = include_bytes!("../../assets/generation/video-nature.png");
const SVG_COMPASS: &[u8] = include_bytes!("../../assets/generation/svg-compass.svg");
const SVG_MONOGRAM: &[u8] = include_bytes!("../../assets/generation/svg-monogram.svg");
const AUDIO_VOICE: &[u8] = include_bytes!("../../assets/generation/audio-voice.svg");
const AUDIO_MUSIC: &[u8] = include_bytes!("../../assets/generation/audio-music.svg");
const QWEN_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/qwen.png");
const Z_IMAGE_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/tongyi-mai.jpg");
const BFL_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/black-forest-labs-white.png");
const LTX_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/ltx.svg");
const WAN_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/wan.png");
const RESEMBLE_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/resemble-ai.png");
const HEXGRAD_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/hexgrad.png");
const ACE_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/ace-step.jpg");
const STARVECTOR_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/starvector.jpg");
const CLAUDE_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/claude.png");
const RECRAFT_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/recraft.png");
const XAI_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/xai.png");
const SEED_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/bytedance-seed.png");
const KLING_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/kling.png");
const GOOGLE_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/google.png");
const MINIMAX_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/minimax.png");
const OPENAI_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/openai.svg");
const QUIVERAI_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/quiverai.png");
const ELEVENLABS_LOGO: &[u8] = include_bytes!("../../assets/generation/logos/elevenlabs.png");

fn brand(
    model: GenerationModel,
    provider: &'static str,
    logo_key: &'static str,
    format: ImageFormat,
    bytes: &[u8],
    credits: Option<u32>,
) -> GenerationModel {
    let model = model.provider(provider).logo(
        logo_key,
        Arc::new(Image::from_bytes(format, bytes.to_vec())),
    );
    if let Some(credits) = credits {
        model.credits_per_output(credits)
    } else {
        model.credit_hint("Usage-based")
    }
}

fn preview(bytes: &[u8]) -> Arc<Image> {
    Arc::new(Image::from_bytes(ImageFormat::Svg, bytes.to_vec()))
}

fn photo_preview(bytes: &[u8]) -> Arc<Image> {
    Arc::new(Image::from_bytes(ImageFormat::Png, bytes.to_vec()))
}

fn video_controls(hd: bool) -> [GenerationOptionGroup; 3] {
    let (canvas, selected_canvas) = if hd {
        (
            [
                GenerationChoice::new("1280x720", "Landscape · 16:9"),
                GenerationChoice::new("720x1280", "Portrait · 9:16"),
                GenerationChoice::new("768x768", "Square · 1:1"),
            ],
            "1280x720",
        )
    } else {
        (
            [
                GenerationChoice::new("768x512", "Landscape · 3:2"),
                GenerationChoice::new("512x768", "Portrait · 2:3"),
                GenerationChoice::new("768x768", "Square · 1:1"),
            ],
            "768x512",
        )
    };
    [
        GenerationOptionGroup::new(
            "frames",
            "Duration",
            [
                GenerationChoice::new("17", "1 second"),
                GenerationChoice::new("33", "2 seconds"),
                GenerationChoice::new("49", "3 seconds"),
                GenerationChoice::new("65", "4 seconds"),
                GenerationChoice::new("81", "5 seconds"),
            ],
            "49",
        ),
        GenerationOptionGroup::new("canvas", "Frame size", canvas, selected_canvas),
        GenerationOptionGroup::new(
            "fps",
            "Frame rate",
            [
                GenerationChoice::new("16", "16 fps"),
                GenerationChoice::new("24", "24 fps"),
            ],
            "16",
        )
        .advanced(true),
    ]
}

fn gateway_image_controls() -> [GenerationOptionGroup; 1] {
    [GenerationOptionGroup::new(
        "size",
        "Canvas size",
        [
            GenerationChoice::new("1024x1024", "Square · 1024 × 1024"),
            GenerationChoice::new("1536x1024", "Landscape · 1536 × 1024"),
            GenerationChoice::new("1024x1536", "Portrait · 1024 × 1536"),
        ],
        "1024x1024",
    )]
}

fn replicate_image_controls() -> [GenerationOptionGroup; 2] {
    [
        GenerationOptionGroup::new(
            "size",
            "Image size",
            [
                GenerationChoice::new("2K", "2K"),
                GenerationChoice::new("3K", "3K"),
            ],
            "2K",
        ),
        GenerationOptionGroup::new(
            "aspect_ratio",
            "Aspect ratio",
            [
                GenerationChoice::new("1:1", "Square · 1:1"),
                GenerationChoice::new("16:9", "Landscape · 16:9"),
                GenerationChoice::new("9:16", "Portrait · 9:16"),
                GenerationChoice::new("4:3", "Classic · 4:3"),
                GenerationChoice::new("3:2", "Photo · 3:2"),
            ],
            "1:1",
        )
        .advanced(true),
    ]
}

fn gateway_video_controls(
    durations: &[u32],
    resolutions: &[&str],
    aspect_ratios: &[&str],
    audio: bool,
    mode: bool,
) -> Vec<GenerationOptionGroup> {
    let mut groups = vec![
        GenerationOptionGroup::new(
            "duration",
            "Duration",
            durations
                .iter()
                .map(|seconds| GenerationChoice::new(seconds.to_string(), format!("{seconds} s"))),
            durations[0].to_string(),
        ),
        GenerationOptionGroup::new(
            "resolution",
            "Resolution",
            resolutions
                .iter()
                .map(|resolution| GenerationChoice::new(*resolution, *resolution)),
            resolutions[0],
        ),
        GenerationOptionGroup::new(
            "aspect_ratio",
            "Aspect ratio",
            aspect_ratios
                .iter()
                .map(|ratio| GenerationChoice::new(*ratio, *ratio)),
            aspect_ratios[0],
        )
        .advanced(true),
    ];
    if audio {
        groups.push(
            GenerationOptionGroup::new(
                "generate_audio",
                "Generate audio",
                [
                    GenerationChoice::new("false", "Off"),
                    GenerationChoice::new("true", "On"),
                ],
                "false",
            )
            .advanced(true),
        );
    }
    if mode {
        groups.push(
            GenerationOptionGroup::new(
                "mode",
                "Quality mode",
                [
                    GenerationChoice::new("std", "Standard"),
                    GenerationChoice::new("pro", "Pro"),
                ],
                "std",
            )
            .advanced(true),
        );
    }
    groups
}

fn gateway_speech_controls(voices: &[&str]) -> [GenerationOptionGroup; 3] {
    [
        GenerationOptionGroup::new(
            "voice",
            "Voice",
            voices
                .iter()
                .map(|voice| GenerationChoice::new(*voice, *voice)),
            voices[0],
        ),
        GenerationOptionGroup::new(
            "output_format",
            "Audio format",
            [
                GenerationChoice::new("mp3", "MP3"),
                GenerationChoice::new("wav", "WAV"),
            ],
            "mp3",
        )
        .advanced(true),
        GenerationOptionGroup::new(
            "speed",
            "Pace",
            [
                GenerationChoice::new("0.75", "Relaxed"),
                GenerationChoice::new("1", "Natural"),
                GenerationChoice::new("1.25", "Brisk"),
                GenerationChoice::new("1.5", "Fast"),
            ],
            "1",
        )
        .advanced(true),
    ]
}

fn replicate_ltx_controls() -> Vec<GenerationOptionGroup> {
    let mut groups = vec![
        GenerationOptionGroup::new(
            "duration",
            "Duration",
            [
                GenerationChoice::new("6", "6 s"),
                GenerationChoice::new("8", "8 s"),
                GenerationChoice::new("10", "10 s"),
            ],
            "6",
        ),
        GenerationOptionGroup::new(
            "aspect_ratio",
            "Aspect ratio",
            [
                GenerationChoice::new("16:9", "Landscape · 16:9"),
                GenerationChoice::new("9:16", "Portrait · 9:16"),
            ],
            "16:9",
        ),
        GenerationOptionGroup::new(
            "fps",
            "Frame rate",
            [
                GenerationChoice::new("24", "24 fps"),
                GenerationChoice::new("25", "25 fps"),
                GenerationChoice::new("48", "48 fps"),
                GenerationChoice::new("50", "50 fps"),
            ],
            "25",
        )
        .advanced(true),
        GenerationOptionGroup::new(
            "camera_motion",
            "Camera motion",
            [
                GenerationChoice::new("none", "Automatic"),
                GenerationChoice::new("static", "Static"),
                GenerationChoice::new("dolly_in", "Dolly in"),
                GenerationChoice::new("dolly_out", "Dolly out"),
                GenerationChoice::new("dolly_left", "Dolly left"),
                GenerationChoice::new("dolly_right", "Dolly right"),
                GenerationChoice::new("jib_up", "Jib up"),
                GenerationChoice::new("jib_down", "Jib down"),
                GenerationChoice::new("focus_shift", "Focus shift"),
            ],
            "none",
        )
        .advanced(true),
    ];
    groups.push(
        GenerationOptionGroup::new(
            "generate_audio",
            "Generate audio",
            [
                GenerationChoice::new("true", "On"),
                GenerationChoice::new("false", "Off"),
            ],
            "true",
        )
        .advanced(true),
    );
    groups
}

fn replicate_seedance_controls() -> Vec<GenerationOptionGroup> {
    vec![
        GenerationOptionGroup::new(
            "duration",
            "Duration",
            [
                GenerationChoice::new("5", "5 s"),
                GenerationChoice::new("10", "10 s"),
                GenerationChoice::new("15", "15 s"),
                GenerationChoice::new("30", "30 s"),
            ],
            "5",
        ),
        GenerationOptionGroup::new(
            "aspect_ratio",
            "Aspect ratio",
            [
                GenerationChoice::new("16:9", "Landscape · 16:9"),
                GenerationChoice::new("9:16", "Portrait · 9:16"),
                GenerationChoice::new("adaptive", "Adapt to source"),
            ],
            "16:9",
        ),
        GenerationOptionGroup::new(
            "generate_audio",
            "Generate audio",
            [
                GenerationChoice::new("true", "On"),
                GenerationChoice::new("false", "Off"),
            ],
            "true",
        )
        .advanced(true),
    ]
}

pub(crate) struct GenerationStory {
    pub(crate) screen: Entity<GenerationScreen>,
    pub(crate) view_data: GenerationViewData,
    media_players: HashMap<SharedString, Entity<GenerationMediaView>>,
    pub(crate) last_action: SharedString,
}

impl GenerationStory {
    fn new(kind: GenerationKind, window: &mut Window, cx: &mut Context<Storybook>) -> Self {
        let view_data = fixture(kind);
        let id = match kind {
            GenerationKind::Image => "storybook-generation-image",
            GenerationKind::Video => "storybook-generation-video",
            GenerationKind::Audio => "storybook-generation-audio",
            GenerationKind::Svg => "storybook-generation-svg",
        };
        let screen = cx.new(|cx| GenerationScreen::new(id, kind, view_data.clone(), window, cx));
        Self {
            screen,
            view_data,
            media_players: HashMap::new(),
            last_action: "Ready — choose a model, try a template, or browse creations".into(),
        }
    }

    pub(crate) fn handle_action(
        &mut self,
        action: &GenerationAction,
        window: &mut Window,
        cx: &mut Context<Storybook>,
    ) {
        self.last_action = format!("{action:?}").into();
        match action {
            GenerationAction::RecipeSelected(recipe) => {
                self.view_data.selected_recipe = Some(*recipe);
                self.view_data.selected_model_id = self
                    .view_data
                    .models
                    .iter()
                    .find(|model| model.recipe == *recipe)
                    .map(|model| model.id.clone());
                self.view_data.selected_template_id = None;
                if *recipe != GenerationRecipe::ImageVideo {
                    self.view_data.end_frame = None;
                }
            }
            GenerationAction::ModelSelected { id } => {
                self.view_data.selected_model_id = Some(id.clone());
                self.view_data.selected_recipe = self
                    .view_data
                    .models
                    .iter()
                    .find(|model| {
                        model.id == *id && Some(model.recipe) == self.view_data.selected_recipe
                    })
                    .or_else(|| self.view_data.models.iter().find(|model| model.id == *id))
                    .map(|model| model.recipe);
                self.view_data.selected_template_id = None;
            }
            GenerationAction::TemplateSelected { id } => {
                self.view_data.selected_template_id = Some(id.clone());
                if let Some(model_id) = self
                    .view_data
                    .templates
                    .iter()
                    .find(|template| template.id == *id)
                    .and_then(|template| template.model_id.clone())
                {
                    self.view_data.selected_model_id = Some(model_id);
                }
                self.view_data.selected_recipe = self
                    .view_data
                    .models
                    .iter()
                    .find(|model| self.view_data.selected_model_id.as_ref() == Some(&model.id))
                    .map(|model| model.recipe);
            }
            GenerationAction::OutputSelected { id } => {
                self.view_data.selected_output_id = Some(id.clone());
                self.ensure_media(id, cx);
            }
            GenerationAction::SourceCleared => {
                self.view_data.source = None;
                self.view_data.end_frame = None;
            }
            GenerationAction::SourceRequested => {
                self.view_data.source = Some(
                    GenerationSource::new("storybook-source-image", "Reference image.png")
                        .preview(photo_preview(PRODUCT_STUDY)),
                );
            }
            GenerationAction::EndFrameRequested => {
                self.view_data.end_frame = Some(
                    GenerationSource::new("storybook-end-frame", "Closing frame.png")
                        .preview(photo_preview(VIDEO_NATURE)),
                );
            }
            GenerationAction::EndFrameCleared => {
                self.view_data.end_frame = None;
            }
            GenerationAction::VoiceReferenceRequested => {
                self.view_data.voice_reference = Some(GenerationVoiceReference::new(
                    "storybook-voice-reference",
                    "Voice sample.wav",
                ));
                self.view_data.voice_consent_granted = false;
            }
            GenerationAction::VoiceReferenceCleared => {
                self.view_data.voice_reference = None;
                self.view_data.voice_consent_granted = false;
            }
            GenerationAction::VoiceConsentChanged(granted) => {
                self.view_data.voice_consent_granted = *granted;
            }
            GenerationAction::OptionSelected {
                model_id,
                key,
                value,
            } => {
                let model_index = self
                    .view_data
                    .models
                    .iter()
                    .position(|model| {
                        model.id == *model_id
                            && Some(model.recipe) == self.view_data.selected_recipe
                    })
                    .or_else(|| {
                        self.view_data
                            .models
                            .iter()
                            .position(|model| model.id == *model_id)
                    });
                if let Some(model) =
                    model_index.and_then(|index| self.view_data.models.get_mut(index))
                {
                    if let Some(group) = model
                        .option_groups
                        .iter_mut()
                        .find(|group| group.key == *key)
                    {
                        group.selected = value.clone();
                    }
                    if model.recipe.kind() == GenerationKind::Video && key == "fps" {
                        let fps = value.parse::<f32>().unwrap_or(16.);
                        if let Some(frames) = model
                            .option_groups
                            .iter_mut()
                            .find(|group| group.key == "frames")
                        {
                            for choice in &mut frames.choices {
                                let count = choice.value.parse::<f32>().unwrap_or(17.);
                                choice.label = format!("{:.1} s", count / fps).into();
                            }
                        }
                    }
                    if model.recipe == GenerationRecipe::Vectorize && key == "mode" {
                        model.supports_prompt = value == "line_art";
                        model.option_groups.retain(|group| group.key != "detail");
                        if value == "flat_color" {
                            model.option_groups.push(vector_detail_controls());
                        }
                    }
                }
            }
            GenerationAction::GenerateRequested(submission) => {
                let output_id: SharedString =
                    format!("storybook-output-{}", self.view_data.outputs.len() + 1).into();
                let title = if submission.prompt.trim().is_empty() {
                    "Untitled creation".to_owned()
                } else {
                    submission.prompt.chars().take(56).collect()
                };
                let mut output = GenerationOutput::new(
                    output_id.clone(),
                    submission.kind,
                    title,
                    submission.model_id.clone(),
                    GenerationOutputStatus::Succeeded,
                )
                .prompt(submission.prompt.clone())
                .created_at("Just now")
                .detail(submission.recipe.label());
                output.preview = match submission.kind {
                    GenerationKind::Image => Some(photo_preview(EDITORIAL_PORTRAIT)),
                    GenerationKind::Video => Some(photo_preview(VIDEO_NATURE)),
                    GenerationKind::Audio => {
                        Some(preview(if submission.recipe == GenerationRecipe::Music {
                            AUDIO_MUSIC
                        } else {
                            AUDIO_VOICE
                        }))
                    }
                    GenerationKind::Svg => Some(preview(SVG_COMPASS)),
                };
                self.view_data.outputs.insert(0, output);
                self.view_data.selected_output_id = Some(output_id);
            }
            GenerationAction::PlayRequested { id } => {
                self.ensure_media(id, cx);
                if let Some(player) = self.media_players.get(id) {
                    player.update(cx, |player, cx| player.play(cx));
                }
            }
            GenerationAction::PreviewClosed { id } => {
                if let Some(player) = self.media_players.get(id) {
                    player.update(cx, |player, cx| player.stop(cx));
                }
            }
            GenerationAction::RefreshRequested => {
                self.view_data.error = None;
            }
            GenerationAction::LoadMoreRequested => {
                self.view_data.has_more = false;
            }
            GenerationAction::KindSelected(_)
            | GenerationAction::DraftChanged
            | GenerationAction::DownloadRequested { .. }
            | GenerationAction::ReusePromptRequested { .. } => {}
        }
        self.screen.update(cx, |screen, cx| {
            screen.set_view_data(self.view_data.clone(), window, cx)
        });
        cx.notify();
    }

    fn ensure_media(&mut self, id: &SharedString, cx: &mut Context<Storybook>) {
        if self.media_players.contains_key(id) {
            return;
        }
        let Some(output) = self
            .view_data
            .outputs
            .iter_mut()
            .find(|output| &output.id == id)
        else {
            return;
        };
        let Some(clip) = output_clip(output) else {
            return;
        };
        let player = cx.new(|cx| GenerationMediaView::new(clip, cx));
        output.playback_view = Some(player.clone().into());
        self.media_players.insert(id.clone(), player);
    }
}

fn output_clip(output: &GenerationOutput) -> Option<GenerationMediaClip> {
    match (output.kind, output.model_id.as_ref()) {
        (GenerationKind::Video, "fanta-video-hd-1") => Some(GenerationMediaClip::ProductVideo),
        (GenerationKind::Video, _) => Some(GenerationMediaClip::NatureVideo),
        (GenerationKind::Audio, "fanta-music-1") => Some(GenerationMediaClip::Music),
        (GenerationKind::Audio, _) => Some(GenerationMediaClip::Voice),
        _ => None,
    }
}

fn vector_detail_controls() -> GenerationOptionGroup {
    GenerationOptionGroup::new(
        "detail",
        "Color detail",
        [
            GenerationChoice::new("low", "Simplified"),
            GenerationChoice::new("medium", "Standard"),
        ],
        "medium",
    )
    .advanced(true)
}

pub(crate) struct GenerationStories {
    pub(crate) image: GenerationStory,
    pub(crate) video: GenerationStory,
    pub(crate) audio: GenerationStory,
    pub(crate) svg: GenerationStory,
}

impl GenerationStories {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Storybook>) -> Self {
        Self {
            image: GenerationStory::new(GenerationKind::Image, window, cx),
            video: GenerationStory::new(GenerationKind::Video, window, cx),
            audio: GenerationStory::new(GenerationKind::Audio, window, cx),
            svg: GenerationStory::new(GenerationKind::Svg, window, cx),
        }
    }
}

impl Storybook {
    pub(crate) fn handle_generation_action(
        &mut self,
        current: GenerationKind,
        action: &GenerationAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let story = match current {
            GenerationKind::Image => &mut self.generation_stories.image,
            GenerationKind::Video => &mut self.generation_stories.video,
            GenerationKind::Audio => &mut self.generation_stories.audio,
            GenerationKind::Svg => &mut self.generation_stories.svg,
        };
        story.handle_action(action, window, cx);
        if let GenerationAction::KindSelected(kind) = action {
            let target = match kind {
                GenerationKind::Image => StoryKind::GenerationImage,
                GenerationKind::Video => StoryKind::GenerationVideo,
                GenerationKind::Audio => StoryKind::GenerationAudio,
                GenerationKind::Svg => StoryKind::GenerationSvg,
            };
            if let Some(open_story) = self
                .story_windows
                .get_mut(&window.window_handle().window_id())
            {
                *open_story = target;
                self.focus_story(target, window, cx);
                cx.notify();
            } else {
                self.activate_gallery_story(target, window, cx);
            }
        }
    }
}

fn fixture(kind: GenerationKind) -> GenerationViewData {
    let (models, templates, outputs, selected_model_id) = match kind {
        GenerationKind::Image => (
            vec![
                brand(GenerationModel::new("fanta-image-1", "Fanta Image", GenerationRecipe::TextImage)
                    .description("Detailed image generation")
                    .supports_negative(true)
                    .supports_seed(true)
                    .option_groups([
                        GenerationOptionGroup::new(
                            "size",
                            "Size",
                            [
                                GenerationChoice::new("1024x1024", "Square · 1024 × 1024"),
                                GenerationChoice::new("1280x768", "Landscape · 1280 × 768"),
                                GenerationChoice::new("768x1280", "Portrait · 768 × 1280"),
                                GenerationChoice::new("2048x2048", "Large · 2048 × 2048"),
                            ],
                            "1024x1024",
                        ),
                        GenerationOptionGroup::new(
                            "steps",
                            "Steps",
                            [
                                GenerationChoice::new("10", "10 · Fast"),
                                GenerationChoice::new("30", "30 · Balanced"),
                                GenerationChoice::new("50", "50 · Detailed"),
                            ],
                            "30",
                        ).advanced(true),
                        GenerationOptionGroup::new(
                            "guidance",
                            "Prompt strength",
                            [
                                GenerationChoice::new("1", "1 · Loose"),
                                GenerationChoice::new("4", "4 · Balanced"),
                                GenerationChoice::new("10", "10 · Strong"),
                            ],
                            "4",
                        ).advanced(true),
                        GenerationOptionGroup::new(
                            "enhance_prompt",
                            "Prompt enhancement",
                            [
                                GenerationChoice::new("false", "Off"),
                                GenerationChoice::new("true", "On"),
                            ],
                            "false",
                        ).advanced(true),
                    ]), "Qwen", "qwen", ImageFormat::Png, QWEN_LOGO, Some(1)),
                brand(GenerationModel::new(
                    "fanta-image-fast-1",
                    "Fanta Image Fast",
                    GenerationRecipe::TextImage,
                )
                .description("Fast image concepts"), "Tongyi-MAI", "z-image", ImageFormat::Jpeg, Z_IMAGE_LOGO, Some(1)),
                brand(GenerationModel::new("flux-schnell", "FLUX Schnell", GenerationRecipe::TextImage)
                    .description("Rapid image drafts"), "Black Forest Labs", "bfl", ImageFormat::Png, BFL_LOGO, Some(1)),
                brand(GenerationModel::new("gateway-recraft-v4.1-utility", "Recraft V4.1 Utility", GenerationRecipe::TextImage)
                    .description("Fast design concepts").option_groups(gateway_image_controls()), "Recraft", "recraft", ImageFormat::Png, RECRAFT_LOGO, None),
                brand(GenerationModel::new("gateway-recraft-v4.1-pro", "Recraft V4.1 Pro", GenerationRecipe::TextImage)
                    .description("High fidelity design images").option_groups(gateway_image_controls()), "Recraft", "recraft", ImageFormat::Png, RECRAFT_LOGO, None),
                brand(GenerationModel::new("gateway-grok-imagine-image", "Grok Imagine Image", GenerationRecipe::TextImage)
                    .description("Expressive image generation").option_groups(gateway_image_controls()), "SpaceXAI", "xai", ImageFormat::Png, XAI_LOGO, None),
                brand(GenerationModel::new("gateway-seedream-4.5", "Seedream 4.5", GenerationRecipe::TextImage)
                    .description("Detailed visual compositions").option_groups(gateway_image_controls()), "ByteDance", "seed", ImageFormat::Png, SEED_LOGO, None),
                brand(GenerationModel::new("gateway-flux-pro-1.1", "FLUX 1.1 Pro", GenerationRecipe::TextImage)
                    .description("Photorealistic image generation").option_groups(gateway_image_controls()), "Black Forest Labs", "bfl", ImageFormat::Png, BFL_LOGO, None),
                brand(GenerationModel::new("replicate-seedream-5-lite", "Seedream 5 Lite", GenerationRecipe::TextImage)
                    .description("Flexible high-resolution imagery")
                    .option_groups(replicate_image_controls()),
                    "ByteDance", "seed", ImageFormat::Png, SEED_LOGO, None),
                brand(GenerationModel::new("replicate-recraft-v4.1", "Recraft V4.1", GenerationRecipe::TextImage)
                    .description("Design-focused image generation")
                    .option_groups([GenerationOptionGroup::new("aspect_ratio", "Aspect ratio", [
                        GenerationChoice::new("Not set", "Automatic"),
                        GenerationChoice::new("1:1", "1:1"),
                        GenerationChoice::new("16:9", "16:9"),
                        GenerationChoice::new("9:16", "9:16"),
                        GenerationChoice::new("4:3", "4:3"),
                        GenerationChoice::new("3:2", "3:2"),
                    ], "Not set")]),
                    "Recraft", "recraft", ImageFormat::Png, RECRAFT_LOGO, None),
                brand(GenerationModel::new("replicate-recraft-v4.1-pro", "Recraft V4.1 Pro", GenerationRecipe::TextImage)
                    .description("Large-format design imagery")
                    .option_groups([GenerationOptionGroup::new("aspect_ratio", "Aspect ratio", [
                        GenerationChoice::new("Not set", "Automatic"),
                        GenerationChoice::new("1:1", "1:1"),
                        GenerationChoice::new("16:9", "16:9"),
                        GenerationChoice::new("9:16", "9:16"),
                        GenerationChoice::new("4:3", "4:3"),
                        GenerationChoice::new("3:2", "3:2"),
                    ], "Not set")]),
                    "Recraft", "recraft", ImageFormat::Png, RECRAFT_LOGO, None),
            ],
            vec![
                GenerationTemplate::new(
                    "image-editorial",
                    "Editorial portrait",
                    kind,
                    "An editorial portrait with directional window light and quiet natural tones",
                )
                .description("Portrait · soft directional light"),
                GenerationTemplate::new(
                    "image-product",
                    "Product study",
                    kind,
                    "A refined product still life with soft studio lighting and clear material detail",
                )
                .description("Product · refined material study"),
                GenerationTemplate::new(
                    "image-landscape",
                    "Atmospheric landscape",
                    kind,
                    "A cinematic volcanic coastline at dawn with a waterfall, drifting mist and a warm horizon",
                )
                .description("Landscape · atmospheric color")
                .preview(photo_preview(VIDEO_NATURE)),
            ],
            vec![
                GenerationOutput::new(
                    "image-result-1",
                    kind,
                    "Morning light — editorial study",
                    "fanta-image-1",
                    GenerationOutputStatus::Succeeded,
                )
                .prompt("An editorial portrait with directional window light and quiet natural tones")
                .created_at("Today")
                .detail("1024 × 1024 · Fanta Image")
                .preview(photo_preview(EDITORIAL_PORTRAIT)),
                GenerationOutput::new(
                    "image-result-2",
                    kind,
                    "Amber vessel — product study",
                    "fanta-image-fast-1",
                    GenerationOutputStatus::Succeeded,
                )
                .prompt("A refined product still life with soft studio lighting")
                .created_at("Yesterday")
                .detail("Fast concept · Fanta Image Fast")
                .preview(photo_preview(PRODUCT_STUDY)),
            ],
            "fanta-image-1",
        ),
        GenerationKind::Video => (
            vec![
                brand(GenerationModel::new("fanta-video-1", "Fanta Video", GenerationRecipe::TextVideo)
                    .description("Motion from a text prompt")
                    .option_groups(video_controls(false)), "Lightricks", "ltx", ImageFormat::Svg, LTX_LOGO, Some(4)),
                brand(GenerationModel::new(
                    "fanta-video-hd-1",
                    "Fanta Video HD",
                    GenerationRecipe::TextVideo,
                )
                .description("Detailed cinematic motion")
                .option_groups(video_controls(true)), "Wan", "wan", ImageFormat::Png, WAN_LOGO, Some(126)),
                brand(GenerationModel::new(
                    "fanta-animate-1",
                    "Fanta Animate",
                    GenerationRecipe::ImageVideo,
                )
                .description("Animate a source image")
                .requires_source(true)
                .option_groups(video_controls(true)), "Wan", "wan", ImageFormat::Png, WAN_LOGO, Some(126)),
                brand(GenerationModel::new("gateway-wan-3-video", "Wan 3.0 Video", GenerationRecipe::TextVideo)
                    .description("Flexible cinematic video")
                    .option_groups(gateway_video_controls(&[5, 10, 15], &["854x480", "1280x720", "1920x1080"], &["16:9", "9:16", "1:1"], false, false)),
                    "Alibaba", "wan", ImageFormat::Png, WAN_LOGO, None),
                brand(GenerationModel::new("gateway-wan-3-prime", "Wan 3.0 Prime", GenerationRecipe::TextVideo)
                    .description("High fidelity Wan motion")
                    .option_groups(gateway_video_controls(&[5, 10, 15], &["854x480", "1280x720", "1920x1080"], &["16:9", "9:16", "1:1"], false, false)),
                    "Alibaba", "wan", ImageFormat::Png, WAN_LOGO, None),
                brand(GenerationModel::new("gateway-kling-3-t2v", "Kling 3.0 Text to Video", GenerationRecipe::TextVideo)
                    .description("Cinematic motion with optional sound")
                    .option_groups(gateway_video_controls(&[5, 10, 15], &["1280x720", "1920x1080"], &["16:9", "9:16", "1:1"], true, true)),
                    "Kling AI", "kling", ImageFormat::Jpeg, KLING_LOGO, None),
                brand(GenerationModel::new("gateway-kling-3-i2v", "Kling 3.0 Image to Video", GenerationRecipe::ImageVideo)
                    .description("Animate start and end frames")
                    .requires_source(true)
                    .supports_end_frame(true)
                    .option_groups(gateway_video_controls(&[5, 10, 15], &["1280x720", "1920x1080"], &["16:9", "9:16", "1:1"], true, true)),
                    "Kling AI", "kling", ImageFormat::Jpeg, KLING_LOGO, None),
                brand(GenerationModel::new("gateway-veo-3.1-fast-studio", "Veo 3.1 Fast", GenerationRecipe::TextVideo)
                    .description("Fast cinematic video with sound")
                    .option_groups(gateway_video_controls(&[4, 6, 8], &["1280x720", "1920x1080", "3840x2160"], &["16:9", "9:16"], true, false)),
                    "Google", "google", ImageFormat::Png, GOOGLE_LOGO, None),
                brand(GenerationModel::new("gateway-minimax-h3-max", "MiniMax H3 Max", GenerationRecipe::TextVideo)
                    .description("Detailed cinematic storytelling")
                    .option_groups(gateway_video_controls(&[5, 10, 15], &["480p", "768p"], &["21:9", "16:9", "4:3", "1:1", "3:4", "9:16"], false, false)),
                    "MiniMax", "minimax", ImageFormat::Jpeg, MINIMAX_LOGO, None),
                brand(GenerationModel::new("gateway-grok-video-1.5", "Grok Imagine Video 1.5", GenerationRecipe::TextVideo)
                    .description("Stylized motion from a prompt")
                    .option_groups(gateway_video_controls(&[5, 10, 15], &["854x480", "1280x720", "1920x1080"], &["16:9", "9:16", "1:1", "4:3", "3:4"], false, false)),
                    "SpaceXAI", "xai", ImageFormat::Png, XAI_LOGO, None),
                brand(GenerationModel::new("gateway-wan-3-video", "Wan 3.0 Video", GenerationRecipe::ImageVideo)
                    .description("Animate a source image")
                    .requires_source(true)
                    .option_groups(gateway_video_controls(&[5, 10, 15], &["854x480", "1280x720", "1920x1080"], &["16:9", "9:16", "1:1"], false, false)),
                    "Alibaba", "wan", ImageFormat::Png, WAN_LOGO, None),
                brand(GenerationModel::new("gateway-wan-3-prime", "Wan 3.0 Prime", GenerationRecipe::ImageVideo)
                    .description("High fidelity image animation")
                    .requires_source(true)
                    .option_groups(gateway_video_controls(&[5, 10, 15], &["854x480", "1280x720", "1920x1080"], &["16:9", "9:16", "1:1"], false, false)),
                    "Alibaba", "wan", ImageFormat::Png, WAN_LOGO, None),
                brand(GenerationModel::new("gateway-veo-3.1-fast-studio", "Veo 3.1 Fast", GenerationRecipe::ImageVideo)
                    .description("Animate frames with optional sound")
                    .requires_source(true)
                    .supports_end_frame(true)
                    .option_groups(gateway_video_controls(&[4, 6, 8], &["1280x720", "1920x1080", "3840x2160"], &["16:9", "9:16"], true, false)),
                    "Google", "google", ImageFormat::Png, GOOGLE_LOGO, None),
                brand(GenerationModel::new("gateway-minimax-h3-max", "MiniMax H3 Max", GenerationRecipe::ImageVideo)
                    .description("Animate frames with precise motion")
                    .requires_source(true)
                    .supports_end_frame(true)
                    .option_groups(gateway_video_controls(&[5, 10, 15], &["480p", "768p"], &["21:9", "16:9", "4:3", "1:1", "3:4", "9:16"], false, false)),
                    "MiniMax", "minimax", ImageFormat::Jpeg, MINIMAX_LOGO, None),
                brand(GenerationModel::new("replicate-wan-2.7-i2v", "Wan 2.7 Image to Video", GenerationRecipe::ImageVideo)
                    .description("Animate two key frames")
                    .requires_source(true)
                    .supports_end_frame(true)
                    .supports_negative(true)
                    .supports_seed(true)
                    .option_groups([
                        GenerationOptionGroup::new("duration", "Duration", [
                            GenerationChoice::new("2", "2 s"),
                            GenerationChoice::new("5", "5 s"),
                            GenerationChoice::new("10", "10 s"),
                            GenerationChoice::new("15", "15 s"),
                        ], "5"),
                        GenerationOptionGroup::new("enable_prompt_expansion", "Prompt expansion", [
                            GenerationChoice::new("true", "On"),
                            GenerationChoice::new("false", "Off"),
                        ], "true").advanced(true),
                    ]),
                    "Wan", "wan", ImageFormat::Png, WAN_LOGO, None),
                brand(GenerationModel::new("replicate-ltx-2.3-pro", "LTX 2.3 Pro", GenerationRecipe::TextVideo)
                    .description("Cinematic motion and camera controls")
                    .option_groups(replicate_ltx_controls()),
                    "Lightricks", "ltx", ImageFormat::Svg, LTX_LOGO, None),
                brand(GenerationModel::new("replicate-ltx-2.3-pro", "LTX 2.3 Pro", GenerationRecipe::ImageVideo)
                    .description("Animate start and end frames")
                    .requires_source(true)
                    .supports_end_frame(true)
                    .option_groups(replicate_ltx_controls()),
                    "Lightricks", "ltx", ImageFormat::Svg, LTX_LOGO, None),
                brand(GenerationModel::new("replicate-seedance-2.5", "Seedance 2.5", GenerationRecipe::TextVideo)
                    .description("Rich motion with optional audio")
                    .supports_seed(true)
                    .option_groups(replicate_seedance_controls()),
                    "ByteDance", "seed", ImageFormat::Png, SEED_LOGO, None),
                brand(GenerationModel::new("replicate-seedance-2.5", "Seedance 2.5", GenerationRecipe::ImageVideo)
                    .description("Animate an image sequence")
                    .requires_source(true)
                    .supports_end_frame(true)
                    .supports_seed(true)
                    .option_groups(replicate_seedance_controls()),
                    "ByteDance", "seed", ImageFormat::Png, SEED_LOGO, None),
            ],
            vec![
                GenerationTemplate::new(
                    "video-product",
                    "Product reveal",
                    kind,
                    "A slow cinematic camera move revealing a sculptural product in soft light",
                )
                .model("fanta-video-hd-1")
                .description("Text to video · cinematic reveal")
                .preview(photo_preview(PRODUCT_STUDY)),
                GenerationTemplate::new(
                    "video-nature",
                    "Nature in motion",
                    kind,
                    "A slow cinematic aerial move over a volcanic coastline and waterfall at dawn",
                )
                .model("fanta-video-1")
                .description("Text to video · atmospheric coast")
                .preview(photo_preview(VIDEO_NATURE)),
                GenerationTemplate::new(
                    "video-abstract",
                    "Abstract motion",
                    kind,
                    "Soft translucent shapes drift and fold in a restrained abstract composition",
                )
                .model("fanta-video-1")
                .description("Text to video · abstract motion")
                .preview(preview(SVG_MONOGRAM)),
                GenerationTemplate::new(
                    "video-animate-product",
                    "Animate a still life",
                    kind,
                    "A slow dolly move around the product with shifting warm highlights and subtle depth",
                )
                .model("fanta-animate-1")
                .description("Image to video · product motion")
                .preview(photo_preview(PRODUCT_STUDY)),
                GenerationTemplate::new(
                    "video-animate-landscape",
                    "Bring a landscape to life",
                    kind,
                    "The camera drifts forward as coastal mist and falling water move naturally",
                )
                .model("fanta-animate-1")
                .description("Image to video · atmospheric motion")
                .preview(photo_preview(VIDEO_NATURE)),
            ],
            vec![
                GenerationOutput::new(
                    "video-result-1",
                    kind,
                    "Sculptural product reveal",
                    "fanta-video-hd-1",
                    GenerationOutputStatus::Succeeded,
                )
                .prompt("A slow cinematic camera move revealing a sculptural product")
                .created_at("Today")
                .detail("Video · Fanta Video HD")
                .preview(photo_preview(PRODUCT_STUDY)),
                GenerationOutput::new(
                    "video-result-2",
                    kind,
                    "Coastal dawn in motion",
                    "fanta-video-1",
                    GenerationOutputStatus::Succeeded,
                )
                .prompt("A slow aerial move over a misty volcanic coastline at dawn")
                .created_at("Yesterday")
                .detail("Video · Fanta Video")
                .preview(photo_preview(VIDEO_NATURE)),
            ],
            "fanta-video-1",
        ),
        GenerationKind::Audio => (
            vec![
                brand(GenerationModel::new("fanta-voice-1", "Fanta Voice", GenerationRecipe::Speech)
                    .description("Expressive spoken audio")
                    .supports_voice_reference(true), "Resemble AI", "resemble", ImageFormat::Png, RESEMBLE_LOGO, Some(1)),
                brand(GenerationModel::new(
                    "fanta-voice-turbo-1",
                    "Fanta Voice Turbo",
                    GenerationRecipe::Speech,
                )
                .description("Fast speech generation")
                .supports_voice_reference(true), "Resemble AI", "resemble", ImageFormat::Png, RESEMBLE_LOGO, Some(1)),
                brand(GenerationModel::new(
                    "fanta-voice-fast-1",
                    "Fanta Voice Fast",
                    GenerationRecipe::Speech,
                )
                .description("Speech with pace control")
                .option_groups([GenerationOptionGroup::new(
                    "speed",
                    "Pace",
                    [
                        GenerationChoice::new("0.8", "Relaxed"),
                        GenerationChoice::new("1.0", "Natural"),
                        GenerationChoice::new("1.2", "Brisk"),
                    ],
                    "1.0",
                )]), "Hexgrad", "hexgrad", ImageFormat::Png, HEXGRAD_LOGO, Some(1)),
                brand(GenerationModel::new("fanta-music-1", "Fanta Music", GenerationRecipe::Music)
                    .description("Music from a prompt"), "ACE-Step", "ace", ImageFormat::Jpeg, ACE_LOGO, Some(4)),
                brand(GenerationModel::new("gateway-grok-tts", "Grok TTS", GenerationRecipe::Speech)
                    .description("Expressive spoken audio")
                    .option_groups(gateway_speech_controls(&["eve"])),
                    "SpaceXAI", "xai", ImageFormat::Png, XAI_LOGO, None),
                brand(GenerationModel::new("gateway-openai-tts-1", "OpenAI TTS 1", GenerationRecipe::Speech)
                    .description("Six distinct speaking voices")
                    .option_groups(gateway_speech_controls(&["alloy", "echo", "fable", "onyx", "nova", "shimmer"])),
                    "OpenAI", "openai", ImageFormat::Svg, OPENAI_LOGO, None),
                brand(GenerationModel::new("gateway-openai-tts-1-hd", "OpenAI TTS 1 HD", GenerationRecipe::Speech)
                    .description("Higher fidelity spoken audio")
                    .option_groups(gateway_speech_controls(&["alloy", "echo", "fable", "onyx", "nova", "shimmer"])),
                    "OpenAI", "openai", ImageFormat::Svg, OPENAI_LOGO, None),
                brand(GenerationModel::new("replicate-speech-2.8-hd", "MiniMax Speech 2.8 HD", GenerationRecipe::Speech)
                    .description("Expressive voices and emotional tone")
                    .option_groups([
                        GenerationOptionGroup::new("voice_id", "Voice", [
                            GenerationChoice::new("Wise_Woman", "Wise Woman"),
                            GenerationChoice::new("Friendly_Person", "Friendly Person"),
                            GenerationChoice::new("Deep_Voice_Man", "Deep Voice Man"),
                            GenerationChoice::new("Calm_Woman", "Calm Woman"),
                        ], "Wise_Woman"),
                        GenerationOptionGroup::new("emotion", "Emotion", [
                            GenerationChoice::new("auto", "Automatic"),
                            GenerationChoice::new("happy", "Happy"),
                            GenerationChoice::new("calm", "Calm"),
                            GenerationChoice::new("sad", "Sad"),
                            GenerationChoice::new("angry", "Angry"),
                            GenerationChoice::new("surprised", "Surprised"),
                        ], "auto").advanced(true),
                        GenerationOptionGroup::new("speed", "Pace", [
                            GenerationChoice::new("0.75", "Relaxed"),
                            GenerationChoice::new("1", "Natural"),
                            GenerationChoice::new("1.25", "Brisk"),
                            GenerationChoice::new("1.5", "Fast"),
                        ], "1").advanced(true),
                        GenerationOptionGroup::new("audio_format", "Audio format", [
                            GenerationChoice::new("mp3", "MP3"),
                            GenerationChoice::new("wav", "WAV"),
                        ], "mp3").advanced(true),
                    ]),
                    "MiniMax", "minimax", ImageFormat::Jpeg, MINIMAX_LOGO, None),
                brand(GenerationModel::new("replicate-eleven-music", "Eleven Music", GenerationRecipe::Music)
                    .description("Instrumental or vocal music")
                    .option_groups([
                        GenerationOptionGroup::new("music_length_ms", "Length", [
                            GenerationChoice::new("10000", "10 s"),
                            GenerationChoice::new("30000", "30 s"),
                            GenerationChoice::new("60000", "1 min"),
                            GenerationChoice::new("120000", "2 min"),
                        ], "30000"),
                        GenerationOptionGroup::new("force_instrumental", "Vocals", [
                            GenerationChoice::new("true", "Instrumental"),
                            GenerationChoice::new("false", "Allow vocals"),
                        ], "true").advanced(true),
                        GenerationOptionGroup::new("output_format", "Audio format", [
                            GenerationChoice::new("mp3_standard", "MP3"),
                            GenerationChoice::new("mp3_high_quality", "MP3 HQ"),
                            GenerationChoice::new("wav_cd_quality", "WAV"),
                        ], "mp3_standard").advanced(true),
                    ]),
                    "ElevenLabs", "elevenlabs", ImageFormat::Png, ELEVENLABS_LOGO, None),
            ],
            vec![
                GenerationTemplate::new(
                    "audio-narration",
                    "Warm narration",
                    kind,
                    "Welcome to a quieter way to create. Let the idea take shape, one detail at a time.",
                )
                .model("fanta-voice-1")
                .description("Speech · warm voice")
                .preview(preview(AUDIO_VOICE)),
                GenerationTemplate::new(
                    "audio-podcast",
                    "Podcast introduction",
                    kind,
                    "Today we explore the small decisions behind enduring creative work.",
                )
                .model("fanta-voice-1")
                .description("Speech · podcast introduction")
                .preview(preview(AUDIO_VOICE)),
                GenerationTemplate::new(
                    "audio-music",
                    "Ambient score",
                    kind,
                    "A warm instrumental ambient score with soft piano, subtle texture, and no vocals",
                )
                .model("fanta-music-1")
                .description("Music · ambient instrumental")
                .preview(preview(AUDIO_MUSIC)),
            ],
            vec![
                GenerationOutput::new(
                    "audio-result-1",
                    kind,
                    "Warm narration",
                    "fanta-voice-1",
                    GenerationOutputStatus::Succeeded,
                )
                .prompt("Welcome to a quieter way to create")
                .created_at("Today")
                .detail("Speech · Fanta Voice")
                .preview(preview(AUDIO_VOICE)),
                GenerationOutput::new(
                    "audio-result-2",
                    kind,
                    "Ambient score",
                    "fanta-music-1",
                    GenerationOutputStatus::Succeeded,
                )
                .prompt("A warm instrumental ambient score with soft piano")
                .created_at("Yesterday")
                .detail("Music · Fanta Music")
                .preview(preview(AUDIO_MUSIC)),
            ],
            "fanta-voice-1",
        ),
        GenerationKind::Svg => (
            vec![
                brand(GenerationModel::new(
                    "claude-sonnet-5",
                    "Claude Sonnet 5",
                    GenerationRecipe::PromptSvg,
                )
                .description("SVG artwork from a prompt"), "Anthropic", "claude", ImageFormat::Png, CLAUDE_LOGO, None),
                brand(GenerationModel::new(
                    "claude-haiku-4-5",
                    "Claude Haiku 4.5",
                    GenerationRecipe::PromptSvg,
                )
                .description("Fast SVG concepts"), "Anthropic", "claude", ImageFormat::Png, CLAUDE_LOGO, None),
                brand(GenerationModel::new("gateway-arrow-1.1-svg", "Arrow 1.1 SVG", GenerationRecipe::PromptSvg)
                    .description("Vector artwork from a prompt"),
                    "QuiverAI", "quiverai", ImageFormat::Png, QUIVERAI_LOGO, None),
                brand(GenerationModel::new("replicate-recraft-v4.1-svg", "Recraft V4.1 SVG", GenerationRecipe::PromptSvg)
                    .description("Design-ready vector artwork")
                    .option_groups([GenerationOptionGroup::new("aspect_ratio", "Aspect ratio", [
                        GenerationChoice::new("Not set", "Automatic"),
                        GenerationChoice::new("1:1", "1:1"),
                        GenerationChoice::new("16:9", "16:9"),
                        GenerationChoice::new("9:16", "9:16"),
                        GenerationChoice::new("4:3", "4:3"),
                        GenerationChoice::new("3:2", "3:2"),
                    ], "Not set")]),
                    "Recraft", "recraft", ImageFormat::Png, RECRAFT_LOGO, None),
                brand(GenerationModel::new("replicate-recraft-v4.1-pro-svg", "Recraft V4.1 Pro SVG", GenerationRecipe::PromptSvg)
                    .description("High fidelity vector artwork")
                    .option_groups([GenerationOptionGroup::new("aspect_ratio", "Aspect ratio", [
                        GenerationChoice::new("Not set", "Automatic"),
                        GenerationChoice::new("1:1", "1:1"),
                        GenerationChoice::new("16:9", "16:9"),
                        GenerationChoice::new("9:16", "9:16"),
                        GenerationChoice::new("4:3", "4:3"),
                        GenerationChoice::new("3:2", "3:2"),
                    ], "Not set")]),
                    "Recraft", "recraft", ImageFormat::Png, RECRAFT_LOGO, None),
                brand(GenerationModel::new("fanta-svg-1", "Fanta SVG", GenerationRecipe::ImageSvg)
                    .description("Create SVG from an image")
                    .requires_source(true), "StarVector", "starvector", ImageFormat::Jpeg, STARVECTOR_LOGO, Some(1)),
                brand(GenerationModel::new(
                    "fanta-vectorize-1",
                    "Fanta Vectorize",
                    GenerationRecipe::Vectorize,
                )
                .description("Trace source artwork as SVG")
                .requires_source(true)
                .supports_prompt(true)
                .option_groups([
                    GenerationOptionGroup::new(
                        "mode",
                        "Vector style",
                        [
                            GenerationChoice::new("line_art", "Line art"),
                            GenerationChoice::new("flat_color", "Flat color"),
                            GenerationChoice::new("trace", "Trace"),
                        ],
                        "line_art",
                    ),
                ]), "Black Forest Labs", "bfl", ImageFormat::Png, BFL_LOGO, Some(1)),
            ],
            vec![
                GenerationTemplate::new(
                    "svg-icon",
                    "Interface icon",
                    kind,
                    "A clean geometric compass icon with balanced strokes and simple SVG paths",
                )
                .model("claude-sonnet-5")
                .description("Prompt to SVG · interface icon")
                .preview(preview(SVG_COMPASS)),
                GenerationTemplate::new(
                    "svg-mark",
                    "Abstract mark",
                    kind,
                    "A distinctive abstract monogram made of a few precise vector shapes",
                )
                .model("claude-sonnet-5")
                .description("Prompt to SVG · identity mark")
                .preview(preview(SVG_MONOGRAM)),
                GenerationTemplate::new(
                    "svg-pattern",
                    "Seamless pattern",
                    kind,
                    "A restrained repeating geometric pattern in an editable SVG",
                )
                .model("claude-sonnet-5")
                .description("Prompt to SVG · graphic pattern")
                .preview(preview(SVG_COMPASS)),
                GenerationTemplate::new(
                    "svg-from-image",
                    "Vector from image",
                    kind,
                    "",
                )
                .model("fanta-svg-1")
                .description("Image to SVG · choose a source")
                .preview(preview(SVG_COMPASS)),
                GenerationTemplate::new(
                    "svg-line-art",
                    "Expressive line art",
                    kind,
                    "Clean continuous black line art with confident, deliberate strokes",
                )
                .model("fanta-vectorize-1")
                .description("Vectorize · line art")
                .preview(preview(SVG_MONOGRAM)),
            ],
            vec![
                GenerationOutput::new(
                    "svg-result-1",
                    kind,
                    "Compass icon exploration",
                    "claude-sonnet-5",
                    GenerationOutputStatus::Succeeded,
                )
                .prompt("A clean geometric compass icon with balanced strokes")
                .created_at("Today")
                .detail("Editable SVG · Claude Sonnet 5")
                .preview(preview(SVG_COMPASS)),
                GenerationOutput::new(
                    "svg-result-2",
                    kind,
                    "Geometric monogram",
                    "claude-haiku-4-5",
                    GenerationOutputStatus::Succeeded,
                )
                .prompt("A distinctive abstract monogram made of precise shapes")
                .created_at("Yesterday")
                .detail("Editable SVG · Claude Haiku 4.5")
                .preview(preview(SVG_MONOGRAM)),
            ],
            "claude-sonnet-5",
        ),
    };
    let selected_output_id = outputs.first().map(|output| output.id.clone());
    let selected_recipe = models
        .iter()
        .find(|model| model.id.as_ref() == selected_model_id)
        .map(|model| model.recipe);
    GenerationViewData {
        models,
        templates,
        outputs,
        selected_model_id: Some(selected_model_id.into()),
        selected_recipe,
        selected_template_id: None,
        selected_output_id,
        source: None,
        end_frame: None,
        voice_reference: None,
        voice_consent_granted: false,
        busy: false,
        error: None,
        has_more: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_fixture_has_usable_models_templates_and_gallery() {
        for kind in [
            GenerationKind::Image,
            GenerationKind::Video,
            GenerationKind::Audio,
            GenerationKind::Svg,
        ] {
            let data = fixture(kind);
            assert!(!data.models.is_empty());
            assert!(!data.templates.is_empty());
            assert!(!data.outputs.is_empty());
            assert!(data.models.iter().all(|model| model.recipe.kind() == kind));
            assert!(
                data.models
                    .iter()
                    .any(|model| { data.selected_model_id.as_ref() == Some(&model.id) })
            );
            assert!(data.templates.iter().all(|template| template.kind == kind));
            for template in &data.templates {
                if let Some(model_id) = &template.model_id {
                    assert!(data.models.iter().any(|model| {
                        &model.id == model_id
                            && model.recipe.kind() == template.kind
                            && (model.supports_prompt || template.prompt.is_empty())
                    }));
                }
            }
            assert!(data.outputs.iter().all(|output| {
                output.kind == kind && data.models.iter().any(|model| model.id == output.model_id)
            }));
            for model in &data.models {
                for option in &model.option_groups {
                    assert!(
                        option
                            .choices
                            .iter()
                            .any(|choice| choice.value == option.selected)
                    );
                }
            }
        }
    }

    #[test]
    fn catalog_fixture_exercises_wide_hosted_and_fallback_choices() {
        for (kind, minimum_count) in [
            (GenerationKind::Image, 10),
            (GenerationKind::Video, 15),
            (GenerationKind::Audio, 9),
            (GenerationKind::Svg, 7),
        ] {
            let data = fixture(kind);
            assert!(
                data.models.len() >= minimum_count,
                "{kind:?} catalog narrowed"
            );
            assert!(data.models.iter().all(|model| model.logo.is_some()));
            assert!(data.models.iter().all(|model| {
                model.credits_per_output.is_some() || model.credit_hint.is_some()
            }));
            assert!(
                data.models
                    .iter()
                    .any(|model| model.id.starts_with("gateway-"))
            );
            assert!(
                data.models
                    .iter()
                    .any(|model| model.id.starts_with("replicate-"))
            );
        }
        let video = fixture(GenerationKind::Video);
        assert!(video.models.iter().any(|model| {
            model.recipe == GenerationRecipe::ImageVideo && model.supports_end_frame
        }));
        let audio = fixture(GenerationKind::Audio);
        assert!(audio.models.iter().any(|model| {
            model
                .option_groups
                .iter()
                .any(|group| group.choices.len() > 3)
        }));
    }
}
