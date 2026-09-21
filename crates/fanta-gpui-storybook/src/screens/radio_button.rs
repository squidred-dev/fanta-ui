//! UI3 radio-button component story.

use std::collections::HashMap;

use crate::*;
use gpui::FocusHandle;

use super::specimen::{
    specimen_card, specimen_control_cell, specimen_row, specimen_rows, specimen_story_root,
};

const FIGMA_VARIANTS: [(RadioButtonVariant, RadioButtonState, bool, bool); 12] = [
    (
        RadioButtonVariant::Input,
        RadioButtonState::Default,
        true,
        false,
    ),
    (
        RadioButtonVariant::Input,
        RadioButtonState::Default,
        false,
        false,
    ),
    (
        RadioButtonVariant::Input,
        RadioButtonState::Focused,
        true,
        false,
    ),
    (
        RadioButtonVariant::Input,
        RadioButtonState::Focused,
        false,
        false,
    ),
    (
        RadioButtonVariant::Input,
        RadioButtonState::Default,
        true,
        true,
    ),
    (
        RadioButtonVariant::Input,
        RadioButtonState::Default,
        false,
        true,
    ),
    (
        RadioButtonVariant::Input,
        RadioButtonState::Focused,
        true,
        true,
    ),
    (
        RadioButtonVariant::Input,
        RadioButtonState::Focused,
        false,
        true,
    ),
    (
        RadioButtonVariant::Button,
        RadioButtonState::Default,
        false,
        true,
    ),
    (
        RadioButtonVariant::Button,
        RadioButtonState::Active,
        false,
        true,
    ),
    (
        RadioButtonVariant::Button,
        RadioButtonState::Focused,
        false,
        true,
    ),
    (
        RadioButtonVariant::Button,
        RadioButtonState::Disabled,
        false,
        true,
    ),
];

pub(crate) struct RadioButtonStory {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) selected_live: usize,
    specimen_selected: HashMap<String, bool>,
    pub(crate) last_action: SharedString,
}

impl RadioButtonStory {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            selected_live: 0,
            specimen_selected: HashMap::new(),
            last_action: "Choose a radio option".into(),
        }
    }
}

impl Storybook {
    fn radio_specimen_cells(
        &self,
        specs: &[(RadioButtonVariant, RadioButtonState, bool, bool)],
        offset: usize,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        specs
            .iter()
            .enumerate()
            .map(
                |(local_index, (variant, state, initial_selected, label_visible))| {
                    let index = offset + local_index;
                    let key = format!("radio-specimen-{index}");
                    let selected = self
                        .radio_button_story
                        .specimen_selected
                        .get(&key)
                        .copied()
                        .unwrap_or(*initial_selected);
                    let mut radio = RadioButton::new(key.clone(), "Radio button")
                        .variant(*variant)
                        .preview_state(*state)
                        .selected(selected)
                        .label_visible(*label_visible);
                    if *state != RadioButtonState::Disabled {
                        let selection_key = key.clone();
                        let description = format!(
                            "{} · {} · {}",
                            variant.label(),
                            state.label(),
                            if *label_visible { "Label" } else { "No label" }
                        );
                        radio = radio.on_select(cx.listener(
                            move |this, selection: &RadioButtonSelection, _, cx| {
                                this.radio_button_story
                                    .specimen_selected
                                    .insert(selection_key.clone(), selection.selected);
                                this.radio_button_story.last_action = format!(
                                    "Selected {description} via {}",
                                    if selection.keyboard {
                                        "keyboard"
                                    } else {
                                        "pointer"
                                    }
                                )
                                .into();
                                cx.notify();
                            },
                        ));
                    }
                    specimen_control_cell(
                        format!(
                            "{} · {} · On={} · Label={}",
                            variant.label(),
                            state.label(),
                            if selected { "On" } else { "Off" },
                            label_visible
                        ),
                        radio.into_any_element(),
                        cx,
                    )
                },
            )
            .collect()
    }

    pub(crate) fn render_radio_button_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let selected = self.radio_button_story.selected_live;
        let live_radios = ["Design", "Prototype", "Inspect"]
            .into_iter()
            .enumerate()
            .map(|(index, label)| {
                RadioButton::new(format!("radio-live-{index}"), label)
                    .selected(index == selected)
                    .on_select(
                        cx.listener(move |this, selection: &RadioButtonSelection, _, cx| {
                            this.radio_button_story.selected_live = index;
                            this.radio_button_story.last_action = format!(
                                "Selected {label} via {}",
                                if selection.keyboard {
                                    "keyboard"
                                } else {
                                    "pointer"
                                }
                            )
                            .into();
                            cx.notify();
                        }),
                    )
                    .into_any_element()
            })
            .collect();

        specimen_story_root("storybook-radio-button")
            .track_focus(&self.radio_button_story.focus_handle)
            .child(specimen_card(
                "radio-live-card",
                "Radio group · controlled interaction",
                "The Storybook host owns group exclusivity. Click or focus an option and press Enter or Space to select it.",
                specimen_rows(vec![specimen_row(
                    "radio-live-row",
                    format!("Selected: {}", ["Design", "Prototype", "Inspect"][selected]),
                    live_radios,
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "radio-figma-card",
                "Radio button · all 12 Figma variants",
                "The matrix contains the eight valid Input combinations and the four Button states declared in Figma. Every enabled specimen emits a typed selection intent.",
                specimen_rows(vec![
                    specimen_row(
                        "radio-input-no-label-row",
                        "Input · Label=False",
                        self.radio_specimen_cells(&FIGMA_VARIANTS[0..4], 0, cx),
                        cx,
                    ),
                    specimen_row(
                        "radio-input-label-row",
                        "Input · Label=True",
                        self.radio_specimen_cells(&FIGMA_VARIANTS[4..8], 4, cx),
                        cx,
                    ),
                    specimen_row(
                        "radio-button-row",
                        "Button · On=Off · Label=True",
                        self.radio_specimen_cells(&FIGMA_VARIANTS[8..12], 8, cx),
                        cx,
                    ),
                ]),
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn render_radio_button_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-radio-button",
            self.render_radio_button_story(cx),
            self.radio_button_story.last_action.clone(),
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn story_lists_the_twelve_valid_figma_variants() {
        assert_eq!(FIGMA_VARIANTS.len(), 12);
        assert_eq!(FIGMA_VARIANTS.iter().filter(|spec| spec.3).count(), 8);
        assert_eq!(
            FIGMA_VARIANTS
                .iter()
                .filter(|spec| spec.0 == RadioButtonVariant::Button)
                .count(),
            4
        );
    }
}
