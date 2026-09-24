use super::{DrawInspectorAction as Action, DrawInspectorViewData, InspectorChoice, controls::*};
use crate::atoms::{
    ControlExt as _, LucideIcon, SemanticColor as Color, SliderBackground, TypographyExt as _,
    TypographyToken, tokens,
};
use crate::molecules::{Slider, SliderAction, SliderPhase, SliderVariant};
use crate::toolbar::DrawBrushCapabilities;
use gpui::StatefulInteractiveElement as _;
use gpui::{
    App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString, Styled as _,
    Subscription, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_component::{h_flex, v_flex};

pub struct DrawInspector {
    id: SharedString,
    data: DrawInspectorViewData,
    capabilities: DrawBrushCapabilities,
    focus: FocusHandle,
    tip: Entity<Picker>,
    blend: Entity<Picker>,
    color: Entity<Entry>,
    size: Entity<Entry>,
    size_slider: Entity<Slider>,
    hardness: Entity<Entry>,
    hardness_slider: Entity<Slider>,
    opacity: Entity<Entry>,
    flow: Entity<Entry>,
    smoothing: Entity<Entry>,
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
        let subscriptions = vec![
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
            flow: Self::numeric(&id, "flow", 0., 100., cx),
            smoothing: Self::numeric(&id, "smoothing", 0., 100., cx),
            id,
            data,
            capabilities: DrawBrushCapabilities::default(),
            focus: cx.focus_handle(),
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
        entry(
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
        )
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

    fn render_presets(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.data.options.brush_tip.clone();
        let disabled = self.data.read_only;
        let mut gallery = h_flex().flex_wrap().gap(px(tokens::Space::XS));
        for tip in &self.data.options.brush_tips {
            let tip = tip.clone();
            let is_selected = tip == selected;
            let mark_height = match tip.as_ref() {
                "Flat" => 8.,
                "Ink" => 5.,
                _ => 12.,
            };
            let mark_radius = if tip.as_ref() == "Flat" { 1. } else { 8. };
            let mark_opacity = if tip.as_ref() == "Soft round" {
                0.45
            } else {
                1.
            };
            let tile_id: SharedString = format!("{}-preset-{tip}", self.id).into();
            gallery = gallery.child(
                v_flex()
                    .id(tile_id.clone())
                    .debug_selector(move || tile_id.to_string())
                    .when(!disabled, |tile| {
                        tile.key_context(crate::atoms::CONTROL_KEY_CONTEXT)
                            .tab_index(0)
                            .cursor_pointer()
                    })
                    .w(px(82.))
                    .h(px(60.))
                    .items_center()
                    .justify_center()
                    .gap(px(tokens::Space::XS))
                    .rounded(px(tokens::Radius::CONTROL))
                    .border_1()
                    .border_color(
                        if is_selected {
                            Color::BorderSelected
                        } else {
                            Color::Border
                        }
                        .resolve(cx),
                    )
                    .bg(if is_selected {
                        Color::BackgroundSelected
                    } else {
                        Color::BackgroundSecondary
                    }
                    .resolve(cx))
                    .child(
                        div()
                            .w(px(44.))
                            .h(px(mark_height))
                            .rounded(px(mark_radius))
                            .bg(Color::Text.resolve(cx).opacity(mark_opacity)),
                    )
                    .child(
                        div()
                            .typography(TypographyToken::BodySmall)
                            .child(tip.clone()),
                    )
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        if disabled {
                            return;
                        }
                        let mut options = this.data.options.clone();
                        options.brush_tip = tip.clone();
                        if options.brush_tip.as_ref() == "Soft round" {
                            options.hardness = 0;
                        }
                        cx.emit(Action::OptionsChangeRequested { options });
                    })),
            );
        }
        gallery
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
            slider.configure(
                SliderVariant::Range,
                SliderBackground::Default,
                disabled,
                cx,
            );
        });
        self.hardness_slider.update(cx, |slider, cx| {
            slider.set_value(f32::from(self.data.options.hardness) / 100., cx);
            slider.configure(
                SliderVariant::Range,
                SliderBackground::Default,
                disabled,
                cx,
            );
        });
        v_flex()
            .id(self.id.clone())
            .key_context(super::PROPERTIES_TABS_KEY_CONTEXT)
            .track_focus(&self.focus)
            .size_full()
            .min_w_0()
            .overflow_y_scroll()
            .occlude()
            .on_pinch(|_, _, cx| cx.stop_propagation())
            .bg(Color::Background.resolve(cx))
            .text_color(Color::Text.resolve(cx))
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .child(selection(
                self.data.tool_name.clone(),
                LucideIcon::Paintbrush,
                cx,
            ))
            .when(self.capabilities.brush_settings, |panel| {
                panel.child(
                    section("Brush", cx).child(
                        body()
                            .when(self.capabilities.brush_tip, |body| {
                                body.child(row("Tip", self.tip.clone(), cx))
                                    .child(self.render_presets(cx))
                            })
                            .child(row("Size · px", self.size.clone(), cx))
                            .child(row("", self.size_slider.clone(), cx))
                            .when(self.capabilities.hardness, |body| {
                                body.child(row("Hardness · %", self.hardness.clone(), cx))
                                    .child(row("", self.hardness_slider.clone(), cx))
                            }),
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
                        body()
                            .child(row("Color", self.color.clone(), cx))
                            .child(row("Blend mode", self.blend.clone(), cx))
                            .child(row("Opacity · %", self.opacity.clone(), cx))
                            .when(self.capabilities.flow, |body| {
                                body.child(row("Flow · %", self.flow.clone(), cx))
                            }),
                    ),
                )
            })
            .when(self.capabilities.smoothing, |panel| {
                panel.child(
                    section("Stroke", cx).child(
                        body()
                            .child(row("Smooth · %", self.smoothing.clone(), cx))
                            .when(
                                self.capabilities.pressure || self.capabilities.anti_alias,
                                |body| {
                                    body.child(
                                        h_flex()
                                            .gap(px(tokens::Space::SM))
                                            .when(self.capabilities.pressure, |row| {
                                                row.child(
                                                    action(
                                                        format!("{}-pressure", self.id).into(),
                                                        "Pen pressure",
                                                        LucideIcon::PenTool,
                                                        !disabled,
                                                        cx,
                                                    )
                                                    .flex_1()
                                                    .when(self.data.options.pressure, |v| {
                                                        v.bg(Color::BackgroundSecondary.resolve(cx))
                                                    })
                                                    .on_activate(cx.listener(|this, _, _, cx| {
                                                        if !this.data.read_only {
                                                            let mut options =
                                                                this.data.options.clone();
                                                            options.pressure = !options.pressure;
                                                            cx.emit(
                                                                Action::OptionsChangeRequested {
                                                                    options,
                                                                },
                                                            );
                                                        }
                                                    })),
                                                )
                                            })
                                            .when(self.capabilities.anti_alias, |row| {
                                                row.child(
                                                    action(
                                                        format!("{}-antialias", self.id).into(),
                                                        "Anti-alias",
                                                        LucideIcon::Spline,
                                                        !disabled,
                                                        cx,
                                                    )
                                                    .flex_1()
                                                    .when(self.data.options.anti_alias, |v| {
                                                        v.bg(Color::BackgroundSecondary.resolve(cx))
                                                    })
                                                    .on_activate(cx.listener(|this, _, _, cx| {
                                                        if !this.data.read_only {
                                                            let mut options =
                                                                this.data.options.clone();
                                                            options.anti_alias =
                                                                !options.anti_alias;
                                                            cx.emit(
                                                                Action::OptionsChangeRequested {
                                                                    options,
                                                                },
                                                            );
                                                        }
                                                    })),
                                                )
                                            }),
                                    )
                                },
                            ),
                    ),
                )
            })
            .when(self.capabilities.save_preset, |panel| {
                panel.child(
                    div().p(px(tokens::Space::LG)).child(
                        action(
                            format!("{}-save-preset", self.id).into(),
                            "Save brush preset",
                            LucideIcon::Plus,
                            !disabled,
                            cx,
                        )
                        .w_full()
                        .bg(Color::BackgroundSecondary.resolve(cx))
                        .on_activate(cx.listener(|this, _, _, cx| {
                            if !this.data.read_only {
                                cx.emit(Action::BrushPresetSaveRequested);
                            }
                        })),
                    ),
                )
            })
    }
}
