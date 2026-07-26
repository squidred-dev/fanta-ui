use super::*;

impl PagesPanel {
    fn collapsed_title(&self) -> SharedString {
        self.selected_page
            .as_ref()
            .and_then(|selected| self.pages.iter().find(|page| &page.id == selected))
            .map(|page| page.title.clone())
            .unwrap_or_else(|| "Pages".into())
    }

    pub(super) fn render_header(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let expanded = self.expanded;
        let title = if expanded {
            SharedString::from("Pages")
        } else {
            self.collapsed_title()
        };

        let header = h_flex()
            .id(SharedString::from(format!("{}-header", self.id)))
            .debug_selector(|| "pages-header".to_owned())
            .key_context(PAGES_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h(px(HEADER_HEIGHT))
            .w_full()
            .flex_shrink_0()
            .cursor_pointer()
            .occlude()
            .hover(|style| style.bg(cx.theme().sidebar_accent.opacity(0.55)))
            .focus(|style| {
                style
                    .bg(cx.theme().sidebar_accent.opacity(0.55))
                    .border_color(cx.theme().selection)
            })
            .when(expanded, |header| {
                header.border_b_1().border_color(cx.theme().border)
            })
            .on_action(cx.listener(|this, _: &ActivatePagesControl, _, cx| {
                this.toggle_expanded(cx);
            }))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, _, _, cx| {
                    cx.stop_propagation();
                    this.dismiss_page_menu(cx);
                }),
            )
            .on_click(cx.listener(|this, _, _, cx| this.toggle_expanded(cx)));
        #[cfg(test)]
        let header = header.on_hover(cx.listener(|this, hovered, _, _| {
            this.header_hovered = *hovered;
        }));

        header
            .child(
                h_flex()
                    .id(SharedString::from(format!("{}-toggle", self.id)))
                    .h_full()
                    .flex_1()
                    .min_w(px(0.))
                    .overflow_hidden()
                    .gap_1()
                    .px_2()
                    .child(
                        Icon::new(if expanded {
                            IconName::ChevronDown
                        } else {
                            IconName::ChevronRight
                        })
                        .xsmall(),
                    )
                    .child(
                        div()
                            .debug_selector(|| "pages-header-title".to_owned())
                            .flex_1()
                            .min_w(px(0.))
                            .truncate()
                            .child(title),
                    ),
            )
            .child(
                div()
                    .relative()
                    .flex_none()
                    .debug_selector(|| "pages-search-trigger".to_owned())
                    .key_context(PAGES_CONTROL_KEY_CONTEXT)
                    .track_focus(&self.find_focus_handle.clone().tab_index(0).tab_stop(true))
                    .focus(|style| style.bg(cx.theme().sidebar_accent).rounded(px(5.)))
                    .on_action(cx.listener(|this, _: &ActivatePagesControl, window, cx| {
                        cx.stop_propagation();
                        this.open_search(window, cx);
                    }))
                    .occlude()
                    .child(
                        Button::new(SharedString::from(format!("{}-search", self.id)))
                            .ghost()
                            .xsmall()
                            .compact()
                            .tab_stop(false)
                            .icon(IconName::Search)
                            .tooltip_with_action(
                                "Find",
                                &FindInPages,
                                Some(PAGES_PANEL_KEY_CONTEXT),
                            )
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_search(window, cx);
                            })),
                    )
                    .child(self.control_bounds_tracker(FocusTooltipKind::Find, cx)),
            )
            .child(
                div()
                    .relative()
                    .flex_none()
                    .debug_selector(|| "pages-add-trigger".to_owned())
                    .key_context(PAGES_CONTROL_KEY_CONTEXT)
                    .track_focus(
                        &self
                            .add_page_focus_handle
                            .clone()
                            .tab_index(0)
                            .tab_stop(true),
                    )
                    .focus(|style| style.bg(cx.theme().sidebar_accent).rounded(px(5.)))
                    .on_action(cx.listener(|this, _: &ActivatePagesControl, window, cx| {
                        cx.stop_propagation();
                        this.begin_new_page(window, cx);
                    }))
                    .occlude()
                    .child(
                        Button::new(SharedString::from(format!("{}-add", self.id)))
                            .ghost()
                            .xsmall()
                            .compact()
                            .tab_stop(false)
                            .icon(IconName::Plus)
                            .tooltip_with_action(
                                "Add new page",
                                &AddPage,
                                Some(PAGES_PANEL_KEY_CONTEXT),
                            )
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.begin_new_page(window, cx);
                            })),
                    )
                    .child(self.control_bounds_tracker(FocusTooltipKind::AddPage, cx)),
            )
            .child(div().w(px(4.)))
            .into_any_element()
    }
}
