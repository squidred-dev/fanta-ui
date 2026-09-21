//! UI3 Checkbox atom story.

use crate::*;
use gpui::FocusHandle;

use super::specimen::{
    specimen_card, specimen_control_cell, specimen_row, specimen_rows, specimen_story_root,
};

pub(crate) struct CheckboxStory {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) value: CheckboxType,
    pub(crate) last_action: SharedString,
}

impl CheckboxStory {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            value: CheckboxType::Unchecked,
            last_action: "Activate the live checkbox to cycle its controlled value".into(),
        }
    }
}

impl Storybook {
    pub(crate) fn render_checkbox_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let live_value = self.checkbox_story.value;
        let live = Checkbox::new("checkbox-live", "Live controlled checkbox")
            .value(live_value)
            .on_activate(cx.listener(|this, event: &ActivateEvent, _, cx| {
                this.checkbox_story.value = match this.checkbox_story.value {
                    CheckboxType::Unchecked => CheckboxType::Checked,
                    CheckboxType::Checked => CheckboxType::Mixed,
                    CheckboxType::Mixed => CheckboxType::Unchecked,
                };
                this.checkbox_story.last_action = format!(
                    "Checkbox is now {} via {}",
                    this.checkbox_story.value.label(),
                    if event.keyboard {
                        "keyboard"
                    } else {
                        "pointer"
                    }
                )
                .into();
                cx.notify();
            }))
            .into_any_element();

        let mut value_rows = Vec::new();
        for value in CheckboxType::ALL {
            let slug = value.label().to_ascii_lowercase();
            let cells = vec![
                specimen_control_cell(
                    "Default",
                    Checkbox::new(
                        SharedString::from(format!("checkbox-{slug}-default")),
                        value.label(),
                    )
                    .value(value)
                    .into_any_element(),
                    cx,
                ),
                specimen_control_cell(
                    "Focused",
                    Checkbox::new(
                        SharedString::from(format!("checkbox-{slug}-focused")),
                        value.label(),
                    )
                    .value(value)
                    .preview_state(CheckboxState::Focused)
                    .into_any_element(),
                    cx,
                ),
                specimen_control_cell(
                    "Disabled",
                    Checkbox::new(
                        SharedString::from(format!("checkbox-{slug}-disabled")),
                        value.label(),
                    )
                    .value(value)
                    .disabled(true)
                    .into_any_element(),
                    cx,
                ),
            ];
            value_rows.push(specimen_row(
                SharedString::from(format!("checkbox-row-{slug}")),
                value.label(),
                cells,
                cx,
            ));
        }

        let modifiers = specimen_row(
            "checkbox-modifiers",
            "Muted and ghost properties",
            vec![
                specimen_control_cell(
                    "Muted",
                    Checkbox::new("checkbox-muted", "Muted")
                        .value(CheckboxType::Checked)
                        .muted(true)
                        .into_any_element(),
                    cx,
                ),
                specimen_control_cell(
                    "Ghost",
                    Checkbox::new("checkbox-ghost", "Ghost")
                        .value(CheckboxType::Checked)
                        .ghost(true)
                        .into_any_element(),
                    cx,
                ),
                specimen_control_cell(
                    "Muted + ghost",
                    Checkbox::new("checkbox-muted-ghost", "Muted ghost")
                        .value(CheckboxType::Mixed)
                        .muted(true)
                        .ghost(true)
                        .into_any_element(),
                    cx,
                ),
            ],
            cx,
        );

        specimen_story_root("storybook-checkbox")
            .track_focus(&self.checkbox_story.focus_handle)
            .child(specimen_card(
                "checkbox-live-card",
                "Checkbox · controlled interaction",
                "The host owns the checked, unchecked, or mixed value. Pointer and Enter/Space activation emit through one handler.",
                specimen_rows(vec![specimen_row(
                    "checkbox-live-row",
                    format!("Current value · {}", live_value.label()),
                    vec![specimen_control_cell("Click or press Enter / Space", live, cx)],
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "checkbox-taxonomy-card",
                "Checkbox · Figma taxonomy",
                "A 24 px row around a 16 px control. The component exposes checked, unchecked, and mixed values; focused, disabled, muted, and ghost are independent presentation axes.",
                specimen_rows(value_rows),
                cx,
            ))
            .child(specimen_card(
                "checkbox-modifiers-card",
                "Checkbox · muted and ghost",
                "Muted lowers emphasis while ghost removes the selected fill without changing the controlled value.",
                specimen_rows(vec![modifiers]),
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn render_checkbox_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-checkbox",
            self.render_checkbox_story(cx),
            self.checkbox_story.last_action.clone(),
            cx,
        )
    }
}
