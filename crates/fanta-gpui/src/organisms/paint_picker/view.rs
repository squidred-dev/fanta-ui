//! Paint picker surface and navigation; leaf editors live in their own modules.
use super::*;

impl PaintPicker {
    fn render_header(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.shader_browser_requested
            && self.active_tab == PaintPickerTab::Libraries
            && self
                .paint
                .as_ref()
                .is_some_and(|paint| paint.paint_type() != DesignPaintType::Shader)
        {
            return h_flex()
                .w_full()
                .h(px(PICKER_HEADER_HEIGHT))
                .flex_none()
                .px(px(tokens::Space::LG))
                .gap(px(tokens::InspectorGeometry::ROW_GAP))
                .border_b_1()
                .border_color(crate::atoms::SemanticColor::BorderPanel.resolve(cx))
                .child(
                    crate::atoms::ui_button(SharedString::from(format!(
                        "{}-shader-browser-back",
                        self.id
                    )))
                    .debug_selector(|| "paint-picker-shader-back".to_owned())
                    .tooltip("Back to paint")
                    .xsmall()
                    .compact()
                    .ghost()
                    .child(render_lucide_icon(
                        LucideIcon::ArrowLeft,
                        crate::atoms::SemanticColor::Text.resolve(cx),
                        tokens::IconSize::SM,
                    ))
                    .on_activate(cx.listener(|this, _, _, cx| {
                        this.active_tab = PaintPickerTab::Custom;
                        this.shader_browser_requested = false;
                        this.scroll_handle.set_offset(Point::default());
                        cx.notify();
                    })),
                )
                .child(
                    div()
                        .flex_1()
                        .typography(crate::atoms::TypographyToken::PanelStrong)
                        .child("Shaders"),
                )
                .child(
                    crate::atoms::ui_button(SharedString::from(format!("{}-close", self.id)))
                        .icon(IconName::Close)
                        .tooltip("Close")
                        .debug_selector(|| "paint-picker-close".to_owned())
                        .xsmall()
                        .compact()
                        .ghost()
                        .on_activate(|_, window, cx| {
                            window.dispatch_action(Box::new(CancelDesignInteraction), cx);
                        }),
                )
                .into_any_element();
        }
        if let Some(
            paint_type @ (DesignPaintType::Pattern
            | DesignPaintType::Image
            | DesignPaintType::Video),
        ) = self.paint.as_ref().map(DesignPaint::paint_type)
        {
            return self
                .render_color_only_header(format!("{} fill", paint_type.label()).into(), cx);
        }
        let shader_active = self
            .paint
            .as_ref()
            .is_some_and(|paint| paint.paint_type() == DesignPaintType::Shader);
        crate::atoms::InspectorTabs::new(
            format!("{}-content-tabs", self.id),
            [
                (
                    format!("{}-custom-tab", self.id).into(),
                    if shader_active {
                        "Parameters"
                    } else {
                        "Custom"
                    }
                    .into(),
                ),
                (
                    format!("{}-libraries-tab", self.id).into(),
                    if shader_active || self.shader_browser_requested {
                        "Shaders"
                    } else {
                        "Libraries"
                    }
                    .into(),
                ),
            ],
        )
        .selected_index(usize::from(self.active_tab == PaintPickerTab::Libraries))
        .on_change(
            cx.listener(|this, selection: &crate::atoms::TabSelection, _, cx| {
                this.active_tab = if selection.index == 0 {
                    PaintPickerTab::Custom
                } else {
                    PaintPickerTab::Libraries
                };
                this.shader_browser_requested = this.active_tab == PaintPickerTab::Libraries
                    && this
                        .paint
                        .as_ref()
                        .is_some_and(|paint| paint.paint_type() == DesignPaintType::Shader);
                this.scroll_handle.set_offset(Point::default());
                cx.notify();
            }),
        )
        .end_child(self.render_creation_menu(cx))
        .end_child(
            crate::atoms::ui_button(SharedString::from(format!("{}-close", self.id)))
                .icon(IconName::Close)
                .tooltip("Close")
                .debug_selector(|| "paint-picker-close".to_owned())
                .xsmall()
                .typography(crate::atoms::TypographyToken::Panel)
                .compact()
                .ghost()
                .on_activate(|_, window, cx| {
                    window.dispatch_action(Box::new(CancelDesignInteraction), cx);
                }),
        )
        .into_any_element()
    }

    fn render_color_only_header(&self, title: SharedString, cx: &mut Context<Self>) -> AnyElement {
        h_flex()
            .w_full()
            .h(px(PICKER_HEADER_HEIGHT))
            .flex_none()
            .gap(px(tokens::Space::SM))
            .px(px(tokens::Space::LG))
            .border_b_1()
            .border_color(crate::atoms::SemanticColor::BorderPanel.resolve(cx))
            .child(
                div()
                    .flex_1()
                    .typography(crate::atoms::TypographyToken::Panel)
                    .font_semibold()
                    .child(title),
            )
            .child(
                crate::atoms::ui_button(SharedString::from(format!("{}-close", self.id)))
                    .icon(IconName::Close)
                    .tooltip("Close")
                    .debug_selector(|| "paint-picker-close".to_owned())
                    .xsmall()
                    .typography(crate::atoms::TypographyToken::Panel)
                    .compact()
                    .ghost()
                    .on_activate(|_, window, cx| {
                        window.dispatch_action(Box::new(CancelDesignInteraction), cx);
                    }),
            )
            .into_any_element()
    }

    fn render_paint_kind_mark(&self, kind: DesignPaintKind, cx: &mut Context<Self>) -> AnyElement {
        let foreground = crate::atoms::SemanticColor::Text.resolve(cx);
        let faint = crate::atoms::SemanticColor::TextTertiary
            .resolve(cx)
            .opacity(0.18);
        let mark = div()
            .relative()
            .size(px(15.))
            .overflow_hidden()
            .rounded(px(2.))
            .border_1()
            .border_color(
                crate::atoms::SemanticColor::TextTertiary
                    .resolve(cx)
                    .opacity(0.72),
            );

        match kind {
            DesignPaintKind::Solid => mark.bg(foreground).into_any_element(),
            DesignPaintKind::LinearGradient => mark
                .bg(linear_gradient(
                    90.,
                    linear_color_stop(foreground, 0.),
                    linear_color_stop(faint, 1.),
                ))
                .into_any_element(),
            DesignPaintKind::RadialGradient => mark
                .bg(faint)
                .child(
                    div()
                        .absolute()
                        .left(px(3.))
                        .top(px(3.))
                        .size(px(7.))
                        .rounded(px(7.))
                        .bg(foreground),
                )
                .into_any_element(),
            DesignPaintKind::AngularGradient => mark
                .bg(linear_gradient(
                    45.,
                    linear_color_stop(foreground, 0.),
                    linear_color_stop(faint, 1.),
                ))
                .into_any_element(),
            DesignPaintKind::DiamondGradient => mark
                .bg(linear_gradient(
                    135.,
                    linear_color_stop(faint, 0.),
                    linear_color_stop(foreground, 1.),
                ))
                .into_any_element(),
            DesignPaintKind::Pattern => mark
                .bg(pattern_slash(
                    crate::atoms::SemanticColor::TextTertiary
                        .resolve(cx)
                        .opacity(0.75),
                    0.45,
                    0.45,
                ))
                .into_any_element(),
            DesignPaintKind::Image => {
                render_lucide_icon(LucideIcon::Image, foreground, tokens::IconSize::SM)
            }
            DesignPaintKind::Video => {
                render_lucide_icon(LucideIcon::Video, foreground, tokens::IconSize::SM)
            }
            DesignPaintKind::Shader => Icon::new(IconName::Asterisk).small().into_any_element(),
            DesignPaintKind::Unsupported => Icon::new(IconName::Info).small().into_any_element(),
        }
    }

    fn render_type_tabs(&self, paint: &DesignPaint, cx: &mut Context<Self>) -> AnyElement {
        debug_assert!(PICKER_WIDTH >= paint_header_min_width());
        let disabled = self.base_editing_disabled();
        let mut tabs = h_flex()
            .w_full()
            .h(px(PAINT_TYPE_ROW_HEIGHT))
            .flex_none()
            .gap(px(tokens::Space::XS))
            .px(px(tokens::Space::LG))
            .border_b_1()
            .border_color(crate::atoms::SemanticColor::BorderPanel.resolve(cx));
        for paint_type in DesignPaintType::ALL {
            let supported = self.supported_paint_types.contains(&paint_type);
            if !supported && paint.paint_type() != paint_type {
                continue;
            }
            tabs = tabs.child(
                crate::atoms::ui_button(SharedString::from(format!(
                    "{}-type-{}",
                    self.id,
                    paint_type.label().to_lowercase()
                )))
                .debug_selector(move || {
                    format!("paint-picker-type-{}", paint_type.label().to_lowercase())
                })
                .tooltip(paint_type.label())
                .xsmall()
                .typography(crate::atoms::TypographyToken::Panel)
                .compact()
                .ghost()
                .w(px(PAINT_HEADER_CONTROL_SIZE))
                .min_w(px(PAINT_HEADER_CONTROL_SIZE))
                .flex_none()
                .h(px(PAINT_HEADER_CONTROL_SIZE))
                .selected(paint.paint_type() == paint_type)
                .disabled(disabled || !supported)
                .child(self.render_paint_kind_mark(
                    if paint_type == DesignPaintType::Gradient && paint.kind.is_gradient() {
                        paint.kind
                    } else {
                        paint_type.default_kind()
                    },
                    cx,
                ))
                .on_activate(cx.listener(move |this, _, _, cx| {
                    this.select_paint_type(paint_type, cx);
                })),
            );
        }
        tabs.into_any_element()
    }

    fn render_paint_utilities(&self, paint: &DesignPaint, cx: &mut Context<Self>) -> AnyElement {
        h_flex()
            .w_full()
            .min_w_0()
            .h(px(PAINT_UTILITY_ROW_HEIGHT))
            .flex_none()
            .px(px(tokens::Space::LG))
            .gap(px(tokens::InspectorGeometry::ROW_GAP))
            .border_b_1()
            .border_color(crate::atoms::SemanticColor::BorderPanel.resolve(cx))
            .child(
                div()
                    .typography(crate::atoms::TypographyToken::PanelCaption)
                    .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child("Blend"),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(self.render_blend_mode_control(paint, cx)),
            )
            .child(
                crate::atoms::ui_button(SharedString::from(format!("{}-contrast", self.id)))
                    .tooltip(
                        self.current_contrast_leaf()
                            .and_then(|leaf| leaf.disabled_reason.clone())
                            .unwrap_or_else(|| "Check WCAG color contrast".into()),
                    )
                    .xsmall()
                    .typography(crate::atoms::TypographyToken::Panel)
                    .compact()
                    .ghost()
                    .w(px(PAINT_HEADER_CONTROL_SIZE))
                    .h(px(PAINT_HEADER_CONTROL_SIZE))
                    .selected(self.contrast_checker_open)
                    .disabled(self.current_contrast_leaf().is_none())
                    .child(self.render_contrast_mark(cx))
                    .on_activate(cx.listener(|this, _, _, cx| {
                        this.contrast_checker_open = !this.contrast_checker_open;
                        cx.notify();
                    })),
            )
            .into_any_element()
    }

    fn render_contrast_mark(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .relative()
            .size(px(15.))
            .rounded(px(15.))
            .border_1()
            .border_color(crate::atoms::SemanticColor::Text.resolve(cx))
            .child(
                div()
                    .absolute()
                    .left(px(3.))
                    .top(px(4.))
                    .w(px(8.))
                    .h(px(1.))
                    .bg(crate::atoms::SemanticColor::Text.resolve(cx)),
            )
            .child(
                div()
                    .absolute()
                    .left(px(3.))
                    .top(px(8.))
                    .w(px(8.))
                    .h(px(1.))
                    .bg(crate::atoms::SemanticColor::Text.resolve(cx)),
            )
            .into_any_element()
    }

    fn render_blend_mode_control(&self, paint: &DesignPaint, cx: &mut Context<Self>) -> AnyElement {
        let picker_for_open = cx.entity();
        let picker_for_trigger = picker_for_open.clone();
        let picker_for_content = picker_for_open.clone();
        let picker_id = self.id.clone();
        let selected_mode = paint.blend_mode;
        let supported_blend_modes = self.supported_blend_modes.clone();
        let blend_trigger =
            crate::atoms::ui_button(SharedString::from(format!("{}-paint-blend-mode", self.id)))
                .label(paint.blend_mode.label())
                .text_left()
                .dropdown_caret(true)
                .debug_selector(|| "paint-picker-blend-mode".to_owned())
                .tooltip("Paint blend mode")
                .xsmall()
                .typography(crate::atoms::TypographyToken::Panel)
                .compact()
                .ghost()
                .w_full()
                .min_w_0()
                .h(px(tokens::RowHeight::FIELD))
                .selected(self.nested_overlay_is_open(PaintPickerOverlay::BlendMode))
                .disabled(self.editing_disabled())
                .on_keyboard_activate(move |window, cx| {
                    picker_for_trigger.update(cx, |this, cx| {
                        if this.nested_overlay_is_open(PaintPickerOverlay::BlendMode) {
                            this.close_nested_overlay(PaintPickerOverlay::BlendMode, window, cx);
                        } else {
                            this.open_nested_overlay(PaintPickerOverlay::BlendMode, window, cx);
                            cx.notify();
                        }
                    });
                });

        let blend = Popover::new(SharedString::from(format!(
            "{}-paint-blend-mode-menu",
            self.id
        )))
        .anchor(Anchor::BottomRight)
        .appearance(false)
        .open(self.nested_overlay_is_open(PaintPickerOverlay::BlendMode))
        .overlay_closable(true)
        .on_open_change(move |open, window, cx| {
            picker_for_open.update(cx, |this, cx| {
                if *open {
                    this.open_nested_overlay(PaintPickerOverlay::BlendMode, window, cx);
                    cx.notify();
                } else {
                    this.dismiss_nested_overlay(
                        PaintPickerOverlay::BlendMode,
                        InspectorOverlayDismissCause::OutsideClick,
                        window,
                        cx,
                    );
                }
            });
        })
        .trigger(blend_trigger)
        .content(move |_, window, cx| {
            crate::molecules::sidebar_popup_surface(
                SharedString::from(format!("{picker_id}-blend-options")),
                cx,
            )
            .debug_selector(|| "paint-picker-blend-menu".to_owned())
            .w(popup_width(window, tokens::MenuWidth::STANDARD))
            .max_h(popup_height(window, tokens::RowHeight::MENU * 10.))
            .p(px(tokens::Space::XS))
            .overflow_y_scroll()
            .children(supported_blend_modes.clone().into_iter().map(|mode| {
                let picker = picker_for_content.clone();
                let picker_for_key = picker.clone();
                let picker_for_hover = picker.clone();
                crate::atoms::ui_button(SharedString::from(format!(
                    "{}-paint-blend-mode-{}",
                    picker_id,
                    mode.label().to_lowercase().replace(' ', "-")
                )))
                .child(picker_menu_option(
                    format!(
                        "paint-picker-blend-{}",
                        mode.label().to_lowercase().replace(' ', "-")
                    ),
                    mode.label(),
                    None,
                    selected_mode == mode,
                    cx,
                ))
                .tooltip(mode.label())
                .xsmall()
                .typography(crate::atoms::TypographyToken::Panel)
                .compact()
                .ghost()
                .w_full()
                .h(px(tokens::RowHeight::MENU))
                .px(px(tokens::Space::SM))
                .selected(selected_mode == mode)
                .debug_selector(move || {
                    format!(
                        "paint-picker-blend-{}",
                        mode.label().to_lowercase().replace(' ', "-")
                    )
                })
                .on_hover(move |hovered, _, cx| {
                    picker_for_hover.update(cx, |this, cx| {
                        this.set_blend_mode_preview(mode, *hovered, cx);
                    });
                })
                .on_activate(move |_, window, cx| {
                    picker.update(cx, |this, cx| {
                        this.select_blend_mode(mode, window, cx);
                    });
                })
                // Enter/Space activate through the bound `ActivateControl`
                // command above; only dismissal and focus-move previews
                // stay key-matched.
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    picker_for_key.update(cx, |this, cx| {
                        this.set_blend_mode_preview(mode, true, cx);
                        match event.keystroke.key.as_str() {
                            "escape"
                                if this.dismiss_nested_overlay(
                                    PaintPickerOverlay::BlendMode,
                                    InspectorOverlayDismissCause::Escape,
                                    window,
                                    cx,
                                ) =>
                            {
                                window.prevent_default();
                                cx.stop_propagation();
                            }
                            "tab" => this.cancel_blend_mode_preview(cx),
                            _ => {}
                        }
                    });
                })
            }))
        })
        .w_full()
        .min_w_0()
        .h(px(tokens::RowHeight::FIELD));

        blend.into_any_element()
    }

    fn render_empty(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .id(self.id.clone())
            .track_focus(&self.focus_handle)
            .w(popup_width(window, PICKER_WIDTH))
            .p(px(tokens::InspectorGeometry::BODY_INSET))
            .items_center()
            .justify_center()
            .gap(px(tokens::InspectorGeometry::ROW_GAP))
            .rounded(px(8.))
            .border_1()
            .border_color(crate::atoms::SemanticColor::BorderPanel.resolve(cx))
            .bg(crate::atoms::sidebar_style(cx).menu_background)
            .child(
                Icon::new(IconName::Palette)
                    .small()
                    .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx)),
            )
            .child(
                div()
                    .typography(crate::atoms::TypographyToken::Panel)
                    .font_semibold()
                    .child("No paint selected"),
            )
            .child(
                div()
                    .typography(crate::atoms::TypographyToken::Panel)
                    .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child("Call set_target to enable the picker."),
            )
            .into_any_element()
    }

    pub(super) fn render_picker(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(paint) = self.paint.as_ref() else {
            return self.render_empty(window, cx);
        };
        if let Some(title) = self.color_only_title.clone() {
            return v_flex()
                .id(self.id.clone())
                .track_focus(&self.focus_handle)
                .relative()
                .w(popup_width(window, PICKER_WIDTH))
                .max_h(popup_height(window, PICKER_MAX_HEIGHT))
                .overflow_hidden()
                .rounded(px(tokens::Radius::MENU))
                .border_1()
                .border_color(crate::atoms::SemanticColor::BorderPanel.resolve(cx))
                .bg(crate::atoms::sidebar_style(cx).menu_background)
                .text_color(crate::atoms::SemanticColor::Text.resolve(cx))
                .shadow_lg()
                .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                    this.handle_picker_key(event, window, cx);
                }))
                .child(self.render_color_only_header(title, cx))
                .child(
                    v_flex()
                        .w_full()
                        .gap(px(tokens::InspectorGeometry::GROUP_GAP))
                        .p(px(tokens::InspectorGeometry::BODY_INSET))
                        .children(self.render_color_editor(cx)),
                )
                .into_any_element();
        }
        let paint_type = paint.paint_type();
        let color_editable = matches!(
            paint_type,
            DesignPaintType::Solid | DesignPaintType::Gradient
        );
        let has_media_tabs = matches!(paint_type, DesignPaintType::Image | DesignPaintType::Video)
            && self.active_tab == PaintPickerTab::Custom;
        let scroll_handle = self.scroll_handle.clone();
        let body = if self.active_tab == PaintPickerTab::Libraries {
            v_flex()
                .id(SharedString::from(format!("{}-libraries-scroll", self.id)))
                .w_full()
                .max_h(
                    popup_height(window, PICKER_MAX_HEIGHT)
                        - px(PICKER_HEADER_HEIGHT
                            + PAINT_TYPE_ROW_HEIGHT
                            + PAINT_UTILITY_ROW_HEIGHT
                            + if has_media_tabs {
                                tokens::InspectorGeometry::SECTION_HEADER
                            } else {
                                0.
                            }),
                )
                .overflow_y_scroll()
                .track_scroll(&self.scroll_handle)
                .child(if self.shader_browser_requested {
                    self.render_shader_browser(paint, cx)
                } else {
                    self.render_variables(paint, cx)
                })
                .into_any_element()
        } else {
            let content = v_flex()
                .w_full()
                .gap(px(tokens::InspectorGeometry::GROUP_GAP))
                .p(px(tokens::InspectorGeometry::BODY_INSET))
                .children(self.render_paint_tools(cx))
                .children(self.render_gradient_controls(paint, cx))
                .when(color_editable, |content| {
                    content
                        .children(self.render_color_editor(cx))
                        .child(self.render_variable_scope(paint, cx))
                })
                .when(!color_editable, |content| {
                    content
                        .child(self.render_source_state(paint, cx))
                        .child(self.render_opacity_only(cx))
                });
            v_flex()
                .id(SharedString::from(format!("{}-custom-scroll", self.id)))
                .w_full()
                .max_h(
                    popup_height(window, PICKER_MAX_HEIGHT)
                        - px(PICKER_HEADER_HEIGHT
                            + PAINT_TYPE_ROW_HEIGHT
                            + PAINT_UTILITY_ROW_HEIGHT
                            + if has_media_tabs {
                                tokens::InspectorGeometry::SECTION_HEADER
                            } else {
                                0.
                            }),
                )
                .overflow_y_scroll()
                .track_scroll(&self.scroll_handle)
                .child(content)
                .into_any_element()
        };

        v_flex()
            .id(self.id.clone())
            .track_focus(&self.focus_handle)
            .relative()
            .w(popup_width(window, PICKER_WIDTH))
            .max_h(popup_height(window, PICKER_MAX_HEIGHT))
            .overflow_hidden()
            .rounded(px(tokens::Radius::MENU))
            .border_1()
            .border_color(crate::atoms::SemanticColor::BorderPanel.resolve(cx))
            .bg(crate::atoms::sidebar_style(cx).menu_background)
            .text_color(crate::atoms::SemanticColor::Text.resolve(cx))
            .shadow_lg()
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                this.handle_picker_key(event, window, cx);
            }))
            .child(self.render_header(cx))
            .when(self.active_tab == PaintPickerTab::Custom, |picker| {
                picker
                    .child(self.render_type_tabs(paint, cx))
                    .child(self.render_paint_utilities(paint, cx))
            })
            .when(has_media_tabs, |picker| {
                picker.child(self.render_media_tabs(cx))
            })
            .child(
                div().relative().min_h_0().w_full().child(body).child(
                    div()
                        .absolute()
                        .top_0()
                        .right_0()
                        .h_full()
                        .child(Scrollbar::new(&scroll_handle).axis(ScrollbarAxis::Vertical)),
                ),
            )
            .into_any_element()
    }
}
