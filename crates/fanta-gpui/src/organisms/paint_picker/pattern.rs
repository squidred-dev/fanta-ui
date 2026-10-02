//! Source-based pattern layout with controlled numeric editing.

use super::*;

impl PaintPicker {
    fn pattern_number_value(&self, field: usize) -> Option<f32> {
        let DesignPaintPayload::Pattern(pattern) = &self.paint.as_ref()?.payload else {
            return None;
        };
        match field {
            0 => Some(pattern.scaling_factor * 100.),
            1 => Some(pattern.spacing.x),
            2 => Some(pattern.spacing.y),
            _ => None,
        }
    }

    fn pattern_number_edit(&self, field: usize, value: f32) -> Option<DesignPaintEdit> {
        if !value.is_finite() || (field == 0 && value < 0.) {
            return None;
        }
        let DesignPaintPayload::Pattern(pattern) = &self.paint.as_ref()?.payload else {
            return None;
        };
        let (property, value) = match field {
            0 => (
                DesignPaintProperty::PatternScalingFactor,
                DesignPaintValue::Number(value / 100.),
            ),
            1 => (
                DesignPaintProperty::PatternSpacing,
                DesignPaintValue::PatternSpacing(DesignPatternSpacing::new(
                    value,
                    pattern.spacing.y,
                )),
            ),
            2 => (
                DesignPaintProperty::PatternSpacing,
                DesignPaintValue::PatternSpacing(DesignPatternSpacing::new(
                    pattern.spacing.x,
                    value,
                )),
            ),
            _ => return None,
        };
        Some(DesignPaintEdit { property, value })
    }

    pub(super) fn sync_pattern_inputs(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        reset: bool,
    ) {
        self.suppress_input_events = true;
        for field in 0..3 {
            if reset
                || !self.pattern_number_inputs[field]
                    .focus_handle(cx)
                    .is_focused(window)
            {
                let text = self
                    .pattern_number_value(field)
                    .map_or_else(String::new, format_decimal);
                self.pattern_number_inputs[field]
                    .update(cx, |input, cx| input.set_value(text, window, cx));
                self.pattern_number_invalid[field] = false;
            }
        }
        self.suppress_input_events = false;
    }

    pub(super) fn handle_pattern_number_input(
        &mut self,
        field: usize,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputEvent::Focus) {
            self.dismissal_event_guard = false;
        }
        if self.suppress_input_events || self.dismissal_event_guard || self.editing_disabled() {
            return;
        }
        let Some(original) = self
            .pattern_number_value(field)
            .and_then(|value| self.pattern_number_edit(field, value))
        else {
            return;
        };
        let text = self.pattern_number_inputs[field].read(cx).value();
        let candidate = text
            .trim()
            .parse::<f32>()
            .ok()
            .and_then(|value| self.pattern_number_edit(field, value));
        match event {
            InputEvent::Focus => self.dismissal_event_guard = false,
            InputEvent::Change => {
                if !self.pattern_number_inputs[field]
                    .focus_handle(cx)
                    .is_focused(window)
                {
                    return;
                }
                // InputState queues programmatic reset notifications. They must
                // not start a fresh transaction while the field stays focused.
                if self.pattern_number_sessions[field].is_none()
                    && candidate.as_ref() == Some(&original)
                {
                    return;
                }
                if self.pattern_number_sessions[field].is_none() {
                    self.pattern_number_sessions[field] = self.begin_text_input_edit(original, cx);
                }
                self.pattern_number_invalid[field] = candidate.is_none();
                if let Some(candidate) = candidate {
                    self.emit_edit(candidate, DesignPanelEditPhase::Preview, cx);
                }
                cx.notify();
            }
            InputEvent::PressEnter { .. } | InputEvent::Blur => {
                if let Some(session) = self.pattern_number_sessions[field].take() {
                    self.finish_text_input_edit(session, candidate, cx);
                    self.sync_pattern_inputs(window, cx, true);
                }
            }
        }
    }

    fn render_pattern_number(
        &self,
        field: usize,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selector = format!(
            "{}-pattern-{}",
            self.id,
            ["scale", "spacing-x", "spacing-y"][field]
        );
        v_flex()
            .flex_1()
            .min_w_0()
            .gap(px(tokens::InspectorGeometry::ROW_GAP))
            .child(
                div()
                    .typography(crate::atoms::TypographyToken::PanelCaption)
                    .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child(label),
            )
            .child(
                div()
                    .debug_selector(move || selector.clone())
                    .w_full()
                    .on_key_down(cx.listener(move |this, event: &KeyDownEvent, window, cx| {
                        if event.keystroke.key == "escape" {
                            if let Some(session) = this.pattern_number_sessions[field].take() {
                                this.finish_text_input_edit(session, None, cx);
                            }
                            this.sync_pattern_inputs(window, cx, true);
                            window.prevent_default();
                            cx.stop_propagation();
                        }
                    }))
                    .child(
                        Input::new(&self.pattern_number_inputs[field])
                            .xsmall()
                            .typography(crate::atoms::TypographyToken::Panel)
                            .h(px(tokens::RowHeight::LIST))
                            .w_full()
                            .disabled(self.editing_disabled())
                            .when(field == 0, |input| input.suffix(div().child("%")))
                            .when(self.pattern_number_invalid[field], |input| {
                                input.border_color(
                                    crate::atoms::SemanticColor::TextDanger.resolve(cx),
                                )
                            }),
                    ),
            )
            .into_any_element()
    }

    fn render_repeat_layout(
        &self,
        tile_type: DesignPatternTileType,
        compact: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let color = crate::atoms::SemanticColor::TextTertiary
            .resolve(cx)
            .opacity(0.75);
        // A schematic of the repeat arrangement, never a fabricated thumbnail
        // of the host-owned source artwork.
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                let (columns, rows) = if compact { (3, 2) } else { (7, 3) };
                let step_x = bounds.size.width.as_f32() / (columns as f32 + 1.);
                let step_y = bounds.size.height.as_f32() / (rows as f32 + 1.);
                let side = step_x.min(step_y) * 0.55;
                for row in 0..rows {
                    for column in 0..columns {
                        let stagger_x = if tile_type == DesignPatternTileType::HorizontalHexagonal
                            && row % 2 == 1
                        {
                            0.5
                        } else {
                            0.
                        };
                        let stagger_y = if tile_type == DesignPatternTileType::VerticalHexagonal
                            && column % 2 == 1
                        {
                            0.5
                        } else {
                            0.
                        };
                        let origin = bounds.origin
                            + point(
                                px((column as f32 + 0.75 + stagger_x) * step_x - side / 2.),
                                px((row as f32 + 0.75 + stagger_y) * step_y - side / 2.),
                            );
                        window.paint_quad(
                            gpui::fill(Bounds::new(origin, gpui::size(px(side), px(side))), color)
                                .corner_radii(px(tokens::Radius::CONTROL / 2.)),
                        );
                    }
                }
            },
        )
        .w_full()
        .h(px(if compact {
            tokens::RowHeight::FIELD
        } else {
            2. * tokens::RowHeight::LIST
        }))
        .into_any_element()
    }

    pub(super) fn render_pattern_settings(
        &self,
        pattern: &DesignPatternPaint,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut choices = h_flex().w_full().gap(px(tokens::Space::XS));
        for tile_type in DesignPatternTileType::ALL {
            let label = match tile_type {
                DesignPatternTileType::Rectangular => "Grid",
                DesignPatternTileType::HorizontalHexagonal => "Hex H",
                DesignPatternTileType::VerticalHexagonal => "Hex V",
            };
            let selector = format!(
                "{}-pattern-mode-{}",
                self.id,
                tile_type.label().to_lowercase().replace(' ', "-")
            );
            choices = choices.child(
                crate::atoms::ui_button(SharedString::from(selector.clone()))
                    .debug_selector(move || selector.clone())
                    .tooltip(tile_type.label())
                    .child(
                        v_flex()
                            .w_full()
                            .min_w_0()
                            .items_center()
                            .gap(px(tokens::InspectorGeometry::ROW_GAP))
                            .child(self.render_repeat_layout(tile_type, true, cx))
                            .child(
                                div()
                                    .typography(crate::atoms::TypographyToken::PanelCaption)
                                    .child(label),
                            ),
                    )
                    .xsmall()
                    .compact()
                    .outline()
                    .flex_1()
                    .min_w_0()
                    .h(px(2. * tokens::RowHeight::FIELD + tokens::Space::XS))
                    .selected(pattern.tile_type == tile_type)
                    .disabled(self.editing_disabled())
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.emit_edit(
                            DesignPaintEdit {
                                property: DesignPaintProperty::PatternTileType,
                                value: DesignPaintValue::PatternTileType(tile_type),
                            },
                            DesignPanelEditPhase::Commit,
                            cx,
                        );
                    })),
            );
        }
        let mut alignment = h_flex().w_full().gap(px(tokens::Space::XS));
        for option in DesignPatternHorizontalAlignment::ALL {
            let selector = format!(
                "{}-pattern-align-{}",
                self.id,
                option.label().to_lowercase()
            );
            alignment = alignment.child(
                crate::atoms::ui_button(SharedString::from(selector.clone()))
                    .debug_selector(move || selector.clone())
                    .label(option.label())
                    .xsmall()
                    .typography(crate::atoms::TypographyToken::PanelCaption)
                    .compact()
                    .ghost()
                    .flex_1()
                    .min_w_0()
                    .selected(pattern.horizontal_alignment == option)
                    .disabled(self.editing_disabled())
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.emit_edit(
                            DesignPaintEdit {
                                property: DesignPaintProperty::PatternHorizontalAlignment,
                                value: DesignPaintValue::PatternHorizontalAlignment(option),
                            },
                            DesignPanelEditPhase::Commit,
                            cx,
                        );
                    })),
            );
        }
        v_flex()
            .w_full()
            .gap(px(tokens::InspectorGeometry::GROUP_GAP))
            .child(
                v_flex()
                    .w_full()
                    .gap(px(tokens::InspectorGeometry::ROW_GAP))
                    .child(
                        div()
                            .typography(crate::atoms::TypographyToken::PanelCaption)
                            .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                            .child("Repeat layout"),
                    )
                    .child(
                        div()
                            .w_full()
                            .rounded(px(tokens::Radius::CONTROL))
                            .border_1()
                            .border_color(crate::atoms::SemanticColor::BorderPanel.resolve(cx))
                            .bg(crate::atoms::SemanticColor::BackgroundPanelField.resolve(cx))
                            .child(self.render_repeat_layout(pattern.tile_type, false, cx)),
                    )
                    .child(choices),
            )
            .child(
                h_flex()
                    .w_full()
                    .gap(px(tokens::InspectorGeometry::ROW_GAP))
                    .child(self.render_pattern_number(0, "Scale", cx))
                    .child(self.render_pattern_number(1, "Spacing X", cx))
                    .child(self.render_pattern_number(2, "Spacing Y", cx)),
            )
            .child(
                v_flex()
                    .gap(px(tokens::InspectorGeometry::ROW_GAP))
                    .child(
                        div()
                            .typography(crate::atoms::TypographyToken::PanelCaption)
                            .text_color(crate::atoms::SemanticColor::TextTertiary.resolve(cx))
                            .child("Alignment"),
                    )
                    .child(alignment),
            )
            .into_any_element()
    }
}
