//! Atoms: the crate's smallest shared interaction and presentation units.
//!
//! The single Enter/Space activation command, the activatable-control builder
//! extensions, the compact button atom, canonical Lucide icons, and
//! the bounds/truncation utilities. Molecules (`crate::molecules`) and the
//! feature organisms compose these instead of re-implementing key contexts,
//! focus rings, and listener pairs per control (ARCHITECTURE.md §9, §16).
//!
//! This module is the curated public atoms API: hosts build custom chrome
//! from these pieces so their controls share the library's activation,
//! focus-ring, icon, and truncation contracts.
//!
//! [`tokens`] is the shared geometry scale behind that chrome, reached as a
//! path (`atoms::tokens::RowHeight::FIELD`) rather than re-exported flat.

use gpui::{App, KeyBinding, actions};

mod activatable;
mod bounds;
mod button;
mod checkbox;
mod color_system;
mod dropdown;
mod input;
mod lucide;
mod radio_button;
mod segmented_control;
mod semantic_button;
mod slider;
mod tabs;
pub mod tokens;
mod tooltip;
mod truncate;
mod typography;
pub use slider::{
    SliderBackground, SliderGradientStop, SliderHandle, SliderHandleVariant, SliderState,
};

pub use activatable::{ActivateEvent, ButtonControlExt, ControlExt};
pub use bounds::track_bounds;
pub use button::icon_button;
pub use checkbox::{Checkbox, CheckboxState, CheckboxType};
pub use color_system::{SemanticColor, SemanticColorGroup};
pub use dropdown::{Dropdown, DropdownSize, DropdownState};
pub use input::{
    ColorChit, ColorChitKind, ColorChitShape, ColorChitSize, ColorInput, ColorInputKind,
    ComboInput, ComboInputDropdown, ComboInputState, FantaTextInput, InputChip, InputChipState,
    InputSize, InputVisualState, NumericInput, NumericInputMulti, TextInputLabel, TextInputVariant,
    VariableCell, VariableChip, VariableChipState,
};
pub use lucide::{LucideIcon, render_lucide_icon};
pub use radio_button::{RadioButton, RadioButtonSelection, RadioButtonState, RadioButtonVariant};
pub use segmented_control::{
    SegmentIcon, SegmentIconOption, SegmentLabel, SegmentState, SegmentedControl,
    SegmentedControlSelection, SegmentedControlState, SegmentedControlVariant,
};
pub use semantic_button::{
    SemanticButton, SemanticButtonIconAlignment, SemanticButtonSize, SemanticButtonState,
    SemanticButtonVariant, SemanticIconButton, SemanticIconButtonKind, SemanticSplitButtonState,
    semantic_button, semantic_icon_button, ui_button,
};
pub use tabs::{Tab, TabSelection, TabState, Tabs};
pub use tooltip::{Tooltip, TooltipDirection, TooltipLink, TooltipLinkAction, TooltipLinkVariant};
pub use truncate::truncating_label;
pub use typography::{TypographyExt, TypographyStyle, TypographyToken};

actions!(fanta_controls, [ActivateControl]);

/// Key context used by focusable custom controls that behave like buttons.
///
/// §16 contract: [`crate::init`] binds Enter/Space to [`ActivateControl`] in
/// this context; the atoms builders set it automatically, so callers only
/// reach for it when hand-building a focusable control from raw elements.
pub const CONTROL_KEY_CONTEXT: &str = "FantaControl";

pub(crate) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("enter", ActivateControl, Some(CONTROL_KEY_CONTEXT)),
        KeyBinding::new("space", ActivateControl, Some(CONTROL_KEY_CONTEXT)),
    ]);
}
