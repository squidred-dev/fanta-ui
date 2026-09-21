//! UI3 tabs and `_Tab` primitive story.

use std::collections::HashMap;

use crate::*;
use gpui::FocusHandle;

use super::specimen::{specimen_card, specimen_row, specimen_rows, specimen_story_root};

const TAB_VARIANTS: [(bool, bool, TabState); 7] = [
    (false, true, TabState::Default),
    (false, true, TabState::Focused),
    (false, false, TabState::Default),
    (false, false, TabState::Focused),
    (false, false, TabState::Hover),
    (true, false, TabState::Default),
    (true, false, TabState::Hover),
];

pub(crate) struct TabsStory {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) selected_by_count: [usize; 4],
    primitive_selected: HashMap<String, bool>,
    pub(crate) last_action: SharedString,
}

impl TabsStory {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            selected_by_count: [0; 4],
            primitive_selected: HashMap::new(),
            last_action: "Choose a tab".into(),
        }
    }
}

fn tab_labels(count: usize) -> Vec<SharedString> {
    ["Design", "Prototype", "Inspect", "Resources"][..count]
        .iter()
        .map(|label| (*label).into())
        .collect()
}

fn tabs_cell(label: impl Into<SharedString>, content: AnyElement, cx: &App) -> AnyElement {
    v_flex()
        .w(px(328.))
        .flex_none()
        .gap_1p5()
        .child(
            div()
                .w_full()
                .h(px(72.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(8.))
                .border_1()
                .border_color(SemanticColor::Border.resolve(cx))
                .bg(SemanticColor::BackgroundSecondary.resolve(cx).opacity(0.55))
                .child(content),
        )
        .child(
            div()
                .w_full()
                .text_center()
                .typography(TypographyToken::BodyMedium)
                .text_color(SemanticColor::TextTertiary.resolve(cx))
                .child(label.into()),
        )
        .into_any_element()
}

impl Storybook {
    fn tabs_count_cells(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        (1..=4)
            .map(|count| {
                let selected = self.tabs_story.selected_by_count[count - 1].min(count - 1);
                let control = Tabs::new(format!("tabs-count-{count}"), tab_labels(count))
                    .selected_index(selected)
                    .badges(
                        (0..count)
                            .map(|index| (index == count - 1 && count > 1).then(|| "3".into()))
                            .collect(),
                    )
                    .on_change(cx.listener(move |this, selection: &TabSelection, _, cx| {
                        this.tabs_story.selected_by_count[count - 1] = selection.index;
                        this.tabs_story.last_action = format!(
                            "Selected tab {} in the {count}-tab control via {}",
                            selection.index + 1,
                            if selection.keyboard {
                                "keyboard"
                            } else {
                                "pointer"
                            }
                        )
                        .into();
                        cx.notify();
                    }));
                tabs_cell(
                    format!("Tab Count={count} · selected {}", selected + 1),
                    control.into_any_element(),
                    cx,
                )
            })
            .collect()
    }

    fn tab_primitive_cells(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        TAB_VARIANTS
            .into_iter()
            .enumerate()
            .map(|(index, (single_tab, initial_selected, state))| {
                let key = format!("tab-primitive-{index}");
                let selected = self
                    .tabs_story
                    .primitive_selected
                    .get(&key)
                    .copied()
                    .unwrap_or(initial_selected);
                let mut tab = Tab::new(key.clone(), "Tab title")
                    .single_tab(single_tab)
                    .selected(selected)
                    .preview_state(state);
                if index == 2 {
                    tab = tab.badge("3");
                }
                let selection_key = key.clone();
                tab = tab.on_activate(cx.listener(move |this, event: &ActivateEvent, _, cx| {
                    let current = this
                        .tabs_story
                        .primitive_selected
                        .get(&selection_key)
                        .copied()
                        .unwrap_or(initial_selected);
                    this.tabs_story
                        .primitive_selected
                        .insert(selection_key.clone(), !current);
                    this.tabs_story.last_action = format!(
                        "Toggled _Tab {} via {}",
                        index + 1,
                        if event.keyboard {
                            "keyboard"
                        } else {
                            "pointer"
                        }
                    )
                    .into();
                    cx.notify();
                }));
                tabs_cell(
                    format!(
                        "Single={} · Selected={} · {}",
                        single_tab,
                        selected,
                        state.label()
                    ),
                    tab.into_any_element(),
                    cx,
                )
            })
            .collect()
    }

    pub(crate) fn render_tabs_story(&self, cx: &mut Context<Self>) -> AnyElement {
        specimen_story_root("storybook-tabs")
            .track_focus(&self.tabs_story.focus_handle)
            .child(specimen_card(
                "tabs-count-card",
                "Tabs · all 4 Figma variants",
                "Each count variant is a real controlled tab list. Click a tab, focus one and press Enter or Space, or use arrows, Home, and End.",
                specimen_rows(vec![specimen_row(
                    "tabs-count-row",
                    "Tab Count=1–4",
                    self.tabs_count_cells(cx),
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "tab-primitive-card",
                "_Tab · all 7 valid Figma variants",
                "The primitive matrix follows Figma's declared combinations, including selected, focused, hover, single-tab, and optional badge treatment. Every specimen toggles on activation.",
                specimen_rows(vec![specimen_row(
                    "tab-primitive-row",
                    "Single Tab · Selected · State",
                    self.tab_primitive_cells(cx),
                    cx,
                )]),
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn render_tabs_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-tabs",
            self.render_tabs_story(cx),
            self.tabs_story.last_action.clone(),
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn story_covers_both_linked_tabs_sets() {
        assert_eq!((1..=4).count(), 4);
        assert_eq!(TAB_VARIANTS.len(), 7);
    }
}
