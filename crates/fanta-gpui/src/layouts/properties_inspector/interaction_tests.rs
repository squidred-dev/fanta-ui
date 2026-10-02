use super::*;
use crate::test_support::{assert_pointer_and_keyboard_parity, mount_component};
use gpui::{Modifiers, TestAppContext, size};
struct TabContent(PropertiesInspectorTab);
impl Render for TabContent {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let tab = self.0;
        div()
            .debug_selector(move || format!("properties-inspector-test-content-{}", tab.id()))
            .child(tab.label())
    }
}
fn children(cx: &mut Context<PropertiesInspector>) -> PropertiesInspectorChildren {
    PropertiesInspectorChildren {
        design: cx
            .new(|_| TabContent(PropertiesInspectorTab::Design))
            .into(),
        motion: cx
            .new(|_| TabContent(PropertiesInspectorTab::Motion))
            .into(),
        draw: cx.new(|_| TabContent(PropertiesInspectorTab::Draw)).into(),
        code: cx.new(|_| TabContent(PropertiesInspectorTab::Code)).into(),
        prototype: cx
            .new(|_| TabContent(PropertiesInspectorTab::Prototype))
            .into(),
        comments: cx
            .new(|_| TabContent(PropertiesInspectorTab::Comments))
            .into(),
    }
}
#[gpui::test]
fn collapse_preserves_zoom_and_tab_and_reopens_sidebar(cx: &mut TestAppContext) {
    let (host, actions, cx) = mount_component(cx, |_, cx| {
        let children = children(cx);
        PropertiesInspector::new("properties-test", children, 100, cx)
    });
    cx.simulate_resize(size(px(480.), px(720.)));
    cx.run_until_parked();
    let position = cx
        .debug_bounds("properties-inspector-tab-list-tab-1")
        .unwrap()
        .center();
    cx.simulate_click(position, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().last(),
        Some(&PropertiesInspectorAction::TabChanged {
            tab: PropertiesInspectorTab::Motion
        })
    );
    let position = cx
        .debug_bounds("properties-inspector-toggle")
        .unwrap()
        .center();
    cx.simulate_click(position, Modifiers::none());
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("properties-inspector-floating-card")
            .is_some()
    );
    assert!(cx.debug_bounds("properties-inspector-tabs").is_none());
    let position = cx.debug_bounds("zoombar-in").unwrap().center();
    cx.simulate_click(position, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().last(),
        Some(&PropertiesInspectorAction::Zoom(
            ZoomControlsAction::ZoomChangeRequested { percent: 200 }
        ))
    );
    let sidebar = cx.read(|cx| host.read(cx).component.clone());
    sidebar.update(cx, |sidebar, cx| sidebar.set_zoom(200, cx));
    let position = cx
        .debug_bounds("properties-inspector-toggle")
        .unwrap()
        .center();
    cx.simulate_click(position, Modifiers::none());
    cx.run_until_parked();
    assert!(cx.debug_bounds("properties-inspector-content").is_some());
    assert_eq!(
        sidebar.read_with(cx, |sidebar, _| sidebar.active_tab()),
        PropertiesInspectorTab::Motion
    );
}
#[gpui::test]
fn header_width_is_stable_at_extreme_zoom_values(cx: &mut TestAppContext) {
    let (host, _, cx) = mount_component(cx, |_, cx| {
        let children = children(cx);
        PropertiesInspector::new("width-test", children, 1, cx)
    });
    cx.simulate_resize(size(px(320.), px(720.)));
    cx.run_until_parked();
    let before = cx.debug_bounds("properties-inspector-header").unwrap();
    let sidebar = cx.read(|cx| host.read(cx).component.clone());
    sidebar.update(cx, |s, cx| s.set_zoom(3200, cx));
    cx.run_until_parked();
    assert_eq!(
        cx.debug_bounds("properties-inspector-header").unwrap(),
        before
    );
    let toggle = cx.debug_bounds("properties-inspector-toggle").unwrap();
    assert!(toggle.right() <= before.right());
}

#[gpui::test]
fn play_is_confined_to_prototype_header_and_obeys_host_availability(cx: &mut TestAppContext) {
    let (host, actions, cx) = mount_component(cx, |_, cx| {
        let children = children(cx);
        PropertiesInspector::new("presentation-test", children, 100, cx)
    });
    cx.simulate_resize(size(px(320.), px(720.)));
    cx.run_until_parked();
    assert!(cx.debug_bounds("properties-inspector-present").is_none());
    let sidebar = cx.read(|app| host.read(app).component.clone());
    sidebar.update(cx, |sidebar, cx| {
        sidebar.set_active_tab(PropertiesInspectorTab::Prototype, cx);
        sidebar.set_can_present(true, cx);
    });
    cx.run_until_parked();
    let header = cx.debug_bounds("properties-inspector-header").unwrap();
    let play = cx.debug_bounds("properties-inspector-present").unwrap();
    assert!(play.left() >= header.left() && play.right() <= header.right());
    assert!(play.top() >= header.top() && play.bottom() <= header.bottom());
    assert_pointer_and_keyboard_parity(
        cx,
        "properties-inspector-present",
        &actions,
        PropertiesInspectorAction::PresentRequested,
    );
    sidebar.update(cx, |sidebar, cx| sidebar.set_can_present(false, cx));
    cx.run_until_parked();
    cx.simulate_click(play.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(actions.borrow().is_empty());
    for tab in PropertiesInspectorTab::ALL {
        sidebar.update(cx, |sidebar, cx| sidebar.set_active_tab(tab, cx));
        cx.run_until_parked();
        assert_eq!(
            cx.debug_bounds("properties-inspector-present").is_some(),
            tab == PropertiesInspectorTab::Prototype
        );
    }
}

#[gpui::test]
fn all_six_modes_use_one_scrollable_tab_bar_and_mount_their_child(cx: &mut TestAppContext) {
    exercise_scrollable_tabs(cx);
}

#[gpui::test]
fn zed_theme_without_a_settings_provider_uses_the_standalone_tab_strip(cx: &mut TestAppContext) {
    cx.update(|cx| theme::init(theme::LoadThemes::JustBase, cx));
    exercise_scrollable_tabs(cx);
}

#[gpui::test]
fn canonical_zed_tab_bar_keeps_all_modes_accessible_at_compact_widths(cx: &mut TestAppContext) {
    struct ThemeSettings(gpui::Font);
    impl theme::ThemeSettingsProvider for ThemeSettings {
        fn ui_font<'a>(&'a self, _: &'a App) -> &'a gpui::Font {
            &self.0
        }
        fn buffer_font<'a>(&'a self, _: &'a App) -> &'a gpui::Font {
            &self.0
        }
        fn ui_font_size(&self, _: &App) -> gpui::Pixels {
            px(16.)
        }
        fn buffer_font_size(&self, _: &App) -> gpui::Pixels {
            px(14.)
        }
        fn ui_density(&self, _: &App) -> theme::UiDensity {
            theme::UiDensity::Default
        }
    }
    cx.update(|cx| {
        theme::init(theme::LoadThemes::JustBase, cx);
        theme::set_theme_settings_provider(Box::new(ThemeSettings(gpui::font("UI Test"))), cx);
    });
    exercise_scrollable_tabs(cx);
}

fn exercise_scrollable_tabs(cx: &mut TestAppContext) {
    let (host, actions, cx) = mount_component(cx, |_, cx| {
        let children = children(cx);
        PropertiesInspector::new("navigation-test", children, 100, cx)
    });
    let sidebar = cx.read(|app| host.read(app).component.clone());
    const TAB_SELECTORS: [&str; 6] = [
        "properties-inspector-tab-list-tab-0",
        "properties-inspector-tab-list-tab-1",
        "properties-inspector-tab-list-tab-2",
        "properties-inspector-tab-list-tab-3",
        "properties-inspector-tab-list-tab-4",
        "properties-inspector-tab-list-tab-5",
    ];
    const CHILD_SELECTORS: [&str; 6] = [
        "properties-inspector-test-content-design",
        "properties-inspector-test-content-motion",
        "properties-inspector-test-content-draw",
        "properties-inspector-test-content-code",
        "properties-inspector-test-content-prototype",
        "properties-inspector-test-content-comments",
    ];
    for width in [320., 400., 472.] {
        cx.simulate_resize(size(px(width), px(720.)));
        sidebar.update(cx, |sidebar, cx| {
            sidebar.set_active_tab(PropertiesInspectorTab::Design, cx)
        });
        cx.run_until_parked();
        let bar = cx.debug_bounds("properties-inspector-tabs").unwrap();
        assert_eq!(
            bar.size.height,
            px(32.),
            "navigation stays in one Zed tab row"
        );
        let positions = TAB_SELECTORS.map(|selector| cx.debug_bounds(selector).unwrap());
        for (index, bounds) in positions.iter().enumerate() {
            assert_eq!(bounds.top(), positions[0].top());
            assert_eq!(bounds.bottom(), positions[0].bottom());
            assert!(
                bounds.size.width >= px(60.),
                "full labels keep natural tab widths"
            );
            if index > 0 {
                assert!(bounds.left() >= positions[index - 1].right());
            }
        }
        assert!(sidebar.read_with(cx, |sidebar, _| sidebar.tabs_scroll.max_offset().x) > px(0.));
        for (index, tab) in PropertiesInspectorTab::ALL.into_iter().enumerate() {
            // A controlled host change must reveal the requested tab just as
            // keyboard and overflow-arrow selection do.
            sidebar.update(cx, |sidebar, cx| sidebar.set_active_tab(tab, cx));
            cx.run_until_parked();
            let visible = cx.debug_bounds(TAB_SELECTORS[index]).unwrap();
            let viewport = sidebar.read_with(cx, |sidebar, _| sidebar.tabs_scroll.bounds());
            assert!(visible.left() >= viewport.left() && visible.right() <= viewport.right());
            actions.borrow_mut().clear();
            cx.simulate_click(visible.center(), Modifiers::none());
            cx.run_until_parked();
            assert_eq!(
                actions.borrow().as_slice(),
                [PropertiesInspectorAction::TabChanged { tab }]
            );
            assert_eq!(
                sidebar.read_with(cx, |sidebar, _| sidebar.active_tab()),
                tab
            );
            for (other, selector) in CHILD_SELECTORS.into_iter().enumerate() {
                assert_eq!(cx.debug_bounds(selector).is_some(), other == index);
            }
            for (selector, expected) in TAB_SELECTORS.into_iter().zip(positions) {
                assert_eq!(
                    cx.debug_bounds(selector).unwrap().size,
                    expected.size,
                    "selection must not resize tabs"
                );
            }
        }
    }
    sidebar.update(cx, |sidebar, cx| {
        sidebar.set_active_tab(PropertiesInspectorTab::Design, cx)
    });
    cx.run_until_parked();
    let design_position = cx.debug_bounds(TAB_SELECTORS[0]).unwrap().center();
    cx.simulate_click(design_position, Modifiers::none());
    cx.run_until_parked();
    actions.borrow_mut().clear();
    cx.simulate_keystrokes("right enter");
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().last(),
        Some(&PropertiesInspectorAction::TabChanged {
            tab: PropertiesInspectorTab::Motion
        })
    );
    assert_eq!(
        sidebar.read_with(cx, |sidebar, _| sidebar.active_tab()),
        PropertiesInspectorTab::Motion
    );
    cx.simulate_keystrokes("home");
    cx.run_until_parked();
    for tab in PropertiesInspectorTab::ALL
        .into_iter()
        .cycle()
        .skip(1)
        .take(6)
    {
        actions.borrow_mut().clear();
        let next = cx
            .debug_bounds("properties-inspector-tab-next")
            .unwrap()
            .center();
        cx.simulate_click(next, Modifiers::none());
        cx.run_until_parked();
        assert_eq!(
            actions.borrow().as_slice(),
            [PropertiesInspectorAction::TabChanged { tab }]
        );
        let index = PropertiesInspectorTab::ALL
            .iter()
            .position(|candidate| *candidate == tab)
            .unwrap();
        let visible = cx.debug_bounds(TAB_SELECTORS[index]).unwrap();
        let viewport = sidebar.read_with(cx, |sidebar, _| sidebar.tabs_scroll.bounds());
        assert!(visible.left() >= viewport.left() && visible.right() <= viewport.right());
    }
    for tab in PropertiesInspectorTab::ALL
        .into_iter()
        .cycle()
        .skip(1)
        .take(6)
    {
        actions.borrow_mut().clear();
        cx.simulate_keystrokes("right");
        cx.run_until_parked();
        assert_eq!(
            actions.borrow().as_slice(),
            [PropertiesInspectorAction::TabChanged { tab }]
        );
    }
    cx.simulate_keystrokes("end");
    cx.run_until_parked();
    assert_eq!(
        sidebar.read_with(cx, |sidebar, _| sidebar.active_tab()),
        PropertiesInspectorTab::Comments
    );
    assert!(sidebar.read_with(cx, |sidebar, _| sidebar.tabs_scroll.offset().x) < px(0.));
    cx.simulate_keystrokes("home");
    cx.run_until_parked();
    assert_eq!(
        sidebar.read_with(cx, |sidebar, _| sidebar.active_tab()),
        PropertiesInspectorTab::Design
    );
    assert_eq!(
        sidebar.read_with(cx, |sidebar, _| sidebar.tabs_scroll.offset().x),
        px(0.)
    );
    cx.simulate_keystrokes("down");
    cx.run_until_parked();
    assert_eq!(
        sidebar.read_with(cx, |sidebar, _| sidebar.active_tab()),
        PropertiesInspectorTab::Motion
    );
    cx.simulate_keystrokes("up");
    cx.run_until_parked();
    assert_eq!(
        sidebar.read_with(cx, |sidebar, _| sidebar.active_tab()),
        PropertiesInspectorTab::Design
    );

    sidebar.update(cx, |sidebar, cx| {
        sidebar.set_active_tab(PropertiesInspectorTab::Comments, cx)
    });
    cx.run_until_parked();
    for width in [400., 320., 472.] {
        cx.simulate_resize(size(px(width), px(720.)));
        cx.run_until_parked();
        let comments = cx.debug_bounds(TAB_SELECTORS[5]).unwrap();
        let viewport = sidebar.read_with(cx, |sidebar, _| sidebar.tabs_scroll.bounds());
        assert!(
            comments.left() >= viewport.left() && comments.right() <= viewport.right(),
            "resizing must keep the selected last tab fully visible"
        );
    }
    sidebar.update(cx, |sidebar, cx| {
        sidebar.tabs_scroll.set_offset(gpui::point(px(0.), px(0.)));
        cx.notify();
    });
    cx.run_until_parked();
    assert_eq!(
        sidebar.read_with(cx, |sidebar, _| sidebar.tabs_scroll.offset().x),
        px(0.),
        "an ordinary render must preserve manual tab-strip scrolling"
    );
}
