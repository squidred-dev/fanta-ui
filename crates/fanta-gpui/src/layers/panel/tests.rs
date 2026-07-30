use std::{cell::RefCell, collections::HashSet, rc::Rc};

use gpui::{
    AppContext as _, Bounds, Context, Entity, Focusable as _, IntoElement, Modifiers, MouseButton,
    MouseDownEvent, MouseUpEvent, ParentElement as _, Pixels, Render, SharedString, Styled as _,
    Subscription, TestAppContext, VisualTestContext, Window, div, px,
};
use gpui_component::Root;

use super::{flatten_visible, menu::menu_sections_for, *};

struct TestHost {
    panel: Entity<LayersPanel>,
    actions: Rc<RefCell<Vec<LayersPanelAction>>>,
    _subscription: Subscription,
}

impl TestHost {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let panel = cx.new(|cx| LayersPanel::new("test-layers", fixture_nodes(), window, cx));
        panel.update(cx, |panel, cx| {
            panel.set_selected_node_ids(vec!["text".into()], cx);
            panel.set_expanded_node_ids(
                vec!["frame".into(), "group".into(), "component".into()],
                cx,
            );
        });

        let actions = Rc::new(RefCell::new(Vec::new()));
        let captured_actions = actions.clone();
        let subscription = cx.subscribe(&panel, move |_, _, action: &LayersPanelAction, _| {
            captured_actions.borrow_mut().push(action.clone());
        });
        Self {
            panel,
            actions,
            _subscription: subscription,
        }
    }
}

impl Render for TestHost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .h_full()
            .max_w(px(340.))
            .max_h(px(520.))
            .child(self.panel.clone())
    }
}

fn fixture_nodes() -> Vec<LayersPanelItem> {
    use LayersPanelNodeKind as Kind;

    vec![
        LayersPanelItem::new("frame", "Checkout", Kind::Frame).children(vec![
            LayersPanelItem::new("group", "Header", Kind::Group)
                .locked(true)
                .children(vec![
                    LayersPanelItem::new("text", "Title", Kind::Text),
                    LayersPanelItem::new("image", "Logo", Kind::Image).visible(false),
                ]),
            LayersPanelItem::new("component", "Primary button", Kind::Component).children(vec![
                LayersPanelItem::new("instance", "Icon instance", Kind::Instance),
            ]),
            LayersPanelItem::new("rectangle", "Background", Kind::Rectangle),
        ]),
        LayersPanelItem::new("section", "Variants", Kind::Section),
        LayersPanelItem::new("slice", "Export slice", Kind::Slice),
    ]
}

fn setup(cx: &mut TestAppContext) -> (Entity<TestHost>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_component::init(cx);
        crate::init(cx);
    });
    let host_slot = Rc::new(RefCell::new(None));
    let captured_host = host_slot.clone();
    let (_, cx) = cx.add_window_view(move |window, cx| {
        let host = cx.new(|cx| TestHost::new(window, cx));
        *captured_host.borrow_mut() = Some(host.clone());
        Root::new(host, window, cx)
    });
    let host = host_slot
        .borrow_mut()
        .take()
        .expect("test host should be installed");
    (host, cx)
}

fn panel(host: &Entity<TestHost>, cx: &VisualTestContext) -> Entity<LayersPanel> {
    cx.read(|app| host.read(app).panel.clone())
}

fn actions(host: &Entity<TestHost>, cx: &VisualTestContext) -> Rc<RefCell<Vec<LayersPanelAction>>> {
    cx.read(|app| host.read(app).actions.clone())
}

fn read_panel<R>(
    panel: &Entity<LayersPanel>,
    cx: &VisualTestContext,
    read: impl FnOnce(&LayersPanel) -> R,
) -> R {
    cx.read(|app| read(panel.read(app)))
}

fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(selector)
        .unwrap_or_else(|| panic!("missing rendered selector: {selector}"))
}

fn click(cx: &mut VisualTestContext, selector: &'static str, modifiers: Modifiers) {
    let position = bounds(cx, selector).center();
    cx.simulate_click(position, modifiers);
    cx.run_until_parked();
}

fn secondary_click(cx: &mut VisualTestContext, selector: &'static str) {
    let position = bounds(cx, selector).center();
    cx.simulate_mouse_move(position, None, Modifiers::none());
    cx.simulate_event(MouseDownEvent {
        position,
        button: MouseButton::Right,
        modifiers: Modifiers::none(),
        click_count: 1,
        first_mouse: false,
    });
    cx.simulate_event(MouseUpEvent {
        position,
        button: MouseButton::Right,
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();
}

fn double_click(cx: &mut VisualTestContext, selector: &'static str) {
    let position = bounds(cx, selector).center();
    cx.simulate_mouse_move(position, None, Modifiers::none());
    cx.simulate_click(position, Modifiers::none());
    cx.simulate_event(MouseDownEvent {
        position,
        button: MouseButton::Left,
        modifiers: Modifiers::none(),
        click_count: 2,
        first_mouse: false,
    });
    cx.simulate_event(MouseUpEvent {
        position,
        button: MouseButton::Left,
        modifiers: Modifiers::none(),
        click_count: 2,
    });
    cx.run_until_parked();
}

fn focus_panel(panel: &Entity<LayersPanel>, cx: &mut VisualTestContext) {
    cx.update(|window, app| {
        panel.focus_handle(app).focus(window);
    });
    cx.run_until_parked();
}

fn actions_for(kind: LayersPanelNodeKind) -> HashSet<LayersPanelContextAction> {
    menu_sections_for(kind)
        .into_iter()
        .flatten()
        .map(|entry| entry.action)
        .collect()
}

#[test]
fn visible_tree_respects_expansion_and_depth() {
    let nodes = fixture_nodes();
    let expanded = vec!["frame".into(), "group".into()];
    let mut visible = Vec::new();
    flatten_visible(&nodes, 0, &expanded, &mut visible);

    let identity_and_depth = visible
        .iter()
        .map(|layer| {
            (
                layer.item.id.as_ref(),
                layer.depth,
                layer
                    .ancestor_ids
                    .iter()
                    .map(SharedString::as_ref)
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        identity_and_depth,
        vec![
            ("frame", 0, vec![]),
            ("group", 1, vec!["frame"]),
            ("text", 2, vec!["frame", "group"]),
            ("image", 2, vec!["frame", "group"]),
            ("component", 1, vec!["frame"]),
            ("rectangle", 1, vec!["frame"]),
            ("section", 0, vec![]),
            ("slice", 0, vec![]),
        ]
    );
}

#[test]
fn every_supported_node_kind_has_safe_common_context_actions() {
    use LayersPanelNodeKind as Kind;
    let kinds = [
        Kind::Frame,
        Kind::Group,
        Kind::Section,
        Kind::Component,
        Kind::ComponentSet,
        Kind::Instance,
        Kind::Text,
        Kind::Image,
        Kind::Video,
        Kind::Rectangle,
        Kind::Ellipse,
        Kind::Polygon,
        Kind::Star,
        Kind::Line,
        Kind::Arrow,
        Kind::Vector,
        Kind::BooleanOperation,
        Kind::Slice,
        Kind::Mask,
        Kind::Pen,
        Kind::Pencil,
        Kind::Other,
    ];

    for kind in kinds {
        let actions = actions_for(kind);
        assert!(actions.contains(&LayersPanelContextAction::Copy));
        assert!(actions.contains(&LayersPanelContextAction::Rename));
        assert!(actions.contains(&LayersPanelContextAction::ShowHide));
        assert!(actions.contains(&LayersPanelContextAction::LockUnlock));
        assert!(actions.contains(&LayersPanelContextAction::Plugins));
    }
}

#[test]
fn context_menu_capabilities_vary_by_node_type() {
    use LayersPanelContextAction as Action;
    use LayersPanelNodeKind as Kind;

    let text = actions_for(Kind::Text);
    assert!(text.contains(&Action::EditText));
    assert!(text.contains(&Action::OutlineStroke));
    assert!(!text.contains(&Action::CropImage));

    let image = actions_for(Kind::Image);
    assert!(image.contains(&Action::CropImage));
    assert!(image.contains(&Action::ReplaceMedia));
    assert!(!image.contains(&Action::EditText));

    let instance = actions_for(Kind::Instance);
    assert!(instance.contains(&Action::GoToMainComponent));
    assert!(instance.contains(&Action::DetachInstance));
    assert!(!instance.contains(&Action::CreateComponent));

    let frame = actions_for(Kind::Frame);
    assert!(frame.contains(&Action::ConvertToSection));
    assert!(frame.contains(&Action::RemoveFrame));
    assert!(frame.contains(&Action::MoreLayoutOptions));

    let section = actions_for(Kind::Section);
    assert!(section.contains(&Action::ConvertToFrame));
    assert!(!section.contains(&Action::FlipHorizontal));
    assert!(!section.contains(&Action::CreateComponent));
}

#[gpui::test]
fn tree_renders_nested_types_selection_and_host_states(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);

    assert!(bounds(cx, "layers-row-frame").size.height > px(0.));
    assert!(bounds(cx, "layers-row-text").size.height > px(0.));
    assert!(bounds(cx, "layers-row-instance").size.height > px(0.));
    assert!(bounds(cx, "layers-lock-group").size.width > px(0.));
    assert!(bounds(cx, "layers-visibility-image").size.width > px(0.));
    assert_eq!(
        read_panel(&panel, cx, |panel| panel.selected_node_ids.clone()),
        vec![SharedString::from("text")]
    );
}

#[gpui::test]
fn hovering_a_nested_row_highlights_its_nearest_visible_group(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);

    let position = bounds(cx, "layers-row-text").center();
    cx.simulate_mouse_move(position, None, Modifiers::none());
    cx.run_until_parked();

    assert_eq!(
        read_panel(&panel, cx, |panel| {
            let visible = panel.visible_layers();
            panel.hovered_group_id(&visible)
        }),
        Some("group".into())
    );
}

#[gpui::test]
fn selecting_a_group_highlights_its_visible_subtree(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    panel.update(cx, |panel, cx| {
        panel.set_selected_node_ids(vec!["group".into()], cx);
    });
    cx.run_until_parked();

    let highlighted = read_panel(&panel, cx, |panel| {
        let visible = panel.visible_layers();
        let selected_groups = panel.selected_group_ids(&visible);
        visible
            .iter()
            .filter(|layer| LayersPanel::layer_is_within_groups(layer, &selected_groups))
            .map(|layer| layer.item.id.clone())
            .collect::<Vec<_>>()
    });
    assert_eq!(
        highlighted,
        vec![
            SharedString::from("group"),
            SharedString::from("text"),
            SharedString::from("image"),
        ]
    );
}

#[gpui::test]
fn layers_header_collapses_and_expands_the_whole_panel(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let actions = actions(&host, cx);

    click(cx, "layers-header", Modifiers::none());
    assert!(!read_panel(&panel, cx, |panel| panel.panel_expanded));
    assert_eq!(bounds(cx, "layers-panel").size.height, px(HEADER_HEIGHT));
    assert_eq!(
        actions.borrow().last(),
        Some(&LayersPanelAction::PanelExpansionChanged { expanded: false })
    );

    click(cx, "layers-header", Modifiers::none());
    assert!(read_panel(&panel, cx, |panel| panel.panel_expanded));
    assert!(bounds(cx, "layers-tree-viewport").size.height > px(0.));
    assert_eq!(
        actions.borrow().last(),
        Some(&LayersPanelAction::PanelExpansionChanged { expanded: true })
    );
}

#[gpui::test]
fn row_clicks_emit_replace_toggle_and_range_selection(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let actions = actions(&host, cx);

    click(cx, "layers-row-rectangle", Modifiers::none());

    let mut toggle = Modifiers::none();
    #[cfg(target_os = "macos")]
    {
        toggle.platform = true;
    }
    #[cfg(not(target_os = "macos"))]
    {
        toggle.control = true;
    }
    click(cx, "layers-row-text", toggle);

    let mut range = Modifiers::none();
    range.shift = true;
    click(cx, "layers-row-image", range);

    assert_eq!(
        actions.borrow().as_slice(),
        &[
            LayersPanelAction::SelectRequested {
                node_id: "rectangle".into(),
                mode: LayersPanelSelectionMode::Replace,
            },
            LayersPanelAction::SelectRequested {
                node_id: "text".into(),
                mode: LayersPanelSelectionMode::Toggle,
            },
            LayersPanelAction::SelectRequested {
                node_id: "image".into(),
                mode: LayersPanelSelectionMode::Range,
            },
        ]
    );
}

#[gpui::test]
fn dragging_a_row_emits_a_typed_move_intent(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let actions = actions(&host, cx);
    let source = bounds(cx, "layers-row-rectangle").center();
    let target_bounds = bounds(cx, "layers-row-section");
    let target = gpui::point(target_bounds.center().x, target_bounds.top() + px(2.));

    cx.simulate_mouse_move(source, None, Modifiers::none());
    cx.simulate_mouse_down(source, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(
        source + gpui::point(px(8.), px(0.)),
        MouseButton::Left,
        Modifiers::none(),
    );
    cx.simulate_mouse_move(target, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(target, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();

    assert_eq!(
        actions.borrow().last(),
        Some(&LayersPanelAction::MoveRequested {
            node_id: "rectangle".into(),
            target_node_id: "section".into(),
            position: LayersPanelDropPosition::Before,
        })
    );
}

#[gpui::test]
fn expansion_and_collapse_all_update_presentation_and_emit_intents(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let actions = actions(&host, cx);

    click(cx, "layers-expand-group", Modifiers::none());
    assert!(!read_panel(&panel, cx, |panel| panel
        .expanded_node_ids
        .contains(&"group".into())));
    assert_eq!(
        actions.borrow().last(),
        Some(&LayersPanelAction::ExpansionChanged {
            node_id: "group".into(),
            expanded: false,
        })
    );

    click(cx, "layers-collapse-all", Modifiers::none());
    assert!(read_panel(&panel, cx, |panel| panel
        .expanded_node_ids
        .is_empty()));
    assert_eq!(
        actions.borrow().last(),
        Some(&LayersPanelAction::CollapseAllRequested)
    );
}

#[gpui::test]
fn row_controls_emit_visibility_and_lock_changes_without_selecting(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let actions = actions(&host, cx);

    click(cx, "layers-lock-group", Modifiers::none());
    click(cx, "layers-visibility-image", Modifiers::none());

    assert_eq!(
        actions.borrow().as_slice(),
        &[
            LayersPanelAction::LockChanged {
                node_id: "group".into(),
                locked: false,
            },
            LayersPanelAction::VisibilityChanged {
                node_id: "image".into(),
                visible: true,
            },
        ]
    );
}

#[gpui::test]
fn contextual_menu_renders_node_specific_actions_and_emits_typed_intent(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let actions = actions(&host, cx);

    secondary_click(cx, "layers-row-instance");
    assert!(bounds(cx, "layers-context-menu").size.width > px(0.));
    assert!(bounds(cx, "layers-menu-detach-instance").size.height > px(0.));
    assert!(cx.debug_bounds("layers-menu-create-component").is_none());

    actions.borrow_mut().clear();
    click(cx, "layers-menu-detach-instance", Modifiers::none());
    assert_eq!(
        actions.borrow().as_slice(),
        &[LayersPanelAction::ContextActionRequested {
            node_id: "instance".into(),
            action: LayersPanelContextAction::DetachInstance,
        }]
    );
}

#[gpui::test]
fn menu_visibility_and_lock_items_use_dedicated_intents(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let actions = actions(&host, cx);

    secondary_click(cx, "layers-row-image");
    actions.borrow_mut().clear();
    cx.update(|window, app| {
        panel.update(app, |panel, cx| {
            panel.activate_menu_action(LayersPanelContextAction::ShowHide, window, cx);
        });
    });
    assert_eq!(
        actions.borrow().as_slice(),
        &[LayersPanelAction::VisibilityChanged {
            node_id: "image".into(),
            visible: true,
        }]
    );

    secondary_click(cx, "layers-row-group");
    actions.borrow_mut().clear();
    cx.update(|window, app| {
        panel.update(app, |panel, cx| {
            panel.activate_menu_action(LayersPanelContextAction::LockUnlock, window, cx);
        });
    });
    assert_eq!(
        actions.borrow().as_slice(),
        &[LayersPanelAction::LockChanged {
            node_id: "group".into(),
            locked: false,
        }]
    );
}

#[gpui::test]
fn long_context_menu_repositions_to_stay_inside_the_panel_height(cx: &mut TestAppContext) {
    let (_host, cx) = setup(cx);

    secondary_click(cx, "layers-row-slice");
    let panel = bounds(cx, "layers-panel");
    let menu = bounds(cx, "layers-context-menu");

    assert!(menu.top() >= panel.top());
    assert!(menu.bottom() <= panel.bottom());
    assert_eq!(bounds(cx, "layers-menu-copy").size.height, px(28.));
    assert!(bounds(cx, "layers-context-menu-scrollbar").size.height > px(0.));
}

#[gpui::test]
fn double_click_rename_commits_only_a_changed_nonempty_title(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let actions = actions(&host, cx);

    double_click(cx, "layers-row-text");
    assert!(read_panel(&panel, cx, |panel| panel.editing.is_some()));
    assert!(bounds(cx, "layers-row-editor").size.height > px(0.));

    cx.update(|window, app| {
        panel.update(app, |panel, cx| {
            panel.rename_input.update(cx, |input, cx| {
                input.set_value("Renamed title", window, cx);
            });
        });
    });
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().last(),
        Some(&LayersPanelAction::RenameRequested {
            node_id: "text".into(),
            title: "Renamed title".into(),
        })
    );
    assert!(read_panel(&panel, cx, |panel| panel.editing.is_none()));
}

#[gpui::test]
fn keyboard_opens_and_escape_closes_a_row_menu_with_focus_restored(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);

    focus_panel(&panel, cx);
    cx.simulate_keystrokes("tab tab tab ctrl-enter");
    cx.run_until_parked();
    assert!(read_panel(&panel, cx, |panel| panel.menu.is_some()));
    assert!(bounds(cx, "layers-context-menu").size.width > px(0.));

    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!(read_panel(&panel, cx, |panel| panel.menu.is_none()));
}
