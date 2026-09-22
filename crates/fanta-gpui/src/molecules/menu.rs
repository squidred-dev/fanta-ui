//! Shared context/popover menu molecule.
//!
//! Layers and Pages each grew a hand-rolled absolutely-positioned menu with
//! its own clamping and chrome. This module owns the shared pieces once:
//! [`clamp_menu_origin`] keeps a menu inside its container on both axes,
//! [`menu_surface`] is the popover chrome, and [`menu_item`] is the row
//! baseline. Callers keep ownership of dismissal, focus continuity, children,
//! and deferral: attach `on_mouse_down_out`, `track_scroll`, first-item
//! `track_focus`, and wrap the surface in `deferred(...).with_priority(...)`
//! at the call site.

use std::rc::Rc;

use crate::atoms::TypographyExt as _;
use gpui::{
    AnyElement, App, Div, ElementId, InteractiveElement as _, IntoElement, ParentElement as _,
    Pixels, Point, RenderOnce, SharedString, Size, Stateful, StatefulInteractiveElement as _,
    Styled as _, Window, div, point, prelude::FluentBuilder as _, px,
};
use gpui_component::{h_flex, v_flex};

use crate::atoms::{
    ActivateEvent, CONTROL_KEY_CONTEXT, ControlExt as _, LucideIcon, SemanticColor,
    TypographyToken, render_lucide_icon, tokens,
};

type ActivateHandler = Rc<dyn Fn(&ActivateEvent, &mut Window, &mut App)>;

/// Visual states in Figma's menu-row component sets.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum MenuRowState {
    #[default]
    Default,
    Hover,
    Disabled,
}

impl MenuRowState {
    pub const ALL: [Self; 3] = [Self::Default, Self::Hover, Self::Disabled];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Hover => "Hover",
            Self::Disabled => "Disabled",
        }
    }
}

/// Leading content on a complex menu row.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum MenuLead {
    #[default]
    None,
    Icon(LucideIcon),
    Avatar(SharedString),
}

/// Trailing content on a complex menu row.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum MenuTrail {
    #[default]
    None,
    Shortcut(SharedString),
    Badge(SharedString),
    Checkbox(bool),
    Mixed,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum MenuCheckVariant {
    #[default]
    Check,
    Dot,
}

impl MenuCheckVariant {
    pub const ALL: [Self; 2] = [Self::Check, Self::Dot];
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum MenuHeadingAlignment {
    #[default]
    Default,
    Toggle,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum MenuMultiSelectVariant {
    #[default]
    Default,
    MixedIcons,
    Avatars,
    LabelOnly,
}

impl MenuMultiSelectVariant {
    pub const ALL: [Self; 4] = [
        Self::Default,
        Self::MixedIcons,
        Self::Avatars,
        Self::LabelOnly,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::MixedIcons => "Mixed icons",
            Self::Avatars => "Avatars",
            Self::LabelOnly => "Label only",
        }
    }
}

fn menu_row_shell(id: ElementId, state: MenuRowState, cx: &App) -> Stateful<Div> {
    let disabled = state == MenuRowState::Disabled;
    let highlighted = state == MenuRowState::Hover;
    let foreground = if disabled {
        SemanticColor::TextDisabled.resolve(cx)
    } else {
        SemanticColor::Text.resolve(cx)
    };

    h_flex()
        .id(id)
        .key_context(CONTROL_KEY_CONTEXT)
        .tab_index(if disabled { -1 } else { 0 })
        .h(px(tokens::RowHeight::MENU))
        .flex_none()
        .mx(px(tokens::Space::XS))
        .px(px(tokens::Space::SM))
        .gap(px(tokens::Space::SM))
        .items_center()
        .rounded(px(tokens::MenuGeometry::ROW_RADIUS))
        .typography(TypographyToken::BodyMedium)
        .text_color(foreground)
        .when(highlighted, |row| {
            row.bg(SemanticColor::BackgroundHover.resolve(cx))
        })
        .when(!disabled, |row| {
            row.cursor_pointer()
                .hover(|style| style.bg(SemanticColor::BackgroundHover.resolve(cx)))
                .focus(|style| style.bg(SemanticColor::BackgroundHover.resolve(cx)))
        })
}

fn menu_foreground(state: MenuRowState, cx: &App) -> gpui::Hsla {
    match state {
        MenuRowState::Disabled => SemanticColor::IconDisabled.resolve(cx),
        MenuRowState::Hover | MenuRowState::Default => SemanticColor::Icon.resolve(cx),
    }
}

fn menu_secondary(state: MenuRowState, cx: &App) -> gpui::Hsla {
    match state {
        MenuRowState::Disabled => SemanticColor::TextDisabled.resolve(cx),
        MenuRowState::Hover | MenuRowState::Default => SemanticColor::TextSecondary.resolve(cx),
    }
}

fn menu_label(label: SharedString) -> AnyElement {
    div()
        .min_w_0()
        .flex_1()
        .truncate()
        .child(label)
        .into_any_element()
}

fn indicator_box(value: Option<bool>, state: MenuRowState, cx: &App) -> AnyElement {
    let color = menu_foreground(state, cx);
    div()
        .size(px(tokens::MenuGeometry::INDICATOR_SIZE))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(tokens::Radius::CONTROL))
        .border_1()
        .border_color(color)
        .when_some(value, |box_, checked| {
            box_.child(render_lucide_icon(
                if checked {
                    LucideIcon::Check
                } else {
                    LucideIcon::Minus
                },
                color,
                tokens::IconSize::XS,
            ))
        })
        .into_any_element()
}

fn menu_toggle(on: bool, cx: &App) -> AnyElement {
    let background = if on {
        SemanticColor::BackgroundBrand.resolve(cx)
    } else {
        SemanticColor::BackgroundDisabled.resolve(cx)
    };
    let knob = SemanticColor::Background.resolve(cx);

    h_flex()
        .w(px(tokens::MenuGeometry::TOGGLE_WIDTH))
        .h(px(tokens::MenuGeometry::TOGGLE_HEIGHT))
        .flex_none()
        .items_center()
        .rounded_full()
        .p(px((tokens::MenuGeometry::TOGGLE_HEIGHT
            - tokens::MenuGeometry::TOGGLE_KNOB)
            / 2.))
        .bg(background)
        .when(on, |track| track.justify_end())
        .child(
            div()
                .size(px(tokens::MenuGeometry::TOGGLE_KNOB))
                .rounded_full()
                .bg(knob),
        )
        .into_any_element()
}

/// UI3 `Menu row/Simple`: label, optional shortcut, and optional submenu.
#[derive(IntoElement)]
pub struct MenuSimpleRow {
    id: ElementId,
    label: SharedString,
    state: MenuRowState,
    shortcut: Option<SharedString>,
    submenu: bool,
    on_activate: Option<ActivateHandler>,
}

impl MenuSimpleRow {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: MenuRowState::Default,
            shortcut: None,
            submenu: false,
            on_activate: None,
        }
    }

    pub const fn preview_state(mut self, state: MenuRowState) -> Self {
        self.state = state;
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub const fn submenu(mut self, submenu: bool) -> Self {
        self.submenu = submenu;
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

impl RenderOnce for MenuSimpleRow {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.state == MenuRowState::Disabled;
        let foreground = menu_foreground(self.state, cx);
        let secondary = menu_secondary(self.state, cx);
        menu_row_shell(self.id, self.state, cx)
            .child(menu_label(self.label))
            .when_some(self.shortcut, |row, shortcut| {
                row.child(div().flex_none().text_color(secondary).child(shortcut))
            })
            .when(self.submenu, |row| {
                row.child(render_lucide_icon(
                    LucideIcon::ChevronRight,
                    foreground,
                    tokens::IconSize::XS,
                ))
            })
            .when_some(self.on_activate.filter(|_| !disabled), |row, handler| {
                row.on_activate(move |event, window, cx| handler(event, window, cx))
            })
    }
}

/// UI3 `Menu row/Complex`: independent leading and trailing content axes.
#[derive(IntoElement)]
pub struct MenuComplexRow {
    id: ElementId,
    label: SharedString,
    state: MenuRowState,
    lead: MenuLead,
    trail: MenuTrail,
    on_activate: Option<ActivateHandler>,
}

impl MenuComplexRow {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: MenuRowState::Default,
            lead: MenuLead::None,
            trail: MenuTrail::None,
            on_activate: None,
        }
    }

    pub const fn preview_state(mut self, state: MenuRowState) -> Self {
        self.state = state;
        self
    }

    pub fn lead(mut self, lead: MenuLead) -> Self {
        self.lead = lead;
        self
    }

    pub fn trail(mut self, trail: MenuTrail) -> Self {
        self.trail = trail;
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

impl RenderOnce for MenuComplexRow {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.state == MenuRowState::Disabled;
        let foreground = menu_foreground(self.state, cx);
        let secondary = menu_secondary(self.state, cx);
        let lead = self.lead;
        let trail = self.trail;
        menu_row_shell(self.id, self.state, cx)
            .when(!matches!(lead, MenuLead::None), |row| {
                row.child(match lead {
                    MenuLead::None => div().into_any_element(),
                    MenuLead::Icon(icon) => {
                        render_lucide_icon(icon, foreground, tokens::IconSize::XS)
                    }
                    MenuLead::Avatar(initials) => div()
                        .size(px(tokens::MenuGeometry::AVATAR_SIZE))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_full()
                        .bg(SemanticColor::BackgroundAssistive.resolve(cx))
                        .text_color(SemanticColor::TextOnBrand.resolve(cx))
                        .typography(TypographyToken::BodySmallStrong)
                        .child(initials)
                        .into_any_element(),
                })
            })
            .child(menu_label(self.label))
            .child(match trail {
                MenuTrail::None => div().into_any_element(),
                MenuTrail::Shortcut(shortcut) => div()
                    .flex_none()
                    .text_color(secondary)
                    .child(shortcut)
                    .into_any_element(),
                MenuTrail::Badge(badge) => div()
                    .h(px(tokens::MenuGeometry::BADGE_HEIGHT))
                    .flex_none()
                    .flex()
                    .items_center()
                    .px(px(tokens::Space::XS))
                    .rounded_full()
                    .bg(SemanticColor::BackgroundSelected.resolve(cx))
                    .typography(TypographyToken::BodySmallStrong)
                    .child(badge)
                    .into_any_element(),
                MenuTrail::Checkbox(checked) => {
                    indicator_box(if checked { Some(true) } else { None }, self.state, cx)
                }
                MenuTrail::Mixed => indicator_box(Some(false), self.state, cx),
            })
            .when_some(self.on_activate.filter(|_| !disabled), |row, handler| {
                row.on_activate(move |event, window, cx| handler(event, window, cx))
            })
    }
}

/// UI3 `Menu row/Checkmark`: controlled check or radio-dot selection.
#[derive(IntoElement)]
pub struct MenuCheckRow {
    id: ElementId,
    label: SharedString,
    state: MenuRowState,
    variant: MenuCheckVariant,
    on: bool,
    submenu: bool,
    shortcut: Option<SharedString>,
    on_activate: Option<ActivateHandler>,
}

impl MenuCheckRow {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: MenuRowState::Default,
            variant: MenuCheckVariant::Check,
            on: false,
            submenu: false,
            shortcut: None,
            on_activate: None,
        }
    }

    pub const fn preview_state(mut self, state: MenuRowState) -> Self {
        self.state = state;
        self
    }
    pub const fn variant(mut self, variant: MenuCheckVariant) -> Self {
        self.variant = variant;
        self
    }
    pub const fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }
    pub const fn submenu(mut self, submenu: bool) -> Self {
        self.submenu = submenu;
        self
    }
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
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

impl RenderOnce for MenuCheckRow {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.state == MenuRowState::Disabled;
        let foreground = menu_foreground(self.state, cx);
        let secondary = menu_secondary(self.state, cx);
        menu_row_shell(self.id, self.state, cx)
            .child(
                div()
                    .size(px(tokens::MenuGeometry::INDICATOR_SIZE))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(self.on, |slot| match self.variant {
                        MenuCheckVariant::Check => slot.child(render_lucide_icon(
                            LucideIcon::Check,
                            foreground,
                            tokens::IconSize::XS,
                        )),
                        MenuCheckVariant::Dot => slot.child(
                            div()
                                .size(px(tokens::Space::XS))
                                .rounded_full()
                                .bg(foreground),
                        ),
                    }),
            )
            .child(menu_label(self.label))
            .when_some(self.shortcut, |row, shortcut| {
                row.child(div().flex_none().text_color(secondary).child(shortcut))
            })
            .when(self.submenu, |row| {
                row.child(render_lucide_icon(
                    LucideIcon::ChevronRight,
                    foreground,
                    tokens::IconSize::XS,
                ))
            })
            .when_some(self.on_activate.filter(|_| !disabled), |row, handler| {
                row.on_activate(move |event, window, cx| handler(event, window, cx))
            })
    }
}

/// UI3 `Menu row/Toggle`: controlled on/off state with an optional icon.
#[derive(IntoElement)]
pub struct MenuToggleRow {
    id: ElementId,
    label: SharedString,
    state: MenuRowState,
    on: bool,
    icon: Option<LucideIcon>,
    shortcut: Option<SharedString>,
    on_activate: Option<ActivateHandler>,
}

impl MenuToggleRow {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: MenuRowState::Default,
            on: false,
            icon: None,
            shortcut: None,
            on_activate: None,
        }
    }
    pub const fn preview_state(mut self, state: MenuRowState) -> Self {
        self.state = state;
        self
    }
    pub const fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }
    pub const fn icon(mut self, icon: LucideIcon) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
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

impl RenderOnce for MenuToggleRow {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.state == MenuRowState::Disabled;
        let foreground = menu_foreground(self.state, cx);
        let secondary = menu_secondary(self.state, cx);
        menu_row_shell(self.id, self.state, cx)
            .child(menu_toggle(self.on, cx))
            .when_some(self.icon, |row, icon| {
                row.child(render_lucide_icon(icon, foreground, tokens::IconSize::XS))
            })
            .child(menu_label(self.label))
            .when_some(self.shortcut, |row, shortcut| {
                row.child(div().flex_none().text_color(secondary).child(shortcut))
            })
            .when_some(self.on_activate.filter(|_| !disabled), |row, handler| {
                row.on_activate(move |event, window, cx| handler(event, window, cx))
            })
    }
}

/// UI3 `Menu row/Toolbar`: tool icon, controlled selection, and shortcut.
#[derive(IntoElement)]
pub struct MenuToolbarRow {
    id: ElementId,
    label: SharedString,
    icon: LucideIcon,
    state: MenuRowState,
    on: bool,
    shortcut: Option<SharedString>,
    on_activate: Option<ActivateHandler>,
}

impl MenuToolbarRow {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, icon: LucideIcon) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon,
            state: MenuRowState::Default,
            on: false,
            shortcut: None,
            on_activate: None,
        }
    }
    pub const fn preview_state(mut self, state: MenuRowState) -> Self {
        self.state = state;
        self
    }
    pub const fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
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

impl RenderOnce for MenuToolbarRow {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.state == MenuRowState::Disabled;
        let foreground = menu_foreground(self.state, cx);
        let secondary = menu_secondary(self.state, cx);
        menu_row_shell(self.id, self.state, cx)
            .child(
                div()
                    .size(px(tokens::MenuGeometry::INDICATOR_SIZE))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(self.on, |slot| {
                        slot.child(render_lucide_icon(
                            LucideIcon::Check,
                            foreground,
                            tokens::IconSize::XS,
                        ))
                    }),
            )
            .child(render_lucide_icon(
                self.icon,
                foreground,
                tokens::IconSize::XS,
            ))
            .child(menu_label(self.label))
            .when_some(self.shortcut, |row, shortcut| {
                row.child(div().flex_none().text_color(secondary).child(shortcut))
            })
            .when_some(self.on_activate.filter(|_| !disabled), |row, handler| {
                row.on_activate(move |event, window, cx| handler(event, window, cx))
            })
    }
}

#[derive(IntoElement)]
pub struct MenuHeading {
    text: SharedString,
    alignment: MenuHeadingAlignment,
    toggle_text: Option<SharedString>,
}

impl MenuHeading {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            alignment: MenuHeadingAlignment::Default,
            toggle_text: None,
        }
    }
    pub const fn alignment(mut self, alignment: MenuHeadingAlignment) -> Self {
        self.alignment = alignment;
        self
    }
    pub fn toggle_text(mut self, text: impl Into<SharedString>) -> Self {
        self.toggle_text = Some(text.into());
        self
    }
}

impl RenderOnce for MenuHeading {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .h(px(tokens::RowHeight::MENU))
            .flex_none()
            .px(px(tokens::Space::MD))
            .items_center()
            .typography(TypographyToken::BodySmallStrong)
            .text_color(SemanticColor::TextSecondary.resolve(cx))
            .child(menu_label(self.text))
            .when(self.alignment == MenuHeadingAlignment::Toggle, |row| {
                row.when_some(self.toggle_text, |row, text| row.child(text))
            })
    }
}

#[derive(IntoElement)]
pub struct MenuDivider;

impl RenderOnce for MenuDivider {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .h(px(tokens::Space::SM))
            .flex_none()
            .flex()
            .items_center()
            .child(
                div()
                    .h(px(tokens::MenuGeometry::DIVIDER_HEIGHT))
                    .w_full()
                    .bg(SemanticColor::BorderMenu.resolve(cx)),
            )
    }
}

#[derive(IntoElement)]
pub struct MenuExpandRow {
    id: ElementId,
    label: SharedString,
    state: MenuRowState,
    expanded: bool,
    on_activate: Option<ActivateHandler>,
}

impl MenuExpandRow {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            state: MenuRowState::Default,
            expanded: false,
            on_activate: None,
        }
    }
    pub const fn preview_state(mut self, state: MenuRowState) -> Self {
        self.state = state;
        self
    }
    pub const fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
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

impl RenderOnce for MenuExpandRow {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let foreground = menu_foreground(self.state, cx);
        menu_row_shell(self.id, self.state, cx)
            .child(menu_label(self.label))
            .child(render_lucide_icon(
                if self.expanded {
                    LucideIcon::ChevronUp
                } else {
                    LucideIcon::ChevronDown
                },
                foreground,
                tokens::IconSize::XS,
            ))
            .when_some(self.on_activate, |row, handler| {
                row.on_activate(move |event, window, cx| handler(event, window, cx))
            })
    }
}

#[derive(IntoElement)]
pub struct MenuFooter {
    text: SharedString,
}

impl MenuFooter {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for MenuFooter {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .min_h(px(tokens::RowHeight::MENU))
            .flex()
            .items_center()
            .justify_center()
            .px(px(tokens::Space::MD))
            .typography(TypographyToken::BodySmall)
            .text_color(SemanticColor::TextSecondary.resolve(cx))
            .child(self.text)
    }
}

/// Inline menu panel chrome for stories and permanently mounted menus.
pub fn menu_panel(id: impl Into<ElementId>, width: Pixels, cx: &App) -> Stateful<Div> {
    v_flex()
        .id(id)
        .w(width)
        .py(px(tokens::Space::SM))
        .rounded(px(tokens::Radius::MENU))
        .border_1()
        .border_color(SemanticColor::BorderMenu.resolve(cx))
        .bg(SemanticColor::BackgroundMenu.resolve(cx))
        .shadow_lg()
}

/// Clamps `anchor` so a menu of size `menu` stays inside `container`.
///
/// Both axes clamp independently and floor at zero, so an oversized menu pins
/// to the container's top-left edge instead of escaping it.
pub fn clamp_menu_origin(
    anchor: Point<Pixels>,
    container: Size<Pixels>,
    menu: Size<Pixels>,
) -> Point<Pixels> {
    point(
        anchor
            .x
            .clamp(px(0.), (container.width - menu.width).max(px(0.))),
        anchor
            .y
            .clamp(px(0.), (container.height - menu.height).max(px(0.))),
    )
}

/// Shared popover-menu chrome positioned at a pre-clamped `origin`.
///
/// Returns the styled surface so the caller attaches a scroll handle,
/// `on_mouse_down_out` dismissal, and children, then wraps the result in
/// `deferred(...).with_priority(...)`.
pub fn menu_surface(
    id: impl Into<ElementId>,
    origin: Point<Pixels>,
    width: Pixels,
    max_height: Pixels,
    radius: Pixels,
    cx: &App,
) -> Stateful<Div> {
    menu_panel(id, width, cx)
        .absolute()
        .left(origin.x)
        .top(origin.y)
        .block_mouse_except_scroll()
        .max_h(max_height)
        .overflow_y_scroll()
        .rounded(radius)
}

/// Menu-row baseline.
///
/// Menus keep the accent-fill focus treatment (no focus ring) so keyboard
/// traversal reads as the row highlight. First-item `track_focus` stays at the
/// call site, which owns menu focus continuity. The caller adds one
/// [`ControlExt::on_activate`] handler and children.
///
/// [`ControlExt::on_activate`]: crate::atoms::ControlExt::on_activate
pub fn context_menu_item(
    id: impl Into<ElementId>,
    height: Pixels,
    enabled: bool,
    cx: &App,
) -> Stateful<Div> {
    menu_item_base(id, height)
        .when(enabled, |row| {
            row.hover(|style| {
                style
                    .bg(SemanticColor::BackgroundBrand.resolve(cx))
                    .text_color(SemanticColor::TextOnBrand.resolve(cx))
            })
            .focus(|style| {
                style
                    .bg(SemanticColor::BackgroundBrand.resolve(cx))
                    .text_color(SemanticColor::TextOnBrand.resolve(cx))
            })
        })
        .when(!enabled, |row| {
            row.tab_index(-1)
                .cursor_default()
                .text_color(SemanticColor::TextDisabled.resolve(cx))
        })
}

pub fn menu_item(id: impl Into<ElementId>, height: Pixels, cx: &App) -> Stateful<Div> {
    menu_item_base(id, height)
        .hover(|style| style.bg(SemanticColor::BackgroundHover.resolve(cx)))
        .focus(|style| style.bg(SemanticColor::BackgroundHover.resolve(cx)))
}

fn menu_item_base(id: impl Into<ElementId>, height: Pixels) -> Stateful<Div> {
    h_flex()
        .id(id)
        .key_context(CONTROL_KEY_CONTEXT)
        .tab_index(0)
        .h(height)
        .flex_none()
        .mx_2()
        .px_2()
        .gap_2()
        .rounded(px(4.))
        .typography(TypographyToken::BodyMedium)
        .cursor_pointer()
}

#[cfg(test)]
mod tests {
    use gpui::{point, px, size};

    use super::{MenuCheckVariant, MenuMultiSelectVariant, MenuRowState, clamp_menu_origin};

    #[test]
    fn figma_menu_axes_are_complete() {
        assert_eq!(MenuRowState::ALL.len(), 3);
        assert_eq!(MenuCheckVariant::ALL.len(), 2);
        assert_eq!(MenuMultiSelectVariant::ALL.len(), 4);
    }

    #[test]
    fn interior_anchor_is_unchanged() {
        assert_eq!(
            clamp_menu_origin(
                point(px(50.), px(80.)),
                size(px(400.), px(600.)),
                size(px(224.), px(300.)),
            ),
            point(px(50.), px(80.))
        );
    }

    #[test]
    fn right_edge_clamps_x() {
        assert_eq!(
            clamp_menu_origin(
                point(px(350.), px(80.)),
                size(px(400.), px(600.)),
                size(px(224.), px(300.)),
            ),
            point(px(176.), px(80.))
        );
    }

    #[test]
    fn bottom_edge_clamps_y() {
        assert_eq!(
            clamp_menu_origin(
                point(px(50.), px(500.)),
                size(px(400.), px(600.)),
                size(px(224.), px(300.)),
            ),
            point(px(50.), px(300.))
        );
    }

    #[test]
    fn corner_clamps_both_axes() {
        assert_eq!(
            clamp_menu_origin(
                point(px(390.), px(590.)),
                size(px(400.), px(600.)),
                size(px(224.), px(300.)),
            ),
            point(px(176.), px(300.))
        );
    }

    #[test]
    fn oversized_menu_pins_to_container_origin() {
        assert_eq!(
            clamp_menu_origin(
                point(px(120.), px(40.)),
                size(px(400.), px(600.)),
                size(px(500.), px(700.)),
            ),
            point(px(0.), px(0.))
        );
    }

    #[test]
    fn negative_anchor_floors_at_zero() {
        assert_eq!(
            clamp_menu_origin(
                point(px(-20.), px(-10.)),
                size(px(400.), px(600.)),
                size(px(224.), px(300.)),
            ),
            point(px(0.), px(0.))
        );
    }
}
