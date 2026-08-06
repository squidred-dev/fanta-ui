//! The Timeline story: mock host state, reducer, and knobs.

use crate::*;

use super::knobs::{self, KnobOption};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TimelineNamedState {
    Default,
}

impl TimelineNamedState {
    pub(crate) const ALL: [Self; 1] = [Self::Default];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
        }
    }
}

pub(crate) struct TimelineScreen {
    pub(crate) timeline: Entity<Timeline>,
    pub(crate) view_data: TimelineViewData,
    pub(crate) last_action: SharedString,
    pub(crate) named_state: TimelineNamedState,
}

impl TimelineScreen {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        let view_data = TimelineViewData::default();
        let timeline = cx.new(|cx| Timeline::new("storybook-timeline", view_data.clone(), cx));
        Self {
            timeline,
            view_data,
            last_action: "Ready — play, loop, seek, zoom, add a keyframe, or ask the agent".into(),
            named_state: TimelineNamedState::Default,
        }
    }

    fn fixture(state: TimelineNamedState) -> TimelineViewData {
        match state {
            TimelineNamedState::Default => TimelineViewData::default(),
        }
    }

    pub(crate) fn apply_named_state(
        &mut self,
        state: TimelineNamedState,
        cx: &mut Context<Storybook>,
    ) {
        self.named_state = state;
        self.view_data = Self::fixture(state);
        let view_data = self.view_data.clone();
        self.timeline
            .update(cx, |timeline, cx| timeline.set_view_data(view_data, cx));
        self.last_action = format!("Story applied the {} Timeline state", state.label()).into();
        cx.notify();
    }

    pub(crate) fn handle_action(
        &mut self,
        timeline: Entity<Timeline>,
        action: &TimelineAction,
        cx: &mut Context<Storybook>,
    ) {
        match action {
            TimelineAction::PlayStateChangeRequested { playing } => {
                self.view_data.playing = *playing;
                self.last_action = if *playing {
                    "Host started timeline playback".into()
                } else {
                    "Host paused timeline playback".into()
                };
            }
            TimelineAction::LoopChangeRequested { looping } => {
                self.view_data.looping = *looping;
                self.last_action = format!("Host set timeline looping to {looping}").into();
            }
            TimelineAction::AddKeyframeRequested { time_ms } => {
                self.last_action = format!("Host inserted a keyframe at {time_ms} ms").into();
            }
            TimelineAction::SeekRequested { time_ms } => {
                self.view_data.current_time_ms = (*time_ms).min(self.view_data.duration_ms);
                self.last_action = format!(
                    "Host moved the playhead to {} ms",
                    self.view_data.current_time_ms
                )
                .into();
            }
            TimelineAction::ZoomChangeRequested { zoom } => {
                self.view_data.zoom = zoom.clamp(0.1, 2.);
                self.last_action =
                    format!("Host changed timeline zoom to {:.0}%", zoom * 100.).into();
            }
            TimelineAction::AskAgentRequested => {
                self.last_action = "Host opened the animation agent".into();
            }
            TimelineAction::EmptyStateDismissed => {
                self.last_action = "Dismissed the empty timeline guidance".into();
            }
            TimelineAction::HelpRequested => {
                self.last_action = "Host opened timeline help".into();
            }
        }
        timeline.update(cx, |timeline, cx| {
            timeline.set_view_data(self.view_data.clone(), cx);
        });
        cx.notify();
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
                "NAMED STATE",
                TimelineNamedState::ALL.map(|state| KnobOption::new(state, state.label())),
                self.timeline_screen.named_state,
                |this, state, _, cx| this.timeline_screen.apply_named_state(state, cx),
                cx,
            )],
            cx,
        )
    }
}
