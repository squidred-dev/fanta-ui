use super::*;
use gpui_component::{Sizable as _, input::Input};

impl Timeline {
    pub(super) fn button(
        &self,
        suffix: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: LucideIcon,
        active: bool,
        enabled: bool,
        cx: &App,
    ) -> Stateful<Div> {
        let suffix = suffix.into();
        let label = label.into();
        icon_button(
            SharedString::from(format!("{}-{suffix}", self.id)),
            px(tokens::ControlSize::CHROME),
            px(tokens::Radius::CONTROL),
            cx,
        )
        .debug_selector(move || format!("timeline-{suffix}"))
        .flex_none()
        .tab_index(if enabled { 0 } else { -1 })
        .when(active, |b| b.bg(Color::BackgroundSelected.resolve(cx)))
        .when(!enabled, |b| b.opacity(0.4).cursor_default())
        .tooltip(move |window, cx| Tooltip::new(label.clone()).build(window, cx))
        .child(render_lucide_icon(
            icon,
            Color::Text.resolve(cx),
            tokens::IconSize::SM,
        ))
    }
    // A control's identity, presentation and typed request stay together at call sites.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn command(
        &self,
        suffix: &'static str,
        label: &'static str,
        icon: LucideIcon,
        active: bool,
        enabled: bool,
        action: TimelineAction,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.button(suffix, label, icon, active, enabled, cx)
            .when(enabled, |b| {
                b.on_activate(cx.listener(move |_, _, _, cx| cx.emit(action.clone())))
            })
            .into_any_element()
    }
    pub(super) fn render_controls(&self, _window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let d = &self.view_data;
        let editing = !d.read_only;
        let has_editable_keys = self
            .editable_keys()
            .iter()
            .any(|k| d.selected_keyframes.contains(&k.id));
        h_flex()
            .id("timeline-controls")
            .debug_selector(|| "timeline-controls".to_owned())
            .h(px(HEADER))
            .w_full()
            .flex_none()
            .px_2()
            .gap_1()
            .overflow_x_scroll()
            .border_b_1()
            .border_color(Color::Border.resolve(cx))
            .child(
                div()
                    .flex_none()
                    .px_2()
                    .typography(TypographyToken::BodyLargeStrong)
                    .child("Timeline"),
            )
            .child(self.command(
                "previous",
                "Previous keyframe",
                LucideIcon::SkipBack,
                false,
                true,
                TimelineAction::SeekRequested {
                    time_ms: self.adjacent_key(false),
                },
                cx,
            ))
            .child(self.command(
                "play",
                if d.playing {
                    "Pause · Space"
                } else {
                    "Play · Space"
                },
                if d.playing {
                    LucideIcon::Pause
                } else {
                    LucideIcon::Play
                },
                d.playing,
                true,
                TimelineAction::PlayStateChangeRequested {
                    playing: !d.playing,
                },
                cx,
            ))
            .child(self.command(
                "next",
                "Next keyframe",
                LucideIcon::SkipForward,
                false,
                true,
                TimelineAction::SeekRequested {
                    time_ms: self.adjacent_key(true),
                },
                cx,
            ))
            .child(self.command(
                "auto-keyframe",
                "Auto-keyframe",
                LucideIcon::CircleDot,
                d.auto_keyframe,
                editing,
                TimelineAction::AutoKeyframeChangeRequested {
                    enabled: !d.auto_keyframe,
                },
                cx,
            ))
            .child(self.command(
                "keyframe",
                "Add keyframe · K",
                LucideIcon::DiamondPlus,
                false,
                editing && !self.selected_tracks().is_empty(),
                TimelineAction::AddKeyframeRequested {
                    time_ms: d.current_time_ms,
                },
                cx,
            ))
            .child(
                div()
                    .id("timeline-current-time")
                    .debug_selector(|| "timeline-current-time".to_owned())
                    .w(px(tokens::InputGeometry::NUMERIC_WIDTH))
                    .flex_none()
                    .tooltip(|window, cx| {
                        Tooltip::new("Current time · Enter to seek").build(window, cx)
                    })
                    .child(
                        Input::new(self.time_input.as_ref().unwrap())
                            .small()
                            .typography(TypographyToken::BodyMedium),
                    ),
            )
            .child(
                div()
                    .flex_none()
                    .text_color(Color::TextTertiary.resolve(cx))
                    .child("/"),
            )
            .child(
                div()
                    .id("timeline-duration")
                    .debug_selector(|| "timeline-duration".to_owned())
                    .w(px(tokens::InputGeometry::NUMERIC_WIDTH))
                    .flex_none()
                    .tooltip(|window, cx| {
                        Tooltip::new("Animation duration · Enter to apply").build(window, cx)
                    })
                    .child(
                        Input::new(self.duration_input.as_ref().unwrap())
                            .small()
                            .disabled(!editing)
                            .typography(TypographyToken::BodyMedium),
                    ),
            )
            .child(
                icon_button(
                    "timeline-time-unit",
                    px(tokens::ControlSize::CHROME),
                    px(tokens::Radius::CONTROL),
                    cx,
                )
                .debug_selector(|| "timeline-time-unit".to_owned())
                .flex_none()
                .child(d.time_unit.label())
                .on_activate(cx.listener(|this, _, _, cx| {
                    cx.emit(TimelineAction::TimeUnitChangeRequested {
                        unit: if this.view_data.time_unit == TimelineTimeUnit::Seconds {
                            TimelineTimeUnit::Milliseconds
                        } else {
                            TimelineTimeUnit::Seconds
                        },
                    })
                })),
            )
            .child(
                self.button(
                    "playback",
                    d.playback_mode().label(),
                    match d.playback_mode() {
                        TimelinePlayback::Once => LucideIcon::ArrowRight,
                        TimelinePlayback::Loop => LucideIcon::Repeat2,
                        TimelinePlayback::PingPong => LucideIcon::ArrowLeftRight,
                    },
                    false,
                    true,
                    cx,
                )
                .on_activate(cx.listener(|this, _, _, cx| {
                    this.overlay = if this.overlay == Some(Overlay::Playback) {
                        None
                    } else {
                        Some(Overlay::Playback)
                    };
                    cx.notify();
                })),
            )
            .child(self.command(
                "snap",
                "Snap timing to nearby keyframes",
                LucideIcon::Magnet,
                d.snapping,
                editing,
                TimelineAction::SnappingChangeRequested {
                    enabled: !d.snapping,
                },
                cx,
            ))
            .child(
                self.button(
                    "easing",
                    "Edit selected keyframe easing",
                    LucideIcon::Spline,
                    false,
                    editing && has_editable_keys,
                    cx,
                )
                .when(editing && has_editable_keys, |b| {
                    b.on_activate(cx.listener(|this, _, _, cx| {
                        this.overlay = Some(Overlay::Easing(TimelineEasingTarget::Keyframes(
                            this.editable_keys()
                                .into_iter()
                                .filter(|k| this.view_data.selected_keyframes.contains(&k.id))
                                .map(|k| k.id)
                                .collect(),
                        )));
                        this.easing_error = None;
                        cx.notify();
                    }))
                }),
            )
            .child(
                self.button(
                    "preset",
                    "Add animation preset",
                    LucideIcon::ListPlus,
                    false,
                    editing && !self.selected_tracks().is_empty() && !d.presets.is_empty(),
                    cx,
                )
                .when(
                    editing && !self.selected_tracks().is_empty() && !d.presets.is_empty(),
                    |b| {
                        b.on_activate(cx.listener(|this, _, _, cx| {
                            this.overlay = Some(Overlay::Presets);
                            cx.notify();
                        }))
                    },
                ),
            )
            .child(self.command(
                "comment",
                "Comment at playhead",
                LucideIcon::MessageSquarePlus,
                false,
                editing,
                TimelineAction::CommentAddRequested {
                    time_ms: d.current_time_ms,
                },
                cx,
            ))
            .child(div().flex_1().min_w(px(tokens::Space::SM)))
            .child(
                self.button(
                    "fit",
                    "Fit animation in view",
                    LucideIcon::ScanLine,
                    false,
                    true,
                    cx,
                )
                .on_activate(cx.listener(|this, _, _, cx| {
                    this.scroll_x = 0.;
                    cx.emit(TimelineAction::ZoomChangeRequested { zoom: 1. });
                    cx.notify();
                })),
            )
            .child(
                div()
                    .debug_selector(|| "timeline-zoom".to_owned())
                    .w(px(tokens::InputGeometry::VARIABLE_CELL_WIDTH))
                    .flex_none()
                    .child(self.zoom_slider.clone()),
            )
            .child(
                div()
                    .w(px(tokens::InputGeometry::MULTI_CELL_WIDTH))
                    .flex_none()
                    .text_right()
                    .child(format!("{:.0}%", d.zoom * 100.)),
            )
            .child(self.command(
                "help",
                "Timeline help",
                LucideIcon::CircleQuestionMark,
                false,
                true,
                TimelineAction::HelpRequested,
                cx,
            ))
            .child(
                self.button(
                    "collapse",
                    if self.collapsed {
                        "Expand timeline"
                    } else {
                        "Collapse timeline"
                    },
                    if self.collapsed {
                        LucideIcon::ChevronUp
                    } else {
                        LucideIcon::ChevronDown
                    },
                    false,
                    true,
                    cx,
                )
                .on_activate(cx.listener(|this, _, _, cx| {
                    this.collapsed = !this.collapsed;
                    this.overlay = None;
                    cx.emit(TimelineAction::CollapsedChanged {
                        collapsed: this.collapsed,
                    });
                    cx.notify();
                })),
            )
            .into_any_element()
    }
    pub(super) fn adjacent_key(&self, next: bool) -> u32 {
        let current = self.view_data.current_time_ms;
        let times = self.view_data.tracks.iter().flat_map(|t| {
            t.properties
                .iter()
                .flat_map(|p| p.keyframes.iter().map(|k| k.time_ms))
        });
        if next {
            times
                .filter(|t| *t > current)
                .min()
                .unwrap_or(self.view_data.duration_ms)
        } else {
            times.filter(|t| *t < current).max().unwrap_or(0)
        }
    }
}
