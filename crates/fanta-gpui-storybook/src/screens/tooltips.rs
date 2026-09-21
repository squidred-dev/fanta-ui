//! UI3 Tooltip and Tooltip link component story.

use crate::*;
use gpui::FocusHandle;

use super::specimen::{
    specimen_card, specimen_control_cell, specimen_row, specimen_rows, specimen_story_root,
};

const LINK_COPY: [(TooltipLinkVariant, &str, Option<&str>); 8] = [
    (TooltipLinkVariant::Url, "Type or paste URL", None),
    (TooltipLinkVariant::Link, "Open google.com", Some("Edit")),
    (
        TooltipLinkVariant::Phone,
        "Call (415) 355-0394",
        Some("Edit"),
    ),
    (
        TooltipLinkVariant::Email,
        "Copy mail@mail.com",
        Some("Send mail · Edit"),
    ),
    (TooltipLinkVariant::Page, "Go to page", Some("Edit")),
    (
        TooltipLinkVariant::Prototype,
        "Open prototype",
        Some("Edit"),
    ),
    (TooltipLinkVariant::Frame, "Go to frame", Some("Edit")),
    (TooltipLinkVariant::File, "Open file", Some("Edit")),
];

pub(crate) struct TooltipsStory {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) hovered_direction: Option<TooltipDirection>,
    pub(crate) last_action: SharedString,
}

impl TooltipsStory {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            hovered_direction: None,
            last_action: "Hover a trigger or activate a tooltip link".into(),
        }
    }
}

fn tooltip_cell(direction: TooltipDirection, cx: &mut Context<Storybook>) -> AnyElement {
    specimen_control_cell(
        direction.label(),
        Tooltip::new(format!("tooltip-direction-{:?}", direction), "Tooltip text")
            .direction(direction)
            .hotkey("⌘V")
            .into_any_element(),
        cx,
    )
}

impl Storybook {
    fn live_tooltip_cells(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        TooltipDirection::ALL
            .into_iter()
            .map(|direction| {
                let visible = self.tooltips_story.hovered_direction == Some(direction);
                let trigger = div()
                    .id(SharedString::from(format!(
                        "tooltip-trigger-{:?}",
                        direction
                    )))
                    .debug_selector(move || format!("tooltip-trigger-{:?}", direction))
                    .key_context(CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .h(px(fanta_gpui::atoms::tokens::RowHeight::FIELD))
                    .px_2()
                    .flex()
                    .items_center()
                    .rounded(px(fanta_gpui::atoms::tokens::ButtonGeometry::RADIUS))
                    .border_1()
                    .border_color(SemanticColor::Border.resolve(cx))
                    .bg(SemanticColor::BackgroundSecondary.resolve(cx))
                    .cursor_pointer()
                    .hover(|style| style.bg(SemanticColor::BackgroundHover.resolve(cx)))
                    .on_hover(cx.listener(move |this, hovered, _, cx| {
                        if *hovered {
                            this.tooltips_story.hovered_direction = Some(direction);
                            this.tooltips_story.last_action =
                                format!("Showing {} tooltip", direction.label()).into();
                        } else if this.tooltips_story.hovered_direction == Some(direction) {
                            this.tooltips_story.hovered_direction = None;
                        }
                        cx.notify();
                    }))
                    .on_activate(cx.listener(move |this, event: &ActivateEvent, _, cx| {
                        this.tooltips_story.hovered_direction = Some(direction);
                        this.tooltips_story.last_action = format!(
                            "Pinned {} tooltip via {}",
                            direction.label(),
                            if event.keyboard {
                                "keyboard"
                            } else {
                                "pointer"
                            }
                        )
                        .into();
                        cx.notify();
                    }))
                    .typography(TypographyToken::BodyMedium)
                    .child(direction.label());
                specimen_control_cell(
                    direction.label(),
                    v_flex()
                        .items_center()
                        .gap_2()
                        .child(trigger)
                        .when(visible, |column| {
                            column.child(
                                Tooltip::new(
                                    format!("tooltip-live-{:?}", direction),
                                    "Tooltip text",
                                )
                                .direction(direction)
                                .hotkey("⌘V"),
                            )
                        })
                        .into_any_element(),
                    cx,
                )
            })
            .collect()
    }

    fn tooltip_link_cells(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        LINK_COPY
            .into_iter()
            .map(|(variant, label, secondary)| {
                let mut link = TooltipLink::new(
                    format!("tooltip-link-{}", variant.label().to_ascii_lowercase()),
                    variant,
                    label,
                )
                .on_activate(cx.listener(
                    move |this, action: &TooltipLinkAction, _, cx| {
                        this.tooltips_story.last_action = format!(
                            "Activated {} via {}",
                            action.variant.label(),
                            if action.keyboard {
                                "keyboard"
                            } else {
                                "pointer"
                            }
                        )
                        .into();
                        cx.notify();
                    },
                ));
                if let Some(secondary) = secondary {
                    link = link.secondary(secondary);
                }
                v_flex()
                    .w(px(304.))
                    .flex_none()
                    .gap_1p5()
                    .child(link)
                    .child(
                        div()
                            .w_full()
                            .text_center()
                            .typography(TypographyToken::BodyMedium)
                            .text_color(SemanticColor::TextTertiary.resolve(cx))
                            .child(variant.label()),
                    )
                    .into_any_element()
            })
            .collect()
    }

    pub(crate) fn render_tooltips_story(&self, cx: &mut Context<Self>) -> AnyElement {
        specimen_story_root("storybook-tooltips")
            .track_focus(&self.tooltips_story.focus_handle)
            .child(specimen_card(
                "tooltip-live-card",
                "Tooltip · live triggers",
                "Hover any trigger, or focus it and press Enter or Space. The host owns visibility while the tooltip component owns direction-specific surface geometry.",
                specimen_rows(vec![specimen_row(
                    "tooltip-live-row",
                    "Interactive triggers",
                    self.live_tooltip_cells(cx),
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "tooltip-direction-card",
                "Tooltip · all 8 Figma directions",
                "The caret follows Figma's Top, Bottom, Left, and Right placements while colors and typography resolve through the active GPUI theme.",
                specimen_rows(vec![specimen_row(
                    "tooltip-direction-row",
                    "Direction",
                    TooltipDirection::ALL
                        .into_iter()
                        .map(|direction| tooltip_cell(direction, cx))
                        .collect(),
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "tooltip-link-card",
                "Tooltip link · all 8 Figma variants",
                "URL, Link, Phone, Email, Page, Prototype, Frame, and File rows emit typed activation intents through pointer and keyboard input.",
                specimen_rows(vec![specimen_row(
                    "tooltip-link-row",
                    "Variant",
                    self.tooltip_link_cells(cx),
                    cx,
                )]),
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn render_tooltips_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-tooltips",
            self.render_tooltips_story(cx),
            self.tooltips_story.last_action.clone(),
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn story_covers_both_linked_tooltip_sets() {
        assert_eq!(TooltipDirection::ALL.len(), 8);
        assert_eq!(LINK_COPY.len(), 8);
    }
}
