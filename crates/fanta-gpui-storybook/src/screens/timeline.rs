//! Mock document adapter for the controlled Motion timeline.
mod fixtures;
mod reducer;
use super::knobs::{self, KnobOption};
use crate::*;
use fanta_gpui::timeline::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TimelineNamedState {
    Default,
    Empty,
    ManyLayers,
    ReadOnly,
}
impl TimelineNamedState {
    pub(crate) const ALL: [Self; 4] =
        [Self::Default, Self::Empty, Self::ManyLayers, Self::ReadOnly];
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Default => "Animated",
            Self::Empty => "Empty",
            Self::ManyLayers => "Many layers",
            Self::ReadOnly => "Read only",
        }
    }
}
pub(crate) struct TimelineScreen {
    pub(crate) timeline: Entity<Timeline>,
    pub(crate) view_data: TimelineViewData,
    pub(crate) last_action: SharedString,
    pub(crate) named_state: TimelineNamedState,
    playback_task: Option<gpui::Task<()>>,
    direction: i64,
    next_id: u64,
}
impl TimelineScreen {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        let view_data = fixtures::fixture(TimelineNamedState::Default);
        let timeline = cx.new(|cx| Timeline::new("storybook-timeline", view_data.clone(), cx));
        Self {
            timeline,
            view_data,
            last_action: "Select keyframes · drag to retime · edit easing between keys".into(),
            named_state: TimelineNamedState::Default,
            playback_task: None,
            direction: 1,
            next_id: 0,
        }
    }
    pub(crate) fn apply_named_state(
        &mut self,
        state: TimelineNamedState,
        cx: &mut Context<Storybook>,
    ) {
        self.playback_task = None;
        self.named_state = state;
        self.view_data = fixtures::fixture(state);
        self.echo(cx);
        self.last_action = format!("Loaded {} timeline", state.label()).into();
        cx.notify();
    }
    fn echo(&self, cx: &mut Context<Storybook>) {
        self.timeline
            .update(cx, |t, cx| t.set_view_data(self.view_data.clone(), cx));
    }
    fn add_key(
        &mut self,
        track_id: Option<&SharedString>,
        property_id: Option<&SharedString>,
        time: u32,
    ) {
        for track in &mut self.view_data.tracks {
            if track.locked || !track_id.map_or(track.selected, |id| *id == track.id) {
                continue;
            }
            if track.properties.is_empty() {
                track.properties.push(TimelineProperty {
                    id: format!("{}-opacity", track.id).into(),
                    name: "Opacity".into(),
                    value: "100%".into(),
                    keyframes: vec![],
                });
            }
            for p in &mut track.properties {
                if property_id.is_some_and(|id| *id != p.id) {
                    continue;
                }
                if p.keyframes.iter().any(|k| k.time_ms == time) {
                    continue;
                }
                self.next_id += 1;
                p.keyframes.push(TimelineKeyframe {
                    id: format!("new-key-{}", self.next_id).into(),
                    time_ms: time,
                    value: p.value.clone(),
                    easing: TimelineEasing::default(),
                });
            }
        }
    }
    fn start_playback(&mut self, cx: &mut Context<Storybook>) {
        if self.playback_task.is_some() || !self.view_data.playing {
            return;
        }
        self.playback_task = Some(cx.spawn(async move |story, cx| {
            let mut previous = std::time::Instant::now();
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(16))
                    .await;
                let now = std::time::Instant::now();
                let elapsed = now.duration_since(previous).as_millis().min(100) as i64;
                previous = now;
                let keep = story
                    .update(cx, |story, cx| {
                        let screen = &mut story.timeline_screen;
                        if !screen.view_data.playing {
                            return false;
                        }
                        let duration = screen.view_data.duration_ms as i64;
                        let mut time =
                            screen.view_data.current_time_ms as i64 + elapsed * screen.direction;
                        match screen.view_data.playback_mode() {
                            TimelinePlayback::Once => {
                                if time >= duration {
                                    time = duration;
                                    screen.view_data.playing = false;
                                    screen.playback_task = None;
                                }
                            }
                            TimelinePlayback::Loop => {
                                time = time.rem_euclid(duration);
                            }
                            TimelinePlayback::PingPong => {
                                if time >= duration {
                                    time = duration;
                                    screen.direction = -1;
                                } else if time <= 0 {
                                    time = 0;
                                    screen.direction = 1;
                                }
                            }
                        }
                        screen.view_data.current_time_ms = time.clamp(0, duration) as u32;
                        screen.echo(cx);
                        cx.notify();
                        screen.view_data.playing
                    })
                    .unwrap_or(false);
                if !keep {
                    break;
                }
            }
        }));
    }
}
impl Storybook {
    pub(crate) fn render_timeline_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-timeline",
            self.timeline_screen.timeline.clone().into_any_element(),
            self.timeline_screen.last_action.clone(),
            cx,
        )
    }
    pub(crate) fn render_timeline_knobs(&self, cx: &mut Context<Self>) -> AnyElement {
        knobs::knobs_panel(
            "timeline-story-knobs",
            vec![knobs::enum_knob_row(
                "timeline-knob-state",
                "SCENE",
                TimelineNamedState::ALL.map(|state| KnobOption::new(state, state.label())),
                self.timeline_screen.named_state,
                |this, state, _, cx| this.timeline_screen.apply_named_state(state, cx),
                cx,
            )],
            cx,
        )
    }
}
