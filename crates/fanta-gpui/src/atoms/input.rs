//! Controlled UI3 input families and their supporting input atoms.
//!
//! Editable text is always supplied as a host-owned [`InputState`]. These
//! elements own presentation only; the host observes `InputEvent` and commits
//! domain changes at its own boundary.

use std::rc::Rc;

use gpui::{
    AnyElement, App, ElementId, Entity, Focusable as _, Hsla, InteractiveElement as _, IntoElement,
    ParentElement as _, RenderOnce, SharedString, Styled as _, Window, div, linear_color_stop,
    linear_gradient, pattern_slash, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    Sizable as _, h_flex,
    input::{Input, InputState},
};

use super::{
    ActivateEvent, CONTROL_KEY_CONTEXT, ControlExt as _, LucideIcon, SemanticColor,
    TypographyExt as _, TypographyToken, render_lucide_icon, tokens,
};

type ActivateHandler = Rc<dyn Fn(&ActivateEvent, &mut Window, &mut App)>;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum InputSize {
    #[default]
    Default,
    Large,
}

impl InputSize {
    pub const ALL: [Self; 2] = [Self::Default, Self::Large];

    pub const fn height(self) -> f32 {
        match self {
            Self::Default => tokens::RowHeight::FIELD,
            Self::Large => tokens::ControlSize::TOOL,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum InputVisualState {
    #[default]
    Default,
    Hover,
    Focused,
    Active,
    Variable,
    Empty,
    Disabled,
}

impl InputVisualState {
    pub const ALL: [Self; 7] = [
        Self::Default,
        Self::Hover,
        Self::Focused,
        Self::Active,
        Self::Variable,
        Self::Empty,
        Self::Disabled,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Hover => "Hover",
            Self::Focused => "Focused",
            Self::Active => "Active",
            Self::Variable => "Variable",
            Self::Empty => "Empty",
            Self::Disabled => "Disabled",
        }
    }
}

fn foreground(state: InputVisualState, cx: &App) -> Hsla {
    if state == InputVisualState::Disabled {
        SemanticColor::TextDisabled.resolve(cx)
    } else {
        SemanticColor::Text.resolve(cx)
    }
}

fn shell_background(state: InputVisualState, cx: &App) -> Hsla {
    match state {
        InputVisualState::Disabled => SemanticColor::BackgroundDisabled.resolve(cx),
        InputVisualState::Hover => SemanticColor::BackgroundHover.resolve(cx),
        InputVisualState::Active => SemanticColor::BackgroundActive.resolve(cx),
        InputVisualState::Variable => SemanticColor::BackgroundSelected.resolve(cx),
        _ => SemanticColor::Background.resolve(cx),
    }
}

fn shell_border(state: InputVisualState, focused: bool, cx: &App) -> Hsla {
    if focused || matches!(state, InputVisualState::Focused | InputVisualState::Active) {
        SemanticColor::BorderSelected.resolve(cx)
    } else {
        SemanticColor::Border.resolve(cx)
    }
}

fn input_editor(
    state: &Entity<InputState>,
    size: InputSize,
    disabled: bool,
    multiline: bool,
) -> Input {
    let input = Input::new(state)
        .appearance(false)
        .bordered(false)
        .focus_bordered(false)
        .disabled(disabled)
        .w_full()
        .min_w_0()
        .px(px(tokens::Space::SM));
    if multiline {
        input.h(px(tokens::InputGeometry::MULTILINE_HEIGHT))
    } else {
        match size {
            InputSize::Default => input.xsmall(),
            InputSize::Large => input.small(),
        }
    }
}

fn accessory(icon: LucideIcon, color: Hsla) -> AnyElement {
    div()
        .size(px(tokens::RowHeight::FIELD))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .child(render_lucide_icon(icon, color, tokens::IconSize::XS))
        .into_any_element()
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextInputVariant {
    #[default]
    SingleLine,
    MultiLine,
    QuickAction,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextInputLabel {
    #[default]
    Text,
    Variable,
}

/// UI3 text input backed by a host-owned `InputState`.
#[derive(IntoElement)]
pub struct FantaTextInput {
    state: Entity<InputState>,
    size: InputSize,
    visual_state: InputVisualState,
    variant: TextInputVariant,
    label: TextInputLabel,
    leading_icon: Option<LucideIcon>,
    dropdown: bool,
    chip: bool,
}

impl FantaTextInput {
    pub fn new(state: &Entity<InputState>) -> Self {
        Self {
            state: state.clone(),
            size: InputSize::Default,
            visual_state: InputVisualState::Default,
            variant: TextInputVariant::SingleLine,
            label: TextInputLabel::Text,
            leading_icon: None,
            dropdown: false,
            chip: false,
        }
    }

    pub const fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }
    pub const fn preview_state(mut self, state: InputVisualState) -> Self {
        self.visual_state = state;
        self
    }
    pub const fn variant(mut self, variant: TextInputVariant) -> Self {
        self.variant = variant;
        self
    }
    pub const fn label(mut self, label: TextInputLabel) -> Self {
        self.label = label;
        self
    }
    pub const fn leading_icon(mut self, icon: LucideIcon) -> Self {
        self.leading_icon = Some(icon);
        self
    }
    pub const fn dropdown(mut self, dropdown: bool) -> Self {
        self.dropdown = dropdown;
        self
    }
    pub const fn chip(mut self, chip: bool) -> Self {
        self.chip = chip;
        self
    }
}

impl RenderOnce for FantaTextInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.visual_state == InputVisualState::Disabled;
        let multiline = self.variant == TextInputVariant::MultiLine;
        let focus = self.state.read(cx).focus_handle(cx);
        let focused = focus.is_focused(window);
        let height = if multiline {
            tokens::InputGeometry::MULTILINE_HEIGHT
        } else {
            self.size.height()
        };
        let icon_color = if disabled {
            SemanticColor::IconDisabled.resolve(cx)
        } else {
            SemanticColor::IconSecondary.resolve(cx)
        };
        let editor = input_editor(&self.state, self.size, disabled, multiline);

        h_flex()
            .track_focus(&focus)
            .w(px(tokens::InputGeometry::TEXT_WIDTH))
            .h(px(height))
            .items_center()
            .overflow_hidden()
            .rounded(px(tokens::ButtonGeometry::RADIUS))
            .border_1()
            .border_color(shell_border(self.visual_state, focused, cx))
            .bg(shell_background(self.visual_state, cx))
            .hover(|style| style.bg(SemanticColor::BackgroundHover.resolve(cx)))
            .when_some(self.leading_icon, |field, icon| {
                field.child(accessory(icon, icon_color))
            })
            .when(
                self.chip || self.label == TextInputLabel::Variable,
                |field| {
                    field.child(
                        div().pl(px(tokens::Space::XS)).child(
                            VariableChip::new("text-input-variable", "Variable")
                                .state(VariableChipState::Default),
                        ),
                    )
                },
            )
            .child(editor)
            .when(self.variant == TextInputVariant::QuickAction, |field| {
                field.child(accessory(LucideIcon::CornerDownLeft, icon_color))
            })
            .when(self.dropdown, |field| {
                field.child(accessory(LucideIcon::ChevronDown, icon_color))
            })
    }
}

/// Compact numeric input. Input filtering/validation is configured on its
/// host-owned `InputState`, allowing applications to choose integer, decimal,
/// signed, or unit-aware syntax without hiding domain policy in the atom.
#[derive(IntoElement)]
pub struct NumericInput {
    state: Entity<InputState>,
    visual_state: InputVisualState,
    disabled: bool,
    variable_icon: bool,
    variable_pill: bool,
    dropdown: bool,
    leading_icon: Option<LucideIcon>,
    trailing_icon: Option<LucideIcon>,
}

impl NumericInput {
    pub fn new(state: &Entity<InputState>) -> Self {
        Self {
            state: state.clone(),
            visual_state: InputVisualState::Default,
            disabled: false,
            variable_icon: false,
            variable_pill: false,
            dropdown: false,
            leading_icon: None,
            trailing_icon: None,
        }
    }

    pub const fn preview_state(mut self, state: InputVisualState) -> Self {
        self.visual_state = state;
        self
    }
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub const fn variable_icon(mut self, visible: bool) -> Self {
        self.variable_icon = visible;
        self
    }
    pub const fn variable_pill(mut self, visible: bool) -> Self {
        self.variable_pill = visible;
        self
    }
    pub const fn dropdown(mut self, visible: bool) -> Self {
        self.dropdown = visible;
        self
    }
    pub const fn leading_icon(mut self, icon: LucideIcon) -> Self {
        self.leading_icon = Some(icon);
        self
    }
    pub const fn trailing_icon(mut self, icon: LucideIcon) -> Self {
        self.trailing_icon = Some(icon);
        self
    }
}

impl RenderOnce for NumericInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.disabled || self.visual_state == InputVisualState::Disabled;
        let focus = self.state.read(cx).focus_handle(cx);
        let focused = focus.is_focused(window);
        let icon_color = if disabled {
            SemanticColor::IconDisabled.resolve(cx)
        } else {
            SemanticColor::IconSecondary.resolve(cx)
        };
        let mut field = h_flex()
            .track_focus(&focus)
            .w(px(tokens::InputGeometry::NUMERIC_WIDTH))
            .h(px(tokens::RowHeight::FIELD))
            .items_center()
            .overflow_hidden()
            .rounded(px(tokens::ButtonGeometry::RADIUS))
            .border_1()
            .border_color(shell_border(self.visual_state, focused, cx))
            .bg(shell_background(
                if disabled {
                    InputVisualState::Disabled
                } else {
                    self.visual_state
                },
                cx,
            ))
            .hover(|style| style.bg(SemanticColor::BackgroundHover.resolve(cx)));
        if let Some(icon) = self.leading_icon {
            field = field.child(accessory(icon, icon_color));
        }
        if self.variable_pill {
            field = field.child(VariableChip::new("numeric-variable", "24"));
        } else {
            field = field.child(input_editor(
                &self.state,
                InputSize::Default,
                disabled,
                false,
            ));
        }
        if self.variable_icon {
            field = field.child(accessory(
                LucideIcon::Variable,
                SemanticColor::IconBrand.resolve(cx),
            ));
        }
        if let Some(icon) = self.trailing_icon {
            field = field.child(accessory(icon, icon_color));
        }
        if self.dropdown {
            field = field.child(accessory(LucideIcon::ChevronDown, icon_color));
        }
        field
    }
}

/// A grouped numeric field. Each cell receives a separate host-owned input
/// state, so partial disable and independent editing remain possible.
#[derive(IntoElement)]
pub struct NumericInputMulti {
    states: Vec<Entity<InputState>>,
    visual_state: InputVisualState,
    partial_disable: bool,
    leading_icon: Option<LucideIcon>,
}

impl NumericInputMulti {
    pub fn new(states: impl IntoIterator<Item = Entity<InputState>>) -> Self {
        Self {
            states: states.into_iter().collect(),
            visual_state: InputVisualState::Default,
            partial_disable: false,
            leading_icon: None,
        }
    }
    pub const fn preview_state(mut self, state: InputVisualState) -> Self {
        self.visual_state = state;
        self
    }
    pub const fn partial_disable(mut self, partial: bool) -> Self {
        self.partial_disable = partial;
        self
    }
    pub const fn leading_icon(mut self, icon: LucideIcon) -> Self {
        self.leading_icon = Some(icon);
        self
    }
}

impl RenderOnce for NumericInputMulti {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focused = self
            .states
            .iter()
            .any(|state| state.read(cx).focus_handle(cx).is_focused(window));
        let mut group = h_flex()
            .h(px(tokens::RowHeight::FIELD))
            .items_center()
            .overflow_hidden()
            .rounded(px(tokens::ButtonGeometry::RADIUS))
            .border_1()
            .border_color(shell_border(self.visual_state, focused, cx))
            .bg(shell_background(self.visual_state, cx));
        if let Some(icon) = self.leading_icon {
            group = group.child(accessory(icon, SemanticColor::IconSecondary.resolve(cx)));
        }
        let len = self.states.len();
        group.children(self.states.into_iter().enumerate().map(|(index, state)| {
            let disabled = self.visual_state == InputVisualState::Disabled
                || (self.partial_disable && index + 1 == len);
            div()
                .w(px(tokens::InputGeometry::MULTI_CELL_WIDTH))
                .h_full()
                .flex_none()
                .border_l_1()
                .border_color(SemanticColor::Border.resolve(cx))
                .bg(if disabled {
                    SemanticColor::BackgroundDisabled.resolve(cx)
                } else {
                    SemanticColor::Background.resolve(cx)
                })
                .child(input_editor(&state, InputSize::Default, disabled, false))
        }))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ColorInputKind {
    #[default]
    Fill,
    Opacity,
    Image,
    Gradient,
    Variable,
}

/// Compound color value input with a swatch/type preview and opacity segment.
#[derive(IntoElement)]
pub struct ColorInput {
    state: Entity<InputState>,
    kind: ColorInputKind,
    visual_state: InputVisualState,
    color: Hsla,
    opacity: u8,
}

impl ColorInput {
    pub fn new(state: &Entity<InputState>, color: Hsla) -> Self {
        Self {
            state: state.clone(),
            kind: ColorInputKind::Fill,
            visual_state: InputVisualState::Default,
            color,
            opacity: 100,
        }
    }
    pub const fn kind(mut self, kind: ColorInputKind) -> Self {
        self.kind = kind;
        self
    }
    pub const fn preview_state(mut self, state: InputVisualState) -> Self {
        self.visual_state = state;
        self
    }
    pub const fn opacity(mut self, opacity: u8) -> Self {
        self.opacity = opacity;
        self
    }
}

impl RenderOnce for ColorInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.visual_state == InputVisualState::Disabled;
        let focus = self.state.read(cx).focus_handle(cx);
        let focused = focus.is_focused(window);
        let chit_kind = match self.kind {
            ColorInputKind::Fill | ColorInputKind::Variable => ColorChitKind::Fill,
            ColorInputKind::Opacity => ColorChitKind::Opacity,
            ColorInputKind::Image => ColorChitKind::Image,
            ColorInputKind::Gradient => ColorChitKind::Gradient,
        };
        h_flex()
            .track_focus(&focus)
            .w(px(tokens::InputGeometry::COLOR_WIDTH))
            .h(px(tokens::RowHeight::FIELD))
            .items_center()
            .overflow_hidden()
            .rounded(px(tokens::ButtonGeometry::RADIUS))
            .border_1()
            .border_color(shell_border(self.visual_state, focused, cx))
            .bg(shell_background(self.visual_state, cx))
            .child(
                ColorChit::new(self.color)
                    .kind(chit_kind)
                    .size(ColorChitSize::Small),
            )
            .child(input_editor(
                &self.state,
                InputSize::Default,
                disabled,
                false,
            ))
            .when(self.kind == ColorInputKind::Variable, |field| {
                field.child(accessory(
                    LucideIcon::Variable,
                    SemanticColor::IconBrand.resolve(cx),
                ))
            })
            .child(
                h_flex()
                    .h_full()
                    .px(px(tokens::Space::SM))
                    .flex_none()
                    .border_l_1()
                    .border_color(SemanticColor::Border.resolve(cx))
                    .typography(TypographyToken::BodyMedium)
                    .text_color(foreground(self.visual_state, cx))
                    .child(format!("{}%", self.opacity)),
            )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ComboInputState {
    #[default]
    Default,
    Hover,
    SelectedChevron,
    SelectedInput,
}

/// Numeric/text input paired with the dedicated UI3 combo dropdown segment.
#[derive(IntoElement)]
pub struct ComboInput {
    id: ElementId,
    state: Entity<InputState>,
    visual_state: ComboInputState,
    leading_icon: Option<LucideIcon>,
    variable: bool,
    on_dropdown: Option<ActivateHandler>,
}

impl ComboInput {
    pub fn new(id: impl Into<ElementId>, state: &Entity<InputState>) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            visual_state: ComboInputState::Default,
            leading_icon: None,
            variable: false,
            on_dropdown: None,
        }
    }
    pub const fn preview_state(mut self, state: ComboInputState) -> Self {
        self.visual_state = state;
        self
    }
    pub const fn leading_icon(mut self, icon: LucideIcon) -> Self {
        self.leading_icon = Some(icon);
        self
    }
    pub const fn variable(mut self, variable: bool) -> Self {
        self.variable = variable;
        self
    }
    pub fn on_dropdown(
        mut self,
        handler: impl Fn(&ActivateEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dropdown = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for ComboInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus = self.state.read(cx).focus_handle(cx);
        let focused =
            focus.is_focused(window) || self.visual_state == ComboInputState::SelectedInput;
        let input_state = if self.visual_state == ComboInputState::Hover {
            InputVisualState::Hover
        } else if focused {
            InputVisualState::Focused
        } else {
            InputVisualState::Default
        };
        h_flex()
            .id(self.id)
            .track_focus(&focus)
            .w(px(tokens::InputGeometry::COMBO_WIDTH))
            .h(px(tokens::RowHeight::FIELD))
            .items_center()
            .overflow_hidden()
            .rounded(px(tokens::ButtonGeometry::RADIUS))
            .border_1()
            .border_color(shell_border(input_state, focused, cx))
            .bg(shell_background(input_state, cx))
            .when_some(self.leading_icon, |field, icon| {
                field.child(accessory(icon, SemanticColor::IconSecondary.resolve(cx)))
            })
            .child(input_editor(&self.state, InputSize::Default, false, false))
            .when(self.variable, |field| {
                field.child(accessory(
                    LucideIcon::Variable,
                    SemanticColor::IconBrand.resolve(cx),
                ))
            })
            .child(
                ComboInputDropdown::new("combo-input-dropdown")
                    .preview_state(if self.visual_state == ComboInputState::SelectedChevron {
                        InputVisualState::Active
                    } else if self.visual_state == ComboInputState::Hover {
                        InputVisualState::Hover
                    } else {
                        InputVisualState::Default
                    })
                    .when_some(self.on_dropdown, |dropdown, handler| {
                        dropdown.on_activate(move |event, window, cx| handler(event, window, cx))
                    }),
            )
    }
}

/// The dedicated trailing dropdown segment from a combo input.
#[derive(IntoElement)]
pub struct ComboInputDropdown {
    id: ElementId,
    state: InputVisualState,
    on_activate: Option<ActivateHandler>,
}

impl ComboInputDropdown {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            state: InputVisualState::Default,
            on_activate: None,
        }
    }
    pub const fn preview_state(mut self, state: InputVisualState) -> Self {
        self.state = state;
        self
    }
    pub fn on_activate(
        mut self,
        handler: impl Fn(&ActivateEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_activate = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for ComboInputDropdown {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let handler = self.on_activate;
        div()
            .id(self.id)
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h_full()
            .w(px(tokens::RowHeight::FIELD))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .border_l_1()
            .border_color(if self.state == InputVisualState::Active {
                SemanticColor::BorderSelected.resolve(cx)
            } else {
                SemanticColor::Border.resolve(cx)
            })
            .bg(shell_background(self.state, cx))
            .cursor_pointer()
            .hover(|style| style.bg(SemanticColor::BackgroundHover.resolve(cx)))
            .focus(|style| style.bg(SemanticColor::BackgroundActive.resolve(cx)))
            .child(render_lucide_icon(
                LucideIcon::ChevronDown,
                SemanticColor::IconSecondary.resolve(cx),
                tokens::IconSize::XS,
            ))
            .when_some(handler, |button, handler| {
                button.on_activate(move |event, window, cx| handler(event, window, cx))
            })
    }
}

/// Compact variable value cell.
#[derive(IntoElement)]
pub struct VariableCell {
    value: SharedString,
}

impl VariableCell {
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl RenderOnce for VariableCell {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .w(px(tokens::InputGeometry::VARIABLE_CELL_WIDTH))
            .h(px(tokens::RowHeight::FIELD))
            .px(px(tokens::Space::XS))
            .gap(px(tokens::Space::XS))
            .rounded(px(tokens::ButtonGeometry::RADIUS))
            .border_1()
            .border_color(SemanticColor::Border.resolve(cx))
            .bg(SemanticColor::Background.resolve(cx))
            .child(render_lucide_icon(
                LucideIcon::Variable,
                SemanticColor::IconBrand.resolve(cx),
                tokens::IconSize::XS,
            ))
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .typography(TypographyToken::BodyMedium)
                    .child(self.value),
            )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum InputChipState {
    #[default]
    Default,
    Focused,
}

/// Small editable-value chip shell with optional close affordance.
#[derive(IntoElement)]
pub struct InputChip {
    id: ElementId,
    value: SharedString,
    state: InputChipState,
    close_button: bool,
    on_activate: Option<ActivateHandler>,
}

impl InputChip {
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            state: InputChipState::Default,
            close_button: false,
            on_activate: None,
        }
    }
    pub const fn preview_state(mut self, state: InputChipState) -> Self {
        self.state = state;
        self
    }
    pub const fn close_button(mut self, visible: bool) -> Self {
        self.close_button = visible;
        self
    }
    pub fn on_activate(
        mut self,
        handler: impl Fn(&ActivateEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_activate = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for InputChip {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let handler = self.on_activate;
        h_flex()
            .id(self.id)
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h(px(tokens::RowHeight::FIELD))
            .px(px(tokens::Space::SM))
            .gap(px(tokens::Space::XS))
            .rounded(px(tokens::ButtonGeometry::RADIUS))
            .border_1()
            .border_color(if self.state == InputChipState::Focused {
                SemanticColor::BorderSelected.resolve(cx)
            } else {
                SemanticColor::Border.resolve(cx)
            })
            .bg(SemanticColor::Background.resolve(cx))
            .typography(TypographyToken::BodyMedium)
            .child(self.value)
            .when(self.close_button, |chip| {
                chip.child(render_lucide_icon(
                    LucideIcon::X,
                    SemanticColor::IconSecondary.resolve(cx),
                    tokens::IconSize::XS,
                ))
            })
            .when_some(handler, |chip, handler| {
                chip.on_activate(move |event, window, cx| handler(event, window, cx))
            })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum VariableChipState {
    #[default]
    Default,
    Selected,
    OnSelected,
    Hover,
    SoftDeleted,
    DisabledSecondary,
    DisabledTertiary,
    ValueNotRendered,
}

/// Variable-backed value chip matching the Figma state taxonomy.
#[derive(IntoElement)]
pub struct VariableChip {
    id: ElementId,
    value: SharedString,
    state: VariableChipState,
}

impl VariableChip {
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            state: VariableChipState::Default,
        }
    }
    pub const fn state(mut self, state: VariableChipState) -> Self {
        self.state = state;
        self
    }
}

impl RenderOnce for VariableChip {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = matches!(
            self.state,
            VariableChipState::DisabledSecondary | VariableChipState::DisabledTertiary
        );
        let background = match self.state {
            VariableChipState::Selected => SemanticColor::BackgroundActive.resolve(cx),
            VariableChipState::OnSelected => SemanticColor::BackgroundSelected.resolve(cx),
            VariableChipState::Hover => SemanticColor::BackgroundHover.resolve(cx),
            _ => SemanticColor::Background.resolve(cx),
        };
        h_flex()
            .id(self.id)
            .h(px(tokens::RowHeight::FIELD))
            .px(px(tokens::Space::SM))
            .gap(px(tokens::Space::XS))
            .rounded(px(tokens::ButtonGeometry::RADIUS))
            .border_1()
            .border_color(if self.state == VariableChipState::OnSelected {
                SemanticColor::BorderSelected.resolve(cx)
            } else {
                SemanticColor::Border.resolve(cx)
            })
            .bg(background)
            .opacity(
                if disabled || self.state == VariableChipState::SoftDeleted {
                    0.48
                } else {
                    1.
                },
            )
            .child(render_lucide_icon(
                LucideIcon::Variable,
                if disabled {
                    SemanticColor::IconDisabled.resolve(cx)
                } else {
                    SemanticColor::IconBrand.resolve(cx)
                },
                tokens::IconSize::XS,
            ))
            .child(
                div()
                    .typography(TypographyToken::BodyMedium)
                    .text_color(if disabled {
                        SemanticColor::TextDisabled.resolve(cx)
                    } else {
                        SemanticColor::Text.resolve(cx)
                    })
                    .when(self.state == VariableChipState::SoftDeleted, |label| {
                        label.line_through()
                    })
                    .child(if self.state == VariableChipState::ValueNotRendered {
                        SharedString::from("—")
                    } else {
                        self.value
                    }),
            )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ColorChitKind {
    #[default]
    Fill,
    Opacity,
    Image,
    Gradient,
    Instance,
}
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ColorChitShape {
    #[default]
    Square,
    Circle,
}
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ColorChitSize {
    #[default]
    Small,
    Large,
}

/// 24/48 px color and media preview used by color inputs and menus.
#[derive(IntoElement)]
pub struct ColorChit {
    color: Hsla,
    kind: ColorChitKind,
    shape: ColorChitShape,
    size: ColorChitSize,
}

impl ColorChit {
    pub fn new(color: Hsla) -> Self {
        Self {
            color,
            kind: ColorChitKind::Fill,
            shape: ColorChitShape::Square,
            size: ColorChitSize::Small,
        }
    }
    pub const fn kind(mut self, kind: ColorChitKind) -> Self {
        self.kind = kind;
        self
    }
    pub const fn shape(mut self, shape: ColorChitShape) -> Self {
        self.shape = shape;
        self
    }
    pub const fn size(mut self, size: ColorChitSize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for ColorChit {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let size = match self.size {
            ColorChitSize::Small => tokens::InputGeometry::CHIT_SMALL,
            ColorChitSize::Large => tokens::InputGeometry::CHIT_LARGE,
        };
        let icon_size = match self.size {
            ColorChitSize::Small => tokens::IconSize::XS,
            ColorChitSize::Large => tokens::IconSize::MD,
        };
        let base = div()
            .relative()
            .size(px(size))
            .flex_none()
            .overflow_hidden()
            .when(self.shape == ColorChitShape::Square, |chit| {
                chit.rounded(px(tokens::ButtonGeometry::RADIUS))
            })
            .when(self.shape == ColorChitShape::Circle, |chit| {
                chit.rounded_full()
            })
            .border_1()
            .border_color(SemanticColor::Border.resolve(cx));
        match self.kind {
            ColorChitKind::Fill => base.bg(self.color),
            ColorChitKind::Opacity => base
                .bg(pattern_slash(
                    SemanticColor::TextTertiary.resolve(cx).opacity(0.25),
                    0.35,
                    0.35,
                ))
                .child(
                    div()
                        .size_full()
                        .when(self.shape == ColorChitShape::Square, |overlay| {
                            overlay.rounded(px(tokens::ButtonGeometry::RADIUS))
                        })
                        .when(self.shape == ColorChitShape::Circle, |overlay| {
                            overlay.rounded_full()
                        })
                        .bg(self.color.opacity(0.52)),
                ),
            ColorChitKind::Gradient => base.bg(linear_gradient(
                90.,
                linear_color_stop(self.color, 0.),
                linear_color_stop(SemanticColor::BackgroundAssistive.resolve(cx), 1.),
            )),
            ColorChitKind::Image => base
                .bg(pattern_slash(
                    SemanticColor::TextTertiary.resolve(cx).opacity(0.25),
                    0.35,
                    0.35,
                ))
                .flex()
                .items_center()
                .justify_center()
                .child(render_lucide_icon(
                    LucideIcon::Image,
                    SemanticColor::Icon.resolve(cx),
                    icon_size,
                )),
            ColorChitKind::Instance => base
                .bg(SemanticColor::BackgroundSecondary.resolve(cx))
                .flex()
                .items_center()
                .justify_center()
                .child(render_lucide_icon(
                    LucideIcon::Component,
                    SemanticColor::IconBrand.resolve(cx),
                    icon_size,
                )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_taxonomy_matches_the_figma_sets() {
        assert_eq!(InputSize::ALL.len(), 2);
        assert_eq!(InputVisualState::ALL.len(), 7);
        assert_eq!(InputSize::Default.height(), 24.);
        assert_eq!(InputSize::Large.height(), 32.);
    }
}
