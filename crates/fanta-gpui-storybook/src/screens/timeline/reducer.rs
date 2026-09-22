use super::*;

impl TimelineScreen {
    pub(crate) fn handle_action(
        &mut self,
        _timeline: Entity<Timeline>,
        action: &TimelineAction,
        cx: &mut Context<Storybook>,
    ) {
        self.last_action = format!("{action:?}").into();
        match action {
            TimelineAction::PlayStateChangeRequested { playing } => {
                self.view_data.playing = *playing;
                self.direction = 1;
                if !playing {
                    self.playback_task = None;
                } else if self.view_data.current_time_ms == self.view_data.duration_ms {
                    self.view_data.current_time_ms = 0;
                }
            }
            TimelineAction::LoopChangeRequested { looping } => self.view_data.looping = *looping,
            TimelineAction::PlaybackChangeRequested { playback } => {
                self.view_data.playback = *playback;
                self.view_data.looping = *playback != TimelinePlayback::Once;
                self.direction = 1;
            }
            TimelineAction::AutoKeyframeChangeRequested { enabled } => {
                self.view_data.auto_keyframe = *enabled
            }
            TimelineAction::SeekRequested { time_ms } => {
                self.view_data.current_time_ms = (*time_ms).min(self.view_data.duration_ms)
            }
            TimelineAction::DurationChangeRequested { duration_ms } => {
                // Mock document policy: refuse truncation of existing keys/clips.
                let last = self
                    .view_data
                    .tracks
                    .iter()
                    .filter_map(|t| t.span().map(|(_, end)| end))
                    .max()
                    .unwrap_or(1);
                self.view_data.duration_ms = (*duration_ms).max(last).max(1);
                self.view_data.current_time_ms = self
                    .view_data
                    .current_time_ms
                    .min(self.view_data.duration_ms);
            }
            TimelineAction::TimeUnitChangeRequested { unit } => self.view_data.time_unit = *unit,
            TimelineAction::ZoomChangeRequested { zoom } => {
                self.view_data.zoom = zoom.clamp(0.25, 16.)
            }
            TimelineAction::SnappingChangeRequested { enabled } => {
                self.view_data.snapping = *enabled
            }
            TimelineAction::AddKeyframeRequested { time_ms } => self.add_key(None, None, *time_ms),
            TimelineAction::PropertyKeyframeRequested {
                track_id,
                property_id,
                time_ms,
            } => self.add_key(Some(track_id), Some(property_id), *time_ms),
            TimelineAction::TrackSelectionRequested { track_ids } => {
                for track in &mut self.view_data.tracks {
                    track.selected = track_ids.contains(&track.id);
                }
                self.view_data.selected_keyframes = self
                    .view_data
                    .tracks
                    .iter()
                    .filter(|t| t.selected)
                    .flat_map(|t| {
                        t.properties
                            .iter()
                            .flat_map(|p| p.keyframes.iter().map(|k| k.id.clone()))
                    })
                    .collect();
            }
            TimelineAction::TrackRenameRequested { track_id, name } => {
                if !self.view_data.read_only
                    && !name.trim().is_empty()
                    && let Some(track) = self
                        .view_data
                        .tracks
                        .iter_mut()
                        .find(|t| t.id == *track_id && !t.locked)
                {
                    track.name = name.trim().to_owned().into();
                }
            }
            TimelineAction::TrackExpansionRequested { track_id, expanded } => {
                if let Some(t) = self.view_data.tracks.iter_mut().find(|t| t.id == *track_id) {
                    t.expanded = *expanded;
                }
            }
            TimelineAction::ExpandAllRequested { expanded } => {
                for t in &mut self.view_data.tracks {
                    t.expanded = *expanded;
                }
            }
            TimelineAction::TrackVisibilityRequested { track_id, visible } => {
                if let Some(t) = self.view_data.tracks.iter_mut().find(|t| t.id == *track_id) {
                    t.visible = *visible;
                }
            }
            TimelineAction::TrackLockRequested { track_id, locked } => {
                if let Some(t) = self.view_data.tracks.iter_mut().find(|t| t.id == *track_id) {
                    t.locked = *locked;
                }
            }
            TimelineAction::KeyframeSelectionRequested { keyframe_ids } => {
                self.view_data.selected_keyframes = keyframe_ids.clone()
            }
            TimelineAction::KeyframesMoveRequested { keyframes } => {
                for t in &mut self.view_data.tracks {
                    if t.locked {
                        continue;
                    }
                    for p in &mut t.properties {
                        for k in &mut p.keyframes {
                            if let Some(changed) = keyframes.iter().find(|v| v.id == k.id) {
                                k.time_ms = changed.time_ms.min(self.view_data.duration_ms);
                            }
                        }
                    }
                }
            }
            TimelineAction::KeyframesDeleteRequested { keyframe_ids } => {
                for t in &mut self.view_data.tracks {
                    if t.locked {
                        continue;
                    }
                    for p in &mut t.properties {
                        p.keyframes.retain(|k| !keyframe_ids.contains(&k.id));
                    }
                }
                self.view_data.selected_keyframes.clear();
            }
            TimelineAction::KeyframesDuplicateRequested {
                keyframe_ids,
                offset_ms,
            } => {
                let mut selected = vec![];
                for t in &mut self.view_data.tracks {
                    if t.locked {
                        continue;
                    }
                    for p in &mut t.properties {
                        let clones: Vec<_> = p
                            .keyframes
                            .iter()
                            .filter(|k| keyframe_ids.contains(&k.id))
                            .cloned()
                            .collect();
                        for mut k in clones {
                            self.next_id += 1;
                            k.id = format!("copy-key-{}", self.next_id).into();
                            k.time_ms = k
                                .time_ms
                                .saturating_add(*offset_ms)
                                .min(self.view_data.duration_ms);
                            selected.push(k.id.clone());
                            p.keyframes.push(k);
                        }
                    }
                }
                self.view_data.selected_keyframes = selected;
            }
            TimelineAction::TrackTimingChangeRequested {
                track_id,
                start_ms,
                end_ms,
            } => {
                if let Some(t) = self
                    .view_data
                    .tracks
                    .iter_mut()
                    .find(|t| t.id == *track_id && !t.locked)
                    && let Some((start, end)) = t.span()
                {
                    let retime = |time: u32| -> u32 {
                        if end == start {
                            *start_ms
                        } else {
                            (*start_ms as f64
                                + (time.saturating_sub(start)) as f64 / (end - start) as f64
                                    * end_ms.saturating_sub(*start_ms) as f64)
                                .round() as u32
                        }
                    };
                    for p in &mut t.properties {
                        for k in &mut p.keyframes {
                            k.time_ms = retime(k.time_ms);
                        }
                    }
                    for c in &mut t.clips {
                        c.start_ms = retime(c.start_ms);
                        c.end_ms = retime(c.end_ms);
                    }
                }
            }
            TimelineAction::ClipTimingChangeRequested {
                track_id,
                clip_id,
                start_ms,
                end_ms,
            } => {
                if let Some(t) = self
                    .view_data
                    .tracks
                    .iter_mut()
                    .find(|t| t.id == *track_id && !t.locked)
                    && let Some(c) = t.clips.iter_mut().find(|c| c.id == *clip_id)
                {
                    c.start_ms = *start_ms;
                    c.end_ms = *end_ms;
                }
            }
            TimelineAction::EasingChangeRequested { target, easing } => {
                for t in &mut self.view_data.tracks {
                    if t.locked {
                        continue;
                    }
                    match target {
                        TimelineEasingTarget::Keyframes(ids) => {
                            for p in &mut t.properties {
                                for k in &mut p.keyframes {
                                    if ids.contains(&k.id) {
                                        k.easing = easing.clone();
                                    }
                                }
                            }
                        }
                        TimelineEasingTarget::Clip { track_id, clip_id } => {
                            if *track_id == t.id
                                && let Some(c) = t.clips.iter_mut().find(|c| c.id == *clip_id)
                            {
                                c.easing = easing.clone();
                            }
                        }
                    }
                }
            }
            TimelineAction::PresetApplyRequested {
                track_ids,
                preset_id,
                time_ms,
            } => {
                if let Some(preset) = self.view_data.presets.iter().find(|p| p.id == *preset_id) {
                    for t in &mut self.view_data.tracks {
                        if track_ids.contains(&t.id) && !t.locked {
                            self.next_id += 1;
                            let end = time_ms.saturating_add(500);
                            self.view_data.duration_ms = self.view_data.duration_ms.max(end);
                            t.clips.push(TimelineClip {
                                id: format!("clip-{}", self.next_id).into(),
                                name: preset.name.clone(),
                                start_ms: *time_ms,
                                end_ms: end,
                                easing: TimelineEasing::default(),
                            });
                        }
                    }
                }
            }
            TimelineAction::CommentAddRequested { time_ms } => {
                self.next_id += 1;
                self.view_data.comments.push(TimelineComment {
                    id: format!("comment-{}", self.next_id).into(),
                    time_ms: *time_ms,
                    label: "New review comment".into(),
                });
            }
            TimelineAction::HeightChangeRequested { height } => self.view_data.height = *height,
            TimelineAction::CommentOpenRequested { .. }
            | TimelineAction::CollapsedChanged { .. }
            | TimelineAction::AskAgentRequested
            | TimelineAction::EmptyStateDismissed
            | TimelineAction::HelpRequested => {}
        }
        self.echo(cx);
        self.start_playback(cx);
        cx.notify();
    }
}
