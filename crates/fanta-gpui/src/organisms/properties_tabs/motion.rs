use super::{
    InspectorChoice, MotionInspectorAction as Action, MotionInspectorViewData, controls::*,
};
use crate::{
    atoms::{
        ControlExt as _, LucideIcon, SemanticColor as Color, TypographyExt as _, TypographyToken,
        tokens, truncating_label,
    },
    timeline::{TimelineEasing, TimelinePlayback},
};
use gpui::StatefulInteractiveElement as _;
use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement as _,
    IntoElement, ParentElement as _, Render, SharedString, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{h_flex, v_flex};

pub struct MotionInspector {
    id: SharedString,
    data: MotionInspectorViewData,
    auto_keyframe_available: bool,
    focus: FocusHandle,
    duration: Entity<Entry>,
    delay: Entity<Entry>,
    easing: Entity<Picker>,
    playback: Entity<Picker>,
    preset: Entity<Picker>,
}
impl EventEmitter<Action> for MotionInspector {}
impl MotionInspector {
    pub fn new(
        id: impl Into<SharedString>,
        data: MotionInspectorViewData,
        cx: &mut Context<Self>,
    ) -> Self {
        let id = id.into();
        Self {
            duration: entry(
                &format!("{id}-duration"),
                EntryKind::Number {
                    min: 1.,
                    max: 3_600_000.,
                },
                cx,
                |this, event, cx| {
                    if !this.data.read_only && this.data.can_edit_timing {
                        cx.emit(Action::DurationChangeRequested {
                            duration_ms: event.0.parse::<f64>().unwrap_or(1.).round() as u32,
                        });
                    }
                },
            ),
            delay: entry(
                &format!("{id}-delay"),
                EntryKind::Number {
                    min: 0.,
                    max: 3_600_000.,
                },
                cx,
                |this, event, cx| {
                    if !this.data.read_only && this.data.can_edit_timing {
                        cx.emit(Action::DelayChangeRequested {
                            delay_ms: event.0.parse::<f64>().unwrap_or(0.).round() as u32,
                        });
                    }
                },
            ),
            easing: picker(&format!("{id}-easing"), cx, |this, event, cx| {
                if !this.data.read_only
                    && this.data.can_edit_timing
                    && let Some(easing) = TimelineEasing::presets()
                        .into_iter()
                        .find(|v| v.label() == event.0.as_ref())
                {
                    cx.emit(Action::EasingChangeRequested { easing });
                }
            }),
            playback: picker(&format!("{id}-playback"), cx, |_, event, cx| {
                if let Some(playback) = TimelinePlayback::ALL
                    .into_iter()
                    .find(|v| v.label() == event.0.as_ref())
                {
                    cx.emit(Action::PlaybackChangeRequested { playback });
                }
            }),
            preset: picker(&format!("{id}-preset"), cx, |this, event, cx| {
                if !this.data.read_only && !this.data.selection_name.is_empty() {
                    cx.emit(Action::PresetApplyRequested {
                        id: event.0.clone(),
                    });
                }
            }),
            id,
            data,
            auto_keyframe_available: true,
            focus: cx.focus_handle(),
        }
    }
    pub fn view_data(&self) -> &MotionInspectorViewData {
        &self.data
    }
    pub fn set_view_data(&mut self, data: MotionInspectorViewData, cx: &mut Context<Self>) {
        self.data = data;
        cx.notify();
    }

    pub fn set_auto_keyframe_available(&mut self, available: bool, cx: &mut Context<Self>) {
        if self.auto_keyframe_available != available {
            self.auto_keyframe_available = available;
            cx.notify();
        }
    }
}
impl Focusable for MotionInspector {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl Render for MotionInspector {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let disabled = self.data.read_only || self.data.selection_name.is_empty();
        let timing_disabled = disabled || !self.data.can_edit_timing;
        sync_entry(
            &self.duration,
            self.data.duration_ms.to_string(),
            timing_disabled,
            cx,
        );
        sync_entry(
            &self.delay,
            self.data.delay_ms.to_string(),
            timing_disabled,
            cx,
        );
        sync_picker(
            &self.easing,
            self.data.easing.label(),
            TimelineEasing::presets()
                .into_iter()
                .map(|v| InspectorChoice::new(v.label(), v.label()))
                .collect(),
            timing_disabled,
            cx,
        );
        sync_picker(
            &self.playback,
            self.data.playback.label(),
            TimelinePlayback::ALL
                .into_iter()
                .map(|v| InspectorChoice::new(v.label(), v.label()))
                .collect(),
            false,
            cx,
        );
        sync_picker(
            &self.preset,
            self.data.selected_preset.clone().unwrap_or_default(),
            self.data.presets.clone(),
            disabled || self.data.presets.is_empty(),
            cx,
        );
        let mut properties = body();
        for property in &self.data.animated_properties {
            let id = property.id.clone();
            properties = properties.child(
                h_flex()
                    .gap(px(tokens::Space::SM))
                    .items_center()
                    .child(
                        truncating_label(property.label.clone())
                            .typography(TypographyToken::BodyMedium),
                    )
                    .child(
                        action(
                            format!("{}-key-{}", self.id, property.id).into(),
                            "",
                            LucideIcon::Diamond,
                            !disabled,
                            cx,
                        )
                        .on_activate(cx.listener(move |this, _, _, cx| {
                            if !this.data.read_only {
                                cx.emit(Action::KeyframeAddRequested {
                                    property_id: id.clone(),
                                });
                            }
                        })),
                    ),
            );
        }
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
            .when(!self.data.selection_name.is_empty(), |v| {
                v.child(selection(
                    self.data.selection_name.clone(),
                    LucideIcon::Clapperboard,
                    cx,
                ))
            })
            .when(self.data.selection_name.is_empty(), |v| {
                v.child(empty(
                    LucideIcon::Clapperboard,
                    "Create motion",
                    "Select a layer to inspect timing, animation styles, and animated properties.",
                    cx,
                ))
            })
            .child(
                section("Animation", cx).child(
                    body()
                        .child(row("Style", self.preset.clone(), cx))
                        .child(row("Duration · ms", self.duration.clone(), cx))
                        .child(row("Delay · ms", self.delay.clone(), cx))
                        .child(row("Easing", self.easing.clone(), cx)),
                ),
            )
            .child(
                section("Playback", cx).child(
                    body()
                        .child(row("Repeat", self.playback.clone(), cx))
                        .child(
                            h_flex()
                                .gap(px(tokens::Space::SM))
                                .child(
                                    action(
                                        format!("{}-play", self.id).into(),
                                        if self.data.playing {
                                            "Pause"
                                        } else {
                                            "Preview"
                                        },
                                        if self.data.playing {
                                            LucideIcon::Pause
                                        } else {
                                            LucideIcon::Play
                                        },
                                        self.data.can_preview,
                                        cx,
                                    )
                                    .flex_1()
                                    .bg(Color::BackgroundSecondary.resolve(cx))
                                    .on_activate(cx.listener(|this, _, _, cx| {
                                        if this.data.can_preview {
                                            cx.emit(Action::PlayingChangeRequested {
                                                playing: !this.data.playing,
                                            });
                                        }
                                    })),
                                )
                                .when(self.auto_keyframe_available, |row| {
                                    row.child(
                                        action(
                                            format!("{}-auto", self.id).into(),
                                            "Auto key",
                                            LucideIcon::Diamond,
                                            !disabled,
                                            cx,
                                        )
                                        .flex_1()
                                        .when(self.data.auto_keyframe, |v| {
                                            v.bg(Color::BackgroundSecondary.resolve(cx))
                                        })
                                        .on_activate(
                                            cx.listener(|this, _, _, cx| {
                                                if !this.data.read_only {
                                                    cx.emit(Action::AutoKeyframeChangeRequested {
                                                        enabled: !this.data.auto_keyframe,
                                                    });
                                                }
                                            }),
                                        ),
                                    )
                                }),
                        ),
                ),
            )
            .child(section("Animated properties", cx).child(properties.when(
                self.data.animated_properties.is_empty(),
                |v| {
                    v.child(
                        div()
                            .typography(TypographyToken::BodyMedium)
                            .text_color(Color::TextSecondary.resolve(cx))
                            .child("Properties with keyframes appear here."),
                    )
                },
            )))
            .when(self.data.timeline_open_available, |view| {
                view.child(
                    div().p(px(tokens::Space::LG)).child(
                        action(
                            format!("{}-timeline", self.id).into(),
                            "Open timeline",
                            LucideIcon::PanelBottom,
                            true,
                            cx,
                        )
                        .w_full()
                        .bg(Color::BackgroundSecondary.resolve(cx))
                        .on_activate(
                            cx.listener(|_, _, _, cx| cx.emit(Action::TimelineOpenRequested)),
                        ),
                    ),
                )
            })
    }
}
