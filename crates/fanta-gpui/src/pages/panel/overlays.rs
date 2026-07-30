use super::*;

impl PagesPanel {
    pub(super) fn control_bounds_tracker(
        &self,
        kind: FocusTooltipKind,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let panel = cx.entity();
        canvas(
            move |bounds, _, app| {
                panel.update(app, |this, _| {
                    this.tooltip_anchor_bounds.insert(kind, bounds);
                });
            },
            |_, _, _, _| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .into_any_element()
    }

    fn render_page_menu_item(
        &mut self,
        label: &'static str,
        selector: &'static str,
        action: PageMenuAction,
        first: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        h_flex()
            .id(SharedString::from(format!("{}-{selector}", self.id)))
            .debug_selector(move || selector.to_owned())
            .key_context(PAGES_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .when(first, |item| {
                item.track_focus(
                    &self
                        .page_menu_focus_handle
                        .clone()
                        .tab_index(0)
                        .tab_stop(true),
                )
            })
            .h(px(36.))
            .flex_none()
            .mx_2()
            .px_3()
            .rounded(px(5.))
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().accent))
            .focus(|style| {
                style
                    .bg(cx.theme().accent)
                    .border_color(cx.theme().selection)
            })
            .on_action(
                cx.listener(move |this, _: &ActivatePagesControl, window, cx| {
                    cx.stop_propagation();
                    this.activate_page_menu_item(action, window, cx);
                }),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                this.activate_page_menu_item(action, window, cx);
            }))
            .child(label)
            .into_any_element()
    }

    pub(super) fn render_page_menu(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let (Some(panel_bounds), Some(menu)) = (self.panel_bounds, self.page_menu.as_ref()) else {
            return div().into_any_element();
        };
        let origin = menu.anchor - panel_bounds.origin;

        deferred(
            v_flex()
                .id(SharedString::from(format!("{}-page-menu", self.id)))
                .absolute()
                .left(origin.x)
                .top(origin.y)
                .debug_selector(|| "pages-page-menu".to_owned())
                .block_mouse_except_scroll()
                .w(px(224.))
                .py_2()
                .rounded(px(12.))
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().popover)
                .shadow_lg()
                .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
                    this.dismiss_page_menu(cx);
                }))
                .child(self.render_page_menu_item(
                    "Copy link to page",
                    "pages-page-menu-copy-link",
                    PageMenuAction::CopyLink,
                    true,
                    cx,
                ))
                .child(div().h(px(1.)).w_full().my_2().bg(cx.theme().border))
                .child(self.render_page_menu_item(
                    "Rename page",
                    "pages-page-menu-rename",
                    PageMenuAction::Rename,
                    false,
                    cx,
                ))
                .child(self.render_page_menu_item(
                    "Duplicate page",
                    "pages-page-menu-duplicate",
                    PageMenuAction::Duplicate,
                    false,
                    cx,
                ))
                .child(div().h(px(1.)).w_full().my_2().bg(cx.theme().border))
                .child(self.render_page_menu_item(
                    "Delete page",
                    "pages-page-menu-delete",
                    PageMenuAction::Delete,
                    false,
                    cx,
                )),
        )
        .with_priority(3)
        .into_any_element()
    }

    pub(super) fn render_focused_tooltip(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let kind = [
            (FocusTooltipKind::Find, &self.find_focus_handle),
            (FocusTooltipKind::AddPage, &self.add_page_focus_handle),
            (FocusTooltipKind::Settings, &self.settings_focus_handle),
            (
                FocusTooltipKind::CloseSearch,
                &self.close_search_focus_handle,
            ),
            (
                FocusTooltipKind::ReplaceCurrent,
                &self.replace_current_focus_handle,
            ),
            (FocusTooltipKind::ReplaceAll, &self.replace_all_focus_handle),
            (
                FocusTooltipKind::PreviousResult,
                &self.previous_result_focus_handle,
            ),
            (FocusTooltipKind::NextResult, &self.next_result_focus_handle),
        ]
        .into_iter()
        .find_map(|(kind, handle)| handle.is_focused(window).then_some(kind))?;
        let panel_bounds = self.panel_bounds?;
        let anchor_bounds = *self.tooltip_anchor_bounds.get(&kind)?;
        let tooltip = match kind {
            FocusTooltipKind::Find => {
                Tooltip::new("Find").action(&FindInPages, Some(PAGES_PANEL_KEY_CONTEXT))
            }
            FocusTooltipKind::AddPage => {
                Tooltip::new("Add new page").action(&AddPage, Some(PAGES_PANEL_KEY_CONTEXT))
            }
            FocusTooltipKind::Settings => Tooltip::new("Settings")
                .action(&ToggleSearchSettings, Some(PAGES_PANEL_KEY_CONTEXT)),
            FocusTooltipKind::CloseSearch => Tooltip::new("Close search")
                .action(&ClosePagesSearch, Some(PAGES_PANEL_KEY_CONTEXT)),
            FocusTooltipKind::ReplaceCurrent => Tooltip::new("Replace current result")
                .action(&ReplaceCurrentResult, Some(PAGES_PANEL_KEY_CONTEXT)),
            FocusTooltipKind::ReplaceAll => Tooltip::new("Replace all results")
                .action(&ReplaceAllResults, Some(PAGES_PANEL_KEY_CONTEXT)),
            FocusTooltipKind::PreviousResult => Tooltip::new("Previous result")
                .action(&PreviousSearchResult, Some(PAGES_PANEL_KEY_CONTEXT)),
            FocusTooltipKind::NextResult => {
                Tooltip::new("Next result").action(&NextSearchResult, Some(PAGES_PANEL_KEY_CONTEXT))
            }
        }
        .build(window, cx);
        let right = panel_bounds.right() - anchor_bounds.right();
        let top = anchor_bounds.bottom() + px(2.) - panel_bounds.top();

        Some(
            deferred(
                div()
                    .absolute()
                    .right(right)
                    .top(top)
                    .debug_selector(|| "pages-focus-tooltip".to_owned())
                    .occlude()
                    .child(tooltip),
            )
            .with_priority(4)
            .into_any_element(),
        )
    }

    pub(super) fn render_filter_menu(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let (Some(panel_bounds), Some(anchor_bounds)) =
            (self.search_panel_bounds, self.filter_anchor_bounds)
        else {
            return div().into_any_element();
        };
        let origin = anchor_bounds.bottom_left() + point(px(0.), px(4.)) - panel_bounds.origin;
        let counts = self.results.element_counts.clone();
        let all_active = self.active_filters.is_empty();
        let mode = self.mode;
        let estimated_height =
            px((PagesPanelElementKind::FILTER_ORDER.len() as f32 + 4.) * 32. + 50.);
        let menu_top = if origin.y < px(0.) { px(0.) } else { origin.y };
        let available_height = panel_bounds.size.height - menu_top;
        let menu_height = if estimated_height > available_height {
            available_height
        } else {
            estimated_height
        };

        deferred(
            v_flex()
                .id(SharedString::from(format!("{}-filter-menu", self.id)))
                .absolute()
                .left(origin.x)
                .top(menu_top)
                .debug_selector(|| "pages-filter-menu".to_owned())
                .capture_key_up(prevent_keyboard_activation_click)
                .block_mouse_except_scroll()
                .w(px(224.))
                .max_h(menu_height)
                .overflow_scroll()
                .track_scroll(&self.filter_menu_scroll_handle)
                .py_2()
                .rounded(px(12.))
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().popover)
                .shadow_lg()
                .child(self.render_mode_item(PanelMode::Find, "Find", mode, cx))
                .child(self.render_mode_item(PanelMode::Replace, "Replace", mode, cx))
                .child(
                    div()
                        .h(px(1.))
                        .w_full()
                        .my_2()
                        .flex_none()
                        .bg(cx.theme().border),
                )
                .children(PagesPanelElementKind::FILTER_ORDER.into_iter().map(|kind| {
                    let active = if kind == PagesPanelElementKind::All {
                        all_active
                    } else {
                        self.active_filters.contains(&kind)
                    };
                    let count = counts
                        .iter()
                        .find(|item| item.kind == kind)
                        .map(|item| item.count);
                    h_flex()
                        .id(SharedString::from(format!(
                            "{}-filter-{}",
                            self.id,
                            kind.label()
                        )))
                        .key_context(PAGES_CONTROL_KEY_CONTEXT)
                        .tab_index(0)
                        .h(px(32.))
                        .flex_none()
                        .mx_2()
                        .px_2()
                        .gap_2()
                        .rounded(px(5.))
                        .cursor_pointer()
                        .hover(|style| style.bg(cx.theme().accent))
                        .focus(|style| {
                            style
                                .bg(cx.theme().accent)
                                .border_color(cx.theme().selection)
                        })
                        .on_action(cx.listener(move |this, _: &ActivatePagesControl, _, cx| {
                            this.toggle_filter(kind, cx);
                        }))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.toggle_filter(kind, cx);
                        }))
                        .child(div().w(px(14.)).when(active, |slot| {
                            slot.child(Icon::new(IconName::Check).xsmall())
                        }))
                        .child(Icon::new(Self::element_icon(kind)).small())
                        .child(div().flex_1().text_sm().child(kind.label()))
                        .when_some(count, |row, count| {
                            row.child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(count.to_string()),
                            )
                        })
                }))
                .child(
                    div()
                        .h(px(1.))
                        .w_full()
                        .my_2()
                        .flex_none()
                        .bg(cx.theme().border),
                )
                .child(self.render_option_item("Match case", self.match_case, true, cx))
                .child(self.render_option_item("Whole words", self.whole_words, false, cx))
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .right_0()
                        .bottom_0()
                        .debug_selector(|| "pages-filter-menu-scrollbar".to_owned())
                        .child(Scrollbar::vertical(&self.filter_menu_scroll_handle).id(
                            SharedString::from(format!("{}-filter-menu-scrollbar", self.id)),
                        )),
                ),
        )
        .with_priority(2)
        .into_any_element()
    }

    fn render_mode_item(
        &mut self,
        item_mode: PanelMode,
        label: &'static str,
        active_mode: PanelMode,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selector = format!("pages-mode-{}", label.to_lowercase());
        let menu_focus_handle = self.filter_menu_focus_handle.clone();
        h_flex()
            .id(SharedString::from(format!("{}-mode-{label}", self.id)))
            .debug_selector(move || selector)
            .key_context(PAGES_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .when(item_mode == PanelMode::Find, |row| {
                row.track_focus(&menu_focus_handle)
            })
            .h(px(32.))
            .flex_none()
            .mx_2()
            .px_2()
            .gap_2()
            .rounded(px(5.))
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().accent))
            .focus(|style| {
                style
                    .bg(cx.theme().accent)
                    .border_color(cx.theme().selection)
            })
            .when(item_mode == active_mode, |row| row.bg(cx.theme().accent))
            .on_action(
                cx.listener(move |this, _: &ActivatePagesControl, window, cx| {
                    this.set_panel_mode(item_mode, window, cx);
                }),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                this.set_panel_mode(item_mode, window, cx);
            }))
            .child(div().w(px(14.)).when(item_mode == active_mode, |slot| {
                slot.child(Icon::new(IconName::Check).xsmall())
            }))
            .child(label)
            .into_any_element()
    }

    fn render_option_item(
        &mut self,
        label: &'static str,
        active: bool,
        match_case: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        h_flex()
            .id(SharedString::from(format!("{}-option-{label}", self.id)))
            .key_context(PAGES_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h(px(32.))
            .flex_none()
            .mx_2()
            .px_2()
            .gap_2()
            .rounded(px(5.))
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().accent))
            .focus(|style| {
                style
                    .bg(cx.theme().accent)
                    .border_color(cx.theme().selection)
            })
            .on_action(cx.listener(move |this, _: &ActivatePagesControl, _, cx| {
                if match_case {
                    this.toggle_match_case(cx);
                } else {
                    this.toggle_whole_words(cx);
                }
            }))
            .on_click(cx.listener(move |this, _, _, cx| {
                if match_case {
                    this.toggle_match_case(cx);
                } else {
                    this.toggle_whole_words(cx);
                }
            }))
            .child(div().w(px(14.)).when(active, |slot| {
                slot.child(Icon::new(IconName::Check).xsmall())
            }))
            .child(label)
            .into_any_element()
    }

    pub(super) fn render_scope_menu(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let (Some(panel_bounds), Some(anchor_bounds)) =
            (self.search_panel_bounds, self.scope_anchor_bounds)
        else {
            return div().into_any_element();
        };
        let origin = anchor_bounds.bottom_left() + point(px(0.), px(4.)) - panel_bounds.origin;
        let menu_focus_handle = self.scope_menu_focus_handle.clone();

        deferred(
            v_flex()
                .absolute()
                .left(origin.x)
                .top(origin.y)
                .debug_selector(|| "pages-scope-menu".to_owned())
                .capture_key_up(prevent_keyboard_activation_click)
                .block_mouse_except_scroll()
                .w(px(168.))
                .py_2()
                .rounded(px(12.))
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().popover)
                .shadow_lg()
                .children(
                    [
                        PagesPanelSearchScope::CurrentPage,
                        PagesPanelSearchScope::AllPages,
                    ]
                    .into_iter()
                    .map(|scope| {
                        let menu_focus_handle = menu_focus_handle.clone();
                        let selector = match scope {
                            PagesPanelSearchScope::CurrentPage => "pages-scope-current-page",
                            PagesPanelSearchScope::AllPages => "pages-scope-all-pages",
                        };
                        h_flex()
                            .id(SharedString::from(format!(
                                "{}-scope-{}",
                                self.id,
                                scope.label()
                            )))
                            .debug_selector(move || selector.to_owned())
                            .key_context(PAGES_CONTROL_KEY_CONTEXT)
                            .tab_index(0)
                            .when(scope == PagesPanelSearchScope::CurrentPage, |row| {
                                row.track_focus(&menu_focus_handle)
                            })
                            .h(px(36.))
                            .flex_none()
                            .mx_2()
                            .px_2()
                            .gap_2()
                            .rounded(px(5.))
                            .cursor_pointer()
                            .hover(|style| style.bg(cx.theme().accent))
                            .focus(|style| {
                                style
                                    .bg(cx.theme().accent)
                                    .border_color(cx.theme().selection)
                            })
                            .on_action(cx.listener(
                                move |this, _: &ActivatePagesControl, window, cx| {
                                    this.set_scope(scope, window, cx);
                                },
                            ))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.set_scope(scope, window, cx);
                            }))
                            .child(div().w(px(14.)).when(scope == self.search_scope, |slot| {
                                slot.child(Icon::new(IconName::Check).xsmall())
                            }))
                            .child(scope.label())
                    }),
                ),
        )
        .with_priority(2)
        .into_any_element()
    }
}
