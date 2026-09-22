use super::*;
use crate::test_support::mount_component;
use gpui::{Modifiers, TestAppContext, size};
struct TabContent;
impl Render for TabContent {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child("Child content")
    }
}
fn children(cx: &mut Context<PropertiesInspector>) -> PropertiesInspectorChildren {
    let content: AnyView = cx.new(|_| TabContent).into();
    PropertiesInspectorChildren {
        design: content.clone(),
        motion: content.clone(),
        draw: content.clone(),
        code: content.clone(),
        prototype: content.clone(),
        comments: content,
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
