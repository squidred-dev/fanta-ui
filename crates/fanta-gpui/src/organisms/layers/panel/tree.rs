use gpui::{ClickEvent, Hsla, InteractiveElement as _, MouseButton, MouseDownEvent};

use crate::toolbar::{ToolbarTool, render_tool_icon};

use super::*;

const LAYER_KIND_ICON_SIZE: f32 = 12.;

struct LayerDragPreview {
    drag: LayerDrag,
}

impl Render for LayerDragPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .h(px(LAYER_ROW_HEIGHT))
            .w(px(220.))
            .px_2()
            .gap_2()
            .rounded(px(6.))
            .border_1()
            .border_color(cx.theme().selection)
            .bg(cx.theme().popover.opacity(0.96))
            .text_color(cx.theme().popover_foreground)
            .shadow_lg()
            .child(
                div()
                    .size(px(LAYER_ICON_SLOT))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(layer_kind_icon(
                        self.drag.kind,
                        layer_kind_color(self.drag.kind, cx),
                        LAYER_KIND_ICON_SIZE,
                    )),
            )
            .child(
                div()
                    .flex_1()
                    .truncate()
                    .text_xs()
                    .child(self.drag.title.clone()),
            )
    }
}

impl LayersPanel {
    pub(super) fn render_tree(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let editing = self.editing.clone();
        let visible_layers = self.visible_layers();
        let hovered_group_id = self.hovered_group_id(&visible_layers);
        let selected_group_ids = self.selected_group_ids(&visible_layers);
        let rows = visible_layers
            .into_iter()
            .enumerate()
            .map(|(index, layer)| {
                let is_editing = editing
                    .as_ref()
                    .is_some_and(|target| target.node_id == layer.item.id);
                if is_editing {
                    self.render_layer_editor(index, &layer)
                } else {
                    let within_selected_group =
                        Self::layer_is_within_groups(&layer, &selected_group_ids);
                    let within_hovered_group =
                        Self::layer_is_within_groups(&layer, hovered_group_id.as_slice());
                    self.render_layer_row(
                        index,
                        layer,
                        within_selected_group,
                        within_hovered_group,
                        cx,
                    )
                }
            })
            .collect::<Vec<_>>();

        v_flex()
            .id(SharedString::from(format!("{}-tree", self.id)))
            .debug_selector(|| "layers-tree-viewport".to_owned())
            .relative()
            .flex_1()
            .min_h(px(0.))
            .w_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll_handle)
            .children(rows)
            .child(
                div()
                    .absolute()
                    .top_0()
                    .right_0()
                    .bottom_0()
                    .left_0()
                    .child(
                        Scrollbar::vertical(&self.scroll_handle)
                            .id(SharedString::from(format!("{}-scrollbar", self.id))),
                    ),
            )
            .into_any_element()
    }

    pub(super) fn hovered_group_id(&self, visible_layers: &[VisibleLayer]) -> Option<SharedString> {
        let hovered_node = self.hovered_node.as_ref()?;
        let hovered_layer = visible_layers
            .iter()
            .find(|layer| layer.item.id == *hovered_node)?;
        if hovered_layer.has_children {
            Some(hovered_layer.item.id.clone())
        } else {
            hovered_layer.ancestor_ids.last().cloned()
        }
    }

    pub(super) fn selected_group_ids(&self, visible_layers: &[VisibleLayer]) -> Vec<SharedString> {
        visible_layers
            .iter()
            .filter(|layer| layer.has_children && self.selected_node_ids.contains(&layer.item.id))
            .map(|layer| layer.item.id.clone())
            .collect()
    }

    pub(super) fn layer_is_within_groups(layer: &VisibleLayer, group_ids: &[SharedString]) -> bool {
        group_ids
            .iter()
            .any(|group_id| layer.item.id == *group_id || layer.ancestor_ids.contains(group_id))
    }

    fn render_layer_editor(&self, index: usize, layer: &VisibleLayer) -> AnyElement {
        h_flex()
            .id(SharedString::from(format!("{}-editor-{index}", self.id)))
            .debug_selector(|| "layers-row-editor".to_owned())
            .h(px(LAYER_ROW_HEIGHT))
            .w_full()
            .flex_none()
            .pl(px(layer.depth as f32 * LAYER_INDENT + LAYER_ICON_SLOT * 2.))
            .pr_1()
            .child(
                div()
                    .key_context(LAYERS_TEXT_ENTRY_KEY_CONTEXT)
                    .h_full()
                    .flex_1()
                    .min_w(px(0.))
                    .child(Input::new(&self.rename_input).xsmall()),
            )
            .into_any_element()
    }

    fn render_layer_row(
        &mut self,
        index: usize,
        layer: VisibleLayer,
        within_selected_group: bool,
        within_hovered_group: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let item = layer.item;
        let selected = self.selected_node_ids.contains(&item.id);
        let expanded = self.expanded_node_ids.contains(&item.id);
        let group_name: SharedString = format!("{}-row-hover-{index}", self.id).into();
        let row_selector = format!("layers-row-{}", item.id);
        let row_item = item.clone();
        let keyboard_item = item.clone();
        let menu_keyboard_item = item.clone();
        let menu_item = item.clone();
        let visibility_item = item.clone();
        let lock_item = item.clone();
        let expansion_id = item.id.clone();
        let bounds_id = item.id.clone();
        let drop_target_id = item.id.clone();
        let can_drop_target_id = item.id.clone();
        let drop_style_target_id = item.id.clone();
        let drop_kind = item.kind;
        let mut invalid_target_ids = Vec::new();
        collect_layer_ids(&item, &mut invalid_target_ids);
        let drag = LayerDrag {
            node_id: item.id.clone(),
            title: item.title.clone(),
            kind: item.kind,
            invalid_target_ids,
        };
        let drop_background = cx.theme().selection.opacity(0.22);
        let drop_border = cx.theme().selection;
        let row_focus_handle = self
            .row_focus_handles
            .entry(item.id.clone())
            .or_insert_with(|| cx.focus_handle())
            .clone()
            .tab_index(0)
            .tab_stop(true);

        let row = list_row(
            SharedString::from(format!("{}-row-{index}", self.id)),
            px(LAYER_ROW_HEIGHT),
            cx,
        )
        .debug_selector(move || row_selector)
        .group(group_name.clone())
        .track_focus(&row_focus_handle)
        .pl(px(layer.depth as f32 * LAYER_INDENT + 4.))
        .pr_1()
        .text_xs()
        .cursor_move()
        .hover(|style| style.bg(cx.theme().sidebar_accent.opacity(0.7)))
        .when(within_hovered_group, |row| {
            row.bg(cx.theme().sidebar_accent.opacity(0.32))
        })
        .when(within_selected_group, |row| {
            row.bg(cx.theme().selection.opacity(0.16))
                .text_color(cx.theme().sidebar_accent_foreground)
        })
        .when(selected, |row| {
            row.bg(cx.theme().selection.opacity(0.32))
                .text_color(cx.theme().sidebar_accent_foreground)
        })
        .when(!item.visible, |row| row.opacity(0.5));

        let hovered_id = item.id.clone();
        let row = row.on_hover(cx.listener(move |this, hovered, _, cx| {
            let changed = if *hovered {
                if this.hovered_node.as_ref() == Some(&hovered_id) {
                    false
                } else {
                    this.hovered_node = Some(hovered_id.clone());
                    true
                }
            } else if this.hovered_node.as_ref() == Some(&hovered_id) {
                this.hovered_node = None;
                true
            } else {
                false
            };
            if changed {
                cx.notify();
            }
        }));

        row.on_drag(drag, |drag, _, _, cx| {
            cx.new(|_| LayerDragPreview { drag: drag.clone() })
        })
        .can_drop(move |drag, _, _| {
            drag.downcast_ref::<LayerDrag>()
                .is_some_and(|drag| !drag.invalid_target_ids.contains(&can_drop_target_id))
        })
        .drag_over::<LayerDrag>(move |style, drag, _, _| {
            if drag.invalid_target_ids.contains(&drop_style_target_id) {
                style
            } else {
                style.bg(drop_background).border_color(drop_border)
            }
        })
        .on_drop(cx.listener(move |this, drag: &LayerDrag, window, cx| {
            cx.stop_propagation();
            this.request_move(
                drag.node_id.clone(),
                drop_target_id.clone(),
                drop_kind,
                window.mouse_position(),
                cx,
            );
        }))
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                cx.stop_propagation();
                this.open_menu(menu_item.clone(), event.position, window, cx);
            }),
        )
        .on_action(cx.listener(move |this, _: &ActivateControl, _, cx| {
            this.request_selection(
                keyboard_item.id.clone(),
                LayersPanelSelectionMode::Replace,
                cx,
            );
        }))
        .on_action(
            cx.listener(move |this, _: &OpenLayerContextMenu, window, cx| {
                cx.stop_propagation();
                this.open_menu_from_keyboard(menu_keyboard_item.clone(), window, cx);
            }),
        )
        .on_action(cx.listener(move |this, _: &ToggleLayerVisibility, _, cx| {
            cx.stop_propagation();
            this.request_visibility(visibility_item.clone(), cx);
        }))
        .on_action(cx.listener(move |this, _: &ToggleLayerLock, _, cx| {
            cx.stop_propagation();
            this.request_lock(lock_item.clone(), cx);
        }))
        // The pointer path carries modifier and double-click semantics that the
        // shared activation payload cannot, so the row keeps a split
        // pointer/keyboard pair converging on the same intent methods; the
        // keyboard-synthesized click is dropped to keep single activation.
        .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
            if event.is_keyboard() {
                return;
            }
            if event.click_count() >= 2 {
                this.begin_rename(row_item.id.clone(), row_item.title.clone(), window, cx);
                return;
            }
            let modifiers = event.modifiers();
            let mode = if modifiers.shift {
                LayersPanelSelectionMode::Range
            } else if modifiers.secondary() {
                LayersPanelSelectionMode::Toggle
            } else {
                LayersPanelSelectionMode::Replace
            };
            this.request_selection(row_item.id.clone(), mode, cx);
        }))
        .child(
            h_flex()
                .w(px(LAYER_ICON_SLOT))
                .h_full()
                .justify_center()
                .when(layer.has_children, |slot| {
                    slot.child(
                        h_flex()
                            .id(SharedString::from(format!("{}-expand-{index}", self.id)))
                            .debug_selector({
                                let selector = format!("layers-expand-{}", expansion_id);
                                move || selector.clone()
                            })
                            .size_full()
                            .justify_center()
                            .rounded(px(3.))
                            .hover(|style| style.bg(cx.theme().sidebar_accent))
                            .tooltip(move |window, cx| {
                                Tooltip::new(if expanded { "Collapse" } else { "Expand" })
                                    .build(window, cx)
                            })
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|_, _: &MouseDownEvent, _, cx| {
                                    cx.stop_propagation();
                                }),
                            )
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.toggle_expansion(expansion_id.clone(), cx);
                            }))
                            .child(
                                Icon::new(if expanded {
                                    IconName::ChevronDown
                                } else {
                                    IconName::ChevronRight
                                })
                                .xsmall(),
                            ),
                    )
                }),
        )
        .child(
            div()
                .w(px(LAYER_ICON_SLOT))
                .h_full()
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .child(layer_kind_icon(
                    item.kind,
                    layer_kind_color(item.kind, cx),
                    LAYER_KIND_ICON_SIZE,
                )),
        )
        .child(truncating_label(item.title.clone()))
        .child(self.render_lock_control(index, item.clone(), group_name.clone(), cx))
        .child(self.render_visibility_control(index, item, group_name, cx))
        .child(track_bounds(cx.entity(), move |this, bounds| {
            this.node_row_bounds.insert(bounds_id.clone(), bounds);
        }))
        .into_any_element()
    }

    fn render_lock_control(
        &self,
        index: usize,
        item: LayersPanelItem,
        group_name: SharedString,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let locked = item.locked;
        h_flex()
            .id(SharedString::from(format!("{}-lock-{index}", self.id)))
            .debug_selector({
                let selector = format!("layers-lock-{}", item.id);
                move || selector.clone()
            })
            .size(px(22.))
            .justify_center()
            .rounded(px(3.))
            .cursor_pointer()
            .when(!locked, |control| {
                control
                    .invisible()
                    .group_hover(group_name, |control| control.visible())
            })
            .hover(|style| style.bg(cx.theme().sidebar_accent))
            .tooltip(move |window, cx| {
                Tooltip::new(if locked { "Unlock" } else { "Lock" })
                    .action(&ToggleLayerLock, Some(LAYERS_PANEL_KEY_CONTEXT))
                    .build(window, cx)
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|_, _: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                }),
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                this.request_lock(item.clone(), cx);
            }))
            .child(render_control_icon(
                if locked {
                    ControlIcon::Lock
                } else {
                    ControlIcon::Unlock
                },
                if locked {
                    cx.theme().sidebar_foreground
                } else {
                    cx.theme().muted_foreground
                },
                LAYER_KIND_ICON_SIZE,
            ))
            .into_any_element()
    }

    fn render_visibility_control(
        &self,
        index: usize,
        item: LayersPanelItem,
        group_name: SharedString,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let visible = item.visible;
        h_flex()
            .id(SharedString::from(format!(
                "{}-visibility-{index}",
                self.id
            )))
            .debug_selector({
                let selector = format!("layers-visibility-{}", item.id);
                move || selector.clone()
            })
            .size(px(22.))
            .justify_center()
            .rounded(px(3.))
            .cursor_pointer()
            .when(visible, |control| {
                control
                    .invisible()
                    .group_hover(group_name, |control| control.visible())
            })
            .hover(|style| style.bg(cx.theme().sidebar_accent))
            .tooltip(move |window, cx| {
                Tooltip::new(if visible { "Hide" } else { "Show" })
                    .action(&ToggleLayerVisibility, Some(LAYERS_PANEL_KEY_CONTEXT))
                    .build(window, cx)
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|_, _: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                }),
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                this.request_visibility(item.clone(), cx);
            }))
            .child(render_control_icon(
                if visible {
                    ControlIcon::Eye
                } else {
                    ControlIcon::EyeClosed
                },
                cx.theme().sidebar_foreground,
                LAYER_KIND_ICON_SIZE,
            ))
            .into_any_element()
    }
}

fn layer_kind_color(kind: LayersPanelNodeKind, cx: &App) -> Hsla {
    if matches!(
        kind,
        LayersPanelNodeKind::Component
            | LayersPanelNodeKind::ComponentSet
            | LayersPanelNodeKind::Instance
    ) {
        cx.theme().selection
    } else {
        cx.theme().muted_foreground
    }
}

fn layer_kind_icon(kind: LayersPanelNodeKind, color: Hsla, size: f32) -> AnyElement {
    use LayersPanelNodeKind as Kind;

    let control_icon = match kind {
        Kind::Frame => Some(ControlIcon::Frame),
        Kind::Group => Some(ControlIcon::Group),
        Kind::Section => Some(ControlIcon::Section),
        // A component set is drawn with the single-component diamond until a
        // dedicated icon exists.
        Kind::Component | Kind::ComponentSet => Some(ControlIcon::Component),
        Kind::Instance => Some(ControlIcon::Instance),
        Kind::Text => Some(ControlIcon::Text),
        Kind::Image => Some(ControlIcon::Image),
        Kind::Video => Some(ControlIcon::Video),
        Kind::Mask => Some(ControlIcon::Mask),
        Kind::BooleanOperation => Some(ControlIcon::BooleanOperation),
        _ => None,
    };
    if let Some(icon) = control_icon {
        return render_control_icon(icon, color, size);
    }

    let tool = match kind {
        Kind::Rectangle => Some(ToolbarTool::Rectangle),
        Kind::Ellipse => Some(ToolbarTool::Ellipse),
        Kind::Polygon => Some(ToolbarTool::Polygon),
        Kind::Star => Some(ToolbarTool::Star),
        Kind::Line => Some(ToolbarTool::Line),
        Kind::Arrow => Some(ToolbarTool::Arrow),
        Kind::Vector => Some(ToolbarTool::NodeEdit),
        Kind::Slice => Some(ToolbarTool::Slice),
        Kind::Pen => Some(ToolbarTool::Pen),
        Kind::Pencil => Some(ToolbarTool::Pencil),
        _ => None,
    };
    if let Some(tool) = tool {
        return render_tool_icon(tool, color, size);
    }

    div()
        .text_color(color)
        .child(Icon::new(IconName::Ellipsis).xsmall())
        .into_any_element()
}
