//! The Vector icons atom story.
//!
//! Every [`ControlIcon`] rendered and labeled through the public
//! `render_control_icon` atom on the shared specimen cell grid — grouped by
//! family, each cell showing the 16 px working size above a 32 px large
//! variant, filterable through the same search-input pattern the icon-asset
//! catalog uses — plus the two organism-context specimens the retired
//! Foundations catalog established: a Layers panel seeded with one row per
//! node kind (labeling the node-kind drawings and their lock/visibility
//! `icon_button` satellites) and a Timeline labeling the transport/agent
//! icons. The toolbar's tool and mode drawings share the same stroke-path
//! machinery but stay crate-private, so they are catalogued live by the
//! Editor toolbar story instead of here.

use crate::*;

use fanta_gpui::prelude::LayersPanelNodeKind;

use super::specimen::{
    framed_panel, specimen_card, specimen_cell, specimen_row, specimen_rows, specimen_story_root,
};

/// Width used by the node-kind specimen panel: the panels' natural width.
const NODE_KIND_SPECIMEN_WIDTH: f32 = 337.;

/// One family group in the icon grid.
pub(crate) struct ControlIconFamily {
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) icons: &'static [(ControlIcon, &'static str)],
}

/// Every [`ControlIcon`] with its canonical label, grouped by the family a
/// host would browse for it. [`control_icon_inventory`] flattens this
/// grouping, so the exhaustive match in this module's tests keeps the grid
/// complete: adding a variant to the library breaks the test until a
/// family lists it.
pub(crate) fn control_icon_families() -> &'static [ControlIconFamily] {
    &[
        ControlIconFamily {
            id: "vector-icons-family-playback",
            label: "Playback & motion",
            icons: &[
                (ControlIcon::Play, "Play"),
                (ControlIcon::Pause, "Pause"),
                (ControlIcon::Loop, "Loop"),
                (ControlIcon::KeyframeDiamond, "KeyframeDiamond"),
                (ControlIcon::Sparkle, "Sparkle"),
            ],
        },
        ControlIconFamily {
            id: "vector-icons-family-variables",
            label: "Variables",
            icons: &[
                (ControlIcon::VariableNumber, "VariableNumber"),
                (ControlIcon::VariableText, "VariableText"),
                (ControlIcon::VariableColor, "VariableColor"),
                (ControlIcon::VariableBoolean, "VariableBoolean"),
            ],
        },
        ControlIconFamily {
            id: "vector-icons-family-node-kinds",
            label: "Node kinds",
            icons: &[
                (ControlIcon::Frame, "Frame"),
                (ControlIcon::Group, "Group"),
                (ControlIcon::Section, "Section"),
                (ControlIcon::Component, "Component"),
                (ControlIcon::Instance, "Instance"),
                (ControlIcon::Text, "Text"),
                (ControlIcon::Image, "Image"),
                (ControlIcon::Video, "Video"),
                (ControlIcon::Mask, "Mask"),
                (ControlIcon::BooleanOperation, "BooleanOperation"),
            ],
        },
        ControlIconFamily {
            id: "vector-icons-family-chrome",
            label: "Chrome & state",
            icons: &[
                (ControlIcon::Help, "Help"),
                (ControlIcon::Close, "Close"),
                (ControlIcon::JumpArrow, "JumpArrow"),
                (ControlIcon::Lock, "Lock"),
                (ControlIcon::Unlock, "Unlock"),
                (ControlIcon::Eye, "Eye"),
                (ControlIcon::EyeClosed, "EyeClosed"),
            ],
        },
    ]
}

/// Every [`ControlIcon`] with its canonical label, flattened from
/// [`control_icon_families`] so the grid and the inventory can never
/// drift apart.
pub(crate) fn control_icon_inventory() -> Vec<(ControlIcon, &'static str)> {
    control_icon_families()
        .iter()
        .flat_map(|family| family.icons.iter().copied())
        .collect()
}

/// One labeled row per [`LayersPanelNodeKind`]. The match in this
/// module's tests keeps the inventory exhaustive: adding a node kind to
/// the library breaks the test until the specimen sheet lists it.
pub(crate) fn node_kind_inventory() -> Vec<LayersPanelItem> {
    use LayersPanelNodeKind as Kind;

    const KINDS: [LayersPanelNodeKind; 22] = [
        Kind::Frame,
        Kind::Group,
        Kind::Section,
        Kind::Component,
        Kind::ComponentSet,
        Kind::Instance,
        Kind::Text,
        Kind::Image,
        Kind::Video,
        Kind::Rectangle,
        Kind::Ellipse,
        Kind::Polygon,
        Kind::Star,
        Kind::Line,
        Kind::Arrow,
        Kind::Vector,
        Kind::BooleanOperation,
        Kind::Slice,
        Kind::Mask,
        Kind::Pen,
        Kind::Pencil,
        Kind::Other,
    ];

    KINDS
        .into_iter()
        .map(|kind| {
            let id = format!(
                "specimen-{}",
                kind.label().to_ascii_lowercase().replace(' ', "-")
            );
            let mut item = LayersPanelItem::new(id, kind.label(), kind);
            // Two rows double as icon_button state specimens: the hidden
            // Video row shows the closed-eye satellite and the locked
            // Rectangle row shows the lock satellite.
            match kind {
                Kind::Video => item.visible = false,
                Kind::Rectangle => item.locked = true,
                _ => {}
            }
            item
        })
        .collect()
}

pub(crate) struct VectorIconsScreen {
    pub(crate) node_kinds: Entity<LayersPanel>,
    pub(crate) transport: Entity<Timeline>,
    pub(crate) node_kind_items: Vec<LayersPanelItem>,
    pub(crate) transport_data: TimelineViewData,
    pub(crate) search_input: Entity<InputState>,
    _search_subscription: Subscription,
    pub(crate) last_action: SharedString,
}

impl VectorIconsScreen {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Storybook>) -> Self {
        let node_kind_items = node_kind_inventory();
        let node_kinds = cx.new(|cx| {
            LayersPanel::new("vector-icons-node-kinds", node_kind_inventory(), window, cx)
        });
        node_kinds.update(cx, |panel, cx| {
            panel.set_selected_node_ids(vec!["specimen-component".into()], cx);
        });
        let transport_data = TimelineViewData {
            current_time_ms: 640,
            ..TimelineViewData::default()
        };
        let transport =
            cx.new(|cx| Timeline::new("vector-icons-transport", transport_data.clone(), cx));
        let search_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Filter by icon or family name"));
        let _search_subscription = cx.subscribe(&search_input, |_, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        });
        Self {
            node_kinds,
            transport,
            node_kind_items,
            transport_data,
            search_input,
            _search_subscription,
            last_action: "Ready — browse the icon grid or toggle a satellite specimen".into(),
        }
    }

    /// The current icon filter, lowercased for the contains matches.
    pub(crate) fn query(&self, cx: &Context<Storybook>) -> String {
        self.search_input
            .read(cx)
            .value()
            .trim()
            .to_ascii_lowercase()
    }

    /// Minimal echo for the node-kind sheet: selection, lock, and
    /// visibility keep working so the icon_button satellites demonstrate
    /// their selected states.
    pub(crate) fn handle_node_kinds_action(
        &mut self,
        panel: Entity<LayersPanel>,
        action: &LayersPanelAction,
        cx: &mut Context<Storybook>,
    ) {
        match action {
            LayersPanelAction::SelectRequested { node_id, .. } => {
                panel.update(cx, |panel, cx| {
                    panel.set_selected_node_ids(vec![node_id.clone()], cx);
                });
                self.last_action = format!("Selected the {node_id} specimen").into();
            }
            LayersPanelAction::VisibilityChanged { node_id, visible } => {
                if let Some(item) = self
                    .node_kind_items
                    .iter_mut()
                    .find(|item| item.id == *node_id)
                {
                    item.visible = *visible;
                }
                panel.update(cx, |panel, cx| {
                    panel.set_nodes(self.node_kind_items.clone(), cx);
                });
                self.last_action = format!("Set {node_id} visibility to {visible}").into();
            }
            LayersPanelAction::LockChanged { node_id, locked } => {
                if let Some(item) = self
                    .node_kind_items
                    .iter_mut()
                    .find(|item| item.id == *node_id)
                {
                    item.locked = *locked;
                }
                panel.update(cx, |panel, cx| {
                    panel.set_nodes(self.node_kind_items.clone(), cx);
                });
                self.last_action = format!("Set {node_id} locked to {locked}").into();
            }
            action => {
                self.last_action =
                    format!("Specimen sheet received {action:?} — display only").into();
            }
        }
        cx.notify();
    }

    pub(crate) fn handle_transport_action(
        &mut self,
        timeline: Entity<Timeline>,
        action: &TimelineAction,
        cx: &mut Context<Storybook>,
    ) {
        match action {
            TimelineAction::PlayStateChangeRequested { playing } => {
                self.transport_data.playing = *playing;
                self.last_action = format!("Transport specimen playing: {playing}").into();
            }
            TimelineAction::LoopChangeRequested { looping } => {
                self.transport_data.looping = *looping;
                self.last_action = format!("Transport specimen looping: {looping}").into();
            }
            TimelineAction::SeekRequested { time_ms } => {
                self.transport_data.current_time_ms =
                    (*time_ms).min(self.transport_data.duration_ms);
                self.last_action = format!("Transport specimen seeked to {time_ms} ms").into();
            }
            TimelineAction::ZoomChangeRequested { zoom } => {
                self.transport_data.zoom = zoom.clamp(0.1, 2.);
                self.last_action = format!("Transport specimen zoom: {zoom:.2}").into();
            }
            action => {
                self.last_action =
                    format!("Transport specimen received {action:?} — display only").into();
            }
        }
        timeline.update(cx, |timeline, cx| {
            timeline.set_view_data(self.transport_data.clone(), cx);
        });
        cx.notify();
    }
}

impl Storybook {
    /// One grid cell: the 16 px working size above the 32 px large
    /// variant, with the variant name labeled under the cell.
    fn render_control_icon_cell(
        &self,
        icon: ControlIcon,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let stack = v_flex()
            .items_center()
            .gap_2()
            .child(render_control_icon(icon, cx.theme().foreground, 16.))
            .child(render_control_icon(icon, cx.theme().foreground, 32.))
            .into_any_element();
        specimen_cell(label, None, stack, cx)
    }

    /// The family-grouped, filterable icon grid.
    fn render_control_icon_grid(&self, cx: &mut Context<Self>) -> AnyElement {
        let query = self.vector_icons_screen.query(cx);
        let total = control_icon_inventory().len();
        let mut visible = 0usize;
        let mut rows = Vec::new();
        for family in control_icon_families() {
            let family_matches = family.label.to_ascii_lowercase().contains(&query);
            let cells: Vec<AnyElement> = family
                .icons
                .iter()
                .copied()
                .filter(|(_, label)| {
                    query.is_empty()
                        || family_matches
                        || label.to_ascii_lowercase().contains(&query)
                })
                .map(|(icon, label)| self.render_control_icon_cell(icon, label, cx))
                .collect();
            if cells.is_empty() {
                continue;
            }
            visible += cells.len();
            rows.push(specimen_row(family.id, family.label, cells, cx));
        }

        let search_row = h_flex()
            .w_full()
            .items_center()
            .justify_between()
            .gap_3()
            .child(
                div().w(px(260.)).max_w_full().child(
                    Input::new(&self.vector_icons_screen.search_input)
                        .prefix(Icon::new(IconName::Search).size_4())
                        .cleanable(true),
                ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("{visible} of {total} drawings")),
            );

        let mut grid = v_flex()
            .id("vector-icons-grid")
            .debug_selector(|| "vector-icons-grid".to_owned())
            .w_full()
            .gap_4()
            .child(search_row);
        if rows.is_empty() {
            grid = grid.child(
                div()
                    .w_full()
                    .h(px(96.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("No drawings match this filter"),
            );
        } else {
            grid = grid.child(specimen_rows(rows));
        }
        grid.into_any_element()
    }

    pub(crate) fn render_vector_icons_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let icon_grid = self.render_control_icon_grid(cx);
        specimen_story_root("storybook-vector-icons")
            .child(specimen_card(
                "vector-icons-control-grid",
                "ControlIcon · every shared stroke-path drawing, by family",
                "The complete semantic icon set, rendered through the public \
                 render_control_icon atom: theme-colored stroke paths on the shared \
                 16-unit grid, so they stay sharp at compact sizes and never depend \
                 on platform fonts. Each cell shows the 16 px working size above \
                 the 32 px large variant; filter by icon or family name. Unicode \
                 glyphs and div art are not icons (§5).",
                icon_grid,
                cx,
            ))
            .child(specimen_card(
                "vector-icons-node-kinds",
                "Node kinds in context · Layers panel and icon_button satellites",
                "One row per LayersPanelNodeKind labels the node-kind drawings as an \
                 organism renders them. The hidden Video and locked Rectangle rows \
                 keep their satellites visible: hover any row for the default \
                 icon_button treatment, Tab to it for the non-shifting focus ring, \
                 and toggle lock or visibility for the selected state.",
                framed_panel(
                    NODE_KIND_SPECIMEN_WIDTH,
                    620.,
                    self.vector_icons_screen
                        .node_kinds
                        .clone()
                        .into_any_element(),
                    cx,
                ),
                cx,
            ))
            .child(specimen_card(
                "vector-icons-transport",
                "Transport and agent icons in context · Timeline",
                "The Timeline organism renders the shared Play/Pause, Loop, keyframe \
                 diamond, Sparkle (agent), and Help icons. Seek, zoom, and the \
                 transport toggles are live.",
                framed_panel(
                    900.,
                    220.,
                    self.vector_icons_screen
                        .transport
                        .clone()
                        .into_any_element(),
                    cx,
                ),
                cx,
            ))
            .child(specimen_card(
                "vector-icons-toolbar-note",
                "Toolbar tool and mode drawings",
                "The Editor toolbar's tool/mode icons share the same stroke-path \
                 machinery but render through the crate-private render_tool_icon, so \
                 they have no standalone grid here. See the Editor toolbar story \
                 under Organisms for every tool drawing live, and the Variables \
                 story for the variable-kind icons in their row gutter.",
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(
                        "Custom icon drawing stays library-internal; hosts pick a \
                         ControlIcon variant instead of assembling paths.",
                    )
                    .into_any_element(),
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn render_vector_icons_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-vector-icons",
            self.render_vector_icons_story(cx),
            self.vector_icons_screen.last_action.clone(),
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_icon_inventory_is_exhaustive_and_uniquely_labeled() {
        let inventory = control_icon_inventory();
        for (icon, label) in &inventory {
            // Exhaustive by construction: a new ControlIcon variant fails to
            // compile here until the inventory above lists it.
            match icon {
                ControlIcon::Play
                | ControlIcon::Pause
                | ControlIcon::KeyframeDiamond
                | ControlIcon::Loop
                | ControlIcon::Sparkle
                | ControlIcon::Help
                | ControlIcon::Close
                | ControlIcon::JumpArrow
                | ControlIcon::VariableNumber
                | ControlIcon::VariableText
                | ControlIcon::VariableColor
                | ControlIcon::VariableBoolean
                | ControlIcon::Component
                | ControlIcon::Instance
                | ControlIcon::Group
                | ControlIcon::Frame
                | ControlIcon::Section
                | ControlIcon::Text
                | ControlIcon::Image
                | ControlIcon::Video
                | ControlIcon::Mask
                | ControlIcon::BooleanOperation
                | ControlIcon::Lock
                | ControlIcon::Unlock
                | ControlIcon::Eye
                | ControlIcon::EyeClosed => {}
            }
            assert!(!label.is_empty());
        }
        let icons: std::collections::HashSet<_> = inventory
            .iter()
            .map(|(icon, _)| format!("{icon:?}"))
            .collect();
        assert_eq!(icons.len(), inventory.len(), "one grid cell per variant");
        let labels: std::collections::HashSet<_> =
            inventory.iter().map(|(_, label)| *label).collect();
        assert_eq!(labels.len(), inventory.len(), "labels stay unique");
        for (icon, label) in &inventory {
            assert_eq!(
                format!("{icon:?}"),
                *label,
                "every grid label matches its variant name"
            );
        }
    }

    #[test]
    fn families_partition_the_inventory_with_unique_ids_and_labels() {
        let families = control_icon_families();
        assert!(
            families.len() >= 4,
            "the grid groups at least four families"
        );
        let ids: std::collections::HashSet<_> = families.iter().map(|family| family.id).collect();
        assert_eq!(ids.len(), families.len(), "family row ids stay unique");
        let labels: std::collections::HashSet<_> =
            families.iter().map(|family| family.label).collect();
        assert_eq!(labels.len(), families.len(), "family labels stay unique");
        for family in families {
            assert!(
                !family.icons.is_empty(),
                "{} must list at least one drawing",
                family.label
            );
        }
        // The inventory is the flattening of the families, so the
        // uniqueness assertions above prove each icon sits in exactly one
        // family.
        assert_eq!(
            control_icon_inventory().len(),
            families
                .iter()
                .map(|family| family.icons.len())
                .sum::<usize>()
        );
    }

    #[test]
    fn node_kind_inventory_is_exhaustive_unique_and_stateful() {
        use LayersPanelNodeKind as Kind;

        let items = node_kind_inventory();
        for item in &items {
            // Exhaustive by construction: a new node kind fails to compile
            // here until the inventory above lists it.
            match item.kind {
                Kind::Frame
                | Kind::Group
                | Kind::Section
                | Kind::Component
                | Kind::ComponentSet
                | Kind::Instance
                | Kind::Text
                | Kind::Image
                | Kind::Video
                | Kind::Rectangle
                | Kind::Ellipse
                | Kind::Polygon
                | Kind::Star
                | Kind::Line
                | Kind::Arrow
                | Kind::Vector
                | Kind::BooleanOperation
                | Kind::Slice
                | Kind::Mask
                | Kind::Pen
                | Kind::Pencil
                | Kind::Other => {}
            }
            assert_eq!(
                item.title.as_ref(),
                item.kind.label(),
                "every specimen row is labeled with its canonical kind name"
            );
        }
        let kinds: std::collections::HashSet<_> = items.iter().map(|item| item.kind).collect();
        assert_eq!(kinds.len(), items.len(), "one specimen row per node kind");
        let ids: std::collections::HashSet<_> = items.iter().map(|item| item.id.clone()).collect();
        assert_eq!(ids.len(), items.len(), "specimen ids stay unique");
        assert_eq!(
            items
                .iter()
                .filter(|item| !item.visible)
                .map(|item| item.kind)
                .collect::<Vec<_>>(),
            vec![Kind::Video],
            "the hidden specimen keeps its closed-eye satellite visible"
        );
        assert_eq!(
            items
                .iter()
                .filter(|item| item.locked)
                .map(|item| item.kind)
                .collect::<Vec<_>>(),
            vec![Kind::Rectangle],
            "the locked specimen keeps its lock satellite visible"
        );
    }
}
