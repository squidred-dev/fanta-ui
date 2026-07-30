#![forbid(unsafe_code)]
#![allow(deprecated)]

mod configuration;
mod design_host;
mod fixtures;
mod gallery;
mod icon_gallery;

use std::collections::{HashMap, HashSet};

use configuration::*;
use design_host::*;
use fanta_gpui::prelude::*;
use fixtures::*;
use gpui::{
    AnyElement, App, AppContext as _, Application, Bounds, ClipboardItem, Context, Entity,
    Focusable as _, InteractiveElement as _, IntoElement, KeyDownEvent, Menu, MenuItem,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement as _, Render,
    ScrollHandle, SharedString, StatefulInteractiveElement as _, Styled as _, Subscription,
    TitlebarOptions, Window, WindowBounds, WindowId, WindowOptions, div,
    prelude::FluentBuilder as _, px, rgba, size,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, Root, Selectable as _, Sizable as _, StyledExt as _, Theme,
    ThemeMode,
    button::{Button, ButtonCustomVariant, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    menu::AppMenuBar,
    resizable::{h_resizable, resizable_panel},
    sidebar::{Sidebar, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
    v_flex,
};
use icon_gallery::IconGallery;

gpui::actions!(fanta_storybook, [OpenStoryWindow, ToggleGalleryTheme]);

const STORYBOOK_KEY_CONTEXT: &str = "FantaStorybook";

struct Storybook {
    active_story: StoryKind,
    launch_mode: StorybookLaunchMode,
    story_windows: HashMap<WindowId, StoryKind>,
    gallery_theme_mode: ThemeMode,
    gallery_search_input: Entity<InputState>,
    gallery_menu_bar: Entity<AppMenuBar>,
    gallery_story_scroll_handle: ScrollHandle,
    icon_gallery: Entity<IconGallery>,
    variables_page: Entity<VariablesPage>,
    variables_view_data: VariablesViewData,
    variables_last_action: SharedString,
    assets_panel: Entity<AssetsPanel>,
    assets_view_data: AssetsViewData,
    assets_last_action: SharedString,
    prototype_panel: Entity<PrototypePanel>,
    prototype_view_data: PrototypeViewData,
    prototype_last_action: SharedString,
    timeline: Entity<Timeline>,
    timeline_view_data: TimelineViewData,
    timeline_last_action: SharedString,
    pseudo_editor: Entity<PseudoEditor>,
    pseudo_left_surface: PseudoEditorLeftSurface,
    pseudo_right_surface: PseudoEditorRightSurface,
    pseudo_variables_visible: bool,
    pseudo_last_action: SharedString,
    pages_panel: Entity<PagesPanel>,
    pages: Vec<PagesPanelItem>,
    active_page: SharedString,
    elements: Vec<MockElement>,
    pages_last_action: SharedString,
    next_page_id: usize,
    layers_panel: Entity<LayersPanel>,
    layers: Vec<LayersPanelItem>,
    selected_layers: Vec<SharedString>,
    expanded_layers: Vec<SharedString>,
    layer_selection_anchor: Option<SharedString>,
    layers_last_action: SharedString,
    design_panel: Entity<DesignPanel>,
    design_nodes: Vec<DesignPanelNode>,
    design_color_styles: DesignColorStyleViewData,
    design_color_style_samples: DesignColorStyleSampleViewData,
    design_paint_styles: DesignPaintStyleViewData,
    design_paint_variables: DesignPaintVariableViewData,
    design_shaders: DesignShaderViewData,
    design_typography_styles: DesignTypographyStyleViewData,
    design_fonts: DesignFontViewData,
    design_effect_styles: DesignEffectStyleViewData,
    design_effect_variables: DesignEffectVariableViewData,
    design_property_variables: DesignVariableViewData,
    design_component_swaps: DesignComponentSwapViewData,
    design_property_bindings:
        HashMap<(SharedString, DesignPanelProperty), DesignPanelPropertyBinding<DesignPanelValue>>,
    design_layout_grid_styles: DesignLayoutGridStyleViewData,
    design_layout_grid_variables: DesignLayoutGridVariableViewData,
    design_frame_presets: HashMap<SharedString, DesignFramePresetViewData>,
    design_page_view_data: DesignPageViewData,
    design_page_local_styles: DesignPageLocalStylesViewData,
    design_variable_mode_views: HashMap<SharedString, DesignVariableModeViewData>,
    design_viewer_properties: HashMap<SharedString, DesignViewerPropertiesViewData>,
    design_export_configurations: HashMap<SharedString, Vec<DesignExportConfiguration>>,
    design_export_modes: HashMap<SharedString, DesignExportMode>,
    design_animated_exports: HashMap<SharedString, DesignAnimatedExportViewData>,
    design_export_previews: HashMap<SharedString, DesignExportPreviewState>,
    design_media_paint_views: HashMap<SharedString, DesignMediaPaintViewData>,
    design_paint_edit_snapshots: HashMap<StoryPaintEditTarget, DesignPaint>,
    design_menu_preview: Option<DesignMenuPreview>,
    design_component_property_name_edits: HashMap<(SharedString, SharedString), SharedString>,
    design_component_property_reorders: HashMap<(SharedString, SharedString), Vec<SharedString>>,
    design_component_variant_option_name_edits:
        HashMap<(SharedString, SharedString, SharedString), SharedString>,
    design_component_variant_option_reorders:
        HashMap<(SharedString, SharedString, SharedString), Vec<SharedString>>,
    design_media_crop_targets: HashSet<StoryPaintEditTarget>,
    design_layout_grid_edit_snapshots: HashMap<StoryLayoutGridEditTarget, DesignLayoutGrid>,
    design_grid_dimensions_edit_snapshots: HashMap<SharedString, DesignLayout>,
    design_smart_selection_spacing:
        HashMap<DesignSmartSelectionAxis, DesignSmartSelectionSpacingValue>,
    design_smart_selection_edit_snapshots:
        HashMap<DesignSmartSelectionAxis, (DesignPanelTarget, DesignSmartSelectionSpacingValue)>,
    design_video_scrub_snapshots: HashMap<StoryPaintEditTarget, f32>,
    design_text_path_edit_snapshots: HashMap<SharedString, Option<DesignTextPathStartData>>,
    design_node_edit_snapshots: HashMap<StoryNodeEditTarget, DesignPanelNode>,
    design_export_edit_snapshots: StoryExportEditSnapshots,
    design_animated_export_edit_snapshots:
        HashMap<SharedString, Option<DesignAnimatedExportViewData>>,
    design_page_background_edit_snapshots: HashMap<SharedString, DesignColor>,
    design_selection_color_edit_snapshots:
        HashMap<StorySelectionColorEditTarget, Vec<DesignPanelNode>>,
    design_vector_edit_snapshots: HashMap<SharedString, Option<DesignVectorEditViewData>>,
    next_design_export_id: usize,
    next_design_media_source_id: usize,
    next_design_resource_id: usize,
    next_design_auto_layout_id: usize,
    selected_design_node: usize,
    design_inspection_scenario: DesignInspectionScenario,
    design_text_range_revision: u64,
    design_panel_width: f32,
    design_panel_resize_drag: Option<DesignPanelResizeDrag>,
    design_additional_labels: bool,
    design_nudge_settings: DesignNudgeSettings,
    design_workspace_mode: DesignPanelWorkspaceMode,
    design_fixture_scroll_handle: ScrollHandle,
    design_last_action: SharedString,
    toolbar: Entity<EditorToolbar>,
    toolbar_mode: ToolbarMode,
    toolbar_tool: ToolbarTool,
    toolbar_zoom: u16,
    toolbar_draw_options: DrawToolbarOptions,
    toolbar_dev_options: DevToolbarOptions,
    toolbar_motion_options: MotionToolbarOptions,
    toolbar_last_action: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl Storybook {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let launch = storybook_launch_from_env();
        let gallery_search_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Search components..."));
        let gallery_menu_bar = AppMenuBar::new(window, cx);
        let gallery_story_scroll_handle = ScrollHandle::new();
        let icon_gallery = cx.new(|cx| IconGallery::new(window, cx));
        let variables_view_data = seed_variables_view_data();
        let variables_page = cx.new(|cx| {
            VariablesPage::new(
                "storybook-variables",
                variables_view_data.clone(),
                window,
                cx,
            )
        });
        let assets_view_data =
            seed_assets_view_data(launch.mode == StorybookLaunchMode::ReferenceFixture);
        let assets_panel =
            cx.new(|cx| AssetsPanel::new("storybook-assets", assets_view_data.clone(), window, cx));
        let prototype_view_data = PrototypeViewData::default();
        let prototype_panel = cx
            .new(|cx| PrototypePanel::new("storybook-prototype", prototype_view_data.clone(), cx));
        let timeline_view_data = TimelineViewData::default();
        let timeline =
            cx.new(|cx| Timeline::new("storybook-timeline", timeline_view_data.clone(), cx));

        let pages = seed_pages();
        let active_page: SharedString = "page-5".into();
        let pages_panel =
            cx.new(|cx| PagesPanel::new("storybook-pages", pages.clone(), window, cx));
        pages_panel.update(cx, |panel, cx| {
            panel.set_selected_page(Some(active_page.clone()), cx);
        });

        let layers = seed_layers();
        let selected_layers = vec!["hero-title".into()];
        let expanded_layers = vec![
            "checkout".into(),
            "header".into(),
            "controls".into(),
            "button-component".into(),
            "vector-art".into(),
        ];
        let layers_panel =
            cx.new(|cx| LayersPanel::new("storybook-layers", layers.clone(), window, cx));
        layers_panel.update(cx, |panel, cx| {
            panel.set_selected_node_ids(selected_layers.clone(), cx);
            panel.set_expanded_node_ids(expanded_layers.clone(), cx);
        });
        let design_nodes = Self::seed_design_nodes();
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
        let design_export_configurations = Self::seed_design_export_configurations(&design_nodes);
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
        let design_animated_exports = Self::seed_design_animated_exports(&design_nodes);
        let design_export_previews = Self::seed_design_export_previews(&design_nodes);
        let design_media_paint_views = Self::seed_design_media_paint_views(&design_nodes);
        let design_color_styles = Self::seed_design_color_styles();
        let design_color_style_samples = Self::seed_design_color_style_samples();
        let design_color_contrast = Self::seed_design_color_contrast(&design_nodes);
        let design_paint_styles = Self::seed_design_paint_styles();
        let design_paint_variables = Self::seed_design_paint_variables();
        let design_shaders = Self::seed_design_shaders();
        let design_typography_styles = Self::seed_design_typography_styles();
        let design_fonts = Self::seed_design_fonts();
        let design_effect_styles = Self::seed_design_effect_styles();
        let design_effect_variables = Self::seed_design_effect_variables();
        let design_property_variables = Self::seed_design_property_variables();
        let design_component_swaps = Self::seed_design_component_swaps();
        let design_layout_grid_styles = Self::seed_design_layout_grid_styles();
        let design_layout_grid_variables = Self::seed_design_layout_grid_variables();
        let design_frame_presets = Self::seed_design_frame_presets(&design_nodes);
        let design_page_view_data = Self::seed_design_page_view_data();
        let design_page_local_styles = Self::seed_design_page_local_styles();
        let design_variable_mode_views = Self::seed_design_variable_mode_views(&design_nodes);
        let design_viewer_properties = Self::seed_design_viewer_properties(&design_nodes);
        let design_inspection_scenario = if std::env::var_os("FANTA_DESIGN_SCENARIO").is_some() {
            DesignInspectionScenario::initial_from_env()
        } else {
            default_design_inspection_scenario_for_node(&design_nodes[selected_design_node])
        };
        let selected_design_node = match design_inspection_scenario {
            DesignInspectionScenario::AddAutoLayoutGroup => design_nodes
                .iter()
                .position(|node| node.id.as_ref() == "add-auto-layout-group"),
            DesignInspectionScenario::AddAutoLayoutMultiple => design_nodes
                .iter()
                .position(|node| node.kind == DesignPanelNodeKind::Rectangle),
            DesignInspectionScenario::HomogeneousMultiple => design_nodes
                .iter()
                .position(|node| node.id.as_ref() == STORY_HOMOGENEOUS_MULTIPLE_NODE_IDS[0]),
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
                    static_capabilities: Self::static_export_capabilities(initial_node),
                    preview: design_export_previews.get(&initial_node.id).cloned(),
                    animated: design_animated_exports.get(&initial_node.id).cloned(),
                },
                cx,
            );
        });
        let toolbar_mode = match std::env::var("FANTA_TOOLBAR_MODE")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "draw" => ToolbarMode::Draw,
            "motion" => ToolbarMode::Motion,
            "dev" => ToolbarMode::Dev,
            _ => ToolbarMode::Design,
        };
        let toolbar_tool = match toolbar_mode {
            ToolbarMode::Draw => ToolbarTool::Brush,
            ToolbarMode::Design => ToolbarTool::Move,
            ToolbarMode::Motion => ToolbarTool::MotionSelect,
            ToolbarMode::Dev => ToolbarTool::Inspect,
        };
        let toolbar_zoom = 100;
        let toolbar_draw_options = DrawToolbarOptions::default();
        let toolbar_dev_options = DevToolbarOptions::default();
        let toolbar_motion_options = MotionToolbarOptions::default();
        let toolbar = cx.new(|cx| {
            EditorToolbar::new(
                "storybook-toolbar",
                toolbar_mode,
                toolbar_tool,
                toolbar_zoom,
                window,
                cx,
            )
        });
        toolbar.update(cx, |toolbar, cx| {
            toolbar.set_agent_options(
                AgentToolbarOptions::new("Hero frame").suggestions([
                    "Explore 3 directions",
                    "Polish this screen",
                    "Animate the hero",
                ]),
                cx,
            );
        });
        toolbar.focus_handle(cx).focus(window);
        match std::env::var("FANTA_TOOLBAR_OVERLAY")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "actions" => toolbar.update(cx, |toolbar, cx| {
                toolbar.open_actions(window, cx);
            }),
            "agent" => toolbar.update(cx, |toolbar, cx| {
                toolbar.open_agent(window, cx);
            }),
            _ => {}
        }

        let pseudo_left_surface = PseudoEditorLeftSurface::default();
        let pseudo_right_surface = PseudoEditorRightSurface::default();
        let pseudo_variables_visible = false;
        let pseudo_editor = cx.new(|cx| {
            PseudoEditor::new(
                "storybook-pseudo-editor",
                PseudoEditorChildren {
                    pages: pages_panel.clone(),
                    layers: layers_panel.clone(),
                    assets: assets_panel.clone(),
                    design: design_panel.clone(),
                    prototype: prototype_panel.clone(),
                    timeline: timeline.clone(),
                    toolbar: toolbar.clone(),
                    variables: variables_page.clone(),
                },
                cx,
            )
        });

        let mut subscriptions = vec![
            cx.subscribe(&gallery_search_input, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            }),
            cx.subscribe(
                &variables_page,
                |story, panel, action: &VariablesAction, cx| {
                    story.handle_variables_action(panel, action, cx);
                },
            ),
            cx.subscribe(
                &assets_panel,
                |story, panel, action: &AssetsPanelAction, cx| {
                    story.handle_assets_action(panel, action, cx);
                },
            ),
            cx.subscribe(
                &prototype_panel,
                |story, panel, action: &PrototypePanelAction, cx| {
                    story.handle_prototype_action(panel, action, cx);
                },
            ),
            cx.subscribe(&timeline, |story, panel, action: &TimelineAction, cx| {
                story.handle_timeline_action(panel, action, cx);
            }),
            cx.subscribe(
                &pseudo_editor,
                |story, editor, action: &PseudoEditorAction, cx| {
                    story.handle_pseudo_editor_action(editor, action, cx);
                },
            ),
            cx.subscribe(
                &pages_panel,
                |story, panel, action: &PagesPanelAction, cx| {
                    story.handle_pages_action(panel, action, cx);
                },
            ),
            cx.subscribe(
                &layers_panel,
                |story, panel, action: &LayersPanelAction, cx| {
                    story.handle_layers_action(panel, action, cx);
                },
            ),
            cx.subscribe(
                &design_panel,
                |story, panel, action: &DesignPanelAction, cx| {
                    story.handle_design_action(panel, action, cx);
                },
            ),
            cx.subscribe(&toolbar, |story, panel, action: &ToolbarAction, cx| {
                story.handle_toolbar_action(panel, action, cx);
            }),
        ];
        let storybook = cx.weak_entity();
        subscriptions.push(cx.on_window_closed(move |cx| {
            let open_window_ids = cx
                .windows()
                .into_iter()
                .map(|window| window.window_id())
                .collect::<HashSet<_>>();
            if let Some(storybook) = storybook.upgrade() {
                storybook.update(cx, |storybook, cx| {
                    let previous_count = storybook.story_windows.len();
                    storybook
                        .story_windows
                        .retain(|window_id, _| open_window_ids.contains(window_id));
                    if storybook.story_windows.len() != previous_count {
                        cx.notify();
                    }
                });
            }
        }));

        let storybook = Self {
            active_story: launch.story,
            launch_mode: launch.mode,
            story_windows: HashMap::new(),
            gallery_theme_mode: Theme::global(cx).mode,
            gallery_search_input,
            gallery_menu_bar,
            gallery_story_scroll_handle,
            icon_gallery,
            variables_page,
            variables_view_data,
            variables_last_action:
                "Ready — switch collections and groups, edit values, or add a mode".into(),
            assets_panel,
            assets_view_data,
            assets_last_action: "Ready — search, filter, or select a component library".into(),
            prototype_panel,
            prototype_view_data,
            prototype_last_action:
                "Ready — switch surfaces, inspect settings, or dismiss either hint".into(),
            timeline,
            timeline_view_data,
            timeline_last_action:
                "Ready — play, loop, seek, zoom, add a keyframe, or ask the agent".into(),
            pseudo_editor,
            pseudo_left_surface,
            pseudo_right_surface,
            pseudo_variables_visible,
            pseudo_last_action: "Ready — switch panels, open Variables, present, or share".into(),
            pages_panel,
            pages,
            active_page,
            elements: seed_elements(),
            pages_last_action: "Ready — try every Pages control".into(),
            next_page_id: 6,
            layers_panel,
            layers,
            selected_layers,
            expanded_layers,
            layer_selection_anchor: Some("hero-title".into()),
            layers_last_action: "Ready — expand, select, rename, or secondary-click a layer".into(),
            design_panel,
            design_nodes,
            design_color_styles,
            design_color_style_samples,
            design_paint_styles,
            design_paint_variables,
            design_shaders,
            design_typography_styles,
            design_fonts,
            design_effect_styles,
            design_effect_variables,
            design_property_variables,
            design_component_swaps,
            design_property_bindings: HashMap::new(),
            design_layout_grid_styles,
            design_layout_grid_variables,
            design_frame_presets,
            design_page_view_data,
            design_page_local_styles,
            design_variable_mode_views,
            design_viewer_properties,
            design_export_configurations,
            design_export_modes,
            design_animated_exports,
            design_export_previews,
            design_media_paint_views,
            design_paint_edit_snapshots: HashMap::new(),
            design_menu_preview: None,
            design_component_property_name_edits: HashMap::new(),
            design_component_property_reorders: HashMap::new(),
            design_component_variant_option_name_edits: HashMap::new(),
            design_component_variant_option_reorders: HashMap::new(),
            design_media_crop_targets: HashSet::new(),
            design_layout_grid_edit_snapshots: HashMap::new(),
            design_grid_dimensions_edit_snapshots: HashMap::new(),
            design_smart_selection_spacing: HashMap::from([
                (
                    DesignSmartSelectionAxis::Horizontal,
                    DesignSmartSelectionSpacingValue::Uniform(24.),
                ),
                (
                    DesignSmartSelectionAxis::Vertical,
                    DesignSmartSelectionSpacingValue::Mixed,
                ),
            ]),
            design_smart_selection_edit_snapshots: HashMap::new(),
            design_video_scrub_snapshots: HashMap::new(),
            design_text_path_edit_snapshots: HashMap::new(),
            design_node_edit_snapshots: HashMap::new(),
            design_export_edit_snapshots: HashMap::new(),
            design_animated_export_edit_snapshots: HashMap::new(),
            design_page_background_edit_snapshots: HashMap::new(),
            design_selection_color_edit_snapshots: HashMap::new(),
            design_vector_edit_snapshots: HashMap::new(),
            next_design_export_id: 100,
            next_design_media_source_id: 100,
            next_design_resource_id: 100,
            next_design_auto_layout_id: 100,
            selected_design_node,
            design_inspection_scenario,
            design_text_range_revision: 0,
            design_panel_width,
            design_panel_resize_drag: None,
            design_additional_labels,
            design_nudge_settings,
            design_workspace_mode,
            design_fixture_scroll_handle: ScrollHandle::new(),
            design_last_action: "Ready — select a node preset and exercise every inspector row"
                .into(),
            toolbar,
            toolbar_mode,
            toolbar_tool,
            toolbar_zoom,
            toolbar_draw_options,
            toolbar_dev_options,
            toolbar_motion_options,
            toolbar_last_action:
                "Ready — open every split tool, Actions, Agent, zoom, and all four modes".into(),
            _subscriptions: subscriptions,
        };
        let panel = storybook.design_panel.clone();
        storybook.apply_design_inspection_context(panel, cx);
        match storybook.active_story {
            StoryKind::Icons => storybook.icon_gallery.focus_handle(cx).focus(window),
            StoryKind::Toolbar => storybook.toolbar.focus_handle(cx).focus(window),
            StoryKind::Pages => storybook.pages_panel.focus_handle(cx).focus(window),
            StoryKind::Layers => storybook.layers_panel.focus_handle(cx).focus(window),
            StoryKind::Design => storybook.design_panel.focus_handle(cx).focus(window),
            StoryKind::Variables => storybook.variables_page.focus_handle(cx).focus(window),
            StoryKind::Assets => storybook.assets_panel.focus_handle(cx).focus(window),
            StoryKind::Prototype => storybook.prototype_panel.focus_handle(cx).focus(window),
            StoryKind::Timeline => storybook.timeline.focus_handle(cx).focus(window),
            StoryKind::PseudoEditor => storybook.pseudo_editor.focus_handle(cx).focus(window),
        }
        storybook
    }

    fn seed_design_nodes() -> Vec<DesignPanelNode> {
        let mut nodes = DesignPanelNodeKind::ALL
            .into_iter()
            .enumerate()
            .map(|(index, kind)| {
                let id = match kind {
                    DesignPanelNodeKind::Widget => "reference-widget".to_owned(),
                    // Widget is inserted immediately before Other in `ALL`;
                    // retain Other's original selector identity.
                    DesignPanelNodeKind::Other => "design-node-18".to_owned(),
                    _ => format!("design-node-{index}"),
                };
                DesignPanelNode::new(id, format!("{} example", kind.label()), kind)
            })
            .collect::<Vec<_>>();

        // Keep the canonical matrix keyed by `ALL`, then append explicit
        // migration fixtures for compatibility-only taxonomy aliases that do
        // not already have a richer reference fixture below. Multiple
        // selection remains an inspection scenario, never a fake node.
        nodes.extend([
            DesignPanelNode::new(
                "compatibility-mask",
                "Compatibility alias · Mask",
                DesignPanelNodeKind::Mask,
            ),
            DesignPanelNode::new(
                "compatibility-table",
                "Compatibility alias · Table (FigJam)",
                DesignPanelNodeKind::Table,
            ),
            DesignPanelNode::new(
                "compatibility-pen",
                "Compatibility alias · Pen path",
                DesignPanelNodeKind::Pen,
            ),
            DesignPanelNode::new(
                "compatibility-pencil",
                "Compatibility alias · Pencil path",
                DesignPanelNodeKind::Pencil,
            ),
        ]);

        if let Some(frame) = nodes.first_mut() {
            frame.name = "Desktop frame · Freeform".into();
            frame.fills = vec![
                DesignPaint::solid(DesignColor::rgb(0x0d, 0x99, 0xff))
                    .with_id("desktop-frame-style-fill"),
            ];
            frame.fill_style_binding = Some(DesignPaintStyleBinding::new(
                DesignPaintStyleSelection::page("paint-style-brand-surface"),
                "Brand / Surface",
            ));
            frame.effects.push(
                DesignEffect::drop_shadow(
                    DesignColor::rgba(0x00, 0x00, 0x00, 0x33),
                    24.,
                    0.,
                    0.,
                    8.,
                )
                .with_id("desktop-frame-shadow"),
            );
            let mut stroke = DesignStroke::for_node(
                DesignPanelNodeKind::Frame,
                DesignPaint::solid(DesignColor::BLACK),
                1.,
                DesignStrokeAlign::Inside,
            );
            stroke.add_paint(DesignPaint::solid(DesignColor::PURPLE));
            stroke.weights.mode = DesignStrokeWeightMode::Custom;
            stroke.weights.top = 1.;
            stroke.weights.right = 2.;
            stroke.weights.bottom = 4.;
            stroke.weights.left = 2.;
            stroke.join = DesignStrokeJoin::Miter;
            stroke.miter_angle = 72.;
            frame.stroke = Some(stroke);
        }

        if let Some(group) = nodes
            .iter_mut()
            .find(|node| node.kind == DesignPanelNodeKind::Group)
        {
            group.id = "add-auto-layout-group".into();
            group.name = "Navigation cluster · Group".into();
        }

        if let Some(other) = nodes
            .iter_mut()
            .find(|node| node.kind == DesignPanelNodeKind::Other)
        {
            other.name = "Plugin node · Host capabilities".into();
            other.capabilities = Some(
                DesignPanelNodeCapabilities::for_node_kind(DesignPanelNodeKind::Other)
                    .with_sections([
                        DesignPanelSection::Position,
                        DesignPanelSection::Layout,
                        DesignPanelSection::Layer,
                        DesignPanelSection::Stroke,
                        DesignPanelSection::LayoutGrid,
                        DesignPanelSection::Export,
                    ])
                    .with_fill(false)
                    .with_effects(false)
                    .with_constraints(true)
                    .with_layout_guides(true),
            );
            other.fills.clear();
            other.effects.clear();
        }

        if let Some(widget) = nodes
            .iter_mut()
            .find(|node| node.kind == DesignPanelNodeKind::Widget)
        {
            widget.name = "Widget · Opaque object".into();
            widget.width = 320.;
            widget.height = 180.;
            widget.capabilities = Some(DesignPanelNodeCapabilities::for_node_kind(
                DesignPanelNodeKind::Widget,
            ));
        }

        if let Some(text) = nodes
            .iter_mut()
            .find(|node| node.kind == DesignPanelNodeKind::Text)
            && let Some(typography) = text.typography.as_mut()
        {
            text.name = "Variable text · Truncated".into();
            typography.weight = 520.;
            typography.style_binding = Some(DesignTypographyStyleBinding::new(
                DesignTypographyStyleSelection::page("page-body-default"),
                "Body / Default",
            ));
            typography.resize = DesignTextResize::AutoHeight;
            typography.truncate = true;
            typography.max_lines = Some(3);
            typography.list = DesignTextList::Bulleted;
            typography.list_spacing = 8.;
            typography.decoration = DesignTextDecoration::Underline;
            typography.decoration_details = Some(DesignTextDecorationDetails {
                style: DesignTextDecorationStyle::Wavy,
                offset: DesignTextDecorationMetric::Pixels(2.),
                thickness: DesignTextDecorationMetric::Percent(110.),
                color: DesignTextDecorationColor::Solid(DesignColor::BLUE),
                skip_ink: true,
            });
            typography.open_type_features = vec![
                DesignOpenTypeFeature::new(
                    DesignOpenTypeFeatureTag::registered("liga")
                        .expect("liga is an exact registered tag"),
                    "Standard ligatures",
                    true,
                    true,
                )
                .with_preview("fi fl ffi"),
                DesignOpenTypeFeature::new(
                    DesignOpenTypeFeatureTag::registered("zero")
                        .expect("zero is an exact registered tag"),
                    "Slashed zero",
                    false,
                    true,
                )
                .with_preview("0 O"),
                DesignOpenTypeFeature {
                    tag: DesignOpenTypeFeatureTag::opaque("FUTURE:round-dots"),
                    name: "Future round dots".into(),
                    default_enabled: false,
                    enabled: false,
                    availability: DesignOpenTypeFeatureAvailability::Unavailable {
                        reason: "The active style does not provide this feature".into(),
                    },
                    preview: Some("i j :".into()),
                },
            ];
            typography.variable_axes = vec![
                DesignFontAxis::new("wght", "Weight", 520., 100., 900., 400.),
                DesignFontAxis::new("wdth", "Width", 96.5, 75., 125., 100.).with_step(0.1),
                DesignFontAxis::new("opsz", "Optical size", 18., 9., 144., 14.).with_step(0.1),
                DesignFontAxis::new("slnt", "Slant", -6., -12., 0., 0.).with_step(0.1),
                DesignFontAxis::new("GRAD", "Grade", 25., -100., 150., 0.),
                DesignFontAxis::new("ital", "Italic", 0., 0., 1., 0.)
                    .read_only_with_reason("The active face controls italic as a named style"),
                DesignFontAxis::new("XTRA", "Counter width", 468., 323., 603., 468.).with_binding(
                    DesignFontAxisBinding::new("variable-counter-width", "Counter width")
                        .with_collection("Type scale"),
                ),
            ];
        }
        if let Some(text_path) = nodes
            .iter_mut()
            .find(|node| node.kind == DesignPanelNodeKind::TextPath)
            && let Some(typography) = text_path.typography.as_mut()
        {
            text_path.name = "Text path · Percent units".into();
            typography.line_height = DesignLineHeight::Percent(125.);
            typography.letter_spacing = DesignLetterSpacing::Percent(2.);
            if let Some(view_data) = text_path.text_path.as_mut() {
                view_data.orientation = DesignTextPathOrientation::Default;
                view_data.can_flip_orientation = true;
                view_data.show_start_data_debug_controls = false;
            }
            text_path.text_path_start_data = DesignTextPathStartData::new(2, 0.35);
            text_path.vector_edit = Some(DesignVectorEditViewData::new([
                DesignVectorVertexViewData::new("text-path-start", 8., 28.)
                    .with_topology(DesignVectorVertexTopology::Endpoint)
                    .with_corner_radius(Some(0.))
                    .with_handle_mirroring(Some(DesignHandleMirroring::Angle))
                    .selected(true),
                DesignVectorVertexViewData::new("text-path-curve", 72., 44.)
                    .with_corner_radius(Some(10.))
                    .with_handle_mirroring(Some(DesignHandleMirroring::AngleAndLength))
                    .selected(true),
                DesignVectorVertexViewData::new("text-path-end", 144., 20.)
                    .with_topology(DesignVectorVertexTopology::Endpoint)
                    .with_corner_radius(Some(0.))
                    .with_handle_mirroring(Some(DesignHandleMirroring::None)),
            ]));
        }
        let mut text_path_debug = DesignPanelNode::new(
            "text-path-api-debug",
            "Text path · API start-data debug",
            DesignPanelNodeKind::TextPath,
        );
        text_path_debug.text_path = Some(
            fanta_gpui::prelude::DesignTextPathViewData::new(DesignTextPathOrientation::Flipped)
                .flippable(false)
                .with_start_data_debug_controls(true),
        );
        text_path_debug.text_path_start_data = DesignTextPathStartData::new(4, 0.625);
        nodes.push(text_path_debug);

        if let Some(vector) = nodes
            .iter_mut()
            .find(|node| node.kind == DesignPanelNodeKind::Vector)
        {
            vector.vector_edit = Some(DesignVectorEditViewData::new([
                DesignVectorVertexViewData::new("network-left", 16., 24.)
                    .with_topology(DesignVectorVertexTopology::Endpoint)
                    .with_corner_radius(Some(4.))
                    .with_handle_mirroring(Some(DesignHandleMirroring::None))
                    .selected(true),
                DesignVectorVertexViewData::new("network-curve", 64., 24.)
                    .with_corner_radius(Some(12.))
                    .with_handle_mirroring(Some(DesignHandleMirroring::Angle))
                    .selected(true),
                DesignVectorVertexViewData::new("network-branch", 96., 72.)
                    .with_topology(DesignVectorVertexTopology::Branch)
                    .with_corner_radius(None)
                    .with_handle_mirroring(None),
                DesignVectorVertexViewData::new("network-end", 156., 56.)
                    .with_topology(DesignVectorVertexTopology::Endpoint)
                    .with_corner_radius(Some(0.))
                    .with_handle_mirroring(Some(DesignHandleMirroring::AngleAndLength)),
            ]));
        }

        if let Some(vector) = nodes
            .iter_mut()
            .find(|node| node.kind == DesignPanelNodeKind::Vector)
            && let Some(stroke) = vector.stroke.as_mut()
        {
            vector.name = "Vector network · Branching".into();
            stroke.edit_context = DesignStrokeEditContext::branching(4);
            stroke.endpoint_cap = DesignStrokeCap::Diamond;
            stroke.add_paint(DesignPaint::solid(DesignColor::BLUE));
        }

        if let Some(pen) = nodes
            .iter_mut()
            .find(|node| node.kind == DesignPanelNodeKind::Pen)
            && let Some(stroke) = pen.stroke.as_mut()
        {
            pen.name = "Pen path · Stretch brush".into();
            stroke.complex_stroke = DesignComplexStroke::StretchBrush(DesignStretchBrushStroke {
                brush: DesignStretchBrushName::Hardboiled,
                direction: DesignStrokeBrushDirection::Backward,
            });
        }

        if let Some(pencil) = nodes
            .iter_mut()
            .find(|node| node.kind == DesignPanelNodeKind::Pencil)
            && let Some(stroke) = pencil.stroke.as_mut()
        {
            pencil.name = "Pencil path · Dynamic".into();
            stroke.complex_stroke = DesignComplexStroke::Dynamic(DesignDynamicStroke {
                frequency: 6.4,
                wiggle: 0.38,
                smoothen: 0.72,
            });
        }

        let horizontal = DesignPanelNode::new(
            "layout-horizontal",
            "Card row · Horizontal",
            DesignPanelNodeKind::Frame,
        )
        .with_layout_mode(DesignLayoutMode::Horizontal);
        let mut vertical = DesignPanelNode::new(
            "layout-vertical",
            "Settings · Vertical",
            DesignPanelNodeKind::Frame,
        )
        .with_layout_mode(DesignLayoutMode::Vertical);
        if let Some(layout) = vertical.layout.as_mut() {
            layout.padding = [12., 24., 16., 20.];
        }
        let mut wrapped = DesignPanelNode::new(
            "layout-wrap",
            "Chips · Horizontal wrap",
            DesignPanelNodeKind::Frame,
        )
        .with_layout_mode(DesignLayoutMode::Horizontal);
        if let Some(layout) = wrapped.layout.as_mut() {
            layout.wrap = true;
            layout.gap = 6.;
            layout.counter_axis_gap = Some(6.);
            layout.padding = [8., 12., 8., 12.];
        }
        let mut wrapped_linked = DesignPanelNode::new(
            "layout-wrap-linked",
            "Tags · Wrap with linked row spacing",
            DesignPanelNodeKind::Frame,
        )
        .with_layout_mode(DesignLayoutMode::Horizontal);
        if let Some(layout) = wrapped_linked.layout.as_mut() {
            layout.set_wrap(true);
            layout.gap = 10.;
            layout.counter_axis_gap = None;
        }
        let mut wrapped_space_between = DesignPanelNode::new(
            "layout-wrap-space-between",
            "Cards · Wrap rows space between",
            DesignPanelNodeKind::Frame,
        )
        .with_layout_mode(DesignLayoutMode::Horizontal);
        if let Some(layout) = wrapped_space_between.layout.as_mut() {
            layout.set_wrap(true);
            let _ =
                layout.set_counter_axis_align_content(DesignCounterAxisAlignContent::SpaceBetween);
        }
        let mut grid =
            DesignPanelNode::new("layout-grid", "Gallery · Grid", DesignPanelNodeKind::Frame)
                .with_layout_mode(DesignLayoutMode::Grid);
        if let Some(layout) = grid.layout.as_mut() {
            layout.horizontal_sizing = DesignSizingMode::Fixed;
            layout.grid_auto_tracks = DesignGridAutoTracks::None;
            layout.grid_items_positioning = DesignGridItemsPositioning::Manual;
            layout.gap = 16.;
            layout.counter_axis_gap = Some(24.);
            layout.grid_columns = vec![
                DesignGridTrack::fraction(1.),
                DesignGridTrack::fraction(1.),
                DesignGridTrack::fixed(160.),
            ];
            layout.grid_rows = vec![DesignGridTrack::hug(), DesignGridTrack::fixed(120.)];
        }
        let mut auto_spacing = DesignPanelNode::new(
            "layout-auto-spacing",
            "Toolbar · Auto spacing and first-on-top",
            DesignPanelNodeKind::Frame,
        )
        .with_layout_mode(DesignLayoutMode::Horizontal);
        if let Some(layout) = auto_spacing.layout.as_mut() {
            layout.horizontal_sizing = DesignSizingMode::Fixed;
            layout.vertical_sizing = DesignSizingMode::Hug;
            layout.item_spacing_mode = DesignItemSpacingMode::Auto;
            layout.gap = 0.;
            layout.padding = [12., 20., 12., 20.];
            layout.clip_content = false;
            layout.include_strokes = true;
            layout.stacking_order = DesignStackingOrder::FirstOnTop;
            layout.baseline_alignment = DesignBaselineAlignment::Baseline;
        }
        let mut responsive_limits = DesignPanelNode::new(
            "layout-responsive-limits",
            "Card · Auto layout min/max disclosures",
            DesignPanelNodeKind::Frame,
        )
        .with_layout_mode(DesignLayoutMode::Vertical);
        if let Some(layout) = responsive_limits.layout.as_mut() {
            layout.horizontal_sizing = DesignSizingMode::Hug;
            layout.vertical_sizing = DesignSizingMode::Hug;
            layout.gap = 12.;
            layout.padding = [20., 24., 20., 24.];
            layout.item.min_width = Some(240.);
            layout.item.max_width = Some(960.);
            layout.item.min_height = Some(120.);
            layout.item.max_height = Some(720.);
        }
        let mut vertical_auto_spacing = DesignPanelNode::new(
            "layout-vertical-auto-spacing",
            "List · Vertical auto spacing",
            DesignPanelNodeKind::Frame,
        )
        .with_layout_mode(DesignLayoutMode::Vertical);
        if let Some(layout) = vertical_auto_spacing.layout.as_mut() {
            layout.horizontal_sizing = DesignSizingMode::Hug;
            layout.vertical_sizing = DesignSizingMode::Fixed;
            layout.alignment_x = 1;
            layout.item_spacing_mode = DesignItemSpacingMode::Auto;
            layout.gap = 0.;
            layout.padding = [8., 16., 24., 32.];
        }
        let mut grid_auto_flow = DesignPanelNode::new(
            "layout-grid-auto-flow",
            "Gallery · Grid auto flow",
            DesignPanelNodeKind::Frame,
        )
        .with_layout_mode(DesignLayoutMode::Grid);
        if let Some(layout) = grid_auto_flow.layout.as_mut() {
            layout.horizontal_sizing = DesignSizingMode::Fixed;
            layout.vertical_sizing = DesignSizingMode::Fixed;
            layout.grid_columns = vec![
                DesignGridTrack::fraction(2.),
                DesignGridTrack::hug(),
                DesignGridTrack::fixed(96.),
            ];
            layout.grid_rows = vec![DesignGridTrack::hug(), DesignGridTrack::fraction(1.)];
            layout.grid_auto_tracks = DesignGridAutoTracks::Rows;
            layout.grid_items_positioning = DesignGridItemsPositioning::RowAutoFlow;
            layout.gap = 12.;
            layout.counter_axis_gap = Some(20.);
        }
        let mut horizontal_item = DesignPanelNode::new(
            "layout-child-horizontal",
            "Card · Horizontal auto-layout child",
            DesignPanelNodeKind::Rectangle,
        );
        if let Some(layout) = horizontal_item.layout.as_mut() {
            layout.horizontal_sizing = DesignSizingMode::Fill;
            layout.vertical_sizing = DesignSizingMode::Fixed;
            layout.item.positioning = DesignLayoutPositioning::InFlow;
            layout.item.layout_grow = 1.;
            layout.item.min_width = Some(160.);
            layout.item.max_width = Some(480.);
            layout.item.max_height = Some(96.);
        }
        let mut absolute_item = DesignPanelNode::new(
            "layout-child-absolute",
            "Card · Absolute stretched child",
            DesignPanelNodeKind::Rectangle,
        );
        if let Some(layout) = absolute_item.layout.as_mut() {
            layout.horizontal_sizing = DesignSizingMode::Fill;
            layout.vertical_sizing = DesignSizingMode::Hug;
            layout.item.positioning = DesignLayoutPositioning::Absolute;
            layout.item.align_self = DesignLayoutAlignSelf::Stretch;
            layout.item.layout_grow = 1.;
            layout.item.min_width = Some(120.);
            layout.item.max_width = Some(640.);
            layout.item.min_height = Some(44.);
            layout.item.max_height = Some(240.);
        }
        let mut vertical_item = DesignPanelNode::new(
            "layout-child-vertical",
            "Card · Vertical auto-layout child",
            DesignPanelNodeKind::Rectangle,
        );
        if let Some(layout) = vertical_item.layout.as_mut() {
            layout.horizontal_sizing = DesignSizingMode::Fill;
            layout.vertical_sizing = DesignSizingMode::Fixed;
            layout.item.positioning = DesignLayoutPositioning::InFlow;
            layout.item.align_self = DesignLayoutAlignSelf::Stretch;
        }
        let mut grid_item = DesignPanelNode::new(
            "layout-grid-child-placement",
            "Card · Grid child placement",
            DesignPanelNodeKind::Rectangle,
        );
        if let Some(layout) = grid_item.layout.as_mut() {
            layout.horizontal_sizing = DesignSizingMode::Fill;
            layout.vertical_sizing = DesignSizingMode::Fill;
            layout.item.grid_row_index = 2;
            layout.item.grid_column_index = 1;
            layout.item.grid_row_span = 2;
            layout.item.grid_column_span = 3;
            layout.item.grid_horizontal_alignment = DesignGridItemAlignment::End;
            layout.item.grid_vertical_alignment = DesignGridItemAlignment::Center;
            layout.item.min_width = Some(96.);
            layout.item.max_height = Some(320.);
        }
        nodes.splice(
            1..1,
            [
                horizontal,
                vertical,
                wrapped,
                wrapped_linked,
                wrapped_space_between,
                grid,
                auto_spacing,
                responsive_limits,
                vertical_auto_spacing,
                grid_auto_flow,
                horizontal_item,
                absolute_item,
                vertical_item,
                grid_item,
            ],
        );

        let gradient_kinds = [
            DesignPaintKind::LinearGradient,
            DesignPaintKind::RadialGradient,
            DesignPaintKind::AngularGradient,
            DesignPaintKind::DiamondGradient,
        ];
        for (index, kind) in gradient_kinds.into_iter().enumerate() {
            let mut gradient = DesignPanelNode::new(
                format!("gradient-{index}"),
                format!("{} gradient", kind.label()),
                DesignPanelNodeKind::Rectangle,
            );
            gradient.fills = vec![DesignPaint::gradient(
                kind,
                vec![
                    DesignGradientStop::new(0., DesignColor::PURPLE),
                    DesignGradientStop::new(0.5, DesignColor::BLUE),
                    DesignGradientStop::new(1., DesignColor::rgb(0x14, 0xae, 0x5c)),
                ],
            )];
            nodes.push(gradient);
        }

        if let Some(slice) = nodes
            .iter_mut()
            .find(|node| node.kind == DesignPanelNodeKind::Slice)
        {
            slice.export_settings = vec![
                DesignExportSetting {
                    scale: 1.,
                    suffix: "".into(),
                    format: DesignExportFormat::Png,
                },
                DesignExportSetting {
                    scale: 1.,
                    suffix: "-vector".into(),
                    format: DesignExportFormat::Svg,
                },
            ];
        }
        if let Some(component) = nodes
            .iter_mut()
            .find(|node| node.kind == DesignPanelNodeKind::Component)
        {
            component.component_properties.insert(
                0,
                DesignComponentProperty::variant(
                    "state",
                    "State",
                    "Default",
                    "Default",
                    vec!["Default".into(), "Hover".into(), "Pressed".into()],
                ),
            );
            component.layout = Some({
                let mut layout = component.layout.clone().unwrap_or_default();
                layout.mode = DesignLayoutMode::Horizontal;
                layout.gap = 8.;
                layout.padding = [10., 16., 10., 16.];
                layout
            });
            if let Some(property) = component
                .component_properties
                .iter_mut()
                .find(|property| property.id == "show-icon")
            {
                property.default_value_binding = Some(DesignComponentPropertyVariableBinding::new(
                    "layer-visible",
                    "Layer visible",
                    DesignVariableResolvedValue::Boolean(true),
                ));
            }
            if let Some(property) = component
                .component_properties
                .iter_mut()
                .find(|property| property.id == "label")
            {
                property.description = Some("Action copy shown inside the button.".into());
                property.documentation_links = vec![DesignDocumentationLink::new(
                    "Button content guidelines",
                    "https://example.com/components/button/content",
                )];
            }

            let choices = component
                .component_properties
                .iter()
                .map(|property| {
                    DesignComponentPropertyChoice::new(
                        property.id.clone(),
                        property.name.clone(),
                        property.definition.kind(),
                    )
                })
                .collect::<Vec<_>>();
            let choice = |property_id: &str| {
                choices
                    .iter()
                    .find(|choice| choice.property_id.as_ref() == property_id)
                    .expect("storybook component property choice")
                    .clone()
            };
            let definitions = component
                .component_properties
                .iter()
                .map(|property| {
                    let mut definition =
                        DesignComponentPropertyDefinitionAuthoring::editable(property.id.clone());
                    if property.definition.kind() == DesignComponentPropertyKind::Variant {
                        definition =
                            definition.with_variant_options(
                                property.preferred_values.iter().enumerate().map(
                                    |(index, name)| {
                                        DesignComponentVariantOptionAuthoring::editable(
                                            format!("{}-option-{index}", property.id),
                                            name.clone(),
                                        )
                                    },
                                ),
                            );
                    }
                    definition
                })
                .collect::<Vec<_>>();
            let applied_properties = vec![
                DesignAppliedComponentPropertyControl::new(
                    "appearance-visible-property",
                    "button-icon-layer",
                    "Leading icon",
                    DesignComponentPropertyApplicationSurface::Appearance,
                    [choice("show-icon"), choice("state")],
                )
                .applied_to("show-icon"),
                DesignAppliedComponentPropertyControl::new(
                    "text-content-property",
                    "button-label-layer",
                    "Label",
                    DesignComponentPropertyApplicationSurface::Text,
                    [choice("label"), choice("state")],
                )
                .applied_to("label"),
                DesignAppliedComponentPropertyControl::new(
                    "nested-swap-property",
                    "button-icon-instance",
                    "Icon instance",
                    DesignComponentPropertyApplicationSurface::NestedInstance,
                    [choice("leading-icon"), choice("content")],
                )
                .applied_to("leading-icon"),
                DesignAppliedComponentPropertyControl::new(
                    "nested-variant-property",
                    "button-state-layer",
                    "State layer",
                    DesignComponentPropertyApplicationSurface::NestedInstance,
                    [choice("state"), choice("show-icon")],
                )
                .applied_to("state"),
                DesignAppliedComponentPropertyControl::new(
                    "nested-slot-property",
                    "button-content-layer",
                    "Content layer",
                    DesignComponentPropertyApplicationSurface::NestedInstance,
                    [choice("content"), choice("leading-icon")],
                )
                .applied_to("content"),
            ];
            let exposure_candidates = [
                (
                    "nested-expose-state",
                    "nested-button",
                    "Nested button",
                    "state",
                ),
                (
                    "nested-expose-label",
                    "nested-button",
                    "Nested button",
                    "label",
                ),
                (
                    "nested-expose-visible",
                    "nested-icon",
                    "Nested icon",
                    "show-icon",
                ),
                (
                    "nested-expose-swap",
                    "nested-icon",
                    "Nested icon",
                    "leading-icon",
                ),
                (
                    "nested-expose-slot",
                    "nested-content",
                    "Nested content",
                    "content",
                ),
            ]
            .into_iter()
            .map(|(candidate_id, instance_id, instance_name, property_id)| {
                DesignNestedComponentPropertyExposureCandidate::new(
                    candidate_id,
                    instance_id,
                    instance_name,
                    choice(property_id),
                )
            })
            .collect::<Vec<_>>();
            if let Some(context) = component.component_context.as_mut() {
                context.description = Some(
                    "Primary action component with label, icon, and content-slot APIs.".into(),
                );
                context.documentation_links = vec![
                    DesignDocumentationLink::new(
                        "Button usage",
                        "https://example.com/components/button",
                    ),
                    DesignDocumentationLink::new(
                        "Button accessibility",
                        "https://example.com/components/button/accessibility",
                    ),
                ];
                context.authoring = Some(DesignComponentAuthoringViewData {
                    create_kinds: vec![
                        DesignComponentPropertyKind::Variant,
                        DesignComponentPropertyKind::Boolean,
                        DesignComponentPropertyKind::Text,
                        DesignComponentPropertyKind::InstanceSwap,
                        DesignComponentPropertyKind::Slot,
                    ],
                    definitions,
                    applied_properties,
                    exposure_candidates,
                });
            }
        }

        if let Some(component_set) = nodes
            .iter_mut()
            .find(|node| node.kind == DesignPanelNodeKind::ComponentSet)
        {
            let definitions = component_set
                .component_properties
                .iter()
                .map(|property| {
                    DesignComponentPropertyDefinitionAuthoring::editable(property.id.clone())
                        .with_variant_options(property.preferred_values.iter().enumerate().map(
                            |(index, name)| {
                                DesignComponentVariantOptionAuthoring::editable(
                                    format!("{}-option-{index}", property.id),
                                    name.clone(),
                                )
                            },
                        ))
                })
                .collect();
            if let Some(context) = component_set.component_context.as_mut() {
                context.description =
                    Some("Button family with ordered State and Size variant axes.".into());
                context.documentation_links = vec![
                    DesignDocumentationLink::new(
                        "Button variants",
                        "https://example.com/components/button/variants",
                    ),
                    DesignDocumentationLink::new(
                        "Variant naming",
                        "https://example.com/components/button/variant-naming",
                    ),
                ];
                context.authoring = Some(DesignComponentAuthoringViewData {
                    create_kinds: vec![
                        DesignComponentPropertyKind::Variant,
                        DesignComponentPropertyKind::Boolean,
                        DesignComponentPropertyKind::Text,
                        DesignComponentPropertyKind::InstanceSwap,
                        DesignComponentPropertyKind::Slot,
                    ],
                    definitions,
                    applied_properties: Vec::new(),
                    exposure_candidates: Vec::new(),
                });
            }
        }

        if let Some(component_source) = nodes
            .iter()
            .find(|node| node.kind == DesignPanelNodeKind::Component)
            .cloned()
        {
            let mut selected_text = DesignPanelNode::new(
                "component-authoring-selected-text",
                "Component authoring · Selected text layer",
                DesignPanelNodeKind::Text,
            );
            selected_text.component_properties = component_source.component_properties;
            selected_text.component_context = component_source.component_context;
            if let Some(context) = selected_text.component_context.as_mut() {
                context.description =
                    Some("Selected text sublayer inside the authored main component.".into());
                if let Some(authoring) = context.authoring.as_mut() {
                    authoring.applied_properties.retain(|control| {
                        control.surface == DesignComponentPropertyApplicationSurface::Text
                    });
                    authoring.exposure_candidates.clear();
                }
            }
            nodes.push(selected_text);
        }

        if let Some(instance) = nodes
            .iter_mut()
            .find(|node| node.kind == DesignPanelNodeKind::Instance)
        {
            if let Some(context) = instance.component_context.as_mut() {
                context.description = Some(
                    "Button instance with local property and nested override examples.".into(),
                );
                context.documentation_links = vec![
                    DesignDocumentationLink::new(
                        "Button instance guidance",
                        "https://example.com/components/button/instances",
                    ),
                    DesignDocumentationLink::new(
                        "Override behavior",
                        "https://example.com/components/button/overrides",
                    ),
                ];
                if let Some(main) = context.main_component.as_mut() {
                    main.description =
                        Some("Published primary action from Product foundations.".into());
                    main.documentation_links = vec![DesignDocumentationLink::new(
                        "Primary button API",
                        "https://example.com/libraries/product-foundations/primary-button",
                    )];
                }
            }
            if let Some(property) = instance
                .component_properties
                .iter_mut()
                .find(|property| property.id == "label")
            {
                *property = property.clone().with_multiline(true);
            }
            if let Some(property) = instance
                .component_properties
                .iter_mut()
                .find(|property| property.id == "show-icon")
            {
                property.resolved_value_binding =
                    Some(DesignComponentPropertyVariableBinding::new(
                        "layer-visible",
                        "Layer visible",
                        DesignVariableResolvedValue::Boolean(true),
                    ));
            }

            let badge_main =
                DesignComponentReference::local("nested-status-badge-main", "Status badge");
            instance.component_properties.extend([
                DesignComponentProperty::text(
                    "nested-badge-label",
                    "Badge label",
                    "New",
                    "Ready\nfor review",
                )
                .with_multiline(true)
                .with_reset_state(DesignComponentResetState::Resettable)
                .from_nested_instance(
                    "nested-status-badge",
                    "Status badge",
                    Some(badge_main.clone()),
                ),
                DesignComponentProperty::boolean("nested-badge-visible", "Show badge", true, true)
                    .from_nested_instance("nested-status-badge", "Status badge", Some(badge_main)),
            ]);
        }

        if let Some(slot) = nodes
            .iter_mut()
            .find(|node| node.kind == DesignPanelNodeKind::Slot)
            && let Some(context) = slot.component_context.as_mut()
        {
            context.description =
                Some("Slot definition for optional card content with preferred insertions.".into());
            context.documentation_links = vec![
                DesignDocumentationLink::new(
                    "Slot authoring",
                    "https://example.com/components/slots/authoring",
                ),
                DesignDocumentationLink::new(
                    "Preferred slot content",
                    "https://example.com/components/slots/preferred-content",
                ),
            ];
        }

        let mut instance_child = DesignPanelNode::new(
            "component-instance-child",
            "Nested layer · Aspect ratio inherited from main",
            DesignPanelNodeKind::Rectangle,
        );
        instance_child.is_component_instance_child = true;
        instance_child.width = 240.;
        instance_child.height = 135.;
        nodes.push(instance_child);

        let mut reference_rectangle = DesignPanelNode::new(
            "reference-rectangle",
            "Rectangle 2",
            DesignPanelNodeKind::Rectangle,
        );
        reference_rectangle.x = 0.;
        reference_rectangle.y = 437.;
        reference_rectangle.width = 393.;
        reference_rectangle.height = 215.;
        reference_rectangle.corner_radii = [0.; 4];
        reference_rectangle.fills = vec![
            DesignPaint::solid(DesignColor::rgb(0xd9, 0xd9, 0xd9))
                .with_id("reference-rectangle-fill"),
        ];
        nodes.push(reference_rectangle);

        let mut reference_frame =
            DesignPanelNode::new("reference-frame", "Frame 2", DesignPanelNodeKind::Frame);
        reference_frame.x = 1_709.;
        reference_frame.y = 424.;
        reference_frame.width = 548.;
        reference_frame.height = 250.;
        reference_frame.corner_radii = [0.; 4];
        reference_frame.fills.clear();
        reference_frame.stroke = None;
        reference_frame.effects.clear();
        reference_frame.layout_grids.clear();
        if let Some(layout) = reference_frame.layout.as_mut() {
            layout.gap = 0.;
            layout.counter_axis_gap = None;
            layout.clip_content = false;
        }
        nodes.push(reference_frame);

        let mut reference_text =
            DesignPanelNode::new("reference-text", "hello", DesignPanelNodeKind::Text);
        reference_text.x = 1_256.;
        reference_text.y = 1_415.;
        reference_text.width = 27.;
        reference_text.height = 15.;
        reference_text.fills =
            vec![DesignPaint::solid(DesignColor::WHITE).with_id("reference-text-fill")];
        if let Some(typography) = reference_text.typography.as_mut() {
            typography.family = "Inter".into();
            typography.style = "Regular".into();
            typography.weight = 400.;
            typography.size = 12.;
            typography.resize = DesignTextResize::AutoWidth;
            typography.letter_spacing = DesignLetterSpacing::Percent(0.);
        }
        nodes.push(reference_text);

        let mut reference_ellipse = DesignPanelNode::new(
            "reference-ellipse",
            "Ellipse 1",
            DesignPanelNodeKind::Ellipse,
        );
        reference_ellipse.x = 4_880.;
        reference_ellipse.y = -337.;
        reference_ellipse.width = 100.;
        reference_ellipse.height = 100.;
        reference_ellipse.corner_radii = [0.; 4];
        reference_ellipse.fills = vec![
            DesignPaint::solid(DesignColor::rgb(0xd9, 0xd9, 0xd9))
                .with_id("reference-ellipse-fill"),
        ];
        reference_ellipse.shape_geometry = DesignShapeGeometry::Ellipse(DesignArcData::FULL_CIRCLE);
        nodes.push(reference_ellipse);

        let mut reference_image = DesignPanelNode::new(
            "reference-image",
            "Screenshot 2026-07-25",
            DesignPanelNodeKind::Image,
        );
        reference_image.x = 1_337.;
        reference_image.y = 817.;
        reference_image.width = 207.;
        reference_image.height = 335.;
        reference_image.corner_radii = [0.; 4];
        nodes.push(reference_image);

        let mut reference_arrow =
            DesignPanelNode::new("reference-arrow", "Arrow 6", DesignPanelNodeKind::Arrow);
        reference_arrow.x = 1_481.;
        reference_arrow.y = 897.;
        reference_arrow.width = 205.43;
        reference_arrow.height = 0.;
        reference_arrow.rotation = 14.37;
        if let Some(stroke) = reference_arrow.stroke.as_mut() {
            stroke.paints =
                vec![DesignPaint::solid(DesignColor::WHITE).with_id("reference-arrow-stroke")];
            stroke.align = DesignStrokeAlign::Inside;
            stroke.weights.set_uniform(2.);
            stroke.start_cap = DesignStrokeCap::None;
            stroke.end_cap = DesignStrokeCap::LineArrow;
        }
        nodes.push(reference_arrow);

        nodes.push(DesignPanelNode::new(
            "reference-video",
            "Product preview",
            DesignPanelNodeKind::Video,
        ));

        for alignment_y in 0_u8..3 {
            for alignment_x in 0_u8..3 {
                let mut alignment = DesignPanelNode::new(
                    format!("layout-alignment-{alignment_x}-{alignment_y}"),
                    format!(
                        "Auto layout · Alignment {}",
                        usize::from(alignment_y) * 3 + usize::from(alignment_x) + 1
                    ),
                    DesignPanelNodeKind::Frame,
                )
                .with_layout_mode(DesignLayoutMode::Horizontal);
                if let Some(layout) = alignment.layout.as_mut() {
                    layout.alignment_x = alignment_x;
                    layout.alignment_y = alignment_y;
                    layout.gap = 12.;
                    layout.padding = [16.; 4];
                }
                nodes.push(alignment);
            }
        }

        let mut partial_arc = DesignPanelNode::new(
            "ellipse-partial-arc",
            "Ellipse · Partial arc",
            DesignPanelNodeKind::Ellipse,
        );
        partial_arc.shape_geometry = DesignShapeGeometry::Ellipse(DesignArcData::new(
            30_f32.to_radians(),
            280_f32.to_radians(),
            0.,
        ));
        nodes.push(partial_arc);

        let mut ring = DesignPanelNode::new(
            "ellipse-ring",
            "Ellipse · Ring",
            DesignPanelNodeKind::Ellipse,
        );
        ring.shape_geometry =
            DesignShapeGeometry::Ellipse(DesignArcData::new(0., std::f32::consts::TAU, 0.62));
        nodes.push(ring);

        for operation in DesignBooleanOperation::ALL {
            if operation == DesignBooleanOperation::Union {
                continue;
            }
            let mut boolean = DesignPanelNode::new(
                format!("boolean-{:?}", operation).to_ascii_lowercase(),
                format!("Boolean · {}", operation.label()),
                DesignPanelNodeKind::BooleanOperation,
            );
            boolean.shape_geometry = DesignShapeGeometry::Boolean(operation);
            nodes.push(boolean);
        }

        for mask_mode in DesignMaskType::ALL {
            let mut mask = DesignPanelNode::new(
                format!("mask-{:?}", mask_mode).to_ascii_lowercase(),
                format!("Vector · {} mask", mask_mode.label()),
                DesignPanelNodeKind::Vector,
            );
            mask.is_mask = true;
            mask.mask_mode = mask_mode;
            mask.mask_type = Some(mask_mode.label().into());
            nodes.push(mask);
        }

        if let Some(section) = nodes
            .iter_mut()
            .find(|node| node.kind == DesignPanelNodeKind::Section)
            .and_then(|node| node.section.as_mut())
        {
            section.dev_status = Some(
                DesignSectionDevStatus::new(DesignSectionDevStatusKind::ReadyForDev)
                    .with_description("Implementation notes are available in Dev Mode.")
                    .changed(true),
            );
        }
        let mut hidden_section = DesignPanelNode::new(
            "section-hidden",
            "Section · Hidden contents",
            DesignPanelNodeKind::Section,
        );
        if let Some(section) = hidden_section.section.as_mut() {
            section.contents_hidden = true;
        }
        nodes.push(hidden_section);
        let mut completed_section = DesignPanelNode::new(
            "section-completed",
            "Section · Completed",
            DesignPanelNodeKind::Section,
        );
        if let Some(section) = completed_section.section.as_mut() {
            section.dev_status = Some(
                DesignSectionDevStatus::new(DesignSectionDevStatusKind::Completed)
                    .with_description("Reviewed and implemented."),
            );
        }
        nodes.push(completed_section);

        let mut repeat_vertical = DesignPanelNode::new(
            "transform-repeat-vertical",
            "Transform group · Vertical pixels",
            DesignPanelNodeKind::TransformGroup,
        );
        let mut vertical_modifier =
            DesignRepeatModifier::linear("repeat-vertical", DesignRepeatAxis::Vertical);
        vertical_modifier.unit = DesignTransformUnit::Pixels;
        vertical_modifier.count = 6;
        vertical_modifier.offset = 24.;
        repeat_vertical.transform_modifiers = vec![vertical_modifier];
        nodes.push(repeat_vertical);

        let mut repeat_radial = DesignPanelNode::new(
            "transform-repeat-radial",
            "Transform group · Radial",
            DesignPanelNodeKind::TransformGroup,
        );
        let mut radial_modifier = DesignRepeatModifier::radial("repeat-radial");
        radial_modifier.unit = DesignTransformUnit::Relative;
        radial_modifier.count = 12;
        radial_modifier.offset = 18.;
        repeat_radial.transform_modifiers = vec![radial_modifier];
        nodes.push(repeat_radial);

        let mut repeat_stack = DesignPanelNode::new(
            "transform-repeat-stack",
            "Transform group · Modifier stack",
            DesignPanelNodeKind::TransformGroup,
        );
        let mut stack_radial = DesignRepeatModifier::radial("repeat-stack-radial");
        stack_radial.count = 8;
        repeat_stack.transform_modifiers = vec![
            DesignRepeatModifier::linear("repeat-stack-horizontal", DesignRepeatAxis::Horizontal),
            DesignRepeatModifier::linear("repeat-stack-vertical", DesignRepeatAxis::Vertical),
            stack_radial,
        ];
        nodes.push(repeat_stack);

        let mut repeat_empty = DesignPanelNode::new(
            "transform-repeat-empty",
            "Transform group · No modifiers",
            DesignPanelNodeKind::TransformGroup,
        );
        repeat_empty.transform_modifiers.clear();
        nodes.push(repeat_empty);

        let mut repeat_horizontal_pixels = DesignPanelNode::new(
            "transform-repeat-horizontal-pixels",
            "Transform group · Horizontal pixels",
            DesignPanelNodeKind::TransformGroup,
        );
        let mut horizontal_pixels =
            DesignRepeatModifier::linear("repeat-horizontal-pixels", DesignRepeatAxis::Horizontal);
        horizontal_pixels.unit = DesignTransformUnit::Pixels;
        horizontal_pixels.count = 9;
        horizontal_pixels.offset = 36.;
        repeat_horizontal_pixels.transform_modifiers = vec![horizontal_pixels];
        nodes.push(repeat_horizontal_pixels);

        let mut paint_plain = DesignPanelNode::new(
            "paint-solid",
            "Paint · Solid",
            DesignPanelNodeKind::Rectangle,
        );
        paint_plain.fills = vec![
            DesignPaint::solid(DesignColor::rgb(0x7c, 0x3a, 0xed)).with_id("plain-solid-fill"),
        ];
        nodes.push(paint_plain);

        let mut contrast_checker = DesignPanelNode::new(
            "paint-contrast",
            "Paint · Contrast checker",
            DesignPanelNodeKind::Rectangle,
        );
        contrast_checker.fills = vec![
            DesignPaint::solid(DesignColor::rgb(0xa0, 0xae, 0xc0)).with_id("contrast-checker-fill"),
        ];
        nodes.push(contrast_checker);

        let mut paint_solid = DesignPanelNode::new(
            "paint-solid-bound",
            "Paint · Bound and read-only solids",
            DesignPanelNodeKind::Rectangle,
        );
        let mut bound_solid =
            DesignPaint::solid(DesignColor::rgb(0x0d, 0x99, 0xff)).with_id("bound-solid");
        if let DesignPaintPayload::Solid(solid) = &mut bound_solid.payload {
            solid.binding = Some(
                DesignPaintBinding::new("brand-primary", "Brand / Primary")
                    .with_collection("Theme"),
            );
        }
        bound_solid.sync_legacy_projection();
        paint_solid.fills = vec![
            bound_solid,
            DesignPaint::solid(DesignColor::rgb(0xff, 0xc7, 0x00))
                .with_id("readonly-solid")
                .with_read_only(true),
        ];
        nodes.push(paint_solid);

        for (index, tile_type) in DesignPatternTileType::ALL.into_iter().enumerate() {
            let mut paint_pattern = DesignPanelNode::new(
                format!("paint-pattern-{index}"),
                format!("Paint · Pattern {}", tile_type.label()),
                DesignPanelNodeKind::Rectangle,
            );
            let mut paint = DesignPaint::pattern(format!("pattern-source-node-{}", index + 1))
                .with_id(format!("pattern-{index}-fill"));
            if let DesignPaintPayload::Pattern(pattern) = &mut paint.payload {
                pattern.tile_type = tile_type;
                pattern.scaling_factor = 0.5 + index as f32 * 0.25;
                pattern.spacing = DesignPatternSpacing::new(
                    0.08 + index as f32 * 0.04,
                    0.12 + index as f32 * 0.04,
                );
                pattern.horizontal_alignment = [
                    DesignPatternHorizontalAlignment::Start,
                    DesignPatternHorizontalAlignment::Center,
                    DesignPatternHorizontalAlignment::End,
                ][index];
            }
            paint.sync_legacy_projection();
            paint_pattern.fills = vec![paint];
            nodes.push(paint_pattern);
        }

        for (index, scale_mode) in DesignMediaPaintScaleMode::ALL.into_iter().enumerate() {
            let source_access = [
                "All source actions",
                "Upload + Edit",
                "Properties only",
                "Make only",
            ][index];
            let mut paint_image = DesignPanelNode::new(
                format!("paint-image-{index}"),
                format!("Paint · Image {} · {source_access}", scale_mode.label()),
                DesignPanelNodeKind::Rectangle,
            );
            let source = DesignPaintSource {
                id: format!("image-source-{index}").into(),
                name: format!("Hero photograph {}", index + 1).into(),
                mime_type: Some("image/jpeg".into()),
                reference: Some(format!("asset://image/{index}").into()),
            };
            let mut paint = DesignPaint::image(source).with_id(format!("image-{index}-fill"));
            if let DesignPaintPayload::Image(image) = &mut paint.payload {
                let rotation = DesignMediaQuarterTurn::ALL[index];
                image.placement = match scale_mode {
                    DesignMediaPaintScaleMode::Fill => DesignMediaPaintPlacement::Fill { rotation },
                    DesignMediaPaintScaleMode::Fit => DesignMediaPaintPlacement::Fit { rotation },
                    DesignMediaPaintScaleMode::Crop => DesignMediaPaintPlacement::Crop {
                        transform: DesignPaintTransform::IDENTITY
                            .rotated(17.)
                            .flipped_vertical(),
                    },
                    DesignMediaPaintScaleMode::Tile => DesignMediaPaintPlacement::Tile {
                        scaling_factor: 0.75 + index as f32 * 0.25,
                        rotation,
                    },
                };
                image.filters = DesignImageFilters {
                    exposure: -0.12 + index as f32 * 0.08,
                    contrast: 0.18,
                    saturation: 0.24 - index as f32 * 0.04,
                    temperature: 0.08,
                    tint: -0.06,
                    highlights: -0.20,
                    shadows: 0.16,
                };
            }
            paint.sync_legacy_projection();
            paint_image.fills = vec![paint];
            nodes.push(paint_image);
        }

        for (index, scale_mode) in DesignMediaPaintScaleMode::ALL.into_iter().enumerate() {
            let mut paint_video = DesignPanelNode::new(
                format!("paint-video-{index}"),
                format!("Paint · Video {}", scale_mode.label()),
                DesignPanelNodeKind::Rectangle,
            );
            let source = DesignPaintSource {
                id: format!("video-source-{index}").into(),
                name: format!("Product demo {}", index + 1).into(),
                mime_type: Some("video/mp4".into()),
                reference: Some(format!("asset://video/{index}").into()),
            };
            let mut paint = DesignPaint::video(source).with_id(format!("video-{index}-fill"));
            if let DesignPaintPayload::Video(video) = &mut paint.payload {
                let rotation = DesignMediaQuarterTurn::ALL[index];
                video.placement = match scale_mode {
                    DesignMediaPaintScaleMode::Fill => DesignMediaPaintPlacement::Fill { rotation },
                    DesignMediaPaintScaleMode::Fit => DesignMediaPaintPlacement::Fit { rotation },
                    DesignMediaPaintScaleMode::Crop => DesignMediaPaintPlacement::Crop {
                        transform: DesignPaintTransform::IDENTITY.rotated(11.),
                    },
                    DesignMediaPaintScaleMode::Tile => DesignMediaPaintPlacement::Tile {
                        scaling_factor: 1. + index as f32 * 0.25,
                        rotation,
                    },
                };
                video.filters = DesignImageFilters {
                    exposure: 0.04,
                    contrast: 0.12,
                    saturation: 0.10,
                    temperature: -0.08,
                    tint: 0.05,
                    highlights: -0.16,
                    shadows: 0.22,
                };
            }
            paint.sync_legacy_projection();
            paint_video.fills = vec![paint];
            nodes.push(paint_video);
        }

        let mut paint_opaque = DesignPanelNode::new(
            "paint-host-opaque",
            "Paint · Shader and unsupported",
            DesignPanelNodeKind::Rectangle,
        );
        paint_opaque.fills = vec![
            DesignPaint::from_payload(DesignPaintPayload::Shader(
                DesignShaderPaint::from_definition(&story_fractal_shader_definition())
                    .expect("the Storybook Fractal noise shader is imported"),
            ))
            .with_id("shader-fill"),
            DesignPaint::unsupported("PLUGIN_PAINT", "{\"opaque\":true}")
                .with_id("unsupported-fill")
                .with_read_only(true),
        ];
        nodes.push(paint_opaque);

        let mut effects_lab = DesignPanelNode::new(
            "effects-all",
            "Effects · Every native type",
            DesignPanelNodeKind::Rectangle,
        );
        effects_lab.effects = DesignEffectKind::ALL
            .into_iter()
            .enumerate()
            .map(|(index, kind)| {
                DesignEffect::new(kind).with_id(format!("effects-all-{index}-{}", kind.label()))
            })
            .collect();
        effects_lab
            .effect_capabilities
            .show_shadow_behind_transparent_areas = true;
        effects_lab.effect_capabilities.set_kind_availability(
            DesignEffectKindAvailability::available(DesignEffectKind::Shader),
        );
        if let Some(drop_shadow) = effects_lab.effects.first_mut() {
            drop_shadow.set_variable_binding(DesignEffectVariableBinding::new(
                DesignEffectVariableField::Color,
                "effect-shadow-color",
                "Shadow / Default",
                "Semantic colors",
            ));
        }
        if let Some(shader) = effects_lab
            .effects
            .iter_mut()
            .find(|effect| effect.kind == DesignEffectKind::Shader)
        {
            shader.set_settings(DesignEffectSettings::Shader(DesignShaderEffect::new(
                "shader:effect:refraction",
                "Refraction study",
                [
                    DesignShaderProperty::new(
                        "enabled",
                        "Enabled",
                        DesignShaderPropertyKind::Boolean,
                        DesignShaderPropertyValue::Boolean(true),
                    ),
                    DesignShaderProperty::new(
                        "label",
                        "Label",
                        DesignShaderPropertyKind::Text,
                        DesignShaderPropertyValue::Text("Glass surface".into()),
                    ),
                    DesignShaderProperty::new(
                        "scale",
                        "Scale",
                        DesignShaderPropertyKind::Number,
                        DesignShaderPropertyValue::Number(4.),
                    ),
                    DesignShaderProperty::new(
                        "image",
                        "Environment image",
                        DesignShaderPropertyKind::Image,
                        DesignShaderPropertyValue::AssetId("image:environment".into()),
                    ),
                    DesignShaderProperty::new(
                        "instance",
                        "Surface component",
                        DesignShaderPropertyKind::InstanceSwap,
                        DesignShaderPropertyValue::AssetId("component:surface".into()),
                    ),
                    DesignShaderProperty::new(
                        "slot",
                        "Content slot",
                        DesignShaderPropertyKind::Slot,
                        DesignShaderPropertyValue::AssetId("slot:content".into()),
                    ),
                    DesignShaderProperty::new(
                        "tint",
                        "Tint",
                        DesignShaderPropertyKind::Color,
                        DesignShaderPropertyValue::Color(DesignColor::PURPLE),
                    ),
                    DesignShaderProperty::new(
                        "center",
                        "Center",
                        DesignShaderPropertyKind::Point,
                        DesignShaderPropertyValue::Point(
                            fanta_gpui::design::DesignEffectVector::new(0.5, 0.5),
                        ),
                    ),
                    DesignShaderProperty::new(
                        "ray",
                        "Refraction line",
                        DesignShaderPropertyKind::Line,
                        DesignShaderPropertyValue::Line {
                            start: fanta_gpui::design::DesignEffectVector::new(0.1, 0.2),
                            end: fanta_gpui::design::DesignEffectVector::new(0.9, 0.8),
                        },
                    ),
                    DesignShaderProperty::new(
                        "lens",
                        "Lens",
                        DesignShaderPropertyKind::Circle,
                        DesignShaderPropertyValue::Circle {
                            center: fanta_gpui::design::DesignEffectVector::new(0.5, 0.5),
                            radius: 0.35,
                        },
                    ),
                    DesignShaderProperty::new(
                        "orbit",
                        "Highlight orbit",
                        DesignShaderPropertyKind::CirclePoint,
                        DesignShaderPropertyValue::CirclePoint {
                            center: fanta_gpui::design::DesignEffectVector::new(0.5, 0.5),
                            radius: 0.3,
                            angle: 45.,
                        },
                    ),
                    DesignShaderProperty::new(
                        "color-point",
                        "Chromatic focus",
                        DesignShaderPropertyKind::ColorPoint,
                        DesignShaderPropertyValue::ColorPoint {
                            point: fanta_gpui::design::DesignEffectVector::new(0.35, 0.6),
                            color: DesignColor::BLUE,
                            variable_id: Some("variable:brand:accent".into()),
                        },
                    ),
                    DesignShaderProperty::new(
                        "gradient",
                        "Dispersion gradient",
                        DesignShaderPropertyKind::Gradient,
                        DesignShaderPropertyValue::Gradient(vec![
                            fanta_gpui::design::DesignShaderGradientStop::new(
                                0.,
                                DesignColor::PURPLE,
                            ),
                            {
                                let mut stop = fanta_gpui::design::DesignShaderGradientStop::new(
                                    0.5,
                                    DesignColor::BLUE,
                                );
                                stop.variable_id = Some("variable:brand:accent".into());
                                stop
                            },
                            fanta_gpui::design::DesignShaderGradientStop::new(
                                1.,
                                DesignColor::WHITE,
                            ),
                        ]),
                    ),
                    DesignShaderProperty::new(
                        "alias",
                        "Bound amount",
                        DesignShaderPropertyKind::Number,
                        DesignShaderPropertyValue::VariableAlias {
                            variable_id: "variable:shader:amount".into(),
                        },
                    ),
                    DesignShaderProperty::new(
                        "future",
                        "Future payload",
                        DesignShaderPropertyKind::Unsupported,
                        DesignShaderPropertyValue::Opaque {
                            type_name: "MESH_PATCH".into(),
                            payload: "{\"patches\":4,\"mode\":\"future\"}".into(),
                        },
                    ),
                ],
            )));
        }
        effects_lab.effects.push(
            DesignEffect::from_settings(
                true,
                DesignEffectSettings::Opaque(fanta_gpui::design::DesignOpaqueEffect::new(
                    "FUTURE_EFFECT",
                    "Future effect",
                    "{\"version\":2}",
                )),
            )
            .with_id("effects-all-future"),
        );
        nodes.push(effects_lab);

        let mut effects_style_bound = DesignPanelNode::new(
            "effects-style-bound",
            "Effects · Bound style",
            DesignPanelNodeKind::Rectangle,
        );
        effects_style_bound.effects = vec![
            DesignEffect::new(DesignEffectKind::BackgroundBlur).with_id("style-blur"),
            DesignEffect::new(DesignEffectKind::DropShadow).with_id("style-shadow"),
        ];
        effects_style_bound.effect_style_binding = Some(DesignEffectStyleBinding::new(
            DesignEffectStyleSelection::library("fanta-effects", "effect-library-overlay"),
            "Overlay / Raised",
        ));
        nodes.push(effects_style_bound);

        let mut arrow_line = DesignPanelNode::new(
            "stroke-line-arrow",
            "Line · Arrow cap",
            DesignPanelNodeKind::Line,
        );
        if let Some(stroke) = arrow_line.stroke.as_mut() {
            stroke.end_cap = DesignStrokeCap::LineArrow;
        }
        nodes.push(arrow_line);

        let mut dashed_line = DesignPanelNode::new(
            "stroke-dashed-arrow",
            "Line · Dashed triangle arrow",
            DesignPanelNodeKind::Line,
        );
        if let Some(stroke) = dashed_line.stroke.as_mut() {
            stroke.set_dash_mode(DesignStrokeDashMode::Custom);
            stroke.set_dash_pattern(vec![12., 6., 2., 6.]);
            stroke.dash_cap = DesignStrokeCap::Round;
            stroke.start_cap = DesignStrokeCap::Circle;
            stroke.end_cap = DesignStrokeCap::TriangleArrow;
        }
        nodes.push(dashed_line);

        let mut simple_dashed_line = DesignPanelNode::new(
            "stroke-dashed-simple",
            "Line · Dashed",
            DesignPanelNodeKind::Line,
        );
        if let Some(stroke) = simple_dashed_line.stroke.as_mut() {
            stroke.set_dash_mode(DesignStrokeDashMode::Dashed);
            stroke.set_dash_pattern(vec![8., 4.]);
        }
        nodes.push(simple_dashed_line);

        let mut brush_vector = DesignPanelNode::new(
            "stroke-brush",
            "Vector · Brush stroke",
            DesignPanelNodeKind::Vector,
        );
        if let Some(stroke) = brush_vector.stroke.as_mut() {
            stroke.complex_stroke = DesignComplexStroke::StretchBrush(DesignStretchBrushStroke {
                brush: DesignStretchBrushName::Noir,
                direction: DesignStrokeBrushDirection::Backward,
            });
            stroke.variable_width = Some(DesignVariableWidthStroke::Preset(
                DesignVariableWidthPreset::Taper,
            ));
        }
        nodes.push(brush_vector);

        let mut scatter_vector = DesignPanelNode::new(
            "stroke-scatter-brush",
            "Vector · Scatter brush",
            DesignPanelNodeKind::Vector,
        );
        if let Some(stroke) = scatter_vector.stroke.as_mut() {
            stroke.complex_stroke = DesignComplexStroke::ScatterBrush(DesignScatterBrushStroke {
                brush: DesignScatterBrushName::Vaporwave,
                gap: 0.75,
                wiggle: 1.25,
                size_jitter: 1.5,
                angular_jitter: -30.,
                rotation: 45.,
            });
        }
        nodes.push(scatter_vector);

        let mut dynamic_vector = DesignPanelNode::new(
            "stroke-dynamic",
            "Vector · Dynamic stroke",
            DesignPanelNodeKind::Vector,
        );
        if let Some(stroke) = dynamic_vector.stroke.as_mut() {
            stroke.complex_stroke = DesignComplexStroke::Dynamic(DesignDynamicStroke {
                frequency: 12.,
                wiggle: 3.8,
                smoothen: 0.72,
            });
        }
        nodes.push(dynamic_vector);

        for preset in DesignVariableWidthPreset::ALL {
            let mut variable_vector = DesignPanelNode::new(
                format!("stroke-variable-{:?}", preset).to_ascii_lowercase(),
                format!("Vector · Variable width · {}", preset.label()),
                DesignPanelNodeKind::Vector,
            );
            if let Some(stroke) = variable_vector.stroke.as_mut() {
                stroke.variable_width = Some(DesignVariableWidthStroke::Preset(preset));
            }
            nodes.push(variable_vector);
        }

        let mut custom_width_vector = DesignPanelNode::new(
            "stroke-variable-custom",
            "Vector · Variable width · Ordered custom points",
            DesignPanelNodeKind::Vector,
        );
        if let Some(stroke) = custom_width_vector.stroke.as_mut() {
            stroke.variable_width = Some(DesignVariableWidthStroke::custom([
                DesignVariableWidthPoint::new(0., 0.2),
                DesignVariableWidthPoint::new(0.35, 1.4),
                DesignVariableWidthPoint::new(1., 0.5),
            ]));
        }
        nodes.push(custom_width_vector);

        let mut opaque_stroke = DesignPanelNode::new(
            "stroke-opaque-custom",
            "Vector · Read-only custom brush",
            DesignPanelNodeKind::Vector,
        );
        if let Some(stroke) = opaque_stroke.stroke.as_mut() {
            stroke.complex_stroke = DesignComplexStroke::Opaque(DesignOpaqueComplexStroke::new(
                "CUSTOM",
                "Imported custom brush",
                "{\"brushName\":\"CUSTOM\",\"source\":\"host\"}",
            ));
        }
        nodes.push(opaque_stroke);

        let mut corner_lab = DesignPanelNode::new(
            "corners-independent",
            "Rectangle · Independent smooth corners",
            DesignPanelNodeKind::Rectangle,
        );
        corner_lab.independent_corners = true;
        corner_lab.corner_radii = [4., 12., 24., 32.];
        corner_lab.corner_smoothing = 0.6;
        nodes.push(corner_lab);

        for resize in DesignTextResize::ALL {
            let mut text = DesignPanelNode::new(
                format!("text-resize-{:?}", resize).to_ascii_lowercase(),
                format!("Text · {}", resize.label()),
                DesignPanelNodeKind::Text,
            );
            if let Some(typography) = text.typography.as_mut() {
                typography.resize = resize;
                typography.truncate = true;
                typography.max_lines = (resize == DesignTextResize::AutoWidth).then_some(2);
            }
            nodes.push(text);
        }

        let mut vertical_hug_text = DesignPanelNode::new(
            "text-max-lines-vertical-hug",
            "Text · Auto layout · Vertical Hug · 4 lines",
            DesignPanelNodeKind::Text,
        );
        if let Some(typography) = vertical_hug_text.typography.as_mut() {
            typography.resize = DesignTextResize::AutoHeight;
            typography.truncate = true;
            typography.max_lines = Some(4);
        }
        vertical_hug_text
            .layout
            .as_mut()
            .expect("Text fixtures support dimensions")
            .vertical_sizing = DesignSizingMode::Hug;
        nodes.push(vertical_hug_text);

        let mut vertical_fixed_text = DesignPanelNode::new(
            "text-max-lines-vertical-fixed",
            "Text · Auto layout · Vertical Fixed · Max lines unavailable",
            DesignPanelNodeKind::Text,
        );
        if let Some(typography) = vertical_fixed_text.typography.as_mut() {
            typography.resize = DesignTextResize::AutoHeight;
            typography.truncate = true;
            typography.max_lines = None;
        }
        vertical_fixed_text
            .layout
            .as_mut()
            .expect("Text fixtures support dimensions")
            .vertical_sizing = DesignSizingMode::Fixed;
        nodes.push(vertical_fixed_text);

        let mut max_height_text = DesignPanelNode::new(
            "text-max-lines-max-height",
            "Text · Max height 96 · Max lines Auto",
            DesignPanelNodeKind::Text,
        );
        if let Some(typography) = max_height_text.typography.as_mut() {
            typography.resize = DesignTextResize::AutoWidth;
            typography.truncate = true;
            typography.max_lines = None;
        }
        max_height_text
            .layout
            .as_mut()
            .expect("Text fixtures support dimensions")
            .item
            .max_height = Some(96.);
        nodes.push(max_height_text);

        let guide_color = DesignColor::rgb(0xf2, 0x48, 0x22);
        let mut uniform_guides = DesignPanelNode::new(
            "guides-uniform",
            "Layout guides · Uniform",
            DesignPanelNodeKind::Frame,
        );
        uniform_guides.layout_grids = vec![
            DesignLayoutGrid::uniform(8., guide_color)
                .with_opacity(18.)
                .with_id("guides-uniform-grid"),
        ];
        uniform_guides.layout_grid_style_binding = Some(DesignLayoutGridStyleBinding::new(
            "grid-style-8pt",
            "Foundations / 8 pt grid",
        ));
        nodes.push(uniform_guides);
        let mut variable_uniform_guides = DesignPanelNode::new(
            "guides-uniform-variable",
            "Layout guides · Uniform variable",
            DesignPanelNodeKind::Frame,
        );
        variable_uniform_guides.layout_grids = vec![
            DesignLayoutGrid::uniform(8., guide_color)
                .with_opacity(18.)
                .with_id("guides-uniform-variable-grid")
                .with_variable_binding(
                    DesignLayoutGridVariableField::SectionSize,
                    DesignLayoutGridVariableBinding::new("spacing-8", "Spacing / 8")
                        .with_collection("Spacing"),
                ),
        ];
        nodes.push(variable_uniform_guides);

        for (index, alignment) in DesignColumnGridAlignment::ALL.into_iter().enumerate() {
            let settings = DesignColumnLayoutGrid {
                alignment,
                count: if index == 0 {
                    DesignLayoutGridCount::Auto
                } else {
                    DesignLayoutGridCount::number((index as u16 + 2) * 3)
                },
                size: 72. + index as f32 * 8.,
                offset: 16. + index as f32 * 4.,
                gutter: 20. + index as f32 * 2.,
                margin: 24. + index as f32 * 4.,
                ..DesignColumnLayoutGrid::default()
            };
            let mut guide = DesignPanelNode::new(
                format!("guides-columns-{:?}", alignment).to_ascii_lowercase(),
                format!("Layout guides · Columns {}", alignment.label()),
                DesignPanelNodeKind::Frame,
            );
            let mut grid = DesignLayoutGrid::columns(settings, guide_color)
                .with_opacity(12.)
                .with_id(format!("guides-columns-{index}"));
            if index == 0 {
                grid = grid
                    .with_variable_binding(
                        DesignLayoutGridVariableField::Count,
                        DesignLayoutGridVariableBinding::new("auto-track-count", "Auto tracks")
                            .with_collection("Responsive"),
                    )
                    .with_variable_binding(
                        DesignLayoutGridVariableField::SectionSize,
                        DesignLayoutGridVariableBinding::new(
                            "grid-section-64",
                            "Grid section / 64",
                        )
                        .with_collection("Spacing"),
                    )
                    .with_variable_binding(
                        DesignLayoutGridVariableField::Offset,
                        DesignLayoutGridVariableBinding::new("spacing-8", "Spacing / 8")
                            .with_collection("Spacing"),
                    )
                    .with_variable_binding(
                        DesignLayoutGridVariableField::GutterSize,
                        DesignLayoutGridVariableBinding::new("grid-gutter-24", "Grid gutter / 24")
                            .with_collection("Spacing"),
                    );
            } else if index == 2 {
                grid = grid.with_variable_binding(
                    DesignLayoutGridVariableField::Count,
                    DesignLayoutGridVariableBinding::new("desktop-column-count", "Desktop columns")
                        .with_collection("Responsive"),
                );
            } else if index == 3 {
                grid = grid.with_variable_binding(
                    DesignLayoutGridVariableField::Offset,
                    DesignLayoutGridVariableBinding::new("spacing-8", "Spacing / 8")
                        .with_collection("Spacing"),
                );
            }
            guide.layout_grids = vec![grid];
            nodes.push(guide);
        }

        for (index, alignment) in DesignRowGridAlignment::ALL.into_iter().enumerate() {
            let settings = DesignRowLayoutGrid {
                alignment,
                count: if index == 0 {
                    DesignLayoutGridCount::Auto
                } else {
                    DesignLayoutGridCount::number((index as u16 + 1) * 4)
                },
                size: 48. + index as f32 * 8.,
                offset: 12. + index as f32 * 4.,
                gutter: 12. + index as f32 * 2.,
                margin: 20. + index as f32 * 4.,
                ..DesignRowLayoutGrid::default()
            };
            let mut guide = DesignPanelNode::new(
                format!("guides-rows-{:?}", alignment).to_ascii_lowercase(),
                format!("Layout guides · Rows {}", alignment.label()),
                DesignPanelNodeKind::Frame,
            );
            let mut grid = DesignLayoutGrid::rows(settings, DesignColor::BLUE)
                .with_opacity(12.)
                .with_id(format!("guides-rows-{index}"));
            if index == 0 {
                grid = grid.with_variable_binding(
                    DesignLayoutGridVariableField::Count,
                    DesignLayoutGridVariableBinding::new("auto-track-count", "Auto tracks")
                        .with_collection("Responsive"),
                );
            } else if index == 2 {
                grid = grid.with_variable_binding(
                    DesignLayoutGridVariableField::Count,
                    DesignLayoutGridVariableBinding::new("desktop-row-count", "Desktop rows")
                        .with_collection("Responsive"),
                );
            }
            guide.layout_grids = vec![grid];
            if index == 3 {
                guide.layout_grid_style_binding = Some(DesignLayoutGridStyleBinding::new(
                    "grid-style-rows",
                    "Responsive / Desktop rows",
                ));
            }
            nodes.push(guide);
        }

        let mut hidden_guides = DesignPanelNode::new(
            "guides-hidden",
            "Layout guides · Hidden",
            DesignPanelNodeKind::Frame,
        );
        let mut hidden_grid = DesignLayoutGrid::uniform(4., DesignColor::BLUE)
            .with_opacity(8.)
            .with_id("guides-hidden-grid");
        hidden_grid.visible = false;
        hidden_guides.layout_grids = vec![hidden_grid];
        nodes.push(hidden_guides);

        let mut locked_style_guides = DesignPanelNode::new(
            "guides-style-locked",
            "Layout guides · Non-detachable style",
            DesignPanelNodeKind::Frame,
        );
        locked_style_guides.layout_grids = vec![
            DesignLayoutGrid::columns(DesignColumnLayoutGrid::default(), guide_color)
                .with_opacity(10.)
                .with_id("guides-locked-columns"),
        ];
        locked_style_guides.layout_grid_style_binding = Some(
            DesignLayoutGridStyleBinding::library(
                "fanta-grid-library",
                "grid-style-locked",
                "Library / Locked desktop grid",
            )
            .with_detach_allowed(false),
        );
        nodes.push(locked_style_guides);

        let mut guide_stack = DesignPanelNode::new(
            "guides-stack",
            "Layout guides · Three-guide stack",
            DesignPanelNodeKind::Frame,
        );
        guide_stack.layout_grids = vec![
            DesignLayoutGrid::uniform(8., DesignColor::BLUE)
                .with_opacity(8.)
                .with_id("guides-stack-uniform"),
            DesignLayoutGrid::columns(DesignColumnLayoutGrid::default(), guide_color)
                .with_opacity(12.)
                .with_id("guides-stack-columns"),
            DesignLayoutGrid::rows(DesignRowLayoutGrid::default(), DesignColor::PURPLE)
                .with_opacity(10.)
                .with_id("guides-stack-rows"),
        ];
        nodes.push(guide_stack);

        let missing_variant_main =
            DesignComponentReference::local("component-set-button", "Button")
                .with_availability(DesignComponentAvailability::Missing);
        let mut variant_context = DesignComponentContext::new(DesignComponentRole::VariantChild)
            .with_main_component(missing_variant_main);
        variant_context.description =
            Some("Variant child with an unresolved component-set reference.".into());
        variant_context.overrides = DesignComponentOverrideSummary {
            overridden_property_count: 2,
            nested_override_count: 0,
            reset_state: DesignComponentResetState::Unavailable {
                reason: "The source component set is missing.".into(),
            },
        };
        let mut mixed_variant_property = DesignComponentProperty::variant(
            "variant-child-state",
            "State",
            "Default",
            "Hover",
            vec!["Default".into(), "Hover".into(), "Pressed".into()],
        )
        .with_reset_state(DesignComponentResetState::Resettable);
        mixed_variant_property.override_state = DesignComponentPropertyOverrideState::Mixed;
        let unavailable_property =
            DesignComponentProperty::text("variant-child-label", "Label", "Button", "Continue")
                .with_reset_state(DesignComponentResetState::Unavailable {
                    reason: "The source property cannot be resolved.".into(),
                });
        let mut variant_child = DesignPanelNode::new(
            "component-variant-child-missing",
            "Component · Variant child · Missing main",
            DesignPanelNodeKind::Component,
        )
        .with_component_context(variant_context);
        variant_child.component_properties = vec![mixed_variant_property, unavailable_property];
        nodes.push(variant_child);

        let preferred_card = DesignComponentReference::remote(
            "slot-content-card",
            "Content card",
            "Product foundations",
        );
        let preferred_empty = DesignComponentReference::local("slot-empty-state", "Empty state");
        let nonpreferred_banner =
            DesignComponentReference::local("slot-promo-banner", "Promo banner");
        let available_slot_main = DesignComponentReference::local("slot-shell", "Content slot");
        let unavailable_slot_main = DesignComponentReference::remote(
            "slot-shell-library",
            "Content slot",
            "Unavailable library",
        )
        .with_availability(DesignComponentAvailability::Unavailable {
            reason: "The component library is not loaded.".into(),
        });

        let make_slot_layer =
            |node_id: &'static str,
             name: &'static str,
             main_component: Option<DesignComponentReference>| {
                let mut instance = DesignSlotChild::instance(node_id, name);
                instance.main_component = main_component;
                instance
            };
        let make_slot_story =
            |id: &'static str,
             name: &'static str,
             main_component: DesignComponentReference,
             property: DesignComponentProperty,
             aggregate_reset_state: DesignComponentResetState| {
                let overridden_property_count = u32::from(
                    property.override_state != DesignComponentPropertyOverrideState::Default,
                );
                let mut context = DesignComponentContext::new(DesignComponentRole::SlotInstance)
                    .with_main_component(main_component);
                context.description =
                    Some("Slot-instance contents are supplied by the Storybook host.".into());
                context.overrides = DesignComponentOverrideSummary {
                    overridden_property_count,
                    nested_override_count: 0,
                    reset_state: aggregate_reset_state,
                };
                let mut node = DesignPanelNode::new(id, name, DesignPanelNodeKind::Instance)
                    .with_component_context(context);
                node.component_properties = vec![property];
                node
            };

        let populated_value = DesignSlotValue {
            children: vec![
                make_slot_layer(
                    "slot-populated-card",
                    "Content card",
                    Some(preferred_card.clone()),
                ),
                DesignSlotChild::layer(
                    "slot-populated-copy",
                    "Supporting copy",
                    DesignPanelNodeKind::Text,
                ),
                DesignSlotChild::layer(
                    "slot-populated-image",
                    "Hero image",
                    DesignPanelNodeKind::Rectangle,
                ),
            ],
        };
        let populated_property = DesignComponentProperty::slot(
            "slot-populated-content",
            "Content",
            DesignSlotValue::default(),
            populated_value,
            DesignSlotSettings {
                stretch_child_on_insert: true,
                display_empty: true,
                minimum_children: None,
                maximum_children: Some(3),
                preferred_values_only: false,
                preferred_values: vec![preferred_card.clone(), preferred_empty.clone()],
            },
            DesignSlotState {
                violations: Vec::new(),
                reset_state: DesignComponentResetState::Resettable,
            },
        )
        .with_reset_state(DesignComponentResetState::Resettable);
        nodes.push(make_slot_story(
            "component-slot-instance-populated",
            "Instance · Slot · Populated",
            available_slot_main.clone(),
            populated_property,
            DesignComponentResetState::Resettable,
        ));

        let exact_capacity_value = DesignSlotValue {
            children: vec![
                make_slot_layer(
                    "slot-capacity-card",
                    "Content card",
                    Some(preferred_card.clone()),
                ),
                make_slot_layer(
                    "slot-capacity-empty",
                    "Empty state",
                    Some(preferred_empty.clone()),
                ),
            ],
        };
        let exact_capacity_property = DesignComponentProperty::slot(
            "slot-exact-capacity-content",
            "Content",
            DesignSlotValue::default(),
            exact_capacity_value,
            DesignSlotSettings {
                stretch_child_on_insert: false,
                display_empty: true,
                minimum_children: Some(1),
                maximum_children: Some(2),
                preferred_values_only: true,
                preferred_values: vec![preferred_card.clone(), preferred_empty.clone()],
            },
            DesignSlotState {
                violations: Vec::new(),
                reset_state: DesignComponentResetState::Resettable,
            },
        )
        .with_reset_state(DesignComponentResetState::Resettable);
        nodes.push(make_slot_story(
            "component-slot-instance-exact-capacity",
            "Instance · Slot · Exact capacity",
            unavailable_slot_main,
            exact_capacity_property,
            DesignComponentResetState::Unavailable {
                reason: "Overrides cannot be reset while the library is unavailable.".into(),
            },
        ));

        let below_minimum_property = DesignComponentProperty::slot(
            "slot-below-minimum-content",
            "Required content",
            DesignSlotValue::default(),
            DesignSlotValue::default(),
            DesignSlotSettings {
                stretch_child_on_insert: true,
                display_empty: true,
                minimum_children: Some(2),
                maximum_children: Some(4),
                preferred_values_only: false,
                preferred_values: vec![preferred_card.clone()],
            },
            DesignSlotState {
                violations: vec![DesignSlotViolation::BelowMinimum {
                    minimum: 2,
                    actual: 0,
                }],
                reset_state: DesignComponentResetState::Clean,
            },
        )
        .with_reset_state(DesignComponentResetState::Clean);
        nodes.push(make_slot_story(
            "component-slot-instance-below-minimum",
            "Instance · Slot · Below minimum",
            available_slot_main.clone(),
            below_minimum_property,
            DesignComponentResetState::Clean,
        ));

        let above_maximum_value = DesignSlotValue {
            children: vec![
                make_slot_layer(
                    "slot-above-card-1",
                    "Content card 1",
                    Some(preferred_card.clone()),
                ),
                make_slot_layer(
                    "slot-above-card-2",
                    "Content card 2",
                    Some(preferred_card.clone()),
                ),
                make_slot_layer(
                    "slot-above-card-3",
                    "Content card 3",
                    Some(preferred_card.clone()),
                ),
            ],
        };
        let above_maximum_property = DesignComponentProperty::slot(
            "slot-above-maximum-content",
            "Content",
            DesignSlotValue::default(),
            above_maximum_value,
            DesignSlotSettings {
                stretch_child_on_insert: true,
                display_empty: true,
                minimum_children: None,
                maximum_children: Some(2),
                preferred_values_only: true,
                preferred_values: vec![preferred_card.clone()],
            },
            DesignSlotState {
                violations: vec![DesignSlotViolation::AboveMaximum {
                    maximum: 2,
                    actual: 3,
                }],
                reset_state: DesignComponentResetState::Resettable,
            },
        )
        .with_reset_state(DesignComponentResetState::Resettable);
        nodes.push(make_slot_story(
            "component-slot-instance-above-maximum",
            "Instance · Slot · Above maximum",
            available_slot_main.clone(),
            above_maximum_property,
            DesignComponentResetState::Resettable,
        ));

        let nonpreferred_value = DesignSlotValue {
            children: vec![make_slot_layer(
                "slot-nonpreferred-banner",
                "Promo banner",
                Some(nonpreferred_banner.clone()),
            )],
        };
        let nonpreferred_property = DesignComponentProperty::slot(
            "slot-nonpreferred-content",
            "Content",
            DesignSlotValue::default(),
            nonpreferred_value,
            DesignSlotSettings {
                stretch_child_on_insert: true,
                display_empty: true,
                minimum_children: None,
                maximum_children: Some(2),
                preferred_values_only: true,
                preferred_values: vec![preferred_card.clone(), preferred_empty],
            },
            DesignSlotState {
                violations: vec![DesignSlotViolation::NonPreferredValue {
                    instance_id: "slot-nonpreferred-banner".into(),
                    component_name: nonpreferred_banner.name.clone(),
                }],
                reset_state: DesignComponentResetState::Resettable,
            },
        )
        .with_reset_state(DesignComponentResetState::Resettable);
        nodes.push(make_slot_story(
            "component-slot-instance-nonpreferred",
            "Instance · Slot · Nonpreferred value",
            available_slot_main.clone(),
            nonpreferred_property,
            DesignComponentResetState::Resettable,
        ));

        let missing_inserted_main_value = DesignSlotValue {
            children: vec![make_slot_layer(
                "slot-missing-inserted-main",
                "Detached content",
                None,
            )],
        };
        let missing_inserted_main_property = DesignComponentProperty::slot(
            "slot-missing-main-content",
            "Content",
            DesignSlotValue::default(),
            missing_inserted_main_value,
            DesignSlotSettings {
                stretch_child_on_insert: true,
                display_empty: true,
                minimum_children: None,
                maximum_children: Some(2),
                preferred_values_only: true,
                preferred_values: vec![preferred_card],
            },
            DesignSlotState {
                violations: vec![DesignSlotViolation::MissingMainComponent {
                    instance_id: "slot-missing-inserted-main".into(),
                }],
                reset_state: DesignComponentResetState::Resettable,
            },
        )
        .with_reset_state(DesignComponentResetState::Resettable);
        nodes.push(make_slot_story(
            "component-slot-instance-missing-main",
            "Instance · Slot · Missing inserted main",
            available_slot_main,
            missing_inserted_main_property,
            DesignComponentResetState::Resettable,
        ));

        nodes.extend(story_homogeneous_multiple_nodes());
        Self::assign_story_paint_ids(&mut nodes);
        nodes
    }

    fn assign_story_paint_ids(nodes: &mut [DesignPanelNode]) {
        for node in nodes {
            let node_id = node.id.clone();
            let assign_collection = |paints: &mut [DesignPaint], collection: &str| {
                for (paint_index, paint) in paints.iter_mut().enumerate() {
                    if paint.id.is_empty() {
                        paint.id = format!("{node_id}-{collection}-{paint_index}").into();
                    }
                    if let DesignPaintPayload::Gradient(gradient) = &mut paint.payload {
                        for (stop_index, stop) in gradient.stops.iter_mut().enumerate() {
                            if stop.id.is_empty() {
                                stop.id = format!("{}-stop-{stop_index}", paint.id).into();
                            }
                        }
                        paint.sync_legacy_projection();
                    }
                }
            };

            assign_collection(&mut node.fills, "fill");
            assign_collection(&mut node.selection_colors, "selection-color");
            if let Some(stroke) = node.stroke.as_mut() {
                assign_collection(&mut stroke.paints, "stroke");
            }
        }
    }

    fn seed_design_color_contrast(nodes: &[DesignPanelNode]) -> DesignColorContrastViewData {
        let mut views = Vec::new();
        for node in nodes {
            for (collection, paints) in [
                (DesignPanelCollection::Fill, node.fills.as_slice()),
                (
                    DesignPanelCollection::Stroke,
                    node.stroke
                        .as_ref()
                        .map_or(&[] as &[DesignPaint], |stroke| stroke.paints.as_slice()),
                ),
            ] {
                for (index, paint) in paints.iter().enumerate() {
                    let automatic_category = if matches!(
                        node.kind,
                        DesignPanelNodeKind::Text | DesignPanelNodeKind::TextPath
                    ) {
                        DesignColorContrastCategory::NormalText
                    } else {
                        DesignColorContrastCategory::Graphics
                    };
                    let effective_background = DesignColor::WHITE;
                    let make_leaf = |color_target, color: DesignColor| {
                        let foreground = story_color_with_opacity(color, paint.opacity);
                        let ratio = story_contrast_ratio(foreground, effective_background);
                        let mut corrections = Vec::new();
                        for category in [
                            DesignColorContrastCategory::LargeText,
                            DesignColorContrastCategory::NormalText,
                            DesignColorContrastCategory::Graphics,
                        ] {
                            for level in DesignColorContrastLevel::ALL {
                                let Some(threshold) = story_contrast_threshold(category, level)
                                else {
                                    continue;
                                };
                                let mut black = story_color_with_opacity(
                                    DesignColor::rgb(0, 0, 0),
                                    paint.opacity,
                                );
                                let mut white =
                                    story_color_with_opacity(DesignColor::WHITE, paint.opacity);
                                black.alpha = color.alpha;
                                white.alpha = color.alpha;
                                let black_ratio = story_contrast_ratio(black, effective_background);
                                let white_ratio = story_contrast_ratio(white, effective_background);
                                let (candidate, candidate_ratio) = if black_ratio >= white_ratio {
                                    (black, black_ratio)
                                } else {
                                    (white, white_ratio)
                                };
                                if candidate_ratio >= threshold {
                                    corrections.push(DesignColorContrastCorrection::new(
                                        category, level, candidate,
                                    ));
                                }
                            }
                        }
                        DesignColorContrastLeafViewData::new(
                            color_target,
                            effective_background,
                            ratio,
                            automatic_category,
                        )
                        .with_corrections(corrections)
                    };
                    let leaves = match &paint.payload {
                        DesignPaintPayload::Solid(solid) => {
                            vec![make_leaf(DesignPaintColorTarget::Solid, solid.color)]
                        }
                        DesignPaintPayload::Gradient(gradient) => gradient
                            .stops
                            .iter()
                            .enumerate()
                            .map(|(stop_index, stop)| {
                                make_leaf(
                                    DesignPaintColorTarget::GradientStop {
                                        stop_id: stop.id.clone(),
                                        index: stop_index,
                                    },
                                    stop.color,
                                )
                            })
                            .collect(),
                        DesignPaintPayload::Pattern(_)
                        | DesignPaintPayload::Image(_)
                        | DesignPaintPayload::Video(_)
                        | DesignPaintPayload::Shader(_)
                        | DesignPaintPayload::Unsupported(_) => Vec::new(),
                    };
                    if !leaves.is_empty() {
                        views.push(DesignColorContrastPaintViewData::new(
                            DesignColorContrastPaintTarget::paint(
                                node.id.clone(),
                                collection,
                                paint.id.clone(),
                                index,
                            ),
                            leaves,
                        ));
                    }
                }
            }
        }
        DesignColorContrastViewData::new(views)
    }

    fn seed_design_shaders() -> DesignShaderViewData {
        let fractal = story_fractal_shader_definition();
        let mesh = DesignShaderDefinition::new(
            "shader:page:mesh",
            "Mesh distortion",
            true,
            [
                DesignShaderPropertyDefinition::new(
                    "def:strength",
                    "Strength",
                    DesignShaderPropertyKind::Number,
                )
                .with_default(DesignShaderPropertyValue::Number(0.5)),
                DesignShaderPropertyDefinition::new(
                    "def:source",
                    "Source image",
                    DesignShaderPropertyKind::Image,
                )
                .with_default(DesignShaderPropertyValue::AssetId("image:hero".into())),
            ],
        );
        let library = DesignShaderLibrary::new(
            "shader-library:material",
            "Material shaders",
            [
                DesignShaderDefinition::new(
                    "shader:library:chromatic-glass",
                    "Chromatic glass",
                    true,
                    [
                        DesignShaderPropertyDefinition::new(
                            "def:refraction",
                            "Refraction",
                            DesignShaderPropertyKind::Number,
                        )
                        .with_default(DesignShaderPropertyValue::Number(0.8)),
                        DesignShaderPropertyDefinition::new(
                            "def:enabled",
                            "Enabled",
                            DesignShaderPropertyKind::Boolean,
                        )
                        .with_default(DesignShaderPropertyValue::Boolean(true)),
                    ],
                ),
                DesignShaderDefinition::new(
                    "shader:library:liquid-metal",
                    "Liquid metal",
                    false,
                    [],
                ),
            ],
        );
        DesignShaderViewData::new([fractal, mesh], [library])
    }

    fn seed_design_page_view_data() -> DesignPageViewData {
        let brand_source = DesignLocalResourceSource::library("library-brand", "Brand system");
        let local_styles = DesignLocalResourceGroup::new(
            "page-local-styles",
            "Styles",
            DesignLocalResourceSource::Local,
            [
                DesignLocalResource::local(
                    "style-brand-primary",
                    "Brand / Primary",
                    DesignLocalResourceKind::PaintStyle,
                ),
                DesignLocalResource::local(
                    "style-body",
                    "Typography / Body",
                    DesignLocalResourceKind::TextStyle,
                ),
                DesignLocalResource::local(
                    "style-soft-shadow",
                    "Effects / Soft shadow",
                    DesignLocalResourceKind::EffectStyle,
                ),
                DesignLocalResource::local(
                    "style-grid-desktop",
                    "Layout / Desktop columns",
                    DesignLocalResourceKind::GridStyle,
                ),
            ],
        );
        let local_variables = DesignLocalResourceGroup::new(
            "page-local-variables",
            "Variable collections",
            DesignLocalResourceSource::Local,
            [DesignLocalResource::local(
                "collection-theme",
                "Theme",
                DesignLocalResourceKind::VariableCollection,
            )],
        );
        let library_resources = DesignLocalResourceGroup::new(
            "brand-library-resources",
            "Brand library",
            brand_source,
            [
                DesignLocalResource::available(
                    "style-marketing-accent",
                    "Marketing / Accent",
                    DesignLocalResourceKind::PaintStyle,
                ),
                DesignLocalResource::unavailable(
                    "style-licensed-display",
                    "Typography / Licensed display",
                    DesignLocalResourceKind::TextStyle,
                    "The library font is not available",
                ),
                DesignLocalResource::available(
                    "collection-density",
                    "Density",
                    DesignLocalResourceKind::VariableCollection,
                ),
            ],
        );
        DesignPageViewData::new(
            "storybook-page",
            DesignPageBackground::new(DesignColor::rgb(0xf5, 0xf5, 0xf5)),
            DesignLocalResourceViewData::new([local_styles, local_variables, library_resources]),
        )
    }

    fn seed_design_page_local_styles() -> DesignPageLocalStylesViewData {
        let text = DesignLocalStyleSection::new(
            DesignLocalStyleKind::Text,
            [DesignLocalStyleEntry::folder(
                "folder-typography",
                "Typography",
                [
                    DesignLocalStyleEntry::style(
                        DesignLocalStyleItem::new(
                            "style-text-body",
                            "Body",
                            DesignLocalStylePreview::Text(DesignTypographyStyle::new(
                                "style-text-body",
                                "Body",
                                "Inter",
                                "Regular",
                                16.,
                            )),
                        )
                        .with_description("Inter Regular · 16"),
                    ),
                    DesignLocalStyleEntry::folder(
                        "folder-typography-display",
                        "Display",
                        [DesignLocalStyleEntry::style(
                            DesignLocalStyleItem::new(
                                "style-text-hero",
                                "Hero",
                                DesignLocalStylePreview::Text(DesignTypographyStyle::new(
                                    "style-text-hero",
                                    "Hero",
                                    "Inter",
                                    "Bold",
                                    64.,
                                )),
                            )
                            .with_description("Inter Bold · 64"),
                        )],
                    ),
                ],
            )],
        );
        let color = DesignLocalStyleSection::new(
            DesignLocalStyleKind::Color,
            [
                DesignLocalStyleEntry::style(
                    DesignLocalStyleItem::new(
                        "style-color-brand",
                        "Brand / Primary",
                        DesignLocalStylePreview::Color(vec![
                            DesignPaint::solid(DesignColor::rgb(80, 70, 229))
                                .with_id("style-color-brand-paint"),
                        ]),
                    )
                    .with_description("Primary product color"),
                ),
                DesignLocalStyleEntry::folder(
                    "folder-semantic-colors",
                    "Semantic",
                    [DesignLocalStyleEntry::style(
                        DesignLocalStyleItem::new(
                            "style-color-success",
                            "Success",
                            DesignLocalStylePreview::Color(vec![
                                DesignPaint::solid(DesignColor::rgb(20, 160, 90))
                                    .with_id("style-color-success-paint"),
                            ]),
                        )
                        .with_description("Positive status"),
                    )],
                ),
            ],
        );
        let effect = DesignLocalStyleSection::new(
            DesignLocalStyleKind::Effect,
            [DesignLocalStyleEntry::style(
                DesignLocalStyleItem::new(
                    "style-effect-soft-shadow",
                    "Soft shadow",
                    DesignLocalStylePreview::Effect(vec![
                        DesignEffect::drop_shadow(DesignColor::rgba(0, 0, 0, 64), 16., 0., 0., 8.)
                            .with_id("style-effect-soft-shadow-effect"),
                    ]),
                )
                .with_description("0 8 16 · 25%"),
            )],
        );
        let layout_guide = DesignLocalStyleSection::new(
            DesignLocalStyleKind::LayoutGuide,
            [DesignLocalStyleEntry::style(
                DesignLocalStyleItem::new(
                    "style-layout-desktop",
                    "Desktop columns",
                    DesignLocalStylePreview::LayoutGuide(vec![
                        DesignLayoutGrid::columns(
                            fanta_gpui::prelude::DesignColumnLayoutGrid::default(),
                            DesignColor::rgb(255, 0, 128),
                        )
                        .with_id("style-layout-desktop-grid"),
                    ]),
                )
                .with_description("12 columns"),
            )],
        );
        DesignPageLocalStylesViewData::for_page(
            "storybook-page",
            [text, color, effect, layout_guide],
        )
    }

    fn seed_design_variable_mode_views(
        nodes: &[DesignPanelNode],
    ) -> HashMap<SharedString, DesignVariableModeViewData> {
        fn theme_collection(explicit_mode: Option<&str>) -> DesignVariableModeCollection {
            let collection = DesignVariableModeCollection::new(
                "collection-theme",
                "Theme",
                DesignLocalResourceSource::Local,
                [
                    DesignVariableMode::new("theme-light", "Light"),
                    DesignVariableMode::new("theme-dark", "Dark"),
                    DesignVariableMode::new("theme-high-contrast", "High contrast"),
                ],
                "theme-light",
                explicit_mode.unwrap_or("theme-light").to_owned(),
            );
            explicit_mode.map_or(collection.clone(), |mode| {
                collection.explicit(mode.to_owned())
            })
        }

        fn density_collection(resolved_mode: &str) -> DesignVariableModeCollection {
            DesignVariableModeCollection::new(
                "collection-density",
                "Density",
                DesignLocalResourceSource::library("library-brand", "Brand system"),
                [
                    DesignVariableMode::new("density-comfortable", "Comfortable"),
                    DesignVariableMode::new("density-compact", "Compact"),
                    DesignVariableMode::new("density-touch", "Touch")
                        .disabled("Touch density is unavailable for this target"),
                ],
                "density-comfortable",
                resolved_mode.to_owned(),
            )
        }

        let mut views = HashMap::new();
        let page_target = DesignPanelTarget::Page {
            page_id: "storybook-page".into(),
        };
        views.insert(
            "storybook-page".into(),
            DesignVariableModeViewData::new(
                page_target,
                [
                    theme_collection(Some("theme-dark")),
                    density_collection("density-compact"),
                ],
            ),
        );
        for (index, node) in nodes.iter().enumerate() {
            let target = DesignPanelTarget::Nodes {
                node_ids: vec![node.id.clone()],
            };
            let theme = if index % 3 == 0 {
                theme_collection(Some("theme-dark"))
            } else {
                theme_collection(None)
            };
            let density = density_collection(if index % 2 == 0 {
                "density-comfortable"
            } else {
                "density-compact"
            });
            views.insert(
                node.id.clone(),
                DesignVariableModeViewData::new(target, [theme, density]),
            );
        }
        views
    }

    fn seed_design_viewer_properties(
        nodes: &[DesignPanelNode],
    ) -> HashMap<SharedString, DesignViewerPropertiesViewData> {
        nodes
            .iter()
            .map(|node| {
                (
                    node.id.clone(),
                    Self::viewer_properties_for_node(node, DesignViewerColorRepresentation::Css),
                )
            })
            .collect()
    }

    fn viewer_properties_for_node(
        node: &DesignPanelNode,
        border_representation: DesignViewerColorRepresentation,
    ) -> DesignViewerPropertiesViewData {
        let mut sections = Vec::new();
        if let Some(context) = node.component_context.as_ref() {
            sections.push(story_viewer_component_section(context));
        }
        sections.push(DesignViewerPropertySection::new(
            "layout",
            "Layout",
            [
                DesignViewerPropertyRow::new("x", "X", story_viewer_number(node.x))
                    .with_property(DesignPanelProperty::X),
                DesignViewerPropertyRow::new("y", "Y", story_viewer_number(node.y))
                    .with_property(DesignPanelProperty::Y),
                DesignViewerPropertyRow::new("width", "Width", story_viewer_px(node.width))
                    .with_property(DesignPanelProperty::Width),
                DesignViewerPropertyRow::new("height", "Height", story_viewer_px(node.height))
                    .with_property(DesignPanelProperty::Height),
            ],
        ));

        if let Some(typography) = &node.typography {
            let content = if node.kind == DesignPanelNodeKind::TextPath {
                "Design follows the path"
            } else {
                "Build thoughtful products faster with a shared design system."
            };
            sections.push(DesignViewerPropertySection::text_content(
                "content", content,
            ));
            let line_height = match typography.line_height {
                DesignLineHeight::Auto => "Auto".to_owned(),
                DesignLineHeight::Pixels(value) => story_viewer_px(value),
                DesignLineHeight::Percent(value) => format!("{}%", story_viewer_number(value)),
            };
            let letter_spacing = match typography.letter_spacing {
                DesignLetterSpacing::Pixels(value) => story_viewer_px(value),
                DesignLetterSpacing::Percent(value) => {
                    format!("{}%", story_viewer_number(value))
                }
            };
            let typography_copy = format!(
                "font-family: \"{}\";\nfont-style: {};\nfont-weight: {};\nfont-size: {};\nline-height: {};\nletter-spacing: {};",
                typography.family,
                typography.style,
                story_viewer_number(typography.weight),
                story_viewer_px(typography.size),
                line_height,
                letter_spacing,
            );
            sections.push(
                DesignViewerPropertySection::new(
                    "typography",
                    "Typography",
                    [
                        DesignViewerPropertyRow::new(
                            "font-family",
                            "Font",
                            typography.family.clone(),
                        )
                        .with_property(DesignPanelProperty::FontFamily),
                        DesignViewerPropertyRow::new(
                            "font-style",
                            "Style",
                            typography.style.clone(),
                        )
                        .with_property(DesignPanelProperty::FontStyle),
                        DesignViewerPropertyRow::new(
                            "font-weight",
                            "Weight",
                            story_viewer_number(typography.weight),
                        )
                        .with_property(DesignPanelProperty::FontWeight),
                        DesignViewerPropertyRow::new(
                            "font-size",
                            "Size",
                            story_viewer_px(typography.size),
                        )
                        .with_property(DesignPanelProperty::FontSize),
                        DesignViewerPropertyRow::new("line-height", "Line height", line_height)
                            .with_property(DesignPanelProperty::LineHeight),
                        DesignViewerPropertyRow::new(
                            "letter-spacing",
                            "Letter spacing",
                            letter_spacing,
                        )
                        .with_property(DesignPanelProperty::LetterSpacing),
                    ],
                )
                .with_copy_value(typography_copy)
                .with_copy_all(),
            );
        }

        if let Some(fill) = node.fills.iter().find(|paint| paint.visible) {
            let summary = story_viewer_color(fill.color, DesignViewerColorRepresentation::Hex);
            sections.push(
                DesignViewerPropertySection::new("fills", "Fills", [])
                    .with_summary(summary.clone())
                    .with_copy_value(summary),
            );
        }

        if let Some(stroke) = &node.stroke {
            let color = stroke
                .paints
                .iter()
                .find(|paint| paint.visible)
                .map_or(DesignColor::BLACK, |paint| paint.color);
            let color_summary = story_viewer_color(color, border_representation);
            let weight = story_viewer_px(stroke.weights.active());
            let css_style =
                if stroke.dashes.mode == fanta_gpui::prelude::DesignStrokeDashMode::Solid {
                    "solid"
                } else {
                    "dashed"
                };
            let copy_value = if border_representation == DesignViewerColorRepresentation::Css {
                if stroke.weights.mode == DesignStrokeWeightMode::Custom {
                    format!(
                        "border-style: {css_style};\nborder-width: {} {} {} {};\nborder-color: {};",
                        story_viewer_px(stroke.weights.top),
                        story_viewer_px(stroke.weights.right),
                        story_viewer_px(stroke.weights.bottom),
                        story_viewer_px(stroke.weights.left),
                        story_viewer_color(color, DesignViewerColorRepresentation::Css),
                    )
                } else {
                    format!(
                        "border: {weight} {css_style} {};",
                        story_viewer_color(color, DesignViewerColorRepresentation::Css),
                    )
                }
            } else {
                color_summary.clone()
            };
            sections.push(
                DesignViewerPropertySection::new(
                    "borders",
                    "Borders",
                    [
                        DesignViewerPropertyRow::new("color", "Color", color_summary),
                        DesignViewerPropertyRow::new("weight", "Weight", weight)
                            .with_property(DesignPanelProperty::StrokeWeight),
                        DesignViewerPropertyRow::new("position", "Position", stroke.align.label())
                            .with_property(DesignPanelProperty::StrokeAlign),
                    ],
                )
                .with_summary(copy_value.clone())
                .with_copy_value(copy_value)
                .with_color_representation(border_representation),
            );
        }

        if !node.effects.is_empty() {
            let summary = node
                .effects
                .iter()
                .map(|effect| effect.kind.label())
                .collect::<Vec<_>>()
                .join(", ");
            sections.push(
                DesignViewerPropertySection::new("effects", "Effects", [])
                    .with_summary(summary.clone())
                    .with_copy_value(summary),
            );
        }

        DesignViewerPropertiesViewData::new(
            DesignPanelTarget::Nodes {
                node_ids: vec![node.id.clone()],
            },
            sections,
        )
    }

    fn seed_design_color_styles() -> DesignColorStyleViewData {
        let page_styles = [
            DesignColorStyle::new(
                "page-brand-primary",
                "Brand / Primary",
                DesignColor::rgb(0x0d, 0x99, 0xff),
            )
            .with_binding(
                DesignPaintBinding::new("brand-primary", "Brand / Primary")
                    .with_collection("Page variables"),
            ),
            DesignColorStyle::new(
                "page-surface-canvas",
                "Surface / Canvas",
                DesignColor::rgb(0xf5, 0xf5, 0xf5),
            ),
            DesignColorStyle::new(
                "page-ink-primary",
                "Ink / Primary",
                DesignColor::rgb(0x1e, 0x1e, 0x1e),
            ),
            DesignColorStyle::new(
                "page-accent-purple",
                "Accent / Purple",
                DesignColor::rgb(0x97, 0x47, 0xff),
            ),
        ];
        let libraries = [
            DesignColorStyleLibrary::new(
                "fanta-foundations",
                "Fanta foundations",
                [
                    DesignColorStyle::new(
                        "foundation-blue-500",
                        "Blue / 500",
                        DesignColor::rgb(0x0d, 0x99, 0xff),
                    )
                    .with_binding(
                        DesignPaintBinding::new("blue-500", "Blue / 500")
                            .with_collection("Primitives"),
                    ),
                    DesignColorStyle::new(
                        "foundation-green-500",
                        "Green / 500",
                        DesignColor::rgb(0x14, 0xae, 0x5c),
                    )
                    .with_binding(
                        DesignPaintBinding::new("green-500", "Green / 500")
                            .with_collection("Primitives"),
                    ),
                    DesignColorStyle::new(
                        "foundation-red-500",
                        "Red / 500",
                        DesignColor::rgb(0xf2, 0x48, 0x22),
                    )
                    .with_binding(
                        DesignPaintBinding::new("red-500", "Red / 500")
                            .with_collection("Primitives"),
                    ),
                ],
            ),
            DesignColorStyleLibrary::new(
                "marketing-theme",
                "Marketing theme",
                [
                    DesignColorStyle::new(
                        "marketing-sun",
                        "Campaign / Sun",
                        DesignColor::rgb(0xff, 0xc7, 0x00),
                    ),
                    DesignColorStyle::new(
                        "marketing-orchid",
                        "Campaign / Orchid",
                        DesignColor::rgb(0xd7, 0x32, 0xa8),
                    ),
                ],
            ),
        ];
        DesignColorStyleViewData::new(page_styles, libraries)
    }

    fn seed_design_color_style_samples() -> DesignColorStyleSampleViewData {
        DesignColorStyleSampleViewData::new(
            [
                DesignColorStyleSample::new(
                    "sample-page-brand",
                    "Brand / Primary",
                    DesignColor::rgb(0x0d, 0x99, 0xff),
                ),
                DesignColorStyleSample::new(
                    "sample-page-ink",
                    "Ink / Primary",
                    DesignColor::rgb(0x1e, 0x1e, 0x1e),
                ),
                DesignColorStyleSample::new(
                    "sample-page-overlay",
                    "Overlay / Soft",
                    DesignColor::rgba(0x97, 0x47, 0xff, 0x80),
                ),
            ],
            [
                DesignColorStyleSampleLibrary::new(
                    "sample-library-foundations",
                    "Fanta foundations",
                    [
                        DesignColorStyleSample::new(
                            "sample-library-blue",
                            "Blue / 500",
                            DesignColor::rgb(0x0d, 0x99, 0xff),
                        ),
                        DesignColorStyleSample::new(
                            "sample-library-green",
                            "Green / 500",
                            DesignColor::rgb(0x14, 0xae, 0x5c),
                        ),
                        DesignColorStyleSample::new(
                            "sample-library-red",
                            "Red / 500",
                            DesignColor::rgb(0xf2, 0x48, 0x22),
                        ),
                    ],
                ),
                DesignColorStyleSampleLibrary::new(
                    "sample-library-marketing",
                    "Marketing theme",
                    [
                        DesignColorStyleSample::new(
                            "sample-library-sun",
                            "Campaign / Sun",
                            DesignColor::rgb(0xff, 0xc7, 0x00),
                        ),
                        DesignColorStyleSample::new(
                            "sample-library-orchid",
                            "Campaign / Orchid",
                            DesignColor::rgb(0xd7, 0x32, 0xa8),
                        )
                        .disabled("The Marketing library is view-only"),
                    ],
                ),
            ],
        )
    }

    fn seed_design_paint_styles() -> DesignPaintStyleViewData {
        let page_styles = [
            DesignPaintStyle::new(
                "paint-style-brand-surface",
                "Brand / Surface",
                [DesignPaint::solid(DesignColor::rgb(0x0d, 0x99, 0xff))
                    .with_id("style-brand-surface-solid")],
            ),
            DesignPaintStyle::new(
                "paint-style-hero-gradient",
                "Marketing / Hero gradient",
                [
                    DesignPaint::gradient(
                        DesignPaintKind::LinearGradient,
                        vec![
                            DesignGradientStop::new(0., DesignColor::PURPLE)
                                .with_id("style-hero-gradient-start"),
                            DesignGradientStop::new(1., DesignColor::BLUE)
                                .with_id("style-hero-gradient-end"),
                        ],
                    )
                    .with_id("style-hero-gradient"),
                    DesignPaint::solid(DesignColor::rgba(0xff, 0xff, 0xff, 0x24))
                        .with_id("style-hero-highlight"),
                ],
            ),
        ];
        let libraries = [DesignPaintStyleLibrary::new(
            "paint-library-foundations",
            "Fanta foundations",
            [
                DesignPaintStyle::new(
                    "paint-style-library-ink",
                    "Ink / Primary",
                    [DesignPaint::solid(DesignColor::rgb(0x1e, 0x1e, 0x1e))
                        .with_id("style-library-ink-solid")],
                ),
                DesignPaintStyle::new(
                    "paint-style-library-campaign",
                    "Campaign / Aurora",
                    [DesignPaint::gradient(
                        DesignPaintKind::RadialGradient,
                        vec![
                            DesignGradientStop::new(0., DesignColor::rgb(0xff, 0xc7, 0x00))
                                .with_id("style-campaign-start"),
                            DesignGradientStop::new(1., DesignColor::rgb(0xd7, 0x32, 0xa8))
                                .with_id("style-campaign-end"),
                        ],
                    )
                    .with_id("style-campaign-gradient")],
                )
                .with_import_state(DesignPaintStyleImportState::Available),
            ],
        )];
        DesignPaintStyleViewData::new(page_styles, libraries)
    }

    fn seed_design_paint_variables() -> DesignPaintVariableViewData {
        let page = DesignVariableSource::page("page-5", "Page 5");
        let library =
            DesignVariableSource::library("paint-library-foundations", "Fanta foundations");
        DesignPaintVariableViewData::new([
            DesignVariable::page(
                "color-brand-primary",
                "Brand / Primary",
                "semantic-colors",
                "Semantic colors",
                DesignVariableResolvedType::Color,
            )
            .with_source(page)
            .with_scopes([DesignVariableScope::AllFills])
            .with_resolved_value(DesignVariableResolvedValue::Color(DesignColor::rgb(
                0x0d, 0x99, 0xff,
            ))),
            DesignVariable::page(
                "color-foundation-green",
                "Green / 500",
                "primitive-colors",
                "Primitive colors",
                DesignVariableResolvedType::Color,
            )
            .with_source(library.clone())
            .with_scopes([
                DesignVariableScope::AllFills,
                DesignVariableScope::StrokeColor,
            ])
            .with_import_state(DesignVariableImportState::Imported)
            .with_resolved_value(DesignVariableResolvedValue::Color(DesignColor::rgb(
                0x14, 0xae, 0x5c,
            ))),
            DesignVariable::page(
                "color-foundation-red",
                "Red / 500",
                "primitive-colors",
                "Primitive colors",
                DesignVariableResolvedType::Color,
            )
            .with_source(library)
            .with_scopes([
                DesignVariableScope::AllFills,
                DesignVariableScope::StrokeColor,
            ])
            .with_import_state(DesignVariableImportState::Available)
            .with_resolved_value(DesignVariableResolvedValue::Color(DesignColor::rgb(
                0xf2, 0x48, 0x22,
            ))),
        ])
    }

    fn seed_design_typography_styles() -> DesignTypographyStyleViewData {
        let page_styles = [
            DesignTypographyStyle::new("page-display-hero", "Display / Hero", "Inter", "Bold", 64.),
            DesignTypographyStyle::new(
                "page-heading-section",
                "Heading / Section",
                "Inter",
                "Semi Bold",
                32.,
            ),
            DesignTypographyStyle::new(
                "page-body-default",
                "Body / Default",
                "Inter",
                "Regular",
                16.,
            ),
            DesignTypographyStyle::new("page-label-small", "Label / Small", "Inter", "Medium", 12.),
        ];
        let libraries = [
            DesignTypographyStyleLibrary::new(
                "fanta-type",
                "Fanta type",
                [
                    DesignTypographyStyle::new(
                        "fanta-title-large",
                        "Title / Large",
                        "Inter",
                        "Bold",
                        40.,
                    ),
                    DesignTypographyStyle::new(
                        "fanta-body-compact",
                        "Body / Compact",
                        "Inter",
                        "Regular",
                        14.,
                    ),
                    DesignTypographyStyle::new(
                        "fanta-code",
                        "Code / Default",
                        "Roboto Mono",
                        "Regular",
                        13.,
                    ),
                ],
            ),
            DesignTypographyStyleLibrary::new(
                "marketing-type",
                "Marketing type",
                [
                    DesignTypographyStyle::new(
                        "marketing-campaign",
                        "Campaign / Display",
                        "Archivo",
                        "Black",
                        72.,
                    ),
                    DesignTypographyStyle::new(
                        "marketing-deck-title",
                        "Deck / Title",
                        "Archivo",
                        "Semi Bold",
                        36.,
                    ),
                    DesignTypographyStyle::new(
                        "marketing-caption",
                        "Caption / Editorial",
                        "Source Serif 4",
                        "Italic",
                        13.,
                    ),
                ],
            ),
        ];
        DesignTypographyStyleViewData::new(page_styles, libraries)
    }

    fn seed_design_fonts() -> DesignFontViewData {
        let unavailable_style = |id: &'static str,
                                 name: &'static str,
                                 availability: DesignFontAvailability|
         -> DesignFontStyle {
            DesignFontStyle {
                id: id.into(),
                name: name.into(),
                weight: None,
                italic: false,
                availability,
                preview: Some("The quick brown fox · 0123456789".into()),
            }
        };
        let families = [
            DesignFontFamily::local(
                "inter",
                "Inter",
                [
                    DesignFontStyle {
                        weight: Some(400),
                        preview: Some("Inter Regular · Aa Bb 0123".into()),
                        ..DesignFontStyle::imported("regular", "Regular")
                    },
                    DesignFontStyle {
                        weight: Some(500),
                        preview: Some("Inter Medium · Aa Bb 0123".into()),
                        ..DesignFontStyle::imported("medium", "Medium")
                    },
                    DesignFontStyle {
                        weight: Some(700),
                        preview: Some("Inter Bold · Aa Bb 0123".into()),
                        ..DesignFontStyle::imported("bold", "Bold")
                    },
                ],
            ),
            DesignFontFamily {
                id: "archivo".into(),
                name: "Archivo".into(),
                source: DesignFontSource::Library {
                    library_id: "marketing-type".into(),
                    library_name: "Marketing type".into(),
                },
                styles: vec![
                    unavailable_style("semi-bold", "Semi Bold", DesignFontAvailability::Available),
                    unavailable_style("black", "Black", DesignFontAvailability::Available),
                ],
            },
            DesignFontFamily::local(
                "retired-brand",
                "Retired Brand Sans",
                [
                    unavailable_style(
                        "regular",
                        "Regular",
                        DesignFontAvailability::Missing {
                            reason: "Font file is not installed".into(),
                        },
                    ),
                    unavailable_style(
                        "restricted",
                        "Restricted",
                        DesignFontAvailability::Unavailable {
                            reason: "Your organization has not enabled this font".into(),
                        },
                    ),
                ],
            ),
        ];
        match std::env::var("FANTA_FONT_CATALOG_STATE")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "loading" => DesignFontViewData::default(),
            "unavailable" => DesignFontViewData {
                state: fanta_gpui::prelude::DesignFontCatalogState::Unavailable {
                    reason: "The Storybook host denied font enumeration".into(),
                },
                families: Vec::new(),
            },
            _ => DesignFontViewData::ready(families),
        }
    }

    fn seed_design_effect_styles() -> DesignEffectStyleViewData {
        DesignEffectStyleViewData::new(
            [
                DesignEffectStyle::new(
                    "effect-page-elevation-1",
                    "Elevation / 1",
                    [DesignEffectKind::DropShadow],
                ),
                DesignEffectStyle::new(
                    "effect-page-frosted",
                    "Surface / Frosted",
                    [
                        DesignEffectKind::BackgroundBlur,
                        DesignEffectKind::DropShadow,
                    ],
                ),
            ],
            [DesignEffectStyleLibrary::new(
                "fanta-effects",
                "Fanta effects",
                [
                    DesignEffectStyle::new(
                        "effect-library-overlay",
                        "Overlay / Raised",
                        [DesignEffectKind::DropShadow, DesignEffectKind::InnerShadow],
                    ),
                    DesignEffectStyle::new(
                        "effect-library-glass",
                        "Material / Glass",
                        [DesignEffectKind::Glass],
                    ),
                ],
            )],
        )
    }

    fn seed_design_layout_grid_styles() -> DesignLayoutGridStyleViewData {
        let guide_color = DesignColor::rgb(0xf2, 0x48, 0x22);
        DesignLayoutGridStyleViewData::new(
            [
                DesignLayoutGridStyle::new(
                    "grid-style-8pt",
                    "Foundations / 8 pt grid",
                    [DesignLayoutGrid::uniform(8., guide_color)
                        .with_opacity(18.)
                        .with_id("style-8pt-grid")],
                ),
                DesignLayoutGridStyle::new(
                    "grid-style-columns",
                    "Responsive / Desktop columns",
                    [DesignLayoutGrid::columns(
                        DesignColumnLayoutGrid {
                            count: DesignLayoutGridCount::number(12),
                            gutter: 24.,
                            margin: 32.,
                            ..DesignColumnLayoutGrid::default()
                        },
                        guide_color,
                    )
                    .with_opacity(12.)
                    .with_id("style-desktop-columns")],
                ),
                DesignLayoutGridStyle::new(
                    "grid-style-rows",
                    "Responsive / Desktop rows",
                    [DesignLayoutGrid::rows(
                        DesignRowLayoutGrid {
                            count: DesignLayoutGridCount::number(8),
                            gutter: 16.,
                            margin: 24.,
                            ..DesignRowLayoutGrid::default()
                        },
                        DesignColor::BLUE,
                    )
                    .with_opacity(12.)
                    .with_id("style-desktop-rows")],
                ),
            ],
            [DesignLayoutGridStyleLibrary::new(
                "fanta-grid-library",
                "Fanta layout grids",
                [
                    DesignLayoutGridStyle::new(
                        "grid-style-locked",
                        "Library / Locked desktop grid",
                        [
                            DesignLayoutGrid::columns(
                                DesignColumnLayoutGrid::default(),
                                guide_color,
                            )
                            .with_id("style-locked-columns"),
                        ],
                    ),
                    DesignLayoutGridStyle::new(
                        "grid-style-editorial",
                        "Editorial / Baseline + columns",
                        [
                            DesignLayoutGrid::uniform(4., DesignColor::BLUE)
                                .with_opacity(8.)
                                .with_id("style-editorial-baseline"),
                            DesignLayoutGrid::columns(
                                DesignColumnLayoutGrid {
                                    count: DesignLayoutGridCount::number(6),
                                    gutter: 24.,
                                    margin: 40.,
                                    ..DesignColumnLayoutGrid::default()
                                },
                                guide_color,
                            )
                            .with_opacity(12.)
                            .with_id("style-editorial-columns"),
                        ],
                    )
                    .available(),
                ],
            )],
        )
    }

    fn seed_design_layout_grid_variables() -> DesignLayoutGridVariableViewData {
        DesignLayoutGridVariableViewData::new([
            DesignVariable::page(
                "desktop-column-count",
                "Desktop columns",
                "responsive",
                "Responsive",
                DesignVariableResolvedType::Float,
            )
            .with_resolved_value(DesignVariableResolvedValue::Float(12.)),
            DesignVariable::page(
                "desktop-row-count",
                "Desktop rows",
                "responsive",
                "Responsive",
                DesignVariableResolvedType::Float,
            )
            .with_resolved_value(DesignVariableResolvedValue::Float(8.)),
            DesignVariable::page(
                "auto-track-count",
                "Auto tracks",
                "responsive",
                "Responsive",
                DesignVariableResolvedType::Float,
            )
            .with_resolved_value(DesignVariableResolvedValue::Float(f32::INFINITY)),
            DesignVariable::page(
                "spacing-8",
                "Spacing / 8",
                "spacing",
                "Spacing",
                DesignVariableResolvedType::Float,
            )
            .with_resolved_value(DesignVariableResolvedValue::Float(8.)),
            DesignVariable::page(
                "grid-gutter-24",
                "Grid gutter / 24",
                "spacing",
                "Spacing",
                DesignVariableResolvedType::Float,
            )
            .with_source(DesignVariableSource::library(
                "layout-library",
                "Layout foundations",
            ))
            .with_import_state(DesignVariableImportState::Imported)
            .with_resolved_value(DesignVariableResolvedValue::Float(24.)),
            DesignVariable::page(
                "grid-section-64",
                "Grid section / 64",
                "spacing",
                "Spacing",
                DesignVariableResolvedType::Float,
            )
            .with_source(DesignVariableSource::library(
                "layout-library",
                "Layout foundations",
            ))
            .with_import_state(DesignVariableImportState::Available)
            .with_resolved_value(DesignVariableResolvedValue::Float(64.)),
            DesignVariable::page(
                "restricted-grid-number",
                "Restricted grid number",
                "library-variables",
                "Library variables",
                DesignVariableResolvedType::Float,
            )
            .with_resolved_value(DesignVariableResolvedValue::Float(8.))
            .disabled("Unavailable in the active variable mode"),
        ])
        .with_create_state(DesignLayoutGridVariableCreateState::Enabled)
    }

    fn seed_design_frame_presets(
        nodes: &[DesignPanelNode],
    ) -> HashMap<SharedString, DesignFramePresetViewData> {
        let groups = vec![
            DesignFramePresetGroup::new(
                "phone",
                "Phone",
                [
                    DesignFramePreset::new("iphone-16-pro", "iPhone 16 Pro", 402., 874.),
                    DesignFramePreset::new("iphone-14-15-pro", "iPhone 14 & 15 Pro", 393., 852.),
                    DesignFramePreset::new("android-small", "Android Small", 360., 800.)
                        .disabled("Disabled by the Storybook host"),
                ],
            ),
            DesignFramePresetGroup::new(
                "tablet",
                "Tablet",
                [
                    DesignFramePreset::new("ipad-pro-11", "iPad Pro 11″", 834., 1194.),
                    DesignFramePreset::new("surface-pro-8", "Surface Pro 8", 1440., 960.),
                ],
            ),
            DesignFramePresetGroup::new(
                "desktop",
                "Desktop",
                [
                    DesignFramePreset::new("desktop", "Desktop", 1440., 1024.),
                    DesignFramePreset::new("macbook-air", "MacBook Air", 1280., 832.),
                ],
            ),
            DesignFramePresetGroup::new(
                "presentation",
                "Presentation",
                [
                    DesignFramePreset::new("slide-16-9", "Slide 16:9", 1920., 1080.),
                    DesignFramePreset::new("slide-4-3", "Slide 4:3", 1024., 768.),
                ],
            ),
            DesignFramePresetGroup::new(
                "watch",
                "Watch",
                [DesignFramePreset::new(
                    "apple-watch-45",
                    "Apple Watch 45 mm",
                    396.,
                    484.,
                )],
            ),
            DesignFramePresetGroup::new(
                "paper",
                "Paper",
                [
                    DesignFramePreset::new("a4", "A4", 595., 842.),
                    DesignFramePreset::new("us-letter", "US Letter", 612., 792.),
                ],
            ),
            DesignFramePresetGroup::new(
                "social-media",
                "Social Media",
                [
                    DesignFramePreset::new("instagram-post", "Instagram Post", 1080., 1080.),
                    DesignFramePreset::new("youtube", "YouTube", 1280., 720.),
                ],
            ),
            DesignFramePresetGroup::new(
                "figma-community",
                "Figma Community",
                [DesignFramePreset::new(
                    "community-cover",
                    "Community cover",
                    1920.,
                    960.,
                )],
            )
            .disabled("Connect a Community catalog to use these presets"),
            DesignFramePresetGroup::new(
                "archive",
                "Archive",
                [
                    DesignFramePreset::new("iphone-8", "iPhone 8", 375., 667.),
                    DesignFramePreset::new("ipad-mini", "iPad mini", 768., 1024.),
                ],
            ),
        ];

        nodes
            .iter()
            .filter(|node| node.kind == DesignPanelNodeKind::Frame)
            .map(|node| {
                (
                    node.id.clone(),
                    DesignFramePresetViewData::new(node.id.clone(), groups.clone()),
                )
            })
            .collect()
    }

    fn seed_design_effect_variables() -> DesignEffectVariableViewData {
        DesignEffectVariableViewData::new([
            DesignEffectVariable::new(
                "effect-radius-md",
                "Radius / Medium",
                "Effects",
                DesignEffectVariableKind::Float,
            ),
            DesignEffectVariable::new(
                "effect-offset-sm",
                "Offset / Small",
                "Effects",
                DesignEffectVariableKind::Float,
            ),
            DesignEffectVariable::new(
                "effect-shadow-color",
                "Shadow / Default",
                "Semantic colors",
                DesignEffectVariableKind::Color,
            )
            .remote(true),
        ])
    }

    fn seed_design_property_variables() -> DesignVariableViewData {
        let page = DesignVariableSource::page("page-5", "Page 5");
        let library = DesignVariableSource::library("fanta-foundations", "Fanta foundations");
        DesignVariableViewData::new([
            DesignVariable::page(
                "size-card",
                "Card",
                "layout-size",
                "Layout / Size",
                DesignVariableResolvedType::Float,
            )
            .with_source(page.clone())
            .with_scopes([DesignVariableScope::WidthHeight])
            .with_resolved_value(DesignVariableResolvedValue::Float(320.))
            .with_search(
                DesignVariableSearchMetadata::new(
                    ["layout", "dimensions"],
                    ["width", "height", "card"],
                )
                .described("Default card width and height"),
            ),
            DesignVariable::page(
                "space-200",
                "200",
                "spacing",
                "Foundations / Spacing",
                DesignVariableResolvedType::Float,
            )
            .with_source(page.clone())
            .with_scopes([DesignVariableScope::Gap])
            .with_resolved_value(DesignVariableResolvedValue::Float(8.))
            .with_search(DesignVariableSearchMetadata::new(
                ["foundations", "spacing"],
                ["gap", "padding", "8"],
            )),
            DesignVariable::page(
                "surface-opacity",
                "Surface",
                "appearance",
                "Appearance",
                DesignVariableResolvedType::Float,
            )
            .with_source(page.clone())
            .with_scopes([DesignVariableScope::Opacity])
            .with_resolved_value(DesignVariableResolvedValue::Float(88.))
            .with_search(DesignVariableSearchMetadata::new(
                ["appearance"],
                ["opacity", "surface"],
            )),
            DesignVariable::page(
                "layer-visible",
                "Layer visible",
                "behavior",
                "Behavior",
                DesignVariableResolvedType::Boolean,
            )
            .with_source(page.clone())
            .with_resolved_value(DesignVariableResolvedValue::Boolean(true))
            .with_search(DesignVariableSearchMetadata::new(
                ["behavior"],
                ["visible", "visibility"],
            )),
            DesignVariable::page(
                "font-body-family",
                "Body family",
                "typography",
                "Typography",
                DesignVariableResolvedType::String,
            )
            .with_source(library.clone())
            .with_scopes([DesignVariableScope::FontFamily])
            .with_import_state(DesignVariableImportState::Imported)
            .with_resolved_value(DesignVariableResolvedValue::String("Inter".into()))
            .with_search(DesignVariableSearchMetadata::new(
                ["typography", "body"],
                ["font", "family", "inter"],
            )),
            DesignVariable::page(
                "font-body-style",
                "Body style",
                "typography",
                "Typography",
                DesignVariableResolvedType::String,
            )
            .with_source(page.clone())
            .with_scopes([DesignVariableScope::FontStyle])
            .with_resolved_value(DesignVariableResolvedValue::String("Medium".into()))
            .with_search(DesignVariableSearchMetadata::new(
                ["typography", "body"],
                ["font", "style", "medium"],
            )),
            DesignVariable::page(
                "font-body-weight",
                "Body weight",
                "typography",
                "Typography",
                DesignVariableResolvedType::Float,
            )
            .with_source(page.clone())
            .with_scopes([DesignVariableScope::FontWeight])
            .with_resolved_value(DesignVariableResolvedValue::Float(500.))
            .with_search(DesignVariableSearchMetadata::new(
                ["typography", "body"],
                ["font", "weight", "500"],
            )),
            DesignVariable::page(
                "font-display-weight",
                "Display weight",
                "typography",
                "Typography",
                DesignVariableResolvedType::Float,
            )
            .with_source(library.clone())
            .with_scopes([DesignVariableScope::FontWeight])
            .with_import_state(DesignVariableImportState::Available)
            .with_resolved_value(DesignVariableResolvedValue::Float(700.))
            .with_search(DesignVariableSearchMetadata::new(
                ["typography", "display"],
                ["font", "weight", "bold", "700"],
            )),
            DesignVariable::page(
                "component-label-campaign",
                "Campaign label",
                "component-copy",
                "Components / Copy",
                DesignVariableResolvedType::String,
            )
            .with_source(library.clone())
            .with_import_state(DesignVariableImportState::Available)
            .with_resolved_value(DesignVariableResolvedValue::String(
                "Start free trial\nNo credit card required".into(),
            ))
            .with_search(DesignVariableSearchMetadata::new(
                ["components", "copy"],
                ["label", "button", "campaign"],
            )),
            DesignVariable::page(
                "radius-container",
                "Container",
                "radius",
                "Foundations / Radius",
                DesignVariableResolvedType::Float,
            )
            .with_source(library)
            .with_scopes([DesignVariableScope::CornerRadius])
            .with_import_state(DesignVariableImportState::Available)
            .with_resolved_value(DesignVariableResolvedValue::Float(16.))
            .with_search(DesignVariableSearchMetadata::new(
                ["foundations", "radius"],
                ["corner", "container", "16"],
            )),
            DesignVariable::page(
                "brand-accent",
                "Accent",
                "semantic-colors",
                "Semantic / Color",
                DesignVariableResolvedType::Color,
            )
            .with_source(page)
            .with_scopes([DesignVariableScope::AllFills])
            .with_resolved_value(DesignVariableResolvedValue::Color(DesignColor::BLUE)),
        ])
    }

    fn seed_design_component_swaps() -> DesignComponentSwapViewData {
        DesignComponentSwapViewData::new([
            DesignComponentSwapCandidate::local(
                DesignComponentReference::local("icon-arrow-right", "Arrow right"),
                "local-key-arrow-right",
                "page-5",
                "Page 5",
            )
            .with_search(DesignComponentSearchMetadata::new(
                ["icons", "arrows"],
                ["arrow", "next", "direction"],
            )),
            DesignComponentSwapCandidate::local(
                DesignComponentReference::local("icon-plus", "Plus"),
                "local-key-plus",
                "page-5",
                "Page 5",
            )
            .with_search(DesignComponentSearchMetadata::new(
                ["icons", "actions"],
                ["plus", "add", "create"],
            )),
            DesignComponentSwapCandidate::local(
                DesignComponentReference::local("icon-status-set", "Status icons"),
                "local-key-status-set",
                "page-5",
                "Page 5",
            )
            .with_asset_kind(DesignComponentAssetKind::ComponentSet)
            .with_search(DesignComponentSearchMetadata::new(
                ["icons", "sets"],
                ["status", "check", "warning"],
            )),
            DesignComponentSwapCandidate::library(
                DesignComponentReference::remote("icon-check", "Check", "Product foundations"),
                "library-key-icon-check",
                "product-foundations",
                "Product foundations",
            )
            .with_import_state(DesignComponentImportState::Imported)
            .with_search(DesignComponentSearchMetadata::new(
                ["icons", "actions"],
                ["check", "success", "complete"],
            )),
            DesignComponentSwapCandidate::library(
                DesignComponentReference::remote("icon-sparkle", "Sparkle", "Marketing components"),
                "library-key-icon-sparkle",
                "marketing-components",
                "Marketing components",
            )
            .with_search(
                DesignComponentSearchMetadata::new(
                    ["icons", "decorative"],
                    ["sparkle", "ai", "magic"],
                )
                .described("Available from the Marketing components library"),
            ),
            DesignComponentSwapCandidate::library(
                DesignComponentReference::remote(
                    "icon-legacy",
                    "Legacy icon",
                    "Archived components",
                )
                .with_availability(DesignComponentAvailability::Unavailable {
                    reason: "Library access is required".into(),
                }),
                "library-key-icon-legacy",
                "archived-components",
                "Archived components",
            )
            .disabled("Library access is required"),
        ])
    }

    fn seed_design_media_paint_views(
        nodes: &[DesignPanelNode],
    ) -> HashMap<SharedString, DesignMediaPaintViewData> {
        nodes
            .iter()
            .filter_map(|node| {
                let paints = node
                    .fills
                    .iter()
                    .enumerate()
                    .filter_map(|(index, paint)| match &paint.payload {
                        DesignPaintPayload::Image(image) => {
                            let transform = image.placement.crop_transform().unwrap_or_default();
                            let capabilities = match paint.id.as_ref() {
                                "image-1-fill" => DesignMediaPaintCapabilities::editor()
                                    .with_source_actions(true, false, true)
                                    .with_accepted_drop_file_kinds(
                                        DesignMediaFileKinds::STANDARD | DesignMediaFileKinds::TIFF,
                                    ),
                                "image-2-fill" => {
                                    DesignMediaPaintCapabilities::property_editor_only()
                                }
                                "image-3-fill" => DesignMediaPaintCapabilities::editor()
                                    .with_source_actions(false, true, false),
                                _ => DesignMediaPaintCapabilities::editor(),
                            };
                            Some(
                                DesignMediaPaintView::new(
                                    DesignPanelCollection::Fill,
                                    paint.id.clone(),
                                    index,
                                )
                                .with_capabilities(capabilities)
                                .with_crop_tool(
                                    DesignMediaCropToolState {
                                        active: paint.id.as_ref() == "image-2-fill",
                                        transform,
                                        zoom: 1.25,
                                        aspect_ratio: DesignMediaCropAspectRatio::Original,
                                    },
                                ),
                            )
                        }
                        DesignPaintPayload::Video(video) => {
                            let transform = video.placement.crop_transform().unwrap_or_default();
                            let preview = match paint.id.as_ref() {
                                "video-0-fill" => DesignVideoPreviewState::ready(18., 3.5, false),
                                "video-1-fill" => DesignVideoPreviewState::ready(31., 12., true),
                                "video-2-fill" => DesignVideoPreviewState::loading(),
                                _ => DesignVideoPreviewState::error(
                                    "The host could not decode this preview",
                                ),
                            };
                            let capabilities = if paint.id.as_ref() == "video-0-fill" {
                                DesignMediaPaintCapabilities::property_editor_only()
                            } else {
                                DesignMediaPaintCapabilities::editor()
                            };
                            Some(
                                DesignMediaPaintView::new(
                                    DesignPanelCollection::Fill,
                                    paint.id.clone(),
                                    index,
                                )
                                .with_capabilities(capabilities)
                                .with_crop_tool(DesignMediaCropToolState {
                                    active: false,
                                    transform,
                                    zoom: 1.,
                                    aspect_ratio: DesignMediaCropAspectRatio::Free,
                                })
                                .with_video_preview(preview),
                            )
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                (!paints.is_empty())
                    .then(|| (node.id.clone(), DesignMediaPaintViewData::new(paints)))
            })
            .collect()
    }

    fn seed_design_export_configurations(
        nodes: &[DesignPanelNode],
    ) -> HashMap<SharedString, Vec<DesignExportConfiguration>> {
        let mut configurations = nodes
            .iter()
            .map(|node| {
                let rows = if node.export_settings.is_empty() {
                    Vec::new()
                } else {
                    node.export_settings
                        .iter()
                        .enumerate()
                        .map(|(index, setting)| {
                            DesignExportConfiguration::from_legacy(
                                format!("{}-export-{index}", node.id),
                                setting,
                            )
                        })
                        .collect()
                };
                (node.id.clone(), rows)
            })
            .collect::<HashMap<_, _>>();

        if let Some(slice) = nodes
            .iter()
            .find(|node| node.kind == DesignPanelNodeKind::Slice)
        {
            let mut scale =
                DesignExportConfiguration::new("slice-png-scale", DesignExportFormat::Png);
            scale.sizing = DesignExportSizing::Scale(2.);
            scale.common.suffix = "@2x".into();

            let mut height =
                DesignExportConfiguration::new("slice-png-height", DesignExportFormat::Png);
            height.sizing = DesignExportSizing::Height(800.);
            height.common.suffix = "-800h".into();

            let mut width =
                DesignExportConfiguration::new("slice-jpg-width", DesignExportFormat::Jpg);
            width.sizing = DesignExportSizing::Width(1440.);
            width.common.suffix = "-1440w".into();

            let mut svg = DesignExportConfiguration::new("slice-svg", DesignExportFormat::Svg);
            svg.common.suffix = "-vector".into();
            let mut pdf = DesignExportConfiguration::new("slice-pdf", DesignExportFormat::Pdf);
            pdf.common.suffix = "-print".into();
            configurations.insert(slice.id.clone(), vec![scale, height, width, svg, pdf]);
        }

        for reference_id in ["reference-text", "reference-arrow", "reference-frame"] {
            if let Some(node) = nodes.iter().find(|node| node.id == reference_id) {
                let format = if reference_id == "reference-frame" {
                    DesignExportFormat::Png
                } else {
                    DesignExportFormat::Svg
                };
                configurations.insert(
                    node.id.clone(),
                    vec![DesignExportConfiguration::new_for_capabilities(
                        format!("{reference_id}-export"),
                        format,
                        Self::static_export_capabilities(node),
                    )],
                );
            }
        }

        configurations.insert(
            "storybook-page".into(),
            vec![DesignExportConfiguration::new(
                "page-export-pdf",
                DesignExportFormat::Pdf,
            )],
        );
        configurations
    }

    fn static_export_capabilities(node: &DesignPanelNode) -> DesignStaticExportCapabilities {
        story_static_export_capabilities(node)
    }

    fn seed_design_animated_exports(
        nodes: &[DesignPanelNode],
    ) -> HashMap<SharedString, DesignAnimatedExportViewData> {
        nodes
            .iter()
            .map(|node| {
                let (capability, settings) = match node.id.as_ref() {
                    "reference-frame" => (
                        DesignAnimatedExportCapability::eligible(1920, 1080),
                        DesignAnimatedExportSettings::default(),
                    ),
                    "layout-horizontal" => {
                        let mut capability = DesignAnimatedExportCapability::eligible(1920, 1080);
                        capability.high_resolution_allowed = false;
                        capability.high_resolution_reason =
                            Some("Upgrade to export above 1080p or 30 FPS".into());
                        (
                            capability,
                            DesignAnimatedExportSettings::Mp4 {
                                sizing: DesignExportSizing::Scale(1.),
                                fps: DesignVideoExportFps::Fps60,
                                quality: fanta_gpui::prelude::DesignExportImageQuality::High,
                            },
                        )
                    }
                    "layout-vertical" => (
                        DesignAnimatedExportCapability::disabled(
                            "Video export requires a top-level animated frame",
                        ),
                        DesignAnimatedExportSettings::for_format(DesignAnimatedExportFormat::WebM),
                    ),
                    "reference-rectangle" => (
                        DesignAnimatedExportCapability::disabled(
                            "Add motion to a top-level frame to export animation",
                        ),
                        DesignAnimatedExportSettings::for_format(DesignAnimatedExportFormat::Gif),
                    ),
                    "reference-arrow" => (
                        DesignAnimatedExportCapability::disabled(
                            "Only top-level animated frames can be exported",
                        ),
                        DesignAnimatedExportSettings::Svg {
                            options: vec![
                                DesignAnimatedSvgOption::new(
                                    "precision",
                                    "Precision",
                                    "Balanced",
                                    ["Compact", "Balanced", "Exact"],
                                ),
                                DesignAnimatedSvgOption::new(
                                    "loop",
                                    "Loop",
                                    "Forever",
                                    ["Once", "Forever"],
                                ),
                            ],
                        },
                    ),
                    _ => (
                        DesignAnimatedExportCapability::disabled(
                            "Add motion to a top-level frame to export animation",
                        ),
                        DesignAnimatedExportSettings::default(),
                    ),
                };
                (
                    node.id.clone(),
                    DesignAnimatedExportViewData::new(capability, settings),
                )
            })
            .collect()
    }

    fn seed_design_export_previews(
        nodes: &[DesignPanelNode],
    ) -> HashMap<SharedString, DesignExportPreviewState> {
        nodes
            .iter()
            .map(|node| {
                let state = match node.id.as_ref() {
                    "reference-frame" => DesignExportPreviewState::Ready(
                        DesignExportPreview::new(1920, 1080)
                            .with_thumbnail("storybook-frame-preview")
                            .with_estimated_output("1.8 MB"),
                    ),
                    "reference-rectangle" => DesignExportPreviewState::Loading,
                    "reference-text" => DesignExportPreviewState::Error {
                        message: "Preview could not be generated".into(),
                    },
                    _ => DesignExportPreviewState::Idle,
                };
                (node.id.clone(), state)
            })
            .collect()
    }

    fn design_node_index_for_kind(&self, kind: DesignPanelNodeKind) -> Option<usize> {
        self.design_nodes.iter().position(|node| node.kind == kind)
    }

    fn design_node_index_for_id(&self, id: &str) -> Option<usize> {
        self.design_nodes
            .iter()
            .position(|node| node.id.as_ref() == id)
    }

    fn design_paint_target_for(
        &self,
        node_id: &SharedString,
        collection: DesignPanelCollection,
    ) -> DesignPaintTarget {
        let selected = &self.design_nodes[self.selected_design_node];
        if collection == DesignPanelCollection::Fill
            && self.design_inspection_scenario == DesignInspectionScenario::TextEdit
            && selected.id == *node_id
            && matches!(
                selected.kind,
                DesignPanelNodeKind::Text | DesignPanelNodeKind::TextPath
            )
        {
            DesignPaintTarget::SelectedTextRangeRevision(self.design_text_range_revision)
        } else {
            DesignPaintTarget::WholeLayer
        }
    }

    fn design_smart_selection_view_data(
        &self,
        target: DesignPanelTarget,
    ) -> Option<DesignSmartSelectionViewData> {
        let spacing = |axis| {
            DesignSmartSelectionSpacingViewData::new(
                self.design_smart_selection_spacing
                    .get(&axis)
                    .copied()
                    .unwrap_or(DesignSmartSelectionSpacingValue::Mixed),
            )
        };
        let available = || DesignSmartSelectionAvailability::Available;
        match self.design_inspection_scenario {
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

    fn design_inspection_context(
        &self,
    ) -> (
        DesignPanelInspectionContext,
        Vec<(
            DesignPanelProperty,
            DesignPanelPropertyValueState<DesignPanelValue>,
        )>,
    ) {
        let selected = self.design_nodes[self.selected_design_node].clone();
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
        let (context, mut property_states) = match self.design_inspection_scenario {
            DesignInspectionScenario::Page
            | DesignInspectionScenario::ViewOnlyPage
            | DesignInspectionScenario::RestrictedPage => (
                DesignPanelInspectionContext::page(self.design_inspection_scenario.permissions()),
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
                    .design_nodes
                    .iter()
                    .find(|node| node.id.as_ref() == STORY_HOMOGENEOUS_MULTIPLE_NODE_IDS[0])
                    .expect("the homogeneous multiple Story keeps its first node")
                    .clone();
                let second = self
                    .design_nodes
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
                    .design_nodes
                    .iter()
                    .enumerate()
                    .find(|(index, node)| {
                        *index != self.selected_design_node
                            && node.kind == DesignPanelNodeKind::Ellipse
                    })
                    .map(|(index, _)| index)
                    .or_else(|| {
                        self.design_nodes
                            .iter()
                            .enumerate()
                            .find(|(index, _)| *index != self.selected_design_node)
                            .map(|(index, _)| index)
                    })
                    .expect("the Design story always seeds multiple node presets");
                let second = self.design_nodes[second_index].clone();
                let permissions = self.design_inspection_scenario.permissions();
                story_multiple_inspection_context(&selected, &second, permissions)
            }
            DesignInspectionScenario::ViewOnlySingle => (
                DesignPanelInspectionContext::single(
                    selected,
                    DesignPanelParentLayout::Freeform,
                    self.design_inspection_scenario.permissions(),
                ),
                Vec::new(),
            ),
            DesignInspectionScenario::RestrictedSingle => (
                DesignPanelInspectionContext::single(
                    selected,
                    DesignPanelParentLayout::Freeform,
                    self.design_inspection_scenario.permissions(),
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
                story_text_edit_inspection_context(selected, self.design_text_range_revision),
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

    fn apply_design_inspection_context(&self, panel: Entity<DesignPanel>, cx: &mut Context<Self>) {
        let (inspection_context, mut property_states) = self.design_inspection_context();
        let selection_header_view_data = match inspection_context.selection().kind() {
            fanta_gpui::prelude::DesignPanelSelectionKind::None => None,
            fanta_gpui::prelude::DesignPanelSelectionKind::Single => {
                let node = &inspection_context.selection().items()[0];
                let mut view_data = DesignSelectionHeaderViewData::for_node_kind(node.kind);
                if node.kind == DesignPanelNodeKind::Ellipse
                    && self.design_inspection_scenario != DesignInspectionScenario::CanvasSingle
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
        let add_auto_layout_view_data = match self.design_inspection_scenario {
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
            _ if self.design_workspace_mode == DesignPanelWorkspaceMode::Draw
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
        let smart_selection_view_data = self.design_smart_selection_view_data(target.clone());
        let export_key = match &target {
            DesignPanelTarget::Page { page_id } => page_id.clone(),
            DesignPanelTarget::Nodes { node_ids } => node_ids
                .first()
                .cloned()
                .unwrap_or_else(|| "storybook-page".into()),
        };
        let media_paint_view_data = self
            .design_media_paint_views
            .get(&export_key)
            .cloned()
            .unwrap_or_default();
        let shader_view_data = self.design_shaders.clone();
        let color_style_sample_view_data = self.design_color_style_samples.clone();
        let color_contrast_view_data = Self::seed_design_color_contrast(&self.design_nodes);
        let paint_style_view_data = self.design_paint_styles.clone();
        let paint_variable_view_data = self.design_paint_variables.clone();
        let layout_grid_style_view_data = self.design_layout_grid_styles.clone();
        let layout_grid_variable_view_data = self.design_layout_grid_variables.clone();
        let frame_preset_view_data = self.design_frame_presets.get(&export_key).cloned();
        let page_view_data = self.design_page_view_data.clone();
        let page_local_styles = self.design_page_local_styles.clone();
        let variable_mode_view_data = match &target {
            DesignPanelTarget::Page { .. } => {
                self.design_variable_mode_views.get(&export_key).cloned()
            }
            DesignPanelTarget::Nodes { node_ids } if node_ids.len() == 1 => {
                self.design_variable_mode_views.get(&export_key).cloned()
            }
            DesignPanelTarget::Nodes { .. } => None,
        };
        let viewer_properties_view_data = match &target {
            DesignPanelTarget::Nodes { node_ids } if node_ids.len() == 1 => {
                self.design_viewer_properties.get(&export_key).cloned()
            }
            DesignPanelTarget::Page { .. } | DesignPanelTarget::Nodes { .. } => None,
        };
        let draw_appearance_view_data =
            story_draw_appearance_view_data(&inspection_context, &target);
        let export_view_data = story_export_projection(
            target,
            &self.design_nodes,
            &self.design_export_configurations,
            &self.design_export_modes,
            &self.design_animated_exports,
            &self.design_export_previews,
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
            for (node_id, property) in self.design_property_bindings.keys() {
                if node_id != selected_node_id {
                    continue;
                }
                let Some(binding) = story_common_property_binding(
                    &self.design_property_bindings,
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
            panel.set_workspace_mode(self.design_workspace_mode, cx);
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
            panel.set_property_variable_view_data(self.design_property_variables.clone(), cx);
            panel.set_component_swap_view_data(self.design_component_swaps.clone(), cx);
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

    fn activate_design_inspection_scenario(
        &mut self,
        scenario: DesignInspectionScenario,
        cx: &mut Context<Self>,
    ) {
        if scenario == DesignInspectionScenario::AddAutoLayoutGroup
            && let Some(group) = self
                .design_nodes
                .iter_mut()
                .find(|node| node.id.as_ref() == "add-auto-layout-group")
        {
            *group = DesignPanelNode::new(
                "add-auto-layout-group",
                "Navigation cluster · Group",
                DesignPanelNodeKind::Group,
            );
        }
        let required_index = match scenario {
            DesignInspectionScenario::AddAutoLayoutGroup => {
                self.design_node_index_for_id("add-auto-layout-group")
            }
            DesignInspectionScenario::AddAutoLayoutMultiple => {
                self.design_node_index_for_kind(DesignPanelNodeKind::Rectangle)
            }
            DesignInspectionScenario::HomogeneousMultiple => {
                self.design_node_index_for_id(STORY_HOMOGENEOUS_MULTIPLE_NODE_IDS[0])
            }
            DesignInspectionScenario::AutoLayoutChild => {
                self.design_node_index_for_id("layout-child-horizontal")
            }
            DesignInspectionScenario::AutoLayoutIgnored => {
                self.design_node_index_for_id("layout-child-absolute")
            }
            DesignInspectionScenario::AutoLayoutVerticalChild => {
                self.design_node_index_for_id("layout-child-vertical")
            }
            DesignInspectionScenario::GridChild => {
                self.design_node_index_for_id("layout-grid-child-placement")
            }
            DesignInspectionScenario::TextEdit | DesignInspectionScenario::PropertyStates => {
                self.design_node_index_for_kind(DesignPanelNodeKind::Text)
            }
            DesignInspectionScenario::VectorEdit => {
                if matches!(
                    self.design_nodes[self.selected_design_node].kind,
                    DesignPanelNodeKind::Vector | DesignPanelNodeKind::TextPath
                ) {
                    None
                } else {
                    self.design_node_index_for_kind(DesignPanelNodeKind::Vector)
                }
            }
            DesignInspectionScenario::Page
            | DesignInspectionScenario::ViewOnlyPage
            | DesignInspectionScenario::RestrictedPage
            | DesignInspectionScenario::EditableSingle
            | DesignInspectionScenario::CanvasSingle
            | DesignInspectionScenario::EditableMultiple
            | DesignInspectionScenario::ViewOnlyMultiple
            | DesignInspectionScenario::RestrictedMultiple
            | DesignInspectionScenario::SmartSelectionNone
            | DesignInspectionScenario::SmartSelectionHorizontal
            | DesignInspectionScenario::SmartSelectionVertical
            | DesignInspectionScenario::SmartSelectionTwoDimensional
            | DesignInspectionScenario::SmartSelectionReadOnly
            | DesignInspectionScenario::ViewOnlySingle
            | DesignInspectionScenario::RestrictedSingle => None,
        };
        if let Some(index) = required_index {
            self.selected_design_node = index;
        }
        for (axis, (_, original)) in std::mem::take(&mut self.design_smart_selection_edit_snapshots)
        {
            self.design_smart_selection_spacing.insert(axis, original);
        }
        self.design_inspection_scenario = scenario;
        self.apply_design_inspection_context(self.design_panel.clone(), cx);
        self.design_last_action = format!("Story switched to {}", scenario.label()).into();
    }

    fn advance_design_text_range(&mut self, cx: &mut Context<Self>) {
        if self.design_inspection_scenario != DesignInspectionScenario::TextEdit {
            return;
        }
        self.design_text_range_revision = self.design_text_range_revision.wrapping_add(1);
        self.apply_design_inspection_context(self.design_panel.clone(), cx);
        self.design_last_action = format!(
            "Host selected a new character range (revision {})",
            self.design_text_range_revision
        )
        .into();
        cx.notify();
    }

    fn handle_variables_action(
        &mut self,
        page: Entity<VariablesPage>,
        action: &VariablesAction,
        cx: &mut Context<Self>,
    ) {
        match action {
            VariablesAction::CollectionSelected { collection_id } => {
                self.variables_view_data.selected_collection_id = collection_id.clone();
                self.variables_view_data.selected_group_id = "all".into();
                self.variables_last_action = format!("Selected collection {collection_id}").into();
            }
            VariablesAction::GroupSelected { group_id } => {
                self.variables_view_data.selected_group_id = group_id.clone();
                self.variables_last_action = format!("Selected variable group {group_id}").into();
            }
            VariablesAction::SearchQueryChanged { query } => {
                self.variables_last_action = if query.is_empty() {
                    "Cleared the variables search".into()
                } else {
                    format!("Filtered variables by “{query}”").into()
                };
            }
            VariablesAction::SearchOptionsRequested => {
                self.variables_last_action = "Host opened Variables search options".into();
            }
            VariablesAction::CreateCollectionRequested => {
                let ordinal = self.variables_view_data.collections.len() + 1;
                let collection_id: SharedString = format!("collection-{ordinal}").into();
                self.variables_view_data
                    .collections
                    .push(VariablesCollection::new(
                        collection_id.clone(),
                        format!("Collection {ordinal}"),
                        self.variables_view_data.variables.len(),
                    ));
                self.variables_view_data.selected_collection_id = collection_id.clone();
                self.variables_last_action =
                    format!("Created and selected {collection_id} through the host adapter").into();
            }
            VariablesAction::CreateVariableRequested => {
                let ordinal = self.variables_view_data.variables.len() + 1;
                let values = self
                    .variables_view_data
                    .modes
                    .iter()
                    .map(|mode| VariableModeValue::new(mode.id.clone(), "0"))
                    .collect::<Vec<_>>();
                self.variables_view_data.variables.push(VariableRow::new(
                    format!("spacing-{ordinal}"),
                    format!("Spacing {ordinal}"),
                    "all",
                    VariableKind::Number,
                    values,
                ));
                let variable_count = self.variables_view_data.variables.len();
                if let Some(collection) =
                    self.variables_view_data
                        .collections
                        .iter_mut()
                        .find(|collection| {
                            collection.id == self.variables_view_data.selected_collection_id
                        })
                {
                    collection.variable_count = variable_count;
                }
                if let Some(group) = self
                    .variables_view_data
                    .groups
                    .iter_mut()
                    .find(|group| group.id.as_ref() == "all")
                {
                    group.variable_count = variable_count;
                }
                self.variables_last_action =
                    format!("Created variable Spacing {ordinal} through the host adapter").into();
            }
            VariablesAction::AddModeRequested => {
                let ordinal = self.variables_view_data.modes.len() + 1;
                let mode_id: SharedString = format!("mode-{ordinal}").into();
                self.variables_view_data.modes.push(VariablesMode::new(
                    mode_id.clone(),
                    format!("Mode {ordinal}"),
                ));
                for variable in &mut self.variables_view_data.variables {
                    let value = match variable.kind {
                        VariableKind::Color => {
                            VariableModeValue::new(mode_id.clone(), "FFFFFF").color("FFFFFF")
                        }
                        VariableKind::Number => VariableModeValue::new(mode_id.clone(), "0"),
                        VariableKind::String => VariableModeValue::new(mode_id.clone(), "Text"),
                        VariableKind::Boolean => VariableModeValue::new(mode_id.clone(), "False"),
                    };
                    variable.values.push(value);
                }
                self.variables_last_action =
                    format!("Added Mode {ordinal} through the host adapter").into();
            }
            VariablesAction::ValueEditRequested {
                variable_id,
                mode_id,
            } => {
                if let Some(variable) = self
                    .variables_view_data
                    .variables
                    .iter_mut()
                    .find(|variable| variable.id == *variable_id)
                {
                    let kind = variable.kind;
                    if let Some(value) = variable
                        .values
                        .iter_mut()
                        .find(|value| value.mode_id == *mode_id)
                    {
                        match kind {
                            VariableKind::Color => {
                                let next = if value.value.as_ref() == "FFFFFF" {
                                    "0D99FF"
                                } else {
                                    "FFFFFF"
                                };
                                value.value = next.into();
                                value.color_hex = Some(next.into());
                            }
                            VariableKind::Number => {
                                value.value = if value.value.as_ref() == "0" {
                                    "8".into()
                                } else {
                                    "0".into()
                                };
                            }
                            VariableKind::String => value.value = "Edited".into(),
                            VariableKind::Boolean => {
                                value.value = if value.value.as_ref() == "True" {
                                    "False".into()
                                } else {
                                    "True".into()
                                };
                            }
                        }
                    }
                }
                self.variables_last_action =
                    format!("Edited {variable_id} in {mode_id} through the host adapter").into();
            }
            VariablesAction::ShareRequested => {
                self.variables_last_action = "Host opened Variables sharing".into();
            }
        }
        page.update(cx, |page, cx| {
            page.set_view_data(self.variables_view_data.clone(), cx);
        });
        cx.notify();
    }

    fn handle_assets_action(
        &mut self,
        panel: Entity<AssetsPanel>,
        action: &AssetsPanelAction,
        cx: &mut Context<Self>,
    ) {
        self.assets_last_action = match action {
            AssetsPanelAction::SearchQueryChanged { query } if query.is_empty() => {
                "Cleared the library search".into()
            }
            AssetsPanelAction::SearchQueryChanged { query } => {
                format!("Filtered libraries by “{query}”").into()
            }
            AssetsPanelAction::LibrarySelected { library_id } => {
                let name = self
                    .assets_view_data
                    .libraries
                    .iter()
                    .find(|library| library.id == *library_id)
                    .map_or(library_id.as_ref(), |library| library.name.as_ref());
                format!("Selected library {name}").into()
            }
            AssetsPanelAction::FiltersRequested => "Host opened Assets filters".into(),
            AssetsPanelAction::AddLibrariesRequested => "Host opened the library browser".into(),
            AssetsPanelAction::RailItemSelected { item } => {
                self.assets_view_data.active_rail_item = *item;
                format!("Host selected the {item:?} editor rail item").into()
            }
        };
        panel.update(cx, |panel, cx| {
            panel.set_view_data(self.assets_view_data.clone(), cx);
        });
        cx.notify();
    }

    fn handle_prototype_action(
        &mut self,
        panel: Entity<PrototypePanel>,
        action: &PrototypePanelAction,
        cx: &mut Context<Self>,
    ) {
        match action {
            PrototypePanelAction::SurfaceChangeRequested { surface } => {
                self.prototype_view_data.surface = *surface;
                self.prototype_last_action = format!("Selected the {surface:?} surface").into();
            }
            PrototypePanelAction::ZoomMenuRequested => {
                self.prototype_view_data.zoom_percent = match self.prototype_view_data.zoom_percent
                {
                    0..=12 => 25,
                    13..=25 => 50,
                    26..=50 => 100,
                    _ => 12,
                };
                self.prototype_last_action = format!(
                    "Host selected {}% prototype zoom",
                    self.prototype_view_data.zoom_percent
                )
                .into();
            }
            PrototypePanelAction::DeviceMenuRequested => {
                self.prototype_view_data.device_name =
                    if self.prototype_view_data.device_name.as_ref() == "No device" {
                        "iPhone 16 Pro".into()
                    } else {
                        "No device".into()
                    };
                self.prototype_last_action = format!(
                    "Host selected prototype device {}",
                    self.prototype_view_data.device_name
                )
                .into();
            }
            PrototypePanelAction::BackgroundEditRequested => {
                self.prototype_view_data.background_hex =
                    if self.prototype_view_data.background_hex.as_ref() == "000000" {
                        "1E1E1E".into()
                    } else {
                        "000000".into()
                    };
                self.prototype_last_action = format!(
                    "Host changed prototype background to #{}",
                    self.prototype_view_data.background_hex
                )
                .into();
            }
            PrototypePanelAction::HintDismissed { hint } => {
                self.prototype_last_action = format!("Dismissed the {hint:?} hint").into();
            }
        }
        panel.update(cx, |panel, cx| {
            panel.set_view_data(self.prototype_view_data.clone(), cx);
        });
        cx.notify();
    }

    fn handle_timeline_action(
        &mut self,
        timeline: Entity<Timeline>,
        action: &TimelineAction,
        cx: &mut Context<Self>,
    ) {
        match action {
            TimelineAction::PlayStateChangeRequested { playing } => {
                self.timeline_view_data.playing = *playing;
                self.timeline_last_action = if *playing {
                    "Host started timeline playback".into()
                } else {
                    "Host paused timeline playback".into()
                };
            }
            TimelineAction::LoopChangeRequested { looping } => {
                self.timeline_view_data.looping = *looping;
                self.timeline_last_action =
                    format!("Host set timeline looping to {looping}").into();
            }
            TimelineAction::AddKeyframeRequested { time_ms } => {
                self.timeline_last_action =
                    format!("Host inserted a keyframe at {time_ms} ms").into();
            }
            TimelineAction::SeekRequested { time_ms } => {
                self.timeline_view_data.current_time_ms =
                    (*time_ms).min(self.timeline_view_data.duration_ms);
                self.timeline_last_action = format!(
                    "Host moved the playhead to {} ms",
                    self.timeline_view_data.current_time_ms
                )
                .into();
            }
            TimelineAction::ZoomChangeRequested { zoom } => {
                self.timeline_view_data.zoom = zoom.clamp(0.1, 2.);
                self.timeline_last_action =
                    format!("Host changed timeline zoom to {:.0}%", zoom * 100.).into();
            }
            TimelineAction::AskAgentRequested => {
                self.timeline_last_action = "Host opened the animation agent".into();
            }
            TimelineAction::EmptyStateDismissed => {
                self.timeline_last_action = "Dismissed the empty timeline guidance".into();
            }
        }
        timeline.update(cx, |timeline, cx| {
            timeline.set_view_data(self.timeline_view_data.clone(), cx);
        });
        cx.notify();
    }

    fn handle_pseudo_editor_action(
        &mut self,
        editor: Entity<PseudoEditor>,
        action: &PseudoEditorAction,
        cx: &mut Context<Self>,
    ) {
        match action {
            PseudoEditorAction::LeftSurfaceChanged { surface } => {
                self.pseudo_left_surface = *surface;
                editor.update(cx, |editor, cx| editor.set_left_surface(*surface, cx));
                self.pseudo_last_action = format!("Selected the {surface:?} left panel").into();
            }
            PseudoEditorAction::RightSurfaceChanged { surface } => {
                self.pseudo_right_surface = *surface;
                editor.update(cx, |editor, cx| editor.set_right_surface(*surface, cx));
                self.pseudo_last_action = format!("Selected the {surface:?} right panel").into();
            }
            PseudoEditorAction::VariablesVisibilityChanged { visible } => {
                self.pseudo_variables_visible = *visible;
                editor.update(cx, |editor, cx| {
                    editor.set_variables_visible(*visible, cx);
                });
                self.pseudo_last_action = if *visible {
                    "Opened the full Variables manager".into()
                } else {
                    "Returned from the Variables manager".into()
                };
            }
            PseudoEditorAction::PresentRequested => {
                self.pseudo_last_action = "Host started prototype presentation".into();
            }
            PseudoEditorAction::ShareRequested => {
                self.pseudo_last_action = "Host opened editor sharing".into();
            }
        }
        cx.notify();
    }

    fn handle_pages_action(
        &mut self,
        panel: Entity<PagesPanel>,
        action: &PagesPanelAction,
        cx: &mut Context<Self>,
    ) {
        match action {
            PagesPanelAction::ExpansionChanged { expanded } => {
                self.pages_last_action = if *expanded {
                    "Host observed: panel expanded".into()
                } else {
                    "Host observed: panel collapsed".into()
                };
            }
            PagesPanelAction::SelectRequested { page_id } => {
                self.active_page = page_id.clone();
                panel.update(cx, |panel, cx| {
                    panel.set_selected_page(Some(page_id.clone()), cx);
                });
                self.pages_last_action = format!("Selected {page_id}").into();
            }
            PagesPanelAction::CreateRequested { title } => {
                let page_id: SharedString = format!("page-{}", self.next_page_id).into();
                self.next_page_id += 1;
                self.pages
                    .push(PagesPanelItem::new(page_id.clone(), title.clone()));
                self.active_page = page_id.clone();
                panel.update(cx, |panel, cx| {
                    panel.set_pages(self.pages.clone(), cx);
                    panel.set_selected_page(Some(page_id.clone()), cx);
                });
                self.pages_last_action = format!("Created {title} through the host adapter").into();
            }
            PagesPanelAction::RenameRequested { page_id, title } => {
                if let Some(page) = self.pages.iter_mut().find(|page| page.id == *page_id) {
                    page.title = title.clone();
                }
                panel.update(cx, |panel, cx| {
                    panel.set_pages(self.pages.clone(), cx);
                });
                self.pages_last_action = format!("Renamed {page_id} to {title}").into();
            }
            PagesPanelAction::DuplicateRequested { page_id } => {
                if let Some((index, source)) = self
                    .pages
                    .iter()
                    .enumerate()
                    .find(|(_, page)| page.id == *page_id)
                    .map(|(index, page)| (index, page.clone()))
                {
                    let duplicate_id: SharedString = format!("page-{}", self.next_page_id).into();
                    self.next_page_id += 1;
                    let duplicate_title: SharedString = format!("{} Copy", source.title).into();
                    self.pages.insert(
                        index + 1,
                        PagesPanelItem::new(duplicate_id.clone(), duplicate_title),
                    );
                    self.active_page = duplicate_id.clone();
                    panel.update(cx, |panel, cx| {
                        panel.set_pages(self.pages.clone(), cx);
                        panel.set_selected_page(Some(duplicate_id.clone()), cx);
                    });
                    self.pages_last_action =
                        format!("Duplicated {page_id} through the host adapter").into();
                }
            }
            PagesPanelAction::DeleteRequested { page_id } => {
                self.pages.retain(|page| page.id != *page_id);
                if self.active_page == *page_id {
                    self.active_page = self
                        .pages
                        .first()
                        .map(|page| page.id.clone())
                        .unwrap_or_else(|| "".into());
                }
                panel.update(cx, |panel, cx| {
                    panel.set_pages(self.pages.clone(), cx);
                    panel.set_selected_page(
                        (!self.active_page.is_empty()).then(|| self.active_page.clone()),
                        cx,
                    );
                });
                self.pages_last_action =
                    format!("Deleted {page_id} through the host adapter").into();
            }
            PagesPanelAction::CopyLinkRequested { page_id } => {
                let link = format!("fanta://pages/{page_id}");
                cx.write_to_clipboard(ClipboardItem::new_string(link.clone()));
                self.pages_last_action = format!("Copied {link}").into();
            }
            PagesPanelAction::SearchRequested(request) => {
                let results = self.search(request);
                panel.update(cx, |panel, cx| {
                    panel.set_search_results(results, cx);
                });
                self.pages_last_action = format!("Host searched for “{}”", request.query).into();
            }
            PagesPanelAction::SearchClosed => {
                self.pages_last_action = "Closed search and returned to Pages".into();
            }
            PagesPanelAction::SearchResultSelected { result_id } => {
                self.pages_last_action = format!("Selected result {result_id}").into();
            }
            PagesPanelAction::NavigateResults {
                direction,
                result_id,
            } => {
                self.pages_last_action = format!("Navigated {direction:?} to {result_id:?}").into();
            }
            PagesPanelAction::ReplaceRequested {
                request,
                result_id,
                replacement,
            } => {
                self.pages_last_action = format!(
                    "Replace “{}” with “{}” in {result_id:?}",
                    request.query, replacement
                )
                .into();
            }
            PagesPanelAction::ReplaceAllRequested {
                request,
                replacement,
            } => {
                self.pages_last_action = format!(
                    "Replace all “{}” with “{}” in {}",
                    request.query,
                    replacement,
                    request.scope.label()
                )
                .into();
            }
        }
        cx.notify();
    }

    fn handle_layers_action(
        &mut self,
        panel: Entity<LayersPanel>,
        action: &LayersPanelAction,
        cx: &mut Context<Self>,
    ) {
        match action {
            LayersPanelAction::PanelExpansionChanged { expanded } => {
                self.layers_last_action = if *expanded {
                    "Host observed: Layers panel expanded".into()
                } else {
                    "Host observed: Layers panel collapsed".into()
                };
            }
            LayersPanelAction::SelectRequested { node_id, mode } => {
                self.apply_layer_selection(node_id, *mode);
                panel.update(cx, |panel, cx| {
                    panel.set_selected_node_ids(self.selected_layers.clone(), cx);
                });
                self.layers_last_action = format!("Selected {node_id} with {mode:?}").into();
            }
            LayersPanelAction::ExpansionChanged { node_id, expanded } => {
                if *expanded {
                    if !self.expanded_layers.contains(node_id) {
                        self.expanded_layers.push(node_id.clone());
                    }
                } else {
                    self.expanded_layers.retain(|id| id != node_id);
                }
                panel.update(cx, |panel, cx| {
                    panel.set_expanded_node_ids(self.expanded_layers.clone(), cx);
                });
                self.layers_last_action = format!(
                    "{} {node_id}",
                    if *expanded { "Expanded" } else { "Collapsed" }
                )
                .into();
            }
            LayersPanelAction::CollapseAllRequested => {
                self.expanded_layers.clear();
                panel.update(cx, |panel, cx| {
                    panel.set_expanded_node_ids(Vec::new(), cx);
                });
                self.layers_last_action = "Collapsed every populated layer".into();
            }
            LayersPanelAction::RenameRequested { node_id, title } => {
                if let Some(node) = find_layer_mut(&mut self.layers, node_id) {
                    node.title = title.clone();
                }
                panel.update(cx, |panel, cx| {
                    panel.set_nodes(self.layers.clone(), cx);
                });
                self.layers_last_action = format!("Renamed {node_id} to {title}").into();
            }
            LayersPanelAction::VisibilityChanged { node_id, visible } => {
                if let Some(node) = find_layer_mut(&mut self.layers, node_id) {
                    node.visible = *visible;
                }
                panel.update(cx, |panel, cx| {
                    panel.set_nodes(self.layers.clone(), cx);
                });
                self.layers_last_action = format!("Set {node_id} visibility to {visible}").into();
            }
            LayersPanelAction::LockChanged { node_id, locked } => {
                if let Some(node) = find_layer_mut(&mut self.layers, node_id) {
                    node.locked = *locked;
                }
                panel.update(cx, |panel, cx| {
                    panel.set_nodes(self.layers.clone(), cx);
                });
                self.layers_last_action = format!("Set {node_id} locked to {locked}").into();
            }
            LayersPanelAction::MoveRequested {
                node_id,
                target_node_id,
                position,
            } => {
                let original_layers = self.layers.clone();
                if let Some(moved) = take_layer(&mut self.layers, node_id)
                    && insert_layer(&mut self.layers, target_node_id, moved, *position)
                {
                    if *position == LayersPanelDropPosition::Inside
                        && !self.expanded_layers.contains(target_node_id)
                    {
                        self.expanded_layers.push(target_node_id.clone());
                    }
                    panel.update(cx, |panel, cx| {
                        panel.set_nodes(self.layers.clone(), cx);
                        panel.set_expanded_node_ids(self.expanded_layers.clone(), cx);
                    });
                    self.layers_last_action =
                        format!("Moved {node_id} {position:?} {target_node_id}").into();
                } else {
                    self.layers = original_layers;
                }
            }
            LayersPanelAction::ContextActionRequested { node_id, action } => {
                self.layers_last_action =
                    format!("Host received {} for {node_id}", action.label()).into();
            }
        }
        cx.notify();
    }

    #[allow(deprecated)]
    fn handle_design_action(
        &mut self,
        panel: Entity<DesignPanel>,
        action: &DesignPanelAction,
        cx: &mut Context<Self>,
    ) {
        if let Some(path) = action.compatibility_path() {
            self.design_last_action = format!(
                "Ignored compatibility-only Design action; use {}",
                path.replacement()
            )
            .into();
            cx.notify();
            return;
        }
        if let DesignPanelAction::SurfaceChangeRequested { current, requested } = action {
            let (active, can_edit) = {
                let panel = panel.read(cx);
                (
                    panel.active_surface(),
                    panel.inspection_context().permissions().can_edit(),
                )
            };
            let mut echoed = active;
            let mut accepted =
                apply_story_surface_change_request(&mut echoed, can_edit, *current, *requested);
            if accepted {
                accepted = panel.update(cx, |panel, cx| panel.set_active_surface(echoed, cx));
            }
            self.design_last_action = if accepted {
                format!(
                    "Host echoed {} → {} without changing the document",
                    current.label(),
                    requested.label()
                )
                .into()
            } else {
                format!(
                    "Host rejected stale or permission-invalid {} → {} request",
                    current.label(),
                    requested.label()
                )
                .into()
            };
            cx.notify();
            return;
        }
        if let DesignPanelAction::MenuPreviewRequested { preview, phase } = action {
            let (current_target, can_edit) = {
                let panel = panel.read(cx);
                (
                    story_design_target(panel.inspection_context()),
                    panel.inspection_context().permissions().can_edit(),
                )
            };
            let current_paint_target = match preview {
                DesignMenuPreview::PaintProperty {
                    node_id,
                    collection,
                    ..
                } => Some(self.design_paint_target_for(node_id, *collection)),
                DesignMenuPreview::NodeProperty { .. }
                | DesignMenuPreview::EffectProperty { .. } => None,
            };
            let accepted = apply_story_menu_preview(
                &mut self.design_menu_preview,
                StoryMenuPreviewContext {
                    nodes: &self.design_nodes,
                    bindings: &self.design_property_bindings,
                    current_target: current_target.as_ref(),
                    current_paint_target,
                    can_edit,
                },
                preview,
                *phase,
            );
            self.design_last_action = if accepted {
                format!("Host accepted {phase:?} for exact menu preview {preview:?}").into()
            } else {
                format!("Host rejected stale or unbalanced menu preview {preview:?}").into()
            };
            cx.notify();
            return;
        }
        if matches!(
            action,
            DesignPanelAction::PropertyChangeRequested { .. }
                | DesignPanelAction::EffectEditRequested {
                    phase: DesignPanelEditPhase::Commit,
                    ..
                }
                | DesignPanelAction::PaintEditRequested {
                    phase: DesignPanelEditPhase::Commit,
                    ..
                }
        ) {
            self.design_menu_preview = None;
        }
        if let Some((target, node_action)) = action.targeted_node_action() {
            let (current_target, can_edit) = {
                let panel = panel.read(cx);
                (
                    story_design_target(panel.inspection_context()),
                    panel.inspection_context().permissions().can_edit(),
                )
            };
            if let DesignPanelAction::PaintEditRequested {
                collection,
                target: paint_target,
                paint_id,
                index,
                edit,
                phase,
                ..
            } = node_action
            {
                let accepted = apply_story_targeted_common_paint_edit(
                    &mut self.design_nodes,
                    &mut self.design_paint_edit_snapshots,
                    current_target.as_ref(),
                    target,
                    can_edit,
                    *collection,
                    *paint_target,
                    paint_id,
                    *index,
                    edit,
                    *phase,
                );
                self.design_last_action = if accepted {
                    format!(
                        "Host atomically applied {phase:?} for {:?} across the common {} collection on {target:?}",
                        edit.property,
                        collection.label(),
                    )
                    .into()
                } else {
                    format!(
                        "Host rejected stale, reordered, or non-common {} paint edit for {target:?}",
                        collection.label(),
                    )
                    .into()
                };
                if accepted {
                    self.apply_design_inspection_context(panel, cx);
                }
                cx.notify();
                return;
            }
            if let DesignPanelAction::EffectAddRequested { kind, .. } = node_action {
                let accepted = apply_story_targeted_effect_add(
                    &mut self.design_nodes,
                    current_target.as_ref(),
                    target,
                    can_edit,
                    *kind,
                );
                self.design_last_action = if accepted {
                    format!(
                        "Host atomically added {} to the exact target {target:?}",
                        kind.label()
                    )
                    .into()
                } else {
                    format!(
                        "Host rejected stale, permission-invalid, or partially unavailable {} for {target:?}",
                        kind.label()
                    )
                    .into()
                };
                if accepted {
                    self.apply_design_inspection_context(panel, cx);
                }
                cx.notify();
                return;
            }
            let Some(retargeted_actions) = story_targeted_node_actions(
                &self.design_nodes,
                current_target.as_ref(),
                target,
                can_edit,
                node_action,
            ) else {
                self.design_last_action =
                    format!("Ignored invalid or stale targeted Design action for {target:?}")
                        .into();
                cx.notify();
                return;
            };
            for retargeted_action in &retargeted_actions {
                self.handle_design_action(panel.clone(), retargeted_action, cx);
            }
            self.design_last_action =
                format!("Host replayed one fully preflighted Design action across {target:?}")
                    .into();
            cx.notify();
            return;
        }
        match action {
            DesignPanelAction::SmartSelectionSpacingEditRequested {
                target,
                axis,
                value,
                phase,
            } => {
                let (current_target, can_edit) = {
                    let panel = panel.read(cx);
                    (
                        story_design_target(panel.inspection_context()),
                        panel.inspection_context().permissions().can_edit(),
                    )
                };
                let projection = current_target.and_then(|current_target| {
                    self.design_smart_selection_view_data(current_target)
                });
                let accepted = apply_story_smart_selection_spacing_edit(
                    &mut self.design_smart_selection_spacing,
                    &mut self.design_smart_selection_edit_snapshots,
                    projection.as_ref(),
                    can_edit,
                    StorySmartSelectionSpacingEdit {
                        target,
                        axis: *axis,
                        value: *value,
                        phase: *phase,
                    },
                );
                if accepted
                    && matches!(
                        phase,
                        DesignPanelEditPhase::Commit | DesignPanelEditPhase::Cancel
                    )
                {
                    self.apply_design_inspection_context(panel.clone(), cx);
                }
                self.design_last_action = if accepted {
                    format!(
                        "Host accepted {phase:?} for {} = {value} on exact target {target:?}",
                        axis.label()
                    )
                    .into()
                } else {
                    format!(
                        "Host rejected stale, read-only, invalid, or out-of-order {} edit for {target:?}",
                        axis.label()
                    )
                    .into()
                };
                cx.notify();
                return;
            }
            DesignPanelAction::SmartSelectionArrangeRequested { target, operation } => {
                let (current_target, can_edit) = {
                    let panel = panel.read(cx);
                    (
                        story_design_target(panel.inspection_context()),
                        panel.inspection_context().permissions().can_edit(),
                    )
                };
                let projection = current_target.and_then(|current_target| {
                    self.design_smart_selection_view_data(current_target)
                });
                let accepted = story_smart_selection_operation_is_current(
                    projection.as_ref(),
                    can_edit,
                    target,
                    *operation,
                );
                self.design_last_action = if accepted {
                    format!(
                        "Host applied {} to exact Smart Selection target {target:?}",
                        operation.label()
                    )
                    .into()
                } else {
                    format!(
                        "Host rejected stale or unavailable {} for {target:?}",
                        operation.label()
                    )
                    .into()
                };
                cx.notify();
                return;
            }
            _ => {}
        }
        if let DesignPanelAction::AddAutoLayoutRequested { target } = action {
            let (current_target, can_edit, structurally_eligible) = {
                let panel = panel.read(cx);
                let context = panel.inspection_context();
                let structurally_eligible = match self.design_inspection_scenario {
                    DesignInspectionScenario::AddAutoLayoutGroup => {
                        context.selection().kind()
                            == fanta_gpui::prelude::DesignPanelSelectionKind::Single
                            && context
                                .selection()
                                .items()
                                .first()
                                .is_some_and(|node| node.kind == DesignPanelNodeKind::Group)
                    }
                    DesignInspectionScenario::AddAutoLayoutMultiple => {
                        context.selection().kind()
                            == fanta_gpui::prelude::DesignPanelSelectionKind::Multiple
                    }
                    _ => false,
                };
                (
                    story_design_target(context),
                    context.permissions().can_edit(),
                    structurally_eligible,
                )
            };
            let echo = apply_story_add_auto_layout(
                &mut self.design_nodes,
                target,
                current_target.as_ref(),
                can_edit && structurally_eligible,
                &mut self.next_design_auto_layout_id,
            );
            if let Some((selected_index, echo)) = echo {
                self.selected_design_node = selected_index;
                self.design_inspection_scenario = DesignInspectionScenario::EditableSingle;
                self.apply_design_inspection_context(panel, cx);
                self.design_last_action = match echo {
                    StoryAddAutoLayoutEcho::Converted { node_id } => {
                        format!("Host converted {node_id} to an auto-layout frame").into()
                    }
                    StoryAddAutoLayoutEcho::Wrapped {
                        wrapper_id,
                        child_ids,
                    } => format!("Host wrapped ordered selection {child_ids:?} in {wrapper_id}")
                        .into(),
                };
            } else {
                self.design_last_action =
                    format!("Host rejected stale or ineligible Add auto layout target {target:?}")
                        .into();
            }
            cx.notify();
            return;
        }
        if let DesignPanelAction::PropertyCopyRequested {
            target,
            property,
            displayed_value,
        } = action
        {
            self.design_last_action =
                story_property_copy_status(target, *property, displayed_value);
            cx.notify();
            return;
        }
        if let DesignPanelAction::DimensionLimitsPreviewRequested {
            node_id,
            axis,
            minimum,
            maximum,
            preview,
        } = action
        {
            let node = self.design_nodes.iter().find(|node| node.id == *node_id);
            let current = node
                .and_then(|node| node.layout.as_ref())
                .map(|layout| match axis {
                    fanta_gpui::prelude::DesignLayoutDimensionAxis::Width => {
                        (layout.item.min_width, layout.item.max_width)
                    }
                    fanta_gpui::prelude::DesignLayoutDimensionAxis::Height => {
                        (layout.item.min_height, layout.item.max_height)
                    }
                });
            let accepted = if *preview {
                current == Some((*minimum, *maximum))
            } else {
                node.is_some()
            };
            self.design_last_action = if accepted {
                format!(
                    "Host {} {:?} limits {:?}…{:?} on {node_id}",
                    if *preview { "previewed" } else { "cleared" },
                    axis,
                    minimum,
                    maximum
                )
                .into()
            } else {
                format!("Host rejected stale {axis:?} limit preview on {node_id}").into()
            };
            cx.notify();
            return;
        }
        if let DesignPanelAction::ViewerSectionCopyRequested {
            target,
            section_id,
            copy_value,
        } = action
        {
            let can_copy = self.design_inspection_context().0.permissions().can_copy();
            let current = match target {
                DesignPanelTarget::Nodes { node_ids } if node_ids.len() == 1 => {
                    self.design_viewer_properties
                        .get(&node_ids[0])
                        .filter(|view_data| &view_data.target == target)
                        .and_then(|view_data| view_data.section(section_id.as_ref()))
                        .and_then(|section| section.copy_value.as_ref())
                        == Some(copy_value)
                }
                DesignPanelTarget::Page { .. } | DesignPanelTarget::Nodes { .. } => false,
            };
            self.design_last_action = if can_copy && current {
                format!("Host copied viewer section {section_id} from {target:?}: “{copy_value}”")
                    .into()
            } else {
                format!("Host rejected stale or restricted viewer copy for {section_id}").into()
            };
            cx.notify();
            return;
        }
        if let DesignPanelAction::ViewerSectionRepresentationChangeRequested {
            target,
            section_id,
            representation,
        } = action
        {
            let can_copy = self.design_inspection_context().0.permissions().can_copy();
            let node_id = match target {
                DesignPanelTarget::Nodes { node_ids } if node_ids.len() == 1 => {
                    Some(node_ids[0].clone())
                }
                DesignPanelTarget::Page { .. } | DesignPanelTarget::Nodes { .. } => None,
            };
            let accepted = node_id.as_ref().is_some_and(|node_id| {
                can_copy
                    && self
                        .design_viewer_properties
                        .get(node_id)
                        .is_some_and(|view_data| {
                            &view_data.target == target
                                && view_data
                                    .section(section_id.as_ref())
                                    .is_some_and(|section| {
                                        section.color_representation.is_some()
                                            && section.color_representation != Some(*representation)
                                    })
                        })
            });
            if accepted
                && let Some(node_id) = node_id
                && let Some(node) = self
                    .design_nodes
                    .iter()
                    .find(|node| node.id == node_id)
                    .cloned()
            {
                self.design_viewer_properties.insert(
                    node_id,
                    Self::viewer_properties_for_node(&node, *representation),
                );
                self.apply_design_inspection_context(panel, cx);
                self.design_last_action = format!(
                    "Host represented viewer section {section_id} as {}",
                    representation.label()
                )
                .into();
            } else {
                self.design_last_action =
                    format!("Host rejected stale viewer representation for {section_id}").into();
            }
            cx.notify();
            return;
        }
        let (current_page_projection, legacy_variables_entry_is_enabled) = {
            let panel = panel.read(cx);
            let page_matches = story_page_local_styles_projection_is_current(
                panel.inspection_context().selection().kind()
                    == fanta_gpui::prelude::DesignPanelSelectionKind::None,
                panel.page_view_data().map(|page| &page.page_id),
                panel.page_local_styles_view_data(),
                &self.design_page_view_data.page_id,
                &self.design_page_local_styles,
            );
            let legacy_variables_entry_is_enabled = matches!(
                panel.variables_entry_point(),
                DesignVariablesEntryPoint::LegacyRightSidebar {
                    disabled_reason: None
                }
            );
            (page_matches, legacy_variables_entry_is_enabled)
        };
        let page_or_mode_handled = match action {
            DesignPanelAction::PageBackgroundEditRequested {
                page_id,
                color,
                phase,
            } => {
                if self.design_page_view_data.page_id == *page_id {
                    apply_story_page_background_edit_phase(
                        &mut self.design_page_view_data.background.color,
                        &mut self.design_page_background_edit_snapshots,
                        page_id,
                        *color,
                        *phase,
                    );
                    self.design_last_action =
                        format!("Host {:?} Page background #{}", phase, color.hex()).into();
                } else {
                    self.design_page_background_edit_snapshots.remove(page_id);
                    self.design_last_action =
                        format!("Ignored stale Page background edit for {page_id}").into();
                }
                true
            }
            DesignPanelAction::PageBackgroundChangeRequested { page_id, color } => {
                if self.design_page_view_data.page_id == *page_id {
                    self.design_page_view_data.background.color = *color;
                    self.design_last_action =
                        format!("Host changed Page background to #{}", color.hex()).into();
                } else {
                    self.design_last_action =
                        format!("Ignored stale Page background intent for {page_id}").into();
                }
                true
            }
            DesignPanelAction::FramePresetApplyRequested {
                node_id,
                selection,
                width,
                height,
            } => {
                let applied = apply_story_frame_preset(
                    &mut self.design_nodes,
                    &self.design_frame_presets,
                    node_id,
                    selection,
                    *width,
                    *height,
                );
                self.design_last_action = if applied {
                    format!(
                        "Host applied Frame preset {}/{} at {} × {}",
                        selection.group_id, selection.preset_id, width, height
                    )
                    .into()
                } else {
                    format!(
                        "Ignored stale or unavailable Frame preset {}/{}",
                        selection.group_id, selection.preset_id
                    )
                    .into()
                };
                true
            }
            DesignPanelAction::LocalResourceBrowseRequested { page_id, category } => {
                self.design_last_action = if self.design_page_view_data.page_id == *page_id {
                    format!("Host opened the {} browser", category.label()).into()
                } else {
                    format!("Ignored stale resource browser intent for {page_id}").into()
                };
                true
            }
            DesignPanelAction::LocalResourceOpenRequested { page_id, resource } => {
                let opened = self.design_page_view_data.page_id == *page_id
                    && self
                        .design_page_view_data
                        .local_resources
                        .resource(resource)
                        .is_some_and(|item| item.availability.can_open());
                self.design_last_action = if opened {
                    format!(
                        "Host opened Page resource {} from group {}",
                        resource.resource_id, resource.group_id
                    )
                    .into()
                } else {
                    format!(
                        "Ignored stale or non-local resource {}",
                        resource.resource_id
                    )
                    .into()
                };
                true
            }
            DesignPanelAction::LocalResourceCreateRequested { page_id, kind } => {
                let accepted = if self.design_page_view_data.page_id == *page_id {
                    let group_id = match kind.category() {
                        DesignLocalResourceCategory::Styles => "page-local-styles",
                        DesignLocalResourceCategory::VariableCollections => "page-local-variables",
                    };
                    if let Some(group) = self
                        .design_page_view_data
                        .local_resources
                        .groups
                        .iter_mut()
                        .find(|group| group.id.as_ref() == group_id)
                    {
                        let resource_id = SharedString::from(format!(
                            "storybook-resource-{}",
                            self.next_design_resource_id
                        ));
                        self.next_design_resource_id += 1;
                        group.resources.push(DesignLocalResource::local(
                            resource_id,
                            format!("New {}", kind.label()),
                            *kind,
                        ));
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                self.design_last_action = if accepted {
                    format!("Host created a local {}", kind.label()).into()
                } else {
                    format!("Ignored stale {} creation intent", kind.label()).into()
                };
                true
            }
            DesignPanelAction::LocalResourceImportRequested { page_id, resource } => {
                let imported = if self.design_page_view_data.page_id == *page_id {
                    self.design_page_view_data
                        .local_resources
                        .groups
                        .iter_mut()
                        .find(|group| {
                            group.id == resource.group_id && group.source == resource.source
                        })
                        .and_then(|group| {
                            group
                                .resources
                                .iter_mut()
                                .find(|item| item.id == resource.resource_id)
                        })
                        .filter(|item| item.availability.can_import())
                        .map(|item| {
                            item.availability = DesignLocalResourceAvailability::Imported;
                        })
                        .is_some()
                } else {
                    false
                };
                self.design_last_action = if imported {
                    format!("Host imported Page resource {}", resource.resource_id).into()
                } else {
                    format!("Ignored unavailable resource {}", resource.resource_id).into()
                };
                true
            }
            DesignPanelAction::LocalStyleCommandRequested { target, command } => {
                let permissions = panel.read(cx).inspection_context().permissions();
                let permission_allows = match command {
                    DesignLocalStyleCommand::Edit | DesignLocalStyleCommand::Duplicate => {
                        permissions.can_edit()
                    }
                    DesignLocalStyleCommand::Copy => permissions.can_copy(),
                    DesignLocalStyleCommand::GoToDefinition => true,
                };
                let mut accepted = current_page_projection
                    && permission_allows
                    && story_local_style_targets_are_current(
                        &self.design_page_local_styles,
                        std::slice::from_ref(target),
                        false,
                    );
                if accepted && *command == DesignLocalStyleCommand::Duplicate {
                    accepted = story_duplicate_local_style(
                        &mut self.design_page_local_styles,
                        target,
                        self.next_design_resource_id,
                    );
                    if accepted {
                        self.next_design_resource_id += 1;
                    }
                }
                self.design_last_action = if accepted {
                    format!("Host ran {} for {}", command.label(), target.style_id).into()
                } else {
                    format!(
                        "Ignored stale or permission-invalid {} for {}",
                        command.label(),
                        target.style_id
                    )
                    .into()
                };
                true
            }
            DesignPanelAction::LocalStyleCreateRequested {
                page_id,
                kind,
                parent_folder_id,
            } => {
                let can_edit = panel.read(cx).inspection_context().permissions().can_edit();
                let accepted = current_page_projection
                    && can_edit
                    && story_create_local_style(
                        &mut self.design_page_local_styles,
                        page_id,
                        *kind,
                        parent_folder_id.as_ref(),
                        self.next_design_resource_id,
                    );
                if accepted {
                    self.next_design_resource_id += 1;
                }
                self.design_last_action = if accepted {
                    format!("Host created a current-file {}", kind.label()).into()
                } else {
                    format!("Ignored stale {} creation", kind.label()).into()
                };
                true
            }
            DesignPanelAction::LocalStyleFolderCreateRequested {
                page_id,
                kind,
                selected,
            } => {
                let can_edit = panel.read(cx).inspection_context().permissions().can_edit();
                let accepted = current_page_projection
                    && can_edit
                    && story_create_local_style_folder(
                        &mut self.design_page_local_styles,
                        page_id,
                        *kind,
                        selected,
                        self.next_design_resource_id,
                    );
                if accepted {
                    self.next_design_resource_id += 1;
                }
                self.design_last_action = if accepted {
                    format!(
                        "Host created a {} folder around {} styles",
                        kind.label(),
                        selected.len()
                    )
                    .into()
                } else {
                    format!("Ignored stale {} folder creation", kind.label()).into()
                };
                true
            }
            DesignPanelAction::LocalStylesDeleteRequested { targets } => {
                let can_edit = panel.read(cx).inspection_context().permissions().can_edit();
                let accepted = current_page_projection
                    && can_edit
                    && story_remove_local_style_targets(
                        &mut self.design_page_local_styles,
                        targets,
                    )
                    .is_some();
                self.design_last_action = if accepted {
                    format!("Host deleted {} exact local styles", targets.len()).into()
                } else {
                    "Ignored stale local-style deletion".into()
                };
                true
            }
            DesignPanelAction::LocalStylesMoveRequested {
                targets,
                destination,
            } => {
                let can_edit = panel.read(cx).inspection_context().permissions().can_edit();
                let accepted = current_page_projection
                    && can_edit
                    && story_move_local_styles(
                        &mut self.design_page_local_styles,
                        targets,
                        destination,
                    );
                self.design_last_action = if accepted {
                    format!(
                        "Host moved {} local styles to index {}",
                        targets.len(),
                        destination.expected_index
                    )
                    .into()
                } else {
                    "Ignored stale local-style move".into()
                };
                true
            }
            DesignPanelAction::VariablesViewOpenRequested { page_id } => {
                self.design_last_action = if current_page_projection
                    && legacy_variables_entry_is_enabled
                    && self.design_page_view_data.page_id == *page_id
                {
                    "Host opened Variables from the legacy right-sidebar entry".into()
                } else {
                    format!("Ignored stale Variables navigation for {page_id}").into()
                };
                true
            }
            DesignPanelAction::VariableModeApplyRequested {
                target,
                collection_id,
                mode_id,
            } => {
                let key = match target {
                    DesignPanelTarget::Page { page_id } => Some(page_id.clone()),
                    DesignPanelTarget::Nodes { node_ids } if node_ids.len() == 1 => {
                        node_ids.first().cloned()
                    }
                    DesignPanelTarget::Nodes { .. } => None,
                };
                let applied = key
                    .and_then(|key| self.design_variable_mode_views.get_mut(&key))
                    .filter(|view_data| view_data.target == *target)
                    .and_then(|view_data| {
                        view_data
                            .collections
                            .iter_mut()
                            .find(|collection| collection.id == *collection_id)
                    })
                    .filter(|collection| {
                        collection
                            .mode(mode_id.as_ref())
                            .is_some_and(|mode| mode.disabled_reason.is_none())
                            && collection.disabled_reason.is_none()
                    })
                    .map(|collection| {
                        collection.resolved_mode_id = mode_id.clone();
                        collection.explicit_mode_id = Some(mode_id.clone());
                    })
                    .is_some();
                self.design_last_action = if applied {
                    format!("Host set explicit mode {mode_id} for collection {collection_id}")
                        .into()
                } else {
                    format!("Ignored stale variable mode {mode_id}").into()
                };
                true
            }
            DesignPanelAction::VariableModeClearRequested {
                target,
                collection_id,
                explicit_mode_id,
            } => {
                let key = match target {
                    DesignPanelTarget::Page { page_id } => Some(page_id.clone()),
                    DesignPanelTarget::Nodes { node_ids } if node_ids.len() == 1 => {
                        node_ids.first().cloned()
                    }
                    DesignPanelTarget::Nodes { .. } => None,
                };
                let cleared = key
                    .and_then(|key| self.design_variable_mode_views.get_mut(&key))
                    .filter(|view_data| view_data.target == *target)
                    .and_then(|view_data| {
                        view_data
                            .collections
                            .iter_mut()
                            .find(|collection| collection.id == *collection_id)
                    })
                    .filter(|collection| {
                        collection.explicit_mode_id.as_ref() == Some(explicit_mode_id)
                            && collection.disabled_reason.is_none()
                    })
                    .map(|collection| {
                        collection.explicit_mode_id = None;
                        collection.resolved_mode_id = collection.default_mode_id.clone();
                    })
                    .is_some();
                self.design_last_action = if cleared {
                    format!("Host cleared explicit mode {explicit_mode_id} for {collection_id}")
                        .into()
                } else {
                    format!("Ignored stale explicit mode {explicit_mode_id}").into()
                };
                true
            }
            _ => false,
        };
        if page_or_mode_handled {
            self.apply_design_inspection_context(panel, cx);
            cx.notify();
            return;
        }

        let (current_export_target, can_export) = {
            let panel = panel.read(cx);
            (
                story_export_target(panel.inspection_context()),
                panel.inspection_context().permissions().can_export(),
            )
        };
        let export_handled = match action {
            DesignPanelAction::ExportConfigurationAddRequested { target } => {
                let configuration_id =
                    SharedString::from(format!("storybook-export-{}", self.next_design_export_id));
                let capabilities = match target {
                    DesignPanelTarget::Page { .. } => DesignStaticExportCapabilities::default(),
                    DesignPanelTarget::Nodes { node_ids } => {
                        story_aggregate_static_export_capabilities(node_ids.iter().filter_map(
                            |node_id| self.design_nodes.iter().find(|node| node.id == *node_id),
                        ))
                    }
                };
                let accepted = apply_story_export_configuration_add(
                    &self.design_nodes,
                    &mut self.design_export_configurations,
                    &current_export_target,
                    target,
                    can_export,
                    DesignExportConfiguration::new_for_capabilities(
                        configuration_id.clone(),
                        DesignExportFormat::Png,
                        capabilities,
                    ),
                );
                if accepted {
                    self.next_design_export_id += 1;
                }
                self.design_last_action = if accepted {
                    format!("Host added export configuration {configuration_id} to {target:?}")
                        .into()
                } else {
                    format!("Ignored stale or mixed export-add target {target:?}").into()
                };
                true
            }
            DesignPanelAction::ExportConfigurationRemoveRequested {
                target,
                configuration_id,
            } => {
                let accepted = apply_story_export_configuration_remove(
                    &self.design_nodes,
                    &mut self.design_export_configurations,
                    &current_export_target,
                    target,
                    can_export,
                    configuration_id,
                );
                self.design_last_action = if accepted {
                    format!("Host removed export configuration {configuration_id} from {target:?}")
                        .into()
                } else {
                    format!(
                        "Ignored stale or mixed export-remove {configuration_id} for {target:?}"
                    )
                    .into()
                };
                true
            }
            DesignPanelAction::ExportConfigurationChangeRequested {
                target,
                configuration_id,
                change,
                phase,
            } => {
                let accepted = apply_story_export_configuration_edit(
                    &self.design_nodes,
                    &mut self.design_export_configurations,
                    &mut self.design_export_edit_snapshots,
                    StoryExportConfigurationEdit {
                        current_target: &current_export_target,
                        requested_target: target,
                        can_export,
                        configuration_id,
                        change,
                        phase: *phase,
                    },
                );
                self.design_last_action = if accepted {
                    format!(
                        "Host observed {phase:?} for export {configuration_id} on exact {target:?}: {change:?}"
                    )
                    .into()
                } else {
                    format!(
                        "Ignored stale or mixed {phase:?} export {configuration_id} for {target:?}"
                    )
                    .into()
                };
                true
            }
            DesignPanelAction::ExportModeChangeRequested { target, mode } => {
                let target_keys = story_export_target_keys(
                    &self.design_nodes,
                    &current_export_target,
                    target,
                    can_export,
                );
                let accepted = target_keys
                    .as_ref()
                    .filter(|keys| keys.len() == 1)
                    .and_then(|keys| keys.first())
                    .filter(|key| self.design_animated_exports.contains_key(*key))
                    .cloned()
                    .map(|key| self.design_export_modes.insert(key, *mode))
                    .is_some();
                self.design_last_action = if accepted {
                    format!("Host switched export mode to {}", mode.label()).into()
                } else {
                    format!("Ignored stale or multi-target export mode for {target:?}").into()
                };
                true
            }
            DesignPanelAction::AnimatedExportChangeRequested {
                target,
                change,
                phase,
            } => {
                let key = story_export_target_keys(
                    &self.design_nodes,
                    &current_export_target,
                    target,
                    can_export,
                )
                .filter(|keys| keys.len() == 1)
                .and_then(|keys| keys.into_iter().next());
                let accepted = key.is_some_and(|key| match phase {
                    DesignPanelEditPhase::Begin => {
                        let Some(original) = self.design_animated_exports.get(&key).cloned() else {
                            return false;
                        };
                        self.design_animated_export_edit_snapshots
                            .entry(key)
                            .or_insert(Some(original));
                        true
                    }
                    DesignPanelEditPhase::Preview | DesignPanelEditPhase::Commit => {
                        let Some(animated) = self.design_animated_exports.get_mut(&key) else {
                            return false;
                        };
                        let changed = if *change
                            == DesignAnimatedExportChange::Format(DesignAnimatedExportFormat::Svg)
                        {
                            animated.settings = DesignAnimatedExportSettings::Svg {
                                options: vec![
                                    DesignAnimatedSvgOption::new(
                                        "precision",
                                        "Precision",
                                        "Balanced",
                                        ["Compact", "Balanced", "Exact"],
                                    ),
                                    DesignAnimatedSvgOption::new(
                                        "loop",
                                        "Loop",
                                        "Forever",
                                        ["Once", "Forever"],
                                    ),
                                ],
                            };
                            true
                        } else {
                            animated.settings.apply_change(change.clone())
                        };
                        if changed && *phase == DesignPanelEditPhase::Commit {
                            self.design_animated_export_edit_snapshots.remove(&key);
                        }
                        changed
                    }
                    DesignPanelEditPhase::Cancel => {
                        let Some(original) =
                            self.design_animated_export_edit_snapshots.remove(&key)
                        else {
                            return false;
                        };
                        if let Some(animated) = original {
                            self.design_animated_exports.insert(key, animated);
                        } else {
                            self.design_animated_exports.remove(&key);
                        }
                        true
                    }
                });
                self.design_last_action = if accepted {
                    format!("Host observed {phase:?} animated export change: {change:?}").into()
                } else {
                    format!("Ignored stale or multi-target animated export for {target:?}").into()
                };
                true
            }
            DesignPanelAction::AnimatedExportRequested { target, settings } => {
                let accepted = story_export_target_keys(
                    &self.design_nodes,
                    &current_export_target,
                    target,
                    can_export,
                )
                .filter(|keys| keys.len() == 1)
                .and_then(|keys| keys.into_iter().next())
                .and_then(|key| self.design_animated_exports.get(&key))
                .is_some_and(|animated| {
                    animated.settings == *settings && animated.capability.allows(settings)
                });
                self.design_last_action = if accepted {
                    format!(
                        "Host exported {target:?} as animated {}",
                        settings.format().label()
                    )
                    .into()
                } else {
                    format!("Ignored stale or multi-target animated export for {target:?}").into()
                };
                true
            }
            DesignPanelAction::ExportAllRequested { target } => {
                let target_keys = story_export_target_keys(
                    &self.design_nodes,
                    &current_export_target,
                    target,
                    can_export,
                );
                let accepted = target_keys.as_ref().is_some_and(|target_keys| {
                    story_uniform_export_configurations(
                        &self.design_export_configurations,
                        target_keys,
                    )
                    .is_some_and(|configurations| !configurations.is_empty())
                });
                self.design_last_action = if accepted {
                    format!("Host atomically exported every configured format for {target:?}")
                        .into()
                } else {
                    format!("Ignored stale, empty, or mixed export target {target:?}").into()
                };
                true
            }
            DesignPanelAction::ExportPreviewRequested { target } => {
                let key = story_export_target_keys(
                    &self.design_nodes,
                    &current_export_target,
                    target,
                    can_export,
                )
                .filter(|keys| keys.len() == 1)
                .and_then(|keys| keys.into_iter().next());
                let accepted = key
                    .map(|key| {
                        self.design_export_previews.insert(
                            key,
                            DesignExportPreviewState::Ready(
                                DesignExportPreview::new(1440, 900)
                                    .with_thumbnail("storybook-requested-preview")
                                    .with_estimated_output("824 KB"),
                            ),
                        );
                    })
                    .is_some();
                self.design_last_action = if accepted {
                    format!("Host prepared an export preview for {target:?}").into()
                } else {
                    format!("Ignored stale or multi-target export preview for {target:?}").into()
                };
                true
            }
            _ => false,
        };
        if export_handled {
            self.apply_design_inspection_context(panel, cx);
            cx.notify();
            return;
        }

        if let DesignPanelAction::SelectionHeaderCommandRequested { target, command } = action {
            self.design_last_action =
                format!("Host received selected-node command {command:?} for {target:?}").into();
            self.apply_design_inspection_context(panel, cx);
            cx.notify();
            return;
        }

        if let DesignPanelAction::SelectionColorOccurrencesSelectRequested {
            target,
            selection_color_id,
            paint_references,
        } = action
        {
            if story_selection_row_is_current(
                &self.design_nodes,
                target,
                selection_color_id,
                paint_references,
            ) {
                self.design_last_action = format!(
                    "Host selected all {} occurrences of selection paint {selection_color_id} for {target:?}",
                    paint_references.len()
                )
                .into();
            } else {
                self.design_last_action =
                    format!("Ignored stale selection-paint occurrences for {target:?}").into();
            }
            self.apply_design_inspection_context(panel, cx);
            cx.notify();
            return;
        }

        if let DesignPanelAction::SelectionColorPaintEditRequested {
            target,
            selection_color_id,
            paint_references,
            edit,
            phase,
        } = action
        {
            let edit_target =
                StorySelectionColorEditTarget::new(target, selection_color_id.clone());
            let references_are_current = story_selection_row_is_current(
                &self.design_nodes,
                target,
                selection_color_id,
                paint_references,
            );
            match phase {
                DesignPanelEditPhase::Begin if references_are_current => {
                    self.design_selection_color_edit_snapshots
                        .entry(edit_target.clone())
                        .or_insert_with(|| self.design_nodes.clone());
                }
                DesignPanelEditPhase::Preview | DesignPanelEditPhase::Commit
                    if references_are_current =>
                {
                    let mut candidate_nodes = self.design_nodes.clone();
                    let applied = paint_references.iter().all(|reference| {
                        candidate_nodes
                            .iter_mut()
                            .find(|node| node.id == reference.node_id)
                            .and_then(|node| story_selection_reference_paint_mut(node, reference))
                            .is_some_and(|paint| {
                                let Some(edit) =
                                    story_selection_paint_edit_for_occurrence(paint, edit)
                                else {
                                    return false;
                                };
                                paint.apply_edit(&edit)
                            })
                    });
                    if applied {
                        self.design_nodes = candidate_nodes;
                    }
                }
                DesignPanelEditPhase::Cancel => {
                    if let Some(original) = self
                        .design_selection_color_edit_snapshots
                        .remove(&edit_target)
                    {
                        self.design_nodes = original;
                    }
                }
                DesignPanelEditPhase::Begin
                | DesignPanelEditPhase::Preview
                | DesignPanelEditPhase::Commit => {}
            }
            if *phase == DesignPanelEditPhase::Commit {
                self.design_selection_color_edit_snapshots
                    .remove(&edit_target);
            }
            self.design_last_action = if references_are_current {
                format!(
                    "Host observed {phase:?} selection-paint {:?} for {selection_color_id} on {target:?}",
                    edit.property
                )
                .into()
            } else {
                format!("Ignored stale selection-paint edit for {target:?}").into()
            };
            self.apply_design_inspection_context(panel, cx);
            cx.notify();
            return;
        }

        if let DesignPanelAction::SelectionColorEditRequested {
            target,
            selection_color_id,
            color,
            paint_references,
            phase,
        } = action
        {
            let edit_target =
                StorySelectionColorEditTarget::new(target, selection_color_id.clone());
            match phase {
                DesignPanelEditPhase::Begin => {
                    let original = self.design_nodes.clone();
                    self.design_selection_color_edit_snapshots
                        .entry(edit_target.clone())
                        .or_insert(original);
                }
                DesignPanelEditPhase::Cancel => {
                    if let Some(original) = self
                        .design_selection_color_edit_snapshots
                        .remove(&edit_target)
                    {
                        self.design_nodes = original;
                    }
                }
                DesignPanelEditPhase::Preview | DesignPanelEditPhase::Commit => {}
            }
            if matches!(
                phase,
                DesignPanelEditPhase::Preview | DesignPanelEditPhase::Commit
            ) {
                for node in &mut self.design_nodes {
                    if let Some((index, selection_color)) = node
                        .selection_color_aggregate
                        .colors
                        .iter_mut()
                        .enumerate()
                        .find(|(_, selection_color)| selection_color.id == *selection_color_id)
                    {
                        selection_color.color = *color;
                        if let Some(legacy_paint) = node.selection_colors.get_mut(index) {
                            legacy_paint.apply_edit(&DesignPaintEdit {
                                property: DesignPaintProperty::Color,
                                value: DesignPaintValue::Color(*color),
                            });
                        }
                    }
                }

                for reference in paint_references {
                    let Some(node) = self
                        .design_nodes
                        .iter_mut()
                        .find(|node| node.id == reference.node_id)
                    else {
                        continue;
                    };
                    let paints = match reference.collection {
                        DesignSelectionPaintCollection::Fill => Some(&mut node.fills),
                        DesignSelectionPaintCollection::Stroke => {
                            node.stroke.as_mut().map(|stroke| &mut stroke.paints)
                        }
                    };
                    let Some(paints) = paints else {
                        continue;
                    };
                    let paint_index = if reference.paint_id.is_empty() {
                        (reference.paint_index < paints.len()).then_some(reference.paint_index)
                    } else {
                        paints
                            .iter()
                            .position(|paint| paint.id == reference.paint_id)
                    };
                    let Some(paint) = paint_index.and_then(|index| paints.get_mut(index)) else {
                        continue;
                    };
                    let property = reference.gradient_stop_id.as_ref().map_or(
                        DesignPaintProperty::Color,
                        |stop_id| DesignPaintProperty::GradientStopColor {
                            stop_id: stop_id.clone(),
                            index: reference.gradient_stop_index.unwrap_or_default(),
                        },
                    );
                    paint.apply_edit(&DesignPaintEdit {
                        property,
                        value: DesignPaintValue::Color(*color),
                    });
                }
            }
            if *phase == DesignPanelEditPhase::Commit {
                self.design_selection_color_edit_snapshots
                    .remove(&edit_target);
            }

            self.design_last_action = format!(
                "Host observed {phase:?} for selection color {selection_color_id} = #{} on {target:?}",
                color.hex()
            )
            .into();
            self.apply_design_inspection_context(panel, cx);
            cx.notify();
            return;
        }

        let selection_resource_handled = match action {
            DesignPanelAction::SelectionColorPaintStyleApplyRequested {
                target,
                selection_color_id,
                paint_references,
                style,
            } => {
                let style_data = self.design_paint_styles.style(style).cloned();
                let collection_targets = story_selection_collection_targets(paint_references);
                let applicable = story_selection_references_are_current(
                    &self.design_nodes,
                    target,
                    paint_references,
                ) && style_data.as_ref().is_some_and(|style| {
                    style.import_state == DesignPaintStyleImportState::Imported
                        && !style.paints.is_empty()
                }) && paint_references.iter().all(|reference| {
                    self.design_nodes
                        .iter()
                        .find(|node| node.id == reference.node_id)
                        .and_then(|node| story_selection_reference_paint(node, reference))
                        .is_some_and(|paint| !paint.read_only)
                });
                if applicable {
                    let style_data =
                        style_data.expect("applicable Selection Paint style is present");
                    for (node_id, collection) in &collection_targets {
                        if let Some(node) = self
                            .design_nodes
                            .iter_mut()
                            .find(|node| node.id == *node_id)
                        {
                            apply_story_paint_style(
                                node,
                                story_design_collection(*collection),
                                style,
                                style_data.clone(),
                            );
                        }
                    }
                }
                self.design_last_action = if applicable {
                    format!(
                        "Host applied Paint style {} to {} exact Selection-color collection{} for {selection_color_id} on {target:?}",
                        style.style_id,
                        collection_targets.len(),
                        if collection_targets.len() == 1 { "" } else { "s" },
                    )
                    .into()
                } else {
                    format!(
                        "Host rejected stale Selection Paint-style apply for {selection_color_id}"
                    )
                    .into()
                };
                true
            }
            DesignPanelAction::SelectionColorPaintStyleImportRequested {
                target,
                selection_color_id,
                paint_references,
                style,
            } => {
                let current = story_selection_references_are_current(
                    &self.design_nodes,
                    target,
                    paint_references,
                );
                let imported = current
                    && self
                        .design_paint_styles
                        .style_mut(style)
                        .filter(|style| {
                            style.import_state == DesignPaintStyleImportState::Available
                        })
                        .map(|style| {
                            style.import_state = DesignPaintStyleImportState::Imported;
                        })
                        .is_some();
                self.design_last_action = if imported {
                    format!(
                        "Host imported Selection Paint style {} for {selection_color_id}",
                        style.style_id
                    )
                    .into()
                } else {
                    format!(
                        "Host rejected stale Selection Paint-style import for {selection_color_id}"
                    )
                    .into()
                };
                true
            }
            DesignPanelAction::SelectionColorPaintStyleCreateRequested {
                target,
                selection_color_id,
                paint_references,
                paints,
            } => {
                let collection_targets = story_selection_collection_targets(paint_references);
                let creatable = !paints.is_empty()
                    && story_selection_references_are_current(
                        &self.design_nodes,
                        target,
                        paint_references,
                    )
                    && collection_targets.iter().all(|(node_id, collection)| {
                        self.design_nodes
                            .iter()
                            .find(|node| node.id == *node_id)
                            .is_some_and(|node| {
                                story_selection_collection_style_binding(node, *collection)
                                    .is_none()
                            })
                    });
                self.design_last_action = if creatable {
                    format!(
                        "Host opened Selection Paint-style creation with {} ordered paint{} for {selection_color_id} on {target:?}",
                        paints.len(),
                        if paints.len() == 1 { "" } else { "s" }
                    )
                    .into()
                } else {
                    format!(
                        "Host rejected stale Selection Paint-style creation for {selection_color_id}"
                    )
                    .into()
                };
                true
            }
            DesignPanelAction::SelectionColorPaintStyleDetachRequested {
                target,
                selection_color_id,
                paint_references,
                style,
            } => {
                let collection_targets = story_selection_collection_targets(paint_references);
                let detachable = story_selection_references_are_current(
                    &self.design_nodes,
                    target,
                    paint_references,
                ) && collection_targets.iter().all(|(node_id, collection)| {
                    self.design_nodes
                        .iter()
                        .find(|node| node.id == *node_id)
                        .and_then(|node| {
                            story_selection_collection_style_binding(node, *collection)
                        })
                        .is_some_and(|binding| binding.can_detach && binding.selection == *style)
                });
                if detachable {
                    for (node_id, collection) in &collection_targets {
                        if let Some(node) = self
                            .design_nodes
                            .iter_mut()
                            .find(|node| node.id == *node_id)
                        {
                            match collection {
                                DesignSelectionPaintCollection::Fill => {
                                    node.fill_style_binding = None;
                                }
                                DesignSelectionPaintCollection::Stroke => {
                                    node.stroke_style_binding = None;
                                }
                            }
                        }
                    }
                }
                self.design_last_action = if detachable {
                    format!(
                        "Host detached Selection Paint style {} for {selection_color_id} on {target:?}",
                        style.style_id
                    )
                    .into()
                } else {
                    format!(
                        "Host rejected stale Selection Paint-style detach for {selection_color_id}"
                    )
                    .into()
                };
                true
            }
            DesignPanelAction::SelectionColorVariableApplyRequested {
                target,
                selection_color_id,
                paint_references,
                variable_id,
            } => {
                let variable = self
                    .design_paint_variables
                    .variable(variable_id.as_ref())
                    .cloned();
                let applicable = story_selection_references_are_current(
                    &self.design_nodes,
                    target,
                    paint_references,
                ) && variable.as_ref().is_some_and(|variable| {
                    variable.disabled_reason.is_none()
                        && matches!(
                            variable.import_state,
                            DesignVariableImportState::Local | DesignVariableImportState::Imported
                        )
                }) && paint_references.iter().all(|reference| {
                    self.design_nodes
                        .iter()
                        .find(|node| node.id == reference.node_id)
                        .is_some_and(|node| {
                            story_selection_collection_style_binding(node, reference.collection)
                                .is_none()
                                && story_selection_reference_paint(node, reference)
                                    .is_some_and(|paint| !paint.read_only)
                        })
                });
                if applicable {
                    let variable =
                        variable.expect("applicable Selection Color variable is present");
                    for reference in paint_references {
                        if let Some(node) = self
                            .design_nodes
                            .iter_mut()
                            .find(|node| node.id == reference.node_id)
                            && let Some(paint) =
                                story_selection_reference_paint_mut(node, reference)
                        {
                            apply_story_paint_variable(
                                paint,
                                &story_selection_color_target(reference),
                                &variable,
                            );
                        }
                    }
                }
                self.design_last_action = if applicable {
                    format!(
                        "Host bound Selection Color variable {variable_id} to {} exact occurrence{} for {selection_color_id} on {target:?}",
                        paint_references.len(),
                        if paint_references.len() == 1 { "" } else { "s" },
                    )
                    .into()
                } else {
                    format!(
                        "Host rejected stale Selection Color-variable apply for {selection_color_id}"
                    )
                    .into()
                };
                true
            }
            DesignPanelAction::SelectionColorVariableImportRequested {
                target,
                selection_color_id,
                paint_references,
                variable_id,
            } => {
                let current = story_selection_references_are_current(
                    &self.design_nodes,
                    target,
                    paint_references,
                );
                let imported = current
                    && self
                        .design_paint_variables
                        .variable_mut(variable_id.as_ref())
                        .filter(|variable| {
                            variable.disabled_reason.is_none()
                                && variable.import_state == DesignVariableImportState::Available
                        })
                        .map(|variable| {
                            variable.import_state = DesignVariableImportState::Imported;
                        })
                        .is_some();
                self.design_last_action = if imported {
                    format!(
                        "Host imported Selection Color variable {variable_id} for {selection_color_id}"
                    )
                    .into()
                } else {
                    format!(
                        "Host rejected stale Selection Color-variable import for {selection_color_id}"
                    )
                    .into()
                };
                true
            }
            DesignPanelAction::SelectionColorVariableCreateRequested {
                target,
                selection_color_id,
                paint_references,
                color,
            } => {
                let creatable = story_selection_references_are_current(
                    &self.design_nodes,
                    target,
                    paint_references,
                ) && paint_references.iter().all(|reference| {
                    self.design_nodes
                        .iter()
                        .find(|node| node.id == reference.node_id)
                        .is_some_and(|node| {
                            story_selection_collection_style_binding(node, reference.collection)
                                .is_none()
                                && story_selection_reference_paint(node, reference).is_some_and(
                                    |paint| {
                                        !paint.read_only
                                            && match (&paint.payload, &reference.gradient_stop_id) {
                                                (DesignPaintPayload::Solid(solid), None) => {
                                                    solid.binding.is_none()
                                                }
                                                (
                                                    DesignPaintPayload::Gradient(gradient),
                                                    Some(stop_id),
                                                ) => {
                                                    let stop = if stop_id.is_empty() {
                                                        reference.gradient_stop_index.and_then(
                                                            |index| gradient.stops.get(index),
                                                        )
                                                    } else {
                                                        gradient
                                                            .stops
                                                            .iter()
                                                            .find(|stop| stop.id == *stop_id)
                                                    };
                                                    stop.is_some_and(|stop| stop.binding.is_none())
                                                }
                                                _ => false,
                                            }
                                    },
                                )
                        })
                });
                self.design_last_action = if creatable {
                    format!(
                        "Host opened Selection Color-variable creation for #{} and {selection_color_id} on {target:?}",
                        color.hex()
                    )
                    .into()
                } else {
                    format!(
                        "Host rejected stale Selection Color-variable creation for {selection_color_id}"
                    )
                    .into()
                };
                true
            }
            DesignPanelAction::SelectionColorVariableDetachRequested {
                target,
                selection_color_id,
                paint_references,
                variable_id,
            } => {
                let detachable = story_selection_references_are_current(
                    &self.design_nodes,
                    target,
                    paint_references,
                ) && paint_references.iter().all(|reference| {
                    self.design_nodes
                        .iter()
                        .find(|node| node.id == reference.node_id)
                        .is_some_and(|node| {
                            story_selection_collection_style_binding(node, reference.collection)
                                .is_none()
                                && story_selection_reference_paint(node, reference).is_some_and(
                                    |paint| {
                                        let target = story_selection_color_target(reference);
                                        match (&paint.payload, target) {
                                            (
                                                DesignPaintPayload::Solid(solid),
                                                DesignPaintColorTarget::Solid,
                                            ) => solid.binding.as_ref().is_some_and(|binding| {
                                                binding.variable_id == *variable_id
                                            }),
                                            (
                                                DesignPaintPayload::Gradient(gradient),
                                                DesignPaintColorTarget::GradientStop {
                                                    stop_id,
                                                    index,
                                                },
                                            ) => {
                                                let stop = if stop_id.is_empty() {
                                                    gradient.stops.get(index)
                                                } else {
                                                    gradient
                                                        .stops
                                                        .iter()
                                                        .find(|stop| stop.id == stop_id)
                                                };
                                                stop.is_some_and(|stop| {
                                                    stop.binding.as_ref().is_some_and(|binding| {
                                                        binding.variable_id == *variable_id
                                                    })
                                                })
                                            }
                                            _ => false,
                                        }
                                    },
                                )
                        })
                });
                if detachable {
                    for reference in paint_references {
                        if let Some(node) = self
                            .design_nodes
                            .iter_mut()
                            .find(|node| node.id == reference.node_id)
                            && let Some(paint) =
                                story_selection_reference_paint_mut(node, reference)
                        {
                            detach_story_paint_variable(
                                paint,
                                &story_selection_color_target(reference),
                                variable_id,
                            );
                        }
                    }
                }
                self.design_last_action = if detachable {
                    format!(
                        "Host detached Selection Color variable {variable_id} for {selection_color_id} on {target:?}"
                    )
                    .into()
                } else {
                    format!(
                        "Host rejected stale Selection Color-variable detach for {selection_color_id}"
                    )
                    .into()
                };
                true
            }
            _ => false,
        };
        if selection_resource_handled {
            self.apply_design_inspection_context(panel, cx);
            cx.notify();
            return;
        }

        match action {
            DesignPanelAction::ArrangeRequested { target, operation } => {
                self.design_last_action =
                    format!("Host applied {operation:?} to {target:?}").into();
                cx.notify();
                return;
            }
            DesignPanelAction::TransformRequested { target, operation } => {
                let (current_target, can_edit) = {
                    let panel = panel.read(cx);
                    (
                        story_design_target(panel.inspection_context()),
                        panel.inspection_context().permissions().can_edit(),
                    )
                };
                let accepted = apply_story_transform_request(
                    &mut self.design_nodes,
                    current_target.as_ref(),
                    target,
                    can_edit,
                    *operation,
                );
                self.design_last_action = if accepted {
                    match operation {
                        fanta_gpui::prelude::DesignTransformOperation::RotateClockwise90 => {
                            format!("Host atomically applied {operation:?} to {target:?}").into()
                        }
                        fanta_gpui::prelude::DesignTransformOperation::FlipHorizontal
                        | fanta_gpui::prelude::DesignTransformOperation::FlipVertical => format!(
                            "Host accepted exact {operation:?} for {target:?}; the Storybook node model does not reflect flip state"
                        )
                        .into(),
                    }
                } else {
                    format!(
                        "Host rejected stale, permission-invalid, or inapplicable {operation:?} for {target:?}"
                    )
                    .into()
                };
                if accepted {
                    self.apply_design_inspection_context(panel, cx);
                }
                cx.notify();
                return;
            }
            DesignPanelAction::ResizeToFitRequested { target } => {
                self.design_last_action = format!("Host resized {target:?} to fit").into();
                cx.notify();
                return;
            }
            _ => {}
        }
        let action_node_id = match action {
            DesignPanelAction::TypographyPropertyChangeRequested { node_id, .. }
            | DesignPanelAction::TypographyPropertyEditRequested { node_id, .. }
            | DesignPanelAction::TypographyVariableAxisEditRequested { node_id, .. }
            | DesignPanelAction::TypographyStyleApplyRequested { node_id, .. }
            | DesignPanelAction::TypographyStyleDetachRequested { node_id, .. }
            | DesignPanelAction::TypographyFontApplyRequested { node_id, .. }
            | DesignPanelAction::TypographyFontImportRequested { node_id, .. }
            | DesignPanelAction::TypographyOpenTypeFeatureChangeRequested { node_id, .. }
            | DesignPanelAction::TextPathFlipOrientationRequested { node_id }
            | DesignPanelAction::TextPathStartChangeRequested { node_id, .. }
            | DesignPanelAction::VectorVertexSelectionEditRequested { node_id, .. }
            | DesignPanelAction::VectorVertexPositionEditRequested { node_id, .. }
            | DesignPanelAction::VectorVertexCornerRadiusEditRequested { node_id, .. }
            | DesignPanelAction::VectorHandleMirroringEditRequested { node_id, .. }
            | DesignPanelAction::PropertyChangeRequested { node_id, .. }
            | DesignPanelAction::PropertyEditRequested { node_id, .. }
            | DesignPanelAction::PropertyVariableApplyRequested { node_id, .. }
            | DesignPanelAction::PropertyVariableImportRequested { node_id, .. }
            | DesignPanelAction::PropertyVariableDetachRequested { node_id, .. }
            | DesignPanelAction::SectionShareRequested { node_id }
            | DesignPanelAction::SectionResolveChangedStatusRequested { node_id }
            | DesignPanelAction::TransformModifierAddRequested { node_id, .. }
            | DesignPanelAction::TransformModifierRemoveRequested { node_id, .. }
            | DesignPanelAction::TransformModifierChangeRequested { node_id, .. }
            | DesignPanelAction::ApplyTransformModifiersRequested { node_id }
            | DesignPanelAction::ComponentPropertyDefinitionCreateRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyDefinitionRenameRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyDefinitionMetadataEditRequested {
                node_id, ..
            }
            | DesignPanelAction::ComponentPropertyDefinitionEditRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyDefinitionDeleteRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyDefinitionReorderRequested { node_id, .. }
            | DesignPanelAction::ComponentVariantOptionCreateRequested { node_id, .. }
            | DesignPanelAction::ComponentVariantOptionRenameRequested { node_id, .. }
            | DesignPanelAction::ComponentVariantOptionDeleteRequested { node_id, .. }
            | DesignPanelAction::ComponentVariantOptionReorderRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyApplyToLayerRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertySwitchOnLayerRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyDetachFromLayerRequested { node_id, .. }
            | DesignPanelAction::NestedComponentPropertyExposeRequested { node_id, .. }
            | DesignPanelAction::NestedComponentPropertyUnexposeRequested { node_id, .. }
            | DesignPanelAction::NestedComponentPropertyPreviewRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyChangeRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyEditRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyResetRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyVariableApplyRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyVariableImportRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyVariableDetachRequested { node_id, .. }
            | DesignPanelAction::ComponentSwapApplyRequested { node_id, .. }
            | DesignPanelAction::ComponentSwapImportRequested { node_id, .. }
            | DesignPanelAction::ComponentSwapPreviewRequested { node_id, .. }
            | DesignPanelAction::ComponentPropertyNestedInstanceSelectRequested {
                node_id, ..
            }
            | DesignPanelAction::ComponentPropertyNestedInstanceGoToMainRequested {
                node_id, ..
            }
            | DesignPanelAction::SlotSettingsChangeRequested { node_id, .. }
            | DesignPanelAction::SlotResetRequested { node_id, .. }
            | DesignPanelAction::SlotClearRequested { node_id, .. }
            | DesignPanelAction::SlotAddInstanceRequested { node_id, .. }
            | DesignPanelAction::SlotChildSelectRequested { node_id, .. }
            | DesignPanelAction::SlotLimitLayersSelectRequested { node_id, .. }
            | DesignPanelAction::SlotChildRemoveRequested { node_id, .. }
            | DesignPanelAction::SlotChildReorderRequested { node_id, .. }
            | DesignPanelAction::SlotChildReplaceRequested { node_id, .. }
            | DesignPanelAction::CollectionItemAddRequested { node_id, .. }
            | DesignPanelAction::CollectionItemRemoveRequested { node_id, .. }
            | DesignPanelAction::EffectAddRequested { node_id, .. }
            | DesignPanelAction::EffectRemoveRequested { node_id, .. }
            | DesignPanelAction::EffectReorderRequested { node_id, .. }
            | DesignPanelAction::EffectEditRequested { node_id, .. }
            | DesignPanelAction::EffectShaderChooseRequested { node_id, .. }
            | DesignPanelAction::EffectShaderPropertyEditorRequested { node_id, .. }
            | DesignPanelAction::EffectShaderPropertyVariableDetachRequested { node_id, .. }
            | DesignPanelAction::EffectStyleApplyRequested { node_id, .. }
            | DesignPanelAction::EffectStyleCreateRequested { node_id, .. }
            | DesignPanelAction::EffectStyleDetachRequested { node_id, .. }
            | DesignPanelAction::EffectVariableApplyRequested { node_id, .. }
            | DesignPanelAction::EffectVariableDetachRequested { node_id, .. }
            | DesignPanelAction::GridDimensionsEditRequested { node_id, .. }
            | DesignPanelAction::GridTrackAddRequested { node_id, .. }
            | DesignPanelAction::GridTrackDeleteRequested { node_id, .. }
            | DesignPanelAction::GridTracksReorderRequested { node_id, .. }
            | DesignPanelAction::LayoutGridStyleApplyRequested { node_id, .. }
            | DesignPanelAction::LayoutGridStyleCreateRequested { node_id, .. }
            | DesignPanelAction::LayoutGridStyleDetachRequested { node_id, .. }
            | DesignPanelAction::LayoutGridStyleImportRequested { node_id, .. }
            | DesignPanelAction::LayoutGridPropertyEditRequested { node_id, .. }
            | DesignPanelAction::LayoutGridRemoveRequested { node_id, .. }
            | DesignPanelAction::LayoutGridVariableApplyRequested { node_id, .. }
            | DesignPanelAction::LayoutGridVariableImportRequested { node_id, .. }
            | DesignPanelAction::LayoutGridVariableDetachRequested { node_id, .. }
            | DesignPanelAction::LayoutGridVariableCreateRequested { node_id, .. }
            | DesignPanelAction::LayoutGridCountVariableApplyRequested { node_id, .. }
            | DesignPanelAction::LayoutGridCountVariableDetachRequested { node_id, .. }
            | DesignPanelAction::PaintEditRequested { node_id, .. }
            | DesignPanelAction::PaintReorderRequested { node_id, .. }
            | DesignPanelAction::PaintSourceReplaceRequested { node_id, .. }
            | DesignPanelAction::PaintMediaSourceActionRequested { node_id, .. }
            | DesignPanelAction::PaintMediaSourceDropRequested { node_id, .. }
            | DesignPanelAction::PaintMediaCropActionRequested { node_id, .. }
            | DesignPanelAction::PaintVideoPreviewActionRequested { node_id, .. }
            | DesignPanelAction::PaintShaderImportRequested { node_id, .. }
            | DesignPanelAction::PaintShaderApplyRequested { node_id, .. }
            | DesignPanelAction::PaintShaderPropertyBindRequested { node_id, .. }
            | DesignPanelAction::PaintShaderPropertyEditorRequested { node_id, .. }
            | DesignPanelAction::PaintShaderPropertyDetachRequested { node_id, .. }
            | DesignPanelAction::PaintStyleApplyRequested { node_id, .. }
            | DesignPanelAction::PaintStyleImportRequested { node_id, .. }
            | DesignPanelAction::PaintStyleCreateRequested { node_id, .. }
            | DesignPanelAction::PaintStyleDetachRequested { node_id, .. }
            | DesignPanelAction::PaintColorVariableApplyRequested { node_id, .. }
            | DesignPanelAction::PaintColorVariableImportRequested { node_id, .. }
            | DesignPanelAction::PaintColorVariableDetachRequested { node_id, .. }
            | DesignPanelAction::PaintColorVariableCreateRequested { node_id, .. }
            | DesignPanelAction::PaintColorStyleSampleRequested { node_id, .. }
            | DesignPanelAction::PaintColorStyleApplyRequested { node_id, .. }
            | DesignPanelAction::PaintColorStyleCreateRequested { node_id, .. }
            | DesignPanelAction::PaintEyedropperRequested { node_id, .. }
            | DesignPanelAction::SwapStrokeEndpointsRequested { node_id }
            | DesignPanelAction::ResetInstanceOverridesRequested { node_id }
            | DesignPanelAction::GoToMainComponentRequested { node_id }
            | DesignPanelAction::DetachInstanceRequested { node_id } => node_id,
            DesignPanelAction::PropertyCopyRequested { .. }
            | DesignPanelAction::DimensionLimitsPreviewRequested { .. }
            | DesignPanelAction::MenuPreviewRequested { .. }
            | DesignPanelAction::ViewerSectionCopyRequested { .. }
            | DesignPanelAction::ViewerSectionRepresentationChangeRequested { .. }
            | DesignPanelAction::SurfaceChangeRequested { .. }
            | DesignPanelAction::SelectionHeaderCommandRequested { .. }
            | DesignPanelAction::AddAutoLayoutRequested { .. }
            | DesignPanelAction::SmartSelectionSpacingEditRequested { .. }
            | DesignPanelAction::SmartSelectionArrangeRequested { .. }
            | DesignPanelAction::TargetedNodeActionRequested { .. }
            | DesignPanelAction::ArrangeRequested { .. }
            | DesignPanelAction::TransformRequested { .. }
            | DesignPanelAction::ResizeToFitRequested { .. }
            | DesignPanelAction::FramePresetApplyRequested { .. }
            | DesignPanelAction::ExportConfigurationAddRequested { .. }
            | DesignPanelAction::ExportConfigurationRemoveRequested { .. }
            | DesignPanelAction::ExportConfigurationChangeRequested { .. }
            | DesignPanelAction::ExportModeChangeRequested { .. }
            | DesignPanelAction::AnimatedExportChangeRequested { .. }
            | DesignPanelAction::AnimatedExportRequested { .. }
            | DesignPanelAction::ExportAllRequested { .. }
            | DesignPanelAction::ExportPreviewRequested { .. }
            | DesignPanelAction::PageBackgroundEditRequested { .. }
            | DesignPanelAction::PageBackgroundChangeRequested { .. }
            | DesignPanelAction::LocalResourceBrowseRequested { .. }
            | DesignPanelAction::LocalResourceOpenRequested { .. }
            | DesignPanelAction::LocalResourceCreateRequested { .. }
            | DesignPanelAction::LocalResourceImportRequested { .. }
            | DesignPanelAction::LocalStyleCommandRequested { .. }
            | DesignPanelAction::LocalStyleCreateRequested { .. }
            | DesignPanelAction::LocalStyleFolderCreateRequested { .. }
            | DesignPanelAction::LocalStylesDeleteRequested { .. }
            | DesignPanelAction::LocalStylesMoveRequested { .. }
            | DesignPanelAction::VariablesViewOpenRequested { .. }
            | DesignPanelAction::VariableModeApplyRequested { .. }
            | DesignPanelAction::VariableModeClearRequested { .. }
            | DesignPanelAction::SelectionColorEditRequested { .. }
            | DesignPanelAction::SelectionColorPaintEditRequested { .. }
            | DesignPanelAction::SelectionColorOccurrencesSelectRequested { .. }
            | DesignPanelAction::SelectionColorPaintStyleApplyRequested { .. }
            | DesignPanelAction::SelectionColorPaintStyleImportRequested { .. }
            | DesignPanelAction::SelectionColorPaintStyleCreateRequested { .. }
            | DesignPanelAction::SelectionColorPaintStyleDetachRequested { .. }
            | DesignPanelAction::SelectionColorVariableApplyRequested { .. }
            | DesignPanelAction::SelectionColorVariableImportRequested { .. }
            | DesignPanelAction::SelectionColorVariableCreateRequested { .. }
            | DesignPanelAction::SelectionColorVariableDetachRequested { .. } => unreachable!(),
            DesignPanelAction::PaintChangeRequested { .. }
            | DesignPanelAction::ExportRequested { .. }
            | DesignPanelAction::ReplaceMediaRequested { .. } => {
                unreachable!("compatibility actions return before node resolution")
            }
        };
        let Some(node_index) = self
            .design_nodes
            .iter()
            .position(|node| node.id == *action_node_id)
        else {
            self.design_last_action =
                format!("Ignored stale Design action for {action_node_id}").into();
            cx.notify();
            return;
        };
        let inside_auto_layout = node_index == self.selected_design_node
            && self
                .design_inspection_context()
                .0
                .parent_layout()
                .is_auto_layout();
        if story_is_section_or_transform_action(action) {
            let (current_target, can_edit) = {
                let panel = panel.read(cx);
                (
                    story_design_target(panel.inspection_context()),
                    panel.inspection_context().permissions().can_edit(),
                )
            };
            let accepted = apply_story_section_or_transform_action(
                &mut self.design_nodes[node_index],
                current_target.as_ref(),
                can_edit,
                action,
            );
            self.design_last_action = story_section_or_transform_action_status(action, accepted);
            if accepted {
                self.apply_design_inspection_context(panel, cx);
            }
            cx.notify();
            return;
        }
        if let Some((collection, target)) = action.paint_target() {
            let expected = self.design_paint_target_for(action_node_id, collection);
            let stale_cancel_is_active = match action {
                DesignPanelAction::PaintEditRequested {
                    node_id,
                    collection,
                    target,
                    paint_id,
                    index,
                    phase: DesignPanelEditPhase::Cancel,
                    ..
                } => self
                    .design_paint_edit_snapshots
                    .contains_key(&StoryPaintEditTarget::new(
                        node_id.clone(),
                        *collection,
                        *target,
                        paint_id.clone(),
                        *index,
                    )),
                DesignPanelAction::PaintMediaCropActionRequested {
                    node_id,
                    collection,
                    target,
                    paint_id,
                    index,
                    action: DesignMediaCropAction::Cancel,
                } => self
                    .design_media_crop_targets
                    .contains(&StoryPaintEditTarget::new(
                        node_id.clone(),
                        *collection,
                        *target,
                        paint_id.clone(),
                        *index,
                    )),
                DesignPanelAction::PaintVideoPreviewActionRequested {
                    node_id,
                    collection,
                    target,
                    paint_id,
                    index,
                    action:
                        DesignVideoPreviewAction::Scrub {
                            phase: DesignPanelEditPhase::Cancel,
                            ..
                        },
                } => self
                    .design_video_scrub_snapshots
                    .contains_key(&StoryPaintEditTarget::new(
                        node_id.clone(),
                        *collection,
                        *target,
                        paint_id.clone(),
                        *index,
                    )),
                _ => false,
            };
            if target != expected && !stale_cancel_is_active {
                self.design_last_action = format!(
                    "Host rejected stale {target:?} paint intent for {action_node_id}; current target is {expected:?}"
                )
                .into();
                cx.notify();
                return;
            }
        }
        let paint_lifecycle_is_valid = match action {
            DesignPanelAction::PaintEditRequested {
                node_id,
                collection,
                target,
                paint_id,
                index,
                phase,
                ..
            } => {
                let target = StoryPaintEditTarget::new(
                    node_id.clone(),
                    *collection,
                    *target,
                    paint_id.clone(),
                    *index,
                );
                match phase {
                    DesignPanelEditPhase::Begin => {
                        !self.design_paint_edit_snapshots.contains_key(&target)
                    }
                    DesignPanelEditPhase::Preview | DesignPanelEditPhase::Cancel => {
                        self.design_paint_edit_snapshots.contains_key(&target)
                    }
                    // Picker buttons and toggles are allowed to emit one
                    // atomic Commit without opening a transaction.
                    DesignPanelEditPhase::Commit => true,
                }
            }
            DesignPanelAction::PaintMediaCropActionRequested {
                node_id,
                collection,
                target,
                paint_id,
                index,
                action,
            } => {
                let target = StoryPaintEditTarget::new(
                    node_id.clone(),
                    *collection,
                    *target,
                    paint_id.clone(),
                    *index,
                );
                match action {
                    DesignMediaCropAction::Begin => self.design_media_crop_targets.insert(target),
                    DesignMediaCropAction::Preview { .. } => {
                        self.design_media_crop_targets.contains(&target)
                    }
                    DesignMediaCropAction::Commit { .. } | DesignMediaCropAction::Cancel => {
                        self.design_media_crop_targets.remove(&target)
                    }
                    DesignMediaCropAction::ResizeToFit => true,
                }
            }
            DesignPanelAction::PaintVideoPreviewActionRequested {
                node_id,
                collection,
                target,
                paint_id,
                index,
                action: DesignVideoPreviewAction::Scrub { phase, .. },
            } => {
                let target = StoryPaintEditTarget::new(
                    node_id.clone(),
                    *collection,
                    *target,
                    paint_id.clone(),
                    *index,
                );
                match phase {
                    DesignPanelEditPhase::Begin => {
                        !self.design_video_scrub_snapshots.contains_key(&target)
                    }
                    DesignPanelEditPhase::Preview
                    | DesignPanelEditPhase::Commit
                    | DesignPanelEditPhase::Cancel => {
                        self.design_video_scrub_snapshots.contains_key(&target)
                    }
                }
            }
            _ => true,
        };
        if !paint_lifecycle_is_valid {
            self.design_last_action =
                format!("Host rejected an out-of-order paint transaction for {action_node_id}")
                    .into();
            cx.notify();
            return;
        }
        let media_drop_capabilities = match action {
            DesignPanelAction::PaintMediaSourceDropRequested {
                collection,
                paint_id,
                index,
                ..
            } => self
                .design_media_paint_views
                .get(action_node_id)
                .and_then(|views| views.paint(*collection, paint_id, *index))
                .map_or_else(DesignMediaPaintCapabilities::default, |view| {
                    view.capabilities
                }),
            _ => DesignMediaPaintCapabilities::viewer(),
        };
        let node_edit_transaction = story_node_edit_transaction(action);
        if let Some((target, phase)) = &node_edit_transaction {
            begin_story_node_edit(
                &self.design_nodes[node_index],
                &mut self.design_node_edit_snapshots,
                target,
                *phase,
            );
        }
        let node = &mut self.design_nodes[node_index];
        match action {
            DesignPanelAction::TypographyPropertyChangeRequested {
                node_id,
                target,
                property,
                value,
            } => {
                apply_design_property_with_parent(node, *property, value, inside_auto_layout);
                self.design_last_action =
                    format!("Host applied {property:?} = {value:?} to {target:?} on {node_id}")
                        .into();
            }
            DesignPanelAction::TypographyPropertyEditRequested {
                node_id,
                target,
                property,
                value,
                phase,
            } => {
                if *phase != DesignPanelEditPhase::Begin {
                    apply_design_property_with_parent(node, *property, value, inside_auto_layout);
                }
                self.design_last_action = format!(
                    "Host observed {phase:?} for {property:?} = {value:?} on {target:?} in {node_id}"
                )
                .into();
            }
            DesignPanelAction::TypographyVariableAxisEditRequested {
                node_id,
                target,
                tag,
                value,
                phase,
            } => {
                let changed = if *phase == DesignPanelEditPhase::Begin {
                    true
                } else {
                    node.typography.as_mut().is_some_and(|typography| {
                        typography
                            .variable_axes
                            .iter_mut()
                            .find(|axis| axis.tag == *tag)
                            .is_some_and(|axis| {
                                if axis.is_editable()
                                    && value.is_finite()
                                    && (axis.min..=axis.max).contains(value)
                                {
                                    axis.value = *value;
                                    true
                                } else {
                                    false
                                }
                            })
                    })
                };
                self.design_last_action = if changed {
                    format!(
                        "Host observed {phase:?} for variable axis {tag} = {value} on {target:?} in {node_id}"
                    )
                    .into()
                } else {
                    format!("Host rejected stale variable axis {tag} on {node_id}").into()
                };
            }
            DesignPanelAction::TypographyStyleApplyRequested {
                node_id,
                target,
                style,
            } => {
                let resolved_style = self.design_typography_styles.style(style).cloned();
                let applied = node
                    .typography
                    .as_mut()
                    .zip(resolved_style.as_ref())
                    .is_some_and(|(typography, resolved)| {
                        typography.family = resolved.family.clone();
                        typography.style = resolved.font_style.clone();
                        typography.size = resolved.size;
                        typography.style_binding = Some(DesignTypographyStyleBinding::new(
                            style.clone(),
                            resolved.name.clone(),
                        ));
                        true
                    });
                self.design_last_action = if applied {
                    format!(
                        "Host applied text style {} to {target:?} on {node_id}",
                        style.style_id
                    )
                    .into()
                } else {
                    format!("Host rejected unavailable text style on {node_id}").into()
                };
            }
            DesignPanelAction::TypographyStyleDetachRequested {
                node_id,
                target,
                style,
            } => {
                let detached = node.typography.as_mut().is_some_and(|typography| {
                    let can_detach = typography
                        .style_binding
                        .as_ref()
                        .is_some_and(|binding| binding.can_detach && binding.selection == *style);
                    if can_detach {
                        typography.style_binding = None;
                    }
                    can_detach
                });
                self.design_last_action = if detached {
                    format!(
                        "Host detached text style {} from {target:?} on {node_id}",
                        style.style_id
                    )
                    .into()
                } else {
                    format!("Host rejected stale or locked text-style detach on {node_id}").into()
                };
            }
            DesignPanelAction::TypographyFontApplyRequested {
                node_id,
                target,
                font,
            } => {
                let resolved = self
                    .design_fonts
                    .font(font)
                    .filter(|(_, style)| style.availability.can_apply())
                    .map(|(family, style)| {
                        (
                            family.name.clone(),
                            style.name.clone(),
                            style.weight.map(f32::from),
                        )
                    });
                let applied = node.typography.as_mut().zip(resolved).is_some_and(
                    |(typography, (family, style, weight))| {
                        typography.family = family;
                        typography.style = style;
                        if let Some(weight) = weight {
                            typography.weight = weight;
                        }
                        typography.style_binding = None;
                        true
                    },
                );
                self.design_last_action = if applied {
                    format!("Host applied font {font:?} to {target:?} on {node_id}").into()
                } else {
                    format!("Host rejected unavailable font on {node_id}").into()
                };
            }
            DesignPanelAction::TypographyFontImportRequested {
                node_id,
                target,
                font,
            } => {
                let imported = self
                    .design_fonts
                    .families
                    .iter_mut()
                    .find(|family| family.id == font.family_id && family.source == font.source)
                    .and_then(|family| {
                        family
                            .styles
                            .iter_mut()
                            .find(|style| style.id == font.style_id)
                    })
                    .is_some_and(|style| {
                        if style.availability.can_import() {
                            style.availability = DesignFontAvailability::Imported;
                            true
                        } else {
                            false
                        }
                    });
                if imported {
                    let fonts = self.design_fonts.clone();
                    panel.update(cx, |panel, cx| {
                        panel.set_font_view_data(fonts, cx);
                    });
                }
                self.design_last_action = if imported {
                    format!(
                        "Host imported font {} for {target:?} on {node_id}; choose it again to apply",
                        font.style_id
                    )
                    .into()
                } else {
                    format!("Host rejected stale font import on {node_id}").into()
                };
            }
            DesignPanelAction::TypographyOpenTypeFeatureChangeRequested {
                node_id,
                target,
                tag,
                enabled,
            } => {
                let changed = node.typography.as_mut().is_some_and(|typography| {
                    typography
                        .open_type_features
                        .iter_mut()
                        .find(|feature| feature.tag == *tag)
                        .is_some_and(|feature| {
                            if feature.availability.is_available() && feature.enabled != *enabled {
                                feature.enabled = *enabled;
                                true
                            } else {
                                false
                            }
                        })
                });
                self.design_last_action = if changed {
                    format!(
                        "Host set OpenType {} to {enabled} for {target:?} on {node_id}",
                        tag.api_name()
                    )
                    .into()
                } else {
                    format!("Host rejected stale OpenType feature on {node_id}").into()
                };
            }
            DesignPanelAction::TextPathFlipOrientationRequested { node_id } => {
                let flipped = apply_story_text_path_flip_orientation(node, node_id);
                self.design_last_action = if flipped {
                    let orientation = node
                        .text_path
                        .as_ref()
                        .map_or("Unknown", |text_path| text_path.orientation.label());
                    format!("Host flipped text orientation to {orientation} on {node_id}").into()
                } else {
                    format!("Host rejected unavailable or stale text-orientation flip on {node_id}")
                        .into()
                };
            }
            DesignPanelAction::TextPathStartChangeRequested {
                node_id,
                data,
                phase,
            } => {
                apply_story_text_path_edit_phase(
                    node,
                    &mut self.design_text_path_edit_snapshots,
                    node_id,
                    *data,
                    *phase,
                );
                self.design_last_action =
                    format!("Host observed {phase:?} for textPathStartData {data:?} in {node_id}")
                        .into();
            }
            DesignPanelAction::VectorVertexSelectionEditRequested {
                node_id,
                selected_vertex_ids,
                phase,
            } => {
                apply_story_vector_edit_phase(
                    node,
                    &mut self.design_vector_edit_snapshots,
                    node_id,
                    StoryVectorEditChange::Selection(selected_vertex_ids.clone()),
                    *phase,
                );
                self.design_last_action = format!(
                    "Host observed {phase:?} vector selection {selected_vertex_ids:?} in {node_id}"
                )
                .into();
            }
            DesignPanelAction::VectorVertexPositionEditRequested {
                node_id,
                vertex_ids,
                axis,
                value,
                phase,
            } => {
                apply_story_vector_edit_phase(
                    node,
                    &mut self.design_vector_edit_snapshots,
                    node_id,
                    StoryVectorEditChange::Position {
                        vertex_ids: vertex_ids.clone(),
                        axis: *axis,
                        value: *value,
                    },
                    *phase,
                );
                self.design_last_action = format!(
                    "Host observed {phase:?} vector {axis:?} = {value} for {vertex_ids:?} in {node_id}"
                )
                .into();
            }
            DesignPanelAction::VectorVertexCornerRadiusEditRequested {
                node_id,
                vertex_ids,
                radius,
                phase,
            } => {
                apply_story_vector_edit_phase(
                    node,
                    &mut self.design_vector_edit_snapshots,
                    node_id,
                    StoryVectorEditChange::CornerRadius {
                        vertex_ids: vertex_ids.clone(),
                        radius: *radius,
                    },
                    *phase,
                );
                self.design_last_action = format!(
                    "Host observed {phase:?} vertex radius {radius} for {vertex_ids:?} in {node_id}"
                )
                .into();
            }
            DesignPanelAction::VectorHandleMirroringEditRequested {
                node_id,
                vertex_ids,
                mirroring,
                phase,
            } => {
                apply_story_vector_edit_phase(
                    node,
                    &mut self.design_vector_edit_snapshots,
                    node_id,
                    StoryVectorEditChange::HandleMirroring {
                        vertex_ids: vertex_ids.clone(),
                        mirroring: *mirroring,
                    },
                    *phase,
                );
                self.design_last_action = format!(
                    "Host observed {phase:?} handle mirroring {mirroring:?} for {vertex_ids:?} in {node_id}"
                )
                .into();
            }
            DesignPanelAction::PropertyChangeRequested {
                node_id,
                property,
                value,
            } => {
                apply_design_property_with_parent(node, *property, value, inside_auto_layout);
                self.design_last_action =
                    format!("Host applied {property:?} = {value:?} to {node_id}").into();
            }
            DesignPanelAction::PropertyEditRequested {
                node_id,
                property,
                value,
                phase,
            } => {
                if *phase != DesignPanelEditPhase::Begin {
                    apply_design_property_with_parent(node, *property, value, inside_auto_layout);
                }
                self.design_last_action =
                    format!("Host observed {phase:?} for {property:?} = {value:?} on {node_id}")
                        .into();
            }
            DesignPanelAction::PropertyVariableApplyRequested {
                node_id,
                target,
                variable_id,
            } => {
                let variable = self
                    .design_property_variables
                    .variable(variable_id.as_ref())
                    .filter(|variable| {
                        variable.is_compatible_with(target)
                            && variable.disabled_reason.is_none()
                            && variable.import_state != DesignVariableImportState::Available
                    })
                    .cloned();
                if let Some(variable) = variable {
                    let resolved = variable.resolved_value.as_ref().map_or_else(
                        || DesignPanelValue::Text("Resolved by host".into()),
                        DesignVariableResolvedValue::panel_value,
                    );
                    if matches!(
                        target.property,
                        DesignPanelProperty::FontFamily
                            | DesignPanelProperty::FontStyle
                            | DesignPanelProperty::FontWeight
                    ) {
                        apply_design_property(node, target.property, &resolved);
                    }
                    let name = format!("{} / {}", variable.collection_name, variable.name);
                    self.design_property_bindings.insert(
                        (node_id.clone(), target.property),
                        DesignPanelPropertyBinding::new(
                            variable.id.clone(),
                            name,
                            DesignPanelBindingKind::Variable,
                            resolved,
                        ),
                    );
                    self.design_last_action = format!(
                        "Host bound {variable_id} to {:?} on {node_id}",
                        target.fields
                    )
                    .into();
                } else {
                    self.design_last_action =
                        format!("Host rejected unavailable variable {variable_id}").into();
                }
            }
            DesignPanelAction::PropertyVariableImportRequested {
                node_id,
                target,
                variable_id,
            } => {
                let imported = self
                    .design_property_variables
                    .variables
                    .iter_mut()
                    .find(|variable| {
                        variable.id == *variable_id
                            && variable.is_compatible_with(target)
                            && variable.disabled_reason.is_none()
                            && variable.import_state == DesignVariableImportState::Available
                    })
                    .is_some_and(|variable| {
                        variable.import_state = DesignVariableImportState::Imported;
                        true
                    });
                self.design_last_action = if imported {
                    format!(
                        "Host imported {variable_id} for {:?} on {node_id}; choose it again to apply",
                        target.property
                    )
                    .into()
                } else {
                    format!("Host rejected stale variable import {variable_id}").into()
                };
            }
            DesignPanelAction::PropertyVariableDetachRequested {
                node_id,
                target,
                variable_id,
            } => {
                let key = (node_id.clone(), target.property);
                let can_detach = self
                    .design_property_bindings
                    .get(&key)
                    .is_some_and(|binding| {
                        binding.kind() == DesignPanelBindingKind::Variable
                            && binding.id() == variable_id
                    });
                if can_detach {
                    self.design_property_bindings.remove(&key);
                }
                self.design_last_action = if can_detach {
                    format!(
                        "Host detached {variable_id} from {:?} on {node_id}",
                        target.fields
                    )
                    .into()
                } else {
                    format!("Host rejected stale variable detach {variable_id}").into()
                };
            }
            DesignPanelAction::ComponentPropertyChangeRequested {
                node_id,
                property_id,
                value,
            } => {
                let instance_role = node.component_context.as_ref().is_some_and(|context| {
                    matches!(
                        context.role,
                        DesignComponentRole::Instance | DesignComponentRole::SlotInstance
                    )
                });
                let applied = node
                    .component_properties
                    .iter_mut()
                    .find(|property| property.id == *property_id)
                    .is_some_and(|property| {
                        if !property.set_resolved_value(value.clone()) {
                            return false;
                        }
                        if instance_role {
                            if property.resolved_value == property.definition.default_value() {
                                property.reset_to_default();
                            } else {
                                property.mark_overridden();
                            }
                        }
                        true
                    });
                refresh_story_component_override_summary(node);
                self.design_last_action = if applied {
                    format!("Host applied component property {property_id} on {node_id}").into()
                } else {
                    format!("Host rejected component property {property_id} on {node_id}").into()
                };
            }
            DesignPanelAction::ComponentPropertyEditRequested {
                node_id,
                property_id,
                value,
                phase,
            } => {
                if *phase != DesignPanelEditPhase::Begin {
                    let instance_role = node.component_context.as_ref().is_some_and(|context| {
                        matches!(
                            context.role,
                            DesignComponentRole::Instance | DesignComponentRole::SlotInstance
                        )
                    });
                    if let Some(property) = node
                        .component_properties
                        .iter_mut()
                        .find(|property| property.id == *property_id)
                        && property.set_resolved_value(value.clone())
                        && instance_role
                    {
                        if property.resolved_value == property.definition.default_value() {
                            property.reset_to_default();
                        } else {
                            property.mark_overridden();
                        }
                    }
                    refresh_story_component_override_summary(node);
                }
                self.design_last_action = format!(
                    "Host observed {phase:?} for component property {property_id} on {node_id}"
                )
                .into();
            }
            DesignPanelAction::ComponentPropertyResetRequested {
                node_id,
                property_id,
            } => {
                if let Some(property) = node
                    .component_properties
                    .iter_mut()
                    .find(|property| property.id == *property_id)
                {
                    property.reset_to_default();
                }
                refresh_story_component_override_summary(node);
                self.design_last_action =
                    format!("Host reset component property {property_id} on {node_id}").into();
            }
            DesignPanelAction::ComponentPropertyVariableApplyRequested {
                node_id,
                target,
                variable_id,
            } => {
                let role = node.component_context.as_ref().map(|context| context.role);
                let variable = self
                    .design_property_variables
                    .variable(variable_id.as_ref())
                    .filter(|variable| {
                        variable.resolved_type == target.resolved_type
                            && variable.disabled_reason.is_none()
                            && variable.import_state != DesignVariableImportState::Available
                            && variable.resolved_value.is_some()
                    })
                    .cloned();
                let applied = node
                    .component_properties
                    .iter_mut()
                    .find(|property| property.id == target.property_id)
                    .zip(role)
                    .filter(|(property, role)| {
                        property.variable_target(*role).as_ref() == Some(target)
                    })
                    .zip(variable.as_ref())
                    .is_some_and(|((property, _), variable)| {
                        let Some(resolved_value) = variable.resolved_value.clone() else {
                            return false;
                        };
                        let binding = DesignComponentPropertyVariableBinding::new(
                            variable.id.clone(),
                            format!("{} / {}", variable.collection_name, variable.name),
                            resolved_value,
                        );
                        match target.field {
                            DesignComponentPropertyVariableField::DefinitionDefaultValue => {
                                property.default_value_binding = Some(binding);
                            }
                            DesignComponentPropertyVariableField::InstanceValue => {
                                property.resolved_value_binding = Some(binding);
                            }
                        }
                        true
                    });
                self.design_last_action = if applied {
                    format!(
                        "Host bound {variable_id} to component property {}.{} on {node_id}",
                        target.property_id,
                        target.field.api_name()
                    )
                    .into()
                } else {
                    format!("Host rejected stale component variable apply {variable_id}").into()
                };
            }
            DesignPanelAction::ComponentPropertyVariableImportRequested {
                node_id,
                target,
                variable_id,
            } => {
                let target_is_current = node
                    .component_context
                    .as_ref()
                    .and_then(|context| {
                        node.component_properties
                            .iter()
                            .find(|property| property.id == target.property_id)
                            .and_then(|property| property.variable_target(context.role))
                    })
                    .as_ref()
                    == Some(target);
                let imported = target_is_current
                    && self
                        .design_property_variables
                        .variables
                        .iter_mut()
                        .find(|variable| {
                            variable.id == *variable_id
                                && variable.resolved_type == target.resolved_type
                                && variable.disabled_reason.is_none()
                                && variable.import_state == DesignVariableImportState::Available
                        })
                        .is_some_and(|variable| {
                            variable.import_state = DesignVariableImportState::Imported;
                            true
                        });
                self.design_last_action = if imported {
                    format!(
                        "Host imported {variable_id} for component property {}.{} on {node_id}; choose it again to apply",
                        target.property_id,
                        target.field.api_name()
                    )
                    .into()
                } else {
                    format!("Host rejected stale component variable import {variable_id}").into()
                };
            }
            DesignPanelAction::ComponentPropertyVariableDetachRequested {
                node_id,
                target,
                variable_id,
            } => {
                let role = node.component_context.as_ref().map(|context| context.role);
                let detached = node
                    .component_properties
                    .iter_mut()
                    .find(|property| property.id == target.property_id)
                    .zip(role)
                    .filter(|(property, role)| {
                        property.variable_target(*role).as_ref() == Some(target)
                    })
                    .is_some_and(|(property, _)| {
                        let binding = match target.field {
                            DesignComponentPropertyVariableField::DefinitionDefaultValue => {
                                &mut property.default_value_binding
                            }
                            DesignComponentPropertyVariableField::InstanceValue => {
                                &mut property.resolved_value_binding
                            }
                        };
                        let can_detach = binding.as_ref().is_some_and(|binding| {
                            binding.variable_id == *variable_id && binding.can_detach
                        });
                        if can_detach {
                            *binding = None;
                        }
                        can_detach
                    });
                self.design_last_action = if detached {
                    format!(
                        "Host detached {variable_id} from component property {}.{} on {node_id}",
                        target.property_id,
                        target.field.api_name()
                    )
                    .into()
                } else {
                    format!("Host rejected stale component variable detach {variable_id}").into()
                };
            }
            DesignPanelAction::ComponentSwapApplyRequested {
                node_id,
                property_id,
                selection,
            } => {
                let replacement = selection.as_ref().and_then(|selection| {
                    self.design_component_swaps
                        .candidate(selection)
                        .filter(|candidate| candidate.can_apply())
                        .map(|candidate| candidate.reference.clone())
                });
                let selection_is_valid = selection.is_none() || replacement.is_some();
                let instance_role = node.component_context.as_ref().is_some_and(|context| {
                    matches!(
                        context.role,
                        DesignComponentRole::Instance | DesignComponentRole::SlotInstance
                    )
                });
                let applied = selection_is_valid
                    && node
                        .component_properties
                        .iter_mut()
                        .find(|property| property.id == *property_id)
                        .is_some_and(|property| {
                            if !property.set_resolved_value(
                                DesignComponentPropertyValue::InstanceSwap(replacement),
                            ) {
                                return false;
                            }
                            if instance_role {
                                if property.resolved_value == property.definition.default_value() {
                                    property.reset_to_default();
                                } else {
                                    property.mark_overridden();
                                }
                            }
                            true
                        });
                refresh_story_component_override_summary(node);
                self.design_last_action = if applied {
                    format!(
                        "Host applied component swap {selection:?} to {property_id} on {node_id}"
                    )
                    .into()
                } else {
                    format!("Host rejected stale component swap on {property_id}").into()
                };
            }
            DesignPanelAction::ComponentSwapImportRequested {
                node_id,
                property_id,
                selection,
            } => {
                let property_accepts_swap = node
                    .component_properties
                    .iter()
                    .find(|property| property.id == *property_id)
                    .is_some_and(|property| {
                        property.kind
                            == fanta_gpui::prelude::DesignComponentPropertyKind::InstanceSwap
                    });
                let imported = property_accepts_swap
                    && self
                        .design_component_swaps
                        .candidates
                        .iter_mut()
                        .find(|candidate| candidate.selection() == *selection)
                        .filter(|candidate| candidate.can_import())
                        .is_some_and(|candidate| {
                            candidate.import_state = DesignComponentImportState::Imported;
                            true
                        });
                self.design_last_action = if imported {
                    format!(
                        "Host imported {} for {property_id} on {node_id}; choose it again to apply",
                        selection.component_key
                    )
                    .into()
                } else {
                    format!(
                        "Host rejected stale component import {}",
                        selection.component_key
                    )
                    .into()
                };
            }
            DesignPanelAction::ComponentSwapPreviewRequested {
                node_id,
                property_id,
                selection,
            } => {
                let valid = selection.as_ref().is_none_or(|selection| {
                    self.design_component_swaps
                        .candidate(selection)
                        .is_some_and(|candidate| candidate.can_apply())
                });
                self.design_last_action = if valid {
                    format!(
                        "Host {} component-swap preview for {property_id} on {node_id}: {selection:?}",
                        if selection.is_some() { "showed" } else { "cleared" }
                    )
                    .into()
                } else {
                    format!("Host rejected stale component-swap preview on {property_id}").into()
                };
            }
            DesignPanelAction::ComponentPropertyNestedInstanceSelectRequested {
                node_id,
                property_id,
                instance_id,
            } => {
                let valid = node
                    .component_properties
                    .iter()
                    .find(|property| property.id == *property_id)
                    .is_some_and(|property| {
                        matches!(
                            &property.origin,
                            DesignComponentPropertyOrigin::NestedInstance {
                                instance_id: current,
                                ..
                            } if current == instance_id
                        )
                    });
                self.design_last_action = if valid {
                    format!(
                        "Host selected nested instance {instance_id} from {property_id} on {node_id}"
                    )
                    .into()
                } else {
                    format!("Host rejected stale nested-instance selection {instance_id}").into()
                };
            }
            DesignPanelAction::ComponentPropertyNestedInstanceGoToMainRequested {
                node_id,
                property_id,
                instance_id,
                main_component_id,
            } => {
                let valid = node
                    .component_properties
                    .iter()
                    .find(|property| property.id == *property_id)
                    .is_some_and(|property| {
                        matches!(
                            &property.origin,
                            DesignComponentPropertyOrigin::NestedInstance {
                                instance_id: current,
                                main_component: Some(main),
                                ..
                            } if current == instance_id && main.id == *main_component_id
                        )
                    });
                self.design_last_action = if valid {
                    format!(
                        "Host opened nested main component {main_component_id} for {instance_id} on {node_id}"
                    )
                    .into()
                } else {
                    format!("Host rejected stale nested-main navigation {main_component_id}").into()
                };
            }
            DesignPanelAction::SlotSettingsChangeRequested {
                node_id,
                property_id,
                expected_settings,
                change,
                phase,
            } => {
                let replacement = node
                    .component_properties
                    .iter()
                    .find(|property| property.id == *property_id)
                    .filter(|property| property.slot_settings() == Some(expected_settings))
                    .cloned()
                    .and_then(|mut property| {
                        property
                            .apply_slot_settings_change(change)
                            .then_some(property)
                    })
                    .filter(|property| {
                        property.slot_settings().is_some_and(|settings| {
                            settings.has_valid_child_range()
                                && story_component_references_are_current(
                                    &self.design_component_swaps,
                                    &settings.preferred_values,
                                )
                        })
                    });
                let accepted = replacement.is_some();
                if *phase != DesignPanelEditPhase::Begin
                    && let Some(replacement) = replacement
                    && let Some(property) = node
                        .component_properties
                        .iter_mut()
                        .find(|property| property.id == *property_id)
                {
                    *property = replacement;
                }
                self.design_last_action = if accepted {
                    format!("Host observed {phase:?} for slot settings {property_id} on {node_id}")
                        .into()
                } else {
                    format!("Host rejected stale slot settings {property_id} on {node_id}").into()
                };
            }
            DesignPanelAction::SlotResetRequested {
                node_id,
                property_id,
            } => {
                if let Some(property) = node
                    .component_properties
                    .iter_mut()
                    .find(|property| property.id == *property_id)
                {
                    property.reset_to_default();
                }
                refresh_story_component_override_summary(node);
                self.design_last_action =
                    format!("Host reset slot {property_id} on {node_id}").into();
            }
            DesignPanelAction::SlotClearRequested {
                node_id,
                property_id,
            } => {
                if let Some(property) = node
                    .component_properties
                    .iter_mut()
                    .find(|property| property.id == *property_id)
                    && property
                        .set_resolved_value(DesignComponentPropertyValue::Slot(Default::default()))
                {
                    property.mark_overridden();
                    property.refresh_slot_violations();
                }
                refresh_story_component_override_summary(node);
                self.design_last_action =
                    format!("Host cleared slot {property_id} on {node_id}").into();
            }
            DesignPanelAction::SlotAddInstanceRequested {
                node_id,
                property_id,
                preferred_component,
            } => {
                if let Some(property) = node
                    .component_properties
                    .iter_mut()
                    .find(|property| property.id == *property_id)
                {
                    let component = preferred_component.clone().or_else(|| {
                        property
                            .slot_settings()
                            .and_then(|settings| {
                                settings
                                    .preferred_values
                                    .iter()
                                    .find(|component| component.availability.is_available())
                            })
                            .cloned()
                    });
                    if let DesignComponentPropertyValue::Slot(mut slot) = property.effective_value()
                    {
                        let instance_index = slot.children.len();
                        let mut instance = if let Some(component) = component {
                            let mut instance = DesignSlotChild::instance(
                                format!("{node_id}-{property_id}-slot-{instance_index}"),
                                component.name.clone(),
                            );
                            instance.main_component = Some(component);
                            instance
                        } else {
                            DesignSlotChild::instance(
                                format!("{node_id}-{property_id}-slot-{instance_index}"),
                                "Inserted layer",
                            )
                        };
                        if instance.name.is_empty() {
                            instance.name = "Inserted layer".into();
                        }
                        slot.children.push(instance);
                        if property.set_resolved_value(DesignComponentPropertyValue::Slot(slot)) {
                            property.mark_overridden();
                            property.refresh_slot_violations();
                        }
                    }
                }
                refresh_story_component_override_summary(node);
                self.design_last_action =
                    format!("Host added an instance to slot {property_id} on {node_id}").into();
            }
            DesignPanelAction::SlotChildSelectRequested {
                node_id,
                property_id,
                child_node_id,
            } => {
                self.design_last_action = format!(
                    "Host selected slot child {child_node_id} in {property_id} on {node_id}"
                )
                .into();
            }
            DesignPanelAction::SlotLimitLayersSelectRequested {
                node_id,
                property_id,
                child_node_ids,
            } => {
                let expected = node
                    .component_properties
                    .iter()
                    .find(|property| property.id == *property_id)
                    .filter(|property| {
                        property
                            .slot_settings()
                            .is_some_and(|settings| settings.preferred_values_only)
                    })
                    .and_then(|property| {
                        let state = property.slot_state.as_ref()?;
                        let violating_ids = state
                            .violations
                            .iter()
                            .filter_map(|violation| match violation {
                                DesignSlotViolation::NonPreferredValue { instance_id, .. }
                                | DesignSlotViolation::MissingMainComponent { instance_id } => {
                                    Some(instance_id.clone())
                                }
                                DesignSlotViolation::NonPreferredChild { child_id, .. } => {
                                    Some(child_id.clone())
                                }
                                DesignSlotViolation::BelowMinimum { .. }
                                | DesignSlotViolation::AboveMaximum { .. } => None,
                            })
                            .collect::<HashSet<_>>();
                        let DesignComponentPropertyValue::Slot(value) = property.effective_value()
                        else {
                            return None;
                        };
                        Some(
                            value
                                .children
                                .iter()
                                .filter(|child| violating_ids.contains(&child.node_id))
                                .map(|child| child.node_id.clone())
                                .collect::<Vec<_>>(),
                        )
                    })
                    .unwrap_or_default();
                let all_selectable = node
                    .component_properties
                    .iter()
                    .find(|property| property.id == *property_id)
                    .and_then(|property| {
                        let DesignComponentPropertyValue::Slot(value) = property.effective_value()
                        else {
                            return None;
                        };
                        Some(expected.iter().all(|expected_id| {
                            value.children.iter().any(|child| {
                                child.node_id == *expected_id && child.capabilities.select
                            })
                        }))
                    })
                    .unwrap_or(false);
                let accepted =
                    !expected.is_empty() && expected == *child_node_ids && all_selectable;
                self.design_last_action = if accepted {
                    format!(
                        "Host selected {} non-preferred slot layers in {property_id} on {node_id}",
                        child_node_ids.len()
                    )
                    .into()
                } else {
                    format!("Host rejected stale slot-limit layers for {property_id} on {node_id}")
                        .into()
                };
            }
            DesignPanelAction::SlotChildRemoveRequested {
                node_id,
                property_id,
                child_node_id,
                index,
            } => {
                if let Some(property) = node
                    .component_properties
                    .iter_mut()
                    .find(|property| property.id == *property_id)
                    && let DesignComponentPropertyValue::Slot(mut slot) = property.effective_value()
                    && let Some(resolved_index) = slot
                        .children
                        .iter()
                        .position(|child| child.node_id == *child_node_id)
                {
                    let _index_hint_matches = resolved_index == *index;
                    slot.children.remove(resolved_index);
                    if property.set_resolved_value(DesignComponentPropertyValue::Slot(slot)) {
                        property.mark_overridden();
                        property.refresh_slot_violations();
                    }
                }
                refresh_story_component_override_summary(node);
                self.design_last_action = format!(
                    "Host removed slot child {child_node_id} from {property_id} on {node_id}"
                )
                .into();
            }
            DesignPanelAction::SlotChildReorderRequested {
                node_id,
                property_id,
                child_node_id,
                from_index,
                to_index,
            } => {
                if let Some(property) = node
                    .component_properties
                    .iter_mut()
                    .find(|property| property.id == *property_id)
                    && let DesignComponentPropertyValue::Slot(mut slot) = property.effective_value()
                    && let Some(resolved_index) = slot
                        .children
                        .iter()
                        .position(|child| child.node_id == *child_node_id)
                    && *to_index < slot.children.len()
                {
                    let _index_hint_matches = resolved_index == *from_index;
                    let child = slot.children.remove(resolved_index);
                    let destination = (*to_index).min(slot.children.len());
                    slot.children.insert(destination, child);
                    if property.set_resolved_value(DesignComponentPropertyValue::Slot(slot)) {
                        property.mark_overridden();
                        property.refresh_slot_violations();
                    }
                }
                refresh_story_component_override_summary(node);
                self.design_last_action = format!(
                    "Host reordered slot child {child_node_id} in {property_id} on {node_id}"
                )
                .into();
            }
            DesignPanelAction::SlotChildReplaceRequested {
                node_id,
                property_id,
                child_node_id,
                replacement,
            } => {
                if let Some(property) = node
                    .component_properties
                    .iter_mut()
                    .find(|property| property.id == *property_id)
                    && let DesignComponentPropertyValue::Slot(mut slot) = property.effective_value()
                    && let Some(child) = slot
                        .children
                        .iter_mut()
                        .find(|child| child.node_id == *child_node_id)
                {
                    child.name = replacement.name.clone();
                    child.kind = DesignPanelNodeKind::Instance;
                    child.main_component = Some(replacement.clone());
                    if property.set_resolved_value(DesignComponentPropertyValue::Slot(slot)) {
                        property.mark_overridden();
                        property.refresh_slot_violations();
                    }
                }
                refresh_story_component_override_summary(node);
                self.design_last_action = format!(
                    "Host swapped slot child {child_node_id} in {property_id} on {node_id}"
                )
                .into();
            }
            DesignPanelAction::EffectAddRequested { node_id, kind } => {
                let added = node.can_use_effect_kind(*kind, None);
                if added {
                    let effect_id = format!(
                        "{node_id}-effect-{}-{}",
                        kind.label().to_ascii_lowercase().replace(' ', "-"),
                        node.effects.len()
                    );
                    node.effects
                        .push(DesignEffect::new(*kind).with_id(effect_id));
                }
                self.design_last_action = if added {
                    format!("Host added {} to {node_id}", kind.label()).into()
                } else {
                    format!(
                        "Host rejected unavailable or maxed {} on {node_id}",
                        kind.label()
                    )
                    .into()
                };
            }
            DesignPanelAction::EffectRemoveRequested {
                node_id,
                effect_id,
                index,
            } => {
                let resolved = if effect_id.is_empty() {
                    (*index < node.effects.len()).then_some(*index)
                } else {
                    node.effect_index_by_id(effect_id.as_ref())
                };
                let removed = resolved.map(|index| node.effects.remove(index));
                self.design_last_action = removed.map_or_else(
                    || format!("Host ignored stale effect removal on {node_id}").into(),
                    |effect| format!("Host removed {} from {node_id}", effect.kind.label()).into(),
                );
            }
            DesignPanelAction::EffectReorderRequested {
                node_id,
                effect_id,
                from_index,
                to_index,
            } => {
                let resolved = if effect_id.is_empty() {
                    (*from_index < node.effects.len()).then_some(*from_index)
                } else {
                    node.effect_index_by_id(effect_id.as_ref())
                };
                let moved = resolved.is_some_and(|from_index| {
                    if from_index == *to_index || *to_index >= node.effects.len() {
                        return false;
                    }
                    let effect = node.effects.remove(from_index);
                    node.effects.insert(*to_index, effect);
                    true
                });
                self.design_last_action = if moved {
                    format!("Host reordered effect {effect_id} on {node_id}").into()
                } else {
                    format!("Host ignored stale effect reorder on {node_id}").into()
                };
            }
            DesignPanelAction::EffectEditRequested {
                node_id,
                effect_id,
                index,
                property,
                shader_property_id,
                value,
                phase,
            } => {
                let resolved = if effect_id.is_empty() {
                    (*index < node.effects.len()).then_some(*index)
                } else {
                    node.effect_index_by_id(effect_id.as_ref())
                };
                let applied = resolved.is_some_and(|resolved_index| {
                    if *phase == DesignPanelEditPhase::Begin {
                        return true;
                    }
                    let mut property = property.with_effect_index(resolved_index);
                    if let DesignPanelProperty::EffectShaderProperty(_, _) = property
                        && let Some(definition_id) = shader_property_id.as_ref()
                    {
                        let Some(effect) = node.effects.get(resolved_index) else {
                            return false;
                        };
                        let DesignEffectSettings::Shader(shader) = &effect.settings else {
                            return false;
                        };
                        let Some(property_index) = shader
                            .properties
                            .iter()
                            .position(|property| property.definition_id == *definition_id)
                        else {
                            return false;
                        };
                        property = DesignPanelProperty::EffectShaderProperty(
                            resolved_index,
                            property_index,
                        );
                    }
                    apply_design_property(node, property, value);
                    true
                });
                self.design_last_action = if applied {
                    format!("Host observed {phase:?} effect edit {property:?} on {node_id}").into()
                } else {
                    format!("Host ignored stale effect edit on {node_id}").into()
                };
            }
            DesignPanelAction::EffectShaderChooseRequested {
                node_id,
                effect_id,
                index,
            } => {
                let resolved = if effect_id.is_empty() {
                    (*index < node.effects.len()).then_some(*index)
                } else {
                    node.effect_index_by_id(effect_id.as_ref())
                };
                let applied = resolved.is_some_and(|index| {
                    let Some(effect) = node.effects.get_mut(index) else {
                        return false;
                    };
                    effect.set_settings(DesignEffectSettings::Shader(DesignShaderEffect::new(
                        "shader:effect:storybook",
                        "Storybook distortion",
                        [
                            DesignShaderProperty::new(
                                "amount",
                                "Amount",
                                DesignShaderPropertyKind::Number,
                                DesignShaderPropertyValue::Number(0.5),
                            ),
                            DesignShaderProperty::new(
                                "active",
                                "Active",
                                DesignShaderPropertyKind::Boolean,
                                DesignShaderPropertyValue::Boolean(true),
                            ),
                        ],
                    )));
                    true
                });
                self.design_last_action = if applied {
                    format!("Host applied an imported Shader effect on {node_id}").into()
                } else {
                    format!("Host ignored stale Shader chooser request on {node_id}").into()
                };
            }
            DesignPanelAction::EffectShaderPropertyEditorRequested {
                node_id,
                effect_id,
                index,
                shader_property_id,
                property_index,
                property_kind,
                target,
                editor,
                current_value,
            } => {
                let applied = apply_story_effect_shader_property_editor(
                    node,
                    effect_id,
                    *index,
                    shader_property_id,
                    *property_index,
                    *property_kind,
                    *target,
                    *editor,
                    current_value,
                );
                self.design_last_action = if applied {
                    format!(
                        "Host opened the {editor:?} editor for Shader property {shader_property_id} on {node_id}"
                    )
                    .into()
                } else {
                    format!("Host rejected a stale Shader property editor on {node_id}").into()
                };
            }
            DesignPanelAction::EffectShaderPropertyVariableDetachRequested {
                node_id,
                effect_id,
                index,
                shader_property_id,
                property_index,
                target,
                variable_id,
            } => {
                let detached = apply_story_effect_shader_variable_detach(
                    node,
                    effect_id,
                    *index,
                    shader_property_id,
                    *property_index,
                    *target,
                    variable_id,
                );
                self.design_last_action = if detached {
                    format!(
                        "Host detached variable {variable_id} from Shader property {shader_property_id} on {node_id}"
                    )
                    .into()
                } else {
                    format!("Host rejected a stale Shader variable detach on {node_id}").into()
                };
            }
            DesignPanelAction::EffectStyleApplyRequested { node_id, style } => {
                let resolved = self.design_effect_styles.style(style).cloned();
                let applied = resolved.is_some_and(|style_data| {
                    node.effects = style_data
                        .effect_kinds
                        .iter()
                        .enumerate()
                        .map(|(index, kind)| {
                            DesignEffect::new(*kind)
                                .with_id(format!("{node_id}-style-effect-{index}"))
                        })
                        .collect();
                    node.effect_style_binding = Some(DesignEffectStyleBinding::new(
                        style.clone(),
                        style_data.name,
                    ));
                    true
                });
                self.design_last_action = if applied {
                    format!("Host applied Effect style {} on {node_id}", style.style_id).into()
                } else {
                    format!("Host rejected unavailable Effect style on {node_id}").into()
                };
            }
            DesignPanelAction::EffectStyleCreateRequested { node_id, effects } => {
                self.design_last_action = format!(
                    "Host opened Effect-style creation for {} ordered effects on {node_id}",
                    effects.len()
                )
                .into();
            }
            DesignPanelAction::EffectStyleDetachRequested { node_id, style } => {
                let detached = node
                    .effect_style_binding
                    .as_ref()
                    .is_some_and(|binding| binding.can_detach && binding.selection == *style);
                if detached {
                    node.effect_style_binding = None;
                }
                self.design_last_action = if detached {
                    format!("Host detached Effect style {} on {node_id}", style.style_id).into()
                } else {
                    format!("Host rejected stale Effect-style detach on {node_id}").into()
                };
            }
            DesignPanelAction::EffectVariableApplyRequested {
                node_id,
                effect_id,
                index,
                field,
                variable_id,
            } => {
                let variable = self
                    .design_effect_variables
                    .variable(variable_id.as_ref())
                    .cloned();
                let resolved = if effect_id.is_empty() {
                    (*index < node.effects.len()).then_some(*index)
                } else {
                    node.effect_index_by_id(effect_id.as_ref())
                };
                let spread_supported = node.effect_capabilities.shadow_spread;
                let applied = resolved
                    .and_then(|index| node.effects.get_mut(index))
                    .zip(variable.as_ref())
                    .is_some_and(|(effect, variable)| {
                        if !effect.variable_fields(spread_supported).contains(field)
                            || variable.kind != field.value_kind()
                        {
                            return false;
                        }
                        effect.set_variable_binding(DesignEffectVariableBinding::new(
                            *field,
                            variable.id.clone(),
                            variable.name.clone(),
                            variable.collection_name.clone(),
                        ));
                        true
                    });
                self.design_last_action = if applied {
                    format!("Host bound {field:?} to {variable_id} on {node_id}").into()
                } else {
                    format!("Host rejected incompatible effect variable on {node_id}").into()
                };
            }
            DesignPanelAction::EffectVariableDetachRequested {
                node_id,
                effect_id,
                index,
                field,
                variable_id,
            } => {
                let resolved = if effect_id.is_empty() {
                    (*index < node.effects.len()).then_some(*index)
                } else {
                    node.effect_index_by_id(effect_id.as_ref())
                };
                let detached = resolved
                    .and_then(|index| node.effects.get_mut(index))
                    .is_some_and(|effect| {
                        let can_detach = effect.variable_binding(*field).is_some_and(|binding| {
                            binding.can_detach && binding.variable_id == *variable_id
                        });
                        if can_detach {
                            effect.remove_variable_binding(*field);
                        }
                        can_detach
                    });
                self.design_last_action = if detached {
                    format!("Host detached {variable_id} from {field:?} on {node_id}").into()
                } else {
                    format!("Host rejected stale effect-variable detach on {node_id}").into()
                };
            }
            DesignPanelAction::CollectionItemAddRequested {
                node_id,
                collection,
                ..
            } => {
                match collection {
                    DesignPanelCollection::Fill => {
                        let paint_id = format!("{node_id}-fill-host-{}", node.fills.len());
                        node.fills
                            .push(DesignPaint::solid(DesignColor::BLUE).with_id(paint_id));
                    }
                    DesignPanelCollection::Stroke => {
                        let paint_count =
                            node.stroke.as_ref().map_or(0, |stroke| stroke.paints.len());
                        let paint = DesignPaint::solid(DesignColor::BLACK)
                            .with_id(format!("{node_id}-stroke-host-{paint_count}"));
                        if let Some(stroke) = node.stroke.as_mut() {
                            stroke.add_paint(paint);
                        } else {
                            node.stroke = Some(DesignStroke::for_node(
                                node.kind,
                                paint,
                                1.,
                                DesignStrokeAlign::Inside,
                            ));
                        }
                    }
                    DesignPanelCollection::Effect => node.effects.push(
                        DesignEffect::drop_shadow(
                            DesignColor::rgba(0x00, 0x00, 0x00, 0x33),
                            16.,
                            0.,
                            0.,
                            4.,
                        )
                        .with_id(format!("{node_id}-legacy-effect-{}", node.effects.len())),
                    ),
                    DesignPanelCollection::LayoutGrid => {
                        if node.layout_grid_style_binding.is_none() {
                            let guide_id =
                                format!("{node_id}-guide-host-{}", node.layout_grids.len());
                            node.layout_grids
                                .push(DesignLayoutGrid::default().with_id(guide_id));
                        }
                    }
                    DesignPanelCollection::Export => {
                        node.export_settings.push(DesignExportSetting {
                            scale: 1.,
                            suffix: "".into(),
                            format: DesignExportFormat::Png,
                        });
                    }
                }
                self.design_last_action =
                    format!("Host added {} to {node_id}", collection.label()).into();
            }
            DesignPanelAction::CollectionItemRemoveRequested {
                node_id,
                collection,
                index,
                ..
            } => {
                match collection {
                    DesignPanelCollection::Fill => {
                        if node.kind == DesignPanelNodeKind::MultipleSelection {
                            remove_index(&mut node.selection_colors, *index);
                        } else {
                            remove_index(&mut node.fills, *index);
                        }
                    }
                    DesignPanelCollection::Stroke => {
                        if let Some(stroke) = node.stroke.as_mut() {
                            stroke.remove_paint(*index);
                        }
                    }
                    DesignPanelCollection::Effect => remove_index(&mut node.effects, *index),
                    DesignPanelCollection::LayoutGrid => {
                        unreachable!("indexed layout-guide removal returns before reducer dispatch")
                    }
                    DesignPanelCollection::Export => {
                        remove_index(&mut node.export_settings, *index);
                    }
                }
                self.design_last_action = format!(
                    "Host removed {} #{index} from {node_id}",
                    collection.label()
                )
                .into();
            }
            DesignPanelAction::LayoutGridPropertyEditRequested {
                node_id,
                guide_id,
                index,
                property,
                value,
                phase,
            } => {
                let target = StoryLayoutGridEditTarget::new(
                    node_id.clone(),
                    guide_id.clone(),
                    *index,
                    *property,
                );
                let applied = apply_story_layout_grid_edit_phase(
                    node,
                    &mut self.design_layout_grid_edit_snapshots,
                    target,
                    *property,
                    value,
                    *phase,
                );
                self.design_last_action = if applied {
                    format!(
                        "Host observed {phase:?} for {property:?} on guide {guide_id} in {node_id}"
                    )
                    .into()
                } else {
                    format!("Host rejected a stale layout-guide edit on {node_id}").into()
                };
            }
            DesignPanelAction::LayoutGridRemoveRequested {
                node_id,
                guide_id,
                index,
            } => {
                let resolved_index = story_layout_grid_index(
                    node,
                    &StoryLayoutGridEditTarget::new(
                        node_id.clone(),
                        guide_id.clone(),
                        *index,
                        DesignPanelProperty::LayoutGridVisible(*index),
                    ),
                );
                let removed = node.layout_grid_style_binding.is_none()
                    && resolved_index.is_some_and(|index| {
                        node.layout_grids.remove(index);
                        true
                    });
                self.design_last_action = if removed {
                    format!("Host removed guide {guide_id} from {node_id}").into()
                } else {
                    format!("Host rejected a stale layout-guide removal on {node_id}").into()
                };
            }
            DesignPanelAction::GridDimensionsEditRequested {
                node_id,
                dimensions,
                phase,
            } => {
                let applied = apply_story_grid_dimensions_edit_phase(
                    node,
                    &mut self.design_grid_dimensions_edit_snapshots,
                    node_id,
                    *dimensions,
                    *phase,
                );
                self.design_last_action = if applied {
                    format!(
                        "Host observed {phase:?} for Grid dimensions {} × {} on {node_id}",
                        dimensions.columns, dimensions.rows
                    )
                    .into()
                } else {
                    format!("Host rejected stale or invalid Grid dimensions on {node_id}").into()
                };
            }
            DesignPanelAction::GridTrackAddRequested {
                node_id,
                axis,
                insertion_index,
            } => {
                let applied = node.layout.as_mut().is_some_and(|layout| {
                    if layout.mode != DesignLayoutMode::Grid
                        || !layout.grid_track_count_is_editable(*axis)
                    {
                        return false;
                    }
                    let tracks = match axis {
                        DesignGridTrackAxis::Column => &mut layout.grid_columns,
                        DesignGridTrackAxis::Row => &mut layout.grid_rows,
                    };
                    if *insertion_index > tracks.len() {
                        return false;
                    }
                    tracks.insert(*insertion_index, DesignGridTrack::hug());
                    true
                });
                self.design_last_action = if applied {
                    format!(
                        "Host inserted a Hug {} track at {insertion_index} on {node_id}",
                        axis.label().to_ascii_lowercase()
                    )
                    .into()
                } else {
                    format!("Host rejected an invalid Grid track insertion on {node_id}").into()
                };
            }
            DesignPanelAction::GridTrackDeleteRequested {
                node_id,
                axis,
                index,
            } => {
                let applied = node.layout.as_mut().is_some_and(|layout| {
                    if !layout.can_delete_grid_track(*axis) {
                        return false;
                    }
                    let tracks = match axis {
                        DesignGridTrackAxis::Column => &mut layout.grid_columns,
                        DesignGridTrackAxis::Row => &mut layout.grid_rows,
                    };
                    if *index >= tracks.len() {
                        return false;
                    }
                    tracks.remove(*index);
                    true
                });
                self.design_last_action = if applied {
                    format!(
                        "Host deleted {} track {index} on {node_id}",
                        axis.label().to_ascii_lowercase()
                    )
                    .into()
                } else {
                    format!("Host preserved the required final Grid track on {node_id}").into()
                };
            }
            DesignPanelAction::GridTracksReorderRequested {
                node_id,
                axis,
                from_indices,
                insertion_index,
            } => {
                let applied = node.layout.as_mut().is_some_and(|layout| {
                    if layout.mode != DesignLayoutMode::Grid {
                        return false;
                    }
                    let tracks = match axis {
                        DesignGridTrackAxis::Column => &mut layout.grid_columns,
                        DesignGridTrackAxis::Row => &mut layout.grid_rows,
                    };
                    reorder_story_grid_tracks(tracks, from_indices, *insertion_index)
                });
                self.design_last_action = if applied {
                    format!(
                        "Host reordered {} tracks {from_indices:?} at {insertion_index} on {node_id}",
                        axis.label().to_ascii_lowercase()
                    )
                    .into()
                } else {
                    format!("Host rejected an invalid Grid track reorder on {node_id}").into()
                };
            }
            DesignPanelAction::LayoutGridStyleApplyRequested { node_id, style } => {
                let resolved = self
                    .design_layout_grid_styles
                    .style(style)
                    .filter(|resource| {
                        resource.import_state == DesignLayoutGridStyleImportState::Imported
                    })
                    .cloned();
                let applied = resolved.is_some_and(|resource| {
                    node.layout_grids = resource.layout_grids;
                    node.layout_grid_style_binding = Some(match &style.source {
                        DesignLayoutGridStyleSource::Page => {
                            DesignLayoutGridStyleBinding::new(style.style_id.clone(), resource.name)
                        }
                        DesignLayoutGridStyleSource::Library { library_id } => {
                            DesignLayoutGridStyleBinding::library(
                                library_id.clone(),
                                style.style_id.clone(),
                                resource.name,
                            )
                        }
                    });
                    true
                });
                self.design_last_action = if applied {
                    format!(
                        "Host atomically applied Grid style {} on {node_id}",
                        style.style_id
                    )
                    .into()
                } else {
                    format!(
                        "Host rejected unavailable Grid style {} on {node_id}",
                        style.style_id
                    )
                    .into()
                };
            }
            DesignPanelAction::LayoutGridStyleCreateRequested {
                node_id,
                layout_grids,
            } => {
                let matches_current =
                    node.layout_grid_style_binding.is_none() && node.layout_grids == *layout_grids;
                if matches_current {
                    let style_number = self
                        .design_layout_grid_styles
                        .page_styles
                        .len()
                        .saturating_add(1);
                    let style_id: SharedString = format!("grid-style-page-{style_number}").into();
                    let name: SharedString = format!("Grid / {style_number}").into();
                    self.design_layout_grid_styles
                        .page_styles
                        .push(DesignLayoutGridStyle::new(
                            style_id.clone(),
                            name.clone(),
                            layout_grids.clone(),
                        ));
                    node.layout_grid_style_binding =
                        Some(DesignLayoutGridStyleBinding::new(style_id.clone(), name));
                    self.design_last_action =
                        format!("Host created Grid style {style_id} on {node_id}").into();
                } else {
                    self.design_last_action =
                        format!("Host rejected a stale Grid style snapshot on {node_id}").into();
                }
            }
            DesignPanelAction::LayoutGridStyleDetachRequested { node_id, style } => {
                let detached = node
                    .layout_grid_style_binding
                    .as_ref()
                    .is_some_and(|binding| binding.can_detach && binding.selection() == *style);
                if detached {
                    node.layout_grid_style_binding = None;
                }
                self.design_last_action = if detached {
                    format!("Host detached Grid style {} on {node_id}", style.style_id).into()
                } else {
                    format!("Host rejected a stale Grid style detach on {node_id}").into()
                };
            }
            DesignPanelAction::LayoutGridStyleImportRequested { node_id, style } => {
                let imported = match &style.source {
                    DesignLayoutGridStyleSource::Page => false,
                    DesignLayoutGridStyleSource::Library { library_id } => self
                        .design_layout_grid_styles
                        .libraries
                        .iter_mut()
                        .find(|library| library.id == *library_id)
                        .and_then(|library| {
                            library
                                .styles
                                .iter_mut()
                                .find(|resource| resource.id == style.style_id)
                        })
                        .is_some_and(|resource| {
                            if resource.import_state != DesignLayoutGridStyleImportState::Available
                            {
                                return false;
                            }
                            resource.import_state = DesignLayoutGridStyleImportState::Imported;
                            true
                        }),
                };
                self.design_last_action = if imported {
                    format!("Host imported Grid style {} for {node_id}", style.style_id).into()
                } else {
                    format!(
                        "Host rejected Grid style import {} for {node_id}",
                        style.style_id
                    )
                    .into()
                };
            }
            DesignPanelAction::LayoutGridVariableApplyRequested {
                node_id,
                target,
                variable_id,
            } => {
                let variable = self
                    .design_layout_grid_variables
                    .variable(variable_id.as_ref())
                    .cloned();
                let applied = variable.is_some_and(|variable| {
                    variable.import_state != DesignVariableImportState::Available
                        && apply_story_layout_grid_variable(node, target, &variable)
                });
                self.design_last_action = if applied {
                    format!(
                        "Host bound Number variable {variable_id} to {} on guide {} in {node_id}",
                        target.field.api_name(),
                        target.guide_id
                    )
                    .into()
                } else {
                    format!("Host rejected incompatible Number variable {variable_id} on {node_id}")
                        .into()
                };
            }
            DesignPanelAction::LayoutGridVariableImportRequested {
                node_id,
                target,
                variable_id,
            } => {
                let compatible = self
                    .design_layout_grid_variables
                    .variable(variable_id.as_ref())
                    .is_some_and(|variable| {
                        variable.import_state == DesignVariableImportState::Available
                            && story_layout_grid_variable_target(node, target)
                                .is_some_and(|(_, value)| value.is_compatible(variable))
                            && node.layout_grid_style_binding.is_none()
                    });
                let imported = compatible
                    && self
                        .design_layout_grid_variables
                        .variable_mut(variable_id.as_ref())
                        .is_some_and(|variable| {
                            variable.import_state = DesignVariableImportState::Imported;
                            true
                        });
                self.design_last_action = if imported {
                    format!("Host imported Number variable {variable_id} for {node_id}").into()
                } else {
                    format!("Host rejected Number-variable import on {node_id}").into()
                };
            }
            DesignPanelAction::LayoutGridVariableDetachRequested {
                node_id,
                target,
                variable_id,
            } => {
                let detached =
                    detach_story_layout_grid_variable(node, target, variable_id.as_ref());
                self.design_last_action = if detached {
                    format!(
                        "Host detached Number variable {variable_id} from {} on guide {} in {node_id}",
                        target.field.api_name(),
                        target.guide_id
                    )
                    .into()
                } else {
                    format!("Host rejected a stale layout-guide variable detach on {node_id}")
                        .into()
                };
            }
            DesignPanelAction::LayoutGridVariableCreateRequested {
                node_id,
                target,
                value,
            } => {
                let variable_number = story_layout_grid_variable_float(*value);
                let variable_index = self.design_layout_grid_variables.variables.len() + 1;
                let variable_id: SharedString =
                    format!("storybook-layout-number-{variable_index}").into();
                let variable_name: SharedString = format!("Layout number {variable_index}").into();
                let variable = variable_number.map(|number| {
                    DesignVariable::page(
                        variable_id.clone(),
                        variable_name,
                        "storybook-created",
                        "Created in Storybook",
                        DesignVariableResolvedType::Float,
                    )
                    .with_resolved_value(DesignVariableResolvedValue::Float(number))
                });
                let created = self.design_layout_grid_variables.create_state.is_enabled()
                    && variable.as_ref().is_some_and(|variable| {
                        apply_story_layout_grid_variable(node, target, variable)
                    });
                if created && let Some(variable) = variable {
                    self.design_layout_grid_variables.variables.push(variable);
                }
                self.design_last_action = if created {
                    format!(
                        "Host created and bound Number variable {variable_id} for {} on {node_id}",
                        target.field.api_name()
                    )
                    .into()
                } else {
                    format!("Host rejected layout-guide variable creation on {node_id}").into()
                };
            }
            DesignPanelAction::LayoutGridCountVariableApplyRequested { node_id, .. }
            | DesignPanelAction::LayoutGridCountVariableDetachRequested { node_id, .. } => {
                self.design_last_action =
                    format!("Host ignored a compatibility-only count-variable action on {node_id}")
                        .into();
            }
            DesignPanelAction::PaintEditRequested {
                node_id,
                collection,
                target,
                paint_id,
                index,
                edit,
                phase,
                ..
            } => {
                let edit_target = StoryPaintEditTarget::new(
                    node_id.clone(),
                    *collection,
                    *target,
                    paint_id.clone(),
                    *index,
                );
                let paints = match collection {
                    DesignPanelCollection::Fill
                        if node.kind == DesignPanelNodeKind::MultipleSelection =>
                    {
                        Some(&mut node.selection_colors)
                    }
                    DesignPanelCollection::Fill => Some(&mut node.fills),
                    DesignPanelCollection::Stroke => {
                        node.stroke.as_mut().map(|stroke| &mut stroke.paints)
                    }
                    DesignPanelCollection::Effect
                    | DesignPanelCollection::LayoutGrid
                    | DesignPanelCollection::Export => None,
                };
                if let Some(paints) = paints {
                    let resolved_index = if paint_id.is_empty() {
                        (*index < paints.len()).then_some(*index)
                    } else {
                        paints.iter().position(|paint| paint.id == *paint_id)
                    };
                    if let Some(paint) = resolved_index.and_then(|index| paints.get_mut(index)) {
                        apply_story_paint_edit_phase(
                            paint,
                            &mut self.design_paint_edit_snapshots,
                            edit_target,
                            edit,
                            *phase,
                        );
                    }
                }
                self.design_last_action = format!(
                    "Host observed {phase:?} for {:?} on {} paint {} in {node_id}",
                    edit.property,
                    collection.label(),
                    if paint_id.is_empty() {
                        format!("#{index}")
                    } else {
                        paint_id.to_string()
                    }
                )
                .into();
            }
            DesignPanelAction::PaintReorderRequested {
                node_id,
                collection,
                paint_id,
                from_index,
                to_index,
                ..
            } => {
                let paints = match collection {
                    DesignPanelCollection::Fill
                        if node.kind == DesignPanelNodeKind::MultipleSelection =>
                    {
                        Some(&mut node.selection_colors)
                    }
                    DesignPanelCollection::Fill => Some(&mut node.fills),
                    DesignPanelCollection::Stroke => {
                        node.stroke.as_mut().map(|stroke| &mut stroke.paints)
                    }
                    DesignPanelCollection::Effect
                    | DesignPanelCollection::LayoutGrid
                    | DesignPanelCollection::Export => None,
                };
                if let Some(paints) = paints {
                    let resolved_from = if paint_id.is_empty() {
                        (*from_index < paints.len()).then_some(*from_index)
                    } else {
                        paints.iter().position(|paint| paint.id == *paint_id)
                    };
                    if let Some(resolved_from) = resolved_from
                        && *to_index < paints.len()
                    {
                        let paint = paints.remove(resolved_from);
                        paints.insert(*to_index, paint);
                    }
                }
                self.design_last_action = format!(
                    "Host moved {} paint {} from {from_index} to {to_index} on {node_id}",
                    collection.label(),
                    if paint_id.is_empty() {
                        "legacy-index".to_owned()
                    } else {
                        paint_id.to_string()
                    }
                )
                .into();
            }
            DesignPanelAction::PaintSourceReplaceRequested {
                node_id,
                collection,
                paint_id,
                index,
                ..
            } => {
                let paints = match collection {
                    DesignPanelCollection::Fill
                        if node.kind == DesignPanelNodeKind::MultipleSelection =>
                    {
                        Some(&mut node.selection_colors)
                    }
                    DesignPanelCollection::Fill => Some(&mut node.fills),
                    DesignPanelCollection::Stroke => {
                        node.stroke.as_mut().map(|stroke| &mut stroke.paints)
                    }
                    DesignPanelCollection::Effect
                    | DesignPanelCollection::LayoutGrid
                    | DesignPanelCollection::Export => None,
                };
                let accepted = paints.is_some_and(|paints| {
                    let resolved_index = if paint_id.is_empty() {
                        (*index < paints.len()).then_some(*index)
                    } else {
                        paints.iter().position(|paint| paint.id == *paint_id)
                    };
                    let Some(paint) = resolved_index.and_then(|index| paints.get_mut(index)) else {
                        return false;
                    };
                    if !matches!(&paint.payload, DesignPaintPayload::Pattern(_)) {
                        return false;
                    }
                    paint.apply_edit(&DesignPaintEdit {
                        property: DesignPaintProperty::PatternSourceNode,
                        value: DesignPaintValue::PatternSourceNode(
                            format!("{node_id}-replacement-pattern-node").into(),
                        ),
                    })
                });
                self.design_last_action = format!(
                    "Host {} Pattern source for {} paint {} on {node_id}",
                    if accepted { "replaced" } else { "rejected" },
                    collection.label(),
                    if paint_id.is_empty() {
                        format!("#{index}")
                    } else {
                        paint_id.to_string()
                    }
                )
                .into();
            }
            DesignPanelAction::PaintMediaSourceActionRequested {
                node_id,
                collection,
                paint_id,
                index,
                source_id,
                action,
                ..
            } => {
                let paints = match collection {
                    DesignPanelCollection::Fill
                        if node.kind == DesignPanelNodeKind::MultipleSelection =>
                    {
                        Some(&mut node.selection_colors)
                    }
                    DesignPanelCollection::Fill => Some(&mut node.fills),
                    DesignPanelCollection::Stroke => {
                        node.stroke.as_mut().map(|stroke| &mut stroke.paints)
                    }
                    DesignPanelCollection::Effect
                    | DesignPanelCollection::LayoutGrid
                    | DesignPanelCollection::Export => None,
                };
                let accepted = paints.is_some_and(|paints| {
                    let resolved_index = if paint_id.is_empty() {
                        (*index < paints.len()).then_some(*index)
                    } else {
                        paints.iter().position(|paint| paint.id == *paint_id)
                    };
                    let Some(paint) = resolved_index.and_then(|index| paints.get_mut(index)) else {
                        return false;
                    };
                    let current_source_id = match &paint.payload {
                        DesignPaintPayload::Image(image) => &image.source.id,
                        DesignPaintPayload::Video(video) => &video.source.id,
                        _ => return false,
                    };
                    if current_source_id != source_id
                        || !action.is_applicable_to(paint.paint_type())
                    {
                        return false;
                    }
                    let source = DesignPaintSource::new(
                        format!("{node_id}-{paint_id}-{}", action.slug()),
                        match action {
                            DesignMediaSourceAction::Upload => "Uploaded source",
                            DesignMediaSourceAction::MakeImage => "Generated image",
                            DesignMediaSourceAction::EditImage => "Edited image",
                        },
                    );
                    paint.apply_edit(&DesignPaintEdit {
                        property: DesignPaintProperty::Source,
                        value: DesignPaintValue::Source(source),
                    })
                });
                self.design_last_action = if accepted {
                    format!(
                        "Host completed {} for {} paint {} on {node_id}",
                        action.label(),
                        collection.label(),
                        if paint_id.is_empty() {
                            format!("#{index}")
                        } else {
                            paint_id.to_string()
                        }
                    )
                    .into()
                } else {
                    format!(
                        "Host rejected stale {} output for {} paint {} on {node_id}",
                        action.label(),
                        collection.label(),
                        if paint_id.is_empty() {
                            format!("#{index}")
                        } else {
                            paint_id.to_string()
                        }
                    )
                    .into()
                };
            }
            DesignPanelAction::PaintMediaSourceDropRequested {
                node_id,
                collection,
                paint_id,
                index,
                expected_source_id,
                expected_media_kind,
                file,
                ..
            } => {
                let paints = match collection {
                    DesignPanelCollection::Fill
                        if node.kind == DesignPanelNodeKind::MultipleSelection =>
                    {
                        Some(&mut node.selection_colors)
                    }
                    DesignPanelCollection::Fill => Some(&mut node.fills),
                    DesignPanelCollection::Stroke => {
                        node.stroke.as_mut().map(|stroke| &mut stroke.paints)
                    }
                    DesignPanelCollection::Effect
                    | DesignPanelCollection::LayoutGrid
                    | DesignPanelCollection::Export => None,
                };
                let accepted = paints.is_some_and(|paints| {
                    apply_story_media_source_drop(
                        paints,
                        paint_id,
                        *index,
                        expected_source_id,
                        *expected_media_kind,
                        file,
                        media_drop_capabilities,
                        &mut self.next_design_media_source_id,
                    )
                });
                self.design_last_action = format!(
                    "Host {} {} file drop for {} paint {} on {node_id}",
                    if accepted { "accepted" } else { "rejected" },
                    file.kind.label(),
                    collection.label(),
                    if paint_id.is_empty() {
                        format!("#{index}")
                    } else {
                        paint_id.to_string()
                    }
                )
                .into();
            }
            DesignPanelAction::PaintShaderImportRequested {
                node_id,
                collection,
                paint_id,
                index,
                shader,
                ..
            } => {
                let imported = resolve_story_shader_mut(&mut self.design_shaders, shader)
                    .is_some_and(|definition| {
                        if definition.imported {
                            return false;
                        }
                        definition.imported = true;
                        if definition.property_definitions.is_empty() {
                            definition.property_definitions =
                                imported_story_shader_properties(&definition.id);
                        }
                        true
                    });
                self.design_last_action = if imported {
                    format!(
                        "Host imported shader {} for {} paint {} on {node_id}",
                        shader.shader_id,
                        collection.label(),
                        if paint_id.is_empty() {
                            format!("#{index}")
                        } else {
                            paint_id.to_string()
                        }
                    )
                    .into()
                } else {
                    format!("Host ignored an already-imported or stale shader on {node_id}").into()
                };
            }
            DesignPanelAction::PaintShaderApplyRequested {
                node_id,
                collection,
                paint_id,
                index,
                shader,
                ..
            } => {
                let payload = self
                    .design_shaders
                    .shader(shader)
                    .and_then(DesignShaderPaint::from_definition);
                let applied = payload
                    .zip(story_paint_mut(node, *collection, paint_id, *index))
                    .is_some_and(|(payload, paint)| {
                        paint.payload = DesignPaintPayload::Shader(payload);
                        paint.sync_legacy_projection();
                        true
                    });
                self.design_last_action = if applied {
                    format!(
                        "Host applied shader {} to {} paint {} on {node_id}",
                        shader.shader_id,
                        collection.label(),
                        if paint_id.is_empty() {
                            format!("#{index}")
                        } else {
                            paint_id.to_string()
                        }
                    )
                    .into()
                } else {
                    format!("Host rejected an unavailable shader on {node_id}").into()
                };
            }
            DesignPanelAction::PaintShaderPropertyBindRequested {
                node_id,
                collection,
                paint_id,
                index,
                definition_id,
                ..
            } => {
                let bound =
                    story_paint_mut(node, *collection, paint_id, *index).is_some_and(|paint| {
                        paint.apply_edit(&DesignPaintEdit {
                            property: DesignPaintProperty::ShaderProperty {
                                definition_id: definition_id.clone(),
                            },
                            value: DesignPaintValue::ShaderProperty(
                                DesignShaderPropertyValue::VariableAlias {
                                    variable_id: format!("storybook-variable:{definition_id}")
                                        .into(),
                                },
                            ),
                        })
                    });
                self.design_last_action = if bound {
                    format!(
                        "Host bound shader property {definition_id} in {} paint on {node_id}",
                        collection.label()
                    )
                    .into()
                } else {
                    format!("Host rejected a stale shader binding on {node_id}").into()
                };
            }
            DesignPanelAction::PaintShaderPropertyEditorRequested {
                node_id,
                collection,
                paint_id,
                index,
                definition_id,
                ..
            } => {
                let edited = apply_story_paint_shader_property_editor(
                    node,
                    *collection,
                    paint_id,
                    *index,
                    definition_id,
                );
                self.design_last_action = if edited {
                    format!("Host edited shader property {definition_id} on {node_id}").into()
                } else {
                    format!("Host rejected a stale shader property editor on {node_id}").into()
                };
            }
            DesignPanelAction::PaintShaderPropertyDetachRequested {
                node_id,
                collection,
                paint_id,
                index,
                definition_id,
                variable_id,
                ..
            } => {
                let fallback = story_paint_mut(node, *collection, paint_id, *index)
                    .and_then(|paint| {
                        let DesignPaintPayload::Shader(shader) = &paint.payload else {
                            return None;
                        };
                        let current_matches = shader
                            .property(definition_id)
                            .and_then(DesignShaderPropertyValue::variable_alias_id)
                            .is_some_and(|current| current == variable_id);
                        current_matches.then(|| shader.shader_id.clone())
                    })
                    .and_then(|shader_id| {
                        self.design_shaders
                            .definition(&shader_id)
                            .and_then(|shader| shader.property(definition_id))
                            .and_then(|property| property.default_value.clone())
                    });
                let detached = fallback
                    .zip(story_paint_mut(node, *collection, paint_id, *index))
                    .is_some_and(|(fallback, paint)| {
                        paint.apply_edit(&DesignPaintEdit {
                            property: DesignPaintProperty::ShaderProperty {
                                definition_id: definition_id.clone(),
                            },
                            value: DesignPaintValue::ShaderProperty(fallback),
                        })
                    });
                self.design_last_action = if detached {
                    format!(
                        "Host detached variable {variable_id} from shader property {definition_id} on {node_id}"
                    )
                    .into()
                } else {
                    format!("Host rejected a stale shader detach on {node_id}").into()
                };
            }
            DesignPanelAction::PaintMediaCropActionRequested {
                node_id,
                collection,
                paint_id,
                index,
                action,
                ..
            } => {
                let paints = match collection {
                    DesignPanelCollection::Fill
                        if node.kind == DesignPanelNodeKind::MultipleSelection =>
                    {
                        Some(&mut node.selection_colors)
                    }
                    DesignPanelCollection::Fill => Some(&mut node.fills),
                    DesignPanelCollection::Stroke => {
                        node.stroke.as_mut().map(|stroke| &mut stroke.paints)
                    }
                    DesignPanelCollection::Effect
                    | DesignPanelCollection::LayoutGrid
                    | DesignPanelCollection::Export => None,
                };
                let paint = paints.and_then(|paints| {
                    let resolved = if paint_id.is_empty() {
                        (*index < paints.len()).then_some(*index)
                    } else {
                        paints.iter().position(|paint| paint.id == *paint_id)
                    }?;
                    paints.get_mut(resolved)
                });
                let committed_transform = paint.as_ref().and_then(|paint| match &paint.payload {
                    DesignPaintPayload::Image(image) => image.placement.crop_transform(),
                    DesignPaintPayload::Video(video) => video.placement.crop_transform(),
                    _ => None,
                });
                let view = self
                    .design_media_paint_views
                    .entry(node_id.clone())
                    .or_default()
                    .paints
                    .iter_mut()
                    .find(|view| view.matches(*collection, paint_id, *index));
                if let Some(view) = view {
                    match action {
                        DesignMediaCropAction::Begin => {
                            view.crop_tool.active = true;
                            if let Some(transform) = committed_transform {
                                view.crop_tool.transform = transform;
                            }
                        }
                        DesignMediaCropAction::Preview {
                            transform,
                            zoom,
                            aspect_ratio,
                        } => {
                            view.crop_tool = DesignMediaCropToolState {
                                active: true,
                                transform: *transform,
                                zoom: *zoom,
                                aspect_ratio: *aspect_ratio,
                            };
                        }
                        DesignMediaCropAction::Commit {
                            transform,
                            zoom,
                            aspect_ratio,
                        } => {
                            view.crop_tool = DesignMediaCropToolState {
                                active: false,
                                transform: *transform,
                                zoom: *zoom,
                                aspect_ratio: *aspect_ratio,
                            };
                            if let Some(paint) = paint {
                                match &mut paint.payload {
                                    DesignPaintPayload::Image(image) => {
                                        image.placement = DesignMediaPaintPlacement::Crop {
                                            transform: *transform,
                                        };
                                    }
                                    DesignPaintPayload::Video(video) => {
                                        video.placement = DesignMediaPaintPlacement::Crop {
                                            transform: *transform,
                                        };
                                    }
                                    _ => {}
                                }
                                paint.sync_legacy_projection();
                            }
                        }
                        DesignMediaCropAction::Cancel => {
                            view.crop_tool.active = false;
                            if let Some(transform) = committed_transform {
                                view.crop_tool.transform = transform;
                            }
                        }
                        DesignMediaCropAction::ResizeToFit => {
                            view.crop_tool.active = false;
                            view.crop_tool.transform = DesignPaintTransform::IDENTITY;
                            if let Some(paint) = paint {
                                match &mut paint.payload {
                                    DesignPaintPayload::Image(image) => {
                                        image.placement = DesignMediaPaintPlacement::Crop {
                                            transform: DesignPaintTransform::IDENTITY,
                                        };
                                    }
                                    DesignPaintPayload::Video(video) => {
                                        video.placement = DesignMediaPaintPlacement::Crop {
                                            transform: DesignPaintTransform::IDENTITY,
                                        };
                                    }
                                    _ => {}
                                }
                                paint.sync_legacy_projection();
                            }
                        }
                    }
                }
                self.design_last_action =
                    format!("Host handled crop {action:?} for {paint_id} on {node_id}").into();
            }
            DesignPanelAction::PaintVideoPreviewActionRequested {
                node_id,
                collection,
                target,
                paint_id,
                index,
                action,
                ..
            } => {
                let edit_target = StoryPaintEditTarget::new(
                    node_id.clone(),
                    *collection,
                    *target,
                    paint_id.clone(),
                    *index,
                );
                let scrub_snapshots = &mut self.design_video_scrub_snapshots;
                let preview = self
                    .design_media_paint_views
                    .entry(node_id.clone())
                    .or_default()
                    .paints
                    .iter_mut()
                    .find(|view| view.matches(*collection, paint_id, *index))
                    .and_then(|view| view.video_preview.as_mut());
                if let Some(preview) = preview
                    && matches!(preview.status, DesignVideoPreviewStatus::Ready)
                {
                    match action {
                        DesignVideoPreviewAction::Play => preview.playing = true,
                        DesignVideoPreviewAction::Pause => preview.playing = false,
                        DesignVideoPreviewAction::Seek { seconds } => {
                            preview.current_seconds = seconds.clamp(0., preview.duration_seconds);
                        }
                        DesignVideoPreviewAction::Scrub { seconds, phase } => match phase {
                            DesignPanelEditPhase::Begin => {
                                scrub_snapshots
                                    .entry(edit_target)
                                    .or_insert(preview.current_seconds);
                            }
                            DesignPanelEditPhase::Preview => {
                                preview.current_seconds =
                                    seconds.clamp(0., preview.duration_seconds);
                            }
                            DesignPanelEditPhase::Commit => {
                                preview.current_seconds =
                                    seconds.clamp(0., preview.duration_seconds);
                                scrub_snapshots.remove(&edit_target);
                            }
                            DesignPanelEditPhase::Cancel => {
                                if let Some(original) = scrub_snapshots.remove(&edit_target) {
                                    preview.current_seconds = original;
                                }
                            }
                        },
                    }
                }
                self.design_last_action =
                    format!("Host handled video preview {action:?} for {paint_id} on {node_id}")
                        .into();
            }
            DesignPanelAction::PaintStyleApplyRequested {
                node_id,
                collection,
                style,
                ..
            } => {
                let applied =
                    self.design_paint_styles
                        .style(style)
                        .cloned()
                        .is_some_and(|style_data| {
                            apply_story_paint_style(node, *collection, style, style_data)
                        });
                self.design_last_action = if applied {
                    format!(
                        "Host applied Paint style {} to {} on {node_id}",
                        style.style_id,
                        collection.label()
                    )
                    .into()
                } else {
                    format!("Host rejected unavailable Paint style on {node_id}").into()
                };
            }
            DesignPanelAction::PaintStyleImportRequested {
                node_id,
                collection,
                style,
                ..
            } => {
                let imported = self
                    .design_paint_styles
                    .style_mut(style)
                    .filter(|style| style.import_state == DesignPaintStyleImportState::Available)
                    .map(|style| {
                        style.import_state = DesignPaintStyleImportState::Imported;
                    })
                    .is_some();
                self.design_last_action = if imported {
                    format!(
                        "Host imported Paint style {} for {}; choose it again to apply",
                        style.style_id,
                        collection.label()
                    )
                    .into()
                } else {
                    format!("Host rejected stale Paint-style import on {node_id}").into()
                };
            }
            DesignPanelAction::PaintStyleCreateRequested {
                node_id,
                collection,
                paints,
                ..
            } => {
                self.design_last_action = format!(
                    "Host opened Paint-style creation for {} ordered {} paint{} on {node_id}",
                    paints.len(),
                    collection.label(),
                    if paints.len() == 1 { "" } else { "s" }
                )
                .into();
            }
            DesignPanelAction::PaintStyleDetachRequested {
                node_id,
                collection,
                style,
                ..
            } => {
                let detached = match collection {
                    DesignPanelCollection::Fill => node
                        .fill_style_binding
                        .as_ref()
                        .is_some_and(|binding| binding.can_detach && binding.selection == *style)
                        .then(|| node.fill_style_binding = None)
                        .is_some(),
                    DesignPanelCollection::Stroke => node
                        .stroke_style_binding
                        .as_ref()
                        .is_some_and(|binding| binding.can_detach && binding.selection == *style)
                        .then(|| node.stroke_style_binding = None)
                        .is_some(),
                    DesignPanelCollection::Effect
                    | DesignPanelCollection::LayoutGrid
                    | DesignPanelCollection::Export => false,
                };
                self.design_last_action = if detached {
                    format!(
                        "Host detached Paint style {} from {} on {node_id}",
                        style.style_id,
                        collection.label()
                    )
                    .into()
                } else {
                    format!("Host rejected stale Paint-style detach on {node_id}").into()
                };
            }
            DesignPanelAction::PaintColorVariableApplyRequested {
                node_id,
                collection,
                paint_id,
                index,
                color_target,
                variable_id,
                ..
            } => {
                let variable = self
                    .design_paint_variables
                    .variable(variable_id.as_ref())
                    .cloned();
                let applied = variable
                    .filter(|variable| {
                        variable.disabled_reason.is_none()
                            && matches!(
                                variable.import_state,
                                DesignVariableImportState::Local
                                    | DesignVariableImportState::Imported
                            )
                    })
                    .is_some_and(|variable| {
                        story_paint_mut(node, *collection, paint_id, *index).is_some_and(|paint| {
                            apply_story_paint_variable(paint, color_target, &variable)
                        })
                    });
                self.design_last_action = if applied {
                    format!(
                        "Host bound Color variable {variable_id} to {:?} in {} paint {} on {node_id}",
                        color_target,
                        collection.label(),
                        if paint_id.is_empty() {
                            format!("#{index}")
                        } else {
                            paint_id.to_string()
                        }
                    )
                    .into()
                } else {
                    format!("Host rejected unavailable Color variable on {node_id}").into()
                };
            }
            DesignPanelAction::PaintColorVariableImportRequested {
                node_id,
                collection,
                variable_id,
                ..
            } => {
                let imported = self
                    .design_paint_variables
                    .variable_mut(variable_id.as_ref())
                    .filter(|variable| {
                        variable.disabled_reason.is_none()
                            && variable.import_state == DesignVariableImportState::Available
                    })
                    .map(|variable| {
                        variable.import_state = DesignVariableImportState::Imported;
                    })
                    .is_some();
                self.design_last_action = if imported {
                    format!(
                        "Host imported Color variable {variable_id} for {}; choose it again to bind",
                        collection.label()
                    )
                    .into()
                } else {
                    format!("Host rejected stale Color-variable import on {node_id}").into()
                };
            }
            DesignPanelAction::PaintColorVariableDetachRequested {
                node_id,
                collection,
                paint_id,
                index,
                color_target,
                variable_id,
                ..
            } => {
                let detached =
                    story_paint_mut(node, *collection, paint_id, *index).is_some_and(|paint| {
                        detach_story_paint_variable(paint, color_target, variable_id)
                    });
                self.design_last_action = if detached {
                    format!(
                        "Host detached Color variable {variable_id} from {:?} on {node_id}",
                        color_target
                    )
                    .into()
                } else {
                    format!("Host rejected stale Color-variable detach on {node_id}").into()
                };
            }
            DesignPanelAction::PaintColorVariableCreateRequested {
                node_id,
                collection,
                color_target,
                color,
                ..
            } => {
                self.design_last_action = format!(
                    "Host opened Color-variable creation for #{} at {:?} in {} on {node_id}",
                    color.hex(),
                    color_target,
                    collection.label()
                )
                .into();
            }
            DesignPanelAction::PaintColorStyleSampleRequested {
                node_id,
                collection,
                paint_id,
                index,
                color_target,
                sample,
                ..
            } => {
                let resolved_sample =
                    resolve_story_color_style_sample(&self.design_color_style_samples, sample)
                        .cloned();
                let applied = resolved_sample
                    .filter(|sample| sample.disabled_reason.is_none())
                    .is_some_and(|sample| {
                        story_paint_mut(node, *collection, paint_id, *index).is_some_and(|paint| {
                            apply_story_color_style_sample(paint, color_target, sample.color)
                        })
                    });
                self.design_last_action = if applied {
                    format!(
                        "Host sampled Color style {} into {:?} in {} paint {} on {node_id}",
                        sample.sample_id,
                        color_target,
                        collection.label(),
                        if paint_id.is_empty() {
                            format!("#{index}")
                        } else {
                            paint_id.to_string()
                        }
                    )
                    .into()
                } else {
                    format!("Host rejected unavailable Color-style sample on {node_id}").into()
                };
            }
            DesignPanelAction::PaintColorStyleApplyRequested {
                node_id,
                collection,
                paint_id,
                index,
                color_target,
                style,
                ..
            } => {
                let resolved_style =
                    resolve_story_color_style(&self.design_color_styles, style).cloned();
                let paints = match collection {
                    DesignPanelCollection::Fill
                        if node.kind == DesignPanelNodeKind::MultipleSelection =>
                    {
                        Some(&mut node.selection_colors)
                    }
                    DesignPanelCollection::Fill => Some(&mut node.fills),
                    DesignPanelCollection::Stroke => {
                        node.stroke.as_mut().map(|stroke| &mut stroke.paints)
                    }
                    DesignPanelCollection::Effect
                    | DesignPanelCollection::LayoutGrid
                    | DesignPanelCollection::Export => None,
                };
                let applied = paints.is_some_and(|paints| {
                    let resolved_index = if paint_id.is_empty() {
                        (*index < paints.len()).then_some(*index)
                    } else {
                        paints.iter().position(|paint| paint.id == *paint_id)
                    };
                    resolved_index
                        .and_then(|index| paints.get_mut(index))
                        .zip(resolved_style.as_ref())
                        .is_some_and(|(paint, style)| {
                            apply_story_color_style(paint, color_target, style)
                        })
                });
                self.design_last_action = if applied {
                    format!(
                        "Host applied color style {} to {} paint {} on {node_id}",
                        style.style_id,
                        collection.label(),
                        if paint_id.is_empty() {
                            format!("#{index}")
                        } else {
                            paint_id.to_string()
                        }
                    )
                    .into()
                } else {
                    format!("Host rejected unavailable color style on {node_id}").into()
                };
            }
            DesignPanelAction::PaintColorStyleCreateRequested {
                node_id,
                collection,
                paint_id,
                index,
                color_target,
                color,
                ..
            } => {
                self.design_last_action = format!(
                    "Host opened color-style creation for #{} on {:?} in {} paint {} on {node_id}",
                    color.hex(),
                    color_target,
                    collection.label(),
                    if paint_id.is_empty() {
                        format!("#{index}")
                    } else {
                        paint_id.to_string()
                    }
                )
                .into();
            }
            DesignPanelAction::PaintEyedropperRequested {
                node_id,
                collection,
                paint_id,
                index,
                color_target,
                ..
            } => {
                self.design_last_action = format!(
                    "Host opened the eyedropper for {:?} in {} paint {} on {node_id}",
                    color_target,
                    collection.label(),
                    if paint_id.is_empty() {
                        format!("#{index}")
                    } else {
                        paint_id.to_string()
                    }
                )
                .into();
            }
            DesignPanelAction::ResetInstanceOverridesRequested { node_id } => {
                for property in &mut node.component_properties {
                    if property.reset_state.is_overridden() {
                        property.reset_to_default();
                    }
                }
                if let Some(context) = node.component_context.as_mut() {
                    context.overrides.nested_override_count = 0;
                }
                refresh_story_component_override_summary(node);
                self.design_last_action = format!("Host reset overrides for {node_id}").into();
            }
            DesignPanelAction::GoToMainComponentRequested { node_id } => {
                self.design_last_action =
                    format!("Host navigated to the main component for {node_id}").into();
            }
            DesignPanelAction::DetachInstanceRequested { node_id } => {
                self.design_last_action = format!("Host detached instance {node_id}").into();
            }
            DesignPanelAction::SwapStrokeEndpointsRequested { node_id } => {
                if let Some(stroke) = node.stroke.as_mut() {
                    std::mem::swap(&mut stroke.start_cap, &mut stroke.end_cap);
                    self.design_last_action =
                        format!("Host swapped stroke endpoints for {node_id}").into();
                } else {
                    self.design_last_action =
                        format!("Ignored stroke endpoint swap for {node_id}").into();
                }
            }
            DesignPanelAction::ComponentPropertyDefinitionCreateRequested {
                node_id,
                kind,
                name,
                description,
                documentation_links,
                definition,
                default_variable_id,
                partition,
                expected_property_order,
                after_property_id,
            } => {
                if !story_component_property_create_is_current(
                    node,
                    *kind,
                    name,
                    description,
                    documentation_links,
                    definition,
                    *partition,
                    expected_property_order,
                    after_property_id.as_ref(),
                ) || !story_component_definition_references_are_current(
                    &self.design_component_swaps,
                    definition,
                ) {
                    self.design_last_action =
                        format!("Host rejected stale component-property create on {node_id}")
                            .into();
                    cx.notify();
                    return;
                }
                let Some(default_value_binding) = story_component_default_variable(
                    &self.design_property_variables,
                    *kind,
                    default_variable_id.as_ref(),
                ) else {
                    self.design_last_action = format!(
                        "Host rejected stale default-variable binding for component-property create on {node_id}"
                    )
                    .into();
                    cx.notify();
                    return;
                };
                let property_id = next_story_component_property_id(node, *kind);
                let mut property = match definition {
                    DesignComponentPropertyDefinition::Variant {
                        default_value,
                        options,
                    } => DesignComponentProperty::variant(
                        property_id.clone(),
                        name.clone(),
                        default_value.clone(),
                        default_value.clone(),
                        options.clone(),
                    ),
                    DesignComponentPropertyDefinition::Boolean { default_value } => {
                        DesignComponentProperty::boolean(
                            property_id.clone(),
                            name.clone(),
                            *default_value,
                            *default_value,
                        )
                    }
                    DesignComponentPropertyDefinition::Text {
                        default_value,
                        multiline,
                    } => DesignComponentProperty::text(
                        property_id.clone(),
                        name.clone(),
                        default_value.clone(),
                        default_value.clone(),
                    )
                    .with_multiline(*multiline),
                    DesignComponentPropertyDefinition::InstanceSwap {
                        default_value,
                        preferred_values,
                    } => DesignComponentProperty::instance_swap(
                        property_id.clone(),
                        name.clone(),
                        default_value.clone(),
                        default_value.clone(),
                        preferred_values.clone(),
                    ),
                    DesignComponentPropertyDefinition::Slot {
                        default_value,
                        settings,
                    } => DesignComponentProperty::slot(
                        property_id.clone(),
                        name.clone(),
                        default_value.clone(),
                        default_value.clone(),
                        settings.clone(),
                        DesignSlotState::default(),
                    ),
                };
                property.description = description.clone();
                property.documentation_links = documentation_links.clone();
                property.default_value_binding = default_value_binding;
                property.refresh_slot_violations();
                let insert_index = if *kind == DesignComponentPropertyKind::Variant {
                    node.component_properties
                        .iter()
                        .take_while(|property| {
                            property.definition.kind() == DesignComponentPropertyKind::Variant
                        })
                        .count()
                } else {
                    node.component_properties.len()
                };
                let variant_option_labels = property.definition.option_labels();
                node.component_properties.insert(insert_index, property);
                if let Some(authoring) = node
                    .component_context
                    .as_mut()
                    .and_then(|context| context.authoring.as_mut())
                {
                    let mut definition =
                        DesignComponentPropertyDefinitionAuthoring::editable(property_id.clone());
                    if *kind == DesignComponentPropertyKind::Variant {
                        definition = definition.with_variant_options(
                            variant_option_labels.clone().into_iter().enumerate().map(
                                |(index, option)| {
                                    DesignComponentVariantOptionAuthoring::editable(
                                        format!("{property_id}-option-{index}"),
                                        option,
                                    )
                                },
                            ),
                        );
                    }
                    authoring.definitions.push(definition);
                }
                self.design_last_action =
                    format!("Host created {} on {node_id}", kind.label()).into();
            }
            DesignPanelAction::ComponentPropertyDefinitionRenameRequested {
                node_id,
                property_id,
                original_name,
                expected_name,
                name,
                phase,
            } => {
                let key = (node_id.clone(), property_id.clone());
                let renamed = node
                    .component_properties
                    .iter_mut()
                    .find(|property| &property.id == property_id)
                    .is_some_and(|property| {
                        apply_story_component_authoring_name_edit(
                            &mut property.name,
                            &mut self.design_component_property_name_edits,
                            key,
                            original_name,
                            expected_name,
                            name,
                            *phase,
                        )
                    });
                self.design_last_action = if renamed {
                    format!("Host echoed {phase:?} rename for {property_id} on {node_id}").into()
                } else {
                    format!("Host rejected stale rename for {property_id} on {node_id}").into()
                };
            }
            DesignPanelAction::ComponentPropertyDefinitionMetadataEditRequested {
                node_id,
                property_id,
                expected_description,
                description,
                expected_documentation_links,
                documentation_links,
            } => {
                let can_edit = node
                    .component_context
                    .as_ref()
                    .and_then(|context| context.authoring.as_ref())
                    .and_then(|authoring| authoring.definition(property_id.as_ref()))
                    .is_some_and(|definition| definition.capabilities.edit_metadata);
                let metadata_is_valid = description
                    .as_ref()
                    .is_none_or(|description| !description.trim().is_empty())
                    && documentation_links
                        .iter()
                        .all(|link| !link.label.trim().is_empty() && !link.url.trim().is_empty());
                let edited = can_edit
                    && metadata_is_valid
                    && node
                        .component_properties
                        .iter_mut()
                        .find(|property| &property.id == property_id)
                        .filter(|property| {
                            property.description == *expected_description
                                && property.documentation_links == *expected_documentation_links
                        })
                        .is_some_and(|property| {
                            property.description = description.clone();
                            property.documentation_links = documentation_links.clone();
                            true
                        });
                self.design_last_action = if edited {
                    format!("Host edited metadata for {property_id} on {node_id}").into()
                } else {
                    format!("Host rejected stale metadata edit for {property_id} on {node_id}")
                        .into()
                };
            }
            DesignPanelAction::ComponentPropertyDefinitionEditRequested {
                node_id,
                property_id,
                expected_description,
                description,
                expected_documentation_links,
                documentation_links,
                expected_definition,
                definition,
            } => {
                let edited = apply_story_component_property_definition_edit(
                    node,
                    &self.design_component_swaps,
                    property_id,
                    expected_description,
                    description,
                    expected_documentation_links,
                    documentation_links,
                    expected_definition,
                    definition,
                );
                self.design_last_action = if edited {
                    format!("Host atomically edited component property {property_id} on {node_id}")
                        .into()
                } else {
                    format!(
                        "Host rejected stale atomic component-property edit for {property_id} on {node_id}"
                    )
                    .into()
                };
            }
            DesignPanelAction::ComponentPropertyDefinitionDeleteRequested {
                node_id,
                property_id,
                expected_name,
            } => {
                if !story_component_property_delete_is_current(
                    node,
                    property_id.as_ref(),
                    expected_name.as_ref(),
                ) {
                    self.design_last_action =
                        format!("Host rejected stale delete for {property_id} on {node_id}").into();
                    cx.notify();
                    return;
                }
                node.component_properties
                    .retain(|property| &property.id != property_id);
                self.design_component_property_name_edits
                    .remove(&(node_id.clone(), property_id.clone()));
                invalidate_story_component_property_reorders(
                    &mut self.design_component_property_reorders,
                    node_id,
                    property_id,
                );
                self.design_component_variant_option_name_edits.retain(
                    |(active_node_id, active_property_id, _), _| {
                        active_node_id != node_id || active_property_id != property_id
                    },
                );
                self.design_component_variant_option_reorders.retain(
                    |(active_node_id, active_property_id, _), _| {
                        active_node_id != node_id || active_property_id != property_id
                    },
                );
                if let Some(authoring) = node
                    .component_context
                    .as_mut()
                    .and_then(|context| context.authoring.as_mut())
                {
                    authoring
                        .definitions
                        .retain(|definition| &definition.property_id != property_id);
                    for control in &mut authoring.applied_properties {
                        control
                            .candidates
                            .retain(|candidate| &candidate.property_id != property_id);
                        if control.applied_property_id.as_ref() == Some(property_id) {
                            control.applied_property_id = None;
                        }
                    }
                }
                self.design_last_action =
                    format!("Host deleted {property_id} from {node_id}").into();
            }
            DesignPanelAction::ComponentPropertyDefinitionReorderRequested {
                node_id,
                property_id,
                partition,
                original_property_order,
                expected_property_order,
                before_property_id,
                phase,
            } => {
                let current_order = node
                    .component_properties
                    .iter()
                    .filter(|property| {
                        DesignComponentPropertyPartition::for_kind(property.definition.kind())
                            == *partition
                    })
                    .map(|property| property.id.clone())
                    .collect::<Vec<_>>();
                let key = (node_id.clone(), property_id.clone());
                let Some(desired_order) = apply_story_component_authoring_reorder(
                    &current_order,
                    &mut self.design_component_property_reorders,
                    key,
                    property_id,
                    original_property_order,
                    expected_property_order,
                    before_property_id.as_ref(),
                    *phase,
                ) else {
                    self.design_last_action =
                        format!("Host rejected stale or unbalanced {phase:?} reorder for {property_id} on {node_id}").into();
                    cx.notify();
                    return;
                };
                if echo_story_component_property_order(node, *partition, &desired_order) {
                    self.design_last_action =
                        format!("Host echoed {phase:?} reorder for {property_id} on {node_id}")
                            .into();
                } else {
                    self.design_last_action =
                        format!("Host failed to echo reorder for {property_id} on {node_id}")
                            .into();
                }
            }
            DesignPanelAction::ComponentVariantOptionCreateRequested {
                node_id,
                property_id,
                name,
                expected_option_order,
                after_option_id,
            } => {
                if !story_component_variant_option_create_is_current(
                    node,
                    property_id.as_ref(),
                    name,
                    expected_option_order,
                    after_option_id.as_ref(),
                ) {
                    self.design_last_action = format!(
                        "Host rejected stale Variant-value create on {property_id} in {node_id}"
                    )
                    .into();
                    cx.notify();
                    return;
                }
                let option_id = next_story_component_variant_option_id(node, property_id.as_ref());
                let option_name = name.clone();
                if let Some(property) = node
                    .component_properties
                    .iter_mut()
                    .find(|property| &property.id == property_id)
                    && let DesignComponentPropertyDefinition::Variant { options, .. } =
                        &mut property.definition
                {
                    options.push(option_name.clone());
                    property.preferred_values = property.definition.option_labels();
                }
                if let Some(definition) = node
                    .component_context
                    .as_mut()
                    .and_then(|context| context.authoring.as_mut())
                    .and_then(|authoring| {
                        authoring
                            .definitions
                            .iter_mut()
                            .find(|definition| &definition.property_id == property_id)
                    })
                {
                    definition.variant_options.push(
                        DesignComponentVariantOptionAuthoring::editable(option_id, option_name),
                    );
                }
                self.design_last_action =
                    format!("Host created a Variant option on {property_id} in {node_id}").into();
            }
            DesignPanelAction::ComponentVariantOptionRenameRequested {
                node_id,
                property_id,
                option_id,
                original_name,
                expected_name,
                name,
                phase,
            } => {
                let key = (node_id.clone(), property_id.clone(), option_id.clone());
                let edited = node
                    .component_context
                    .as_mut()
                    .and_then(|context| context.authoring.as_mut())
                    .and_then(|authoring| {
                        authoring
                            .definitions
                            .iter_mut()
                            .find(|definition| &definition.property_id == property_id)
                    })
                    .and_then(|definition| {
                        definition
                            .variant_options
                            .iter_mut()
                            .find(|option| &option.id == option_id)
                    })
                    .is_some_and(|option| {
                        apply_story_component_authoring_name_edit(
                            &mut option.name,
                            &mut self.design_component_variant_option_name_edits,
                            key,
                            original_name,
                            expected_name,
                            name,
                            *phase,
                        )
                    });
                if edited {
                    let option_index_and_name = node
                        .component_context
                        .as_ref()
                        .and_then(|context| context.authoring.as_ref())
                        .and_then(|authoring| authoring.definition(property_id.as_ref()))
                        .and_then(|definition| {
                            definition
                                .variant_options
                                .iter()
                                .position(|option| &option.id == option_id)
                                .map(|index| {
                                    (index, definition.variant_options[index].name.clone())
                                })
                        });
                    if let Some((option_index, next_name)) = option_index_and_name
                        && let Some(property) = node
                            .component_properties
                            .iter_mut()
                            .find(|property| &property.id == property_id)
                        && let DesignComponentPropertyDefinition::Variant { options, .. } =
                            &mut property.definition
                        && let Some(option) = options.get_mut(option_index)
                    {
                        *option = next_name;
                        property.preferred_values = property.definition.option_labels();
                    }
                    self.design_last_action =
                        format!("Host echoed {phase:?} rename for {option_id} on {node_id}").into();
                } else {
                    self.design_last_action =
                        format!("Host rejected stale Variant-value rename for {option_id}").into();
                }
            }
            DesignPanelAction::ComponentVariantOptionDeleteRequested {
                node_id,
                property_id,
                option_id,
                expected_name,
            } => {
                if !story_component_variant_option_delete_is_current(
                    node,
                    property_id.as_ref(),
                    option_id.as_ref(),
                    expected_name.as_ref(),
                ) {
                    self.design_last_action =
                        format!("Host rejected stale Variant-value delete for {option_id}").into();
                    cx.notify();
                    return;
                }
                let option_index = node
                    .component_context
                    .as_ref()
                    .and_then(|context| context.authoring.as_ref())
                    .and_then(|authoring| authoring.definition(property_id.as_ref()))
                    .and_then(|definition| {
                        definition
                            .variant_options
                            .iter()
                            .position(|option| &option.id == option_id)
                    });
                if let Some(option_index) = option_index {
                    self.design_component_variant_option_name_edits.remove(&(
                        node_id.clone(),
                        property_id.clone(),
                        option_id.clone(),
                    ));
                    invalidate_story_component_variant_option_reorders(
                        &mut self.design_component_variant_option_reorders,
                        node_id,
                        property_id,
                    );
                    if let Some(property) = node
                        .component_properties
                        .iter_mut()
                        .find(|property| &property.id == property_id)
                        && let DesignComponentPropertyDefinition::Variant { options, .. } =
                            &mut property.definition
                        && option_index < options.len()
                    {
                        options.remove(option_index);
                        property.preferred_values = property.definition.option_labels();
                    }
                    if let Some(definition) = node
                        .component_context
                        .as_mut()
                        .and_then(|context| context.authoring.as_mut())
                        .and_then(|authoring| {
                            authoring
                                .definitions
                                .iter_mut()
                                .find(|definition| &definition.property_id == property_id)
                        })
                        && option_index < definition.variant_options.len()
                    {
                        definition.variant_options.remove(option_index);
                    }
                    self.design_last_action =
                        format!("Host deleted {option_id} on {node_id}").into();
                }
            }
            DesignPanelAction::ComponentVariantOptionReorderRequested {
                node_id,
                property_id,
                option_id,
                original_option_order,
                expected_option_order,
                before_option_id,
                phase,
            } => {
                let current_order = node
                    .component_context
                    .as_ref()
                    .and_then(|context| context.authoring.as_ref())
                    .and_then(|authoring| authoring.definition(property_id.as_ref()))
                    .map(|definition| {
                        definition
                            .variant_options
                            .iter()
                            .map(|option| option.id.clone())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let key = (node_id.clone(), property_id.clone(), option_id.clone());
                let Some(desired_order) = apply_story_component_authoring_reorder(
                    &current_order,
                    &mut self.design_component_variant_option_reorders,
                    key,
                    option_id,
                    original_option_order,
                    expected_option_order,
                    before_option_id.as_ref(),
                    *phase,
                ) else {
                    self.design_last_action =
                        format!("Host rejected stale or unbalanced {phase:?} Variant-value reorder for {option_id}").into();
                    cx.notify();
                    return;
                };
                if echo_story_component_variant_option_order(
                    node,
                    property_id.as_ref(),
                    &desired_order,
                ) {
                    self.design_last_action =
                        format!("Host echoed {phase:?} reorder for {option_id} on {node_id}")
                            .into();
                } else {
                    self.design_last_action =
                        format!("Host failed to echo Variant-value reorder for {option_id}").into();
                }
            }
            DesignPanelAction::ComponentPropertyApplyToLayerRequested {
                node_id,
                control_id,
                property_id,
                ..
            }
            | DesignPanelAction::ComponentPropertySwitchOnLayerRequested {
                node_id,
                control_id,
                to_property_id: property_id,
                ..
            } => {
                if let Some(control) = node
                    .component_context
                    .as_mut()
                    .and_then(|context| context.authoring.as_mut())
                    .and_then(|authoring| {
                        authoring
                            .applied_properties
                            .iter_mut()
                            .find(|control| &control.control_id == control_id)
                    })
                {
                    control.applied_property_id = Some(property_id.clone());
                    self.design_last_action =
                        format!("Host applied {property_id} to a layer in {node_id}").into();
                }
            }
            DesignPanelAction::ComponentPropertyDetachFromLayerRequested {
                node_id,
                control_id,
                ..
            } => {
                if let Some(control) = node
                    .component_context
                    .as_mut()
                    .and_then(|context| context.authoring.as_mut())
                    .and_then(|authoring| {
                        authoring
                            .applied_properties
                            .iter_mut()
                            .find(|control| &control.control_id == control_id)
                    })
                {
                    control.applied_property_id = None;
                    self.design_last_action =
                        format!("Host detached a property from a layer in {node_id}").into();
                }
            }
            DesignPanelAction::NestedComponentPropertyExposeRequested {
                node_id,
                candidate_id,
                ..
            } => {
                if let Some(candidate) = node
                    .component_context
                    .as_mut()
                    .and_then(|context| context.authoring.as_mut())
                    .and_then(|authoring| {
                        authoring
                            .exposure_candidates
                            .iter_mut()
                            .find(|candidate| &candidate.candidate_id == candidate_id)
                    })
                {
                    candidate.exposed_property_id = Some(format!("exposed-{candidate_id}").into());
                    self.design_last_action =
                        format!("Host exposed {candidate_id} on {node_id}").into();
                }
            }
            DesignPanelAction::NestedComponentPropertyUnexposeRequested {
                node_id,
                candidate_id,
                ..
            } => {
                if let Some(candidate) = node
                    .component_context
                    .as_mut()
                    .and_then(|context| context.authoring.as_mut())
                    .and_then(|authoring| {
                        authoring
                            .exposure_candidates
                            .iter_mut()
                            .find(|candidate| &candidate.candidate_id == candidate_id)
                    })
                {
                    candidate.exposed_property_id = None;
                    self.design_last_action =
                        format!("Host unexposed {candidate_id} on {node_id}").into();
                }
            }
            DesignPanelAction::NestedComponentPropertyPreviewRequested {
                node_id,
                candidate_id,
                preview,
                ..
            } => {
                self.design_last_action = format!(
                    "Host {} preview for {candidate_id} on {node_id}",
                    if *preview { "started" } else { "ended" }
                )
                .into();
            }
            DesignPanelAction::PropertyCopyRequested { .. }
            | DesignPanelAction::DimensionLimitsPreviewRequested { .. }
            | DesignPanelAction::MenuPreviewRequested { .. }
            | DesignPanelAction::ViewerSectionCopyRequested { .. }
            | DesignPanelAction::ViewerSectionRepresentationChangeRequested { .. }
            | DesignPanelAction::SurfaceChangeRequested { .. }
            | DesignPanelAction::SelectionHeaderCommandRequested { .. }
            | DesignPanelAction::AddAutoLayoutRequested { .. }
            | DesignPanelAction::SmartSelectionSpacingEditRequested { .. }
            | DesignPanelAction::SmartSelectionArrangeRequested { .. }
            | DesignPanelAction::TargetedNodeActionRequested { .. }
            | DesignPanelAction::ArrangeRequested { .. }
            | DesignPanelAction::TransformRequested { .. }
            | DesignPanelAction::SectionShareRequested { .. }
            | DesignPanelAction::SectionResolveChangedStatusRequested { .. }
            | DesignPanelAction::TransformModifierAddRequested { .. }
            | DesignPanelAction::TransformModifierRemoveRequested { .. }
            | DesignPanelAction::TransformModifierChangeRequested { .. }
            | DesignPanelAction::ApplyTransformModifiersRequested { .. }
            | DesignPanelAction::ResizeToFitRequested { .. }
            | DesignPanelAction::FramePresetApplyRequested { .. }
            | DesignPanelAction::ExportConfigurationAddRequested { .. }
            | DesignPanelAction::ExportConfigurationRemoveRequested { .. }
            | DesignPanelAction::ExportConfigurationChangeRequested { .. }
            | DesignPanelAction::ExportModeChangeRequested { .. }
            | DesignPanelAction::AnimatedExportChangeRequested { .. }
            | DesignPanelAction::AnimatedExportRequested { .. }
            | DesignPanelAction::ExportAllRequested { .. }
            | DesignPanelAction::ExportPreviewRequested { .. }
            | DesignPanelAction::PageBackgroundEditRequested { .. }
            | DesignPanelAction::PageBackgroundChangeRequested { .. }
            | DesignPanelAction::LocalResourceBrowseRequested { .. }
            | DesignPanelAction::LocalResourceOpenRequested { .. }
            | DesignPanelAction::LocalResourceCreateRequested { .. }
            | DesignPanelAction::LocalResourceImportRequested { .. }
            | DesignPanelAction::LocalStyleCommandRequested { .. }
            | DesignPanelAction::LocalStyleCreateRequested { .. }
            | DesignPanelAction::LocalStyleFolderCreateRequested { .. }
            | DesignPanelAction::LocalStylesDeleteRequested { .. }
            | DesignPanelAction::LocalStylesMoveRequested { .. }
            | DesignPanelAction::VariablesViewOpenRequested { .. }
            | DesignPanelAction::VariableModeApplyRequested { .. }
            | DesignPanelAction::VariableModeClearRequested { .. }
            | DesignPanelAction::SelectionColorEditRequested { .. }
            | DesignPanelAction::SelectionColorPaintEditRequested { .. }
            | DesignPanelAction::SelectionColorOccurrencesSelectRequested { .. }
            | DesignPanelAction::SelectionColorPaintStyleApplyRequested { .. }
            | DesignPanelAction::SelectionColorPaintStyleImportRequested { .. }
            | DesignPanelAction::SelectionColorPaintStyleCreateRequested { .. }
            | DesignPanelAction::SelectionColorPaintStyleDetachRequested { .. }
            | DesignPanelAction::SelectionColorVariableApplyRequested { .. }
            | DesignPanelAction::SelectionColorVariableImportRequested { .. }
            | DesignPanelAction::SelectionColorVariableCreateRequested { .. }
            | DesignPanelAction::SelectionColorVariableDetachRequested { .. } => unreachable!(),
            DesignPanelAction::PaintChangeRequested { .. }
            | DesignPanelAction::ExportRequested { .. }
            | DesignPanelAction::ReplaceMediaRequested { .. } => {
                unreachable!("compatibility actions return before reducer dispatch")
            }
        }
        if let Some((target, phase)) = node_edit_transaction {
            finish_story_node_edit(
                &mut self.design_nodes[node_index],
                &mut self.design_node_edit_snapshots,
                target,
                phase,
            );
        }
        self.apply_design_inspection_context(panel, cx);
        cx.notify();
    }

    fn handle_toolbar_action(
        &mut self,
        toolbar: Entity<EditorToolbar>,
        action: &ToolbarAction,
        cx: &mut Context<Self>,
    ) {
        match action {
            ToolbarAction::ModeChangeRequested { mode } => {
                self.toolbar_mode = *mode;
                self.toolbar_tool = match mode {
                    ToolbarMode::Design => ToolbarTool::Move,
                    ToolbarMode::Draw => ToolbarTool::Pencil,
                    ToolbarMode::Motion => ToolbarTool::MotionSelect,
                    ToolbarMode::Dev => ToolbarTool::Inspect,
                };
                toolbar.update(cx, |toolbar, cx| {
                    toolbar.set_mode(*mode, cx);
                    toolbar.set_active_tool(self.toolbar_tool, cx);
                });
                self.set_design_workspace_mode(
                    if *mode == ToolbarMode::Draw {
                        DesignPanelWorkspaceMode::Draw
                    } else {
                        DesignPanelWorkspaceMode::Design
                    },
                    "Toolbar",
                    cx,
                );
                self.toolbar_last_action = format!("Host switched to {} mode", mode.label()).into();
            }
            ToolbarAction::ToolChangeRequested { mode, tool } => {
                self.toolbar_mode = *mode;
                self.toolbar_tool = *tool;
                toolbar.update(cx, |toolbar, cx| {
                    toolbar.set_mode(*mode, cx);
                    toolbar.set_active_tool(*tool, cx);
                });
                self.set_design_workspace_mode(
                    if *mode == ToolbarMode::Draw {
                        DesignPanelWorkspaceMode::Draw
                    } else {
                        DesignPanelWorkspaceMode::Design
                    },
                    "Toolbar",
                    cx,
                );
                self.toolbar_last_action =
                    format!("Host selected {} in {}", tool.label(), mode.label()).into();
            }
            ToolbarAction::SecondaryControlInvoked { mode, control } => {
                let tool = match control {
                    ToolbarSecondaryControl::DevInspect => Some(ToolbarTool::Inspect),
                    ToolbarSecondaryControl::DevAnnotate => Some(ToolbarTool::Annotation),
                    ToolbarSecondaryControl::DevMeasure => Some(ToolbarTool::Measure),
                    ToolbarSecondaryControl::MotionAddKeyframe => {
                        self.toolbar_last_action = "Host inserted a keyframe at 0.0s".into();
                        None
                    }
                    _ => None,
                };
                if let Some(tool) = tool {
                    self.toolbar_tool = tool;
                    toolbar.update(cx, |toolbar, cx| {
                        toolbar.set_active_tool(tool, cx);
                    });
                }
                if !matches!(control, ToolbarSecondaryControl::MotionAddKeyframe) {
                    self.toolbar_last_action =
                        format!("Host invoked {} in {}", control.label(), mode.label()).into();
                }
            }
            ToolbarAction::ControlChangeRequested {
                mode,
                control,
                value,
            } => {
                match (control, value) {
                    (
                        ToolbarSecondaryControl::DrawStrokeColor,
                        ToolbarControlValue::Color(value),
                    ) => self.toolbar_draw_options.stroke_color = value.clone(),
                    (
                        ToolbarSecondaryControl::DrawBrushStyle,
                        ToolbarControlValue::Choice(value),
                    ) => self.toolbar_draw_options.brush_style = value.clone(),
                    (
                        ToolbarSecondaryControl::DrawStrokeWeight,
                        ToolbarControlValue::Integer(value),
                    ) => {
                        self.toolbar_draw_options.stroke_weight =
                            u16::try_from((*value).max(1)).unwrap_or(1);
                    }
                    (
                        ToolbarSecondaryControl::DrawSmoothing,
                        ToolbarControlValue::Integer(value),
                    ) => {
                        self.toolbar_draw_options.smoothing =
                            u8::try_from((*value).clamp(0, 100)).unwrap_or(0);
                    }
                    (ToolbarSecondaryControl::DrawPressure, ToolbarControlValue::Toggle(value)) => {
                        self.toolbar_draw_options.pressure = *value
                    }
                    (
                        ToolbarSecondaryControl::DevReadyForDevelopment,
                        ToolbarControlValue::Toggle(value),
                    ) => self.toolbar_dev_options.ready_for_development = *value,
                    (
                        ToolbarSecondaryControl::MotionPlayPause,
                        ToolbarControlValue::Toggle(value),
                    ) => self.toolbar_motion_options.playing = *value,
                    (ToolbarSecondaryControl::MotionLoop, ToolbarControlValue::Toggle(value)) => {
                        self.toolbar_motion_options.looping = *value
                    }
                    (
                        ToolbarSecondaryControl::MotionAutoKeyframe,
                        ToolbarControlValue::Toggle(value),
                    ) => self.toolbar_motion_options.auto_keyframe = *value,
                    (
                        ToolbarSecondaryControl::MotionAnimationStyle,
                        ToolbarControlValue::Choice(value),
                    ) => self.toolbar_motion_options.animation_style = value.clone(),
                    _ => {}
                }
                toolbar.update(cx, |toolbar, cx| {
                    toolbar.set_draw_options(self.toolbar_draw_options.clone(), cx);
                    toolbar.set_dev_options(self.toolbar_dev_options, cx);
                    toolbar.set_motion_options(self.toolbar_motion_options.clone(), cx);
                });
                self.toolbar_last_action = format!(
                    "Host applied {} = {value:?} in {}",
                    control.label(),
                    mode.label()
                )
                .into();
            }
            ToolbarAction::CommandQueryChanged { query } => {
                self.toolbar_last_action = if query.is_empty() {
                    "Actions search cleared".into()
                } else {
                    format!("Actions query: “{query}”").into()
                };
            }
            ToolbarAction::CommandInvoked { command } => {
                let requested_mode = match command {
                    ToolbarCommand::OpenDesignMode => Some(ToolbarMode::Design),
                    ToolbarCommand::OpenDrawMode => Some(ToolbarMode::Draw),
                    ToolbarCommand::OpenMotionMode => Some(ToolbarMode::Motion),
                    ToolbarCommand::OpenDevMode => Some(ToolbarMode::Dev),
                    _ => None,
                };
                if let Some(mode) = requested_mode {
                    self.toolbar_mode = mode;
                    self.toolbar_tool = match mode {
                        ToolbarMode::Design => ToolbarTool::Move,
                        ToolbarMode::Draw => ToolbarTool::Pencil,
                        ToolbarMode::Motion => ToolbarTool::MotionSelect,
                        ToolbarMode::Dev => ToolbarTool::Inspect,
                    };
                    toolbar.update(cx, |toolbar, cx| {
                        toolbar.set_mode(mode, cx);
                        toolbar.set_active_tool(self.toolbar_tool, cx);
                    });
                    self.set_design_workspace_mode(
                        if mode == ToolbarMode::Draw {
                            DesignPanelWorkspaceMode::Draw
                        } else {
                            DesignPanelWorkspaceMode::Design
                        },
                        "Toolbar command",
                        cx,
                    );
                }
                self.toolbar_last_action =
                    format!("Host ran {} · {}", command.category(), command.label()).into();
            }
            ToolbarAction::AiPromptSubmitted { prompt } => {
                self.toolbar_last_action =
                    format!("Agent task started: “{prompt}” · working in parallel").into();
            }
            ToolbarAction::AgentVisibilityChanged { visible } => {
                self.toolbar_last_action = if *visible {
                    "Agent composer opened with Hero frame context".into()
                } else {
                    "Agent composer closed".into()
                };
            }
            ToolbarAction::AgentAttachmentRequested => {
                self.toolbar_last_action =
                    "Host opened the Agent context and attachment picker".into();
            }
            ToolbarAction::AgentVoiceInputRequested => {
                self.toolbar_last_action = "Host started Agent voice input".into();
            }
            ToolbarAction::ZoomChangeRequested { percent } => {
                self.toolbar_zoom = *percent;
                toolbar.update(cx, |toolbar, cx| {
                    toolbar.set_zoom_percent(*percent, cx);
                });
                self.toolbar_last_action = format!("Host set canvas zoom to {percent}%").into();
            }
        }
        cx.notify();
    }

    fn apply_layer_selection(&mut self, node_id: &SharedString, mode: LayersPanelSelectionMode) {
        match mode {
            LayersPanelSelectionMode::Replace => {
                self.selected_layers = vec![node_id.clone()];
                self.layer_selection_anchor = Some(node_id.clone());
            }
            LayersPanelSelectionMode::Toggle => {
                if let Some(index) = self
                    .selected_layers
                    .iter()
                    .position(|selected| selected == node_id)
                {
                    self.selected_layers.remove(index);
                } else {
                    self.selected_layers.push(node_id.clone());
                }
                self.layer_selection_anchor = Some(node_id.clone());
            }
            LayersPanelSelectionMode::Range => {
                let flattened = flatten_layer_ids(&self.layers);
                let anchor = self
                    .layer_selection_anchor
                    .as_ref()
                    .and_then(|anchor| flattened.iter().position(|id| id == anchor));
                let target = flattened.iter().position(|id| id == node_id);
                if let (Some(anchor), Some(target)) = (anchor, target) {
                    let start = anchor.min(target);
                    let end = anchor.max(target);
                    self.selected_layers = flattened[start..=end].to_vec();
                } else {
                    self.selected_layers = vec![node_id.clone()];
                    self.layer_selection_anchor = Some(node_id.clone());
                }
            }
        }
    }

    fn search(&self, request: &PagesPanelSearchRequest) -> PagesPanelSearchResults {
        if request.query.is_empty() {
            return PagesPanelSearchResults::default();
        }

        let query = request.query.as_ref();
        let query_for_match = if request.match_case {
            query.to_owned()
        } else {
            query.to_lowercase()
        };
        let query_matches = |title: &str| {
            let haystack = if request.match_case {
                title.to_owned()
            } else {
                title.to_lowercase()
            };
            if request.whole_words {
                haystack
                    .split(|character: char| !character.is_alphanumeric())
                    .any(|word| word == query_for_match)
            } else {
                haystack.contains(&query_for_match)
            }
        };
        let in_scope = |element: &&MockElement| {
            request.scope == PagesPanelSearchScope::AllPages || element.page_id == self.active_page
        };
        let query_results = self
            .elements
            .iter()
            .filter(in_scope)
            .filter(|element| query_matches(&element.result.title))
            .collect::<Vec<_>>();

        let mut element_counts = vec![PagesPanelElementCount::new(
            PagesPanelElementKind::All,
            query_results.len(),
        )];
        element_counts.extend(
            PagesPanelElementKind::FILTER_ORDER
                .into_iter()
                .filter(|kind| *kind != PagesPanelElementKind::All)
                .map(|kind| {
                    PagesPanelElementCount::new(
                        kind,
                        query_results
                            .iter()
                            .filter(|element| element.result.kind == kind)
                            .count(),
                    )
                }),
        );
        let items = query_results
            .into_iter()
            .filter(|element| {
                request.element_kinds.is_empty()
                    || request.element_kinds.contains(&element.result.kind)
            })
            .map(|element| element.result.clone())
            .collect::<Vec<_>>();

        PagesPanelSearchResults {
            total: items.len(),
            items,
            element_counts,
        }
    }

    fn focus_story(&self, kind: StoryKind, window: &mut Window, cx: &mut Context<Self>) {
        match kind {
            StoryKind::Icons => self.icon_gallery.focus_handle(cx).focus(window),
            StoryKind::Toolbar => self.toolbar.focus_handle(cx).focus(window),
            StoryKind::Pages => self.pages_panel.focus_handle(cx).focus(window),
            StoryKind::Layers => self.layers_panel.focus_handle(cx).focus(window),
            StoryKind::Design => self.design_panel.focus_handle(cx).focus(window),
            StoryKind::Variables => self.variables_page.focus_handle(cx).focus(window),
            StoryKind::Assets => self.assets_panel.focus_handle(cx).focus(window),
            StoryKind::Prototype => self.prototype_panel.focus_handle(cx).focus(window),
            StoryKind::Timeline => self.timeline.focus_handle(cx).focus(window),
            StoryKind::PseudoEditor => self.pseudo_editor.focus_handle(cx).focus(window),
        }
    }

    fn activate_gallery_story(
        &mut self,
        kind: StoryKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.active_story = kind;
        self.gallery_story_scroll_handle
            .set_offset(Default::default());
        self.focus_story(kind, window, cx);
        cx.notify();
    }

    fn render_navigation_item(
        &self,
        kind: StoryKind,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let active = self.active_story == kind;
        Button::new(SharedString::from(format!(
            "storybook-nav-{}",
            label.to_lowercase()
        )))
        .label(label)
        .custom(
            ButtonCustomVariant::new(cx)
                .color(cx.theme().transparent)
                .foreground(cx.theme().foreground)
                .border(cx.theme().transparent)
                .hover(cx.theme().accent)
                .active(cx.theme().selection.opacity(0.32)),
        )
        .selected(active)
        .h(px(32.))
        .px_3()
        .rounded(px(6.))
        .border_0()
        .cursor_pointer()
        .when(active, |item| {
            item.font_semibold()
                .hover(|style| style.bg(cx.theme().accent))
        })
        .on_click(cx.listener(move |this, _, window, cx| {
            this.activate_gallery_story(kind, window, cx);
        }))
        .into_any_element()
    }

    fn render_design_context_selector(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut buttons = h_flex().w_full().gap_2().flex_wrap();
        for scenario in DesignInspectionScenario::ALL {
            let active = self.design_inspection_scenario == scenario;
            buttons = buttons.child(
                Button::new(SharedString::from(format!(
                    "design-inspection-scenario-{}",
                    scenario.id()
                )))
                .label(scenario.label())
                .custom(
                    ButtonCustomVariant::new(cx)
                        .color(cx.theme().secondary)
                        .foreground(cx.theme().foreground)
                        .border(if active {
                            cx.theme().selection
                        } else {
                            cx.theme().border
                        })
                        .hover(cx.theme().accent)
                        .active(cx.theme().selection.opacity(0.22)),
                )
                .xsmall()
                .selected(active)
                .h(px(30.))
                .px_2()
                .rounded(px(5.))
                .border_1()
                .when(active, |button| {
                    button.hover(|style| style.bg(cx.theme().accent))
                })
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.activate_design_inspection_scenario(scenario, cx);
                    this.design_panel.focus_handle(cx).focus(window);
                    cx.notify();
                })),
            );
        }
        v_flex()
            .w_full()
            .gap_2()
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("INSPECTION CONTEXT"),
            )
            .child(buttons)
            .when(
                self.design_inspection_scenario == DesignInspectionScenario::TextEdit,
                |content| {
                    content.child(
                        Button::new("design-text-range-revision")
                            .custom(
                                ButtonCustomVariant::new(cx)
                                    .color(cx.theme().selection.opacity(0.14))
                                    .foreground(cx.theme().foreground)
                                    .border(cx.theme().selection)
                                    .hover(cx.theme().selection.opacity(0.24))
                                    .active(cx.theme().selection.opacity(0.14)),
                            )
                            .xsmall()
                            .selected(true)
                            .w_full()
                            .h(px(30.))
                            .px_2()
                            .rounded(px(5.))
                            .border_1()
                            .cursor_pointer()
                            .hover(|style| style.bg(cx.theme().selection.opacity(0.24)))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.advance_design_text_range(cx);
                                this.design_panel.focus_handle(cx).focus(window);
                            }))
                            .child(
                                h_flex()
                                    .w_full()
                                    .justify_between()
                                    .child(div().text_xs().child(format!(
                                        "Selected text · revision {}",
                                        self.design_text_range_revision
                                    )))
                                    .child(div().text_xs().child("Select next range")),
                            ),
                    )
                },
            )
            .into_any_element()
    }

    fn set_design_panel_width(
        &mut self,
        width: f32,
        interaction: &'static str,
        cx: &mut Context<Self>,
    ) {
        self.design_panel_resize_drag = None;
        self.design_panel_width = clamp_design_panel_width(width);
        self.design_last_action = format!(
            "Story {interaction} the inspector viewport to {:.0} px — no document intent",
            self.design_panel_width
        )
        .into();
        cx.notify();
    }

    fn begin_design_panel_resize(&mut self, pointer_x: f32, cx: &mut Context<Self>) {
        self.design_panel_resize_drag = Some(DesignPanelResizeDrag::new(
            pointer_x,
            self.design_panel_width,
        ));
        self.design_last_action =
            format!("Story began resizing at {:.0} px", self.design_panel_width).into();
        cx.notify();
    }

    fn update_design_panel_resize(&mut self, pointer_x: f32, cx: &mut Context<Self>) {
        let Some(drag) = self.design_panel_resize_drag else {
            return;
        };
        let width = drag.width_at(pointer_x);
        if (self.design_panel_width - width).abs() < f32::EPSILON {
            return;
        }
        self.design_panel_width = width;
        self.design_last_action =
            format!("Story is resizing the inspector to {width:.0} px").into();
        cx.notify();
    }

    fn finish_design_panel_resize(&mut self, cx: &mut Context<Self>) {
        if self.design_panel_resize_drag.take().is_none() {
            return;
        }
        self.design_last_action = format!(
            "Story resized the inspector viewport to {:.0} px — no document intent",
            self.design_panel_width
        )
        .into();
        cx.notify();
    }

    fn handle_design_panel_resize_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let modifiers = event.keystroke.modifiers;
        if modifiers.control || modifiers.alt || modifiers.platform || modifiers.function {
            return;
        }
        let Some(width) = design_panel_width_after_key(
            self.design_panel_width,
            event.keystroke.key.as_str(),
            modifiers.shift,
        ) else {
            return;
        };
        window.prevent_default();
        cx.stop_propagation();
        self.set_design_panel_width(width, "keyboard-resized", cx);
    }

    fn render_design_resize_handle(&self, cx: &mut Context<Self>) -> AnyElement {
        let active = self.design_panel_resize_drag.is_some();
        let tooltip = format!(
            "Resize Design inspector · {:.0} px · drag or use arrows and +/− (Shift for 32 px)",
            self.design_panel_width
        );
        Button::new("design-panel-resize-handle")
            .debug_selector(|| "design-panel-resize-handle".to_owned())
            .tooltip(tooltip)
            .xsmall()
            .compact()
            .selected(active)
            .w(px(10.))
            .h_full()
            .p_0()
            .cursor_col_resize()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    this.begin_design_panel_resize(f32::from(event.position.x), cx);
                }),
            )
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                this.handle_design_panel_resize_key(event, window, cx);
            }))
            .child(div().w(px(2.)).h(px(36.)).rounded_full().bg(if active {
                cx.theme().selection
            } else {
                cx.theme().border
            }))
            .into_any_element()
    }

    fn render_design_width_selector(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut buttons = h_flex().w_full().gap_2();
        for width in DESIGN_PANEL_WIDTH_PRESETS {
            let active = (self.design_panel_width - width).abs() < f32::EPSILON;
            buttons = buttons.child(
                Button::new(SharedString::from(format!("design-panel-width-{width:.0}")))
                    .label(format!("{width:.0}"))
                    .tooltip(format!("Set inspector width to {width:.0} px"))
                    .xsmall()
                    .compact()
                    .flex_1()
                    .selected(active)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set_design_panel_width(width, "set", cx);
                    })),
            );
        }
        v_flex()
            .w_full()
            .gap_2()
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("PANEL WIDTH · {:.0} PX", self.design_panel_width)),
            )
            .child(buttons)
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Drag the inspector divider, or use arrows and +/− (Shift: 32 px)."),
            )
            .into_any_element()
    }

    fn set_design_workspace_mode(
        &mut self,
        workspace_mode: DesignPanelWorkspaceMode,
        source: &'static str,
        cx: &mut Context<Self>,
    ) {
        self.design_workspace_mode = workspace_mode;
        self.apply_design_inspection_context(self.design_panel.clone(), cx);
        self.design_last_action = format!(
            "{source} set the inspector workspace to {} — surface and document state preserved",
            workspace_mode.label()
        )
        .into();
        cx.notify();
    }

    fn render_design_workspace_toggle(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut buttons = h_flex().w_full().gap_2();
        for workspace_mode in [
            DesignPanelWorkspaceMode::Design,
            DesignPanelWorkspaceMode::Draw,
        ] {
            buttons = buttons.child(
                Button::new(SharedString::from(format!(
                    "design-workspace-{}",
                    workspace_mode.label().to_ascii_lowercase()
                )))
                .label(workspace_mode.label())
                .tooltip(format!(
                    "Render the {} workspace without changing the accepted sidebar surface",
                    workspace_mode.label()
                ))
                .xsmall()
                .compact()
                .flex_1()
                .selected(self.design_workspace_mode == workspace_mode)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_design_workspace_mode(workspace_mode, "Story", cx);
                })),
            );
        }
        v_flex()
            .w_full()
            .gap_2()
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("INSPECTOR WORKSPACE"),
            )
            .child(buttons)
            .into_any_element()
    }

    fn render_design_additional_labels_toggle(&self, cx: &mut Context<Self>) -> AnyElement {
        let enabled = self.design_additional_labels;
        v_flex()
            .w_full()
            .gap_2()
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("VIEW PREFERENCE"),
            )
            .child(
                Button::new("design-additional-labels")
                    .label(format!(
                        "Additional labels · {}",
                        design_additional_labels_status(enabled)
                    ))
                    .tooltip(
                        "Show explanatory labels in compact Design controls (presentation only)",
                    )
                    .xsmall()
                    .compact()
                    .w_full()
                    .selected(enabled)
                    .on_click(cx.listener(|this, _, _, cx| {
                        let enabled = !this.design_additional_labels;
                        this.design_additional_labels = enabled;
                        this.design_panel.update(cx, |panel, cx| {
                            panel.set_additional_labels(enabled, cx);
                        });
                        this.design_last_action = format!(
                            "Story set Additional labels {} — no document intent",
                            design_additional_labels_status(enabled).to_ascii_lowercase()
                        )
                        .into();
                        cx.notify();
                    })),
            )
            .into_any_element()
    }

    fn render_design_nudge_preferences(&self, cx: &mut Context<Self>) -> AnyElement {
        let default = DesignNudgeSettings::default();
        let precise = DesignNudgeSettings::new(0.5, 8.).expect("valid Storybook nudge preset");
        let current = self.design_nudge_settings;
        v_flex()
            .w_full()
            .gap_2()
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("KEYBOARD NUDGE · SMALL / BIG"),
            )
            .child(
                h_flex()
                    .w_full()
                    .gap_2()
                    .child(
                        Button::new("design-nudge-default")
                            .label("Default · 1 / 10")
                            .tooltip("Use Figma's default small and big nudge values")
                            .xsmall()
                            .compact()
                            .flex_1()
                            .selected(current == default)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.design_nudge_settings = default;
                                this.design_panel.update(cx, |panel, cx| {
                                    panel.set_nudge_settings(default, cx);
                                });
                                this.design_last_action =
                                    "Story set keyboard nudge to 1 / 10 — no document intent"
                                        .into();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("design-nudge-precise")
                            .label("Custom · 0.5 / 8")
                            .tooltip("Use a precise custom small/big nudge preference")
                            .xsmall()
                            .compact()
                            .flex_1()
                            .selected(current == precise)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.design_nudge_settings = precise;
                                this.design_panel.update(cx, |panel, cx| {
                                    panel.set_nudge_settings(precise, cx);
                                });
                                this.design_last_action =
                                    "Story set keyboard nudge to 0.5 / 8 — no document intent"
                                        .into();
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(
                        "Arrow uses Small; Shift+Arrow uses Big across numeric inspector fields.",
                    ),
            )
            .into_any_element()
    }

    fn render_design_selector(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut buttons = h_flex().w_full().gap_2().flex_wrap();
        let last_index = self.design_nodes.len().saturating_sub(1);
        for (index, node) in self.design_nodes.iter().cloned().enumerate() {
            let selected = self.selected_design_node == index;
            let kind = node.kind;
            let label = node.name.clone();
            buttons = buttons.child(
                Button::new(SharedString::from(format!("design-preset-{index}")))
                    .debug_selector(move || format!("design-preset-{index}"))
                    .when(index == last_index, |button| {
                        button.debug_selector(|| "design-preset-last".to_owned())
                    })
                    .custom(
                        ButtonCustomVariant::new(cx)
                            .color(cx.theme().secondary)
                            .foreground(cx.theme().foreground)
                            .border(if selected {
                                cx.theme().selection
                            } else {
                                cx.theme().border
                            })
                            .hover(cx.theme().accent)
                            .active(cx.theme().selection.opacity(0.22)),
                    )
                    .xsmall()
                    .selected(selected)
                    .h(px(30.))
                    .px_2()
                    .rounded(px(5.))
                    .border_1()
                    .when(selected, |button| {
                        button.hover(|style| style.bg(cx.theme().accent))
                    })
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.selected_design_node = index;
                        this.design_inspection_scenario =
                            default_design_inspection_scenario_for_node(&node);
                        this.apply_design_inspection_context(this.design_panel.clone(), cx);
                        this.design_panel.focus_handle(cx).focus(window);
                        this.design_last_action = format!(
                            "Story selected {} preset in editable single context",
                            kind.label()
                        )
                        .into();
                        cx.notify();
                    }))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(16.))
                                    .text_center()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(kind.glyph()),
                            )
                            .child(div().text_xs().child(label)),
                    ),
            );
        }
        v_flex()
            .w_full()
            .gap_2()
            .child(
                div()
                    .debug_selector(|| "design-node-variation-matrix-heading".to_owned())
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("NODE + VARIATION MATRIX"),
            )
            .child(div().id("design-selector-scroll").pb_6().child(buttons))
            .into_any_element()
    }

    fn render_toolbar_demo_frame(&self, cx: &mut Context<Self>) -> AnyElement {
        let agent = self.toolbar.clone();
        let accent = cx.theme().selection;
        let accent_foreground = if (accent.l - cx.theme().foreground.l).abs()
            >= (accent.l - cx.theme().background.l).abs()
        {
            cx.theme().foreground
        } else {
            cx.theme().background
        };

        // These fixed colors belong to the mock document artwork inside the selected frame.
        // Editor chrome around the artwork uses active theme tokens below.
        let artwork = match self.toolbar_mode {
            ToolbarMode::Draw => h_flex()
                .size_full()
                .relative()
                .overflow_hidden()
                .bg(rgba(0xfff8ebff))
                .child(
                    div()
                        .absolute()
                        .left(px(72.))
                        .top(px(54.))
                        .size(px(142.))
                        .rounded_full()
                        .bg(rgba(0xff6b4aff)),
                )
                .child(
                    div()
                        .absolute()
                        .left(px(164.))
                        .top(px(112.))
                        .size(px(176.))
                        .rounded_full()
                        .bg(rgba(0x7c3aeddd)),
                )
                .child(
                    div()
                        .absolute()
                        .right(px(42.))
                        .top(px(44.))
                        .w(px(132.))
                        .h(px(220.))
                        .rounded(px(68.))
                        .bg(rgba(0xffd028ff)),
                )
                .child(
                    div()
                        .absolute()
                        .left(px(58.))
                        .bottom(px(52.))
                        .text_size(px(42.))
                        .font_semibold()
                        .text_color(rgba(0x1e1e1eff))
                        .child("MAKE"),
                )
                .child(
                    div()
                        .absolute()
                        .left(px(178.))
                        .bottom(px(18.))
                        .text_size(px(52.))
                        .font_semibold()
                        .text_color(rgba(0xffffffff))
                        .child("WAVES"),
                )
                .into_any_element(),
            _ => h_flex()
                .size_full()
                .bg(rgba(0xf7f8faff))
                .child(
                    v_flex()
                        .w(px(118.))
                        .h_full()
                        .p_3()
                        .gap_2()
                        .bg(rgba(0x111827ff))
                        .child(div().size(px(24.)).rounded(px(7.)).bg(rgba(0x6366f1ff)))
                        .child(
                            div()
                                .h(px(8.))
                                .w(px(68.))
                                .rounded_full()
                                .bg(rgba(0xffffff66)),
                        )
                        .child(
                            div()
                                .h(px(8.))
                                .w(px(82.))
                                .rounded_full()
                                .bg(rgba(0xffffff33)),
                        )
                        .child(
                            div()
                                .h(px(8.))
                                .w(px(56.))
                                .rounded_full()
                                .bg(rgba(0xffffff33)),
                        )
                        .child(div().flex_1())
                        .child(
                            h_flex()
                                .h(px(30.))
                                .px_2()
                                .rounded(px(8.))
                                .bg(rgba(0x6366f1ff))
                                .text_xs()
                                .text_color(rgba(0xffffffff))
                                .child("Upgrade"),
                        ),
                )
                .child(
                    v_flex()
                        .flex_1()
                        .min_w(px(0.))
                        .h_full()
                        .p_5()
                        .gap_3()
                        .child(
                            h_flex()
                                .child(
                                    v_flex()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgba(0x6b7280ff))
                                                .child("OVERVIEW"),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(22.))
                                                .font_semibold()
                                                .text_color(rgba(0x111827ff))
                                                .child("Good morning, Maya"),
                                        ),
                                )
                                .child(div().flex_1())
                                .child(div().size(px(32.)).rounded_full().bg(rgba(0xf59e0bff))),
                        )
                        .child(
                            h_flex()
                                .h(px(92.))
                                .gap_3()
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .h_full()
                                        .p_3()
                                        .rounded(px(12.))
                                        .bg(rgba(0x6366f1ff))
                                        .text_color(rgba(0xffffffff))
                                        .child(div().text_xs().child("Revenue"))
                                        .child(
                                            div()
                                                .mt_2()
                                                .text_size(px(20.))
                                                .font_semibold()
                                                .child("$48.2k"),
                                        ),
                                )
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .h_full()
                                        .p_3()
                                        .rounded(px(12.))
                                        .bg(rgba(0xffffffff))
                                        .border_1()
                                        .border_color(rgba(0x00000010))
                                        .text_color(rgba(0x111827ff))
                                        .child(div().text_xs().child("Customers"))
                                        .child(
                                            div()
                                                .mt_2()
                                                .text_size(px(20.))
                                                .font_semibold()
                                                .child("1,429"),
                                        ),
                                )
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .h_full()
                                        .p_3()
                                        .rounded(px(12.))
                                        .bg(rgba(0xffffffff))
                                        .border_1()
                                        .border_color(rgba(0x00000010))
                                        .text_color(rgba(0x111827ff))
                                        .child(div().text_xs().child("Conversion"))
                                        .child(
                                            div()
                                                .mt_2()
                                                .text_size(px(20.))
                                                .font_semibold()
                                                .child("12.8%"),
                                        ),
                                ),
                        )
                        .child(
                            v_flex()
                                .flex_1()
                                .min_h(px(0.))
                                .p_3()
                                .rounded(px(12.))
                                .bg(rgba(0xffffffff))
                                .border_1()
                                .border_color(rgba(0x00000010))
                                .child(
                                    h_flex()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_semibold()
                                                .text_color(rgba(0x111827ff))
                                                .child("Activity"),
                                        )
                                        .child(div().flex_1())
                                        .child(
                                            div()
                                                .px_2()
                                                .py_1()
                                                .rounded(px(6.))
                                                .bg(rgba(0xf3f4f6ff))
                                                .text_xs()
                                                .text_color(rgba(0x6b7280ff))
                                                .child("Last 30 days"),
                                        ),
                                )
                                .child(h_flex().flex_1().items_end().gap_2().pt_3().children(
                                    [44., 68., 38., 82., 58., 94., 74.].map(|height| {
                                        div()
                                            .flex_1()
                                            .h(px(height))
                                            .rounded_t(px(4.))
                                            .bg(rgba(0x6366f1cc))
                                    }),
                                )),
                        ),
                )
                .into_any_element(),
        };

        div()
            .relative()
            .w(px(596.))
            .h(px(360.))
            .rounded(px(3.))
            .border_2()
            .border_color(accent)
            .bg(cx.theme().background)
            .shadow_lg()
            .child(
                div()
                    .absolute()
                    .left(px(-2.))
                    .top(px(-24.))
                    .px_2()
                    .h(px(22.))
                    .flex()
                    .items_center()
                    .rounded_t(px(5.))
                    .bg(accent)
                    .text_xs()
                    .font_semibold()
                    .text_color(accent_foreground)
                    .child("Hero frame"),
            )
            .child(
                h_flex()
                    .id("toolbar-story-agent-context")
                    .absolute()
                    .right(px(-17.))
                    .top(px(-17.))
                    .size(px(34.))
                    .justify_center()
                    .rounded_full()
                    .border_2()
                    .border_color(cx.theme().background)
                    .bg(cx.theme().primary)
                    .text_color(cx.theme().primary_foreground)
                    .shadow_lg()
                    .cursor_pointer()
                    .hover(|style| style.bg(cx.theme().primary_hover))
                    .on_click(cx.listener(move |_this, _, window, cx| {
                        agent.update(cx, |toolbar, cx| {
                            toolbar.open_agent(window, cx);
                        });
                    }))
                    .child("✦"),
            )
            .child(artwork)
            .when(self.toolbar_mode == ToolbarMode::Dev, |frame| {
                frame
                    .child(
                        div()
                            .absolute()
                            .left(px(116.))
                            .top(px(118.))
                            .w(px(176.))
                            .h(px(1.))
                            .bg(cx.theme().danger),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(px(174.))
                            .top(px(99.))
                            .px_2()
                            .py_1()
                            .rounded(px(5.))
                            .bg(cx.theme().danger)
                            .text_xs()
                            .font_semibold()
                            .text_color(cx.theme().danger_foreground)
                            .child("176"),
                    )
            })
            .when(self.toolbar_mode == ToolbarMode::Motion, |frame| {
                frame.child(
                    h_flex()
                        .absolute()
                        .right(px(16.))
                        .top(px(16.))
                        .h(px(28.))
                        .px_2()
                        .gap_1()
                        .rounded(px(8.))
                        .bg(cx.theme().primary)
                        .text_xs()
                        .font_semibold()
                        .text_color(cx.theme().primary_foreground)
                        .child("◆  3 keyframes"),
                )
            })
            .into_any_element()
    }

    fn render_toolbar_timeline(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.toolbar_mode != ToolbarMode::Motion {
            return div().into_any_element();
        }
        let mut tracks = v_flex().flex_1().min_h(px(0.)).py_2();
        for (index, label) in ["Position", "Scale", "Opacity"].into_iter().enumerate() {
            tracks = tracks.child(
                h_flex()
                    .h(px(26.))
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .w(px(138.))
                            .px_3()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(label),
                    )
                    .child(
                        div()
                            .relative()
                            .flex_1()
                            .h_full()
                            .child(
                                div()
                                    .absolute()
                                    .left(px(68. + index as f32 * 26.))
                                    .top(px(4.))
                                    .text_color(cx.theme().selection)
                                    .child("◆"),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .left(px(218. + index as f32 * 34.))
                                    .top(px(4.))
                                    .text_color(cx.theme().selection)
                                    .child("◆"),
                            ),
                    ),
            );
        }
        v_flex()
            .absolute()
            .left_0()
            .right_0()
            .bottom_0()
            .h(px(142.))
            .bg(cx.theme().popover)
            .border_t_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .h(px(34.))
                    .px_3()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .text_xs()
                    .text_color(cx.theme().popover_foreground)
                    .child("Timeline")
                    .child(div().flex_1())
                    .child("0 ms       500       1000       1500       2000 ms"),
            )
            .child(tracks)
            .into_any_element()
    }

    fn render_toolbar_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let agent = self.toolbar.clone();
        let mode_accent = cx.theme().selection;

        h_flex()
            .flex_1()
            .min_h(px(0.))
            .child(
                v_flex()
                    .w(px(226.))
                    .h_full()
                    .bg(cx.theme().sidebar)
                    .text_color(cx.theme().sidebar_foreground)
                    .border_r_1()
                    .border_color(cx.theme().sidebar_border)
                    .child(
                        h_flex()
                            .h(px(48.))
                            .px_3()
                            .gap_2()
                            .border_b_1()
                            .border_color(cx.theme().sidebar_border)
                            .child(div().size(px(24.)).rounded(px(7.)).bg(cx.theme().primary))
                            .child(
                                v_flex()
                                    .gap(px(1.))
                                    .child(div().text_sm().font_semibold().child("Commerce OS"))
                                    .child(
                                        div()
                                            .text_size(px(10.))
                                            .text_color(cx.theme().muted_foreground)
                                            .child("Drafts / Toolbar clone"),
                                    ),
                            ),
                    )
                    .child(
                        h_flex()
                            .h(px(38.))
                            .px_3()
                            .gap_2()
                            .text_xs()
                            .font_semibold()
                            .child("File"),
                    )
                    .child(
                        h_flex()
                            .id("toolbar-story-agents")
                            .h(px(38.))
                            .mx_2()
                            .px_2()
                            .gap_2()
                            .rounded(px(7.))
                            .cursor_pointer()
                            .hover(|style| style.bg(cx.theme().sidebar_accent))
                            .on_click(cx.listener(move |_this, _, window, cx| {
                                agent.update(cx, |toolbar, cx| {
                                    toolbar.open_agent(window, cx);
                                });
                            }))
                            .child(
                                div()
                                    .w(px(20.))
                                    .text_center()
                                    .font_semibold()
                                    .text_color(cx.theme().primary)
                                    .child("✦"),
                            )
                            .child(div().flex_1().text_sm().child("Agents"))
                            .child(div().size(px(6.)).rounded_full().bg(cx.theme().primary)),
                    )
                    .child(
                        h_flex()
                            .h(px(38.))
                            .mx_2()
                            .px_2()
                            .gap_2()
                            .rounded(px(7.))
                            .bg(cx.theme().sidebar_accent)
                            .child(
                                div()
                                    .w(px(20.))
                                    .text_center()
                                    .text_color(cx.theme().sidebar_accent_foreground)
                                    .child("▤"),
                            )
                            .child(div().text_sm().font_semibold().child("Layers")),
                    )
                    .child(
                        h_flex()
                            .h(px(38.))
                            .mx_2()
                            .px_2()
                            .gap_2()
                            .rounded(px(7.))
                            .child(
                                div()
                                    .w(px(20.))
                                    .text_center()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("◇"),
                            )
                            .child(div().text_sm().child("Assets")),
                    )
                    .child(
                        v_flex()
                            .mt_3()
                            .mx_2()
                            .pt_2()
                            .gap_1()
                            .border_t_1()
                            .border_color(cx.theme().sidebar_border)
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .text_size(px(10.))
                                    .font_semibold()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("PAGES"),
                            )
                            .child(
                                h_flex()
                                    .h(px(32.))
                                    .px_2()
                                    .rounded(px(6.))
                                    .bg(cx.theme().sidebar_accent)
                                    .text_sm()
                                    .child("Dashboard"),
                            )
                            .child(h_flex().h(px(32.)).px_2().text_sm().child("Components")),
                    )
                    .child(div().flex_1())
                    .child(
                        v_flex()
                            .m_3()
                            .p_3()
                            .gap_1()
                            .rounded(px(10.))
                            .bg(cx.theme().secondary)
                            .child(
                                div()
                                    .text_size(px(10.))
                                    .font_semibold()
                                    .text_color(mode_accent)
                                    .child(format!(
                                        "{} MODE",
                                        self.toolbar_mode.label().to_uppercase()
                                    )),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(self.toolbar_last_action.clone()),
                            ),
                    ),
            )
            .child(
                div()
                    .debug_selector(|| "toolbar-story-canvas".to_owned())
                    .relative()
                    .flex_1()
                    .min_w(px(0.))
                    .h_full()
                    .overflow_hidden()
                    .bg(cx.theme().muted)
                    .child(
                        h_flex()
                            .size_full()
                            .justify_center()
                            .items_center()
                            .pb(if self.toolbar_mode == ToolbarMode::Motion {
                                px(190.)
                            } else {
                                px(92.)
                            })
                            .child(self.render_toolbar_demo_frame(cx)),
                    )
                    .child(self.render_toolbar_timeline(cx))
                    .child(self.toolbar.clone()),
            )
            .child(
                v_flex()
                    .w(px(258.))
                    .h_full()
                    .bg(cx.theme().background)
                    .text_color(cx.theme().foreground)
                    .border_l_1()
                    .border_color(cx.theme().border)
                    .child(
                        h_flex()
                            .h(px(48.))
                            .px_4()
                            .gap_4()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child(
                                div()
                                    .h_full()
                                    .flex()
                                    .items_center()
                                    .border_b_2()
                                    .border_color(mode_accent)
                                    .text_sm()
                                    .font_semibold()
                                    .child(self.toolbar_mode.label()),
                            )
                            .child(
                                div()
                                    .h_full()
                                    .flex()
                                    .items_center()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(if self.toolbar_mode == ToolbarMode::Dev {
                                        "Plugins"
                                    } else {
                                        "Prototype"
                                    }),
                            ),
                    )
                    .child(
                        v_flex()
                            .p_4()
                            .gap_4()
                            .child(
                                v_flex()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_semibold()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("SELECTION"),
                                    )
                                    .child(
                                        h_flex()
                                            .child(div().text_sm().child("Hero frame"))
                                            .child(div().flex_1())
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(mode_accent)
                                                    .child(self.toolbar_tool.label()),
                                            ),
                                    ),
                            )
                            .child(div().h(px(1.)).bg(cx.theme().border))
                            .child(
                                v_flex()
                                    .gap_2()
                                    .child(div().text_sm().font_semibold().child(
                                        match self.toolbar_mode {
                                            ToolbarMode::Draw => "Stroke",
                                            ToolbarMode::Design => "Layout",
                                            ToolbarMode::Motion => "Animation",
                                            ToolbarMode::Dev => "Inspect",
                                        },
                                    ))
                                    .child(
                                        h_flex()
                                            .h(px(34.))
                                            .px_2()
                                            .rounded(px(7.))
                                            .bg(cx.theme().secondary)
                                            .text_xs()
                                            .child(match self.toolbar_mode {
                                                ToolbarMode::Draw => format!(
                                                    "{} · {} px",
                                                    self.toolbar_draw_options.brush_style,
                                                    self.toolbar_draw_options.stroke_weight
                                                ),
                                                ToolbarMode::Design => {
                                                    "Auto layout · Vertical".to_owned()
                                                }
                                                ToolbarMode::Motion => format!(
                                                    "{} · {} ms",
                                                    self.toolbar_motion_options.animation_style,
                                                    self.toolbar_motion_options.duration_ms
                                                ),
                                                ToolbarMode::Dev => "CSS · Web · px".to_owned(),
                                            }),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn last_action_for_story(&self, kind: StoryKind) -> SharedString {
        match kind {
            StoryKind::Icons => "Browse or filter every bundled icon".into(),
            StoryKind::Toolbar => self.toolbar_last_action.clone(),
            StoryKind::Pages => self.pages_last_action.clone(),
            StoryKind::Layers => self.layers_last_action.clone(),
            StoryKind::Design => self.design_last_action.clone(),
            StoryKind::Variables => self.variables_last_action.clone(),
            StoryKind::Assets => self.assets_last_action.clone(),
            StoryKind::Prototype => self.prototype_last_action.clone(),
            StoryKind::Timeline => self.timeline_last_action.clone(),
            StoryKind::PseudoEditor => self.pseudo_last_action.clone(),
        }
    }

    fn render_design_story_harness(&self, cx: &mut Context<Self>) -> AnyElement {
        let selected = &self.design_nodes[self.selected_design_node];
        let selected_name = selected.name.clone();
        let selected_kind = selected.kind;
        let selected_size = format!(
            "{} × {}",
            selected.width.round() as i64,
            selected.height.round() as i64
        );

        h_flex()
            .size_full()
            .min_h(px(0.))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if event.dragging() {
                    this.update_design_panel_resize(f32::from(event.position.x), cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| {
                    this.finish_design_panel_resize(cx);
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| {
                    this.finish_design_panel_resize(cx);
                }),
            )
            .child(
                v_flex()
                    .id("design-fixture-rail-scroll")
                    .debug_selector(|| "design-fixture-rail-scroll".to_owned())
                    .w(px(260.))
                    .h_full()
                    .min_h(px(0.))
                    .overflow_y_scroll()
                    .track_scroll(&self.design_fixture_scroll_handle)
                    .p_3()
                    .gap_3()
                    .border_r_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().sidebar)
                    .child(self.render_design_context_selector(cx))
                    .child(self.render_design_workspace_toggle(cx))
                    .child(self.render_design_width_selector(cx))
                    .child(self.render_design_additional_labels_toggle(cx))
                    .child(self.render_design_nudge_preferences(cx))
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("LAST TYPED INTENT"),
                            )
                            .child(
                                div()
                                    .max_h(px(52.))
                                    .overflow_hidden()
                                    .text_xs()
                                    .child(self.design_last_action.clone()),
                            ),
                    )
                    .child(self.render_design_selector(cx)),
            )
            .child(
                v_flex()
                    .flex_1()
                    .h_full()
                    .min_w(px(0.))
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .bg(cx.theme().muted.opacity(0.45))
                    .child(
                        div()
                            .w(px(264.))
                            .h(px(176.))
                            .rounded(px(10.))
                            .border_1()
                            .border_color(cx.theme().selection)
                            .bg(cx.theme().secondary)
                            .shadow_lg()
                            .child(
                                v_flex()
                                    .size_full()
                                    .items_center()
                                    .justify_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_size(px(24.))
                                            .text_color(cx.theme().selection)
                                            .child(selected_kind.glyph()),
                                    )
                                    .child(
                                        div()
                                            .max_w(px(220.))
                                            .truncate()
                                            .text_sm()
                                            .font_semibold()
                                            .child(selected_name),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(selected_size),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("The mock canvas keeps left-opening inspectors visible"),
                    ),
            )
            .child(self.render_design_resize_handle(cx))
            .child(
                div()
                    .w(px(self.design_panel_width))
                    .h_full()
                    .min_h(px(0.))
                    .overflow_hidden()
                    .border_l_1()
                    .border_color(cx.theme().border)
                    .child(self.design_panel.clone()),
            )
            .into_any_element()
    }

    fn render_story_component(&self, story: StoryKind, cx: &mut Context<Self>) -> AnyElement {
        match story {
            StoryKind::Icons => self.icon_gallery.clone().into_any_element(),
            StoryKind::Toolbar => self.render_toolbar_story(cx),
            StoryKind::Pages => self.pages_panel.clone().into_any_element(),
            StoryKind::Layers => self.layers_panel.clone().into_any_element(),
            StoryKind::Design => self.design_panel.clone().into_any_element(),
            StoryKind::Variables => self.variables_page.clone().into_any_element(),
            StoryKind::Assets => self.assets_panel.clone().into_any_element(),
            StoryKind::Prototype => self.prototype_panel.clone().into_any_element(),
            StoryKind::Timeline => self.timeline.clone().into_any_element(),
            StoryKind::PseudoEditor => self.pseudo_editor.clone().into_any_element(),
        }
    }

    fn render_gallery_story_component(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.active_story {
            StoryKind::Design => self.render_design_story_harness(cx),
            story => self.render_story_component(story, cx),
        }
    }

    fn render_reference_last_action(
        &self,
        last_action: SharedString,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .absolute()
            .left(px(12.))
            .bottom(px(12.))
            .max_w(px(460.))
            .px(px(10.))
            .py(px(7.))
            .rounded(px(6.))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().popover.opacity(0.96))
            .shadow_lg()
            .text_size(px(11.))
            .child(last_action)
            .into_any_element()
    }
}

impl Render for Storybook {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(story) = self
            .story_windows
            .get(&window.window_handle().window_id())
            .copied()
        {
            return self.render_story_window(story, cx);
        }

        if self.launch_mode == StorybookLaunchMode::Gallery {
            return self.render_gallery_shell(cx);
        }

        if self.active_story == StoryKind::Icons {
            return self.icon_gallery.clone().into_any_element();
        }

        if self.active_story == StoryKind::Variables {
            let show_last_action = !self.variables_last_action.as_ref().starts_with("Ready");
            return v_flex()
                .id("storybook-reference-variables")
                .debug_selector(|| "storybook-reference-variables".to_owned())
                .relative()
                .size_full()
                .min_h(px(0.))
                .overflow_hidden()
                .bg(cx.theme().background)
                .child(self.variables_page.clone())
                .when(show_last_action, |fixture| {
                    fixture.child(
                        self.render_reference_last_action(self.variables_last_action.clone(), cx),
                    )
                })
                .into_any_element();
        }

        if self.active_story == StoryKind::Assets {
            let show_last_action = !self.assets_last_action.as_ref().starts_with("Ready");
            return v_flex()
                .id("storybook-reference-assets")
                .debug_selector(|| "storybook-reference-assets".to_owned())
                .relative()
                .size_full()
                .min_h(px(0.))
                .overflow_hidden()
                .bg(cx.theme().background)
                .child(self.assets_panel.clone())
                .when(show_last_action, |fixture| {
                    fixture.child(
                        self.render_reference_last_action(self.assets_last_action.clone(), cx),
                    )
                })
                .into_any_element();
        }

        if self.active_story == StoryKind::Prototype {
            let show_last_action = !self.prototype_last_action.as_ref().starts_with("Ready");
            return v_flex()
                .id("storybook-reference-prototype")
                .debug_selector(|| "storybook-reference-prototype".to_owned())
                .relative()
                .size_full()
                .min_h(px(0.))
                .overflow_hidden()
                .bg(cx.theme().background)
                .child(self.prototype_panel.clone())
                .when(show_last_action, |fixture| {
                    fixture.child(
                        self.render_reference_last_action(self.prototype_last_action.clone(), cx),
                    )
                })
                .into_any_element();
        }

        if self.active_story == StoryKind::Timeline {
            let show_last_action = !self.timeline_last_action.as_ref().starts_with("Ready");
            return v_flex()
                .id("storybook-reference-timeline")
                .debug_selector(|| "storybook-reference-timeline".to_owned())
                .relative()
                .size_full()
                .min_h(px(0.))
                .overflow_hidden()
                .bg(cx.theme().background)
                .child(self.timeline.clone())
                .when(show_last_action, |fixture| {
                    fixture.child(
                        self.render_reference_last_action(self.timeline_last_action.clone(), cx),
                    )
                })
                .into_any_element();
        }

        if self.active_story == StoryKind::PseudoEditor {
            let show_last_action = !self.pseudo_last_action.as_ref().starts_with("Ready");
            return v_flex()
                .id("storybook-reference-pseudo-editor")
                .debug_selector(|| "storybook-reference-pseudo-editor".to_owned())
                .relative()
                .size_full()
                .min_h(px(0.))
                .overflow_hidden()
                .bg(cx.theme().background)
                .child(self.pseudo_editor.clone())
                .when(show_last_action, |fixture| {
                    fixture.child(
                        self.render_reference_last_action(self.pseudo_last_action.clone(), cx),
                    )
                })
                .into_any_element();
        }

        if self.active_story == StoryKind::Toolbar {
            return v_flex()
                .size_full()
                .bg(cx.theme().background)
                .text_color(cx.theme().foreground)
                .child(
                    h_flex()
                        .h(px(56.))
                        .w_full()
                        .px_5()
                        .gap_2()
                        .border_b_1()
                        .border_color(cx.theme().border)
                        .child(div().mr_3().font_semibold().child("Fanta GPUI"))
                        .child(self.render_navigation_item(StoryKind::Toolbar, "Toolbar", cx))
                        .child(self.render_navigation_item(StoryKind::Pages, "Pages", cx))
                        .child(self.render_navigation_item(StoryKind::Layers, "Layers", cx))
                        .child(self.render_navigation_item(StoryKind::Design, "Design", cx))
                        .child(self.render_navigation_item(StoryKind::Variables, "Variables", cx))
                        .child(self.render_navigation_item(StoryKind::Assets, "Assets", cx))
                        .child(self.render_navigation_item(StoryKind::Prototype, "Prototype", cx))
                        .child(self.render_navigation_item(StoryKind::Timeline, "Timeline", cx))
                        .child(self.render_navigation_item(
                            StoryKind::PseudoEditor,
                            "Pseudo editor",
                            cx,
                        )),
                )
                .child(self.render_toolbar_story(cx))
                .into_any_element();
        }

        if self.active_story == StoryKind::Design {
            let selected = &self.design_nodes[self.selected_design_node];
            let selected_name = selected.name.clone();
            let selected_kind = selected.kind;
            let selected_size = format!(
                "{} × {}",
                selected.width.round() as i64,
                selected.height.round() as i64
            );
            return v_flex()
                .size_full()
                .bg(cx.theme().background)
                .text_color(cx.theme().foreground)
                .child(
                    h_flex()
                        .h(px(56.))
                        .w_full()
                        .px_5()
                        .gap_2()
                        .border_b_1()
                        .border_color(cx.theme().border)
                        .child(div().mr_3().font_semibold().child("Fanta GPUI"))
                        .child(self.render_navigation_item(StoryKind::Toolbar, "Toolbar", cx))
                        .child(self.render_navigation_item(StoryKind::Pages, "Pages", cx))
                        .child(self.render_navigation_item(StoryKind::Layers, "Layers", cx))
                        .child(self.render_navigation_item(StoryKind::Design, "Design", cx))
                        .child(self.render_navigation_item(StoryKind::Variables, "Variables", cx))
                        .child(self.render_navigation_item(StoryKind::Assets, "Assets", cx))
                        .child(self.render_navigation_item(StoryKind::Prototype, "Prototype", cx))
                        .child(self.render_navigation_item(StoryKind::Timeline, "Timeline", cx))
                        .child(self.render_navigation_item(
                            StoryKind::PseudoEditor,
                            "Pseudo editor",
                            cx,
                        )),
                )
                .child(
                    h_flex()
                        .flex_1()
                        .min_h(px(0.))
                        .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                            if event.dragging() {
                                this.update_design_panel_resize(f32::from(event.position.x), cx);
                            }
                        }))
                        .on_mouse_up(
                            MouseButton::Left,
                            cx.listener(|this, _: &MouseUpEvent, _, cx| {
                                this.finish_design_panel_resize(cx);
                            }),
                        )
                        .on_mouse_up_out(
                            MouseButton::Left,
                            cx.listener(|this, _: &MouseUpEvent, _, cx| {
                                this.finish_design_panel_resize(cx);
                            }),
                        )
                        .child(
                            v_flex()
                                .id("design-fixture-rail-scroll")
                                .debug_selector(|| "design-fixture-rail-scroll".to_owned())
                                .w(px(260.))
                                .h_full()
                                .min_h(px(0.))
                                .overflow_y_scroll()
                                .track_scroll(&self.design_fixture_scroll_handle)
                                .p_3()
                                .gap_3()
                                .border_r_1()
                                .border_color(cx.theme().border)
                                .bg(cx.theme().secondary.opacity(0.32))
                                .child(self.render_design_context_selector(cx))
                                .child(self.render_design_workspace_toggle(cx))
                                .child(self.render_design_width_selector(cx))
                                .child(self.render_design_additional_labels_toggle(cx))
                                .child(self.render_design_nudge_preferences(cx))
                                .child(
                                    v_flex()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("LAST TYPED INTENT"),
                                        )
                                        .child(
                                            div()
                                                .max_h(px(52.))
                                                .overflow_hidden()
                                                .text_xs()
                                                .child(self.design_last_action.clone()),
                                        ),
                                )
                                .child(self.render_design_selector(cx)),
                        )
                        .child(
                            v_flex()
                                .flex_1()
                                .h_full()
                                .min_w(px(0.))
                                .items_center()
                                .justify_center()
                                .gap_3()
                                .bg(cx.theme().muted.opacity(0.45))
                                .child(
                                    div()
                                        .w(px(264.))
                                        .h(px(176.))
                                        .rounded(px(10.))
                                        .border_1()
                                        .border_color(cx.theme().selection)
                                        .bg(cx.theme().secondary)
                                        .shadow_lg()
                                        .child(
                                            v_flex()
                                                .size_full()
                                                .items_center()
                                                .justify_center()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .text_size(px(24.))
                                                        .text_color(cx.theme().selection)
                                                        .child(selected_kind.glyph()),
                                                )
                                                .child(
                                                    div()
                                                        .max_w(px(220.))
                                                        .truncate()
                                                        .text_sm()
                                                        .font_semibold()
                                                        .child(selected_name),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child(selected_size),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(
                                            "Neutral canvas keeps left-opening inspectors visible",
                                        ),
                                ),
                        )
                        .child(self.render_design_resize_handle(cx))
                        .child(
                            div()
                                .w(px(self.design_panel_width))
                                .h_full()
                                .min_h(px(0.))
                                .overflow_hidden()
                                .border_l_1()
                                .border_color(cx.theme().border)
                                .child(self.design_panel.clone()),
                        ),
                )
                .into_any_element();
        }

        let (eyebrow, title, description, adapter_description, last_action, component): (
            &'static str,
            &'static str,
            &'static str,
            &'static str,
            SharedString,
            AnyElement,
        ) = match self.active_story {
            StoryKind::Icons => unreachable!("icon story is rendered above"),
            StoryKind::Toolbar => unreachable!("toolbar story is rendered above"),
            StoryKind::Variables
            | StoryKind::Assets
            | StoryKind::Prototype
            | StoryKind::Timeline
            | StoryKind::PseudoEditor => {
                unreachable!("reference stories are rendered at full size above")
            }
            StoryKind::Pages => (
                "PAGES PANEL",
                "Pages panel",
                "Collapse it, add and rename pages, secondary-click a page, or open Find to \
                 exercise filters, Replace, result scope, and navigation.",
                "The story owns page and result data, listens to PagesPanelAction, applies mock \
                 mutations, and feeds fresh read models back into the component.",
                self.pages_last_action.clone(),
                self.pages_panel.clone().into_any_element(),
            ),
            StoryKind::Layers => (
                "LAYERS PANEL",
                "Layers panel",
                "Collapse the panel, drag rows to reorder or reparent them, expand the tree, use \
                 Command/Shift selection, double-click to rename, toggle lock and visibility, \
                 and secondary-click every node type to compare its menu.",
                "The story owns the mock layer tree, selection, expansion, lock, and visibility. \
                 LayersPanel emits typed intents only; this adapter applies them and calls the \
                 setters with updated host data.",
                self.layers_last_action.clone(),
                self.layers_panel.clone().into_any_element(),
            ),
            StoryKind::Design => (
                "DESIGN PANEL",
                "Design inspector",
                "Switch among page, single, multiple, view-only, text-edit, and vector-edit \
                 inspection contexts, then select every node preset and auto-layout variation. \
                 Edit nested current-file Page styles and background, drag styles across folders, \
                 compare resolved/explicit variable modes, add/remove collection rows, and open \
                 fill or stroke swatches.",
                "The story owns selection cardinality, permissions, edit mode, mixed and bound \
                 property states, Page-local style trees, variable modes, and node read models. It \
                 applies DesignPanelAction intents and echoes the active inspection context back; \
                 DesignPanel retains only transient presentation state.",
                self.design_last_action.clone(),
                self.design_panel.clone().into_any_element(),
            ),
        };

        v_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                h_flex()
                    .h(px(56.))
                    .w_full()
                    .px_5()
                    .gap_2()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(div().mr_3().font_semibold().child("Fanta GPUI"))
                    .child(self.render_navigation_item(StoryKind::Toolbar, "Toolbar", cx))
                    .child(self.render_navigation_item(StoryKind::Pages, "Pages", cx))
                    .child(self.render_navigation_item(StoryKind::Layers, "Layers", cx))
                    .child(self.render_navigation_item(StoryKind::Design, "Design", cx))
                    .child(self.render_navigation_item(StoryKind::Variables, "Variables", cx))
                    .child(self.render_navigation_item(StoryKind::Assets, "Assets", cx))
                    .child(self.render_navigation_item(StoryKind::Prototype, "Prototype", cx))
                    .child(self.render_navigation_item(StoryKind::Timeline, "Timeline", cx))
                    .child(self.render_navigation_item(
                        StoryKind::PseudoEditor,
                        "Pseudo editor",
                        cx,
                    )),
            )
            .child(
                h_flex()
                    .flex_1()
                    .min_h(px(0.))
                    .items_start()
                    .child(
                        v_flex()
                            .w(px(360.))
                            .h_full()
                            .p_4()
                            .gap_3()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(eyebrow),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h(px(0.))
                                    .w_full()
                                    .overflow_hidden()
                                    .child(component),
                            ),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .h_full()
                            .border_l_1()
                            .border_color(cx.theme().border)
                            .p_8()
                            .gap_4()
                            .child(div().text_2xl().font_semibold().child(title))
                            .child(
                                div()
                                    .max_w(px(620.))
                                    .text_color(cx.theme().muted_foreground)
                                    .child(description),
                            )
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("LAST TYPED INTENT"),
                                    )
                                    .child(div().text_sm().child(last_action)),
                            )
                            .child(
                                div()
                                    .mt_4()
                                    .max_w(px(620.))
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(adapter_description),
                            )
                            .when(self.active_story == StoryKind::Design, |details| {
                                details
                                    .child(self.render_design_context_selector(cx))
                                    .child(self.render_design_selector(cx))
                            }),
                    ),
            )
            .into_any_element()
    }
}

fn find_layer_mut<'a>(
    nodes: &'a mut [LayersPanelItem],
    node_id: &SharedString,
) -> Option<&'a mut LayersPanelItem> {
    for node in nodes {
        if node.id == *node_id {
            return Some(node);
        }
        if let Some(found) = find_layer_mut(&mut node.children, node_id) {
            return Some(found);
        }
    }
    None
}

fn take_layer(nodes: &mut Vec<LayersPanelItem>, node_id: &SharedString) -> Option<LayersPanelItem> {
    if let Some(index) = nodes.iter().position(|node| node.id == *node_id) {
        return Some(nodes.remove(index));
    }
    for node in nodes {
        if let Some(found) = take_layer(&mut node.children, node_id) {
            return Some(found);
        }
    }
    None
}

fn insert_layer(
    nodes: &mut Vec<LayersPanelItem>,
    target_node_id: &SharedString,
    item: LayersPanelItem,
    position: LayersPanelDropPosition,
) -> bool {
    if position == LayersPanelDropPosition::Inside {
        for node in nodes {
            if node.id == *target_node_id {
                node.children.push(item);
                return true;
            }
            if insert_layer(&mut node.children, target_node_id, item.clone(), position) {
                return true;
            }
        }
        return false;
    }

    if let Some(index) = nodes.iter().position(|node| node.id == *target_node_id) {
        let insertion_index = match position {
            LayersPanelDropPosition::Before => index,
            LayersPanelDropPosition::After => index + 1,
            LayersPanelDropPosition::Inside => unreachable!(),
        };
        nodes.insert(insertion_index, item);
        return true;
    }
    for node in nodes {
        if insert_layer(&mut node.children, target_node_id, item.clone(), position) {
            return true;
        }
    }
    false
}

fn flatten_layer_ids(nodes: &[LayersPanelItem]) -> Vec<SharedString> {
    fn collect(nodes: &[LayersPanelItem], ids: &mut Vec<SharedString>) {
        for node in nodes {
            ids.push(node.id.clone());
            collect(&node.children, ids);
        }
    }

    let mut ids = Vec::new();
    collect(nodes, &mut ids);
    ids
}

fn refresh_story_component_override_summary(node: &mut DesignPanelNode) {
    let overridden_property_count = node
        .component_properties
        .iter()
        .filter(|property| property.reset_state.is_overridden())
        .count()
        .try_into()
        .unwrap_or(u32::MAX);
    if let Some(context) = node.component_context.as_mut() {
        context.overrides.overridden_property_count = overridden_property_count;
        context.overrides.reset_state =
            if overridden_property_count > 0 || context.overrides.nested_override_count > 0 {
                DesignComponentResetState::Resettable
            } else {
                DesignComponentResetState::Clean
            };
    }
}

fn remove_index<T>(items: &mut Vec<T>, index: usize) {
    if index < items.len() {
        items.remove(index);
    }
}

fn story_layout_grid_index(
    node: &DesignPanelNode,
    target: &StoryLayoutGridEditTarget,
) -> Option<usize> {
    if target.guide_id.is_empty() {
        target
            .legacy_index
            .filter(|index| *index < node.layout_grids.len())
    } else {
        node.layout_grids
            .iter()
            .position(|guide| guide.id == target.guide_id)
    }
}

fn apply_story_layout_grid_edit_phase(
    node: &mut DesignPanelNode,
    snapshots: &mut HashMap<StoryLayoutGridEditTarget, DesignLayoutGrid>,
    target: StoryLayoutGridEditTarget,
    property: DesignPanelProperty,
    value: &DesignPanelValue,
    phase: DesignPanelEditPhase,
) -> bool {
    if property.layout_grid_index().is_none() || node.layout_grid_style_binding.is_some() {
        snapshots.remove(&target);
        return false;
    }
    let Some(index) = story_layout_grid_index(node, &target) else {
        snapshots.remove(&target);
        return false;
    };
    match phase {
        DesignPanelEditPhase::Begin => {
            let mut candidate = node.layout_grids[index].clone();
            if !apply_story_layout_grid_property(&mut candidate, property, value) {
                return false;
            }
            snapshots
                .entry(target)
                .or_insert_with(|| node.layout_grids[index].clone());
            true
        }
        DesignPanelEditPhase::Preview => {
            apply_story_layout_grid_property(&mut node.layout_grids[index], property, value)
        }
        DesignPanelEditPhase::Commit => {
            let applied =
                apply_story_layout_grid_property(&mut node.layout_grids[index], property, value);
            snapshots.remove(&target);
            applied
        }
        DesignPanelEditPhase::Cancel => {
            let Some(original) = snapshots.remove(&target) else {
                return false;
            };
            node.layout_grids[index] = original;
            true
        }
    }
}

fn story_layout_grid_variable_target(
    node: &DesignPanelNode,
    target: &DesignLayoutGridVariableTarget,
) -> Option<(usize, DesignLayoutGridVariableValue)> {
    let index = if target.guide_id.is_empty() {
        (target.index < node.layout_grids.len()).then_some(target.index)
    } else {
        node.layout_grids
            .iter()
            .position(|guide| guide.id == target.guide_id)
    }?;
    let (resolved, value) = node.layout_grids[index]
        .variable_target(index, target.property.with_layout_grid_index(index))?;
    (resolved.field == target.field
        && resolved.property.with_layout_grid_index(0) == target.property.with_layout_grid_index(0))
    .then_some((index, value))
}

fn story_layout_grid_variable_float(value: DesignLayoutGridVariableValue) -> Option<f32> {
    match value {
        DesignLayoutGridVariableValue::Number(value) if value.is_finite() && value >= 0. => {
            Some(value)
        }
        DesignLayoutGridVariableValue::Count(DesignLayoutGridCount::Auto) => Some(f32::INFINITY),
        DesignLayoutGridVariableValue::Count(DesignLayoutGridCount::Number(value)) => {
            Some(f32::from(value))
        }
        DesignLayoutGridVariableValue::Number(_) => None,
    }
}

fn apply_story_layout_grid_variable(
    node: &mut DesignPanelNode,
    target: &DesignLayoutGridVariableTarget,
    variable: &DesignVariable,
) -> bool {
    if node.layout_grid_style_binding.is_some()
        || variable.import_state == DesignVariableImportState::Available
    {
        return false;
    }
    let Some((index, current_value)) = story_layout_grid_variable_target(node, target) else {
        return false;
    };
    if !current_value.is_compatible(variable)
        || node.layout_grids[index]
            .variable_binding(target.field)
            .is_some_and(|binding| binding.read_only_reason.is_some())
    {
        return false;
    }
    let Some(DesignVariableResolvedValue::Float(resolved)) = variable.resolved_value else {
        return false;
    };
    let property = target.property.with_layout_grid_index(index);
    let applied = match (property, &mut node.layout_grids[index].settings) {
        (DesignPanelProperty::LayoutGridSize(_), DesignLayoutGridSettings::Uniform(settings))
            if resolved.is_finite() && resolved >= 0. =>
        {
            settings.size = resolved;
            true
        }
        (DesignPanelProperty::LayoutGridSize(_), DesignLayoutGridSettings::Columns(settings))
            if !settings.alignment.is_stretch() && resolved.is_finite() && resolved >= 0. =>
        {
            settings.size = resolved;
            true
        }
        (DesignPanelProperty::LayoutGridSize(_), DesignLayoutGridSettings::Rows(settings))
            if !settings.alignment.is_stretch() && resolved.is_finite() && resolved >= 0. =>
        {
            settings.size = resolved;
            true
        }
        (DesignPanelProperty::LayoutGridCount(_), DesignLayoutGridSettings::Columns(settings)) => {
            settings.count = if resolved.is_infinite() && resolved.is_sign_positive() {
                DesignLayoutGridCount::Auto
            } else {
                DesignLayoutGridCount::number(resolved as u16)
            };
            true
        }
        (DesignPanelProperty::LayoutGridCount(_), DesignLayoutGridSettings::Rows(settings)) => {
            settings.count = if resolved.is_infinite() && resolved.is_sign_positive() {
                DesignLayoutGridCount::Auto
            } else {
                DesignLayoutGridCount::number(resolved as u16)
            };
            true
        }
        (DesignPanelProperty::LayoutGridOffset(_), DesignLayoutGridSettings::Columns(settings))
            if settings.alignment.supports_offset() && resolved.is_finite() && resolved >= 0. =>
        {
            settings.offset = resolved;
            true
        }
        (DesignPanelProperty::LayoutGridOffset(_), DesignLayoutGridSettings::Rows(settings))
            if settings.alignment.supports_offset() && resolved.is_finite() && resolved >= 0. =>
        {
            settings.offset = resolved;
            true
        }
        (DesignPanelProperty::LayoutGridMargin(_), DesignLayoutGridSettings::Columns(settings))
            if settings.alignment.is_stretch() && resolved.is_finite() && resolved >= 0. =>
        {
            settings.margin = resolved;
            true
        }
        (DesignPanelProperty::LayoutGridMargin(_), DesignLayoutGridSettings::Rows(settings))
            if settings.alignment.is_stretch() && resolved.is_finite() && resolved >= 0. =>
        {
            settings.margin = resolved;
            true
        }
        (DesignPanelProperty::LayoutGridGutter(_), DesignLayoutGridSettings::Columns(settings))
            if resolved.is_finite() && resolved >= 0. =>
        {
            settings.gutter = resolved;
            true
        }
        (DesignPanelProperty::LayoutGridGutter(_), DesignLayoutGridSettings::Rows(settings))
            if resolved.is_finite() && resolved >= 0. =>
        {
            settings.gutter = resolved;
            true
        }
        _ => false,
    };
    if applied {
        node.layout_grids[index].set_variable_binding(
            target.field,
            Some(
                DesignLayoutGridVariableBinding::new(variable.id.clone(), variable.name.clone())
                    .with_collection(variable.collection_name.clone()),
            ),
        );
    }
    applied
}

fn detach_story_layout_grid_variable(
    node: &mut DesignPanelNode,
    target: &DesignLayoutGridVariableTarget,
    variable_id: &str,
) -> bool {
    if node.layout_grid_style_binding.is_some() {
        return false;
    }
    let Some((index, _)) = story_layout_grid_variable_target(node, target) else {
        return false;
    };
    let detachable = node.layout_grids[index]
        .variable_binding(target.field)
        .is_some_and(|binding| {
            binding.variable_id.as_ref() == variable_id
                && binding.can_detach
                && binding.read_only_reason.is_none()
        });
    if detachable {
        node.layout_grids[index].set_variable_binding(target.field, None);
    }
    detachable
}

fn apply_story_layout_grid_property(
    guide: &mut DesignLayoutGrid,
    property: DesignPanelProperty,
    value: &DesignPanelValue,
) -> bool {
    if guide
        .variable_target(0, property.with_layout_grid_index(0))
        .is_some_and(|(target, _)| guide.variable_binding(target.field).is_some())
    {
        return false;
    }
    match (property.with_layout_grid_index(0), value) {
        (DesignPanelProperty::LayoutGridKind(_), DesignPanelValue::GridKind(value)) => {
            guide.settings = match value {
                DesignGridKind::Uniform => {
                    DesignLayoutGridSettings::Uniform(DesignUniformLayoutGrid::default())
                }
                DesignGridKind::Columns => {
                    DesignLayoutGridSettings::Columns(DesignColumnLayoutGrid::default())
                }
                DesignGridKind::Rows => {
                    DesignLayoutGridSettings::Rows(DesignRowLayoutGrid::default())
                }
            };
            true
        }
        (DesignPanelProperty::LayoutGridVisible(_), DesignPanelValue::Bool(value)) => {
            guide.visible = *value;
            true
        }
        (
            DesignPanelProperty::LayoutGridAlignment(_),
            DesignPanelValue::ColumnGridAlignment(value),
        ) => {
            let DesignLayoutGridSettings::Columns(settings) = &mut guide.settings else {
                return false;
            };
            settings.alignment = *value;
            true
        }
        (
            DesignPanelProperty::LayoutGridAlignment(_),
            DesignPanelValue::RowGridAlignment(value),
        ) => {
            let DesignLayoutGridSettings::Rows(settings) = &mut guide.settings else {
                return false;
            };
            settings.alignment = *value;
            true
        }
        (DesignPanelProperty::LayoutGridCount(_), DesignPanelValue::LayoutGridCount(value)) => {
            match &mut guide.settings {
                DesignLayoutGridSettings::Columns(settings) => {
                    settings.count = *value;
                    true
                }
                DesignLayoutGridSettings::Rows(settings) => {
                    settings.count = *value;
                    true
                }
                DesignLayoutGridSettings::Uniform(_) => false,
            }
        }
        (DesignPanelProperty::LayoutGridSize(_), DesignPanelValue::Number(value))
            if value.is_finite() && *value >= 0. =>
        {
            match &mut guide.settings {
                DesignLayoutGridSettings::Uniform(settings) => settings.size = *value,
                DesignLayoutGridSettings::Columns(settings) if !settings.alignment.is_stretch() => {
                    settings.size = *value;
                }
                DesignLayoutGridSettings::Rows(settings) if !settings.alignment.is_stretch() => {
                    settings.size = *value;
                }
                DesignLayoutGridSettings::Columns(_) | DesignLayoutGridSettings::Rows(_) => {
                    return false;
                }
            }
            true
        }
        (DesignPanelProperty::LayoutGridOffset(_), DesignPanelValue::Number(value))
            if value.is_finite() && *value >= 0. =>
        {
            match &mut guide.settings {
                DesignLayoutGridSettings::Columns(settings)
                    if settings.alignment.supports_offset() =>
                {
                    settings.offset = *value;
                    true
                }
                DesignLayoutGridSettings::Rows(settings)
                    if settings.alignment.supports_offset() =>
                {
                    settings.offset = *value;
                    true
                }
                DesignLayoutGridSettings::Uniform(_)
                | DesignLayoutGridSettings::Columns(_)
                | DesignLayoutGridSettings::Rows(_) => false,
            }
        }
        (DesignPanelProperty::LayoutGridGutter(_), DesignPanelValue::Number(value))
            if value.is_finite() && *value >= 0. =>
        {
            match &mut guide.settings {
                DesignLayoutGridSettings::Columns(settings) => settings.gutter = *value,
                DesignLayoutGridSettings::Rows(settings) => settings.gutter = *value,
                DesignLayoutGridSettings::Uniform(_) => return false,
            }
            true
        }
        (DesignPanelProperty::LayoutGridMargin(_), DesignPanelValue::Number(value))
            if value.is_finite() && *value >= 0. =>
        {
            match &mut guide.settings {
                DesignLayoutGridSettings::Columns(settings) if settings.alignment.is_stretch() => {
                    settings.margin = *value;
                    true
                }
                DesignLayoutGridSettings::Rows(settings) if settings.alignment.is_stretch() => {
                    settings.margin = *value;
                    true
                }
                DesignLayoutGridSettings::Uniform(_)
                | DesignLayoutGridSettings::Columns(_)
                | DesignLayoutGridSettings::Rows(_) => false,
            }
        }
        (DesignPanelProperty::LayoutGridColor(_), DesignPanelValue::Color(value)) => {
            guide.color = *value;
            true
        }
        (DesignPanelProperty::LayoutGridOpacity(_), DesignPanelValue::Number(value))
            if value.is_finite() =>
        {
            guide.opacity = value.clamp(0., 100.);
            true
        }
        _ => false,
    }
}

fn apply_story_frame_preset(
    nodes: &mut [DesignPanelNode],
    catalogs: &HashMap<SharedString, DesignFramePresetViewData>,
    node_id: &SharedString,
    selection: &DesignFramePresetSelection,
    width: f32,
    height: f32,
) -> bool {
    let valid_preset = catalogs
        .get(node_id)
        .filter(|view_data| view_data.target_node_id == *node_id && view_data.can_apply(selection))
        .and_then(|view_data| view_data.preset(selection))
        .is_some_and(|(_, preset)| preset.width == width && preset.height == height);
    valid_preset
        && nodes
            .iter_mut()
            .find(|node| node.id == *node_id && node.kind == DesignPanelNodeKind::Frame)
            .map(|node| {
                node.width = width;
                node.height = height;
            })
            .is_some()
}

fn story_whole_layer_paints(
    node: &DesignPanelNode,
    collection: DesignPanelCollection,
) -> Option<&[DesignPaint]> {
    match collection {
        DesignPanelCollection::Fill => Some(&node.fills),
        DesignPanelCollection::Stroke => {
            node.stroke.as_ref().map(|stroke| stroke.paints.as_slice())
        }
        DesignPanelCollection::Effect
        | DesignPanelCollection::LayoutGrid
        | DesignPanelCollection::Export => None,
    }
}

fn story_whole_layer_paints_mut(
    node: &mut DesignPanelNode,
    collection: DesignPanelCollection,
) -> Option<&mut [DesignPaint]> {
    match collection {
        DesignPanelCollection::Fill => Some(&mut node.fills),
        DesignPanelCollection::Stroke => node
            .stroke
            .as_mut()
            .map(|stroke| stroke.paints.as_mut_slice()),
        DesignPanelCollection::Effect
        | DesignPanelCollection::LayoutGrid
        | DesignPanelCollection::Export => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_story_targeted_common_paint_edit(
    nodes: &mut [DesignPanelNode],
    snapshots: &mut HashMap<StoryPaintEditTarget, DesignPaint>,
    current_target: Option<&DesignPanelTarget>,
    requested_target: &DesignPanelTarget,
    can_edit: bool,
    collection: DesignPanelCollection,
    paint_target: DesignPaintTarget,
    paint_id: &SharedString,
    fallback_index: usize,
    edit: &DesignPaintEdit,
    phase: DesignPanelEditPhase,
) -> bool {
    if paint_target != DesignPaintTarget::WholeLayer
        || !matches!(
            collection,
            DesignPanelCollection::Fill | DesignPanelCollection::Stroke
        )
    {
        return false;
    }
    let Some(node_indices) =
        story_exact_editable_node_indices(nodes, current_target, requested_target, can_edit)
    else {
        return false;
    };
    let Some(reference_index) = node_indices.first().copied() else {
        return false;
    };
    let reference = &nodes[reference_index];
    let collection_is_unbound = match collection {
        DesignPanelCollection::Fill => reference.fill_style_binding.is_none(),
        DesignPanelCollection::Stroke => reference.stroke_style_binding.is_none(),
        DesignPanelCollection::Effect
        | DesignPanelCollection::LayoutGrid
        | DesignPanelCollection::Export => false,
    };
    let Some(reference_paints) = story_whole_layer_paints(reference, collection) else {
        return false;
    };
    let resolved_index = if paint_id.is_empty() {
        (fallback_index < reference_paints.len()).then_some(fallback_index)
    } else {
        reference_paints
            .iter()
            .position(|paint| paint.id == *paint_id)
    };
    let Some(resolved_index) = resolved_index else {
        return false;
    };
    let collections_remain_common = node_indices.iter().all(|node_index| {
        let node = &nodes[*node_index];
        let supported = match collection {
            DesignPanelCollection::Fill => {
                node.supports_fill()
                    && node.supports_section(DesignPanelSection::Fill)
                    && node.fill_style_binding.is_none()
            }
            DesignPanelCollection::Stroke => {
                node.supports_stroke()
                    && node.supports_section(DesignPanelSection::Stroke)
                    && node.stroke_style_binding.is_none()
            }
            DesignPanelCollection::Effect
            | DesignPanelCollection::LayoutGrid
            | DesignPanelCollection::Export => false,
        };
        let common = match collection {
            DesignPanelCollection::Fill => story_fill_collection_is_common(reference, node),
            DesignPanelCollection::Stroke => story_stroke_collection_is_common(reference, node),
            DesignPanelCollection::Effect
            | DesignPanelCollection::LayoutGrid
            | DesignPanelCollection::Export => false,
        };
        supported && common
    });
    if !collection_is_unbound || !collections_remain_common {
        return false;
    }

    let mut edits = Vec::with_capacity(node_indices.len());
    for node_index in node_indices {
        let node = &nodes[node_index];
        let Some(paint) = story_whole_layer_paints(node, collection)
            .and_then(|paints| paints.get(resolved_index))
        else {
            return false;
        };
        if paint.read_only {
            return false;
        }
        let key = StoryPaintEditTarget::new(
            node.id.clone(),
            collection,
            paint_target,
            paint.id.clone(),
            resolved_index,
        );
        let mut candidate = paint.clone();
        if phase != DesignPanelEditPhase::Cancel && !candidate.apply_edit(edit) {
            return false;
        }
        edits.push((node_index, key, paint.clone(), candidate));
    }

    let snapshot_count = edits
        .iter()
        .filter(|(_, key, _, _)| snapshots.contains_key(key))
        .count();
    let lifecycle_is_valid = match phase {
        DesignPanelEditPhase::Begin => snapshot_count == 0,
        DesignPanelEditPhase::Preview | DesignPanelEditPhase::Cancel => {
            snapshot_count == edits.len()
        }
        DesignPanelEditPhase::Commit => snapshot_count == 0 || snapshot_count == edits.len(),
    };
    if !lifecycle_is_valid {
        return false;
    }

    if phase == DesignPanelEditPhase::Begin {
        for (_, key, original, _) in edits {
            snapshots.insert(key, original);
        }
        return true;
    }

    let replacements = edits
        .iter()
        .map(|(node_index, key, _, candidate)| {
            let replacement = if phase == DesignPanelEditPhase::Cancel {
                snapshots
                    .get(key)
                    .cloned()
                    .expect("the complete Cancel snapshot set was preflighted")
            } else {
                candidate.clone()
            };
            (*node_index, key.clone(), replacement)
        })
        .collect::<Vec<_>>();
    for (node_index, _, replacement) in &replacements {
        story_whole_layer_paints_mut(&mut nodes[*node_index], collection)
            .expect("the common collection was preflighted")[resolved_index] = replacement.clone();
    }
    if matches!(
        phase,
        DesignPanelEditPhase::Commit | DesignPanelEditPhase::Cancel
    ) {
        for (_, key, _) in replacements {
            snapshots.remove(&key);
        }
    }
    true
}

fn story_targeted_node_actions(
    nodes: &[DesignPanelNode],
    current_target: Option<&DesignPanelTarget>,
    target: &DesignPanelTarget,
    can_edit: bool,
    action: &DesignPanelAction,
) -> Option<Vec<DesignPanelAction>> {
    let node_indices = story_exact_editable_node_indices(nodes, current_target, target, can_edit)?;
    if !node_indices
        .iter()
        .all(|index| story_targeted_node_action_is_applicable(&nodes[*index], action))
    {
        return None;
    }
    let DesignPanelTarget::Nodes { node_ids } = target else {
        unreachable!("exact node-target validation rejects Page targets")
    };
    node_ids
        .iter()
        .map(|node_id| action.retargeted_legacy_node_action(node_id.clone()))
        .collect()
}

fn story_exact_editable_node_indices(
    nodes: &[DesignPanelNode],
    current_target: Option<&DesignPanelTarget>,
    requested_target: &DesignPanelTarget,
    can_edit: bool,
) -> Option<Vec<usize>> {
    if !can_edit || current_target != Some(requested_target) {
        return None;
    }
    let DesignPanelTarget::Nodes { node_ids } = requested_target else {
        return None;
    };
    let unique_ids = node_ids
        .iter()
        .map(SharedString::as_ref)
        .collect::<HashSet<_>>();
    if node_ids.is_empty()
        || unique_ids.len() != node_ids.len()
        || node_ids.iter().any(|node_id| node_id.trim().is_empty())
    {
        return None;
    }
    node_ids
        .iter()
        .map(|node_id| {
            let mut matches = nodes
                .iter()
                .enumerate()
                .filter(|(_, node)| node.id == *node_id);
            let (index, _) = matches.next()?;
            matches.next().is_none().then_some(index)
        })
        .collect()
}

fn story_targeted_node_action_is_applicable(
    node: &DesignPanelNode,
    action: &DesignPanelAction,
) -> bool {
    match action {
        DesignPanelAction::PropertyChangeRequested {
            property, value, ..
        }
        | DesignPanelAction::PropertyEditRequested {
            property, value, ..
        } => story_multiple_property_edit_is_applicable(node, *property, value),
        DesignPanelAction::CollectionItemAddRequested {
            collection, target, ..
        } => {
            *target == DesignPaintTarget::WholeLayer
                && story_collection_add_is_applicable(node, *collection)
        }
        DesignPanelAction::EffectAddRequested { kind, .. } => {
            story_effect_add_is_applicable(node, *kind)
        }
        // The Storybook intentionally rejects multi-node leaves whose
        // sub-resource identity or shared-catalog side effects cannot be
        // preflighted by this compact mock reducer. A production host may
        // support more leaves through one native document transaction.
        _ => false,
    }
}

fn story_multiple_property_edit_is_applicable(
    node: &DesignPanelNode,
    property: DesignPanelProperty,
    value: &DesignPanelValue,
) -> bool {
    match (property, value) {
        (DesignPanelProperty::X | DesignPanelProperty::Y, DesignPanelValue::Number(value)) => {
            value.is_finite()
                && node.supports_position_coordinates()
                && node.supports_section(DesignPanelSection::Position)
        }
        (DesignPanelProperty::Rotation, DesignPanelValue::Number(value)) => {
            value.is_finite()
                && node.supports_transforms()
                && node.supports_section(DesignPanelSection::Position)
        }
        (
            DesignPanelProperty::Width | DesignPanelProperty::Height,
            DesignPanelValue::Number(value),
        ) => {
            value.is_finite()
                && *value >= 0.
                && node.supports_dimensions()
                && node.supports_section(DesignPanelSection::Layout)
        }
        (DesignPanelProperty::LockAspectRatio, DesignPanelValue::Bool(_)) => {
            !node.is_component_instance_child
                && node.supports_aspect_ratio_lock()
                && node.supports_dimensions()
                && node.supports_section(DesignPanelSection::Layout)
        }
        (
            DesignPanelProperty::HorizontalConstraint | DesignPanelProperty::VerticalConstraint,
            DesignPanelValue::Constraint(_),
        ) => node.supports_constraints() && node.supports_section(DesignPanelSection::Position),
        (DesignPanelProperty::Visible, DesignPanelValue::Bool(_)) => {
            node.supports_visibility() && node.supports_section(DesignPanelSection::Layer)
        }
        (DesignPanelProperty::BlendMode, DesignPanelValue::BlendMode(_)) => {
            node.supports_layer_appearance() && node.supports_section(DesignPanelSection::Layer)
        }
        (DesignPanelProperty::Opacity, DesignPanelValue::Number(value)) => {
            value.is_finite()
                && (0. ..=100.).contains(value)
                && node.supports_layer_appearance()
                && node.supports_section(DesignPanelSection::Layer)
        }
        (DesignPanelProperty::CornerRadius, DesignPanelValue::Number(value)) => {
            value.is_finite()
                && *value >= 0.
                && node.corner_capabilities.uniform_radius
                && node.supports_section(DesignPanelSection::Layer)
        }
        (
            DesignPanelProperty::CornerRadiusTopLeft
            | DesignPanelProperty::CornerRadiusTopRight
            | DesignPanelProperty::CornerRadiusBottomRight
            | DesignPanelProperty::CornerRadiusBottomLeft,
            DesignPanelValue::Number(value),
        ) => {
            value.is_finite()
                && *value >= 0.
                && node.corner_capabilities.independent_radii
                && node.supports_section(DesignPanelSection::Layer)
        }
        (DesignPanelProperty::IndependentCorners, DesignPanelValue::Bool(_)) => {
            node.corner_capabilities.independent_radii
                && node.supports_section(DesignPanelSection::Layer)
        }
        (DesignPanelProperty::CornerSmoothing, DesignPanelValue::Ratio(value)) => {
            value.is_finite()
                && (0. ..=1.).contains(value)
                && node.corner_capabilities.smoothing
                && node.supports_section(DesignPanelSection::Layer)
        }
        _ => false,
    }
}

fn story_collection_add_is_applicable(
    node: &DesignPanelNode,
    collection: DesignPanelCollection,
) -> bool {
    match collection {
        DesignPanelCollection::Fill => {
            node.supports_fill()
                && node.supports_section(DesignPanelSection::Fill)
                && node.fill_style_binding.is_none()
        }
        DesignPanelCollection::Stroke => {
            node.supports_stroke()
                && node.supports_section(DesignPanelSection::Stroke)
                && node.stroke_style_binding.is_none()
        }
        DesignPanelCollection::Effect => {
            story_effect_add_is_applicable(node, DesignEffectKind::DropShadow)
        }
        DesignPanelCollection::LayoutGrid => {
            node.supports_layout_guides()
                && node.supports_section(DesignPanelSection::LayoutGrid)
                && node.layout_grid_style_binding.is_none()
        }
        // Multi-node export has a dedicated exact-target reducer.
        DesignPanelCollection::Export => false,
    }
}

fn story_effect_add_is_applicable(node: &DesignPanelNode, kind: DesignEffectKind) -> bool {
    node.supports_effects()
        && node.supports_section(DesignPanelSection::Effects)
        && node.effect_style_binding.is_none()
        && node.can_use_effect_kind(kind, None)
}

fn apply_story_targeted_effect_add(
    nodes: &mut [DesignPanelNode],
    current_target: Option<&DesignPanelTarget>,
    requested_target: &DesignPanelTarget,
    can_edit: bool,
    kind: DesignEffectKind,
) -> bool {
    let Some(node_indices) =
        story_exact_editable_node_indices(nodes, current_target, requested_target, can_edit)
    else {
        return false;
    };
    if !node_indices
        .iter()
        .all(|index| story_effect_add_is_applicable(&nodes[*index], kind))
    {
        return false;
    }
    for index in node_indices {
        let node = &mut nodes[index];
        let effect_id = format!(
            "{}-effect-{}-{}",
            node.id,
            kind.label().to_ascii_lowercase().replace(' ', "-"),
            node.effects.len()
        );
        node.effects
            .push(DesignEffect::new(kind).with_id(effect_id));
    }
    true
}

fn apply_story_transform_request(
    nodes: &mut [DesignPanelNode],
    current_target: Option<&DesignPanelTarget>,
    requested_target: &DesignPanelTarget,
    can_edit: bool,
    operation: fanta_gpui::prelude::DesignTransformOperation,
) -> bool {
    let Some(node_indices) =
        story_exact_editable_node_indices(nodes, current_target, requested_target, can_edit)
    else {
        return false;
    };
    if !node_indices
        .iter()
        .all(|index| nodes[*index].supports_section(DesignPanelSection::Position))
    {
        return false;
    }
    if operation == fanta_gpui::prelude::DesignTransformOperation::RotateClockwise90 {
        for index in node_indices {
            let node = &mut nodes[index];
            node.rotation = (node.rotation + 90.).rem_euclid(360.);
        }
    }
    true
}

fn story_common_property_binding<'a>(
    bindings: &'a HashMap<
        (SharedString, DesignPanelProperty),
        DesignPanelPropertyBinding<DesignPanelValue>,
    >,
    node_ids: &[SharedString],
    property: DesignPanelProperty,
) -> Option<&'a DesignPanelPropertyBinding<DesignPanelValue>> {
    let first_id = node_ids.first()?;
    let binding = bindings.get(&(first_id.clone(), property))?;
    node_ids
        .iter()
        .skip(1)
        .all(|node_id| bindings.get(&(node_id.clone(), property)) == Some(binding))
        .then_some(binding)
}

fn story_node_edit_transaction(
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

fn begin_story_node_edit(
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

fn finish_story_node_edit(
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

fn apply_story_paint_edit_phase(
    paint: &mut DesignPaint,
    snapshots: &mut HashMap<StoryPaintEditTarget, DesignPaint>,
    target: StoryPaintEditTarget,
    edit: &DesignPaintEdit,
    phase: DesignPanelEditPhase,
) {
    match phase {
        DesignPanelEditPhase::Begin => {
            snapshots.entry(target).or_insert_with(|| paint.clone());
        }
        DesignPanelEditPhase::Preview => {
            paint.apply_edit(edit);
        }
        DesignPanelEditPhase::Commit => {
            paint.apply_edit(edit);
            snapshots.remove(&target);
        }
        DesignPanelEditPhase::Cancel => {
            // Paint Cancel does not promise that `edit` contains the original
            // value. Restore the mock host snapshot captured at Begin.
            if let Some(original) = snapshots.remove(&target) {
                *paint = original;
            }
        }
    }
}

fn apply_story_page_background_edit_phase(
    current: &mut DesignColor,
    snapshots: &mut HashMap<SharedString, DesignColor>,
    page_id: &SharedString,
    color: DesignColor,
    phase: DesignPanelEditPhase,
) {
    match phase {
        DesignPanelEditPhase::Begin => {
            snapshots.entry(page_id.clone()).or_insert(*current);
        }
        DesignPanelEditPhase::Preview => {
            snapshots.entry(page_id.clone()).or_insert(*current);
            *current = color;
        }
        DesignPanelEditPhase::Commit => {
            *current = color;
            snapshots.remove(page_id);
        }
        DesignPanelEditPhase::Cancel => {
            if let Some(original) = snapshots.remove(page_id) {
                *current = original;
            }
        }
    }
}

fn apply_story_text_path_flip_orientation(
    node: &mut DesignPanelNode,
    node_id: &SharedString,
) -> bool {
    if node.id != *node_id || node.kind != DesignPanelNodeKind::TextPath {
        return false;
    }
    let Some(text_path) = node.text_path.as_mut() else {
        return false;
    };
    if !text_path.can_flip_orientation {
        return false;
    }
    text_path.orientation = text_path.orientation.toggled();
    true
}

fn apply_story_text_path_edit_phase(
    node: &mut DesignPanelNode,
    snapshots: &mut HashMap<SharedString, Option<DesignTextPathStartData>>,
    node_id: &SharedString,
    data: DesignTextPathStartData,
    phase: DesignPanelEditPhase,
) {
    match phase {
        DesignPanelEditPhase::Begin => {
            snapshots
                .entry(node_id.clone())
                .or_insert(node.text_path_start_data);
        }
        DesignPanelEditPhase::Preview => {
            node.text_path_start_data = Some(data);
        }
        DesignPanelEditPhase::Commit => {
            node.text_path_start_data = Some(data);
            snapshots.remove(node_id);
        }
        DesignPanelEditPhase::Cancel => {
            if let Some(original) = snapshots.remove(node_id) {
                node.text_path_start_data = original;
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum StoryVectorEditChange {
    Selection(Vec<SharedString>),
    Position {
        vertex_ids: Vec<SharedString>,
        axis: DesignVectorCoordinateAxis,
        value: f32,
    },
    CornerRadius {
        vertex_ids: Vec<SharedString>,
        radius: f32,
    },
    HandleMirroring {
        vertex_ids: Vec<SharedString>,
        mirroring: DesignHandleMirroring,
    },
}

fn apply_story_vector_edit_phase(
    node: &mut DesignPanelNode,
    snapshots: &mut HashMap<SharedString, Option<DesignVectorEditViewData>>,
    node_id: &SharedString,
    change: StoryVectorEditChange,
    phase: DesignPanelEditPhase,
) -> bool {
    match phase {
        DesignPanelEditPhase::Begin => {
            let Some(mut candidate) = node.vector_edit.clone() else {
                return false;
            };
            if !apply_story_vector_edit_change(&mut candidate, &change) {
                return false;
            }
            snapshots
                .entry(node_id.clone())
                .or_insert_with(|| node.vector_edit.clone());
            true
        }
        DesignPanelEditPhase::Preview => node
            .vector_edit
            .as_mut()
            .is_some_and(|view_data| apply_story_vector_edit_change(view_data, &change)),
        DesignPanelEditPhase::Commit => {
            let applied = node
                .vector_edit
                .as_mut()
                .is_some_and(|view_data| apply_story_vector_edit_change(view_data, &change));
            snapshots.remove(node_id);
            applied
        }
        DesignPanelEditPhase::Cancel => snapshots.remove(node_id).is_some_and(|original| {
            node.vector_edit = original;
            true
        }),
    }
}

fn apply_story_vector_edit_change(
    view_data: &mut DesignVectorEditViewData,
    change: &StoryVectorEditChange,
) -> bool {
    if !view_data.is_valid() || view_data.read_only {
        return false;
    }
    match change {
        StoryVectorEditChange::Selection(selected_vertex_ids) => {
            if selected_vertex_ids.iter().enumerate().any(|(index, id)| {
                selected_vertex_ids[index + 1..].contains(id)
                    || view_data.vertex(id.as_ref()).is_none()
            }) {
                return false;
            }
            for vertex in &mut view_data.vertices {
                vertex.selected = selected_vertex_ids.contains(&vertex.id);
            }
            true
        }
        StoryVectorEditChange::Position {
            vertex_ids,
            axis,
            value,
        } => {
            if !value.is_finite()
                || !view_data.can_edit_coordinates()
                || !story_vector_targets_current_selection(view_data, vertex_ids)
            {
                return false;
            }
            for vertex in &mut view_data.vertices {
                if vertex_ids.contains(&vertex.id) {
                    match axis {
                        DesignVectorCoordinateAxis::X => vertex.position.x = *value,
                        DesignVectorCoordinateAxis::Y => vertex.position.y = *value,
                    }
                }
            }
            true
        }
        StoryVectorEditChange::CornerRadius { vertex_ids, radius } => {
            if !radius.is_finite()
                || *radius < 0.
                || !view_data.can_edit_corner_radius()
                || !story_vector_targets_current_selection(view_data, vertex_ids)
            {
                return false;
            }
            for vertex in &mut view_data.vertices {
                if vertex_ids.contains(&vertex.id) {
                    vertex.corner_radius = Some(*radius);
                }
            }
            true
        }
        StoryVectorEditChange::HandleMirroring {
            vertex_ids,
            mirroring,
        } => {
            if !view_data.can_edit_handle_mirroring()
                || !story_vector_targets_current_selection(view_data, vertex_ids)
            {
                return false;
            }
            for vertex in &mut view_data.vertices {
                if vertex_ids.contains(&vertex.id) {
                    vertex.handle_mirroring = Some(*mirroring);
                }
            }
            true
        }
    }
}

fn story_vector_targets_current_selection(
    view_data: &DesignVectorEditViewData,
    vertex_ids: &[SharedString],
) -> bool {
    let selected = view_data.selected_vertex_ids();
    view_data.contains_exact_vertices(vertex_ids)
        && selected.len() == vertex_ids.len()
        && selected.iter().all(|id| vertex_ids.contains(id))
}

fn story_paint_mut<'a>(
    node: &'a mut DesignPanelNode,
    collection: DesignPanelCollection,
    paint_id: &SharedString,
    index: usize,
) -> Option<&'a mut DesignPaint> {
    let paints = match collection {
        DesignPanelCollection::Fill if node.kind == DesignPanelNodeKind::MultipleSelection => {
            Some(&mut node.selection_colors)
        }
        DesignPanelCollection::Fill => Some(&mut node.fills),
        DesignPanelCollection::Stroke => node.stroke.as_mut().map(|stroke| &mut stroke.paints),
        DesignPanelCollection::Effect
        | DesignPanelCollection::LayoutGrid
        | DesignPanelCollection::Export => None,
    }?;
    let resolved = if paint_id.is_empty() {
        (index < paints.len()).then_some(index)
    } else {
        paints.iter().position(|paint| paint.id == *paint_id)
    }?;
    paints.get_mut(resolved)
}

fn story_paint_shader_editor_value(
    current: &DesignShaderPropertyValue,
) -> Option<DesignShaderPropertyValue> {
    let vector = fanta_gpui::design::DesignEffectVector::new;
    Some(match current {
        DesignShaderPropertyValue::Boolean(value) => DesignShaderPropertyValue::Boolean(!value),
        DesignShaderPropertyValue::Text(value) => {
            DesignShaderPropertyValue::Text(format!("{value} · edited").into())
        }
        DesignShaderPropertyValue::Number(value) => DesignShaderPropertyValue::Number(value + 0.25),
        DesignShaderPropertyValue::AssetId(_) => {
            DesignShaderPropertyValue::AssetId("image:replacement".into())
        }
        DesignShaderPropertyValue::Color(_) => DesignShaderPropertyValue::Color(DesignColor::BLUE),
        DesignShaderPropertyValue::Point(_) => DesignShaderPropertyValue::Point(vector(0.25, 0.75)),
        DesignShaderPropertyValue::Line { .. } => DesignShaderPropertyValue::Line {
            start: vector(0.2, 0.8),
            end: vector(0.8, 0.2),
        },
        DesignShaderPropertyValue::Circle { .. } => DesignShaderPropertyValue::Circle {
            center: vector(0.4, 0.6),
            radius: 0.42,
        },
        DesignShaderPropertyValue::CirclePoint { .. } => DesignShaderPropertyValue::CirclePoint {
            center: vector(0.45, 0.55),
            radius: 0.38,
            angle: 135.,
        },
        DesignShaderPropertyValue::ColorPoint { .. } => DesignShaderPropertyValue::ColorPoint {
            point: vector(0.65, 0.35),
            color: DesignColor::PURPLE,
            variable_id: None,
        },
        DesignShaderPropertyValue::Gradient(_) => DesignShaderPropertyValue::Gradient(vec![
            fanta_gpui::design::DesignShaderGradientStop::new(0., DesignColor::BLUE),
            fanta_gpui::design::DesignShaderGradientStop::new(0.5, DesignColor::WHITE),
            fanta_gpui::design::DesignShaderGradientStop::new(1., DesignColor::PURPLE),
        ]),
        DesignShaderPropertyValue::Opaque { type_name, .. } => DesignShaderPropertyValue::Opaque {
            type_name: type_name.clone(),
            payload: "{\"storybookEdited\":true}".into(),
        },
        DesignShaderPropertyValue::VariableAlias { .. } => return None,
    })
}

fn apply_story_paint_shader_property_editor(
    node: &mut DesignPanelNode,
    collection: DesignPanelCollection,
    paint_id: &SharedString,
    index: usize,
    definition_id: &SharedString,
) -> bool {
    let Some(paint) = story_paint_mut(node, collection, paint_id, index) else {
        return false;
    };
    let DesignPaintPayload::Shader(shader) = &paint.payload else {
        return false;
    };
    let Some(next) = shader
        .property(definition_id)
        .and_then(story_paint_shader_editor_value)
    else {
        return false;
    };
    paint.apply_edit(&DesignPaintEdit {
        property: DesignPaintProperty::ShaderProperty {
            definition_id: definition_id.clone(),
        },
        value: DesignPaintValue::ShaderProperty(next),
    })
}

fn story_selection_reference_paint<'a>(
    node: &'a DesignPanelNode,
    reference: &DesignSelectionPaintReference,
) -> Option<&'a DesignPaint> {
    if node.id != reference.node_id {
        return None;
    }
    let paints = match reference.collection {
        DesignSelectionPaintCollection::Fill => Some(node.fills.as_slice()),
        DesignSelectionPaintCollection::Stroke => {
            node.stroke.as_ref().map(|stroke| stroke.paints.as_slice())
        }
    }?;
    let resolved = if reference.paint_id.is_empty() {
        (reference.paint_index < paints.len()).then_some(reference.paint_index)
    } else {
        paints
            .iter()
            .position(|paint| paint.id == reference.paint_id)
    }?;
    paints.get(resolved)
}

fn story_selection_reference_paint_mut<'a>(
    node: &'a mut DesignPanelNode,
    reference: &DesignSelectionPaintReference,
) -> Option<&'a mut DesignPaint> {
    if node.id != reference.node_id {
        return None;
    }
    let paints = match reference.collection {
        DesignSelectionPaintCollection::Fill => Some(&mut node.fills),
        DesignSelectionPaintCollection::Stroke => {
            node.stroke.as_mut().map(|stroke| &mut stroke.paints)
        }
    }?;
    let resolved = if reference.paint_id.is_empty() {
        (reference.paint_index < paints.len()).then_some(reference.paint_index)
    } else {
        paints
            .iter()
            .position(|paint| paint.id == reference.paint_id)
    }?;
    paints.get_mut(resolved)
}

fn story_selection_paint_edit_for_occurrence(
    paint: &DesignPaint,
    edit: &DesignPaintEdit,
) -> Option<DesignPaintEdit> {
    let mut edit = edit.clone();
    match &mut edit.property {
        DesignPaintProperty::GradientStopColor { stop_id, index }
        | DesignPaintProperty::GradientStopPosition { stop_id, index }
        | DesignPaintProperty::GradientStopRemove { stop_id, index } => {
            let DesignPaintPayload::Gradient(gradient) = &paint.payload else {
                return None;
            };
            let target_stop = gradient.stops.get(*index)?;
            *stop_id = target_stop.id.clone();
        }
        _ => {}
    }
    match (&edit.property, &mut edit.value) {
        (DesignPaintProperty::GradientStopAdd, DesignPaintValue::GradientStop(stop)) => {
            let DesignPaintPayload::Gradient(gradient) = &paint.payload else {
                return None;
            };
            stop.id = if paint.id.is_empty() {
                "".into()
            } else {
                format!("{}-selection-stop-{}", paint.id, gradient.stops.len()).into()
            };
        }
        (
            DesignPaintProperty::Payload,
            DesignPaintValue::Payload(DesignPaintPayload::Gradient(gradient)),
        ) => {
            for (index, stop) in gradient.stops.iter_mut().enumerate() {
                stop.id = if paint.id.is_empty() {
                    "".into()
                } else {
                    format!("{}-selection-stop-{index}", paint.id).into()
                };
            }
        }
        _ => {}
    }
    Some(edit)
}

fn story_selection_color_target(
    reference: &DesignSelectionPaintReference,
) -> DesignPaintColorTarget {
    reference
        .gradient_stop_id
        .as_ref()
        .map_or(DesignPaintColorTarget::Solid, |stop_id| {
            DesignPaintColorTarget::GradientStop {
                stop_id: stop_id.clone(),
                index: reference.gradient_stop_index.unwrap_or_default(),
            }
        })
}

fn story_selection_references_are_current(
    nodes: &[DesignPanelNode],
    target: &DesignPanelTarget,
    references: &[DesignSelectionPaintReference],
) -> bool {
    let DesignPanelTarget::Nodes { node_ids } = target else {
        return false;
    };
    !node_ids.is_empty()
        && !references.is_empty()
        && node_ids
            .iter()
            .all(|node_id| nodes.iter().any(|node| node.id == *node_id))
        && references.iter().all(|reference| {
            node_ids.contains(&reference.node_id)
                && nodes
                    .iter()
                    .find(|node| node.id == reference.node_id)
                    .and_then(|node| story_selection_reference_paint(node, reference))
                    .is_some_and(|paint| {
                        if let Some(stop_id) = &reference.gradient_stop_id {
                            let DesignPaintPayload::Gradient(gradient) = &paint.payload else {
                                return false;
                            };
                            if stop_id.is_empty() {
                                reference
                                    .gradient_stop_index
                                    .is_some_and(|index| index < gradient.stops.len())
                            } else {
                                gradient.stops.iter().any(|stop| stop.id == *stop_id)
                            }
                        } else {
                            matches!(
                                &paint.payload,
                                DesignPaintPayload::Solid(_) | DesignPaintPayload::Gradient(_)
                            )
                        }
                    })
        })
}

fn story_selection_row_is_current(
    nodes: &[DesignPanelNode],
    target: &DesignPanelTarget,
    selection_color_id: &SharedString,
    references: &[DesignSelectionPaintReference],
) -> bool {
    if !story_selection_references_are_current(nodes, target, references) {
        return false;
    }
    let DesignPanelTarget::Nodes { node_ids } = target else {
        return false;
    };
    let selected = node_ids
        .iter()
        .filter_map(|node_id| nodes.iter().find(|node| node.id == *node_id))
        .collect::<Vec<_>>();
    selected.len() == node_ids.len()
        && aggregate_story_selection_colors(selected)
            .color(selection_color_id.as_ref())
            .is_some_and(|row| row.paint_references == references)
}

fn story_selection_collection_targets(
    references: &[DesignSelectionPaintReference],
) -> Vec<(SharedString, DesignSelectionPaintCollection)> {
    let mut targets = Vec::new();
    for reference in references {
        let target = (reference.node_id.clone(), reference.collection);
        if !targets.contains(&target) {
            targets.push(target);
        }
    }
    targets
}

fn story_selection_collection_style_binding(
    node: &DesignPanelNode,
    collection: DesignSelectionPaintCollection,
) -> Option<&DesignPaintStyleBinding> {
    match collection {
        DesignSelectionPaintCollection::Fill => node.fill_style_binding.as_ref(),
        DesignSelectionPaintCollection::Stroke => node.stroke_style_binding.as_ref(),
    }
}

fn story_design_collection(collection: DesignSelectionPaintCollection) -> DesignPanelCollection {
    match collection {
        DesignSelectionPaintCollection::Fill => DesignPanelCollection::Fill,
        DesignSelectionPaintCollection::Stroke => DesignPanelCollection::Stroke,
    }
}

fn apply_story_paint_style(
    node: &mut DesignPanelNode,
    collection: DesignPanelCollection,
    selection: &DesignPaintStyleSelection,
    style: DesignPaintStyle,
) -> bool {
    if style.import_state != DesignPaintStyleImportState::Imported || style.paints.is_empty() {
        return false;
    }
    let mut paints = style.paints;
    assign_story_applied_style_paint_ids(&node.id, collection, &selection.style_id, &mut paints);
    let binding = DesignPaintStyleBinding::new(selection.clone(), style.name);
    match collection {
        DesignPanelCollection::Fill => {
            node.fills = paints;
            node.fill_style_binding = Some(binding);
            true
        }
        DesignPanelCollection::Stroke => {
            if node.stroke.is_none() {
                let first_paint = paints
                    .first()
                    .cloned()
                    .expect("non-empty Paint style checked above");
                node.stroke = Some(DesignStroke::for_node(
                    node.kind,
                    first_paint,
                    1.,
                    DesignStrokeAlign::Center,
                ));
            }
            node.stroke
                .as_mut()
                .expect("stroke was created above")
                .paints = paints;
            node.stroke_style_binding = Some(binding);
            true
        }
        DesignPanelCollection::Effect
        | DesignPanelCollection::LayoutGrid
        | DesignPanelCollection::Export => false,
    }
}

fn assign_story_applied_style_paint_ids(
    node_id: &SharedString,
    collection: DesignPanelCollection,
    style_id: &SharedString,
    paints: &mut [DesignPaint],
) {
    for (paint_index, paint) in paints.iter_mut().enumerate() {
        paint.id = format!(
            "{node_id}-{}-{style_id}-{paint_index}",
            collection.label().to_ascii_lowercase()
        )
        .into();
        if let DesignPaintPayload::Gradient(gradient) = &mut paint.payload {
            for (stop_index, stop) in gradient.stops.iter_mut().enumerate() {
                stop.id = format!("{}-stop-{stop_index}", paint.id).into();
            }
            paint.sync_legacy_projection();
        }
    }
}

fn apply_story_paint_variable(
    paint: &mut DesignPaint,
    target: &DesignPaintColorTarget,
    variable: &DesignVariable,
) -> bool {
    let Some(DesignVariableResolvedValue::Color(color)) = variable.resolved_value.as_ref() else {
        return false;
    };
    let mut binding = DesignPaintBinding::new(variable.id.clone(), variable.name.clone());
    binding.collection_name = Some(variable.collection_name.clone());
    let applied = match (&mut paint.payload, target) {
        (DesignPaintPayload::Solid(solid), DesignPaintColorTarget::Solid) => {
            solid.color = *color;
            solid.binding = Some(binding);
            true
        }
        (
            DesignPaintPayload::Gradient(gradient),
            DesignPaintColorTarget::GradientStop { stop_id, index },
        ) => {
            let stop = if stop_id.is_empty() {
                gradient.stops.get_mut(*index)
            } else {
                gradient.stops.iter_mut().find(|stop| stop.id == *stop_id)
            };
            stop.is_some_and(|stop| {
                stop.color = *color;
                stop.binding = Some(binding);
                true
            })
        }
        _ => false,
    };
    if applied {
        paint.sync_legacy_projection();
    }
    applied
}

fn detach_story_paint_variable(
    paint: &mut DesignPaint,
    target: &DesignPaintColorTarget,
    variable_id: &SharedString,
) -> bool {
    let detached = match (&mut paint.payload, target) {
        (DesignPaintPayload::Solid(solid), DesignPaintColorTarget::Solid) => {
            let matches = solid
                .binding
                .as_ref()
                .is_some_and(|binding| binding.variable_id == *variable_id);
            if matches {
                solid.binding = None;
            }
            matches
        }
        (
            DesignPaintPayload::Gradient(gradient),
            DesignPaintColorTarget::GradientStop { stop_id, index },
        ) => {
            let stop = if stop_id.is_empty() {
                gradient.stops.get_mut(*index)
            } else {
                gradient.stops.iter_mut().find(|stop| stop.id == *stop_id)
            };
            stop.is_some_and(|stop| {
                let matches = stop
                    .binding
                    .as_ref()
                    .is_some_and(|binding| binding.variable_id == *variable_id);
                if matches {
                    stop.binding = None;
                }
                matches
            })
        }
        _ => false,
    };
    if detached {
        paint.sync_legacy_projection();
    }
    detached
}

fn resolve_story_shader_mut<'a>(
    view_data: &'a mut DesignShaderViewData,
    selection: &DesignShaderSelection,
) -> Option<&'a mut DesignShaderDefinition> {
    match &selection.source {
        DesignShaderSource::Page => view_data
            .page_shaders
            .iter_mut()
            .find(|shader| shader.id == selection.shader_id),
        DesignShaderSource::Library { library_id } => view_data
            .libraries
            .iter_mut()
            .find(|library| library.id == *library_id)?
            .shaders
            .iter_mut()
            .find(|shader| shader.id == selection.shader_id),
    }
}

fn story_fractal_shader_definition() -> DesignShaderDefinition {
    let vector = fanta_gpui::design::DesignEffectVector::new;
    DesignShaderDefinition::new(
        "shader:page:fractal-noise",
        "Fractal noise",
        true,
        [
            DesignShaderPropertyDefinition::new(
                "def:scale",
                "Scale",
                DesignShaderPropertyKind::Number,
            )
            .with_default(DesignShaderPropertyValue::Number(4.))
            .with_description("Frequency multiplier"),
            DesignShaderPropertyDefinition::new(
                "def:tint",
                "Tint",
                DesignShaderPropertyKind::Color,
            )
            .with_default(DesignShaderPropertyValue::Color(DesignColor::PURPLE)),
            DesignShaderPropertyDefinition::new(
                "def:animate",
                "Animate",
                DesignShaderPropertyKind::Boolean,
            )
            .with_default(DesignShaderPropertyValue::Boolean(true)),
            DesignShaderPropertyDefinition::new(
                "def:center",
                "Center",
                DesignShaderPropertyKind::Point,
            )
            .with_default(DesignShaderPropertyValue::Point(vector(0.5, 0.5))),
            DesignShaderPropertyDefinition::new(
                "def:ray",
                "Refraction line",
                DesignShaderPropertyKind::Line,
            )
            .with_default(DesignShaderPropertyValue::Line {
                start: vector(0.1, 0.25),
                end: vector(0.9, 0.75),
            }),
            DesignShaderPropertyDefinition::new(
                "def:lens",
                "Lens",
                DesignShaderPropertyKind::Circle,
            )
            .with_default(DesignShaderPropertyValue::Circle {
                center: vector(0.45, 0.5),
                radius: 0.32,
            }),
            DesignShaderPropertyDefinition::new(
                "def:orbit",
                "Highlight orbit",
                DesignShaderPropertyKind::CirclePoint,
            )
            .with_default(DesignShaderPropertyValue::CirclePoint {
                center: vector(0.5, 0.5),
                radius: 0.4,
                angle: 40.,
            }),
            DesignShaderPropertyDefinition::new(
                "def:color-point",
                "Chromatic focus",
                DesignShaderPropertyKind::ColorPoint,
            )
            .with_default(DesignShaderPropertyValue::ColorPoint {
                point: vector(0.35, 0.65),
                color: DesignColor::BLUE,
                variable_id: None,
            }),
            DesignShaderPropertyDefinition::new(
                "def:gradient",
                "Dispersion gradient",
                DesignShaderPropertyKind::Gradient,
            )
            .with_default(DesignShaderPropertyValue::Gradient(vec![
                fanta_gpui::design::DesignShaderGradientStop::new(0., DesignColor::PURPLE),
                fanta_gpui::design::DesignShaderGradientStop::new(0.55, DesignColor::BLUE),
                fanta_gpui::design::DesignShaderGradientStop::new(1., DesignColor::WHITE),
            ])),
            DesignShaderPropertyDefinition::new(
                "def:future",
                "Future payload",
                DesignShaderPropertyKind::Unsupported,
            )
            .with_default(DesignShaderPropertyValue::Opaque {
                type_name: "MESH_PATCH".into(),
                payload: "{\"patches\":4,\"mode\":\"future\"}".into(),
            }),
        ],
    )
}

fn imported_story_shader_properties(shader_id: &str) -> Vec<DesignShaderPropertyDefinition> {
    match shader_id {
        "shader:library:liquid-metal" => vec![
            DesignShaderPropertyDefinition::new(
                "def:flow",
                "Flow",
                DesignShaderPropertyKind::Number,
            )
            .with_default(DesignShaderPropertyValue::Number(0.65)),
            DesignShaderPropertyDefinition::new(
                "def:color",
                "Metal color",
                DesignShaderPropertyKind::Color,
            )
            .with_default(DesignShaderPropertyValue::Color(DesignColor::WHITE)),
        ],
        _ => Vec::new(),
    }
}

fn story_shader_property_fallback(kind: DesignShaderPropertyKind) -> DesignShaderPropertyValue {
    let origin = fanta_gpui::design::DesignEffectVector::new(0., 0.);
    match kind {
        DesignShaderPropertyKind::Boolean => DesignShaderPropertyValue::Boolean(false),
        DesignShaderPropertyKind::Text => DesignShaderPropertyValue::Text(SharedString::default()),
        DesignShaderPropertyKind::Number => DesignShaderPropertyValue::Number(0.),
        DesignShaderPropertyKind::Image => {
            DesignShaderPropertyValue::AssetId("image:storybook-default".into())
        }
        DesignShaderPropertyKind::InstanceSwap => {
            DesignShaderPropertyValue::AssetId("component:storybook-default".into())
        }
        DesignShaderPropertyKind::Slot => {
            DesignShaderPropertyValue::AssetId("slot:storybook-default".into())
        }
        DesignShaderPropertyKind::Color => DesignShaderPropertyValue::Color(DesignColor::BLACK),
        DesignShaderPropertyKind::Point => DesignShaderPropertyValue::Point(origin),
        DesignShaderPropertyKind::Line => DesignShaderPropertyValue::Line {
            start: origin,
            end: fanta_gpui::design::DesignEffectVector::new(1., 1.),
        },
        DesignShaderPropertyKind::Circle => DesignShaderPropertyValue::Circle {
            center: origin,
            radius: 0.,
        },
        DesignShaderPropertyKind::CirclePoint => DesignShaderPropertyValue::CirclePoint {
            center: origin,
            radius: 0.,
            angle: 0.,
        },
        DesignShaderPropertyKind::ColorPoint => DesignShaderPropertyValue::ColorPoint {
            point: origin,
            color: DesignColor::BLACK,
            variable_id: None,
        },
        DesignShaderPropertyKind::Gradient => DesignShaderPropertyValue::Gradient(vec![
            fanta_gpui::design::DesignShaderGradientStop::new(0., DesignColor::BLACK),
            fanta_gpui::design::DesignShaderGradientStop::new(1., DesignColor::WHITE),
        ]),
        DesignShaderPropertyKind::Unsupported => DesignShaderPropertyValue::Opaque {
            type_name: "Unsupported".into(),
            payload: "Restored by the Storybook host".into(),
        },
    }
}

fn story_effect_shader_property_mut<'a>(
    node: &'a mut DesignPanelNode,
    effect_id: &SharedString,
    effect_index: usize,
    shader_property_id: &SharedString,
    property_index: usize,
) -> Option<&'a mut DesignShaderProperty> {
    let effect_index = if effect_id.is_empty() {
        (effect_index < node.effects.len()).then_some(effect_index)
    } else {
        node.effect_index_by_id(effect_id.as_ref())
    }?;
    let effect = node.effects.get_mut(effect_index)?;
    let DesignEffectSettings::Shader(shader) = &mut effect.settings else {
        return None;
    };
    let property_index = if shader_property_id.is_empty() {
        (property_index < shader.properties.len()).then_some(property_index)
    } else {
        shader
            .properties
            .iter()
            .position(|property| property.definition_id == *shader_property_id)
    }?;
    shader.properties.get_mut(property_index)
}

#[allow(clippy::too_many_arguments)]
fn apply_story_effect_shader_property_editor(
    node: &mut DesignPanelNode,
    effect_id: &SharedString,
    effect_index: usize,
    shader_property_id: &SharedString,
    property_index: usize,
    property_kind: DesignShaderPropertyKind,
    target: DesignShaderPropertyEditorTarget,
    editor: DesignShaderPropertyEditorKind,
    current_value: &DesignShaderPropertyValue,
) -> bool {
    let Some(property) = story_effect_shader_property_mut(
        node,
        effect_id,
        effect_index,
        shader_property_id,
        property_index,
    ) else {
        return false;
    };
    if property.kind != property_kind || property.read_only || property.value != *current_value {
        return false;
    }
    match (editor, target) {
        (DesignShaderPropertyEditorKind::Resource, DesignShaderPropertyEditorTarget::Value) => {
            let asset_id = match property.kind {
                DesignShaderPropertyKind::Image => "image:storybook-replacement",
                DesignShaderPropertyKind::InstanceSwap => "component:storybook-replacement",
                DesignShaderPropertyKind::Slot => "slot:storybook-replacement",
                _ => return false,
            };
            property.value = DesignShaderPropertyValue::AssetId(asset_id.into());
        }
        (DesignShaderPropertyEditorKind::Variable, DesignShaderPropertyEditorTarget::Value) => {
            property.value = DesignShaderPropertyValue::VariableAlias {
                variable_id: format!("variable:shader:{}", property.definition_id).into(),
            };
        }
        (
            DesignShaderPropertyEditorKind::Variable,
            DesignShaderPropertyEditorTarget::ColorPointColor,
        ) => {
            let DesignShaderPropertyValue::ColorPoint { variable_id, .. } = &mut property.value
            else {
                return false;
            };
            *variable_id = Some(format!("variable:shader:{}", property.definition_id).into());
        }
        (
            DesignShaderPropertyEditorKind::Variable,
            DesignShaderPropertyEditorTarget::GradientStopColor(stop_index),
        ) => {
            let DesignShaderPropertyValue::Gradient(stops) = &mut property.value else {
                return false;
            };
            let Some(stop) = stops.get_mut(stop_index) else {
                return false;
            };
            stop.variable_id = Some(
                format!(
                    "variable:shader:{}:stop:{stop_index}",
                    property.definition_id
                )
                .into(),
            );
        }
        (DesignShaderPropertyEditorKind::Resource, _) => return false,
    }
    true
}

fn apply_story_effect_shader_variable_detach(
    node: &mut DesignPanelNode,
    effect_id: &SharedString,
    effect_index: usize,
    shader_property_id: &SharedString,
    property_index: usize,
    target: DesignShaderPropertyEditorTarget,
    variable_id: &SharedString,
) -> bool {
    let Some(property) = story_effect_shader_property_mut(
        node,
        effect_id,
        effect_index,
        shader_property_id,
        property_index,
    ) else {
        return false;
    };
    match target {
        DesignShaderPropertyEditorTarget::Value => {
            let DesignShaderPropertyValue::VariableAlias {
                variable_id: current,
            } = &property.value
            else {
                return false;
            };
            if current != variable_id {
                return false;
            }
            property.value = story_shader_property_fallback(property.kind);
        }
        DesignShaderPropertyEditorTarget::ColorPointColor => {
            let DesignShaderPropertyValue::ColorPoint {
                variable_id: current,
                ..
            } = &mut property.value
            else {
                return false;
            };
            if current.as_ref() != Some(variable_id) {
                return false;
            }
            *current = None;
        }
        DesignShaderPropertyEditorTarget::GradientStopColor(stop_index) => {
            let DesignShaderPropertyValue::Gradient(stops) = &mut property.value else {
                return false;
            };
            let Some(stop) = stops.get_mut(stop_index) else {
                return false;
            };
            if stop.variable_id.as_ref() != Some(variable_id) {
                return false;
            }
            stop.variable_id = None;
        }
    }
    true
}

fn resolve_story_color_style_sample<'a>(
    view_data: &'a DesignColorStyleSampleViewData,
    selection: &DesignColorStyleSampleSelection,
) -> Option<&'a DesignColorStyleSample> {
    match &selection.source {
        DesignColorStyleSampleSource::Page => view_data
            .page_samples
            .iter()
            .find(|sample| sample.id == selection.sample_id),
        DesignColorStyleSampleSource::Library { library_id } => view_data
            .libraries
            .iter()
            .find(|library| library.id == *library_id)?
            .samples
            .iter()
            .find(|sample| sample.id == selection.sample_id),
    }
}

fn apply_story_color_style_sample(
    paint: &mut DesignPaint,
    target: &DesignPaintColorTarget,
    color: DesignColor,
) -> bool {
    let applied = match (&mut paint.payload, target) {
        (DesignPaintPayload::Solid(solid), DesignPaintColorTarget::Solid)
            if solid.binding.is_none() =>
        {
            solid.color = DesignColor::rgb(color.red, color.green, color.blue);
            paint.opacity = f32::from(color.alpha) / 255. * 100.;
            true
        }
        (
            DesignPaintPayload::Gradient(gradient),
            DesignPaintColorTarget::GradientStop { stop_id, index },
        ) => {
            let stop = if stop_id.is_empty() {
                gradient.stops.get_mut(*index)
            } else {
                gradient.stops.iter_mut().find(|stop| stop.id == *stop_id)
            };
            stop.is_some_and(|stop| {
                if stop.binding.is_some() {
                    return false;
                }
                stop.color = color;
                true
            })
        }
        _ => false,
    };
    if applied {
        paint.sync_legacy_projection();
    }
    applied
}

fn resolve_story_color_style<'a>(
    view_data: &'a DesignColorStyleViewData,
    selection: &DesignColorStyleSelection,
) -> Option<&'a DesignColorStyle> {
    match &selection.source {
        DesignColorStyleSource::Page => view_data
            .page_styles
            .iter()
            .find(|style| style.id == selection.style_id),
        DesignColorStyleSource::Library { library_id } => view_data
            .libraries
            .iter()
            .find(|library| library.id == *library_id)
            .and_then(|library| {
                library
                    .styles
                    .iter()
                    .find(|style| style.id == selection.style_id)
            }),
    }
}

fn apply_story_color_style(
    paint: &mut DesignPaint,
    target: &DesignPaintColorTarget,
    style: &DesignColorStyle,
) -> bool {
    let applied = match (&mut paint.payload, target) {
        (DesignPaintPayload::Solid(solid), DesignPaintColorTarget::Solid) => {
            solid.color = style.color;
            solid.binding = style.binding.clone();
            true
        }
        (
            DesignPaintPayload::Gradient(gradient),
            DesignPaintColorTarget::GradientStop { stop_id, index },
        ) => {
            let stop = if stop_id.is_empty() {
                gradient.stops.get_mut(*index)
            } else {
                gradient.stops.iter_mut().find(|stop| stop.id == *stop_id)
            };
            stop.is_some_and(|stop| {
                stop.color = style.color;
                stop.binding = style.binding.clone();
                true
            })
        }
        _ => false,
    };
    if applied {
        paint.sync_legacy_projection();
    }
    applied
}

fn apply_story_surface_change_request(
    active: &mut DesignPanelSurface,
    can_edit: bool,
    current: DesignPanelSurface,
    requested: DesignPanelSurface,
) -> bool {
    if *active != current
        || current == requested
        || !current.is_available(can_edit)
        || !requested.is_available(can_edit)
    {
        return false;
    }
    *active = requested;
    true
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum StoryAddAutoLayoutEcho {
    Converted {
        node_id: SharedString,
    },
    Wrapped {
        wrapper_id: SharedString,
        child_ids: Vec<SharedString>,
    },
}

fn story_design_target(context: &DesignPanelInspectionContext) -> Option<DesignPanelTarget> {
    (!context.selection().is_empty()).then(|| DesignPanelTarget::Nodes {
        node_ids: context
            .selection()
            .items()
            .iter()
            .map(|node| node.id.clone())
            .collect(),
    })
}

fn story_is_section_or_transform_action(action: &DesignPanelAction) -> bool {
    matches!(
        action,
        DesignPanelAction::SectionShareRequested { .. }
            | DesignPanelAction::SectionResolveChangedStatusRequested { .. }
            | DesignPanelAction::TransformModifierAddRequested { .. }
            | DesignPanelAction::TransformModifierRemoveRequested { .. }
            | DesignPanelAction::TransformModifierChangeRequested { .. }
            | DesignPanelAction::ApplyTransformModifiersRequested { .. }
    )
}

fn story_section_or_transform_action_status(
    action: &DesignPanelAction,
    accepted: bool,
) -> SharedString {
    if accepted {
        format!("Host accepted exact Section/Transform intent {action:?}").into()
    } else {
        format!(
            "Host rejected stale, permission-invalid, capability-disabled, or inapplicable Section/Transform intent {action:?}"
        )
        .into()
    }
}

fn story_target_is_exact_single_node(
    current_target: Option<&DesignPanelTarget>,
    node_id: &SharedString,
) -> bool {
    matches!(
        current_target,
        Some(DesignPanelTarget::Nodes { node_ids })
            if node_ids.len() == 1 && node_ids.first() == Some(node_id)
    )
}

fn apply_story_section_or_transform_action(
    node: &mut DesignPanelNode,
    current_target: Option<&DesignPanelTarget>,
    can_edit: bool,
    action: &DesignPanelAction,
) -> bool {
    let action_node_id = match action {
        DesignPanelAction::SectionShareRequested { node_id }
        | DesignPanelAction::SectionResolveChangedStatusRequested { node_id }
        | DesignPanelAction::TransformModifierAddRequested { node_id, .. }
        | DesignPanelAction::TransformModifierRemoveRequested { node_id, .. }
        | DesignPanelAction::TransformModifierChangeRequested { node_id, .. }
        | DesignPanelAction::ApplyTransformModifiersRequested { node_id } => node_id,
        _ => return false,
    };
    if action_node_id != &node.id
        || !story_target_is_exact_single_node(current_target, action_node_id)
    {
        return false;
    }

    match action {
        DesignPanelAction::SectionShareRequested { .. } => {
            node.supports_section(DesignPanelSection::Section)
                && node
                    .section
                    .as_ref()
                    .is_some_and(|section| section.capabilities.share)
        }
        DesignPanelAction::SectionResolveChangedStatusRequested { .. } => {
            let can_resolve = can_edit
                && node.supports_section(DesignPanelSection::Section)
                && node.section.as_ref().is_some_and(|section| {
                    section.capabilities.resolve_changed_status
                        && section
                            .dev_status
                            .as_ref()
                            .is_some_and(|status| status.changed)
                });
            if can_resolve
                && let Some(status) = node
                    .section
                    .as_mut()
                    .and_then(|section| section.dev_status.as_mut())
            {
                status.changed = false;
            }
            can_resolve
        }
        DesignPanelAction::TransformModifierAddRequested { repeat_type, .. } => {
            if !can_edit
                || node.kind != DesignPanelNodeKind::TransformGroup
                || !node.supports_section(DesignPanelSection::Transform)
            {
                return false;
            }
            let mut suffix = node.transform_modifiers.len();
            let modifier_id = loop {
                let candidate = format!("{}-repeat-host-{suffix}", node.id);
                if node
                    .transform_modifiers
                    .iter()
                    .all(|modifier| modifier.id.as_ref() != candidate.as_str())
                {
                    break candidate;
                }
                suffix += 1;
            };
            node.transform_modifiers.push(match repeat_type {
                DesignRepeatType::Linear => {
                    DesignRepeatModifier::linear(modifier_id, DesignRepeatAxis::Horizontal)
                }
                DesignRepeatType::Radial => DesignRepeatModifier::radial(modifier_id),
            });
            true
        }
        DesignPanelAction::TransformModifierRemoveRequested {
            modifier_id, index, ..
        } => {
            let can_remove = can_edit
                && node.kind == DesignPanelNodeKind::TransformGroup
                && node.supports_section(DesignPanelSection::Transform)
                && node
                    .transform_modifiers
                    .get(*index)
                    .is_some_and(|modifier| modifier.id == *modifier_id);
            if can_remove {
                node.transform_modifiers.remove(*index);
            }
            can_remove
        }
        DesignPanelAction::TransformModifierChangeRequested {
            modifier_id,
            index,
            change,
            phase,
            ..
        } => {
            let change_is_valid = match change {
                DesignTransformModifierChange::Count(count) => *count >= 1,
                DesignTransformModifierChange::Offset(offset) => offset.is_finite(),
                DesignTransformModifierChange::Mode(_) | DesignTransformModifierChange::Unit(_) => {
                    true
                }
            };
            let can_change = can_edit
                && node.kind == DesignPanelNodeKind::TransformGroup
                && node.supports_section(DesignPanelSection::Transform)
                && change_is_valid
                && node
                    .transform_modifiers
                    .get(*index)
                    .is_some_and(|modifier| modifier.id == *modifier_id);
            if !can_change {
                return false;
            }
            if *phase != DesignPanelEditPhase::Begin {
                let modifier = &mut node.transform_modifiers[*index];
                match change {
                    DesignTransformModifierChange::Mode(mode) => modifier.mode = *mode,
                    DesignTransformModifierChange::Count(count) => modifier.count = *count,
                    DesignTransformModifierChange::Unit(unit) => modifier.unit = *unit,
                    DesignTransformModifierChange::Offset(offset) => modifier.offset = *offset,
                }
            }
            true
        }
        DesignPanelAction::ApplyTransformModifiersRequested { .. } => {
            let can_apply = can_edit
                && node.kind == DesignPanelNodeKind::TransformGroup
                && node.supports_section(DesignPanelSection::Transform)
                && !node.transform_modifiers.is_empty();
            if can_apply {
                node.transform_modifiers.clear();
            }
            can_apply
        }
        _ => false,
    }
}

fn story_draw_appearance_view_data(
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

fn story_export_target(context: &DesignPanelInspectionContext) -> DesignPanelTarget {
    story_design_target(context).unwrap_or_else(|| DesignPanelTarget::Page {
        page_id: "storybook-page".into(),
    })
}

fn story_export_target_keys(
    nodes: &[DesignPanelNode],
    current_target: &DesignPanelTarget,
    requested_target: &DesignPanelTarget,
    can_export: bool,
) -> Option<Vec<SharedString>> {
    if !can_export || current_target != requested_target {
        return None;
    }
    match requested_target {
        DesignPanelTarget::Page { page_id } => Some(vec![page_id.clone()]),
        DesignPanelTarget::Nodes { node_ids } => {
            let unique = node_ids
                .iter()
                .map(SharedString::as_ref)
                .collect::<HashSet<_>>();
            (node_ids.len() == unique.len()
                && !node_ids.is_empty()
                && node_ids
                    .iter()
                    .all(|node_id| nodes.iter().any(|node| node.id == *node_id)))
            .then(|| node_ids.clone())
        }
    }
}

fn story_uniform_export_configurations(
    configurations: &HashMap<SharedString, Vec<DesignExportConfiguration>>,
    target_keys: &[SharedString],
) -> Option<Vec<DesignExportConfiguration>> {
    let first_key = target_keys.first()?;
    let first = configurations.get(first_key).cloned().unwrap_or_default();
    target_keys
        .iter()
        .skip(1)
        .all(|key| configurations.get(key).cloned().unwrap_or_default() == first)
        .then_some(first)
}

fn story_static_export_capabilities(node: &DesignPanelNode) -> DesignStaticExportCapabilities {
    let text = matches!(
        node.kind,
        DesignPanelNodeKind::Text | DesignPanelNodeKind::TextPath
    );
    DesignStaticExportCapabilities {
        can_include_text_bounding_box: text,
        can_include_svg_bounds: text,
        svg_outline_text_default: text,
        svg_simplify_stroke_default: !text,
    }
}

fn story_aggregate_static_export_capabilities<'a>(
    nodes: impl IntoIterator<Item = &'a DesignPanelNode>,
) -> DesignStaticExportCapabilities {
    let capabilities = nodes
        .into_iter()
        .map(story_static_export_capabilities)
        .collect::<Vec<_>>();
    if capabilities.is_empty() {
        return DesignStaticExportCapabilities::default();
    }
    DesignStaticExportCapabilities {
        can_include_text_bounding_box: capabilities
            .iter()
            .all(|capability| capability.can_include_text_bounding_box),
        can_include_svg_bounds: capabilities
            .iter()
            .all(|capability| capability.can_include_svg_bounds),
        svg_outline_text_default: capabilities
            .iter()
            .all(|capability| capability.svg_outline_text_default),
        svg_simplify_stroke_default: capabilities
            .iter()
            .all(|capability| capability.svg_simplify_stroke_default),
    }
}

fn story_export_projection(
    target: DesignPanelTarget,
    nodes: &[DesignPanelNode],
    configurations: &HashMap<SharedString, Vec<DesignExportConfiguration>>,
    modes: &HashMap<SharedString, DesignExportMode>,
    animated_exports: &HashMap<SharedString, DesignAnimatedExportViewData>,
    previews: &HashMap<SharedString, DesignExportPreviewState>,
) -> DesignExportViewData {
    match &target {
        DesignPanelTarget::Page { page_id } => DesignExportViewData {
            target: target.clone(),
            configurations: configurations.get(page_id).cloned().unwrap_or_default(),
            mode: modes.get(page_id).copied().unwrap_or_default(),
            static_capabilities: Default::default(),
            preview: previews.get(page_id).cloned(),
            animated: animated_exports.get(page_id).cloned(),
        },
        DesignPanelTarget::Nodes { node_ids } if node_ids.len() == 1 => {
            let node_id = &node_ids[0];
            let node = nodes.iter().find(|node| node.id == *node_id);
            DesignExportViewData {
                target: target.clone(),
                configurations: configurations.get(node_id).cloned().unwrap_or_default(),
                mode: modes.get(node_id).copied().unwrap_or_default(),
                static_capabilities: node
                    .map(story_static_export_capabilities)
                    .unwrap_or_default(),
                preview: previews.get(node_id).cloned(),
                animated: animated_exports.get(node_id).cloned(),
            }
        }
        DesignPanelTarget::Nodes { node_ids } => {
            let selected_nodes = node_ids
                .iter()
                .filter_map(|node_id| nodes.iter().find(|node| node.id == *node_id))
                .collect::<Vec<_>>();
            let exact_target = !node_ids.is_empty()
                && selected_nodes.len() == node_ids.len()
                && node_ids
                    .iter()
                    .map(SharedString::as_ref)
                    .collect::<HashSet<_>>()
                    .len()
                    == node_ids.len();
            DesignExportViewData {
                target: target.clone(),
                configurations: if exact_target {
                    story_uniform_export_configurations(configurations, node_ids)
                        .unwrap_or_default()
                } else {
                    Vec::new()
                },
                // Figma exposes multi-layer export as static export. Motion export,
                // previews, and the Static/Animated switch are single-target only.
                mode: DesignExportMode::Static,
                static_capabilities: if exact_target {
                    story_aggregate_static_export_capabilities(selected_nodes)
                } else {
                    DesignStaticExportCapabilities::default()
                },
                preview: None,
                animated: None,
            }
        }
    }
}

fn apply_story_export_configuration_add(
    nodes: &[DesignPanelNode],
    configurations: &mut HashMap<SharedString, Vec<DesignExportConfiguration>>,
    current_target: &DesignPanelTarget,
    requested_target: &DesignPanelTarget,
    can_export: bool,
    configuration: DesignExportConfiguration,
) -> bool {
    let Some(target_keys) =
        story_export_target_keys(nodes, current_target, requested_target, can_export)
    else {
        return false;
    };
    let Some(current) = story_uniform_export_configurations(configurations, &target_keys) else {
        return false;
    };
    if current
        .iter()
        .any(|candidate| candidate.id == configuration.id)
    {
        return false;
    }
    for key in target_keys {
        configurations
            .entry(key)
            .or_default()
            .push(configuration.clone());
    }
    true
}

fn apply_story_export_configuration_remove(
    nodes: &[DesignPanelNode],
    configurations: &mut HashMap<SharedString, Vec<DesignExportConfiguration>>,
    current_target: &DesignPanelTarget,
    requested_target: &DesignPanelTarget,
    can_export: bool,
    configuration_id: &SharedString,
) -> bool {
    let Some(target_keys) =
        story_export_target_keys(nodes, current_target, requested_target, can_export)
    else {
        return false;
    };
    let Some(current) = story_uniform_export_configurations(configurations, &target_keys) else {
        return false;
    };
    if !current
        .iter()
        .any(|configuration| configuration.id == *configuration_id)
    {
        return false;
    }
    for key in target_keys {
        configurations
            .entry(key)
            .or_default()
            .retain(|configuration| configuration.id != *configuration_id);
    }
    true
}

struct StoryExportConfigurationEdit<'a> {
    current_target: &'a DesignPanelTarget,
    requested_target: &'a DesignPanelTarget,
    can_export: bool,
    configuration_id: &'a SharedString,
    change: &'a DesignExportConfigurationChange,
    phase: DesignPanelEditPhase,
}

fn apply_story_export_configuration_edit(
    nodes: &[DesignPanelNode],
    configurations: &mut HashMap<SharedString, Vec<DesignExportConfiguration>>,
    snapshots: &mut StoryExportEditSnapshots,
    edit: StoryExportConfigurationEdit<'_>,
) -> bool {
    let StoryExportConfigurationEdit {
        current_target,
        requested_target,
        can_export,
        configuration_id,
        change,
        phase,
    } = edit;
    let Some(target_keys) =
        story_export_target_keys(nodes, current_target, requested_target, can_export)
    else {
        return false;
    };
    let edit_target = StoryExportEditTarget::new(requested_target, configuration_id.clone());
    if phase == DesignPanelEditPhase::Cancel {
        let Some(originals) = snapshots.remove(&edit_target) else {
            return false;
        };
        for (key, original) in originals {
            if let Some(original) = original {
                configurations.insert(key, original);
            } else {
                configurations.remove(&key);
            }
        }
        return true;
    }

    let Some(current) = story_uniform_export_configurations(configurations, &target_keys) else {
        return false;
    };
    if !current
        .iter()
        .any(|configuration| configuration.id == *configuration_id)
    {
        return false;
    }
    if phase == DesignPanelEditPhase::Begin {
        snapshots.entry(edit_target).or_insert_with(|| {
            target_keys
                .iter()
                .map(|key| (key.clone(), configurations.get(key).cloned()))
                .collect()
        });
        return true;
    }

    let target_capabilities = match requested_target {
        DesignPanelTarget::Page { .. } => DesignStaticExportCapabilities::default(),
        DesignPanelTarget::Nodes { node_ids } => story_aggregate_static_export_capabilities(
            node_ids
                .iter()
                .filter_map(|node_id| nodes.iter().find(|node| node.id == *node_id)),
        ),
    };
    for key in &target_keys {
        let Some(configuration) = configurations.get_mut(key).and_then(|configurations| {
            configurations
                .iter_mut()
                .find(|configuration| configuration.id == *configuration_id)
        }) else {
            // The complete target was validated above, so this can only be an
            // unexpected host-state race. Do not continue a partial write.
            return false;
        };
        configuration.apply_change(change.clone());
        if matches!(change, DesignExportConfigurationChange::Format(_)) {
            configuration.apply_target_defaults(target_capabilities);
        }
    }
    if phase == DesignPanelEditPhase::Commit {
        snapshots.remove(&edit_target);
    }
    true
}

fn story_menu_property_value(
    node: &DesignPanelNode,
    property: DesignPanelProperty,
) -> Option<DesignPanelValue> {
    match property {
        DesignPanelProperty::BlendMode => Some(DesignPanelValue::BlendMode(node.blend_mode)),
        DesignPanelProperty::HorizontalSizing => Some(DesignPanelValue::SizingMode(
            node.layout.as_ref()?.horizontal_sizing,
        )),
        DesignPanelProperty::VerticalSizing => Some(DesignPanelValue::SizingMode(
            node.layout.as_ref()?.vertical_sizing,
        )),
        DesignPanelProperty::StrokeAlign => {
            Some(DesignPanelValue::StrokeAlign(node.stroke.as_ref()?.align))
        }
        DesignPanelProperty::EffectKind(index) => Some(DesignPanelValue::EffectKind(
            node.effects.get(index)?.settings.kind(),
        )),
        DesignPanelProperty::EffectShadowBlendMode(index) => {
            let blend_mode = match &node.effects.get(index)?.settings {
                DesignEffectSettings::DropShadow(settings) => settings.blend_mode,
                DesignEffectSettings::InnerShadow(settings) => settings.blend_mode,
                _ => return None,
            };
            Some(DesignPanelValue::BlendMode(blend_mode))
        }
        DesignPanelProperty::EffectNoiseBlendMode(index) => {
            let DesignEffectSettings::Noise(settings) = &node.effects.get(index)?.settings else {
                return None;
            };
            Some(DesignPanelValue::BlendMode(settings.blend_mode))
        }
        DesignPanelProperty::EffectBlurType(index) => {
            let settings = match &node.effects.get(index)?.settings {
                DesignEffectSettings::LayerBlur(settings)
                | DesignEffectSettings::BackgroundBlur(settings) => settings,
                _ => return None,
            };
            Some(DesignPanelValue::EffectBlurType(settings.blur_type()))
        }
        DesignPanelProperty::EffectNoiseType(index) => {
            let DesignEffectSettings::Noise(settings) = &node.effects.get(index)?.settings else {
                return None;
            };
            Some(DesignPanelValue::EffectNoiseType(
                settings.colors.noise_type(),
            ))
        }
        _ => None,
    }
}

fn story_menu_paint_value(
    node: &DesignPanelNode,
    collection: DesignPanelCollection,
    paint_id: &SharedString,
    index: usize,
    property: DesignPaintProperty,
) -> Option<DesignPaintValue> {
    let paints = match collection {
        DesignPanelCollection::Fill => node.fills.as_slice(),
        DesignPanelCollection::Stroke => node.stroke.as_ref()?.paints.as_slice(),
        DesignPanelCollection::Effect
        | DesignPanelCollection::LayoutGrid
        | DesignPanelCollection::Export => return None,
    };
    let paint = paints.get(index)?;
    if paint.read_only || (!paint_id.is_empty() && paint.id != *paint_id) {
        return None;
    }
    match property {
        DesignPaintProperty::BlendMode => Some(DesignPaintValue::BlendMode(paint.blend_mode)),
        _ => None,
    }
}

struct StoryMenuPreviewContext<'a> {
    nodes: &'a [DesignPanelNode],
    bindings: &'a HashMap<
        (SharedString, DesignPanelProperty),
        DesignPanelPropertyBinding<DesignPanelValue>,
    >,
    current_target: Option<&'a DesignPanelTarget>,
    current_paint_target: Option<DesignPaintTarget>,
    can_edit: bool,
}

fn apply_story_menu_preview(
    active: &mut Option<DesignMenuPreview>,
    context: StoryMenuPreviewContext<'_>,
    preview: &DesignMenuPreview,
    phase: DesignMenuPreviewPhase,
) -> bool {
    let StoryMenuPreviewContext {
        nodes,
        bindings,
        current_target,
        current_paint_target,
        can_edit,
    } = context;
    if phase == DesignMenuPreviewPhase::End {
        if active.as_ref() != Some(preview) {
            return false;
        }
        *active = None;
        return true;
    }
    if active.is_some() || !can_edit {
        return false;
    }
    let valid = match preview {
        DesignMenuPreview::NodeProperty {
            target,
            property,
            original,
            candidate,
        } => {
            let DesignPanelTarget::Nodes { node_ids } = target else {
                return false;
            };
            current_target == Some(target)
                && original != candidate
                && !node_ids.is_empty()
                && node_ids.iter().all(|node_id| {
                    !bindings.contains_key(&(node_id.clone(), *property))
                        && nodes
                            .iter()
                            .find(|node| node.id == *node_id)
                            .and_then(|node| story_menu_property_value(node, *property))
                            .as_ref()
                            == Some(original)
                })
        }
        DesignMenuPreview::EffectProperty {
            node_id,
            effect_id,
            index,
            property,
            original,
            candidate,
        } => {
            let exact_target = DesignPanelTarget::Nodes {
                node_ids: vec![node_id.clone()],
            };
            let node = nodes.iter().find(|node| node.id == *node_id);
            let effect = node.and_then(|node| node.effects.get(*index));
            current_target == Some(&exact_target)
                && original != candidate
                && !bindings.contains_key(&(node_id.clone(), *property))
                && effect.is_some_and(|effect| effect_id.is_empty() || effect.id == *effect_id)
                && node
                    .and_then(|node| story_menu_property_value(node, *property))
                    .as_ref()
                    == Some(original)
        }
        DesignMenuPreview::PaintProperty {
            node_id,
            collection,
            target,
            paint_id,
            index,
            property,
            original,
            candidate,
        } => {
            let exact_target = DesignPanelTarget::Nodes {
                node_ids: vec![node_id.clone()],
            };
            let node = nodes.iter().find(|node| node.id == *node_id);
            current_target == Some(&exact_target)
                && current_paint_target == Some(*target)
                && original != candidate
                && node.is_some_and(|node| {
                    let style_is_bound = match collection {
                        DesignPanelCollection::Fill => node.fill_style_binding.is_some(),
                        DesignPanelCollection::Stroke => node.stroke_style_binding.is_some(),
                        DesignPanelCollection::Effect
                        | DesignPanelCollection::LayoutGrid
                        | DesignPanelCollection::Export => true,
                    };
                    !style_is_bound
                        && story_menu_paint_value(
                            node,
                            *collection,
                            paint_id,
                            *index,
                            property.clone(),
                        )
                        .as_ref()
                            == Some(original)
                })
        }
    };
    if valid {
        *active = Some(preview.clone());
    }
    valid
}

#[derive(Clone, Copy)]
struct StorySmartSelectionSpacingEdit<'a> {
    target: &'a DesignPanelTarget,
    axis: DesignSmartSelectionAxis,
    value: f32,
    phase: DesignPanelEditPhase,
}

fn apply_story_smart_selection_spacing_edit(
    spacing_values: &mut HashMap<DesignSmartSelectionAxis, DesignSmartSelectionSpacingValue>,
    snapshots: &mut HashMap<
        DesignSmartSelectionAxis,
        (DesignPanelTarget, DesignSmartSelectionSpacingValue),
    >,
    projection: Option<&DesignSmartSelectionViewData>,
    can_edit: bool,
    edit: StorySmartSelectionSpacingEdit<'_>,
) -> bool {
    let StorySmartSelectionSpacingEdit {
        target: action_target,
        axis,
        value,
        phase,
    } = edit;
    let projection_is_current = projection.is_some_and(|view_data| {
        view_data.target == *action_target
            && view_data.spacing_is_editable(axis)
            && view_data.is_valid()
    });
    let active_snapshot = snapshots.get(&axis).cloned();
    let lifecycle_is_valid = match phase {
        DesignPanelEditPhase::Begin => active_snapshot.is_none(),
        DesignPanelEditPhase::Preview => active_snapshot
            .as_ref()
            .is_some_and(|(active_target, _)| active_target == action_target),
        DesignPanelEditPhase::Commit => active_snapshot
            .as_ref()
            .is_none_or(|(active_target, _)| active_target == action_target),
        DesignPanelEditPhase::Cancel => active_snapshot
            .as_ref()
            .is_some_and(|(active_target, _)| active_target == action_target),
    };
    if !value.is_finite()
        || !lifecycle_is_valid
        || (phase != DesignPanelEditPhase::Cancel && (!can_edit || !projection_is_current))
    {
        return false;
    }

    match phase {
        DesignPanelEditPhase::Begin => {
            let original = spacing_values
                .get(&axis)
                .copied()
                .unwrap_or(DesignSmartSelectionSpacingValue::Mixed);
            snapshots.insert(axis, (action_target.clone(), original));
        }
        DesignPanelEditPhase::Preview => {
            spacing_values.insert(axis, DesignSmartSelectionSpacingValue::Uniform(value));
        }
        DesignPanelEditPhase::Commit => {
            spacing_values.insert(axis, DesignSmartSelectionSpacingValue::Uniform(value));
            snapshots.remove(&axis);
        }
        DesignPanelEditPhase::Cancel => {
            if let Some((_, original)) = snapshots.remove(&axis) {
                spacing_values.insert(axis, original);
            }
        }
    }
    true
}

fn story_smart_selection_operation_is_current(
    projection: Option<&DesignSmartSelectionViewData>,
    can_edit: bool,
    action_target: &DesignPanelTarget,
    operation: DesignSmartSelectionOperation,
) -> bool {
    can_edit
        && projection.is_some_and(|view_data| {
            view_data.target == *action_target
                && view_data.operation_is_available(operation)
                && view_data.is_valid()
        })
}

fn apply_story_add_auto_layout(
    nodes: &mut Vec<DesignPanelNode>,
    action_target: &DesignPanelTarget,
    current_target: Option<&DesignPanelTarget>,
    can_edit: bool,
    next_wrapper_id: &mut usize,
) -> Option<(usize, StoryAddAutoLayoutEcho)> {
    if !can_edit || current_target != Some(action_target) {
        return None;
    }
    let DesignPanelTarget::Nodes { node_ids } = action_target else {
        return None;
    };
    let unique_ids = node_ids.iter().collect::<HashSet<_>>();
    if node_ids.is_empty()
        || unique_ids.len() != node_ids.len()
        || node_ids.iter().any(|node_id| node_id.is_empty())
        || node_ids
            .iter()
            .any(|node_id| !nodes.iter().any(|node| node.id == *node_id))
    {
        return None;
    }

    if node_ids.len() == 1 {
        let node_index = nodes.iter().position(|node| node.id == node_ids[0])?;
        let node = &mut nodes[node_index];
        let is_active_owner = node.supports_auto_layout_container()
            && node
                .layout
                .as_ref()
                .is_some_and(|layout| layout.mode != DesignLayoutMode::None);
        if is_active_owner {
            return None;
        }
        if node.kind == DesignPanelNodeKind::Group || node.supports_auto_layout_container() {
            if node.kind == DesignPanelNodeKind::Group {
                node.kind = DesignPanelNodeKind::Frame;
                node.capabilities = None;
            }
            let layout = node.layout.get_or_insert_with(DesignLayout::default);
            layout.mode = DesignLayoutMode::Vertical;
            layout.horizontal_sizing = DesignSizingMode::Hug;
            layout.vertical_sizing = DesignSizingMode::Hug;
            node.name = format!("{} · Auto layout", node.name).into();
            return Some((
                node_index,
                StoryAddAutoLayoutEcho::Converted {
                    node_id: node.id.clone(),
                },
            ));
        }
    }

    let selected = node_ids
        .iter()
        .filter_map(|node_id| nodes.iter().find(|node| node.id == *node_id))
        .collect::<Vec<_>>();
    let min_x = selected
        .iter()
        .map(|node| node.x)
        .fold(f32::INFINITY, f32::min);
    let min_y = selected
        .iter()
        .map(|node| node.y)
        .fold(f32::INFINITY, f32::min);
    let max_x = selected
        .iter()
        .map(|node| node.x + node.width)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = selected
        .iter()
        .map(|node| node.y + node.height)
        .fold(f32::NEG_INFINITY, f32::max);
    let flow = if max_x - min_x >= max_y - min_y {
        DesignLayoutMode::Horizontal
    } else {
        DesignLayoutMode::Vertical
    };
    let wrapper_id = loop {
        let candidate = SharedString::from(format!("storybook-auto-layout-{}", *next_wrapper_id));
        *next_wrapper_id += 1;
        if nodes.iter().all(|node| node.id != candidate) {
            break candidate;
        }
    };
    let mut wrapper = DesignPanelNode::new(
        wrapper_id.clone(),
        format!("Auto layout wrapper · {} layers", node_ids.len()),
        DesignPanelNodeKind::Frame,
    )
    .with_layout_mode(flow);
    wrapper.x = min_x;
    wrapper.y = min_y;
    wrapper.width = (max_x - min_x).max(1.);
    wrapper.height = (max_y - min_y).max(1.);
    nodes.push(wrapper);
    Some((
        nodes.len() - 1,
        StoryAddAutoLayoutEcho::Wrapped {
            wrapper_id,
            child_ids: node_ids.clone(),
        },
    ))
}

fn story_homogeneous_multiple_nodes() -> [DesignPanelNode; 2] {
    let make_fills = |prefix: &str| {
        let mut accent =
            DesignPaint::solid(DesignColor::PURPLE).with_id(format!("{prefix}-fill-accent"));
        accent.opacity = 35.;
        vec![
            DesignPaint::solid(DesignColor::BLUE).with_id(format!("{prefix}-fill-surface")),
            accent,
        ]
    };
    let make_stroke = |prefix: &str| {
        let mut paint =
            DesignPaint::solid(DesignColor::BLACK).with_id(format!("{prefix}-stroke-outline"));
        paint.opacity = 80.;
        DesignStroke::for_node(
            DesignPanelNodeKind::Rectangle,
            paint,
            3.,
            DesignStrokeAlign::Inside,
        )
    };

    let mut first = DesignPanelNode::new(
        STORY_HOMOGENEOUS_MULTIPLE_NODE_IDS[0],
        "Common paints · Card A",
        DesignPanelNodeKind::Rectangle,
    );
    first.x = 48.;
    first.y = 64.;
    first.width = 240.;
    first.corner_radii = [16.; 4];
    first.fills = make_fills("homogeneous-first");
    first.stroke = Some(make_stroke("homogeneous-first"));
    first.effects.push(
        DesignEffect::drop_shadow(DesignColor::rgba(0x00, 0x00, 0x00, 0x33), 16., 0., 0., 4.)
            .with_id("homogeneous-first-shadow"),
    );

    let mut second = DesignPanelNode::new(
        STORY_HOMOGENEOUS_MULTIPLE_NODE_IDS[1],
        "Common paints · Card B",
        DesignPanelNodeKind::Rectangle,
    );
    second.x = 336.;
    second.y = 112.;
    second.width = 320.;
    second.corner_radii = [28.; 4];
    second.fills = make_fills("homogeneous-second");
    second.stroke = Some(make_stroke("homogeneous-second"));
    second
        .effects
        .push(DesignEffect::new(DesignEffectKind::LayerBlur).with_id("homogeneous-second-blur"));

    [first, second]
}

fn story_multiple_inspection_context(
    first: &DesignPanelNode,
    second: &DesignPanelNode,
    permissions: DesignPanelPermissions,
) -> (
    DesignPanelInspectionContext,
    Vec<(
        DesignPanelProperty,
        DesignPanelPropertyValueState<DesignPanelValue>,
    )>,
) {
    let (aggregate, property_states) = aggregate_story_multiple_selection(first, second);
    let selection = DesignPanelMultipleSelection::new(aggregate, second.clone());
    (
        DesignPanelInspectionContext::multiple(
            selection,
            DesignPanelParentLayout::Mixed,
            permissions,
        ),
        property_states,
    )
}

fn story_widget_dimension_property_states(
    width: f32,
    height: f32,
) -> Vec<(
    DesignPanelProperty,
    DesignPanelPropertyValueState<DesignPanelValue>,
)> {
    [
        (DesignPanelProperty::Width, width),
        (DesignPanelProperty::Height, height),
    ]
    .into_iter()
    .map(|(property, value)| {
        (
            property,
            DesignPanelPropertyValueState::Uniform(DesignPanelValue::Number(value))
                .read_only_with_reason("Widget dimensions are read-only in the Plugin API"),
        )
    })
    .collect()
}

fn story_paint_semantic_projection(paint: &DesignPaint) -> DesignPaint {
    let mut projected = paint.clone();
    projected.id = SharedString::default();
    if let DesignPaintPayload::Gradient(gradient) = &mut projected.payload {
        for stop in &mut gradient.stops {
            stop.id = SharedString::default();
        }
    }
    projected.sync_legacy_projection();
    projected
}

fn story_paint_collections_are_semantically_equal(
    first: &[DesignPaint],
    second: &[DesignPaint],
) -> bool {
    first.len() == second.len()
        && first.iter().zip(second).all(|(first, second)| {
            story_paint_semantic_projection(first) == story_paint_semantic_projection(second)
        })
}

fn story_strokes_are_semantically_equal(
    first: Option<&DesignStroke>,
    second: Option<&DesignStroke>,
) -> bool {
    match (first, second) {
        (None, None) => true,
        (Some(first), Some(second)) => {
            let mut first = first.clone();
            let mut second = second.clone();
            for paint in &mut first.paints {
                *paint = story_paint_semantic_projection(paint);
            }
            for paint in &mut second.paints {
                *paint = story_paint_semantic_projection(paint);
            }
            first == second
        }
        (None, Some(_)) | (Some(_), None) => false,
    }
}

fn story_fill_collection_is_common(first: &DesignPanelNode, second: &DesignPanelNode) -> bool {
    first.fill_style_binding == second.fill_style_binding
        && story_paint_collections_are_semantically_equal(&first.fills, &second.fills)
}

fn story_stroke_collection_is_common(first: &DesignPanelNode, second: &DesignPanelNode) -> bool {
    first.stroke_style_binding == second.stroke_style_binding
        && story_strokes_are_semantically_equal(first.stroke.as_ref(), second.stroke.as_ref())
}

fn aggregate_story_multiple_selection(
    first: &DesignPanelNode,
    second: &DesignPanelNode,
) -> (
    DesignPanelNode,
    Vec<(
        DesignPanelProperty,
        DesignPanelPropertyValueState<DesignPanelValue>,
    )>,
) {
    let corner_capabilities = DesignCornerCapabilities {
        uniform_radius: first.corner_capabilities.uniform_radius
            && second.corner_capabilities.uniform_radius,
        independent_radii: first.corner_capabilities.independent_radii
            && second.corner_capabilities.independent_radii,
        smoothing: first.corner_capabilities.smoothing && second.corner_capabilities.smoothing,
    };
    let dimensions = first.supports_dimensions() && second.supports_dimensions();
    let visibility = first.supports_visibility() && second.supports_visibility();
    let position_coordinates =
        first.supports_position_coordinates() && second.supports_position_coordinates();
    let arrange = first.supports_arrange() && second.supports_arrange();
    let transforms = first.supports_transforms() && second.supports_transforms();
    let aspect_ratio_lock =
        first.supports_aspect_ratio_lock() && second.supports_aspect_ratio_lock();
    let auto_layout_child =
        first.supports_auto_layout_child() && second.supports_auto_layout_child();
    let add_auto_layout = first.supports_add_auto_layout() && second.supports_add_auto_layout();
    let fill = first.supports_fill() && second.supports_fill();
    let stroke = first.supports_stroke() && second.supports_stroke();
    let layer_appearance = first.supports_layer_appearance() && second.supports_layer_appearance();
    let effects = first.supports_effects() && second.supports_effects();
    let layout_guides = first.supports_layout_guides() && second.supports_layout_guides();
    let constraints = first.supports_constraints() && second.supports_constraints();
    let common_fills = fill && story_fill_collection_is_common(first, second);
    let common_stroke = stroke && story_stroke_collection_is_common(first, second);
    let selection_color_aggregate = aggregate_story_selection_colors_for_collections(
        [first, second],
        !common_fills,
        !common_stroke,
    );
    let mut sections = vec![DesignPanelSection::Position];
    if dimensions {
        sections.push(DesignPanelSection::Layout);
    }
    if visibility || layer_appearance || corner_capabilities.has_any() {
        sections.push(DesignPanelSection::Layer);
    }
    if !selection_color_aggregate.colors.is_empty() {
        sections.push(DesignPanelSection::Selection);
    }
    if fill {
        sections.push(DesignPanelSection::Fill);
    }
    if stroke {
        sections.push(DesignPanelSection::Stroke);
    }
    if effects {
        sections.push(DesignPanelSection::Effects);
    }
    if layout_guides {
        sections.push(DesignPanelSection::LayoutGrid);
    }
    sections.push(DesignPanelSection::Export);

    let mut effect_capabilities = DesignEffectCapabilities {
        kind_availability: Vec::new(),
        shadow_spread: first.effect_capabilities.shadow_spread
            && second.effect_capabilities.shadow_spread,
        show_shadow_behind_transparent_areas: first
            .effect_capabilities
            .show_shadow_behind_transparent_areas
            && second
                .effect_capabilities
                .show_shadow_behind_transparent_areas,
    };
    for kind in DesignEffectKind::ALL {
        let available = first.effect_style_binding.is_none()
            && second.effect_style_binding.is_none()
            && first.can_use_effect_kind(kind, None)
            && second.can_use_effect_kind(kind, None);
        effect_capabilities.set_kind_availability(if available {
            DesignEffectKindAvailability::available(kind)
        } else {
            DesignEffectKindAvailability::unavailable(
                kind,
                "Unavailable or at its limit for part of the selection",
            )
        });
    }

    let mut aggregate = first.clone();
    aggregate.name = "2 layers selected".into();
    aggregate.is_component_instance_child =
        first.is_component_instance_child || second.is_component_instance_child;
    aggregate.capabilities = Some(DesignPanelNodeCapabilities {
        sections,
        dimensions,
        visibility,
        position_coordinates,
        arrange,
        transforms,
        aspect_ratio_lock,
        auto_layout_child,
        add_auto_layout,
        auto_layout_container: false,
        grid_auto_layout: false,
        resize_to_fit: first.supports_resize_to_fit() && second.supports_resize_to_fit(),
        clip_content: false,
        fill,
        stroke,
        layer_appearance,
        pass_through_blend: first.supports_pass_through_blend()
            && second.supports_pass_through_blend(),
        effects,
        constraints,
        layout_guides,
    });
    aggregate.layout = None;
    aggregate.corner_capabilities = corner_capabilities;
    if common_fills {
        if first.fill_shows_in_exports != second.fill_shows_in_exports {
            aggregate.fill_shows_in_exports = None;
        }
    } else {
        aggregate.fill_style_binding = None;
        aggregate.fills.clear();
        aggregate.fill_shows_in_exports = None;
    }
    if !common_stroke {
        aggregate.stroke = None;
        aggregate.stroke_style_binding = None;
    }
    aggregate.effects.clear();
    aggregate.effect_capabilities = effect_capabilities;
    aggregate.effect_style_binding = None;
    aggregate.layout_grids.clear();
    aggregate.layout_grid_style_binding = None;
    aggregate.export_settings.clear();
    aggregate.typography = None;
    aggregate.text_path = None;
    aggregate.text_path_start_data = None;
    aggregate.vector_edit = None;
    aggregate.component_context = None;
    aggregate.component_properties.clear();
    aggregate.media = None;
    aggregate.shape_geometry = DesignShapeGeometry::None;
    aggregate.section = None;
    aggregate.transform_modifiers.clear();
    aggregate.is_mask = false;
    aggregate.mask_type = None;
    aggregate.selection_color_aggregate = selection_color_aggregate;
    aggregate.selection_colors.clear();

    let state = |same, value| {
        if same {
            DesignPanelPropertyValueState::Uniform(value)
        } else {
            DesignPanelPropertyValueState::Mixed
        }
    };
    let property_states = vec![
        (
            DesignPanelProperty::Visible,
            state(
                first.visible == second.visible,
                DesignPanelValue::Bool(first.visible),
            ),
        ),
        (
            DesignPanelProperty::X,
            state(first.x == second.x, DesignPanelValue::Number(first.x)),
        ),
        (
            DesignPanelProperty::Y,
            state(first.y == second.y, DesignPanelValue::Number(first.y)),
        ),
        (
            DesignPanelProperty::Width,
            state(
                first.width == second.width,
                DesignPanelValue::Number(first.width),
            ),
        ),
        (
            DesignPanelProperty::Height,
            state(
                first.height == second.height,
                DesignPanelValue::Number(first.height),
            ),
        ),
        (
            DesignPanelProperty::Rotation,
            state(
                first.rotation == second.rotation,
                DesignPanelValue::Number(first.rotation),
            ),
        ),
        (
            DesignPanelProperty::LockAspectRatio,
            state(
                first.lock_aspect_ratio == second.lock_aspect_ratio,
                DesignPanelValue::Bool(first.lock_aspect_ratio),
            ),
        ),
        (
            DesignPanelProperty::HorizontalConstraint,
            state(
                first.horizontal_constraint == second.horizontal_constraint,
                DesignPanelValue::Constraint(first.horizontal_constraint),
            ),
        ),
        (
            DesignPanelProperty::VerticalConstraint,
            state(
                first.vertical_constraint == second.vertical_constraint,
                DesignPanelValue::Constraint(first.vertical_constraint),
            ),
        ),
        (
            DesignPanelProperty::Opacity,
            state(
                first.opacity == second.opacity,
                DesignPanelValue::Number(first.opacity),
            ),
        ),
        (
            DesignPanelProperty::BlendMode,
            state(
                first.blend_mode == second.blend_mode,
                DesignPanelValue::BlendMode(first.blend_mode),
            ),
        ),
        (
            DesignPanelProperty::CornerRadius,
            state(
                first.corner_radii == second.corner_radii,
                DesignPanelValue::Number(first.corner_radii[0]),
            ),
        ),
        (
            DesignPanelProperty::CornerRadiusTopLeft,
            state(
                first.corner_radii[0] == second.corner_radii[0],
                DesignPanelValue::Number(first.corner_radii[0]),
            ),
        ),
        (
            DesignPanelProperty::CornerRadiusTopRight,
            state(
                first.corner_radii[1] == second.corner_radii[1],
                DesignPanelValue::Number(first.corner_radii[1]),
            ),
        ),
        (
            DesignPanelProperty::CornerRadiusBottomRight,
            state(
                first.corner_radii[2] == second.corner_radii[2],
                DesignPanelValue::Number(first.corner_radii[2]),
            ),
        ),
        (
            DesignPanelProperty::CornerRadiusBottomLeft,
            state(
                first.corner_radii[3] == second.corner_radii[3],
                DesignPanelValue::Number(first.corner_radii[3]),
            ),
        ),
        (
            DesignPanelProperty::IndependentCorners,
            state(
                first.independent_corners == second.independent_corners,
                DesignPanelValue::Bool(first.independent_corners),
            ),
        ),
        (
            DesignPanelProperty::CornerSmoothing,
            state(
                first.corner_smoothing == second.corner_smoothing,
                DesignPanelValue::Ratio(first.corner_smoothing),
            ),
        ),
    ];

    (aggregate, property_states)
}

fn aggregate_story_selection_colors<'a>(
    nodes: impl IntoIterator<Item = &'a DesignPanelNode>,
) -> DesignSelectionColors {
    aggregate_story_selection_colors_for_collections(nodes, true, true)
}

fn aggregate_story_selection_colors_for_collections<'a>(
    nodes: impl IntoIterator<Item = &'a DesignPanelNode>,
    include_fills: bool,
    include_strokes: bool,
) -> DesignSelectionColors {
    let mut colors = Vec::<DesignSelectionColor>::new();
    for node in nodes {
        if include_fills {
            push_story_selection_paint_colors(
                &mut colors,
                node,
                DesignSelectionPaintCollection::Fill,
                &node.fills,
            );
        }
        if include_strokes && let Some(stroke) = node.stroke.as_ref() {
            push_story_selection_paint_colors(
                &mut colors,
                node,
                DesignSelectionPaintCollection::Stroke,
                &stroke.paints,
            );
        }
    }
    DesignSelectionColors::new(colors)
}

fn push_story_selection_paint_colors(
    colors: &mut Vec<DesignSelectionColor>,
    node: &DesignPanelNode,
    collection: DesignSelectionPaintCollection,
    paints: &[DesignPaint],
) {
    let style_binding = match collection {
        DesignSelectionPaintCollection::Fill => node.fill_style_binding.clone(),
        DesignSelectionPaintCollection::Stroke => node.stroke_style_binding.clone(),
    };
    for (paint_index, paint) in paints.iter().enumerate() {
        let paint_reference = || {
            DesignSelectionPaintReference::paint(
                node.id.clone(),
                collection,
                paint.id.clone(),
                paint_index,
            )
        };
        match &paint.payload {
            DesignPaintPayload::Solid(solid) => {
                push_story_selection_color(
                    colors,
                    solid.color,
                    paint_reference(),
                    paint,
                    paints,
                    style_binding.clone(),
                    solid.binding.clone(),
                    paint.read_only,
                );
            }
            DesignPaintPayload::Gradient(_) => push_story_selection_color(
                colors,
                paint.color,
                paint_reference(),
                paint,
                paints,
                style_binding.clone(),
                None,
                paint.read_only,
            ),
            DesignPaintPayload::Pattern(_)
            | DesignPaintPayload::Image(_)
            | DesignPaintPayload::Video(_)
            | DesignPaintPayload::Shader(_)
            | DesignPaintPayload::Unsupported(_) => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn push_story_selection_color(
    colors: &mut Vec<DesignSelectionColor>,
    color: DesignColor,
    reference: DesignSelectionPaintReference,
    paint: &DesignPaint,
    style_paints: &[DesignPaint],
    style_binding: Option<DesignPaintStyleBinding>,
    binding: Option<fanta_gpui::prelude::DesignPaintBinding>,
    read_only: bool,
) {
    if let Some(existing) = colors.iter_mut().find(|existing| {
        existing.color == color
            && story_selection_paint_semantics_equal(&existing.paint, paint)
            && story_selection_paint_collections_equal(&existing.style_paints, style_paints)
            && existing.style_binding.as_ref() == style_binding.as_ref()
            && existing.binding.as_ref() == binding.as_ref()
            && existing.read_only == read_only
    }) {
        existing.paint_references.push(reference);
        existing.occurrence_count = existing.occurrence_count.saturating_add(1);
        return;
    }

    let mut selection_color =
        DesignSelectionColor::new(story_selection_color_id(&reference), color, [reference])
            .with_paint(paint.clone())
            .with_style_context(style_paints.iter().cloned(), style_binding)
            .read_only(read_only);
    if let Some(binding) = binding {
        selection_color = selection_color.with_binding(binding);
    }
    colors.push(selection_color);
}

fn story_selection_color_id(reference: &DesignSelectionPaintReference) -> SharedString {
    let paint_identity = if reference.paint_id.is_empty() {
        format!("index-{}", reference.paint_index)
    } else {
        reference.paint_id.to_string()
    };
    format!(
        "selection-color-{}-{}-{paint_identity}",
        reference.node_id,
        reference.collection.label().to_ascii_lowercase(),
    )
    .into()
}

fn story_selection_paint_collections_equal(left: &[DesignPaint], right: &[DesignPaint]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| story_selection_paint_semantics_equal(left, right))
}

fn story_selection_paint_semantics_equal(left: &DesignPaint, right: &DesignPaint) -> bool {
    normalize_story_selection_paint(left) == normalize_story_selection_paint(right)
}

#[allow(clippy::too_many_arguments)]
fn apply_story_media_source_drop(
    paints: &mut [DesignPaint],
    paint_id: &SharedString,
    index: usize,
    expected_source_id: &SharedString,
    expected_media_kind: DesignMediaKind,
    file: &DesignMediaDroppedFile,
    capabilities: DesignMediaPaintCapabilities,
    next_media_source_id: &mut usize,
) -> bool {
    if DesignMediaFileKind::from_path(&file.path) != Some(file.kind)
        || !capabilities.allows_file_drop(file.kind)
    {
        return false;
    }
    let resolved_index = if paint_id.is_empty() {
        (index < paints.len()).then_some(index)
    } else {
        paints.iter().position(|paint| paint.id == *paint_id)
    };
    let Some(paint) = resolved_index.and_then(|index| paints.get_mut(index)) else {
        return false;
    };
    if paint.read_only {
        return false;
    }
    let (current_media_kind, current_source_id, placement, filters) = match &paint.payload {
        DesignPaintPayload::Image(image) => (
            DesignMediaKind::Image,
            image.source.id.clone(),
            image.placement,
            image.filters,
        ),
        DesignPaintPayload::Video(video) => (
            DesignMediaKind::Video,
            video.source.id.clone(),
            video.placement,
            video.filters,
        ),
        _ => return false,
    };
    if current_media_kind != expected_media_kind || current_source_id != *expected_source_id {
        return false;
    }

    let source_name = file
        .path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| file.kind.label().to_owned());
    let mut source = DesignPaintSource::new(
        allocate_story_media_source_id(next_media_source_id),
        source_name,
    );
    source.mime_type = Some(file.kind.mime_type().into());

    let mut replacement = match file.media_kind() {
        DesignMediaKind::Image => DesignPaint::image(source),
        DesignMediaKind::Video => DesignPaint::video(source),
    };
    match &mut replacement.payload {
        DesignPaintPayload::Image(image) => {
            image.placement = placement;
            image.filters = filters;
        }
        DesignPaintPayload::Video(video) => {
            video.placement = placement;
            video.filters = filters;
        }
        _ => unreachable!("the replacement constructors are media-only"),
    }
    replacement.id = paint.id.clone();
    replacement.opacity = paint.opacity;
    replacement.visible = paint.visible;
    replacement.blend_mode = paint.blend_mode;
    replacement.read_only = paint.read_only;
    *paint = replacement;
    true
}

fn allocate_story_media_source_id(next_media_source_id: &mut usize) -> SharedString {
    let source_id = format!("storybook-media-source-{}", *next_media_source_id).into();
    *next_media_source_id = (*next_media_source_id)
        .checked_add(1)
        .expect("the Storybook media source ID space is exhausted");
    source_id
}

fn normalize_story_selection_paint(paint: &DesignPaint) -> DesignPaint {
    let mut paint = paint.clone();
    paint.id = "".into();
    for stop in &mut paint.gradient_stops {
        stop.id = "".into();
    }
    if let DesignPaintPayload::Gradient(gradient) = &mut paint.payload {
        for stop in &mut gradient.stops {
            stop.id = "".into();
        }
    }
    paint
}

fn reorder_story_grid_tracks(
    tracks: &mut Vec<DesignGridTrack>,
    from_indices: &[usize],
    insertion_index: usize,
) -> bool {
    if from_indices.is_empty() || insertion_index > tracks.len() {
        return false;
    }
    let mut selected = vec![false; tracks.len()];
    for index in from_indices {
        let Some(slot) = selected.get_mut(*index) else {
            return false;
        };
        if *slot {
            return false;
        }
        *slot = true;
    }
    let moved = tracks
        .iter()
        .copied()
        .zip(&selected)
        .filter_map(|(track, selected)| (*selected).then_some(track))
        .collect::<Vec<_>>();
    let removed_before_insertion = selected
        .iter()
        .take(insertion_index)
        .filter(|selected| **selected)
        .count();
    let adjusted_insertion = insertion_index.saturating_sub(removed_before_insertion);
    let mut remaining = tracks
        .iter()
        .copied()
        .zip(selected)
        .filter_map(|(track, selected)| (!selected).then_some(track))
        .collect::<Vec<_>>();
    if adjusted_insertion > remaining.len() {
        return false;
    }
    remaining.splice(adjusted_insertion..adjusted_insertion, moved);
    *tracks = remaining;
    true
}

fn apply_story_grid_dimensions_edit_phase(
    node: &mut DesignPanelNode,
    snapshots: &mut HashMap<SharedString, DesignLayout>,
    node_id: &SharedString,
    dimensions: DesignGridDimensions,
    phase: DesignPanelEditPhase,
) -> bool {
    if node.id != *node_id || !dimensions.is_valid() {
        return false;
    }
    if phase == DesignPanelEditPhase::Cancel {
        let Some(snapshot) = snapshots.remove(node_id) else {
            return false;
        };
        node.layout = Some(snapshot);
        return true;
    }
    let Some(layout) = node
        .layout
        .as_mut()
        .filter(|layout| layout.mode == DesignLayoutMode::Grid)
    else {
        return false;
    };
    if layout.grid_auto_tracks == DesignGridAutoTracks::Rows
        && dimensions.rows != layout.grid_rows.len()
    {
        return false;
    }
    match phase {
        DesignPanelEditPhase::Begin => {
            if snapshots.contains_key(node_id) {
                return false;
            }
            snapshots.insert(node_id.clone(), layout.clone());
        }
        DesignPanelEditPhase::Preview => {
            if !snapshots.contains_key(node_id) {
                return false;
            }
        }
        DesignPanelEditPhase::Commit => {}
        DesignPanelEditPhase::Cancel => unreachable!("Cancel returned before Grid validation"),
    }

    layout
        .grid_columns
        .resize(dimensions.columns, DesignGridTrack::hug());
    if layout.grid_auto_tracks == DesignGridAutoTracks::None {
        layout
            .grid_rows
            .resize(dimensions.rows, DesignGridTrack::hug());
    }
    if phase == DesignPanelEditPhase::Commit {
        snapshots.remove(node_id);
    }
    true
}

fn apply_design_property(
    node: &mut DesignPanelNode,
    property: DesignPanelProperty,
    value: &DesignPanelValue,
) {
    apply_design_property_with_parent(node, property, value, false);
}

fn apply_design_property_with_parent(
    node: &mut DesignPanelNode,
    property: DesignPanelProperty,
    value: &DesignPanelValue,
    inside_auto_layout: bool,
) {
    match (property, value) {
        (DesignPanelProperty::X, DesignPanelValue::Number(value)) => node.x = *value,
        (DesignPanelProperty::Y, DesignPanelValue::Number(value)) => node.y = *value,
        (DesignPanelProperty::Width, DesignPanelValue::Number(value)) => {
            if node.lock_aspect_ratio && node.width.abs() > f32::EPSILON {
                node.height *= *value / node.width;
            }
            node.width = *value;
        }
        (DesignPanelProperty::Height, DesignPanelValue::Number(value)) => {
            if node.lock_aspect_ratio && node.height.abs() > f32::EPSILON {
                node.width *= *value / node.height;
            }
            node.height = *value;
        }
        (DesignPanelProperty::Rotation, DesignPanelValue::Number(value)) => node.rotation = *value,
        (DesignPanelProperty::LockAspectRatio, DesignPanelValue::Bool(value)) => {
            if !node.is_component_instance_child {
                node.lock_aspect_ratio = *value;
            }
        }
        (DesignPanelProperty::HorizontalConstraint, DesignPanelValue::Constraint(constraint)) => {
            node.horizontal_constraint = *constraint
        }
        (DesignPanelProperty::VerticalConstraint, DesignPanelValue::Constraint(constraint)) => {
            node.vertical_constraint = *constraint
        }
        (DesignPanelProperty::LayoutMode, DesignPanelValue::LayoutMode(mode)) => {
            let supports_auto_layout = node.supports_auto_layout_container();
            let supports_grid = node.supports_grid_auto_layout();
            if let Some(layout) = node.layout.as_mut() {
                layout.set_mode_for_capabilities(supports_auto_layout, supports_grid, *mode);
            }
        }
        (DesignPanelProperty::HorizontalSizing, DesignPanelValue::SizingMode(mode)) => {
            if let Some(layout) = node.layout.as_mut()
                && !(layout.mode == DesignLayoutMode::Grid
                    && *mode == DesignSizingMode::Hug
                    && layout.grid_columns.iter().any(|track| {
                        track.sizing == fanta_gpui::prelude::DesignGridTrackSizing::Fraction
                    }))
            {
                layout.horizontal_sizing = *mode;
            }
        }
        (DesignPanelProperty::VerticalSizing, DesignPanelValue::SizingMode(mode)) => {
            if let Some(layout) = node.layout.as_mut()
                && !(layout.mode == DesignLayoutMode::Grid
                    && *mode == DesignSizingMode::Hug
                    && layout.grid_rows.iter().any(|track| {
                        track.sizing == fanta_gpui::prelude::DesignGridTrackSizing::Fraction
                    }))
            {
                layout.vertical_sizing = *mode;
            }
            node.normalize_text_max_lines(inside_auto_layout);
        }
        (
            DesignPanelProperty::AutoLayoutAlignment,
            DesignPanelValue::AutoLayoutAlignment(alignment),
        ) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.alignment_x = alignment.x;
                layout.alignment_y = alignment.y;
            }
        }
        (DesignPanelProperty::Wrap, DesignPanelValue::Bool(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.wrap = *value && layout.mode == DesignLayoutMode::Horizontal;
            }
        }
        (DesignPanelProperty::Gap, DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.gap = *value;
            }
        }
        (DesignPanelProperty::ItemSpacingMode, DesignPanelValue::ItemSpacingMode(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item_spacing_mode = *value;
            }
        }
        (
            DesignPanelProperty::CounterAxisAlignContent,
            DesignPanelValue::CounterAxisAlignContent(value),
        ) => {
            if let Some(layout) = node.layout.as_mut() {
                let _ = layout.set_counter_axis_align_content(*value);
            }
        }
        (DesignPanelProperty::CounterAxisGap, DesignPanelValue::OptionalNumber(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                if layout.mode == DesignLayoutMode::Grid {
                    if value.is_some_and(|gap| gap.is_finite() && gap >= 0.) {
                        layout.counter_axis_gap = *value;
                    }
                } else {
                    let _ = layout.set_counter_axis_gap(*value);
                }
            }
        }
        (DesignPanelProperty::PaddingVertical, DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && value.is_finite()
                && *value >= 0.
            {
                layout.padding[0] = *value;
                layout.padding[2] = *value;
            }
        }
        (DesignPanelProperty::PaddingHorizontal, DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && value.is_finite()
                && *value >= 0.
            {
                layout.padding[1] = *value;
                layout.padding[3] = *value;
            }
        }
        (DesignPanelProperty::PaddingShorthand, DesignPanelValue::NumberList(values))
            if values.iter().all(|value| value.is_finite() && *value >= 0.) =>
        {
            let expanded = match values.as_slice() {
                [all] => Some([*all; 4]),
                [vertical, horizontal] => Some([*vertical, *horizontal, *vertical, *horizontal]),
                [top, horizontal, bottom] => Some([*top, *horizontal, *bottom, *horizontal]),
                [top, right, bottom, left] => Some([*top, *right, *bottom, *left]),
                _ => None,
            };
            if let (Some(layout), Some(expanded)) = (node.layout.as_mut(), expanded) {
                layout.padding = expanded;
            }
        }
        (DesignPanelProperty::PaddingTop, DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.padding[0] = *value;
            }
        }
        (DesignPanelProperty::PaddingRight, DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.padding[1] = *value;
            }
        }
        (DesignPanelProperty::PaddingBottom, DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.padding[2] = *value;
            }
        }
        (DesignPanelProperty::PaddingLeft, DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.padding[3] = *value;
            }
        }
        (DesignPanelProperty::ClipContent, DesignPanelValue::Bool(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.clip_content = *value;
            }
        }
        (DesignPanelProperty::IncludeStrokes, DesignPanelValue::Bool(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.include_strokes = *value;
            }
        }
        (DesignPanelProperty::StackingOrder, DesignPanelValue::StackingOrder(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.stacking_order = *value;
            }
        }
        (DesignPanelProperty::BaselineAlignment, DesignPanelValue::BaselineAlignment(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.baseline_alignment = *value;
            }
        }
        (DesignPanelProperty::MinWidth, DesignPanelValue::OptionalNumber(value)) => {
            let linked_minimum = (node.lock_aspect_ratio && node.width.abs() > f32::EPSILON)
                .then(|| value.map(|value| value * node.height / node.width));
            if let Some(layout) = node.layout.as_mut() {
                layout.item.min_width = *value;
                if let Some(linked_minimum) = linked_minimum {
                    layout.item.min_height = linked_minimum;
                }
            }
        }
        (DesignPanelProperty::MaxWidth, DesignPanelValue::OptionalNumber(value)) => {
            let linked_maximum = (node.lock_aspect_ratio && node.width.abs() > f32::EPSILON)
                .then(|| value.map(|value| value * node.height / node.width));
            if let Some(layout) = node.layout.as_mut() {
                layout.item.max_width = *value;
            }
            if let Some(linked_maximum) = linked_maximum {
                let _ = node.set_layout_max_height(linked_maximum);
            }
        }
        (DesignPanelProperty::MinHeight, DesignPanelValue::OptionalNumber(value)) => {
            let linked_minimum = (node.lock_aspect_ratio && node.height.abs() > f32::EPSILON)
                .then(|| value.map(|value| value * node.width / node.height));
            if let Some(layout) = node.layout.as_mut() {
                layout.item.min_height = *value;
                if let Some(linked_minimum) = linked_minimum {
                    layout.item.min_width = linked_minimum;
                }
            }
        }
        (DesignPanelProperty::MaxHeight, DesignPanelValue::OptionalNumber(value)) => {
            let linked_maximum = (node.lock_aspect_ratio && node.height.abs() > f32::EPSILON)
                .then(|| value.map(|value| value * node.width / node.height));
            let _ = node.set_layout_max_height(*value);
            if let Some(linked_maximum) = linked_maximum
                && let Some(layout) = node.layout.as_mut()
            {
                layout.item.max_width = linked_maximum;
            }
        }
        (DesignPanelProperty::LayoutPositioning, DesignPanelValue::LayoutPositioning(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.positioning = *value;
            }
        }
        (DesignPanelProperty::LayoutAlignSelf, DesignPanelValue::LayoutAlignSelf(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.align_self = *value;
            }
        }
        (DesignPanelProperty::LayoutGrow, DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.layout_grow = *value;
            }
        }
        (DesignPanelProperty::GridAutoTracks, DesignPanelValue::GridAutoTracks(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && layout.mode == DesignLayoutMode::Grid
            {
                layout.grid_auto_tracks = *value;
            }
        }
        (
            DesignPanelProperty::GridItemsPositioning,
            DesignPanelValue::GridItemsPositioning(value),
        ) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.grid_items_positioning = *value;
            }
        }
        (DesignPanelProperty::GridColumnCount, DesignPanelValue::Integer(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && layout.mode == DesignLayoutMode::Grid
                && let Ok(count) = usize::try_from(*value)
                && count >= 1
            {
                layout.grid_columns.resize(count, DesignGridTrack::hug());
            }
        }
        (DesignPanelProperty::GridRowCount, DesignPanelValue::Integer(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && layout.mode == DesignLayoutMode::Grid
                && layout.grid_auto_tracks == DesignGridAutoTracks::None
                && let Ok(count) = usize::try_from(*value)
                && count >= 1
            {
                layout.grid_rows.resize(count, DesignGridTrack::hug());
            }
        }
        (DesignPanelProperty::GridColumnTrack(index), DesignPanelValue::GridTrack(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && value.is_valid()
                && !(layout.horizontal_sizing == DesignSizingMode::Hug
                    && value.sizing == fanta_gpui::prelude::DesignGridTrackSizing::Fraction)
                && let Some(track) = layout.grid_columns.get_mut(index)
            {
                *track = *value;
            }
        }
        (DesignPanelProperty::GridRowTrack(index), DesignPanelValue::GridTrack(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && value.is_valid()
                && !(layout.vertical_sizing == DesignSizingMode::Hug
                    && value.sizing == fanta_gpui::prelude::DesignGridTrackSizing::Fraction)
                && let Some(track) = layout.grid_rows.get_mut(index)
            {
                *track = *value;
            }
        }
        (DesignPanelProperty::GridColumnTrackValue(index), DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && value.is_finite()
                && *value > 0.
                && let Some(track) = layout.grid_columns.get_mut(index)
                && track.sizing != fanta_gpui::prelude::DesignGridTrackSizing::Hug
            {
                track.value = *value;
            }
        }
        (DesignPanelProperty::GridRowTrackValue(index), DesignPanelValue::Number(value)) => {
            if let Some(layout) = node.layout.as_mut()
                && value.is_finite()
                && *value > 0.
                && let Some(track) = layout.grid_rows.get_mut(index)
                && track.sizing != fanta_gpui::prelude::DesignGridTrackSizing::Hug
            {
                track.value = *value;
            }
        }
        (DesignPanelProperty::GridRowIndex, DesignPanelValue::Integer(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.grid_row_index =
                    usize::try_from(value.saturating_sub(1).max(0)).unwrap_or(usize::MAX);
            }
        }
        (DesignPanelProperty::GridColumnIndex, DesignPanelValue::Integer(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.grid_column_index =
                    usize::try_from(value.saturating_sub(1).max(0)).unwrap_or(usize::MAX);
            }
        }
        (DesignPanelProperty::GridRowSpan, DesignPanelValue::Integer(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.grid_row_span = (*value).clamp(1, i64::from(u16::MAX)) as u16;
            }
        }
        (DesignPanelProperty::GridColumnSpan, DesignPanelValue::Integer(value)) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.grid_column_span = (*value).clamp(1, i64::from(u16::MAX)) as u16;
            }
        }
        (
            DesignPanelProperty::GridHorizontalAlignment,
            DesignPanelValue::GridItemAlignment(value),
        ) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.grid_horizontal_alignment = *value;
            }
        }
        (
            DesignPanelProperty::GridVerticalAlignment,
            DesignPanelValue::GridItemAlignment(value),
        ) => {
            if let Some(layout) = node.layout.as_mut() {
                layout.item.grid_vertical_alignment = *value;
            }
        }
        (DesignPanelProperty::Visible, DesignPanelValue::Bool(value)) => node.visible = *value,
        (DesignPanelProperty::FillShowsInExports, DesignPanelValue::Bool(value)) => {
            if let Some(show_in_exports) = node.fill_shows_in_exports.as_mut() {
                *show_in_exports = *value;
            }
        }
        (DesignPanelProperty::Opacity, DesignPanelValue::Number(value)) => node.opacity = *value,
        (DesignPanelProperty::BlendMode, DesignPanelValue::BlendMode(mode)) => {
            node.blend_mode = *mode;
        }
        (DesignPanelProperty::CornerRadius, DesignPanelValue::Number(value)) => {
            node.corner_radii = [*value; 4];
        }
        (DesignPanelProperty::IndependentCorners, DesignPanelValue::Bool(value)) => {
            node.independent_corners = *value;
        }
        (DesignPanelProperty::CornerRadiusTopLeft, DesignPanelValue::Number(value)) => {
            node.corner_radii[0] = *value
        }
        (DesignPanelProperty::CornerRadiusTopRight, DesignPanelValue::Number(value)) => {
            node.corner_radii[1] = *value
        }
        (DesignPanelProperty::CornerRadiusBottomRight, DesignPanelValue::Number(value)) => {
            node.corner_radii[2] = *value
        }
        (DesignPanelProperty::CornerRadiusBottomLeft, DesignPanelValue::Number(value)) => {
            node.corner_radii[3] = *value
        }
        (DesignPanelProperty::CornerSmoothing, DesignPanelValue::Ratio(value)) => {
            node.corner_smoothing = value.clamp(0., 1.);
        }
        (DesignPanelProperty::ComponentProperty(index), DesignPanelValue::Text(value)) => {
            if let Some(property) = node.component_properties.get_mut(index) {
                property.value = value.clone();
            }
        }
        (DesignPanelProperty::FontFamily, DesignPanelValue::Text(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.family = value.clone();
            }
        }
        (DesignPanelProperty::FontStyle, DesignPanelValue::Text(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.style = value.clone();
            }
        }
        (DesignPanelProperty::FontWeight, DesignPanelValue::Number(value)) => {
            if let Some(typography) = node.typography.as_mut()
                && value.is_finite()
            {
                typography.weight = value.clamp(1., 1000.);
            }
        }
        (DesignPanelProperty::FontSize, DesignPanelValue::Number(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.size = value.max(1.);
            }
        }
        (DesignPanelProperty::LineHeight, DesignPanelValue::LineHeight(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.line_height = *value;
            }
        }
        (DesignPanelProperty::LetterSpacing, DesignPanelValue::LetterSpacing(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.letter_spacing = *value;
            }
        }
        (DesignPanelProperty::TextLeadingTrim, DesignPanelValue::TextLeadingTrim(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.leading_trim = *value;
            }
        }
        (DesignPanelProperty::ParagraphSpacing, DesignPanelValue::Number(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.paragraph_spacing = *value;
            }
        }
        (DesignPanelProperty::ParagraphIndent, DesignPanelValue::Number(value)) => {
            if let Some(typography) = node.typography.as_mut()
                && typography.horizontal_alignment
                    == fanta_gpui::prelude::DesignTextHorizontalAlignment::Left
            {
                typography.paragraph_indent = *value;
            }
        }
        (DesignPanelProperty::ListSpacing, DesignPanelValue::Number(value)) => {
            if let Some(typography) = node.typography.as_mut()
                && typography.list != DesignTextList::None
                && value.is_finite()
                && *value >= 0.
            {
                typography.list_spacing = *value;
            }
        }
        (DesignPanelProperty::TextHangingPunctuation, DesignPanelValue::Bool(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.hanging_punctuation = *value;
            }
        }
        (DesignPanelProperty::TextHangingLists, DesignPanelValue::Bool(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.hanging_lists = *value;
            }
        }
        (
            DesignPanelProperty::HorizontalTextAlignment,
            DesignPanelValue::TextHorizontalAlignment(value),
        ) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.horizontal_alignment = *value;
            }
        }
        (
            DesignPanelProperty::VerticalTextAlignment,
            DesignPanelValue::TextVerticalAlignment(value),
        ) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.vertical_alignment = *value;
            }
        }
        (DesignPanelProperty::TextResize, DesignPanelValue::TextResize(value)) => {
            let _ = node.set_text_resize(*value);
            node.normalize_text_max_lines(inside_auto_layout);
        }
        (DesignPanelProperty::TextTruncate, DesignPanelValue::Bool(value)) => {
            let _ = node.set_text_truncation(*value);
            node.normalize_text_max_lines(inside_auto_layout);
        }
        (DesignPanelProperty::TextMaxLines, DesignPanelValue::OptionalNumber(value)) => {
            let max_lines = value
                .and_then(|value| (value.is_finite() && value >= 1.).then(|| value.round() as u32));
            if value.is_none() || max_lines.is_some() {
                let _ = node.set_text_max_lines(max_lines, inside_auto_layout);
            }
        }
        (DesignPanelProperty::TextDecoration, DesignPanelValue::TextDecoration(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.decoration = *value;
                if *value == DesignTextDecoration::None {
                    typography.decoration_details = None;
                } else if typography.decoration_details.is_none() {
                    typography.decoration_details = Some(DesignTextDecorationDetails::default());
                }
            }
        }
        (
            DesignPanelProperty::TextDecorationStyle,
            DesignPanelValue::TextDecorationStyle(value),
        ) => {
            if let Some(details) = node
                .typography
                .as_mut()
                .and_then(|typography| typography.decoration_details.as_mut())
            {
                details.style = *value;
            }
        }
        (
            DesignPanelProperty::TextDecorationOffset,
            DesignPanelValue::TextDecorationMetric(value),
        ) => {
            if value.is_finite()
                && let Some(details) = node
                    .typography
                    .as_mut()
                    .and_then(|typography| typography.decoration_details.as_mut())
            {
                details.offset = *value;
            }
        }
        (
            DesignPanelProperty::TextDecorationThickness,
            DesignPanelValue::TextDecorationMetric(value),
        ) => {
            if value.is_non_negative()
                && let Some(details) = node
                    .typography
                    .as_mut()
                    .and_then(|typography| typography.decoration_details.as_mut())
            {
                details.thickness = *value;
            }
        }
        (
            DesignPanelProperty::TextDecorationColor,
            DesignPanelValue::TextDecorationColor(value),
        ) => {
            if let Some(details) = node
                .typography
                .as_mut()
                .and_then(|typography| typography.decoration_details.as_mut())
            {
                details.color = *value;
            }
        }
        (DesignPanelProperty::TextDecorationSkipInk, DesignPanelValue::Bool(value)) => {
            if let Some(details) = node
                .typography
                .as_mut()
                .and_then(|typography| typography.decoration_details.as_mut())
            {
                details.skip_ink = *value;
            }
        }
        (DesignPanelProperty::TextCase, DesignPanelValue::TextCase(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.case = *value;
            }
        }
        (DesignPanelProperty::TextList, DesignPanelValue::TextList(value)) => {
            if let Some(typography) = node.typography.as_mut() {
                typography.list = *value;
            }
        }
        (DesignPanelProperty::PolygonCount, DesignPanelValue::Integer(value)) => {
            if let DesignShapeGeometry::Polygon(geometry) = &mut node.shape_geometry {
                geometry.point_count = (*value).clamp(3, 60) as u16;
            }
        }
        (DesignPanelProperty::StarPointCount, DesignPanelValue::Integer(value)) => {
            if let DesignShapeGeometry::Star(geometry) = &mut node.shape_geometry {
                geometry.point_count = (*value).clamp(3, 60) as u16;
            }
        }
        (DesignPanelProperty::StarInnerRadius, DesignPanelValue::Ratio(value)) => {
            if let DesignShapeGeometry::Star(geometry) = &mut node.shape_geometry {
                geometry.inner_radius = value.clamp(0., 1.);
            }
        }
        (DesignPanelProperty::ArcStartingAngle, DesignPanelValue::AngleRadians(value)) => {
            if let DesignShapeGeometry::Ellipse(arc) = &mut node.shape_geometry {
                let sweep = arc.sweep_angle();
                arc.starting_angle = *value;
                arc.ending_angle = *value + sweep;
            }
        }
        (DesignPanelProperty::ArcSweep, DesignPanelValue::AngleRadians(value)) => {
            if let DesignShapeGeometry::Ellipse(arc) = &mut node.shape_geometry {
                arc.ending_angle = arc.starting_angle + *value;
            }
        }
        (DesignPanelProperty::ArcEndingAngle, DesignPanelValue::AngleRadians(value)) => {
            if let DesignShapeGeometry::Ellipse(arc) = &mut node.shape_geometry {
                arc.ending_angle = *value;
            }
        }
        (DesignPanelProperty::ArcInnerRadius, DesignPanelValue::Ratio(value)) => {
            if let DesignShapeGeometry::Ellipse(arc) = &mut node.shape_geometry {
                arc.inner_radius = value.clamp(0., 1.);
            }
        }
        (DesignPanelProperty::BooleanOperation, DesignPanelValue::BooleanOperation(operation)) => {
            node.shape_geometry = DesignShapeGeometry::Boolean(*operation);
        }
        (DesignPanelProperty::IsMask, DesignPanelValue::Bool(value)) => {
            node.is_mask = *value;
            node.mask_type = value.then(|| node.mask_mode.label().into());
        }
        (DesignPanelProperty::MaskType, DesignPanelValue::MaskType(value)) => {
            node.is_mask = true;
            node.mask_mode = *value;
            node.mask_type = Some(value.label().into());
        }
        (DesignPanelProperty::SectionContentsHidden, DesignPanelValue::Bool(value)) => {
            if let Some(section) = node.section.as_mut() {
                section.contents_hidden = *value;
            }
        }
        (DesignPanelProperty::SectionDevStatus, DesignPanelValue::SectionDevStatus(value)) => {
            if let Some(section) = node.section.as_mut() {
                section.dev_status = value.map(DesignSectionDevStatus::new);
            }
        }
        (
            DesignPanelProperty::TransformRepeatType(index),
            DesignPanelValue::RepeatType(repeat_type),
        ) => {
            if let Some(modifier) = node.transform_modifiers.get_mut(index) {
                modifier.mode = match repeat_type {
                    DesignRepeatType::Linear => DesignRepeatMode::Linear(
                        modifier.mode.axis().unwrap_or(DesignRepeatAxis::Horizontal),
                    ),
                    DesignRepeatType::Radial => DesignRepeatMode::Radial,
                };
            }
        }
        (DesignPanelProperty::TransformRepeatAxis(index), DesignPanelValue::RepeatAxis(axis)) => {
            if let Some(modifier) = node.transform_modifiers.get_mut(index) {
                modifier.mode = DesignRepeatMode::Linear(*axis);
            }
        }
        (DesignPanelProperty::TransformRepeatCount(index), DesignPanelValue::Integer(value)) => {
            if let Some(modifier) = node.transform_modifiers.get_mut(index) {
                modifier.count = u32::try_from((*value).max(1)).unwrap_or(u32::MAX);
            }
        }
        (
            DesignPanelProperty::TransformRepeatUnit(index),
            DesignPanelValue::TransformUnit(unit),
        ) => {
            if let Some(modifier) = node.transform_modifiers.get_mut(index) {
                modifier.unit = *unit;
            }
        }
        (DesignPanelProperty::TransformRepeatOffset(index), DesignPanelValue::Number(value)) => {
            if let Some(modifier) = node.transform_modifiers.get_mut(index) {
                modifier.offset = *value;
            }
        }
        (
            DesignPanelProperty::PaintOpacity { collection, index },
            DesignPanelValue::Number(value),
        ) => {
            let paint = match collection {
                DesignPanelCollection::Fill
                    if node.kind == DesignPanelNodeKind::MultipleSelection =>
                {
                    node.selection_colors.get_mut(index)
                }
                DesignPanelCollection::Fill => node.fills.get_mut(index),
                DesignPanelCollection::Stroke => node
                    .stroke
                    .as_mut()
                    .and_then(|stroke| stroke.paints.get_mut(index)),
                DesignPanelCollection::Effect
                | DesignPanelCollection::LayoutGrid
                | DesignPanelCollection::Export => None,
            };
            if let Some(paint) = paint {
                paint.opacity = *value;
            }
        }
        (
            DesignPanelProperty::PaintVisible { collection, index },
            DesignPanelValue::Bool(value),
        ) => {
            let paint = match collection {
                DesignPanelCollection::Fill
                    if node.kind == DesignPanelNodeKind::MultipleSelection =>
                {
                    node.selection_colors.get_mut(index)
                }
                DesignPanelCollection::Fill => node.fills.get_mut(index),
                DesignPanelCollection::Stroke => node
                    .stroke
                    .as_mut()
                    .and_then(|stroke| stroke.paints.get_mut(index)),
                DesignPanelCollection::Effect
                | DesignPanelCollection::LayoutGrid
                | DesignPanelCollection::Export => None,
            };
            if let Some(paint) = paint {
                paint.visible = *value;
            }
        }
        (DesignPanelProperty::StrokeWeight, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.weights.set_active(*value);
            }
        }
        (DesignPanelProperty::StrokeWeightMode, DesignPanelValue::StrokeWeightMode(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.weights.set_mode(*value);
            }
        }
        (DesignPanelProperty::StrokeWeightTop, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.weights.top = *value;
            }
        }
        (DesignPanelProperty::StrokeWeightRight, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.weights.right = *value;
            }
        }
        (DesignPanelProperty::StrokeWeightBottom, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.weights.bottom = *value;
            }
        }
        (DesignPanelProperty::StrokeWeightLeft, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.weights.left = *value;
            }
        }
        (DesignPanelProperty::StrokeAlign, DesignPanelValue::StrokeAlign(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && stroke.alignment_options().contains(value)
            {
                stroke.align = *value;
            }
        }
        (DesignPanelProperty::StrokeStartCap, DesignPanelValue::StrokeCap(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.start_cap = *value;
            }
        }
        (DesignPanelProperty::StrokeEndCap, DesignPanelValue::StrokeCap(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.end_cap = *value;
            }
        }
        (DesignPanelProperty::StrokeEndpointCap, DesignPanelValue::StrokeCap(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.endpoint_cap = *value;
            }
        }
        (DesignPanelProperty::StrokeDashMode, DesignPanelValue::StrokeDashMode(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.set_dash_mode(*value);
            }
        }
        (DesignPanelProperty::StrokeDashPattern, DesignPanelValue::NumberList(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.set_dash_pattern(value.clone());
            }
        }
        (DesignPanelProperty::StrokeDashCap, DesignPanelValue::StrokeCap(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.dash_cap = *value;
            }
        }
        (DesignPanelProperty::StrokeJoin, DesignPanelValue::StrokeJoin(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.join = *value;
            }
        }
        (DesignPanelProperty::StrokeMiterAngle, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.miter_angle = value.clamp(0., 180.);
            }
        }
        (
            DesignPanelProperty::StrokeVariableWidth,
            DesignPanelValue::StrokeVariableWidth(value),
        ) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.set_variable_width(value.clone());
            }
        }
        (
            DesignPanelProperty::StrokeVariableWidthPointPosition(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(stroke) = node.stroke.as_mut()
                && stroke.supports_variable_width()
                && let Some(variable_width) = stroke.variable_width.as_mut()
            {
                variable_width.set_point_position(index, *value);
            }
        }
        (
            DesignPanelProperty::StrokeVariableWidthPointWidth(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(stroke) = node.stroke.as_mut()
                && stroke.supports_variable_width()
                && let Some(variable_width) = stroke.variable_width.as_mut()
            {
                variable_width.set_point_width(index, *value);
            }
        }
        (DesignPanelProperty::StrokeType, DesignPanelValue::StrokeType(value)) => {
            if let Some(stroke) = node.stroke.as_mut() {
                stroke.set_type(*value);
            }
        }
        (DesignPanelProperty::StrokeStretchBrush, DesignPanelValue::StrokeStretchBrush(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::StretchBrush(stretch) = &mut stroke.complex_stroke
            {
                stretch.brush = *value;
            }
        }
        (
            DesignPanelProperty::StrokeBrushDirection,
            DesignPanelValue::StrokeBrushDirection(value),
        ) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::StretchBrush(stretch) = &mut stroke.complex_stroke
            {
                stretch.direction = *value;
            }
        }
        (DesignPanelProperty::StrokeScatterBrush, DesignPanelValue::StrokeScatterBrush(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::ScatterBrush(scatter) = &mut stroke.complex_stroke
            {
                scatter.brush = *value;
            }
        }
        (DesignPanelProperty::StrokeScatterGap, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::ScatterBrush(scatter) = &mut stroke.complex_stroke
            {
                let candidate = DesignScatterBrushStroke {
                    gap: *value,
                    ..*scatter
                };
                if candidate.is_valid() {
                    *scatter = candidate;
                }
            }
        }
        (DesignPanelProperty::StrokeScatterWiggle, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::ScatterBrush(scatter) = &mut stroke.complex_stroke
            {
                let candidate = DesignScatterBrushStroke {
                    wiggle: *value,
                    ..*scatter
                };
                if candidate.is_valid() {
                    *scatter = candidate;
                }
            }
        }
        (DesignPanelProperty::StrokeScatterSizeJitter, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::ScatterBrush(scatter) = &mut stroke.complex_stroke
            {
                let candidate = DesignScatterBrushStroke {
                    size_jitter: *value,
                    ..*scatter
                };
                if candidate.is_valid() {
                    *scatter = candidate;
                }
            }
        }
        (DesignPanelProperty::StrokeScatterAngularJitter, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::ScatterBrush(scatter) = &mut stroke.complex_stroke
            {
                let candidate = DesignScatterBrushStroke {
                    angular_jitter: *value,
                    ..*scatter
                };
                if candidate.is_valid() {
                    *scatter = candidate;
                }
            }
        }
        (DesignPanelProperty::StrokeScatterRotation, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::ScatterBrush(scatter) = &mut stroke.complex_stroke
            {
                let candidate = DesignScatterBrushStroke {
                    rotation: *value,
                    ..*scatter
                };
                if candidate.is_valid() {
                    *scatter = candidate;
                }
            }
        }
        (DesignPanelProperty::StrokeDynamicFrequency, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::Dynamic(dynamic) = &mut stroke.complex_stroke
            {
                let candidate = DesignDynamicStroke {
                    frequency: *value,
                    ..*dynamic
                };
                if candidate.is_valid() {
                    *dynamic = candidate;
                }
            }
        }
        (DesignPanelProperty::StrokeDynamicWiggle, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::Dynamic(dynamic) = &mut stroke.complex_stroke
            {
                let candidate = DesignDynamicStroke {
                    wiggle: *value,
                    ..*dynamic
                };
                if candidate.is_valid() {
                    *dynamic = candidate;
                }
            }
        }
        (DesignPanelProperty::StrokeDynamicSmoothen, DesignPanelValue::Number(value)) => {
            if let Some(stroke) = node.stroke.as_mut()
                && let DesignComplexStroke::Dynamic(dynamic) = &mut stroke.complex_stroke
            {
                let candidate = DesignDynamicStroke {
                    smoothen: *value,
                    ..*dynamic
                };
                if candidate.is_valid() {
                    *dynamic = candidate;
                }
            }
        }
        (DesignPanelProperty::EffectKind(index), DesignPanelValue::EffectKind(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                effect.set_kind(*value);
            }
        }
        (DesignPanelProperty::EffectVisible(index), DesignPanelValue::Bool(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                effect.visible = *value;
            }
        }
        (DesignPanelProperty::EffectShadowColor(index), DesignPanelValue::Color(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                match &mut effect.settings {
                    DesignEffectSettings::DropShadow(settings) => settings.color = *value,
                    DesignEffectSettings::InnerShadow(settings) => settings.color = *value,
                    _ => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectShadowBlendMode(index), DesignPanelValue::BlendMode(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                match &mut effect.settings {
                    DesignEffectSettings::DropShadow(settings) => settings.blend_mode = *value,
                    DesignEffectSettings::InnerShadow(settings) => settings.blend_mode = *value,
                    _ => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectShadowBlur(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                match &mut effect.settings {
                    DesignEffectSettings::DropShadow(settings) => settings.radius = *value,
                    DesignEffectSettings::InnerShadow(settings) => settings.radius = *value,
                    _ => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectShadowSpread(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                match &mut effect.settings {
                    DesignEffectSettings::DropShadow(settings) => settings.spread = *value,
                    DesignEffectSettings::InnerShadow(settings) => settings.spread = *value,
                    _ => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectShadowOffsetX(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                match &mut effect.settings {
                    DesignEffectSettings::DropShadow(settings) => settings.offset.x = *value,
                    DesignEffectSettings::InnerShadow(settings) => settings.offset.x = *value,
                    _ => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectShadowOffsetY(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                match &mut effect.settings {
                    DesignEffectSettings::DropShadow(settings) => settings.offset.y = *value,
                    DesignEffectSettings::InnerShadow(settings) => settings.offset.y = *value,
                    _ => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (
            DesignPanelProperty::EffectDropShadowShowBehindNode(index),
            DesignPanelValue::Bool(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::DropShadow(settings) = &mut effect.settings
            {
                settings.show_behind_node = *value;
            }
        }
        (DesignPanelProperty::EffectBlurType(index), DesignPanelValue::EffectBlurType(value)) => {
            if let Some(effect) = node.effects.get_mut(index) {
                match &mut effect.settings {
                    DesignEffectSettings::LayerBlur(settings)
                    | DesignEffectSettings::BackgroundBlur(settings) => {
                        settings.set_blur_type(*value);
                    }
                    _ => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (
            DesignPanelProperty::EffectBlurRadius(index)
            | DesignPanelProperty::EffectProgressiveBlurEndRadius(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index) {
                match &mut effect.settings {
                    DesignEffectSettings::LayerBlur(settings)
                    | DesignEffectSettings::BackgroundBlur(settings) => {
                        settings.set_end_radius(*value);
                    }
                    _ => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (
            DesignPanelProperty::EffectProgressiveBlurStartRadius(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::LayerBlur(settings)
                | DesignEffectSettings::BackgroundBlur(settings) = &mut effect.settings
                && let fanta_gpui::design::DesignBlurEffect::Progressive { start_radius, .. } =
                    settings
            {
                *start_radius = *value;
            }
        }
        (
            DesignPanelProperty::EffectProgressiveBlurStartOffsetX(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::LayerBlur(settings)
                | DesignEffectSettings::BackgroundBlur(settings) = &mut effect.settings
                && let fanta_gpui::design::DesignBlurEffect::Progressive { start_offset, .. } =
                    settings
            {
                start_offset.x = *value;
            }
        }
        (
            DesignPanelProperty::EffectProgressiveBlurStartOffsetY(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::LayerBlur(settings)
                | DesignEffectSettings::BackgroundBlur(settings) = &mut effect.settings
                && let fanta_gpui::design::DesignBlurEffect::Progressive { start_offset, .. } =
                    settings
            {
                start_offset.y = *value;
            }
        }
        (
            DesignPanelProperty::EffectProgressiveBlurEndOffsetX(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::LayerBlur(settings)
                | DesignEffectSettings::BackgroundBlur(settings) = &mut effect.settings
                && let fanta_gpui::design::DesignBlurEffect::Progressive { end_offset, .. } =
                    settings
            {
                end_offset.x = *value;
            }
        }
        (
            DesignPanelProperty::EffectProgressiveBlurEndOffsetY(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::LayerBlur(settings)
                | DesignEffectSettings::BackgroundBlur(settings) = &mut effect.settings
                && let fanta_gpui::design::DesignBlurEffect::Progressive { end_offset, .. } =
                    settings
            {
                end_offset.y = *value;
            }
        }
        (DesignPanelProperty::EffectNoiseType(index), DesignPanelValue::EffectNoiseType(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Noise(settings) = &mut effect.settings
            {
                settings.colors.set_noise_type(*value);
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectNoisePrimaryColor(index), DesignPanelValue::Color(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Noise(settings) = &mut effect.settings
            {
                match &mut settings.colors {
                    DesignNoiseColors::Monotone { color }
                    | DesignNoiseColors::Duotone { color, .. } => *color = *value,
                    DesignNoiseColors::Multitone { .. } => {}
                }
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectNoiseSecondaryColor(index), DesignPanelValue::Color(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Noise(settings) = &mut effect.settings
                && let DesignNoiseColors::Duotone {
                    secondary_color, ..
                } = &mut settings.colors
            {
                *secondary_color = *value;
            }
        }
        (DesignPanelProperty::EffectNoiseOpacity(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Noise(settings) = &mut effect.settings
                && let DesignNoiseColors::Multitone { opacity } = &mut settings.colors
            {
                *opacity = *value;
            }
        }
        (DesignPanelProperty::EffectNoiseBlendMode(index), DesignPanelValue::BlendMode(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Noise(settings) = &mut effect.settings
            {
                settings.blend_mode = *value;
            }
        }
        (DesignPanelProperty::EffectNoiseSizeX(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Noise(settings) = &mut effect.settings
            {
                settings.size.x = *value;
            }
        }
        (DesignPanelProperty::EffectNoiseSizeY(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Noise(settings) = &mut effect.settings
            {
                settings.size.y = *value;
            }
        }
        (DesignPanelProperty::EffectNoiseDensity(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Noise(settings) = &mut effect.settings
            {
                settings.density = *value;
            }
        }
        (DesignPanelProperty::EffectTextureSizeX(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Texture(settings) = &mut effect.settings
            {
                settings.size.x = *value;
            }
        }
        (DesignPanelProperty::EffectTextureSizeY(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Texture(settings) = &mut effect.settings
            {
                settings.size.y = *value;
            }
        }
        (DesignPanelProperty::EffectTextureRadius(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Texture(settings) = &mut effect.settings
            {
                settings.radius = *value;
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectTextureClipToShape(index), DesignPanelValue::Bool(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Texture(settings) = &mut effect.settings
            {
                settings.clip_to_shape = *value;
            }
        }
        (
            DesignPanelProperty::EffectGlassLightIntensity(index),
            DesignPanelValue::Number(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Glass(settings) = &mut effect.settings
            {
                settings.light_intensity = *value;
            }
        }
        (DesignPanelProperty::EffectGlassLightAngle(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Glass(settings) = &mut effect.settings
            {
                settings.light_angle = *value;
            }
        }
        (DesignPanelProperty::EffectGlassRefraction(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Glass(settings) = &mut effect.settings
            {
                settings.refraction = *value;
            }
        }
        (DesignPanelProperty::EffectGlassDepth(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Glass(settings) = &mut effect.settings
            {
                settings.depth = *value;
            }
        }
        (DesignPanelProperty::EffectGlassDispersion(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Glass(settings) = &mut effect.settings
            {
                settings.dispersion = *value;
            }
        }
        (DesignPanelProperty::EffectGlassFrost(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Glass(settings) = &mut effect.settings
            {
                settings.frost = *value;
                effect.sync_compatibility_summary();
            }
        }
        (DesignPanelProperty::EffectGlassSplay(index), DesignPanelValue::Number(value)) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Glass(settings) = &mut effect.settings
            {
                settings.splay = *value;
            }
        }
        (
            DesignPanelProperty::EffectShaderProperty(index, property_index),
            DesignPanelValue::ShaderProperty(value),
        ) => {
            if let Some(effect) = node.effects.get_mut(index)
                && let DesignEffectSettings::Shader(shader) = &mut effect.settings
                && let Some(property) = shader.properties.get_mut(property_index)
                && !property.read_only
                && value.is_compatible_with(property.kind)
            {
                property.value = value.clone();
            }
        }
        (DesignPanelProperty::ExportSuffix(index), DesignPanelValue::Text(value)) => {
            if let Some(export) = node.export_settings.get_mut(index) {
                export.suffix = value.clone();
            }
        }
        (DesignPanelProperty::ExportFormat(index), DesignPanelValue::ExportFormat(value)) => {
            if let Some(export) = node.export_settings.get_mut(index) {
                export.format = *value;
            }
        }
        (unhandled_property, unhandled_value) => {
            debug_assert!(
                false,
                "Storybook host has no reducer for {unhandled_property:?} with {unhandled_value:?}"
            );
        }
    }
}

fn story_page_local_styles_projection_is_current(
    selection_is_page: bool,
    panel_page_id: Option<&SharedString>,
    panel_styles: Option<&DesignPageLocalStylesViewData>,
    host_page_id: &SharedString,
    host_styles: &DesignPageLocalStylesViewData,
) -> bool {
    selection_is_page
        && panel_page_id == Some(host_page_id)
        && panel_styles == Some(host_styles)
        && host_styles.target
            == DesignPanelTarget::Page {
                page_id: host_page_id.clone(),
            }
        && host_styles.is_valid()
}

fn story_local_style_entries_mut<'a>(
    view_data: &'a mut DesignPageLocalStylesViewData,
    kind: DesignLocalStyleKind,
    parent_folder_id: Option<&str>,
) -> Option<&'a mut Vec<DesignLocalStyleEntry>> {
    fn folder_entries_mut<'a>(
        entries: &'a mut [DesignLocalStyleEntry],
        folder_id: &str,
    ) -> Option<&'a mut Vec<DesignLocalStyleEntry>> {
        for entry in entries {
            if let DesignLocalStyleEntry::Folder {
                id,
                entries: children,
                ..
            } = entry
            {
                if id.as_ref() == folder_id {
                    return Some(children);
                }
                if let Some(found) = folder_entries_mut(children, folder_id) {
                    return Some(found);
                }
            }
        }
        None
    }

    let section = view_data
        .sections
        .iter_mut()
        .find(|section| section.kind == kind)?;
    match parent_folder_id {
        None => Some(&mut section.entries),
        Some(folder_id) => folder_entries_mut(&mut section.entries, folder_id),
    }
}

fn story_local_style_insertion_is_current(
    view_data: &DesignPageLocalStylesViewData,
    insertion: &DesignLocalStyleInsertion,
) -> bool {
    if !view_data.is_valid()
        || insertion
            .parent_folder_id
            .as_ref()
            .is_some_and(|folder_id| {
                !view_data.entry_path_is_enabled(insertion.kind, folder_id.as_ref())
            })
    {
        return false;
    }
    let Some(entries) = view_data.entries(
        insertion.kind,
        insertion
            .parent_folder_id
            .as_ref()
            .map(|folder_id| folder_id.as_ref()),
    ) else {
        return false;
    };
    if insertion.expected_index > entries.len() {
        return false;
    }
    let before = insertion
        .expected_index
        .checked_sub(1)
        .and_then(|index| entries.get(index))
        .map(DesignLocalStyleEntry::id);
    let after = entries
        .get(insertion.expected_index)
        .map(DesignLocalStyleEntry::id);
    before == insertion.expected_before_id.as_ref() && after == insertion.expected_after_id.as_ref()
}

fn story_local_style_targets_are_current(
    view_data: &DesignPageLocalStylesViewData,
    targets: &[DesignLocalStyleTarget],
    allow_empty: bool,
) -> bool {
    view_data.is_valid()
        && (!targets.is_empty() || allow_empty)
        && targets.iter().collect::<HashSet<_>>().len() == targets.len()
        && targets.iter().all(|target| {
            view_data.resolve_style(target).is_some()
                && view_data.entry_path_is_enabled(target.kind, target.style_id.as_ref())
        })
}

fn story_remove_local_style_targets(
    view_data: &mut DesignPageLocalStylesViewData,
    targets: &[DesignLocalStyleTarget],
) -> Option<Vec<DesignLocalStyleEntry>> {
    if !story_local_style_targets_are_current(view_data, targets, false) {
        return None;
    }
    let mut removed = Vec::with_capacity(targets.len());
    for target in targets {
        let entries = story_local_style_entries_mut(
            view_data,
            target.kind,
            target
                .parent_folder_id
                .as_ref()
                .map(|folder_id| folder_id.as_ref()),
        )?;
        let current_index = entries
            .iter()
            .position(|entry| entry.id() == &target.style_id)?;
        removed.push(entries.remove(current_index));
    }
    Some(removed)
}

fn story_default_local_style(kind: DesignLocalStyleKind, id: SharedString) -> DesignLocalStyleItem {
    let preview =
        match kind {
            DesignLocalStyleKind::Text => DesignLocalStylePreview::Text(
                DesignTypographyStyle::new(id.clone(), "New text style", "Inter", "Regular", 16.),
            ),
            DesignLocalStyleKind::Color => DesignLocalStylePreview::Color(vec![
                DesignPaint::solid(DesignColor::BLACK).with_id(format!("{id}-paint")),
            ]),
            DesignLocalStyleKind::Effect => DesignLocalStylePreview::Effect(vec![
                DesignEffect::new(DesignEffectKind::DropShadow).with_id(format!("{id}-effect")),
            ]),
            DesignLocalStyleKind::LayoutGuide => DesignLocalStylePreview::LayoutGuide(vec![
                DesignLayoutGrid::uniform(8., DesignColor::rgb(255, 0, 128))
                    .with_id(format!("{id}-guide")),
            ]),
        };
    DesignLocalStyleItem::new(id, "New style", preview)
}

fn story_create_local_style(
    view_data: &mut DesignPageLocalStylesViewData,
    page_id: &SharedString,
    kind: DesignLocalStyleKind,
    parent_folder_id: Option<&SharedString>,
    next_id: usize,
) -> bool {
    if view_data.page_id() != Some(page_id)
        || !view_data.is_valid()
        || view_data.create_disabled_reason.is_some()
        || parent_folder_id
            .as_ref()
            .is_some_and(|folder_id| !view_data.entry_path_is_enabled(kind, folder_id.as_ref()))
    {
        return false;
    }
    if view_data.section(kind).is_none() {
        if parent_folder_id.is_some() {
            return false;
        }
        view_data
            .sections
            .push(DesignLocalStyleSection::new(kind, []));
    }
    let Some(entries) = story_local_style_entries_mut(
        view_data,
        kind,
        parent_folder_id.map(|folder_id| folder_id.as_ref()),
    ) else {
        return false;
    };
    let id = SharedString::from(format!("storybook-local-style-{next_id}"));
    entries.push(DesignLocalStyleEntry::style(story_default_local_style(
        kind, id,
    )));
    true
}

fn story_duplicate_local_style(
    view_data: &mut DesignPageLocalStylesViewData,
    target: &DesignLocalStyleTarget,
    next_id: usize,
) -> bool {
    let Some(mut duplicate) = view_data.resolve_style(target).cloned() else {
        return false;
    };
    if !view_data.entry_path_is_enabled(target.kind, target.style_id.as_ref()) {
        return false;
    }
    duplicate.id = SharedString::from(format!("storybook-local-style-{next_id}"));
    duplicate.name = SharedString::from(format!("{} copy", duplicate.name));
    duplicate.disabled_reason = None;
    let Some(entries) = story_local_style_entries_mut(
        view_data,
        target.kind,
        target
            .parent_folder_id
            .as_ref()
            .map(|folder_id| folder_id.as_ref()),
    ) else {
        return false;
    };
    entries.insert(
        target.expected_index + 1,
        DesignLocalStyleEntry::style(duplicate),
    );
    true
}

fn story_create_local_style_folder(
    view_data: &mut DesignPageLocalStylesViewData,
    page_id: &SharedString,
    kind: DesignLocalStyleKind,
    selected: &[DesignLocalStyleTarget],
    next_id: usize,
) -> bool {
    if view_data.page_id() != Some(page_id)
        || !view_data.is_valid()
        || view_data.create_disabled_reason.is_some()
        || !story_local_style_targets_are_current(view_data, selected, true)
        || selected
            .iter()
            .any(|target| target.page_id != *page_id || target.kind != kind)
    {
        return false;
    }
    if view_data.section(kind).is_none() {
        if !selected.is_empty() {
            return false;
        }
        view_data
            .sections
            .push(DesignLocalStyleSection::new(kind, []));
    }
    let parent_folder_id = selected
        .first()
        .and_then(|target| target.parent_folder_id.clone());
    if selected
        .iter()
        .any(|target| target.parent_folder_id != parent_folder_id)
    {
        return false;
    }
    let insertion_index = selected
        .iter()
        .map(|target| target.expected_index)
        .min()
        .unwrap_or_else(|| {
            view_data
                .entries(
                    kind,
                    parent_folder_id
                        .as_ref()
                        .map(|folder_id| folder_id.as_ref()),
                )
                .map_or(0, <[DesignLocalStyleEntry]>::len)
        });
    let children = if selected.is_empty() {
        Vec::new()
    } else {
        let Some(children) = story_remove_local_style_targets(view_data, selected) else {
            return false;
        };
        children
    };
    let Some(entries) = story_local_style_entries_mut(
        view_data,
        kind,
        parent_folder_id
            .as_ref()
            .map(|folder_id| folder_id.as_ref()),
    ) else {
        return false;
    };
    entries.insert(
        insertion_index.min(entries.len()),
        DesignLocalStyleEntry::folder(
            format!("storybook-local-style-folder-{next_id}"),
            "New folder",
            children,
        ),
    );
    true
}

fn story_move_local_styles(
    view_data: &mut DesignPageLocalStylesViewData,
    targets: &[DesignLocalStyleTarget],
    destination: &DesignLocalStyleInsertion,
) -> bool {
    if !story_local_style_targets_are_current(view_data, targets, false)
        || targets.iter().any(|target| target.kind != destination.kind)
        || !story_local_style_insertion_is_current(view_data, destination)
    {
        return false;
    }
    let removed_before_destination = targets
        .iter()
        .filter(|target| {
            target.kind == destination.kind
                && target.parent_folder_id == destination.parent_folder_id
                && target.expected_index < destination.expected_index
        })
        .count();
    let insertion_index = destination
        .expected_index
        .saturating_sub(removed_before_destination);
    let Some(removed) = story_remove_local_style_targets(view_data, targets) else {
        return false;
    };
    let Some(entries) = story_local_style_entries_mut(
        view_data,
        destination.kind,
        destination
            .parent_folder_id
            .as_ref()
            .map(|folder_id| folder_id.as_ref()),
    ) else {
        return false;
    };
    if insertion_index > entries.len() {
        return false;
    }
    for (offset, entry) in removed.into_iter().enumerate() {
        entries.insert(insertion_index + offset, entry);
    }
    true
}

fn apply_figma_ui3_storybook_theme(cx: &mut App) {
    let theme = Theme::global_mut(cx);
    theme.background = rgba(0x2c2c2cff).into();
    theme.foreground = rgba(0xffffffff).into();
    theme.secondary = rgba(0x383838ff).into();
    theme.secondary_hover = rgba(0x444444ff).into();
    theme.secondary_active = rgba(0x4d4d4dff).into();
    theme.accent = rgba(0x444444ff).into();
    theme.accent_foreground = rgba(0xffffffff).into();
    theme.border = rgba(0x444444ff).into();
    theme.input = rgba(0x444444ff).into();
    theme.muted = rgba(0x383838ff).into();
    theme.muted_foreground = rgba(0xbfbfbfff).into();
    theme.popover = rgba(0x2c2c2cff).into();
    theme.popover_foreground = rgba(0xffffffff).into();
    theme.selection = rgba(0x0c8ce9ff).into();
    theme.sidebar = rgba(0x2c2c2cff).into();
    theme.sidebar_border = rgba(0x444444ff).into();
    theme.sidebar_foreground = rgba(0xffffffff).into();
    theme.tab = rgba(0x2c2c2cff).into();
    theme.tab_active = rgba(0x383838ff).into();
    theme.tab_active_foreground = rgba(0xffffffff).into();
    theme.title_bar = rgba(0x2c2c2cff).into();
    theme.title_bar_border = rgba(0x444444ff).into();
    theme.radius = px(4.);
}

fn main() {
    let launch = storybook_launch_from_env();
    Application::new()
        .with_assets(StorybookAssets)
        .run(move |cx: &mut App| {
            gpui_component::init(cx);
            fanta_gpui::init(cx);
            match launch.mode {
                StorybookLaunchMode::Gallery => Theme::change(storybook_theme_from_env(), None, cx),
                StorybookLaunchMode::ReferenceFixture => {
                    Theme::change(ThemeMode::Dark, None, cx);
                    apply_figma_ui3_storybook_theme(cx);
                }
            }
            cx.set_menus(vec![
                Menu {
                    name: "Fanta GPUI".into(),
                    items: vec![MenuItem::action("Toggle Theme", ToggleGalleryTheme)],
                },
                Menu {
                    name: "Edit".into(),
                    items: Vec::new(),
                },
                Menu {
                    name: "Window".into(),
                    items: vec![MenuItem::action("Open Story Window", OpenStoryWindow)],
                },
                Menu {
                    name: "Help".into(),
                    items: Vec::new(),
                },
            ]);
            cx.activate(true);

            let initial_story = launch.story;
            let (reference_width, reference_height) = if launch.mode == StorybookLaunchMode::Gallery
            {
                (1240., 820.)
            } else {
                initial_story.reference_window_size()
            };
            let window_width = storybook_window_dimension("FANTA_STORYBOOK_WIDTH", reference_width);
            let window_height =
                storybook_window_dimension("FANTA_STORYBOOK_HEIGHT", reference_height);
            let bounds = Bounds::centered(None, size(px(window_width), px(window_height)), cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(
                    px(window_width.clamp(300., 980.)),
                    px(window_height.clamp(240., 620.)),
                )),
                titlebar: Some(TitlebarOptions {
                    title: Some("Fanta GPUI Storybook".into()),
                    ..Default::default()
                }),
                ..Default::default()
            };

            cx.open_window(options, |window, cx| {
                window.activate_window();
                let storybook = cx.new(|cx| Storybook::new(window, cx));
                cx.new(|cx| Root::new(storybook, window, cx))
            })
            .expect("failed to open the Storybook window");
        });
}

#[cfg(test)]
mod tests;
