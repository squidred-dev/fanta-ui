use super::*;

impl PagesPanel {
    pub(super) fn element_icon(kind: PagesPanelElementKind) -> IconName {
        match kind {
            PagesPanelElementKind::All => IconName::GalleryVerticalEnd,
            PagesPanelElementKind::Text => IconName::ALargeSmall,
            PagesPanelElementKind::FrameGroup => IconName::Frame,
            PagesPanelElementKind::Component => IconName::Asterisk,
            PagesPanelElementKind::Instance => IconName::Copy,
            PagesPanelElementKind::Image => IconName::File,
            PagesPanelElementKind::Shape => IconName::ChartPie,
            PagesPanelElementKind::Other => IconName::Ellipsis,
        }
    }

    fn render_search_toolbar(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let replace = self.mode == PanelMode::Replace;
        let can_replace_one = !self.results.items.is_empty();
        let can_replace_all = self.results.total > 0;
        let panel = cx.entity();
        let settings_focus_handle = self
            .settings_focus_handle
            .clone()
            .tab_index(0)
            .tab_stop(true);
        let close_search_focus_handle = self
            .close_search_focus_handle
            .clone()
            .tab_index(0)
            .tab_stop(true);
        let replace_current_focus_handle = self
            .replace_current_focus_handle
            .clone()
            .tab_index(0)
            .tab_stop(true);
        let replace_all_focus_handle = self
            .replace_all_focus_handle
            .clone()
            .tab_index(0)
            .tab_stop(true);
        v_flex()
            .w_full()
            .gap_2()
            .p_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .w_full()
                    .gap_1()
                    .child(
                        div().flex_1().child(
                            Input::new(&self.search_input)
                                .small()
                                .prefix(Icon::new(IconName::Search).small()),
                        ),
                    )
                    .child(
                        div()
                            .relative()
                            .flex_none()
                            .debug_selector(|| "pages-filter-trigger".to_owned())
                            .key_context(PAGES_CONTROL_KEY_CONTEXT)
                            .track_focus(&settings_focus_handle)
                            .focus(|style| style.bg(cx.theme().accent).rounded(px(5.)))
                            .on_action(cx.listener(|this, _: &ActivatePagesControl, window, cx| {
                                cx.stop_propagation();
                                this.open_search_settings(window, cx);
                            }))
                            .child(
                                Button::new(SharedString::from(format!("{}-filters", self.id)))
                                    .ghost()
                                    .small()
                                    .compact()
                                    .tab_stop(false)
                                    .icon(IconName::Settings2)
                                    .tooltip_with_action(
                                        "Settings",
                                        &ToggleSearchSettings,
                                        Some(PAGES_PANEL_KEY_CONTEXT),
                                    )
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.toggle_search_settings(window, cx);
                                    })),
                            )
                            .child(self.control_bounds_tracker(FocusTooltipKind::Settings, cx))
                            .child(
                                canvas(
                                    move |bounds, _, app| {
                                        panel.update(app, |this, _| {
                                            this.filter_anchor_bounds = Some(bounds);
                                        });
                                    },
                                    |_, _, _, _| {},
                                )
                                .absolute()
                                .top_0()
                                .left_0()
                                .size_full(),
                            ),
                    )
                    .child(
                        div()
                            .relative()
                            .flex_none()
                            .debug_selector(|| "pages-close-search".to_owned())
                            .key_context(PAGES_CONTROL_KEY_CONTEXT)
                            .track_focus(&close_search_focus_handle)
                            .focus(|style| style.bg(cx.theme().accent).rounded(px(5.)))
                            .on_action(cx.listener(|this, _: &ActivatePagesControl, window, cx| {
                                cx.stop_propagation();
                                this.close_search(window, cx);
                            }))
                            .child(
                                Button::new(SharedString::from(format!(
                                    "{}-close-search",
                                    self.id
                                )))
                                .ghost()
                                .small()
                                .compact()
                                .tab_stop(false)
                                .icon(IconName::Close)
                                .tooltip_with_action(
                                    "Close search",
                                    &ClosePagesSearch,
                                    Some(PAGES_PANEL_KEY_CONTEXT),
                                )
                                .on_click(cx.listener(
                                    |this, _, window, cx| {
                                        this.close_search(window, cx);
                                    },
                                )),
                            )
                            .child(self.control_bounds_tracker(FocusTooltipKind::CloseSearch, cx)),
                    ),
            )
            .when(replace, |toolbar| {
                toolbar
                    .child(
                        Input::new(&self.replace_input)
                            .small()
                            .prefix(Icon::new(IconName::ArrowRight).small()),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .justify_end()
                            .gap_2()
                            .child(
                                div()
                                    .relative()
                                    .flex_none()
                                    .debug_selector(|| "pages-replace-one".to_owned())
                                    .key_context(PAGES_CONTROL_KEY_CONTEXT)
                                    .when(can_replace_one, |control| {
                                        control.track_focus(&replace_current_focus_handle)
                                    })
                                    .focus(|style| style.bg(cx.theme().accent).rounded(px(5.)))
                                    .on_action(cx.listener(
                                        move |this, _: &ActivatePagesControl, _, cx| {
                                            cx.stop_propagation();
                                            if can_replace_one {
                                                this.request_replace(false, cx);
                                            }
                                        },
                                    ))
                                    .child(
                                        Button::new(SharedString::from(format!(
                                            "{}-replace-one",
                                            self.id
                                        )))
                                        .label("Replace")
                                        .small()
                                        .tab_stop(false)
                                        .disabled(!can_replace_one)
                                        .tooltip_with_action(
                                            "Replace current result",
                                            &ReplaceCurrentResult,
                                            Some(PAGES_PANEL_KEY_CONTEXT),
                                        )
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.request_replace(false, cx);
                                            }),
                                        ),
                                    )
                                    .child(self.control_bounds_tracker(
                                        FocusTooltipKind::ReplaceCurrent,
                                        cx,
                                    )),
                            )
                            .child(
                                div()
                                    .relative()
                                    .flex_none()
                                    .debug_selector(|| "pages-replace-all".to_owned())
                                    .key_context(PAGES_CONTROL_KEY_CONTEXT)
                                    .when(can_replace_all, |control| {
                                        control.track_focus(&replace_all_focus_handle)
                                    })
                                    .focus(|style| style.bg(cx.theme().accent).rounded(px(5.)))
                                    .on_action(cx.listener(
                                        move |this, _: &ActivatePagesControl, _, cx| {
                                            cx.stop_propagation();
                                            if can_replace_all {
                                                this.request_replace(true, cx);
                                            }
                                        },
                                    ))
                                    .child(
                                        Button::new(SharedString::from(format!(
                                            "{}-replace-all",
                                            self.id
                                        )))
                                        .label("Replace all")
                                        .small()
                                        .tab_stop(false)
                                        .disabled(!can_replace_all)
                                        .tooltip_with_action(
                                            "Replace all results",
                                            &ReplaceAllResults,
                                            Some(PAGES_PANEL_KEY_CONTEXT),
                                        )
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.request_replace(true, cx);
                                            }),
                                        ),
                                    )
                                    .child(
                                        self.control_bounds_tracker(
                                            FocusTooltipKind::ReplaceAll,
                                            cx,
                                        ),
                                    ),
                            ),
                    )
            })
            .when(!self.active_filters.is_empty(), |toolbar| {
                toolbar.child(h_flex().w_full().gap_1().flex_wrap().children(
                    self.active_filters.clone().into_iter().map(|kind| {
                        h_flex()
                            .id(SharedString::from(format!(
                                "{}-filter-pill-{}",
                                self.id,
                                kind.label()
                            )))
                            .key_context(PAGES_CONTROL_KEY_CONTEXT)
                            .tab_index(0)
                            .h(px(28.))
                            .gap_1()
                            .px_2()
                            .rounded(px(6.))
                            .border_1()
                            .border_color(cx.theme().border)
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
                            .child(kind.label())
                            .child(Icon::new(IconName::Close).xsmall())
                    }),
                ))
            })
            .into_any_element()
    }

    fn render_results_header(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let result_label = result_count_label(self.results.total);
        let can_navigate = !self.results.items.is_empty();
        let panel = cx.entity();
        let previous_result_focus_handle = self
            .previous_result_focus_handle
            .clone()
            .tab_index(0)
            .tab_stop(true);
        let next_result_focus_handle = self
            .next_result_focus_handle
            .clone()
            .tab_index(0)
            .tab_stop(true);
        let scope_trigger_focus_handle = self
            .scope_trigger_focus_handle
            .clone()
            .tab_index(0)
            .tab_stop(true);
        h_flex()
            .debug_selector(|| "pages-results-header".to_owned())
            .h(px(48.))
            .w_full()
            .min_w(px(0.))
            .px_3()
            .gap_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .debug_selector(|| "pages-result-count".to_owned())
                    .flex_none()
                    .whitespace_nowrap()
                    .text_sm()
                    .child(result_label),
            )
            .child(div().flex_none().text_sm().child("·"))
            .child(
                h_flex()
                    .id(SharedString::from(format!("{}-scope", self.id)))
                    .relative()
                    .flex_none()
                    .whitespace_nowrap()
                    .debug_selector(|| "pages-scope-trigger".to_owned())
                    .key_context(PAGES_CONTROL_KEY_CONTEXT)
                    .track_focus(&scope_trigger_focus_handle)
                    .gap_1()
                    .text_sm()
                    .cursor_pointer()
                    .focus(|style| {
                        style
                            .bg(cx.theme().accent)
                            .border_color(cx.theme().selection)
                    })
                    .on_action(cx.listener(|this, _: &ActivatePagesControl, window, cx| {
                        cx.stop_propagation();
                        this.open_search_scope(window, cx);
                    }))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.toggle_search_scope(window, cx);
                    }))
                    .child(self.search_scope.label())
                    .child(Icon::new(IconName::ChevronDown).xsmall())
                    .child(
                        canvas(
                            move |bounds, _, app| {
                                panel.update(app, |this, _| {
                                    this.scope_anchor_bounds = Some(bounds);
                                });
                            },
                            |_, _, _, _| {},
                        )
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full(),
                    ),
            )
            .child(div().flex_1())
            .child(
                div()
                    .relative()
                    .flex_none()
                    .debug_selector(|| "pages-previous-result".to_owned())
                    .key_context(PAGES_CONTROL_KEY_CONTEXT)
                    .when(can_navigate, |control| {
                        control.track_focus(&previous_result_focus_handle)
                    })
                    .focus(|style| style.bg(cx.theme().accent).rounded(px(5.)))
                    .on_action(cx.listener(move |this, _: &ActivatePagesControl, _, cx| {
                        cx.stop_propagation();
                        if can_navigate {
                            this.navigate_results(PagesPanelResultDirection::Previous, cx);
                        }
                    }))
                    .child(
                        Button::new(SharedString::from(format!("{}-previous", self.id)))
                            .ghost()
                            .xsmall()
                            .compact()
                            .tab_stop(false)
                            .icon(IconName::ChevronUp)
                            .disabled(!can_navigate)
                            .tooltip_with_action(
                                "Previous result",
                                &PreviousSearchResult,
                                Some(PAGES_PANEL_KEY_CONTEXT),
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_results(PagesPanelResultDirection::Previous, cx);
                            })),
                    )
                    .child(self.control_bounds_tracker(FocusTooltipKind::PreviousResult, cx)),
            )
            .child(
                div()
                    .relative()
                    .flex_none()
                    .debug_selector(|| "pages-next-result".to_owned())
                    .key_context(PAGES_CONTROL_KEY_CONTEXT)
                    .when(can_navigate, |control| {
                        control.track_focus(&next_result_focus_handle)
                    })
                    .focus(|style| style.bg(cx.theme().accent).rounded(px(5.)))
                    .on_action(cx.listener(move |this, _: &ActivatePagesControl, _, cx| {
                        cx.stop_propagation();
                        if can_navigate {
                            this.navigate_results(PagesPanelResultDirection::Next, cx);
                        }
                    }))
                    .child(
                        Button::new(SharedString::from(format!("{}-next", self.id)))
                            .ghost()
                            .xsmall()
                            .compact()
                            .tab_stop(false)
                            .icon(IconName::ChevronDown)
                            .disabled(!can_navigate)
                            .tooltip_with_action(
                                "Next result",
                                &NextSearchResult,
                                Some(PAGES_PANEL_KEY_CONTEXT),
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_results(PagesPanelResultDirection::Next, cx);
                            })),
                    )
                    .child(self.control_bounds_tracker(FocusTooltipKind::NextResult, cx)),
            )
            .into_any_element()
    }

    fn render_results(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let active = self.active_result;
        let replacement = self.replace_input.read(cx).value();
        let replace_mode = self.mode == PanelMode::Replace;
        let empty = self.results.items.is_empty();

        v_flex()
            .debug_selector(|| "pages-results-list".to_owned())
            .w_full()
            .when(empty, |results| {
                results.child(
                    div()
                        .debug_selector(|| "pages-results-empty".to_owned())
                        .w_full()
                        .p_4()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(empty_results_label(self.search_scope)),
                )
            })
            .children(
                self.results
                    .items
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(index, result)| {
                        let is_active = active == Some(index);
                        let row = h_flex()
                            .id(SharedString::from(format!("{}-result-{index}", self.id)))
                            .debug_selector(move || format!("pages-result-{index}"))
                            .key_context(PAGES_CONTROL_KEY_CONTEXT)
                            .tab_index(0)
                            .min_h(px(52.))
                            .flex_none()
                            .w_full()
                            .gap_2()
                            .px_4()
                            .py_2()
                            .cursor_pointer()
                            .hover(|style| style.bg(cx.theme().accent))
                            .focus(|style| {
                                style
                                    .bg(cx.theme().accent)
                                    .border_l_2()
                                    .border_color(cx.theme().selection)
                            })
                            .when(is_active, |row| {
                                row.bg(cx.theme().list_active)
                                    .border_l_2()
                                    .border_color(cx.theme().list_active_border)
                            });
                        #[cfg(test)]
                        let row = row.on_hover(cx.listener(move |this, hovered, _, _| {
                            if *hovered {
                                this.hovered_result = Some(index);
                            } else if this.hovered_result == Some(index) {
                                this.hovered_result = None;
                            }
                        }));
                        row.on_click(cx.listener(move |this, _, _, cx| {
                            this.select_result(index, cx);
                        }))
                        .on_action(cx.listener(move |this, _: &ActivatePagesControl, _, cx| {
                            this.select_result(index, cx);
                        }))
                        .child(Icon::new(Self::element_icon(result.kind)).small())
                        .child(
                            v_flex()
                                .gap_0p5()
                                .child(
                                    h_flex()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_sm()
                                                .when(
                                                    replace_mode && !replacement.is_empty(),
                                                    |text| {
                                                        text.text_color(cx.theme().muted_foreground)
                                                            .line_through()
                                                    },
                                                )
                                                .child(result.title),
                                        )
                                        .when(replace_mode && !replacement.is_empty(), |line| {
                                            line.child(replacement.clone())
                                        }),
                                )
                                .when_some(result.parent, |column, parent| {
                                    column.child(
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(parent),
                                    )
                                }),
                        )
                    }),
            )
            .into_any_element()
    }

    pub(super) fn render_search(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let panel = cx.entity();
        v_flex()
            .relative()
            .size_full()
            .min_h(px(280.))
            .bg(cx.theme().sidebar)
            .child(
                canvas(
                    move |bounds, _, app| {
                        panel.update(app, |this, _| {
                            this.search_panel_bounds = Some(bounds);
                        });
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            )
            .child(self.render_search_toolbar(cx))
            .child(self.render_results_header(cx))
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h(px(0.))
                    .child(
                        div()
                            .id(SharedString::from(format!("{}-results-scroll", self.id)))
                            .debug_selector(|| "pages-results-viewport".to_owned())
                            .absolute()
                            .top_0()
                            .right_0()
                            .bottom_0()
                            .left_0()
                            .overflow_scroll()
                            .track_scroll(&self.results_scroll_handle)
                            .child(self.render_results(cx)),
                    )
                    .child(
                        div()
                            .debug_selector(|| "pages-results-scrollbar-layer".to_owned())
                            .absolute()
                            .top_0()
                            .right_0()
                            .bottom_0()
                            .left_0()
                            .child(
                                Scrollbar::vertical(&self.results_scroll_handle).id(
                                    SharedString::from(format!("{}-results-scrollbar", self.id)),
                                ),
                            ),
                    ),
            )
            .when(self.filter_menu_open, |panel| {
                panel.child(self.render_filter_menu(cx))
            })
            .when(self.scope_menu_open, |panel| {
                panel.child(self.render_scope_menu(cx))
            })
            .into_any_element()
    }
}
