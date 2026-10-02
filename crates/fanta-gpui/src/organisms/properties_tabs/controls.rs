//! Shared presentation controls; accepted values always come from each tab host.
use super::InspectorChoice;
use crate::{
    atoms::{
        ControlExt as _, Dropdown, LucideIcon, SemanticColor as Color, TypographyExt as _,
        TypographyToken, render_lucide_icon,
        tokens::{self, Space},
        track_bounds, truncating_label,
    },
    molecules::{
        InspectorFieldAccess, InspectorMetrics, inspector_action_button, inspector_field_grid,
        inspector_row, inspector_section, popup_height, popup_surface, popup_width,
        sidebar_menu_item,
    },
};
use gpui::AppContext as _;
use gpui::{
    App, Bounds, Context, Div, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, ParentElement as _, Pixels, Render, SharedString,
    Stateful, StatefulInteractiveElement as _, Styled as _, Subscription, Window, div, point,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{
    Sizable as _, h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};

pub(super) fn section(title: &'static str, cx: &App) -> Div {
    inspector_section(cx)
        .border_color(Color::BorderPanel.resolve(cx))
        .child(
            h_flex()
                .h(px(tokens::InspectorGeometry::SECTION_HEADER))
                .px(px(tokens::InspectorGeometry::BODY_INSET))
                .items_center()
                .typography(TypographyToken::PanelStrong)
                .child(title),
        )
}
pub(super) fn body() -> Div {
    inspector_field_grid(InspectorMetrics::default())
        .pb(px(tokens::InspectorGeometry::BODY_BOTTOM))
        .gap(px(tokens::InspectorGeometry::ROW_GAP))
}
pub(super) fn row(label: &'static str, control: impl IntoElement, cx: &App) -> Div {
    inspector_row(InspectorMetrics::default())
        .child(
            div()
                .w(px(tokens::InputGeometry::VARIABLE_CELL_WIDTH))
                .flex_none()
                .typography(TypographyToken::Panel)
                .text_color(crate::atoms::sidebar_style(cx).muted_text)
                .child(label),
        )
        .child(div().flex_1().min_w_0().child(control))
}
pub(super) fn action(
    id: SharedString,
    label: impl Into<SharedString>,
    icon: LucideIcon,
    enabled: bool,
    cx: &App,
) -> Stateful<Div> {
    let access = if enabled {
        InspectorFieldAccess::Editable
    } else {
        InspectorFieldAccess::Disabled { reason: None }
    };
    inspector_action_button(id.clone(), &access, InspectorMetrics::default(), cx)
        .debug_selector(move || id.to_string())
        .gap(px(Space::SM))
        .typography(TypographyToken::Panel)
        .text_color(crate::atoms::sidebar_style(cx).text)
        .child(render_lucide_icon(
            icon,
            if enabled {
                Color::Icon
            } else {
                Color::IconDisabled
            }
            .resolve(cx),
            tokens::IconSize::SM,
        ))
        .child(label.into())
}
pub(super) fn empty(
    icon: LucideIcon,
    title: &'static str,
    description: &'static str,
    cx: &App,
) -> Div {
    v_flex()
        .w_full()
        .p(px(Space::LG))
        .gap(px(Space::SM))
        .child(render_lucide_icon(
            icon,
            Color::IconSecondary.resolve(cx),
            tokens::ControlSize::TOOL,
        ))
        .child(div().typography(TypographyToken::PanelStrong).child(title))
        .child(
            div()
                .typography(TypographyToken::Panel)
                .text_color(crate::atoms::sidebar_style(cx).muted_text)
                .child(description),
        )
}
pub(super) fn selection(name: SharedString, icon: LucideIcon, cx: &App) -> Div {
    h_flex()
        .px(px(Space::LG))
        .h(px(tokens::RowHeight::PAGE))
        .flex_none()
        .gap(px(Space::SM))
        .items_center()
        .border_b_1()
        .border_color(Color::BorderPanel.resolve(cx))
        .child(render_lucide_icon(
            icon,
            Color::IconSecondary.resolve(cx),
            tokens::IconSize::SM,
        ))
        .child(truncating_label(name).typography(TypographyToken::PanelStrong))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Picked(pub SharedString);
pub(super) struct Picker {
    id: SharedString,
    value: SharedString,
    choices: Vec<InspectorChoice>,
    disabled: bool,
    open: bool,
    cursor: usize,
    bounds: Bounds<Pixels>,
    focus: FocusHandle,
    menu_focus: FocusHandle,
}
impl EventEmitter<Picked> for Picker {}
impl Picker {
    pub fn new(id: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        Self {
            id: id.into(),
            value: SharedString::default(),
            choices: Vec::new(),
            disabled: false,
            open: false,
            cursor: 0,
            bounds: Bounds::default(),
            focus: cx.focus_handle(),
            menu_focus: cx.focus_handle(),
        }
    }
    pub fn sync(
        &mut self,
        value: SharedString,
        choices: Vec<InspectorChoice>,
        disabled: bool,
        cx: &mut Context<Self>,
    ) {
        if self.value != value || self.choices != choices || self.disabled != disabled {
            self.value = value;
            self.choices = choices;
            self.disabled = disabled;
            self.cursor = self.cursor.min(self.choices.len().saturating_sub(1));
            if disabled {
                self.open = false;
            }
            cx.notify();
        }
    }
}
impl Render for Picker {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let label = self
            .choices
            .iter()
            .find(|v| v.id == self.value)
            .map(|v| v.label.clone())
            .unwrap_or_else(|| {
                if self.value.is_empty() {
                    "Choose…".into()
                } else {
                    self.value.clone()
                }
            });
        let mut root = div()
            .id(self.id.clone())
            .debug_selector({
                let id = self.id.clone();
                move || id.to_string()
            })
            .relative()
            .w_full()
            .track_focus(&self.focus)
            .child(
                Dropdown::new(SharedString::from(format!("{}-trigger", self.id)), label)
                    .full_width(true)
                    .disabled(self.disabled)
                    .sidebar_style(true)
                    .inset(true)
                    .stroke(false)
                    .on_activate(cx.listener(|this, _, window, cx| {
                        this.open = !this.open;
                        if this.open {
                            this.cursor = this
                                .choices
                                .iter()
                                .position(|choice| choice.id == this.value)
                                .unwrap_or_default();
                            this.menu_focus.focus(window, cx);
                        } else {
                            this.focus.focus(window, cx);
                        }
                        cx.notify();
                    })),
            )
            .child(track_bounds(cx.entity(), |this, bounds| {
                this.bounds = bounds
            }))
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" {
                    this.open = false;
                    this.focus.focus(window, cx);
                    cx.stop_propagation();
                    cx.notify();
                }
            }));
        if self.open {
            let mut menu = popup_surface(
                SharedString::from(format!("{}-menu", self.id)),
                px(tokens::Radius::MENU),
                cx,
            )
            .debug_selector({
                let id = format!("{}-menu", self.id);
                move || id.clone()
            })
            .w(popup_width(
                window,
                self.bounds
                    .size
                    .width
                    .as_f32()
                    .max(tokens::InputGeometry::TEXT_WIDTH),
            ))
            .max_h(popup_height(window, tokens::MenuWidth::PICKER))
            .py(px(Space::XS))
            .bg(crate::atoms::sidebar_style(cx).menu_background)
            .text_color(crate::atoms::sidebar_style(cx).text)
            .typography(TypographyToken::Panel)
            .track_focus(&self.menu_focus)
            .occlude()
            .on_pinch(|_, _, cx| cx.stop_propagation())
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                this.open = false;
                cx.notify();
            }))
            .on_key_down(cx.listener(
                |this, event: &gpui::KeyDownEvent, window, cx| {
                    if event.keystroke.key == "escape" {
                        this.open = false;
                        this.focus.focus(window, cx);
                    } else if !this.choices.is_empty() {
                        match event.keystroke.key.as_str() {
                            "up" => {
                                this.cursor =
                                    (this.cursor + this.choices.len() - 1) % this.choices.len()
                            }
                            "down" => this.cursor = (this.cursor + 1) % this.choices.len(),
                            "home" => this.cursor = 0,
                            "end" => this.cursor = this.choices.len() - 1,
                            "enter" | "space" => {
                                if !this.disabled {
                                    let value = this.choices[this.cursor].id.clone();
                                    this.open = false;
                                    this.focus.focus(window, cx);
                                    cx.emit(Picked(value));
                                }
                            }
                            _ => return,
                        }
                    } else {
                        return;
                    }
                    window.prevent_default();
                    cx.stop_propagation();
                    cx.notify();
                },
            ));
            for (index, choice) in self.choices.iter().enumerate() {
                let selected = choice.id == self.value;
                let value = choice.id.clone();
                menu = menu.child(
                    sidebar_menu_item(
                        SharedString::from(format!("{}-{}", self.id, choice.id)),
                        px(tokens::RowHeight::MENU),
                        true,
                        cx,
                    )
                    .typography(TypographyToken::Panel)
                    .when(index == self.cursor, |row| {
                        row.bg(Color::BackgroundPanelHover.resolve(cx))
                    })
                    .on_hover(cx.listener(move |this, hovered, _, cx| {
                        if *hovered {
                            this.cursor = index;
                            cx.notify();
                        }
                    }))
                    .debug_selector({
                        let id = format!("{}-{}", self.id, choice.id);
                        move || id.clone()
                    })
                    .child(div().w(px(tokens::IconSize::SM)).when(selected, |v| {
                        v.child(render_lucide_icon(
                            LucideIcon::Check,
                            Color::Icon.resolve(cx),
                            tokens::IconSize::SM,
                        ))
                    }))
                    .child(truncating_label(choice.label.clone()))
                    .on_activate(cx.listener(move |this, _, window, cx| {
                        this.open = false;
                        this.focus.focus(window, cx);
                        if !this.disabled && this.choices.iter().any(|choice| choice.id == value) {
                            cx.emit(Picked(value.clone()));
                        }
                        cx.notify();
                    })),
                );
            }
            root = root.child(
                gpui::deferred(
                    gpui::anchored()
                        .position(point(
                            self.bounds.left(),
                            self.bounds.bottom() + px(Space::XS),
                        ))
                        .snap_to_window_with_margin(px(crate::molecules::POPUP_SAFE_MARGIN))
                        .child(menu),
                )
                .with_priority(30),
            );
        }
        root
    }
}

#[derive(Clone, Copy)]
pub(super) enum EntryKind {
    Number { min: f64, max: f64 },
    Color,
    Text,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Entered(pub SharedString);
pub(super) struct Entry {
    id: SharedString,
    value: SharedString,
    kind: EntryKind,
    disabled: bool,
    input: Option<Entity<InputState>>,
    subscriptions: Vec<Subscription>,
    invalid: bool,
    unit: Option<&'static str>,
}
impl EventEmitter<Entered> for Entry {}
impl Entry {
    pub fn new(id: impl Into<SharedString>, kind: EntryKind) -> Self {
        Self {
            id: id.into(),
            value: SharedString::default(),
            kind,
            disabled: false,
            input: None,
            subscriptions: Vec::new(),
            invalid: false,
            unit: None,
        }
    }
    pub fn sync(&mut self, value: SharedString, disabled: bool, cx: &mut Context<Self>) {
        if self.value != value || self.disabled != disabled {
            self.value = value;
            self.disabled = disabled;
            cx.notify();
        }
    }
    pub fn set_unit(&mut self, unit: &'static str) {
        self.unit = Some(unit);
    }
    fn validated(&self, value: &str) -> Option<SharedString> {
        match self.kind {
            EntryKind::Number { min, max } => value
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite() && *v >= min && *v <= max)
                .map(|v| v.to_string().into()),
            EntryKind::Color => crate::color::parse_hex_rgba(value.trim())
                .map(|_| value.trim().trim_start_matches('#').to_uppercase().into()),
            EntryKind::Text => (!value.trim().is_empty()).then(|| value.trim().into()),
        }
    }
    fn commit(&mut self, value: &str, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        if let Some(value) = self.validated(value) {
            self.invalid = false;
            if value != self.value {
                cx.emit(Entered(value));
            }
        } else {
            self.invalid = true;
        }
        cx.notify();
    }
}
impl Render for Entry {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.input.is_none() {
            let input = cx.new(|cx| InputState::new(window, cx));
            self.subscriptions.push(cx.subscribe_in(
                &input,
                window,
                |this, input, event: &InputEvent, _, cx| {
                    if matches!(event, InputEvent::PressEnter { .. } | InputEvent::Blur)
                        && !this.disabled
                    {
                        this.commit(&input.read(cx).value(), cx);
                    }
                },
            ));
            self.input = Some(input);
        }
        let color_swatch = matches!(self.kind, EntryKind::Color).then(|| {
            let color = crate::color::parse_hex_rgba(&self.value)
                .map(|hex| gpui::Hsla::from(gpui::rgba(hex)))
                .unwrap_or(Color::BackgroundPanelField.resolve(cx));
            div()
                .size(px(tokens::ControlSize::INLINE))
                .flex_none()
                .rounded(px(tokens::Radius::CONTROL))
                .border_1()
                .border_color(Color::BorderPanel.resolve(cx))
                .bg(color)
        });
        let input = self.input.as_ref().unwrap();
        if !input.focus_handle(cx).is_focused(window)
            && input.read(cx).value() != self.value
            && !self.invalid
        {
            input.update(cx, |input, cx| {
                input.set_value(self.value.clone(), window, cx)
            });
        }
        div()
            .id(self.id.clone())
            .debug_selector({
                let id = self.id.clone();
                move || id.to_string()
            })
            .w_full()
            .min_w_0()
            .child(
                Input::new(input)
                    .small()
                    // Own the field surface so Input's unit wrapper stays transparent.
                    .appearance(false)
                    .typography(TypographyToken::Panel)
                    .bg(Color::BackgroundPanelField.resolve(cx))
                    .rounded(px(tokens::Radius::CONTROL))
                    .border_1()
                    .border_color(if input.focus_handle(cx).is_focused(window) {
                        crate::atoms::sidebar_style(cx).focused_border
                    } else {
                        Color::BorderPanel.resolve(cx).opacity(0.)
                    })
                    .when_some(color_swatch, |input, swatch| input.prefix(swatch))
                    .when_some(self.unit, |input, unit| {
                        input.suffix(
                            div()
                                .typography(TypographyToken::PanelCaption)
                                .text_color(crate::atoms::sidebar_style(cx).muted_text)
                                .child(unit),
                        )
                    })
                    .disabled(self.disabled),
            )
            .when(self.invalid, |v| {
                v.child(
                    div()
                        .typography(TypographyToken::PanelCaption)
                        .text_color(Color::TextDanger.resolve(cx))
                        .child("Enter a valid value"),
                )
            })
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                if event.keystroke.key == "enter" {
                    if let Some(input) = &this.input {
                        let value = input.read(cx).value();
                        this.commit(&value, cx);
                    }
                    window.prevent_default();
                    cx.stop_propagation();
                } else if event.keystroke.key == "escape" {
                    this.invalid = false;
                    if let Some(input) = &this.input {
                        input.update(cx, |input, cx| {
                            input.set_value(this.value.clone(), window, cx)
                        });
                    }
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
    }
}

pub(super) fn picker<C: 'static>(
    id: &str,
    cx: &mut Context<C>,
    on_change: impl Fn(&mut C, &Picked, &mut Context<C>) + 'static,
) -> Entity<Picker> {
    let field = cx.new(|cx| Picker::new(SharedString::from(id.to_owned()), cx));
    cx.subscribe(&field, move |this, _, event, cx| on_change(this, event, cx))
        .detach();
    field
}
pub(super) fn entry<C: 'static>(
    id: &str,
    kind: EntryKind,
    cx: &mut Context<C>,
    on_change: impl Fn(&mut C, &Entered, &mut Context<C>) + 'static,
) -> Entity<Entry> {
    let field = cx.new(|_| Entry::new(SharedString::from(id.to_owned()), kind));
    cx.subscribe(&field, move |this, _, event, cx| on_change(this, event, cx))
        .detach();
    field
}
pub(super) fn sync_picker(
    field: &Entity<Picker>,
    value: impl Into<SharedString>,
    choices: Vec<InspectorChoice>,
    disabled: bool,
    cx: &mut App,
) {
    field.update(cx, |field, cx| {
        field.sync(value.into(), choices, disabled, cx)
    });
}
pub(super) fn sync_entry(
    field: &Entity<Entry>,
    value: impl Into<SharedString>,
    disabled: bool,
    cx: &mut App,
) {
    field.update(cx, |field, cx| field.sync(value.into(), disabled, cx));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::mount_component;
    use gpui::{Modifiers, TestAppContext, size};

    #[test]
    fn numeric_entries_reject_non_finite_and_out_of_range_drafts() {
        let entry = Entry::new("number", EntryKind::Number { min: 0., max: 100. });
        for invalid in ["NaN", "inf", "-1", "101", "not a number"] {
            assert!(entry.validated(invalid).is_none());
        }
        assert_eq!(entry.validated(" 42.5 "), Some("42.5".into()));
        let color = Entry::new("color", EntryKind::Color);
        assert_eq!(color.validated("#abc"), Some("ABC".into()));
        assert!(color.validated("not a color").is_none());
    }

    #[gpui::test]
    fn choice_menu_clamps_to_window_and_returns_opaque_id(cx: &mut TestAppContext) {
        let (host, actions, cx) = mount_component(cx, |_, cx| {
            let mut picker = Picker::new("choice", cx);
            picker.sync(
                "a".into(),
                vec![
                    InspectorChoice::new("a", "First"),
                    InspectorChoice::new("opaque-b", "A long second choice label"),
                ],
                false,
                cx,
            );
            picker
        });
        cx.simulate_resize(size(px(320.), px(400.)));
        cx.run_until_parked();
        let bounds = cx.debug_bounds("choice").unwrap();
        cx.simulate_click(bounds.center(), Modifiers::none());
        cx.run_until_parked();
        let choice = cx
            .debug_bounds("choice-opaque-b")
            .expect("menu option should render");
        assert!(choice.left() >= px(0.) && choice.right() <= px(320.));
        assert!(choice.top() >= px(0.) && choice.bottom() <= px(400.));
        cx.simulate_click(choice.center(), Modifiers::none());
        cx.run_until_parked();
        assert_eq!(&*actions.borrow(), &[Picked("opaque-b".into())]);
        cx.read(|app| assert_eq!(host.read(app).component.read(app).value, "a"));
    }
}
