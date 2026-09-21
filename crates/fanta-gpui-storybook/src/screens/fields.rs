//! The Inspector fields molecule story.
//!
//! Every controlled field kind in `molecules::inspector::fields` mounted in
//! one property grid; the value-state by access matrix each of them obeys;
//! the multi-selection fold that actually produces `InspectorValue::Mixed`
//! rather than a cell that merely says so; and a live log of the
//! `InspectorEditPhase` lifecycle, so Begin -> Preview -> Commit is
//! something the reader watches arrive under their own pointer instead of
//! something the doc comments assert.

use fanta_gpui::atoms::TypographyExt as _;
use gpui::{Bounds, FocusHandle, Hsla, Pixels};

use crate::*;

use super::{
    knobs::{self, KnobOption},
    spec::{NamedState, state_matrix},
    specimen::{framed_panel, specimen_card, specimen_row, specimen_rows, specimen_story_root},
};

/// The docked rail widths the width knob offers. 360 px is the library's
/// compact breakpoint, so the widest preset is the last width that still
/// counts as a narrow inspector.
const FIELDS_PANEL_WIDTHS: [f32; 3] = [240., 288., 360.];

/// Height of the catalog panel: sized for the tallest layout the knobs can
/// ask for (stacked labels at comfortable density) so a density change
/// never clips the last row.
const FIELDS_PANEL_HEIGHT: f32 = 424.;

/// The scrub's sensitivity: one pixel of horizontal travel is one unit.
const SCRUB_UNITS_PER_PIXEL: f64 = 1.;

/// The live number field's upper bound, supplied by this mock host the way
/// a real domain supplies its own legal range.
const NUMBER_CEILING: f64 = 720.;

/// Seed value of the live number field and the catalog's number row.
const NUMBER_SEED: f64 = 184.;

/// Seed value of the live slider and the catalog's slider row.
const SLIDER_SEED: f64 = 0.6;

/// Entries the live edit log keeps before it drops the oldest.
const EDIT_LOG_CAPACITY: usize = 10;

/// The layers the Mixed specimen folds into one controlled value. Two agree
/// and one does not, so selecting all three is genuinely mixed.
pub(crate) const MIXED_SELECTION_LAYERS: [(&str, f64); 3] = [
    ("Card / Primary", 8.),
    ("Card / Secondary", 8.),
    ("Badge", 24.),
];

/// The radius the Mixed specimen's commit writes to every selected layer.
const MIXED_COMMIT_RADIUS: f64 = 12.;

/// The corner radii the Mixed specimen starts from.
pub(crate) fn seed_radii() -> [f64; MIXED_SELECTION_LAYERS.len()] {
    MIXED_SELECTION_LAYERS.map(|(_, radius)| radius)
}

/// The value axis: what the host handed the field for this frame.
///
/// These are the three cases of [`InspectorValue`], not three widget
/// styles — the field renders each one differently because the host said
/// something different, never because the field remembered something.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FieldValueState {
    Unset,
    Mixed,
    Uniform,
}

impl NamedState for FieldValueState {
    const ALL: &'static [Self] = &[Self::Unset, Self::Mixed, Self::Uniform];

    fn label(self) -> &'static str {
        match self {
            Self::Unset => "Unset",
            Self::Mixed => "Mixed",
            Self::Uniform => "Uniform",
        }
    }
}

impl FieldValueState {
    /// The element-id fragment this case contributes to a specimen.
    pub(crate) const fn slug(self) -> &'static str {
        match self {
            Self::Unset => "unset",
            Self::Mixed => "mixed",
            Self::Uniform => "uniform",
        }
    }

    /// The controlled value a host with this state would supply.
    fn value<T>(self, uniform: T) -> InspectorValue<T> {
        match self {
            Self::Unset => InspectorValue::Unset,
            Self::Mixed => InspectorValue::Mixed,
            Self::Uniform => InspectorValue::Uniform(uniform),
        }
    }
}

/// The access axis: whether this host projection will accept an edit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FieldAccess {
    Editable,
    ReadOnly,
    Disabled,
}

impl NamedState for FieldAccess {
    const ALL: &'static [Self] = &[Self::Editable, Self::ReadOnly, Self::Disabled];

    fn label(self) -> &'static str {
        match self {
            Self::Editable => "Editable",
            Self::ReadOnly => "Read-only",
            Self::Disabled => "Disabled",
        }
    }
}

impl FieldAccess {
    /// The element-id fragment this case contributes to a specimen.
    pub(crate) const fn slug(self) -> &'static str {
        match self {
            Self::Editable => "editable",
            Self::ReadOnly => "read-only",
            Self::Disabled => "disabled",
        }
    }

    fn access(self) -> InspectorFieldAccess {
        match self {
            Self::Editable => InspectorFieldAccess::Editable,
            Self::ReadOnly => {
                InspectorFieldAccess::read_only(Some(SharedString::from("Bound to a style")))
            }
            Self::Disabled => {
                InspectorFieldAccess::disabled(Some(SharedString::from("Not available on a group")))
            }
        }
    }

    fn presentation(self) -> InspectorFieldPresentation {
        InspectorFieldPresentation::new(self.access())
    }

    /// What the reader will see this access do to every field at once.
    const fn effect(self) -> &'static str {
        match self {
            Self::Editable => "every field takes edits",
            Self::ReadOnly => "fields stay readable and focusable but refuse every edit",
            Self::Disabled => "fields dim and leave the tab order entirely",
        }
    }
}

/// One line of the live edit log.
pub(crate) struct FieldsEditLogEntry {
    /// The property the edit came from.
    pub(crate) field: &'static str,
    /// The phase the field emitted, or `None` when the field refused to
    /// emit anything at all.
    pub(crate) phase: Option<InspectorEditPhase>,
    /// The payload, spelled the way the controlled edit carries it.
    pub(crate) detail: SharedString,
}

pub(crate) struct FieldsScreen {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) access: FieldAccess,
    pub(crate) value_state: FieldValueState,
    pub(crate) density: InspectorDensity,
    pub(crate) label_placement: InspectorLabelPlacement,
    pub(crate) width: f32,
    /// The live scrubbable number field; it owns its own draft text.
    pub(crate) number: InspectorNumberField,
    /// The host value behind the live number field. Only a commit replaces
    /// it, which is what makes cancel visibly non-destructive.
    pub(crate) number_value: InspectorValue<f64>,
    /// Pointer origin and starting value of an open scrub.
    number_scrub: Option<(f32, f64)>,
    /// The live slider. It retains only the transaction, so the candidate
    /// value lives here in the mock host.
    pub(crate) slider: InspectorSliderField,
    pub(crate) slider_value: f64,
    pub(crate) slider_draft: Option<f64>,
    slider_bounds: Bounds<Pixels>,
    /// The Mixed specimen's per-layer radii and selection.
    pub(crate) radii: [f64; MIXED_SELECTION_LAYERS.len()],
    pub(crate) selection: [bool; MIXED_SELECTION_LAYERS.len()],
    pub(crate) log: Vec<FieldsEditLogEntry>,
    /// Repeat previews the fields swallowed instead of re-emitting.
    pub(crate) coalesced: usize,
    pub(crate) last_action: SharedString,
}

impl FieldsScreen {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        let access = FieldAccess::Editable;
        let value_state = FieldValueState::Uniform;
        let number_value = value_state.value(NUMBER_SEED);
        Self {
            focus_handle: cx.focus_handle(),
            access,
            value_state,
            density: InspectorDensity::Compact,
            label_placement: InspectorLabelPlacement::Leading,
            width: FIELDS_PANEL_WIDTHS[1],
            number: InspectorNumberField::new(number_value.clone(), access.presentation()),
            number_value,
            number_scrub: None,
            slider: InspectorSliderField::new(
                InspectorValue::Uniform(SLIDER_SEED),
                access.presentation(),
            ),
            slider_value: SLIDER_SEED,
            slider_draft: None,
            slider_bounds: Bounds::default(),
            radii: seed_radii(),
            selection: [true, true, false],
            log: Vec::new(),
            coalesced: 0,
            last_action: "Ready — drag the X field or the opacity slider and watch the phases \
                          arrive"
                .into(),
        }
    }

    /// The layout policy the grid inherits from the knobs.
    fn layout(&self) -> InspectorGridLayout {
        InspectorGridLayout::new(
            InspectorMetrics::current(),
            self.density,
            self.label_placement,
        )
    }

    fn presentation(&self) -> InspectorFieldPresentation {
        self.access.presentation()
    }

    /// Hands the live fields the next host snapshot. An open transaction
    /// keeps its draft across this, which is the whole point of `sync`.
    fn resync(&mut self) {
        self.number
            .sync(self.number_value.clone(), self.presentation());
        self.slider.sync(
            InspectorValue::Uniform(self.slider_value),
            self.presentation(),
        );
    }

    pub(crate) fn set_access(&mut self, access: FieldAccess, cx: &mut Context<Storybook>) {
        self.access = access;
        self.resync();
        self.last_action = format!("Access is {} — {}", access.label(), access.effect()).into();
        cx.notify();
    }

    pub(crate) fn set_value_state(&mut self, state: FieldValueState, cx: &mut Context<Storybook>) {
        self.value_state = state;
        self.number_value = state.value(NUMBER_SEED);
        self.resync();
        self.last_action = format!(
            "Host value is {} — the live X field was reseeded from the host",
            state.label()
        )
        .into();
        cx.notify();
    }

    pub(crate) fn set_density(&mut self, density: InspectorDensity, cx: &mut Context<Storybook>) {
        self.density = density;
        self.last_action =
            format!("Density is {} for the whole grid", density_label(density)).into();
        cx.notify();
    }

    pub(crate) fn set_label_placement(
        &mut self,
        placement: InspectorLabelPlacement,
        cx: &mut Context<Storybook>,
    ) {
        self.label_placement = placement;
        self.last_action = format!(
            "Labels are {} for every row in the grid",
            placement_label(placement)
        )
        .into();
        cx.notify();
    }

    pub(crate) fn set_width(&mut self, width: f32, cx: &mut Context<Storybook>) {
        self.width = width;
        self.last_action = format!("Panel width is {width:.0} px").into();
        cx.notify();
    }

    fn record(
        &mut self,
        field: &'static str,
        phase: InspectorEditPhase,
        detail: impl Into<SharedString>,
    ) {
        if self.log.len() == EDIT_LOG_CAPACITY {
            self.log.remove(0);
        }
        self.log.push(FieldsEditLogEntry {
            field,
            phase: Some(phase),
            detail: detail.into(),
        });
    }

    fn record_refusal(&mut self, field: &'static str, detail: impl Into<SharedString>) {
        if self.log.len() == EDIT_LOG_CAPACITY {
            self.log.remove(0);
        }
        self.log.push(FieldsEditLogEntry {
            field,
            phase: None,
            detail: detail.into(),
        });
    }

    fn record_edit(&mut self, field: &'static str, edit: &InspectorControlledEdit<f64>) {
        self.record(field, edit.phase, describe_value(&edit.value));
    }

    pub(crate) fn clear_log(&mut self, cx: &mut Context<Storybook>) {
        self.log.clear();
        self.coalesced = 0;
        self.last_action = "Cleared the edit log".into();
        cx.notify();
    }

    pub(crate) fn begin_number_scrub(&mut self, position_x: f32, cx: &mut Context<Storybook>) {
        let start = self
            .number_value
            .as_uniform()
            .copied()
            .unwrap_or(NUMBER_SEED);
        match self.number.begin_with(start, format_number(start)) {
            Some(edit) => {
                self.number_scrub = Some((position_x, start));
                self.record_edit("X", &edit);
                self.last_action = "Began an X edit — drag to preview, release to commit".into();
            }
            None => {
                self.record_refusal("X", format!("begin refused · {}", self.access.label()));
                self.last_action = format!(
                    "{} access refused the X edit before it began",
                    self.access.label()
                )
                .into();
            }
        }
        cx.notify();
    }

    pub(crate) fn update_number_scrub(&mut self, position_x: f32, cx: &mut Context<Storybook>) {
        let Some((origin_x, start)) = self.number_scrub else {
            return;
        };
        let value = (start + f64::from(position_x - origin_x) * SCRUB_UNITS_PER_PIXEL)
            .round()
            .clamp(0., NUMBER_CEILING);
        match self.number.scrub_to(value, format_number) {
            Some(edit) => self.record_edit("X", &edit),
            None => self.coalesced += 1,
        }
        cx.notify();
    }

    pub(crate) fn finish_number_scrub(&mut self, commit: bool, cx: &mut Context<Storybook>) {
        if self.number_scrub.take().is_none() {
            return;
        }
        let edit = if commit {
            self.number.commit_with(|text| text.parse().ok())
        } else {
            self.number.cancel()
        };
        let Some(edit) = edit else {
            return;
        };
        if edit.phase == InspectorEditPhase::Commit {
            self.number_value = edit.value.clone();
            self.last_action = "Committed the X edit — the host value moved once".into();
        } else {
            self.last_action =
                "Cancelled the X edit — the payload carries the value the edit began from".into();
        }
        self.record_edit("X", &edit);
        cx.notify();
    }

    /// One keystroke is one whole transaction: begin, a single nudged
    /// preview, then commit.
    pub(crate) fn nudge_number(&mut self, delta: f64, cx: &mut Context<Storybook>) {
        if self.number.is_editing() {
            return;
        }
        let start = self
            .number_value
            .as_uniform()
            .copied()
            .unwrap_or(NUMBER_SEED);
        let Some(begin) = self.number.begin_with(start, format_number(start)) else {
            self.record_refusal("X", format!("nudge refused · {}", self.access.label()));
            self.last_action =
                format!("{} access refused the keyboard nudge", self.access.label()).into();
            cx.notify();
            return;
        };
        self.record_edit("X", &begin);
        match self.number.nudge_with(
            delta,
            |text| text.parse().ok(),
            |value| Some(value.clamp(0., NUMBER_CEILING)),
            format_number,
        ) {
            Some(preview) => self.record_edit("X", &preview),
            None => self.coalesced += 1,
        }
        if let Some(commit) = self.number.commit_with(|text| text.parse().ok()) {
            self.number_value = commit.value.clone();
            self.record_edit("X", &commit);
        }
        self.last_action =
            format!("Nudged X by {delta:+} — begin, preview, commit in one key").into();
        cx.notify();
    }

    /// The slider ratio under a window-space pointer position.
    fn slider_ratio_at(&self, position_x: f32) -> f64 {
        let width = f32::from(self.slider_bounds.size.width);
        if width <= 0. {
            return self.slider_value;
        }
        let ratio = ((position_x - f32::from(self.slider_bounds.origin.x)) / width).clamp(0., 1.);
        (f64::from(ratio) * 100.).round() / 100.
    }

    pub(crate) fn set_slider_bounds(&mut self, bounds: Bounds<Pixels>) {
        self.slider_bounds = bounds;
    }

    pub(crate) fn begin_slider(&mut self, position_x: f32, cx: &mut Context<Storybook>) {
        let value = self.slider_ratio_at(position_x);
        match self.slider.begin(value) {
            Some(edit) => {
                self.slider_draft = Some(value);
                self.record_edit("Opacity", &edit);
                self.last_action = "Began an opacity edit — drag across the track".into();
            }
            None => {
                self.record_refusal(
                    "Opacity",
                    format!("begin refused · {}", self.access.label()),
                );
                self.last_action = format!(
                    "{} access refused the opacity edit before it began",
                    self.access.label()
                )
                .into();
            }
        }
        cx.notify();
    }

    pub(crate) fn update_slider(&mut self, position_x: f32, cx: &mut Context<Storybook>) {
        if self.slider_draft.is_none() {
            return;
        }
        let value = self.slider_ratio_at(position_x);
        self.slider_draft = Some(value);
        match self.slider.preview(value) {
            Some(edit) => self.record_edit("Opacity", &edit),
            None => self.coalesced += 1,
        }
        cx.notify();
    }

    pub(crate) fn finish_slider(&mut self, commit: bool, cx: &mut Context<Storybook>) {
        if self.slider_draft.take().is_none() {
            return;
        }
        let edit = if commit {
            self.slider.commit()
        } else {
            self.slider.cancel()
        };
        let Some(edit) = edit else {
            return;
        };
        if edit.phase == InspectorEditPhase::Commit {
            if let Some(value) = edit.value.as_uniform() {
                self.slider_value = *value;
            }
            self.last_action = "Committed the opacity edit".into();
        } else {
            self.last_action =
                "Released outside the track — the opacity edit cancelled and snapped back".into();
        }
        self.record_edit("Opacity", &edit);
        cx.notify();
    }

    /// The fold a real host performs: no targets is unset, agreeing targets
    /// are uniform, disagreeing targets are mixed.
    pub(crate) fn selection_value(&self) -> InspectorValue<f64> {
        let mut radii = self
            .selection
            .iter()
            .enumerate()
            .filter(|(_, selected)| **selected)
            .map(|(index, _)| self.radii[index]);
        let Some(first) = radii.next() else {
            return InspectorValue::Unset;
        };
        if radii.all(|radius| radius == first) {
            InspectorValue::Uniform(first)
        } else {
            InspectorValue::Mixed
        }
    }

    pub(crate) fn toggle_selection(&mut self, index: usize, cx: &mut Context<Storybook>) {
        self.selection[index] = !self.selection[index];
        self.last_action = format!(
            "{} {} — the folded value is now {}",
            if self.selection[index] {
                "Selected"
            } else {
                "Deselected"
            },
            MIXED_SELECTION_LAYERS[index].0,
            describe_value(&self.selection_value())
        )
        .into();
        cx.notify();
    }

    /// The one-shot commit gate every non-transactional field shares.
    pub(crate) fn commit_selection_radius(&mut self, cx: &mut Context<Storybook>) {
        let frame = InspectorFieldFrame::new(self.selection_value(), self.presentation());
        match frame.commit(MIXED_COMMIT_RADIUS) {
            Some(edit) => {
                for (index, selected) in self.selection.iter().enumerate() {
                    if *selected {
                        self.radii[index] = MIXED_COMMIT_RADIUS;
                    }
                }
                self.record_edit("Radius", &edit);
                self.last_action = format!(
                    "Committed radius {} to every selected layer — Mixed collapsed to Uniform",
                    format_number(MIXED_COMMIT_RADIUS)
                )
                .into();
            }
            None => {
                self.record_refusal(
                    "Radius",
                    format!("commit refused · {}", self.access.label()),
                );
                self.last_action = format!(
                    "{} access refused the radius commit — the frame gates it, not the caller",
                    self.access.label()
                )
                .into();
            }
        }
        cx.notify();
    }

    pub(crate) fn reset_selection(&mut self, cx: &mut Context<Storybook>) {
        self.radii = seed_radii();
        self.selection = [true, true, false];
        self.last_action = "Reset the selection fixture to two agreeing layers".into();
        cx.notify();
    }
}

fn format_number(value: f64) -> String {
    if (value - value.round()).abs() < 0.05 {
        format!("{:.0}", value.round())
    } else {
        format!("{value:.1}")
    }
}

fn format_percent(value: f64) -> String {
    format!("{:.0}%", value * 100.)
}

/// Spells a controlled payload the way the edit carries it, so a cancel
/// that restores `Mixed` is readable as exactly that.
fn describe_value(value: &InspectorValue<f64>) -> SharedString {
    match value {
        InspectorValue::Unset => "Unset".into(),
        InspectorValue::Mixed => "Mixed".into(),
        InspectorValue::Uniform(value) => format!("Uniform({})", format_number(*value)).into(),
    }
}

/// The caption a field paints for a controlled value, plus whether it is a
/// placeholder rather than a real value.
fn value_caption<T>(
    value: &InspectorValue<T>,
    uniform: impl FnOnce(&T) -> String,
) -> (SharedString, bool) {
    match value {
        InspectorValue::Unset => ("None".into(), true),
        InspectorValue::Mixed => ("—".into(), true),
        InspectorValue::Uniform(value) => (uniform(value).into(), false),
    }
}

const fn phase_label(phase: InspectorEditPhase) -> &'static str {
    match phase {
        InspectorEditPhase::Begin => "Begin",
        InspectorEditPhase::Preview => "Preview",
        InspectorEditPhase::Commit => "Commit",
        InspectorEditPhase::Cancel => "Cancel",
    }
}

fn phase_color(phase: InspectorEditPhase, cx: &Context<Storybook>) -> Hsla {
    match phase {
        InspectorEditPhase::Begin => fanta_gpui::atoms::SemanticColor::TextBrand.resolve(cx),
        InspectorEditPhase::Preview => fanta_gpui::atoms::SemanticColor::TextWarning.resolve(cx),
        InspectorEditPhase::Commit => fanta_gpui::atoms::SemanticColor::TextSuccess.resolve(cx),
        InspectorEditPhase::Cancel => fanta_gpui::atoms::SemanticColor::TextDanger.resolve(cx),
    }
}

const fn density_label(density: InspectorDensity) -> &'static str {
    match density {
        InspectorDensity::Compact => "Compact",
        InspectorDensity::Comfortable => "Comfortable",
    }
}

const fn placement_label(placement: InspectorLabelPlacement) -> &'static str {
    match placement {
        InspectorLabelPlacement::Leading => "Leading",
        InspectorLabelPlacement::Stacked => "Stacked",
    }
}

/// The value caption mounted inside a field frame.
fn field_caption(
    caption: SharedString,
    placeholder: bool,
    cx: &mut Context<Storybook>,
) -> AnyElement {
    div()
        .flex_1()
        .min_w(px(0.))
        .truncate()
        .when(placeholder, |text| {
            text.text_color(
                fanta_gpui::atoms::SemanticColor::TextTertiary
                    .resolve(cx)
                    .opacity(0.7),
            )
        })
        .child(caption)
        .into_any_element()
}

/// The two-state switch an [`InspectorToggleField`] frames. Mixed paints a
/// dash and stays put: the toggle refuses to guess which way it should go.
fn switch_specimen(
    value: &InspectorValue<bool>,
    editable: bool,
    cx: &mut Context<Storybook>,
) -> AnyElement {
    let on = matches!(value, InspectorValue::Uniform(true));
    let mixed = value.is_mixed();
    let knob = if mixed {
        render_lucide_icon(
            LucideIcon::Minus,
            fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx),
            10.,
        )
    } else {
        div()
            .size(px(10.))
            .rounded_full()
            .bg(if editable {
                fanta_gpui::atoms::SemanticColor::Text.resolve(cx)
            } else {
                fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx)
            })
            .into_any_element()
    };
    h_flex()
        .w(px(26.))
        .h(px(14.))
        .flex_none()
        .items_center()
        .px(px(2.))
        .rounded_full()
        .bg(if on {
            fanta_gpui::atoms::SemanticColor::BackgroundSelected.resolve(cx)
        } else {
            fanta_gpui::atoms::SemanticColor::Border.resolve(cx)
        })
        .when(on, |track| track.justify_end())
        .when(mixed, |track| track.justify_center())
        .child(knob)
        .into_any_element()
}

/// The box an [`InspectorCheckboxField`] frames: checked, empty, or the
/// indeterminate dash that only Mixed earns.
fn checkbox_specimen(value: &InspectorValue<bool>, cx: &mut Context<Storybook>) -> AnyElement {
    let checked = matches!(value, InspectorValue::Uniform(true));
    let glyph = value.is_mixed().then_some(LucideIcon::Minus);
    let glyph = if checked {
        Some(LucideIcon::Check)
    } else {
        glyph
    };
    div()
        .size(px(14.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(3.))
        .border_1()
        .border_color(if checked {
            fanta_gpui::atoms::SemanticColor::BackgroundSelected.resolve(cx)
        } else {
            fanta_gpui::atoms::SemanticColor::Border.resolve(cx)
        })
        .when(checked, |field| {
            field.bg(fanta_gpui::atoms::SemanticColor::BackgroundSelected.resolve(cx))
        })
        .when(value.is_unset(), |field| field.opacity(0.62))
        .when_some(glyph, |field, icon| {
            field.child(render_lucide_icon(
                icon,
                if checked {
                    fanta_gpui::atoms::SemanticColor::Background.resolve(cx)
                } else {
                    fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx)
                },
                10.,
            ))
        })
        .into_any_element()
}

/// The static track an [`InspectorSliderField`] frames in the catalog.
fn slider_track_specimen(value: &InspectorValue<f64>, cx: &mut Context<Storybook>) -> AnyElement {
    let ratio = value.as_uniform().copied().unwrap_or(0.) as f32;
    div()
        .flex_1()
        .min_w(px(0.))
        .h(px(4.))
        .rounded_full()
        .bg(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
        .when(value.is_uniform(), |track| {
            track.child(
                div()
                    .h_full()
                    .w(gpui::relative(ratio))
                    .rounded_full()
                    .bg(fanta_gpui::atoms::SemanticColor::BackgroundSelected.resolve(cx)),
            )
        })
        .into_any_element()
}

/// The slot a control occupies in a property row: it claims the leftover
/// width in a leading row and the full width in a stacked one.
fn control_slot(layout: InspectorGridLayout, control: AnyElement) -> gpui::Div {
    div()
        .min_w(px(0.))
        .when(
            layout.label_placement == InspectorLabelPlacement::Leading,
            |slot| slot.flex_1(),
        )
        .when(
            layout.label_placement == InspectorLabelPlacement::Stacked,
            |slot| slot.w_full(),
        )
        .child(control)
}

/// One labeled property row that inherits the grid's density and placement.
fn property_row(
    layout: InspectorGridLayout,
    label: &'static str,
    control: AnyElement,
) -> AnyElement {
    inspector_row_with_layout(layout)
        .child(inspector_field_label(layout).child(label))
        .child(control_slot(layout, control))
        .into_any_element()
}

impl Storybook {
    /// Every field kind, once, at whatever the knobs currently say.
    fn render_fields_catalog_grid(&self, cx: &mut Context<Self>) -> AnyElement {
        let screen = &self.fields_screen;
        let layout = screen.layout();
        let metrics = layout.metrics;
        let presentation = screen.presentation();
        let state = screen.value_state;
        let editable = screen.access == FieldAccess::Editable;

        let text = InspectorTextField::new(
            state.value(SharedString::from("Hero card")),
            presentation.clone(),
        );
        let (text_caption, text_placeholder) =
            value_caption(text.frame().value(), |value| value.to_string());
        let text_row = property_row(
            layout,
            "Name",
            text.render("fields-kind-text", metrics, cx)
                .w_full()
                .child(field_caption(text_caption, text_placeholder, cx))
                .into_any_element(),
        );

        let number = InspectorNumberField::new(state.value(NUMBER_SEED), presentation.clone());
        let (number_caption, number_placeholder) =
            value_caption(number.frame().value(), |value| format_number(*value));
        let number_row = property_row(
            layout,
            "X",
            number
                .render("fields-kind-number", metrics, cx)
                .w_full()
                .child(field_caption(number_caption, number_placeholder, cx))
                .child(
                    div()
                        .flex_none()
                        .text_color(
                            fanta_gpui::atoms::SemanticColor::TextTertiary
                                .resolve(cx)
                                .opacity(0.7),
                        )
                        .child("px"),
                )
                .into_any_element(),
        );

        let picker = InspectorPickerField::new(
            state.value(SharedString::from("Vertical")),
            presentation.clone(),
        );
        let (picker_caption, picker_placeholder) =
            value_caption(picker.frame().value(), |value| value.to_string());
        let picker_row = property_row(
            layout,
            "Direction",
            picker
                .render("fields-kind-picker", metrics, cx)
                .w_full()
                .child(field_caption(picker_caption, picker_placeholder, cx))
                .child(render_lucide_icon(
                    LucideIcon::ChevronDown,
                    fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx),
                    12.,
                ))
                .into_any_element(),
        );

        let slider = InspectorSliderField::new(state.value(SLIDER_SEED), presentation.clone());
        let (slider_caption, slider_placeholder) =
            value_caption(slider.frame().value(), |value| format_percent(*value));
        let slider_track = slider_track_specimen(slider.frame().value(), cx);
        let slider_row = property_row(
            layout,
            "Opacity",
            slider
                .render("fields-kind-slider", metrics, cx)
                .w_full()
                .child(slider_track)
                .child(
                    div()
                        .w(px(34.))
                        .flex_none()
                        .when(slider_placeholder, |caption| {
                            caption.text_color(
                                fanta_gpui::atoms::SemanticColor::TextTertiary
                                    .resolve(cx)
                                    .opacity(0.7),
                            )
                        })
                        .child(slider_caption),
                )
                .into_any_element(),
        );

        let toggle = InspectorToggleField::new(state.value(true), presentation.clone());
        let switch = switch_specimen(toggle.frame().value(), editable, cx);
        let toggle_row = property_row(
            layout,
            "Clip content",
            toggle
                .render("fields-kind-toggle", metrics, cx)
                .w(px(48.))
                .flex_none()
                .justify_center()
                .child(switch)
                .into_any_element(),
        );

        let checkbox = InspectorCheckboxField::new(state.value(true), presentation.clone());
        let box_specimen = checkbox_specimen(checkbox.frame().value(), cx);
        let checkbox_row = property_row(
            layout,
            "Visible",
            checkbox
                .render("fields-kind-checkbox", metrics, cx)
                .w(px(48.))
                .flex_none()
                .justify_center()
                .child(box_specimen)
                .into_any_element(),
        );

        let fill = fanta_gpui::atoms::SemanticColor::TextBrand.resolve(cx);
        let swatch = InspectorColorSwatch::new(state.value(fill), presentation.clone());
        let color = InspectorColorField::new(state.value(fill), presentation);
        let (color_caption, color_placeholder) =
            value_caption(color.frame().value(), |_| "0C8CE9".to_owned());
        let swatch_element = swatch
            .render(
                fanta_gpui::atoms::SemanticColor::BackgroundTertiary.resolve(cx),
                metrics,
                cx,
            )
            .w(metrics.row_height);
        let color_row = property_row(
            layout,
            "Fill",
            h_flex()
                .w_full()
                .min_w(px(0.))
                .gap(metrics.control_gap)
                .child(swatch_element)
                .child(
                    color
                        .render("fields-kind-color", metrics, cx)
                        .flex_1()
                        .min_w(px(0.))
                        .child(field_caption(color_caption, color_placeholder, cx)),
                )
                .into_any_element(),
        );

        inspector_field_grid_with_layout(layout)
            .id("fields-catalog-grid")
            .debug_selector(|| "fields-catalog-grid".to_owned())
            .py_2()
            .child(text_row)
            .child(number_row)
            .child(picker_row)
            .child(slider_row)
            .child(toggle_row)
            .child(checkbox_row)
            .child(color_row)
            .into_any_element()
    }

    fn render_fields_catalog(&self, cx: &mut Context<Self>) -> AnyElement {
        let screen = &self.fields_screen;
        let width = screen.width;
        let summary = format!(
            "{} · {} value · {} density · {} labels · {width:.0} px",
            screen.access.label(),
            screen.value_state.label(),
            density_label(screen.density),
            placement_label(screen.label_placement),
        );
        let grid = self.render_fields_catalog_grid(cx);
        specimen_card(
            "fields-catalog",
            "Every field kind in one grid",
            "Text, number, picker, slider, toggle, checkbox, color swatch, and color \
             field — each one a controlled object over the same field frame. The grid \
             chooses density and label placement once for every row; an individual \
             field never picks its own. Drive the knobs and watch all seven rows \
             change together: the value knob is the host's snapshot, the access knob \
             is the host's projection, and neither is state the fields remember.",
            specimen_rows(vec![specimen_row(
                "fields-catalog-row",
                summary,
                vec![framed_panel(width, FIELDS_PANEL_HEIGHT, grid, cx)],
                cx,
            )]),
            cx,
        )
    }

    fn render_fields_matrix_cell(
        &self,
        row: FieldValueState,
        column: FieldAccess,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let metrics = InspectorMetrics::current();
        let presentation = column.presentation();
        let number = InspectorNumberField::new(row.value(NUMBER_SEED), presentation.clone());
        let (caption, placeholder) =
            value_caption(number.frame().value(), |value| format_number(*value));
        let checkbox = InspectorCheckboxField::new(row.value(true), presentation);
        let box_specimen = checkbox_specimen(checkbox.frame().value(), cx);
        let id = format!("fields-matrix-{}-{}", row.slug(), column.slug());
        let selector = id.clone();
        v_flex()
            .id(SharedString::from(id.clone()))
            .debug_selector(move || selector)
            .items_center()
            .gap_1p5()
            .child(
                number
                    .render(SharedString::from(format!("{id}-number")), metrics, cx)
                    .w(px(72.))
                    .flex_none()
                    .child(field_caption(caption, placeholder, cx)),
            )
            .child(
                checkbox
                    .render(SharedString::from(format!("{id}-checkbox")), metrics, cx)
                    .w(px(72.))
                    .flex_none()
                    .justify_center()
                    .child(box_specimen),
            )
            .into_any_element()
    }

    fn render_fields_matrix(&self, cx: &mut Context<Self>) -> AnyElement {
        let matrix = state_matrix(
            FieldValueState::ALL,
            FieldAccess::ALL,
            |row, column, cx| self.render_fields_matrix_cell(row, column, cx),
            cx,
        );
        specimen_card(
            "fields-state-matrix",
            "Value state × access, on one number field and one checkbox",
            "Nine cells, two orthogonal axes. Down the rows: what the host knows — no \
             value, several disagreeing values, one shared value. Across the columns: \
             what the host permits. Unset shows a muted placeholder, Mixed shows the \
             dash (and the checkbox its indeterminate mark), and only Uniform paints a \
             real value. Read-only keeps the frame legible and focusable; disabled \
             dims it and drops it out of the tab order. The two axes never leak into \
             each other: a read-only field still says which of the three values it \
             holds.",
            matrix,
            cx,
        )
    }

    fn render_fields_mixed(&self, cx: &mut Context<Self>) -> AnyElement {
        let screen = &self.fields_screen;
        let metrics = InspectorMetrics::current();
        let access = screen.access.access();
        let value = screen.selection_value();
        let (caption, placeholder) = value_caption(&value, |radius| format_number(*radius));
        let selected_count = screen
            .selection
            .iter()
            .filter(|selected| **selected)
            .count();

        let mut layers = v_flex()
            .id("fields-mixed-selection")
            .debug_selector(|| "fields-mixed-selection".to_owned())
            .w(px(240.))
            .flex_none()
            .p_1()
            .gap_0p5()
            .rounded(px(6.))
            .border_1()
            .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
            .bg(fanta_gpui::atoms::SemanticColor::Background.resolve(cx));
        for (index, (name, _)) in MIXED_SELECTION_LAYERS.iter().enumerate() {
            let selected = screen.selection[index];
            let radius = screen.radii[index];
            let name = *name;
            layers = layers.child(
                list_row(
                    SharedString::from(format!("fields-mixed-layer-{index}")),
                    px(26.),
                    cx,
                )
                .debug_selector(move || format!("fields-mixed-layer-{index}"))
                .px_1p5()
                .gap_2()
                .hover(|style| {
                    style.bg(fanta_gpui::atoms::SemanticColor::BackgroundHover.resolve(cx))
                })
                .when(selected, |row| {
                    row.bg(fanta_gpui::atoms::SemanticColor::BackgroundSelected
                        .resolve(cx)
                        .opacity(0.18))
                })
                .on_activate(cx.listener(move |this, _: &ActivateEvent, _, cx| {
                    this.fields_screen.toggle_selection(index, cx);
                }))
                .child(checkbox_specimen(&InspectorValue::Uniform(selected), cx))
                .child(
                    truncating_label(name)
                        .typography(fanta_gpui::atoms::TypographyToken::BodyMedium),
                )
                .child(
                    div()
                        .flex_none()
                        .text_size(px(10.))
                        .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                        .child(format!("radius {}", format_number(radius))),
                ),
            );
        }

        let field = InspectorNumberField::new(value.clone(), screen.presentation());
        let readout = v_flex()
            .id("fields-mixed-readout")
            .debug_selector(|| "fields-mixed-readout".to_owned())
            .w(px(240.))
            .flex_none()
            .gap_2()
            .child(
                div()
                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child(format!(
                        "{selected_count} of {} layers selected",
                        MIXED_SELECTION_LAYERS.len()
                    )),
            )
            .child(
                h_flex()
                    .w_full()
                    .gap(metrics.control_gap)
                    .child(
                        div()
                            .w(px(64.))
                            .flex_none()
                            .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                            .child("Corner radius"),
                    )
                    .child(
                        field
                            .render("fields-mixed-field", metrics, cx)
                            .flex_1()
                            .min_w(px(0.))
                            .child(field_caption(caption, placeholder, cx)),
                    ),
            )
            .child(
                div()
                    .id("fields-mixed-value")
                    .debug_selector(|| "fields-mixed-value".to_owned())
                    .w_full()
                    .px_1p5()
                    .py_0p5()
                    .rounded(px(4.))
                    .border_1()
                    .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                    .text_size(px(10.))
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child(format!("InspectorValue::{}", describe_value(&value))),
            )
            .child(
                inspector_action_group(metrics)
                    .child(
                        inspector_action_button("fields-mixed-commit", &access, metrics, cx)
                            .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                            .debug_selector(|| "fields-mixed-commit".to_owned())
                            .child(format!("Set radius {}", format_number(MIXED_COMMIT_RADIUS)))
                            .on_activate(cx.listener(|this, _: &ActivateEvent, _, cx| {
                                this.fields_screen.commit_selection_radius(cx);
                            })),
                    )
                    .child(
                        inspector_action_button(
                            "fields-mixed-reset",
                            &InspectorFieldAccess::Editable,
                            metrics,
                            cx,
                        )
                        .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                        .debug_selector(|| "fields-mixed-reset".to_owned())
                        .child("Reset")
                        .on_activate(cx.listener(
                            |this, _: &ActivateEvent, _, cx| {
                                this.fields_screen.reset_selection(cx);
                            },
                        )),
                    ),
            );

        specimen_card(
            "fields-mixed",
            "Mixed is a fold, not a mode",
            "Three layers, two of which agree. Select the first two and the field \
             reads Uniform(8); add the Badge and the same field becomes the dash, \
             because the host folded disagreeing targets into InspectorValue::Mixed. \
             Deselect everything and it is Unset. Commit a radius and the dash \
             collapses back to a uniform value in the same frame — and under \
             read-only or disabled access the frame's commit gate returns nothing at \
             all, which the edit log below records as a refusal.",
            specimen_rows(vec![specimen_row(
                "fields-mixed-row",
                "Click a layer to add or remove it from the selection",
                vec![layers.into_any_element(), readout.into_any_element()],
                cx,
            )]),
            cx,
        )
    }

    fn render_fields_live_number(&self, cx: &mut Context<Self>) -> AnyElement {
        let screen = &self.fields_screen;
        let metrics = InspectorMetrics::current();
        let (caption, placeholder) = match screen.number.draft() {
            Some(draft) => (SharedString::from(draft.to_owned()), false),
            None => value_caption(&screen.number_value, |value| format_number(*value)),
        };
        screen
            .number
            .render("fields-live-number", metrics, cx)
            .debug_selector(|| "fields-live-number".to_owned())
            .w_full()
            .cursor_col_resize()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    this.fields_screen
                        .begin_number_scrub(f32::from(event.position.x), cx);
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if event.dragging() {
                    this.fields_screen
                        .update_number_scrub(f32::from(event.position.x), cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| {
                    this.fields_screen.finish_number_scrub(true, cx);
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| {
                    this.fields_screen.finish_number_scrub(false, cx);
                }),
            )
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                match event.keystroke.key.as_str() {
                    "up" => this.fields_screen.nudge_number(1., cx),
                    "down" => this.fields_screen.nudge_number(-1., cx),
                    _ => {}
                }
            }))
            .child(field_caption(caption, placeholder, cx))
            .child(
                div()
                    .flex_none()
                    .text_color(
                        fanta_gpui::atoms::SemanticColor::TextTertiary
                            .resolve(cx)
                            .opacity(0.7),
                    )
                    .child("px"),
            )
            .into_any_element()
    }

    fn render_fields_live_slider(&self, cx: &mut Context<Self>) -> AnyElement {
        let screen = &self.fields_screen;
        let metrics = InspectorMetrics::current();
        let value = screen.slider_draft.unwrap_or(screen.slider_value);
        let entity = cx.entity();
        let track = div()
            .id("fields-live-slider")
            .debug_selector(|| "fields-live-slider".to_owned())
            .relative()
            .flex_1()
            .min_w(px(0.))
            .h(px(16.))
            .flex()
            .items_center()
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    this.fields_screen
                        .begin_slider(f32::from(event.position.x), cx);
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if event.dragging() {
                    this.fields_screen
                        .update_slider(f32::from(event.position.x), cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| {
                    this.fields_screen.finish_slider(true, cx);
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| {
                    this.fields_screen.finish_slider(false, cx);
                }),
            )
            .child(
                div()
                    .w_full()
                    .h(px(4.))
                    .rounded_full()
                    .bg(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                    .child(
                        div()
                            .h_full()
                            .w(gpui::relative(value as f32))
                            .rounded_full()
                            .bg(fanta_gpui::atoms::SemanticColor::BackgroundSelected.resolve(cx)),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .top(px(3.))
                    .left(gpui::relative(value as f32))
                    .ml(px(-5.))
                    .size(px(10.))
                    .rounded_full()
                    .border_1()
                    .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                    .bg(fanta_gpui::atoms::SemanticColor::Text.resolve(cx)),
            )
            .child(track_bounds(entity, |story: &mut Storybook, bounds| {
                story.fields_screen.set_slider_bounds(bounds);
            }));

        screen
            .slider
            .render("fields-live-slider-frame", metrics, cx)
            .w_full()
            .child(track)
            .child(div().w(px(34.)).flex_none().child(format_percent(value)))
            .into_any_element()
    }

    fn render_fields_edit_log(&self, cx: &mut Context<Self>) -> AnyElement {
        let screen = &self.fields_screen;
        let mut column = v_flex()
            .id("fields-edit-log")
            .debug_selector(|| "fields-edit-log".to_owned())
            .w(px(360.))
            .max_w_full()
            .flex_none()
            .gap_1()
            .p_2()
            .rounded(px(6.))
            .border_1()
            .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
            .bg(fanta_gpui::atoms::SemanticColor::Background.resolve(cx));
        if screen.log.is_empty() {
            column = column.child(
                div()
                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child("No edits yet — press and drag the X field or the opacity track."),
            );
        }
        for entry in &screen.log {
            let (tag, color) = match entry.phase {
                Some(phase) => (phase_label(phase), phase_color(phase, cx)),
                None => (
                    "Refused",
                    fanta_gpui::atoms::SemanticColor::TextDanger.resolve(cx),
                ),
            };
            column = column.child(
                h_flex()
                    .w_full()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .w(px(56.))
                            .flex_none()
                            .px_1()
                            .rounded(px(3.))
                            .border_1()
                            .border_color(color)
                            .text_size(px(10.))
                            .text_color(color)
                            .child(tag),
                    )
                    .child(
                        div()
                            .w(px(52.))
                            .flex_none()
                            .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                            .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                            .child(entry.field),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .truncate()
                            .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                            .child(entry.detail.clone()),
                    ),
            );
        }
        column
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .pt_1()
                    .child(
                        div()
                            .text_size(px(10.))
                            .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                            .child(format!(
                                "{} repeat previews coalesced away",
                                screen.coalesced
                            )),
                    )
                    .child(
                        fanta_gpui::atoms::ui_button("fields-clear-log")
                            .label("Clear")
                            .xsmall()
                            .compact()
                            .on_activate(cx.listener(|this, _: &ActivateEvent, _, cx| {
                                this.fields_screen.clear_log(cx);
                            })),
                    ),
            )
            .into_any_element()
    }

    fn render_fields_edit_phase(&self, cx: &mut Context<Self>) -> AnyElement {
        let layout = self.fields_screen.layout();
        let number = self.render_fields_live_number(cx);
        let slider = self.render_fields_live_slider(cx);
        let log = self.render_fields_edit_log(cx);
        let controls = inspector_field_grid_with_layout(layout)
            .id("fields-live-grid")
            .debug_selector(|| "fields-live-grid".to_owned())
            .w(px(260.))
            .flex_none()
            .py_2()
            .child(property_row(layout, "X", number))
            .child(property_row(layout, "Opacity", slider))
            .into_any_element();

        specimen_card(
            "fields-edit-phase",
            "The edit phases, live",
            "Press the X field and drag sideways, or drag across the opacity track. \
             The log records exactly what the field emitted: one Begin, a Preview per \
             distinct candidate, and one terminal phase. Drag back over a value you \
             already passed and nothing new arrives — repeat previews are coalesced, \
             and the counter under the log says how many were swallowed. Release over \
             the control to commit; release outside it to cancel and watch the payload \
             carry the value the transaction began from, mixed or unset included. With \
             the X field focused, Up and Down run the whole transaction from one \
             keystroke. Set access to read-only and the field refuses to begin at all.",
            specimen_rows(vec![specimen_row(
                "fields-edit-phase-row",
                "Drag a control on the left; read what it emitted on the right",
                vec![controls, log],
                cx,
            )]),
            cx,
        )
    }

    pub(crate) fn render_fields_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let catalog = self.render_fields_catalog(cx);
        let matrix = self.render_fields_matrix(cx);
        let mixed = self.render_fields_mixed(cx);
        let edit_phase = self.render_fields_edit_phase(cx);
        specimen_story_root("storybook-fields")
            .track_focus(&self.fields_screen.focus_handle)
            .child(catalog)
            .child(matrix)
            .child(mixed)
            .child(edit_phase)
            .into_any_element()
    }

    pub(crate) fn render_fields_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        let story = self.render_fields_story(cx);
        self.spec_reference("storybook-reference-fields", story, cx)
    }

    pub(crate) fn render_fields_knobs(&self, cx: &mut Context<Self>) -> AnyElement {
        let screen = &self.fields_screen;
        let access = screen.access;
        let value_state = screen.value_state;
        let density = screen.density;
        let placement = screen.label_placement;
        let width = screen.width;
        knobs::knobs_panel(
            "fields-story-knobs",
            vec![
                knobs::enum_knob_row(
                    "fields-knob-access",
                    "ACCESS",
                    FieldAccess::ALL
                        .iter()
                        .map(|access| KnobOption::new(*access, access.label())),
                    access,
                    |this, access, _, cx| this.fields_screen.set_access(access, cx),
                    cx,
                ),
                knobs::enum_knob_row(
                    "fields-knob-value",
                    "VALUE STATE",
                    FieldValueState::ALL
                        .iter()
                        .map(|state| KnobOption::new(*state, state.label())),
                    value_state,
                    |this, state, _, cx| this.fields_screen.set_value_state(state, cx),
                    cx,
                ),
                knobs::enum_knob_row(
                    "fields-knob-density",
                    "DENSITY",
                    [InspectorDensity::Compact, InspectorDensity::Comfortable]
                        .map(|density| KnobOption::new(density, density_label(density))),
                    density,
                    |this, density, _, cx| this.fields_screen.set_density(density, cx),
                    cx,
                ),
                knobs::enum_knob_row(
                    "fields-knob-labels",
                    "LABEL PLACEMENT",
                    [
                        InspectorLabelPlacement::Leading,
                        InspectorLabelPlacement::Stacked,
                    ]
                    .map(|placement| KnobOption::new(placement, placement_label(placement))),
                    placement,
                    |this, placement, _, cx| this.fields_screen.set_label_placement(placement, cx),
                    cx,
                ),
                knobs::numeric_knob_row(
                    "fields-knob-width",
                    "PANEL WIDTH",
                    FIELDS_PANEL_WIDTHS
                        .map(|width| KnobOption::new(width, format!("{width:.0} px"))),
                    width,
                    |this, width, _, cx| this.fields_screen.set_width(width, cx),
                    cx,
                ),
            ],
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_specimen_layers_actually_disagree() {
        let radii = seed_radii();
        assert!(
            radii.iter().any(|radius| *radius != radii[0]),
            "the Mixed specimen must contain a layer that disagrees, or selecting \
             every layer would never fold to InspectorValue::Mixed"
        );
    }

    #[test]
    fn value_captions_distinguish_unset_from_mixed() {
        let (unset, unset_placeholder) =
            value_caption(&InspectorValue::<f64>::Unset, |value| value.to_string());
        let (mixed, mixed_placeholder) =
            value_caption(&InspectorValue::<f64>::Mixed, |value| value.to_string());
        let (uniform, uniform_placeholder) =
            value_caption(&InspectorValue::Uniform(8.), |value| format_number(*value));
        assert_ne!(unset, mixed, "unset and mixed must not read the same");
        assert_eq!(uniform, SharedString::from("8"));
        assert!(unset_placeholder && mixed_placeholder && !uniform_placeholder);
    }

    #[test]
    fn state_axes_name_every_case_once() {
        assert_eq!(FieldValueState::ALL.len(), 3);
        assert_eq!(FieldAccess::ALL.len(), 3);
        for state in FieldValueState::ALL {
            assert!(!state.label().is_empty() && !state.slug().is_empty());
        }
        for access in FieldAccess::ALL {
            assert!(!access.label().is_empty() && !access.slug().is_empty());
            assert_eq!(
                access.presentation().access.is_editable(),
                *access == FieldAccess::Editable,
                "only the editable column may accept edits"
            );
        }
    }
}
