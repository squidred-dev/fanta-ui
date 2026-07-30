use gpui::{ClickEvent, InteractiveElement as _, MouseButton, MouseDownEvent};

use super::*;

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
                    .text_color(
                        if matches!(
                            self.drag.kind,
                            LayersPanelNodeKind::Component
                                | LayersPanelNodeKind::ComponentSet
                                | LayersPanelNodeKind::Instance
                        ) {
                            cx.theme().selection
                        } else {
                            cx.theme().muted_foreground
                        },
                    )
                    .child(layer_kind_icon(self.drag.kind)),
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
            .overflow_scroll()
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
                        Scrollbar::new(&self.scroll_handle)
                            .id(SharedString::from(format!("{}-scrollbar", self.id)))
                            .axis(ScrollbarAxis::Both),
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
            .min_w(layer_title_min_width(&layer.item.title, layer.depth))
            .flex_none()
            .pl(px(layer.depth as f32 * LAYER_INDENT + LAYER_ICON_SLOT * 2.))
            .pr_1()
            .child(
                div()
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
        let menu_item = item.clone();
        let menu_keyboard_item = item.clone();
        let expansion_id = item.id.clone();
        let bounds_id = item.id.clone();
        let bounds_panel = cx.entity();
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

        let row = h_flex()
            .id(SharedString::from(format!("{}-row-{index}", self.id)))
            .debug_selector(move || row_selector)
            .group(group_name.clone())
            .key_context(LAYERS_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .relative()
            .h(px(LAYER_ROW_HEIGHT))
            .w_full()
            .min_w(layer_title_min_width(&item.title, layer.depth))
            .flex_none()
            .pl(px(layer.depth as f32 * LAYER_INDENT + 4.))
            .pr_1()
            .text_xs()
            .cursor_move()
            .border_1()
            .border_color(cx.theme().transparent)
            .hover(|style| style.bg(cx.theme().sidebar_accent.opacity(0.7)))
            .focus(|style| {
                style
                    .bg(cx.theme().sidebar_accent.opacity(0.7))
                    .border_color(cx.theme().selection)
            })
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
        .on_action(cx.listener(move |this, _: &ActivateLayersControl, _, cx| {
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
        .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
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
                            .key_context(LAYERS_CONTROL_KEY_CONTEXT)
                            .tab_index(0)
                            .size_full()
                            .justify_center()
                            .rounded(px(3.))
                            .border_1()
                            .border_color(cx.theme().transparent)
                            .hover(|style| style.bg(cx.theme().sidebar_accent))
                            .focus(|style| {
                                style
                                    .bg(cx.theme().sidebar_accent)
                                    .border_color(cx.theme().selection)
                            })
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|_, _: &MouseDownEvent, _, cx| {
                                    cx.stop_propagation();
                                }),
                            )
                            .on_action(cx.listener({
                                let expansion_id = expansion_id.clone();
                                move |this, _: &ActivateLayersControl, _, cx| {
                                    cx.stop_propagation();
                                    this.toggle_expansion(expansion_id.clone(), cx);
                                }
                            }))
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
                .flex_none()
                .text_center()
                .text_color(
                    if matches!(
                        item.kind,
                        LayersPanelNodeKind::Component
                            | LayersPanelNodeKind::ComponentSet
                            | LayersPanelNodeKind::Instance
                    ) {
                        cx.theme().selection
                    } else {
                        cx.theme().muted_foreground
                    },
                )
                .child(layer_kind_icon(item.kind)),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.))
                .whitespace_nowrap()
                .overflow_hidden()
                .child(item.title.clone()),
        )
        .child(self.render_lock_control(index, item.clone(), group_name.clone(), cx))
        .child(self.render_visibility_control(index, item, group_name, cx))
        .child(
            canvas(
                move |bounds, _, app| {
                    bounds_panel.update(app, |this, _| {
                        this.node_row_bounds.insert(bounds_id.clone(), bounds);
                    });
                },
                |_, _, _, _| {},
            )
            .absolute()
            .top_0()
            .left_0()
            .size_full(),
        )
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
            .key_context(LAYERS_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .size(px(22.))
            .justify_center()
            .rounded(px(3.))
            .border_1()
            .border_color(cx.theme().transparent)
            .cursor_pointer()
            .when(!locked, |control| {
                control
                    .invisible()
                    .group_hover(group_name, |control| control.visible())
            })
            .hover(|style| style.bg(cx.theme().sidebar_accent))
            .focus(|style| {
                style
                    .visible()
                    .bg(cx.theme().sidebar_accent)
                    .border_color(cx.theme().selection)
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|_, _: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                }),
            )
            .on_action(cx.listener({
                let item = item.clone();
                move |this, _: &ActivateLayersControl, _, cx| {
                    cx.stop_propagation();
                    this.request_lock(item.clone(), cx);
                }
            }))
            .on_click(cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                this.request_lock(item.clone(), cx);
            }))
            .child(lock_icon(locked, cx))
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
            .key_context(LAYERS_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .size(px(22.))
            .justify_center()
            .rounded(px(3.))
            .border_1()
            .border_color(cx.theme().transparent)
            .cursor_pointer()
            .when(visible, |control| {
                control
                    .invisible()
                    .group_hover(group_name, |control| control.visible())
            })
            .hover(|style| style.bg(cx.theme().sidebar_accent))
            .focus(|style| {
                style
                    .visible()
                    .bg(cx.theme().sidebar_accent)
                    .border_color(cx.theme().selection)
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|_, _: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                }),
            )
            .on_action(cx.listener({
                let item = item.clone();
                move |this, _: &ActivateLayersControl, _, cx| {
                    cx.stop_propagation();
                    this.request_visibility(item.clone(), cx);
                }
            }))
            .on_click(cx.listener(move |this, _, _, cx| {
                cx.stop_propagation();
                this.request_visibility(item.clone(), cx);
            }))
            .child(
                Icon::new(if visible {
                    IconName::Eye
                } else {
                    IconName::EyeOff
                })
                .xsmall(),
            )
            .into_any_element()
    }
}

fn layer_kind_icon(kind: LayersPanelNodeKind) -> AnyElement {
    let icon = match kind {
        LayersPanelNodeKind::Frame => Some(IconName::Frame),
        LayersPanelNodeKind::Group => Some(IconName::FolderClosed),
        LayersPanelNodeKind::Section => Some(IconName::PanelBottom),
        LayersPanelNodeKind::ComponentSet => Some(IconName::LayoutDashboard),
        LayersPanelNodeKind::Text => Some(IconName::ALargeSmall),
        LayersPanelNodeKind::Image => Some(IconName::GalleryVerticalEnd),
        LayersPanelNodeKind::Star => Some(IconName::Star),
        LayersPanelNodeKind::Line => Some(IconName::Minus),
        LayersPanelNodeKind::Arrow => Some(IconName::ArrowRight),
        LayersPanelNodeKind::Other => Some(IconName::Ellipsis),
        _ => None,
    };

    if let Some(icon) = icon {
        Icon::new(icon).xsmall().into_any_element()
    } else {
        div()
            .size(px(14.))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(11.))
            .font_semibold()
            .child(kind.glyph())
            .into_any_element()
    }
}

fn lock_icon(locked: bool, cx: &App) -> AnyElement {
    let color = if locked {
        cx.theme().sidebar_foreground
    } else {
        cx.theme().muted_foreground
    };
    let shackle = div()
        .absolute()
        .top_0()
        .left(if locked { px(3.) } else { px(5.) })
        .w(px(8.))
        .h(px(8.))
        .rounded(px(4.))
        .border_1()
        .border_color(color);

    div()
        .relative()
        .size(px(14.))
        .child(shackle)
        .child(
            div()
                .absolute()
                .left(px(2.))
                .bottom(px(1.))
                .w(px(10.))
                .h(px(7.))
                .rounded(px(2.))
                .bg(color),
        )
        .into_any_element()
}
