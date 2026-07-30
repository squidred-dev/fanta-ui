use super::*;

impl LayersPanel {
    pub(super) fn toggle_panel_expanded(&mut self, cx: &mut Context<Self>) {
        self.panel_expanded = !self.panel_expanded;
        if !self.panel_expanded {
            self.menu = None;
            self.hovered_node = None;
        }
        cx.emit(LayersPanelAction::PanelExpansionChanged {
            expanded: self.panel_expanded,
        });
        cx.notify();
    }

    pub(super) fn on_close_overlay(
        &mut self,
        _: &CloseLayersOverlay,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(menu) = self.menu.take() {
            menu.return_focus
                .unwrap_or_else(|| self.focus_handle.clone())
                .focus(window);
            cx.stop_propagation();
            cx.notify();
        } else if self.editing.take().is_some() {
            self.focus_panel_after_action(window, cx);
            cx.stop_propagation();
            cx.notify();
        }
    }

    pub(super) fn collapse_all(&mut self, cx: &mut Context<Self>) {
        self.menu = None;
        self.expanded_node_ids.clear();
        cx.emit(LayersPanelAction::CollapseAllRequested);
        cx.notify();
    }

    pub(super) fn toggle_expansion(&mut self, node_id: SharedString, cx: &mut Context<Self>) {
        let expanded = if let Some(index) = self
            .expanded_node_ids
            .iter()
            .position(|expanded| *expanded == node_id)
        {
            self.expanded_node_ids.remove(index);
            false
        } else {
            self.expanded_node_ids.push(node_id.clone());
            true
        };
        self.menu = None;
        cx.emit(LayersPanelAction::ExpansionChanged { node_id, expanded });
        cx.notify();
    }

    pub(super) fn request_selection(
        &mut self,
        node_id: SharedString,
        mode: LayersPanelSelectionMode,
        cx: &mut Context<Self>,
    ) {
        self.menu = None;
        cx.emit(LayersPanelAction::SelectRequested { node_id, mode });
        cx.notify();
    }

    pub(super) fn request_move(
        &mut self,
        node_id: SharedString,
        target_node_id: SharedString,
        target_kind: LayersPanelNodeKind,
        pointer: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        let Some(target_bounds) = self.node_row_bounds.get(&target_node_id).copied() else {
            return;
        };
        let position = layer_drop_position(target_kind, target_bounds, pointer);
        self.menu = None;
        cx.emit(LayersPanelAction::MoveRequested {
            node_id,
            target_node_id,
            position,
        });
        cx.notify();
    }

    pub(super) fn begin_rename(
        &mut self,
        node_id: SharedString,
        original_title: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.menu = None;
        self.editing = Some(LayerEditorTarget {
            node_id,
            original_title: original_title.clone(),
        });
        self.rename_input.update(cx, |input, cx| {
            input.set_value(original_title, window, cx);
            input.focus(window, cx);
        });
        self.select_editor_after_render = true;
        cx.notify();
    }

    pub(super) fn commit_rename(
        &mut self,
        restore_focus: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(target) = self.editing.take() else {
            return;
        };
        let title = self.rename_input.read(cx).value();
        let title = title.trim();
        if !title.is_empty() && title != target.original_title.as_ref() {
            cx.emit(LayersPanelAction::RenameRequested {
                node_id: target.node_id,
                title: title.to_owned().into(),
            });
        }
        if restore_focus {
            self.focus_panel_after_action(window, cx);
        }
        cx.notify();
    }

    pub(super) fn open_menu(
        &mut self,
        item: LayersPanelItem,
        anchor: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editing = None;
        let return_focus = window.focused(cx);
        if !self.selected_node_ids.contains(&item.id) {
            cx.emit(LayersPanelAction::SelectRequested {
                node_id: item.id.clone(),
                mode: LayersPanelSelectionMode::Replace,
            });
        }
        self.menu = Some(LayerMenuState {
            item,
            anchor,
            return_focus,
        });
        self.menu_scroll_handle.set_offset(Point::default());
        let focus_handle = self.menu_focus_handle.clone();
        window.defer(cx, move |window, _| {
            focus_handle.focus(window);
        });
        cx.notify();
    }

    pub(super) fn open_menu_from_keyboard(
        &mut self,
        item: LayersPanelItem,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(bounds) = self.node_row_bounds.get(&item.id).copied() else {
            return;
        };
        self.open_menu(
            item,
            bounds.bottom_left() + gpui::point(px(0.), px(4.)),
            window,
            cx,
        );
    }

    pub(super) fn activate_menu_action(
        &mut self,
        action: LayersPanelContextAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(menu) = self.menu.take() else {
            return;
        };
        match action {
            LayersPanelContextAction::Rename => {
                self.begin_rename(menu.item.id, menu.item.title, window, cx);
            }
            LayersPanelContextAction::ShowHide => {
                cx.emit(LayersPanelAction::VisibilityChanged {
                    node_id: menu.item.id,
                    visible: !menu.item.visible,
                });
                self.focus_panel_after_action(window, cx);
            }
            LayersPanelContextAction::LockUnlock => {
                cx.emit(LayersPanelAction::LockChanged {
                    node_id: menu.item.id,
                    locked: !menu.item.locked,
                });
                self.focus_panel_after_action(window, cx);
            }
            action => {
                cx.emit(LayersPanelAction::ContextActionRequested {
                    node_id: menu.item.id,
                    action,
                });
                self.focus_panel_after_action(window, cx);
            }
        }
        cx.notify();
    }

    pub(super) fn request_visibility(&mut self, item: LayersPanelItem, cx: &mut Context<Self>) {
        cx.emit(LayersPanelAction::VisibilityChanged {
            node_id: item.id,
            visible: !item.visible,
        });
        cx.notify();
    }

    pub(super) fn request_lock(&mut self, item: LayersPanelItem, cx: &mut Context<Self>) {
        cx.emit(LayersPanelAction::LockChanged {
            node_id: item.id,
            locked: !item.locked,
        });
        cx.notify();
    }

    fn focus_panel_after_action(&self, window: &mut Window, cx: &mut Context<Self>) {
        let focus_handle = self.focus_handle.clone();
        window.defer(cx, move |window, _| {
            focus_handle.focus(window);
        });
    }
}

pub(super) fn layer_drop_position(
    target_kind: LayersPanelNodeKind,
    bounds: Bounds<Pixels>,
    pointer: Point<Pixels>,
) -> LayersPanelDropPosition {
    let upper_edge = bounds.top() + bounds.size.height * 0.3;
    let lower_edge = bounds.bottom() - bounds.size.height * 0.3;

    if pointer.y < upper_edge {
        LayersPanelDropPosition::Before
    } else if pointer.y > lower_edge {
        LayersPanelDropPosition::After
    } else if target_kind.is_container() {
        LayersPanelDropPosition::Inside
    } else if pointer.y < bounds.center().y {
        LayersPanelDropPosition::Before
    } else {
        LayersPanelDropPosition::After
    }
}
