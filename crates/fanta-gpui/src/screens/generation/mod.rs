//! Zed-themed, host-controlled creation workspace for image, video, audio and
//! SVG generation. Model capabilities and generated items are supplied by the
//! host; this entity keeps only draft text, focus, scrolling and validation.

mod lightbox;
mod model;
mod template_art;
pub use model::*;

use std::collections::HashMap;

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable, Hsla,
    Image, InteractiveElement as _, IntoElement, ObjectFit, ParentElement as _, Render, Role,
    ScrollHandle, SharedString, StatefulInteractiveElement as _, Styled as _, StyledImage as _,
    Subscription, Toggled, Window, div, img, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    IndexPath, Theme as ComponentTheme,
    input::{Input, InputEvent, InputState},
    select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
};

use crate::atoms::{CONTROL_KEY_CONTEXT, ControlExt as _};
use ui::StyledTypography as _;

/// Width at which the composer and gallery sit side by side.
pub const GENERATION_SCREEN_MIN_WIDTH: f32 = 540.;
pub const GENERATION_SCREEN_MIN_HEIGHT: f32 = 480.;
pub(crate) const GENERATION_SCREEN_KEY_CONTEXT: &str = "FantaGenerationScreen";

#[derive(Clone, Copy)]
struct GenerationStyle {
    background: Hsla,
    panel: Hsla,
    surface: Hsla,
    hover: Hsla,
    selected: Hsla,
    border: Hsla,
    focus: Hsla,
    text: Hsla,
    muted: Hsla,
    accent: Hsla,
}

impl GenerationStyle {
    fn current(cx: &App) -> Self {
        if cx.try_global::<theme::GlobalTheme>().is_some() {
            let colors = theme::GlobalTheme::theme(cx).colors();
            Self {
                background: colors.editor_background,
                panel: colors.panel_background,
                surface: colors.surface_background,
                hover: colors.element_hover,
                selected: colors.element_selected,
                border: colors.border_variant,
                focus: colors.border_focused,
                text: colors.text,
                muted: colors.text_muted,
                accent: colors.text_accent,
            }
        } else {
            let colors = ComponentTheme::global(cx);
            Self {
                background: colors.background,
                panel: colors.sidebar,
                surface: colors.secondary,
                hover: colors.secondary_hover,
                selected: colors.list_active,
                border: colors.border,
                focus: colors.ring,
                text: colors.foreground,
                muted: colors.muted_foreground,
                accent: colors.primary,
            }
        }
    }
}

#[derive(Clone)]
struct ModelSelectItem {
    id: SharedString,
    label: SharedString,
    provider: SharedString,
    logo: Option<GenerationModelLogo>,
    recipe: &'static str,
    description: SharedString,
    cost: Option<SharedString>,
}

impl SelectItem for ModelSelectItem {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        format!(
            "{} {} {} {} {}",
            self.label, self.id, self.provider, self.recipe, self.description
        )
        .into()
    }

    fn display_title(&self) -> Option<AnyElement> {
        let mut label = div().flex().items_center().gap(px(8.));
        if let Some(logo) = &self.logo {
            label = label.child(
                img(logo.image.clone())
                    .w(px(22.))
                    .h(px(22.))
                    .object_fit(ObjectFit::Contain),
            );
        }
        Some(label.child(self.label.clone()).into_any_element())
    }

    fn render(&self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = GenerationStyle::current(cx);
        div()
            .w_full()
            .flex()
            .items_center()
            .gap(px(10.))
            .py(px(6.))
            .when_some(self.logo.clone(), |row, logo| {
                row.child(
                    div()
                        .w(px(34.))
                        .h(px(34.))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_sm()
                        .border_1()
                        .border_color(colors.border)
                        .bg(colors.background)
                        .child(
                            img(logo.image)
                                .w(px(27.))
                                .h(px(27.))
                                .object_fit(ObjectFit::Contain),
                        ),
                )
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(px(3.))
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(colors.text)
                            .child(self.label.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(10.))
                            .text_color(colors.muted)
                            .child(format!(
                                "{} · {} · {}",
                                self.provider, self.recipe, self.description
                            )),
                    ),
            )
            .when_some(self.cost.clone(), |row, cost| {
                row.child(
                    div()
                        .flex_none()
                        .text_size(px(10.))
                        .text_color(colors.accent)
                        .child(cost),
                )
            })
    }

    fn value(&self) -> &Self::Value {
        &self.id
    }
}

impl SelectItem for GenerationChoice {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.label.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }
}

struct OptionSelectState {
    state: Entity<SelectState<SearchableVec<GenerationChoice>>>,
    snapshot: GenerationOptionGroup,
    _subscription: Subscription,
}

#[derive(Clone, Copy)]
enum ControlAppearance {
    Normal,
    Selected,
    Prominent,
    Disabled,
    Toggled(bool),
}

pub struct GenerationScreen {
    id: SharedString,
    kind: GenerationKind,
    data: GenerationViewData,
    prompt: Entity<InputState>,
    negative: Entity<InputState>,
    seed: Entity<InputState>,
    model_select: Entity<SelectState<SearchableVec<ModelSelectItem>>>,
    option_selects: HashMap<(SharedString, SharedString), OptionSelectState>,
    validation_error: Option<SharedString>,
    show_advanced: bool,
    focus_handle: FocusHandle,
    lightbox_focus_handle: FocusHandle,
    lightbox_output_id: Option<SharedString>,
    composer_scroll: ScrollHandle,
    gallery_scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<GenerationAction> for GenerationScreen {}

impl GenerationScreen {
    fn selected_model_for(
        kind: GenerationKind,
        data: &GenerationViewData,
    ) -> Option<&GenerationModel> {
        let selected_id = data.selected_model_id.as_ref()?;
        data.models.iter().find(|model| {
            &model.id == selected_id
                && model.recipe.kind() == kind
                && data
                    .selected_recipe
                    .is_none_or(|recipe| model.recipe == recipe)
        })
    }

    pub fn new(
        id: impl Into<SharedString>,
        kind: GenerationKind,
        data: GenerationViewData,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let prompt_hint = Self::prompt_hint(kind, data.selected_recipe);
        let prompt = cx.new(|cx| InputState::new(window, cx).placeholder(prompt_hint));
        prompt.update(cx, |input, cx| input.set_multiline(Some(8), cx));
        let negative =
            cx.new(|cx| InputState::new(window, cx).placeholder("What should be avoided?"));
        negative.update(cx, |input, cx| input.set_multiline(Some(4), cx));
        let seed = cx.new(|cx| InputState::new(window, cx).placeholder("Random"));
        let model_items = Self::model_select_items(kind, &data);
        let selected_index = data.selected_model_id.as_ref().and_then(|id| {
            model_items
                .iter()
                .position(|model| &model.id == id)
                .map(|index| IndexPath::default().row(index))
        });
        let model_select = cx.new(|cx| {
            SelectState::new(SearchableVec::new(model_items), selected_index, window, cx)
                .searchable(true)
        });
        let mut subscriptions = [prompt.clone(), negative.clone(), seed.clone()]
            .into_iter()
            .map(|input| {
                cx.subscribe(&input, |this, _, event: &InputEvent, cx| {
                    if matches!(event, InputEvent::Change) {
                        this.validation_error = None;
                        cx.emit(GenerationAction::DraftChanged);
                        cx.notify();
                    }
                })
            })
            .collect::<Vec<_>>();
        subscriptions.push(cx.subscribe_in(
            &model_select,
            window,
            |_, select, event: &SelectEvent<SearchableVec<ModelSelectItem>>, window, cx| {
                let SelectEvent::Confirm(Some(id)) = event else {
                    return;
                };
                cx.emit(GenerationAction::ModelSelected { id: id.clone() });
                let select = select.clone();
                cx.defer_in(window, move |this, window, cx| {
                    let controlled = this.data.selected_model_id.clone();
                    select.update(cx, |state, cx| {
                        if let Some(id) = controlled.as_ref() {
                            state.set_selected_value(id, window, cx);
                        } else {
                            state.set_selected_index(None, window, cx);
                        }
                    });
                });
            },
        ));
        let mut screen = Self {
            id: id.into(),
            kind,
            data,
            prompt,
            negative,
            seed,
            model_select,
            option_selects: HashMap::new(),
            validation_error: None,
            show_advanced: true,
            focus_handle: cx.focus_handle(),
            lightbox_focus_handle: cx.focus_handle(),
            lightbox_output_id: None,
            composer_scroll: ScrollHandle::new(),
            gallery_scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        };
        screen.sync_option_selects(window, cx);
        screen
    }

    fn sync_option_selects(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let selected_model = self.selected_model().cloned();
        let Some(model) = selected_model else {
            self.option_selects.clear();
            return;
        };
        self.option_selects.retain(|(model_id, key), _| {
            model_id == &model.id
                && model
                    .option_groups
                    .iter()
                    .any(|group| &group.key == key && group.choices.len() > 3)
        });
        for group in model
            .option_groups
            .into_iter()
            .filter(|group| group.choices.len() > 3)
        {
            let key = (model.id.clone(), group.key.clone());
            if let Some(existing) = self.option_selects.get_mut(&key) {
                if existing.snapshot != group {
                    let selected_index = group
                        .choices
                        .iter()
                        .position(|choice| choice.value == group.selected)
                        .map(|index| IndexPath::default().row(index));
                    existing.state.update(cx, |state, cx| {
                        state.set_items(SearchableVec::new(group.choices.clone()), window, cx);
                        state.set_selected_index(selected_index, window, cx);
                    });
                    existing.snapshot = group;
                }
                continue;
            }
            let selected_index = group
                .choices
                .iter()
                .position(|choice| choice.value == group.selected)
                .map(|index| IndexPath::default().row(index));
            let state = cx.new(|cx| {
                SelectState::new(
                    SearchableVec::new(group.choices.clone()),
                    selected_index,
                    window,
                    cx,
                )
                .searchable(group.choices.len() > 8)
            });
            let model_id = model.id.clone();
            let option_key = group.key.clone();
            let subscription = cx.subscribe_in(
                &state,
                window,
                move |_,
                      state,
                      event: &SelectEvent<SearchableVec<GenerationChoice>>,
                      window,
                      cx| {
                    let SelectEvent::Confirm(Some(value)) = event else {
                        return;
                    };
                    cx.emit(GenerationAction::OptionSelected {
                        model_id: model_id.clone(),
                        key: option_key.clone(),
                        value: value.clone(),
                    });
                    let state = state.clone();
                    let model_id = model_id.clone();
                    let option_key = option_key.clone();
                    cx.defer_in(window, move |this, window, cx| {
                        let controlled = this
                            .selected_model()
                            .filter(|model| model.id == model_id)
                            .and_then(|model| {
                                model
                                    .option_groups
                                    .iter()
                                    .find(|group| group.key == option_key)
                            })
                            .map(|group| group.selected.clone());
                        state.update(cx, |state, cx| {
                            if let Some(value) = controlled.as_ref() {
                                state.set_selected_value(value, window, cx);
                            } else {
                                state.set_selected_index(None, window, cx);
                            }
                        });
                    });
                },
            );
            self.option_selects.insert(
                key,
                OptionSelectState {
                    state,
                    snapshot: group,
                    _subscription: subscription,
                },
            );
        }
    }

    fn model_select_items(kind: GenerationKind, data: &GenerationViewData) -> Vec<ModelSelectItem> {
        data.models
            .iter()
            .filter(|model| model.recipe.kind() == kind)
            .filter(|model| {
                data.selected_recipe
                    .is_none_or(|recipe| model.recipe == recipe)
            })
            .map(|model| ModelSelectItem {
                id: model.id.clone(),
                label: model.label.clone(),
                provider: model.provider_label.clone(),
                logo: model.logo.clone(),
                recipe: model.recipe.label(),
                description: model.description.clone(),
                cost: model.cost_label(),
            })
            .collect()
    }

    pub fn set_view_data(
        &mut self,
        data: GenerationViewData,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let recipe_changed = self.data.selected_recipe != data.selected_recipe;
        let selection_changed =
            recipe_changed || self.data.selected_model_id != data.selected_model_id;
        let model_picker_changed = self.data.models != data.models
            || self.data.selected_model_id != data.selected_model_id
            || self.data.selected_recipe != data.selected_recipe;
        let selected_model = Self::selected_model_for(self.kind, &data);
        let draft_context_changed = self.data.selected_model_id != data.selected_model_id
            || self.selected_model() != selected_model
            || self.data.source.as_ref().map(|source| &source.id)
                != data.source.as_ref().map(|source| &source.id)
            || self.data.end_frame.as_ref().map(|source| &source.id)
                != data.end_frame.as_ref().map(|source| &source.id)
            || self
                .data
                .voice_reference
                .as_ref()
                .map(|reference| &reference.id)
                != data.voice_reference.as_ref().map(|reference| &reference.id)
            || self.data.voice_consent_granted != data.voice_consent_granted;
        self.data = data;
        if selection_changed {
            self.show_advanced = true;
        }
        self.sync_option_selects(window, cx);
        if recipe_changed {
            let placeholder = Self::prompt_hint(self.kind, self.data.selected_recipe);
            self.prompt.update(cx, |input, cx| {
                input.set_placeholder(placeholder, window, cx)
            });
        }
        if self.lightbox_output_id.as_ref().is_some_and(|id| {
            !self
                .data
                .outputs
                .iter()
                .any(|output| &output.id == id && output.kind == self.kind)
        }) {
            self.lightbox_output_id = None;
        }
        if model_picker_changed {
            let items = Self::model_select_items(self.kind, &self.data);
            let selected_id = self.data.selected_model_id.clone();
            self.model_select.update(cx, |select, cx| {
                select.set_items(SearchableVec::new(items), window, cx);
                if let Some(id) = selected_id.as_ref() {
                    select.set_selected_value(id, window, cx);
                } else {
                    select.set_selected_index(None, window, cx);
                }
            });
        }
        if draft_context_changed {
            self.validation_error = None;
        }
        cx.notify();
    }

    pub fn kind(&self) -> GenerationKind {
        self.kind
    }

    pub fn draft(&self, cx: &App) -> GenerationDraft {
        GenerationDraft {
            prompt: self.prompt.read(cx).value(),
            negative: self.negative.read(cx).value(),
            seed: self.seed.read(cx).value(),
        }
    }

    pub fn set_draft(
        &mut self,
        draft: GenerationDraft,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.prompt
            .update(cx, |input, cx| input.set_value(draft.prompt, window, cx));
        self.negative
            .update(cx, |input, cx| input.set_value(draft.negative, window, cx));
        self.seed
            .update(cx, |input, cx| input.set_value(draft.seed, window, cx));
        self.validation_error = None;
        cx.notify();
    }

    fn prompt_hint(kind: GenerationKind, recipe: Option<GenerationRecipe>) -> &'static str {
        match recipe {
            Some(GenerationRecipe::ImageVideo) => "Describe how the start frame should move…",
            Some(GenerationRecipe::Speech) => "Write the words to speak…",
            Some(GenerationRecipe::Music) => "Describe the mood, rhythm, and instruments…",
            Some(GenerationRecipe::Vectorize) => "Describe the line-art style…",
            _ => match kind {
                GenerationKind::Image => "Describe the image you want to create…",
                GenerationKind::Video => "Describe the scene and motion…",
                GenerationKind::Audio => "Enter spoken text or describe the music…",
                GenerationKind::Svg => "Describe the vector artwork…",
            },
        }
    }

    fn selected_model(&self) -> Option<&GenerationModel> {
        Self::selected_model_for(self.kind, &self.data)
    }

    fn selected_output(&self) -> Option<&GenerationOutput> {
        self.data
            .selected_output_id
            .as_ref()
            .and_then(|id| self.data.outputs.iter().find(|output| &output.id == id))
            .filter(|output| output.kind == self.kind)
    }

    fn open_lightbox(&mut self, id: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        if !self
            .data
            .outputs
            .iter()
            .any(|output| output.id == id && output.kind == self.kind)
        {
            return;
        }
        self.lightbox_output_id = Some(id.clone());
        cx.emit(GenerationAction::OutputSelected { id });
        self.lightbox_focus_handle.focus(window, cx);
        cx.notify();
    }

    fn close_lightbox(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.lightbox_output_id.take() {
            cx.emit(GenerationAction::PreviewClosed { id });
        }
        self.focus_handle.focus(window, cx);
        cx.notify();
    }

    fn step_lightbox(&mut self, direction: isize, cx: &mut Context<Self>) {
        let ids = self
            .data
            .outputs
            .iter()
            .filter(|output| output.kind == self.kind)
            .map(|output| output.id.clone())
            .collect::<Vec<_>>();
        let Some(current) = self
            .lightbox_output_id
            .as_ref()
            .and_then(|id| ids.iter().position(|candidate| candidate == id))
        else {
            return;
        };
        let next = current as isize + direction;
        if next < 0 || next >= ids.len() as isize {
            return;
        }
        let next = next as usize;
        let id = ids[next].clone();
        cx.emit(GenerationAction::PreviewClosed {
            id: ids[current].clone(),
        });
        self.lightbox_output_id = Some(id.clone());
        cx.emit(GenerationAction::OutputSelected { id });
        cx.notify();
    }

    fn build_submission(&self, cx: &App) -> Result<GenerationSubmission, &'static str> {
        let model = self.selected_model().ok_or("Choose a model to continue.")?;
        let prompt = self.prompt.read(cx).value();
        let requires_source = model.requires_source || model.recipe.requires_source();
        if requires_source && self.data.source.is_none() {
            return Err("Choose a source image to continue.");
        }
        if model.supports_voice_reference
            && self.data.voice_reference.is_some()
            && !self.data.voice_consent_granted
        {
            return Err("Confirm permission to use the reference voice.");
        }
        let seed_text = self.seed.read(cx).value();
        let seed = if model.supports_seed && !seed_text.trim().is_empty() {
            Some(
                seed_text
                    .trim()
                    .parse()
                    .map_err(|_| "Seed must be a whole number, starting at zero.")?,
            )
        } else {
            None
        };
        let negative_text = self.negative.read(cx).value();
        let negative =
            (model.supports_negative && !negative_text.trim().is_empty()).then_some(negative_text);
        let options = model
            .option_groups
            .iter()
            .filter(|group| {
                group
                    .choices
                    .iter()
                    .any(|choice| choice.value == group.selected)
            })
            .map(|group| GenerationOptionSelection {
                key: group.key.clone(),
                value: group.selected.clone(),
            })
            .collect();
        let submission = GenerationSubmission {
            kind: self.kind,
            recipe: model.recipe,
            model_id: model.id.clone(),
            prompt: if model.supports_prompt {
                prompt
            } else {
                "".into()
            },
            negative,
            seed,
            source_id: if requires_source {
                self.data.source.as_ref().map(|source| source.id.clone())
            } else {
                None
            },
            end_frame_id: if model.supports_end_frame {
                self.data.end_frame.as_ref().map(|frame| frame.id.clone())
            } else {
                None
            },
            voice_reference_id: if model.supports_voice_reference {
                self.data
                    .voice_reference
                    .as_ref()
                    .map(|reference| reference.id.clone())
            } else {
                None
            },
            voice_consent_granted: model.supports_voice_reference
                && self.data.voice_consent_granted,
            options,
        };
        submission
            .validate_against(model)
            .map_err(|error| error.message())?;
        Ok(submission)
    }

    fn submit(&mut self, cx: &mut Context<Self>) {
        if self.data.busy {
            return;
        }
        match self.build_submission(cx) {
            Ok(submission) => {
                self.validation_error = None;
                cx.emit(GenerationAction::GenerateRequested(submission));
            }
            Err(message) => self.validation_error = Some(message.into()),
        }
        cx.notify();
    }

    fn apply_template(&mut self, id: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(template) = self
            .data
            .templates
            .iter()
            .find(|template| template.id == id)
        {
            self.prompt.update(cx, |input, cx| {
                input.set_value(template.prompt.clone(), window, cx)
            });
            self.validation_error = None;
            cx.emit(GenerationAction::TemplateSelected { id });
            cx.notify();
        }
    }

    fn reuse_prompt(&mut self, id: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(prompt) = self
            .data
            .outputs
            .iter()
            .find(|output| output.id == id)
            .and_then(|output| output.prompt.clone())
        {
            self.prompt
                .update(cx, |input, cx| input.set_value(prompt, window, cx));
        }
        cx.emit(GenerationAction::ReusePromptRequested { id });
        cx.notify();
    }

    fn control(
        &self,
        suffix: impl AsRef<str>,
        label: impl Into<SharedString>,
        appearance: ControlAppearance,
        colors: GenerationStyle,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> AnyElement {
        let label = label.into();
        let selected = matches!(
            appearance,
            ControlAppearance::Selected | ControlAppearance::Toggled(true)
        );
        let prominent = matches!(appearance, ControlAppearance::Prominent);
        let enabled = !matches!(appearance, ControlAppearance::Disabled);
        let toggle = match appearance {
            ControlAppearance::Toggled(true) => Some(Toggled::True),
            ControlAppearance::Toggled(false) => Some(Toggled::False),
            _ => None,
        };
        let base = div()
            .id(SharedString::from(format!(
                "{}-{}",
                self.id,
                suffix.as_ref()
            )))
            .role(if toggle.is_some() {
                Role::CheckBox
            } else {
                Role::Button
            })
            .aria_label(label.clone())
            .when(
                matches!(appearance, ControlAppearance::Selected),
                |control| control.aria_selected(true),
            )
            .when_some(toggle, |control, state| control.aria_toggled(state))
            .debug_selector(|| format!("generation-{}", suffix.as_ref()))
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(if enabled { 0 } else { -1 })
            .min_h(px(if prominent { 42. } else { 29. }))
            .px(px(if prominent { 15. } else { 10. }))
            .py(px(5.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(5.))
            .border_1()
            .border_color(if prominent {
                colors.accent
            } else if selected {
                colors.focus
            } else {
                colors.border
            })
            .bg(if prominent {
                colors.accent
            } else if selected {
                colors.selected
            } else {
                colors.surface
            })
            .text_color(if !enabled {
                colors.muted
            } else if prominent {
                colors.background
            } else if selected {
                colors.accent
            } else {
                colors.text
            })
            .text_size(px(12.))
            .font_weight(if prominent {
                gpui::FontWeight::SEMIBOLD
            } else {
                gpui::FontWeight::NORMAL
            })
            .child(label);
        if enabled {
            base.cursor_pointer()
                .hover(|style| {
                    style.bg(if prominent {
                        colors.accent.opacity(0.85)
                    } else {
                        colors.hover
                    })
                })
                .focus(|style| style.border_color(colors.focus))
                .on_activate(move |_, window, cx| handler(window, cx))
                .into_any_element()
        } else {
            base.into_any_element()
        }
    }

    fn field(
        &self,
        selector: &'static str,
        label: impl Into<SharedString>,
        state: &Entity<InputState>,
        height: f32,
        colors: GenerationStyle,
    ) -> AnyElement {
        div()
            .debug_selector(move || selector.to_owned())
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(colors.muted)
                    .child(label.into()),
            )
            .child(
                div()
                    .h(px(height))
                    .rounded_sm()
                    .border_1()
                    .border_color(colors.border)
                    .bg(colors.surface)
                    .child(
                        Input::new(state)
                            .appearance(false)
                            .bordered(false)
                            .focus_bordered(false)
                            .w_full()
                            .h_full(),
                    ),
            )
            .into_any_element()
    }

    fn render_header(&self, colors: GenerationStyle, cx: &mut Context<Self>) -> AnyElement {
        let mut tabs = div().flex().gap(px(24.));
        for kind in GenerationKind::ALL {
            let screen = cx.entity();
            let selected = kind == self.kind;
            tabs = tabs.child(
                div()
                    .id(SharedString::from(format!("{}-kind-{kind:?}", self.id)))
                    .role(Role::Tab)
                    .aria_label(kind.label())
                    .aria_selected(selected)
                    .key_context(CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .py(px(11.))
                    .border_b_1()
                    .border_color(if selected {
                        colors.accent
                    } else {
                        colors.background
                    })
                    .text_size(px(12.))
                    .font_weight(if selected {
                        gpui::FontWeight::SEMIBOLD
                    } else {
                        gpui::FontWeight::NORMAL
                    })
                    .text_color(if selected { colors.text } else { colors.muted })
                    .cursor_pointer()
                    .hover(|style| style.text_color(colors.text))
                    .focus(|style| style.border_color(colors.focus))
                    .child(kind.label())
                    .on_activate(move |_, _, cx| {
                        screen.update(cx, |_, cx| cx.emit(GenerationAction::KindSelected(kind)));
                    }),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap(px(14.))
            .px(px(28.))
            .pt(px(22.))
            .border_b_1()
            .border_color(colors.border)
            .bg(colors.background)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(5.))
                    .child(
                        div()
                            .text_size(px(10.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(colors.accent)
                            .child("FANTA  /  GENERATIVE STUDIO"),
                    )
                    .child(
                        div()
                            .text_size(px(25.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(colors.text)
                            .child(match self.kind {
                                GenerationKind::Image => "Create images",
                                GenerationKind::Video => "Create video",
                                GenerationKind::Audio => "Create audio",
                                GenerationKind::Svg => "Create SVG vectors",
                            }),
                    )
                    .child(
                        div().text_size(px(11.)).text_color(colors.muted).child(
                            "Choose a model, shape the prompt, and explore what you create.",
                        ),
                    ),
            )
            .child(tabs)
            .into_any_element()
    }

    fn render_operation_picker(
        &self,
        colors: GenerationStyle,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let candidates: &[GenerationRecipe] = match self.kind {
            GenerationKind::Image => &[GenerationRecipe::TextImage],
            GenerationKind::Video => &[GenerationRecipe::TextVideo, GenerationRecipe::ImageVideo],
            GenerationKind::Audio => &[GenerationRecipe::Speech, GenerationRecipe::Music],
            GenerationKind::Svg => &[
                GenerationRecipe::PromptSvg,
                GenerationRecipe::ImageSvg,
                GenerationRecipe::Vectorize,
            ],
        };
        let available = candidates
            .iter()
            .copied()
            .filter(|recipe| self.data.models.iter().any(|model| model.recipe == *recipe))
            .collect::<Vec<_>>();
        if available.len() < 2 {
            return None;
        }
        let selected = self
            .data
            .selected_recipe
            .or_else(|| self.selected_model().map(|model| model.recipe));
        let mut cards = div().flex().flex_wrap().gap(px(8.));
        for recipe in available {
            let active = selected == Some(recipe);
            let description = match recipe {
                GenerationRecipe::TextVideo => "Describe a scene",
                GenerationRecipe::ImageVideo => "Animate a start frame",
                GenerationRecipe::Speech => "Create spoken audio",
                GenerationRecipe::Music => "Compose from a prompt",
                GenerationRecipe::PromptSvg => "Generate vector art",
                GenerationRecipe::ImageSvg => "Image to SVG paths",
                GenerationRecipe::Vectorize => "Trace source artwork",
                GenerationRecipe::TextImage => "Generate from text",
            };
            let screen = cx.entity();
            cards = cards.child(
                div()
                    .id(SharedString::from(format!("{}-recipe-{recipe:?}", self.id)))
                    .role(Role::Button)
                    .aria_label(recipe.label())
                    .aria_selected(active)
                    .debug_selector(|| format!("generation-recipe-{recipe:?}"))
                    .key_context(CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .w(px(164.))
                    .h(px(62.))
                    .flex_none()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap(px(5.))
                    .px(px(11.))
                    .rounded(px(5.))
                    .border_1()
                    .border_color(if active { colors.focus } else { colors.border })
                    .bg(if active {
                        colors.selected
                    } else {
                        colors.surface
                    })
                    .cursor_pointer()
                    .hover(|style| style.bg(colors.hover))
                    .focus(|style| style.border_color(colors.focus))
                    .child(
                        div()
                            .text_size(px(12.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(if active { colors.accent } else { colors.text })
                            .child(recipe.label()),
                    )
                    .child(
                        div()
                            .text_size(px(10.))
                            .text_color(colors.muted)
                            .child(description),
                    )
                    .on_activate(move |_, _, cx| {
                        screen.update(cx, |_, cx| {
                            cx.emit(GenerationAction::RecipeSelected(recipe))
                        });
                    }),
            );
        }
        Some(
            div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(self.section_label("OPERATION", colors))
                .child(cards)
                .into_any_element(),
        )
    }

    fn render_model_picker(&self, colors: GenerationStyle, _: &mut Context<Self>) -> AnyElement {
        let count = self
            .data
            .models
            .iter()
            .filter(|model| model.recipe.kind() == self.kind)
            .filter(|model| {
                self.data
                    .selected_recipe
                    .is_none_or(|recipe| model.recipe == recipe)
            })
            .count();
        let detail = if let Some(model) = self.selected_model() {
            format!("{}  ·  {}", model.provider_label, model.description)
        } else if count == 0 {
            "No models available. Refresh the catalog.".to_owned()
        } else {
            "Choose a model to see its generation settings.".to_owned()
        };
        let advanced_hint = self.selected_model().and_then(|model| {
            let count = model
                .option_groups
                .iter()
                .filter(|group| group.advanced)
                .count()
                + usize::from(model.supports_negative)
                + usize::from(model.supports_seed);
            (count > 0).then(|| {
                if self.show_advanced {
                    format!("{count} advanced controls open below")
                } else {
                    format!("{count} advanced controls available below")
                }
            })
        });
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(self.section_label("MODEL", colors))
                    .child(
                        div()
                            .text_size(px(10.))
                            .text_color(colors.muted)
                            .child(format!("{count} available")),
                    ),
            )
            .child(
                div()
                    .id(SharedString::from(format!("{}-model-select", self.id)))
                    .debug_selector(|| "generation-model-select".to_owned())
                    .w_full()
                    .child(
                        Select::new(&self.model_select)
                            .w_full()
                            .h(px(40.))
                            .menu_width(px(372.))
                            .placeholder("Choose a model")
                            .search_placeholder("Search models…")
                            .disabled(count == 0),
                    ),
            )
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(colors.muted)
                    .child(detail),
            )
            .when_some(advanced_hint, |picker, hint| {
                picker.child(
                    div()
                        .text_size(px(10.))
                        .text_color(colors.accent)
                        .child(hint),
                )
            })
            .into_any_element()
    }

    fn section_label(&self, label: &'static str, colors: GenerationStyle) -> AnyElement {
        div()
            .text_size(px(10.))
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(colors.muted)
            .child(label)
            .into_any_element()
    }

    fn render_source(&self, colors: GenerationStyle, cx: &mut Context<Self>) -> AnyElement {
        let screen = cx.entity();
        let source = self.data.source.as_ref();
        let mut content = div()
            .flex()
            .items_center()
            .gap(px(10.))
            .p(px(10.))
            .rounded_sm()
            .border_1()
            .border_color(colors.border)
            .bg(colors.surface);
        if let Some(source) = source {
            content = content.child(self.preview(
                &source.preview,
                GenerationKind::Image,
                44.,
                ObjectFit::Cover,
                colors,
            ));
            content = content.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_size(px(12.))
                    .text_color(colors.text)
                    .child(source.name.clone()),
            );
            let screen_for_clear = cx.entity();
            content = content.child(self.control(
                "clear-source",
                "Remove",
                ControlAppearance::Normal,
                colors,
                move |_, cx| {
                    screen_for_clear.update(cx, |_, cx| cx.emit(GenerationAction::SourceCleared));
                },
            ));
        } else {
            content = content
                .child(
                    div()
                        .flex_1()
                        .text_size(px(12.))
                        .text_color(colors.muted)
                        .child("No source image selected"),
                )
                .child(self.control(
                    "choose-source",
                    "Choose image",
                    ControlAppearance::Normal,
                    colors,
                    move |_, cx| {
                        screen.update(cx, |_, cx| cx.emit(GenerationAction::SourceRequested));
                    },
                ));
        }
        div()
            .flex()
            .flex_col()
            .gap(px(7.))
            .child(self.section_label("SOURCE IMAGE", colors))
            .child(content)
            .into_any_element()
    }

    fn render_frame_card(
        &self,
        source: Option<&GenerationSource>,
        end_frame: bool,
        enabled: bool,
        note: &'static str,
        colors: GenerationStyle,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let stage = div()
            .w_full()
            .h(px(82.))
            .rounded_sm()
            .border_1()
            .border_color(colors.border)
            .bg(colors.background)
            .overflow_hidden()
            .flex()
            .items_center()
            .justify_center();
        let stage = if let Some(preview) = source.and_then(|source| source.preview.as_ref()) {
            stage.child(
                img(preview.clone())
                    .w_full()
                    .h_full()
                    .object_fit(ObjectFit::Cover),
            )
        } else {
            stage.child(
                div()
                    .text_size(px(11.))
                    .text_color(colors.muted)
                    .child(if end_frame {
                        "End frame"
                    } else {
                        "Start frame"
                    }),
            )
        };
        let mut card = div()
            .w(px(164.))
            .flex_none()
            .flex()
            .flex_col()
            .gap(px(8.))
            .p(px(9.))
            .rounded(px(5.))
            .border_1()
            .border_color(colors.border)
            .bg(colors.surface)
            .child(
                div()
                    .text_size(px(11.))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors.text)
                    .child(if end_frame {
                        "END FRAME"
                    } else {
                        "START FRAME"
                    }),
            )
            .child(stage);
        if let Some(source) = source {
            card = card.child(
                div()
                    .text_size(px(10.))
                    .text_color(colors.muted)
                    .child(source.name.clone()),
            );
            let screen = cx.entity();
            card = card.child(self.control(
                if end_frame {
                    "end-remove"
                } else {
                    "start-remove"
                },
                "Remove",
                ControlAppearance::Normal,
                colors,
                move |_, cx| {
                    screen.update(cx, |_, cx| {
                        cx.emit(if end_frame {
                            GenerationAction::EndFrameCleared
                        } else {
                            GenerationAction::SourceCleared
                        });
                    });
                },
            ));
        } else if enabled {
            let screen = cx.entity();
            card = card.child(self.control(
                if end_frame {
                    "end-choose"
                } else {
                    "start-choose"
                },
                "Choose image",
                ControlAppearance::Normal,
                colors,
                move |_, cx| {
                    screen.update(cx, |_, cx| {
                        cx.emit(if end_frame {
                            GenerationAction::EndFrameRequested
                        } else {
                            GenerationAction::SourceRequested
                        });
                    });
                },
            ));
        } else {
            card = card.child(
                div()
                    .text_size(px(10.))
                    .text_color(colors.muted)
                    .child(note),
            );
        }
        card.into_any_element()
    }

    fn render_video_frames(
        &self,
        model: &GenerationModel,
        colors: GenerationStyle,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let has_start = self.data.source.is_some();
        let end_available = model.supports_end_frame && has_start;
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(self.section_label("FRAMES", colors))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .child(self.render_frame_card(
                        self.data.source.as_ref(),
                        false,
                        true,
                        "",
                        colors,
                        cx,
                    ))
                    .child(self.render_frame_card(
                        self.data.end_frame.as_ref(),
                        true,
                        end_available,
                        if model.supports_end_frame {
                            "Choose a start frame first"
                        } else {
                            "Unavailable for this model"
                        },
                        colors,
                        cx,
                    )),
            )
            .into_any_element()
    }

    fn render_option_group(
        &self,
        model_id: SharedString,
        group: &GenerationOptionGroup,
        colors: GenerationStyle,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if group.choices.len() > 3
            && let Some(select) = self
                .option_selects
                .get(&(model_id.clone(), group.key.clone()))
        {
            return div()
                .flex()
                .flex_col()
                .gap(px(7.))
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(colors.muted)
                        .child(group.label.clone()),
                )
                .child(
                    div()
                        .id(SharedString::from(format!(
                            "{}-option-select-{}-{}",
                            self.id, model_id, group.key
                        )))
                        .debug_selector(|| "generation-option-select".to_owned())
                        .w_full()
                        .child(
                            Select::new(&select.state)
                                .w_full()
                                .h(px(36.))
                                .menu_width(px(312.))
                                .placeholder("Choose an option")
                                .search_placeholder("Search options…"),
                        ),
                )
                .into_any_element();
        }
        let mut choices = div().flex().flex_wrap().gap(px(6.));
        for choice in &group.choices {
            let screen = cx.entity();
            let model_id = model_id.clone();
            let key = group.key.clone();
            let value = choice.value.clone();
            let selected = group.selected == value;
            choices = choices.child(self.control(
                format!("option-{}-{}-{}", model_id, key, value),
                choice.label.clone(),
                if selected {
                    ControlAppearance::Selected
                } else {
                    ControlAppearance::Normal
                },
                colors,
                move |_, cx| {
                    screen.update(cx, |_, cx| {
                        cx.emit(GenerationAction::OptionSelected {
                            model_id: model_id.clone(),
                            key: key.clone(),
                            value: value.clone(),
                        })
                    });
                },
            ));
        }
        div()
            .flex()
            .flex_col()
            .gap(px(7.))
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(colors.muted)
                    .child(group.label.clone()),
            )
            .child(choices)
            .into_any_element()
    }

    fn render_voice_reference(
        &self,
        colors: GenerationStyle,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let screen = cx.entity();
        let mut field = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .p(px(9.))
            .rounded_sm()
            .border_1()
            .border_color(colors.border)
            .bg(colors.surface);
        if let Some(reference) = &self.data.voice_reference {
            field = field.child(
                div()
                    .flex_1()
                    .text_size(px(12.))
                    .text_color(colors.text)
                    .child(reference.name.clone()),
            );
            field = field.child(self.control(
                "voice-clear",
                "Remove",
                ControlAppearance::Normal,
                colors,
                move |_, cx| {
                    screen.update(cx, |_, cx| cx.emit(GenerationAction::VoiceReferenceCleared));
                },
            ));
        } else {
            field = field.child(
                div()
                    .flex_1()
                    .text_size(px(11.))
                    .text_color(colors.muted)
                    .child("Optional voice sample"),
            );
            field = field.child(self.control(
                "voice-choose",
                "Choose audio",
                ControlAppearance::Normal,
                colors,
                move |_, cx| {
                    screen.update(cx, |_, cx| {
                        cx.emit(GenerationAction::VoiceReferenceRequested)
                    });
                },
            ));
        }
        let mut section = div()
            .flex()
            .flex_col()
            .gap(px(7.))
            .child(self.section_label("VOICE REFERENCE", colors))
            .child(field);
        if self.data.voice_reference.is_some() {
            let screen = cx.entity();
            let checked = self.data.voice_consent_granted;
            section = section.child(self.control(
                "voice-consent",
                if checked {
                    "Permission confirmed"
                } else {
                    "I have permission to use this voice"
                },
                ControlAppearance::Toggled(checked),
                colors,
                move |_, cx| {
                    screen.update(cx, |_, cx| {
                        cx.emit(GenerationAction::VoiceConsentChanged(!checked))
                    });
                },
            ));
        }
        section.into_any_element()
    }

    fn render_composer(&self, colors: GenerationStyle, cx: &mut Context<Self>) -> AnyElement {
        let mut stack = div().flex().flex_col().gap(px(14.)).p(px(18.));
        if let Some(operation) = self.render_operation_picker(colors, cx) {
            stack = stack.child(operation);
        }
        stack = stack.child(self.render_model_picker(colors, cx));
        if let Some(model) = self.selected_model() {
            if model.supports_prompt {
                let prompt_label = if let Some(limit) = model.max_prompt_chars {
                    format!(
                        "{} · {}/{}",
                        if model.recipe == GenerationRecipe::Speech {
                            "SPOKEN TEXT"
                        } else {
                            "PROMPT"
                        },
                        self.prompt.read(cx).value().encode_utf16().count(),
                        limit
                    )
                } else {
                    (if model.recipe == GenerationRecipe::Speech {
                        "SPOKEN TEXT"
                    } else {
                        "PROMPT"
                    })
                    .to_owned()
                };
                stack = stack.child(self.field(
                    "generation-prompt-field",
                    prompt_label,
                    &self.prompt,
                    96.,
                    colors,
                ));
            }
            if model.recipe == GenerationRecipe::ImageVideo {
                stack = stack.child(self.render_video_frames(model, colors, cx));
            } else if model.requires_source || model.recipe.requires_source() {
                stack = stack.child(self.render_source(colors, cx));
            }
            if model.supports_voice_reference {
                stack = stack.child(self.render_voice_reference(colors, cx));
            }
            for group in model.option_groups.iter().filter(|group| !group.advanced) {
                stack = stack.child(self.render_option_group(model.id.clone(), group, colors, cx));
            }
            if model.option_groups.iter().any(|group| group.advanced)
                || model.supports_negative
                || model.supports_seed
            {
                let screen = cx.entity();
                let expanded = self.show_advanced;
                let advanced_count = model
                    .option_groups
                    .iter()
                    .filter(|group| group.advanced)
                    .count()
                    + usize::from(model.supports_negative)
                    + usize::from(model.supports_seed);
                stack = stack.child(self.control(
                    "advanced-settings",
                    if expanded {
                        format!("Advanced settings · {advanced_count}  ▴")
                    } else {
                        format!("Advanced settings · {advanced_count}  ▾")
                    },
                    if expanded {
                        ControlAppearance::Selected
                    } else {
                        ControlAppearance::Normal
                    },
                    colors,
                    move |_, cx| {
                        screen.update(cx, |this, cx| {
                            this.show_advanced = !this.show_advanced;
                            cx.notify();
                        });
                    },
                ));
                if expanded {
                    for group in model.option_groups.iter().filter(|group| group.advanced) {
                        stack = stack.child(self.render_option_group(
                            model.id.clone(),
                            group,
                            colors,
                            cx,
                        ));
                    }
                    if model.supports_negative {
                        stack = stack.child(self.field(
                            "generation-negative-field",
                            "NEGATIVE PROMPT",
                            &self.negative,
                            72.,
                            colors,
                        ));
                    }
                    if model.supports_seed {
                        stack = stack.child(self.field(
                            "generation-seed-field",
                            "SEED",
                            &self.seed,
                            32.,
                            colors,
                        ));
                    }
                }
            }
        }
        if let Some(error) = self.validation_error.as_ref().or(self.data.error.as_ref()) {
            stack = stack.child(
                div()
                    .p(px(9.))
                    .rounded_sm()
                    .border_1()
                    .border_color(colors.focus)
                    .text_color(colors.text)
                    .text_size(px(11.))
                    .child(error.clone()),
            );
        }
        let screen = cx.entity();
        let submit = self.control(
            "submit",
            if self.data.busy {
                "Generating…"
            } else {
                "Generate"
            },
            if self.selected_model().is_some() && !self.data.busy {
                ControlAppearance::Prominent
            } else {
                ControlAppearance::Disabled
            },
            colors,
            move |_, cx| {
                screen.update(cx, |this, cx| this.submit(cx));
            },
        );
        let cost = self
            .selected_model()
            .and_then(GenerationModel::cost_label)
            .unwrap_or_else(|| "Choose a model for pricing".into());
        let footer = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .p(px(16.))
            .border_t_1()
            .border_color(colors.border)
            .bg(colors.panel)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .gap(px(12.))
                    .child(self.section_label("COST PER OUTPUT", colors))
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(colors.accent)
                            .child(cost),
                    ),
            )
            .child(submit);
        div()
            .w(px(382.))
            .flex_none()
            .h_full()
            .flex()
            .flex_col()
            .bg(colors.panel)
            .border_r_1()
            .border_color(colors.border)
            .child(
                div()
                    .id(SharedString::from(format!("{}-composer-scroll", self.id)))
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.composer_scroll)
                    .child(stack),
            )
            .child(footer)
            .into_any_element()
    }

    fn preview(
        &self,
        image: &Option<std::sync::Arc<Image>>,
        kind: GenerationKind,
        height: f32,
        fit: ObjectFit,
        colors: GenerationStyle,
    ) -> AnyElement {
        let shell = div()
            .w_full()
            .h(px(height))
            .rounded_sm()
            .border_1()
            .border_color(colors.border)
            .bg(colors.surface)
            .overflow_hidden()
            .flex()
            .items_center()
            .justify_center();
        if let Some(image) = image {
            shell
                .child(img(image.clone()).w_full().h_full().object_fit(fit))
                .into_any_element()
        } else {
            shell
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.muted)
                        .child(match kind {
                            GenerationKind::Image => "Image preview",
                            GenerationKind::Video => "Video preview",
                            GenerationKind::Audio => "Audio waveform",
                            GenerationKind::Svg => "SVG preview",
                        }),
                )
                .into_any_element()
        }
    }

    fn render_template(
        &self,
        template: &GenerationTemplate,
        colors: GenerationStyle,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let screen = cx.entity();
        let id = template.id.clone();
        let selected = self.data.selected_template_id.as_ref() == Some(&id);
        let preview = template
            .preview
            .clone()
            .unwrap_or_else(|| template_art::for_template(template));
        div()
            .id(SharedString::from(format!("{}-template-{}", self.id, id)))
            .role(Role::Button)
            .aria_label(template.title.clone())
            .aria_selected(selected)
            .debug_selector(|| format!("generation-template-{}", id))
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .w(px(190.))
            .flex_none()
            .flex()
            .flex_col()
            .gap(px(7.))
            .p(px(8.))
            .rounded(px(5.))
            .border_1()
            .border_color(if selected {
                colors.focus
            } else {
                colors.border
            })
            .bg(colors.surface)
            .cursor_pointer()
            .hover(|style| style.bg(colors.hover))
            .focus(|style| style.border_color(colors.focus))
            .child(self.preview(&Some(preview), template.kind, 96., ObjectFit::Cover, colors))
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(colors.text)
                    .child(template.title.clone()),
            )
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(colors.muted)
                    .child(template.description.clone()),
            )
            .on_activate(move |_, window, cx| {
                screen.update(cx, |this, cx| this.apply_template(id.clone(), window, cx));
            })
            .into_any_element()
    }

    fn render_output_card(
        &self,
        output: &GenerationOutput,
        colors: GenerationStyle,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let screen = cx.entity();
        let id = output.id.clone();
        let selected = self.data.selected_output_id.as_ref() == Some(&id);
        let model_label = self
            .data
            .models
            .iter()
            .find(|model| model.id == output.model_id)
            .map(|model| model.label.clone())
            .unwrap_or_else(|| output.model_id.clone());
        div()
            .id(SharedString::from(format!("{}-output-{}", self.id, id)))
            .role(Role::Button)
            .aria_label(output.title.clone())
            .aria_selected(selected)
            .debug_selector(|| format!("generation-output-{}", id))
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .w(px(215.))
            .flex_none()
            .flex()
            .flex_col()
            .gap(px(7.))
            .p(px(8.))
            .rounded(px(5.))
            .border_1()
            .border_color(if selected {
                colors.focus
            } else {
                colors.border
            })
            .bg(colors.surface)
            .cursor_pointer()
            .hover(|style| style.bg(colors.hover))
            .focus(|style| style.border_color(colors.focus))
            .child(self.preview(&output.preview, output.kind, 144., ObjectFit::Cover, colors))
            .child(
                div()
                    .w_full()
                    .min_w_0()
                    .truncate()
                    .text_size(px(12.))
                    .text_color(colors.text)
                    .child(output.title.clone()),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .justify_between()
                    .gap(px(6.))
                    .text_size(px(10.))
                    .text_color(colors.muted)
                    .child(div().flex_1().min_w_0().truncate().child(model_label))
                    .child(div().flex_none().child(output.status.label())),
            )
            .on_activate(move |_, window, cx| {
                screen.update(cx, |this, cx| this.open_lightbox(id.clone(), window, cx));
            })
            .into_any_element()
    }

    fn render_selected_output(
        &self,
        colors: GenerationStyle,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let output = self.selected_output()?;
        let model_label = self
            .data
            .models
            .iter()
            .find(|model| model.id == output.model_id)
            .map(|model| model.label.clone())
            .unwrap_or_else(|| output.model_id.clone());
        let id_for_download = output.id.clone();
        let id_for_add = output.id.clone();
        let id_for_play = output.id.clone();
        let id_for_prompt = output.id.clone();
        let id_for_open = output.id.clone();
        let screen_for_download = cx.entity();
        let screen_for_add = cx.entity();
        let screen_for_play = cx.entity();
        let screen_for_prompt = cx.entity();
        let screen_for_open = cx.entity();
        let mut actions = div().flex().flex_wrap().gap(px(6.));
        actions = actions.child(self.control(
            "open-selected",
            "Open preview",
            if output.can_add_to_project {
                ControlAppearance::Normal
            } else {
                ControlAppearance::Prominent
            },
            colors,
            move |window, cx| {
                screen_for_open.update(cx, |this, cx| {
                    this.open_lightbox(id_for_open.clone(), window, cx)
                });
            },
        ));
        if output.status == GenerationOutputStatus::Succeeded {
            if output.can_add_to_project {
                actions = actions.child(self.control(
                    "add-selected-to-project",
                    "Add to project",
                    ControlAppearance::Prominent,
                    colors,
                    move |_, cx| {
                        screen_for_add.update(cx, |_, cx| {
                            cx.emit(GenerationAction::AddToProjectRequested {
                                id: id_for_add.clone(),
                            })
                        });
                    },
                ));
            }
            if matches!(output.kind, GenerationKind::Audio | GenerationKind::Video) {
                actions = actions.child(self.control(
                    "play-selected",
                    "Play",
                    ControlAppearance::Normal,
                    colors,
                    move |window, cx| {
                        screen_for_play.update(cx, |this, cx| {
                            this.open_lightbox(id_for_play.clone(), window, cx);
                            cx.emit(GenerationAction::PlayRequested {
                                id: id_for_play.clone(),
                            });
                        });
                    },
                ));
            }
            actions = actions.child(self.control(
                "download-selected",
                "Download",
                ControlAppearance::Normal,
                colors,
                move |_, cx| {
                    screen_for_download.update(cx, |_, cx| {
                        cx.emit(GenerationAction::DownloadRequested {
                            id: id_for_download.clone(),
                        })
                    });
                },
            ));
        }
        if output.prompt.is_some() {
            actions = actions.child(self.control(
                "reuse-selected",
                "Use prompt",
                ControlAppearance::Normal,
                colors,
                move |window, cx| {
                    screen_for_prompt.update(cx, |this, cx| {
                        this.reuse_prompt(id_for_prompt.clone(), window, cx)
                    });
                },
            ));
        }
        let prompt_excerpt = output.prompt.as_ref().map(|prompt| {
            let mut chars = prompt.chars();
            let excerpt: String = chars.by_ref().take(120).collect();
            if chars.next().is_some() {
                format!("{excerpt}…")
            } else {
                excerpt
            }
        });
        let information = div()
            .min_w(px(220.))
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(11.))
            .child(self.section_label("SELECTED CREATION", colors))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .gap(px(16.))
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(colors.text)
                            .child(output.title.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(colors.muted)
                            .child(output.status.label()),
                    ),
            )
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(colors.muted)
                    .child(format!("{}  ·  {}", model_label, output.created_at)),
            )
            .when_some(prompt_excerpt, |information, prompt| {
                information.child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.muted)
                        .child(prompt),
                )
            })
            .child(div().flex_1())
            .child(actions);
        let detail = div()
            .flex()
            .flex_wrap()
            .gap(px(18.))
            .p(px(14.))
            .rounded(px(6.))
            .border_1()
            .border_color(colors.border)
            .bg(colors.panel)
            .child(div().w(px(330.)).flex_none().child(self.preview(
                &output.preview,
                output.kind,
                208.,
                if matches!(output.kind, GenerationKind::Image | GenerationKind::Video) {
                    ObjectFit::Cover
                } else {
                    ObjectFit::Contain
                },
                colors,
            )))
            .child(information);
        Some(detail.into_any_element())
    }

    fn render_gallery(&self, colors: GenerationStyle, cx: &mut Context<Self>) -> AnyElement {
        let selected_recipe = self
            .data
            .selected_recipe
            .or_else(|| self.selected_model().map(|model| model.recipe));
        let templates = self
            .data
            .templates
            .iter()
            .filter(|template| template.kind == self.kind)
            .filter(|template| {
                selected_recipe.is_none_or(|recipe| {
                    template.model_id.as_ref().is_none_or(|id| {
                        self.data
                            .models
                            .iter()
                            .any(|model| &model.id == id && model.recipe == recipe)
                    })
                })
            });
        let outputs = self
            .data
            .outputs
            .iter()
            .filter(|output| output.kind == self.kind);
        let screen = cx.entity();
        let mut template_cards = div().flex().flex_wrap().gap(px(9.));
        let mut template_count = 0;
        for template in templates {
            template_count += 1;
            template_cards = template_cards.child(self.render_template(template, colors, cx));
        }
        let mut output_cards = div().flex().flex_wrap().gap(px(9.));
        let mut output_count = 0;
        for output in outputs {
            output_count += 1;
            output_cards = output_cards.child(self.render_output_card(output, colors, cx));
        }
        let mut content = div().flex().flex_col().gap(px(20.)).p(px(26.)).child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap(px(16.))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .child(
                            div()
                                .text_size(px(21.))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(colors.text)
                                .child("Your creations"),
                        )
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(colors.muted)
                                .child(format!("{output_count} recent creations")),
                        ),
                )
                .child(self.control(
                    "refresh",
                    "Refresh",
                    ControlAppearance::Normal,
                    colors,
                    move |_, cx| {
                        screen.update(cx, |_, cx| cx.emit(GenerationAction::RefreshRequested));
                    },
                )),
        );
        if output_count == 0 {
            content = content.child(
                div()
                    .h(px(150.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_sm()
                    .border_1()
                    .border_color(colors.border)
                    .bg(colors.surface)
                    .text_size(px(12.))
                    .text_color(colors.muted)
                    .child(format!(
                        "Your generated {} will appear here.",
                        self.kind.label().to_lowercase()
                    )),
            );
        } else {
            if let Some(selected) = self.render_selected_output(colors, cx) {
                content = content.child(selected);
            }
            content = content.child(self.section_label("RECENT WORK", colors));
            content = content.child(output_cards);
        }
        if self.data.has_more {
            let screen = cx.entity();
            content = content.child(self.control(
                "load-more",
                "Load more creations",
                ControlAppearance::Normal,
                colors,
                move |_, cx| {
                    screen.update(cx, |_, cx| cx.emit(GenerationAction::LoadMoreRequested));
                },
            ));
        }
        if template_count > 0 {
            content = content
                .child(
                    div()
                        .border_t_1()
                        .border_color(colors.border)
                        .pt(px(22.))
                        .child(self.section_label("START WITH A TEMPLATE", colors)),
                )
                .child(template_cards);
        }
        div()
            .id(SharedString::from(format!("{}-gallery-scroll", self.id)))
            .flex_1()
            .min_w(px(400.))
            .h_full()
            .bg(colors.background)
            .overflow_y_scroll()
            .track_scroll(&self.gallery_scroll)
            .child(content)
            .into_any_element()
    }
}

impl Focusable for GenerationScreen {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for GenerationScreen {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = GenerationStyle::current(cx);
        let root = div()
            .id(self.id.clone())
            .relative()
            .key_context(GENERATION_SCREEN_KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
            .min_w(px(GENERATION_SCREEN_MIN_WIDTH))
            .min_h(px(GENERATION_SCREEN_MIN_HEIGHT))
            .flex()
            .flex_col()
            .bg(colors.background)
            .text_color(colors.text);
        let root = if let Some(lightbox) = self.render_lightbox(colors, cx) {
            root.child(lightbox)
        } else {
            let body = div()
                .id(SharedString::from(format!("{}-body", self.id)))
                .flex()
                .flex_wrap()
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .child(self.render_composer(colors, cx))
                .child(self.render_gallery(colors, cx));
            root.child(self.render_header(colors, cx)).child(body)
        };
        if cx.try_global::<theme::GlobalTheme>().is_some() {
            root.font_ui(cx)
        } else {
            root
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_model_matches_recipe_when_id_is_shared() {
        let data = GenerationViewData {
            models: vec![
                GenerationModel::new("ltx-video", "LTX", GenerationRecipe::TextVideo),
                GenerationModel::new("ltx-video", "LTX", GenerationRecipe::ImageVideo),
            ],
            selected_recipe: Some(GenerationRecipe::ImageVideo),
            selected_model_id: Some("ltx-video".into()),
            ..Default::default()
        };

        let selected = GenerationScreen::selected_model_for(GenerationKind::Video, &data);
        assert_eq!(
            selected.map(|model| model.recipe),
            Some(GenerationRecipe::ImageVideo)
        );
        assert!(GenerationScreen::selected_model_for(GenerationKind::Audio, &data).is_none());
    }
}
