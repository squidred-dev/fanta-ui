//! The Layers story: mock layer tree, fixtures, reducer, and knobs.

use crate::*;

use super::harness;
use super::knobs::{self, KnobOption};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LayersNamedState {
    Default,
    DeepNesting,
    AllLocked,
}

impl LayersNamedState {
    pub(crate) const ALL: [Self; 3] = [Self::Default, Self::DeepNesting, Self::AllLocked];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::DeepNesting => "Deep nesting",
            Self::AllLocked => "All locked",
        }
    }
}

pub(crate) fn seed_layers() -> Vec<LayersPanelItem> {
    use fanta_gpui::prelude::LayersPanelNodeKind as Kind;

    vec![
        LayersPanelItem::new("checkout", "Checkout", Kind::Frame).children(vec![
            LayersPanelItem::new("header", "Header", Kind::Group).children(vec![
                LayersPanelItem::new("brand-image", "Brand mark", Kind::Image),
                LayersPanelItem::new("hero-title", "Checkout title", Kind::Text),
                LayersPanelItem::new("divider", "Divider", Kind::Line),
                LayersPanelItem::new("flow-arrow", "Flow arrow", Kind::Arrow),
            ]),
            LayersPanelItem::new("controls", "Controls", Kind::ComponentSet).children(vec![
                LayersPanelItem::new("button-component", "Primary button", Kind::Component)
                    .children(vec![
                        LayersPanelItem::new("button-label", "Label", Kind::Text),
                        LayersPanelItem::new("button-icon", "Icon", Kind::Instance),
                    ]),
            ]),
            LayersPanelItem::new("hero-video", "Product preview", Kind::Video).visible(false),
            LayersPanelItem::new("background", "Background", Kind::Rectangle).locked(true),
            LayersPanelItem::new("avatar", "Avatar", Kind::Ellipse),
            LayersPanelItem::new("badge", "Badge", Kind::Polygon),
            LayersPanelItem::new("favorite", "Favorite", Kind::Star),
            LayersPanelItem::new("vector-art", "Vector artwork", Kind::BooleanOperation).children(
                vec![
                    LayersPanelItem::new("pen-path", "Pen path", Kind::Pen),
                    LayersPanelItem::new("pencil-path", "Pencil path", Kind::Pencil),
                    LayersPanelItem::new("vector-path", "Vector path", Kind::Vector),
                ],
            ),
            LayersPanelItem::new("mask", "Avatar mask", Kind::Mask),
        ]),
        LayersPanelItem::new("variants", "Variants", Kind::Section),
        LayersPanelItem::new("export-slice", "Export slice", Kind::Slice),
        LayersPanelItem::new("other", "Imported node", Kind::Other),
    ]
}

pub(crate) fn deep_nesting_layers() -> Vec<LayersPanelItem> {
    use fanta_gpui::prelude::LayersPanelNodeKind as Kind;

    let mut node = LayersPanelItem::new("depth-8", "Depth 8", Kind::Text);
    for depth in (1..8).rev() {
        node = LayersPanelItem::new(
            format!("depth-{depth}"),
            format!("Depth {depth}"),
            if depth % 2 == 0 {
                Kind::Group
            } else {
                Kind::Frame
            },
        )
        .children(vec![node]);
    }
    vec![node]
}

pub(crate) fn all_locked_layers() -> Vec<LayersPanelItem> {
    fn lock_all(nodes: &mut [LayersPanelItem]) {
        for node in nodes {
            node.locked = true;
            lock_all(&mut node.children);
        }
    }

    let mut layers = seed_layers();
    lock_all(&mut layers);
    layers
}

pub(crate) struct LayersScreen {
    pub(crate) panel: Entity<LayersPanel>,
    pub(crate) layers: Vec<LayersPanelItem>,
    pub(crate) selected: Vec<SharedString>,
    pub(crate) expanded: Vec<SharedString>,
    pub(crate) selection_anchor: Option<SharedString>,
    pub(crate) last_action: SharedString,
    pub(crate) named_state: LayersNamedState,
}

impl LayersScreen {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Storybook>) -> Self {
        let layers = seed_layers();
        let selected: Vec<SharedString> = vec!["hero-title".into()];
        let expanded: Vec<SharedString> = vec![
            "checkout".into(),
            "header".into(),
            "controls".into(),
            "button-component".into(),
            "vector-art".into(),
        ];
        let panel = cx.new(|cx| LayersPanel::new("storybook-layers", layers.clone(), window, cx));
        panel.update(cx, |panel, cx| {
            panel.set_selected_node_ids(selected.clone(), cx);
            panel.set_expanded_node_ids(expanded.clone(), cx);
        });
        Self {
            panel,
            layers,
            selected,
            expanded,
            selection_anchor: Some("hero-title".into()),
            last_action: "Ready — expand, select, rename, or secondary-click a layer".into(),
            named_state: LayersNamedState::Default,
        }
    }

    fn fixture(
        state: LayersNamedState,
    ) -> (Vec<LayersPanelItem>, Vec<SharedString>, Vec<SharedString>) {
        match state {
            LayersNamedState::Default => (
                seed_layers(),
                vec!["hero-title".into()],
                vec![
                    "checkout".into(),
                    "header".into(),
                    "controls".into(),
                    "button-component".into(),
                    "vector-art".into(),
                ],
            ),
            LayersNamedState::DeepNesting => {
                let layers = deep_nesting_layers();
                let expanded = (1..8)
                    .map(|depth| SharedString::from(format!("depth-{depth}")))
                    .collect();
                (layers, vec!["depth-8".into()], expanded)
            }
            LayersNamedState::AllLocked => (
                all_locked_layers(),
                Vec::new(),
                vec!["checkout".into(), "header".into()],
            ),
        }
    }

    pub(crate) fn apply_named_state(
        &mut self,
        state: LayersNamedState,
        cx: &mut Context<Storybook>,
    ) {
        self.named_state = state;
        let (layers, selected, expanded) = Self::fixture(state);
        self.layers = layers;
        self.selected = selected;
        self.expanded = expanded;
        self.selection_anchor = self.selected.first().cloned();
        let (layers, selected, expanded) = (
            self.layers.clone(),
            self.selected.clone(),
            self.expanded.clone(),
        );
        self.panel.update(cx, |panel, cx| {
            panel.set_nodes(layers, cx);
            panel.set_selected_node_ids(selected, cx);
            panel.set_expanded_node_ids(expanded, cx);
        });
        self.last_action = format!("Story applied the {} Layers state", state.label()).into();
        cx.notify();
    }

    pub(crate) fn apply_layer_selection(
        &mut self,
        node_id: &SharedString,
        mode: LayersPanelSelectionMode,
    ) {
        match mode {
            LayersPanelSelectionMode::Replace => {
                self.selected = vec![node_id.clone()];
                self.selection_anchor = Some(node_id.clone());
            }
            LayersPanelSelectionMode::Toggle => {
                if let Some(index) = self
                    .selected
                    .iter()
                    .position(|selected| selected == node_id)
                {
                    self.selected.remove(index);
                } else {
                    self.selected.push(node_id.clone());
                }
                self.selection_anchor = Some(node_id.clone());
            }
            LayersPanelSelectionMode::Range => {
                let flattened = flatten_layer_ids(&self.layers);
                let anchor = self
                    .selection_anchor
                    .as_ref()
                    .and_then(|anchor| flattened.iter().position(|id| id == anchor));
                let target = flattened.iter().position(|id| id == node_id);
                if let (Some(anchor), Some(target)) = (anchor, target) {
                    let start = anchor.min(target);
                    let end = anchor.max(target);
                    self.selected = flattened[start..=end].to_vec();
                } else {
                    self.selected = vec![node_id.clone()];
                    self.selection_anchor = Some(node_id.clone());
                }
            }
        }
    }

    pub(crate) fn handle_action(
        &mut self,
        panel: Entity<LayersPanel>,
        action: &LayersPanelAction,
        cx: &mut Context<Storybook>,
    ) {
        match action {
            LayersPanelAction::PanelExpansionChanged { expanded } => {
                self.last_action = if *expanded {
                    "Host observed: Layers panel expanded".into()
                } else {
                    "Host observed: Layers panel collapsed".into()
                };
            }
            LayersPanelAction::SelectRequested { node_id, mode } => {
                self.apply_layer_selection(node_id, *mode);
                panel.update(cx, |panel, cx| {
                    panel.set_selected_node_ids(self.selected.clone(), cx);
                });
                self.last_action = format!("Selected {node_id} with {mode:?}").into();
            }
            LayersPanelAction::ExpansionChanged { node_id, expanded } => {
                if *expanded {
                    if !self.expanded.contains(node_id) {
                        self.expanded.push(node_id.clone());
                    }
                } else {
                    self.expanded.retain(|id| id != node_id);
                }
                panel.update(cx, |panel, cx| {
                    panel.set_expanded_node_ids(self.expanded.clone(), cx);
                });
                self.last_action = format!(
                    "{} {node_id}",
                    if *expanded { "Expanded" } else { "Collapsed" }
                )
                .into();
            }
            LayersPanelAction::CollapseAllRequested => {
                self.expanded.clear();
                panel.update(cx, |panel, cx| {
                    panel.set_expanded_node_ids(Vec::new(), cx);
                });
                self.last_action = "Collapsed every populated layer".into();
            }
            LayersPanelAction::RenameRequested { node_id, title } => {
                if let Some(node) = find_layer_mut(&mut self.layers, node_id) {
                    node.title = title.clone();
                }
                panel.update(cx, |panel, cx| {
                    panel.set_nodes(self.layers.clone(), cx);
                });
                self.last_action = format!("Renamed {node_id} to {title}").into();
            }
            LayersPanelAction::VisibilityChanged { node_id, visible } => {
                if let Some(node) = find_layer_mut(&mut self.layers, node_id) {
                    node.visible = *visible;
                }
                panel.update(cx, |panel, cx| {
                    panel.set_nodes(self.layers.clone(), cx);
                });
                self.last_action = format!("Set {node_id} visibility to {visible}").into();
            }
            LayersPanelAction::LockChanged { node_id, locked } => {
                if let Some(node) = find_layer_mut(&mut self.layers, node_id) {
                    node.locked = *locked;
                }
                panel.update(cx, |panel, cx| {
                    panel.set_nodes(self.layers.clone(), cx);
                });
                self.last_action = format!("Set {node_id} locked to {locked}").into();
            }
            LayersPanelAction::MoveRequested {
                node_id,
                target_node_id,
                position,
            } => {
                let original_layers = self.layers.clone();
                if let Some(moved) = take_layer(&mut self.layers, node_id)
                    && insert_layer(&mut self.layers, target_node_id, moved, *position)
                {
                    if *position == LayersPanelDropPosition::Inside
                        && !self.expanded.contains(target_node_id)
                    {
                        self.expanded.push(target_node_id.clone());
                    }
                    panel.update(cx, |panel, cx| {
                        panel.set_nodes(self.layers.clone(), cx);
                        panel.set_expanded_node_ids(self.expanded.clone(), cx);
                    });
                    self.last_action =
                        format!("Moved {node_id} {position:?} {target_node_id}").into();
                } else {
                    self.layers = original_layers;
                }
            }
            LayersPanelAction::ContextActionRequested { node_id, action } => {
                self.last_action = format!("Host received {} for {node_id}", action.label()).into();
            }
        }
        cx.notify();
    }
}

fn find_layer_mut<'a>(
    nodes: &'a mut [LayersPanelItem],
    node_id: &SharedString,
) -> Option<&'a mut LayersPanelItem> {
    for node in nodes {
        if node.id == *node_id {
            return Some(node);
        }
        if let Some(found) = find_layer_mut(&mut node.children, node_id) {
            return Some(found);
        }
    }
    None
}

fn take_layer(nodes: &mut Vec<LayersPanelItem>, node_id: &SharedString) -> Option<LayersPanelItem> {
    if let Some(index) = nodes.iter().position(|node| node.id == *node_id) {
        return Some(nodes.remove(index));
    }
    for node in nodes {
        if let Some(found) = take_layer(&mut node.children, node_id) {
            return Some(found);
        }
    }
    None
}

fn insert_layer(
    nodes: &mut Vec<LayersPanelItem>,
    target_node_id: &SharedString,
    item: LayersPanelItem,
    position: LayersPanelDropPosition,
) -> bool {
    if position == LayersPanelDropPosition::Inside {
        for node in nodes {
            if node.id == *target_node_id {
                node.children.push(item);
                return true;
            }
            if insert_layer(&mut node.children, target_node_id, item.clone(), position) {
                return true;
            }
        }
        return false;
    }

    if let Some(index) = nodes.iter().position(|node| node.id == *target_node_id) {
        let insertion_index = match position {
            LayersPanelDropPosition::Before => index,
            LayersPanelDropPosition::After => index + 1,
            LayersPanelDropPosition::Inside => unreachable!(),
        };
        nodes.insert(insertion_index, item);
        return true;
    }
    for node in nodes {
        if insert_layer(&mut node.children, target_node_id, item.clone(), position) {
            return true;
        }
    }
    false
}

fn flatten_layer_ids(nodes: &[LayersPanelItem]) -> Vec<SharedString> {
    fn collect(nodes: &[LayersPanelItem], ids: &mut Vec<SharedString>) {
        for node in nodes {
            ids.push(node.id.clone());
            collect(&node.children, ids);
        }
    }

    let mut ids = Vec::new();
    collect(nodes, &mut ids);
    ids
}

impl Storybook {
    pub(crate) fn render_layers_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_story_shell(
            "storybook-reference-layers",
            harness::ReferenceStoryCopy {
                eyebrow: "LAYERS PANEL",
                title: "Layers panel",
                description: "Collapse the panel, drag rows to reorder or reparent them, \
                              expand the tree, use Command/Shift selection, double-click to \
                              rename, toggle lock and visibility, and secondary-click every \
                              node type to compare its menu.",
                adapter_description: "The story owns the mock layer tree, selection, \
                                      expansion, lock, and visibility. LayersPanel emits \
                                      typed intents only; this adapter applies them and \
                                      calls the setters with updated host data.",
            },
            self.layers_screen.last_action.clone(),
            self.layers_screen.panel.clone().into_any_element(),
            cx,
        )
    }

    pub(crate) fn render_layers_knobs(&self, cx: &mut Context<Self>) -> AnyElement {
        knobs::knobs_panel(
            "layers-story-knobs",
            vec![knobs::enum_knob_row(
                "layers-knob-state",
                "NAMED STATE",
                LayersNamedState::ALL.map(|state| KnobOption::new(state, state.label())),
                self.layers_screen.named_state,
                |this, state, _, cx| this.layers_screen.apply_named_state(state, cx),
                cx,
            )],
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layers_named_states_return_reseedable_fixtures() {
        fn depth(nodes: &[LayersPanelItem]) -> usize {
            nodes
                .iter()
                .map(|node| 1 + depth(&node.children))
                .max()
                .unwrap_or(0)
        }

        fn all_locked(nodes: &[LayersPanelItem]) -> bool {
            nodes
                .iter()
                .all(|node| node.locked && all_locked(&node.children))
        }

        let (default_layers, selected, expanded) = LayersScreen::fixture(LayersNamedState::Default);
        assert!(!default_layers.is_empty());
        assert!(!selected.is_empty());
        assert!(!expanded.is_empty());
        assert!(!all_locked(&default_layers));

        let (deep, _, expanded) = LayersScreen::fixture(LayersNamedState::DeepNesting);
        assert_eq!(depth(&deep), 8);
        assert_eq!(expanded.len(), 7, "every populated depth starts expanded");

        let (locked, selected, _) = LayersScreen::fixture(LayersNamedState::AllLocked);
        assert!(all_locked(&locked));
        assert!(selected.is_empty());
    }
}
