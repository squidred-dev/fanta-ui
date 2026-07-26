use std::{cell::RefCell, rc::Rc};

use gpui::{
    AppContext as _, Bounds, Context, Entity, Focusable as _, IntoElement, KeyDownEvent,
    KeyUpEvent, Keystroke, Modifiers, MouseButton, MouseDownEvent, MouseUpEvent,
    ParentElement as _, Pixels, Render, ScrollDelta, ScrollWheelEvent, SharedString, Styled as _,
    Subscription, TestAppContext, VisualTestContext, Window, div, point, px, size,
};
use gpui_component::Root;

use super::{
    PAGE_ROW_HEIGHT, PageEditorTarget, PagesPanel, PanelMode, empty_results_label, next_page_title,
    result_count_label,
};
use crate::pages::{
    AddPage, FindInPages, PagesPanelAction, PagesPanelElementKind, PagesPanelItem,
    PagesPanelResultDirection, PagesPanelSearchResult, PagesPanelSearchResults,
    PagesPanelSearchScope,
};

struct TestHost {
    panel: Entity<PagesPanel>,
    actions: Rc<RefCell<Vec<PagesPanelAction>>>,
    _subscription: Subscription,
}

impl TestHost {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let panel = cx.new(|cx| {
            PagesPanel::new(
                "test-pages",
                vec![
                    PagesPanelItem::new("page-1", "Page 1"),
                    PagesPanelItem::new("page-2", "Page 2"),
                    PagesPanelItem::new("page-3", "Page 3"),
                ],
                window,
                cx,
            )
        });
        let actions = Rc::new(RefCell::new(Vec::new()));
        let captured_actions = actions.clone();
        let subscription = cx.subscribe(&panel, move |_, _, action: &PagesPanelAction, _| {
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
            .max_w(px(320.))
            .max_h(px(220.))
            .child(self.panel.clone())
    }
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
        .expect("test host should be installed in the component root");
    (host, cx)
}

fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(selector)
        .unwrap_or_else(|| panic!("missing rendered selector: {selector}"))
}

fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let position = bounds(cx, selector).center();
    cx.simulate_click(position, Modifiers::none());
}

fn double_click(cx: &mut VisualTestContext, selector: &'static str) {
    let position = bounds(cx, selector).center();
    double_click_at(cx, position);
}

fn double_click_at(cx: &mut VisualTestContext, position: gpui::Point<Pixels>) {
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
}

fn key_down(cx: &mut VisualTestContext, key: &str) {
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke::parse(key).expect("valid test keystroke"),
        is_held: false,
    });
    cx.run_until_parked();
}

fn key_up(cx: &mut VisualTestContext, key: &str) {
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse(key).expect("valid test keystroke"),
    });
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

fn panel(host: &Entity<TestHost>, cx: &VisualTestContext) -> Entity<PagesPanel> {
    cx.read(|app| host.read(app).panel.clone())
}

fn actions(host: &Entity<TestHost>, cx: &VisualTestContext) -> Rc<RefCell<Vec<PagesPanelAction>>> {
    cx.read(|app| host.read(app).actions.clone())
}

fn read_panel<R>(
    panel: &Entity<PagesPanel>,
    cx: &VisualTestContext,
    read: impl FnOnce(&PagesPanel) -> R,
) -> R {
    cx.read(|app| read(panel.read(app)))
}

fn set_one_search_result(panel: &Entity<PagesPanel>, cx: &mut VisualTestContext) {
    cx.update(|_, app| {
        panel.update(app, |panel, cx| {
            panel.set_search_results(
                PagesPanelSearchResults {
                    total: 1,
                    items: vec![
                        PagesPanelSearchResult::new(
                            "result-1",
                            "Long page heading",
                            PagesPanelElementKind::Text,
                        )
                        .parent("Hero"),
                    ],
                    element_counts: Vec::new(),
                },
                cx,
            );
        });
    });
    cx.run_until_parked();
}

fn focus_panel(panel: &Entity<PagesPanel>, cx: &mut VisualTestContext) {
    cx.update(|window, app| {
        panel.focus_handle(app).focus(window);
    });
    cx.run_until_parked();
}

#[test]
fn next_page_title_uses_the_highest_page_number_not_the_count() {
    let pages = vec![
        PagesPanelItem::new("a", "Page 1"),
        PagesPanelItem::new("b", "Page 3"),
        PagesPanelItem::new("c", "Marketing"),
    ];

    assert_eq!(next_page_title(&pages).as_ref(), "Page 4");
}

#[test]
fn next_page_title_starts_at_one() {
    let pages = vec![PagesPanelItem::new("a", "Untitled")];

    assert_eq!(next_page_title(&pages).as_ref(), "Page 1");
}

#[test]
fn empty_results_use_a_count_header_and_a_scoped_body_message() {
    assert_eq!(result_count_label(0), "0 results");
    assert_eq!(result_count_label(1), "1 result");
    assert_eq!(result_count_label(12), "12 results");
    assert_eq!(
        empty_results_label(PagesPanelSearchScope::CurrentPage),
        "No results on this page"
    );
    assert_eq!(
        empty_results_label(PagesPanelSearchScope::AllPages),
        "No results on all pages"
    );
}

#[gpui::test]
fn empty_results_render_in_the_scrollable_body_without_header_overflow(cx: &mut TestAppContext) {
    let (_host, cx) = setup(cx);

    click(cx, "pages-search-trigger");
    click(cx, "pages-scope-trigger");
    click(cx, "pages-scope-all-pages");

    let panel = bounds(cx, "pages-search-panel");
    let header = bounds(cx, "pages-results-header");
    let count = bounds(cx, "pages-result-count");
    let previous = bounds(cx, "pages-previous-result");
    let next = bounds(cx, "pages-next-result");
    let viewport = bounds(cx, "pages-results-viewport");
    let empty = bounds(cx, "pages-results-empty");

    assert!(header.right() <= panel.right());
    assert!(count.left() >= header.left() && count.right() <= header.right());
    assert!(previous.left() >= header.left());
    assert!(next.right() <= header.right());
    assert!(empty.top() >= viewport.top());
    assert!(empty.right() <= viewport.right());
}

#[gpui::test]
fn zero_results_prevent_replace_and_navigation_actions(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let actions = actions(&host, cx);

    click(cx, "pages-search-trigger");
    click(cx, "pages-filter-trigger");
    click(cx, "pages-mode-replace");

    actions.borrow_mut().clear();
    click(cx, "pages-previous-result");
    click(cx, "pages-next-result");
    click(cx, "pages-replace-one");
    click(cx, "pages-replace-all");

    assert!(
        actions.borrow().is_empty(),
        "disabled zero-result controls must not emit host intents"
    );
}

#[gpui::test]
fn registered_commands_drive_the_same_headless_component_state(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);

    focus_panel(&panel, cx);
    cx.dispatch_action(FindInPages);
    assert_eq!(read_panel(&panel, cx, |panel| panel.mode), PanelMode::Find);

    cx.dispatch_action(AddPage);
    assert_eq!(read_panel(&panel, cx, |panel| panel.mode), PanelMode::Pages);
    assert!(matches!(
        read_panel(&panel, cx, |panel| panel.editing.clone()),
        Some(PageEditorTarget::New { .. })
    ));
}

#[gpui::test]
fn shortcuts_and_tab_activation_cover_pages_and_search_controls(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);

    focus_panel(&panel, cx);
    cx.simulate_keystrokes("tab enter");
    assert!(
        !read_panel(&panel, cx, |panel| panel.expanded),
        "Enter should activate the header reached with Tab"
    );

    cx.simulate_keystrokes("tab enter");
    assert_eq!(
        read_panel(&panel, cx, |panel| panel.mode),
        PanelMode::Find,
        "the next tab stop should be the Find button"
    );

    cx.simulate_keystrokes("tab enter");
    assert!(
        read_panel(&panel, cx, |panel| panel.filter_menu_open),
        "Tab should leave the search input and Enter should open Settings"
    );

    cx.simulate_keystrokes("tab enter");
    assert_eq!(
        read_panel(&panel, cx, |panel| panel.mode),
        PanelMode::Replace,
        "the open Settings menu should move focus to Find, then Tab to Replace"
    );

    cx.simulate_keystrokes("tab enter");
    assert!(
        read_panel(&panel, cx, |panel| panel.scope_menu_open),
        "Tab should leave the replace input for the search-scope control"
    );

    cx.simulate_keystrokes("tab enter");
    assert_eq!(
        read_panel(&panel, cx, |panel| panel.search_scope),
        PagesPanelSearchScope::AllPages,
        "the scope menu should focus This page, then Tab to All pages"
    );
}

#[gpui::test]
fn closing_search_from_the_keyboard_restores_tab_focus_and_expands_the_header(
    cx: &mut TestAppContext,
) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);

    click(cx, "pages-header");
    assert!(!read_panel(&panel, cx, |panel| panel.expanded));

    focus_panel(&panel, cx);
    cx.simulate_keystrokes("secondary-f");
    assert_eq!(read_panel(&panel, cx, |panel| panel.mode), PanelMode::Find);

    cx.simulate_keystrokes("tab tab enter");
    cx.run_until_parked();
    assert_eq!(read_panel(&panel, cx, |panel| panel.mode), PanelMode::Pages);

    cx.simulate_keystrokes("tab enter");
    assert!(
        read_panel(&panel, cx, |panel| panel.expanded),
        "the restored panel focus should make the collapsed header the next tab stop"
    );
}

#[gpui::test]
fn complete_keyboard_lifecycle_keeps_filter_and_scope_menus_open(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let actions = actions(&host, cx);

    focus_panel(&panel, cx);
    cx.simulate_keystrokes("secondary-f tab");
    key_down(cx, "enter");
    assert!(read_panel(&panel, cx, |panel| panel.filter_menu_open));
    assert!(bounds(cx, "pages-filter-menu").size.height > px(0.));
    key_up(cx, "enter");
    assert!(read_panel(&panel, cx, |panel| panel.filter_menu_open));
    assert_eq!(read_panel(&panel, cx, |panel| panel.mode), PanelMode::Find);

    click(cx, "pages-filter-trigger");
    assert!(!read_panel(&panel, cx, |panel| panel.filter_menu_open));

    focus_panel(&panel, cx);
    cx.simulate_keystrokes("tab tab tab tab");
    key_down(cx, "enter");
    assert!(read_panel(&panel, cx, |panel| panel.scope_menu_open));
    assert!(bounds(cx, "pages-scope-menu").size.height > px(0.));
    key_up(cx, "enter");
    assert!(read_panel(&panel, cx, |panel| panel.scope_menu_open));
    assert_eq!(
        read_panel(&panel, cx, |panel| panel.search_scope),
        PagesPanelSearchScope::CurrentPage
    );

    click(cx, "pages-scope-trigger");
    click(cx, "pages-filter-trigger");
    cx.run_until_parked();
    actions.borrow_mut().clear();
    cx.simulate_keystrokes("tab tab tab");
    key_down(cx, "enter");
    key_up(cx, "enter");
    assert_eq!(
        read_panel(&panel, cx, |panel| panel.active_filters.clone()),
        vec![PagesPanelElementKind::Text],
        "one Enter key lifecycle must toggle a filter exactly once"
    );
    assert_eq!(
        actions
            .borrow()
            .iter()
            .filter(|action| matches!(action, PagesPanelAction::SearchRequested(_)))
            .count(),
        1
    );
}

#[gpui::test]
fn tab_focus_opens_the_same_command_tooltips_as_pointer_hover(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);

    focus_panel(&panel, cx);
    cx.simulate_keystrokes("tab tab");
    cx.run_until_parked();
    let find_tooltip = bounds(cx, "pages-focus-tooltip");
    assert!(find_tooltip.size.width > px(0.) && find_tooltip.size.height > px(0.));

    cx.simulate_keystrokes("tab");
    cx.run_until_parked();
    let add_tooltip = bounds(cx, "pages-focus-tooltip");
    assert!(add_tooltip.size.width > px(0.) && add_tooltip.size.height > px(0.));
    assert_ne!(
        add_tooltip.origin, find_tooltip.origin,
        "the tooltip should follow the newly focused Add page control"
    );
}

#[gpui::test]
fn tab_order_reaches_page_and_result_rows(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let actions = actions(&host, cx);

    focus_panel(&panel, cx);
    actions.borrow_mut().clear();
    cx.simulate_keystrokes("tab tab tab tab enter");
    assert_eq!(
        actions.borrow().as_slice(),
        &[PagesPanelAction::SelectRequested {
            page_id: "page-1".into(),
        }],
        "header, Find, Add, then the first page row should form the page tab order"
    );

    focus_panel(&panel, cx);
    actions.borrow_mut().clear();
    cx.simulate_keystrokes("shift-tab enter");
    assert_eq!(
        actions.borrow().as_slice(),
        &[PagesPanelAction::SelectRequested {
            page_id: "page-3".into(),
        }],
        "Shift-Tab should traverse the page tab order in reverse"
    );

    set_one_search_result(&panel, cx);
    focus_panel(&panel, cx);
    cx.simulate_keystrokes("secondary-f");
    actions.borrow_mut().clear();
    cx.simulate_keystrokes("tab tab tab tab tab tab enter");
    assert_eq!(
        actions.borrow().as_slice(),
        &[PagesPanelAction::SearchResultSelected {
            result_id: "result-1".into(),
        }],
        "search input, Settings, Close, scope, navigation, then result rows should be tabbable"
    );
}

#[gpui::test]
fn adding_a_page_scrolls_and_focuses_the_editor_then_restores_tab_navigation(
    cx: &mut TestAppContext,
) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let actions = actions(&host, cx);
    let pages = (1..=18)
        .map(|index| PagesPanelItem::new(format!("page-{index}"), format!("Page {index}")))
        .collect();
    cx.update(|_, app| {
        panel.update(app, |panel, cx| panel.set_pages(pages, cx));
    });
    cx.run_until_parked();

    click(cx, "pages-add-trigger");
    cx.run_until_parked();
    let viewport = bounds(cx, "pages-scroll-viewport");
    let editor = bounds(cx, "pages-page-editor");
    let scroll_handle = read_panel(&panel, cx, |panel| panel.pages_scroll_handle.clone());
    assert!(editor.top() >= viewport.top());
    assert!(editor.bottom() <= viewport.bottom());
    assert_eq!(scroll_handle.offset().y, -scroll_handle.max_offset().height);

    cx.simulate_keystrokes("x");
    let editor_value = cx.read(|app| panel.read(app).rename_input.read(app).value());
    assert!(
        editor_value.ends_with('x'),
        "typing should go directly to the newly revealed editor"
    );

    actions.borrow_mut().clear();
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().as_slice(),
        &[PagesPanelAction::CreateRequested {
            title: editor_value,
        }]
    );
    assert!(read_panel(&panel, cx, |panel| panel.editing.is_none()));

    cx.simulate_keystrokes("tab enter");
    assert!(
        !read_panel(&panel, cx, |panel| panel.expanded),
        "committing the new page should restore the panel as a tab-navigation owner"
    );
}

#[gpui::test]
fn double_enter_renames_the_focused_page_row(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let actions = actions(&host, cx);

    focus_panel(&panel, cx);
    actions.borrow_mut().clear();
    cx.simulate_keystrokes("tab tab tab tab enter enter");
    cx.run_until_parked();

    assert_eq!(
        actions.borrow().as_slice(),
        &[PagesPanelAction::SelectRequested {
            page_id: "page-1".into(),
        }]
    );
    assert!(matches!(
        read_panel(&panel, cx, |panel| panel.editing.clone()),
        Some(PageEditorTarget::Existing { ref page_id, .. })
            if page_id.as_ref() == "page-1"
    ));
    assert!(bounds(cx, "pages-page-editor").size.height > px(0.));
}

#[gpui::test]
fn result_navigation_shortcuts_emit_typed_intents(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let actions = actions(&host, cx);
    set_one_search_result(&panel, cx);
    focus_panel(&panel, cx);

    cx.simulate_keystrokes("secondary-f");
    assert_eq!(read_panel(&panel, cx, |panel| panel.mode), PanelMode::Find);

    actions.borrow_mut().clear();
    cx.simulate_keystrokes("shift-secondary-f shift-secondary-d");
    assert_eq!(
        actions.borrow().as_slice(),
        &[
            PagesPanelAction::NavigateResults {
                direction: PagesPanelResultDirection::Next,
                result_id: Some("result-1".into()),
            },
            PagesPanelAction::NavigateResults {
                direction: PagesPanelResultDirection::Previous,
                result_id: Some("result-1".into()),
            },
        ]
    );
}

#[gpui::test]
fn clicking_another_page_cancels_rename_and_new_page_edits(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let actions = actions(&host, cx);

    let first_page_bounds = bounds(cx, "pages-row-page-1");
    let viewport_bounds = bounds(cx, "pages-scroll-viewport");
    let first_page_center = first_page_bounds.center();
    cx.simulate_mouse_move(first_page_center, None, Modifiers::none());
    assert_eq!(
        read_panel(&panel, cx, |panel| panel.hovered_page.clone()),
        Some("page-1".into()),
        "row bounds: {first_page_bounds:?}; viewport bounds: {viewport_bounds:?}"
    );
    double_click(cx, "pages-row-page-1");
    let editing = read_panel(&panel, cx, |panel| panel.editing.clone());
    assert!(
        editing.is_some(),
        "double click did not start editing; actions: {:?}",
        actions.borrow().as_slice()
    );
    assert!(bounds(cx, "pages-page-editor").size.height > px(0.));
    assert!(matches!(
        editing,
        Some(PageEditorTarget::Existing { ref page_id, .. }) if page_id.as_ref() == "page-1"
    ));

    actions.borrow_mut().clear();
    click(cx, "pages-row-page-2");
    assert!(read_panel(&panel, cx, |panel| panel.editing.is_none()));
    assert_eq!(
        actions.borrow().as_slice(),
        &[PagesPanelAction::SelectRequested {
            page_id: "page-2".into(),
        }]
    );

    click(cx, "pages-add-trigger");
    assert!(matches!(
        read_panel(&panel, cx, |panel| panel.editing.clone()),
        Some(PageEditorTarget::New { .. })
    ));

    actions.borrow_mut().clear();
    click(cx, "pages-row-page-3");
    assert!(read_panel(&panel, cx, |panel| panel.editing.is_none()));
    assert_eq!(
        actions.borrow().as_slice(),
        &[PagesPanelAction::SelectRequested {
            page_id: "page-3".into(),
        }]
    );
    assert!(!actions.borrow().iter().any(|action| matches!(
        action,
        PagesPanelAction::CreateRequested { .. } | PagesPanelAction::RenameRequested { .. }
    )));
}

#[gpui::test]
fn page_context_menu_emits_headless_intents_and_starts_inline_rename(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let actions = actions(&host, cx);

    secondary_click(cx, "pages-row-page-2");
    assert!(bounds(cx, "pages-page-menu-copy-link").size.width > px(0.));
    actions.borrow_mut().clear();
    click(cx, "pages-page-menu-copy-link");
    assert_eq!(
        actions.borrow().as_slice(),
        &[PagesPanelAction::CopyLinkRequested {
            page_id: "page-2".into(),
        }]
    );

    secondary_click(cx, "pages-row-page-2");
    actions.borrow_mut().clear();
    click(cx, "pages-page-menu-duplicate");
    assert_eq!(
        actions.borrow().as_slice(),
        &[PagesPanelAction::DuplicateRequested {
            page_id: "page-2".into(),
        }]
    );

    secondary_click(cx, "pages-row-page-2");
    actions.borrow_mut().clear();
    click(cx, "pages-page-menu-delete");
    assert_eq!(
        actions.borrow().as_slice(),
        &[PagesPanelAction::DeleteRequested {
            page_id: "page-2".into(),
        }]
    );

    secondary_click(cx, "pages-row-page-1");
    actions.borrow_mut().clear();
    click(cx, "pages-page-menu-rename");
    assert!(matches!(
        read_panel(&panel, cx, |panel| panel.editing.clone()),
        Some(PageEditorTarget::Existing { ref page_id, .. })
            if page_id.as_ref() == "page-1"
    ));
    assert!(
        actions.borrow().is_empty(),
        "rename is emitted only after the inline editor commits"
    );
    assert!(bounds(cx, "pages-page-editor").size.height > px(0.));
}

#[gpui::test]
fn page_menu_is_row_scoped_and_ctrl_enter_opens_it_for_keyboard_users(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let actions = actions(&host, cx);

    secondary_click(cx, "pages-header");
    assert!(cx.debug_bounds("pages-page-menu").is_none());

    focus_panel(&panel, cx);
    cx.simulate_keystrokes("tab tab tab tab ctrl-enter");
    cx.run_until_parked();
    assert!(bounds(cx, "pages-page-menu").size.height > px(0.));
    assert!(bounds(cx, "pages-page-menu-copy-link").size.height > px(0.));

    actions.borrow_mut().clear();
    cx.simulate_keystrokes("enter");
    cx.run_until_parked();
    assert_eq!(
        actions.borrow().as_slice(),
        &[PagesPanelAction::CopyLinkRequested {
            page_id: "page-1".into(),
        }]
    );

    secondary_click(cx, "pages-row-page-2");
    assert!(bounds(cx, "pages-page-menu").size.height > px(0.));
    click(cx, "pages-header");
    cx.run_until_parked();
    assert!(
        read_panel(&panel, cx, |panel| panel.page_menu.is_none()),
        "a header click must dismiss the panel-owned page menu state"
    );
    assert!(!read_panel(&panel, cx, |panel| panel.expanded));

    secondary_click(cx, "pages-header");
    assert!(
        read_panel(&panel, cx, |panel| panel.page_menu.is_none()),
        "clipped rows must never receive a secondary click through the collapsed header"
    );
}

#[gpui::test]
fn search_result_rows_do_not_open_the_page_context_menu(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let actions = actions(&host, cx);
    set_one_search_result(&panel, cx);
    click(cx, "pages-search-trigger");

    actions.borrow_mut().clear();
    secondary_click(cx, "pages-result-0");

    assert!(cx.debug_bounds("pages-page-menu-copy-link").is_none());
    assert!(actions.borrow().is_empty());
}

#[gpui::test]
fn popovers_are_top_left_anchored_and_replace_does_not_displace_filter(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let actions = actions(&host, cx);
    set_one_search_result(&panel, cx);

    click(cx, "pages-search-trigger");
    let find_trigger = bounds(cx, "pages-filter-trigger");
    click(cx, "pages-filter-trigger");
    let find_menu = bounds(cx, "pages-filter-menu");
    assert_eq!(find_menu.origin.x, find_trigger.origin.x);
    assert_eq!(find_menu.origin.y, find_trigger.bottom() + px(4.));

    click(cx, "pages-mode-replace");
    assert_eq!(
        read_panel(&panel, cx, |panel| panel.mode),
        PanelMode::Replace
    );
    let replace_trigger = bounds(cx, "pages-filter-trigger");
    assert_eq!(replace_trigger.origin, find_trigger.origin);

    click(cx, "pages-filter-trigger");
    let replace_menu = bounds(cx, "pages-filter-menu");
    assert_eq!(replace_menu.origin, find_menu.origin);
    assert_eq!(replace_menu.origin.x, replace_trigger.origin.x);
    assert_eq!(replace_menu.origin.y, replace_trigger.bottom() + px(4.));

    click(cx, "pages-filter-trigger");
    let scope_trigger = bounds(cx, "pages-scope-trigger");
    click(cx, "pages-scope-trigger");
    let scope_menu = bounds(cx, "pages-scope-menu");
    assert_eq!(scope_menu.origin.x, scope_trigger.origin.x);
    assert_eq!(scope_menu.origin.y, scope_trigger.bottom() + px(4.));

    actions.borrow_mut().clear();
    click(cx, "pages-scope-all-pages");
    assert_eq!(
        read_panel(&panel, cx, |panel| panel.search_scope),
        PagesPanelSearchScope::AllPages
    );
    assert!(matches!(
        actions.borrow().last(),
        Some(PagesPanelAction::SearchRequested(request))
            if request.scope == PagesPanelSearchScope::AllPages
    ));
}

#[gpui::test]
fn popup_occlusion_prevents_hover_from_leaking_to_results(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    set_one_search_result(&panel, cx);
    click(cx, "pages-search-trigger");
    click(cx, "pages-filter-trigger");

    let overlap = bounds(cx, "pages-filter-menu").intersect(&bounds(cx, "pages-result-0"));
    assert!(overlap.size.width > px(0.) && overlap.size.height > px(0.));
    let overlap_point = overlap.center();

    click(cx, "pages-filter-trigger");
    cx.simulate_mouse_move(overlap_point, None, Modifiers::none());
    assert_eq!(
        read_panel(&panel, cx, |panel| panel.hovered_result),
        Some(0)
    );

    click(cx, "pages-filter-trigger");
    cx.simulate_mouse_move(overlap_point, None, Modifiers::none());
    assert_eq!(read_panel(&panel, cx, |panel| panel.hovered_result), None);
}

#[gpui::test]
fn collapsed_header_hover_excludes_its_action_buttons(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let actions = actions(&host, cx);

    click(cx, "pages-header");
    assert!(!read_panel(&panel, cx, |panel| panel.expanded));

    let header = bounds(cx, "pages-header");
    cx.simulate_mouse_move(
        point(header.left() + px(12.), header.center().y),
        None,
        Modifiers::none(),
    );
    assert!(read_panel(&panel, cx, |panel| panel.header_hovered));

    let search_button_center = bounds(cx, "pages-search-trigger").center();
    cx.simulate_mouse_move(search_button_center, None, Modifiers::none());
    assert!(!read_panel(&panel, cx, |panel| panel.header_hovered));

    actions.borrow_mut().clear();
    click(cx, "pages-search-trigger");
    assert_eq!(read_panel(&panel, cx, |panel| panel.mode), PanelMode::Find);
    assert!(!read_panel(&panel, cx, |panel| panel.expanded));
    assert!(actions.borrow().is_empty());
}

#[gpui::test]
fn collapsed_long_page_titles_truncate_before_the_header_actions(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let long_id: SharedString = "long-selected-page".into();
    let long_title =
        "This selected page title is intentionally much wider than the collapsed Pages panel";
    cx.update(|_, app| {
        panel.update(app, |panel, cx| {
            panel.set_pages(
                vec![
                    PagesPanelItem::new(long_id.clone(), long_title),
                    PagesPanelItem::new("page-2", "Page 2"),
                ],
                cx,
            );
            panel.set_selected_page(Some(long_id.clone()), cx);
        });
    });
    cx.run_until_parked();

    click(cx, "pages-header");
    let assert_constrained = |cx: &mut VisualTestContext| {
        let header = bounds(cx, "pages-header");
        let title = bounds(cx, "pages-header-title");
        let search = bounds(cx, "pages-search-trigger");
        let add = bounds(cx, "pages-add-trigger");
        assert!(title.right() <= search.left());
        assert!(search.right() <= add.left());
        assert!(add.right() <= header.right());
    };
    assert_constrained(cx);

    cx.simulate_resize(size(px(220.), px(180.)));
    cx.run_until_parked();
    assert_constrained(cx);
}

#[gpui::test]
fn long_page_names_scroll_horizontally_and_many_pages_scroll_vertically(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let pages = (1..=14)
        .map(|index| {
            let title = if index == 1 {
                "This page name is intentionally far wider than the Pages panel viewport".to_owned()
            } else {
                format!("Page {index}")
            };
            PagesPanelItem::new(format!("long-page-{index}"), title)
        })
        .collect();
    cx.update(|_, app| {
        panel.update(app, |panel, cx| panel.set_pages(pages, cx));
    });
    cx.run_until_parked();

    let viewport = bounds(cx, "pages-scroll-viewport");
    let scrollbar_layer = bounds(cx, "pages-scrollbar-layer");
    let long_row = bounds(cx, "pages-row-long-page-1");
    let scroll_handle = read_panel(&panel, cx, |panel| panel.pages_scroll_handle.clone());
    assert_eq!(scrollbar_layer, viewport);
    assert_eq!(scroll_handle.bounds(), viewport);
    assert!(long_row.size.width > viewport.size.width);
    assert!(scroll_handle.max_offset().width > px(0.));
    assert!(scroll_handle.max_offset().height > px(0.));

    cx.simulate_event(ScrollWheelEvent {
        position: viewport.center(),
        delta: ScrollDelta::Pixels(point(px(-180.), px(0.))),
        ..Default::default()
    });
    assert!(scroll_handle.offset().x < px(0.));

    cx.simulate_event(ScrollWheelEvent {
        position: viewport.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-120.))),
        ..Default::default()
    });
    assert!(scroll_handle.offset().y < px(0.));
    assert_eq!(bounds(cx, "pages-scrollbar-layer"), viewport);

    cx.simulate_resize(size(px(260.), px(180.)));
    cx.run_until_parked();
    let resized_viewport = bounds(cx, "pages-scroll-viewport");
    assert_ne!(resized_viewport.size, viewport.size);
    assert_eq!(bounds(cx, "pages-scrollbar-layer"), resized_viewport);
    assert_eq!(scroll_handle.bounds(), resized_viewport);
    assert_eq!(
        resized_viewport.right(),
        bounds(cx, "pages-scrollbar-layer").right()
    );
    assert_eq!(
        resized_viewport.bottom(),
        bounds(cx, "pages-scrollbar-layer").bottom()
    );
}

#[gpui::test]
fn editing_a_long_horizontally_scrolled_page_keeps_a_full_width_input(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let long_title = "This extremely long page title remains editable even after the pages list has been scrolled all the way horizontally";
    cx.update(|_, app| {
        panel.update(app, |panel, cx| {
            panel.set_pages(
                vec![
                    PagesPanelItem::new("long-page", long_title),
                    PagesPanelItem::new("short-page", "Page 2"),
                ],
                cx,
            );
        });
    });
    cx.run_until_parked();

    let viewport = bounds(cx, "pages-scroll-viewport");
    let scroll_handle = read_panel(&panel, cx, |panel| panel.pages_scroll_handle.clone());
    cx.simulate_event(ScrollWheelEvent {
        position: viewport.center(),
        delta: ScrollDelta::Pixels(point(-scroll_handle.max_offset().width, px(0.))),
        ..Default::default()
    });
    assert!(scroll_handle.offset().x < px(0.));

    let row = bounds(cx, "pages-row-long-page");
    double_click_at(cx, point(viewport.left() + px(24.), row.center().y));
    cx.run_until_parked();

    assert_eq!(scroll_handle.offset().x, px(0.));
    let editor = bounds(cx, "pages-page-editor");
    let input_slot = bounds(cx, "pages-page-editor-input-slot");
    let visible_slot = input_slot.intersect(&bounds(cx, "pages-scroll-viewport"));
    assert_eq!(editor.size.height, px(PAGE_ROW_HEIGHT));
    assert_eq!(input_slot.size.height, px(PAGE_ROW_HEIGHT));
    assert!(
        visible_slot.size.width >= viewport.size.width - px(24.),
        "the editor must remain visible and fill the viewport instead of flex-collapsing"
    );

    cx.simulate_keystrokes("x");
    let editor_value = cx.read(|app| panel.read(app).rename_input.read(app).value());
    assert!(
        editor_value.ends_with('x'),
        "typing must continue to reach the long-title editor"
    );
}

#[gpui::test]
fn search_results_have_a_bounded_scrollable_viewport(cx: &mut TestAppContext) {
    let (host, cx) = setup(cx);
    let panel = panel(&host, cx);
    let items = (1..=24)
        .map(|index| {
            PagesPanelSearchResult::new(
                format!("result-{index}"),
                format!("Result {index}"),
                PagesPanelElementKind::Text,
            )
            .parent("Frame")
        })
        .collect::<Vec<_>>();
    cx.update(|_, app| {
        panel.update(app, |panel, cx| {
            panel.set_search_results(
                PagesPanelSearchResults {
                    total: items.len(),
                    items,
                    element_counts: Vec::new(),
                },
                cx,
            );
        });
    });
    cx.run_until_parked();
    click(cx, "pages-search-trigger");

    let viewport = bounds(cx, "pages-results-viewport");
    let scrollbar_layer = bounds(cx, "pages-results-scrollbar-layer");
    let scroll_handle = read_panel(&panel, cx, |panel| panel.results_scroll_handle.clone());
    assert_eq!(scrollbar_layer, viewport);
    assert_eq!(scroll_handle.bounds(), viewport);
    assert!(scroll_handle.max_offset().height > px(0.));

    cx.simulate_event(ScrollWheelEvent {
        position: viewport.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-160.))),
        ..Default::default()
    });
    assert!(scroll_handle.offset().y < px(0.));
    assert_eq!(bounds(cx, "pages-results-scrollbar-layer"), viewport);

    cx.simulate_resize(size(px(260.), px(180.)));
    cx.run_until_parked();
    let resized_viewport = bounds(cx, "pages-results-viewport");
    assert_ne!(resized_viewport.size, viewport.size);
    assert_eq!(
        bounds(cx, "pages-results-scrollbar-layer"),
        resized_viewport
    );
    assert_eq!(scroll_handle.bounds(), resized_viewport);
}
