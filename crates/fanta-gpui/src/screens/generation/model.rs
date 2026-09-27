//! Host data and typed requests for the four generation-only workspaces.
//!
//! The host constructs the model catalog from its authenticated model list and
//! maps a submission to the appropriate service. The UI does not infer model
//! capability from an ID, charge credits, or persist generated media.

use std::{error::Error, fmt, sync::Arc};

use gpui::{AnyView, Image, SharedString};

/// The generation API receives seeds as JSON numbers; larger integers cannot
/// round-trip through JavaScript without losing precision.
pub const MAX_GENERATION_SEED: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GenerationKind {
    Image,
    Video,
    Audio,
    Svg,
}

impl GenerationKind {
    pub const ALL: [Self; 4] = [Self::Image, Self::Video, Self::Audio, Self::Svg];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Image => "Images",
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Svg => "SVG vectors",
        }
    }

    pub const fn singular(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Svg => "SVG vector",
        }
    }
}

/// A generation path, distinct from edit, mask, and post-production operations.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GenerationRecipe {
    TextImage,
    TextVideo,
    ImageVideo,
    Speech,
    Music,
    PromptSvg,
    ImageSvg,
    Vectorize,
}

impl GenerationRecipe {
    pub const fn kind(self) -> GenerationKind {
        match self {
            Self::TextImage => GenerationKind::Image,
            Self::TextVideo | Self::ImageVideo => GenerationKind::Video,
            Self::Speech | Self::Music => GenerationKind::Audio,
            Self::PromptSvg | Self::ImageSvg | Self::Vectorize => GenerationKind::Svg,
        }
    }

    pub const fn requires_source(self) -> bool {
        matches!(self, Self::ImageVideo | Self::ImageSvg | Self::Vectorize)
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::TextImage => "Text to image",
            Self::TextVideo => "Text to video",
            Self::ImageVideo => "Image to video",
            Self::Speech => "Text to speech",
            Self::Music => "Text to music",
            Self::PromptSvg => "Prompt to SVG",
            Self::ImageSvg => "Image to SVG",
            Self::Vectorize => "Vectorize an image",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationChoice {
    pub value: SharedString,
    pub label: SharedString,
}

impl GenerationChoice {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

/// A host-declared control. Keys/values are opaque to the UI and translated by
/// the host into validated request fields, including model-specific `input`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationOptionGroup {
    pub key: SharedString,
    pub label: SharedString,
    pub choices: Vec<GenerationChoice>,
    pub selected: SharedString,
    /// Secondary control shown in the expandable model settings section.
    pub advanced: bool,
}

impl GenerationOptionGroup {
    pub fn new(
        key: impl Into<SharedString>,
        label: impl Into<SharedString>,
        choices: impl IntoIterator<Item = GenerationChoice>,
        selected: impl Into<SharedString>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            choices: choices.into_iter().collect(),
            selected: selected.into(),
            advanced: false,
        }
    }

    pub fn advanced(mut self, value: bool) -> Self {
        self.advanced = value;
        self
    }
}

/// A host-decoded provider mark. The key identifies immutable logo content so
/// controlled catalog snapshots can be compared without comparing pixels.
#[derive(Clone)]
pub struct GenerationModelLogo {
    pub key: SharedString,
    pub image: Arc<Image>,
}

impl GenerationModelLogo {
    pub fn new(key: impl Into<SharedString>, image: Arc<Image>) -> Self {
        Self {
            key: key.into(),
            image,
        }
    }
}

impl fmt::Debug for GenerationModelLogo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GenerationModelLogo")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl PartialEq for GenerationModelLogo {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for GenerationModelLogo {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationModel {
    pub id: SharedString,
    pub label: SharedString,
    /// Resolved engine/provider name, supplied by the host catalog adapter.
    pub provider_label: SharedString,
    pub logo: Option<GenerationModelLogo>,
    pub description: SharedString,
    pub recipe: GenerationRecipe,
    /// Current authenticated catalog price. The backend charges per output.
    pub credits_per_output: Option<u32>,
    /// Optional host-supplied text for a variable-price model such as chat.
    pub credit_hint: Option<SharedString>,
    pub requires_source: bool,
    /// Reserved for models whose backend worker accepts an explicit end image.
    pub supports_end_frame: bool,
    pub supports_prompt: bool,
    /// UTF-16 string-unit limit for this model's prompt. The default of 1,200
    /// matches the current generation API; the host can override it.
    /// `None` disables the component-side length check.
    pub max_prompt_chars: Option<usize>,
    pub supports_negative: bool,
    pub supports_seed: bool,
    pub supports_voice_reference: bool,
    pub option_groups: Vec<GenerationOptionGroup>,
}

impl GenerationModel {
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        recipe: GenerationRecipe,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            provider_label: SharedString::default(),
            logo: None,
            description: SharedString::default(),
            recipe,
            credits_per_output: None,
            credit_hint: None,
            requires_source: recipe.requires_source(),
            supports_end_frame: false,
            supports_prompt: !matches!(
                recipe,
                GenerationRecipe::ImageSvg | GenerationRecipe::Vectorize
            ),
            max_prompt_chars: Some(1_200),
            supports_negative: false,
            supports_seed: false,
            supports_voice_reference: false,
            option_groups: Vec::new(),
        }
    }

    pub fn description(mut self, value: impl Into<SharedString>) -> Self {
        self.description = value.into();
        self
    }

    pub fn provider(mut self, value: impl Into<SharedString>) -> Self {
        self.provider_label = value.into();
        self
    }

    pub fn logo(mut self, key: impl Into<SharedString>, image: Arc<Image>) -> Self {
        self.logo = Some(GenerationModelLogo::new(key, image));
        self
    }

    pub fn credits_per_output(mut self, credits: u32) -> Self {
        self.credits_per_output = Some(credits);
        self
    }

    pub fn cost_label(&self) -> Option<SharedString> {
        self.credits_per_output
            .map(|credits| {
                format!(
                    "{credits} {} / output",
                    if credits == 1 { "credit" } else { "credits" }
                )
                .into()
            })
            .or_else(|| self.credit_hint.clone())
    }

    pub fn credit_hint(mut self, value: impl Into<SharedString>) -> Self {
        self.credit_hint = Some(value.into());
        self
    }

    pub fn requires_source(mut self, value: bool) -> Self {
        self.requires_source = value;
        self
    }

    pub fn supports_end_frame(mut self, value: bool) -> Self {
        self.supports_end_frame = value;
        self
    }

    pub fn supports_prompt(mut self, value: bool) -> Self {
        self.supports_prompt = value;
        self
    }

    pub fn max_prompt_chars(mut self, value: impl Into<Option<usize>>) -> Self {
        self.max_prompt_chars = value.into();
        self
    }

    pub fn supports_negative(mut self, value: bool) -> Self {
        self.supports_negative = value;
        self
    }

    pub fn supports_seed(mut self, value: bool) -> Self {
        self.supports_seed = value;
        self
    }

    pub fn supports_voice_reference(mut self, value: bool) -> Self {
        self.supports_voice_reference = value;
        self
    }

    pub fn option_groups(
        mut self,
        groups: impl IntoIterator<Item = GenerationOptionGroup>,
    ) -> Self {
        self.option_groups = groups.into_iter().collect();
        self
    }
}

#[derive(Clone)]
pub struct GenerationSource {
    pub id: SharedString,
    pub name: SharedString,
    /// Already decoded by the host; no remote URL is fetched by the component.
    pub preview: Option<Arc<Image>>,
}

impl GenerationSource {
    pub fn new(id: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            preview: None,
        }
    }

    pub fn preview(mut self, image: Arc<Image>) -> Self {
        self.preview = Some(image);
        self
    }
}

/// An audio sample supplied by the host for supported speech models. The host
/// obtains the user's consent before it submits audio to the backend.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationVoiceReference {
    pub id: SharedString,
    pub name: SharedString,
}

impl GenerationVoiceReference {
    pub fn new(id: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }
}

#[derive(Clone)]
pub struct GenerationTemplate {
    pub id: SharedString,
    pub title: SharedString,
    pub kind: GenerationKind,
    pub model_id: Option<SharedString>,
    pub prompt: SharedString,
    pub description: SharedString,
    pub preview: Option<Arc<Image>>,
}

impl GenerationTemplate {
    pub fn new(
        id: impl Into<SharedString>,
        title: impl Into<SharedString>,
        kind: GenerationKind,
        prompt: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            kind,
            model_id: None,
            prompt: prompt.into(),
            description: SharedString::default(),
            preview: None,
        }
    }

    pub fn description(mut self, value: impl Into<SharedString>) -> Self {
        self.description = value.into();
        self
    }

    pub fn model(mut self, id: impl Into<SharedString>) -> Self {
        self.model_id = Some(id.into());
        self
    }

    pub fn preview(mut self, image: Arc<Image>) -> Self {
        self.preview = Some(image);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenerationOutputStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Canceled,
}

impl GenerationOutputStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Generating",
            Self::Succeeded => "Ready",
            Self::Failed => "Failed",
            Self::Canceled => "Canceled",
        }
    }
}

#[derive(Clone)]
pub struct GenerationOutput {
    pub id: SharedString,
    pub kind: GenerationKind,
    pub title: SharedString,
    pub model_id: SharedString,
    pub status: GenerationOutputStatus,
    pub prompt: Option<SharedString>,
    pub created_at: SharedString,
    pub detail: SharedString,
    /// Host-supplied decoded still image or thumbnail (including SVG previews).
    pub preview: Option<Arc<Image>>,
    /// Optional host-owned inline audio/video player mounted in the lightbox.
    pub playback_view: Option<AnyView>,
}

impl GenerationOutput {
    pub fn new(
        id: impl Into<SharedString>,
        kind: GenerationKind,
        title: impl Into<SharedString>,
        model_id: impl Into<SharedString>,
        status: GenerationOutputStatus,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            title: title.into(),
            model_id: model_id.into(),
            status,
            prompt: None,
            created_at: SharedString::default(),
            detail: SharedString::default(),
            preview: None,
            playback_view: None,
        }
    }

    pub fn prompt(mut self, value: impl Into<SharedString>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn created_at(mut self, value: impl Into<SharedString>) -> Self {
        self.created_at = value.into();
        self
    }

    pub fn detail(mut self, value: impl Into<SharedString>) -> Self {
        self.detail = value.into();
        self
    }

    pub fn preview(mut self, image: Arc<Image>) -> Self {
        self.preview = Some(image);
        self
    }

    pub fn playback_view(mut self, view: impl Into<AnyView>) -> Self {
        self.playback_view = Some(view.into());
        self
    }
}

#[derive(Clone, Default)]
pub struct GenerationViewData {
    pub models: Vec<GenerationModel>,
    pub selected_recipe: Option<GenerationRecipe>,
    pub templates: Vec<GenerationTemplate>,
    pub outputs: Vec<GenerationOutput>,
    pub selected_model_id: Option<SharedString>,
    pub selected_template_id: Option<SharedString>,
    pub selected_output_id: Option<SharedString>,
    pub source: Option<GenerationSource>,
    pub end_frame: Option<GenerationSource>,
    pub voice_reference: Option<GenerationVoiceReference>,
    pub voice_consent_granted: bool,
    pub busy: bool,
    pub error: Option<SharedString>,
    pub has_more: bool,
}

/// Transient text owned by the screen. A host may snapshot it before changing
/// workspaces and restore it when the user returns.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GenerationDraft {
    pub prompt: SharedString,
    pub negative: SharedString,
    pub seed: SharedString,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationOptionSelection {
    pub key: SharedString,
    pub value: SharedString,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GenerationSubmission {
    pub kind: GenerationKind,
    pub recipe: GenerationRecipe,
    pub model_id: SharedString,
    pub prompt: SharedString,
    pub negative: Option<SharedString>,
    pub seed: Option<u64>,
    pub source_id: Option<SharedString>,
    pub end_frame_id: Option<SharedString>,
    pub voice_reference_id: Option<SharedString>,
    pub voice_consent_granted: bool,
    pub options: Vec<GenerationOptionSelection>,
}

/// Why a draft cannot be handed to the host as a generation request.
///
/// The host still validates the request against its current catalog and the
/// backend before submitting it. This check keeps the screen from emitting a
/// stale or internally inconsistent draft after models and controls change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GenerationValidationError {
    ModelMismatch,
    RecipeMismatch,
    KindMismatch,
    PromptRequired,
    PromptUnsupported,
    PromptTooLong,
    SourceRequired,
    SourceUnsupported,
    EndFrameRequiresStart,
    EndFrameUnsupported,
    VoiceReferenceUnsupported,
    VoiceConsentRequired,
    NegativeUnsupported,
    SeedUnsupported,
    SeedOutOfRange,
    UnknownOption,
    DuplicateOption,
    InvalidOption,
    MissingOption,
}

impl GenerationValidationError {
    pub const fn message(self) -> &'static str {
        match self {
            Self::ModelMismatch | Self::RecipeMismatch | Self::KindMismatch => {
                "Choose a model for this generation tool."
            }
            Self::PromptRequired => "Enter a prompt to continue.",
            Self::PromptUnsupported => "This model does not accept a prompt.",
            Self::PromptTooLong => "The prompt is too long for this model.",
            Self::SourceRequired => "Choose a source image to continue.",
            Self::SourceUnsupported => "This model does not accept a source image.",
            Self::EndFrameRequiresStart => "Choose a start frame before an end frame.",
            Self::EndFrameUnsupported => "This model does not accept an end frame.",
            Self::VoiceReferenceUnsupported => "This model does not accept a reference voice.",
            Self::VoiceConsentRequired => "Confirm permission to use the reference voice.",
            Self::NegativeUnsupported => "This model does not accept a negative prompt.",
            Self::SeedUnsupported => "This model does not accept a seed.",
            Self::SeedOutOfRange => "Seed must be at most 9007199254740991.",
            Self::UnknownOption
            | Self::DuplicateOption
            | Self::InvalidOption
            | Self::MissingOption => "Choose valid settings for this model.",
        }
    }
}

impl fmt::Display for GenerationValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl Error for GenerationValidationError {}

impl GenerationSubmission {
    /// Check the request against the selected, host-supplied model before
    /// emitting `GenerateRequested`. Option keys and values stay opaque; only
    /// membership in the declared model controls is checked here.
    pub fn validate_against(
        &self,
        model: &GenerationModel,
    ) -> Result<(), GenerationValidationError> {
        if self.model_id != model.id {
            return Err(GenerationValidationError::ModelMismatch);
        }
        if self.recipe != model.recipe {
            return Err(GenerationValidationError::RecipeMismatch);
        }
        if self.kind != model.recipe.kind() {
            return Err(GenerationValidationError::KindMismatch);
        }

        let prompt = self.prompt.trim();
        if model
            .max_prompt_chars
            .is_some_and(|limit| self.prompt.encode_utf16().count() > limit)
        {
            return Err(GenerationValidationError::PromptTooLong);
        }
        if !model.supports_prompt && !prompt.is_empty() {
            return Err(GenerationValidationError::PromptUnsupported);
        }
        if prompt.is_empty()
            && matches!(
                self.recipe,
                GenerationRecipe::TextImage
                    | GenerationRecipe::TextVideo
                    | GenerationRecipe::ImageVideo
                    | GenerationRecipe::Speech
                    | GenerationRecipe::Music
                    | GenerationRecipe::PromptSvg
            )
        {
            return Err(GenerationValidationError::PromptRequired);
        }

        let requires_source = model.requires_source || self.recipe.requires_source();
        let has_source = self
            .source_id
            .as_ref()
            .is_some_and(|id| !id.trim().is_empty());
        if requires_source && !has_source {
            return Err(GenerationValidationError::SourceRequired);
        }
        if self.source_id.is_some() && !requires_source {
            return Err(GenerationValidationError::SourceUnsupported);
        }
        if self.end_frame_id.is_some() && !has_source {
            return Err(GenerationValidationError::EndFrameRequiresStart);
        }
        if self
            .end_frame_id
            .as_ref()
            .is_some_and(|id| id.trim().is_empty())
        {
            return Err(GenerationValidationError::EndFrameUnsupported);
        }
        if self.end_frame_id.is_some()
            && (self.recipe != GenerationRecipe::ImageVideo || !model.supports_end_frame)
        {
            return Err(GenerationValidationError::EndFrameUnsupported);
        }

        if let Some(reference) = &self.voice_reference_id {
            if reference.trim().is_empty()
                || self.recipe != GenerationRecipe::Speech
                || !model.supports_voice_reference
            {
                return Err(GenerationValidationError::VoiceReferenceUnsupported);
            }
            if !self.voice_consent_granted {
                return Err(GenerationValidationError::VoiceConsentRequired);
            }
        }
        if self.negative.is_some() && !model.supports_negative {
            return Err(GenerationValidationError::NegativeUnsupported);
        }
        if self.seed.is_some() && !model.supports_seed {
            return Err(GenerationValidationError::SeedUnsupported);
        }
        if self.seed.is_some_and(|seed| seed > MAX_GENERATION_SEED) {
            return Err(GenerationValidationError::SeedOutOfRange);
        }

        let mut seen = Vec::with_capacity(self.options.len());
        for option in &self.options {
            if seen.contains(&&option.key) {
                return Err(GenerationValidationError::DuplicateOption);
            }
            seen.push(&option.key);
            let Some(group) = model
                .option_groups
                .iter()
                .find(|group| group.key == option.key)
            else {
                return Err(GenerationValidationError::UnknownOption);
            };
            if !group
                .choices
                .iter()
                .any(|choice| choice.value == option.value)
            {
                return Err(GenerationValidationError::InvalidOption);
            }
        }
        if model
            .option_groups
            .iter()
            .any(|group| !seen.contains(&&group.key))
        {
            return Err(GenerationValidationError::MissingOption);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenerationAction {
    KindSelected(GenerationKind),
    RecipeSelected(GenerationRecipe),
    DraftChanged,
    ModelSelected {
        id: SharedString,
    },
    OptionSelected {
        model_id: SharedString,
        key: SharedString,
        value: SharedString,
    },
    SourceRequested,
    SourceCleared,
    EndFrameRequested,
    EndFrameCleared,
    VoiceReferenceRequested,
    VoiceReferenceCleared,
    VoiceConsentChanged(bool),
    TemplateSelected {
        id: SharedString,
    },
    GenerateRequested(GenerationSubmission),
    OutputSelected {
        id: SharedString,
    },
    PreviewClosed {
        id: SharedString,
    },
    DownloadRequested {
        id: SharedString,
    },
    PlayRequested {
        id: SharedString,
    },
    ReusePromptRequested {
        id: SharedString,
    },
    RefreshRequested,
    LoadMoreRequested,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_for(model: &GenerationModel) -> GenerationSubmission {
        GenerationSubmission {
            kind: model.recipe.kind(),
            recipe: model.recipe,
            model_id: model.id.clone(),
            prompt: "A lantern in a rainy street".into(),
            negative: None,
            seed: None,
            source_id: model.requires_source.then(|| "asset-1".into()),
            end_frame_id: None,
            voice_reference_id: None,
            voice_consent_granted: false,
            options: model
                .option_groups
                .iter()
                .map(|group| GenerationOptionSelection {
                    key: group.key.clone(),
                    value: group.selected.clone(),
                })
                .collect(),
        }
    }

    #[test]
    fn recipes_stay_with_their_media_kind() {
        for recipe in [GenerationRecipe::Speech, GenerationRecipe::Music] {
            assert_eq!(recipe.kind(), GenerationKind::Audio);
        }
        for recipe in [
            GenerationRecipe::PromptSvg,
            GenerationRecipe::ImageSvg,
            GenerationRecipe::Vectorize,
        ] {
            assert_eq!(recipe.kind(), GenerationKind::Svg);
        }
    }

    #[test]
    fn source_requirements_follow_generation_recipe() {
        assert!(GenerationRecipe::ImageVideo.requires_source());
        assert!(GenerationRecipe::ImageSvg.requires_source());
        assert!(GenerationRecipe::Vectorize.requires_source());
        assert!(!GenerationRecipe::TextVideo.requires_source());
        assert!(
            GenerationModel::new("i2v", "Animate", GenerationRecipe::ImageVideo).requires_source
        );
        assert!(GenerationModel::new("trace", "Trace", GenerationRecipe::ImageSvg).requires_source);
        assert!(!GenerationModel::new("t2v", "Video", GenerationRecipe::TextVideo).requires_source);
    }

    #[test]
    fn end_frame_requires_a_capable_image_to_video_model_and_start_frame() {
        let model = GenerationModel::new("i2v", "Animate", GenerationRecipe::ImageVideo);
        let mut request = request_for(&model);
        request.end_frame_id = Some("ending".into());
        assert_eq!(
            request.validate_against(&model),
            Err(GenerationValidationError::EndFrameUnsupported)
        );
        let capable = model.supports_end_frame(true);
        assert_eq!(request.validate_against(&capable), Ok(()));
        request.source_id = None;
        assert_eq!(
            request.validate_against(&capable),
            Err(GenerationValidationError::SourceRequired)
        );
    }

    #[test]
    fn rejects_stale_model_and_cross_kind_submissions() {
        let model = GenerationModel::new("fanta-image-1", "Image", GenerationRecipe::TextImage);
        let mut request = request_for(&model);
        assert_eq!(request.validate_against(&model), Ok(()));

        request.model_id = "retired-model".into();
        assert_eq!(
            request.validate_against(&model),
            Err(GenerationValidationError::ModelMismatch)
        );
        request.model_id = model.id.clone();
        request.recipe = GenerationRecipe::TextVideo;
        assert_eq!(
            request.validate_against(&model),
            Err(GenerationValidationError::RecipeMismatch)
        );
        request.recipe = model.recipe;
        request.kind = GenerationKind::Audio;
        assert_eq!(
            request.validate_against(&model),
            Err(GenerationValidationError::KindMismatch)
        );
    }

    #[test]
    fn prompt_and_source_requirements_follow_recipe_not_just_visibility_flags() {
        let text_video =
            GenerationModel::new("fanta-video-1", "Video", GenerationRecipe::TextVideo);
        let mut request = request_for(&text_video);
        request.prompt = "  ".into();
        assert_eq!(
            request.validate_against(&text_video),
            Err(GenerationValidationError::PromptRequired)
        );
        request.prompt = "Pan across the skyline".into();
        request.source_id = Some("asset-1".into());
        assert_eq!(
            request.validate_against(&text_video),
            Err(GenerationValidationError::SourceUnsupported)
        );

        let image_video =
            GenerationModel::new("fanta-animate-1", "Animate", GenerationRecipe::ImageVideo)
                .requires_source(false);
        let mut request = request_for(&image_video);
        request.prompt = "".into();
        assert_eq!(
            request.validate_against(&image_video),
            Err(GenerationValidationError::PromptRequired)
        );
        request.prompt = "A slow camera push through drifting mist".into();
        assert_eq!(
            request.validate_against(&image_video),
            Err(GenerationValidationError::SourceRequired)
        );
        request.source_id = Some("asset-1".into());
        assert_eq!(request.validate_against(&image_video), Ok(()));

        for recipe in [GenerationRecipe::ImageSvg, GenerationRecipe::Vectorize] {
            let model = GenerationModel::new("vector", "Vector", recipe).requires_source(false);
            let mut request = request_for(&model);
            request.prompt = "".into();
            assert_eq!(
                request.validate_against(&model),
                Err(GenerationValidationError::SourceRequired)
            );
            request.source_id = Some("asset-1".into());
            assert_eq!(request.validate_against(&model), Ok(()));
        }

        let starvector = GenerationModel::new("fanta-svg-1", "SVG", GenerationRecipe::ImageSvg);
        let mut request = request_for(&starvector);
        assert_eq!(
            request.validate_against(&starvector),
            Err(GenerationValidationError::PromptUnsupported)
        );
        request.prompt = "".into();
        assert_eq!(request.validate_against(&starvector), Ok(()));

        let image = GenerationModel::new("fanta-image-1", "Image", GenerationRecipe::TextImage);
        let mut request = request_for(&image);
        request.prompt = "x".repeat(1_201).into();
        assert_eq!(
            request.validate_against(&image),
            Err(GenerationValidationError::PromptTooLong)
        );
        let short_limit = image.clone().max_prompt_chars(12);
        request.prompt = "A longer prompt".into();
        assert_eq!(
            request.validate_against(&short_limit),
            Err(GenerationValidationError::PromptTooLong)
        );
        assert_eq!(
            request.validate_against(&short_limit.max_prompt_chars(None)),
            Ok(())
        );

        let raw_limit = image.max_prompt_chars(4);
        request.prompt = "  hi  ".into();
        assert_eq!(
            request.validate_against(&raw_limit),
            Err(GenerationValidationError::PromptTooLong)
        );
        request.prompt = "😀😀😀".into();
        assert_eq!(
            request.validate_against(&raw_limit),
            Err(GenerationValidationError::PromptTooLong)
        );
    }

    #[test]
    fn rejects_controls_the_selected_model_did_not_declare() {
        let fast = GenerationModel::new("fanta-image-fast-1", "Fast", GenerationRecipe::TextImage);
        let mut request = request_for(&fast);
        request.negative = Some("blur".into());
        assert_eq!(
            request.validate_against(&fast),
            Err(GenerationValidationError::NegativeUnsupported)
        );
        request.negative = None;
        request.seed = Some(0);
        assert_eq!(
            request.validate_against(&fast),
            Err(GenerationValidationError::SeedUnsupported)
        );

        let qwen = GenerationModel::new("fanta-image-1", "Image", GenerationRecipe::TextImage)
            .supports_negative(true)
            .supports_seed(true);
        let mut request = request_for(&qwen);
        request.negative = Some("blur".into());
        request.seed = Some(0); // zero is a valid backend seed
        assert_eq!(request.validate_against(&qwen), Ok(()));
        request.seed = Some(MAX_GENERATION_SEED + 1);
        assert_eq!(
            request.validate_against(&qwen),
            Err(GenerationValidationError::SeedOutOfRange)
        );
    }

    #[test]
    fn voice_reference_requires_supported_speech_model_and_consent() {
        let voice = GenerationModel::new("fanta-voice-1", "Voice", GenerationRecipe::Speech)
            .supports_voice_reference(true);
        let mut request = request_for(&voice);
        request.voice_reference_id = Some("voice-1".into());
        assert_eq!(
            request.validate_against(&voice),
            Err(GenerationValidationError::VoiceConsentRequired)
        );
        request.voice_consent_granted = true;
        assert_eq!(request.validate_against(&voice), Ok(()));

        let fast_voice =
            GenerationModel::new("fanta-voice-fast-1", "Fast", GenerationRecipe::Speech);
        request.model_id = fast_voice.id.clone();
        assert_eq!(
            request.validate_against(&fast_voice),
            Err(GenerationValidationError::VoiceReferenceUnsupported)
        );

        let music = GenerationModel::new("fanta-music-1", "Music", GenerationRecipe::Music)
            .supports_voice_reference(true);
        request.model_id = music.id.clone();
        request.recipe = music.recipe;
        assert_eq!(
            request.validate_against(&music),
            Err(GenerationValidationError::VoiceReferenceUnsupported)
        );
    }

    #[test]
    fn option_values_must_match_selected_model_catalog() {
        let model = GenerationModel::new(
            "fanta-vectorize-1",
            "Vectorize",
            GenerationRecipe::Vectorize,
        )
        .option_groups([
            GenerationOptionGroup::new(
                "mode",
                "Mode",
                [
                    GenerationChoice::new("line_art", "Line art"),
                    GenerationChoice::new("trace", "Trace"),
                ],
                "line_art",
            ),
            GenerationOptionGroup::new(
                "detail",
                "Detail",
                [
                    GenerationChoice::new("low", "Low"),
                    GenerationChoice::new("high", "High"),
                ],
                "high",
            ),
        ]);
        let mut request = request_for(&model);
        request.prompt = "".into(); // optional vectorization guidance
        assert_eq!(request.validate_against(&model), Ok(()));

        request.options[0].value = "flat_color".into();
        assert_eq!(
            request.validate_against(&model),
            Err(GenerationValidationError::InvalidOption)
        );
        request.options[0].value = "trace".into();
        request.options.push(request.options[0].clone());
        assert_eq!(
            request.validate_against(&model),
            Err(GenerationValidationError::DuplicateOption)
        );
        request.options.pop();
        request.options[0].key = "operation".into();
        assert_eq!(
            request.validate_against(&model),
            Err(GenerationValidationError::UnknownOption)
        );
        request.options.remove(0);
        assert_eq!(
            request.validate_against(&model),
            Err(GenerationValidationError::MissingOption)
        );
    }
}
