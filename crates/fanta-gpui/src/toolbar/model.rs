use gpui::SharedString;

/// The four editing surfaces exposed by the Figma-style mode tray.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ToolbarMode {
    Draw,
    #[default]
    Design,
    Motion,
    Dev,
}

impl ToolbarMode {
    /// Modes in the same order as the current Figma toolbar.
    pub const ALL: &'static [Self] = &[Self::Draw, Self::Design, Self::Motion, Self::Dev];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Draw => "Draw",
            Self::Design => "Design",
            Self::Motion => "Motion",
            Self::Dev => "Dev",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Draw => "Illustrate with expressive vector tools",
            Self::Design => "Create and edit interface designs",
            Self::Motion => "Animate layers on a keyframe timeline",
            Self::Dev => "Inspect, annotate, measure, and hand off",
        }
    }

    pub const fn glyph(self) -> &'static str {
        match self {
            Self::Draw => "〰",
            Self::Design => "⌁",
            Self::Motion => "◆",
            Self::Dev => "</>",
        }
    }

    pub const fn shortcut(self) -> Option<&'static str> {
        match self {
            Self::Dev => Some("⇧ D"),
            _ => None,
        }
    }

    pub const fn layout(self) -> &'static [ToolbarItem] {
        match self {
            Self::Draw => DRAW_LAYOUT,
            Self::Design => DESIGN_LAYOUT,
            Self::Motion => MOTION_LAYOUT,
            Self::Dev => DEV_LAYOUT,
        }
    }
}

/// Selectable canvas tools addressable by the toolbar and its typed intents.
///
/// Mode layouts expose the tools that belong in Figma's primary strip. Hosts
/// may also use the remaining variants when presenting contextual flyouts.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ToolbarTool {
    #[default]
    Move,
    Hand,
    Scale,
    Frame,
    Section,
    Slice,
    Rectangle,
    Line,
    Arrow,
    Ellipse,
    Polygon,
    Star,
    ImageVideo,
    Pen,
    Pencil,
    Text,
    Comment,
    Annotation,
    Measure,
    Resources,
    Actions,
    Brush,
    PaintBucket,
    ShapeBuilder,
    Lasso,
    VariableWidth,
    Inspect,
    ColorPicker,
    Code,
    Variables,
    ReadyForDev,
    MotionSelect,
    AddKeyframe,
    MotionPath,
    AnimationStyle,
    TimeComment,
    AutoKeyframe,
    PlayPreview,
}

impl ToolbarTool {
    pub const ALL: &'static [Self] = &[
        Self::Move,
        Self::Hand,
        Self::Scale,
        Self::Frame,
        Self::Section,
        Self::Slice,
        Self::Rectangle,
        Self::Line,
        Self::Arrow,
        Self::Ellipse,
        Self::Polygon,
        Self::Star,
        Self::ImageVideo,
        Self::Pen,
        Self::Pencil,
        Self::Text,
        Self::Comment,
        Self::Annotation,
        Self::Measure,
        Self::Resources,
        Self::Actions,
        Self::Brush,
        Self::PaintBucket,
        Self::ShapeBuilder,
        Self::Lasso,
        Self::VariableWidth,
        Self::Inspect,
        Self::ColorPicker,
        Self::Code,
        Self::Variables,
        Self::ReadyForDev,
        Self::MotionSelect,
        Self::AddKeyframe,
        Self::MotionPath,
        Self::AnimationStyle,
        Self::TimeComment,
        Self::AutoKeyframe,
        Self::PlayPreview,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Move => "Move",
            Self::Hand => "Hand",
            Self::Scale => "Scale",
            Self::Frame => "Frame",
            Self::Section => "Section",
            Self::Slice => "Slice",
            Self::Rectangle => "Rectangle",
            Self::Line => "Line",
            Self::Arrow => "Arrow",
            Self::Ellipse => "Ellipse",
            Self::Polygon => "Polygon",
            Self::Star => "Star",
            Self::ImageVideo => "Image/video",
            Self::Pen => "Pen",
            Self::Pencil => "Pencil",
            Self::Text => "Text",
            Self::Comment => "Comment",
            Self::Annotation => "Annotation",
            Self::Measure => "Measurement",
            Self::Resources => "Resources",
            Self::Actions => "Actions",
            Self::Brush => "Brush",
            Self::PaintBucket => "Paint bucket",
            Self::ShapeBuilder => "Shape builder",
            Self::Lasso => "Lasso",
            Self::VariableWidth => "Variable width",
            Self::Inspect => "Inspect",
            Self::ColorPicker => "Color picker",
            Self::Code => "Code",
            Self::Variables => "Variables",
            Self::ReadyForDev => "Mark ready for dev",
            Self::MotionSelect => "Motion select",
            Self::AddKeyframe => "Add keyframe",
            Self::MotionPath => "Motion path",
            Self::AnimationStyle => "Animation style",
            Self::TimeComment => "Time-based comment",
            Self::AutoKeyframe => "Auto keyframe",
            Self::PlayPreview => "Play preview",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Move => "Select and move objects",
            Self::Hand => "Pan around the canvas",
            Self::Scale => "Resize entire objects and layers",
            Self::Frame => "Create a frame container",
            Self::Section => "Organize designs into a section",
            Self::Slice => "Define an export region",
            Self::Rectangle => "Draw a rectangle",
            Self::Line => "Draw a straight line",
            Self::Arrow => "Draw an arrow",
            Self::Ellipse => "Draw an ellipse",
            Self::Polygon => "Draw a polygon",
            Self::Star => "Draw a star",
            Self::ImageVideo => "Place image or video",
            Self::Pen => "Build precise vector paths",
            Self::Pencil => "Draw a freehand vector path",
            Self::Text => "Create a text layer",
            Self::Comment => "Pin a comment to the canvas",
            Self::Annotation => "Add a developer annotation",
            Self::Measure => "Add a persistent measurement",
            Self::Resources => "Search components, libraries, plugins, and widgets",
            Self::Actions => "Search actions, AI tools, plugins, and widgets",
            Self::Brush => "Paint with a vector brush",
            Self::PaintBucket => "Fill an enclosed vector region",
            Self::ShapeBuilder => "Combine or subtract overlapping shapes",
            Self::Lasso => "Select an irregular group of vector points",
            Self::VariableWidth => "Edit width along a vector stroke",
            Self::Inspect => "Inspect layer properties",
            Self::ColorPicker => "Sample colors and variables from the canvas",
            Self::Code => "View generated or connected code",
            Self::Variables => "Explore variables and aliases",
            Self::ReadyForDev => "Mark the current selection ready for development",
            Self::MotionSelect => "Select layers and keyframes",
            Self::AddKeyframe => "Add a keyframe at the playhead",
            Self::MotionPath => "Edit an animated layer's path",
            Self::AnimationStyle => "Apply a preset animation style",
            Self::TimeComment => "Comment at the current timeline time",
            Self::AutoKeyframe => "Record changes as keyframes",
            Self::PlayPreview => "Play or pause the animation preview",
        }
    }

    pub const fn shortcut(self) -> Option<&'static str> {
        match self {
            Self::Move => Some("V"),
            Self::Hand => Some("H"),
            Self::Scale => Some("K"),
            Self::Frame => Some("F"),
            Self::Section => Some("⇧ S"),
            Self::Slice => Some("S"),
            Self::Rectangle => Some("R"),
            Self::Line => Some("L"),
            Self::Arrow => Some("⇧ L"),
            Self::Ellipse => Some("O"),
            Self::ImageVideo => Some("⇧ ⌘ K"),
            Self::Pen => Some("P"),
            Self::Pencil => Some("⇧ P"),
            Self::Text => Some("T"),
            Self::Comment => Some("C"),
            Self::Annotation => Some("⇧ T"),
            Self::Measure => Some("⇧ M"),
            _ => None,
        }
    }

    pub const fn glyph(self) -> &'static str {
        match self {
            Self::Move => "↖",
            Self::Hand => "✋",
            Self::Scale => "↗",
            Self::Frame => "#",
            Self::Section => "▱",
            Self::Slice => "⌗",
            Self::Rectangle => "□",
            Self::Line => "╱",
            Self::Arrow => "↗",
            Self::Ellipse => "○",
            Self::Polygon => "⬡",
            Self::Star => "☆",
            Self::ImageVideo => "▧",
            Self::Pen => "⌁",
            Self::Pencil => "✎",
            Self::Text => "T",
            Self::Comment => "◯",
            Self::Annotation => "¶",
            Self::Measure => "↔",
            Self::Resources => "◇",
            Self::Actions => "✦",
            Self::Brush => "╱",
            Self::PaintBucket => "◈",
            Self::ShapeBuilder => "◒",
            Self::Lasso => "⌁",
            Self::VariableWidth => "≈",
            Self::Inspect => "⌖",
            Self::ColorPicker => "◉",
            Self::Code => "</>",
            Self::Variables => "{}",
            Self::ReadyForDev => "✓",
            Self::MotionSelect => "◆",
            Self::AddKeyframe => "◇+",
            Self::MotionPath => "⌁",
            Self::AnimationStyle => "✧",
            Self::TimeComment => "◷",
            Self::AutoKeyframe => "●",
            Self::PlayPreview => "▶",
        }
    }

    pub fn is_available_in(self, mode: ToolbarMode) -> bool {
        mode.layout().iter().any(|item| match item {
            ToolbarItem::Tool(tool) => *tool == self,
            ToolbarItem::Group(group) => group.tools().contains(&self),
            ToolbarItem::Separator => false,
        })
    }
}

/// A split-button family whose additional tools are exposed in a flyout.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ToolbarToolGroup {
    Move,
    Region,
    Shape,
    Creation,
    Feedback,
    Illustration,
    VectorEdit,
    DevHandoff,
    MotionTimeline,
}

impl ToolbarToolGroup {
    pub const ALL: &'static [Self] = &[
        Self::Move,
        Self::Region,
        Self::Shape,
        Self::Creation,
        Self::Feedback,
        Self::Illustration,
        Self::VectorEdit,
        Self::DevHandoff,
        Self::MotionTimeline,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Move => "Move tools",
            Self::Region => "Region tools",
            Self::Shape => "Shape tools",
            Self::Creation => "Creation tools",
            Self::Feedback => "Comment tools",
            Self::Illustration => "Illustration tools",
            Self::VectorEdit => "Vector editing tools",
            Self::DevHandoff => "Developer handoff tools",
            Self::MotionTimeline => "Motion tools",
        }
    }

    pub const fn tools(self) -> &'static [ToolbarTool] {
        match self {
            Self::Move => &[ToolbarTool::Move, ToolbarTool::Hand, ToolbarTool::Scale],
            Self::Region => &[ToolbarTool::Frame, ToolbarTool::Section, ToolbarTool::Slice],
            Self::Shape => &[
                ToolbarTool::Rectangle,
                ToolbarTool::Line,
                ToolbarTool::Arrow,
                ToolbarTool::Ellipse,
                ToolbarTool::Polygon,
                ToolbarTool::Star,
                ToolbarTool::ImageVideo,
            ],
            Self::Creation => &[ToolbarTool::Pen, ToolbarTool::Pencil],
            Self::Feedback => &[
                ToolbarTool::Comment,
                ToolbarTool::Annotation,
                ToolbarTool::Measure,
            ],
            Self::Illustration => &[ToolbarTool::Brush, ToolbarTool::PaintBucket],
            Self::VectorEdit => &[
                ToolbarTool::ShapeBuilder,
                ToolbarTool::Lasso,
                ToolbarTool::VariableWidth,
            ],
            Self::DevHandoff => &[
                ToolbarTool::Inspect,
                ToolbarTool::Annotation,
                ToolbarTool::Measure,
                ToolbarTool::ReadyForDev,
                ToolbarTool::Code,
                ToolbarTool::Variables,
            ],
            Self::MotionTimeline => &[
                ToolbarTool::MotionSelect,
                ToolbarTool::PlayPreview,
                ToolbarTool::AddKeyframe,
                ToolbarTool::AutoKeyframe,
                ToolbarTool::MotionPath,
                ToolbarTool::AnimationStyle,
                ToolbarTool::TimeComment,
            ],
        }
    }

    pub const fn default_tool(self) -> ToolbarTool {
        self.tools()[0]
    }

    pub fn display_tool(self, active: ToolbarTool) -> ToolbarTool {
        if self.tools().contains(&active) {
            active
        } else {
            self.default_tool()
        }
    }
}

/// One position in a mode's primary toolbar layout.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ToolbarItem {
    Group(ToolbarToolGroup),
    Tool(ToolbarTool),
    Separator,
}

const DESIGN_LAYOUT: &[ToolbarItem] = &[
    ToolbarItem::Group(ToolbarToolGroup::Move),
    ToolbarItem::Group(ToolbarToolGroup::Region),
    ToolbarItem::Group(ToolbarToolGroup::Shape),
    ToolbarItem::Group(ToolbarToolGroup::Creation),
    ToolbarItem::Tool(ToolbarTool::Text),
    ToolbarItem::Group(ToolbarToolGroup::Feedback),
    ToolbarItem::Tool(ToolbarTool::Actions),
];

const DRAW_LAYOUT: &[ToolbarItem] = &[
    ToolbarItem::Group(ToolbarToolGroup::Move),
    ToolbarItem::Separator,
    ToolbarItem::Tool(ToolbarTool::Pen),
    ToolbarItem::Group(ToolbarToolGroup::Illustration),
    ToolbarItem::Tool(ToolbarTool::Pencil),
    ToolbarItem::Group(ToolbarToolGroup::VectorEdit),
    ToolbarItem::Separator,
    ToolbarItem::Group(ToolbarToolGroup::Region),
    ToolbarItem::Group(ToolbarToolGroup::Shape),
    ToolbarItem::Tool(ToolbarTool::Text),
    ToolbarItem::Group(ToolbarToolGroup::Feedback),
    ToolbarItem::Tool(ToolbarTool::Actions),
];

const DEV_LAYOUT: &[ToolbarItem] = &[
    ToolbarItem::Group(ToolbarToolGroup::Move),
    ToolbarItem::Separator,
    ToolbarItem::Tool(ToolbarTool::ColorPicker),
    ToolbarItem::Tool(ToolbarTool::Measure),
    ToolbarItem::Tool(ToolbarTool::Annotation),
    ToolbarItem::Tool(ToolbarTool::Comment),
    ToolbarItem::Tool(ToolbarTool::Actions),
];

const MOTION_LAYOUT: &[ToolbarItem] = &[
    ToolbarItem::Group(ToolbarToolGroup::Move),
    ToolbarItem::Group(ToolbarToolGroup::Region),
    ToolbarItem::Group(ToolbarToolGroup::Shape),
    ToolbarItem::Group(ToolbarToolGroup::Creation),
    ToolbarItem::Tool(ToolbarTool::Text),
    ToolbarItem::Group(ToolbarToolGroup::Feedback),
    ToolbarItem::Group(ToolbarToolGroup::MotionTimeline),
    ToolbarItem::Tool(ToolbarTool::Actions),
];

/// Context controls displayed in the secondary toolbar for specialized modes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ToolbarSecondaryControl {
    DrawStrokeColor,
    DrawBrushStyle,
    DrawStrokeWeight,
    DrawSmoothing,
    DrawPressure,
    DevInspect,
    DevAnnotate,
    DevMeasure,
    DevReadyForDevelopment,
    MotionPlayPause,
    MotionLoop,
    MotionAutoKeyframe,
    MotionAddKeyframe,
    MotionAnimationStyle,
    MotionTimeline,
    MotionTimeComment,
}

impl ToolbarSecondaryControl {
    pub const fn label(self) -> &'static str {
        match self {
            Self::DrawStrokeColor => "Stroke color",
            Self::DrawBrushStyle => "Brush style",
            Self::DrawStrokeWeight => "Stroke weight",
            Self::DrawSmoothing => "Smoothing",
            Self::DrawPressure => "Pressure",
            Self::DevInspect => "Inspect",
            Self::DevAnnotate => "Annotate",
            Self::DevMeasure => "Measure",
            Self::DevReadyForDevelopment => "Mark ready for dev",
            Self::MotionPlayPause => "Play / pause",
            Self::MotionLoop => "Loop",
            Self::MotionAutoKeyframe => "Auto keyframe",
            Self::MotionAddKeyframe => "Add keyframe",
            Self::MotionAnimationStyle => "Animation style",
            Self::MotionTimeline => "Timeline",
            Self::MotionTimeComment => "Time comment",
        }
    }
}

/// A controlled value associated with a mode-specific secondary control.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToolbarControlValue {
    Toggle(bool),
    Integer(i32),
    Text(SharedString),
    Color(SharedString),
    Choice(SharedString),
}

/// Host-provided Draw toolbar values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrawToolbarOptions {
    pub stroke_color: SharedString,
    pub brush_style: SharedString,
    pub stroke_weight: u16,
    pub smoothing: u8,
    pub pressure: bool,
}

impl Default for DrawToolbarOptions {
    fn default() -> Self {
        Self {
            stroke_color: "#1E1E1E".into(),
            brush_style: "Solid".into(),
            stroke_weight: 8,
            smoothing: 32,
            pressure: true,
        }
    }
}

/// Host-provided Dev Mode toolbar values.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DevToolbarOptions {
    pub ready_for_development: bool,
}

/// Host-provided Motion toolbar and playback values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotionToolbarOptions {
    pub playing: bool,
    pub looping: bool,
    pub auto_keyframe: bool,
    pub current_time_ms: u32,
    pub duration_ms: u32,
    pub animation_style: SharedString,
}

impl Default for MotionToolbarOptions {
    fn default() -> Self {
        Self {
            playing: false,
            looping: true,
            auto_keyframe: false,
            current_time_ms: 0,
            duration_ms: 2_000,
            animation_style: "Fade in".into(),
        }
    }
}

/// Host-provided context shown in the contextual Agent composer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentToolbarOptions {
    pub context_label: SharedString,
    pub mention_hint: SharedString,
    pub suggestions: Vec<SharedString>,
}

impl AgentToolbarOptions {
    pub fn new(context_label: impl Into<SharedString>) -> Self {
        Self {
            context_label: context_label.into(),
            ..Self::default()
        }
    }

    pub fn suggestions<I, S>(mut self, suggestions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.suggestions = suggestions.into_iter().take(3).map(Into::into).collect();
        self
    }

    pub fn mention_hint(mut self, mention_hint: impl Into<SharedString>) -> Self {
        self.mention_hint = mention_hint.into();
        self
    }
}

impl Default for AgentToolbarOptions {
    fn default() -> Self {
        Self {
            context_label: "Selection".into(),
            mention_hint: "@ mention components, variables, or libraries".into(),
            suggestions: vec![
                "Explore 3 directions".into(),
                "Polish this screen".into(),
                "Animate the selection".into(),
            ],
        }
    }
}

/// Searchable actions available from the Actions command bar.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ToolbarCommand {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Duplicate,
    Delete,
    SelectAll,
    DeselectAll,
    Group,
    Ungroup,
    FrameSelection,
    AddAutoLayout,
    CreateComponent,
    DetachInstance,
    FindAndReplace,
    ToggleRulers,
    ToggleLayoutGrids,
    ToggleOutlines,
    TogglePixelPreview,
    ToggleUi,
    MinimizeUi,
    ZoomToFit,
    ZoomToSelection,
    Import,
    Export,
    PlaceImageVideo,
    OpenResources,
    OpenPlugins,
    OpenWidgets,
    OpenVariables,
    GenerateDesign,
    ReplaceContent,
    RewriteText,
    TranslateText,
    RenameLayers,
    RemoveBackground,
    GenerateImage,
    MakePrototype,
    OpenDesignMode,
    OpenDrawMode,
    OpenMotionMode,
    OpenDevMode,
    ViewVersionHistory,
    CopyLink,
    KeyboardShortcuts,
    Preferences,
    Present,
    Share,
}

impl ToolbarCommand {
    pub const ALL: &'static [Self] = &[
        Self::GenerateDesign,
        Self::ReplaceContent,
        Self::GenerateImage,
        Self::MakePrototype,
        Self::RenameLayers,
        Self::RemoveBackground,
        Self::RewriteText,
        Self::TranslateText,
        Self::FindAndReplace,
        Self::Undo,
        Self::Redo,
        Self::Cut,
        Self::Copy,
        Self::Paste,
        Self::Duplicate,
        Self::Delete,
        Self::SelectAll,
        Self::DeselectAll,
        Self::Group,
        Self::Ungroup,
        Self::FrameSelection,
        Self::AddAutoLayout,
        Self::CreateComponent,
        Self::DetachInstance,
        Self::ToggleRulers,
        Self::ToggleLayoutGrids,
        Self::ToggleOutlines,
        Self::TogglePixelPreview,
        Self::ToggleUi,
        Self::MinimizeUi,
        Self::ZoomToFit,
        Self::ZoomToSelection,
        Self::Import,
        Self::Export,
        Self::PlaceImageVideo,
        Self::OpenResources,
        Self::OpenPlugins,
        Self::OpenWidgets,
        Self::OpenVariables,
        Self::OpenDesignMode,
        Self::OpenDrawMode,
        Self::OpenMotionMode,
        Self::OpenDevMode,
        Self::ViewVersionHistory,
        Self::CopyLink,
        Self::KeyboardShortcuts,
        Self::Preferences,
        Self::Present,
        Self::Share,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::Cut => "Cut",
            Self::Copy => "Copy",
            Self::Paste => "Paste",
            Self::Duplicate => "Duplicate",
            Self::Delete => "Delete",
            Self::SelectAll => "Select all",
            Self::DeselectAll => "Deselect all",
            Self::Group => "Group selection",
            Self::Ungroup => "Ungroup selection",
            Self::FrameSelection => "Frame selection",
            Self::AddAutoLayout => "Add auto layout",
            Self::CreateComponent => "Create component",
            Self::DetachInstance => "Detach instance",
            Self::FindAndReplace => "Find and replace",
            Self::ToggleRulers => "Show / hide rulers",
            Self::ToggleLayoutGrids => "Show / hide layout grids",
            Self::ToggleOutlines => "Show / hide outlines",
            Self::TogglePixelPreview => "Toggle pixel preview",
            Self::ToggleUi => "Show / hide UI",
            Self::MinimizeUi => "Minimize UI",
            Self::ZoomToFit => "Zoom to fit",
            Self::ZoomToSelection => "Zoom to selection",
            Self::Import => "Import",
            Self::Export => "Export",
            Self::PlaceImageVideo => "Place image or video",
            Self::OpenResources => "Open resources",
            Self::OpenPlugins => "Browse plugins",
            Self::OpenWidgets => "Browse widgets",
            Self::OpenVariables => "Open variables",
            Self::GenerateDesign => "Generate a design",
            Self::ReplaceContent => "Replace content",
            Self::RewriteText => "Rewrite text",
            Self::TranslateText => "Translate text",
            Self::RenameLayers => "Rename layers",
            Self::RemoveBackground => "Remove background",
            Self::GenerateImage => "Generate an image",
            Self::MakePrototype => "Make a prototype",
            Self::OpenDesignMode => "Switch to Design",
            Self::OpenDrawMode => "Switch to Draw",
            Self::OpenMotionMode => "Switch to Motion",
            Self::OpenDevMode => "Switch to Dev Mode",
            Self::ViewVersionHistory => "Show version history",
            Self::CopyLink => "Copy link",
            Self::KeyboardShortcuts => "Keyboard shortcuts",
            Self::Preferences => "Preferences",
            Self::Present => "Present",
            Self::Share => "Share",
        }
    }

    pub const fn category(self) -> &'static str {
        match self {
            Self::GenerateDesign
            | Self::ReplaceContent
            | Self::RewriteText
            | Self::TranslateText
            | Self::RenameLayers
            | Self::RemoveBackground
            | Self::GenerateImage
            | Self::MakePrototype => "AI",
            Self::OpenResources | Self::OpenPlugins | Self::OpenWidgets | Self::OpenVariables => {
                "Resources"
            }
            Self::ToggleRulers
            | Self::ToggleLayoutGrids
            | Self::ToggleOutlines
            | Self::TogglePixelPreview
            | Self::ToggleUi
            | Self::MinimizeUi
            | Self::ZoomToFit
            | Self::ZoomToSelection => "View",
            Self::OpenDesignMode
            | Self::OpenDrawMode
            | Self::OpenMotionMode
            | Self::OpenDevMode => "Modes",
            Self::Import
            | Self::Export
            | Self::PlaceImageVideo
            | Self::ViewVersionHistory
            | Self::CopyLink
            | Self::Preferences
            | Self::KeyboardShortcuts => "File",
            Self::Present | Self::Share => "Collaboration",
            _ => "Commands",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::GenerateDesign => "Create editable layers with Figma Agent",
            Self::ReplaceContent => "Generate replacement copy for selected layers",
            Self::GenerateImage => "Create an image from a prompt",
            Self::MakePrototype => "Connect selected frames into a prototype",
            Self::OpenPlugins => "Run a plugin from the Community",
            Self::OpenResources => "Search components, libraries, and assets",
            Self::OpenMotionMode => "Open the animation timeline",
            Self::OpenDevMode => "Inspect the design for implementation",
            Self::Present => "Preview this file as a prototype",
            Self::Share => "Invite collaborators or copy a share link",
            _ => "Run this action in the current file",
        }
    }

    pub const fn shortcut(self) -> Option<&'static str> {
        match self {
            Self::Undo => Some("⌘ Z"),
            Self::Redo => Some("⇧ ⌘ Z"),
            Self::Cut => Some("⌘ X"),
            Self::Copy => Some("⌘ C"),
            Self::Paste => Some("⌘ V"),
            Self::Duplicate => Some("⌘ D"),
            Self::Delete => Some("⌫"),
            Self::SelectAll => Some("⌘ A"),
            Self::DeselectAll => Some("⇧ ⌘ A"),
            Self::Group => Some("⌘ G"),
            Self::Ungroup => Some("⇧ ⌘ G"),
            Self::FrameSelection => Some("⌥ ⌘ G"),
            Self::AddAutoLayout => Some("⇧ A"),
            Self::CreateComponent => Some("⌥ ⌘ K"),
            Self::FindAndReplace => Some("⌘ F"),
            Self::ToggleRulers => Some("⇧ R"),
            Self::ToggleUi => Some("⌘ \\"),
            Self::MinimizeUi => Some("⇧ \\"),
            Self::PlaceImageVideo => Some("⇧ ⌘ K"),
            Self::OpenDevMode => Some("⇧ D"),
            _ => None,
        }
    }
}

/// Typed intents emitted by [`super::EditorToolbar`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToolbarAction {
    ModeChangeRequested {
        mode: ToolbarMode,
    },
    ToolChangeRequested {
        mode: ToolbarMode,
        tool: ToolbarTool,
    },
    SecondaryControlInvoked {
        mode: ToolbarMode,
        control: ToolbarSecondaryControl,
    },
    ControlChangeRequested {
        mode: ToolbarMode,
        control: ToolbarSecondaryControl,
        value: ToolbarControlValue,
    },
    CommandQueryChanged {
        query: SharedString,
    },
    CommandInvoked {
        command: ToolbarCommand,
    },
    AiPromptSubmitted {
        prompt: SharedString,
    },
    AgentVisibilityChanged {
        visible: bool,
    },
    AgentAttachmentRequested,
    AgentVoiceInputRequested,
    ZoomChangeRequested {
        percent: u16,
    },
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn every_mode_has_a_complete_layout() {
        for mode in ToolbarMode::ALL {
            assert!(
                !mode.layout().is_empty(),
                "{} must have tools",
                mode.label()
            );
            assert!(
                mode.layout()
                    .contains(&ToolbarItem::Tool(ToolbarTool::Actions)),
                "{} must expose Actions",
                mode.label()
            );
        }
    }

    #[test]
    fn group_defaults_are_members_and_group_lists_are_unique() {
        for group in ToolbarToolGroup::ALL {
            assert!(group.tools().contains(&group.default_tool()));
            let unique = group.tools().iter().copied().collect::<HashSet<_>>();
            assert_eq!(
                unique.len(),
                group.tools().len(),
                "{} contains duplicate tools",
                group.label()
            );
        }
    }

    #[test]
    fn global_tool_and_command_catalogs_are_unique() {
        assert_eq!(
            ToolbarTool::ALL
                .iter()
                .copied()
                .collect::<HashSet<_>>()
                .len(),
            ToolbarTool::ALL.len()
        );
        assert_eq!(
            ToolbarCommand::ALL
                .iter()
                .copied()
                .collect::<HashSet<_>>()
                .len(),
            ToolbarCommand::ALL.len()
        );
    }

    #[test]
    fn specialist_modes_expose_their_flyout_groups() {
        assert!(
            ToolbarMode::Draw
                .layout()
                .contains(&ToolbarItem::Group(ToolbarToolGroup::Illustration))
        );
        assert!(
            ToolbarMode::Motion
                .layout()
                .contains(&ToolbarItem::Group(ToolbarToolGroup::MotionTimeline))
        );
        assert!(ToolbarTool::PaintBucket.is_available_in(ToolbarMode::Draw));
        assert!(ToolbarTool::MotionPath.is_available_in(ToolbarMode::Motion));
    }

    #[test]
    fn agent_suggestions_are_bounded_for_the_compact_composer() {
        let options = AgentToolbarOptions::new("Card").suggestions(["One", "Two", "Three", "Four"]);
        assert_eq!(options.context_label, "Card");
        assert_eq!(options.suggestions.len(), 3);
        assert_eq!(options.suggestions[2], "Three");
    }
}
