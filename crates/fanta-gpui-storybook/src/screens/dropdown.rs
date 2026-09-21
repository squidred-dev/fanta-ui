//! UI3 Dropdown trigger atom story.

use crate::*;
use gpui::FocusHandle;

use super::specimen::{
    specimen_card, specimen_control_cell, specimen_row, specimen_rows, specimen_story_root,
};

const DROPDOWN_VALUES: [&str; 3] = ["Value", "Option one", "Option two"];

pub(crate) struct DropdownStory {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) selected: usize,
    pub(crate) last_action: SharedString,
}

impl DropdownStory {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            selected: 0,
            last_action: "Activate the live dropdown to select the next mock option".into(),
        }
    }
}

impl Storybook {
    fn dropdown_state_cells(
        &self,
        size: DropdownSize,
        leading_icon: bool,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let size_slug = match size {
            DropdownSize::Default => "default",
            DropdownSize::Large => "large",
        };
        let icon_slug = if leading_icon { "icon" } else { "plain" };
        let mut cells = DropdownState::ALL
            .into_iter()
            .map(|state| {
                let mut dropdown = Dropdown::new(
                    SharedString::from(format!("dropdown-{size_slug}-{icon_slug}-{state:?}")),
                    "Value",
                )
                .size(size)
                .preview_state(state);
                if leading_icon {
                    dropdown = dropdown.leading_icon(LucideIcon::Square);
                }
                specimen_control_cell(state.label(), dropdown.into_any_element(), cx)
            })
            .collect::<Vec<_>>();
        let mut disabled = Dropdown::new(
            SharedString::from(format!("dropdown-{size_slug}-{icon_slug}-disabled")),
            "Value",
        )
        .size(size)
        .disabled(true);
        if leading_icon {
            disabled = disabled.leading_icon(LucideIcon::Square);
        }
        cells.push(specimen_control_cell(
            "Disabled",
            disabled.into_any_element(),
            cx,
        ));
        cells
    }

    pub(crate) fn render_dropdown_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let selected = self.dropdown_story.selected;
        let live = Dropdown::new("dropdown-live", DROPDOWN_VALUES[selected])
            .leading_icon(LucideIcon::Square)
            .on_activate(cx.listener(|this, event: &ActivateEvent, _, cx| {
                this.dropdown_story.selected =
                    (this.dropdown_story.selected + 1) % DROPDOWN_VALUES.len();
                this.dropdown_story.last_action = format!(
                    "Selected {} via {}",
                    DROPDOWN_VALUES[this.dropdown_story.selected],
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

        specimen_story_root("storybook-dropdown")
            .track_focus(&self.dropdown_story.focus_handle)
            .child(specimen_card(
                "dropdown-live-card",
                "Dropdown · controlled interaction",
                "This component is the compact trigger. The host owns the selected value and the popup; activation uses the shared pointer and keyboard path.",
                specimen_rows(vec![specimen_row(
                    "dropdown-live-row",
                    format!("Selected · {}", DROPDOWN_VALUES[selected]),
                    vec![specimen_control_cell("Click or press Enter / Space", live, cx)],
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "dropdown-states-card",
                "Dropdown · states and sizes",
                "The Figma set uses a fixed 117 px width, 24 and 32 px heights, and default, focused, active, and disabled states.",
                specimen_rows(vec![
                    specimen_row(
                        "dropdown-default-row",
                        "Default size · 117 × 24 px",
                        self.dropdown_state_cells(DropdownSize::Default, false, cx),
                        cx,
                    ),
                    specimen_row(
                        "dropdown-large-row",
                        "Large size · 117 × 32 px",
                        self.dropdown_state_cells(DropdownSize::Large, false, cx),
                        cx,
                    ),
                ]),
                cx,
            ))
            .child(specimen_card(
                "dropdown-properties-card",
                "Dropdown · stroke and leading icon",
                "Stroke and leading-icon visibility are independent component properties at both sizes.",
                specimen_rows(vec![
                    specimen_row(
                        "dropdown-icon-row",
                        "Leading icon",
                        self.dropdown_state_cells(DropdownSize::Default, true, cx),
                        cx,
                    ),
                    specimen_row(
                        "dropdown-stroke-row",
                        "Stroke",
                        vec![
                            specimen_control_cell(
                                "Stroke on",
                                Dropdown::new("dropdown-stroke-on", "Value")
                                    .into_any_element(),
                                cx,
                            ),
                            specimen_control_cell(
                                "Stroke off",
                                Dropdown::new("dropdown-stroke-off", "Value")
                                    .stroke(false)
                                    .into_any_element(),
                                cx,
                            ),
                        ],
                        cx,
                    ),
                ]),
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn render_dropdown_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-dropdown",
            self.render_dropdown_story(cx),
            self.dropdown_story.last_action.clone(),
            cx,
        )
    }
}
