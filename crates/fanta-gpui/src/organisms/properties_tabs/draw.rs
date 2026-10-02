use super::{DrawInspectorAction as Action, DrawInspectorViewData, InspectorChoice, controls::*};
use crate::atoms::{
    ControlExt as _, LucideIcon, SemanticColor as Color, SliderBackground, TypographyExt as _,
    TypographyToken, icon_button, render_lucide_icon, tokens,
};
use crate::molecules::{
    InspectorFieldAccess, InspectorMetrics, Slider, SliderAction, SliderPhase, SliderVariant,
    inspector_action_button, inspector_section,
};
use crate::toolbar::DrawBrushCapabilities;
use gpui::StatefulInteractiveElement as _;
use gpui::{
    App, AppContext as _, Context, Div, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString, Styled as _,
    Subscription, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    Disableable as _, Sizable as _, checkbox::Checkbox, h_flex, tooltip::Tooltip, v_flex,
};

pub struct DrawInspector {
    id: SharedString,
    data: DrawInspectorViewData,
    capabilities: DrawBrushCapabilities,
    focus: FocusHandle,
    pressure_focus: FocusHandle,
    anti_alias_focus: FocusHandle,
    tip: Entity<Picker>,
    blend: Entity<Picker>,
    color: Entity<Entry>,
    size: Entity<Entry>,
    size_slider: Entity<Slider>,
    hardness: Entity<Entry>,
    hardness_slider: Entity<Slider>,
    opacity: Entity<Entry>,
    opacity_slider: Entity<Slider>,
    flow: Entity<Entry>,
    flow_slider: Entity<Slider>,
    smoothing: Entity<Entry>,
    smoothing_slider: Entity<Slider>,
    _subscriptions: Vec<Subscription>,
}
impl EventEmitter<Action> for DrawInspector {}
impl DrawInspector {
    pub fn new(
        id: impl Into<SharedString>,
        data: DrawInspectorViewData,
        cx: &mut Context<Self>,
    ) -> Self {
        let id = id.into();
        let size_slider = cx.new(|cx| Slider::new(format!("{id}-size-slider"), 0., cx));
        let hardness_slider = cx.new(|cx| Slider::new(format!("{id}-hardness-slider"), 1., cx));
        let opacity_slider = cx.new(|cx| Slider::new(format!("{id}-opacity-slider"), 1., cx));
        let flow_slider = cx.new(|cx| Slider::new(format!("{id}-flow-slider"), 1., cx));
        let smoothing_slider = cx.new(|cx| Slider::new(format!("{id}-smoothing-slider"), 0., cx));
        let mut subscriptions = vec![
            cx.subscribe(&size_slider, |this, _, event: &SliderAction, cx| {
                if this.data.read_only || event.phase == SliderPhase::Begin {
                    return;
                }
                let mut options = this.data.options.clone();
                options.size = (1. + event.value.clamp(0., 1.).powi(2) * 4999.).round() as u16;
                cx.emit(Action::OptionsChangeRequested { options });
            }),
            cx.subscribe(&hardness_slider, |this, _, event: &SliderAction, cx| {
                if this.data.read_only || event.phase == SliderPhase::Begin {
                    return;
                }
                let mut options = this.data.options.clone();
                options.hardness = (event.value.clamp(0., 1.) * 100.).round() as u8;
                cx.emit(Action::OptionsChangeRequested { options });
            }),
        ];
        for (slider, field) in [
            (&opacity_slider, "opacity"),
            (&flow_slider, "flow"),
            (&smoothing_slider, "smoothing"),
        ] {
            subscriptions.push(
                cx.subscribe(slider, move |this, _, event: &SliderAction, cx| {
                    if this.data.read_only || event.phase == SliderPhase::Begin {
                        return;
                    }
                    let mut options = this.data.options.clone();
                    let value = (event.value.clamp(0., 1.) * 100.).round() as u8;
                    match field {
                        "opacity" => options.opacity = value,
                        "flow" => options.flow = value,
                        _ => options.smoothing = value,
                    }
                    cx.emit(Action::OptionsChangeRequested { options });
                }),
            );
        }
        Self {
            tip: picker(&format!("{id}-tip"), cx, |this, event, cx| {
                if !this.data.read_only {
                    let mut options = this.data.options.clone();
                    options.brush_tip = event.0.clone();
                    if options.brush_tip.as_ref() == "Soft round" {
                        options.hardness = 0;
                    }
                    cx.emit(Action::OptionsChangeRequested { options });
                }
            }),
            blend: picker(&format!("{id}-blend"), cx, |this, event, cx| {
                if !this.data.read_only {
                    cx.emit(Action::BlendModeChangeRequested {
                        id: event.0.clone(),
                    });
                }
            }),
            color: entry(
                &format!("{id}-color"),
                EntryKind::Color,
                cx,
                |this, event, cx| {
                    if !this.data.read_only {
                        cx.emit(Action::ColorChangeRequested {
                            hex: event.0.clone(),
                        });
                    }
                },
            ),
            size: Self::numeric(&id, "size", 1., 5000., cx),
            size_slider,
            hardness: Self::numeric(&id, "hardness", 0., 100., cx),
            hardness_slider,
            opacity: Self::numeric(&id, "opacity", 0., 100., cx),
            opacity_slider,
            flow: Self::numeric(&id, "flow", 0., 100., cx),
            flow_slider,
            smoothing: Self::numeric(&id, "smoothing", 0., 100., cx),
            smoothing_slider,
            id,
            data,
            capabilities: DrawBrushCapabilities::default(),
            focus: cx.focus_handle(),
            pressure_focus: cx.focus_handle(),
            anti_alias_focus: cx.focus_handle(),
            _subscriptions: subscriptions,
        }
    }
    fn numeric(
        id: &str,
        field: &'static str,
        min: f64,
        max: f64,
        cx: &mut Context<Self>,
    ) -> Entity<Entry> {
        let control = entry(
            &format!("{id}-{field}"),
            EntryKind::Number { min, max },
            cx,
            move |this, event, cx| {
                if this.data.read_only {
                    return;
                }
                let value = event.0.parse::<f64>().unwrap_or(min).round() as u16;
                let mut options = this.data.options.clone();
                match field {
                    "size" => options.size = value,
                    "hardness" => options.hardness = value as u8,
                    "opacity" => options.opacity = value as u8,
                    "flow" => options.flow = value as u8,
                    _ => options.smoothing = value as u8,
                }
                cx.emit(Action::OptionsChangeRequested {
                    options: options.normalized(),
                });
            },
        );
        control.update(cx, |entry, _| {
            entry.set_unit(if field == "size" { "px" } else { "%" })
        });
        control
    }
    pub fn view_data(&self) -> &DrawInspectorViewData {
        &self.data
    }
    pub fn set_view_data(&mut self, data: DrawInspectorViewData, cx: &mut Context<Self>) {
        self.data = data;
        cx.notify();
    }

    pub fn set_capabilities(
        &mut self,
        capabilities: DrawBrushCapabilities,
        cx: &mut Context<Self>,
    ) {
        if self.capabilities != capabilities {
            self.capabilities = capabilities;
            cx.notify();
        }
    }

    /// A small accepted round/flat tip sample. Textured host tips keep their
    /// name in the selector; the inspector does not invent a texture renderer.
    fn render_tip_sample(&self, cx: &App) -> impl IntoElement {
        let color = crate::color::parse_hex_rgba(&self.data.color_hex)
            .map(|hex| gpui::Hsla::from(gpui::rgba(hex)))
            .unwrap_or(crate::atoms::sidebar_style(cx).text)
            .opacity(f32::from(self.data.options.opacity) / 100.);
        let tip = self.data.options.brush_tip.clone();
        let diameter =
            f32::from(self.data.options.size).clamp(tokens::Space::XS, tokens::ControlSize::INLINE);
        let tooltip = format!(
            "{tip} tip · {} px · Preview scaled to fit",
            self.data.options.size
        );
        let mut sample = div()
            .id(format!("{}-tip-sample", self.id))
            .debug_selector(|| "draw-tip-sample".to_owned())
            .relative()
            .size(px(tokens::RowHeight::FIELD))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(tokens::Radius::CONTROL))
            .bg(Color::BackgroundPanelField.resolve(cx))
            .overflow_hidden()
            .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx));
        if matches!(tip.as_ref(), "Round" | "Soft round") {
            let hardness = f32::from(self.data.options.hardness) / 100.;
            for pass in (0..3).rev() {
                let size = diameter * (1. - (1. - hardness) * (2 - pass) as f32 / 3.);
                sample = sample.child(
                    div()
                        .absolute()
                        .left(px((tokens::RowHeight::FIELD - size) / 2.))
                        .top(px((tokens::RowHeight::FIELD - size) / 2.))
                        .size(px(size))
                        .rounded_full()
                        .bg(color.opacity(if pass == 0 {
                            1.
                        } else {
                            (1. - hardness) * 0.18
                        })),
                );
            }
        } else if tip.as_ref() == "Flat" {
            sample = sample.child(
                div()
                    .w(px(diameter))
                    .h(px(tokens::Space::XS))
                    .rounded(px(tokens::Space::NONE))
                    .bg(color),
            );
        } else {
            sample = sample.child(render_lucide_icon(
                LucideIcon::Paintbrush,
                crate::atoms::sidebar_style(cx).muted_icon,
                tokens::IconSize::SM,
            ));
        }
        sample
    }

    fn parameter_block(
        label: &'static str,
        slider: Entity<Slider>,
        entry: Entity<Entry>,
        cx: &App,
    ) -> impl IntoElement {
        v_flex()
            .flex_1()
            .min_w_0()
            .gap(px(tokens::InspectorGeometry::CAPTION_GAP))
            .child(
                div()
                    .typography(TypographyToken::PanelCaption)
                    .text_color(crate::atoms::sidebar_style(cx).muted_text)
                    .child(label),
            )
            .child(entry)
            .child(slider)
    }

    fn save_preset_action(&self, cx: &mut Context<Self>) -> impl IntoElement {
        icon_button(
            format!("{}-save-preset", self.id),
            px(tokens::RowHeight::FIELD),
            px(tokens::Radius::CONTROL),
            cx,
        )
        .debug_selector(|| "draw-save-preset".to_owned())
        .tab_index(if self.data.read_only { -1 } else { 0 })
        .when(self.data.read_only, |button| button.opacity(0.6))
        .tooltip(|window, cx| Tooltip::new("Save brush preset").build(window, cx))
        .child(render_lucide_icon(
            LucideIcon::Plus,
            crate::atoms::sidebar_style(cx).muted_icon,
            tokens::IconSize::SM,
        ))
        .on_activate(cx.listener(|this, _, _, cx| {
            if !this.data.read_only && this.capabilities.save_preset {
                cx.emit(Action::BrushPresetSaveRequested);
            }
        }))
    }

    fn parameter_row(
        label: &'static str,
        slider: Entity<Slider>,
        entry: Entity<Entry>,
        cx: &App,
    ) -> impl IntoElement {
        h_flex()
            .w_full()
            .min_w_0()
            .h(px(tokens::RowHeight::FIELD))
            .items_center()
            .gap(px(tokens::Space::SM))
            .child(
                div()
                    .w(px(tokens::InputGeometry::NUMERIC_WIDTH))
                    .flex_none()
                    .typography(TypographyToken::PanelCaption)
                    .text_color(crate::atoms::sidebar_style(cx).muted_text)
                    .child(label),
            )
            .child(div().flex_1().min_w_0().child(slider))
            .child(
                div()
                    .w(px(tokens::InputGeometry::NUMERIC_WIDTH))
                    .flex_none()
                    .child(entry),
            )
    }

    fn settings_body() -> Div {
        v_flex()
            .w_full()
            .min_w_0()
            .px(px(tokens::InspectorGeometry::BODY_INSET))
            .pb(px(tokens::InspectorGeometry::BODY_BOTTOM))
            .gap(px(tokens::InspectorGeometry::GROUP_GAP))
    }

    fn toggle(
        &self,
        field: &'static str,
        label: &'static str,
        checked: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let access = if self.data.read_only {
            InspectorFieldAccess::Disabled { reason: None }
        } else {
            InspectorFieldAccess::Editable
        };
        let id: SharedString = format!("{}-{field}", self.id).into();
        let focus = if field == "pressure" {
            self.pressure_focus.clone()
        } else {
            self.anti_alias_focus.clone()
        };
        inspector_action_button(id.clone(), &access, InspectorMetrics::default(), cx)
            .debug_selector(move || id.to_string())
            .track_focus(&focus.tab_stop(!self.data.read_only))
            .flex_1()
            .min_w_0()
            .justify_start()
            .px(px(tokens::Space::NONE))
            .gap(px(tokens::Space::SM))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    if !this.data.read_only {
                        let focus = if field == "pressure" {
                            &this.pressure_focus
                        } else {
                            &this.anti_alias_focus
                        };
                        focus.focus(window, cx);
                    }
                }),
            )
            .on_activate(cx.listener(move |this, _, _, cx| this.request_toggle(field, cx)))
            .child(
                div().w_full().min_w_0().child(
                    Checkbox::new(SharedString::from(format!("{}-{field}-checkbox", self.id)))
                        .debug_selector(move || format!("draw-{field}-checkbox-content"))
                        .small()
                        .w_full()
                        .h(px(tokens::RowHeight::FIELD))
                        .items_center()
                        .label(label)
                        .typography(TypographyToken::Panel)
                        .checked(checked)
                        .disabled(self.data.read_only)
                        .tab_stop(false),
                ),
            )
    }

    fn request_toggle(&mut self, field: &'static str, cx: &mut Context<Self>) {
        if self.data.read_only {
            return;
        }
        let mut options = self.data.options.clone();
        if field == "pressure" && self.capabilities.pressure {
            options.pressure = !options.pressure;
        } else if field == "antialias" && self.capabilities.anti_alias {
            options.anti_alias = !options.anti_alias;
        } else {
            return;
        }
        cx.emit(Action::OptionsChangeRequested { options });
    }
}
impl Focusable for DrawInspector {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl Render for DrawInspector {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let disabled = self.data.read_only;
        sync_picker(
            &self.tip,
            self.data.options.brush_tip.clone(),
            self.data
                .options
                .brush_tips
                .iter()
                .map(|tip| InspectorChoice::new(tip.clone(), tip.clone()))
                .collect(),
            disabled,
            cx,
        );
        sync_picker(
            &self.blend,
            self.data.blend_mode.clone(),
            self.data.blend_modes.clone(),
            disabled,
            cx,
        );
        for (field, value) in [
            (&self.size, self.data.options.size as u32),
            (&self.hardness, self.data.options.hardness as u32),
            (&self.opacity, self.data.options.opacity as u32),
            (&self.flow, self.data.options.flow as u32),
            (&self.smoothing, self.data.options.smoothing as u32),
        ] {
            sync_entry(field, value.to_string(), disabled, cx);
        }
        sync_entry(&self.color, self.data.color_hex.clone(), disabled, cx);
        self.size_slider.update(cx, |slider, cx| {
            let value = (f32::from(self.data.options.size.saturating_sub(1)) / 4999.).sqrt();
            slider.set_value(value, cx);
            slider.set_compact(true, cx);
            slider.configure(
                SliderVariant::Range,
                SliderBackground::Default,
                disabled,
                cx,
            );
        });
        self.hardness_slider.update(cx, |slider, cx| {
            slider.set_value(f32::from(self.data.options.hardness) / 100., cx);
            slider.set_compact(true, cx);
            slider.configure(
                SliderVariant::Range,
                SliderBackground::Default,
                disabled,
                cx,
            );
        });
        for (slider, value) in [
            (&self.opacity_slider, self.data.options.opacity),
            (&self.flow_slider, self.data.options.flow),
            (&self.smoothing_slider, self.data.options.smoothing),
        ] {
            slider.update(cx, |slider, cx| {
                slider.set_value(f32::from(value) / 100., cx);
                slider.set_compact(true, cx);
                slider.configure(
                    SliderVariant::Range,
                    SliderBackground::Default,
                    disabled,
                    cx,
                );
            });
        }
        v_flex()
            .id(self.id.clone())
            .key_context(super::PROPERTIES_TABS_KEY_CONTEXT)
            .track_focus(&self.focus)
            .size_full()
            .min_w_0()
            .overflow_y_scroll()
            .occlude()
            .on_pinch(|_, _, cx| cx.stop_propagation())
            .bg(Color::BackgroundPanel.resolve(cx))
            .text_color(crate::atoms::sidebar_style(cx).text)
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .child(selection(
                self.data.tool_name.clone(),
                LucideIcon::Paintbrush,
                cx,
            ))
            .when(self.capabilities.brush_settings, |panel| {
                panel.child(
                    inspector_section(cx).child(
                        Self::settings_body()
                            .pt(px(tokens::InspectorGeometry::BODY_INSET))
                            .when(self.capabilities.brush_tip, |body| {
                                body.child(
                                    v_flex()
                                        .w_full()
                                        .min_w_0()
                                        .gap(px(tokens::InspectorGeometry::CAPTION_GAP))
                                        .child(
                                            div()
                                                .typography(TypographyToken::PanelCaption)
                                                .text_color(
                                                    crate::atoms::sidebar_style(cx).muted_text,
                                                )
                                                .child("Tip"),
                                        )
                                        .child(
                                            h_flex()
                                                .w_full()
                                                .min_w_0()
                                                .items_center()
                                                .gap(px(tokens::InspectorGeometry::ROW_GAP))
                                                .child(self.render_tip_sample(cx))
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .min_w_0()
                                                        .child(self.tip.clone()),
                                                )
                                                .when(self.capabilities.save_preset, |row| {
                                                    row.child(self.save_preset_action(cx))
                                                }),
                                        ),
                                )
                            })
                            .when(self.capabilities.hardness, |body| {
                                body.child(
                                    h_flex()
                                        .w_full()
                                        .min_w_0()
                                        .gap(px(tokens::InspectorGeometry::ROW_GAP))
                                        .child(Self::parameter_block(
                                            "Size",
                                            self.size_slider.clone(),
                                            self.size.clone(),
                                            cx,
                                        ))
                                        .child(Self::parameter_block(
                                            "Hardness",
                                            self.hardness_slider.clone(),
                                            self.hardness.clone(),
                                            cx,
                                        )),
                                )
                            })
                            .when(!self.capabilities.hardness, |body| {
                                body.child(Self::parameter_row(
                                    "Size",
                                    self.size_slider.clone(),
                                    self.size.clone(),
                                    cx,
                                ))
                            })
                            .when(
                                self.capabilities.save_preset && !self.capabilities.brush_tip,
                                |body| {
                                    body.child(
                                        h_flex().justify_end().child(self.save_preset_action(cx)),
                                    )
                                },
                            ),
                    ),
                )
            })
            .when(!self.capabilities.brush_settings, |panel| {
                panel.child(empty(
                    LucideIcon::Paintbrush,
                    "Brush settings",
                    "Select Brush, Pencil, or Eraser to edit brush settings.",
                    cx,
                ))
            })
            .when(self.capabilities.paint, |panel| {
                panel.child(
                    section("Paint", cx).child(
                        Self::settings_body()
                            .child(
                                v_flex()
                                    .w_full()
                                    .min_w_0()
                                    .gap(px(tokens::InspectorGeometry::ROW_GAP))
                                    .child(row("Color", self.color.clone(), cx))
                                    .child(row("Blend mode", self.blend.clone(), cx)),
                            )
                            .when(self.capabilities.flow, |body| {
                                body.child(
                                    h_flex()
                                        .w_full()
                                        .min_w_0()
                                        .gap(px(tokens::InspectorGeometry::ROW_GAP))
                                        .child(Self::parameter_block(
                                            "Opacity",
                                            self.opacity_slider.clone(),
                                            self.opacity.clone(),
                                            cx,
                                        ))
                                        .child(Self::parameter_block(
                                            "Flow",
                                            self.flow_slider.clone(),
                                            self.flow.clone(),
                                            cx,
                                        )),
                                )
                            })
                            .when(!self.capabilities.flow, |body| {
                                body.child(Self::parameter_row(
                                    "Opacity",
                                    self.opacity_slider.clone(),
                                    self.opacity.clone(),
                                    cx,
                                ))
                            }),
                    ),
                )
            })
            .when(
                self.capabilities.smoothing
                    || self.capabilities.pressure
                    || self.capabilities.anti_alias,
                |panel| {
                    panel.child(
                        section("Stroke", cx).child(
                            Self::settings_body()
                                .when(self.capabilities.smoothing, |body| {
                                    body.child(Self::parameter_row(
                                        "Smoothing",
                                        self.smoothing_slider.clone(),
                                        self.smoothing.clone(),
                                        cx,
                                    ))
                                })
                                .when(
                                    self.capabilities.pressure || self.capabilities.anti_alias,
                                    |body| {
                                        body.child(
                                            h_flex()
                                                .w_full()
                                                .min_w_0()
                                                .gap(px(tokens::InspectorGeometry::ROW_GAP))
                                                .when(self.capabilities.pressure, |row| {
                                                    row.child(self.toggle(
                                                        "pressure",
                                                        "Pen pressure",
                                                        self.data.options.pressure,
                                                        cx,
                                                    ))
                                                })
                                                .when(self.capabilities.anti_alias, |row| {
                                                    row.child(self.toggle(
                                                        "antialias",
                                                        "Anti-alias",
                                                        self.data.options.anti_alias,
                                                        cx,
                                                    ))
                                                }),
                                        )
                                    },
                                ),
                        ),
                    )
                },
            )
    }
}
