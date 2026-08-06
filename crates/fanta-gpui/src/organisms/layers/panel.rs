use std::collections::{HashMap, HashSet};

use gpui::{
    AnyElement, App, AppContext as _, Bounds, Context, Entity, EventEmitter, FocusHandle,
    Focusable, InteractiveElement as _, IntoElement, ParentElement as _, Pixels, Point, Render,
    ScrollHandle, SharedString, StatefulInteractiveElement as _, Styled as _, Subscription, Window,
    div, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _, h_flex,
    input::{Input, InputEvent, InputState, SelectAll},
    scroll::Scrollbar,
    tooltip::Tooltip,
    v_flex,
};

use crate::atoms::{
    ActivateControl, CONTROL_KEY_CONTEXT, ControlExt as _, ControlIcon, icon_button,
    render_control_icon, track_bounds, truncating_label,
};
use crate::molecules::list_row;

use super::{
    CloseLayersOverlay, CollapseLayer, ConfirmLayersTextEntry, ExpandLayer, FocusNextLayer,
    FocusPreviousLayer, LAYERS_PANEL_KEY_CONTEXT, LAYERS_TEXT_ENTRY_KEY_CONTEXT, LayersPanelAction,
    LayersPanelContextAction, LayersPanelDropPosition, LayersPanelItem, LayersPanelNodeKind,
    LayersPanelSelectionMode, OpenLayerContextMenu, ToggleLayerLock, ToggleLayerVisibility,
};

mod events;
mod menu;
#[cfg(test)]
mod tests;
mod tree;

/// Narrowest width the panel lays out without clipping: Figma's narrow
/// left rail. Layer titles truncate ahead of their lock/visibility
/// satellites down to this floor (ARCHITECTURE.md §5).
pub const LAYERS_PANEL_MIN_WIDTH: f32 = 240.;
/// Shortest height the expanded panel stays useful at; the tree scrolls.
pub const LAYERS_PANEL_MIN_HEIGHT: f32 = 400.;

const HEADER_HEIGHT: f32 = 40.;
const LAYER_ROW_HEIGHT: f32 = 28.;
const LAYER_INDENT: f32 = 16.;
const LAYER_ICON_SLOT: f32 = 18.;

#[derive(Clone, Debug, Eq, PartialEq)]
struct LayerEditorTarget {
    node_id: SharedString,
    original_title: SharedString,
}

#[derive(Clone, Debug)]
struct LayerMenuState {
    item: LayersPanelItem,
    anchor: Point<Pixels>,
    return_focus: Option<FocusHandle>,
}

#[derive(Clone, Debug)]
struct LayerDrag {
    node_id: SharedString,
    title: SharedString,
    kind: LayersPanelNodeKind,
    invalid_target_ids: Vec<SharedString>,
}

#[derive(Clone, Debug)]
struct VisibleLayer {
    item: LayersPanelItem,
    depth: usize,
    has_children: bool,
    ancestor_ids: Vec<SharedString>,
}

/// Stateful, host-controlled Figma-like Layers panel.
///
/// The host supplies the node tree, selection, expanded identifiers, visibility,
/// and lock state. The panel owns only interaction continuity such as hover,
/// focus, rename drafts, scrolling, and the open contextual menu.
pub struct LayersPanel {
    id: SharedString,
    focus_handle: FocusHandle,
    menu_focus_handle: FocusHandle,
    nodes: Vec<LayersPanelItem>,
    panel_expanded: bool,
    selected_node_ids: Vec<SharedString>,
    expanded_node_ids: Vec<SharedString>,
    editing: Option<LayerEditorTarget>,
    select_editor_after_render: bool,
    rename_input: Entity<InputState>,
    menu: Option<LayerMenuState>,
    panel_bounds: Option<Bounds<Pixels>>,
    node_row_bounds: HashMap<SharedString, Bounds<Pixels>>,
    row_focus_handles: HashMap<SharedString, FocusHandle>,
    scroll_handle: ScrollHandle,
    menu_scroll_handle: ScrollHandle,
    hovered_node: Option<SharedString>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<LayersPanelAction> for LayersPanel {}

impl LayersPanel {
    /// Creates a Layers panel and expands every populated root container.
    pub fn new(
        id: impl Into<SharedString>,
        nodes: Vec<LayersPanelItem>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let rename_input = cx.new(|cx| InputState::new(window, cx).placeholder("Layer name"));
        let subscription = cx.subscribe_in(
            &rename_input,
            window,
            |this, _, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { .. } => this.commit_rename(true, window, cx),
                InputEvent::Blur => this.commit_rename(false, window, cx),
                InputEvent::Change | InputEvent::Focus => {}
            },
        );
        let expanded_node_ids = nodes
            .iter()
            .filter(|node| !node.children.is_empty())
            .map(|node| node.id.clone())
            .collect();

        Self {
            id: id.into(),
            focus_handle: cx.focus_handle(),
            menu_focus_handle: cx.focus_handle(),
            nodes,
            panel_expanded: true,
            selected_node_ids: Vec::new(),
            expanded_node_ids,
            editing: None,
            select_editor_after_render: false,
            rename_input,
            menu: None,
            panel_bounds: None,
            node_row_bounds: HashMap::new(),
            row_focus_handles: HashMap::new(),
            scroll_handle: ScrollHandle::new(),
            menu_scroll_handle: ScrollHandle::new(),
            hovered_node: None,
            _subscriptions: vec![subscription],
        }
    }

    /// Replaces the host-controlled tree.
    pub fn set_nodes(&mut self, nodes: Vec<LayersPanelItem>, cx: &mut Context<Self>) {
        self.nodes = nodes;
        let mut live_ids = Vec::new();
        for node in &self.nodes {
            collect_layer_ids(node, &mut live_ids);
        }
        let live_ids: HashSet<SharedString> = live_ids.into_iter().collect();
        self.node_row_bounds
            .retain(|node_id, _| live_ids.contains(node_id));
        self.row_focus_handles
            .retain(|node_id, _| live_ids.contains(node_id));
        if self
            .editing
            .as_ref()
            .is_some_and(|target| find_item(&self.nodes, &target.node_id).is_none())
        {
            self.editing = None;
        }
        if self
            .menu
            .as_ref()
            .is_some_and(|menu| find_item(&self.nodes, &menu.item.id).is_none())
        {
            self.menu = None;
        }
        if self
            .hovered_node
            .as_ref()
            .is_some_and(|node_id| find_item(&self.nodes, node_id).is_none())
        {
            self.hovered_node = None;
        }
        cx.notify();
    }

    /// Replaces the host-controlled selection.
    pub fn set_selected_node_ids(
        &mut self,
        selected_node_ids: Vec<SharedString>,
        cx: &mut Context<Self>,
    ) {
        self.selected_node_ids = selected_node_ids;
        cx.notify();
    }

    /// Replaces the host-controlled expansion state of the whole panel.
    pub fn set_expanded(&mut self, expanded: bool, cx: &mut Context<Self>) {
        self.panel_expanded = expanded;
        if !expanded {
            self.menu = None;
            self.hovered_node = None;
        }
        cx.notify();
    }

    /// Replaces the host-controlled expanded node identifiers.
    pub fn set_expanded_node_ids(
        &mut self,
        expanded_node_ids: Vec<SharedString>,
        cx: &mut Context<Self>,
    ) {
        self.expanded_node_ids = expanded_node_ids;
        cx.notify();
    }

    /// Returns the current expanded identifiers, including locally-triggered
    /// presentation updates that a host may accept or replace.
    pub fn expanded_node_ids(&self) -> &[SharedString] {
        &self.expanded_node_ids
    }

    fn visible_layers(&self) -> Vec<VisibleLayer> {
        let mut visible = Vec::new();
        flatten_visible(&self.nodes, 0, &self.expanded_node_ids, &mut visible);
        visible
    }

    fn render_header(&self, cx: &mut Context<Self>) -> AnyElement {
        let expanded = self.panel_expanded;
        h_flex()
            .id(SharedString::from(format!("{}-header", self.id)))
            .debug_selector(|| "layers-header".to_owned())
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h(px(HEADER_HEIGHT))
            .w_full()
            .flex_shrink_0()
            .px_3()
            .justify_between()
            .border_b_1()
            .border_color(if expanded {
                cx.theme().border
            } else {
                cx.theme().transparent
            })
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().sidebar_accent.opacity(0.55)))
            .focus(|style| {
                style
                    .bg(cx.theme().sidebar_accent.opacity(0.55))
                    .border_color(cx.theme().selection)
            })
            .on_activate(cx.listener(|this, _, _, cx| {
                this.toggle_panel_expanded(cx);
            }))
            .child(
                h_flex()
                    .flex_1()
                    .min_w(px(0.))
                    .gap_1()
                    .child(
                        Icon::new(if expanded {
                            IconName::ChevronDown
                        } else {
                            IconName::ChevronRight
                        })
                        .xsmall(),
                    )
                    .child(div().text_sm().font_semibold().child("Layers")),
            )
            .child(
                icon_button(
                    SharedString::from(format!("{}-collapse-all", self.id)),
                    px(26.),
                    px(4.),
                    cx,
                )
                .debug_selector(|| "layers-collapse-all".to_owned())
                .tooltip(|window, cx| Tooltip::new("Collapse all layers").build(window, cx))
                .on_activate(cx.listener(|this, _, _, cx| {
                    cx.stop_propagation();
                    this.collapse_all(cx);
                }))
                .child(Icon::new(IconName::ChevronUp).xsmall()),
            )
            .into_any_element()
    }
}

impl Focusable for LayersPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for LayersPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.select_editor_after_render {
            self.select_editor_after_render = false;
            window.on_next_frame(|window, cx| {
                window.dispatch_action(Box::new(SelectAll), cx);
            });
        }

        v_flex()
            .id(self.id.clone())
            .debug_selector(|| "layers-panel".to_owned())
            .key_context(LAYERS_PANEL_KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_close_overlay))
            // The focused single-line Input already emitted PressEnter (which
            // commits the rename draft) before propagating Enter; consuming
            // the propagated action here keeps the keystroke's "\n" key_char
            // out of the single-line field.
            .on_action(cx.listener(|_, _: &ConfirmLayersTextEntry, _, _| {}))
            .on_action(cx.listener(Self::on_focus_previous_layer))
            .on_action(cx.listener(Self::on_focus_next_layer))
            .on_action(cx.listener(Self::on_collapse_layer))
            .on_action(cx.listener(Self::on_expand_layer))
            .relative()
            .w_full()
            .max_h_full()
            .min_h(px(0.))
            .when(self.panel_expanded, |panel| panel.size_full())
            .when(!self.panel_expanded, |panel| {
                panel.h(px(HEADER_HEIGHT)).flex_none()
            })
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().sidebar_foreground)
            .border_1()
            .border_color(cx.theme().border)
            .child(track_bounds(cx.entity(), |this, bounds| {
                this.panel_bounds = Some(bounds);
            }))
            .child(self.render_header(cx))
            .when(self.panel_expanded, |panel| {
                panel.child(self.render_tree(cx))
            })
            .when(self.panel_expanded && self.menu.is_some(), |panel| {
                panel.child(self.render_context_menu(window, cx))
            })
    }
}

fn flatten_visible(
    nodes: &[LayersPanelItem],
    depth: usize,
    expanded_node_ids: &[SharedString],
    visible: &mut Vec<VisibleLayer>,
) {
    flatten_visible_with_ancestors(nodes, depth, expanded_node_ids, &[], visible);
}

fn flatten_visible_with_ancestors(
    nodes: &[LayersPanelItem],
    depth: usize,
    expanded_node_ids: &[SharedString],
    ancestor_ids: &[SharedString],
    visible: &mut Vec<VisibleLayer>,
) {
    for node in nodes {
        let has_children = !node.children.is_empty();
        visible.push(VisibleLayer {
            item: node.clone(),
            depth,
            has_children,
            ancestor_ids: ancestor_ids.to_vec(),
        });
        if has_children && expanded_node_ids.contains(&node.id) {
            let mut child_ancestors = ancestor_ids.to_vec();
            child_ancestors.push(node.id.clone());
            flatten_visible_with_ancestors(
                &node.children,
                depth + 1,
                expanded_node_ids,
                &child_ancestors,
                visible,
            );
        }
    }
}

fn find_item<'a>(
    nodes: &'a [LayersPanelItem],
    node_id: &SharedString,
) -> Option<&'a LayersPanelItem> {
    for node in nodes {
        if node.id == *node_id {
            return Some(node);
        }
        if let Some(found) = find_item(&node.children, node_id) {
            return Some(found);
        }
    }
    None
}

fn collect_layer_ids(node: &LayersPanelItem, ids: &mut Vec<SharedString>) {
    ids.push(node.id.clone());
    for child in &node.children {
        collect_layer_ids(child, ids);
    }
}
