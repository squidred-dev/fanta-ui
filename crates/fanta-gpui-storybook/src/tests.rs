use super::*;
use crate::screens::design::fixtures as design_fixtures;
use crate::screens::design::fixtures::*;
use crate::screens::design::reducers::*;
use crate::screens::design::*;
use fanta_gpui::prelude::{DesignBlendMode, DesignStrokeType};
use gpui::{Modifiers, ScrollDelta, ScrollWheelEvent, TestAppContext, VisualTestContext, point};

fn setup_design_story(cx: &mut TestAppContext) -> (Entity<Storybook>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_component::init(cx);
        fanta_gpui::init(cx);
        Theme::change(ThemeMode::Dark, None, cx);
        apply_figma_ui3_storybook_theme(cx);
    });
    let storybook_slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let captured_storybook = storybook_slot.clone();
    let (_, cx) = cx.add_window_view(move |window, cx| {
        let storybook = cx.new(|cx| Storybook::new(window, cx));
        *captured_storybook.borrow_mut() = Some(storybook.clone());
        Root::new(storybook, window, cx)
    });
    let storybook = storybook_slot
        .borrow_mut()
        .take()
        .expect("test Storybook should be installed");
    cx.update(|_, app| {
        storybook.update(app, |storybook, cx| {
            storybook.active_story = StoryKind::Design;
            storybook.launch_mode = StorybookLaunchMode::ReferenceFixture;
            cx.notify();
        });
    });
    cx.simulate_resize(size(px(1240.), px(820.)));
    cx.run_until_parked();
    (storybook, cx)
}

#[test]
fn launch_without_story_env_opens_the_gallery() {
    assert_eq!(
        parse_storybook_launch(None, None),
        Ok(StorybookLaunch {
            story: StoryKind::PseudoEditor,
            mode: StorybookLaunchMode::Gallery,
        })
    );
}

#[test]
fn explicit_story_env_opens_a_reference_fixture() {
    assert_eq!(
        parse_storybook_launch(Some("toolbar"), None),
        Ok(StorybookLaunch {
            story: StoryKind::Toolbar,
            mode: StorybookLaunchMode::ReferenceFixture,
        })
    );
}

#[test]
fn unknown_story_names_are_rejected_with_the_valid_id_list() {
    let error = parse_storybook_launch(Some("not-a-story"), Some("design"))
        .expect_err("unknown reference stories must not silently fall back");
    assert!(error.contains("not-a-story"), "{error}");
    assert!(error.contains("FANTA_STORYBOOK_STORY"), "{error}");
    for descriptor in screens::registry() {
        assert!(
            error.contains(descriptor.id),
            "the error must list {:?}: {error}",
            descriptor.id
        );
    }

    let error = parse_storybook_launch(None, Some("mystery"))
        .expect_err("unknown gallery stories must not silently fall back");
    assert!(error.contains("FANTA_STORYBOOK_GALLERY_STORY"), "{error}");
}

#[test]
fn gallery_story_env_selects_a_story_without_bypassing_the_gallery() {
    assert_eq!(
        parse_storybook_launch(None, Some("design")),
        Ok(StorybookLaunch {
            story: StoryKind::Design,
            mode: StorybookLaunchMode::Gallery,
        })
    );
    assert_eq!(
        parse_storybook_launch(None, Some("icon-gallery")),
        Ok(StorybookLaunch {
            story: StoryKind::Icons,
            mode: StorybookLaunchMode::Gallery,
        })
    );
}

#[test]
fn icon_catalog_covers_every_bundled_icon() {
    assert_eq!(screens::icons::icon_count(), screens::icons::ICON_COUNT);
    assert_eq!(screens::icons::ICON_COUNT, 86);
    let names = screens::icons::icon_names();
    assert_eq!(
        names.iter().copied().collect::<HashSet<_>>().len(),
        names.len(),
        "the icon catalog must not hide a missing icon behind a duplicate entry"
    );
}

#[test]
fn gallery_theme_parser_defaults_light_and_accepts_dark_case_insensitively() {
    assert_eq!(parse_storybook_theme(None), ThemeMode::Light);
    assert_eq!(parse_storybook_theme(Some("light")), ThemeMode::Light);
    assert_eq!(parse_storybook_theme(Some("DARK")), ThemeMode::Dark);
    assert_eq!(parse_storybook_theme(Some("unknown")), ThemeMode::Light);
}

#[gpui::test]
fn gallery_keeps_its_shell_when_a_story_opens_in_a_shared_window(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_component::init(cx);
        fanta_gpui::init(cx);
        Theme::change(ThemeMode::Light, None, cx);
    });
    let storybook_slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let captured_storybook = storybook_slot.clone();
    let (_, visual_cx) = cx.add_window_view(move |window, cx| {
        let storybook = cx.new(|cx| Storybook::new(window, cx));
        *captured_storybook.borrow_mut() = Some(storybook.clone());
        Root::new(storybook, window, cx)
    });
    let storybook = storybook_slot
        .borrow_mut()
        .take()
        .expect("test Storybook should be installed");
    visual_cx.update(|_, app| {
        storybook.update(app, |storybook, cx| {
            storybook.active_story = StoryKind::PseudoEditor;
            storybook.launch_mode = StorybookLaunchMode::Gallery;
            cx.notify();
        });
    });
    visual_cx.simulate_resize(size(px(1240.), px(820.)));
    visual_cx.run_until_parked();

    assert!(visual_cx.debug_bounds("storybook-gallery-shell").is_some());
    assert_eq!(
        visual_cx.read(|app| Theme::global(app).mode),
        ThemeMode::Light
    );

    for story in screens::registry().iter().map(|descriptor| descriptor.kind) {
        visual_cx.update(|_, app| {
            storybook.update(app, |storybook, cx| {
                storybook.active_story = story;
                cx.notify();
            });
        });
        visual_cx.run_until_parked();
        assert!(
            visual_cx
                .debug_bounds("storybook-gallery-story-surface")
                .is_some(),
            "{} should render inside the Gallery",
            story.title()
        );
        if story == StoryKind::Icons {
            assert!(
                visual_cx.debug_bounds("storybook-icon-gallery").is_some(),
                "the Icons foundation page should render its complete catalog"
            );
        }
    }

    visual_cx.update(|_, app| {
        storybook.update(app, |storybook, cx| {
            storybook.toggle_gallery_theme(cx);
        });
    });
    visual_cx.run_until_parked();
    assert_eq!(
        visual_cx.read(|app| Theme::global(app).mode),
        ThemeMode::Dark
    );

    visual_cx.update(|_, app| {
        storybook.update(app, |storybook, cx| {
            storybook.active_story = StoryKind::Design;
            cx.notify();
        });
    });
    visual_cx.run_until_parked();
    assert!(
        visual_cx
            .debug_bounds("design-fixture-rail-scroll")
            .is_some(),
        "the Gallery Design story must retain its property/scenario mock harness"
    );

    visual_cx.update(|_, app| {
        storybook.update(app, |storybook, cx| {
            storybook
                .story_windows
                .insert(WindowId::from(42), StoryKind::Design);
            cx.notify();
        });
    });

    assert_eq!(
        visual_cx.read(|app| {
            storybook
                .read(app)
                .story_windows
                .values()
                .copied()
                .collect::<Vec<_>>()
        }),
        vec![StoryKind::Design],
        "the new window must keep the story selected at click time"
    );
    assert!(
        visual_cx.debug_bounds("storybook-gallery-shell").is_some(),
        "registering a story window must not replace the Gallery shell"
    );

    visual_cx.update(|_, app| {
        storybook.update(app, |storybook, cx| {
            storybook.active_story = StoryKind::Pages;
            cx.notify();
        });
    });
    assert_eq!(
        visual_cx.read(|app| {
            storybook
                .read(app)
                .story_windows
                .values()
                .copied()
                .collect::<Vec<_>>()
        }),
        vec![StoryKind::Design],
        "Gallery navigation must not retarget an already-open story window"
    );
}
#[gpui::test]
fn every_reference_fixture_renders_through_the_registry(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_component::init(cx);
        fanta_gpui::init(cx);
        Theme::change(ThemeMode::Dark, None, cx);
        apply_figma_ui3_storybook_theme(cx);
    });
    let storybook_slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let captured_storybook = storybook_slot.clone();
    let (_, visual_cx) = cx.add_window_view(move |window, cx| {
        let storybook = cx.new(|cx| Storybook::new(window, cx));
        *captured_storybook.borrow_mut() = Some(storybook.clone());
        Root::new(storybook, window, cx)
    });
    let storybook = storybook_slot
        .borrow_mut()
        .take()
        .expect("test Storybook should be installed");
    visual_cx.simulate_resize(size(px(1240.), px(820.)));

    let fixture_selectors: &[(StoryKind, &str)] = &[
        (StoryKind::Welcome, "storybook-reference-welcome"),
        (StoryKind::Buttons, "storybook-reference-buttons"),
        (StoryKind::Labels, "storybook-reference-labels"),
        (StoryKind::Icons, "storybook-icon-gallery"),
        (StoryKind::Menus, "storybook-reference-menus"),
        (StoryKind::ListRows, "storybook-reference-list-rows"),
        (StoryKind::Popups, "storybook-reference-popups"),
        (StoryKind::Toolbar, "storybook-reference-toolbar"),
        (StoryKind::Pages, "storybook-reference-pages"),
        (StoryKind::Layers, "storybook-reference-layers"),
        (
            StoryKind::FileInspector,
            "storybook-reference-file-inspector",
        ),
        (StoryKind::Design, "design-fixture-rail-scroll"),
        (StoryKind::Variables, "storybook-reference-variables"),
        (StoryKind::Prototype, "storybook-reference-prototype"),
        (StoryKind::Timeline, "storybook-reference-timeline"),
        (StoryKind::PseudoEditor, "storybook-reference-pseudo-editor"),
    ];
    assert_eq!(
        fixture_selectors.len(),
        screens::registry().len(),
        "every registered story must pin a reference fixture selector"
    );
    for (story, selector) in fixture_selectors {
        assert_eq!(
            story.descriptor().kind,
            *story,
            "{selector} must resolve its descriptor"
        );
        visual_cx.update(|_, app| {
            storybook.update(app, |storybook, cx| {
                storybook.active_story = *story;
                storybook.launch_mode = StorybookLaunchMode::ReferenceFixture;
                cx.notify();
            });
        });
        visual_cx.run_until_parked();
        assert!(
            visual_cx.debug_bounds(selector).is_some(),
            "the {} reference fixture must render {selector}",
            story.title()
        );
    }
}

#[gpui::test]
fn pseudo_editor_toolbar_contains_primary_zoom_and_agent_surfaces(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_component::init(cx);
        fanta_gpui::init(cx);
        Theme::change(ThemeMode::Light, None, cx);
    });
    let storybook_slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let captured_storybook = storybook_slot.clone();
    let (_, visual_cx) = cx.add_window_view(move |window, cx| {
        let storybook = cx.new(|cx| Storybook::new(window, cx));
        *captured_storybook.borrow_mut() = Some(storybook.clone());
        Root::new(storybook, window, cx)
    });
    let storybook = storybook_slot
        .borrow_mut()
        .take()
        .expect("test Storybook should be installed");
    let toolbar = visual_cx.read(|app| storybook.read(app).toolbar_screen.toolbar.clone());
    visual_cx.update(|_, app| {
        storybook.update(app, |storybook, cx| {
            storybook.active_story = StoryKind::PseudoEditor;
            storybook.launch_mode = StorybookLaunchMode::Gallery;
            // Pin the story viewport at the reference size: this test is
            // about the toolbar's own layout, not the fluid harness.
            storybook.story_viewport.apply_preset(1440., 900.);
            cx.notify();
        });
    });
    visual_cx.simulate_resize(size(px(1240.), px(820.)));

    for mode in ToolbarMode::ALL {
        visual_cx.update(|_, app| {
            toolbar.update(app, |toolbar, cx| {
                toolbar.set_mode(*mode, cx);
                toolbar.set_zoom_percent(3_200, cx);
            });
        });
        visual_cx.run_until_parked();

        let canvas = visual_cx
            .debug_bounds("pseudo-editor-canvas")
            .expect("the pseudo editor canvas should render");
        let root = visual_cx
            .debug_bounds("editor-toolbar")
            .expect("the toolbar root should render");
        let surface = visual_cx
            .debug_bounds("editor-toolbar-surface")
            .expect("the main toolbar should render");
        let zoom = visual_cx
            .debug_bounds("toolbar-zoom-control")
            .expect("the zoom control should render");
        let agent = visual_cx
            .debug_bounds("toolbar-agent-launcher")
            .expect("the Agent launcher should render");

        assert!(root.is_contained_within(&canvas));
        for (name, bounds) in [("zoom", zoom), ("Agent", agent)] {
            assert!(
                bounds.is_contained_within(&surface),
                "{mode:?} {name} surface {bounds:?} must fit inside toolbar dock {surface:?}"
            );
        }
        assert_eq!(
            surface, root,
            "{mode:?} root should adopt the dock's intrinsic bounds"
        );
    }
}

#[gpui::test]
fn design_fixture_rail_scroll_reaches_the_node_variation_matrix(cx: &mut TestAppContext) {
    let (storybook, cx) = setup_design_story(cx);
    let rail = cx
        .debug_bounds("design-fixture-rail-scroll")
        .expect("fixture rail should render");
    let scroll_handle = cx.read(|app| {
        let storybook = storybook.read(app);
        storybook.design_screen.fixture_scroll_handle.clone()
    });
    assert_eq!(scroll_handle.bounds(), rail);
    assert!(scroll_handle.max_offset().y > px(0.));

    let last_before_scroll = cx
        .debug_bounds("design-preset-last")
        .expect("last node preset should be laid out");
    assert!(last_before_scroll.top() >= rail.bottom());

    cx.simulate_event(ScrollWheelEvent {
        position: rail.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-100_000.))),
        ..Default::default()
    });
    cx.run_until_parked();

    assert!(scroll_handle.offset().y < px(0.));
    let last_after_scroll = cx
        .debug_bounds("design-preset-last")
        .expect("last node preset should remain rendered after scrolling");
    assert!(last_after_scroll.top() < rail.bottom());
    assert!(last_after_scroll.bottom() > rail.top());
}

#[test]
fn storybook_additional_labels_toggle_has_explicit_accessible_states() {
    assert_eq!(design_additional_labels_status(false), "Off");
    assert_eq!(design_additional_labels_status(true), "On");
    assert!(!parse_design_additional_labels(None));
    assert!(!parse_design_additional_labels(Some("0")));
    assert!(parse_design_additional_labels(Some("1")));
    assert!(parse_design_additional_labels(Some("ON")));
    assert!(parse_design_additional_labels(Some("true")));
}

#[test]
fn storybook_nudge_presets_expose_default_and_custom_preferences() {
    let default = DesignNudgeSettings::default();
    let custom = DesignNudgeSettings::new(0.5, 8.).expect("valid custom preset");
    assert_eq!((default.small(), default.big()), (1., 10.));
    assert_eq!((custom.small(), custom.big()), (0.5, 8.));
    assert_ne!(custom, default);
}

#[test]
fn page_local_style_reducer_rejects_stale_selection_and_page_projections() {
    let page_id: SharedString = "storybook-page".into();
    let styles = design_fixtures::seed_design_page_local_styles();
    assert!(story_page_local_styles_projection_is_current(
        true,
        Some(&page_id),
        Some(&styles),
        &page_id,
        &styles,
    ));
    assert!(!story_page_local_styles_projection_is_current(
        false,
        Some(&page_id),
        Some(&styles),
        &page_id,
        &styles,
    ));

    let stale_page: SharedString = "stale-page".into();
    assert!(!story_page_local_styles_projection_is_current(
        true,
        Some(&stale_page),
        Some(&styles),
        &page_id,
        &styles,
    ));
    let stale_styles =
        DesignPageLocalStylesViewData::for_page("stale-page", styles.sections.clone());
    assert!(!story_page_local_styles_projection_is_current(
        true,
        Some(&page_id),
        Some(&stale_styles),
        &page_id,
        &styles,
    ));
}

#[test]
fn design_story_media_nodes_cover_independent_source_action_permissions() {
    let nodes = design_fixtures::seed_design_nodes();
    let views = design_fixtures::seed_design_media_paint_views(&nodes);
    let expected = [
        ("paint-image-0", "image-0-fill", [true, true, true]),
        ("paint-image-1", "image-1-fill", [true, false, true]),
        ("paint-image-2", "image-2-fill", [false, false, false]),
        ("paint-image-3", "image-3-fill", [false, true, false]),
    ];
    for (node_id, paint_id, allowed) in expected {
        let view = views
            .get(node_id)
            .and_then(|views| {
                views.paint(
                    DesignPanelCollection::Fill,
                    &SharedString::from(paint_id),
                    0,
                )
            })
            .expect("story image media view");
        assert_eq!(
            DesignMediaSourceAction::ALL
                .map(|action| view.capabilities.allows_source_action(action)),
            allowed
        );
    }
    assert!(
        nodes
            .iter()
            .any(|node| { node.id.as_ref() == "paint-image-3" && node.name.contains("Make only") })
    );
    let tiff_opt_in = views
        .get("paint-image-1")
        .and_then(|views| {
            views.paint(
                DesignPanelCollection::Fill,
                &SharedString::from("image-1-fill"),
                0,
            )
        })
        .expect("TIFF opt-in story media view");
    assert!(
        tiff_opt_in
            .capabilities
            .allows_file_drop(DesignMediaFileKind::Tiff)
    );
    assert!(
        !views
            .get("paint-image-0")
            .and_then(|views| {
                views.paint(
                    DesignPanelCollection::Fill,
                    &SharedString::from("image-0-fill"),
                    0,
                )
            })
            .expect("standard story media view")
            .capabilities
            .allows_file_drop(DesignMediaFileKind::Tiff)
    );
}

#[test]
fn story_media_drop_preserves_identity_order_common_and_media_settings_across_kind() {
    let mut transform = DesignPaintTransform::IDENTITY;
    transform.tx = 0.25;
    transform.ty = -0.125;
    let placement = DesignMediaPaintPlacement::Crop { transform };
    let filters = DesignImageFilters {
        exposure: 0.2,
        contrast: -0.1,
        saturation: 0.3,
        ..DesignImageFilters::default()
    };
    let untouched = DesignPaint::solid(DesignColor::WHITE).with_id("background");
    let mut media =
        DesignPaint::image(DesignPaintSource::new("old-source", "Old")).with_id("media");
    media.opacity = 64.;
    media.visible = false;
    media.blend_mode = fanta_gpui::prelude::DesignBlendMode::Multiply;
    if let DesignPaintPayload::Image(image) = &mut media.payload {
        image.placement = placement;
        image.filters = filters;
    }
    let mut paints = vec![untouched.clone(), media];
    let absolute_file = DesignMediaDroppedFile::new(
        std::path::PathBuf::from("/private/tmp/secret/clip.WEBM"),
        DesignMediaFileKind::Webm,
    );
    let mut next_media_source_id = 100;

    assert!(apply_story_media_source_drop(
        &mut paints,
        &"media".into(),
        0,
        &"old-source".into(),
        DesignMediaKind::Image,
        &absolute_file,
        DesignMediaPaintCapabilities::editor(),
        &mut next_media_source_id,
    ));
    assert_eq!(paints.len(), 2);
    assert_eq!(paints[0], untouched);
    assert_eq!(paints[1].id.as_ref(), "media");
    assert_eq!(paints[1].opacity, 64.);
    assert!(!paints[1].visible);
    assert_eq!(
        paints[1].blend_mode,
        fanta_gpui::prelude::DesignBlendMode::Multiply
    );
    let DesignPaintPayload::Video(video) = &paints[1].payload else {
        panic!("WebM replaces Image with a Video payload");
    };
    assert_eq!(video.placement, placement);
    assert_eq!(video.filters, filters);
    assert_eq!(video.source.name.as_ref(), "clip.WEBM");
    assert_eq!(
        video.source.mime_type.as_ref().map(SharedString::as_ref),
        Some("video/webm")
    );
    assert!(!video.source.id.contains("/private/tmp"));
    assert!(video.source.reference.is_none());
    let current_video_source_id = video.source.id.clone();

    let after_video = paints.clone();
    assert!(!apply_story_media_source_drop(
        &mut paints,
        &"media".into(),
        1,
        &"stale-source".into(),
        DesignMediaKind::Video,
        &DesignMediaDroppedFile::new(
            std::path::PathBuf::from("replacement.png"),
            DesignMediaFileKind::Png,
        ),
        DesignMediaPaintCapabilities::editor(),
        &mut next_media_source_id,
    ));
    assert!(!apply_story_media_source_drop(
        &mut paints,
        &"media".into(),
        1,
        &current_video_source_id,
        DesignMediaKind::Video,
        &DesignMediaDroppedFile::new(
            std::path::PathBuf::from("scan.tiff"),
            DesignMediaFileKind::Tiff,
        ),
        DesignMediaPaintCapabilities::editor(),
        &mut next_media_source_id,
    ));
    assert_eq!(paints, after_video);
}

#[test]
fn story_media_drop_allocates_fresh_host_ids_and_rejects_superseded_source_ids() {
    let mut paints = vec![
        DesignPaint::image(DesignPaintSource::new("original-source", "Original")).with_id("media"),
    ];
    let file = DesignMediaDroppedFile::new(
        std::path::PathBuf::from("/private/tmp/private/photo.PNG"),
        DesignMediaFileKind::Png,
    );
    let mut next_media_source_id = 700;

    assert!(apply_story_media_source_drop(
        &mut paints,
        &"media".into(),
        0,
        &"original-source".into(),
        DesignMediaKind::Image,
        &file,
        DesignMediaPaintCapabilities::editor(),
        &mut next_media_source_id,
    ));
    let first_source_id = match &paints[0].payload {
        DesignPaintPayload::Image(image) => image.source.id.clone(),
        _ => panic!("the PNG replacement remains an Image"),
    };

    assert!(apply_story_media_source_drop(
        &mut paints,
        &"media".into(),
        0,
        &first_source_id,
        DesignMediaKind::Image,
        &file,
        DesignMediaPaintCapabilities::editor(),
        &mut next_media_source_id,
    ));
    let second_source_id = match &paints[0].payload {
        DesignPaintPayload::Image(image) => image.source.id.clone(),
        _ => panic!("the second PNG replacement remains an Image"),
    };
    assert_ne!(first_source_id, second_source_id);
    assert_eq!(first_source_id.as_ref(), "storybook-media-source-700");
    assert_eq!(second_source_id.as_ref(), "storybook-media-source-701");
    assert!(!first_source_id.contains("/private/tmp"));
    assert!(!second_source_id.contains("/private/tmp"));

    let after_second_drop = paints.clone();
    assert!(!apply_story_media_source_drop(
        &mut paints,
        &"media".into(),
        0,
        &first_source_id,
        DesignMediaKind::Image,
        &file,
        DesignMediaPaintCapabilities::editor(),
        &mut next_media_source_id,
    ));
    assert_eq!(paints, after_second_drop);
    assert_eq!(
        next_media_source_id, 702,
        "rejected stale drops do not consume host source identities"
    );
}

#[test]
fn story_section_reducer_rejects_stale_disabled_viewer_and_resolved_intents() {
    let mut section = DesignPanelNode::new("section", "Section", DesignPanelNodeKind::Section);
    section
        .section
        .as_mut()
        .expect("Section preset properties")
        .dev_status =
        Some(DesignSectionDevStatus::new(DesignSectionDevStatusKind::ReadyForDev).changed(true));
    let current_target = DesignPanelTarget::Nodes {
        node_ids: vec![section.id.clone()],
    };
    let resolve = DesignPanelAction::SectionResolveChangedStatusRequested {
        node_id: section.id.clone(),
    };

    let viewer_snapshot = section.clone();
    assert!(!apply_story_section_or_transform_action(
        &mut section,
        Some(&current_target),
        false,
        &resolve,
    ));
    assert_eq!(section, viewer_snapshot);

    let stale_target = DesignPanelTarget::Nodes {
        node_ids: vec!["another-node".into()],
    };
    assert!(!apply_story_section_or_transform_action(
        &mut section,
        Some(&stale_target),
        true,
        &resolve,
    ));
    assert_eq!(section, viewer_snapshot);

    let mut disabled = section.clone();
    disabled
        .section
        .as_mut()
        .expect("Section preset properties")
        .capabilities
        .resolve_changed_status = false;
    let disabled_snapshot = disabled.clone();
    assert!(!apply_story_section_or_transform_action(
        &mut disabled,
        Some(&current_target),
        true,
        &resolve,
    ));
    assert_eq!(disabled, disabled_snapshot);

    assert!(apply_story_section_or_transform_action(
        &mut section,
        Some(&current_target),
        true,
        &resolve,
    ));
    assert!(
        !section
            .section
            .as_ref()
            .and_then(|section| section.dev_status.as_ref())
            .expect("resolved status remains present")
            .changed
    );
    let resolved_snapshot = section.clone();
    assert!(!apply_story_section_or_transform_action(
        &mut section,
        Some(&current_target),
        true,
        &resolve,
    ));
    assert_eq!(section, resolved_snapshot);
    assert!(
        story_section_or_transform_action_status(&resolve, false).contains("rejected"),
        "the live reducer exposes rejection instead of claiming a stale mutation succeeded"
    );

    let mut share_disabled = viewer_snapshot;
    share_disabled
        .section
        .as_mut()
        .expect("Section preset properties")
        .capabilities
        .share = false;
    let share = DesignPanelAction::SectionShareRequested {
        node_id: share_disabled.id.clone(),
    };
    assert!(!apply_story_section_or_transform_action(
        &mut share_disabled,
        Some(&current_target),
        false,
        &share,
    ));
}

#[test]
fn story_transform_reducer_requires_exact_index_capability_and_edit_permission() {
    let mut transform = DesignPanelNode::new(
        "transform",
        "Transform group",
        DesignPanelNodeKind::TransformGroup,
    );
    transform
        .transform_modifiers
        .push(DesignRepeatModifier::radial("repeat-1"));
    let current_target = DesignPanelTarget::Nodes {
        node_ids: vec![transform.id.clone()],
    };
    let stale_remove = DesignPanelAction::TransformModifierRemoveRequested {
        node_id: transform.id.clone(),
        modifier_id: transform.transform_modifiers[0].id.clone(),
        index: 0,
    };
    transform.transform_modifiers.swap(0, 1);
    let reordered_snapshot = transform.clone();
    assert!(!apply_story_section_or_transform_action(
        &mut transform,
        Some(&current_target),
        true,
        &stale_remove,
    ));
    assert_eq!(transform, reordered_snapshot);

    let current_modifier_id = transform.transform_modifiers[0].id.clone();
    let change = DesignPanelAction::TransformModifierChangeRequested {
        node_id: transform.id.clone(),
        modifier_id: current_modifier_id.clone(),
        index: 0,
        change: DesignTransformModifierChange::Count(3),
        phase: DesignPanelEditPhase::Commit,
    };
    assert!(!apply_story_section_or_transform_action(
        &mut transform,
        Some(&current_target),
        false,
        &change,
    ));
    assert_eq!(transform, reordered_snapshot);

    for invalid_change in [
        DesignTransformModifierChange::Count(0),
        DesignTransformModifierChange::Offset(f32::NAN),
    ] {
        let invalid = DesignPanelAction::TransformModifierChangeRequested {
            node_id: transform.id.clone(),
            modifier_id: current_modifier_id.clone(),
            index: 0,
            change: invalid_change,
            phase: DesignPanelEditPhase::Commit,
        };
        assert!(!apply_story_section_or_transform_action(
            &mut transform,
            Some(&current_target),
            true,
            &invalid,
        ));
        assert_eq!(transform, reordered_snapshot);
    }

    let mut disabled = transform.clone();
    let mut disabled_capabilities =
        DesignPanelNodeCapabilities::for_node_kind(DesignPanelNodeKind::TransformGroup);
    disabled_capabilities
        .sections
        .retain(|section| *section != DesignPanelSection::Transform);
    disabled.capabilities = Some(disabled_capabilities);
    let disabled_snapshot = disabled.clone();
    let add = DesignPanelAction::TransformModifierAddRequested {
        node_id: disabled.id.clone(),
        repeat_type: DesignRepeatType::Linear,
    };
    assert!(!apply_story_section_or_transform_action(
        &mut disabled,
        Some(&current_target),
        true,
        &add,
    ));
    assert_eq!(disabled, disabled_snapshot);

    assert!(apply_story_section_or_transform_action(
        &mut transform,
        Some(&current_target),
        true,
        &change,
    ));
    assert_eq!(transform.transform_modifiers[0].count, 3);
    let exact_remove = DesignPanelAction::TransformModifierRemoveRequested {
        node_id: transform.id.clone(),
        modifier_id: transform.transform_modifiers[0].id.clone(),
        index: 0,
    };
    assert!(apply_story_section_or_transform_action(
        &mut transform,
        Some(&current_target),
        true,
        &exact_remove,
    ));
}

#[test]
fn design_story_width_parser_drag_and_keyboard_are_continuous_and_clamped() {
    assert_eq!(
        DESIGN_PANEL_WIDTH_PRESETS,
        [320., 400., 472.],
        "the established deterministic presets remain available"
    );
    assert_eq!(parse_design_panel_width(None), 472.);
    assert_eq!(parse_design_panel_width(Some("400")), 400.);
    assert_eq!(parse_design_panel_width(Some("365.5")), 365.5);
    assert_eq!(parse_design_panel_width(Some("120")), 320.);
    assert_eq!(parse_design_panel_width(Some("900")), 640.);
    assert_eq!(parse_design_panel_width(Some("NaN")), 472.);
    assert_eq!(parse_design_panel_width(Some("not-a-width")), 472.);

    let drag = DesignPanelResizeDrag::new(900., 400.);
    assert_eq!(drag.width_at(850.), 450.);
    assert_eq!(drag.width_at(1_200.), 320.);
    assert_eq!(drag.width_at(400.), 640.);

    assert_eq!(
        design_panel_width_after_key(400., "left", false),
        Some(392.)
    );
    assert_eq!(
        design_panel_width_after_key(400., "right", false),
        Some(408.)
    );
    assert_eq!(design_panel_width_after_key(400., "+", true), Some(432.));
    assert_eq!(design_panel_width_after_key(320., "-", false), Some(320.));
    assert_eq!(design_panel_width_after_key(640., "up", false), Some(640.));
    assert_eq!(design_panel_width_after_key(400., "enter", false), None);
}

#[test]
fn design_harness_stacks_below_its_breakpoint_and_caps_the_panel_width() {
    assert!(!design_harness_stacked(1200.));
    assert!(!design_harness_stacked(DESIGN_HARNESS_STACK_BREAKPOINT));
    assert!(design_harness_stacked(DESIGN_HARNESS_STACK_BREAKPOINT - 1.));
    assert!(design_harness_stacked(DESIGN_STORY_MIN_WIDTH));

    // Wide layouts reserve the rail, a canvas sliver, and the handle; the
    // user's width survives whenever that chrome fits.
    assert_eq!(design_harness_panel_width(472., 1200.), 472.);
    assert_eq!(design_harness_panel_width(640., 1200.), 640.);
    assert_eq!(design_harness_panel_width(640., 900.), 480.);

    // Stacked layouts reserve only the resize handle, so the inspector
    // keeps as much of the story width as its own clamp allows.
    assert_eq!(
        design_harness_panel_width(472., DESIGN_STORY_MIN_WIDTH),
        DESIGN_STORY_MIN_WIDTH - DESIGN_HARNESS_RESIZE_HANDLE_WIDTH
    );
    assert_eq!(
        design_harness_panel_width(320., DESIGN_STORY_MIN_WIDTH),
        320.
    );
    // Out-of-range user widths still clamp before the harness cap applies.
    assert_eq!(design_harness_panel_width(9_000., 1200.), 640.);
    assert_eq!(design_harness_panel_width(f32::NAN, 1200.), 472.);
    // The cap never pushes the inspector below its own 320 px minimum;
    // the story floor (panel minimum + handle + margins by definition)
    // keeps that minimum reachable on screen.
    assert_eq!(design_harness_panel_width(472., 300.), 320.);
    assert_eq!(
        StoryKind::Design.descriptor().min_story_size(),
        (DESIGN_STORY_MIN_WIDTH, DESIGN_STORY_MIN_HEIGHT),
        "the Design story floor is the stacked harness minimum"
    );
}

#[test]
fn story_component_authoring_name_sessions_reject_unbalanced_and_restore_cancel() {
    let key = (SharedString::from("component"), SharedString::from("label"));
    let original = SharedString::from("Label");
    let preview = SharedString::from("Primary label");
    let committed = SharedString::from("Action label");
    let mut current = original.clone();
    let mut sessions = HashMap::new();

    assert!(!apply_story_component_authoring_name_edit(
        &mut current,
        &mut sessions,
        key.clone(),
        &original,
        &original,
        &preview,
        DesignPanelEditPhase::Preview,
    ));
    assert!(apply_story_component_authoring_name_edit(
        &mut current,
        &mut sessions,
        key.clone(),
        &original,
        &original,
        &original,
        DesignPanelEditPhase::Begin,
    ));
    assert!(!apply_story_component_authoring_name_edit(
        &mut current,
        &mut sessions,
        key.clone(),
        &original,
        &original,
        &original,
        DesignPanelEditPhase::Begin,
    ));
    assert!(apply_story_component_authoring_name_edit(
        &mut current,
        &mut sessions,
        key.clone(),
        &original,
        &original,
        &preview,
        DesignPanelEditPhase::Preview,
    ));
    assert_eq!(current, preview);
    assert!(!apply_story_component_authoring_name_edit(
        &mut current,
        &mut sessions,
        key.clone(),
        &original,
        &original,
        &committed,
        DesignPanelEditPhase::Commit,
    ));
    assert!(apply_story_component_authoring_name_edit(
        &mut current,
        &mut sessions,
        key.clone(),
        &original,
        &preview,
        &original,
        DesignPanelEditPhase::Cancel,
    ));
    assert_eq!(current, original);
    assert!(sessions.is_empty());
    assert!(!apply_story_component_authoring_name_edit(
        &mut current,
        &mut sessions,
        key,
        &original,
        &original,
        &original,
        DesignPanelEditPhase::Cancel,
    ));
}

#[test]
fn story_component_authoring_reorder_sessions_reject_stale_anchors_and_duplicate_terminals() {
    let key = (
        SharedString::from("component"),
        SharedString::from("content"),
    );
    let original = vec!["label".into(), "icon".into(), "content".into()];
    let mut current = original.clone();
    let mut sessions = HashMap::new();
    let before_label = SharedString::from("label");
    let outside_partition = SharedString::from("variant-state");
    let duplicate_members = vec!["label".into(), "label".into(), "content".into()];

    assert!(
        apply_story_component_authoring_reorder(
            &current,
            &mut sessions,
            key.clone(),
            &"content".into(),
            &duplicate_members,
            &original,
            Some(&before_label),
            DesignPanelEditPhase::Begin,
        )
        .is_none()
    );
    assert!(
        apply_story_component_authoring_reorder(
            &current,
            &mut sessions,
            key.clone(),
            &"content".into(),
            &original,
            &original,
            Some(&before_label),
            DesignPanelEditPhase::Preview,
        )
        .is_none()
    );
    assert_eq!(
        apply_story_component_authoring_reorder(
            &current,
            &mut sessions,
            key.clone(),
            &"content".into(),
            &original,
            &original,
            None,
            DesignPanelEditPhase::Begin,
        ),
        Some(original.clone())
    );
    assert!(
        apply_story_component_authoring_reorder(
            &current,
            &mut sessions,
            key.clone(),
            &"content".into(),
            &original,
            &original,
            None,
            DesignPanelEditPhase::Begin,
        )
        .is_none()
    );
    current = apply_story_component_authoring_reorder(
        &current,
        &mut sessions,
        key.clone(),
        &"content".into(),
        &original,
        &original,
        Some(&before_label),
        DesignPanelEditPhase::Preview,
    )
    .expect("balanced preview");
    assert_eq!(
        current,
        vec![
            SharedString::from("content"),
            SharedString::from("label"),
            SharedString::from("icon"),
        ]
    );
    assert!(
        apply_story_component_authoring_reorder(
            &current,
            &mut sessions,
            key.clone(),
            &"content".into(),
            &original,
            &current,
            Some(&outside_partition),
            DesignPanelEditPhase::Preview,
        )
        .is_none()
    );
    current = apply_story_component_authoring_reorder(
        &current,
        &mut sessions,
        key.clone(),
        &"content".into(),
        &original,
        &current,
        None,
        DesignPanelEditPhase::Cancel,
    )
    .expect("balanced cancel");
    assert_eq!(current, original);
    assert!(sessions.is_empty());
    assert!(
        apply_story_component_authoring_reorder(
            &current,
            &mut sessions,
            key,
            &"content".into(),
            &original,
            &current,
            None,
            DesignPanelEditPhase::Cancel,
        )
        .is_none()
    );
}

#[test]
fn story_component_authoring_create_delete_and_variant_value_guards_are_exact() {
    let node = design_fixtures::seed_design_nodes()
        .into_iter()
        .find(|node| node.kind == DesignPanelNodeKind::Component)
        .expect("component authoring fixture");
    let regular_order = node
        .component_properties
        .iter()
        .filter(|property| {
            DesignComponentPropertyPartition::for_kind(property.definition.kind())
                == DesignComponentPropertyPartition::Regular
        })
        .map(|property| property.id.clone())
        .collect::<Vec<_>>();
    let name = SharedString::from("Supporting text");
    let text_definition = DesignComponentPropertyDefinition::Text {
        default_value: "Details".into(),
        multiline: false,
    };
    assert!(story_component_property_create_is_current(
        &node,
        DesignComponentPropertyKind::Text,
        &name,
        &None,
        &[],
        &text_definition,
        DesignComponentPropertyPartition::Regular,
        &regular_order,
        regular_order.last(),
    ));
    assert!(!story_component_property_create_is_current(
        &node,
        DesignComponentPropertyKind::Boolean,
        &name,
        &None,
        &[],
        &text_definition,
        DesignComponentPropertyPartition::Regular,
        &regular_order,
        regular_order.last(),
    ));
    let mut stale_order = regular_order.clone();
    stale_order.reverse();
    assert!(!story_component_property_create_is_current(
        &node,
        DesignComponentPropertyKind::Text,
        &name,
        &None,
        &[],
        &text_definition,
        DesignComponentPropertyPartition::Regular,
        &stale_order,
        stale_order.last(),
    ));
    assert!(story_component_property_delete_is_current(
        &node, "label", "Label"
    ));
    assert!(!story_component_property_delete_is_current(
        &node,
        "label",
        "Stale label"
    ));

    let state = node
        .component_context
        .as_ref()
        .and_then(|context| context.authoring.as_ref())
        .and_then(|authoring| authoring.definition("state"))
        .expect("Variant authoring");
    let option_order = state
        .variant_options
        .iter()
        .map(|option| option.id.clone())
        .collect::<Vec<_>>();
    assert!(story_component_variant_option_create_is_current(
        &node,
        "state",
        &"Focused".into(),
        &option_order,
        option_order.last(),
    ));
    assert!(story_component_variant_option_delete_is_current(
        &node,
        "state",
        "state-option-1",
        "Hover",
    ));
    assert!(!story_component_variant_option_delete_is_current(
        &node,
        "state",
        "state-option-1",
        "Stale",
    ));
}

#[test]
fn story_component_authoring_combined_slot_edit_is_atomic_and_stale_guarded() {
    let component_catalog = design_fixtures::seed_design_component_swaps();
    let arrow = component_catalog
        .candidates
        .iter()
        .find(|candidate| candidate.reference.id.as_ref() == "icon-arrow-right")
        .expect("arrow component fixture")
        .reference
        .clone();
    let plus = component_catalog
        .candidates
        .iter()
        .find(|candidate| candidate.reference.id.as_ref() == "icon-plus")
        .expect("plus component fixture")
        .reference
        .clone();
    let property_id = SharedString::from("content-slot");
    let old_description = Some(SharedString::from("Original slot guidance"));
    let old_links = vec![DesignDocumentationLink::new(
        "Original slot docs",
        "https://example.com/slot/original",
    )];
    let old_settings = DesignSlotSettings {
        minimum_children: Some(1),
        maximum_children: Some(4),
        preferred_values: vec![arrow.clone()],
        ..DesignSlotSettings::default()
    };
    let old_definition = DesignComponentPropertyDefinition::Slot {
        default_value: DesignSlotValue::default(),
        settings: old_settings,
    };
    let mut property = DesignComponentProperty::slot(
        property_id.clone(),
        "Content",
        DesignSlotValue::default(),
        DesignSlotValue::default(),
        match &old_definition {
            DesignComponentPropertyDefinition::Slot { settings, .. } => settings.clone(),
            _ => unreachable!("test constructs a Slot definition"),
        },
        DesignSlotState::default(),
    )
    .with_description("Original slot guidance");
    property.documentation_links = old_links.clone();
    let mut node = DesignPanelNode::new("slot-main", "Slot main", DesignPanelNodeKind::Component);
    node.component_properties = vec![property];
    node.component_context
        .as_mut()
        .expect("component context")
        .authoring = Some(DesignComponentAuthoringViewData {
        definitions: vec![DesignComponentPropertyDefinitionAuthoring::editable(
            property_id.clone(),
        )],
        ..DesignComponentAuthoringViewData::default()
    });

    let new_description = Some(SharedString::from("Updated slot guidance"));
    let new_links = vec![
        DesignDocumentationLink::new("Original slot docs", "https://example.com/slot/original"),
        DesignDocumentationLink::new("Insertion behavior", "https://example.com/slot/insertion"),
    ];
    let new_definition = DesignComponentPropertyDefinition::Slot {
        default_value: DesignSlotValue::default(),
        settings: DesignSlotSettings {
            stretch_child_on_insert: false,
            display_empty: false,
            minimum_children: None,
            maximum_children: Some(2),
            preferred_values_only: true,
            preferred_values: vec![arrow, plus],
        },
    };

    assert!(apply_story_component_property_definition_edit(
        &mut node,
        &component_catalog,
        &property_id,
        &old_description,
        &new_description,
        &old_links,
        &new_links,
        &old_definition,
        &new_definition,
    ));
    let edited = node
        .component_properties
        .iter()
        .find(|property| property.id == property_id)
        .expect("atomically edited Slot");
    assert_eq!(edited.description, new_description);
    assert_eq!(edited.documentation_links, new_links);
    assert_eq!(edited.definition, new_definition);

    let committed = node.clone();
    let stale_definition = DesignComponentPropertyDefinition::Slot {
        default_value: DesignSlotValue::default(),
        settings: DesignSlotSettings {
            maximum_children: Some(1),
            ..DesignSlotSettings::default()
        },
    };
    assert!(!apply_story_component_property_definition_edit(
        &mut node,
        &component_catalog,
        &property_id,
        &old_description,
        &Some("This must not partially apply".into()),
        &old_links,
        &[DesignDocumentationLink::new(
            "Stale docs",
            "https://example.com/slot/stale",
        )],
        &old_definition,
        &stale_definition,
    ));
    assert_eq!(
        node, committed,
        "a stale combined confirmation must reject metadata and Slot settings together",
    );
}

#[test]
fn story_component_authoring_delete_invalidates_related_reorders_and_reuses_only_free_ids() {
    let node_id = SharedString::from("component");
    let other_node_id = SharedString::from("other-component");
    let deleted_property_id = SharedString::from("label");
    let mut property_reorders = HashMap::from([
        (
            (node_id.clone(), SharedString::from("label")),
            vec!["label".into(), "content".into()],
        ),
        (
            (node_id.clone(), SharedString::from("content")),
            vec!["label".into(), "content".into()],
        ),
        (
            (node_id.clone(), SharedString::from("state")),
            vec!["state".into()],
        ),
        (
            (other_node_id.clone(), SharedString::from("content")),
            vec!["label".into(), "content".into()],
        ),
    ]);
    invalidate_story_component_property_reorders(
        &mut property_reorders,
        &node_id,
        &deleted_property_id,
    );
    assert_eq!(property_reorders.len(), 2);
    assert!(property_reorders.contains_key(&(node_id.clone(), "state".into())));
    assert!(property_reorders.contains_key(&(other_node_id.clone(), "content".into())));

    let property_id = SharedString::from("state");
    let mut option_reorders = HashMap::from([
        (
            (
                node_id.clone(),
                property_id.clone(),
                SharedString::from("state-option-0"),
            ),
            vec!["state-option-0".into(), "state-option-1".into()],
        ),
        (
            (
                node_id.clone(),
                property_id.clone(),
                SharedString::from("state-option-1"),
            ),
            vec!["state-option-0".into(), "state-option-1".into()],
        ),
        (
            (
                node_id.clone(),
                SharedString::from("size"),
                SharedString::from("size-option-0"),
            ),
            vec!["size-option-0".into()],
        ),
        (
            (
                other_node_id.clone(),
                property_id.clone(),
                SharedString::from("state-option-0"),
            ),
            vec!["state-option-0".into()],
        ),
    ]);
    invalidate_story_component_variant_option_reorders(
        &mut option_reorders,
        &node_id,
        &property_id,
    );
    assert_eq!(option_reorders.len(), 2);
    assert!(option_reorders.contains_key(&(
        node_id.clone(),
        "size".into(),
        "size-option-0".into()
    )));
    assert!(option_reorders.contains_key(&(other_node_id, property_id, "state-option-0".into())));

    let mut node = design_fixtures::seed_design_nodes()
        .into_iter()
        .find(|node| node.kind == DesignPanelNodeKind::Component)
        .expect("component authoring fixture");
    node.component_properties.extend([
        DesignComponentProperty::text("storybook-text-0", "Generated zero", "Zero", "Zero"),
        DesignComponentProperty::text("storybook-text-1", "Generated one", "One", "One"),
        DesignComponentProperty::text("storybook-text-2", "Generated two", "Two", "Two"),
    ]);
    node.component_properties
        .retain(|property| property.id.as_ref() != "storybook-text-1");
    let next_property_id =
        next_story_component_property_id(&node, DesignComponentPropertyKind::Text);
    assert_eq!(next_property_id, "storybook-text-1");
    assert!(
        node.component_properties
            .iter()
            .all(|property| property.id != next_property_id),
        "delete-then-create must never collide with a surviving property ID",
    );

    let definition = node
        .component_context
        .as_mut()
        .and_then(|context| context.authoring.as_mut())
        .and_then(|authoring| {
            authoring
                .definitions
                .iter_mut()
                .find(|definition| definition.property_id.as_ref() == "state")
        })
        .expect("Variant authoring");
    definition.variant_options = vec![
        DesignComponentVariantOptionAuthoring::editable("state-option-0", "Default"),
        DesignComponentVariantOptionAuthoring::editable("state-option-1", "Hover"),
        DesignComponentVariantOptionAuthoring::editable("state-option-2", "Pressed"),
    ];
    definition
        .variant_options
        .retain(|option| option.id.as_ref() != "state-option-1");
    let next_option_id = next_story_component_variant_option_id(&node, "state");
    assert_eq!(next_option_id, "state-option-1");
    assert!(
        node.component_context
            .as_ref()
            .and_then(|context| context.authoring.as_ref())
            .and_then(|authoring| authoring.definition("state"))
            .expect("Variant authoring")
            .variant_options
            .iter()
            .all(|option| option.id != next_option_id),
        "delete-then-create must never collide with a surviving Variant-value ID",
    );
}

#[test]
fn story_component_authoring_echo_keeps_partition_and_variant_ids_in_sync() {
    let mut node = design_fixtures::seed_design_nodes()
        .into_iter()
        .find(|node| node.kind == DesignPanelNodeKind::Component)
        .expect("component authoring fixture");
    let variant_ids = node
        .component_properties
        .iter()
        .filter(|property| property.definition.kind() == DesignComponentPropertyKind::Variant)
        .map(|property| property.id.clone())
        .collect::<Vec<_>>();
    let mut regular_ids = node
        .component_properties
        .iter()
        .filter(|property| property.definition.kind() != DesignComponentPropertyKind::Variant)
        .map(|property| property.id.clone())
        .collect::<Vec<_>>();
    regular_ids.reverse();
    let duplicate_regular_ids = vec![regular_ids[0].clone(); regular_ids.len()];
    let before_duplicate_property_order = node.clone();
    assert!(!echo_story_component_property_order(
        &mut node,
        DesignComponentPropertyPartition::Regular,
        &duplicate_regular_ids,
    ));
    assert_eq!(
        node, before_duplicate_property_order,
        "a duplicate property order must be rejected atomically",
    );
    assert!(echo_story_component_property_order(
        &mut node,
        DesignComponentPropertyPartition::Regular,
        &regular_ids,
    ));
    assert_eq!(
        node.component_properties
            .iter()
            .take(variant_ids.len())
            .map(|property| property.id.clone())
            .collect::<Vec<_>>(),
        variant_ids,
        "Variant definitions remain ahead of regular definitions",
    );

    let mut option_ids = node
        .component_context
        .as_ref()
        .and_then(|context| context.authoring.as_ref())
        .and_then(|authoring| authoring.definition("state"))
        .expect("Variant authoring")
        .variant_options
        .iter()
        .map(|option| option.id.clone())
        .collect::<Vec<_>>();
    option_ids.rotate_left(1);
    let duplicate_option_ids = vec![option_ids[0].clone(); option_ids.len()];
    let before_duplicate_option_order = node.clone();
    assert!(!echo_story_component_variant_option_order(
        &mut node,
        "state",
        &duplicate_option_ids,
    ));
    assert_eq!(
        node, before_duplicate_option_order,
        "a duplicate Variant-value order must be rejected atomically",
    );
    assert!(echo_story_component_variant_option_order(
        &mut node,
        "state",
        &option_ids,
    ));
    let authored_names = node
        .component_context
        .as_ref()
        .and_then(|context| context.authoring.as_ref())
        .and_then(|authoring| authoring.definition("state"))
        .expect("Variant authoring")
        .variant_options
        .iter()
        .map(|option| option.name.clone())
        .collect::<Vec<_>>();
    let definition_names = node
        .component_properties
        .iter()
        .find(|property| property.id == "state")
        .and_then(|property| match &property.definition {
            DesignComponentPropertyDefinition::Variant { options, .. } => Some(options.clone()),
            _ => None,
        })
        .expect("Variant definition");
    assert_eq!(authored_names, definition_names);

    let mut mismatched = node.clone();
    let DesignComponentPropertyDefinition::Variant { options, .. } = &mut mismatched
        .component_properties
        .iter_mut()
        .find(|property| property.id == "state")
        .expect("Variant property")
        .definition
    else {
        unreachable!("state remains a Variant");
    };
    options[0] = "Host drift".into();
    let before_mismatched_reorder = mismatched.clone();
    assert!(!echo_story_component_variant_option_order(
        &mut mismatched,
        "state",
        &option_ids,
    ));
    assert_eq!(
        mismatched, before_mismatched_reorder,
        "a mismatched backing Variant must be rejected atomically",
    );
}

#[test]
fn design_story_selector_covers_compatibility_kinds_without_fake_multiple_node() {
    let nodes = design_fixtures::seed_design_nodes();
    let kinds = nodes.iter().map(|node| node.kind).collect::<HashSet<_>>();

    for (index, kind) in DesignPanelNodeKind::ALL.into_iter().enumerate() {
        let canonical_id = match kind {
            DesignPanelNodeKind::Group => "add-auto-layout-group".to_owned(),
            DesignPanelNodeKind::Widget => "reference-widget".to_owned(),
            DesignPanelNodeKind::Other => "design-node-18".to_owned(),
            _ => format!("design-node-{index}"),
        };
        assert!(
            nodes
                .iter()
                .any(|node| node.id.as_ref() == canonical_id.as_str() && node.kind == kind),
            "the canonical selector fixture {canonical_id} must keep its stable identity"
        );
    }

    for kind in DesignPanelNodeKind::COMPATIBILITY_ALL {
        if kind == DesignPanelNodeKind::MultipleSelection {
            assert!(
                !kinds.contains(&kind),
                "multiple selection must remain inspection context, not a fake node"
            );
        } else {
            assert!(
                kinds.contains(&kind),
                "the Story selector is missing the {:?} compatibility fixture",
                kind
            );
        }
    }
    assert!(DesignInspectionScenario::ALL.contains(&DesignInspectionScenario::EditableMultiple));

    for (id, expected_kind) in [
        ("reference-widget", DesignPanelNodeKind::Widget),
        ("reference-image", DesignPanelNodeKind::Image),
        ("reference-video", DesignPanelNodeKind::Video),
        ("reference-arrow", DesignPanelNodeKind::Arrow),
        ("compatibility-mask", DesignPanelNodeKind::Mask),
        ("compatibility-table", DesignPanelNodeKind::Table),
        ("compatibility-pen", DesignPanelNodeKind::Pen),
        ("compatibility-pencil", DesignPanelNodeKind::Pencil),
    ] {
        assert!(
            nodes
                .iter()
                .any(|node| node.id.as_ref() == id && node.kind == expected_kind),
            "missing stable compatibility fixture {id}"
        );
    }

    let unique_ids = nodes
        .iter()
        .map(|node| node.id.as_ref())
        .collect::<HashSet<_>>();
    assert_eq!(
        unique_ids.len(),
        nodes.len(),
        "compatibility fixtures must not destabilize selector identity"
    );
}

#[test]
fn design_story_widget_fixture_is_exact_and_dimensions_are_read_only() {
    let widget = design_fixtures::seed_design_nodes()
        .into_iter()
        .find(|node| node.id.as_ref() == "reference-widget")
        .expect("the Widget reference fixture keeps its stable identity");
    assert_eq!(widget.kind, DesignPanelNodeKind::Widget);
    assert_eq!(
        widget
            .capabilities
            .as_ref()
            .expect("Widget uses an authoritative capability snapshot")
            .sections,
        [
            DesignPanelSection::Position,
            DesignPanelSection::Layout,
            DesignPanelSection::Layer,
            DesignPanelSection::Export,
        ]
    );
    assert!(widget.supports_visibility());
    assert!(widget.supports_position_coordinates());
    assert!(!widget.supports_arrange());
    assert!(!widget.supports_transforms());
    assert!(!widget.supports_aspect_ratio_lock());
    assert!(!widget.supports_auto_layout_child());
    assert!(!widget.supports_add_auto_layout());
    assert!(!widget.supports_fill());
    assert!(!widget.supports_stroke());
    assert!(!widget.supports_layer_appearance());
    assert!(!widget.supports_effects());
    assert!(!widget.supports_constraints());
    assert!(!widget.supports_layout_guides());

    let states = story_widget_dimension_property_states(widget.width, widget.height);
    for (property, expected) in [
        (DesignPanelProperty::Width, widget.width),
        (DesignPanelProperty::Height, widget.height),
    ] {
        let state = states
            .iter()
            .find_map(|(candidate, state)| (*candidate == property).then_some(state))
            .expect("Widget width and height both have explicit Story states");
        assert!(state.is_read_only());
        assert_eq!(state.resolved(), Some(&DesignPanelValue::Number(expected)),);
    }
}

#[test]
fn design_story_permission_selection_scenarios_have_stable_ids_and_launch_values() {
    let scenario_ids = DesignInspectionScenario::ALL
        .map(DesignInspectionScenario::id)
        .into_iter()
        .collect::<HashSet<_>>();
    assert_eq!(
        scenario_ids.len(),
        DesignInspectionScenario::ALL.len(),
        "scenario IDs must remain unique when the selector order changes"
    );
    for scenario in DesignInspectionScenario::ALL {
        assert_eq!(
            DesignInspectionScenario::from_launch_value(scenario.id()),
            scenario,
            "the stable scenario ID must also be its deterministic launch value"
        );
    }

    for (launch_value, expected) in [
        ("viewer-page", DesignInspectionScenario::ViewOnlyPage),
        ("viewer-single", DesignInspectionScenario::ViewOnlySingle),
        (
            "viewer-multiple",
            DesignInspectionScenario::ViewOnlyMultiple,
        ),
        (
            "restricted-viewer-page",
            DesignInspectionScenario::RestrictedPage,
        ),
        (
            "restricted-viewer-single",
            DesignInspectionScenario::RestrictedSingle,
        ),
        (
            "restricted-viewer-multiple",
            DesignInspectionScenario::RestrictedMultiple,
        ),
    ] {
        assert_eq!(
            DesignInspectionScenario::from_launch_value(launch_value),
            expected
        );
    }

    assert_eq!(
        DesignInspectionScenario::Page.label(),
        "Editable page / no selection"
    );
    assert_eq!(
        DesignInspectionScenario::ViewOnlyMultiple.label(),
        "View-only multiple"
    );
    assert_eq!(
        DesignInspectionScenario::RestrictedMultiple.label(),
        "Restricted multiple"
    );
    for (scenario, permissions) in [
        (
            DesignInspectionScenario::ViewOnlyPage,
            DesignPanelPermissions::viewer(),
        ),
        (
            DesignInspectionScenario::ViewOnlySingle,
            DesignPanelPermissions::viewer(),
        ),
        (
            DesignInspectionScenario::ViewOnlyMultiple,
            DesignPanelPermissions::viewer(),
        ),
        (
            DesignInspectionScenario::RestrictedPage,
            DesignPanelPermissions::restricted_viewer(),
        ),
        (
            DesignInspectionScenario::RestrictedSingle,
            DesignPanelPermissions::restricted_viewer(),
        ),
        (
            DesignInspectionScenario::RestrictedMultiple,
            DesignPanelPermissions::restricted_viewer(),
        ),
    ] {
        assert_eq!(scenario.permissions(), permissions);
    }
}

#[test]
fn design_story_viewer_page_and_multiple_contexts_preserve_exact_cardinality() {
    let nodes = design_fixtures::seed_design_nodes();
    let first = nodes
        .iter()
        .find(|node| node.id.as_ref() == "reference-frame")
        .expect("stable first viewer fixture");
    let second = nodes
        .iter()
        .find(|node| node.id != first.id && node.kind == DesignPanelNodeKind::Ellipse)
        .expect("stable second viewer fixture");

    for permissions in [
        DesignPanelPermissions::viewer(),
        DesignPanelPermissions::restricted_viewer(),
    ] {
        let page: DesignPanelInspectionContext = DesignPanelInspectionContext::page(permissions);
        assert_eq!(
            page.selection().kind(),
            fanta_gpui::prelude::DesignPanelSelectionKind::None
        );
        assert_eq!(page.permissions(), permissions);

        let (multiple, property_states) =
            story_multiple_inspection_context(first, second, permissions);
        assert_eq!(
            multiple.selection().kind(),
            fanta_gpui::prelude::DesignPanelSelectionKind::Multiple
        );
        assert_eq!(multiple.permissions(), permissions);
        assert!(!property_states.is_empty());

        let target = story_design_target(&multiple).expect("multiple node target");
        let DesignPanelTarget::Nodes { node_ids } = target else {
            panic!("multiple inspection must retain a node target");
        };
        assert_eq!(node_ids, vec![first.id.clone(), second.id.clone()]);
        assert_ne!(
            node_ids.len(),
            1,
            "viewer projection lookup must take its deliberate aggregate/empty branch"
        );
    }

    for scenario in [
        DesignInspectionScenario::ViewOnlyPage,
        DesignInspectionScenario::RestrictedPage,
        DesignInspectionScenario::ViewOnlyMultiple,
        DesignInspectionScenario::RestrictedMultiple,
    ] {
        assert!(DesignInspectionScenario::ALL.contains(&scenario));
    }
}

#[test]
fn storybook_surface_host_echo_covers_all_surfaces_and_rejects_stale_permissions() {
    let mut editor_surface = DesignPanelSurface::Design;
    assert!(apply_story_surface_change_request(
        &mut editor_surface,
        true,
        DesignPanelSurface::Design,
        DesignPanelSurface::Prototype,
    ));
    assert_eq!(editor_surface, DesignPanelSurface::Prototype);
    assert!(!apply_story_surface_change_request(
        &mut editor_surface,
        true,
        DesignPanelSurface::Design,
        DesignPanelSurface::Prototype,
    ));
    assert!(!apply_story_surface_change_request(
        &mut editor_surface,
        true,
        DesignPanelSurface::Prototype,
        DesignPanelSurface::Comment,
    ));
    assert_eq!(
        editor_surface,
        DesignPanelSurface::Prototype,
        "stale and cross-permission requests must not mutate host state"
    );

    let mut viewer_surface = DesignPanelSurface::Properties;
    assert!(apply_story_surface_change_request(
        &mut viewer_surface,
        false,
        DesignPanelSurface::Properties,
        DesignPanelSurface::Comment,
    ));
    assert_eq!(viewer_surface, DesignPanelSurface::Comment);
    assert!(!apply_story_surface_change_request(
        &mut viewer_surface,
        false,
        DesignPanelSurface::Comment,
        DesignPanelSurface::Design,
    ));
    assert_eq!(viewer_surface, DesignPanelSurface::Comment);
}

#[test]
fn storybook_viewer_copy_fixture_reports_the_exact_host_intent() {
    assert!(DesignInspectionScenario::ALL.contains(&DesignInspectionScenario::ViewOnlySingle));
    assert!(DesignPanelPermissions::viewer().can_copy());
    assert!(!DesignPanelPermissions::restricted_viewer().can_copy());

    let target = DesignPanelTarget::Nodes {
        node_ids: vec!["reference-frame".into()],
    };
    assert_eq!(
        story_property_copy_status(
            &target,
            DesignPanelProperty::Width,
            &SharedString::from("1,024  ◇"),
        )
        .as_ref(),
        "Host copied Width = “1,024  ◇” from Nodes { node_ids: [\"reference-frame\"] }"
    );
}

#[test]
fn storybook_viewer_component_projection_preserves_role_docs_order_and_exact_copy() {
    let nodes = design_fixtures::seed_design_nodes();
    let instance = nodes
        .iter()
        .find(|node| node.kind == DesignPanelNodeKind::Instance)
        .expect("Instance viewer fixture");
    let view =
        design_fixtures::viewer_properties_for_node(instance, DesignViewerColorRepresentation::Css);
    assert!(view.is_valid());
    assert_eq!(
        view.sections
            .iter()
            .take(2)
            .map(|section| section.id.as_ref())
            .collect::<Vec<_>>(),
        ["instance", "layout"],
        "component metadata stays ahead of ordinary viewer properties"
    );

    let section = view.section("instance").expect("Instance projection");
    assert_eq!(section.title.as_ref(), "Instance");
    assert_eq!(section.copy_label.as_ref(), "Copy all");
    assert_eq!(
        section
            .rows
            .iter()
            .map(|row| row.id.as_ref())
            .collect::<Vec<_>>(),
        [
            "role",
            "main-component",
            "main-component-origin",
            "main-component-availability",
            "description",
            "documentation-0",
            "documentation-1",
            "documentation-2",
        ]
    );
    assert_eq!(
        section
            .rows
            .iter()
            .skip(5)
            .map(|row| (row.label.as_ref(), row.displayed_value.as_ref()))
            .collect::<Vec<_>>(),
        [
            (
                "Button instance guidance",
                "https://example.com/components/button/instances",
            ),
            (
                "Override behavior",
                "https://example.com/components/button/overrides",
            ),
            (
                "Primary button API",
                "https://example.com/libraries/product-foundations/primary-button",
            ),
        ],
        "context links precede main-reference links without reordering"
    );
    assert_eq!(
        section.copy_value.as_ref().map(SharedString::as_ref),
        Some(
            "Role: Instance\n\
             Main component: Primary button\n\
             Origin: Library · Product foundations\n\
             Availability: Available\n\
             Description: Button instance with local property and nested override examples.\n\
             Button instance guidance: https://example.com/components/button/instances\n\
             Override behavior: https://example.com/components/button/overrides\n\
             Primary button API: https://example.com/libraries/product-foundations/primary-button"
        )
    );

    for (kind, expected_role) in [
        (
            DesignPanelNodeKind::Component,
            DesignComponentRole::StandaloneMain,
        ),
        (
            DesignPanelNodeKind::ComponentSet,
            DesignComponentRole::ComponentSet,
        ),
        (DesignPanelNodeKind::Instance, DesignComponentRole::Instance),
        (
            DesignPanelNodeKind::Slot,
            DesignComponentRole::SlotDefinition,
        ),
    ] {
        let node = nodes
            .iter()
            .find(|node| node.kind == kind)
            .unwrap_or_else(|| panic!("missing {kind:?} documentation fixture"));
        let context = node
            .component_context
            .as_ref()
            .expect("component context fixture");
        assert_eq!(context.role, expected_role);
        assert!(
            context
                .description
                .as_ref()
                .is_some_and(|value| !value.is_empty())
        );
        assert!(!context.documentation_links.is_empty());
        let section_id = if expected_role.uses_instance_section() {
            "instance"
        } else {
            "component"
        };
        assert!(
            design_fixtures::viewer_properties_for_node(node, DesignViewerColorRepresentation::Css)
                .section(section_id)
                .is_some()
        );
    }

    let component = nodes
        .iter()
        .find(|node| node.kind == DesignPanelNodeKind::Component)
        .expect("Component property documentation fixture");
    let label = component
        .component_properties
        .iter()
        .find(|property| property.id.as_ref() == "label")
        .expect("Label property");
    assert_eq!(
        label.documentation_links,
        [DesignDocumentationLink::new(
            "Button content guidelines",
            "https://example.com/components/button/content",
        )]
    );
}

#[test]
fn storybook_viewer_component_projection_handles_absent_missing_and_unavailable_mains() {
    let project = |id: &str, context: DesignComponentContext| {
        let mut node = DesignPanelNode::new(
            id.to_owned(),
            "Viewer component edge",
            DesignPanelNodeKind::Instance,
        );
        node.component_context = Some(context);
        design_fixtures::viewer_properties_for_node(&node, DesignViewerColorRepresentation::Css)
    };

    let absent = project(
        "viewer-main-absent",
        DesignComponentContext::new(DesignComponentRole::Instance),
    );
    let absent = absent.section("instance").expect("absent-main projection");
    assert_eq!(
        absent
            .row("main-component")
            .expect("main row")
            .displayed_value
            .as_ref(),
        "No main component"
    );
    assert_eq!(
        absent
            .row("main-component-origin")
            .expect("origin row")
            .displayed_value
            .as_ref(),
        "Not available"
    );
    assert_eq!(
        absent
            .row("main-component-availability")
            .expect("availability row")
            .displayed_value
            .as_ref(),
        "Missing"
    );

    let missing = project(
        "viewer-main-missing",
        DesignComponentContext::new(DesignComponentRole::Instance).with_main_component(
            DesignComponentReference::local("missing-main", "Deleted button")
                .with_availability(DesignComponentAvailability::Missing),
        ),
    );
    let missing = missing
        .section("instance")
        .expect("missing-main projection");
    assert_eq!(
        missing
            .row("main-component-origin")
            .expect("origin row")
            .displayed_value
            .as_ref(),
        "Local"
    );
    assert_eq!(
        missing
            .row("main-component-availability")
            .expect("availability row")
            .displayed_value
            .as_ref(),
        "Missing"
    );

    let unavailable = project(
        "viewer-main-unavailable",
        DesignComponentContext::new(DesignComponentRole::Instance).with_main_component(
            DesignComponentReference::remote("disabled-main", "Legacy button", "Archived system")
                .with_availability(DesignComponentAvailability::Unavailable {
                    reason: "Library access is disabled".into(),
                }),
        ),
    );
    let unavailable = unavailable
        .section("instance")
        .expect("unavailable-main projection");
    assert_eq!(
        unavailable
            .row("main-component-origin")
            .expect("origin row")
            .displayed_value
            .as_ref(),
        "Library · Archived system"
    );
    assert_eq!(
        unavailable
            .row("main-component-availability")
            .expect("availability row")
            .displayed_value
            .as_ref(),
        "Unavailable · Library access is disabled"
    );
}

#[test]
fn storybook_viewer_ui3_fixtures_cover_text_content_and_all_border_representations() {
    let nodes = design_fixtures::seed_design_nodes();
    let text = nodes
        .iter()
        .find(|node| node.kind == DesignPanelNodeKind::Text)
        .expect("Text viewer fixture");
    let text_view =
        design_fixtures::viewer_properties_for_node(text, DesignViewerColorRepresentation::Css);
    assert!(
        text_view.section("content").is_some_and(
            |section| section.title.as_ref() == "Content" && section.copy_value.is_some()
        )
    );
    assert!(
        text_view
            .section("typography")
            .is_some_and(
                |section| section.copy_label.as_ref() == "Copy all" && !section.rows.is_empty()
            )
    );

    let frame = nodes
        .iter()
        .find(|node| node.stroke.is_some())
        .expect("Borders viewer fixture");
    for representation in DesignViewerColorRepresentation::ALL {
        let view = design_fixtures::viewer_properties_for_node(frame, representation);
        let borders = view.section("borders").expect("Borders projection");
        assert_eq!(borders.title.as_ref(), "Borders");
        assert_eq!(borders.color_representation, Some(representation));
        assert!(borders.copy_value.is_some());
    }
}

#[test]
fn storybook_min_max_fixtures_cover_owner_child_grid_and_ignored_contexts() {
    let nodes = design_fixtures::seed_design_nodes();
    let fixture = |id: &str| {
        nodes
            .iter()
            .find(|node| node.id.as_ref() == id)
            .unwrap_or_else(|| panic!("missing deterministic min/max fixture {id}"))
    };
    let limits = |node: &DesignPanelNode| {
        let item = &node.layout.as_ref().expect("layout fixture").item;
        (
            item.min_width,
            item.max_width,
            item.min_height,
            item.max_height,
        )
    };

    let owner = fixture("layout-responsive-limits");
    assert_eq!(
        owner.layout.as_ref().expect("owner layout").mode,
        DesignLayoutMode::Vertical
    );
    assert_eq!(
        limits(owner),
        (Some(240.), Some(960.), Some(120.), Some(720.))
    );

    let child = fixture("layout-child-horizontal");
    assert!(DesignInspectionScenario::ALL.contains(&DesignInspectionScenario::AutoLayoutChild));
    assert_eq!(limits(child), (Some(160.), Some(480.), None, Some(96.)));

    let grid_child = fixture("layout-grid-child-placement");
    assert!(DesignInspectionScenario::ALL.contains(&DesignInspectionScenario::GridChild));
    assert_eq!(limits(grid_child), (Some(96.), None, None, Some(320.)));

    let ignored = fixture("layout-child-absolute");
    assert!(DesignInspectionScenario::ALL.contains(&DesignInspectionScenario::AutoLayoutIgnored));
    assert_eq!(
        limits(ignored),
        (Some(120.), Some(640.), Some(44.), Some(240.))
    );
}

#[test]
fn storybook_text_path_fixtures_cover_native_flip_and_opt_in_api_debug_controls() {
    let mut nodes = design_fixtures::seed_design_nodes();
    let native_index = nodes
        .iter()
        .position(|node| {
            node.kind == DesignPanelNodeKind::TextPath && node.id.as_ref() != "text-path-api-debug"
        })
        .expect("native TextPath fixture");
    let native_id = nodes[native_index].id.clone();
    let native = nodes[native_index]
        .text_path
        .expect("native TextPath view data");
    assert_eq!(native.orientation, DesignTextPathOrientation::Default);
    assert!(native.can_flip_orientation);
    assert!(!native.show_start_data_debug_controls);

    assert!(apply_story_text_path_flip_orientation(
        &mut nodes[native_index],
        &native_id,
    ));
    assert_eq!(
        nodes[native_index]
            .text_path
            .expect("flipped TextPath")
            .orientation,
        DesignTextPathOrientation::Flipped
    );
    assert!(!apply_story_text_path_flip_orientation(
        &mut nodes[native_index],
        &"stale-text-path".into(),
    ));

    let debug = nodes
        .iter_mut()
        .find(|node| node.id.as_ref() == "text-path-api-debug")
        .expect("API debug TextPath fixture");
    let debug_id = debug.id.clone();
    let debug_view = debug.text_path.expect("debug TextPath view data");
    assert_eq!(debug_view.orientation, DesignTextPathOrientation::Flipped);
    assert!(!debug_view.can_flip_orientation);
    assert!(debug_view.show_start_data_debug_controls);
    assert!(
        !apply_story_text_path_flip_orientation(debug, &debug_id),
        "the mock host must revalidate the current capability"
    );
}

#[test]
fn storybook_text_max_lines_fixtures_cover_every_native_availability_branch() {
    let nodes = design_fixtures::seed_design_nodes();
    let fixture = |id: &str| {
        nodes
            .iter()
            .find(|node| node.id.as_ref() == id)
            .unwrap_or_else(|| panic!("missing deterministic text fixture {id}"))
    };

    let auto_width = fixture("text-resize-autowidth");
    let typography = auto_width.typography.as_ref().expect("Auto width text");
    assert_eq!(typography.resize, DesignTextResize::AutoWidth);
    assert!(typography.truncate);
    assert_eq!(typography.max_lines, Some(2));
    assert!(auto_width.text_max_lines_are_available(false));

    let auto_height = fixture("text-resize-autoheight");
    let typography = auto_height.typography.as_ref().expect("Auto height text");
    assert_eq!(typography.resize, DesignTextResize::AutoHeight);
    assert!(typography.truncate);
    assert_eq!(typography.max_lines, None);
    assert!(auto_height.text_max_lines_are_available(false));

    let fixed = fixture("text-resize-fixed");
    let typography = fixed.typography.as_ref().expect("Fixed text");
    assert_eq!(typography.resize, DesignTextResize::Fixed);
    assert!(typography.truncate);
    assert_eq!(typography.max_lines, None);
    assert!(!fixed.text_max_lines_are_available(false));

    let vertical_hug = fixture("text-max-lines-vertical-hug");
    assert!(vertical_hug.text_max_lines_are_available(true));
    assert_eq!(
        default_design_inspection_scenario_for_node(vertical_hug),
        DesignInspectionScenario::AutoLayoutVerticalChild
    );
    assert_eq!(
        vertical_hug
            .typography
            .as_ref()
            .expect("vertical Hug text")
            .max_lines,
        Some(4)
    );

    let vertical_fixed = fixture("text-max-lines-vertical-fixed");
    assert!(!vertical_fixed.text_max_lines_are_available(true));
    assert_eq!(
        default_design_inspection_scenario_for_node(vertical_fixed),
        DesignInspectionScenario::AutoLayoutVerticalChild
    );
    assert_eq!(
        vertical_fixed
            .typography
            .as_ref()
            .expect("vertical Fixed text")
            .max_lines,
        None
    );

    let max_height = fixture("text-max-lines-max-height");
    assert_eq!(
        max_height
            .layout
            .as_ref()
            .expect("Max-height text layout")
            .item
            .max_height,
        Some(96.)
    );
    assert_eq!(
        max_height
            .typography
            .as_ref()
            .expect("Max-height text typography")
            .max_lines,
        None
    );
}

#[test]
fn storybook_text_height_limit_reducer_canonicalizes_atomically() {
    let mut node = design_fixtures::seed_design_nodes()
        .into_iter()
        .find(|node| node.id.as_ref() == "text-max-lines-vertical-hug")
        .expect("vertical Hug fixture");
    node.layout.as_mut().expect("text layout").item.max_height = Some(96.);

    apply_design_property_with_parent(
        &mut node,
        DesignPanelProperty::TextMaxLines,
        &DesignPanelValue::OptionalNumber(Some(3.)),
        true,
    );
    assert_eq!(
        node.typography.as_ref().expect("text typography").max_lines,
        Some(3)
    );
    assert_eq!(
        node.layout.as_ref().expect("text layout").item.max_height,
        None
    );

    apply_design_property_with_parent(
        &mut node,
        DesignPanelProperty::MaxHeight,
        &DesignPanelValue::OptionalNumber(Some(120.)),
        true,
    );
    assert_eq!(
        node.layout.as_ref().expect("text layout").item.max_height,
        Some(120.)
    );
    assert_eq!(
        node.typography.as_ref().expect("text typography").max_lines,
        None
    );

    apply_design_property_with_parent(
        &mut node,
        DesignPanelProperty::TextMaxLines,
        &DesignPanelValue::OptionalNumber(Some(5.)),
        true,
    );
    assert_eq!(
        node.typography.as_ref().expect("text typography").max_lines,
        Some(5)
    );
    apply_design_property_with_parent(
        &mut node,
        DesignPanelProperty::VerticalSizing,
        &DesignPanelValue::SizingMode(DesignSizingMode::Fixed),
        true,
    );
    assert_eq!(
        node.typography.as_ref().expect("text typography").max_lines,
        None,
        "losing vertical Hug restores the native Auto value"
    );
    apply_design_property_with_parent(
        &mut node,
        DesignPanelProperty::TextMaxLines,
        &DesignPanelValue::OptionalNumber(Some(2.)),
        true,
    );
    assert_eq!(
        node.typography.as_ref().expect("text typography").max_lines,
        None,
        "the reducer rejects numeric Max lines while unavailable"
    );

    apply_design_property_with_parent(
        &mut node,
        DesignPanelProperty::VerticalSizing,
        &DesignPanelValue::SizingMode(DesignSizingMode::Hug),
        true,
    );
    apply_design_property_with_parent(
        &mut node,
        DesignPanelProperty::TextMaxLines,
        &DesignPanelValue::OptionalNumber(Some(2.)),
        true,
    );
    apply_design_property_with_parent(
        &mut node,
        DesignPanelProperty::TextTruncate,
        &DesignPanelValue::Bool(false),
        true,
    );
    assert_eq!(
        node.typography.as_ref().expect("text typography").max_lines,
        None,
        "disabling ending truncation restores Max lines to Auto"
    );

    apply_design_property_with_parent(
        &mut node,
        DesignPanelProperty::TextTruncate,
        &DesignPanelValue::Bool(true),
        true,
    );
    apply_design_property_with_parent(
        &mut node,
        DesignPanelProperty::TextMaxLines,
        &DesignPanelValue::OptionalNumber(Some(3.)),
        true,
    );
    apply_design_property_with_parent(
        &mut node,
        DesignPanelProperty::TextResize,
        &DesignPanelValue::TextResize(DesignTextResize::Fixed),
        true,
    );
    assert_eq!(
        node.typography.as_ref().expect("text typography").max_lines,
        None,
        "Fixed size restores Max lines to Auto"
    );
}

#[test]
fn storybook_text_edit_scenario_exposes_an_exact_initial_range_revision() {
    let context = story_text_edit_inspection_context(
        DesignPanelNode::new("text", "Text", DesignPanelNodeKind::Text),
        0,
    );

    assert_eq!(context.edit_mode(), DesignPanelEditMode::Text);
    assert_eq!(context.text_range_revision(), Some(0));
}

#[test]
fn story_style_browser_catalogs_cover_page_library_and_importable_rows() {
    let paints = design_fixtures::seed_design_paint_styles();
    assert!(!paints.page_styles.is_empty());
    assert!(paints.libraries.iter().any(|library| {
        library.name.as_ref() == "Fanta foundations"
            && library
                .styles
                .iter()
                .any(|style| style.import_state == DesignPaintStyleImportState::Available)
    }));

    let effects = design_fixtures::seed_design_effect_styles();
    assert!(!effects.page_styles.is_empty());
    assert!(
        effects
            .libraries
            .iter()
            .any(|library| library.name.as_ref() == "Fanta effects" && !library.styles.is_empty())
    );

    let grids = design_fixtures::seed_design_layout_grid_styles();
    assert!(!grids.page_styles.is_empty());
    assert!(grids.libraries.iter().any(|library| {
        library.name.as_ref() == "Fanta layout grids"
            && library
                .styles
                .iter()
                .any(|style| style.import_state == DesignLayoutGridStyleImportState::Available)
    }));
}

#[test]
fn story_typography_variables_cover_family_style_and_numeric_weight() {
    let variables = design_fixtures::seed_design_property_variables();
    for (id, resolved_type, scope) in [
        (
            "font-body-family",
            DesignVariableResolvedType::String,
            DesignVariableScope::FontFamily,
        ),
        (
            "font-body-style",
            DesignVariableResolvedType::String,
            DesignVariableScope::FontStyle,
        ),
        (
            "font-body-weight",
            DesignVariableResolvedType::Float,
            DesignVariableScope::FontWeight,
        ),
        (
            "font-display-weight",
            DesignVariableResolvedType::Float,
            DesignVariableScope::FontWeight,
        ),
    ] {
        let variable = variables.variable(id).expect("typography variable fixture");
        assert_eq!(variable.resolved_type, resolved_type);
        assert!(variable.scopes.contains(&scope));
    }
    assert_eq!(
        variables
            .variable("font-display-weight")
            .expect("importable weight")
            .import_state,
        DesignVariableImportState::Available
    );

    let mut node = DesignPanelNode::new("text", "Text", DesignPanelNodeKind::Text);
    apply_design_property(
        &mut node,
        DesignPanelProperty::FontWeight,
        &DesignPanelValue::Number(650.),
    );
    let typography = node.typography.as_ref().expect("text typography");
    assert_eq!(typography.weight, 650.);
    assert_eq!(typography.style.as_ref(), "Regular");

    apply_design_property(
        &mut node,
        DesignPanelProperty::FontWeight,
        &DesignPanelValue::Number(1200.),
    );
    assert_eq!(
        node.typography.as_ref().expect("text typography").weight,
        1000.
    );
    let target = node
        .property_variable_target(DesignPanelProperty::FontWeight)
        .expect("fontWeight variable target");
    assert_eq!(target.fields[0].api_name(), "fontWeight");
    assert_eq!(target.resolved_type, DesignVariableResolvedType::Float);
    assert_eq!(
        target.compatible_scope,
        Some(DesignVariableScope::FontWeight)
    );
    let fonts = design_fixtures::seed_design_fonts();
    let medium = fonts
        .families
        .iter()
        .flat_map(|family| &family.styles)
        .find(|style| style.name.as_ref() == "Medium")
        .expect("weighted font catalog fixture");
    assert_eq!(medium.weight, Some(500));
}

#[test]
fn storybook_plugin_header_exercises_menu_and_viewer_safe_controls() {
    let [menu, direct] = storybook_plugin_header_controls();

    assert!(menu.is_menu());
    assert!(menu.command().is_none());
    assert_eq!(menu.menu_items.len(), 2);
    assert!(!menu.menu_items[0].effective_access().requires_edit());
    assert!(menu.menu_items[1].effective_access().requires_edit());
    assert!(!direct.is_menu());
    assert!(!direct.effective_access().requires_edit());
}

#[test]
fn story_fill_shader_fixture_covers_every_complex_property_editor() {
    let shaders = design_fixtures::seed_design_shaders();
    let definition = shaders
        .definition("shader:page:fractal-noise")
        .expect("Fractal noise catalog fixture");
    let nodes = design_fixtures::seed_design_nodes();
    let paint = nodes
        .iter()
        .find(|node| node.id.as_ref() == "paint-host-opaque")
        .and_then(|node| {
            node.fills
                .iter()
                .find(|paint| paint.id.as_ref() == "shader-fill")
        })
        .expect("fill Shader fixture");
    let DesignPaintPayload::Shader(shader) = &paint.payload else {
        panic!("the fill Shader fixture must retain its Shader payload");
    };
    let expected = [
        ("def:center", DesignShaderPropertyKind::Point),
        ("def:ray", DesignShaderPropertyKind::Line),
        ("def:lens", DesignShaderPropertyKind::Circle),
        ("def:orbit", DesignShaderPropertyKind::CirclePoint),
        ("def:color-point", DesignShaderPropertyKind::ColorPoint),
        ("def:gradient", DesignShaderPropertyKind::Gradient),
        ("def:future", DesignShaderPropertyKind::Unsupported),
    ];

    for (definition_id, kind) in expected {
        assert_eq!(
            definition
                .property(definition_id)
                .map(|property| property.kind),
            Some(kind),
            "{definition_id} must remain reachable through the fill Shader catalog"
        );
        assert!(
            shader
                .property(definition_id)
                .is_some_and(|value| value.is_compatible_with(kind)),
            "{definition_id} must have a compatible assignment in the selected fill fixture"
        );
    }
    assert_eq!(
        shader.properties.len(),
        definition.property_definitions.len(),
        "the selected fill must be derived from every catalog default"
    );
}

#[test]
fn story_fill_shader_complex_property_editor_applies_deterministic_sample_edits() {
    let vector = fanta_gpui::design::DesignEffectVector::new;
    let mut node = design_fixtures::seed_design_nodes()
        .into_iter()
        .find(|node| node.id.as_ref() == "paint-host-opaque")
        .expect("fill Shader fixture");
    node.fills.swap(0, 1);
    let expected = [
        (
            "def:center",
            DesignShaderPropertyValue::Point(vector(0.25, 0.75)),
        ),
        (
            "def:ray",
            DesignShaderPropertyValue::Line {
                start: vector(0.2, 0.8),
                end: vector(0.8, 0.2),
            },
        ),
        (
            "def:lens",
            DesignShaderPropertyValue::Circle {
                center: vector(0.4, 0.6),
                radius: 0.42,
            },
        ),
        (
            "def:orbit",
            DesignShaderPropertyValue::CirclePoint {
                center: vector(0.45, 0.55),
                radius: 0.38,
                angle: 135.,
            },
        ),
        (
            "def:color-point",
            DesignShaderPropertyValue::ColorPoint {
                point: vector(0.65, 0.35),
                color: DesignColor::PURPLE,
                variable_id: None,
            },
        ),
        (
            "def:gradient",
            DesignShaderPropertyValue::Gradient(vec![
                fanta_gpui::design::DesignShaderGradientStop::new(0., DesignColor::BLUE),
                fanta_gpui::design::DesignShaderGradientStop::new(0.5, DesignColor::WHITE),
                fanta_gpui::design::DesignShaderGradientStop::new(1., DesignColor::PURPLE),
            ]),
        ),
        (
            "def:future",
            DesignShaderPropertyValue::Opaque {
                type_name: "MESH_PATCH".into(),
                payload: "{\"storybookEdited\":true}".into(),
            },
        ),
    ];

    for (definition_id, expected_value) in expected {
        let definition_id = SharedString::from(definition_id);
        let before = node
            .fills
            .iter()
            .find(|paint| paint.id.as_ref() == "shader-fill")
            .and_then(|paint| match &paint.payload {
                DesignPaintPayload::Shader(shader) => shader.property(&definition_id).cloned(),
                _ => None,
            })
            .expect("complex fill Shader assignment");
        assert_ne!(before, expected_value);
        assert!(
            apply_story_paint_shader_property_editor(
                &mut node,
                DesignPanelCollection::Fill,
                &"shader-fill".into(),
                0,
                &definition_id,
            ),
            "{definition_id} must resolve by stable paint identity after reorder"
        );
        let after = node
            .fills
            .iter()
            .find(|paint| paint.id.as_ref() == "shader-fill")
            .and_then(|paint| match &paint.payload {
                DesignPaintPayload::Shader(shader) => shader.property(&definition_id),
                _ => None,
            })
            .expect("edited complex fill Shader assignment");
        assert_eq!(
            after, &expected_value,
            "{definition_id} must apply a visible deterministic Story host edit"
        );
    }
}

#[test]
fn story_effect_shader_fixture_covers_every_typed_property_editor() {
    let nodes = design_fixtures::seed_design_nodes();
    let effects = nodes
        .iter()
        .find(|node| node.id.as_ref() == "effects-all")
        .expect("all-effects fixture");
    let shader = effects
        .effects
        .iter()
        .find_map(|effect| match &effect.settings {
            DesignEffectSettings::Shader(shader) => Some(shader),
            _ => None,
        })
        .expect("Shader effect fixture");
    let kinds = shader
        .properties
        .iter()
        .map(|property| property.kind)
        .collect::<HashSet<_>>();
    assert_eq!(
        kinds,
        DesignShaderPropertyKind::ALL.into_iter().collect(),
        "the Storybook must keep every Shader-property renderer reachable"
    );
    assert!(shader.properties.iter().any(|property| {
        matches!(
            &property.value,
            DesignShaderPropertyValue::VariableAlias { variable_id }
                if variable_id.as_ref() == "variable:shader:amount"
        )
    }));
    assert!(shader.properties.iter().any(|property| {
        matches!(
            &property.value,
            DesignShaderPropertyValue::Opaque { type_name, payload }
                if type_name.as_ref() == "MESH_PATCH"
                    && payload.as_ref().contains("future")
        )
    }));
    assert!(shader.properties.iter().any(|property| {
        matches!(
            &property.value,
            DesignShaderPropertyValue::Gradient(stops)
                if stops.len() == 3
                    && stops[1]
                        .variable_id
                        .as_ref()
                        .map(SharedString::as_ref)
                        == Some("variable:brand:accent")
        )
    }));
}

#[test]
fn story_effect_shader_resource_reducer_resolves_stable_ids_after_reorder() {
    let alias = DesignShaderProperty::new(
        "alias-definition",
        "Alias",
        DesignShaderPropertyKind::Number,
        DesignShaderPropertyValue::VariableAlias {
            variable_id: "variable:amount".into(),
        },
    );
    let image = DesignShaderProperty::new(
        "image-definition",
        "Image",
        DesignShaderPropertyKind::Image,
        DesignShaderPropertyValue::AssetId("image:current".into()),
    );
    let shader = DesignEffect::from_settings(
        true,
        DesignEffectSettings::Shader(DesignShaderEffect::new(
            "shader-definition",
            "Shader",
            [alias, image],
        )),
    )
    .with_id("shader-effect");
    let mut node = DesignPanelNode::new("shader-node", "Shader", DesignPanelNodeKind::Rectangle);
    node.effects = vec![
        DesignEffect::new(DesignEffectKind::DropShadow).with_id("shadow-effect"),
        shader,
    ];

    assert!(apply_story_effect_shader_property_editor(
        &mut node,
        &"shader-effect".into(),
        0,
        &"image-definition".into(),
        0,
        DesignShaderPropertyKind::Image,
        DesignShaderPropertyEditorTarget::Value,
        DesignShaderPropertyEditorKind::Resource,
        &DesignShaderPropertyValue::AssetId("image:current".into()),
    ));
    let image_property = story_effect_shader_property_mut(
        &mut node,
        &"shader-effect".into(),
        0,
        &"image-definition".into(),
        0,
    )
    .expect("stable image property");
    assert_eq!(
        image_property.value,
        DesignShaderPropertyValue::AssetId("image:storybook-replacement".into())
    );

    assert!(apply_story_effect_shader_variable_detach(
        &mut node,
        &"shader-effect".into(),
        0,
        &"alias-definition".into(),
        1,
        DesignShaderPropertyEditorTarget::Value,
        &"variable:amount".into(),
    ));
    let alias_property = story_effect_shader_property_mut(
        &mut node,
        &"shader-effect".into(),
        0,
        &"alias-definition".into(),
        1,
    )
    .expect("stable alias property");
    assert_eq!(alias_property.value, DesignShaderPropertyValue::Number(0.));
}

#[test]
fn story_frame_preset_matrix_covers_figma_groups_and_reducer_revalidates_identity() {
    let mut nodes = design_fixtures::seed_design_nodes();
    let catalogs = design_fixtures::seed_design_frame_presets(&nodes);
    let frame_id = nodes
        .iter()
        .find(|node| node.kind == DesignPanelNodeKind::Frame)
        .map(|node| node.id.clone())
        .expect("Frame fixture");
    let catalog = catalogs.get(&frame_id).expect("Frame catalog");
    assert_eq!(
        catalog
            .groups
            .iter()
            .map(|group| group.label.as_ref())
            .collect::<Vec<_>>(),
        [
            "Phone",
            "Tablet",
            "Desktop",
            "Presentation",
            "Watch",
            "Paper",
            "Social Media",
            "Figma Community",
            "Archive",
        ]
    );

    let selection = DesignFramePresetSelection::new("phone", "iphone-14-15-pro");
    assert!(apply_story_frame_preset(
        &mut nodes, &catalogs, &frame_id, &selection, 393., 852.,
    ));
    let frame = nodes
        .iter()
        .find(|node| node.id == frame_id)
        .expect("resized Frame");
    assert_eq!((frame.width, frame.height), (393., 852.));

    assert!(!apply_story_frame_preset(
        &mut nodes, &catalogs, &frame_id, &selection, 393., 851.,
    ));
    assert!(!apply_story_frame_preset(
        &mut nodes,
        &catalogs,
        &frame_id,
        &DesignFramePresetSelection::new("phone", "android-small"),
        360.,
        800.,
    ));
    let frame = nodes
        .iter()
        .find(|node| node.id == frame_id)
        .expect("Frame stays present");
    assert_eq!((frame.width, frame.height), (393., 852.));
}

#[test]
fn story_component_fixtures_cover_bindings_nested_origins_and_swap_import_states() {
    let nodes = design_fixtures::seed_design_nodes();
    let main = nodes
        .iter()
        .find(|node| node.kind == DesignPanelNodeKind::Component)
        .expect("main component fixture");
    let default_binding = main
        .component_properties
        .iter()
        .find(|property| property.id.as_ref() == "show-icon")
        .and_then(|property| property.default_value_binding.as_ref())
        .expect("definition default binding");
    assert_eq!(default_binding.variable_id.as_ref(), "layer-visible");

    let instance = nodes
        .iter()
        .find(|node| node.kind == DesignPanelNodeKind::Instance)
        .expect("instance fixture");
    let label = instance
        .component_properties
        .iter()
        .find(|property| property.id.as_ref() == "label")
        .expect("multiline label fixture");
    assert!(matches!(
        label.definition,
        fanta_gpui::prelude::DesignComponentPropertyDefinition::Text {
            multiline: true,
            ..
        }
    ));
    assert!(
        instance
            .component_properties
            .iter()
            .any(|property| matches!(
                &property.origin,
                DesignComponentPropertyOrigin::NestedInstance {
                    instance_id,
                    main_component: Some(_),
                    ..
                } if instance_id.as_ref() == "nested-status-badge"
            ))
    );
    assert!(
        instance
            .component_properties
            .iter()
            .find(|property| property.id.as_ref() == "show-icon")
            .is_some_and(|property| property.resolved_value_binding.is_some())
    );

    let swaps = design_fixtures::seed_design_component_swaps();
    assert!(swaps.candidates.iter().any(|candidate| {
        candidate.asset_kind == DesignComponentAssetKind::ComponentSet
            && candidate.import_state == DesignComponentImportState::Local
    }));
    assert!(
        swaps
            .candidates
            .iter()
            .any(DesignComponentSwapCandidate::can_import)
    );
    assert!(swaps.candidates.iter().any(|candidate| {
        candidate.import_state == DesignComponentImportState::Imported && candidate.can_apply()
    }));
}

#[test]
fn homogeneous_multiple_scenario_projects_only_common_fill_and_stroke_collections() {
    assert_eq!(
        DesignInspectionScenario::from_launch_value("homogeneous-multiple"),
        DesignInspectionScenario::HomogeneousMultiple
    );
    assert!(DesignInspectionScenario::ALL.contains(&DesignInspectionScenario::HomogeneousMultiple));

    let nodes = design_fixtures::seed_design_nodes();
    let first = nodes
        .iter()
        .find(|node| node.id.as_ref() == STORY_HOMOGENEOUS_MULTIPLE_NODE_IDS[0])
        .expect("first homogeneous multiple fixture");
    let second = nodes
        .iter()
        .find(|node| node.id.as_ref() == STORY_HOMOGENEOUS_MULTIPLE_NODE_IDS[1])
        .expect("second homogeneous multiple fixture");
    assert_ne!(first.fills[0].id, second.fills[0].id);
    assert_ne!(
        first.stroke.as_ref().expect("first stroke").paints[0].id,
        second.stroke.as_ref().expect("second stroke").paints[0].id,
    );

    let (aggregate, states) = aggregate_story_multiple_selection(first, second);
    assert!(story_paint_collections_are_semantically_equal(
        &aggregate.fills,
        &first.fills
    ));
    assert!(story_paint_collections_are_semantically_equal(
        &aggregate.fills,
        &second.fills
    ));
    assert_eq!(
        aggregate
            .fills
            .iter()
            .map(|paint| paint.id.as_ref())
            .collect::<Vec<_>>(),
        first
            .fills
            .iter()
            .map(|paint| paint.id.as_ref())
            .collect::<Vec<_>>(),
        "the aggregate uses the first selected node only for common row identities"
    );
    assert!(story_strokes_are_semantically_equal(
        aggregate.stroke.as_ref(),
        first.stroke.as_ref(),
    ));
    assert!(story_strokes_are_semantically_equal(
        aggregate.stroke.as_ref(),
        second.stroke.as_ref(),
    ));
    assert!(aggregate.selection_color_aggregate.colors.is_empty());
    assert!(
        !aggregate
            .capabilities
            .as_ref()
            .expect("authoritative aggregate capabilities")
            .sections
            .contains(&DesignPanelSection::Selection),
        "common Fill and Stroke rows must not be duplicated under Selection colors"
    );
    assert!(aggregate.effects.is_empty());
    assert!(aggregate.layout.is_none());
    assert_eq!(aggregate.shape_geometry, DesignShapeGeometry::None);
    assert!(
        states.iter().any(|(property, state)| {
            *property == DesignPanelProperty::Width && state.is_mixed()
        })
    );
}

#[test]
fn targeted_common_paint_edit_rejects_reordered_and_stale_members_atomically() {
    let [first, second] = story_homogeneous_multiple_nodes();
    let exact = DesignPanelTarget::Nodes {
        node_ids: vec![first.id.clone(), second.id.clone()],
    };
    let reordered = DesignPanelTarget::Nodes {
        node_ids: vec![second.id.clone(), first.id.clone()],
    };
    let edit = DesignPaintEdit {
        property: DesignPaintProperty::Color,
        value: DesignPaintValue::Color(DesignColor::WHITE),
    };
    let paint_id = first.fills[0].id.clone();
    let mut nodes = vec![first.clone(), second.clone()];
    let before = nodes.clone();
    let mut snapshots = HashMap::new();

    assert!(!apply_story_targeted_common_paint_edit(
        &mut nodes,
        &mut snapshots,
        Some(&reordered),
        &exact,
        true,
        DesignPanelCollection::Fill,
        DesignPaintTarget::WholeLayer,
        &paint_id,
        0,
        &edit,
        DesignPanelEditPhase::Commit,
    ));
    assert_eq!(nodes, before);
    assert!(snapshots.is_empty());

    let _ = nodes[1].fills[0].apply_edit(&DesignPaintEdit {
        property: DesignPaintProperty::Color,
        value: DesignPaintValue::Color(DesignColor::BLACK),
    });
    let stale = nodes.clone();
    assert!(!apply_story_targeted_common_paint_edit(
        &mut nodes,
        &mut snapshots,
        Some(&exact),
        &exact,
        true,
        DesignPanelCollection::Fill,
        DesignPaintTarget::WholeLayer,
        &paint_id,
        0,
        &edit,
        DesignPanelEditPhase::Commit,
    ));
    assert_eq!(
        nodes, stale,
        "a stale later member must not leave the first member edited"
    );
    assert!(snapshots.is_empty());
}

#[test]
fn targeted_common_fill_and_stroke_edits_mutate_every_exact_member() {
    let [first, second] = story_homogeneous_multiple_nodes();
    let target = DesignPanelTarget::Nodes {
        node_ids: vec![first.id.clone(), second.id.clone()],
    };
    let fill_id = first.fills[0].id.clone();
    let stroke_id = first
        .stroke
        .as_ref()
        .expect("first homogeneous stroke")
        .paints[0]
        .id
        .clone();
    let unselected =
        DesignPanelNode::new("unselected", "Unselected", DesignPanelNodeKind::Rectangle);
    let mut nodes = vec![second, unselected.clone(), first];
    let mut snapshots = HashMap::new();
    let fill_color = DesignColor::rgb(0x21, 0xc5, 0x87);

    assert!(apply_story_targeted_common_paint_edit(
        &mut nodes,
        &mut snapshots,
        Some(&target),
        &target,
        true,
        DesignPanelCollection::Fill,
        DesignPaintTarget::WholeLayer,
        &fill_id,
        1,
        &DesignPaintEdit {
            property: DesignPaintProperty::Color,
            value: DesignPaintValue::Color(fill_color),
        },
        DesignPanelEditPhase::Commit,
    ));
    assert!(apply_story_targeted_common_paint_edit(
        &mut nodes,
        &mut snapshots,
        Some(&target),
        &target,
        true,
        DesignPanelCollection::Stroke,
        DesignPaintTarget::WholeLayer,
        &stroke_id,
        7,
        &DesignPaintEdit {
            property: DesignPaintProperty::Opacity,
            value: DesignPaintValue::Number(42.),
        },
        DesignPanelEditPhase::Commit,
    ));

    let DesignPanelTarget::Nodes { node_ids } = &target else {
        panic!("the homogeneous fixture target is node-based");
    };
    for node_id in node_ids {
        let node = nodes
            .iter()
            .find(|node| node.id == *node_id)
            .expect("exact selected member");
        assert!(matches!(
            &node.fills[0].payload,
            DesignPaintPayload::Solid(solid) if solid.color == fill_color
        ));
        assert_eq!(
            node.stroke.as_ref().expect("common stroke").paints[0].opacity,
            42.
        );
    }
    assert_eq!(
        nodes
            .iter()
            .find(|node| node.id.as_ref() == "unselected")
            .expect("unselected node"),
        &unselected
    );
    assert!(snapshots.is_empty());
}

#[test]
fn targeted_story_reducer_expands_the_exact_ordered_selection_atomically() {
    let mut nodes = vec![
        DesignPanelNode::new("second", "Second", DesignPanelNodeKind::Rectangle),
        DesignPanelNode::new("first", "First", DesignPanelNodeKind::Rectangle),
        DesignPanelNode::new("unselected", "Unselected", DesignPanelNodeKind::Rectangle),
    ];
    let target = DesignPanelTarget::Nodes {
        node_ids: vec!["first".into(), "second".into()],
    };
    let leaf = DesignPanelAction::PropertyChangeRequested {
        node_id: "aggregate-first".into(),
        property: DesignPanelProperty::Width,
        value: DesignPanelValue::Number(640.),
    };
    let expanded = story_targeted_node_actions(&nodes, Some(&target), &target, true, &leaf)
        .expect("every exact selected ID exists");

    assert!(matches!(
        expanded.as_slice(),
        [
            DesignPanelAction::PropertyChangeRequested { node_id: first, .. },
            DesignPanelAction::PropertyChangeRequested { node_id: second, .. },
        ] if first.as_ref() == "first" && second.as_ref() == "second"
    ));
    for action in &expanded {
        let DesignPanelAction::PropertyChangeRequested {
            node_id,
            property,
            value,
        } = action
        else {
            panic!("the expansion must preserve the leaf action");
        };
        let node = nodes
            .iter_mut()
            .find(|node| node.id == *node_id)
            .expect("prevalidated target");
        apply_design_property(node, *property, value);
    }
    assert_eq!(
        nodes
            .iter()
            .find(|node| node.id.as_ref() == "first")
            .map(|node| node.width),
        Some(640.)
    );
    assert_eq!(
        nodes
            .iter()
            .find(|node| node.id.as_ref() == "second")
            .map(|node| node.width),
        Some(640.)
    );
    assert_ne!(
        nodes
            .iter()
            .find(|node| node.id.as_ref() == "unselected")
            .map(|node| node.width),
        Some(640.)
    );

    let stale_target = DesignPanelTarget::Nodes {
        node_ids: vec!["first".into(), "removed".into()],
    };
    assert!(
        story_targeted_node_actions(&nodes, Some(&stale_target), &stale_target, true, &leaf)
            .is_none(),
        "a missing member rejects the complete target before any reducer runs"
    );
}

#[test]
fn targeted_story_reducer_rejects_non_exact_non_unique_read_only_and_inapplicable_targets() {
    let supported_nodes = vec![
        DesignPanelNode::new("first", "First", DesignPanelNodeKind::Rectangle),
        DesignPanelNode::new("second", "Second", DesignPanelNodeKind::Rectangle),
    ];
    let mut unsupported = DesignPanelNode::new("second", "Second", DesignPanelNodeKind::Rectangle);
    let mut unsupported_capabilities =
        DesignPanelNodeCapabilities::for_node_kind(DesignPanelNodeKind::Rectangle);
    unsupported_capabilities.dimensions = false;
    unsupported_capabilities
        .sections
        .retain(|section| *section != DesignPanelSection::Layout);
    unsupported.capabilities = Some(unsupported_capabilities);
    let unsupported_nodes = vec![
        DesignPanelNode::new("first", "First", DesignPanelNodeKind::Rectangle),
        unsupported,
    ];
    let exact = DesignPanelTarget::Nodes {
        node_ids: vec!["first".into(), "second".into()],
    };
    let reordered = DesignPanelTarget::Nodes {
        node_ids: vec!["second".into(), "first".into()],
    };
    let missing = DesignPanelTarget::Nodes {
        node_ids: vec!["first".into(), "removed".into()],
    };
    let duplicate = DesignPanelTarget::Nodes {
        node_ids: vec!["first".into(), "first".into()],
    };
    let leaf = DesignPanelAction::PropertyChangeRequested {
        node_id: "aggregate".into(),
        property: DesignPanelProperty::Width,
        value: DesignPanelValue::Number(320.),
    };

    for (current, requested, can_edit, reason) in [
        (&reordered, &exact, true, "reordered current target"),
        (&missing, &missing, true, "missing member"),
        (&duplicate, &duplicate, true, "duplicate member"),
        (&exact, &exact, false, "read-only permission"),
    ] {
        assert!(
            story_targeted_node_actions(
                &supported_nodes,
                Some(current),
                requested,
                can_edit,
                &leaf,
            )
            .is_none(),
            "{reason} must reject before leaf replay"
        );
    }
    assert!(
        story_targeted_node_actions(&unsupported_nodes, Some(&exact), &exact, true, &leaf,)
            .is_none(),
        "a per-node capability mismatch must reject before leaf replay"
    );
}

#[test]
fn targeted_effect_add_preflights_every_node_before_mutating_any_node() {
    let kind = DesignEffectKind::LayerBlur;
    let first = DesignPanelNode::new("first", "First", DesignPanelNodeKind::Rectangle);
    let mut second = DesignPanelNode::new("second", "Second", DesignPanelNodeKind::Rectangle);
    for index in 0..kind.maximum_per_node() {
        second
            .effects
            .push(DesignEffect::new(kind).with_id(format!("second-blur-{index}")));
    }
    let unselected =
        DesignPanelNode::new("unselected", "Unselected", DesignPanelNodeKind::Rectangle);
    let maxed_nodes = vec![first.clone(), second.clone(), unselected.clone()];
    let addable_nodes = vec![
        first.clone(),
        DesignPanelNode::new("second", "Second", DesignPanelNodeKind::Rectangle),
        unselected.clone(),
    ];
    let exact = DesignPanelTarget::Nodes {
        node_ids: vec!["first".into(), "second".into()],
    };
    let reordered = DesignPanelTarget::Nodes {
        node_ids: vec!["second".into(), "first".into()],
    };
    let missing = DesignPanelTarget::Nodes {
        node_ids: vec!["first".into(), "removed".into()],
    };
    let duplicate = DesignPanelTarget::Nodes {
        node_ids: vec!["first".into(), "first".into()],
    };

    for (current, requested, can_edit, reason) in [
        (&reordered, &exact, true, "reordered current target"),
        (&missing, &missing, true, "missing member"),
        (&duplicate, &duplicate, true, "duplicate member"),
        (&exact, &exact, false, "read-only permission"),
    ] {
        let mut attempt = addable_nodes.clone();
        assert!(
            !apply_story_targeted_effect_add(
                &mut attempt,
                Some(current),
                requested,
                can_edit,
                kind,
            ),
            "{reason} must reject"
        );
        assert_eq!(
            attempt, addable_nodes,
            "{reason} must not mutate an earlier valid member"
        );
    }
    let mut maxed_attempt = maxed_nodes.clone();
    assert!(!apply_story_targeted_effect_add(
        &mut maxed_attempt,
        Some(&exact),
        &exact,
        true,
        kind,
    ));
    assert_eq!(
        maxed_attempt, maxed_nodes,
        "one maxed member must reject before an earlier valid member is mutated"
    );

    let (aggregate, _) = aggregate_story_multiple_selection(&first, &second);
    assert!(
        !aggregate
            .effect_capabilities
            .kind_is_available(DesignEffectKind::LayerBlur),
        "the synthetic empty aggregate must retain real per-node effect limits"
    );

    let mut successful_nodes = addable_nodes;
    assert!(apply_story_targeted_effect_add(
        &mut successful_nodes,
        Some(&exact),
        &exact,
        true,
        kind,
    ));
    assert_eq!(successful_nodes[0].effect_count(kind), 1);
    assert_eq!(successful_nodes[1].effect_count(kind), 1);
    assert_eq!(
        successful_nodes[2], unselected,
        "an unselected node must remain untouched"
    );
}

#[test]
fn transform_reducer_requires_one_exact_editable_applicable_target_before_mutating() {
    let mut base = vec![
        DesignPanelNode::new("first", "First", DesignPanelNodeKind::Rectangle),
        DesignPanelNode::new("second", "Second", DesignPanelNodeKind::Ellipse),
        DesignPanelNode::new("unselected", "Unselected", DesignPanelNodeKind::Rectangle),
    ];
    base[0].rotation = 15.;
    base[1].rotation = 35.;
    base[2].rotation = 55.;
    let exact = DesignPanelTarget::Nodes {
        node_ids: vec!["first".into(), "second".into()],
    };
    let reordered = DesignPanelTarget::Nodes {
        node_ids: vec!["second".into(), "first".into()],
    };
    let missing = DesignPanelTarget::Nodes {
        node_ids: vec!["first".into(), "removed".into()],
    };
    let duplicate = DesignPanelTarget::Nodes {
        node_ids: vec!["first".into(), "first".into()],
    };

    for (current, requested, can_edit, reason) in [
        (&reordered, &exact, true, "reordered current target"),
        (&missing, &missing, true, "missing member"),
        (&duplicate, &duplicate, true, "duplicate member"),
        (&exact, &exact, false, "read-only permission"),
    ] {
        let mut attempt = base.clone();
        assert!(
            !apply_story_transform_request(
                &mut attempt,
                Some(current),
                requested,
                can_edit,
                fanta_gpui::prelude::DesignTransformOperation::RotateClockwise90,
            ),
            "{reason} must reject"
        );
        assert_eq!(attempt, base, "{reason} must leave every node unchanged");
    }

    let mut inapplicable = base.clone();
    let mut capabilities = DesignPanelNodeCapabilities::for_node_kind(DesignPanelNodeKind::Ellipse);
    capabilities
        .sections
        .retain(|section| *section != DesignPanelSection::Position);
    inapplicable[1].capabilities = Some(capabilities);
    let before_inapplicable = inapplicable.clone();
    assert!(!apply_story_transform_request(
        &mut inapplicable,
        Some(&exact),
        &exact,
        true,
        fanta_gpui::prelude::DesignTransformOperation::RotateClockwise90,
    ));
    assert_eq!(
        inapplicable, before_inapplicable,
        "an inapplicable later member must not leave the first member rotated"
    );

    let mut applied = base.clone();
    assert!(apply_story_transform_request(
        &mut applied,
        Some(&exact),
        &exact,
        true,
        fanta_gpui::prelude::DesignTransformOperation::RotateClockwise90,
    ));
    assert_eq!(applied[0].rotation, 105.);
    assert_eq!(applied[1].rotation, 125.);
    assert_eq!(applied[2].rotation, 55.);
}

#[test]
fn story_multiple_aggregate_exposes_only_common_sections_and_no_first_node_leaves() {
    let mut first = DesignPanelNode::new("text", "Text", DesignPanelNodeKind::Text);
    first.width = 240.;
    first.fills = vec![DesignPaint::solid(DesignColor::BLUE).with_id("mixed-text-fill")];
    first
        .effects
        .push(DesignEffect::new(DesignEffectKind::DropShadow).with_id("text-shadow"));
    let mut second = DesignPanelNode::new("ellipse", "Ellipse", DesignPanelNodeKind::Ellipse);
    second.width = 480.;
    second.fills = vec![DesignPaint::solid(DesignColor::PURPLE).with_id("mixed-ellipse-fill")];
    second
        .effects
        .push(DesignEffect::new(DesignEffectKind::LayerBlur).with_id("ellipse-blur"));

    let (aggregate, states) = aggregate_story_multiple_selection(&first, &second);
    let capabilities = aggregate
        .capabilities
        .as_ref()
        .expect("the multiple aggregate is exact and authoritative");

    assert!(capabilities.sections.contains(&DesignPanelSection::Fill));
    assert!(capabilities.sections.contains(&DesignPanelSection::Effects));
    assert!(
        capabilities
            .sections
            .contains(&DesignPanelSection::Selection)
    );
    assert!(
        !capabilities
            .sections
            .contains(&DesignPanelSection::Typography)
    );
    assert!(
        !capabilities
            .sections
            .contains(&DesignPanelSection::Geometry)
    );
    assert!(
        !capabilities
            .sections
            .contains(&DesignPanelSection::Component)
    );
    assert!(aggregate.typography.is_none());
    assert!(aggregate.component_context.is_none());
    assert!(aggregate.component_properties.is_empty());
    assert_eq!(aggregate.shape_geometry, DesignShapeGeometry::None);
    assert!(aggregate.fills.is_empty());
    assert!(aggregate.stroke.is_none());
    assert!(aggregate.effects.is_empty());
    assert!(aggregate.fill_style_binding.is_none());
    assert!(aggregate.effect_style_binding.is_none());
    assert!(
        states.iter().any(|(property, state)| {
            *property == DesignPanelProperty::Width && state.is_mixed()
        })
    );
    let height_state = states
        .iter()
        .find_map(|(property, state)| (*property == DesignPanelProperty::Height).then_some(state))
        .expect("the aggregate resolves Height explicitly");
    assert_eq!(height_state.is_mixed(), first.height != second.height);
    if first.height == second.height {
        assert_eq!(
            height_state.resolved(),
            Some(&DesignPanelValue::Number(first.height)),
        );
    }
}

#[test]
fn story_draw_projection_requires_uniform_geometry_and_exact_order() {
    let mut first = DesignPanelNode::new("first", "First", DesignPanelNodeKind::Rectangle);
    first.width = 160.;
    first.height = 80.;
    let mut second = DesignPanelNode::new("second", "Second", DesignPanelNodeKind::Rectangle);
    second.width = 320.;
    second.height = 80.;

    let (mixed, mixed_states) =
        story_multiple_inspection_context(&first, &second, DesignPanelPermissions::editor());
    let mixed_target = story_design_target(&mixed).expect("exact mixed target");
    assert!(
        mixed_states.iter().any(|(property, state)| {
            *property == DesignPanelProperty::Width && state.is_mixed()
        })
    );
    assert!(
        story_draw_appearance_view_data(&mixed, &mixed_target).is_none(),
        "the Draw range must not be derived from member one when geometry is mixed",
    );

    second.width = first.width;
    let (uniform, uniform_states) =
        story_multiple_inspection_context(&first, &second, DesignPanelPermissions::editor());
    let exact_target = story_design_target(&uniform).expect("exact uniform target");
    assert!(uniform_states.iter().all(|(property, state)| {
        !matches!(
            property,
            DesignPanelProperty::Width | DesignPanelProperty::Height
        ) || state.resolved().is_some()
    }));
    let projection = story_draw_appearance_view_data(&uniform, &exact_target)
        .expect("uniform geometry has one exact aggregate range");
    assert_eq!(projection.target, exact_target);
    assert_eq!(projection.corner_radius_range.max(), 160.);

    for stale_target in [
        DesignPanelTarget::Nodes {
            node_ids: vec!["second".into(), "first".into()],
        },
        DesignPanelTarget::Nodes {
            node_ids: vec!["first".into(), "removed".into()],
        },
    ] {
        assert!(
            story_draw_appearance_view_data(&uniform, &stale_target).is_none(),
            "reordered and stale geometry projections must stay inert",
        );
    }
}

#[test]
fn story_multi_export_projection_never_masquerades_as_the_first_node() {
    let first = DesignPanelNode::new("first", "First", DesignPanelNodeKind::Text);
    let second = DesignPanelNode::new("second", "Second", DesignPanelNodeKind::Rectangle);
    let nodes = vec![first, second];
    let first_configuration = DesignExportConfiguration::new("first-only", DesignExportFormat::Svg);
    let second_configuration =
        DesignExportConfiguration::new("second-only", DesignExportFormat::Png);
    let mut configurations = HashMap::from([
        ("first".into(), vec![first_configuration]),
        ("second".into(), vec![second_configuration]),
    ]);
    let modes = HashMap::from([
        ("first".into(), DesignExportMode::Animated),
        ("second".into(), DesignExportMode::Static),
    ]);
    let animated = HashMap::from([(
        "first".into(),
        DesignAnimatedExportViewData::new(
            DesignAnimatedExportCapability::eligible(1920, 1080),
            DesignAnimatedExportSettings::default(),
        ),
    )]);
    let previews = HashMap::from([(
        "first".into(),
        DesignExportPreviewState::Ready(DesignExportPreview::new(1920, 1080)),
    )]);
    let target = DesignPanelTarget::Nodes {
        node_ids: vec!["first".into(), "second".into()],
    };

    let mixed = story_export_projection(
        target.clone(),
        &nodes,
        &configurations,
        &modes,
        &animated,
        &previews,
    );
    assert_eq!(mixed.target, target);
    assert!(
        mixed.configurations.is_empty(),
        "different per-node rows project as mixed/empty, never as node 1"
    );
    assert_eq!(mixed.mode, DesignExportMode::Static);
    assert!(mixed.animated.is_none());
    assert!(mixed.preview.is_none());
    assert!(
        !mixed.static_capabilities.can_include_text_bounding_box,
        "multi capabilities are the safe intersection, not node 1's text capability"
    );

    let shared = DesignExportConfiguration::new("shared", DesignExportFormat::Png);
    configurations.insert("first".into(), vec![shared.clone()]);
    configurations.insert("second".into(), vec![shared.clone()]);
    let uniform = story_export_projection(
        target,
        &nodes,
        &configurations,
        &modes,
        &animated,
        &previews,
    );
    assert_eq!(uniform.configurations, vec![shared]);
    assert_eq!(uniform.mode, DesignExportMode::Static);
    assert!(uniform.animated.is_none());
    assert!(uniform.preview.is_none());
}

#[test]
fn story_multi_export_reducer_is_atomic_and_rejects_reordered_or_stale_targets() {
    let nodes = vec![
        DesignPanelNode::new("second", "Second", DesignPanelNodeKind::Rectangle),
        DesignPanelNode::new("first", "First", DesignPanelNodeKind::Text),
        DesignPanelNode::new("unselected", "Unselected", DesignPanelNodeKind::Rectangle),
    ];
    let exact_target = DesignPanelTarget::Nodes {
        node_ids: vec!["first".into(), "second".into()],
    };
    let reordered_target = DesignPanelTarget::Nodes {
        node_ids: vec!["second".into(), "first".into()],
    };
    let stale_target = DesignPanelTarget::Nodes {
        node_ids: vec!["first".into(), "removed".into()],
    };
    let mut original = DesignExportConfiguration::new("shared", DesignExportFormat::Png);
    original.common.suffix = "-original".into();
    let mut configurations = HashMap::from([
        ("first".into(), vec![original.clone()]),
        ("second".into(), vec![original.clone()]),
        ("unselected".into(), vec![original.clone()]),
    ]);
    let before = configurations.clone();
    let mut snapshots = HashMap::new();
    let shared_id = SharedString::from("shared");
    let stale_change = DesignExportConfigurationChange::Suffix("-stale".into());

    assert!(!apply_story_export_configuration_edit(
        &nodes,
        &mut configurations,
        &mut snapshots,
        StoryExportConfigurationEdit {
            current_target: &exact_target,
            requested_target: &reordered_target,
            can_export: true,
            configuration_id: &shared_id,
            change: &stale_change,
            phase: DesignPanelEditPhase::Commit,
        },
    ));
    assert_eq!(configurations, before);
    assert!(!apply_story_export_configuration_remove(
        &nodes,
        &mut configurations,
        &exact_target,
        &stale_target,
        true,
        &shared_id,
    ));
    assert_eq!(configurations, before);

    let preview_change = DesignExportConfigurationChange::Suffix("-preview".into());
    assert!(apply_story_export_configuration_edit(
        &nodes,
        &mut configurations,
        &mut snapshots,
        StoryExportConfigurationEdit {
            current_target: &exact_target,
            requested_target: &exact_target,
            can_export: true,
            configuration_id: &shared_id,
            change: &preview_change,
            phase: DesignPanelEditPhase::Begin,
        },
    ));
    assert!(apply_story_export_configuration_edit(
        &nodes,
        &mut configurations,
        &mut snapshots,
        StoryExportConfigurationEdit {
            current_target: &exact_target,
            requested_target: &exact_target,
            can_export: true,
            configuration_id: &shared_id,
            change: &preview_change,
            phase: DesignPanelEditPhase::Preview,
        },
    ));
    for node_id in ["first", "second"] {
        assert_eq!(
            configurations[node_id][0].common.suffix.as_ref(),
            "-preview"
        );
    }
    assert_eq!(
        configurations["unselected"][0].common.suffix.as_ref(),
        "-original"
    );
    assert!(apply_story_export_configuration_edit(
        &nodes,
        &mut configurations,
        &mut snapshots,
        StoryExportConfigurationEdit {
            current_target: &exact_target,
            requested_target: &exact_target,
            can_export: true,
            configuration_id: &shared_id,
            change: &preview_change,
            phase: DesignPanelEditPhase::Cancel,
        },
    ));
    assert_eq!(configurations, before);
    assert!(snapshots.is_empty());

    let added = DesignExportConfiguration::new("added", DesignExportFormat::Png);
    assert!(apply_story_export_configuration_add(
        &nodes,
        &mut configurations,
        &exact_target,
        &exact_target,
        true,
        added.clone(),
    ));
    assert_eq!(configurations["first"].last(), Some(&added));
    assert_eq!(configurations["second"].last(), Some(&added));
    assert_eq!(configurations["unselected"], vec![original.clone()]);
    assert!(apply_story_export_configuration_remove(
        &nodes,
        &mut configurations,
        &exact_target,
        &exact_target,
        true,
        &added.id,
    ));
    assert_eq!(configurations["first"], vec![original.clone()]);
    assert_eq!(configurations["second"], vec![original.clone()]);
    assert_eq!(configurations["unselected"], vec![original]);

    configurations.get_mut("second").unwrap().clear();
    let mixed_before = configurations.clone();
    assert!(!apply_story_export_configuration_remove(
        &nodes,
        &mut configurations,
        &exact_target,
        &exact_target,
        true,
        &shared_id,
    ));
    assert_eq!(
        configurations, mixed_before,
        "mixed state rejects before node 1 can be mutated"
    );
}

#[test]
fn story_multiple_bindings_are_bound_only_when_every_target_matches() {
    let shared = DesignPanelPropertyBinding::new(
        "shared-opacity",
        "Shared opacity",
        DesignPanelBindingKind::Variable,
        DesignPanelValue::Number(72.),
    );
    let different = DesignPanelPropertyBinding::new(
        "different-opacity",
        "Different opacity",
        DesignPanelBindingKind::Variable,
        DesignPanelValue::Number(72.),
    );
    let mut bindings = HashMap::from([
        (("one".into(), DesignPanelProperty::Opacity), shared.clone()),
        (("two".into(), DesignPanelProperty::Opacity), shared.clone()),
        (("three".into(), DesignPanelProperty::Opacity), different),
    ]);

    assert_eq!(
        story_common_property_binding(
            &bindings,
            &["one".into(), "two".into()],
            DesignPanelProperty::Opacity,
        ),
        Some(&shared)
    );
    assert!(
        story_common_property_binding(
            &bindings,
            &["one".into(), "three".into()],
            DesignPanelProperty::Opacity,
        )
        .is_none()
    );
    bindings.remove(&("two".into(), DesignPanelProperty::Opacity));
    assert!(
        story_common_property_binding(
            &bindings,
            &["one".into(), "two".into()],
            DesignPanelProperty::Opacity,
        )
        .is_none()
    );
}

#[test]
fn story_selection_colors_do_not_merge_different_binding_or_lock_states() {
    let color = DesignColor::PURPLE;
    let brand = DesignPaintBinding::new("brand", "Brand");
    let accent = DesignPaintBinding::new("accent", "Accent");
    let node = |id: &'static str, binding: Option<DesignPaintBinding>, read_only| {
        let mut node = DesignPanelNode::new(id, id, DesignPanelNodeKind::Rectangle);
        let mut paint = DesignPaint::solid(color).with_id(format!("{id}-fill"));
        if let DesignPaintPayload::Solid(solid) = &mut paint.payload {
            solid.binding = binding;
        }
        paint.read_only = read_only;
        node.fills = vec![paint];
        node
    };
    let nodes = [
        node("brand-one", Some(brand.clone()), false),
        node("brand-two", Some(brand.clone()), false),
        node("unbound", None, false),
        node("accent", Some(accent.clone()), false),
        node("locked", None, true),
    ];
    let aggregate = aggregate_story_selection_colors(nodes.iter());

    assert_eq!(aggregate.colors.len(), 4);
    assert_eq!(
        aggregate
            .colors
            .iter()
            .find(|row| row.binding.as_ref() == Some(&brand) && !row.read_only)
            .map(|row| row.occurrence_count),
        Some(2)
    );
    assert!(
        aggregate
            .colors
            .iter()
            .any(|row| { row.binding.is_none() && !row.read_only && row.occurrence_count == 1 })
    );
    assert!(
        aggregate
            .colors
            .iter()
            .any(|row| { row.binding.as_ref() == Some(&accent) && !row.read_only })
    );
    assert!(
        aggregate
            .colors
            .iter()
            .any(|row| row.binding.is_none() && row.read_only)
    );
    assert_eq!(
        aggregate
            .colors
            .iter()
            .map(|row| row.id.clone())
            .collect::<std::collections::HashSet<_>>()
            .len(),
        aggregate.colors.len(),
        "each visually identical but semantically distinct row needs a unique stable ID"
    );
}

#[test]
fn story_selection_colors_keep_full_paint_and_paint_style_identity() {
    let style_a =
        DesignPaintStyleBinding::new(DesignPaintStyleSelection::page("style-a"), "Style A");
    let style_b =
        DesignPaintStyleBinding::new(DesignPaintStyleSelection::page("style-b"), "Style B");
    let node = |id: &'static str, opacity: f32, style: DesignPaintStyleBinding| {
        let mut node = DesignPanelNode::new(id, id, DesignPanelNodeKind::Rectangle);
        let mut paint = DesignPaint::solid(DesignColor::BLUE).with_id(format!("{id}-fill"));
        paint.opacity = opacity;
        node.fills = vec![paint];
        node.fill_style_binding = Some(style);
        node
    };
    let mut nodes = [
        node("a-one", 100., style_a.clone()),
        node("a-two", 100., style_a.clone()),
        node("a-translucent", 42., style_a.clone()),
        node("b-one", 100., style_b),
    ];
    let aggregate = aggregate_story_selection_colors(nodes.iter());

    assert_eq!(
        aggregate.colors.len(),
        3,
        "same RGBA is insufficient when Paint or Paint-style identity differs"
    );
    let repeated = aggregate
        .colors
        .iter()
        .find(|row| {
            row.paint.opacity == 100.
                && row.style_binding.as_ref() == Some(&style_a)
                && row.occurrence_count == 2
        })
        .expect("semantically identical paints should retain one aggregate row");
    assert_eq!(repeated.style_paints.len(), 1);
    assert_eq!(
        repeated
            .paint_references
            .iter()
            .map(|reference| reference.node_id.as_ref())
            .collect::<Vec<_>>(),
        ["a-one", "a-two"],
        "occurrence references preserve exact host order"
    );
    let stable_id = repeated.id.clone();

    for node in &mut nodes {
        node.fills[0].apply_edit(&DesignPaintEdit {
            property: DesignPaintProperty::Color,
            value: DesignPaintValue::Color(DesignColor::PURPLE),
        });
    }
    let recolored = aggregate_story_selection_colors(nodes.iter());
    assert_eq!(
        recolored
            .colors
            .iter()
            .find(|row| {
                row.paint_references
                    .first()
                    .is_some_and(|reference| reference.node_id.as_ref() == "a-one")
            })
            .map(|row| row.id.clone()),
        Some(stable_id),
        "aggregate row identity must not be derived from mutable RGBA"
    );
}

#[test]
fn story_selection_colors_group_each_gradient_paint_once() {
    let node = |node_id: &'static str, paint_id: &'static str| {
        let mut node = DesignPanelNode::new(node_id, node_id, DesignPanelNodeKind::Rectangle);
        node.fills = vec![
            DesignPaint::gradient(
                DesignPaintKind::LinearGradient,
                vec![
                    fanta_gpui::design::DesignGradientStop::new(0., DesignColor::BLUE)
                        .with_id(format!("{paint_id}-start")),
                    fanta_gpui::design::DesignGradientStop::new(1., DesignColor::PURPLE)
                        .with_id(format!("{paint_id}-end")),
                ],
            )
            .with_id(paint_id),
        ];
        node
    };
    let nodes = [
        node("gradient-one", "gradient-one-fill"),
        node("gradient-two", "gradient-two-fill"),
    ];

    let aggregate = aggregate_story_selection_colors(nodes.iter());

    assert_eq!(
        aggregate.colors.len(),
        1,
        "a normal gradient contributes one Selection colors row, not one per stop"
    );
    let row = &aggregate.colors[0];
    assert_eq!(row.occurrence_count, 2);
    assert!(matches!(row.paint.payload, DesignPaintPayload::Gradient(_)));
    assert!(
        row.paint_references
            .iter()
            .all(|reference| reference.gradient_stop_id.is_none()
                && reference.gradient_stop_index.is_none()),
        "canonical references target the full gradient paint"
    );
    assert_eq!(
        row.paint_references
            .iter()
            .map(|reference| reference.node_id.as_ref())
            .collect::<Vec<_>>(),
        ["gradient-one", "gradient-two"],
        "occurrence order follows the exact host selection order"
    );
}

#[test]
fn story_selection_resource_targets_require_exact_current_references() {
    let mut first = DesignPanelNode::new("first", "First", DesignPanelNodeKind::Rectangle);
    first.fills = vec![DesignPaint::solid(DesignColor::BLUE).with_id("stable-first-fill")];
    let mut second = DesignPanelNode::new("second", "Second", DesignPanelNodeKind::Rectangle);
    second.fills = vec![DesignPaint::solid(DesignColor::BLUE).with_id("stable-second-fill")];
    let nodes = vec![first, second];
    let target = DesignPanelTarget::Nodes {
        node_ids: vec!["second".into(), "first".into()],
    };
    let references = vec![
        DesignSelectionPaintReference::paint(
            "second",
            DesignSelectionPaintCollection::Fill,
            "stable-second-fill",
            0,
        ),
        DesignSelectionPaintReference::paint(
            "first",
            DesignSelectionPaintCollection::Fill,
            "stable-first-fill",
            0,
        ),
    ];

    assert!(story_selection_references_are_current(
        &nodes,
        &target,
        &references
    ));
    let mut stale = references.clone();
    stale[0].paint_id = "stale".into();
    assert!(!story_selection_references_are_current(
        &nodes, &target, &stale
    ));
    assert!(!story_selection_references_are_current(
        &nodes,
        &DesignPanelTarget::Nodes {
            node_ids: vec!["first".into()],
        },
        &references,
    ));
}

#[test]
fn cancelled_story_paint_edit_restores_the_begin_snapshot() {
    let mut paint = DesignPaint::solid(DesignColor::WHITE).with_id("stable-fill");
    let target = StoryPaintEditTarget::new(
        "rectangle".into(),
        DesignPanelCollection::Fill,
        DesignPaintTarget::WholeLayer,
        "stable-fill".into(),
        3,
    );
    let preview = DesignPaintEdit {
        property: DesignPaintProperty::Color,
        value: DesignPaintValue::Color(DesignColor::BLACK),
    };
    let non_restorative_cancel_payload = DesignPaintEdit {
        property: DesignPaintProperty::Color,
        value: DesignPaintValue::Color(DesignColor::PURPLE),
    };
    let mut snapshots = HashMap::new();

    apply_story_paint_edit_phase(
        &mut paint,
        &mut snapshots,
        target.clone(),
        &preview,
        DesignPanelEditPhase::Begin,
    );
    apply_story_paint_edit_phase(
        &mut paint,
        &mut snapshots,
        target.clone(),
        &preview,
        DesignPanelEditPhase::Preview,
    );
    assert_eq!(paint.color, DesignColor::BLACK);

    apply_story_paint_edit_phase(
        &mut paint,
        &mut snapshots,
        target,
        &non_restorative_cancel_payload,
        DesignPanelEditPhase::Cancel,
    );
    assert_eq!(paint.color, DesignColor::WHITE);
    assert!(snapshots.is_empty());
}

#[test]
fn story_paint_style_apply_replaces_the_whole_ordered_collection() {
    let selection = DesignPaintStyleSelection::page("layered-style");
    let style = DesignPaintStyle::new(
        "layered-style",
        "Layered",
        [
            DesignPaint::gradient(
                DesignPaintKind::LinearGradient,
                vec![
                    DesignGradientStop::new(0., DesignColor::PURPLE).with_id("source-start"),
                    DesignGradientStop::new(1., DesignColor::BLUE).with_id("source-end"),
                ],
            )
            .with_id("source-gradient"),
            DesignPaint::solid(DesignColor::WHITE).with_id("source-highlight"),
        ],
    );
    let mut node = DesignPanelNode::new(
        "style-target",
        "Style target",
        DesignPanelNodeKind::Rectangle,
    );
    node.fills = vec![DesignPaint::solid(DesignColor::BLACK).with_id("old-fill")];

    assert!(apply_story_paint_style(
        &mut node,
        DesignPanelCollection::Fill,
        &selection,
        style.clone(),
    ));
    assert_eq!(node.fills.len(), 2);
    assert_eq!(
        node.fills[0].id.as_ref(),
        "style-target-fill-layered-style-0"
    );
    assert_eq!(
        node.fills[1].id.as_ref(),
        "style-target-fill-layered-style-1"
    );
    let DesignPaintPayload::Gradient(gradient) = &node.fills[0].payload else {
        panic!("first style paint should remain a gradient");
    };
    assert_eq!(
        gradient
            .stops
            .iter()
            .map(|stop| stop.id.as_ref())
            .collect::<Vec<_>>(),
        [
            "style-target-fill-layered-style-0-stop-0",
            "style-target-fill-layered-style-0-stop-1",
        ]
    );
    assert_eq!(
        node.fill_style_binding
            .as_ref()
            .map(|binding| &binding.selection),
        Some(&selection)
    );

    let fills_after_apply = node.fills.clone();
    assert!(!apply_story_paint_style(
        &mut node,
        DesignPanelCollection::Fill,
        &selection,
        style.with_import_state(DesignPaintStyleImportState::Available),
    ));
    assert_eq!(
        node.fills, fills_after_apply,
        "available library styles require a separate import intent"
    );
}

#[test]
fn story_color_style_sample_changes_only_the_target_leaf() {
    let mut solid = DesignPaint::solid(DesignColor::BLACK).with_id("solid");
    solid.opacity = 100.;
    assert!(apply_story_color_style_sample(
        &mut solid,
        &DesignPaintColorTarget::Solid,
        DesignColor::rgba(0x0d, 0x99, 0xff, 0x80),
    ));
    let DesignPaintPayload::Solid(solid_payload) = &solid.payload else {
        panic!("paint should remain solid");
    };
    assert_eq!(solid_payload.color, DesignColor::rgb(0x0d, 0x99, 0xff));
    assert!(solid_payload.binding.is_none());
    assert!((solid.opacity - f32::from(0x80_u8) / 255. * 100.).abs() < 0.001);

    let mut gradient = DesignPaint::gradient(
        DesignPaintKind::LinearGradient,
        vec![
            DesignGradientStop::new(0., DesignColor::WHITE).with_id("start"),
            DesignGradientStop::new(1., DesignColor::BLACK).with_id("end"),
        ],
    );
    gradient.opacity = 44.;
    assert!(apply_story_color_style_sample(
        &mut gradient,
        &DesignPaintColorTarget::GradientStop {
            stop_id: "end".into(),
            index: 0,
        },
        DesignColor::rgba(0x97, 0x47, 0xff, 0x40),
    ));
    let DesignPaintPayload::Gradient(payload) = &gradient.payload else {
        panic!("paint should remain a gradient");
    };
    assert_eq!(payload.stops[0].color, DesignColor::WHITE);
    assert_eq!(
        payload.stops[1].color,
        DesignColor::rgba(0x97, 0x47, 0xff, 0x40)
    );
    assert!(payload.stops[1].binding.is_none());
    assert_eq!(gradient.opacity, 44.);
}

#[test]
fn story_color_variable_binding_targets_one_stable_gradient_leaf() {
    let mut paint = DesignPaint::gradient(
        DesignPaintKind::LinearGradient,
        vec![
            DesignGradientStop::new(0., DesignColor::WHITE).with_id("start"),
            DesignGradientStop::new(1., DesignColor::BLACK).with_id("end"),
        ],
    )
    .with_id("gradient");
    let variable = DesignVariable::page(
        "brand-color",
        "Brand",
        "semantic",
        "Semantic",
        DesignVariableResolvedType::Color,
    )
    .with_resolved_value(DesignVariableResolvedValue::Color(DesignColor::BLUE));
    let target = DesignPaintColorTarget::GradientStop {
        stop_id: "end".into(),
        index: 0,
    };

    assert!(apply_story_paint_variable(&mut paint, &target, &variable));
    let DesignPaintPayload::Gradient(gradient) = &paint.payload else {
        panic!("paint should remain a gradient");
    };
    assert_eq!(gradient.stops[0].color, DesignColor::WHITE);
    assert!(gradient.stops[0].binding.is_none());
    assert_eq!(gradient.stops[1].color, DesignColor::BLUE);
    assert_eq!(
        gradient.stops[1]
            .binding
            .as_ref()
            .map(|binding| binding.variable_id.as_ref()),
        Some("brand-color")
    );

    assert!(!detach_story_paint_variable(
        &mut paint,
        &target,
        &"stale-variable".into(),
    ));
    assert!(detach_story_paint_variable(
        &mut paint,
        &target,
        &"brand-color".into(),
    ));
    let DesignPaintPayload::Gradient(gradient) = &paint.payload else {
        panic!("paint should remain a gradient");
    };
    assert_eq!(
        gradient.stops[1].color,
        DesignColor::BLUE,
        "detaching preserves the resolved color"
    );
    assert!(gradient.stops[1].binding.is_none());
}

#[test]
fn story_layout_guide_edit_phases_follow_identity_and_preserve_host_reorder() {
    let first = DesignLayoutGrid::uniform(8., DesignColor::BLUE).with_id("guide-first");
    let second = DesignLayoutGrid::uniform(12., DesignColor::PURPLE).with_id("guide-second");
    let mut node = DesignPanelNode::new("frame", "Frame", DesignPanelNodeKind::Frame);
    node.layout_grids = vec![first, second];
    let mut snapshots = HashMap::new();
    let begin_target = StoryLayoutGridEditTarget::new(
        node.id.clone(),
        "guide-first".into(),
        0,
        DesignPanelProperty::LayoutGridSize(0),
    );

    assert!(apply_story_layout_grid_edit_phase(
        &mut node,
        &mut snapshots,
        begin_target,
        DesignPanelProperty::LayoutGridSize(0),
        &DesignPanelValue::Number(8.),
        DesignPanelEditPhase::Begin,
    ));
    node.layout_grids.swap(0, 1);
    let reordered_target = StoryLayoutGridEditTarget::new(
        node.id.clone(),
        "guide-first".into(),
        1,
        DesignPanelProperty::LayoutGridSize(1),
    );
    assert!(apply_story_layout_grid_edit_phase(
        &mut node,
        &mut snapshots,
        reordered_target.clone(),
        DesignPanelProperty::LayoutGridSize(1),
        &DesignPanelValue::Number(24.),
        DesignPanelEditPhase::Preview,
    ));
    assert_eq!(node.layout_grids[0].id.as_ref(), "guide-second");
    assert_eq!(node.layout_grids[1].id.as_ref(), "guide-first");
    assert_eq!(
        node.layout_grids[1].settings,
        DesignLayoutGridSettings::Uniform(DesignUniformLayoutGrid { size: 24. })
    );

    assert!(apply_story_layout_grid_edit_phase(
        &mut node,
        &mut snapshots,
        reordered_target,
        DesignPanelProperty::LayoutGridSize(1),
        &DesignPanelValue::Number(999.),
        DesignPanelEditPhase::Cancel,
    ));
    assert_eq!(
        node.layout_grids
            .iter()
            .map(|guide| guide.id.as_ref())
            .collect::<Vec<_>>(),
        vec!["guide-second", "guide-first"],
        "Cancel restores only the stable guide and keeps the host order"
    );
    assert_eq!(
        node.layout_grids[1].settings,
        DesignLayoutGridSettings::Uniform(DesignUniformLayoutGrid { size: 8. })
    );
    assert!(snapshots.is_empty());

    let commit_begin_target = StoryLayoutGridEditTarget::new(
        node.id.clone(),
        "guide-first".into(),
        1,
        DesignPanelProperty::LayoutGridSize(1),
    );
    assert!(apply_story_layout_grid_edit_phase(
        &mut node,
        &mut snapshots,
        commit_begin_target,
        DesignPanelProperty::LayoutGridSize(1),
        &DesignPanelValue::Number(8.),
        DesignPanelEditPhase::Begin,
    ));
    node.layout_grids.swap(0, 1);
    let commit_target = StoryLayoutGridEditTarget::new(
        node.id.clone(),
        "guide-first".into(),
        0,
        DesignPanelProperty::LayoutGridSize(0),
    );
    assert!(apply_story_layout_grid_edit_phase(
        &mut node,
        &mut snapshots,
        commit_target,
        DesignPanelProperty::LayoutGridSize(0),
        &DesignPanelValue::Number(32.),
        DesignPanelEditPhase::Commit,
    ));
    assert_eq!(node.layout_grids[0].id.as_ref(), "guide-first");
    assert_eq!(
        node.layout_grids[0].settings,
        DesignLayoutGridSettings::Uniform(DesignUniformLayoutGrid { size: 32. })
    );
    assert!(snapshots.is_empty());

    let stale_remove_hint = StoryLayoutGridEditTarget::new(
        node.id.clone(),
        "guide-first".into(),
        1,
        DesignPanelProperty::LayoutGridVisible(1),
    );
    assert_eq!(
        story_layout_grid_index(&node, &stale_remove_hint),
        Some(0),
        "stable identity wins over the stale compatibility index"
    );
}

#[test]
fn story_layout_guide_variables_cover_every_numeric_leaf_by_stable_identity() {
    let uniform = DesignLayoutGrid::uniform(8., DesignColor::BLUE).with_id("uniform");
    let columns = DesignLayoutGrid::columns(
        DesignColumnLayoutGrid {
            alignment: DesignColumnGridAlignment::Left,
            ..DesignColumnLayoutGrid::default()
        },
        DesignColor::PURPLE,
    )
    .with_id("columns");
    let rows =
        DesignLayoutGrid::rows(DesignRowLayoutGrid::default(), DesignColor::BLUE).with_id("rows");
    let mut node = DesignPanelNode::new("frame", "Frame", DesignPanelNodeKind::Frame);
    node.layout_grids = vec![uniform, columns, rows];

    let number = DesignVariable::page(
        "number-32",
        "Number / 32",
        "spacing",
        "Spacing",
        DesignVariableResolvedType::Float,
    )
    .with_resolved_value(DesignVariableResolvedValue::Float(32.));
    let count = DesignVariable::page(
        "count-6",
        "Count / 6",
        "responsive",
        "Responsive",
        DesignVariableResolvedType::Float,
    )
    .with_resolved_value(DesignVariableResolvedValue::Float(6.));

    let uniform_target = node.layout_grids[0]
        .variable_target(0, DesignPanelProperty::LayoutGridSize(0))
        .expect("uniform size")
        .0;
    let column_size = node.layout_grids[1]
        .variable_target(1, DesignPanelProperty::LayoutGridSize(1))
        .expect("column width")
        .0;
    let column_offset = node.layout_grids[1]
        .variable_target(1, DesignPanelProperty::LayoutGridOffset(1))
        .expect("column offset")
        .0;
    let column_gutter = node.layout_grids[1]
        .variable_target(1, DesignPanelProperty::LayoutGridGutter(1))
        .expect("column gutter")
        .0;
    let column_count = node.layout_grids[1]
        .variable_target(1, DesignPanelProperty::LayoutGridCount(1))
        .expect("column count")
        .0;
    let row_margin = node.layout_grids[2]
        .variable_target(2, DesignPanelProperty::LayoutGridMargin(2))
        .expect("row margin")
        .0;
    let row_gutter = node.layout_grids[2]
        .variable_target(2, DesignPanelProperty::LayoutGridGutter(2))
        .expect("row gutter")
        .0;

    node.layout_grids.swap(0, 2);
    for target in [
        &uniform_target,
        &column_size,
        &column_offset,
        &column_gutter,
        &row_margin,
        &row_gutter,
    ] {
        assert!(apply_story_layout_grid_variable(&mut node, target, &number));
    }
    assert!(apply_story_layout_grid_variable(
        &mut node,
        &column_count,
        &count
    ));
    assert_eq!(
        story_layout_grid_variable_target(&node, &uniform_target).map(|(index, _)| index),
        Some(2),
        "the stable guide ID wins over the stale index"
    );
    assert!(
        !apply_story_layout_grid_property(
            &mut node.layout_grids[2],
            DesignPanelProperty::LayoutGridSize(2),
            &DesignPanelValue::Number(99.),
        ),
        "direct edits stay locked while the exact leaf is variable-bound"
    );
    assert!(detach_story_layout_grid_variable(
        &mut node,
        &uniform_target,
        "number-32",
    ));
    assert_eq!(
        node.layout_grids[2].variable_binding(DesignLayoutGridVariableField::SectionSize),
        None
    );

    let row_index = node
        .layout_grids
        .iter()
        .position(|guide| guide.id.as_ref() == "rows")
        .expect("rows");
    node.layout_grids[row_index].set_variable_binding(
        DesignLayoutGridVariableField::GutterSize,
        Some(DesignLayoutGridVariableBinding::new("locked", "Locked").read_only("Library policy")),
    );
    assert!(!apply_story_layout_grid_variable(
        &mut node,
        &row_gutter,
        &number,
    ));
    assert!(!detach_story_layout_grid_variable(
        &mut node,
        &row_gutter,
        "locked",
    ));
}

#[test]
fn cancelled_story_node_edit_restores_the_complete_begin_snapshot() {
    let mut node = DesignPanelNode::new("rectangle", "Rectangle", DesignPanelNodeKind::Rectangle);
    node.width = 240.;
    node.opacity = 0.75;
    let target = StoryNodeEditTarget::new("rectangle".into(), "property:Width");
    let mut snapshots = HashMap::new();

    begin_story_node_edit(&node, &mut snapshots, &target, DesignPanelEditPhase::Begin);
    node.width = 480.;
    node.opacity = 0.2;
    finish_story_node_edit(
        &mut node,
        &mut snapshots,
        target,
        DesignPanelEditPhase::Cancel,
    );

    assert_eq!(node.width, 240.);
    assert_eq!(node.opacity, 0.75);
    assert!(snapshots.is_empty());
}

#[test]
fn cancelled_story_font_weight_edit_restores_the_typography_snapshot() {
    let mut node = DesignPanelNode::new("text", "Text", DesignPanelNodeKind::Text);
    node.typography.as_mut().expect("text typography").weight = 400.;
    let action = DesignPanelAction::TypographyPropertyEditRequested {
        node_id: "text".into(),
        target: fanta_gpui::prelude::DesignTypographyTarget::WholeLayer,
        property: DesignPanelProperty::FontWeight,
        value: DesignPanelValue::Number(400.),
        phase: DesignPanelEditPhase::Begin,
    };
    let (target, phase) =
        story_node_edit_transaction(&action).expect("font weight is a typography transaction");
    let mut snapshots = HashMap::new();

    begin_story_node_edit(&node, &mut snapshots, &target, phase);
    apply_design_property(
        &mut node,
        DesignPanelProperty::FontWeight,
        &DesignPanelValue::Number(700.),
    );
    finish_story_node_edit(
        &mut node,
        &mut snapshots,
        target,
        DesignPanelEditPhase::Cancel,
    );

    assert_eq!(
        node.typography.as_ref().expect("text typography").weight,
        400.
    );
    assert!(snapshots.is_empty());
}

#[test]
fn story_aspect_ratio_echo_couples_dimensions_and_proportional_limits() {
    let mut node = DesignPanelNode::new("rectangle", "Rectangle", DesignPanelNodeKind::Rectangle);
    node.width = 320.;
    node.height = 180.;
    node.lock_aspect_ratio = true;

    apply_design_property(
        &mut node,
        DesignPanelProperty::Width,
        &DesignPanelValue::Number(640.),
    );
    assert_eq!((node.width, node.height), (640., 360.));

    apply_design_property(
        &mut node,
        DesignPanelProperty::Height,
        &DesignPanelValue::Number(180.),
    );
    assert_eq!((node.width, node.height), (320., 180.));

    apply_design_property(
        &mut node,
        DesignPanelProperty::MinWidth,
        &DesignPanelValue::OptionalNumber(Some(200.)),
    );
    let layout = node.layout.as_ref().expect("rectangle layout");
    assert_eq!(layout.item.min_width, Some(200.));
    assert_eq!(layout.item.min_height, Some(112.5));

    apply_design_property(
        &mut node,
        DesignPanelProperty::MaxHeight,
        &DesignPanelValue::OptionalNumber(Some(90.)),
    );
    let layout = node.layout.as_ref().expect("rectangle layout");
    assert_eq!(layout.item.max_height, Some(90.));
    assert_eq!(layout.item.max_width, Some(160.));

    apply_design_property(
        &mut node,
        DesignPanelProperty::MinHeight,
        &DesignPanelValue::OptionalNumber(None),
    );
    let layout = node.layout.as_ref().expect("rectangle layout");
    assert_eq!(layout.item.min_height, None);
    assert_eq!(layout.item.min_width, None);

    let target = StoryNodeEditTarget::new("rectangle".into(), "property:Width");
    let mut snapshots = HashMap::new();
    begin_story_node_edit(&node, &mut snapshots, &target, DesignPanelEditPhase::Begin);
    apply_design_property(
        &mut node,
        DesignPanelProperty::Width,
        &DesignPanelValue::Number(480.),
    );
    assert_eq!((node.width, node.height), (480., 270.));
    finish_story_node_edit(
        &mut node,
        &mut snapshots,
        target,
        DesignPanelEditPhase::Cancel,
    );
    assert_eq!((node.width, node.height), (320., 180.));
    assert!(snapshots.is_empty());
}

#[test]
fn story_arc_start_and_sweep_echo_preserve_each_appearance_dimension() {
    let mut node = DesignPanelNode::new("partial-arc", "Partial arc", DesignPanelNodeKind::Ellipse);
    node.shape_geometry = DesignShapeGeometry::Ellipse(DesignArcData::new(
        std::f32::consts::FRAC_PI_2,
        std::f32::consts::PI,
        0.25,
    ));

    apply_design_property(
        &mut node,
        DesignPanelProperty::ArcSweep,
        &DesignPanelValue::AngleRadians(std::f32::consts::PI),
    );
    let DesignShapeGeometry::Ellipse(arc) = node.shape_geometry else {
        panic!("ellipse geometry");
    };
    assert!((arc.starting_angle - std::f32::consts::FRAC_PI_2).abs() < 0.000_001);
    assert!((arc.ending_angle - 3. * std::f32::consts::FRAC_PI_2).abs() < 0.000_001);
    assert!((arc.sweep_angle() - std::f32::consts::PI).abs() < 0.000_001);

    apply_design_property(
        &mut node,
        DesignPanelProperty::ArcStartingAngle,
        &DesignPanelValue::AngleRadians(0.),
    );
    let DesignShapeGeometry::Ellipse(arc) = node.shape_geometry else {
        panic!("ellipse geometry");
    };
    assert!(arc.starting_angle.abs() < 0.000_001);
    assert!((arc.ending_angle - std::f32::consts::PI).abs() < 0.000_001);
    assert!((arc.sweep_angle() - std::f32::consts::PI).abs() < 0.000_001);
}

#[test]
fn story_component_instance_child_rejects_aspect_ratio_lock() {
    let mut node = DesignPanelNode::new("nested", "Nested", DesignPanelNodeKind::Rectangle);
    node.is_component_instance_child = true;

    apply_design_property(
        &mut node,
        DesignPanelProperty::LockAspectRatio,
        &DesignPanelValue::Bool(true),
    );

    assert!(!node.lock_aspect_ratio);
    assert!(
        design_fixtures::seed_design_nodes()
            .iter()
            .any(
                |candidate| candidate.id.as_ref() == "component-instance-child"
                    && candidate.is_component_instance_child
            ),
        "the story exposes the unavailable-setting fixture"
    );
}

#[test]
#[should_panic(expected = "Storybook host has no reducer")]
fn story_property_reducer_rejects_mismatched_property_values() {
    let mut node = DesignPanelNode::new("rectangle", "Rectangle", DesignPanelNodeKind::Rectangle);
    apply_design_property(
        &mut node,
        DesignPanelProperty::Width,
        &DesignPanelValue::Bool(true),
    );
}

#[test]
fn story_grid_reorder_uses_figma_insertion_coordinates() {
    let mut tracks = vec![
        DesignGridTrack::fixed(80.),
        DesignGridTrack::fixed(120.),
        DesignGridTrack::fixed(160.),
    ];

    assert!(reorder_story_grid_tracks(&mut tracks, &[0], 3));
    assert_eq!(
        tracks,
        vec![
            DesignGridTrack::fixed(120.),
            DesignGridTrack::fixed(160.),
            DesignGridTrack::fixed(80.),
        ]
    );
    assert!(reorder_story_grid_tracks(&mut tracks, &[2], 0));
    assert_eq!(
        tracks,
        vec![
            DesignGridTrack::fixed(80.),
            DesignGridTrack::fixed(120.),
            DesignGridTrack::fixed(160.),
        ]
    );
    assert!(!reorder_story_grid_tracks(&mut tracks, &[0, 0], 2));
    assert!(!reorder_story_grid_tracks(&mut tracks, &[3], 0));
}

#[test]
fn story_grid_transition_and_counts_echo_current_design_semantics() {
    let mut node = DesignPanelNode::new("grid", "Grid", DesignPanelNodeKind::Frame);

    apply_design_property(
        &mut node,
        DesignPanelProperty::LayoutMode,
        &DesignPanelValue::LayoutMode(DesignLayoutMode::Grid),
    );
    let layout = node.layout.as_ref().expect("Grid layout");
    assert_eq!(layout.horizontal_sizing, DesignSizingMode::Hug);
    assert_eq!(layout.vertical_sizing, DesignSizingMode::Hug);
    assert_eq!(layout.grid_columns, vec![DesignGridTrack::hug()]);
    assert_eq!(layout.grid_rows, vec![DesignGridTrack::hug()]);
    assert_eq!(layout.grid_auto_tracks, DesignGridAutoTracks::Rows);
    assert_eq!(
        layout.grid_items_positioning,
        DesignGridItemsPositioning::RowAutoFlow
    );

    apply_design_property(
        &mut node,
        DesignPanelProperty::GridRowCount,
        &DesignPanelValue::Integer(4),
    );
    assert_eq!(
        node.layout.as_ref().expect("Grid layout").grid_rows.len(),
        1,
        "automatic rows keep row count host-managed"
    );
    apply_design_property(
        &mut node,
        DesignPanelProperty::GridColumnCount,
        &DesignPanelValue::Integer(3),
    );
    assert_eq!(
        node.layout.as_ref().expect("Grid layout").grid_columns,
        vec![
            DesignGridTrack::hug(),
            DesignGridTrack::hug(),
            DesignGridTrack::hug(),
        ]
    );
}

#[test]
fn story_grid_dimensions_reducer_is_atomic_phased_and_preserves_derived_rows() {
    let mut node = DesignPanelNode::new("grid", "Grid", DesignPanelNodeKind::Frame)
        .with_layout_mode(DesignLayoutMode::Grid);
    let layout = node.layout.as_mut().expect("Grid layout");
    layout.grid_auto_tracks = DesignGridAutoTracks::None;
    layout.grid_columns = vec![DesignGridTrack::fixed(80.), DesignGridTrack::fraction(1.)];
    layout.grid_rows = vec![DesignGridTrack::fixed(40.), DesignGridTrack::fixed(64.)];
    let original = layout.clone();
    let mut snapshots = HashMap::new();
    let node_id: SharedString = "grid".into();

    assert!(apply_story_grid_dimensions_edit_phase(
        &mut node,
        &mut snapshots,
        &node_id,
        DesignGridDimensions {
            columns: 2,
            rows: 2,
        },
        DesignPanelEditPhase::Begin,
    ));
    assert!(apply_story_grid_dimensions_edit_phase(
        &mut node,
        &mut snapshots,
        &node_id,
        DesignGridDimensions {
            columns: 4,
            rows: 3,
        },
        DesignPanelEditPhase::Preview,
    ));
    let preview = node.layout.as_ref().expect("Grid preview");
    assert_eq!(preview.grid_columns.len(), 4);
    assert_eq!(preview.grid_rows.len(), 3);
    assert_eq!(preview.grid_columns[0], DesignGridTrack::fixed(80.));
    assert_eq!(preview.grid_columns[1], DesignGridTrack::fraction(1.));
    assert!(apply_story_grid_dimensions_edit_phase(
        &mut node,
        &mut snapshots,
        &node_id,
        DesignGridDimensions {
            columns: 2,
            rows: 2,
        },
        DesignPanelEditPhase::Cancel,
    ));
    assert_eq!(node.layout.as_ref(), Some(&original));
    assert!(snapshots.is_empty());

    assert!(apply_story_grid_dimensions_edit_phase(
        &mut node,
        &mut snapshots,
        &node_id,
        DesignGridDimensions {
            columns: 3,
            rows: 1,
        },
        DesignPanelEditPhase::Commit,
    ));
    let committed = node.layout.as_ref().expect("Grid commit");
    assert_eq!(committed.grid_columns.len(), 3);
    assert_eq!(committed.grid_rows.len(), 1);
    assert!(snapshots.is_empty());

    let derived_rows = committed.grid_rows.clone();
    node.layout.as_mut().expect("Grid layout").grid_auto_tracks = DesignGridAutoTracks::Rows;
    assert!(!apply_story_grid_dimensions_edit_phase(
        &mut node,
        &mut snapshots,
        &node_id,
        DesignGridDimensions {
            columns: 5,
            rows: 2,
        },
        DesignPanelEditPhase::Commit,
    ));
    assert!(apply_story_grid_dimensions_edit_phase(
        &mut node,
        &mut snapshots,
        &node_id,
        DesignGridDimensions {
            columns: 5,
            rows: 1,
        },
        DesignPanelEditPhase::Commit,
    ));
    let automatic = node.layout.as_ref().expect("automatic Grid rows");
    assert_eq!(automatic.grid_columns.len(), 5);
    assert_eq!(automatic.grid_rows, derived_rows);
}

#[test]
fn story_stroke_reducer_preserves_lossless_forms_and_exact_domains() {
    let mut node = DesignPanelNode::new("vector", "Vector", DesignPanelNodeKind::Vector);

    apply_design_property(
        &mut node,
        DesignPanelProperty::StrokeDashMode,
        &DesignPanelValue::StrokeDashMode(DesignStrokeDashMode::Custom),
    );
    apply_design_property(
        &mut node,
        DesignPanelProperty::StrokeDashPattern,
        &DesignPanelValue::NumberList(vec![12., 6., 2., 6.]),
    );
    assert_eq!(
        node.stroke.as_ref().expect("stroke").dashes.pattern,
        vec![12., 6., 2., 6.]
    );

    let ordered_points = [
        DesignVariableWidthPoint::new(0.75, 1.5),
        DesignVariableWidthPoint::new(0.25, 0.5),
    ];
    apply_design_property(
        &mut node,
        DesignPanelProperty::StrokeVariableWidth,
        &DesignPanelValue::StrokeVariableWidth(Some(DesignVariableWidthStroke::custom(
            ordered_points,
        ))),
    );
    assert_eq!(
        node.stroke
            .as_ref()
            .expect("stroke")
            .variable_width
            .as_ref()
            .and_then(DesignVariableWidthStroke::points),
        Some(ordered_points.as_slice())
    );

    apply_design_property(
        &mut node,
        DesignPanelProperty::StrokeType,
        &DesignPanelValue::StrokeType(DesignStrokeType::ScatterBrush),
    );
    apply_design_property(
        &mut node,
        DesignPanelProperty::StrokeScatterGap,
        &DesignPanelValue::Number(0.24),
    );
    let DesignComplexStroke::ScatterBrush(scatter) =
        &node.stroke.as_ref().expect("stroke").complex_stroke
    else {
        panic!("scatter stroke")
    };
    assert_eq!(scatter.gap, 0.25, "out-of-domain gap is rejected");

    apply_design_property(
        &mut node,
        DesignPanelProperty::StrokeScatterGap,
        &DesignPanelValue::Number(0.75),
    );
    let DesignComplexStroke::ScatterBrush(scatter) =
        &node.stroke.as_ref().expect("stroke").complex_stroke
    else {
        panic!("scatter stroke")
    };
    assert_eq!(scatter.gap, 0.75);

    apply_design_property(
        &mut node,
        DesignPanelProperty::StrokeType,
        &DesignPanelValue::StrokeType(DesignStrokeType::Dynamic),
    );
    assert!(
        node.stroke
            .as_ref()
            .expect("stroke")
            .variable_width
            .is_none(),
        "dynamic transition removes incompatible variable-width data"
    );
    apply_design_property(
        &mut node,
        DesignPanelProperty::StrokeDynamicFrequency,
        &DesignPanelValue::Number(20.01),
    );
    let DesignComplexStroke::Dynamic(dynamic) =
        &node.stroke.as_ref().expect("stroke").complex_stroke
    else {
        panic!("dynamic stroke")
    };
    assert_eq!(dynamic.frequency, 1., "out-of-domain frequency is rejected");
}

#[test]
fn cancelled_story_vector_edits_restore_stable_vertex_snapshots() {
    let mut node = DesignPanelNode::new("vector", "Vector", DesignPanelNodeKind::Vector);
    node.vector_edit = Some(DesignVectorEditViewData::new([
        DesignVectorVertexViewData::new("vertex-b", 48., 24.)
            .with_corner_radius(Some(12.))
            .with_handle_mirroring(Some(DesignHandleMirroring::Angle))
            .selected(true),
        DesignVectorVertexViewData::new("vertex-a", 16., 24.)
            .with_topology(DesignVectorVertexTopology::Endpoint)
            .with_corner_radius(Some(4.))
            .with_handle_mirroring(Some(DesignHandleMirroring::None))
            .selected(true),
    ]));
    let original = node.vector_edit.clone();
    let node_id = node.id.clone();
    let ids = vec![
        SharedString::from("vertex-b"),
        SharedString::from("vertex-a"),
    ];
    let mut snapshots = HashMap::new();

    assert!(apply_story_vector_edit_phase(
        &mut node,
        &mut snapshots,
        &node_id,
        StoryVectorEditChange::Position {
            vertex_ids: ids.clone(),
            axis: DesignVectorCoordinateAxis::X,
            value: 48.,
        },
        DesignPanelEditPhase::Begin,
    ));
    assert_eq!(node.vector_edit, original, "Begin only captures a snapshot");
    node.vector_edit
        .as_mut()
        .expect("vector edit")
        .vertices
        .reverse();
    assert!(apply_story_vector_edit_phase(
        &mut node,
        &mut snapshots,
        &node_id,
        StoryVectorEditChange::Position {
            vertex_ids: ids.clone(),
            axis: DesignVectorCoordinateAxis::X,
            value: 80.,
        },
        DesignPanelEditPhase::Preview,
    ));
    assert!(
        node.vector_edit
            .as_ref()
            .expect("vector edit")
            .vertices
            .iter()
            .all(|vertex| (vertex.position.x - 80.).abs() < f32::EPSILON)
    );
    assert!(apply_story_vector_edit_phase(
        &mut node,
        &mut snapshots,
        &node_id,
        StoryVectorEditChange::Position {
            vertex_ids: ids.clone(),
            axis: DesignVectorCoordinateAxis::X,
            value: 999.,
        },
        DesignPanelEditPhase::Cancel,
    ));
    assert_eq!(node.vector_edit, original);

    for (preview, cancel) in [
        (
            StoryVectorEditChange::CornerRadius {
                vertex_ids: ids.clone(),
                radius: 20.,
            },
            StoryVectorEditChange::CornerRadius {
                vertex_ids: ids.clone(),
                radius: 999.,
            },
        ),
        (
            StoryVectorEditChange::HandleMirroring {
                vertex_ids: ids.clone(),
                mirroring: DesignHandleMirroring::AngleAndLength,
            },
            StoryVectorEditChange::HandleMirroring {
                vertex_ids: ids.clone(),
                mirroring: DesignHandleMirroring::None,
            },
        ),
        (
            StoryVectorEditChange::Selection(vec!["vertex-a".into()]),
            StoryVectorEditChange::Selection(Vec::new()),
        ),
    ] {
        assert!(apply_story_vector_edit_phase(
            &mut node,
            &mut snapshots,
            &node_id,
            preview.clone(),
            DesignPanelEditPhase::Begin,
        ));
        assert!(apply_story_vector_edit_phase(
            &mut node,
            &mut snapshots,
            &node_id,
            preview,
            DesignPanelEditPhase::Preview,
        ));
        assert_ne!(node.vector_edit, original);
        assert!(apply_story_vector_edit_phase(
            &mut node,
            &mut snapshots,
            &node_id,
            cancel,
            DesignPanelEditPhase::Cancel,
        ));
        assert_eq!(node.vector_edit, original);
        assert!(snapshots.is_empty());
    }
}

#[test]
fn cancelled_story_text_path_edit_restores_the_begin_snapshot() {
    let mut node = DesignPanelNode::new("text-path", "Text path", DesignPanelNodeKind::TextPath);
    let original = DesignTextPathStartData::new(2, 0.35).expect("valid start");
    node.text_path_start_data = Some(original);
    let node_id = node.id.clone();
    let mut snapshots = HashMap::new();

    apply_story_text_path_edit_phase(
        &mut node,
        &mut snapshots,
        &node_id,
        DesignTextPathStartData::new(4, 0.5).expect("valid begin"),
        DesignPanelEditPhase::Begin,
    );
    apply_story_text_path_edit_phase(
        &mut node,
        &mut snapshots,
        &node_id,
        DesignTextPathStartData::new(4, 0.75).expect("valid preview"),
        DesignPanelEditPhase::Preview,
    );
    assert_eq!(
        node.text_path_start_data,
        DesignTextPathStartData::new(4, 0.75)
    );
    apply_story_text_path_edit_phase(
        &mut node,
        &mut snapshots,
        &node_id,
        DesignTextPathStartData::new(9, 0.9).expect("valid cancel candidate"),
        DesignPanelEditPhase::Cancel,
    );

    assert_eq!(node.text_path_start_data, Some(original));
    assert!(snapshots.is_empty());
}

#[test]
fn cancelled_story_page_background_edit_restores_the_begin_snapshot() {
    let page_id = SharedString::from("page");
    let original = DesignColor::rgba(0x12, 0x34, 0x56, 0x78);
    let preview = DesignColor::rgba(0xde, 0xad, 0xbe, 0xef);
    let mut current = original;
    let mut snapshots = HashMap::new();

    apply_story_page_background_edit_phase(
        &mut current,
        &mut snapshots,
        &page_id,
        preview,
        DesignPanelEditPhase::Begin,
    );
    assert_eq!(current, original, "Begin only captures host state");
    apply_story_page_background_edit_phase(
        &mut current,
        &mut snapshots,
        &page_id,
        preview,
        DesignPanelEditPhase::Preview,
    );
    assert_eq!(current, preview);
    apply_story_page_background_edit_phase(
        &mut current,
        &mut snapshots,
        &page_id,
        DesignColor::BLACK,
        DesignPanelEditPhase::Cancel,
    );
    assert_eq!(current, original);
    assert!(snapshots.is_empty());

    apply_story_page_background_edit_phase(
        &mut current,
        &mut snapshots,
        &page_id,
        DesignColor::PURPLE,
        DesignPanelEditPhase::Commit,
    );
    assert_eq!(current, DesignColor::PURPLE);
    assert!(snapshots.is_empty());
}

#[test]
fn story_typography_reducer_applies_list_and_complete_decoration_details() {
    let mut node = DesignPanelNode::new("text", "Text", DesignPanelNodeKind::Text);
    apply_design_property(
        &mut node,
        DesignPanelProperty::TextList,
        &DesignPanelValue::TextList(DesignTextList::Bulleted),
    );
    apply_design_property(
        &mut node,
        DesignPanelProperty::ListSpacing,
        &DesignPanelValue::Number(12.),
    );
    apply_design_property(
        &mut node,
        DesignPanelProperty::TextDecoration,
        &DesignPanelValue::TextDecoration(DesignTextDecoration::Underline),
    );
    apply_design_property(
        &mut node,
        DesignPanelProperty::TextDecorationStyle,
        &DesignPanelValue::TextDecorationStyle(DesignTextDecorationStyle::Dotted),
    );
    apply_design_property(
        &mut node,
        DesignPanelProperty::TextDecorationOffset,
        &DesignPanelValue::TextDecorationMetric(DesignTextDecorationMetric::Pixels(-2.)),
    );
    apply_design_property(
        &mut node,
        DesignPanelProperty::TextDecorationThickness,
        &DesignPanelValue::TextDecorationMetric(DesignTextDecorationMetric::Percent(125.)),
    );
    apply_design_property(
        &mut node,
        DesignPanelProperty::TextDecorationColor,
        &DesignPanelValue::TextDecorationColor(DesignTextDecorationColor::Solid(DesignColor::BLUE)),
    );
    apply_design_property(
        &mut node,
        DesignPanelProperty::TextDecorationSkipInk,
        &DesignPanelValue::Bool(false),
    );

    let typography = node.typography.as_ref().expect("text typography");
    assert_eq!(typography.list_spacing, 12.);
    assert_eq!(
        typography.decoration_details,
        Some(DesignTextDecorationDetails {
            style: DesignTextDecorationStyle::Dotted,
            offset: DesignTextDecorationMetric::Pixels(-2.),
            thickness: DesignTextDecorationMetric::Percent(125.),
            color: DesignTextDecorationColor::Solid(DesignColor::BLUE),
            skip_ink: false,
        })
    );
}

#[test]
fn story_smart_selection_reducer_preserves_mixed_values_and_exact_transaction_targets() {
    let target = DesignPanelTarget::Nodes {
        node_ids: vec!["second".into(), "first".into()],
    };
    let stale_reordered_target = DesignPanelTarget::Nodes {
        node_ids: vec!["first".into(), "second".into()],
    };
    let projection = DesignSmartSelectionViewData::two_dimensional(
        target.clone(),
        DesignSmartSelectionSpacingViewData::uniform(24.),
        DesignSmartSelectionSpacingViewData::mixed(),
    );
    let mut values = HashMap::from([
        (
            DesignSmartSelectionAxis::Horizontal,
            DesignSmartSelectionSpacingValue::Uniform(24.),
        ),
        (
            DesignSmartSelectionAxis::Vertical,
            DesignSmartSelectionSpacingValue::Mixed,
        ),
    ]);
    let mut snapshots = HashMap::new();

    assert!(apply_story_smart_selection_spacing_edit(
        &mut values,
        &mut snapshots,
        Some(&projection),
        true,
        StorySmartSelectionSpacingEdit {
            target: &target,
            axis: DesignSmartSelectionAxis::Horizontal,
            value: 24.,
            phase: DesignPanelEditPhase::Begin,
        },
    ));
    assert_eq!(
        values[&DesignSmartSelectionAxis::Horizontal],
        DesignSmartSelectionSpacingValue::Uniform(24.),
        "Begin captures host state without mutating the controlled value"
    );
    assert!(apply_story_smart_selection_spacing_edit(
        &mut values,
        &mut snapshots,
        Some(&projection),
        true,
        StorySmartSelectionSpacingEdit {
            target: &target,
            axis: DesignSmartSelectionAxis::Horizontal,
            value: 40.,
            phase: DesignPanelEditPhase::Preview,
        },
    ));
    assert_eq!(
        values[&DesignSmartSelectionAxis::Horizontal],
        DesignSmartSelectionSpacingValue::Uniform(40.)
    );
    assert!(!apply_story_smart_selection_spacing_edit(
        &mut values,
        &mut snapshots,
        Some(&projection),
        true,
        StorySmartSelectionSpacingEdit {
            target: &stale_reordered_target,
            axis: DesignSmartSelectionAxis::Horizontal,
            value: 60.,
            phase: DesignPanelEditPhase::Preview,
        },
    ));
    assert_eq!(
        values[&DesignSmartSelectionAxis::Horizontal],
        DesignSmartSelectionSpacingValue::Uniform(40.)
    );
    assert!(apply_story_smart_selection_spacing_edit(
        &mut values,
        &mut snapshots,
        None,
        false,
        StorySmartSelectionSpacingEdit {
            target: &target,
            axis: DesignSmartSelectionAxis::Horizontal,
            value: 24.,
            phase: DesignPanelEditPhase::Cancel,
        },
    ));
    assert_eq!(
        values[&DesignSmartSelectionAxis::Horizontal],
        DesignSmartSelectionSpacingValue::Uniform(24.),
        "an exact active Cancel restores the host snapshot after context access changes"
    );
    assert!(snapshots.is_empty());

    assert!(apply_story_smart_selection_spacing_edit(
        &mut values,
        &mut snapshots,
        Some(&projection),
        true,
        StorySmartSelectionSpacingEdit {
            target: &target,
            axis: DesignSmartSelectionAxis::Vertical,
            value: 0.,
            phase: DesignPanelEditPhase::Begin,
        },
    ));
    assert_eq!(
        values[&DesignSmartSelectionAxis::Vertical],
        DesignSmartSelectionSpacingValue::Mixed
    );
    assert!(apply_story_smart_selection_spacing_edit(
        &mut values,
        &mut snapshots,
        Some(&projection),
        true,
        StorySmartSelectionSpacingEdit {
            target: &target,
            axis: DesignSmartSelectionAxis::Vertical,
            value: 18.,
            phase: DesignPanelEditPhase::Preview,
        },
    ));
    assert!(apply_story_smart_selection_spacing_edit(
        &mut values,
        &mut snapshots,
        Some(&projection),
        true,
        StorySmartSelectionSpacingEdit {
            target: &target,
            axis: DesignSmartSelectionAxis::Vertical,
            value: 18.,
            phase: DesignPanelEditPhase::Commit,
        },
    ));
    assert_eq!(
        values[&DesignSmartSelectionAxis::Vertical],
        DesignSmartSelectionSpacingValue::Uniform(18.)
    );
    assert!(snapshots.is_empty());
    assert!(!apply_story_smart_selection_spacing_edit(
        &mut values,
        &mut snapshots,
        Some(&projection),
        true,
        StorySmartSelectionSpacingEdit {
            target: &target,
            axis: DesignSmartSelectionAxis::Vertical,
            value: f32::NAN,
            phase: DesignPanelEditPhase::Commit,
        },
    ));
}

#[test]
fn story_smart_selection_arrange_validation_rejects_stale_disabled_and_read_only_requests() {
    let target = DesignPanelTarget::Nodes {
        node_ids: vec!["second".into(), "first".into()],
    };
    let reordered = DesignPanelTarget::Nodes {
        node_ids: vec!["first".into(), "second".into()],
    };
    let projection =
        DesignSmartSelectionViewData::new(target.clone(), DesignSmartSelectionKind::None)
            .with_operation(
                DesignSmartSelectionOperation::TidyUp,
                DesignSmartSelectionAvailability::Available,
            )
            .with_operation(
                DesignSmartSelectionOperation::DistributeHorizontal,
                DesignSmartSelectionAvailability::disabled("Not aligned"),
            );
    assert!(story_smart_selection_operation_is_current(
        Some(&projection),
        true,
        &target,
        DesignSmartSelectionOperation::TidyUp,
    ));
    assert!(!story_smart_selection_operation_is_current(
        Some(&projection),
        true,
        &reordered,
        DesignSmartSelectionOperation::TidyUp,
    ));
    assert!(!story_smart_selection_operation_is_current(
        Some(&projection),
        true,
        &target,
        DesignSmartSelectionOperation::DistributeHorizontal,
    ));
    assert!(!story_smart_selection_operation_is_current(
        Some(&projection.clone().read_only("Locked")),
        true,
        &target,
        DesignSmartSelectionOperation::TidyUp,
    ));
    assert!(!story_smart_selection_operation_is_current(
        Some(&projection),
        false,
        &target,
        DesignSmartSelectionOperation::TidyUp,
    ));
}

#[test]
fn story_add_auto_layout_converts_a_group_only_for_the_exact_current_target() {
    let mut nodes = vec![DesignPanelNode::new(
        "group",
        "Navigation group",
        DesignPanelNodeKind::Group,
    )];
    let target = DesignPanelTarget::Nodes {
        node_ids: vec!["group".into()],
    };
    let stale = DesignPanelTarget::Nodes {
        node_ids: vec!["other".into()],
    };
    let original = nodes.clone();
    let mut next_id = 10;

    assert!(
        apply_story_add_auto_layout(&mut nodes, &target, Some(&stale), true, &mut next_id,)
            .is_none()
    );
    assert_eq!(nodes, original, "a stale target must not mutate mock state");
    assert!(
        apply_story_add_auto_layout(&mut nodes, &target, Some(&target), false, &mut next_id,)
            .is_none()
    );
    assert_eq!(
        nodes, original,
        "view-only inspection must not mutate mock state"
    );

    let (selected_index, echo) =
        apply_story_add_auto_layout(&mut nodes, &target, Some(&target), true, &mut next_id)
            .expect("the exact editable Group target is eligible");
    assert_eq!(selected_index, 0);
    assert_eq!(
        echo,
        StoryAddAutoLayoutEcho::Converted {
            node_id: "group".into(),
        }
    );
    assert_eq!(nodes[0].kind, DesignPanelNodeKind::Frame);
    assert_eq!(
        nodes[0].layout.as_ref().expect("echoed layout").mode,
        DesignLayoutMode::Vertical
    );
}

#[test]
fn story_add_auto_layout_wraps_an_ordered_multi_selection_and_rejects_reordering() {
    let mut nodes = vec![
        DesignPanelNode::new("first", "First", DesignPanelNodeKind::Rectangle),
        DesignPanelNode::new("second", "Second", DesignPanelNodeKind::Ellipse),
    ];
    let action_target = DesignPanelTarget::Nodes {
        node_ids: vec!["second".into(), "first".into()],
    };
    let reordered_current = DesignPanelTarget::Nodes {
        node_ids: vec!["first".into(), "second".into()],
    };
    let original = nodes.clone();
    let mut next_id = 20;

    assert!(
        apply_story_add_auto_layout(
            &mut nodes,
            &action_target,
            Some(&reordered_current),
            true,
            &mut next_id,
        )
        .is_none()
    );
    assert_eq!(nodes, original);
    assert_eq!(next_id, 20);

    let (selected_index, echo) = apply_story_add_auto_layout(
        &mut nodes,
        &action_target,
        Some(&action_target),
        true,
        &mut next_id,
    )
    .expect("the exact ordered multiple selection is eligible");
    assert_eq!(selected_index, 2);
    assert_eq!(
        echo,
        StoryAddAutoLayoutEcho::Wrapped {
            wrapper_id: "storybook-auto-layout-20".into(),
            child_ids: vec!["second".into(), "first".into()],
        }
    );
    assert_eq!(nodes[0], original[0]);
    assert_eq!(nodes[1], original[1]);
    assert_eq!(nodes[2].kind, DesignPanelNodeKind::Frame);
    assert_ne!(
        nodes[2].layout.as_ref().expect("wrapper layout").mode,
        DesignLayoutMode::None
    );
}

#[test]
fn story_menu_preview_reducer_balances_exact_payloads_and_rejects_stale_or_bound_begin() {
    let node = DesignPanelNode::new("rectangle", "Rectangle", DesignPanelNodeKind::Rectangle);
    let nodes = vec![node.clone()];
    let target = DesignPanelTarget::Nodes {
        node_ids: vec!["rectangle".into()],
    };
    let preview = DesignMenuPreview::NodeProperty {
        target: target.clone(),
        property: DesignPanelProperty::BlendMode,
        original: DesignPanelValue::BlendMode(DesignBlendMode::Normal),
        candidate: DesignPanelValue::BlendMode(DesignBlendMode::Multiply),
    };
    let mut active = None;
    let mut bindings = HashMap::new();

    assert!(!apply_story_menu_preview(
        &mut active,
        StoryMenuPreviewContext {
            nodes: &nodes,
            bindings: &bindings,
            current_target: Some(&target),
            current_paint_target: None,
            can_edit: false,
        },
        &preview,
        DesignMenuPreviewPhase::Begin,
    ));
    assert!(apply_story_menu_preview(
        &mut active,
        StoryMenuPreviewContext {
            nodes: &nodes,
            bindings: &bindings,
            current_target: Some(&target),
            current_paint_target: None,
            can_edit: true,
        },
        &preview,
        DesignMenuPreviewPhase::Begin,
    ));
    assert_eq!(active.as_ref(), Some(&preview));
    assert!(!apply_story_menu_preview(
        &mut active,
        StoryMenuPreviewContext {
            nodes: &nodes,
            bindings: &bindings,
            current_target: Some(&target),
            current_paint_target: None,
            can_edit: true,
        },
        &preview,
        DesignMenuPreviewPhase::Begin,
    ));
    let stale_end = DesignMenuPreview::NodeProperty {
        target: target.clone(),
        property: DesignPanelProperty::BlendMode,
        original: DesignPanelValue::BlendMode(DesignBlendMode::Normal),
        candidate: DesignPanelValue::BlendMode(DesignBlendMode::Screen),
    };
    assert!(!apply_story_menu_preview(
        &mut active,
        StoryMenuPreviewContext {
            nodes: &nodes,
            bindings: &bindings,
            current_target: Some(&target),
            current_paint_target: None,
            can_edit: true,
        },
        &stale_end,
        DesignMenuPreviewPhase::End,
    ));
    assert!(apply_story_menu_preview(
        &mut active,
        StoryMenuPreviewContext {
            nodes: &nodes,
            bindings: &bindings,
            current_target: Some(&target),
            current_paint_target: None,
            can_edit: false,
        },
        &preview,
        DesignMenuPreviewPhase::End,
    ));
    assert_eq!(active, None);
    assert_eq!(nodes, vec![node], "preview state must remain transient");

    bindings.insert(
        ("rectangle".into(), DesignPanelProperty::BlendMode),
        DesignPanelPropertyBinding::new(
            "blend-variable",
            "Blend",
            fanta_gpui::prelude::DesignPanelBindingKind::Variable,
            DesignPanelValue::BlendMode(DesignBlendMode::Normal),
        ),
    );
    assert!(!apply_story_menu_preview(
        &mut active,
        StoryMenuPreviewContext {
            nodes: &nodes,
            bindings: &bindings,
            current_target: Some(&target),
            current_paint_target: None,
            can_edit: true,
        },
        &preview,
        DesignMenuPreviewPhase::Begin,
    ));
}

#[test]
fn story_menu_preview_reducer_uses_exact_paint_scope_id_index_and_effect_id() {
    let mut node = DesignPanelNode::new("rectangle", "Rectangle", DesignPanelNodeKind::Rectangle);
    node.fills = vec![
        DesignPaint::solid(DesignColor::BLUE).with_id("paint-a"),
        DesignPaint::solid(DesignColor::PURPLE).with_id("paint-b"),
    ];
    node.effects = vec![
        DesignEffect::new(DesignEffectKind::DropShadow).with_id("effect-a"),
        DesignEffect::new(DesignEffectKind::LayerBlur).with_id("effect-b"),
    ];
    let target = DesignPanelTarget::Nodes {
        node_ids: vec!["rectangle".into()],
    };
    let bindings = HashMap::new();
    let paint_preview = DesignMenuPreview::PaintProperty {
        node_id: "rectangle".into(),
        collection: DesignPanelCollection::Fill,
        target: DesignPaintTarget::WholeLayer,
        paint_id: "paint-a".into(),
        index: 0,
        property: DesignPaintProperty::BlendMode,
        original: DesignPaintValue::BlendMode(DesignBlendMode::Normal),
        candidate: DesignPaintValue::BlendMode(DesignBlendMode::Screen),
    };
    let mut active = None;
    assert!(apply_story_menu_preview(
        &mut active,
        StoryMenuPreviewContext {
            nodes: &[node.clone()],
            bindings: &bindings,
            current_target: Some(&target),
            current_paint_target: Some(DesignPaintTarget::WholeLayer),
            can_edit: true,
        },
        &paint_preview,
        DesignMenuPreviewPhase::Begin,
    ));
    assert!(apply_story_menu_preview(
        &mut active,
        StoryMenuPreviewContext {
            nodes: &[node.clone()],
            bindings: &bindings,
            current_target: Some(&target),
            current_paint_target: None,
            can_edit: true,
        },
        &paint_preview,
        DesignMenuPreviewPhase::End,
    ));

    let stale_paint = DesignMenuPreview::PaintProperty {
        node_id: "rectangle".into(),
        collection: DesignPanelCollection::Fill,
        target: DesignPaintTarget::WholeLayer,
        paint_id: "paint-a".into(),
        index: 1,
        property: DesignPaintProperty::BlendMode,
        original: DesignPaintValue::BlendMode(DesignBlendMode::Normal),
        candidate: DesignPaintValue::BlendMode(DesignBlendMode::Screen),
    };
    assert!(!apply_story_menu_preview(
        &mut active,
        StoryMenuPreviewContext {
            nodes: &[node.clone()],
            bindings: &bindings,
            current_target: Some(&target),
            current_paint_target: Some(DesignPaintTarget::WholeLayer),
            can_edit: true,
        },
        &stale_paint,
        DesignMenuPreviewPhase::Begin,
    ));

    let effect_preview = DesignMenuPreview::EffectProperty {
        node_id: "rectangle".into(),
        effect_id: "effect-a".into(),
        index: 0,
        property: DesignPanelProperty::EffectKind(0),
        original: DesignPanelValue::EffectKind(DesignEffectKind::DropShadow),
        candidate: DesignPanelValue::EffectKind(DesignEffectKind::InnerShadow),
    };
    node.effects.swap(0, 1);
    assert!(!apply_story_menu_preview(
        &mut active,
        StoryMenuPreviewContext {
            nodes: &[node],
            bindings: &bindings,
            current_target: Some(&target),
            current_paint_target: None,
            can_edit: true,
        },
        &effect_preview,
        DesignMenuPreviewPhase::Begin,
    ));
}

fn setup_gallery_storybook(cx: &mut TestAppContext) -> (Entity<Storybook>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_component::init(cx);
        fanta_gpui::init(cx);
        Theme::change(ThemeMode::Light, None, cx);
    });
    let storybook_slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let captured_storybook = storybook_slot.clone();
    let (_, visual_cx) = cx.add_window_view(move |window, cx| {
        let storybook = cx.new(|cx| Storybook::new(window, cx));
        *captured_storybook.borrow_mut() = Some(storybook.clone());
        Root::new(storybook, window, cx)
    });
    let storybook = storybook_slot
        .borrow_mut()
        .take()
        .expect("test Storybook should be installed");
    visual_cx.update(|_, app| {
        storybook.update(app, |storybook, cx| {
            storybook.launch_mode = StorybookLaunchMode::Gallery;
            cx.notify();
        });
    });
    visual_cx.simulate_resize(size(px(1440.), px(900.)));
    visual_cx.run_until_parked();
    (storybook, visual_cx)
}

#[gpui::test]
fn gallery_viewport_harness_presets_and_scrubs_resize_the_surface(cx: &mut TestAppContext) {
    let (storybook, cx) = setup_gallery_storybook(cx);
    cx.update(|window, app| {
        storybook.update(app, |storybook, cx| {
            storybook.activate_gallery_story(StoryKind::Pages, window, cx);
        });
    });
    cx.run_until_parked();

    // Pages seeds fluid like every story; pin its registered Default
    // preset so the scrub assertions continue from an authored size.
    cx.update(|_, app| {
        storybook.update(app, |storybook, cx| {
            assert!(
                storybook.story_viewport.fluid,
                "Pages must seed its registered fluid mode"
            );
            storybook.story_viewport.apply_preset(337., 716.);
            cx.notify();
        });
    });
    cx.run_until_parked();
    let surface = cx
        .debug_bounds("storybook-gallery-story-surface")
        .expect("the Pages story surface should render");
    assert_eq!(surface.size.width, px(337.));
    assert_eq!(surface.size.height, px(716.));
    assert!(
        cx.debug_bounds("story-viewport-readout").is_some(),
        "the harness must render its live pixel readout"
    );

    // Preset chips repin the viewport to another registered natural size.
    cx.update(|_, app| {
        storybook.update(app, |storybook, cx| {
            storybook.story_viewport.apply_preset(240., 716.);
            cx.notify();
        });
    });
    cx.run_until_parked();
    let narrow = cx
        .debug_bounds("storybook-gallery-story-surface")
        .expect("the narrowed surface should render");
    assert_eq!(narrow.size.width, px(240.));

    // Dragging the width handle scrubs the surface live.
    let handle = cx
        .debug_bounds("story-viewport-width-handle")
        .expect("the width scrub handle should render");
    cx.simulate_event(MouseDownEvent {
        position: handle.center(),
        button: MouseButton::Left,
        ..Default::default()
    });
    cx.run_until_parked();
    cx.simulate_event(MouseMoveEvent {
        position: handle.center() + point(px(64.), px(0.)),
        pressed_button: Some(MouseButton::Left),
        ..Default::default()
    });
    cx.simulate_event(MouseUpEvent {
        position: handle.center() + point(px(64.), px(0.)),
        button: MouseButton::Left,
        ..Default::default()
    });
    cx.run_until_parked();
    let scrubbed = cx
        .debug_bounds("storybook-gallery-story-surface")
        .expect("the scrubbed surface should render");
    assert_eq!(scrubbed.size.width, px(304.));

    // Dragging the height handle scrubs the other axis.
    // The height handle lays out below the visible canvas; scroll the
    // gallery canvas down first, like a user would.
    cx.update(|_, app| {
        storybook.update(app, |storybook, cx| {
            storybook
                .gallery_story_scroll_handle
                .set_offset(point(px(0.), px(-240.)));
            cx.notify();
        });
    });
    cx.run_until_parked();
    let handle = cx
        .debug_bounds("story-viewport-height-handle")
        .expect("the height scrub handle should render");
    let grab = handle.center();
    cx.simulate_event(MouseDownEvent {
        position: grab,
        button: MouseButton::Left,
        ..Default::default()
    });
    cx.run_until_parked();
    cx.simulate_event(MouseMoveEvent {
        position: grab + point(px(0.), px(-40.)),
        pressed_button: Some(MouseButton::Left),
        ..Default::default()
    });
    cx.simulate_event(MouseUpEvent {
        position: grab + point(px(0.), px(-40.)),
        button: MouseButton::Left,
        ..Default::default()
    });
    cx.run_until_parked();
    let scrubbed = cx
        .debug_bounds("storybook-gallery-story-surface")
        .expect("the height-scrubbed surface should render");
    assert_eq!(scrubbed.size.height, px(676.));

    // Activating another story reseeds the viewport from the registry:
    // the pinned scrub state is dropped for Prototype's fluid mode.
    cx.update(|window, app| {
        storybook.update(app, |storybook, cx| {
            storybook.activate_gallery_story(StoryKind::Prototype, window, cx);
        });
    });
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("storybook-gallery-story-surface").is_some(),
        "the Prototype surface should render"
    );
    cx.update(|_, app| {
        storybook.update(app, |storybook, _| {
            assert!(
                storybook.story_viewport.fluid,
                "activating another story must reseed its registered fluid mode"
            );
        });
    });
}

#[gpui::test]
fn gallery_fluid_viewports_fill_the_measured_story_area(cx: &mut TestAppContext) {
    let (storybook, cx) = setup_gallery_storybook(cx);
    cx.update(|window, app| {
        storybook.update(app, |storybook, cx| {
            storybook.activate_gallery_story(StoryKind::Welcome, window, cx);
        });
    });
    cx.run_until_parked();

    let padding = screens::viewport::GALLERY_CANVAS_PADDING;
    let handle = screens::viewport::VIEWPORT_HANDLE_THICKNESS;
    let assert_fills = |cx: &mut VisualTestContext, label: &str| {
        let canvas_area = cx
            .debug_bounds("storybook-gallery-canvas")
            .expect("the gallery canvas should render");
        let surface = cx
            .debug_bounds("storybook-gallery-story-surface")
            .expect("the fluid story surface should render");
        let expected_width = f32::from(canvas_area.size.width) - 2. * padding - handle;
        assert!(
            (f32::from(surface.size.width) - expected_width).abs() <= 1.5,
            "{label}: a fluid surface must fill the story area width up to the scrub \
             handle (got {:?}, expected {expected_width})",
            surface.size.width,
        );
        let expected_bottom = f32::from(canvas_area.bottom()) - padding - handle;
        assert!(
            (f32::from(surface.bottom()) - expected_bottom).abs() <= 1.5,
            "{label}: a fluid surface must fill the story area height down to the scrub \
             handle (got {:?}, expected {expected_bottom})",
            surface.bottom(),
        );
    };
    assert_fills(cx, "1440x900");

    // Both axes keep tracking as the window shrinks and grows again.
    cx.simulate_resize(size(px(1000.), px(700.)));
    cx.run_until_parked();
    assert_fills(cx, "1000x700");
    cx.simulate_resize(size(px(1360.), px(860.)));
    cx.run_until_parked();
    assert_fills(cx, "1360x860");

    // Fluid mode itself never pins a fixed size; the surface probe feeds
    // the live readout instead.
    cx.update(|_, app| {
        storybook.update(app, |storybook, _| {
            assert!(
                storybook.story_viewport.fluid,
                "window resizes must never pin the fluid viewport"
            );
            assert!(
                storybook.story_viewport.available.is_some(),
                "the area probe should have recorded the story area"
            );
        });
    });
}

#[gpui::test]
fn design_story_stacks_its_harness_at_the_minimum_viewport(cx: &mut TestAppContext) {
    let (storybook, cx) = setup_gallery_storybook(cx);
    cx.update(|window, app| {
        storybook.update(app, |storybook, cx| {
            storybook.activate_gallery_story(StoryKind::Design, window, cx);
        });
    });
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("design-fixture-rail-scroll").is_some(),
        "a wide Design story keeps the fixture rail beside the inspector"
    );

    cx.update(|_, app| {
        storybook.update(app, |storybook, cx| {
            storybook
                .story_viewport
                .apply_preset(DESIGN_STORY_MIN_WIDTH, DESIGN_STORY_MIN_HEIGHT);
            cx.notify();
        });
    });
    cx.run_until_parked();

    let surface = cx
        .debug_bounds("storybook-gallery-story-surface")
        .expect("the Design story surface should render at its floor");
    assert_eq!(surface.size.width, px(DESIGN_STORY_MIN_WIDTH));
    assert_eq!(surface.size.height, px(DESIGN_STORY_MIN_HEIGHT));
    assert!(
        cx.debug_bounds("design-fixture-rail-scroll").is_none(),
        "the stacked harness must fold the fixture rail away"
    );
    let panel = cx
        .debug_bounds("design-harness-panel")
        .expect("the inspector stays mounted at the floor");
    assert!(
        f32::from(panel.size.width) + 0.5 >= 320.,
        "the inspector must keep its own 320 px minimum at the story floor \
         (got {:?})",
        panel.size.width
    );
    assert!(
        panel.right() <= surface.right() + px(1.),
        "the inspector must fit inside the story surface"
    );

    // The folded story controls expand on demand and reveal the full
    // scenario/node matrix in a scrollable section.
    let toggle = cx
        .debug_bounds("design-harness-controls-toggle")
        .expect("the stacked harness offers its controls toggle");
    assert!(
        cx.debug_bounds("design-harness-controls-scroll").is_none(),
        "the story controls stay collapsed by default at the floor"
    );
    cx.simulate_click(toggle.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("design-harness-controls-scroll").is_some(),
        "expanding must reveal the scrollable story controls"
    );
    assert!(
        cx.debug_bounds("design-node-variation-matrix-heading")
            .is_some(),
        "the node matrix folds into the expanded controls"
    );

    // The bespoke width resizer keeps working at the floor: dragging the
    // handle updates the stored width while the rendered panel stays
    // capped inside the surface.
    let handle = cx
        .debug_bounds("design-panel-resize-handle")
        .expect("the width resizer survives the stacked layout");
    cx.simulate_event(MouseDownEvent {
        position: handle.center(),
        button: MouseButton::Left,
        ..Default::default()
    });
    cx.run_until_parked();
    cx.simulate_event(MouseMoveEvent {
        position: handle.center() + point(px(30.), px(0.)),
        pressed_button: Some(MouseButton::Left),
        ..Default::default()
    });
    cx.simulate_event(MouseUpEvent {
        position: handle.center() + point(px(30.), px(0.)),
        button: MouseButton::Left,
        ..Default::default()
    });
    cx.run_until_parked();
    cx.update(|_, app| {
        storybook.update(app, |storybook, _| {
            assert_eq!(
                storybook.design_screen.panel_width, 442.,
                "the drag must scrub the stored inspector width"
            );
        });
    });
    let panel = cx
        .debug_bounds("design-harness-panel")
        .expect("the inspector survives the resize interaction");
    assert!(
        panel.right() <= surface.right() + px(1.),
        "the resized inspector must stay capped inside the story surface"
    );
}

#[gpui::test]
fn variables_story_reflows_to_its_minimum_viewport(cx: &mut TestAppContext) {
    let (storybook, cx) = setup_gallery_storybook(cx);
    cx.update(|window, app| {
        storybook.update(app, |storybook, cx| {
            storybook.activate_gallery_story(StoryKind::Variables, window, cx);
        });
    });
    cx.run_until_parked();

    cx.update(|_, app| {
        storybook.update(app, |storybook, cx| {
            storybook
                .story_viewport
                .apply_preset(VARIABLES_SCREEN_MIN_WIDTH, VARIABLES_SCREEN_MIN_HEIGHT);
            cx.notify();
        });
    });
    // The page measures its own width during a draw and defers the
    // shared-column update past it, so park twice.
    cx.run_until_parked();
    cx.run_until_parked();

    let surface = cx
        .debug_bounds("storybook-gallery-story-surface")
        .expect("the Variables story surface should render at its floor");
    assert_eq!(surface.size.width, px(VARIABLES_SCREEN_MIN_WIDTH));
    assert_eq!(surface.size.height, px(VARIABLES_SCREEN_MIN_HEIGHT));
    let sidebar = cx
        .debug_bounds("variables-sidebar")
        .expect("the sidebar stays mounted at the floor");
    assert_eq!(
        sidebar.size.width,
        px(180.),
        "the sidebar compresses to its Phase G floor"
    );
    let name_header = cx
        .debug_bounds("variables-name-header")
        .expect("the name header stays mounted at the floor");
    assert_eq!(
        name_header.size.width,
        px(140.),
        "the name column compresses to its Phase G floor"
    );
    let add_mode = cx
        .debug_bounds("variables-add-mode")
        .expect("the add-mode cell stays mounted at the floor");
    assert!(
        add_mode.right() <= surface.right() + px(1.),
        "the add-mode cell must stay pinned inside the page while the \
         mode columns clip"
    );
    let create_variable = cx
        .debug_bounds("variables-create-variable")
        .expect("the create-variable row stays mounted at the floor");
    assert!(
        create_variable.bottom() <= surface.bottom() + px(1.),
        "the create-variable row must stay inside the floor-height page"
    );

    // Interaction at the floor: the pinned header cell still adds a mode
    // through the host adapter.
    cx.simulate_click(add_mode.center(), Modifiers::none());
    cx.run_until_parked();
    cx.update(|_, app| {
        storybook.update(app, |storybook, _| {
            assert_eq!(
                storybook.variables_screen.view_data.modes.len(),
                3,
                "the add-mode intent must reach the story reducer"
            );
            assert!(
                storybook
                    .variables_screen
                    .last_action
                    .contains("Added Mode 3"),
                "unexpected last action: {}",
                storybook.variables_screen.last_action
            );
        });
    });
    let add_mode = cx
        .debug_bounds("variables-add-mode")
        .expect("the add-mode cell survives adding a third mode");
    assert!(
        add_mode.right() <= surface.right() + px(1.),
        "the add-mode cell stays pinned after the table gains a mode column"
    );
}

#[gpui::test]
fn variables_story_supplies_the_selected_collections_table(cx: &mut TestAppContext) {
    let (storybook, cx) = setup_gallery_storybook(cx);
    cx.update(|window, app| {
        storybook.update(app, |storybook, cx| {
            storybook.activate_gallery_story(StoryKind::Variables, window, cx);
        });
    });
    cx.run_until_parked();

    assert!(cx.debug_bounds("variables-value-color-mode-1").is_some());
    let second_collection = cx
        .debug_bounds("variables-collection-collection-2")
        .expect("the second collection should render");
    cx.simulate_click(second_collection.center(), Modifiers::none());
    cx.run_until_parked();

    assert!(cx.debug_bounds("variables-value-color-mode-1").is_none());
    assert!(cx.debug_bounds("variables-empty-create").is_some());
    cx.update(|_, app| {
        storybook.update(app, |storybook, _| {
            assert_eq!(
                storybook.variables_screen.view_data.selected_collection_id,
                "collection-2"
            );
            assert!(storybook.variables_screen.view_data.variables.is_empty());
        });
    });

    let first_collection = cx
        .debug_bounds("variables-collection-collection-1")
        .expect("the first collection should remain available");
    cx.simulate_click(first_collection.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(cx.debug_bounds("variables-value-color-mode-1").is_some());
}

#[gpui::test]
fn gallery_sidebar_auto_collapses_below_the_narrow_threshold(cx: &mut TestAppContext) {
    let (storybook, cx) = setup_gallery_storybook(cx);
    assert!(
        cx.debug_bounds("storybook-gallery-sidebar").is_some(),
        "a wide gallery window keeps the sidebar expanded"
    );

    cx.simulate_resize(size(px(720.), px(640.)));
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("storybook-gallery-sidebar").is_none(),
        "the sidebar must auto-collapse below the narrow window threshold"
    );
    assert!(
        cx.debug_bounds("storybook-sidebar-toggle").is_some(),
        "a collapsed sidebar keeps its reopen affordance in the top bar"
    );

    cx.simulate_resize(size(px(1440.), px(900.)));
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("storybook-gallery-sidebar").is_some(),
        "the sidebar returns when the window grows past the threshold"
    );

    // An explicit collapse pins the sidebar closed across window growth.
    let toggle = cx
        .debug_bounds("storybook-sidebar-toggle")
        .expect("the sidebar toggle should render");
    cx.simulate_click(toggle.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("storybook-gallery-sidebar").is_none(),
        "the toggle must collapse the sidebar"
    );
    cx.simulate_resize(size(px(1500.), px(900.)));
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("storybook-gallery-sidebar").is_none(),
        "an explicitly collapsed sidebar stays collapsed as the window grows"
    );

    // Reopening at a wide width returns the sidebar to automatic mode.
    let toggle = cx
        .debug_bounds("storybook-sidebar-toggle")
        .expect("the sidebar toggle should render while collapsed");
    cx.simulate_click(toggle.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("storybook-gallery-sidebar").is_some(),
        "the toggle must reopen the sidebar"
    );
    assert_eq!(
        cx.update(|_, app| storybook.read(app).sidebar_pin),
        SidebarPin::Auto,
        "reopening at a wide width must return the sidebar to auto-collapse mode"
    );

    // At the registered 320x240 window minimum the shell stays usable:
    // the sidebar auto-collapses again and the story surface stays
    // mounted inside the scrollable canvas instead of being clipped away.
    cx.simulate_resize(size(px(320.), px(240.)));
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("storybook-gallery-sidebar").is_none(),
        "the sidebar must collapse at the window minimum"
    );
    assert!(
        cx.debug_bounds("storybook-gallery-canvas").is_some(),
        "the story canvas must survive the window minimum"
    );
    assert!(
        cx.debug_bounds("storybook-gallery-story-surface").is_some(),
        "the story surface must stay mounted at the window minimum"
    );
}

#[gpui::test]
fn gallery_knobs_sections_collapse_below_the_narrow_threshold(cx: &mut TestAppContext) {
    let (storybook, cx) = setup_gallery_storybook(cx);
    cx.update(|window, app| {
        storybook.update(app, |storybook, cx| {
            storybook.activate_gallery_story(StoryKind::Pages, window, cx);
        });
    });
    cx.run_until_parked();

    // The knobs section lays out below the fixed Pages surface; scroll
    // the canvas down to it like a user would.
    fn scroll_to_bottom(storybook: &Entity<Storybook>, cx: &mut VisualTestContext) {
        cx.update(|_, app| {
            storybook.update(app, |storybook, cx| {
                storybook
                    .gallery_story_scroll_handle
                    .set_offset(point(px(0.), px(-4000.)));
                cx.notify();
            });
        });
        cx.run_until_parked();
    }
    scroll_to_bottom(&storybook, cx);
    assert!(
        cx.debug_bounds("pages-story-knobs").is_some(),
        "a wide window shows the story's knobs panel by default"
    );
    assert!(
        cx.debug_bounds("storybook-knobs-toggle").is_some(),
        "the knobs section renders its collapse toggle"
    );

    cx.simulate_resize(size(px(720.), px(640.)));
    cx.run_until_parked();
    scroll_to_bottom(&storybook, cx);
    assert!(
        cx.debug_bounds("pages-story-knobs").is_none(),
        "a narrow window collapses the knobs section by default"
    );
    let toggle = cx
        .debug_bounds("storybook-knobs-toggle")
        .expect("the collapsed knobs section keeps its toggle");
    cx.simulate_click(toggle.center(), Modifiers::none());
    cx.run_until_parked();
    scroll_to_bottom(&storybook, cx);
    assert!(
        cx.debug_bounds("pages-story-knobs").is_some(),
        "the toggle must reopen the knobs at a narrow width"
    );
    assert_eq!(
        cx.update(|_, app| storybook.read(app).knobs_user_expanded),
        Some(true),
        "the explicit knobs choice persists on the storybook entity"
    );

    // The explicit choice survives switching stories.
    cx.update(|window, app| {
        storybook.update(app, |storybook, cx| {
            storybook.activate_gallery_story(StoryKind::Layers, window, cx);
        });
    });
    cx.run_until_parked();
    scroll_to_bottom(&storybook, cx);
    assert!(
        cx.debug_bounds("layers-story-knobs").is_some(),
        "the reopened knobs choice persists across story switches"
    );
}

#[gpui::test]
fn story_windows_scroll_below_the_smallest_registered_viewport(cx: &mut TestAppContext) {
    let (storybook, cx) = setup_gallery_storybook(cx);
    let window_id = cx.update(|window, _| window.window_handle().window_id());
    cx.update(|_, app| {
        storybook.update(app, |storybook, cx| {
            storybook.story_windows.insert(window_id, StoryKind::Pages);
            cx.notify();
        });
    });
    cx.simulate_resize(size(px(400.), px(300.)));
    cx.run_until_parked();

    let window_root = cx
        .debug_bounds("storybook-story-window")
        .expect("the story window root should render");
    let content = cx
        .debug_bounds("storybook-story-window-content")
        .expect("the story window content should render");
    let (min_width, min_height) = StoryKind::Pages.descriptor().min_story_size();
    assert!(
        content.size.width >= px(min_width) && content.size.height >= px(min_height),
        "content {content:?} must hold the story's smallest viewport {min_width}×{min_height}"
    );
    assert!(
        window_root.size.height < content.size.height,
        "the undersized window must scroll the story instead of clipping it"
    );
}

#[gpui::test]
fn keyboard_help_toggle_lists_shared_and_story_bindings(cx: &mut TestAppContext) {
    let (storybook, cx) = setup_gallery_storybook(cx);
    cx.update(|window, app| {
        storybook.update(app, |storybook, cx| {
            storybook.activate_gallery_story(StoryKind::Toolbar, window, cx);
        });
    });
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("storybook-keyboard-help-panel").is_none(),
        "the help panel stays hidden until requested"
    );

    let toggle = cx
        .debug_bounds("storybook-keyboard-help-toggle")
        .expect("the footer help toggle should render");
    cx.simulate_click(toggle.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("storybook-keyboard-help-panel").is_some(),
        "the help toggle must open the binding listing"
    );

    cx.simulate_click(toggle.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("storybook-keyboard-help-panel").is_none(),
        "the help toggle must close the binding listing again"
    );
}

/// Upper bound on one story's tab-stop walk. Every surviving ring must
/// cycle well inside it; the bound only exists so a broken focus graph
/// fails fast instead of hanging the suite.
const TAB_WALK_BOUND: usize = 1024;

/// Walks `focus_next` from the story's current focus until the ring
/// revisits a stop, recording each focused element's identity.
fn walk_tab_ring(cx: &mut VisualTestContext, story_title: &str) -> Vec<String> {
    cx.update(|window, app| {
        let mut visited = Vec::new();
        for _ in 0..TAB_WALK_BOUND {
            window.focus_next(app);
            let Some(focused) = window.focused(app) else {
                panic!("{story_title}: tab traversal must never drop window focus");
            };
            let identity = format!("{focused:?}");
            if visited.contains(&identity) {
                return visited;
            }
            visited.push(identity);
        }
        panic!("{story_title}: tab traversal must cycle within {TAB_WALK_BOUND} stops");
    })
}

/// Opens the story's primary overlay state when it has one that a test
/// can reach without pointer coordinates.
fn open_overlay_state(
    storybook: &Entity<Storybook>,
    kind: StoryKind,
    cx: &mut VisualTestContext,
) -> bool {
    match kind {
        StoryKind::Toolbar => {
            cx.update(|window, app| {
                storybook.update(app, |storybook, cx| {
                    storybook.apply_toolbar_overlay(
                        screens::toolbar::ToolbarOverlay::Actions,
                        window,
                        cx,
                    );
                });
            });
            true
        }
        StoryKind::Pages => {
            cx.update(|window, app| {
                storybook.update(app, |storybook, cx| {
                    storybook.pages_screen.apply_named_state(
                        screens::pages::PagesNamedState::SearchActive,
                        window,
                        cx,
                    );
                });
            });
            true
        }
        StoryKind::PseudoEditor => {
            cx.update(|_, app| {
                storybook.update(app, |storybook, cx| {
                    let editor = storybook.pseudo_screen.editor.clone();
                    storybook.pseudo_screen.handle_action(
                        editor,
                        &PseudoEditorAction::VariablesVisibilityChanged { visible: true },
                        cx,
                    );
                });
            });
            true
        }
        StoryKind::Menus => {
            cx.update(|_, app| {
                storybook.update(app, |storybook, cx| {
                    storybook
                        .menus_screen
                        .open_menu_at(point(px(60.), px(40.)), "from the walker");
                    cx.notify();
                });
            });
            true
        }
        StoryKind::Popups => {
            cx.update(|_, app| {
                storybook.update(app, |storybook, cx| {
                    storybook.popups_screen.set_popup_open(true, "walker");
                    cx.notify();
                });
            });
            true
        }
        _ => false,
    }
}

#[gpui::test]
fn every_registered_story_passes_the_bounded_keyboard_walk(cx: &mut TestAppContext) {
    let (storybook, cx) = setup_gallery_storybook(cx);

    for descriptor in screens::registry() {
        let kind = descriptor.kind;
        cx.update(|window, app| {
            storybook.update(app, |storybook, cx| {
                storybook.activate_gallery_story(kind, window, cx);
            });
        });
        cx.run_until_parked();

        // (a) The registry focus hook lands real keyboard focus.
        assert!(
            cx.update(|window, app| window.focused(app).is_some()),
            "{} must focus a story control on activation",
            descriptor.title
        );

        // (b) The bounded walk terminates in a cycle without dropping
        // focus, so Tab can never trap or escape the window.
        let ring = walk_tab_ring(cx, descriptor.title);
        assert!(
            !ring.is_empty(),
            "{} must expose at least one tab stop",
            descriptor.title
        );

        // (c) Escape is safe with no overlay open…
        cx.simulate_keystrokes("escape");
        cx.run_until_parked();

        // …and with the story's primary overlay state open, where the
        // story has one a test can reach without pointer coordinates.
        if open_overlay_state(&storybook, kind, cx) {
            cx.run_until_parked();
            let overlay_ring = walk_tab_ring(cx, descriptor.title);
            assert!(
                !overlay_ring.is_empty(),
                "{} overlay state must keep a focusable ring",
                descriptor.title
            );
            cx.update(|window, app| {
                storybook.update(app, |storybook, cx| {
                    storybook.focus_story(kind, window, cx);
                });
            });
            cx.simulate_keystrokes("escape");
            cx.run_until_parked();
        }
    }
}
