use super::*;
use gpui_component::Sizable as _;

impl Timeline {
    fn grid(&self) -> Vec<u32> {
        let step = self.step_ms();
        let first = (self.scroll_x / self.pixels_per_ms()) as u32 / step * step;
        let visible = (self.ruler_bounds.size.width.as_f32() / self.pixels_per_ms()) as u32;
        let last = first
            .saturating_add(visible)
            .saturating_add(step)
            .min(self.view_data.duration_ms);
        (first..=last).step_by(step as usize).take(100).collect()
    }
    fn lane(&self, id: SharedString, cx: &App) -> Stateful<Div> {
        let mut lane = div()
            .id(id)
            .relative()
            .flex_1()
            .min_w(px(0.))
            .h_full()
            .overflow_hidden();
        for time in self.grid() {
            lane = lane.child(
                div()
                    .absolute()
                    .left(px(self.time_x(time)))
                    .top_0()
                    .bottom_0()
                    .w(px(1.))
                    .bg(Color::Border.resolve(cx).opacity(0.35)),
            );
        }
        lane
    }
    fn playhead(&self, cx: &App) -> Div {
        div()
            .debug_selector(|| "timeline-playhead-line".to_owned())
            .absolute()
            .left(px(self.time_x(self.view_data.current_time_ms)))
            .top_0()
            .bottom_0()
            .w(px(1.))
            .bg(if self.view_data.auto_keyframe {
                Color::BackgroundDanger.resolve(cx)
            } else {
                Color::BackgroundBrand.resolve(cx)
            })
    }
    pub(super) fn render_ruler(&self, cx: &mut Context<Self>) -> AnyElement {
        let all_expanded = self.view_data.tracks.iter().all(|t| t.expanded);
        let mut ruler = self
            .lane("timeline-ruler".into(), cx)
            .debug_selector(|| "timeline-ruler".to_owned())
            .cursor_ew_resize()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, e: &MouseDownEvent, window, cx| {
                    this.focus_handle.focus(window, cx);
                    this.drag = Some(Drag::Seek);
                    cx.emit(TimelineAction::SeekRequested {
                        time_ms: this.time_at(e.position),
                    });
                    cx.stop_propagation();
                }),
            )
            .child(track_bounds(cx.entity(), |this, bounds| {
                this.ruler_bounds = bounds
            }));
        for time in self.grid() {
            ruler = ruler.child(
                div()
                    .absolute()
                    .left(px(self.time_x(time) + tokens::Space::XS))
                    .top(px(tokens::Space::LG + tokens::Space::XS))
                    .typography(TypographyToken::BodyMedium)
                    .text_color(Color::TextSecondary.resolve(cx))
                    .child(format!(
                        "{}{}",
                        self.view_data.time_unit.format(time),
                        self.view_data.time_unit.label()
                    )),
            );
        }
        for comment in &self.view_data.comments {
            let id = comment.id.clone();
            ruler = ruler.child(
                self.button(
                    format!("comment-{}", comment.id),
                    comment.label.clone(),
                    LucideIcon::MessageCircle,
                    false,
                    true,
                    cx,
                )
                .absolute()
                .left(px(self.time_x(comment.time_ms) - tokens::Space::SM))
                .top_0()
                .size(px(tokens::ControlSize::INLINE))
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_activate(cx.listener(move |_, _, _, cx| {
                    cx.emit(TimelineAction::CommentOpenRequested {
                        comment_id: id.clone(),
                    })
                })),
            );
        }
        let end = self.time_x(match self.drag {
            Some(Drag::Duration { time }) => time,
            _ => self.view_data.duration_ms,
        });
        ruler = ruler.child(self.playhead(cx)).child(
            div()
                .absolute()
                .left(px(
                    self.time_x(self.view_data.current_time_ms) - tokens::Space::XS
                ))
                .top_0()
                .w(px(tokens::Space::SM))
                .h(px(tokens::Space::SM))
                .rounded_b(px(tokens::Radius::CONTROL))
                .bg(if self.view_data.auto_keyframe {
                    Color::BackgroundDanger.resolve(cx)
                } else {
                    Color::BackgroundBrand.resolve(cx)
                }),
        );
        if !self.view_data.read_only {
            ruler = ruler.child(
                div()
                    .id("timeline-duration-handle")
                    .debug_selector(|| "timeline-duration-handle".to_owned())
                    .absolute()
                    .left(px(end - tokens::Space::XS))
                    .top_0()
                    .bottom_0()
                    .w(px(tokens::Space::SM))
                    .cursor_ew_resize()
                    .border_l_1()
                    .border_color(Color::TextTertiary.resolve(cx))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, cx| {
                            window.prevent_default();
                            this.drag = Some(Drag::Duration {
                                time: this.view_data.duration_ms,
                            });
                            cx.stop_propagation();
                        }),
                    ),
            );
        }
        h_flex()
            .h(px(tokens::RowHeight::SECTION_HEADER))
            .flex_none()
            .w_full()
            .border_b_1()
            .border_color(Color::Border.resolve(cx))
            .child(
                h_flex()
                    .debug_selector(|| "timeline-layer-header".to_owned())
                    .w(px(RAIL))
                    .h_full()
                    .flex_none()
                    .px_2()
                    .gap_1()
                    .border_r_1()
                    .border_color(Color::Border.resolve(cx))
                    .child(self.command(
                        "expand-all",
                        if all_expanded {
                            "Collapse all layers"
                        } else {
                            "Expand all layers"
                        },
                        LucideIcon::ListTree,
                        false,
                        true,
                        TimelineAction::ExpandAllRequested {
                            expanded: !all_expanded,
                        },
                        cx,
                    ))
                    .child(
                        div()
                            .flex_1()
                            .typography(TypographyToken::BodyMediumStrong)
                            .child("Layers"),
                    )
                    .child(
                        div()
                            .text_color(Color::TextTertiary.resolve(cx))
                            .child(self.view_data.tracks.len().to_string()),
                    ),
            )
            .child(ruler)
            .into_any_element()
    }
    fn track_label(&self, track: &TimelineTrack, cx: &mut Context<Self>) -> AnyElement {
        let id = track.id.clone();
        let expand_id = id.clone();
        let lock_id = id.clone();
        let visibility_id = id.clone();
        let expanded = track.expanded;
        let locked = track.locked;
        let visible = track.visible;
        h_flex()
            .id(SharedString::from(format!("track-label-{id}")))
            .debug_selector(move || format!("timeline-track-{id}"))
            .w(px(RAIL))
            .h_full()
            .flex_none()
            .px_1()
            .gap_1()
            .border_r_1()
            .border_color(Color::Border.resolve(cx))
            .key_context(crate::atoms::CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .on_activate(cx.listener({
                let id = track.id.clone();
                move |this, _, window, cx| {
                    if this.input_focused(window, cx) {
                        return;
                    }
                    cx.emit(TimelineAction::TrackSelectionRequested {
                        track_ids: if this.view_data.tracks.iter().any(|t| t.id == id) {
                            vec![id.clone()]
                        } else {
                            vec![]
                        },
                    })
                }
            }))
            .child(
                self.button(
                    format!("expand-{}", track.id),
                    "Expand layer properties",
                    if expanded {
                        LucideIcon::ChevronDown
                    } else {
                        LucideIcon::ChevronRight
                    },
                    false,
                    true,
                    cx,
                )
                .size(px(tokens::ControlSize::INLINE))
                .ml(px(track.depth.min(5) as f32 * tokens::Space::SM))
                .on_activate(cx.listener(move |_, _, _, cx| {
                    cx.emit(TimelineAction::TrackExpansionRequested {
                        track_id: expand_id.clone(),
                        expanded: !expanded,
                    })
                })),
            )
            .child(render_lucide_icon(
                LucideIcon::Layers,
                Color::TextSecondary.resolve(cx),
                tokens::IconSize::SM,
            ))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener({
                            let id = track.id.clone();
                            move |this, event: &MouseDownEvent, window, cx| {
                                if event.click_count == 2
                                    && this.renaming_track.as_ref() != Some(&id)
                                {
                                    this.begin_rename(id.clone(), window, cx);
                                    cx.stop_propagation();
                                }
                            }
                        }),
                    )
                    .child(if self.renaming_track.as_ref() == Some(&track.id) {
                        gpui_component::input::Input::new(self.rename_input.as_ref().unwrap())
                            .small()
                            .typography(TypographyToken::BodyMedium)
                            .into_any_element()
                    } else {
                        truncating_label(track.name.clone())
                            .typography(TypographyToken::BodyMediumStrong)
                            .into_any_element()
                    }),
            )
            .child(
                self.button(
                    format!("visibility-{}", track.id),
                    "Toggle layer visibility",
                    if visible {
                        LucideIcon::Eye
                    } else {
                        LucideIcon::EyeOff
                    },
                    false,
                    !self.view_data.read_only,
                    cx,
                )
                .size(px(tokens::ControlSize::INLINE))
                .when(!self.view_data.read_only, |b| {
                    b.on_activate(cx.listener(move |_, _, _, cx| {
                        cx.emit(TimelineAction::TrackVisibilityRequested {
                            track_id: visibility_id.clone(),
                            visible: !visible,
                        })
                    }))
                }),
            )
            .child(
                self.button(
                    format!("lock-{}", track.id),
                    "Toggle layer lock",
                    if locked {
                        LucideIcon::Lock
                    } else {
                        LucideIcon::LockOpen
                    },
                    false,
                    !self.view_data.read_only,
                    cx,
                )
                .size(px(tokens::ControlSize::INLINE))
                .when(!self.view_data.read_only, |b| {
                    b.on_activate(cx.listener(move |_, _, _, cx| {
                        cx.emit(TimelineAction::TrackLockRequested {
                            track_id: lock_id.clone(),
                            locked: !locked,
                        })
                    }))
                }),
            )
            .into_any_element()
    }
    fn span(
        &self,
        track: &TimelineTrack,
        clip: Option<&TimelineClip>,
        start: u32,
        end: u32,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let editable = !self.view_data.read_only && !track.locked;
        let track_id = track.id.clone();
        let clip_id = clip.map(|c| c.id.clone());
        let (shown_start, shown_end) = self.preview_span(&track_id, clip_id.as_ref(), start, end);
        let left = self.time_x(shown_start);
        let width =
            ((shown_end - shown_start) as f32 * self.pixels_per_ms()).max(tokens::Space::SM);
        let mut span = div()
            .id(SharedString::from(format!(
                "span-{track_id}-{}",
                clip_id.as_deref().unwrap_or("layer")
            )))
            .debug_selector({
                let id = track_id.clone();
                let clip = clip_id.clone();
                move || format!("timeline-span-{id}-{}", clip.as_deref().unwrap_or("layer"))
            })
            .absolute()
            .left(px(left))
            .top(px(tokens::Space::SM))
            .w(px(width))
            .h(px(tokens::ControlSize::INLINE))
            .rounded(px(tokens::Radius::CONTROL))
            .bg(Color::BackgroundSelected.resolve(cx))
            .border_1()
            .border_color(Color::BorderSelected.resolve(cx))
            .overflow_hidden()
            .when(editable, |b| {
                b.cursor_grab().on_mouse_down(
                    MouseButton::Left,
                    cx.listener({
                        let id = track_id.clone();
                        let clip_id = clip_id.clone();
                        move |this, e: &MouseDownEvent, window, cx| {
                            this.focus_handle.focus(window, cx);
                            this.drag = Some(Drag::Span {
                                origin: e.position,
                                track_id: id.clone(),
                                clip_id: clip_id.clone(),
                                start,
                                end,
                                trim: Trim::Move,
                                delta: 0,
                            });
                            cx.stop_propagation();
                        }
                    }),
                )
            })
            .when_some(clip, |b, clip| {
                b.child(
                    div()
                        .px_2()
                        .truncate()
                        .typography(TypographyToken::BodyMedium)
                        .child(clip.name.clone()),
                )
            });
        if let Some(clip) = clip
            && editable
        {
            let clip_id = clip.id.clone();
            let track_id = track.id.clone();
            span = span.on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, _, _, cx| {
                    this.overlay = Some(Overlay::Easing(TimelineEasingTarget::Clip {
                        track_id: track_id.clone(),
                        clip_id: clip_id.clone(),
                    }));
                    cx.stop_propagation();
                    cx.notify();
                }),
            );
        }
        if editable {
            for (trim, right) in [(Trim::Start, false), (Trim::End, true)] {
                let track_id = track_id.clone();
                let clip_id = clip_id.clone();
                span = span.child(
                    div()
                        .id(if right { "end" } else { "start" })
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .w(px(tokens::Space::XS))
                        .when(right, |b| b.right_0())
                        .when(!right, |b| b.left_0())
                        .bg(Color::BorderSelected.resolve(cx))
                        .cursor_ew_resize()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, e: &MouseDownEvent, window, cx| {
                                this.focus_handle.focus(window, cx);
                                this.drag = Some(Drag::Span {
                                    origin: e.position,
                                    track_id: track_id.clone(),
                                    clip_id: clip_id.clone(),
                                    start,
                                    end,
                                    trim,
                                    delta: 0,
                                });
                                cx.stop_propagation();
                            }),
                        ),
                );
            }
        }
        span.into_any_element()
    }
    fn property_row(
        &self,
        track: &TimelineTrack,
        property: &TimelineProperty,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let track_id = track.id.clone();
        let property_id = property.id.clone();
        let enabled = !track.locked && !self.view_data.read_only;
        let mut lane = self
            .lane(
                SharedString::from(format!("lane-{track_id}-{property_id}")),
                cx,
            )
            .debug_selector({
                let id = property.id.clone();
                move || format!("timeline-property-lane-{id}")
            });
        let mut sorted: Vec<_> = property.keyframes.iter().collect();
        sorted.sort_by_key(|k| k.time_ms);
        for pair in sorted.windows(2) {
            let from = pair[0];
            let to = pair[1];
            let left = self.time_x(self.preview_key_time(from));
            let width = self.time_x(self.preview_key_time(to)) - left;
            if width > 0. {
                let id = from.id.clone();
                lane = lane.child(
                    div()
                        .id(SharedString::from(format!("easing-{id}")))
                        .absolute()
                        .left(px(left))
                        .top(px(ROW / 2. - tokens::Space::XS))
                        .w(px(width))
                        .h(px(tokens::Space::SM))
                        .cursor_pointer()
                        .child(
                            div()
                                .absolute()
                                .top(px(tokens::Space::XS))
                                .w_full()
                                .h(px(1.))
                                .bg(Color::BorderSelected.resolve(cx)),
                        )
                        .tooltip({
                            let easing = from.easing.label();
                            move |window, cx| Tooltip::new(easing).build(window, cx)
                        })
                        .when(enabled, |b| {
                            b.on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    this.overlay = Some(Overlay::Easing(
                                        TimelineEasingTarget::Keyframes(vec![id.clone()]),
                                    ));
                                    this.easing_error = None;
                                    cx.stop_propagation();
                                    cx.notify();
                                }),
                            )
                        }),
                );
            }
        }
        for key in &property.keyframes {
            let id = key.id.clone();
            let selected = self.view_data.selected_keyframes.contains(&id);
            let time = key.time_ms;
            let x = self.time_x(self.preview_key_time(key));
            let record = id.clone();
            lane = lane.child(
                self.button(
                    format!("key-{id}"),
                    format!(
                        "{} · {}{} · {}",
                        property.name,
                        self.view_data.time_unit.format(time),
                        self.view_data.time_unit.label(),
                        key.value
                    ),
                    LucideIcon::Diamond,
                    selected,
                    true,
                    cx,
                )
                .size(px(tokens::ControlSize::INLINE))
                .absolute()
                .left(px(x - tokens::Space::SM))
                .top(px((ROW - tokens::ControlSize::INLINE) / 2.))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener({
                        let id = id.clone();
                        move |this, e: &MouseDownEvent, window, cx| {
                            this.focus_handle.focus(window, cx);
                            this.begin_keys(id.clone(), e, enabled, cx);
                            if e.click_count == 2 {
                                cx.emit(TimelineAction::SeekRequested { time_ms: time });
                            }
                            cx.stop_propagation();
                        }
                    }),
                )
                .on_action(
                    cx.listener(move |this, _: &crate::atoms::ActivateControl, _, cx| {
                        cx.emit(TimelineAction::KeyframeSelectionRequested {
                            keyframe_ids: if this.view_data.selected_keyframes.contains(&id) {
                                this.view_data.selected_keyframes.clone()
                            } else {
                                vec![id.clone()]
                            },
                        })
                    }),
                )
                .child(track_bounds(cx.entity(), move |this, bounds| {
                    this.key_bounds.push((record.clone(), bounds))
                })),
            );
        }
        let prop_selector = property.id.clone();
        h_flex()
            .debug_selector(move || format!("timeline-property-row-{prop_selector}"))
            .w_full()
            .h(px(ROW))
            .flex_none()
            .border_b_1()
            .border_color(Color::Border.resolve(cx).opacity(0.4))
            .child(
                h_flex()
                    .w(px(RAIL))
                    .h_full()
                    .flex_none()
                    .pl_6()
                    .pr_1()
                    .gap_1()
                    .border_r_1()
                    .border_color(Color::Border.resolve(cx))
                    .child(
                        truncating_label(property.name.clone())
                            .text_color(Color::TextSecondary.resolve(cx)),
                    )
                    .child(
                        div()
                            .w(px(tokens::InputGeometry::MULTI_CELL_WIDTH))
                            .truncate()
                            .text_right()
                            .text_color(Color::TextTertiary.resolve(cx))
                            .child(property.value.clone()),
                    )
                    .child(
                        self.button(
                            format!("add-{}", property.id),
                            "Add property keyframe",
                            LucideIcon::DiamondPlus,
                            false,
                            enabled,
                            cx,
                        )
                        .size(px(tokens::ControlSize::INLINE))
                        .when(enabled, |b| {
                            b.on_activate(cx.listener(move |this, _, _, cx| {
                                cx.emit(TimelineAction::PropertyKeyframeRequested {
                                    track_id: track_id.clone(),
                                    property_id: property_id.clone(),
                                    time_ms: this.view_data.current_time_ms,
                                })
                            }))
                        }),
                    ),
            )
            .child(lane.child(self.playhead(cx)))
            .into_any_element()
    }
    pub(super) fn render_tracks(&self, _window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let mut rows = v_flex()
            .id("timeline-rows")
            .debug_selector(|| "timeline-rows".to_owned())
            .relative()
            .w_full()
            .flex_1()
            .min_h(px(0.))
            .overflow_y_scroll()
            .track_scroll(&self.rows_scroll)
            .child(track_bounds(cx.entity(), |this, bounds| {
                this.body_bounds = bounds
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, e: &MouseDownEvent, window, cx| {
                    if e.position.x > this.ruler_bounds.left() {
                        this.focus_handle.focus(window, cx);
                        this.drag = Some(Drag::Marquee {
                            origin: e.position,
                            current: e.position,
                            additive: e.modifiers.shift,
                        });
                        cx.notify();
                    }
                }),
            );
        for track in &self.view_data.tracks {
            let mut lane = self.lane(SharedString::from(format!("track-lane-{}", track.id)), cx);
            if let Some((start, end)) = track.span() {
                lane = lane.child(self.span(track, None, start, end, cx));
            }
            rows = rows.child(
                h_flex()
                    .w_full()
                    .h(px(ROW))
                    .flex_none()
                    .bg(if track.selected {
                        Color::BackgroundSelected.resolve(cx)
                    } else {
                        Color::BackgroundSecondary.resolve(cx)
                    })
                    .border_b_1()
                    .border_color(Color::Border.resolve(cx))
                    .child(self.track_label(track, cx))
                    .child(lane.child(self.playhead(cx))),
            );
            if track.expanded {
                for property in &track.properties {
                    rows = rows.child(self.property_row(track, property, cx));
                }
                for clip in &track.clips {
                    rows = rows.child(
                        h_flex()
                            .w_full()
                            .h(px(ROW))
                            .flex_none()
                            .border_b_1()
                            .border_color(Color::Border.resolve(cx).opacity(0.4))
                            .child(
                                h_flex()
                                    .w(px(RAIL))
                                    .h_full()
                                    .flex_none()
                                    .pl_6()
                                    .pr_2()
                                    .gap_1()
                                    .border_r_1()
                                    .border_color(Color::Border.resolve(cx))
                                    .child(render_lucide_icon(
                                        LucideIcon::Clapperboard,
                                        Color::TextTertiary.resolve(cx),
                                        tokens::IconSize::SM,
                                    ))
                                    .child(truncating_label(clip.name.clone())),
                            )
                            .child(
                                self.lane(SharedString::from(format!("clip-lane-{}", clip.id)), cx)
                                    .child(self.span(
                                        track,
                                        Some(clip),
                                        clip.start_ms,
                                        clip.end_ms,
                                        cx,
                                    ))
                                    .child(self.playhead(cx)),
                            ),
                    );
                }
            }
        }
        if self.view_data.tracks.is_empty() && self.show_empty_state {
            rows = rows.child(
                v_flex()
                    .debug_selector(|| "timeline-empty-card".to_owned())
                    .w_full()
                    .flex_1()
                    .min_h(px(ROW * 3.))
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .p_4()
                    .child(render_lucide_icon(
                        LucideIcon::Clapperboard,
                        Color::TextTertiary.resolve(cx),
                        tokens::IconSize::MD,
                    ))
                    .child(
                        div()
                            .typography(TypographyToken::BodyLargeStrong)
                            .child("Build your first animation"),
                    )
                    .child(div().text_color(Color::TextSecondary.resolve(cx)).child(
                        "Select a layer, then add a property keyframe or an animation preset.",
                    ))
                    .child(
                        self.button(
                            "dismiss-empty",
                            "Dismiss guidance",
                            LucideIcon::X,
                            false,
                            true,
                            cx,
                        )
                        .on_activate(cx.listener(|this, _, _, cx| {
                            this.show_empty_state = false;
                            cx.emit(TimelineAction::EmptyStateDismissed);
                            cx.notify();
                        })),
                    ),
            );
        }
        if let Some(Drag::Marquee {
            origin, current, ..
        }) = self.drag
        {
            let left = origin.x.min(current.x) - self.body_bounds.left();
            let top = origin.y.min(current.y) - self.body_bounds.top();
            rows = rows.child(
                div()
                    .absolute()
                    .left(left)
                    .top(top)
                    .w((origin.x - current.x).abs())
                    .h((origin.y - current.y).abs())
                    .border_1()
                    .border_color(Color::BorderSelected.resolve(cx))
                    .bg(Color::BackgroundSelected.resolve(cx).opacity(0.3)),
            );
        }
        rows.into_any_element()
    }
}
