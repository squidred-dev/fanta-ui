//! The Labels & truncation atom story.
//!
//! `truncating_label` demonstrated directly: full-width rows whose labels
//! ellipsize live as the shared story viewport is scrubbed narrower, plus
//! the organism-context specimen the retired Foundations catalog
//! established — a Layers panel mounted at a deliberately cramped width so
//! every long title truncates instead of widening the panel.

use crate::*;

use fanta_gpui::prelude::LayersPanelNodeKind;

use super::specimen::{
    framed_panel, specimen_card, specimen_row, specimen_rows, specimen_story_root,
};

/// Width used by the truncation specimen panel: deliberately cramped.
const TRUNCATION_SPECIMEN_WIDTH: f32 = 240.;

/// Long-titled rows mounted at a cramped width so every label exercises
/// `truncating_label` instead of forcing horizontal scroll extents.
pub(crate) fn truncation_inventory() -> Vec<LayersPanelItem> {
    use LayersPanelNodeKind as Kind;

    vec![
        LayersPanelItem::new(
            "truncate-frame",
            "An extremely long frame name that must truncate instead of widening its panel",
            Kind::Frame,
        ),
        LayersPanelItem::new(
            "truncate-text",
            "Typography specimen with an unreasonably verbose layer title for truncation",
            Kind::Text,
        ),
        LayersPanelItem::new(
            "truncate-component",
            "Component specimen · primary action button · desktop · hover · 2026 refresh",
            Kind::Component,
        ),
    ]
}

/// The direct `truncating_label` demo rows: icon + label + fixed satellite.
pub(crate) fn label_demo_rows() -> [(&'static str, LucideIcon); 3] {
    [
        (
            "A very long navigation frame title that ellipsizes as the viewport narrows",
            LucideIcon::Frame,
        ),
        (
            "Body copy layer with an unreasonably verbose editorial name for the demo",
            LucideIcon::TypeIcon,
        ),
        (
            "Primary action component · desktop · hover · localized · 2026 refresh",
            LucideIcon::Component,
        ),
    ]
}

pub(crate) struct LabelsScreen {
    pub(crate) truncation: Entity<LayersPanel>,
    pub(crate) last_action: SharedString,
}

impl LabelsScreen {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Storybook>) -> Self {
        let truncation =
            cx.new(|cx| LayersPanel::new("labels-truncation", truncation_inventory(), window, cx));
        Self {
            truncation,
            last_action: "Ready — scrub the viewport width and watch every label ellipsize".into(),
        }
    }

    pub(crate) fn handle_truncation_action(
        &mut self,
        panel: Entity<LayersPanel>,
        action: &LayersPanelAction,
        cx: &mut Context<Storybook>,
    ) {
        if let LayersPanelAction::SelectRequested { node_id, .. } = action {
            panel.update(cx, |panel, cx| {
                panel.set_selected_node_ids(vec![node_id.clone()], cx);
            });
            self.last_action = format!("Selected the {node_id} truncation specimen").into();
            cx.notify();
        }
    }
}

impl Storybook {
    fn render_label_demo_rows(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut rows = v_flex()
            .id("labels-live-rows")
            .debug_selector(|| "labels-live-rows".to_owned())
            .w_full()
            .gap_2();
        for (index, (text, icon)) in label_demo_rows().into_iter().enumerate() {
            rows = rows.child(
                h_flex()
                    .w_full()
                    .h(px(30.))
                    .items_center()
                    .gap_2()
                    .px_2()
                    .rounded(px(4.))
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().secondary.opacity(0.35))
                    .child(render_lucide_icon(icon, cx.theme().muted_foreground, 13.))
                    .child(truncating_label(text).text_sm())
                    .child(
                        div()
                            .flex_none()
                            .px_2()
                            .py_0p5()
                            .rounded(px(4.))
                            .border_1()
                            .border_color(cx.theme().border)
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("satellite {}", index + 1)),
                    ),
            );
        }
        rows.into_any_element()
    }

    pub(crate) fn render_labels_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let demo_rows = self.render_label_demo_rows(cx);
        specimen_story_root("storybook-labels")
            .child(specimen_card(
                "labels-live-truncation",
                "truncating_label · live at the story viewport width",
                "Each row is icon + truncating_label + a fixed satellite. The label \
                 claims the remaining row width and ellipsizes overflow on one line, \
                 so the fixed neighbors never move. Scrub the story viewport width \
                 handle (or pick the Narrow preset) and watch the labels give way \
                 instead of widening the row.",
                specimen_rows(vec![specimen_row(
                    "labels-live-row",
                    "Live rows · scrub the viewport width and the labels give way",
                    vec![demo_rows],
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "labels-panel-truncation",
                "In context · Layers panel at a cramped width",
                "The same atom inside the list_row chrome, mounted at a deliberately \
                 cramped 240 px: every long title truncates instead of widening the \
                 panel or growing a horizontal scroll extent.",
                specimen_rows(vec![specimen_row(
                    "labels-panel-row",
                    "Layers panel at a deliberately cramped 240 px",
                    vec![framed_panel(
                        TRUNCATION_SPECIMEN_WIDTH,
                        180.,
                        self.labels_screen.truncation.clone().into_any_element(),
                        cx,
                    )],
                    cx,
                )]),
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn render_labels_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-labels",
            self.render_labels_story(cx),
            self.labels_screen.last_action.clone(),
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncation_specimens_exceed_their_panel_width() {
        let items = truncation_inventory();
        assert!(!items.is_empty());
        for item in &items {
            assert!(
                item.title.len() > 60,
                "{:?} must be long enough to truncate at {TRUNCATION_SPECIMEN_WIDTH} px",
                item.title
            );
        }
        for (text, _) in label_demo_rows() {
            assert!(
                text.len() > 60,
                "{text:?} must be long enough to truncate at the Narrow preset"
            );
        }
    }
}
