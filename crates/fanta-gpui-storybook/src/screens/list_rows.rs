//! The List rows molecule story.
//!
//! `list_row` specimens composed from the public atoms: a node-kind icon, a
//! `truncating_label`, and a lock `icon_button` satellite per row. Rows show
//! the caller-added hover and selected treatments, the molecule's reserved
//! focus ring, and emit selection/lock demo intents through the single
//! activation path.

use fanta_gpui::atoms::TypographyExt as _;
use gpui::FocusHandle;

use crate::*;

use super::specimen::{specimen_card, specimen_row, specimen_rows, specimen_story_root};

const ROW_HEIGHT: f32 = 30.;
/// The panels' natural width, so the column reads like a real Layers or
/// Pages panel.
const ROW_PANEL_WIDTH: f32 = 337.;

/// The specimen rows: label, leading node-kind icon, and the state tag
/// naming what each row demonstrates.
pub(crate) const LIST_ROW_SPECIMENS: [(&str, LucideIcon, Option<&str>); 5] = [
    ("Navigation frame", LucideIcon::Frame, Some("hover me")),
    ("Hero component", LucideIcon::Component, Some("selected")),
    ("Body copy", LucideIcon::TypeIcon, Some("Tab focuses")),
    ("Cover image", LucideIcon::Image, Some("locked")),
    ("Boolean union", LucideIcon::Combine, None),
];

pub(crate) struct ListRowsScreen {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) selected: Option<usize>,
    pub(crate) locked: [bool; LIST_ROW_SPECIMENS.len()],
    pub(crate) last_action: SharedString,
}

impl ListRowsScreen {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        let mut locked = [false; LIST_ROW_SPECIMENS.len()];
        // One row seeds locked so the satellite's selected state is visible
        // before any interaction.
        locked[3] = true;
        Self {
            focus_handle: cx.focus_handle(),
            selected: Some(1),
            locked,
            last_action: "Ready — activate a row to select it, or toggle a lock satellite".into(),
        }
    }
}

impl Storybook {
    fn render_list_row_specimen(&self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        let (label, icon, state_tag) = LIST_ROW_SPECIMENS[index];
        let selected = self.list_rows_screen.selected == Some(index);
        let locked = self.list_rows_screen.locked[index];
        let satellite_id = SharedString::from(format!("list-rows-lock-{index}"));
        list_row(
            SharedString::from(format!("list-rows-row-{index}")),
            px(ROW_HEIGHT),
            cx,
        )
        .debug_selector(move || format!("list-rows-row-{index}"))
        .px_2()
        .gap_2()
        .hover(|style| style.bg(fanta_gpui::atoms::SemanticColor::BackgroundHover.resolve(cx)))
        .when(selected, |row| {
            row.bg(fanta_gpui::atoms::SemanticColor::BackgroundSelected
                .resolve(cx)
                .opacity(0.18))
        })
        .on_activate(cx.listener(move |this, event: &ActivateEvent, _, cx| {
            this.list_rows_screen.selected = Some(index);
            this.list_rows_screen.last_action = format!(
                "Selected the {label} row via {}",
                if event.keyboard {
                    "Enter/Space"
                } else {
                    "pointer"
                }
            )
            .into();
            cx.notify();
        }))
        .child(render_lucide_icon(
            icon,
            fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx),
            13.,
        ))
        .child(truncating_label(label).typography(fanta_gpui::atoms::TypographyToken::BodyLarge))
        .when_some(state_tag, |row, tag| {
            row.child(
                div()
                    .flex_none()
                    .px_1p5()
                    .rounded(px(3.))
                    .border_1()
                    .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                    .text_size(px(10.))
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child(tag),
            )
        })
        .child(
            icon_button(satellite_id, px(22.), px(4.), cx)
                .debug_selector(move || format!("list-rows-lock-{index}"))
                .when(locked, |button| {
                    button.bg(fanta_gpui::atoms::SemanticColor::BackgroundHover.resolve(cx))
                })
                .on_activate(cx.listener(move |this, event: &ActivateEvent, _, cx| {
                    cx.stop_propagation();
                    this.list_rows_screen.locked[index] = !this.list_rows_screen.locked[index];
                    this.list_rows_screen.last_action = format!(
                        "Set the {label} row lock to {} via {}",
                        this.list_rows_screen.locked[index],
                        if event.keyboard {
                            "Enter/Space"
                        } else {
                            "pointer"
                        }
                    )
                    .into();
                    cx.notify();
                }))
                .child(render_lucide_icon(
                    if locked {
                        LucideIcon::Lock
                    } else {
                        LucideIcon::LockOpen
                    },
                    if locked {
                        fanta_gpui::atoms::SemanticColor::Text.resolve(cx)
                    } else {
                        fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx)
                    },
                    12.,
                )),
        )
        .into_any_element()
    }

    pub(crate) fn render_list_rows_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut rows = v_flex()
            .id("list-rows-panel")
            .debug_selector(|| "list-rows-panel".to_owned())
            .track_focus(&self.list_rows_screen.focus_handle)
            .w(px(ROW_PANEL_WIDTH))
            .max_w_full()
            .flex_none()
            .p_2()
            .gap_1()
            .rounded(px(8.))
            .border_1()
            .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
            .bg(fanta_gpui::atoms::SemanticColor::BackgroundSecondary
                .resolve(cx)
                .opacity(0.35));
        for index in 0..LIST_ROW_SPECIMENS.len() {
            rows = rows.child(self.render_list_row_specimen(index, cx));
        }

        specimen_story_root("storybook-list-rows")
            .child(specimen_card(
                "list-rows-specimens",
                "list_row · selection, hover, focus, and satellites",
                "The molecule owns the §9 row recipe: key context, a tab stop, \
                 pointer cursor, and a focus ring whose border width is reserved \
                 while unfocused. The caller adds the hover fill, the selected \
                 tint, children, and one on_activate handler; the tags on the \
                 rows name the state each one demonstrates. The lock satellite is \
                 an icon_button whose activation stops propagation so it never \
                 also selects the row.",
                specimen_rows(vec![specimen_row(
                    "list-rows-panel-row",
                    "Panel column at the panels' natural 337 px width",
                    vec![rows.into_any_element()],
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "list-rows-roving-hint",
                "Roving navigation in real panels",
                "These specimens are plain tab stops so the states stay easy to \
                 inspect. Dense production lists rove instead (§9): the row is the \
                 single tab stop, Up/Down move focus between rows, and satellites \
                 are reached through row-scoped key bindings that emit the same \
                 typed intents. The Layers and Pages organisms demonstrate the \
                 full recipe.",
                div()
                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child("Open the Layers panel story under Organisms to compare.")
                    .into_any_element(),
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn render_list_rows_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-list-rows",
            self.render_list_rows_story(cx),
            self.list_rows_screen.last_action.clone(),
            cx,
        )
    }
}
