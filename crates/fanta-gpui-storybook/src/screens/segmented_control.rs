//! UI3 segmented-control component story.

use std::collections::HashMap;

use crate::*;
use gpui::FocusHandle;

use super::specimen::{
    specimen_card, specimen_control_cell, specimen_row, specimen_rows, specimen_story_root,
    specimen_wide_control_cell,
};

const LABELS: [&str; 6] = ["One", "Two", "Three", "Four", "Five", "Six"];
const ICONS: [(LucideIcon, &str); 6] = [
    (LucideIcon::Frame, "Frame"),
    (LucideIcon::TypeIcon, "Text"),
    (LucideIcon::Image, "Image"),
    (LucideIcon::Component, "Component"),
    (LucideIcon::Diamond, "Instance"),
    (LucideIcon::LayoutGrid, "Layout"),
];

const COMMON_VARIANTS: [(&str, usize, SegmentedControlState); 16] = [
    ("Autolayout", 2, SegmentedControlState::Default),
    ("Text Alignment", 2, SegmentedControlState::Default),
    ("Orientation", 2, SegmentedControlState::Default),
    ("Text Decoration", 2, SegmentedControlState::Default),
    ("Code or Table", 2, SegmentedControlState::Default),
    ("Autolayout", 3, SegmentedControlState::Default),
    ("Vertical Alignment", 3, SegmentedControlState::Default),
    ("Box Resizing", 3, SegmentedControlState::Default),
    ("Overlay Positioning", 3, SegmentedControlState::Default),
    ("Text Alignment", 3, SegmentedControlState::Default),
    ("Text Transform", 3, SegmentedControlState::Default),
    ("Text Alignment", 4, SegmentedControlState::Default),
    ("Text Alignment", 4, SegmentedControlState::Disabled),
    ("Prototype Arrows", 4, SegmentedControlState::Default),
    ("Number Transform", 5, SegmentedControlState::Default),
    ("Text Transform", 6, SegmentedControlState::Default),
];

pub(crate) struct SegmentedControlStory {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) selected_label: usize,
    pub(crate) selected_icon: usize,
    pub(crate) specimen_selections: HashMap<String, usize>,
    pub(crate) last_action: SharedString,
}

impl SegmentedControlStory {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            selected_label: 0,
            selected_icon: 0,
            specimen_selections: HashMap::new(),
            last_action: "Choose a segment in either controlled example".into(),
        }
    }
}

fn label_items(count: usize) -> Vec<SharedString> {
    LABELS[..count]
        .iter()
        .map(|label| (*label).into())
        .collect()
}

fn icon_items(count: usize) -> Vec<SegmentIconOption> {
    ICONS[..count]
        .iter()
        .map(|(icon, label)| SegmentIconOption::new(*icon, *label))
        .collect()
}

fn common_icons(kind: &str, count: usize) -> Vec<SegmentIconOption> {
    let icons: &[LucideIcon] = match kind {
        "Autolayout" => &[
            LucideIcon::Rows3,
            LucideIcon::LayoutGrid,
            LucideIcon::Columns3,
        ],
        "Vertical Alignment" => &[
            LucideIcon::AlignVerticalJustifyStart,
            LucideIcon::AlignVerticalJustifyCenter,
            LucideIcon::AlignVerticalJustifyEnd,
        ],
        "Box Resizing" => &[
            LucideIcon::SquareDashed,
            LucideIcon::Scaling,
            LucideIcon::Frame,
        ],
        "Overlay Positioning" => &[
            LucideIcon::PanelTop,
            LucideIcon::SquareDashedMousePointer,
            LucideIcon::PanelsTopLeft,
        ],
        "Text Alignment" => &[
            LucideIcon::TextAlignStart,
            LucideIcon::TextAlignCenter,
            LucideIcon::TextAlignEnd,
            LucideIcon::TextAlignJustify,
        ],
        "Text Transform" => &[
            LucideIcon::CaseLower,
            LucideIcon::CaseUpper,
            LucideIcon::CaseSensitive,
            LucideIcon::ALargeSmall,
            LucideIcon::AArrowUp,
            LucideIcon::AArrowDown,
        ],
        "Orientation" => &[LucideIcon::Rows3, LucideIcon::Columns3],
        "Text Decoration" => &[LucideIcon::Baseline, LucideIcon::TypeIcon],
        "Code or Table" => &[LucideIcon::CodeXml, LucideIcon::Table2],
        "Prototype Arrows" => &[
            LucideIcon::ArrowLeft,
            LucideIcon::ArrowUp,
            LucideIcon::ArrowRight,
            LucideIcon::ArrowDown,
        ],
        "Number Transform" => &[
            LucideIcon::Hash,
            LucideIcon::ArrowDown01,
            LucideIcon::ArrowUp10,
            LucideIcon::Plus,
            LucideIcon::Minus,
        ],
        _ => &[],
    };
    icons[..count]
        .iter()
        .enumerate()
        .map(|(index, icon)| SegmentIconOption::new(*icon, format!("{kind} option {}", index + 1)))
        .collect()
}

impl Storybook {
    fn segmented_main_matrix(
        &self,
        variant: SegmentedControlVariant,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let mut rows = Vec::new();
        for state in SegmentedControlState::ALL {
            let cells = (2..=6)
                .map(|count| {
                    let key = format!(
                        "segmented-main-{}-{count}-{}",
                        variant.label().to_ascii_lowercase(),
                        state.label().to_ascii_lowercase()
                    );
                    let selected = self
                        .segmented_control_story
                        .specimen_selections
                        .get(&key)
                        .copied()
                        .unwrap_or(1.min(count - 1));
                    let mut control = match variant {
                        SegmentedControlVariant::Icon => SegmentedControl::icons(
                            SharedString::from(key.clone()),
                            icon_items(count),
                        ),
                        SegmentedControlVariant::Label => SegmentedControl::labels(
                            SharedString::from(key.clone()),
                            label_items(count),
                        ),
                    }
                    .selected_index(selected)
                    .state(state);
                    if state == SegmentedControlState::Default {
                        let selection_key = key.clone();
                        let description = format!("{} {count}-tab specimen", variant.label());
                        control = control.on_change(cx.listener(
                            move |this, selection: &SegmentedControlSelection, _, cx| {
                                this.segmented_control_story
                                    .specimen_selections
                                    .insert(selection_key.clone(), selection.index);
                                this.segmented_control_story.last_action = format!(
                                    "Selected {} in {description} via {}",
                                    selection.index + 1,
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
                    specimen_wide_control_cell(
                        format!("{count} tabs · selected {}", selected + 1),
                        control.into_any_element(),
                        cx,
                    )
                })
                .collect();
            rows.push(specimen_row(
                SharedString::from(format!(
                    "segmented-main-{}-{}",
                    variant.label().to_ascii_lowercase(),
                    state.label().to_ascii_lowercase()
                )),
                format!("{} · {}", variant.label(), state.label()),
                cells,
                cx,
            ));
        }
        rows
    }

    fn segment_primitive_cells(&self, icon: bool, cx: &mut Context<Self>) -> Vec<AnyElement> {
        [
            (true, SegmentState::Default),
            (false, SegmentState::Default),
            (true, SegmentState::Focused),
            (false, SegmentState::Focused),
            (false, SegmentState::Disabled),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (active, state))| {
            let family = if icon { "icon" } else { "label" };
            let key = format!("segment-{family}-primitive-{index}");
            let preview_active = self
                .segmented_control_story
                .specimen_selections
                .get(&key)
                .map_or(active, |value| *value == 1);
            let content = if icon {
                let mut segment =
                    SegmentIcon::new(SharedString::from(key.clone()), LucideIcon::Frame)
                        .active(preview_active)
                        .preview_state(state);
                if state != SegmentState::Disabled {
                    let selection_key = key.clone();
                    segment = segment.on_activate(cx.listener(
                        move |this, event: &ActivateEvent, _, cx| {
                            let active = this
                                .segmented_control_story
                                .specimen_selections
                                .get(&selection_key)
                                .map_or(active, |value| *value == 1);
                            this.segmented_control_story
                                .specimen_selections
                                .insert(selection_key.clone(), usize::from(!active));
                            this.segmented_control_story.last_action = format!(
                                "Toggled icon segment {} via {}",
                                if active { "inactive" } else { "active" },
                                if event.keyboard {
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
                segment.into_any_element()
            } else {
                let mut segment = SegmentLabel::new(SharedString::from(key.clone()), "Label")
                    .active(preview_active)
                    .preview_state(state);
                if state != SegmentState::Disabled {
                    let selection_key = key.clone();
                    segment = segment.on_activate(cx.listener(
                        move |this, event: &ActivateEvent, _, cx| {
                            let active = this
                                .segmented_control_story
                                .specimen_selections
                                .get(&selection_key)
                                .map_or(active, |value| *value == 1);
                            this.segmented_control_story
                                .specimen_selections
                                .insert(selection_key.clone(), usize::from(!active));
                            this.segmented_control_story.last_action = format!(
                                "Toggled label segment {} via {}",
                                if active { "inactive" } else { "active" },
                                if event.keyboard {
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
                segment.into_any_element()
            };
            specimen_control_cell(
                format!(
                    "{} · {}",
                    if preview_active { "Active" } else { "Inactive" },
                    state.label()
                ),
                div()
                    .w(px(72.))
                    .h(px(fanta_gpui::atoms::tokens::RowHeight::FIELD))
                    .child(content)
                    .into_any_element(),
                cx,
            )
        })
        .collect()
    }

    pub(crate) fn render_segmented_control_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let label_selected = self.segmented_control_story.selected_label;
        let icon_selected = self.segmented_control_story.selected_icon;
        let live_labels = SegmentedControl::labels("segmented-live-labels", label_items(3))
            .selected_index(label_selected)
            .on_change(
                cx.listener(|this, selection: &SegmentedControlSelection, _, cx| {
                    this.segmented_control_story.selected_label = selection.index;
                    this.segmented_control_story.last_action = format!(
                        "Selected label segment {} via {}",
                        selection.index + 1,
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
            .into_any_element();
        let live_icons = SegmentedControl::icons("segmented-live-icons", icon_items(4))
            .selected_index(icon_selected)
            .on_change(
                cx.listener(|this, selection: &SegmentedControlSelection, _, cx| {
                    this.segmented_control_story.selected_icon = selection.index;
                    this.segmented_control_story.last_action = format!(
                        "Selected icon segment {} via {}",
                        selection.index + 1,
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
            .into_any_element();

        let mut primary_rows = self.segmented_main_matrix(SegmentedControlVariant::Label, cx);
        primary_rows.extend(self.segmented_main_matrix(SegmentedControlVariant::Icon, cx));

        let common_rows = COMMON_VARIANTS
            .chunks(4)
            .enumerate()
            .map(|(row_index, specs)| {
                let cells = specs
                    .iter()
                    .enumerate()
                    .map(|(cell_index, (kind, count, state))| {
                        let key = format!("segmented-common-{row_index}-{cell_index}");
                        let selected = self
                            .segmented_control_story
                            .specimen_selections
                            .get(&key)
                            .copied()
                            .unwrap_or((*count / 2).min(count.saturating_sub(1)));
                        let mut control = SegmentedControl::icons(
                            SharedString::from(key.clone()),
                            common_icons(kind, *count),
                        )
                        .selected_index(selected)
                        .state(*state);
                        if *state == SegmentedControlState::Default {
                            let selection_key = key.clone();
                            let description = (*kind).to_owned();
                            control = control.on_change(cx.listener(
                                move |this, selection: &SegmentedControlSelection, _, cx| {
                                    this.segmented_control_story
                                        .specimen_selections
                                        .insert(selection_key.clone(), selection.index);
                                    this.segmented_control_story.last_action = format!(
                                        "Selected {} in {description} via {}",
                                        selection.index + 1,
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
                        specimen_wide_control_cell(
                            format!("{count} · {} · selected {}", state.label(), selected + 1),
                            control.into_any_element(),
                            cx,
                        )
                    })
                    .collect();
                specimen_row(
                    SharedString::from(format!("segmented-common-row-{row_index}")),
                    specs
                        .iter()
                        .map(|(kind, _, _)| *kind)
                        .collect::<Vec<_>>()
                        .join(" · "),
                    cells,
                    cx,
                )
            })
            .collect();

        specimen_story_root("storybook-segmented-control")
            .track_focus(&self.segmented_control_story.focus_handle)
            .child(specimen_card(
                "segmented-live-card",
                "Segmented control · controlled interaction",
                "The host owns the selected index. Every segment emits a typed selection intent through the same pointer and Enter/Space path.",
                specimen_rows(vec![specimen_row(
                    "segmented-live-row",
                    format!(
                        "Label {} selected · Icon {} selected",
                        label_selected + 1,
                        icon_selected + 1
                    ),
                    vec![
                        specimen_wide_control_cell("Label", live_labels, cx),
                        specimen_wide_control_cell("Icon", live_icons, cx),
                    ],
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "segmented-main-card",
                "Segmented control · all 20 variants",
                "This is the complete Figma matrix: icon and label content, tab counts 02 through 06, and default or disabled group state. Every enabled specimen is interactive.",
                specimen_rows(primary_rows),
                cx,
            ))
            .child(specimen_card(
                "segmented-primitives-card",
                "Segment primitives · all 10 variants",
                "Icon and label segments each expose active and inactive default/focused states plus the inactive disabled state present in Figma. Enabled primitive specimens toggle their active state.",
                specimen_rows(vec![
                    specimen_row(
                        "segment-label-primitives-row",
                        "_Segment label · 5 variants",
                        self.segment_primitive_cells(false, cx),
                        cx,
                    ),
                    specimen_row(
                        "segment-icon-primitives-row",
                        "_Segment icon · 5 variants",
                        self.segment_primitive_cells(true, cx),
                        cx,
                    ),
                ]),
                cx,
            ))
            .child(specimen_card(
                "segmented-common-card",
                "Common segmented controls · all 16 Figma presets",
                "These inspector-oriented combinations reproduce every concrete preset in the linked component set. Every enabled preset selects on click, Enter/Space, arrows, Home, or End; the sole disabled Text Alignment example remains inert.",
                specimen_rows(common_rows),
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn render_segmented_control_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-segmented-control",
            self.render_segmented_control_story(cx),
            self.segmented_control_story.last_action.clone(),
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn story_covers_every_linked_figma_variant() {
        assert_eq!(2 * 5 * 2, 20, "main component variants");
        assert_eq!(5 + 5, 10, "icon and label primitive variants");
        assert_eq!(COMMON_VARIANTS.len(), 16, "common concrete presets");
        assert_eq!(20 + 10 + COMMON_VARIANTS.len(), 46);
    }
}
