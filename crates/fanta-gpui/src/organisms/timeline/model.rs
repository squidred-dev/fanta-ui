use gpui::SharedString;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TimelinePlayback {
    Once,
    #[default]
    Loop,
    PingPong,
}
impl TimelinePlayback {
    pub const ALL: [Self; 3] = [Self::Once, Self::Loop, Self::PingPong];
    pub const fn label(self) -> &'static str {
        match self {
            Self::Once => "Once",
            Self::Loop => "Loop",
            Self::PingPong => "Ping-pong",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TimelineTimeUnit {
    #[default]
    Seconds,
    Milliseconds,
}
impl TimelineTimeUnit {
    pub fn format(self, ms: u32) -> String {
        match self {
            Self::Seconds => format!("{:.2}", ms as f64 / 1000.),
            Self::Milliseconds => ms.to_string(),
        }
    }
    pub const fn label(self) -> &'static str {
        match self {
            Self::Seconds => "s",
            Self::Milliseconds => "ms",
        }
    }
    pub fn parse(self, value: &str) -> Option<u32> {
        let value =
            value.trim().parse::<f64>().ok()? * if self == Self::Seconds { 1000. } else { 1. };
        (value.is_finite() && value >= 0. && value <= u32::MAX as f64).then(|| value.round() as u32)
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub enum TimelineEasing {
    Linear,
    EaseIn,
    #[default]
    EaseOut,
    EaseInOut,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    Hold,
    Gentle,
    Quick,
    Bouncy,
    Slow,
    CubicBezier([f32; 4]),
    Spring {
        bounce: f32,
    },
}
impl TimelineEasing {
    pub fn presets() -> Vec<Self> {
        vec![
            Self::Linear,
            Self::EaseIn,
            Self::EaseOut,
            Self::EaseInOut,
            Self::EaseInBack,
            Self::EaseOutBack,
            Self::EaseInOutBack,
            Self::Hold,
            Self::Gentle,
            Self::Quick,
            Self::Bouncy,
            Self::Slow,
        ]
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::EaseIn => "Ease in",
            Self::EaseOut => "Ease out",
            Self::EaseInOut => "Ease in and out",
            Self::EaseInBack => "Ease in back",
            Self::EaseOutBack => "Ease out back",
            Self::EaseInOutBack => "Ease in and out back",
            Self::Hold => "Hold",
            Self::Gentle => "Gentle spring",
            Self::Quick => "Quick spring",
            Self::Bouncy => "Bouncy spring",
            Self::Slow => "Slow spring",
            Self::CubicBezier(_) => "Custom Bézier",
            Self::Spring { .. } => "Custom spring",
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineKeyframe {
    pub id: SharedString,
    pub time_ms: u32,
    pub value: SharedString,
    pub easing: TimelineEasing,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineProperty {
    pub id: SharedString,
    pub name: SharedString,
    pub value: SharedString,
    pub keyframes: Vec<TimelineKeyframe>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineClip {
    pub id: SharedString,
    pub name: SharedString,
    pub start_ms: u32,
    pub end_ms: u32,
    pub easing: TimelineEasing,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineTrack {
    pub id: SharedString,
    pub name: SharedString,
    pub depth: usize,
    pub expanded: bool,
    pub selected: bool,
    pub locked: bool,
    pub visible: bool,
    pub properties: Vec<TimelineProperty>,
    pub clips: Vec<TimelineClip>,
}
impl TimelineTrack {
    pub fn new(id: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            depth: 0,
            expanded: true,
            selected: false,
            locked: false,
            visible: true,
            properties: vec![],
            clips: vec![],
        }
    }
    pub fn span(&self) -> Option<(u32, u32)> {
        let times: Vec<_> = self
            .properties
            .iter()
            .flat_map(|p| p.keyframes.iter().map(|k| k.time_ms))
            .chain(self.clips.iter().flat_map(|c| [c.start_ms, c.end_ms]))
            .collect();
        Some((*times.iter().min()?, *times.iter().max()?))
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineComment {
    pub id: SharedString,
    pub time_ms: u32,
    pub label: SharedString,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TimelinePreset {
    pub id: SharedString,
    pub name: SharedString,
}

/// Host-owned animation data. Identifiers are opaque; the component never edits a document.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineViewData {
    pub duration_ms: u32,
    pub current_time_ms: u32,
    pub playing: bool,
    /// Compatibility field: false selects Once; use playback for Loop/PingPong.
    pub looping: bool,
    pub playback: TimelinePlayback,
    pub auto_keyframe: bool,
    pub time_unit: TimelineTimeUnit,
    pub zoom: f32,
    pub snapping: bool,
    pub tracks: Vec<TimelineTrack>,
    pub selected_keyframes: Vec<SharedString>,
    pub comments: Vec<TimelineComment>,
    pub presets: Vec<TimelinePreset>,
    pub read_only: bool,
    pub height: u16,
}
impl Default for TimelineViewData {
    fn default() -> Self {
        Self {
            duration_ms: 2000,
            current_time_ms: 0,
            playing: false,
            looping: true,
            playback: TimelinePlayback::Loop,
            auto_keyframe: false,
            time_unit: TimelineTimeUnit::Seconds,
            zoom: 1.,
            snapping: true,
            tracks: vec![],
            selected_keyframes: vec![],
            comments: vec![],
            presets: vec![],
            read_only: false,
            height: 360,
        }
    }
}
impl TimelineViewData {
    pub fn normalized(mut self) -> Self {
        self.duration_ms = self.duration_ms.max(1);
        self.current_time_ms = self.current_time_ms.min(self.duration_ms);
        self.zoom = if self.zoom.is_finite() {
            self.zoom.clamp(0.25, 16.)
        } else {
            1.
        };
        self.height = self.height.clamp(160, 960);
        self
    }
    pub fn playback_mode(&self) -> TimelinePlayback {
        if self.looping {
            self.playback
        } else {
            TimelinePlayback::Once
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineKeyframeTime {
    pub id: SharedString,
    pub time_ms: u32,
}
#[derive(Clone, Debug, PartialEq)]
pub enum TimelineEasingTarget {
    Keyframes(Vec<SharedString>),
    Clip {
        track_id: SharedString,
        clip_id: SharedString,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub enum TimelineAction {
    PlayStateChangeRequested {
        playing: bool,
    },
    LoopChangeRequested {
        looping: bool,
    },
    PlaybackChangeRequested {
        playback: TimelinePlayback,
    },
    AutoKeyframeChangeRequested {
        enabled: bool,
    },
    AddKeyframeRequested {
        time_ms: u32,
    },
    PropertyKeyframeRequested {
        track_id: SharedString,
        property_id: SharedString,
        time_ms: u32,
    },
    SeekRequested {
        time_ms: u32,
    },
    DurationChangeRequested {
        duration_ms: u32,
    },
    TimeUnitChangeRequested {
        unit: TimelineTimeUnit,
    },
    ZoomChangeRequested {
        zoom: f32,
    },
    SnappingChangeRequested {
        enabled: bool,
    },
    TrackSelectionRequested {
        track_ids: Vec<SharedString>,
    },
    TrackRenameRequested {
        track_id: SharedString,
        name: SharedString,
    },
    TrackExpansionRequested {
        track_id: SharedString,
        expanded: bool,
    },
    ExpandAllRequested {
        expanded: bool,
    },
    TrackVisibilityRequested {
        track_id: SharedString,
        visible: bool,
    },
    TrackLockRequested {
        track_id: SharedString,
        locked: bool,
    },
    KeyframeSelectionRequested {
        keyframe_ids: Vec<SharedString>,
    },
    KeyframesMoveRequested {
        keyframes: Vec<TimelineKeyframeTime>,
    },
    KeyframesDeleteRequested {
        keyframe_ids: Vec<SharedString>,
    },
    KeyframesDuplicateRequested {
        keyframe_ids: Vec<SharedString>,
        offset_ms: u32,
    },
    TrackTimingChangeRequested {
        track_id: SharedString,
        start_ms: u32,
        end_ms: u32,
    },
    ClipTimingChangeRequested {
        track_id: SharedString,
        clip_id: SharedString,
        start_ms: u32,
        end_ms: u32,
    },
    EasingChangeRequested {
        target: TimelineEasingTarget,
        easing: TimelineEasing,
    },
    PresetApplyRequested {
        track_ids: Vec<SharedString>,
        preset_id: SharedString,
        time_ms: u32,
    },
    CommentAddRequested {
        time_ms: u32,
    },
    CommentOpenRequested {
        comment_id: SharedString,
    },
    HeightChangeRequested {
        height: u16,
    },
    CollapsedChanged {
        collapsed: bool,
    },
    /// Legacy host command, no built-in Agent UI.
    AskAgentRequested,
    EmptyStateDismissed,
    HelpRequested,
}
