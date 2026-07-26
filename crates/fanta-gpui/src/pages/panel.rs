use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[cfg(not(test))]
use gpui::{Animation, AnimationExt as _, ElementId};
use gpui::{
    AnyElement, App, AppContext as _, Bounds, ClickEvent, Context, Entity, EventEmitter,
    FocusHandle, Focusable, InteractiveElement as _, IntoElement, KeyUpEvent, MouseButton,
    MouseDownEvent, ParentElement as _, Pixels, Point, Render, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled as _, Subscription, Window, canvas, deferred, div,
    point, prelude::FluentBuilder as _, px,
};
#[cfg(not(test))]
use gpui_component::animation::cubic_bezier;
use gpui_component::{
    ActiveTheme as _, Disableable as _, Icon, IconName, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState, SelectAll},
    scroll::{Scrollbar, ScrollbarAxis},
    tooltip::Tooltip,
    v_flex,
};

use super::{
    ActivatePagesControl, AddPage, ClosePagesSearch, FindInPages, NextSearchResult,
    OpenPageContextMenu, PAGES_CONTROL_KEY_CONTEXT, PAGES_PANEL_KEY_CONTEXT, PagesPanelAction,
    PagesPanelElementKind, PagesPanelItem, PagesPanelResultDirection, PagesPanelSearchRequest,
    PagesPanelSearchResults, PagesPanelSearchScope, PreviousSearchResult, ReplaceAllResults,
    ReplaceCurrentResult, TogglePagesPanel, ToggleSearchSettings,
};

mod events;
mod header;
mod overlays;
mod page_list;
mod search;
#[cfg(test)]
mod tests;

const HEADER_HEIGHT: f32 = 40.;
const PAGE_ROW_HEIGHT: f32 = 32.;
const PAGE_ROW_GAP: f32 = 4.;
const PAGE_PADDING: f32 = 8.;
const MAX_PAGE_LIST_HEIGHT: f32 = 320.;
const APPROXIMATE_PAGE_CHARACTER_WIDTH: f32 = 8.;
const DOUBLE_ENTER_INTERVAL: Duration = Duration::from_millis(500);
#[cfg(not(test))]
const REVEAL_DURATION: f64 = 0.18;

#[derive(Clone, Debug, Eq, PartialEq)]
enum PageEditorTarget {
    Existing {
        page_id: SharedString,
        original_title: SharedString,
    },
    New {
        suggested_title: SharedString,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum PanelMode {
    #[default]
    Pages,
    Find,
    Replace,
}

#[derive(Clone, Debug)]
struct PageMenuState {
    page_id: SharedString,
    page_title: SharedString,
    anchor: Point<Pixels>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum FocusTooltipKind {
    Find,
    AddPage,
    Settings,
    CloseSearch,
    ReplaceCurrent,
    ReplaceAll,
    PreviousResult,
    NextResult,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PageMenuAction {
    CopyLink,
    Rename,
    Duplicate,
    Delete,
}

/// Stateful, headless Pages panel.
///
/// Create this as a GPUI entity, subscribe to [`PagesPanelAction`], and render
/// the entity directly:
///
/// ```ignore
/// let panel = cx.new(|cx| PagesPanel::new("pages", pages, window, cx));
/// cx.subscribe(&panel, |host, _, event, cx| host.handle_pages(event, cx));
/// ```
///
/// The entity owns only presentation state: open/closed animation, focused
/// inputs, draft text, menus, filters, and active search result. Page data,
/// selection, search results, and all document mutations stay with the host.
pub struct PagesPanel {
    id: SharedString,
    focus_handle: FocusHandle,
    filter_menu_focus_handle: FocusHandle,
    scope_menu_focus_handle: FocusHandle,
    page_menu_focus_handle: FocusHandle,
    find_focus_handle: FocusHandle,
    add_page_focus_handle: FocusHandle,
    settings_focus_handle: FocusHandle,
    close_search_focus_handle: FocusHandle,
    replace_current_focus_handle: FocusHandle,
    replace_all_focus_handle: FocusHandle,
    previous_result_focus_handle: FocusHandle,
    next_result_focus_handle: FocusHandle,
    pages: Vec<PagesPanelItem>,
    expanded: bool,
    selected_page: Option<SharedString>,
    mode: PanelMode,
    editing: Option<PageEditorTarget>,
    select_page_editor_after_render: bool,
    rename_input: Entity<InputState>,
    search_input: Entity<InputState>,
    replace_input: Entity<InputState>,
    active_filters: Vec<PagesPanelElementKind>,
    search_scope: PagesPanelSearchScope,
    match_case: bool,
    whole_words: bool,
    filter_menu_open: bool,
    scope_menu_open: bool,
    page_menu: Option<PageMenuState>,
    panel_bounds: Option<Bounds<Pixels>>,
    search_panel_bounds: Option<Bounds<Pixels>>,
    filter_anchor_bounds: Option<Bounds<Pixels>>,
    scope_anchor_bounds: Option<Bounds<Pixels>>,
    tooltip_anchor_bounds: HashMap<FocusTooltipKind, Bounds<Pixels>>,
    page_row_bounds: HashMap<SharedString, Bounds<Pixels>>,
    pages_scroll_handle: ScrollHandle,
    results_scroll_handle: ScrollHandle,
    results: PagesPanelSearchResults,
    active_result: Option<usize>,
    last_keyboard_page_activation: Option<(SharedString, Instant)>,
    #[cfg(test)]
    hovered_result: Option<usize>,
    #[cfg(test)]
    header_hovered: bool,
    #[cfg(test)]
    hovered_page: Option<SharedString>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<PagesPanelAction> for PagesPanel {}

impl PagesPanel {
    /// Creates an expanded Pages panel.
    pub fn new(
        id: impl Into<SharedString>,
        pages: Vec<PagesPanelItem>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let rename_input = cx.new(|cx| InputState::new(window, cx).placeholder("Page name"));
        let search_input = cx.new(|cx| InputState::new(window, cx).placeholder("Find…"));
        let replace_input = cx.new(|cx| InputState::new(window, cx).placeholder("Replace with…"));

        let subscriptions = vec![
            cx.subscribe_in(
                &rename_input,
                window,
                |this, _, event: &InputEvent, window, cx| match event {
                    InputEvent::PressEnter { .. } => {
                        this.commit_page_name(true, window, cx);
                    }
                    InputEvent::Blur => this.commit_page_name(false, window, cx),
                    InputEvent::Change | InputEvent::Focus => {}
                },
            ),
            cx.subscribe(
                &search_input,
                |this, _, event: &InputEvent, cx| match event {
                    InputEvent::Change => this.emit_search_request(cx),
                    InputEvent::PressEnter { secondary } => {
                        let direction = if *secondary {
                            PagesPanelResultDirection::Previous
                        } else {
                            PagesPanelResultDirection::Next
                        };
                        this.navigate_results(direction, cx);
                    }
                    InputEvent::Focus | InputEvent::Blur => {}
                },
            ),
            cx.subscribe(&replace_input, |_this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            }),
        ];

        Self {
            id: id.into(),
            focus_handle: cx.focus_handle(),
            filter_menu_focus_handle: cx.focus_handle(),
            scope_menu_focus_handle: cx.focus_handle(),
            page_menu_focus_handle: cx.focus_handle(),
            find_focus_handle: cx.focus_handle(),
            add_page_focus_handle: cx.focus_handle(),
            settings_focus_handle: cx.focus_handle(),
            close_search_focus_handle: cx.focus_handle(),
            replace_current_focus_handle: cx.focus_handle(),
            replace_all_focus_handle: cx.focus_handle(),
            previous_result_focus_handle: cx.focus_handle(),
            next_result_focus_handle: cx.focus_handle(),
            pages,
            expanded: true,
            selected_page: None,
            mode: PanelMode::Pages,
            editing: None,
            select_page_editor_after_render: false,
            rename_input,
            search_input,
            replace_input,
            active_filters: Vec::new(),
            search_scope: PagesPanelSearchScope::default(),
            match_case: false,
            whole_words: false,
            filter_menu_open: false,
            scope_menu_open: false,
            page_menu: None,
            panel_bounds: None,
            search_panel_bounds: None,
            filter_anchor_bounds: None,
            scope_anchor_bounds: None,
            tooltip_anchor_bounds: HashMap::new(),
            page_row_bounds: HashMap::new(),
            pages_scroll_handle: ScrollHandle::new(),
            results_scroll_handle: ScrollHandle::new(),
            results: PagesPanelSearchResults::default(),
            active_result: None,
            last_keyboard_page_activation: None,
            #[cfg(test)]
            hovered_result: None,
            #[cfg(test)]
            header_hovered: false,
            #[cfg(test)]
            hovered_page: None,
            _subscriptions: subscriptions,
        }
    }

    /// Replaces the host-controlled page read model.
    pub fn set_pages(&mut self, pages: Vec<PagesPanelItem>, cx: &mut Context<Self>) {
        self.pages = pages;
        cx.notify();
    }

    /// Replaces the host-controlled selected page.
    pub fn set_selected_page(
        &mut self,
        selected_page: Option<SharedString>,
        cx: &mut Context<Self>,
    ) {
        if selected_page
            .as_ref()
            .is_some_and(|selected| self.pages.last().is_some_and(|page| page.id == *selected))
        {
            self.pages_scroll_handle.scroll_to_bottom();
        }
        self.selected_page = selected_page;
        cx.notify();
    }

    /// Replaces the host-controlled expanded state.
    pub fn set_expanded(&mut self, expanded: bool, cx: &mut Context<Self>) {
        self.expanded = expanded;
        cx.notify();
    }

    /// Replaces the host-controlled search result read model.
    pub fn set_search_results(&mut self, results: PagesPanelSearchResults, cx: &mut Context<Self>) {
        self.results = results;
        self.active_result = if self.results.items.is_empty() {
            None
        } else {
            Some(
                self.active_result
                    .unwrap_or(0)
                    .min(self.results.items.len() - 1),
            )
        };
        cx.notify();
    }

    /// Returns the current search request, useful when initially populating a
    /// host result model.
    pub fn search_request(&self, cx: &App) -> PagesPanelSearchRequest {
        self.build_search_request(cx)
    }
}

impl Focusable for PagesPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PagesPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.select_page_editor_after_render {
            self.select_page_editor_after_render = false;
            window.on_next_frame(|window, cx| {
                window.dispatch_action(Box::new(SelectAll), cx);
            });
        }
        let id = self.id.clone();
        let panel = cx.entity();
        let root = v_flex()
            .id(id)
            .key_context(PAGES_PANEL_KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_toggle_panel))
            .on_action(cx.listener(Self::on_find_in_pages))
            .on_action(cx.listener(Self::on_add_page))
            .on_action(cx.listener(Self::on_toggle_search_settings))
            .on_action(cx.listener(Self::on_close_search))
            .on_action(cx.listener(Self::on_previous_search_result))
            .on_action(cx.listener(Self::on_next_search_result))
            .on_action(cx.listener(Self::on_replace_current_result))
            .on_action(cx.listener(Self::on_replace_all_results))
            .relative()
            .w_full()
            .max_h_full()
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().sidebar_foreground)
            .border_1()
            .border_color(cx.theme().border)
            .child(
                canvas(
                    move |bounds, _, app| {
                        panel.update(app, |this, _| {
                            this.panel_bounds = Some(bounds);
                        });
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            );
        let root = if self.mode == PanelMode::Pages {
            root.child(self.render_header(cx))
                .child(self.render_pages(cx))
        } else {
            root.debug_selector(|| "pages-search-panel".to_owned())
                .size_full()
                .child(self.render_search(cx))
        };
        let tooltip = self.render_focused_tooltip(window, cx);
        root.when(self.page_menu.is_some(), |root| {
            root.child(self.render_page_menu(cx))
        })
        .when_some(tooltip, |root, tooltip| root.child(tooltip))
    }
}

fn result_count_label(total: usize) -> String {
    if total == 1 {
        "1 result".to_owned()
    } else {
        format!("{total} results")
    }
}

fn page_title_min_width(character_count: usize) -> Pixels {
    px(character_count as f32 * APPROXIMATE_PAGE_CHARACTER_WIDTH + PAGE_PADDING * 2.)
}

fn prevent_keyboard_activation_click(event: &KeyUpEvent, window: &mut Window, _: &mut App) {
    if !event.keystroke.modifiers.modified()
        && matches!(event.keystroke.key.as_str(), "enter" | "space")
    {
        window.prevent_default();
    }
}

fn empty_results_label(scope: PagesPanelSearchScope) -> String {
    format!("No results on {}", scope.label().to_lowercase())
}

fn next_page_title(pages: &[PagesPanelItem]) -> SharedString {
    let next = pages
        .iter()
        .filter_map(|page| {
            page.title
                .strip_prefix("Page ")
                .and_then(|suffix| suffix.parse::<u64>().ok())
        })
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    format!("Page {next}").into()
}
