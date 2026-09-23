use super::{DrawInspectorAction as Action, DrawInspectorViewData, InspectorChoice, controls::*};
use crate::atoms::{ControlExt as _, LucideIcon, SemanticColor as Color, tokens};
use crate::toolbar::DrawBrushCapabilities;
use gpui::StatefulInteractiveElement as _;
use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, SharedString, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
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
    hardness: Entity<Entry>,
    opacity: Entity<Entry>,
    flow: Entity<Entry>,
    smoothing: Entity<Entry>,
}
impl EventEmitter<Action> for DrawInspector {}
impl DrawInspector {
    pub fn new(
        id: impl Into<SharedString>,
        data: DrawInspectorViewData,
        cx: &mut Context<Self>,
    ) -> Self {
        let id = id.into();
        Self {
            tip: picker(&format!("{id}-tip"), cx, |this, event, cx| {
                if !this.data.read_only {
                    let mut options = this.data.options.clone();
                    options.brush_tip = event.0.clone();
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
            hardness: Self::numeric(&id, "hardness", 0., 100., cx),
            opacity: Self::numeric(&id, "opacity", 0., 100., cx),
            flow: Self::numeric(&id, "flow", 0., 100., cx),
            smoothing: Self::numeric(&id, "smoothing", 0., 100., cx),
            id,
            data,
            capabilities: DrawBrushCapabilities::default(),
            focus: cx.focus_handle(),
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
        v_flex()
            .id(self.id.clone())
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
            .child(
                section("Brush", cx).child(
                    body()
                        .when(self.capabilities.brush_tip, |body| {
                            body.child(row("Tip", self.tip.clone(), cx))
                        })
                        .child(row("Size · px", self.size.clone(), cx))
                        .when(self.capabilities.hardness, |body| {
                            body.child(row("Hardness · %", self.hardness.clone(), cx))
                        }),
                ),
            )
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
