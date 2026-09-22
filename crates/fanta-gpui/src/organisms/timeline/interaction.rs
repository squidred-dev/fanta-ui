use super::*;

impl Timeline {
    pub(super) fn editable_keys(&self) -> Vec<TimelineKeyframeTime> {
        self.view_data
            .tracks
            .iter()
            .filter(|t| !t.locked)
            .flat_map(|t| {
                t.properties.iter().flat_map(|p| {
                    p.keyframes.iter().map(|k| TimelineKeyframeTime {
                        id: k.id.clone(),
                        time_ms: k.time_ms,
                    })
                })
            })
            .collect()
    }
    pub(super) fn preview_key_time(&self, key: &TimelineKeyframe) -> u32 {
        if let Some(Drag::Keys { times, delta, .. }) = &self.drag
            && let Some(k) = times.iter().find(|k| k.id == key.id)
        {
            return (k.time_ms as i64 + delta) as u32;
        }
        key.time_ms
    }
    pub(super) fn preview_span(
        &self,
        track: &SharedString,
        clip: Option<&SharedString>,
        start: u32,
        end: u32,
    ) -> (u32, u32) {
        if let Some(Drag::Span {
            track_id,
            clip_id,
            start,
            end,
            trim,
            delta,
            ..
        }) = &self.drag
            && track_id == track
            && clip_id.as_ref() == clip
        {
            return match trim {
                Trim::Move => ((*start as i64 + delta) as u32, (*end as i64 + delta) as u32),
                Trim::Start => ((*start as i64 + delta) as u32, *end),
                Trim::End => (*start, (*end as i64 + delta) as u32),
            };
        }
        (start, end)
    }
    pub(super) fn begin_keys(
        &mut self,
        id: SharedString,
        event: &MouseDownEvent,
        editable: bool,
        cx: &mut Context<Self>,
    ) {
        let mut selected = self.view_data.selected_keyframes.clone();
        if event.modifiers.shift {
            if selected.contains(&id) {
                selected.retain(|k| k != &id);
            } else {
                selected.push(id.clone());
            }
        } else if !selected.contains(&id) {
            selected = vec![id.clone()];
        }
        cx.emit(TimelineAction::KeyframeSelectionRequested {
            keyframe_ids: selected.clone(),
        });
        if editable && selected.contains(&id) {
            let times = self
                .editable_keys()
                .into_iter()
                .filter(|k| selected.contains(&k.id))
                .collect();
            self.drag = Some(Drag::Keys {
                origin: event.position,
                times,
                delta: 0,
            });
        }
        cx.notify();
    }
    fn snapped_delta(&self, raw: i64, anchor: u32, excluded: &[SharedString]) -> i64 {
        if !self.view_data.snapping {
            return raw;
        }
        let target = anchor as i64 + raw;
        let threshold = (6. / self.pixels_per_ms()).ceil() as i64;
        let nearest = self
            .editable_keys()
            .into_iter()
            .filter(|k| !excluded.contains(&k.id))
            .map(|k| k.time_ms)
            .chain([
                0,
                self.view_data.duration_ms,
                self.view_data.current_time_ms,
            ])
            .min_by_key(|time| (*time as i64 - target).abs());
        if let Some(time) = nearest
            && (time as i64 - target).abs() <= threshold
        {
            time as i64 - anchor as i64
        } else {
            raw
        }
    }
    pub(super) fn drag_move(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        let Some(mut drag) = self.drag.take() else {
            return;
        };
        match &mut drag {
            Drag::Seek => cx.emit(TimelineAction::SeekRequested {
                time_ms: self.time_at(position),
            }),
            Drag::Keys {
                origin,
                times,
                delta,
            } => {
                let raw = ((position.x - origin.x).as_f32() / self.pixels_per_ms()).round() as i64;
                if let (Some(first), Some(last)) = (
                    times.iter().map(|k| k.time_ms).min(),
                    times.iter().map(|k| k.time_ms).max(),
                ) {
                    let raw = self.snapped_delta(
                        raw,
                        first,
                        &times.iter().map(|k| k.id.clone()).collect::<Vec<_>>(),
                    );
                    *delta = raw.clamp(
                        -(first as i64),
                        self.view_data.duration_ms as i64 - last as i64,
                    );
                }
            }
            Drag::Span {
                origin,
                start,
                end,
                trim,
                delta,
                ..
            } => {
                let raw = ((position.x - origin.x).as_f32() / self.pixels_per_ms()).round() as i64;
                *delta = match trim {
                    Trim::Move => raw.clamp(
                        -(*start as i64),
                        self.view_data.duration_ms.saturating_sub(*end) as i64,
                    ),
                    Trim::Start => raw.clamp(
                        -(*start as i64),
                        end.saturating_sub(*start).saturating_sub(1) as i64,
                    ),
                    Trim::End => raw.clamp(
                        -((*end).saturating_sub(*start).saturating_sub(1) as i64),
                        self.view_data.duration_ms.saturating_sub(*end) as i64,
                    ),
                };
            }
            Drag::Duration { time } => {
                *time = (((position.x - self.ruler_bounds.left()).as_f32() + self.scroll_x
                    - GUTTER)
                    / self.pixels_per_ms())
                .round()
                .clamp(1., u32::MAX as f32) as u32
            }
            Drag::Height {
                origin,
                height,
                draft,
            } => {
                *draft = (*height as f32 + (*origin - position.y).as_f32())
                    .round()
                    .clamp(160., 960.) as u16
            }
            Drag::Marquee { current, .. } => *current = position,
        }
        self.drag = Some(drag);
        cx.notify();
    }
    pub(super) fn finish_drag(&mut self, cx: &mut Context<Self>) {
        let Some(drag) = self.drag.take() else {
            return;
        };
        match drag {
            Drag::Keys { times, delta, .. } if delta != 0 => {
                cx.emit(TimelineAction::KeyframesMoveRequested {
                    keyframes: times
                        .into_iter()
                        .map(|k| TimelineKeyframeTime {
                            id: k.id,
                            time_ms: (k.time_ms as i64 + delta) as u32,
                        })
                        .collect(),
                })
            }
            Drag::Span {
                track_id,
                clip_id,
                start,
                end,
                trim,
                delta,
                ..
            } if delta != 0 => {
                let (start_ms, end_ms) = match trim {
                    Trim::Move => ((start as i64 + delta) as u32, (end as i64 + delta) as u32),
                    Trim::Start => ((start as i64 + delta) as u32, end),
                    Trim::End => (start, (end as i64 + delta) as u32),
                };
                cx.emit(if let Some(clip_id) = clip_id {
                    TimelineAction::ClipTimingChangeRequested {
                        track_id,
                        clip_id,
                        start_ms,
                        end_ms,
                    }
                } else {
                    TimelineAction::TrackTimingChangeRequested {
                        track_id,
                        start_ms,
                        end_ms,
                    }
                });
            }
            Drag::Span {
                track_id,
                clip_id: Some(clip_id),
                trim: Trim::Move,
                delta: 0,
                ..
            } => {
                self.overlay = Some(Overlay::Easing(TimelineEasingTarget::Clip {
                    track_id,
                    clip_id,
                }));
                self.easing_error = None;
            }
            Drag::Duration { time } => {
                cx.emit(TimelineAction::DurationChangeRequested { duration_ms: time })
            }
            Drag::Height { draft, .. } => {
                cx.emit(TimelineAction::HeightChangeRequested { height: draft })
            }
            Drag::Marquee {
                origin,
                current,
                additive,
            } => {
                let mut ids = if additive {
                    self.view_data.selected_keyframes.clone()
                } else {
                    vec![]
                };
                let region = Bounds::from_corners(
                    point(origin.x.min(current.x), origin.y.min(current.y)),
                    point(origin.x.max(current.x), origin.y.max(current.y)),
                );
                for (id, bounds) in &self.key_bounds {
                    if region.contains(&bounds.center()) && !ids.contains(id) {
                        ids.push(id.clone());
                    }
                }
                cx.emit(TimelineAction::KeyframeSelectionRequested { keyframe_ids: ids });
            }
            _ => {}
        }
        cx.notify();
    }
    pub(super) fn wheel(&mut self, event: &ScrollWheelEvent, cx: &mut Context<Self>) {
        cx.stop_propagation();
        if self.overlay.is_some() || event.position.y < self.ruler_bounds.top() {
            return;
        }
        let delta = event.delta.pixel_delta(px(ROW));
        if event.modifiers.secondary() {
            let zoom = (self.view_data.zoom * 2_f32.powf(delta.y.as_f32() / 240.)).clamp(0.25, 16.);
            cx.emit(TimelineAction::ZoomChangeRequested { zoom });
        } else {
            let dx = if event.modifiers.shift {
                delta.y.as_f32()
            } else {
                delta.x.as_f32()
            };
            self.scroll_x = (self.scroll_x - dx).clamp(0., self.max_scroll());
            cx.notify();
        }
    }
    pub(super) fn key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.keystroke.key == "escape" {
            self.renaming_track = None;
            self.drag = None;
            self.overlay = None;
            self.easing_error = None;
            self.focus_handle.focus(window, cx);
            cx.stop_propagation();
            cx.notify();
            return;
        }
        if self.input_focused(window, cx) {
            return;
        }
        if event.keystroke.key == "f2" {
            if let Some(id) = self
                .view_data
                .tracks
                .iter()
                .find(|t| t.selected && !t.locked)
                .map(|t| t.id.clone())
            {
                self.begin_rename(id, window, cx);
                cx.stop_propagation();
            }
            return;
        }
        let modifier = event.keystroke.modifiers;
        if self.overlay.is_some() {
            return;
        }
        let selected: Vec<_> = self
            .editable_keys()
            .into_iter()
            .filter(|k| self.view_data.selected_keyframes.contains(&k.id))
            .collect();
        match event.keystroke.key.as_str() {
            "space" => cx.emit(TimelineAction::PlayStateChangeRequested {
                playing: !self.view_data.playing,
            }),
            "home" => cx.emit(TimelineAction::SeekRequested { time_ms: 0 }),
            "end" => cx.emit(TimelineAction::SeekRequested {
                time_ms: self.view_data.duration_ms,
            }),
            "a" if modifier.secondary() => cx.emit(TimelineAction::KeyframeSelectionRequested {
                keyframe_ids: self.editable_keys().into_iter().map(|k| k.id).collect(),
            }),
            "k" if !self.view_data.read_only && !self.selected_tracks().is_empty() => {
                cx.emit(TimelineAction::AddKeyframeRequested {
                    time_ms: self.view_data.current_time_ms,
                })
            }
            "backspace" | "delete" if !self.view_data.read_only && !selected.is_empty() => {
                cx.emit(TimelineAction::KeyframesDeleteRequested {
                    keyframe_ids: selected.into_iter().map(|k| k.id).collect(),
                })
            }
            "d" if modifier.secondary() && !self.view_data.read_only && !selected.is_empty() => cx
                .emit(TimelineAction::KeyframesDuplicateRequested {
                    keyframe_ids: selected.into_iter().map(|k| k.id).collect(),
                    offset_ms: 100,
                }),
            "left" | "right" => {
                let delta = if event.keystroke.key == "left" { -1 } else { 1 }
                    * if modifier.shift { 10 } else { 1 };
                if selected.is_empty() || self.view_data.read_only {
                    cx.emit(TimelineAction::SeekRequested {
                        time_ms: (self.view_data.current_time_ms as i64 + delta)
                            .clamp(0, self.view_data.duration_ms as i64)
                            as u32,
                    });
                } else {
                    let min = selected.iter().map(|k| k.time_ms).min().unwrap();
                    let max = selected.iter().map(|k| k.time_ms).max().unwrap();
                    let delta = delta.clamp(
                        -(min as i64),
                        self.view_data.duration_ms as i64 - max as i64,
                    );
                    cx.emit(TimelineAction::KeyframesMoveRequested {
                        keyframes: selected
                            .into_iter()
                            .map(|k| TimelineKeyframeTime {
                                id: k.id,
                                time_ms: (k.time_ms as i64 + delta) as u32,
                            })
                            .collect(),
                    });
                }
            }
            _ => return,
        }
        cx.stop_propagation();
    }
}
