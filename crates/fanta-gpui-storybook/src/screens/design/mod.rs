//! The Design story screen: mock-host state, env-seeded launch defaults,
//! scenario/config vocabulary, and the controlled inspection-context echo.
//!
//! The story's fixtures live in [`fixtures`], the reference/gallery chrome in
//! [`harness`], and the intent reducer in [`reducers`].

pub(crate) mod fixtures;
mod harness;
pub(crate) mod reducers;

use crate::*;

#[allow(unused_imports)]
use {fixtures::*, reducers::*};

pub(crate) const fn design_additional_labels_status(enabled: bool) -> &'static str {
    if enabled { "On" } else { "Off" }
}

pub(crate) fn parse_design_additional_labels(value: Option<&str>) -> bool {
    value.is_some_and(|value| {
        value == "1" || value.eq_ignore_ascii_case("on") || value.eq_ignore_ascii_case("true")
    })
}

pub(crate) const DESIGN_PANEL_WIDTH_PRESETS: [f32; 3] = [320., 400., 472.];
const DESIGN_PANEL_MIN_WIDTH: f32 = 320.;
const DESIGN_PANEL_MAX_WIDTH: f32 = 640.;
const DESIGN_PANEL_DEFAULT_WIDTH: f32 = 472.;
const DESIGN_PANEL_KEYBOARD_STEP: f32 = 8.;
const DESIGN_PANEL_KEYBOARD_COARSE_STEP: f32 = 32.;

/// Width of the fixture rail beside the inspector in the wide harness.
pub(crate) const DESIGN_HARNESS_RAIL_WIDTH: f32 = 260.;
/// Thickness of the bespoke inspector width-resize handle.
pub(crate) const DESIGN_HARNESS_RESIZE_HANDLE_WIDTH: f32 = 10.;
/// Smallest mock-canvas sliver the wide harness preserves beside the
/// inspector before the panel width is capped.
const DESIGN_HARNESS_CANVAS_MIN_WIDTH: f32 = 150.;
/// Below this harness width the story chrome stacks: the fixture rail
/// folds into a collapsible controls section under the inspector, which
/// stays the priority element.
pub(crate) const DESIGN_HARNESS_STACK_BREAKPOINT: f32 = 900.;
/// Tallest the expanded stacked controls grow before scrolling, so the
/// inspector keeps most of the height even while they are open.
pub(crate) const DESIGN_HARNESS_CONTROLS_MAX_HEIGHT: f32 = 240.;
/// The Design story's honest floor: the inspector at its own 320 px
/// minimum plus the resize handle and stacked-chrome margins…
pub(crate) const DESIGN_STORY_MIN_WIDTH: f32 =
    DESIGN_PANEL_MIN_WIDTH + DESIGN_HARNESS_RESIZE_HANDLE_WIDTH + 50.;
/// …and enough height for a usable inspector column above the collapsed
/// controls bar of the stacked harness.
pub(crate) const DESIGN_STORY_MIN_HEIGHT: f32 = 520.;

/// Whether the story harness uses the stacked layout at this width.
pub(crate) fn design_harness_stacked(harness_width: f32) -> bool {
    harness_width < DESIGN_HARNESS_STACK_BREAKPOINT
}

/// The width the inspector renders at inside the harness: the user's
/// chosen panel width, capped so the harness chrome that sits beside the
/// panel (rail, canvas sliver, and resize handle when wide; just the
/// handle when stacked) always fits. The cap never pushes the panel
/// below its own [`DESIGN_PANEL_MIN_WIDTH`]; the story floor keeps that
/// minimum reachable inside the story surface.
pub(crate) fn design_harness_panel_width(panel_width: f32, harness_width: f32) -> f32 {
    let panel_width = clamp_design_panel_width(panel_width);
    let reserved = if design_harness_stacked(harness_width) {
        DESIGN_HARNESS_RESIZE_HANDLE_WIDTH
    } else {
        DESIGN_HARNESS_RAIL_WIDTH
            + DESIGN_HARNESS_CANVAS_MIN_WIDTH
            + DESIGN_HARNESS_RESIZE_HANDLE_WIDTH
    };
    panel_width.min((harness_width - reserved).max(DESIGN_PANEL_MIN_WIDTH))
}

pub(crate) fn clamp_design_panel_width(width: f32) -> f32 {
    if width.is_finite() {
        width.clamp(DESIGN_PANEL_MIN_WIDTH, DESIGN_PANEL_MAX_WIDTH)
    } else {
        DESIGN_PANEL_DEFAULT_WIDTH
    }
}

pub(crate) fn parse_design_panel_width(value: Option<&str>) -> f32 {
    value
        .and_then(|value| value.parse::<f32>().ok())
        .map(clamp_design_panel_width)
        .unwrap_or(DESIGN_PANEL_DEFAULT_WIDTH)
}

pub(crate) fn design_panel_width_after_key(width: f32, key: &str, shift: bool) -> Option<f32> {
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
pub(crate) struct DesignPanelResizeDrag {
    start_pointer_x: f32,
    start_width: f32,
}

impl DesignPanelResizeDrag {
    pub(crate) fn new(start_pointer_x: f32, start_width: f32) -> Self {
        Self {
            start_pointer_x,
            start_width: clamp_design_panel_width(start_width),
        }
    }

    pub(crate) fn width_at(self, pointer_x: f32) -> f32 {
        clamp_design_panel_width(self.start_width + self.start_pointer_x - pointer_x)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DesignInspectionScenario {
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
    pub(crate) fn initial_from_env() -> Self {
        Self::from_launch_value(&std::env::var("FANTA_DESIGN_SCENARIO").unwrap_or_default())
    }

    pub(crate) fn from_launch_value(value: &str) -> Self {
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

    pub(crate) const ALL: [Self; 25] = [
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

    pub(crate) const fn id(self) -> &'static str {
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

    pub(crate) const fn permissions(self) -> DesignPanelPermissions {
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

    pub(crate) const fn label(self) -> &'static str {
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

pub(crate) const STORY_HOMOGENEOUS_MULTIPLE_NODE_IDS: [&str; 2] =
    ["homogeneous-multiple-first", "homogeneous-multiple-second"];

pub(crate) fn story_text_edit_inspection_context(
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

pub(crate) fn default_design_inspection_scenario_for_node(
    node: &DesignPanelNode,
) -> DesignInspectionScenario {
    match node.id.as_ref() {
        "text-max-lines-vertical-hug" | "text-max-lines-vertical-fixed" => {
            DesignInspectionScenario::AutoLayoutVerticalChild
        }
        _ => DesignInspectionScenario::EditableSingle,
    }
}

pub(crate) struct DesignScreen {
    pub(crate) panel: Entity<DesignPanel>,
    pub(crate) nodes: Vec<DesignPanelNode>,
    pub(crate) color_styles: DesignColorStyleViewData,
    pub(crate) color_style_samples: DesignColorStyleSampleViewData,
    pub(crate) paint_styles: DesignPaintStyleViewData,
    pub(crate) paint_variables: DesignPaintVariableViewData,
    pub(crate) shaders: DesignShaderViewData,
    pub(crate) typography_styles: DesignTypographyStyleViewData,
    pub(crate) fonts: DesignFontViewData,
    pub(crate) effect_styles: DesignEffectStyleViewData,
    pub(crate) effect_variables: DesignEffectVariableViewData,
    pub(crate) property_variables: DesignVariableViewData,
    pub(crate) component_swaps: DesignComponentSwapViewData,
    pub(crate) property_bindings:
        HashMap<(SharedString, DesignPanelProperty), DesignPanelPropertyBinding<DesignPanelValue>>,
    pub(crate) layout_grid_styles: DesignLayoutGridStyleViewData,
    pub(crate) layout_grid_variables: DesignLayoutGridVariableViewData,
    pub(crate) frame_presets: HashMap<SharedString, DesignFramePresetViewData>,
    pub(crate) page_view_data: DesignPageViewData,
    pub(crate) page_local_styles: DesignPageLocalStylesViewData,
    pub(crate) variable_mode_views: HashMap<SharedString, DesignVariableModeViewData>,
    pub(crate) viewer_properties: HashMap<SharedString, DesignViewerPropertiesViewData>,
    pub(crate) export_configurations: HashMap<SharedString, Vec<DesignExportConfiguration>>,
    pub(crate) export_modes: HashMap<SharedString, DesignExportMode>,
    pub(crate) animated_exports: HashMap<SharedString, DesignAnimatedExportViewData>,
    pub(crate) export_previews: HashMap<SharedString, DesignExportPreviewState>,
    pub(crate) media_paint_views: HashMap<SharedString, DesignMediaPaintViewData>,
    pub(crate) paint_edit_snapshots: HashMap<StoryPaintEditTarget, DesignPaint>,
    pub(crate) menu_preview: Option<DesignMenuPreview>,
    pub(crate) component_property_name_edits: HashMap<(SharedString, SharedString), SharedString>,
    pub(crate) component_property_reorders:
        HashMap<(SharedString, SharedString), Vec<SharedString>>,
    pub(crate) component_variant_option_name_edits:
        HashMap<(SharedString, SharedString, SharedString), SharedString>,
    pub(crate) component_variant_option_reorders:
        HashMap<(SharedString, SharedString, SharedString), Vec<SharedString>>,
    pub(crate) media_crop_targets: HashSet<StoryPaintEditTarget>,
    pub(crate) layout_grid_edit_snapshots: HashMap<StoryLayoutGridEditTarget, DesignLayoutGrid>,
    pub(crate) grid_dimensions_edit_snapshots: HashMap<SharedString, DesignLayout>,
    pub(crate) smart_selection_spacing:
        HashMap<DesignSmartSelectionAxis, DesignSmartSelectionSpacingValue>,
    pub(crate) smart_selection_edit_snapshots:
        HashMap<DesignSmartSelectionAxis, (DesignPanelTarget, DesignSmartSelectionSpacingValue)>,
    pub(crate) video_scrub_snapshots: HashMap<StoryPaintEditTarget, f32>,
    pub(crate) text_path_edit_snapshots: HashMap<SharedString, Option<DesignTextPathStartData>>,
    pub(crate) node_edit_snapshots: HashMap<StoryNodeEditTarget, DesignPanelNode>,
    pub(crate) export_edit_snapshots: StoryExportEditSnapshots,
    pub(crate) animated_export_edit_snapshots:
        HashMap<SharedString, Option<DesignAnimatedExportViewData>>,
    pub(crate) page_background_edit_snapshots: HashMap<SharedString, DesignColor>,
    pub(crate) selection_color_edit_snapshots:
        HashMap<StorySelectionColorEditTarget, Vec<DesignPanelNode>>,
    pub(crate) vector_edit_snapshots: HashMap<SharedString, Option<DesignVectorEditViewData>>,
    pub(crate) next_export_id: usize,
    pub(crate) next_media_source_id: usize,
    pub(crate) next_resource_id: usize,
    pub(crate) next_auto_layout_id: usize,
    pub(crate) selected_node: usize,
    pub(crate) inspection_scenario: DesignInspectionScenario,
    pub(crate) text_range_revision: u64,
    pub(crate) panel_width: f32,
    pub(crate) panel_resize_drag: Option<DesignPanelResizeDrag>,
    /// Whether the stacked harness (below
    /// [`DESIGN_HARNESS_STACK_BREAKPOINT`]) shows its folded story
    /// controls; collapsed by default so the inspector keeps the height.
    pub(crate) harness_controls_expanded: bool,
    pub(crate) additional_labels: bool,
    pub(crate) nudge_settings: DesignNudgeSettings,
    pub(crate) workspace_mode: DesignPanelWorkspaceMode,
    pub(crate) fixture_scroll_handle: ScrollHandle,
    pub(crate) last_action: SharedString,
}

impl DesignScreen {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Storybook>) -> Self {
        let design_nodes = fixtures::seed_design_nodes();
        let requested_design_node = std::env::var("FANTA_DESIGN_NODE")
            .unwrap_or_default()
            .to_ascii_lowercase();
        let selected_design_node = design_nodes
            .iter()
            .position(|node| {
                node.id
                    .as_ref()
                    .eq_ignore_ascii_case(&requested_design_node)
                    || node
                        .name
                        .as_ref()
                        .eq_ignore_ascii_case(&requested_design_node)
                    || node
                        .kind
                        .label()
                        .eq_ignore_ascii_case(&requested_design_node)
            })
            .unwrap_or_default();
        let design_panel_width_env = std::env::var("FANTA_DESIGN_PANEL_WIDTH").ok();
        let design_panel_width = parse_design_panel_width(design_panel_width_env.as_deref());
        let design_additional_labels_env = std::env::var("FANTA_DESIGN_ADDITIONAL_LABELS").ok();
        let design_additional_labels =
            parse_design_additional_labels(design_additional_labels_env.as_deref());
        let design_nudge_settings = DesignNudgeSettings::default();
        let design_workspace_mode = if std::env::var("FANTA_DESIGN_WORKSPACE")
            .or_else(|_| std::env::var("FANTA_TOOLBAR_MODE"))
            .is_ok_and(|mode| mode.eq_ignore_ascii_case("draw"))
        {
            DesignPanelWorkspaceMode::Draw
        } else {
            DesignPanelWorkspaceMode::Design
        };
        let design_export_configurations =
            fixtures::seed_design_export_configurations(&design_nodes);
        let mut design_export_modes = design_nodes
            .iter()
            .map(|node| (node.id.clone(), DesignExportMode::Static))
            .collect::<HashMap<_, _>>();
        if std::env::var("FANTA_DESIGN_EXPORT_MODE")
            .is_ok_and(|mode| mode.eq_ignore_ascii_case("animated"))
        {
            design_export_modes.insert(
                design_nodes[selected_design_node].id.clone(),
                DesignExportMode::Animated,
            );
        }
        let design_animated_exports = fixtures::seed_design_animated_exports(&design_nodes);
        let design_export_previews = fixtures::seed_design_export_previews(&design_nodes);
        let design_media_paint_views = fixtures::seed_design_media_paint_views(&design_nodes);
        let design_color_styles = fixtures::seed_design_color_styles();
        let design_color_style_samples = fixtures::seed_design_color_style_samples();
        let design_color_contrast = fixtures::seed_design_color_contrast(&design_nodes);
        let design_paint_styles = fixtures::seed_design_paint_styles();
        let design_paint_variables = fixtures::seed_design_paint_variables();
        let design_shaders = fixtures::seed_design_shaders();
        let design_typography_styles = fixtures::seed_design_typography_styles();
        let design_fonts = fixtures::seed_design_fonts();
        let design_effect_styles = fixtures::seed_design_effect_styles();
        let design_effect_variables = fixtures::seed_design_effect_variables();
        let design_property_variables = fixtures::seed_design_property_variables();
        let design_component_swaps = fixtures::seed_design_component_swaps();
        let design_layout_grid_styles = fixtures::seed_design_layout_grid_styles();
        let design_layout_grid_variables = fixtures::seed_design_layout_grid_variables();
        let design_frame_presets = fixtures::seed_design_frame_presets(&design_nodes);
        let design_page_view_data = fixtures::seed_design_page_view_data();
        let design_page_local_styles = fixtures::seed_design_page_local_styles();
        let design_variable_mode_views = fixtures::seed_design_variable_mode_views(&design_nodes);
        let design_viewer_properties = fixtures::seed_design_viewer_properties(&design_nodes);
        let design_inspection_scenario = if std::env::var_os("FANTA_DESIGN_SCENARIO").is_some() {
            DesignInspectionScenario::initial_from_env()
        } else {
            default_design_inspection_scenario_for_node(&design_nodes[selected_design_node])
        };
        // At launch only the scenarios whose fixtures require a specific
        // node retarget the FANTA_DESIGN_NODE selection; they resolve it
        // through the same inventory table the runtime selector uses.
        let selected_design_node = match design_inspection_scenario {
            DesignInspectionScenario::AddAutoLayoutGroup
            | DesignInspectionScenario::AddAutoLayoutMultiple
            | DesignInspectionScenario::HomogeneousMultiple => fixtures::node_index_for_selector(
                &design_nodes,
                fixtures::scenario_node_selector(design_inspection_scenario),
            ),
            _ => None,
        }
        .unwrap_or(selected_design_node);
        let design_panel = cx.new(|cx| {
            DesignPanel::new(
                "storybook-design",
                design_nodes[selected_design_node].clone(),
                window,
                cx,
            )
        });
        let initial_target = DesignPanelTarget::Nodes {
            node_ids: vec![design_nodes[selected_design_node].id.clone()],
        };
        let initial_configurations = design_export_configurations
            .get(&design_nodes[selected_design_node].id)
            .cloned()
            .unwrap_or_default();
        let initial_node = &design_nodes[selected_design_node];
        design_panel.update(cx, |panel, cx| {
            panel.set_color_style_view_data(design_color_styles.clone(), cx);
            panel.set_color_style_sample_view_data(design_color_style_samples.clone(), cx);
            panel.set_color_contrast_view_data(design_color_contrast.clone(), cx);
            panel.set_paint_style_view_data(design_paint_styles.clone(), cx);
            panel.set_paint_variable_view_data(design_paint_variables.clone(), cx);
            panel.set_shader_view_data(design_shaders.clone(), cx);
            panel.set_media_paint_view_data(
                design_media_paint_views
                    .get(&initial_node.id)
                    .cloned()
                    .unwrap_or_default(),
                cx,
            );
            panel.set_typography_style_view_data(design_typography_styles.clone(), cx);
            panel.set_font_view_data(design_fonts.clone(), cx);
            panel.set_effect_style_view_data(design_effect_styles.clone(), cx);
            panel.set_effect_variable_view_data(design_effect_variables.clone(), cx);
            panel.set_property_variable_view_data(design_property_variables.clone(), cx);
            panel.set_component_swap_view_data(design_component_swaps.clone(), cx);
            panel.set_layout_grid_style_view_data(design_layout_grid_styles.clone(), cx);
            panel.set_layout_grid_variable_view_data(design_layout_grid_variables.clone(), cx);
            if let Some(view_data) = design_frame_presets.get(&initial_node.id).cloned() {
                panel.set_frame_preset_view_data(view_data, cx);
            }
            panel.set_page_view_data(design_page_view_data.clone(), cx);
            panel.set_page_local_styles_view_data(design_page_local_styles.clone(), cx);
            if let Some(view_data) = design_variable_mode_views.get(&initial_node.id).cloned() {
                panel.set_variable_mode_view_data(view_data, cx);
            }
            if let Some(view_data) = design_viewer_properties.get(&initial_node.id).cloned() {
                panel.set_viewer_properties_view_data(view_data, cx);
            }
            panel.set_additional_labels(design_additional_labels, cx);
            panel.set_nudge_settings(design_nudge_settings, cx);
            panel.set_export_view_data(
                DesignExportViewData {
                    target: initial_target,
                    configurations: initial_configurations,
                    mode: design_export_modes
                        .get(&initial_node.id)
                        .copied()
                        .unwrap_or_default(),
                    static_capabilities: fixtures::static_export_capabilities(initial_node),
                    preview: design_export_previews.get(&initial_node.id).cloned(),
                    animated: design_animated_exports.get(&initial_node.id).cloned(),
                },
                cx,
            );
        });
        let screen = Self {
            panel: design_panel,
            nodes: design_nodes,
            color_styles: design_color_styles,
            color_style_samples: design_color_style_samples,
            paint_styles: design_paint_styles,
            paint_variables: design_paint_variables,
            shaders: design_shaders,
            typography_styles: design_typography_styles,
            fonts: design_fonts,
            effect_styles: design_effect_styles,
            effect_variables: design_effect_variables,
            property_variables: design_property_variables,
            component_swaps: design_component_swaps,
            property_bindings: HashMap::new(),
            layout_grid_styles: design_layout_grid_styles,
            layout_grid_variables: design_layout_grid_variables,
            frame_presets: design_frame_presets,
            page_view_data: design_page_view_data,
            page_local_styles: design_page_local_styles,
            variable_mode_views: design_variable_mode_views,
            viewer_properties: design_viewer_properties,
            export_configurations: design_export_configurations,
            export_modes: design_export_modes,
            animated_exports: design_animated_exports,
            export_previews: design_export_previews,
            media_paint_views: design_media_paint_views,
            paint_edit_snapshots: HashMap::new(),
            menu_preview: None,
            component_property_name_edits: HashMap::new(),
            component_property_reorders: HashMap::new(),
            component_variant_option_name_edits: HashMap::new(),
            component_variant_option_reorders: HashMap::new(),
            media_crop_targets: HashSet::new(),
            layout_grid_edit_snapshots: HashMap::new(),
            grid_dimensions_edit_snapshots: HashMap::new(),
            smart_selection_spacing: HashMap::from([
                (
                    DesignSmartSelectionAxis::Horizontal,
                    DesignSmartSelectionSpacingValue::Uniform(24.),
                ),
                (
                    DesignSmartSelectionAxis::Vertical,
                    DesignSmartSelectionSpacingValue::Mixed,
                ),
            ]),
            smart_selection_edit_snapshots: HashMap::new(),
            video_scrub_snapshots: HashMap::new(),
            text_path_edit_snapshots: HashMap::new(),
            node_edit_snapshots: HashMap::new(),
            export_edit_snapshots: HashMap::new(),
            animated_export_edit_snapshots: HashMap::new(),
            page_background_edit_snapshots: HashMap::new(),
            selection_color_edit_snapshots: HashMap::new(),
            vector_edit_snapshots: HashMap::new(),
            next_export_id: 100,
            next_media_source_id: 100,
            next_resource_id: 100,
            next_auto_layout_id: 100,
            selected_node: selected_design_node,
            inspection_scenario: design_inspection_scenario,
            text_range_revision: 0,
            panel_width: design_panel_width,
            panel_resize_drag: None,
            harness_controls_expanded: false,
            additional_labels: design_additional_labels,
            nudge_settings: design_nudge_settings,
            workspace_mode: design_workspace_mode,
            fixture_scroll_handle: ScrollHandle::new(),
            last_action: "Ready — select a node preset and exercise every inspector row".into(),
        };
        screen.apply_inspection_context(&screen.panel, cx);
        screen
    }

    pub(crate) fn paint_target_for(
        &self,
        node_id: &SharedString,
        collection: DesignPanelCollection,
    ) -> DesignPaintTarget {
        let selected = &self.nodes[self.selected_node];
        if collection == DesignPanelCollection::Fill
            && self.inspection_scenario == DesignInspectionScenario::TextEdit
            && selected.id == *node_id
            && matches!(
                selected.kind,
                DesignPanelNodeKind::Text | DesignPanelNodeKind::TextPath
            )
        {
            DesignPaintTarget::SelectedTextRangeRevision(self.text_range_revision)
        } else {
            DesignPaintTarget::WholeLayer
        }
    }

    pub(crate) fn smart_selection_view_data(
        &self,
        target: DesignPanelTarget,
    ) -> Option<DesignSmartSelectionViewData> {
        let spacing = |axis| {
            DesignSmartSelectionSpacingViewData::new(
                self.smart_selection_spacing
                    .get(&axis)
                    .copied()
                    .unwrap_or(DesignSmartSelectionSpacingValue::Mixed),
            )
        };
        let available = || DesignSmartSelectionAvailability::Available;
        match self.inspection_scenario {
            DesignInspectionScenario::SmartSelectionNone => Some(
                DesignSmartSelectionViewData::new(target, DesignSmartSelectionKind::None)
                    .with_operation(
                        DesignSmartSelectionOperation::DistributeHorizontal,
                        DesignSmartSelectionAvailability::disabled(
                            "The layers are not aligned for horizontal distribution",
                        ),
                    )
                    .with_operation(
                        DesignSmartSelectionOperation::DistributeVertical,
                        DesignSmartSelectionAvailability::disabled(
                            "The layers are not aligned for vertical distribution",
                        ),
                    )
                    .with_operation(DesignSmartSelectionOperation::TidyUp, available()),
            ),
            DesignInspectionScenario::SmartSelectionHorizontal => Some(
                DesignSmartSelectionViewData::horizontal(
                    target,
                    spacing(DesignSmartSelectionAxis::Horizontal),
                )
                .with_operation(
                    DesignSmartSelectionOperation::DistributeHorizontal,
                    available(),
                )
                .with_operation(
                    DesignSmartSelectionOperation::DistributeVertical,
                    DesignSmartSelectionAvailability::disabled(
                        "This selection does not overlap on the vertical axis",
                    ),
                )
                .with_operation(DesignSmartSelectionOperation::TidyUp, available()),
            ),
            DesignInspectionScenario::SmartSelectionVertical => Some(
                DesignSmartSelectionViewData::vertical(
                    target,
                    spacing(DesignSmartSelectionAxis::Vertical),
                )
                .with_operation(
                    DesignSmartSelectionOperation::DistributeHorizontal,
                    DesignSmartSelectionAvailability::disabled(
                        "This selection does not overlap on the horizontal axis",
                    ),
                )
                .with_operation(
                    DesignSmartSelectionOperation::DistributeVertical,
                    available(),
                )
                .with_operation(DesignSmartSelectionOperation::TidyUp, available()),
            ),
            DesignInspectionScenario::SmartSelectionTwoDimensional => Some(
                DesignSmartSelectionViewData::two_dimensional(
                    target,
                    spacing(DesignSmartSelectionAxis::Horizontal),
                    spacing(DesignSmartSelectionAxis::Vertical),
                )
                .with_operation(
                    DesignSmartSelectionOperation::DistributeHorizontal,
                    available(),
                )
                .with_operation(
                    DesignSmartSelectionOperation::DistributeVertical,
                    available(),
                )
                .with_operation(
                    DesignSmartSelectionOperation::TidyUp,
                    DesignSmartSelectionAvailability::disabled(
                        "This selection already has tidy two-dimensional spacing",
                    ),
                ),
            ),
            DesignInspectionScenario::SmartSelectionReadOnly => Some(
                DesignSmartSelectionViewData::two_dimensional(
                    target,
                    DesignSmartSelectionSpacingViewData::uniform(24.),
                    DesignSmartSelectionSpacingViewData::uniform(16.),
                )
                .with_operation(
                    DesignSmartSelectionOperation::DistributeHorizontal,
                    available(),
                )
                .with_operation(
                    DesignSmartSelectionOperation::DistributeVertical,
                    available(),
                )
                .with_operation(DesignSmartSelectionOperation::TidyUp, available())
                .read_only("One or more selected layers are locked"),
            ),
            _ => None,
        }
    }

    pub(crate) fn inspection_context(
        &self,
    ) -> (
        DesignPanelInspectionContext,
        Vec<(
            DesignPanelProperty,
            DesignPanelPropertyValueState<DesignPanelValue>,
        )>,
    ) {
        let selected = self.nodes[self.selected_node].clone();
        let widget_dimensions = (selected.kind == DesignPanelNodeKind::Widget)
            .then_some((selected.width, selected.height));
        let auto_layout_participation =
            selected
                .layout
                .as_ref()
                .map_or(
                    DesignPanelAutoLayoutParticipation::InFlow,
                    |layout| match layout.item.positioning {
                        DesignLayoutPositioning::InFlow => {
                            DesignPanelAutoLayoutParticipation::InFlow
                        }
                        DesignLayoutPositioning::Absolute => {
                            DesignPanelAutoLayoutParticipation::Ignored
                        }
                    },
                );
        let passive_corner_states = matches!(
            selected.kind,
            DesignPanelNodeKind::Text
                | DesignPanelNodeKind::TextPath
                | DesignPanelNodeKind::Ellipse
                | DesignPanelNodeKind::Line
                | DesignPanelNodeKind::Arrow
        )
        .then(|| {
            vec![(
                DesignPanelProperty::CornerRadius,
                DesignPanelPropertyValueState::Uniform(DesignPanelValue::Number(
                    selected.corner_radii[0],
                ))
                .read_only_with_reason("Corner radius is not editable for this node"),
            )]
        })
        .unwrap_or_default();
        let (context, mut property_states) = match self.inspection_scenario {
            DesignInspectionScenario::Page
            | DesignInspectionScenario::ViewOnlyPage
            | DesignInspectionScenario::RestrictedPage => (
                DesignPanelInspectionContext::page(self.inspection_scenario.permissions()),
                Vec::new(),
            ),
            DesignInspectionScenario::EditableSingle
            | DesignInspectionScenario::AddAutoLayoutGroup => (
                DesignPanelInspectionContext::single(
                    selected,
                    DesignPanelParentLayout::Freeform,
                    DesignPanelPermissions::editor(),
                ),
                passive_corner_states,
            ),
            DesignInspectionScenario::CanvasSingle => (
                DesignPanelInspectionContext::single(
                    selected,
                    DesignPanelParentLayout::Canvas,
                    DesignPanelPermissions::editor(),
                ),
                passive_corner_states,
            ),
            DesignInspectionScenario::HomogeneousMultiple => {
                let first = self
                    .nodes
                    .iter()
                    .find(|node| node.id.as_ref() == STORY_HOMOGENEOUS_MULTIPLE_NODE_IDS[0])
                    .expect("the homogeneous multiple Story keeps its first node")
                    .clone();
                let second = self
                    .nodes
                    .iter()
                    .find(|node| node.id.as_ref() == STORY_HOMOGENEOUS_MULTIPLE_NODE_IDS[1])
                    .expect("the homogeneous multiple Story keeps its second node")
                    .clone();
                story_multiple_inspection_context(&first, &second, DesignPanelPermissions::editor())
            }
            DesignInspectionScenario::EditableMultiple
            | DesignInspectionScenario::ViewOnlyMultiple
            | DesignInspectionScenario::RestrictedMultiple
            | DesignInspectionScenario::SmartSelectionNone
            | DesignInspectionScenario::SmartSelectionHorizontal
            | DesignInspectionScenario::SmartSelectionVertical
            | DesignInspectionScenario::SmartSelectionTwoDimensional
            | DesignInspectionScenario::SmartSelectionReadOnly
            | DesignInspectionScenario::AddAutoLayoutMultiple => {
                let second_index = self
                    .nodes
                    .iter()
                    .enumerate()
                    .find(|(index, node)| {
                        *index != self.selected_node && node.kind == DesignPanelNodeKind::Ellipse
                    })
                    .map(|(index, _)| index)
                    .or_else(|| {
                        self.nodes
                            .iter()
                            .enumerate()
                            .find(|(index, _)| *index != self.selected_node)
                            .map(|(index, _)| index)
                    })
                    .expect("the Design story always seeds multiple node presets");
                let second = self.nodes[second_index].clone();
                let permissions = self.inspection_scenario.permissions();
                story_multiple_inspection_context(&selected, &second, permissions)
            }
            DesignInspectionScenario::ViewOnlySingle => (
                DesignPanelInspectionContext::single(
                    selected,
                    DesignPanelParentLayout::Freeform,
                    self.inspection_scenario.permissions(),
                ),
                Vec::new(),
            ),
            DesignInspectionScenario::RestrictedSingle => (
                DesignPanelInspectionContext::single(
                    selected,
                    DesignPanelParentLayout::Freeform,
                    self.inspection_scenario.permissions(),
                ),
                Vec::new(),
            ),
            DesignInspectionScenario::PropertyStates => {
                let width = selected.width;
                let height = selected.height;
                let opacity = selected.opacity;
                (
                    DesignPanelInspectionContext::single(
                        selected,
                        DesignPanelParentLayout::Freeform,
                        DesignPanelPermissions::editor(),
                    ),
                    vec![
                        (
                            DesignPanelProperty::Width,
                            DesignPanelPropertyValueState::Uniform(DesignPanelValue::Number(width))
                                .read_only_with_reason("Controlled by the component definition"),
                        ),
                        (
                            DesignPanelProperty::Opacity,
                            DesignPanelPropertyValueState::bound(DesignPanelPropertyBinding::new(
                                "storybook-opacity-variable",
                                "Surface / Opacity",
                                DesignPanelBindingKind::Variable,
                                DesignPanelValue::Number(opacity),
                            ))
                            .read_only_with_reason("Variable mode is locked"),
                        ),
                        (
                            DesignPanelProperty::CornerRadius,
                            DesignPanelPropertyValueState::Unset,
                        ),
                        (
                            DesignPanelProperty::Height,
                            DesignPanelPropertyValueState::bound(DesignPanelPropertyBinding::new(
                                "storybook-height-style",
                                "Layout / Compact height",
                                DesignPanelBindingKind::Style,
                                DesignPanelValue::Number(height),
                            )),
                        ),
                        (
                            DesignPanelProperty::Rotation,
                            DesignPanelPropertyValueState::Mixed
                                .read_only_with_reason("Selection geometry is locked"),
                        ),
                        (
                            DesignPanelProperty::X,
                            DesignPanelPropertyValueState::Mixed.read_only_with_reason(
                                "The selected values cannot be edited together",
                            ),
                        ),
                        (
                            DesignPanelProperty::Y,
                            DesignPanelPropertyValueState::Unset
                                .read_only_with_reason("Unavailable in this edit mode"),
                        ),
                        (
                            DesignPanelProperty::FontFamily,
                            DesignPanelPropertyValueState::bound(DesignPanelPropertyBinding::new(
                                "type-style-body",
                                "Typography / Body",
                                DesignPanelBindingKind::Style,
                                DesignPanelValue::Text("Inter".into()),
                            )),
                        ),
                        (
                            DesignPanelProperty::FontStyle,
                            DesignPanelPropertyValueState::bound(DesignPanelPropertyBinding::new(
                                "type-style-body-locked",
                                "Typography / Body locked",
                                DesignPanelBindingKind::Style,
                                DesignPanelValue::Text("Regular".into()),
                            ))
                            .read_only_with_reason("The text style is locked"),
                        ),
                    ],
                )
            }
            DesignInspectionScenario::AutoLayoutChild => (
                DesignPanelInspectionContext::single(
                    selected,
                    DesignPanelParentLayout::auto_layout(
                        DesignPanelAutoLayoutDirection::Horizontal,
                        DesignPanelAutoLayoutWrap::NoWrap,
                        auto_layout_participation,
                    ),
                    DesignPanelPermissions::editor(),
                ),
                Vec::new(),
            ),
            DesignInspectionScenario::AutoLayoutIgnored => (
                DesignPanelInspectionContext::single(
                    selected,
                    DesignPanelParentLayout::auto_layout(
                        DesignPanelAutoLayoutDirection::Horizontal,
                        DesignPanelAutoLayoutWrap::NoWrap,
                        auto_layout_participation,
                    ),
                    DesignPanelPermissions::editor(),
                ),
                Vec::new(),
            ),
            DesignInspectionScenario::AutoLayoutVerticalChild => (
                DesignPanelInspectionContext::single(
                    selected,
                    DesignPanelParentLayout::auto_layout(
                        DesignPanelAutoLayoutDirection::Vertical,
                        DesignPanelAutoLayoutWrap::NoWrap,
                        auto_layout_participation,
                    ),
                    DesignPanelPermissions::editor(),
                ),
                Vec::new(),
            ),
            DesignInspectionScenario::GridChild => (
                DesignPanelInspectionContext::single(
                    selected,
                    DesignPanelParentLayout::auto_layout(
                        DesignPanelAutoLayoutDirection::Grid,
                        DesignPanelAutoLayoutWrap::NoWrap,
                        auto_layout_participation,
                    ),
                    DesignPanelPermissions::editor(),
                ),
                Vec::new(),
            ),
            DesignInspectionScenario::TextEdit => (
                story_text_edit_inspection_context(selected, self.text_range_revision),
                Vec::new(),
            ),
            DesignInspectionScenario::VectorEdit => {
                let mut selected = selected;
                if let Some(stroke) = selected.stroke.as_mut() {
                    stroke.edit_context = stroke.edit_context.with_vertex_selection(1, 1);
                }
                (
                    DesignPanelInspectionContext::single(
                        selected,
                        DesignPanelParentLayout::Freeform,
                        DesignPanelPermissions::editor(),
                    )
                    .with_edit_mode(DesignPanelEditMode::Vector)
                    .expect("the Vector story preset is an editable single selection"),
                    Vec::new(),
                )
            }
        };
        if let Some((width, height)) = widget_dimensions
            && context.selection().kind() == fanta_gpui::prelude::DesignPanelSelectionKind::Single
        {
            property_states.retain(|(property, _)| {
                !matches!(
                    property,
                    DesignPanelProperty::Width | DesignPanelProperty::Height
                )
            });
            property_states.extend(story_widget_dimension_property_states(width, height));
        }
        (context, property_states)
    }

    pub(crate) fn apply_inspection_context(
        &self,
        panel: &Entity<DesignPanel>,
        cx: &mut Context<Storybook>,
    ) {
        let (inspection_context, mut property_states) = self.inspection_context();
        let selection_header_view_data = match inspection_context.selection().kind() {
            fanta_gpui::prelude::DesignPanelSelectionKind::None => None,
            fanta_gpui::prelude::DesignPanelSelectionKind::Single => {
                let node = &inspection_context.selection().items()[0];
                let mut view_data = DesignSelectionHeaderViewData::for_node_kind(node.kind);
                if node.kind == DesignPanelNodeKind::Ellipse
                    && self.inspection_scenario != DesignInspectionScenario::CanvasSingle
                {
                    view_data.primary_controls.insert(
                        0,
                        DesignSelectionHeaderControl::direct(
                            DesignSelectionHeaderControlKind::SelectMatchingLayers,
                        ),
                    );
                }
                if node.kind == DesignPanelNodeKind::Other {
                    let [plugin_menu, plugin_action] = storybook_plugin_header_controls();
                    view_data.primary_controls.push(plugin_menu);
                    view_data.overflow_controls.push(plugin_action);
                }
                Some(view_data)
            }
            fanta_gpui::prelude::DesignPanelSelectionKind::Multiple => {
                let count = inspection_context.selection().len();
                Some(DesignSelectionHeaderViewData::for_multiple_selection(count))
            }
        };
        let target = match inspection_context.selection().kind() {
            fanta_gpui::prelude::DesignPanelSelectionKind::None => DesignPanelTarget::Page {
                page_id: "storybook-page".into(),
            },
            fanta_gpui::prelude::DesignPanelSelectionKind::Single
            | fanta_gpui::prelude::DesignPanelSelectionKind::Multiple => DesignPanelTarget::Nodes {
                node_ids: inspection_context
                    .selection()
                    .items()
                    .iter()
                    .map(|node| node.id.clone())
                    .collect(),
            },
        };
        let selection_header_target = target.clone();
        let add_auto_layout_view_data = match self.inspection_scenario {
            DesignInspectionScenario::AddAutoLayoutGroup
                if inspection_context.selection().kind()
                    == fanta_gpui::prelude::DesignPanelSelectionKind::Single
                    && inspection_context
                        .selection()
                        .items()
                        .first()
                        .is_some_and(|node| node.kind == DesignPanelNodeKind::Group) =>
            {
                Some(DesignAddAutoLayoutViewData::eligible(target.clone()))
            }
            DesignInspectionScenario::AddAutoLayoutMultiple
                if inspection_context.selection().kind()
                    == fanta_gpui::prelude::DesignPanelSelectionKind::Multiple =>
            {
                Some(DesignAddAutoLayoutViewData::eligible(target.clone()))
            }
            _ if self.workspace_mode == DesignPanelWorkspaceMode::Draw
                && inspection_context.selection().kind()
                    == fanta_gpui::prelude::DesignPanelSelectionKind::Single
                && inspection_context
                    .selection()
                    .items()
                    .first()
                    .is_some_and(|node| {
                        node.supports_auto_layout_container()
                            && node
                                .layout
                                .as_ref()
                                .is_none_or(|layout| layout.mode == DesignLayoutMode::None)
                    }) =>
            {
                Some(DesignAddAutoLayoutViewData::eligible(target.clone()))
            }
            _ => None,
        };
        let smart_selection_view_data = self.smart_selection_view_data(target.clone());
        let export_key = match &target {
            DesignPanelTarget::Page { page_id } => page_id.clone(),
            DesignPanelTarget::Nodes { node_ids } => node_ids
                .first()
                .cloned()
                .unwrap_or_else(|| "storybook-page".into()),
        };
        let media_paint_view_data = self
            .media_paint_views
            .get(&export_key)
            .cloned()
            .unwrap_or_default();
        let shader_view_data = self.shaders.clone();
        let color_style_sample_view_data = self.color_style_samples.clone();
        let color_contrast_view_data = fixtures::seed_design_color_contrast(&self.nodes);
        let paint_style_view_data = self.paint_styles.clone();
        let paint_variable_view_data = self.paint_variables.clone();
        let layout_grid_style_view_data = self.layout_grid_styles.clone();
        let layout_grid_variable_view_data = self.layout_grid_variables.clone();
        let frame_preset_view_data = self.frame_presets.get(&export_key).cloned();
        let page_view_data = self.page_view_data.clone();
        let page_local_styles = self.page_local_styles.clone();
        let variable_mode_view_data = match &target {
            DesignPanelTarget::Page { .. } => self.variable_mode_views.get(&export_key).cloned(),
            DesignPanelTarget::Nodes { node_ids } if node_ids.len() == 1 => {
                self.variable_mode_views.get(&export_key).cloned()
            }
            DesignPanelTarget::Nodes { .. } => None,
        };
        let viewer_properties_view_data = match &target {
            DesignPanelTarget::Nodes { node_ids } if node_ids.len() == 1 => {
                self.viewer_properties.get(&export_key).cloned()
            }
            DesignPanelTarget::Page { .. } | DesignPanelTarget::Nodes { .. } => None,
        };
        let draw_appearance_view_data =
            story_draw_appearance_view_data(&inspection_context, &target);
        let export_view_data = story_export_projection(
            target,
            &self.nodes,
            &self.export_configurations,
            &self.export_modes,
            &self.animated_exports,
            &self.export_previews,
        );
        let is_page = inspection_context.selection().kind()
            == fanta_gpui::prelude::DesignPanelSelectionKind::None;
        let selected_node_ids = inspection_context
            .selection()
            .items()
            .iter()
            .map(|node| node.id.clone())
            .collect::<Vec<_>>();
        if let Some(selected_node_id) = selected_node_ids.first() {
            for (node_id, property) in self.property_bindings.keys() {
                if node_id != selected_node_id {
                    continue;
                }
                let Some(binding) = story_common_property_binding(
                    &self.property_bindings,
                    &selected_node_ids,
                    *property,
                ) else {
                    continue;
                };
                if let Some((_, state)) = property_states
                    .iter_mut()
                    .find(|(candidate, _)| candidate == property)
                {
                    let protected = state.is_read_only()
                        || state
                            .binding()
                            .is_some_and(|binding| binding.kind() == DesignPanelBindingKind::Style);
                    if !protected {
                        *state = DesignPanelPropertyValueState::bound(binding.clone());
                    }
                } else {
                    property_states.push((
                        *property,
                        DesignPanelPropertyValueState::bound(binding.clone()),
                    ));
                }
            }
        }
        panel.update(cx, |panel, cx| {
            panel.set_workspace_mode(self.workspace_mode, cx);
            if is_page {
                panel.set_node(
                    DesignPanelNode::new("storybook-page", "Page 5", DesignPanelNodeKind::Frame),
                    cx,
                );
            }
            panel.set_inspection_context(inspection_context, cx);
            if let Some(view_data) = selection_header_view_data {
                panel.set_selection_header_view_data_for_target(
                    selection_header_target,
                    view_data,
                    cx,
                );
            } else {
                panel.clear_selection_header_view_data(cx);
            }
            panel.set_property_value_states(property_states, cx);
            panel.set_property_variable_view_data(self.property_variables.clone(), cx);
            panel.set_component_swap_view_data(self.component_swaps.clone(), cx);
            panel.set_media_paint_view_data(media_paint_view_data, cx);
            panel.set_shader_view_data(shader_view_data, cx);
            panel.set_color_style_sample_view_data(color_style_sample_view_data, cx);
            panel.set_color_contrast_view_data(color_contrast_view_data, cx);
            panel.set_paint_style_view_data(paint_style_view_data, cx);
            panel.set_paint_variable_view_data(paint_variable_view_data, cx);
            panel.set_layout_grid_style_view_data(layout_grid_style_view_data, cx);
            panel.set_layout_grid_variable_view_data(layout_grid_variable_view_data, cx);
            if let Some(view_data) = frame_preset_view_data {
                panel.set_frame_preset_view_data(view_data, cx);
            } else {
                panel.clear_frame_preset_view_data(cx);
            }
            panel.set_page_view_data(page_view_data, cx);
            panel.set_page_local_styles_view_data(page_local_styles, cx);
            if let Some(view_data) = variable_mode_view_data {
                panel.set_variable_mode_view_data(view_data, cx);
            } else {
                panel.clear_variable_mode_view_data(cx);
            }
            if let Some(view_data) = viewer_properties_view_data {
                panel.set_viewer_properties_view_data(view_data, cx);
            } else {
                panel.clear_viewer_properties_view_data(cx);
            }
            if let Some(view_data) = add_auto_layout_view_data {
                panel.set_add_auto_layout_view_data(view_data, cx);
            } else {
                panel.clear_add_auto_layout_view_data(cx);
            }
            if let Some(view_data) = draw_appearance_view_data {
                assert!(
                    panel.set_draw_appearance_view_data(view_data, cx),
                    "Storybook Draw appearance data must remain valid"
                );
            } else {
                panel.clear_draw_appearance_view_data(cx);
            }
            if let Some(view_data) = smart_selection_view_data {
                panel.set_smart_selection_view_data(view_data, cx);
            } else {
                panel.clear_smart_selection_view_data(cx);
            }
            panel.set_export_view_data(export_view_data, cx);
        });
    }

    pub(crate) fn activate_inspection_scenario(
        &mut self,
        scenario: DesignInspectionScenario,
        cx: &mut Context<Storybook>,
    ) {
        if scenario == DesignInspectionScenario::AddAutoLayoutGroup
            && let Some(group) = self
                .nodes
                .iter_mut()
                .find(|node| node.id.as_ref() == "add-auto-layout-group")
        {
            *group = DesignPanelNode::new(
                "add-auto-layout-group",
                "Navigation cluster · Group",
                DesignPanelNodeKind::Group,
            );
        }
        // VectorEdit keeps an already-vector-capable preset; every other
        // scenario resolves its required node through the shared inventory
        // table in `fixtures`.
        let required_index = if scenario == DesignInspectionScenario::VectorEdit
            && matches!(
                self.nodes[self.selected_node].kind,
                DesignPanelNodeKind::Vector | DesignPanelNodeKind::TextPath
            ) {
            None
        } else {
            fixtures::node_index_for_selector(
                &self.nodes,
                fixtures::scenario_node_selector(scenario),
            )
        };
        if let Some(index) = required_index {
            self.selected_node = index;
        }
        for (axis, (_, original)) in std::mem::take(&mut self.smart_selection_edit_snapshots) {
            self.smart_selection_spacing.insert(axis, original);
        }
        self.inspection_scenario = scenario;
        self.apply_inspection_context(&self.panel, cx);
        self.last_action = format!("Story switched to {}", scenario.label()).into();
    }

    pub(crate) fn advance_text_range(&mut self, cx: &mut Context<Storybook>) {
        if self.inspection_scenario != DesignInspectionScenario::TextEdit {
            return;
        }
        self.text_range_revision = self.text_range_revision.wrapping_add(1);
        self.apply_inspection_context(&self.panel, cx);
        self.last_action = format!(
            "Host selected a new character range (revision {})",
            self.text_range_revision
        )
        .into();
        cx.notify();
    }

    pub(crate) fn set_panel_width(
        &mut self,
        width: f32,
        interaction: &'static str,
        cx: &mut Context<Storybook>,
    ) {
        self.panel_resize_drag = None;
        self.panel_width = clamp_design_panel_width(width);
        self.last_action = format!(
            "Story {interaction} the inspector viewport to {:.0} px — no document intent",
            self.panel_width
        )
        .into();
        cx.notify();
    }

    pub(crate) fn toggle_harness_controls(&mut self, cx: &mut Context<Storybook>) {
        self.harness_controls_expanded = !self.harness_controls_expanded;
        self.last_action = if self.harness_controls_expanded {
            "Story expanded the stacked harness controls — no document intent"
        } else {
            "Story collapsed the stacked harness controls — no document intent"
        }
        .into();
        cx.notify();
    }

    pub(crate) fn begin_panel_resize(&mut self, pointer_x: f32, cx: &mut Context<Storybook>) {
        self.panel_resize_drag = Some(DesignPanelResizeDrag::new(pointer_x, self.panel_width));
        self.last_action = format!("Story began resizing at {:.0} px", self.panel_width).into();
        cx.notify();
    }

    pub(crate) fn update_panel_resize(&mut self, pointer_x: f32, cx: &mut Context<Storybook>) {
        let Some(drag) = self.panel_resize_drag else {
            return;
        };
        let width = drag.width_at(pointer_x);
        if (self.panel_width - width).abs() < f32::EPSILON {
            return;
        }
        self.panel_width = width;
        self.last_action = format!("Story is resizing the inspector to {width:.0} px").into();
        cx.notify();
    }

    pub(crate) fn finish_panel_resize(&mut self, cx: &mut Context<Storybook>) {
        if self.panel_resize_drag.take().is_none() {
            return;
        }
        self.last_action = format!(
            "Story resized the inspector viewport to {:.0} px — no document intent",
            self.panel_width
        )
        .into();
        cx.notify();
    }

    pub(crate) fn handle_panel_resize_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Storybook>,
    ) {
        let modifiers = event.keystroke.modifiers;
        if modifiers.control || modifiers.alt || modifiers.platform || modifiers.function {
            return;
        }
        let Some(width) = design_panel_width_after_key(
            self.panel_width,
            event.keystroke.key.as_str(),
            modifiers.shift,
        ) else {
            return;
        };
        window.prevent_default();
        cx.stop_propagation();
        self.set_panel_width(width, "keyboard-resized", cx);
    }

    pub(crate) fn set_workspace_mode(
        &mut self,
        workspace_mode: DesignPanelWorkspaceMode,
        source: &'static str,
        cx: &mut Context<Storybook>,
    ) {
        self.workspace_mode = workspace_mode;
        self.apply_inspection_context(&self.panel, cx);
        self.last_action = format!(
            "{source} set the inspector workspace to {} — surface and document state preserved",
            workspace_mode.label()
        )
        .into();
        cx.notify();
    }
}

pub(crate) fn story_node_edit_transaction(
    action: &DesignPanelAction,
) -> Option<(StoryNodeEditTarget, DesignPanelEditPhase)> {
    let (node_id, transaction_id, phase) = match action {
        DesignPanelAction::TypographyPropertyEditRequested {
            node_id,
            target,
            property,
            phase,
            ..
        } => (
            node_id,
            format!("typography:{target:?}:{property:?}"),
            *phase,
        ),
        DesignPanelAction::TypographyVariableAxisEditRequested {
            node_id,
            target,
            tag,
            phase,
            ..
        } => (node_id, format!("typography-axis:{target:?}:{tag}"), *phase),
        DesignPanelAction::PropertyEditRequested {
            node_id,
            property,
            phase,
            ..
        } => (node_id, format!("property:{property:?}"), *phase),
        DesignPanelAction::TransformModifierChangeRequested {
            node_id,
            modifier_id,
            index,
            phase,
            ..
        } => (
            node_id,
            format!("transform-modifier:{modifier_id}:{index}"),
            *phase,
        ),
        DesignPanelAction::ComponentPropertyEditRequested {
            node_id,
            property_id,
            phase,
            ..
        } => (node_id, format!("component-property:{property_id}"), *phase),
        DesignPanelAction::SlotSettingsChangeRequested {
            node_id,
            property_id,
            phase,
            ..
        } => (node_id, format!("slot-settings:{property_id}"), *phase),
        DesignPanelAction::EffectEditRequested {
            node_id,
            effect_id,
            index,
            property,
            shader_property_id,
            phase,
            ..
        } => (
            node_id,
            format!(
                "effect:{effect_id}:{index}:{property:?}:{}",
                shader_property_id
                    .as_ref()
                    .map(|property_id| property_id.as_ref())
                    .unwrap_or("")
            ),
            *phase,
        ),
        // Paint, TextPath, and vector-edit transactions keep specialized
        // stable-identity snapshots because they resolve leaves differently.
        _ => return None,
    };
    Some((
        StoryNodeEditTarget::new(node_id.clone(), transaction_id),
        phase,
    ))
}
pub(crate) fn begin_story_node_edit(
    node: &DesignPanelNode,
    snapshots: &mut HashMap<StoryNodeEditTarget, DesignPanelNode>,
    target: &StoryNodeEditTarget,
    phase: DesignPanelEditPhase,
) {
    if phase == DesignPanelEditPhase::Begin {
        snapshots
            .entry(target.clone())
            .or_insert_with(|| node.clone());
    }
}
pub(crate) fn finish_story_node_edit(
    node: &mut DesignPanelNode,
    snapshots: &mut HashMap<StoryNodeEditTarget, DesignPanelNode>,
    target: StoryNodeEditTarget,
    phase: DesignPanelEditPhase,
) {
    match phase {
        DesignPanelEditPhase::Commit => {
            snapshots.remove(&target);
        }
        DesignPanelEditPhase::Cancel => {
            if let Some(original) = snapshots.remove(&target) {
                *node = original;
            }
        }
        DesignPanelEditPhase::Begin | DesignPanelEditPhase::Preview => {}
    }
}
pub(crate) fn story_design_target(
    context: &DesignPanelInspectionContext,
) -> Option<DesignPanelTarget> {
    (!context.selection().is_empty()).then(|| DesignPanelTarget::Nodes {
        node_ids: context
            .selection()
            .items()
            .iter()
            .map(|node| node.id.clone())
            .collect(),
    })
}
pub(crate) fn story_draw_appearance_view_data(
    context: &DesignPanelInspectionContext,
    target: &DesignPanelTarget,
) -> Option<DesignDrawAppearanceViewData> {
    if story_design_target(context).as_ref() != Some(target) {
        return None;
    }
    let nodes = context.selection().items();
    let first = nodes.first()?;
    if nodes.iter().skip(1).any(|node| {
        node.width != first.width
            || node.height != first.height
            || !node.width.is_finite()
            || !node.height.is_finite()
    }) || !first.width.is_finite()
        || !first.height.is_finite()
    {
        return None;
    }

    let mock_corner_maximum = first.width.abs().max(first.height.abs()).max(100.).ceil();
    DesignDrawSliderRange::new(0., mock_corner_maximum, 1.)
        .map(|range| DesignDrawAppearanceViewData::new(target.clone(), range))
        .filter(DesignDrawAppearanceViewData::is_valid)
}
