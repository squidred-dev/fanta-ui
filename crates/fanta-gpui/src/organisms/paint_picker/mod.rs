//! Reusable, host-controlled paint picker for fills, strokes, and color surfaces.
//!
//! The picker owns focus, draft input text, and the selected gradient stop. It
//! never mutates a document or treats its local paint snapshot as
//! authoritative: every edit emits [`PaintPickerAction::Edit`], and the host
//! accepts that edit by calling [`PaintPicker::set_target`] with fresh data.

use crate::atoms::TypographyExt as _;
#[cfg(test)]
use std::path::PathBuf;

use gpui::{
    Anchor, AnyElement, App, AppContext as _, Bounds, Context, DragMoveEvent, Empty, Entity,
    EntityId, EventEmitter, ExternalPaths, FocusHandle, Focusable, InteractiveElement as _,
    IntoElement, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ParentElement as _, Pixels, Point, Render, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled as _, Subscription, Window, canvas, div,
    linear_color_stop, linear_gradient, pattern_slash, point, prelude::FluentBuilder as _, px,
    relative,
};
use gpui_component::{
    Disableable as _, Icon, IconName, Selectable as _, Sizable as _, StyledExt as _,
    button::ButtonVariants as _,
    h_flex,
    input::{Input, InputEvent, InputState},
    popover::Popover,
    scroll::{Scrollbar, ScrollbarAxis},
    v_flex,
};

use crate::atoms::{
    ActivateControl, ButtonControlExt as _, CONTROL_KEY_CONTEXT, LucideIcon, render_lucide_icon,
    tokens, track_bounds,
};
use crate::molecules::{
    InspectorOverlayDismissCause, InspectorOverlayDismissIntent, InspectorOverlayFocusTarget,
    popup_height, popup_width,
};

mod contrast;
mod gradient;
mod media;
mod resource_browser;
mod shader;
mod solid_color;
mod view;

use gradient::*;
#[cfg(test)]
use media::CROP_ROTATION_STEP_DEGREES;
pub(crate) use media::media_drop_from_paths;
#[cfg(test)]
use media::rotated_crop_preview;
use solid_color::*;

#[cfg(test)]
use crate::design::DesignMediaPaintCapabilities;
use crate::design::{
    CancelDesignInteraction, DesignBlendMode, DesignColor, DesignColorContrastCategory,
    DesignColorContrastLeafViewData, DesignColorContrastLevel, DesignColorContrastPaintViewData,
    DesignColorStyleSample, DesignColorStyleSampleSelection, DesignColorStyleSampleViewData,
    DesignGradientPaint, DesignGradientStop, DesignImageFilter, DesignImageFilters,
    DesignImagePaint, DesignMediaCropAction, DesignMediaCropToolState, DesignMediaDroppedFile,
    DesignMediaKind, DesignMediaPaintPlacement, DesignMediaPaintScaleMode, DesignMediaPaintView,
    DesignMediaPaintViewData, DesignMediaSourceAction, DesignMenuPreviewPhase, DesignPaint,
    DesignPaintBinding, DesignPaintColorTarget, DesignPaintEdit, DesignPaintKind,
    DesignPaintPayload, DesignPaintProperty, DesignPaintSource, DesignPaintTransform,
    DesignPaintType, DesignPaintValue, DesignPaintVariableViewData, DesignPanelCollection,
    DesignPanelEditPhase, DesignPatternHorizontalAlignment, DesignPatternPaint,
    DesignPatternSource, DesignPatternSpacing, DesignPatternTileType, DesignShaderDefinition,
    DesignShaderPaint, DesignShaderPropertyDefinition, DesignShaderPropertyValue,
    DesignShaderSelection, DesignShaderViewData, DesignSolidPaint, DesignVariable,
    DesignVariableImportState, DesignVariableResolvedValue, DesignVariableSource, DesignVideoPaint,
    DesignVideoPreviewAction, DesignVideoPreviewState, DesignVideoPreviewStatus,
};

const PICKER_WIDTH: f32 = tokens::MenuWidth::PICKER;
const PICKER_MAX_HEIGHT: f32 = 520.;
const PICKER_HEADER_HEIGHT: f32 = tokens::RowHeight::SECTION_HEADER;
const PAINT_TYPE_ROW_HEIGHT: f32 = tokens::RowHeight::SECTION_HEADER;
const PAINT_HEADER_CONTROL_SIZE: f32 = tokens::ControlSize::CHROME;
const PAINT_HEADER_GAP: f32 = tokens::Space::XS;
const PAINT_HEADER_HORIZONTAL_PADDING: f32 = tokens::Space::LG;
const PAINT_HEADER_TRAILING_CONTROL_COUNT: usize = 2;
const COLOR_AREA_HEIGHT: f32 = 248.;
const SELECTOR_INSET: f32 = tokens::Space::SM;
const KEYBOARD_FINE_STEP: f32 = 0.01;
const KEYBOARD_COARSE_STEP: f32 = 0.1;
const HUE_FINE_STEP: f32 = 1.;
const HUE_COARSE_STEP: f32 = 10.;

const fn paint_header_min_width() -> f32 {
    let controls = DesignPaintType::ALL.len() + PAINT_HEADER_TRAILING_CONTROL_COUNT;
    controls as f32 * PAINT_HEADER_CONTROL_SIZE
        // The zero-width flexible spacer between paint types and tools adds
        // one child, so the row has `controls` gaps rather than `controls - 1`.
        + controls as f32 * PAINT_HEADER_GAP
        + PAINT_HEADER_HORIZONTAL_PADDING
}

/// Opaque host identity for the paint currently shown by the picker.
#[derive(Clone, Debug, Eq)]
pub struct PaintPickerTarget {
    pub node_id: SharedString,
    pub collection: DesignPanelCollection,
    pub index: usize,
    pub paint_id: SharedString,
}

impl PartialEq for PaintPickerTarget {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
            && self.collection == other.collection
            && if self.paint_id.is_empty() || other.paint_id.is_empty() {
                self.index == other.index
            } else {
                self.paint_id == other.paint_id
            }
    }
}

impl PaintPickerTarget {
    fn exactly_eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
            && self.collection == other.collection
            && self.index == other.index
            && self.paint_id == other.paint_id
    }
}

#[derive(Clone, Debug, Eq)]
struct BlendModeMenuPreview {
    target: PaintPickerTarget,
    original: DesignBlendMode,
    candidate: DesignBlendMode,
}

impl PartialEq for BlendModeMenuPreview {
    fn eq(&self, other: &Self) -> bool {
        self.target.exactly_eq(&other.target)
            && self.original == other.original
            && self.candidate == other.candidate
    }
}

/// Typed intent emitted by the picker. The host accepts edits by supplying fresh data.
#[derive(Clone, Debug, PartialEq)]
pub enum PaintPickerAction {
    /// Dismiss the containing surface. Hosts should cancel any active edit before unmounting.
    CloseRequested,
    Edit {
        target: PaintPickerTarget,
        edit: Box<DesignPaintEdit>,
        phase: DesignPanelEditPhase,
    },
    BlendModePreview {
        target: PaintPickerTarget,
        original: DesignBlendMode,
        candidate: DesignBlendMode,
        phase: DesignMenuPreviewPhase,
    },
    SourceReplaceRequested {
        target: PaintPickerTarget,
    },
    MediaSourceActionRequested {
        target: PaintPickerTarget,
        source_id: SharedString,
        action: DesignMediaSourceAction,
    },
    MediaSourceDropRequested {
        target: PaintPickerTarget,
        expected_source_id: SharedString,
        expected_media_kind: DesignMediaKind,
        file: DesignMediaDroppedFile,
    },
    MediaCropActionRequested {
        target: PaintPickerTarget,
        action: DesignMediaCropAction,
    },
    VideoPreviewActionRequested {
        target: PaintPickerTarget,
        action: DesignVideoPreviewAction,
    },
    ShaderImportRequested {
        target: PaintPickerTarget,
        shader: DesignShaderSelection,
    },
    ShaderApplyRequested {
        target: PaintPickerTarget,
        shader: DesignShaderSelection,
    },
    ShaderPropertyBindRequested {
        target: PaintPickerTarget,
        definition_id: SharedString,
    },
    ShaderPropertyEditorRequested {
        target: PaintPickerTarget,
        definition_id: SharedString,
    },
    ShaderPropertyDetachRequested {
        target: PaintPickerTarget,
        definition_id: SharedString,
        variable_id: SharedString,
    },
    ColorVariableApplyRequested {
        target: PaintPickerTarget,
        color_target: DesignPaintColorTarget,
        variable_id: SharedString,
    },
    ColorVariableImportRequested {
        target: PaintPickerTarget,
        color_target: DesignPaintColorTarget,
        variable_id: SharedString,
    },
    ColorVariableDetachRequested {
        target: PaintPickerTarget,
        color_target: DesignPaintColorTarget,
        variable_id: SharedString,
    },
    ColorVariableCreateRequested {
        target: PaintPickerTarget,
        color_target: DesignPaintColorTarget,
        color: DesignColor,
    },
    /// Opens whole-Paint style creation from the exact picker leaf that
    /// exposed the creation menu. The surrounding panel revalidates the
    /// stable paint/gradient-stop identity before emitting its public action.
    PaintStyleCreateRequested {
        target: PaintPickerTarget,
        color_target: DesignPaintColorTarget,
    },
    ColorStyleSampleRequested {
        target: PaintPickerTarget,
        color_target: DesignPaintColorTarget,
        sample: DesignColorStyleSampleSelection,
    },
    EyedropperRequested {
        target: PaintPickerTarget,
        color_target: DesignPaintColorTarget,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MediaSettingsView {
    placement: DesignMediaPaintPlacement,
    filters: DesignImageFilters,
    crop_tool: DesignMediaCropToolState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContinuousControl {
    ColorArea,
    Hue,
    Alpha,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum PaintPickerTab {
    #[default]
    Custom,
    Libraries,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum PaintCreationKind {
    #[default]
    Style,
    Variable,
}

impl PaintCreationKind {
    const ALL: [Self; 2] = [Self::Style, Self::Variable];
    const VARIABLE_ONLY: [Self; 1] = [Self::Variable];

    const fn label(self) -> &'static str {
        match self {
            Self::Style => "Create style",
            Self::Variable => "Create variable",
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum PaintResourceScope {
    #[default]
    Page,
    Library {
        library_id: SharedString,
        library_name: SharedString,
    },
}

impl PaintResourceScope {
    fn label(&self) -> SharedString {
        match self {
            Self::Page => "Styles and variables on this page".into(),
            Self::Library { library_name, .. } => library_name.clone(),
        }
    }

    fn same_identity(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Page, Self::Page) => true,
            (
                Self::Library {
                    library_id: left, ..
                },
                Self::Library {
                    library_id: right, ..
                },
            ) => left == right,
            (Self::Page, Self::Library { .. }) | (Self::Library { .. }, Self::Page) => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ContinuousPaintEdit {
    edit: DesignPaintEdit,
}

#[derive(Clone)]
struct DragColorArea(EntityId);

impl Render for DragColorArea {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

/// One nested surface retained by the paint picker.
///
/// The outer Design inspector coordinates the paint-picker popover itself;
/// this identity is deliberately local to the retained child and covers only
/// menus rendered inside that popover.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PaintPickerOverlay {
    BlendMode,
    ColorFormat,
    GradientKind,
    PatternSource,
    Creation,
    ResourceScope,
}

#[derive(Clone, Debug)]
struct OpenPaintPickerOverlay {
    overlay: PaintPickerOverlay,
    focus_return: Option<InspectorOverlayFocusTarget>,
}

/// Typed stack for nested paint-picker menus.
///
/// The most recently opened surface is deterministically topmost. Covered
/// ancestors therefore cannot also react to the same Escape or outside click,
/// and every close path shares the inspector focus-return contract.
#[derive(Default)]
struct PaintPickerOverlayCoordinator {
    open: Vec<OpenPaintPickerOverlay>,
}

impl PaintPickerOverlayCoordinator {
    fn is_open(&self, overlay: PaintPickerOverlay) -> bool {
        self.open.iter().any(|entry| entry.overlay == overlay)
    }

    fn topmost(&self) -> Option<PaintPickerOverlay> {
        self.open.last().map(|entry| entry.overlay)
    }

    fn open(
        &mut self,
        overlay: PaintPickerOverlay,
        focus_return: Option<InspectorOverlayFocusTarget>,
    ) {
        let retained_focus = self
            .open
            .iter()
            .position(|entry| entry.overlay == overlay)
            .and_then(|index| self.open.remove(index).focus_return);
        self.open.push(OpenPaintPickerOverlay {
            overlay,
            focus_return: focus_return.or(retained_focus),
        });
    }

    fn dismissal_intent(
        &self,
        overlay: PaintPickerOverlay,
        cause: InspectorOverlayDismissCause,
    ) -> Option<InspectorOverlayDismissIntent<PaintPickerOverlay>> {
        (self.topmost() == Some(overlay)).then(|| {
            let focus_return = self
                .open
                .last()
                .and_then(|entry| entry.focus_return.clone());
            InspectorOverlayDismissIntent::new(overlay, cause, focus_return)
        })
    }

    fn close(
        &mut self,
        overlay: PaintPickerOverlay,
    ) -> Option<Option<InspectorOverlayFocusTarget>> {
        let index = self
            .open
            .iter()
            .position(|entry| entry.overlay == overlay)?;
        Some(self.open.remove(index).focus_return)
    }

    fn finish_dismissal(&mut self, intent: &InspectorOverlayDismissIntent<PaintPickerOverlay>) {
        let _ = self.close(*intent.overlay());
    }

    fn clear(&mut self) {
        self.open.clear();
    }
}

/// Stateful presentation for a single host-selected paint.
///
/// Keep one entity alive while the surrounding popover is mounted. Calling
/// [`set_target`](Self::set_target) replaces the controlled snapshot without
/// changing the target document.
pub struct PaintPicker {
    id: SharedString,
    focus_handle: FocusHandle,
    hue_slider: Entity<crate::molecules::Slider>,
    alpha_slider: Entity<crate::molecules::Slider>,
    target: Option<PaintPickerTarget>,
    paint: Option<DesignPaint>,
    paint_variable_view_data: DesignPaintVariableViewData,
    color_style_sample_view_data: DesignColorStyleSampleViewData,
    media_view_data: DesignMediaPaintViewData,
    paint_style_creation_enabled: bool,
    shader_view_data: DesignShaderViewData,
    supported_paint_types: Vec<DesignPaintType>,
    supported_blend_modes: Vec<DesignBlendMode>,
    disabled: bool,
    active_tab: PaintPickerTab,
    shader_browser_requested: bool,
    color_only_title: Option<SharedString>,
    color_only_color_editable: bool,
    color_only_opacity_editable: bool,
    color_format: ColorFormat,
    color_format_menu_index: usize,
    color_format_menu_focus_handle: FocusHandle,
    creation_menu_index: usize,
    creation_menu_focus_handle: FocusHandle,
    resource_scope: PaintResourceScope,
    resource_scope_menu_index: usize,
    resource_scope_menu_focus_handle: FocusHandle,
    gradient_kind_menu_index: usize,
    gradient_kind_menu_focus_handle: FocusHandle,
    nested_overlays: PaintPickerOverlayCoordinator,
    blend_mode_menu_preview: Option<BlendModeMenuPreview>,
    contrast_checker_open: bool,
    contrast_view_data: Option<DesignColorContrastPaintViewData>,
    contrast_category: DesignColorContrastCategory,
    contrast_level: DesignColorContrastLevel,
    scroll_handle: ScrollHandle,
    selected_stop: usize,
    selected_stop_id: SharedString,
    remembered_hue: f32,
    /// Exact saturation/value selected by the pointer. RGB quantization near
    /// black cannot faithfully reconstruct saturation, so retain the pointer
    /// coordinates for selector placement while this target remains active.
    color_area_position: Option<(f32, f32)>,
    color_area_bounds: Bounds<Pixels>,
    hue_bounds: Bounds<Pixels>,
    alpha_bounds: Bounds<Pixels>,
    gradient_bounds: Bounds<Pixels>,
    dragging_stop: Option<usize>,
    continuous_edit: Option<ContinuousPaintEdit>,
    resource_search_input: Entity<InputState>,
    hex_input: Entity<InputState>,
    color_channel_inputs: [Entity<InputState>; 3],
    opacity_input: Entity<InputState>,
    stop_position_input: Entity<InputState>,
    hex_invalid: bool,
    color_channel_invalid: [bool; 3],
    opacity_invalid: bool,
    stop_position_invalid: bool,
    suppress_input_events: bool,
    dismissal_event_guard: bool,
    /// A focused text field owns one Begin→(Preview)*→terminal session.
    ///
    /// Enter ends the session even though GPUI keeps the field focused; the
    /// following Blur must therefore be inert. A later Change while focus is
    /// retained opens a fresh session before emitting Preview.
    hex_edit_session: Option<DesignPaintEdit>,
    hex_original_opacity: Option<f32>,
    color_channel_edit_sessions: [Option<DesignPaintEdit>; 3],
    opacity_edit_session: Option<DesignPaintEdit>,
    stop_position_edit_session: Option<DesignPaintEdit>,
    ignore_next_hex_change: bool,
    ignore_next_color_channel_changes: [bool; 3],
    ignore_next_opacity_change: bool,
    ignore_next_stop_position_change: bool,
    _subscriptions: Vec<Subscription>,
}

pub(crate) use PaintPickerAction as PaintPickerEvent;

impl EventEmitter<PaintPickerEvent> for PaintPicker {}

impl Focusable for PaintPicker {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl PaintPicker {
    pub fn new(id: impl Into<SharedString>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let hue_slider = cx.new(|cx| crate::molecules::Slider::new("color-picker-hue", 0., cx));
        let alpha_slider = cx.new(|cx| crate::molecules::Slider::new("color-picker-alpha", 1., cx));
        let resource_search_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Search styles and variables"));
        let hex_input = cx.new(|cx| InputState::new(window, cx).placeholder("RRGGBB"));
        let color_channel_inputs = [
            cx.new(|cx| InputState::new(window, cx).placeholder("0")),
            cx.new(|cx| InputState::new(window, cx).placeholder("0")),
            cx.new(|cx| InputState::new(window, cx).placeholder("0")),
        ];
        let opacity_input = cx.new(|cx| InputState::new(window, cx).placeholder("100"));
        let stop_position_input = cx.new(|cx| InputState::new(window, cx).placeholder("0"));

        let hex_subscription = cx.subscribe_in(
            &hex_input,
            window,
            |this, _, event: &InputEvent, window, cx| {
                this.handle_hex_input(event, window, cx);
            },
        );
        let opacity_subscription = cx.subscribe_in(
            &opacity_input,
            window,
            |this, _, event: &InputEvent, window, cx| {
                this.handle_opacity_input(event, window, cx);
            },
        );
        let stop_position_subscription = cx.subscribe_in(
            &stop_position_input,
            window,
            |this, _, event: &InputEvent, window, cx| {
                this.handle_stop_position_input(event, window, cx);
            },
        );
        let resource_search_subscription =
            cx.subscribe(&resource_search_input, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            });
        let mut subscriptions = vec![
            cx.subscribe(
                &hue_slider,
                |this, _, action: &crate::molecules::SliderAction, cx| {
                    this.handle_slider_action(ContinuousControl::Hue, action, cx)
                },
            ),
            cx.subscribe(
                &alpha_slider,
                |this, _, action: &crate::molecules::SliderAction, cx| {
                    this.handle_slider_action(ContinuousControl::Alpha, action, cx)
                },
            ),
            resource_search_subscription,
            hex_subscription,
            opacity_subscription,
            stop_position_subscription,
        ];
        for (channel, input) in color_channel_inputs.iter().enumerate() {
            subscriptions.push(cx.subscribe_in(
                input,
                window,
                move |this, _, event: &InputEvent, window, cx| {
                    this.handle_color_channel_input(channel, event, window, cx);
                },
            ));
        }

        Self {
            hue_slider,
            alpha_slider,
            id: id.into(),
            focus_handle: cx.focus_handle(),
            target: None,
            paint: None,
            paint_variable_view_data: DesignPaintVariableViewData::default(),
            color_style_sample_view_data: DesignColorStyleSampleViewData::default(),
            media_view_data: DesignMediaPaintViewData::default(),
            paint_style_creation_enabled: true,
            shader_view_data: DesignShaderViewData::default(),
            supported_paint_types: DesignPaintType::ALL.to_vec(),
            supported_blend_modes: DesignBlendMode::NON_PASS_THROUGH.to_vec(),
            disabled: false,
            active_tab: PaintPickerTab::Custom,
            shader_browser_requested: false,
            color_only_title: None,
            color_only_color_editable: true,
            color_only_opacity_editable: true,
            color_format: ColorFormat::Hex,
            color_format_menu_index: ColorFormat::Hex.index(),
            color_format_menu_focus_handle: cx.focus_handle(),
            creation_menu_index: PaintCreationKind::Style as usize,
            creation_menu_focus_handle: cx.focus_handle(),
            resource_scope: PaintResourceScope::Page,
            resource_scope_menu_index: 0,
            resource_scope_menu_focus_handle: cx.focus_handle(),
            gradient_kind_menu_index: 0,
            gradient_kind_menu_focus_handle: cx.focus_handle(),
            nested_overlays: PaintPickerOverlayCoordinator::default(),
            blend_mode_menu_preview: None,
            contrast_checker_open: false,
            contrast_view_data: None,
            contrast_category: DesignColorContrastCategory::Auto,
            contrast_level: DesignColorContrastLevel::Aa,
            scroll_handle: ScrollHandle::new(),
            selected_stop: 0,
            selected_stop_id: "".into(),
            remembered_hue: 0.,
            color_area_position: None,
            color_area_bounds: Bounds::default(),
            hue_bounds: Bounds::default(),
            alpha_bounds: Bounds::default(),
            gradient_bounds: Bounds::default(),
            dragging_stop: None,
            continuous_edit: None,
            resource_search_input,
            hex_input,
            color_channel_inputs,
            opacity_input,
            stop_position_input,
            hex_invalid: false,
            color_channel_invalid: [false; 3],
            opacity_invalid: false,
            stop_position_invalid: false,
            suppress_input_events: false,
            dismissal_event_guard: false,
            hex_edit_session: None,
            hex_original_opacity: None,
            color_channel_edit_sessions: [None, None, None],
            opacity_edit_session: None,
            stop_position_edit_session: None,
            ignore_next_hex_change: false,
            ignore_next_color_channel_changes: [false; 3],
            ignore_next_opacity_change: false,
            ignore_next_stop_position_change: false,
            _subscriptions: subscriptions,
        }
    }

    pub fn has_open_menu(&self) -> bool {
        !self.nested_overlays.open.is_empty()
    }

    fn nested_overlay_is_open(&self, overlay: PaintPickerOverlay) -> bool {
        self.nested_overlays.is_open(overlay)
    }

    fn open_nested_overlay(&mut self, overlay: PaintPickerOverlay, window: &Window, cx: &App) {
        // Popover transfers pointer focus before on_open_change. Do not save
        // the menu being opened as its own return target: it disappears on close.
        let focus_return = window.focused(cx).map(|handle| {
            if overlay == PaintPickerOverlay::ColorFormat
                && handle == self.color_format_menu_focus_handle
            {
                InspectorOverlayFocusTarget::new(self.focus_handle.clone())
            } else {
                InspectorOverlayFocusTarget::new(handle)
            }
        });
        self.nested_overlays.open(overlay, focus_return);
    }

    fn cleanup_nested_overlay(&mut self, overlay: PaintPickerOverlay, cx: &mut Context<Self>) {
        if overlay == PaintPickerOverlay::BlendMode {
            self.cancel_blend_mode_preview(cx);
        }
    }

    fn close_nested_overlay(
        &mut self,
        overlay: PaintPickerOverlay,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(focus_return) = self.nested_overlays.close(overlay) else {
            return false;
        };
        self.cleanup_nested_overlay(overlay, cx);
        if let Some(focus_return) = focus_return {
            focus_return.restore(window, cx);
        }
        cx.notify();
        true
    }

    fn dismiss_nested_overlay(
        &mut self,
        overlay: PaintPickerOverlay,
        cause: InspectorOverlayDismissCause,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(intent) = self.nested_overlays.dismissal_intent(overlay, cause) else {
            return false;
        };
        self.cleanup_nested_overlay(overlay, cx);
        self.nested_overlays.finish_dismissal(&intent);
        intent.restore_focus(window, cx);
        cx.notify();
        true
    }

    fn forget_nested_overlay(&mut self, overlay: PaintPickerOverlay) -> bool {
        self.nested_overlays.close(overlay).is_some()
    }

    fn clear_nested_overlays(&mut self) {
        self.nested_overlays.clear();
    }

    pub fn set_supported_paint_types(
        &mut self,
        supported: &[DesignPaintType],
        cx: &mut Context<Self>,
    ) {
        let supported: Vec<_> = DesignPaintType::ALL
            .into_iter()
            .filter(|paint_type| supported.contains(paint_type))
            .collect();
        if self.supported_paint_types != supported {
            self.supported_paint_types = supported;
            cx.notify();
        }
    }

    pub fn set_supported_blend_modes(
        &mut self,
        supported: &[DesignBlendMode],
        cx: &mut Context<Self>,
    ) {
        let supported: Vec<_> = DesignBlendMode::NON_PASS_THROUGH
            .into_iter()
            .filter(|mode| supported.contains(mode))
            .collect();
        if self.supported_blend_modes != supported {
            self.supported_blend_modes = supported;
            cx.notify();
        }
    }

    /// Supplies the authoritative paint snapshot and opaque target identity.
    pub fn set_target(
        &mut self,
        node_id: impl Into<SharedString>,
        collection: DesignPanelCollection,
        index: usize,
        paint: DesignPaint,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let target = PaintPickerTarget {
            node_id: node_id.into(),
            collection,
            index,
            paint_id: paint.id.clone(),
        };
        let target_changed = self.target.as_ref() != Some(&target);
        let paint = normalize_paint(paint);
        let preview_invalidated = self.blend_mode_menu_preview.as_ref().is_some_and(|active| {
            !active.target.exactly_eq(&target) || active.original != paint.blend_mode
        });
        if preview_invalidated {
            self.cancel_blend_mode_preview(cx);
        }

        let previous_stop_id = self.selected_stop_id.clone();
        if target_changed {
            self.reset_resource_search(window, cx);
            self.active_tab = PaintPickerTab::Custom;
            self.shader_browser_requested = paint.paint_type() == DesignPaintType::Shader;
            self.clear_nested_overlays();
            self.creation_menu_index = PaintCreationKind::Style as usize;
            self.gradient_kind_menu_index = 0;
            self.contrast_checker_open = false;
            self.contrast_view_data = None;
            self.contrast_category = DesignColorContrastCategory::Auto;
            self.contrast_level = DesignColorContrastLevel::Aa;
            self.scroll_handle.set_offset(Point::default());
            self.selected_stop = 0;
            self.selected_stop_id = paint
                .gradient_stops
                .first()
                .map_or_else(|| "".into(), |stop| stop.id.clone());
            self.color_area_position = None;
            self.hex_invalid = false;
            self.color_channel_invalid = [false; 3];
            self.opacity_invalid = false;
            self.stop_position_invalid = false;
            self.dragging_stop = None;
            self.continuous_edit = None;
            self.reset_text_input_edit_sessions();
        } else {
            if !matches!(&paint.payload, DesignPaintPayload::Pattern(_)) {
                self.forget_nested_overlay(PaintPickerOverlay::PatternSource);
            }
            self.selected_stop = if !previous_stop_id.is_empty() {
                paint
                    .gradient_stops
                    .iter()
                    .position(|stop| stop.id == previous_stop_id)
                    .unwrap_or_else(|| clamp_stop_index(&paint, self.selected_stop))
            } else {
                clamp_stop_index(&paint, self.selected_stop)
            };
            self.selected_stop_id = paint
                .gradient_stops
                .get(self.selected_stop)
                .map_or_else(|| "".into(), |stop| stop.id.clone());
            if self.dragging_stop.is_some() {
                self.dragging_stop = Some(self.selected_stop);
            }
        }

        self.target = Some(target);
        self.paint = Some(paint);
        self.remember_current_hue();

        let preserve_hex_draft =
            !target_changed && self.hex_input.focus_handle(cx).is_focused(window);
        let preserve_opacity_draft =
            !target_changed && self.opacity_input.focus_handle(cx).is_focused(window);
        self.sync_inputs(
            window,
            cx,
            preserve_hex_draft,
            preserve_opacity_draft,
            !target_changed,
        );
        cx.notify();
    }

    /// Clears the target and returns the picker to its inert empty state.
    pub fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.cancel_blend_mode_preview(cx);
        self.reset_resource_search(window, cx);
        self.target = None;
        self.paint = None;
        self.active_tab = PaintPickerTab::Custom;
        self.shader_browser_requested = false;
        self.color_only_title = None;
        self.color_only_color_editable = true;
        self.color_only_opacity_editable = true;
        self.color_format = ColorFormat::Hex;
        self.clear_nested_overlays();
        self.creation_menu_index = PaintCreationKind::Style as usize;
        self.resource_scope = PaintResourceScope::Page;
        self.resource_scope_menu_index = 0;
        self.gradient_kind_menu_index = 0;
        self.contrast_checker_open = false;
        self.contrast_view_data = None;
        self.contrast_category = DesignColorContrastCategory::Auto;
        self.contrast_level = DesignColorContrastLevel::Aa;
        self.scroll_handle.set_offset(Point::default());
        self.selected_stop = 0;
        self.selected_stop_id = "".into();
        self.remembered_hue = 0.;
        self.color_area_position = None;
        self.hex_invalid = false;
        self.color_channel_invalid = [false; 3];
        self.opacity_invalid = false;
        self.stop_position_invalid = false;
        self.dragging_stop = None;
        self.continuous_edit = None;
        self.reset_text_input_edit_sessions();
        self.sync_inputs(window, cx, false, false, false);
        cx.notify();
    }

    /// Enables or disables every editing surface without discarding the target.
    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        if self.disabled != disabled {
            self.disabled = disabled;
            if disabled {
                self.cancel_blend_mode_preview(cx);
                self.reset_text_input_edit_sessions();
                for overlay in [
                    PaintPickerOverlay::Creation,
                    PaintPickerOverlay::ResourceScope,
                    PaintPickerOverlay::GradientKind,
                    PaintPickerOverlay::PatternSource,
                    PaintPickerOverlay::BlendMode,
                ] {
                    self.forget_nested_overlay(overlay);
                }
            }
            cx.notify();
        }
    }

    /// Suppresses deferred input notifications while the containing Popover
    /// is being dismissed. The next controlled target sync re-enables input
    /// handling.
    pub fn prepare_for_dismissal(&mut self, cx: &mut Context<Self>) {
        self.cancel_blend_mode_preview(cx);
        self.suppress_input_events = true;
        self.dismissal_event_guard = true;
        self.continuous_edit = None;
        self.dragging_stop = None;
        self.color_area_position = None;
        for overlay in [
            PaintPickerOverlay::Creation,
            PaintPickerOverlay::ResourceScope,
            PaintPickerOverlay::GradientKind,
            PaintPickerOverlay::PatternSource,
            PaintPickerOverlay::BlendMode,
        ] {
            self.forget_nested_overlay(overlay);
        }
        self.reset_text_input_edit_sessions();
    }

    /// Restricts the retained picker to the shared solid-color editor.
    ///
    /// Design-panel surfaces such as Page background and text decoration use
    /// the same RGB/HSL/HSB, hue, alpha, and keyboard interaction as paints
    /// without exposing paint-type or library controls.
    pub(crate) fn set_color_only(&mut self, title: Option<SharedString>, cx: &mut Context<Self>) {
        if self.color_only_title != title {
            self.cancel_blend_mode_preview(cx);
            self.color_only_title = title;
            if self.color_only_title.is_none() {
                self.color_only_color_editable = true;
                self.color_only_opacity_editable = true;
            }
            self.active_tab = PaintPickerTab::Custom;
            self.clear_nested_overlays();
            self.scroll_handle.set_offset(Point::default());
            cx.notify();
        }
    }

    /// Independently gates RGB/eyedropper and alpha edits for retained
    /// color-only pickers.
    ///
    /// This is presentation state only. It lets a host keep one picker
    /// inspectable when, for example, a layout-guide Color leaf is bound but
    /// its Opacity leaf remains editable. Global [`Self::set_disabled`] and a
    /// read-only paint still take precedence.
    pub(crate) fn set_color_only_editability(
        &mut self,
        color_editable: bool,
        opacity_editable: bool,
        cx: &mut Context<Self>,
    ) {
        if self.color_only_color_editable == color_editable
            && self.color_only_opacity_editable == opacity_editable
        {
            return;
        }
        self.cancel_text_input_edit_sessions(cx);
        self.color_only_color_editable = color_editable;
        self.color_only_opacity_editable = opacity_editable;
        cx.notify();
    }

    pub(crate) const fn color_only_editability(&self) -> (bool, bool) {
        (
            self.color_only_color_editable,
            self.color_only_opacity_editable,
        )
    }

    pub(crate) fn is_disabled(&self) -> bool {
        self.disabled
    }

    #[cfg(test)]
    pub(crate) fn properties_editing_is_disabled(&self) -> bool {
        self.editing_disabled()
    }

    #[cfg(test)]
    pub(crate) fn source_replacement_is_disabled(&self) -> bool {
        self.source_replacement_disabled()
    }

    #[cfg(test)]
    pub(crate) fn media_source_action_is_disabled(&self, action: DesignMediaSourceAction) -> bool {
        self.media_source_action_disabled(action)
    }

    pub fn target(&self) -> Option<&PaintPickerTarget> {
        self.target.as_ref()
    }

    pub fn paint(&self) -> Option<&DesignPaint> {
        self.paint.as_ref()
    }

    fn select_blend_mode(
        &mut self,
        blend_mode: DesignBlendMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.close_nested_overlay(PaintPickerOverlay::BlendMode, window, cx) {
            // Programmatic activation can exercise the same preview lifecycle
            // without mounting the menu; the terminal End remains mandatory.
            self.cancel_blend_mode_preview(cx);
        }
        if !self.supported_blend_modes.contains(&blend_mode) {
            cx.notify();
            return;
        }
        let _ = self.emit_edit(
            DesignPaintEdit {
                property: DesignPaintProperty::BlendMode,
                value: DesignPaintValue::BlendMode(blend_mode),
            },
            DesignPanelEditPhase::Commit,
            cx,
        );
        cx.notify();
    }

    fn set_blend_mode_preview(
        &mut self,
        blend_mode: DesignBlendMode,
        preview: bool,
        cx: &mut Context<Self>,
    ) {
        if !preview {
            if self
                .blend_mode_menu_preview
                .as_ref()
                .is_some_and(|active| active.candidate == blend_mode)
            {
                self.cancel_blend_mode_preview(cx);
            }
            return;
        }
        if self.editing_disabled() || blend_mode == DesignBlendMode::PassThrough {
            return;
        }
        let (Some(target), Some(paint)) = (self.target.clone(), self.paint.as_ref()) else {
            return;
        };
        if paint.blend_mode == blend_mode {
            return;
        }
        let next = BlendModeMenuPreview {
            target,
            original: paint.blend_mode,
            candidate: blend_mode,
        };
        if self.blend_mode_menu_preview.as_ref() == Some(&next) {
            return;
        }
        self.cancel_blend_mode_preview(cx);
        self.blend_mode_menu_preview = Some(next.clone());
        cx.emit(PaintPickerEvent::BlendModePreview {
            target: next.target,
            original: next.original,
            candidate: next.candidate,
            phase: DesignMenuPreviewPhase::Begin,
        });
    }

    fn cancel_blend_mode_preview(&mut self, cx: &mut Context<Self>) {
        let Some(active) = self.blend_mode_menu_preview.take() else {
            return;
        };
        cx.emit(PaintPickerEvent::BlendModePreview {
            target: active.target,
            original: active.original,
            candidate: active.candidate,
            phase: DesignMenuPreviewPhase::End,
        });
    }

    fn handle_picker_key(&self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if is_eyedropper_shortcut(event) {
            self.request_eyedropper(cx);
            window.prevent_default();
            cx.stop_propagation();
        }
    }

    fn reset_text_input_edit_sessions(&mut self) {
        self.hex_edit_session = None;
        self.hex_original_opacity = None;
        self.color_channel_edit_sessions = [None, None, None];
        self.opacity_edit_session = None;
        self.stop_position_edit_session = None;
        self.ignore_next_hex_change = false;
        self.ignore_next_color_channel_changes = [false; 3];
        self.ignore_next_opacity_change = false;
        self.ignore_next_stop_position_change = false;
    }

    fn cancel_text_input_edit_sessions(&mut self, cx: &mut Context<Self>) {
        let mut sessions = Vec::new();
        sessions.extend(self.hex_edit_session.take());
        for session in &mut self.color_channel_edit_sessions {
            sessions.extend(session.take());
        }
        sessions.extend(self.opacity_edit_session.take());
        sessions.extend(self.stop_position_edit_session.take());
        for session in sessions {
            self.finish_text_input_edit(session, None, cx);
        }
    }

    fn emit_edit(
        &self,
        edit: DesignPaintEdit,
        phase: DesignPanelEditPhase,
        cx: &mut Context<Self>,
    ) -> Option<DesignPaint> {
        if self.editing_disabled() {
            return None;
        }
        let mut paint = self.paint.clone()?;
        if phase != DesignPanelEditPhase::Begin && !paint.apply_edit(&edit) {
            return None;
        }
        if phase != DesignPanelEditPhase::Begin && self.paint.as_ref() == Some(&paint) {
            return None;
        }
        let target = self.target.clone()?;
        cx.emit(PaintPickerEvent::Edit {
            target,
            edit: Box::new(edit),
            phase,
        });
        Some(paint)
    }

    fn emit_payload(&self, paint: DesignPaint, cx: &mut Context<Self>) {
        if self.base_editing_disabled() || self.paint.as_ref() == Some(&paint) {
            return;
        }
        let Some(target) = self.target.clone() else {
            return;
        };
        let edit = DesignPaintEdit {
            property: DesignPaintProperty::Payload,
            value: DesignPaintValue::Payload(paint.payload.clone()),
        };
        cx.emit(PaintPickerEvent::Edit {
            target,
            edit: Box::new(edit),
            phase: DesignPanelEditPhase::Commit,
        });
    }

    fn begin_continuous_edit(&mut self, edit: DesignPaintEdit, cx: &mut Context<Self>) {
        self.cancel_text_input_edit_sessions(cx);
        self.continuous_edit = None;
        if self
            .emit_edit(edit.clone(), DesignPanelEditPhase::Begin, cx)
            .is_some()
        {
            self.continuous_edit = Some(ContinuousPaintEdit { edit });
        }
    }

    fn preview_continuous_edit(&mut self, edit: DesignPaintEdit, cx: &mut Context<Self>) {
        if self
            .emit_edit(edit.clone(), DesignPanelEditPhase::Preview, cx)
            .is_some()
        {
            self.continuous_edit = Some(ContinuousPaintEdit { edit });
        }
    }

    fn commit_continuous_edit(&mut self, cx: &mut Context<Self>) {
        let Some(continuous) = self.continuous_edit.take() else {
            return;
        };
        let Some(target) = self.target.clone() else {
            return;
        };
        cx.emit(PaintPickerEvent::Edit {
            target,
            edit: Box::new(continuous.edit),
            phase: DesignPanelEditPhase::Commit,
        });
    }

    fn select_paint_type(&mut self, paint_type: DesignPaintType, cx: &mut Context<Self>) {
        if self.base_editing_disabled() || !self.supported_paint_types.contains(&paint_type) {
            return;
        }
        self.cancel_text_input_edit_sessions(cx);
        let Some(paint) = self.paint.as_ref() else {
            return;
        };
        if paint_type == DesignPaintType::Shader && paint.paint_type() != DesignPaintType::Shader {
            self.active_tab = PaintPickerTab::Libraries;
            self.shader_browser_requested = true;
            self.scroll_handle.set_offset(Point::default());
            cx.notify();
            return;
        }
        self.shader_browser_requested = paint_type == DesignPaintType::Shader;
        let candidate = paint_for_type(paint, paint_type, self.selected_stop);
        self.selected_stop = clamp_stop_index(&candidate, self.selected_stop);
        self.selected_stop_id = candidate
            .gradient_stops
            .get(self.selected_stop)
            .map_or_else(|| "".into(), |stop| stop.id.clone());
        self.emit_payload(candidate, cx);
    }

    fn select_paint_kind(&mut self, kind: DesignPaintKind, cx: &mut Context<Self>) {
        if self.base_editing_disabled() || !self.supported_paint_types.contains(&kind.paint_type())
        {
            return;
        }
        if !kind.is_gradient() {
            self.select_paint_type(kind.paint_type(), cx);
            return;
        }
        self.cancel_text_input_edit_sessions(cx);
        let Some(paint) = self.paint.as_ref() else {
            return;
        };
        if !paint.kind.is_gradient() {
            let candidate = paint_for_kind(paint, kind, self.selected_stop);
            self.selected_stop = clamp_stop_index(&candidate, self.selected_stop);
            self.selected_stop_id = candidate
                .gradient_stops
                .get(self.selected_stop)
                .map_or_else(|| "".into(), |stop| stop.id.clone());
            self.emit_payload(candidate, cx);
            return;
        }
        let _ = self.emit_edit(
            DesignPaintEdit {
                property: DesignPaintProperty::GradientKind,
                value: DesignPaintValue::PaintKind(kind),
            },
            DesignPanelEditPhase::Commit,
            cx,
        );
    }
}

impl Render for PaintPicker {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id(SharedString::from(format!("{}-surface", self.id)))
            .occlude()
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .on_pinch(|_, _, cx| cx.stop_propagation())
            .on_action(cx.listener(|_, _: &CancelDesignInteraction, _, cx| {
                cx.emit(PaintPickerAction::CloseRequested);
                // The surrounding inspector also dismisses its host-owned overlay.
                cx.propagate();
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" && !this.has_open_menu() {
                    cx.emit(PaintPickerAction::CloseRequested);
                    window.prevent_default();
                    cx.stop_propagation();
                }
            }))
            .on_action(|_: &gpui_component::input::Enter, window, cx| {
                window.prevent_default();
                cx.stop_propagation();
            })
            .child(self.render_picker(window, cx))
    }
}

fn is_eyedropper_shortcut(event: &KeyDownEvent) -> bool {
    let modifiers = event.keystroke.modifiers;
    let unmodified_i = event.keystroke.key.eq_ignore_ascii_case("i") && !modifiers.modified();
    let mac_control_c = cfg!(target_os = "macos")
        && event.keystroke.key.eq_ignore_ascii_case("c")
        && modifiers.control
        && !modifiers.alt
        && !modifiers.shift
        && !modifiers.platform
        && !modifiers.function;
    unmodified_i || mac_control_c
}

fn paint_picker_locked(paint: &DesignPaint) -> bool {
    paint.read_only || matches!(&paint.payload, DesignPaintPayload::Unsupported(_))
}

fn paint_color_locked(paint: &DesignPaint, selected_stop: usize) -> bool {
    paint_picker_locked(paint) || paint.selected_color_is_bound(selected_stop)
}

fn selected_color_binding(
    paint: &DesignPaint,
    selected_stop: usize,
) -> Option<&DesignPaintBinding> {
    match &paint.payload {
        DesignPaintPayload::Solid(solid) => solid.binding.as_ref(),
        DesignPaintPayload::Gradient(gradient) => gradient
            .stops
            .get(selected_stop.min(gradient.stops.len().saturating_sub(1)))
            .and_then(|stop| stop.binding.as_ref()),
        _ => None,
    }
}

fn paint_variable_matches_binding(
    variable: &DesignVariable,
    selected_binding: Option<&DesignPaintBinding>,
) -> bool {
    selected_binding.is_some_and(|binding| binding.variable_id == variable.id)
}

fn paint_variable_color(variable: &DesignVariable) -> DesignColor {
    match variable.resolved_value.as_ref() {
        Some(DesignVariableResolvedValue::Color(color)) => *color,
        _ => DesignColor::BLACK,
    }
}

fn paint_resource_matches<'a>(
    normalized_query: &str,
    values: impl IntoIterator<Item = &'a str>,
) -> bool {
    normalized_query.is_empty()
        || values
            .into_iter()
            .any(|value| value.to_lowercase().contains(normalized_query))
}

fn paint_color_target(paint: &DesignPaint, selected_stop: usize) -> Option<DesignPaintColorTarget> {
    match &paint.payload {
        DesignPaintPayload::Solid(_) => Some(DesignPaintColorTarget::Solid),
        DesignPaintPayload::Gradient(gradient) => {
            let index = selected_stop.min(gradient.stops.len().saturating_sub(1));
            let stop = gradient.stops.get(index)?;
            Some(DesignPaintColorTarget::GradientStop {
                stop_id: stop.id.clone(),
                index,
            })
        }
        DesignPaintPayload::Pattern(_)
        | DesignPaintPayload::Image(_)
        | DesignPaintPayload::Video(_)
        | DesignPaintPayload::Shader(_)
        | DesignPaintPayload::Unsupported(_) => None,
    }
}

fn selected_color(paint: &DesignPaint, selected_stop: usize) -> DesignColor {
    if paint.kind.is_gradient() {
        paint
            .gradient_stops
            .get(clamp_stop_index(paint, selected_stop))
            .map_or(paint.color, |stop| stop.color)
    } else {
        paint.color
    }
}

fn picker_opacity(paint: &DesignPaint, selected_stop: usize) -> f32 {
    if paint.kind.is_gradient() {
        f32::from(selected_color(paint, selected_stop).alpha) / 255. * 100.
    } else {
        paint.opacity.clamp(0., 100.)
    }
}

fn picker_opacity_edit(paint: &DesignPaint, selected_stop: usize, opacity: f32) -> DesignPaintEdit {
    let opacity = opacity.clamp(0., 100.);
    if paint.kind.is_gradient() {
        let mut color = selected_color(paint, selected_stop);
        color.alpha = channel_from_unit(opacity / 100.);
        color_edit(paint, selected_stop, color)
    } else {
        DesignPaintEdit {
            property: DesignPaintProperty::Opacity,
            value: DesignPaintValue::Number(opacity),
        }
    }
}

fn rgb_edit_color(
    paint: &DesignPaint,
    selected_stop: usize,
    mut color: DesignColor,
) -> DesignColor {
    color.alpha = if paint.kind.is_gradient() {
        selected_color(paint, selected_stop).alpha
    } else {
        u8::MAX
    };
    color
}

fn color_input_edit(
    paint: &DesignPaint,
    selected_stop: usize,
    parsed: ParsedColorInput,
) -> DesignPaintEdit {
    let mut color = rgb_edit_color(paint, selected_stop, parsed.color);
    if parsed.explicit_alpha && paint.kind.is_gradient() {
        color.alpha = parsed.color.alpha;
    }
    color_edit(paint, selected_stop, color)
}

fn picker_color_with_opacity(paint: &DesignPaint, selected_stop: usize) -> DesignColor {
    let mut color = selected_color(paint, selected_stop);
    color.alpha = channel_from_unit(picker_opacity(paint, selected_stop) / 100.);
    color
}

fn color_text_value(format: ColorFormat, paint: &DesignPaint, selected_stop: usize) -> String {
    let color = picker_color_with_opacity(paint, selected_stop);
    match format {
        ColorFormat::Css => css_rgba(color),
        ColorFormat::Hex | ColorFormat::Rgb | ColorFormat::Hsl | ColorFormat::Hsb => {
            rgba_hex(color)
        }
    }
}

fn normalize_paint(mut paint: DesignPaint) -> DesignPaint {
    paint.opacity = if paint.opacity.is_finite() {
        paint.opacity.clamp(0., 100.)
    } else {
        100.
    };
    if let DesignPaintPayload::Gradient(gradient) = &mut paint.payload {
        gradient.stops.retain(|stop| stop.position.is_finite());
        for stop in &mut gradient.stops {
            stop.position = stop.position.clamp(0., 1.);
        }
        gradient.stops.sort_by(|left, right| {
            left.position
                .partial_cmp(&right.position)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        match gradient.stops.as_slice() {
            [] => {
                let mut transparent = paint.color;
                transparent.alpha = 0;
                gradient.stops = vec![
                    DesignGradientStop::new(0., paint.color),
                    DesignGradientStop::new(1., transparent),
                ];
            }
            [only] => {
                let second = DesignGradientStop::new(1., only.color);
                gradient.stops.push(second);
            }
            _ => {}
        }
    }
    paint.sync_legacy_projection();
    paint
}

fn paint_for_kind(paint: &DesignPaint, kind: DesignPaintKind, selected_stop: usize) -> DesignPaint {
    let mut candidate = paint_for_type(paint, kind.paint_type(), selected_stop);
    if let DesignPaintPayload::Gradient(gradient) = &mut candidate.payload
        && kind.is_gradient()
    {
        gradient.kind = kind;
        candidate.sync_legacy_projection();
    }
    candidate
}

fn paint_for_type(
    paint: &DesignPaint,
    paint_type: DesignPaintType,
    selected_stop: usize,
) -> DesignPaint {
    if paint.paint_type() == paint_type {
        return paint.clone();
    }

    let color = selected_color(paint, selected_stop);
    let mut candidate = paint.clone();
    candidate.payload = match paint_type {
        DesignPaintType::Solid => DesignPaintPayload::Solid(DesignSolidPaint {
            color,
            binding: None,
        }),
        DesignPaintType::Gradient => {
            let mut transparent = color;
            transparent.alpha = 0;
            DesignPaintPayload::Gradient(DesignGradientPaint {
                kind: DesignPaintKind::LinearGradient,
                stops: vec![
                    DesignGradientStop::new(0., color),
                    DesignGradientStop::new(1., transparent),
                ],
                transform: DesignPaintTransform::IDENTITY,
            })
        }
        DesignPaintType::Pattern => DesignPaintPayload::Pattern(DesignPatternPaint {
            source_node_id: "".into(),
            tile_type: DesignPatternTileType::Rectangular,
            scaling_factor: 1.,
            spacing: DesignPatternSpacing::default(),
            horizontal_alignment: DesignPatternHorizontalAlignment::Center,
        }),
        DesignPaintType::Image => DesignPaintPayload::Image(DesignImagePaint {
            source: DesignPaintSource::default(),
            placement: DesignMediaPaintPlacement::default(),
            filters: DesignImageFilters::default(),
        }),
        DesignPaintType::Video => DesignPaintPayload::Video(DesignVideoPaint {
            source: DesignPaintSource::default(),
            placement: DesignMediaPaintPlacement::default(),
            filters: DesignImageFilters::default(),
        }),
        DesignPaintType::Shader | DesignPaintType::Unsupported => return candidate,
    };
    candidate.sync_legacy_projection();
    candidate
}

fn color_edit(paint: &DesignPaint, selected_stop: usize, color: DesignColor) -> DesignPaintEdit {
    let property = if paint.kind.is_gradient() {
        let index = clamp_stop_index(paint, selected_stop);
        DesignPaintProperty::GradientStopColor {
            stop_id: paint.gradient_stops[index].id.clone(),
            index,
        }
    } else {
        DesignPaintProperty::Color
    };
    DesignPaintEdit {
        property,
        value: DesignPaintValue::Color(color),
    }
}

#[cfg(test)]
mod tests;
