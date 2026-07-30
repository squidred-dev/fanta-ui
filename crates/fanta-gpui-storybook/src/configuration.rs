use fanta_gpui::prelude::{
    DesignPanelEditMode, DesignPanelInspectionContext, DesignPanelNode, DesignPanelParentLayout,
    DesignPanelPermissions,
};
use gpui_component::ThemeMode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StoryKind {
    Icons,
    Toolbar,
    Pages,
    Layers,
    Design,
    Variables,
    Assets,
    Prototype,
    Timeline,
    PseudoEditor,
}

impl StoryKind {
    pub(super) const ALL: [Self; 10] = [
        Self::Icons,
        Self::Toolbar,
        Self::PseudoEditor,
        Self::Pages,
        Self::Layers,
        Self::Design,
        Self::Assets,
        Self::Prototype,
        Self::Variables,
        Self::Timeline,
    ];

    fn from_name(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "icons" | "icon" | "icon-gallery" | "icon-catalog" => Self::Icons,
            "pages" => Self::Pages,
            "layers" => Self::Layers,
            "design" => Self::Design,
            "variables" | "variable" | "variables-page" => Self::Variables,
            "assets" | "assets-panel" => Self::Assets,
            "prototype" | "prototype-panel" => Self::Prototype,
            "timeline" | "motion-timeline" => Self::Timeline,
            "pseudo" | "pseudo-editor" | "editor" => Self::PseudoEditor,
            _ => Self::Toolbar,
        }
    }

    pub(super) const fn reference_window_size(self) -> (f32, f32) {
        match self {
            Self::Icons => (1240., 820.),
            Self::Variables => (1677., 1048.),
            Self::Timeline => (1728., 333.),
            Self::Assets => (337., 716.),
            Self::Prototype => (473., 716.),
            Self::PseudoEditor => (1440., 900.),
            Self::Toolbar | Self::Pages | Self::Layers | Self::Design => (1240., 820.),
        }
    }

    pub(super) const fn title(self) -> &'static str {
        match self {
            Self::Icons => "Icons",
            Self::Toolbar => "Editor toolbar",
            Self::Pages => "Pages panel",
            Self::Layers => "Layers panel",
            Self::Design => "Design inspector",
            Self::Variables => "Variables",
            Self::Assets => "Assets panel",
            Self::Prototype => "Prototype panel",
            Self::Timeline => "Timeline",
            Self::PseudoEditor => "Pseudo editor",
        }
    }

    pub(super) const fn description(self) -> &'static str {
        match self {
            Self::Icons => {
                "Every icon bundled by gpui-component, with canonical names and asset paths."
            }
            Self::Toolbar => {
                "A mode-aware Figma-style toolbar with tool groups, actions, zoom, and agent overlays."
            }
            Self::Pages => {
                "Interactive page navigation, search, replace, filters, and typed host intents."
            }
            Self::Layers => {
                "A controlled layer tree with selection, expansion, reordering, and contextual actions."
            }
            Self::Design => {
                "A host-controlled inspector covering layout, appearance, typography, variables, and export."
            }
            Self::Variables => {
                "A full variables workspace with collections, groups, modes, values, and creation intents."
            }
            Self::Assets => {
                "A library browser for current-file components, UI kits, filtering, and selection."
            }
            Self::Prototype => {
                "Prototype settings and connection guidance presented as a controlled inspector."
            }
            Self::Timeline => {
                "Motion controls, a time ruler, seeking, zoom, keyframes, and an empty-state agent prompt."
            }
            Self::PseudoEditor => {
                "All Fanta GPUI surfaces assembled into one simulated editor workspace."
            }
        }
    }

    pub(super) const fn gallery_surface_size(self) -> (f32, f32) {
        match self {
            Self::Icons => (1200., 900.),
            Self::Toolbar => (1180., 720.),
            Self::Pages | Self::Layers => (337., 716.),
            Self::Design => (1200., 760.),
            Self::Variables => (1500., 900.),
            Self::Assets => (337., 716.),
            Self::Prototype => (473., 716.),
            Self::Timeline => (1200., 333.),
            Self::PseudoEditor => (1440., 900.),
        }
    }

    pub(super) const fn gallery_uses_fluid_width(self) -> bool {
        matches!(
            self,
            Self::Icons
                | Self::Toolbar
                | Self::Design
                | Self::Variables
                | Self::Timeline
                | Self::PseudoEditor
        )
    }

    pub(super) fn matches_query(self, query: &str) -> bool {
        query.is_empty()
            || self.title().to_ascii_lowercase().contains(query)
            || self.description().to_ascii_lowercase().contains(query)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StorybookLaunchMode {
    Gallery,
    ReferenceFixture,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct StorybookLaunch {
    pub(super) story: StoryKind,
    pub(super) mode: StorybookLaunchMode,
}

pub(super) fn parse_storybook_launch(
    reference_story_env: Option<&str>,
    gallery_story_env: Option<&str>,
) -> StorybookLaunch {
    match reference_story_env {
        Some(story) => StorybookLaunch {
            story: StoryKind::from_name(story),
            mode: StorybookLaunchMode::ReferenceFixture,
        },
        None => StorybookLaunch {
            story: gallery_story_env
                .map(StoryKind::from_name)
                .unwrap_or(StoryKind::PseudoEditor),
            mode: StorybookLaunchMode::Gallery,
        },
    }
}

pub(super) fn storybook_launch_from_env() -> StorybookLaunch {
    let reference_story = std::env::var_os("FANTA_STORYBOOK_STORY");
    let gallery_story = std::env::var_os("FANTA_STORYBOOK_GALLERY_STORY");
    parse_storybook_launch(
        reference_story
            .as_deref()
            .map(|value| value.to_string_lossy())
            .as_deref(),
        gallery_story
            .as_deref()
            .map(|value| value.to_string_lossy())
            .as_deref(),
    )
}

pub(super) fn parse_storybook_theme(value: Option<&str>) -> ThemeMode {
    if value.is_some_and(|value| value.eq_ignore_ascii_case("dark")) {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    }
}

pub(super) fn storybook_theme_from_env() -> ThemeMode {
    let value = std::env::var_os("FANTA_STORYBOOK_THEME");
    parse_storybook_theme(
        value
            .as_deref()
            .map(|value| value.to_string_lossy())
            .as_deref(),
    )
}

pub(super) fn storybook_window_dimension(name: &str, fallback: f32) -> f32 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|value| value.is_finite() && *value >= 240.)
        .unwrap_or(fallback)
}

pub(super) const fn design_additional_labels_status(enabled: bool) -> &'static str {
    if enabled { "On" } else { "Off" }
}

pub(super) fn parse_design_additional_labels(value: Option<&str>) -> bool {
    value.is_some_and(|value| {
        value == "1" || value.eq_ignore_ascii_case("on") || value.eq_ignore_ascii_case("true")
    })
}

pub(super) const DESIGN_PANEL_WIDTH_PRESETS: [f32; 3] = [320., 400., 472.];
const DESIGN_PANEL_MIN_WIDTH: f32 = 320.;
const DESIGN_PANEL_MAX_WIDTH: f32 = 640.;
const DESIGN_PANEL_DEFAULT_WIDTH: f32 = 472.;
const DESIGN_PANEL_KEYBOARD_STEP: f32 = 8.;
const DESIGN_PANEL_KEYBOARD_COARSE_STEP: f32 = 32.;

pub(super) fn clamp_design_panel_width(width: f32) -> f32 {
    if width.is_finite() {
        width.clamp(DESIGN_PANEL_MIN_WIDTH, DESIGN_PANEL_MAX_WIDTH)
    } else {
        DESIGN_PANEL_DEFAULT_WIDTH
    }
}

pub(super) fn parse_design_panel_width(value: Option<&str>) -> f32 {
    value
        .and_then(|value| value.parse::<f32>().ok())
        .map(clamp_design_panel_width)
        .unwrap_or(DESIGN_PANEL_DEFAULT_WIDTH)
}

pub(super) fn design_panel_width_after_key(width: f32, key: &str, shift: bool) -> Option<f32> {
    let step = if shift {
        DESIGN_PANEL_KEYBOARD_COARSE_STEP
    } else {
        DESIGN_PANEL_KEYBOARD_STEP
    };
    let delta = match key {
        "left" | "down" | "-" | "_" => -step,
        "right" | "up" | "+" | "=" => step,
        _ => return None,
    };
    Some(clamp_design_panel_width(width + delta))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct DesignPanelResizeDrag {
    start_pointer_x: f32,
    start_width: f32,
}

impl DesignPanelResizeDrag {
    pub(super) fn new(start_pointer_x: f32, start_width: f32) -> Self {
        Self {
            start_pointer_x,
            start_width: clamp_design_panel_width(start_width),
        }
    }

    pub(super) fn width_at(self, pointer_x: f32) -> f32 {
        clamp_design_panel_width(self.start_width + self.start_pointer_x - pointer_x)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DesignInspectionScenario {
    Page,
    ViewOnlyPage,
    RestrictedPage,
    EditableSingle,
    CanvasSingle,
    ViewOnlySingle,
    RestrictedSingle,
    EditableMultiple,
    HomogeneousMultiple,
    ViewOnlyMultiple,
    RestrictedMultiple,
    SmartSelectionNone,
    SmartSelectionHorizontal,
    SmartSelectionVertical,
    SmartSelectionTwoDimensional,
    SmartSelectionReadOnly,
    AddAutoLayoutGroup,
    AddAutoLayoutMultiple,
    PropertyStates,
    AutoLayoutChild,
    AutoLayoutIgnored,
    AutoLayoutVerticalChild,
    GridChild,
    TextEdit,
    VectorEdit,
}

impl DesignInspectionScenario {
    pub(super) fn initial_from_env() -> Self {
        Self::from_launch_value(&std::env::var("FANTA_DESIGN_SCENARIO").unwrap_or_default())
    }

    pub(super) fn from_launch_value(value: &str) -> Self {
        let value = value.to_ascii_lowercase();
        if let Some(scenario) = Self::ALL
            .into_iter()
            .find(|scenario| scenario.id() == value.as_str())
        {
            return scenario;
        }

        match value.as_str() {
            "none" => Self::Page,
            "canvas" => Self::CanvasSingle,
            "multiple" => Self::EditableMultiple,
            "multiple-homogeneous" | "common-paints" => Self::HomogeneousMultiple,
            "smart-selection-none" | "smart-none" => Self::SmartSelectionNone,
            "smart-selection-horizontal" | "smart-horizontal" => Self::SmartSelectionHorizontal,
            "smart-selection-vertical" | "smart-vertical" => Self::SmartSelectionVertical,
            "smart-selection-2d" | "smart-selection-two-dimensional" | "smart-2d" => {
                Self::SmartSelectionTwoDimensional
            }
            "smart-selection-read-only" | "smart-read-only" => Self::SmartSelectionReadOnly,
            "auto-layout-group" => Self::AddAutoLayoutGroup,
            "auto-layout-multiple" => Self::AddAutoLayoutMultiple,
            "view-only" | "viewer" => Self::ViewOnlySingle,
            "viewer-page" => Self::ViewOnlyPage,
            "viewer-single" => Self::ViewOnlySingle,
            "viewer-multiple" => Self::ViewOnlyMultiple,
            "restricted" | "restricted-viewer" => Self::RestrictedSingle,
            "restricted-viewer-page" => Self::RestrictedPage,
            "restricted-viewer-single" => Self::RestrictedSingle,
            "restricted-viewer-multiple" => Self::RestrictedMultiple,
            "property-states" | "states" => Self::PropertyStates,
            "auto-layout" => Self::AutoLayoutChild,
            "ignored" => Self::AutoLayoutIgnored,
            "vertical-wrap"
            | "auto-layout-vertical-wrap"
            | "vertical-child"
            | "auto-layout-vertical-child" => Self::AutoLayoutVerticalChild,
            "grid" => Self::GridChild,
            _ => Self::EditableSingle,
        }
    }

    pub(super) const ALL: [Self; 25] = [
        Self::Page,
        Self::ViewOnlyPage,
        Self::RestrictedPage,
        Self::EditableSingle,
        Self::CanvasSingle,
        Self::ViewOnlySingle,
        Self::RestrictedSingle,
        Self::EditableMultiple,
        Self::HomogeneousMultiple,
        Self::ViewOnlyMultiple,
        Self::RestrictedMultiple,
        Self::SmartSelectionNone,
        Self::SmartSelectionHorizontal,
        Self::SmartSelectionVertical,
        Self::SmartSelectionTwoDimensional,
        Self::SmartSelectionReadOnly,
        Self::AddAutoLayoutGroup,
        Self::AddAutoLayoutMultiple,
        Self::PropertyStates,
        Self::AutoLayoutChild,
        Self::AutoLayoutIgnored,
        Self::AutoLayoutVerticalChild,
        Self::GridChild,
        Self::TextEdit,
        Self::VectorEdit,
    ];

    pub(super) const fn id(self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::ViewOnlyPage => "view-only-page",
            Self::RestrictedPage => "restricted-page",
            Self::EditableSingle => "editable-single",
            Self::CanvasSingle => "canvas-single",
            Self::ViewOnlySingle => "view-only-single",
            Self::RestrictedSingle => "restricted-single",
            Self::EditableMultiple => "editable-multiple",
            Self::HomogeneousMultiple => "homogeneous-multiple",
            Self::ViewOnlyMultiple => "view-only-multiple",
            Self::RestrictedMultiple => "restricted-multiple",
            Self::SmartSelectionNone => "smart-selection-none",
            Self::SmartSelectionHorizontal => "smart-selection-horizontal",
            Self::SmartSelectionVertical => "smart-selection-vertical",
            Self::SmartSelectionTwoDimensional => "smart-selection-two-dimensional",
            Self::SmartSelectionReadOnly => "smart-selection-read-only",
            Self::AddAutoLayoutGroup => "add-auto-layout-group",
            Self::AddAutoLayoutMultiple => "add-auto-layout-multiple",
            Self::PropertyStates => "property-states",
            Self::AutoLayoutChild => "auto-layout-child",
            Self::AutoLayoutIgnored => "auto-layout-ignored",
            Self::AutoLayoutVerticalChild => "auto-layout-vertical-child",
            Self::GridChild => "grid-child",
            Self::TextEdit => "text-edit",
            Self::VectorEdit => "vector-edit",
        }
    }

    pub(super) const fn permissions(self) -> DesignPanelPermissions {
        match self {
            Self::ViewOnlyPage | Self::ViewOnlySingle | Self::ViewOnlyMultiple => {
                DesignPanelPermissions::viewer()
            }
            Self::RestrictedPage | Self::RestrictedSingle | Self::RestrictedMultiple => {
                DesignPanelPermissions::restricted_viewer()
            }
            _ => DesignPanelPermissions::editor(),
        }
    }

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Page => "Editable page / no selection",
            Self::ViewOnlyPage => "View-only page / no selection",
            Self::RestrictedPage => "Restricted page / no selection",
            Self::EditableSingle => "Editable single",
            Self::CanvasSingle => "Canvas single",
            Self::ViewOnlySingle => "View-only single",
            Self::RestrictedSingle => "Restricted single",
            Self::EditableMultiple => "Editable multiple",
            Self::HomogeneousMultiple => "Homogeneous multiple · Common Fill and Stroke",
            Self::ViewOnlyMultiple => "View-only multiple",
            Self::RestrictedMultiple => "Restricted multiple",
            Self::SmartSelectionNone => "Smart selection · None",
            Self::SmartSelectionHorizontal => "Smart selection · Horizontal",
            Self::SmartSelectionVertical => "Smart selection · Vertical",
            Self::SmartSelectionTwoDimensional => "Smart selection · 2D",
            Self::SmartSelectionReadOnly => "Smart selection · Read only",
            Self::AddAutoLayoutGroup => "Add auto layout · Group",
            Self::AddAutoLayoutMultiple => "Add auto layout · Multiple",
            Self::PropertyStates => "Bound + read-only",
            Self::AutoLayoutChild => "Auto-layout child",
            Self::AutoLayoutIgnored => "Ignored auto-layout child",
            Self::AutoLayoutVerticalChild => "Vertical auto-layout child",
            Self::GridChild => "Grid child",
            Self::TextEdit => "Text edit mode",
            Self::VectorEdit => "Vector edit mode",
        }
    }
}

pub(super) const STORY_HOMOGENEOUS_MULTIPLE_NODE_IDS: [&str; 2] =
    ["homogeneous-multiple-first", "homogeneous-multiple-second"];

pub(super) fn story_text_edit_inspection_context(
    selected: DesignPanelNode,
    text_range_revision: u64,
) -> DesignPanelInspectionContext {
    DesignPanelInspectionContext::single(
        selected,
        DesignPanelParentLayout::Freeform,
        DesignPanelPermissions::editor(),
    )
    .with_edit_mode(DesignPanelEditMode::Text)
    .expect("the Text story preset is an editable single selection")
    .with_text_range_revision(text_range_revision)
    .expect("the Text story preset has an exact host range revision")
}

pub(super) fn default_design_inspection_scenario_for_node(
    node: &DesignPanelNode,
) -> DesignInspectionScenario {
    match node.id.as_ref() {
        "text-max-lines-vertical-hug" | "text-max-lines-vertical-fixed" => {
            DesignInspectionScenario::AutoLayoutVerticalChild
        }
        _ => DesignInspectionScenario::EditableSingle,
    }
}
