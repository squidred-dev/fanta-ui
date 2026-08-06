//! The reusable runtime-knob framework mounted under Gallery stories.
//!
//! The chip styling generalizes the Design story's bespoke scenario and
//! preset rows so every screen can offer named states and option toggles
//! without re-implementing selection chrome. Restart-required environment
//! variables stay supported by seeding a knob's initial value from its
//! env var, so the screenshot-CI launch interface keeps working.

use crate::*;

/// One selectable value inside an enum or numeric knob row.
pub(crate) struct KnobOption<T> {
    pub(crate) value: T,
    pub(crate) label: SharedString,
}

impl<T> KnobOption<T> {
    pub(crate) fn new(value: T, label: impl Into<SharedString>) -> Self {
        Self {
            value,
            label: label.into(),
        }
    }
}

/// The container each screen mounts its knob rows into.
pub(crate) fn knobs_panel(
    id: &'static str,
    rows: Vec<AnyElement>,
    cx: &mut Context<Storybook>,
) -> AnyElement {
    v_flex()
        .id(SharedString::from(id))
        .debug_selector(move || id.to_owned())
        .w_full()
        .mt_3()
        .p_3()
        .gap_3()
        .rounded(px(8.))
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().sidebar)
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("KNOBS"),
        )
        .children(rows)
        .into_any_element()
}

fn knob_heading(label: &'static str, cx: &mut Context<Storybook>) -> AnyElement {
    div()
        .text_xs()
        .text_color(cx.theme().muted_foreground)
        .child(label)
        .into_any_element()
}

/// The shared selection chip used by knob rows and the viewport harness.
pub(crate) fn knob_chip(
    id: SharedString,
    label: SharedString,
    active: bool,
    on_select: impl Fn(&mut Storybook, &mut Window, &mut Context<Storybook>) + 'static,
    cx: &mut Context<Storybook>,
) -> Button {
    Button::new(id)
        .label(label)
        .custom(
            ButtonCustomVariant::new(cx)
                .color(cx.theme().secondary)
                .foreground(cx.theme().foreground)
                .border(if active {
                    cx.theme().selection
                } else {
                    cx.theme().border
                })
                .hover(cx.theme().accent)
                .active(cx.theme().selection.opacity(0.22)),
        )
        .xsmall()
        .selected(active)
        .h(px(30.))
        .px_2()
        .rounded(px(5.))
        .border_1()
        .when(active, |button| {
            button.hover(|style| style.bg(cx.theme().accent))
        })
        .cursor_pointer()
        .on_click(cx.listener(move |this, _, window, cx| {
            on_select(this, window, cx);
        }))
}

/// A wrapping chip row selecting one value of an enum-like knob.
pub(crate) fn enum_knob_row<T>(
    id_prefix: &'static str,
    heading: &'static str,
    options: impl IntoIterator<Item = KnobOption<T>>,
    current: T,
    on_select: impl Fn(&mut Storybook, T, &mut Window, &mut Context<Storybook>) + Copy + 'static,
    cx: &mut Context<Storybook>,
) -> AnyElement
where
    T: Copy + PartialEq + 'static,
{
    let mut chips = h_flex().w_full().gap_2().flex_wrap();
    for (index, option) in options.into_iter().enumerate() {
        let value = option.value;
        chips = chips.child(knob_chip(
            SharedString::from(format!("{id_prefix}-{index}")),
            option.label,
            value == current,
            move |this, window, cx| on_select(this, value, window, cx),
            cx,
        ));
    }
    v_flex()
        .w_full()
        .gap_2()
        .child(knob_heading(heading, cx))
        .child(chips)
        .into_any_element()
}

/// A compact preset row for numeric knobs; presets share the row width.
pub(crate) fn numeric_knob_row<T>(
    id_prefix: &'static str,
    heading: &'static str,
    presets: impl IntoIterator<Item = KnobOption<T>>,
    current: T,
    on_select: impl Fn(&mut Storybook, T, &mut Window, &mut Context<Storybook>) + Copy + 'static,
    cx: &mut Context<Storybook>,
) -> AnyElement
where
    T: Copy + PartialEq + 'static,
{
    let mut buttons = h_flex().w_full().gap_2();
    for (index, preset) in presets.into_iter().enumerate() {
        let value = preset.value;
        buttons = buttons.child(
            knob_chip(
                SharedString::from(format!("{id_prefix}-{index}")),
                preset.label,
                value == current,
                move |this, window, cx| on_select(this, value, window, cx),
                cx,
            )
            .flex_1(),
        );
    }
    v_flex()
        .w_full()
        .gap_2()
        .child(knob_heading(heading, cx))
        .child(buttons)
        .into_any_element()
}

/// A full-width On/Off toggle knob.
pub(crate) fn bool_knob_row(
    id: &'static str,
    heading: &'static str,
    label: &'static str,
    value: bool,
    on_toggle: impl Fn(&mut Storybook, bool, &mut Window, &mut Context<Storybook>) + 'static,
    cx: &mut Context<Storybook>,
) -> AnyElement {
    v_flex()
        .w_full()
        .gap_2()
        .child(knob_heading(heading, cx))
        .child(
            Button::new(SharedString::from(id))
                .label(format!("{label} · {}", if value { "On" } else { "Off" }))
                .xsmall()
                .compact()
                .w_full()
                .selected(value)
                .on_click(cx.listener(move |this, _, window, cx| {
                    on_toggle(this, !value, window, cx);
                })),
        )
        .into_any_element()
}
