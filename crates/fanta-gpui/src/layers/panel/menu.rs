use gpui::{InteractiveElement as _, MouseDownEvent, deferred};

use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct MenuEntry {
    pub(super) action: LayersPanelContextAction,
}

impl MenuEntry {
    const fn new(action: LayersPanelContextAction) -> Self {
        Self { action }
    }
}

impl LayersPanel {
    pub(super) fn render_context_menu(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let (Some(panel_bounds), Some(menu)) = (self.panel_bounds, self.menu.as_ref()) else {
            return div().into_any_element();
        };
        let origin = menu.anchor - panel_bounds.origin;
        let sections = menu_sections_for(menu.item.kind);
        let row_count = sections.iter().map(Vec::len).sum::<usize>();
        let separator_count = sections.len().saturating_sub(1);
        let estimated_height = px(row_count as f32 * 28. + separator_count as f32 * 9. + 16.);
        let maximum_height = px(620.);
        let menu_height = if estimated_height > maximum_height {
            maximum_height
        } else {
            estimated_height
        };
        let menu_height = if menu_height > panel_bounds.size.height {
            panel_bounds.size.height
        } else {
            menu_height
        };
        let latest_top = panel_bounds.size.height - menu_height;
        let menu_top = if origin.y > latest_top {
            latest_top
        } else if origin.y < px(0.) {
            px(0.)
        } else {
            origin.y
        };
        let mut first = true;
        let mut content = v_flex();

        for (section_index, section) in sections.into_iter().enumerate() {
            if section_index > 0 {
                content = content.child(
                    div()
                        .h(px(1.))
                        .w_full()
                        .my_1()
                        .flex_none()
                        .bg(cx.theme().border),
                );
            }
            for entry in section {
                content = content.child(self.render_menu_item(entry, first, cx));
                first = false;
            }
        }

        deferred(
            content
                .id(SharedString::from(format!("{}-context-menu", self.id)))
                .absolute()
                .left(origin.x)
                .top(menu_top)
                .debug_selector(|| "layers-context-menu".to_owned())
                .block_mouse_except_scroll()
                .w(px(224.))
                .max_h(menu_height)
                .overflow_scroll()
                .track_scroll(&self.menu_scroll_handle)
                .py_2()
                .rounded(px(10.))
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().popover)
                .shadow_lg()
                .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
                    if this.menu.take().is_some() {
                        cx.notify();
                    }
                }))
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .right_0()
                        .bottom_0()
                        .debug_selector(|| "layers-context-menu-scrollbar".to_owned())
                        .child(Scrollbar::vertical(&self.menu_scroll_handle).id(
                            SharedString::from(format!("{}-context-menu-scrollbar", self.id)),
                        )),
                ),
        )
        .with_priority(4)
        .into_any_element()
    }

    fn render_menu_item(
        &mut self,
        entry: MenuEntry,
        first: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let action = entry.action;
        let selector = format!("layers-menu-{}", action.selector_slug());
        h_flex()
            .id(SharedString::from(format!(
                "{}-menu-{}",
                self.id,
                action.selector_slug()
            )))
            .debug_selector(move || selector)
            .key_context(LAYERS_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .when(first, |item| {
                item.track_focus(&self.menu_focus_handle.clone().tab_index(0).tab_stop(true))
            })
            .h(px(28.))
            .flex_none()
            .mx_2()
            .px_2()
            .gap_2()
            .rounded(px(4.))
            .text_xs()
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().accent))
            .focus(|style| {
                style
                    .bg(cx.theme().accent)
                    .border_1()
                    .border_color(cx.theme().selection)
            })
            .on_action(
                cx.listener(move |this, _: &ActivateLayersControl, window, cx| {
                    cx.stop_propagation();
                    this.activate_menu_action(action, window, cx);
                }),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                this.activate_menu_action(action, window, cx);
            }))
            .child(div().flex_1().child(action.label()))
            .when_some(action.shortcut(), |row, shortcut| {
                row.child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child(shortcut),
                )
            })
            .when(action.has_submenu(), |row| {
                row.child(Icon::new(IconName::ChevronRight).xsmall())
            })
            .into_any_element()
    }
}

pub(super) fn menu_sections_for(kind: LayersPanelNodeKind) -> Vec<Vec<MenuEntry>> {
    use LayersPanelContextAction as Action;

    let mut sections = vec![
        entries(&[
            Action::Copy,
            Action::PasteToReplace,
            Action::CopyPasteAs,
            Action::SendToFigmaMake,
            Action::FindSimilarDesigns,
            Action::AddMotion,
        ]),
        entries(&[Action::MoveToPage, Action::BringToFront, Action::SendToBack]),
    ];

    let structural = match kind {
        LayersPanelNodeKind::Section => entries(&[
            Action::ConvertToFrame,
            Action::Rename,
            Action::RenameLayers,
            Action::SetAsThumbnail,
        ]),
        LayersPanelNodeKind::Frame => entries(&[
            Action::ConvertToSection,
            Action::GroupSelection,
            Action::FrameSelection,
            Action::RemoveFrame,
            Action::Rename,
            Action::RenameLayers,
            Action::Flatten,
            Action::OutlineStroke,
            Action::UseAsMask,
            Action::SetAsThumbnail,
        ]),
        LayersPanelNodeKind::Group => entries(&[
            Action::ConvertToFrame,
            Action::ConvertToSection,
            Action::GroupSelection,
            Action::FrameSelection,
            Action::Ungroup,
            Action::Rename,
            Action::RenameLayers,
            Action::Flatten,
            Action::OutlineStroke,
            Action::UseAsMask,
            Action::SetAsThumbnail,
        ]),
        LayersPanelNodeKind::Component | LayersPanelNodeKind::ComponentSet => entries(&[
            Action::GroupSelection,
            Action::FrameSelection,
            Action::Rename,
            Action::RenameLayers,
            Action::Flatten,
            Action::OutlineStroke,
            Action::UseAsMask,
            Action::SetAsThumbnail,
        ]),
        LayersPanelNodeKind::Instance => entries(&[
            Action::GroupSelection,
            Action::FrameSelection,
            Action::Rename,
            Action::RenameLayers,
            Action::Flatten,
            Action::GoToMainComponent,
            Action::DetachInstance,
            Action::ResetInstance,
        ]),
        LayersPanelNodeKind::Text => entries(&[
            Action::EditText,
            Action::GroupSelection,
            Action::FrameSelection,
            Action::Rename,
            Action::RenameLayers,
            Action::Flatten,
            Action::OutlineStroke,
            Action::UseAsMask,
        ]),
        LayersPanelNodeKind::Image => entries(&[
            Action::CropImage,
            Action::ReplaceMedia,
            Action::GroupSelection,
            Action::FrameSelection,
            Action::Rename,
            Action::RenameLayers,
            Action::Flatten,
            Action::UseAsMask,
        ]),
        LayersPanelNodeKind::Video => entries(&[
            Action::ReplaceMedia,
            Action::GroupSelection,
            Action::FrameSelection,
            Action::Rename,
            Action::RenameLayers,
            Action::UseAsMask,
        ]),
        LayersPanelNodeKind::Slice => entries(&[Action::Rename, Action::RenameLayers]),
        LayersPanelNodeKind::Mask => entries(&[
            Action::GroupSelection,
            Action::FrameSelection,
            Action::Ungroup,
            Action::Rename,
            Action::RenameLayers,
            Action::Flatten,
        ]),
        LayersPanelNodeKind::Rectangle
        | LayersPanelNodeKind::Ellipse
        | LayersPanelNodeKind::Polygon
        | LayersPanelNodeKind::Star
        | LayersPanelNodeKind::Line
        | LayersPanelNodeKind::Arrow
        | LayersPanelNodeKind::Vector
        | LayersPanelNodeKind::BooleanOperation
        | LayersPanelNodeKind::Pen
        | LayersPanelNodeKind::Pencil
        | LayersPanelNodeKind::Other => entries(&[
            Action::GroupSelection,
            Action::FrameSelection,
            Action::Rename,
            Action::RenameLayers,
            Action::Flatten,
            Action::OutlineStroke,
            Action::UseAsMask,
        ]),
    };
    sections.push(structural);

    let mut layout = Vec::new();
    if kind.supports_auto_layout() {
        layout.push(MenuEntry::new(Action::AddAutoLayout));
        if kind.is_container() {
            layout.push(MenuEntry::new(Action::MoreLayoutOptions));
        }
    }
    if kind.can_create_component() {
        layout.push(MenuEntry::new(Action::CreateComponent));
    }
    if !layout.is_empty() {
        sections.push(layout);
    }

    sections.push(entries(&[Action::Plugins, Action::Widgets]));
    sections.push(entries(&[Action::ShowHide, Action::LockUnlock]));
    if kind.can_flip() {
        sections.push(entries(&[Action::FlipHorizontal, Action::FlipVertical]));
    }
    sections
}

fn entries(actions: &[LayersPanelContextAction]) -> Vec<MenuEntry> {
    actions.iter().copied().map(MenuEntry::new).collect()
}
