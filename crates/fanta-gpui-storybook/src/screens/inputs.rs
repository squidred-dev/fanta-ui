//! UI3 input-family story derived from the Figma Inputs component sets.

use crate::*;
use gpui::FocusHandle;

use super::specimen::{
    specimen_card, specimen_control_cell, specimen_row, specimen_rows, specimen_story_root,
    specimen_wide_control_cell,
};

fn text_state(
    window: &mut Window,
    cx: &mut Context<Storybook>,
    value: &'static str,
    placeholder: &'static str,
) -> Entity<InputState> {
    cx.new(|cx| {
        InputState::new(window, cx)
            .placeholder(placeholder)
            .default_value(value)
    })
}

fn multiline_state(
    window: &mut Window,
    cx: &mut Context<Storybook>,
    value: &'static str,
) -> Entity<InputState> {
    cx.new(|cx| {
        InputState::new(window, cx)
            .multi_line(true)
            .rows(3)
            .placeholder("Write a description…")
            .default_value(value)
    })
}

fn numeric_state(
    window: &mut Window,
    cx: &mut Context<Storybook>,
    value: &'static str,
) -> Entity<InputState> {
    cx.new(|cx| {
        InputState::new(window, cx)
            .placeholder("0")
            .default_value(value)
            .validate(|value, _| {
                value == "-" || value == "." || value == "-." || value.parse::<f64>().is_ok()
            })
    })
}

fn hex_state(
    window: &mut Window,
    cx: &mut Context<Storybook>,
    value: &'static str,
) -> Entity<InputState> {
    cx.new(|cx| {
        InputState::new(window, cx)
            .placeholder("000000")
            .default_value(value)
            .validate(|value, _| value.len() <= 8 && value.chars().all(|c| c.is_ascii_hexdigit()))
    })
}

fn subscribe_to_input(
    cx: &mut Context<Storybook>,
    state: &Entity<InputState>,
    label: &'static str,
) -> Subscription {
    cx.subscribe(state, move |story, input, event: &InputEvent, cx| {
        let value = input.read(cx).value();
        let action = match event {
            InputEvent::Change => Some(format!("{label} changed to ‘{value}’")),
            InputEvent::PressEnter { secondary } => Some(format!(
                "{label} committed{} as ‘{}’",
                if *secondary {
                    " with secondary Enter"
                } else {
                    ""
                },
                value
            )),
            _ => None,
        };
        if let Some(action) = action {
            story.inputs_story.last_action = action.into();
            cx.notify();
        }
    })
}

pub(crate) struct InputsStory {
    pub(crate) live_text: Entity<InputState>,
    live_number: Entity<InputState>,
    live_color: Entity<InputState>,
    live_combo: Entity<InputState>,
    text_states: Vec<Entity<InputState>>,
    text_variants: Vec<Entity<InputState>>,
    numeric_states: Vec<Entity<InputState>>,
    numeric_properties: Vec<Entity<InputState>>,
    numeric_multi: Vec<Entity<InputState>>,
    numeric_multi_partial: Vec<Entity<InputState>>,
    color_states: Vec<Entity<InputState>>,
    combo_states: Vec<Entity<InputState>>,
    pub(crate) last_action: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl InputsStory {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Storybook>) -> Self {
        let live_text = text_state(window, cx, "Card title", "Text");
        let live_number = numeric_state(window, cx, "24");
        let live_color = hex_state(window, cx, "FF24BD");
        let live_combo = numeric_state(window, cx, "16");

        let text_states = InputVisualState::ALL
            .into_iter()
            .map(|state| {
                text_state(
                    window,
                    cx,
                    if state == InputVisualState::Empty {
                        ""
                    } else {
                        "Text"
                    },
                    "Text",
                )
            })
            .collect::<Vec<_>>();
        let text_variants = vec![
            text_state(window, cx, "Text", "Text"),
            text_state(window, cx, "Large text", "Text"),
            multiline_state(window, cx, "Multi-line text can wrap across rows."),
            text_state(window, cx, "Quick action", "Text"),
            text_state(window, cx, "can view", "Text"),
            text_state(window, cx, "token-value", "Text"),
        ];
        let numeric_states = InputVisualState::ALL
            .into_iter()
            .map(|state| {
                numeric_state(
                    window,
                    cx,
                    if state == InputVisualState::Empty {
                        ""
                    } else {
                        "24"
                    },
                )
            })
            .collect::<Vec<_>>();
        let numeric_properties = (0..5)
            .map(|_| numeric_state(window, cx, "24"))
            .collect::<Vec<_>>();
        let numeric_multi = ["24", "16", "24", "16"]
            .into_iter()
            .map(|value| numeric_state(window, cx, value))
            .collect::<Vec<_>>();
        let numeric_multi_partial = ["24", "16", "24", "16"]
            .into_iter()
            .map(|value| numeric_state(window, cx, value))
            .collect::<Vec<_>>();
        let color_states = ["FF24BD", "24", "Image", "Angular", "brand"]
            .into_iter()
            .map(|value| {
                if value == "FF24BD" {
                    hex_state(window, cx, value)
                } else {
                    text_state(window, cx, value, "Value")
                }
            })
            .collect::<Vec<_>>();
        let combo_states = ["16", "16", "16", "24"]
            .into_iter()
            .map(|value| numeric_state(window, cx, value))
            .collect::<Vec<_>>();

        let mut subscriptions = vec![
            subscribe_to_input(cx, &live_text, "Text input"),
            subscribe_to_input(cx, &live_number, "Numeric input"),
            subscribe_to_input(cx, &live_color, "Color input"),
            subscribe_to_input(cx, &live_combo, "Combo input"),
        ];
        for state in text_states
            .iter()
            .chain(text_variants.iter())
            .chain(numeric_states.iter())
            .chain(numeric_properties.iter())
            .chain(numeric_multi.iter())
            .chain(numeric_multi_partial.iter())
            .chain(color_states.iter())
            .chain(combo_states.iter())
        {
            subscriptions.push(subscribe_to_input(cx, state, "Input specimen"));
        }

        Self {
            live_text,
            live_number,
            live_color,
            live_combo,
            text_states,
            text_variants,
            numeric_states,
            numeric_properties,
            numeric_multi,
            numeric_multi_partial,
            color_states,
            combo_states,
            last_action: "Ready · edit any field to exercise its host-owned InputState".into(),
            _subscriptions: subscriptions,
        }
    }

    pub(crate) fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.live_text.focus_handle(cx)
    }
}

impl Storybook {
    fn input_state_cells(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        InputVisualState::ALL
            .into_iter()
            .zip(self.inputs_story.text_states.iter())
            .map(|(state, input)| {
                specimen_wide_control_cell(
                    state.label(),
                    FantaTextInput::new(input)
                        .preview_state(state)
                        .into_any_element(),
                    cx,
                )
            })
            .collect()
    }

    pub(crate) fn render_inputs_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let magenta = rgba(0xff24bdff).into();
        let text_variants = &self.inputs_story.text_variants;
        let numeric_properties = &self.inputs_story.numeric_properties;
        let color_states = &self.inputs_story.color_states;
        let combo_states = &self.inputs_story.combo_states;

        specimen_story_root("storybook-inputs")
            .child(specimen_card(
                "inputs-live-card",
                "Inputs · live controlled editing",
                "Each field uses a host-owned gpui-component InputState. Numeric input rejects non-numeric text and the color field accepts at most eight hexadecimal digits.",
                specimen_rows(vec![specimen_row(
                    "inputs-live-row",
                    "Edit, select, and commit with Enter",
                    vec![
                        specimen_wide_control_cell(
                            "Text",
                            FantaTextInput::new(&self.inputs_story.live_text)
                                .leading_icon(LucideIcon::Search)
                                .into_any_element(),
                            cx,
                        ),
                        specimen_control_cell(
                            "Numeric",
                            NumericInput::new(&self.inputs_story.live_number)
                                .variable_icon(true)
                                .into_any_element(),
                            cx,
                        ),
                        specimen_wide_control_cell(
                            "Color",
                            ColorInput::new(&self.inputs_story.live_color, magenta)
                                .opacity(72)
                                .into_any_element(),
                            cx,
                        ),
                        specimen_control_cell(
                            "Combo dropdown",
                            ComboInput::new("live-combo", &self.inputs_story.live_combo)
                                .variable(true)
                                .on_dropdown(cx.listener(|this, _, _, cx| {
                                    this.inputs_story.last_action = "Combo dropdown activated".into();
                                    cx.notify();
                                }))
                                .into_any_element(),
                            cx,
                        ),
                    ],
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "text-input-card",
                "Text input",
                "Single-line, multi-line, and quick-action variants share default and large sizes. Leading icon, dropdown, variable label, and chip are independent properties.",
                specimen_rows(vec![
                    specimen_row("text-input-states", "States", self.input_state_cells(cx), cx),
                    specimen_row(
                        "text-input-variants",
                        "Variants and properties",
                        vec![
                            specimen_wide_control_cell("Single line", FantaTextInput::new(&text_variants[0]).into_any_element(), cx),
                            specimen_wide_control_cell("Large", FantaTextInput::new(&text_variants[1]).size(InputSize::Large).into_any_element(), cx),
                            specimen_wide_control_cell("Multi line", FantaTextInput::new(&text_variants[2]).variant(TextInputVariant::MultiLine).into_any_element(), cx),
                            specimen_wide_control_cell("Quick action", FantaTextInput::new(&text_variants[3]).variant(TextInputVariant::QuickAction).into_any_element(), cx),
                            specimen_wide_control_cell("Leading icon + dropdown", FantaTextInput::new(&text_variants[4]).leading_icon(LucideIcon::Search).dropdown(true).into_any_element(), cx),
                            specimen_wide_control_cell("Variable label + chip", FantaTextInput::new(&text_variants[5]).label(TextInputLabel::Variable).chip(true).into_any_element(), cx),
                        ],
                        cx,
                    ),
                ]),
                cx,
            ))
            .child(specimen_card(
                "numeric-input-card",
                "Numeric input and numeric input multi",
                "The numeric family covers visual states, variable affordances, optional icons/dropdown, and grouped values with partial disable.",
                specimen_rows(vec![
                    specimen_row(
                        "numeric-input-states",
                        "States",
                        InputVisualState::ALL
                            .into_iter()
                            .zip(self.inputs_story.numeric_states.iter())
                            .map(|(state, input)| specimen_control_cell(state.label(), NumericInput::new(input).preview_state(state).into_any_element(), cx))
                            .collect(),
                        cx,
                    ),
                    specimen_row(
                        "numeric-input-properties",
                        "Properties",
                        vec![
                            specimen_control_cell("Variable icon", NumericInput::new(&numeric_properties[0]).variable_icon(true).into_any_element(), cx),
                            specimen_control_cell("Variable pill", NumericInput::new(&numeric_properties[1]).variable_pill(true).into_any_element(), cx),
                            specimen_control_cell("Dropdown", NumericInput::new(&numeric_properties[2]).dropdown(true).into_any_element(), cx),
                            specimen_control_cell("Leading icon", NumericInput::new(&numeric_properties[3]).leading_icon(LucideIcon::MoveDiagonal2).into_any_element(), cx),
                            specimen_control_cell("Trailing icon", NumericInput::new(&numeric_properties[4]).trailing_icon(LucideIcon::X).into_any_element(), cx),
                        ],
                        cx,
                    ),
                    specimen_row(
                        "numeric-input-multi",
                        "Grouped · default and partial disable",
                        vec![
                            specimen_wide_control_cell("Default", NumericInputMulti::new(self.inputs_story.numeric_multi.clone()).leading_icon(LucideIcon::Focus).into_any_element(), cx),
                            specimen_wide_control_cell("Partial disable", NumericInputMulti::new(self.inputs_story.numeric_multi_partial.clone()).leading_icon(LucideIcon::Focus).partial_disable(true).into_any_element(), cx),
                        ],
                        cx,
                    ),
                ]),
                cx,
            ))
            .child(specimen_card(
                "color-combo-input-card",
                "Color input and combo input",
                "Color type changes the chit and value treatment. Combo selection can target the editable input or its dedicated dropdown segment.",
                specimen_rows(vec![
                    specimen_row(
                        "color-input-types",
                        "Color types",
                        [ColorInputKind::Fill, ColorInputKind::Opacity, ColorInputKind::Image, ColorInputKind::Gradient, ColorInputKind::Variable]
                            .into_iter()
                            .zip(color_states.iter())
                            .map(|(kind, state)| specimen_wide_control_cell(format!("{kind:?}"), ColorInput::new(state, magenta).kind(kind).opacity(24).into_any_element(), cx))
                            .collect(),
                        cx,
                    ),
                    specimen_row(
                        "combo-input-states",
                        "Combo states",
                        [ComboInputState::Default, ComboInputState::Hover, ComboInputState::SelectedChevron, ComboInputState::SelectedInput]
                            .into_iter()
                            .zip(combo_states.iter())
                            .map(|(state, input)| specimen_control_cell(format!("{state:?}"), ComboInput::new(SharedString::from(format!("combo-{state:?}")), input).preview_state(state).into_any_element(), cx))
                            .collect(),
                        cx,
                    ),
                ]),
                cx,
            ))
            .child(specimen_card(
                "input-supporting-atoms-card",
                "Input supporting atoms",
                "Variable cells, editable chips, variable chips, combo dropdown segments, and 24/48 px chits are reusable pieces rather than one-off field decoration.",
                specimen_rows(vec![
                    specimen_row(
                        "input-supporting-cells",
                        "Variable cell, input chip, and dropdown segment",
                        vec![
                            specimen_control_cell("Variable cell", VariableCell::new("Value").into_any_element(), cx),
                            specimen_control_cell("Input chip", InputChip::new("input-chip", "24").close_button(true).into_any_element(), cx),
                            specimen_control_cell("Focused chip", InputChip::new("input-chip-focus", "24").preview_state(InputChipState::Focused).into_any_element(), cx),
                            specimen_control_cell("Dropdown · default", div().h(px(24.)).child(ComboInputDropdown::new("combo-drop-default")).into_any_element(), cx),
                            specimen_control_cell("Dropdown · hover", div().h(px(24.)).child(ComboInputDropdown::new("combo-drop-hover").preview_state(InputVisualState::Hover)).into_any_element(), cx),
                            specimen_control_cell("Dropdown · active", div().h(px(24.)).child(ComboInputDropdown::new("combo-drop-active").preview_state(InputVisualState::Active)).into_any_element(), cx),
                        ],
                        cx,
                    ),
                    specimen_row(
                        "variable-chip-states",
                        "Variable chip states",
                        [VariableChipState::Default, VariableChipState::Selected, VariableChipState::OnSelected, VariableChipState::Hover, VariableChipState::SoftDeleted, VariableChipState::DisabledSecondary, VariableChipState::DisabledTertiary, VariableChipState::ValueNotRendered]
                            .into_iter()
                            .map(|state| specimen_control_cell(format!("{state:?}"), VariableChip::new(SharedString::from(format!("variable-{state:?}")), "24").state(state).into_any_element(), cx))
                            .collect(),
                        cx,
                    ),
                    specimen_row(
                        "color-chit-24",
                        "Chit 24 · type and shape",
                        [ColorChitKind::Fill, ColorChitKind::Opacity, ColorChitKind::Gradient, ColorChitKind::Image, ColorChitKind::Instance]
                            .into_iter()
                            .map(|kind| specimen_control_cell(format!("{kind:?}"), h_flex().gap_2().child(ColorChit::new(magenta).kind(kind)).child(ColorChit::new(magenta).kind(kind).shape(ColorChitShape::Circle)).into_any_element(), cx))
                            .collect(),
                        cx,
                    ),
                    specimen_row(
                        "color-chit-48",
                        "Chit 48 · type and shape",
                        [ColorChitKind::Fill, ColorChitKind::Opacity, ColorChitKind::Gradient, ColorChitKind::Image]
                            .into_iter()
                            .map(|kind| specimen_control_cell(format!("{kind:?}"), h_flex().gap_2().child(ColorChit::new(magenta).kind(kind).size(ColorChitSize::Large)).child(ColorChit::new(magenta).kind(kind).shape(ColorChitShape::Circle).size(ColorChitSize::Large)).into_any_element(), cx))
                            .collect(),
                        cx,
                    ),
                ]),
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn render_inputs_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-inputs",
            self.render_inputs_story(cx),
            self.inputs_story.last_action.clone(),
            cx,
        )
    }
}
