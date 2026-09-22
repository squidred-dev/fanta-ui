//! Host-supplied read models and intents for the independent inspector tabs.
use crate::timeline::{TimelineEasing, TimelinePlayback};
use crate::toolbar::DrawToolbarOptions;
use gpui::SharedString;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectorChoice {
    pub id: SharedString,
    pub label: SharedString,
}
impl InspectorChoice {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MotionInspectorViewData {
    pub selection_name: SharedString,
    pub duration_ms: u32,
    pub delay_ms: u32,
    pub easing: TimelineEasing,
    pub playback: TimelinePlayback,
    pub playing: bool,
    pub auto_keyframe: bool,
    pub presets: Vec<InspectorChoice>,
    pub selected_preset: Option<SharedString>,
    pub animated_properties: Vec<InspectorChoice>,
    pub read_only: bool,
}
impl Default for MotionInspectorViewData {
    fn default() -> Self {
        Self {
            selection_name: SharedString::default(),
            duration_ms: 1000,
            delay_ms: 0,
            easing: TimelineEasing::EaseInOut,
            playback: TimelinePlayback::Once,
            playing: false,
            auto_keyframe: false,
            presets: Vec::new(),
            selected_preset: None,
            animated_properties: Vec::new(),
            read_only: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum MotionInspectorAction {
    DurationChangeRequested { duration_ms: u32 },
    DelayChangeRequested { delay_ms: u32 },
    EasingChangeRequested { easing: TimelineEasing },
    PlaybackChangeRequested { playback: TimelinePlayback },
    PlayingChangeRequested { playing: bool },
    AutoKeyframeChangeRequested { enabled: bool },
    PresetApplyRequested { id: SharedString },
    KeyframeAddRequested { property_id: SharedString },
    TimelineOpenRequested,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodeInspectorViewData {
    pub selection_name: SharedString,
    pub languages: Vec<InspectorChoice>,
    pub language: SharedString,
    pub code: SharedString,
    pub properties: Vec<InspectorChoice>,
    pub wrap_lines: bool,
}
impl Default for CodeInspectorViewData {
    fn default() -> Self {
        Self {
            selection_name: SharedString::default(),
            languages: vec![
                InspectorChoice::new("css", "CSS"),
                InspectorChoice::new("swiftui", "SwiftUI"),
                InspectorChoice::new("jsx", "JSX"),
            ],
            language: "css".into(),
            code: SharedString::default(),
            properties: Vec::new(),
            wrap_lines: false,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodeInspectorAction {
    LanguageChangeRequested { language: SharedString },
    CopyRequested { code: SharedString },
    CopyPropertyRequested { property_id: SharedString },
    WrapLinesChangeRequested { enabled: bool },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CommentsFilter {
    #[default]
    Open,
    All,
    Resolved,
}
impl CommentsFilter {
    pub const ALL: [Self; 3] = [Self::Open, Self::All, Self::Resolved];
    pub const fn label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::All => "All comments",
            Self::Resolved => "Resolved",
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectorComment {
    pub id: SharedString,
    pub author: SharedString,
    pub time_label: SharedString,
    pub body: SharedString,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectorCommentThread {
    pub id: SharedString,
    pub location: SharedString,
    pub resolved: bool,
    pub unread: bool,
    pub comments: Vec<InspectorComment>,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CommentsInspectorViewData {
    pub threads: Vec<InspectorCommentThread>,
    pub filter: CommentsFilter,
    pub selected_thread: Option<SharedString>,
    pub can_comment: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommentsInspectorAction {
    ThreadClearRequested,
    FilterChangeRequested {
        filter: CommentsFilter,
    },
    ThreadSelectRequested {
        id: SharedString,
    },
    ResolveChangeRequested {
        id: SharedString,
        resolved: bool,
    },
    CommentAddRequested {
        body: SharedString,
    },
    ReplyRequested {
        thread_id: SharedString,
        body: SharedString,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrawInspectorViewData {
    pub tool_name: SharedString,
    pub options: DrawToolbarOptions,
    pub color_hex: SharedString,
    pub blend_modes: Vec<InspectorChoice>,
    pub blend_mode: SharedString,
    pub read_only: bool,
}
impl Default for DrawInspectorViewData {
    fn default() -> Self {
        Self {
            tool_name: "Brush".into(),
            options: DrawToolbarOptions::default(),
            color_hex: "000000".into(),
            blend_modes: vec![
                InspectorChoice::new("normal", "Normal"),
                InspectorChoice::new("multiply", "Multiply"),
                InspectorChoice::new("screen", "Screen"),
                InspectorChoice::new("overlay", "Overlay"),
            ],
            blend_mode: "normal".into(),
            read_only: false,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DrawInspectorAction {
    OptionsChangeRequested { options: DrawToolbarOptions },
    ColorChangeRequested { hex: SharedString },
    BlendModeChangeRequested { id: SharedString },
    BrushPresetSaveRequested,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrototypeConnection {
    pub id: SharedString,
    pub trigger: SharedString,
    pub action: SharedString,
    pub destination: SharedString,
    pub animation: SharedString,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrototypeInspectorViewData {
    pub selection_name: SharedString,
    pub devices: Vec<InspectorChoice>,
    pub device: SharedString,
    pub background_hex: SharedString,
    pub connections: Vec<PrototypeConnection>,
    pub flow_name: Option<SharedString>,
    pub read_only: bool,
}
impl Default for PrototypeInspectorViewData {
    fn default() -> Self {
        Self {
            selection_name: SharedString::default(),
            devices: vec![InspectorChoice::new("none", "No device")],
            device: "none".into(),
            background_hex: "000000".into(),
            connections: Vec::new(),
            flow_name: None,
            read_only: false,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrototypeInspectorAction {
    DeviceChangeRequested { id: SharedString },
    BackgroundChangeRequested { hex: SharedString },
    ConnectionAddRequested,
    ConnectionEditRequested { id: SharedString },
    ConnectionRemoveRequested { id: SharedString },
    FlowStartRequested,
    FlowRenameRequested { name: SharedString },
    PresentRequested,
}
