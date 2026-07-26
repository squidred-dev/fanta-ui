#[cfg(not(test))]
use std::time::Duration;

#[cfg(not(test))]
use gpui::{Animation, AnimationExt as _, ElementId};
use gpui::{
    AnyElement, App, AppContext as _, Bounds, ClickEvent, Context, Entity, EventEmitter,
    FocusHandle, Focusable, InteractiveElement as _, IntoElement, MouseButton, ParentElement as _,
    Pixels, Render, ScrollHandle, SharedString, StatefulInteractiveElement as _, Styled as _,
    Subscription, Window, canvas, deferred, div, point, prelude::FluentBuilder as _, px,
};
#[cfg(not(test))]
use gpui_component::animation::cubic_bezier;
use gpui_component::{
    ActiveTheme as _, Disableable as _, Icon, IconName, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState, SelectAll},
    menu::{ContextMenuExt as _, PopupMenuItem},
    scroll::{ScrollableElement as _, ScrollbarAxis},
    v_flex,
};

use super::{
    ActivatePagesControl, AddPage, ClosePagesSearch, FindInPages, NextSearchResult,
    PAGES_CONTROL_KEY_CONTEXT, PAGES_PANEL_KEY_CONTEXT, PagesPanelAction, PagesPanelElementKind,
    PagesPanelItem, PagesPanelResultDirection, PagesPanelSearchRequest, PagesPanelSearchResults,
    PagesPanelSearchScope, PreviousSearchResult, ReplaceAllResults, ReplaceCurrentResult,
    TogglePagesPanel, ToggleSearchSettings,
};

const HEADER_HEIGHT: f32 = 40.;
const PAGE_ROW_HEIGHT: f32 = 32.;
const PAGE_ROW_GAP: f32 = 4.;
const PAGE_PADDING: f32 = 8.;
const MAX_PAGE_LIST_HEIGHT: f32 = 320.;
const APPROXIMATE_PAGE_CHARACTER_WIDTH: f32 = 8.;
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
    pages: Vec<PagesPanelItem>,
    expanded: bool,
    selected_page: Option<SharedString>,
    mode: PanelMode,
    editing: Option<PageEditorTarget>,
    rename_input: Entity<InputState>,
    search_input: Entity<InputState>,
    replace_input: Entity<InputState>,
    active_filters: Vec<PagesPanelElementKind>,
    search_scope: PagesPanelSearchScope,
    match_case: bool,
    whole_words: bool,
    filter_menu_open: bool,
    scope_menu_open: bool,
    search_panel_bounds: Option<Bounds<Pixels>>,
    filter_anchor_bounds: Option<Bounds<Pixels>>,
    scope_anchor_bounds: Option<Bounds<Pixels>>,
    pages_scroll_handle: ScrollHandle,
    results_scroll_handle: ScrollHandle,
    results: PagesPanelSearchResults,
    active_result: Option<usize>,
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
            cx.subscribe(
                &rename_input,
                |this, _, event: &InputEvent, cx| match event {
                    InputEvent::PressEnter { .. } | InputEvent::Blur => {
                        this.commit_page_name(cx);
                    }
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
            pages,
            expanded: true,
            selected_page: None,
            mode: PanelMode::Pages,
            editing: None,
            rename_input,
            search_input,
            replace_input,
            active_filters: Vec::new(),
            search_scope: PagesPanelSearchScope::default(),
            match_case: false,
            whole_words: false,
            filter_menu_open: false,
            scope_menu_open: false,
            search_panel_bounds: None,
            filter_anchor_bounds: None,
            scope_anchor_bounds: None,
            pages_scroll_handle: ScrollHandle::new(),
            results_scroll_handle: ScrollHandle::new(),
            results: PagesPanelSearchResults::default(),
            active_result: None,
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

    fn collapsed_title(&self) -> SharedString {
        self.selected_page
            .as_ref()
            .and_then(|selected| self.pages.iter().find(|page| &page.id == selected))
            .map(|page| page.title.clone())
            .unwrap_or_else(|| "Pages".into())
    }

    fn toggle_expanded(&mut self, cx: &mut Context<Self>) {
        self.expanded = !self.expanded;
        cx.emit(PagesPanelAction::ExpansionChanged {
            expanded: self.expanded,
        });
        cx.notify();
    }

    fn toggle_search_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.mode == PanelMode::Pages {
            return;
        }
        self.filter_menu_open = !self.filter_menu_open;
        self.scope_menu_open = false;
        if self.filter_menu_open {
            let focus_handle = self.filter_menu_focus_handle.clone();
            window.defer(cx, move |window, _| {
                focus_handle.focus(window);
            });
        }
        cx.notify();
    }

    fn toggle_search_scope(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.mode == PanelMode::Pages {
            return;
        }
        self.scope_menu_open = !self.scope_menu_open;
        self.filter_menu_open = false;
        if self.scope_menu_open {
            let focus_handle = self.scope_menu_focus_handle.clone();
            window.defer(cx, move |window, _| {
                focus_handle.focus(window);
            });
        }
        cx.notify();
    }

    fn on_toggle_panel(&mut self, _: &TogglePagesPanel, _: &mut Window, cx: &mut Context<Self>) {
        if self.mode == PanelMode::Pages {
            self.toggle_expanded(cx);
        }
    }

    fn on_find_in_pages(&mut self, _: &FindInPages, window: &mut Window, cx: &mut Context<Self>) {
        self.open_search(window, cx);
    }

    fn on_add_page(&mut self, _: &AddPage, window: &mut Window, cx: &mut Context<Self>) {
        self.begin_new_page(window, cx);
    }

    fn on_toggle_search_settings(
        &mut self,
        _: &ToggleSearchSettings,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle_search_settings(window, cx);
    }

    fn on_close_search(&mut self, _: &ClosePagesSearch, _: &mut Window, cx: &mut Context<Self>) {
        if self.mode != PanelMode::Pages {
            self.close_search(cx);
        }
    }

    fn on_previous_search_result(
        &mut self,
        _: &PreviousSearchResult,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.navigate_results(PagesPanelResultDirection::Previous, cx);
    }

    fn on_next_search_result(
        &mut self,
        _: &NextSearchResult,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.navigate_results(PagesPanelResultDirection::Next, cx);
    }

    fn on_replace_current_result(
        &mut self,
        _: &ReplaceCurrentResult,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.request_replace(false, cx);
    }

    fn on_replace_all_results(
        &mut self,
        _: &ReplaceAllResults,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.request_replace(true, cx);
    }

    fn begin_new_page(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.expanded {
            self.expanded = true;
            cx.emit(PagesPanelAction::ExpansionChanged { expanded: true });
        }
        self.mode = PanelMode::Pages;

        let title = next_page_title(&self.pages);
        self.editing = Some(PageEditorTarget::New {
            suggested_title: title.clone(),
        });
        self.focus_page_editor(title, window, cx);
    }

    fn begin_rename(
        &mut self,
        page_id: SharedString,
        title: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editing = Some(PageEditorTarget::Existing {
            page_id,
            original_title: title.clone(),
        });
        self.focus_page_editor(title, window, cx);
    }

    fn focus_page_editor(
        &mut self,
        title: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.rename_input.update(cx, |input, cx| {
            input.set_value(title, window, cx);
            input.focus(window, cx);
        });
        window.dispatch_action(Box::new(SelectAll), cx);
        cx.notify();
    }

    fn commit_page_name(&mut self, cx: &mut Context<Self>) {
        let Some(target) = self.editing.take() else {
            return;
        };
        let entered = self.rename_input.read(cx).value();
        let entered = entered.trim().to_owned();

        match target {
            PageEditorTarget::Existing {
                page_id,
                original_title,
            } => {
                let title = if entered.is_empty() {
                    original_title
                } else {
                    entered.into()
                };
                cx.emit(PagesPanelAction::RenameRequested { page_id, title });
            }
            PageEditorTarget::New { suggested_title } => {
                let title = if entered.is_empty() {
                    suggested_title
                } else {
                    entered.into()
                };
                cx.emit(PagesPanelAction::CreateRequested { title });
            }
        }
        cx.notify();
    }

    fn cancel_page_edit(&mut self, cx: &mut Context<Self>) {
        if self.editing.take().is_some() {
            cx.notify();
        }
    }

    fn open_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.commit_page_name(cx);
        self.mode = PanelMode::Find;
        self.filter_menu_open = false;
        self.scope_menu_open = false;
        self.search_input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
        cx.notify();
    }

    fn close_search(&mut self, cx: &mut Context<Self>) {
        self.mode = PanelMode::Pages;
        self.filter_menu_open = false;
        self.scope_menu_open = false;
        cx.emit(PagesPanelAction::SearchClosed);
        cx.notify();
    }

    fn build_search_request(&self, cx: &App) -> PagesPanelSearchRequest {
        PagesPanelSearchRequest {
            query: self.search_input.read(cx).value(),
            scope: self.search_scope,
            element_kinds: self.active_filters.clone(),
            match_case: self.match_case,
            whole_words: self.whole_words,
        }
    }

    fn emit_search_request(&mut self, cx: &mut Context<Self>) {
        self.active_result = None;
        cx.emit(PagesPanelAction::SearchRequested(
            self.build_search_request(cx),
        ));
        cx.notify();
    }

    fn toggle_filter(&mut self, kind: PagesPanelElementKind, cx: &mut Context<Self>) {
        if kind == PagesPanelElementKind::All {
            self.active_filters.clear();
        } else if let Some(index) = self.active_filters.iter().position(|item| *item == kind) {
            self.active_filters.remove(index);
        } else {
            self.active_filters.push(kind);
            self.active_filters.sort_unstable();
        }
        self.emit_search_request(cx);
    }

    fn toggle_match_case(&mut self, cx: &mut Context<Self>) {
        self.match_case = !self.match_case;
        self.emit_search_request(cx);
    }

    fn toggle_whole_words(&mut self, cx: &mut Context<Self>) {
        self.whole_words = !self.whole_words;
        self.emit_search_request(cx);
    }

    fn set_panel_mode(&mut self, mode: PanelMode, window: &mut Window, cx: &mut Context<Self>) {
        self.mode = mode;
        self.filter_menu_open = false;
        if mode == PanelMode::Replace {
            self.replace_input.update(cx, |input, cx| {
                input.focus(window, cx);
            });
        } else {
            self.search_input.update(cx, |input, cx| {
                input.focus(window, cx);
            });
        }
        cx.notify();
    }

    fn set_scope(
        &mut self,
        scope: PagesPanelSearchScope,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.search_scope = scope;
        self.scope_menu_open = false;
        self.emit_search_request(cx);
        self.search_input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
    }

    fn select_result(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(result) = self.results.items.get(index) else {
            return;
        };
        self.active_result = Some(index);
        cx.emit(PagesPanelAction::SearchResultSelected {
            result_id: result.id.clone(),
        });
        cx.notify();
    }

    fn navigate_results(&mut self, direction: PagesPanelResultDirection, cx: &mut Context<Self>) {
        let len = self.results.items.len();
        if len == 0 {
            return;
        }
        let next = Some(match (self.active_result, direction) {
            (None, _) => 0,
            (Some(0), PagesPanelResultDirection::Previous) => len - 1,
            (Some(index), PagesPanelResultDirection::Previous) => index - 1,
            (Some(index), PagesPanelResultDirection::Next) => (index + 1) % len,
        });
        self.active_result = next;
        let result_id = next.map(|index| self.results.items[index].id.clone());
        cx.emit(PagesPanelAction::NavigateResults {
            direction,
            result_id,
        });
        cx.notify();
    }

    fn request_replace(&mut self, all: bool, cx: &mut Context<Self>) {
        if (all && self.results.total == 0) || (!all && self.results.items.is_empty()) {
            return;
        }
        let request = self.build_search_request(cx);
        let replacement = self.replace_input.read(cx).value();
        if all {
            cx.emit(PagesPanelAction::ReplaceAllRequested {
                request,
                replacement,
            });
        } else {
            let result_id = self
                .active_result
                .and_then(|index| self.results.items.get(index))
                .map(|result| result.id.clone());
            cx.emit(PagesPanelAction::ReplaceRequested {
                request,
                result_id,
                replacement,
            });
        }
    }

    fn page_reveal_height(&self) -> f32 {
        let count = self.pages.len() + usize::from(self.editing_is_new());
        (PAGE_PADDING * 2.
            + count as f32 * PAGE_ROW_HEIGHT
            + count.saturating_sub(1) as f32 * PAGE_ROW_GAP)
            .min(MAX_PAGE_LIST_HEIGHT)
    }

    fn editing_is_new(&self) -> bool {
        matches!(self.editing, Some(PageEditorTarget::New { .. }))
    }

    fn render_header(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let expanded = self.expanded;
        let title = if expanded {
            SharedString::from("Pages")
        } else {
            self.collapsed_title()
        };

        let header = h_flex()
            .id(SharedString::from(format!("{}-header", self.id)))
            .debug_selector(|| "pages-header".to_owned())
            .key_context(PAGES_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h(px(HEADER_HEIGHT))
            .w_full()
            .flex_shrink_0()
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().sidebar_accent.opacity(0.55)))
            .focus(|style| {
                style
                    .bg(cx.theme().sidebar_accent.opacity(0.55))
                    .border_color(cx.theme().selection)
            })
            .when(expanded, |header| {
                header.border_b_1().border_color(cx.theme().border)
            })
            .on_action(cx.listener(|this, _: &ActivatePagesControl, _, cx| {
                this.toggle_expanded(cx);
            }))
            .on_click(cx.listener(|this, _, _, cx| this.toggle_expanded(cx)));
        #[cfg(test)]
        let header = header.on_hover(cx.listener(|this, hovered, _, _| {
            this.header_hovered = *hovered;
        }));

        header
            .child(
                h_flex()
                    .id(SharedString::from(format!("{}-toggle", self.id)))
                    .h_full()
                    .flex_1()
                    .gap_1()
                    .px_2()
                    .child(
                        Icon::new(if expanded {
                            IconName::ChevronDown
                        } else {
                            IconName::ChevronRight
                        })
                        .xsmall(),
                    )
                    .child(title),
            )
            .child(
                div()
                    .flex_none()
                    .debug_selector(|| "pages-search-trigger".to_owned())
                    .key_context(PAGES_CONTROL_KEY_CONTEXT)
                    .on_action(cx.listener(|this, _: &ActivatePagesControl, window, cx| {
                        cx.stop_propagation();
                        this.open_search(window, cx);
                    }))
                    .occlude()
                    .child(
                        Button::new(SharedString::from(format!("{}-search", self.id)))
                            .ghost()
                            .xsmall()
                            .compact()
                            .icon(IconName::Search)
                            .tooltip_with_action(
                                "Find",
                                &FindInPages,
                                Some(PAGES_PANEL_KEY_CONTEXT),
                            )
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_search(window, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex_none()
                    .debug_selector(|| "pages-add-trigger".to_owned())
                    .key_context(PAGES_CONTROL_KEY_CONTEXT)
                    .on_action(cx.listener(|this, _: &ActivatePagesControl, window, cx| {
                        cx.stop_propagation();
                        this.begin_new_page(window, cx);
                    }))
                    .occlude()
                    .child(
                        Button::new(SharedString::from(format!("{}-add", self.id)))
                            .ghost()
                            .xsmall()
                            .compact()
                            .icon(IconName::Plus)
                            .tooltip_with_action(
                                "Add new page",
                                &AddPage,
                                Some(PAGES_PANEL_KEY_CONTEXT),
                            )
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.begin_new_page(window, cx);
                            })),
                    ),
            )
            .child(div().w(px(4.)))
            .into_any_element()
    }

    fn render_pages(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let selected_page = self.selected_page.clone();
        let editing = self.editing.clone();
        let content_min_width = px(self
            .pages
            .iter()
            .map(|page| page.title.chars().count())
            .max()
            .unwrap_or(0) as f32
            * APPROXIMATE_PAGE_CHARACTER_WIDTH
            + PAGE_PADDING * 2.);
        let rows = self
            .pages
            .clone()
            .into_iter()
            .enumerate()
            .map(|(index, page)| {
                let is_selected = selected_page.as_ref() == Some(&page.id);
                let is_editing = matches!(
                    editing.as_ref(),
                    Some(PageEditorTarget::Existing { page_id, .. }) if page_id == &page.id
                );
                if is_editing {
                    self.render_page_editor(format!("{}-edit-{index}", self.id))
                } else {
                    self.render_page_row(index, page, is_selected, cx)
                }
            })
            .collect::<Vec<_>>();

        let content = v_flex()
            .w_full()
            .min_w(content_min_width)
            .p_2()
            .gap_1()
            .children(rows)
            .when(self.editing_is_new(), |content| {
                content.child(self.render_page_editor(format!("{}-new-page", self.id)))
            });

        let expanded = self.expanded;
        let height = self.page_reveal_height();
        let reveal = div()
            .w_full()
            .h(px(if expanded { height } else { 0. }))
            .min_h(px(0.))
            .overflow_hidden()
            .child(
                div()
                    .id(SharedString::from(format!("{}-pages-scroll", self.id)))
                    .debug_selector(|| "pages-scroll-viewport".to_owned())
                    .relative()
                    .size_full()
                    .overflow_scroll()
                    .track_scroll(&self.pages_scroll_handle)
                    .child(content)
                    .scrollbar(&self.pages_scroll_handle, ScrollbarAxis::Both),
            );
        #[cfg(test)]
        {
            reveal.into_any_element()
        }
        #[cfg(not(test))]
        {
            reveal
                .with_animation(
                    ElementId::NamedInteger(
                        SharedString::from(format!("{}-reveal", self.id)),
                        expanded as u64,
                    ),
                    Animation::new(Duration::from_secs_f64(REVEAL_DURATION))
                        .with_easing(cubic_bezier(0.4, 0., 0.2, 1.)),
                    move |element, delta| {
                        let progress = if expanded { delta } else { 1. - delta };
                        element.h(px(height * progress)).opacity(progress)
                    },
                )
                .into_any_element()
        }
    }

    fn render_page_editor(&self, id: impl Into<SharedString>) -> AnyElement {
        h_flex()
            .id(id.into())
            .debug_selector(|| "pages-page-editor".to_owned())
            .h(px(PAGE_ROW_HEIGHT))
            .w_full()
            .child(Input::new(&self.rename_input).xsmall())
            .into_any_element()
    }

    fn render_page_row(
        &mut self,
        index: usize,
        page: PagesPanelItem,
        is_selected: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let page_id = page.id.clone();
        let page_title = page.title.clone();
        let keyboard_page_id = page.id.clone();
        let context_page_id = page.id.clone();
        let context_page_title = page.title.clone();
        let panel = cx.entity();
        let row_selector = format!("pages-row-{}", page.id);
        let row_min_width = px(page.title.chars().count() as f32
            * APPROXIMATE_PAGE_CHARACTER_WIDTH
            + PAGE_PADDING * 2.);
        let row = h_flex()
            .id(SharedString::from(format!("{}-page-{index}", self.id)))
            .debug_selector(move || row_selector)
            .key_context(PAGES_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h(px(PAGE_ROW_HEIGHT))
            .w_full()
            .min_w(row_min_width)
            .px_2()
            .rounded(px(4.))
            .text_sm()
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().sidebar_accent.opacity(0.65)))
            .focus(|style| {
                style
                    .bg(cx.theme().sidebar_accent.opacity(0.65))
                    .border_1()
                    .border_color(cx.theme().selection)
            })
            .when(is_selected, |row| {
                row.bg(cx.theme().sidebar_accent)
                    .text_color(cx.theme().sidebar_accent_foreground)
                    .font_semibold()
            });
        #[cfg(test)]
        let row = {
            let hovered_page_id = page_id.clone();
            row.on_hover(cx.listener(move |this, hovered, _, _| {
                if *hovered {
                    this.hovered_page = Some(hovered_page_id.clone());
                } else if this.hovered_page.as_ref() == Some(&hovered_page_id) {
                    this.hovered_page = None;
                }
            }))
        };
        row.on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _, _, cx| this.cancel_page_edit(cx)),
        )
        .on_action(cx.listener(move |this, _: &ActivatePagesControl, _, cx| {
            this.cancel_page_edit(cx);
            cx.emit(PagesPanelAction::SelectRequested {
                page_id: keyboard_page_id.clone(),
            });
        }))
        .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
            if event.click_count() >= 2 {
                this.begin_rename(page_id.clone(), page_title.clone(), window, cx);
            } else {
                cx.emit(PagesPanelAction::SelectRequested {
                    page_id: page_id.clone(),
                });
            }
        }))
        .child(div().flex_none().whitespace_nowrap().child(page.title))
        .context_menu(move |menu, _, _| {
            let copy_link_panel = panel.clone();
            let copy_link_page_id = context_page_id.clone();
            let rename_panel = panel.clone();
            let rename_page_id = context_page_id.clone();
            let rename_page_title = context_page_title.clone();
            let duplicate_panel = panel.clone();
            let duplicate_page_id = context_page_id.clone();
            let delete_panel = panel.clone();
            let delete_page_id = context_page_id.clone();

            menu.min_w(px(224.))
                .item(page_context_menu_item(
                    "Copy link to page",
                    "pages-page-menu-copy-link",
                    move |_, _, app| {
                        copy_link_panel.update(app, |_, cx| {
                            cx.emit(PagesPanelAction::CopyLinkRequested {
                                page_id: copy_link_page_id.clone(),
                            });
                        });
                    },
                ))
                .separator()
                .item(page_context_menu_item(
                    "Rename page",
                    "pages-page-menu-rename",
                    move |_, window, app| {
                        rename_panel.update(app, |panel, cx| {
                            panel.begin_rename(
                                rename_page_id.clone(),
                                rename_page_title.clone(),
                                window,
                                cx,
                            );
                        });
                    },
                ))
                .item(page_context_menu_item(
                    "Duplicate page",
                    "pages-page-menu-duplicate",
                    move |_, _, app| {
                        duplicate_panel.update(app, |_, cx| {
                            cx.emit(PagesPanelAction::DuplicateRequested {
                                page_id: duplicate_page_id.clone(),
                            });
                        });
                    },
                ))
                .separator()
                .item(page_context_menu_item(
                    "Delete page",
                    "pages-page-menu-delete",
                    move |_, _, app| {
                        delete_panel.update(app, |_, cx| {
                            cx.emit(PagesPanelAction::DeleteRequested {
                                page_id: delete_page_id.clone(),
                            });
                        });
                    },
                ))
        })
        .into_any_element()
    }

    fn element_icon(kind: PagesPanelElementKind) -> IconName {
        match kind {
            PagesPanelElementKind::All => IconName::GalleryVerticalEnd,
            PagesPanelElementKind::Text => IconName::ALargeSmall,
            PagesPanelElementKind::FrameGroup => IconName::Frame,
            PagesPanelElementKind::Component => IconName::Asterisk,
            PagesPanelElementKind::Instance => IconName::Copy,
            PagesPanelElementKind::Image => IconName::File,
            PagesPanelElementKind::Shape => IconName::ChartPie,
            PagesPanelElementKind::Other => IconName::Ellipsis,
        }
    }

    fn render_search_toolbar(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let replace = self.mode == PanelMode::Replace;
        let can_replace_one = !self.results.items.is_empty();
        let can_replace_all = self.results.total > 0;
        let panel = cx.entity();
        v_flex()
            .w_full()
            .gap_2()
            .p_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .w_full()
                    .gap_1()
                    .child(
                        div().flex_1().child(
                            Input::new(&self.search_input)
                                .small()
                                .prefix(Icon::new(IconName::Search).small()),
                        ),
                    )
                    .child(
                        div()
                            .relative()
                            .flex_none()
                            .debug_selector(|| "pages-filter-trigger".to_owned())
                            .key_context(PAGES_CONTROL_KEY_CONTEXT)
                            .on_action(cx.listener(|this, _: &ActivatePagesControl, window, cx| {
                                cx.stop_propagation();
                                this.toggle_search_settings(window, cx);
                            }))
                            .child(
                                Button::new(SharedString::from(format!("{}-filters", self.id)))
                                    .ghost()
                                    .small()
                                    .compact()
                                    .icon(IconName::Settings2)
                                    .tooltip_with_action(
                                        "Settings",
                                        &ToggleSearchSettings,
                                        Some(PAGES_PANEL_KEY_CONTEXT),
                                    )
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.toggle_search_settings(window, cx);
                                    })),
                            )
                            .child(
                                canvas(
                                    move |bounds, _, app| {
                                        panel.update(app, |this, _| {
                                            this.filter_anchor_bounds = Some(bounds);
                                        });
                                    },
                                    |_, _, _, _| {},
                                )
                                .absolute()
                                .top_0()
                                .left_0()
                                .size_full(),
                            ),
                    )
                    .child(
                        div()
                            .flex_none()
                            .debug_selector(|| "pages-close-search".to_owned())
                            .key_context(PAGES_CONTROL_KEY_CONTEXT)
                            .on_action(cx.listener(|this, _: &ActivatePagesControl, _, cx| {
                                cx.stop_propagation();
                                this.close_search(cx);
                            }))
                            .child(
                                Button::new(SharedString::from(format!(
                                    "{}-close-search",
                                    self.id
                                )))
                                .ghost()
                                .small()
                                .compact()
                                .icon(IconName::Close)
                                .tooltip_with_action(
                                    "Close search",
                                    &ClosePagesSearch,
                                    Some(PAGES_PANEL_KEY_CONTEXT),
                                )
                                .on_click(cx.listener(|this, _, _, cx| this.close_search(cx))),
                            ),
                    ),
            )
            .when(replace, |toolbar| {
                toolbar
                    .child(
                        Input::new(&self.replace_input)
                            .small()
                            .prefix(Icon::new(IconName::ArrowRight).small()),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .justify_end()
                            .gap_2()
                            .child(
                                div()
                                    .flex_none()
                                    .debug_selector(|| "pages-replace-one".to_owned())
                                    .key_context(PAGES_CONTROL_KEY_CONTEXT)
                                    .on_action(cx.listener(
                                        move |this, _: &ActivatePagesControl, _, cx| {
                                            cx.stop_propagation();
                                            if can_replace_one {
                                                this.request_replace(false, cx);
                                            }
                                        },
                                    ))
                                    .child(
                                        Button::new(SharedString::from(format!(
                                            "{}-replace-one",
                                            self.id
                                        )))
                                        .label("Replace")
                                        .small()
                                        .disabled(!can_replace_one)
                                        .tooltip_with_action(
                                            "Replace current result",
                                            &ReplaceCurrentResult,
                                            Some(PAGES_PANEL_KEY_CONTEXT),
                                        )
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.request_replace(false, cx);
                                            }),
                                        ),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_none()
                                    .debug_selector(|| "pages-replace-all".to_owned())
                                    .key_context(PAGES_CONTROL_KEY_CONTEXT)
                                    .on_action(cx.listener(
                                        move |this, _: &ActivatePagesControl, _, cx| {
                                            cx.stop_propagation();
                                            if can_replace_all {
                                                this.request_replace(true, cx);
                                            }
                                        },
                                    ))
                                    .child(
                                        Button::new(SharedString::from(format!(
                                            "{}-replace-all",
                                            self.id
                                        )))
                                        .label("Replace all")
                                        .small()
                                        .disabled(!can_replace_all)
                                        .tooltip_with_action(
                                            "Replace all results",
                                            &ReplaceAllResults,
                                            Some(PAGES_PANEL_KEY_CONTEXT),
                                        )
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.request_replace(true, cx);
                                            }),
                                        ),
                                    ),
                            ),
                    )
            })
            .when(!self.active_filters.is_empty(), |toolbar| {
                toolbar.child(h_flex().w_full().gap_1().flex_wrap().children(
                    self.active_filters.clone().into_iter().map(|kind| {
                        h_flex()
                            .id(SharedString::from(format!(
                                "{}-filter-pill-{}",
                                self.id,
                                kind.label()
                            )))
                            .key_context(PAGES_CONTROL_KEY_CONTEXT)
                            .tab_index(0)
                            .h(px(28.))
                            .gap_1()
                            .px_2()
                            .rounded(px(6.))
                            .border_1()
                            .border_color(cx.theme().border)
                            .cursor_pointer()
                            .hover(|style| style.bg(cx.theme().accent))
                            .focus(|style| {
                                style
                                    .bg(cx.theme().accent)
                                    .border_color(cx.theme().selection)
                            })
                            .on_action(cx.listener(move |this, _: &ActivatePagesControl, _, cx| {
                                this.toggle_filter(kind, cx);
                            }))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.toggle_filter(kind, cx);
                            }))
                            .child(kind.label())
                            .child(Icon::new(IconName::Close).xsmall())
                    }),
                ))
            })
            .into_any_element()
    }

    fn render_results_header(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let result_label = result_count_label(self.results.total);
        let can_navigate = !self.results.items.is_empty();
        let panel = cx.entity();
        h_flex()
            .debug_selector(|| "pages-results-header".to_owned())
            .h(px(48.))
            .w_full()
            .min_w(px(0.))
            .px_3()
            .gap_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .debug_selector(|| "pages-result-count".to_owned())
                    .flex_none()
                    .whitespace_nowrap()
                    .text_sm()
                    .child(result_label),
            )
            .child(div().flex_none().text_sm().child("·"))
            .child(
                h_flex()
                    .id(SharedString::from(format!("{}-scope", self.id)))
                    .relative()
                    .flex_none()
                    .whitespace_nowrap()
                    .debug_selector(|| "pages-scope-trigger".to_owned())
                    .key_context(PAGES_CONTROL_KEY_CONTEXT)
                    .tab_index(0)
                    .gap_1()
                    .text_sm()
                    .cursor_pointer()
                    .focus(|style| {
                        style
                            .bg(cx.theme().accent)
                            .border_color(cx.theme().selection)
                    })
                    .on_action(cx.listener(|this, _: &ActivatePagesControl, window, cx| {
                        this.toggle_search_scope(window, cx);
                    }))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.toggle_search_scope(window, cx);
                    }))
                    .child(self.search_scope.label())
                    .child(Icon::new(IconName::ChevronDown).xsmall())
                    .child(
                        canvas(
                            move |bounds, _, app| {
                                panel.update(app, |this, _| {
                                    this.scope_anchor_bounds = Some(bounds);
                                });
                            },
                            |_, _, _, _| {},
                        )
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full(),
                    ),
            )
            .child(div().flex_1())
            .child(
                div()
                    .flex_none()
                    .debug_selector(|| "pages-previous-result".to_owned())
                    .key_context(PAGES_CONTROL_KEY_CONTEXT)
                    .on_action(cx.listener(move |this, _: &ActivatePagesControl, _, cx| {
                        cx.stop_propagation();
                        if can_navigate {
                            this.navigate_results(PagesPanelResultDirection::Previous, cx);
                        }
                    }))
                    .child(
                        Button::new(SharedString::from(format!("{}-previous", self.id)))
                            .ghost()
                            .xsmall()
                            .compact()
                            .icon(IconName::ChevronUp)
                            .disabled(!can_navigate)
                            .tooltip_with_action(
                                "Previous result",
                                &PreviousSearchResult,
                                Some(PAGES_PANEL_KEY_CONTEXT),
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_results(PagesPanelResultDirection::Previous, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex_none()
                    .debug_selector(|| "pages-next-result".to_owned())
                    .key_context(PAGES_CONTROL_KEY_CONTEXT)
                    .on_action(cx.listener(move |this, _: &ActivatePagesControl, _, cx| {
                        cx.stop_propagation();
                        if can_navigate {
                            this.navigate_results(PagesPanelResultDirection::Next, cx);
                        }
                    }))
                    .child(
                        Button::new(SharedString::from(format!("{}-next", self.id)))
                            .ghost()
                            .xsmall()
                            .compact()
                            .icon(IconName::ChevronDown)
                            .disabled(!can_navigate)
                            .tooltip_with_action(
                                "Next result",
                                &NextSearchResult,
                                Some(PAGES_PANEL_KEY_CONTEXT),
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_results(PagesPanelResultDirection::Next, cx);
                            })),
                    ),
            )
            .into_any_element()
    }

    fn render_results(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let active = self.active_result;
        let replacement = self.replace_input.read(cx).value();
        let replace_mode = self.mode == PanelMode::Replace;
        let empty = self.results.items.is_empty();

        v_flex()
            .debug_selector(|| "pages-results-list".to_owned())
            .w_full()
            .when(empty, |results| {
                results.child(
                    div()
                        .debug_selector(|| "pages-results-empty".to_owned())
                        .w_full()
                        .p_4()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(empty_results_label(self.search_scope)),
                )
            })
            .children(
                self.results
                    .items
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(index, result)| {
                        let is_active = active == Some(index);
                        let row = h_flex()
                            .id(SharedString::from(format!("{}-result-{index}", self.id)))
                            .debug_selector(move || format!("pages-result-{index}"))
                            .key_context(PAGES_CONTROL_KEY_CONTEXT)
                            .tab_index(0)
                            .min_h(px(52.))
                            .w_full()
                            .gap_2()
                            .px_4()
                            .py_2()
                            .cursor_pointer()
                            .hover(|style| style.bg(cx.theme().accent))
                            .focus(|style| {
                                style
                                    .bg(cx.theme().accent)
                                    .border_l_2()
                                    .border_color(cx.theme().selection)
                            })
                            .when(is_active, |row| {
                                row.bg(cx.theme().list_active)
                                    .border_l_2()
                                    .border_color(cx.theme().list_active_border)
                            });
                        #[cfg(test)]
                        let row = row.on_hover(cx.listener(move |this, hovered, _, _| {
                            if *hovered {
                                this.hovered_result = Some(index);
                            } else if this.hovered_result == Some(index) {
                                this.hovered_result = None;
                            }
                        }));
                        row.on_click(cx.listener(move |this, _, _, cx| {
                            this.select_result(index, cx);
                        }))
                        .on_action(cx.listener(move |this, _: &ActivatePagesControl, _, cx| {
                            this.select_result(index, cx);
                        }))
                        .child(Icon::new(Self::element_icon(result.kind)).small())
                        .child(
                            v_flex()
                                .gap_0p5()
                                .child(
                                    h_flex()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_sm()
                                                .when(
                                                    replace_mode && !replacement.is_empty(),
                                                    |text| {
                                                        text.text_color(cx.theme().muted_foreground)
                                                            .line_through()
                                                    },
                                                )
                                                .child(result.title),
                                        )
                                        .when(replace_mode && !replacement.is_empty(), |line| {
                                            line.child(replacement.clone())
                                        }),
                                )
                                .when_some(result.parent, |column, parent| {
                                    column.child(
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(parent),
                                    )
                                }),
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_filter_menu(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let (Some(panel_bounds), Some(anchor_bounds)) =
            (self.search_panel_bounds, self.filter_anchor_bounds)
        else {
            return div().into_any_element();
        };
        let origin = anchor_bounds.bottom_left() + point(px(0.), px(4.)) - panel_bounds.origin;
        let counts = self.results.element_counts.clone();
        let all_active = self.active_filters.is_empty();
        let mode = self.mode;

        deferred(
            v_flex()
                .absolute()
                .left(origin.x)
                .top(origin.y)
                .debug_selector(|| "pages-filter-menu".to_owned())
                .occlude()
                .w(px(224.))
                .py_2()
                .rounded(px(12.))
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().popover)
                .shadow_lg()
                .child(self.render_mode_item(PanelMode::Find, "Find", mode, cx))
                .child(self.render_mode_item(PanelMode::Replace, "Replace", mode, cx))
                .child(div().h(px(1.)).w_full().my_2().bg(cx.theme().border))
                .children(PagesPanelElementKind::FILTER_ORDER.into_iter().map(|kind| {
                    let active = if kind == PagesPanelElementKind::All {
                        all_active
                    } else {
                        self.active_filters.contains(&kind)
                    };
                    let count = counts
                        .iter()
                        .find(|item| item.kind == kind)
                        .map(|item| item.count);
                    h_flex()
                        .id(SharedString::from(format!(
                            "{}-filter-{}",
                            self.id,
                            kind.label()
                        )))
                        .key_context(PAGES_CONTROL_KEY_CONTEXT)
                        .tab_index(0)
                        .h(px(32.))
                        .mx_2()
                        .px_2()
                        .gap_2()
                        .rounded(px(5.))
                        .cursor_pointer()
                        .hover(|style| style.bg(cx.theme().accent))
                        .focus(|style| {
                            style
                                .bg(cx.theme().accent)
                                .border_color(cx.theme().selection)
                        })
                        .on_action(cx.listener(move |this, _: &ActivatePagesControl, _, cx| {
                            this.toggle_filter(kind, cx);
                        }))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.toggle_filter(kind, cx);
                        }))
                        .child(div().w(px(14.)).when(active, |slot| {
                            slot.child(Icon::new(IconName::Check).xsmall())
                        }))
                        .child(Icon::new(Self::element_icon(kind)).small())
                        .child(div().flex_1().text_sm().child(kind.label()))
                        .when_some(count, |row, count| {
                            row.child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(count.to_string()),
                            )
                        })
                }))
                .child(div().h(px(1.)).w_full().my_2().bg(cx.theme().border))
                .child(self.render_option_item("Match case", self.match_case, true, cx))
                .child(self.render_option_item("Whole words", self.whole_words, false, cx)),
        )
        .with_priority(2)
        .into_any_element()
    }

    fn render_mode_item(
        &mut self,
        item_mode: PanelMode,
        label: &'static str,
        active_mode: PanelMode,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selector = format!("pages-mode-{}", label.to_lowercase());
        let menu_focus_handle = self.filter_menu_focus_handle.clone();
        h_flex()
            .id(SharedString::from(format!("{}-mode-{label}", self.id)))
            .debug_selector(move || selector)
            .key_context(PAGES_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .when(item_mode == PanelMode::Find, |row| {
                row.track_focus(&menu_focus_handle)
            })
            .h(px(32.))
            .mx_2()
            .px_2()
            .gap_2()
            .rounded(px(5.))
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().accent))
            .focus(|style| {
                style
                    .bg(cx.theme().accent)
                    .border_color(cx.theme().selection)
            })
            .when(item_mode == active_mode, |row| row.bg(cx.theme().accent))
            .on_action(
                cx.listener(move |this, _: &ActivatePagesControl, window, cx| {
                    this.set_panel_mode(item_mode, window, cx);
                }),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                this.set_panel_mode(item_mode, window, cx);
            }))
            .child(div().w(px(14.)).when(item_mode == active_mode, |slot| {
                slot.child(Icon::new(IconName::Check).xsmall())
            }))
            .child(label)
            .into_any_element()
    }

    fn render_option_item(
        &mut self,
        label: &'static str,
        active: bool,
        match_case: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        h_flex()
            .id(SharedString::from(format!("{}-option-{label}", self.id)))
            .key_context(PAGES_CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .h(px(32.))
            .mx_2()
            .px_2()
            .gap_2()
            .rounded(px(5.))
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().accent))
            .focus(|style| {
                style
                    .bg(cx.theme().accent)
                    .border_color(cx.theme().selection)
            })
            .on_action(cx.listener(move |this, _: &ActivatePagesControl, _, cx| {
                if match_case {
                    this.toggle_match_case(cx);
                } else {
                    this.toggle_whole_words(cx);
                }
            }))
            .on_click(cx.listener(move |this, _, _, cx| {
                if match_case {
                    this.toggle_match_case(cx);
                } else {
                    this.toggle_whole_words(cx);
                }
            }))
            .child(div().w(px(14.)).when(active, |slot| {
                slot.child(Icon::new(IconName::Check).xsmall())
            }))
            .child(label)
            .into_any_element()
    }

    fn render_scope_menu(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let (Some(panel_bounds), Some(anchor_bounds)) =
            (self.search_panel_bounds, self.scope_anchor_bounds)
        else {
            return div().into_any_element();
        };
        let origin = anchor_bounds.bottom_left() + point(px(0.), px(4.)) - panel_bounds.origin;
        let menu_focus_handle = self.scope_menu_focus_handle.clone();

        deferred(
            v_flex()
                .absolute()
                .left(origin.x)
                .top(origin.y)
                .debug_selector(|| "pages-scope-menu".to_owned())
                .occlude()
                .w(px(168.))
                .py_2()
                .rounded(px(12.))
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().popover)
                .shadow_lg()
                .children(
                    [
                        PagesPanelSearchScope::CurrentPage,
                        PagesPanelSearchScope::AllPages,
                    ]
                    .into_iter()
                    .map(|scope| {
                        let menu_focus_handle = menu_focus_handle.clone();
                        let selector = match scope {
                            PagesPanelSearchScope::CurrentPage => "pages-scope-current-page",
                            PagesPanelSearchScope::AllPages => "pages-scope-all-pages",
                        };
                        h_flex()
                            .id(SharedString::from(format!(
                                "{}-scope-{}",
                                self.id,
                                scope.label()
                            )))
                            .debug_selector(move || selector.to_owned())
                            .key_context(PAGES_CONTROL_KEY_CONTEXT)
                            .tab_index(0)
                            .when(scope == PagesPanelSearchScope::CurrentPage, |row| {
                                row.track_focus(&menu_focus_handle)
                            })
                            .h(px(36.))
                            .mx_2()
                            .px_2()
                            .gap_2()
                            .rounded(px(5.))
                            .cursor_pointer()
                            .hover(|style| style.bg(cx.theme().accent))
                            .focus(|style| {
                                style
                                    .bg(cx.theme().accent)
                                    .border_color(cx.theme().selection)
                            })
                            .on_action(cx.listener(
                                move |this, _: &ActivatePagesControl, window, cx| {
                                    this.set_scope(scope, window, cx);
                                },
                            ))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.set_scope(scope, window, cx);
                            }))
                            .child(div().w(px(14.)).when(scope == self.search_scope, |slot| {
                                slot.child(Icon::new(IconName::Check).xsmall())
                            }))
                            .child(scope.label())
                    }),
                ),
        )
        .with_priority(2)
        .into_any_element()
    }

    fn render_search(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let panel = cx.entity();
        v_flex()
            .relative()
            .size_full()
            .min_h(px(280.))
            .bg(cx.theme().sidebar)
            .child(
                canvas(
                    move |bounds, _, app| {
                        panel.update(app, |this, _| {
                            this.search_panel_bounds = Some(bounds);
                        });
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            )
            .child(self.render_search_toolbar(cx))
            .child(self.render_results_header(cx))
            .child(
                div()
                    .id(SharedString::from(format!("{}-results-scroll", self.id)))
                    .debug_selector(|| "pages-results-viewport".to_owned())
                    .relative()
                    .flex_1()
                    .min_h(px(0.))
                    .overflow_scroll()
                    .track_scroll(&self.results_scroll_handle)
                    .child(self.render_results(cx))
                    .vertical_scrollbar(&self.results_scroll_handle),
            )
            .when(self.filter_menu_open, |panel| {
                panel.child(self.render_filter_menu(cx))
            })
            .when(self.scope_menu_open, |panel| {
                panel.child(self.render_scope_menu(cx))
            })
            .into_any_element()
    }
}

impl Focusable for PagesPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PagesPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let id = self.id.clone();
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
            .w_full()
            .max_h_full()
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().sidebar_foreground)
            .border_1()
            .border_color(cx.theme().border);
        if self.mode == PanelMode::Pages {
            root.overflow_hidden()
                .child(self.render_header(cx))
                .child(self.render_pages(cx))
                .into_any_element()
        } else {
            root.debug_selector(|| "pages-search-panel".to_owned())
                .size_full()
                .child(self.render_search(cx))
                .into_any_element()
        }
    }
}

fn result_count_label(total: usize) -> String {
    if total == 1 {
        "1 result".to_owned()
    } else {
        format!("{total} results")
    }
}

fn empty_results_label(scope: PagesPanelSearchScope) -> String {
    format!("No results on {}", scope.label().to_lowercase())
}

fn page_context_menu_item(
    label: &'static str,
    selector: &'static str,
    handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> PopupMenuItem {
    PopupMenuItem::element(move |_, cx| {
        div()
            .debug_selector(move || selector.to_owned())
            .tab_index(0)
            .w_full()
            .focus(|style| style.bg(cx.theme().accent))
            .child(label)
    })
    .on_click(handler)
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

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use gpui::{
        AppContext as _, Bounds, Context, Entity, Focusable as _, IntoElement, Modifiers,
        MouseButton, MouseDownEvent, MouseUpEvent, ParentElement as _, Pixels, Render, ScrollDelta,
        ScrollWheelEvent, Styled as _, Subscription, TestAppContext, VisualTestContext, Window,
        div, point, px,
    };
    use gpui_component::Root;

    use super::{
        PageEditorTarget, PagesPanel, PanelMode, empty_results_label, next_page_title,
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
            div().w(px(320.)).h(px(220.)).child(self.panel.clone())
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

    fn actions(
        host: &Entity<TestHost>,
        cx: &VisualTestContext,
    ) -> Rc<RefCell<Vec<PagesPanelAction>>> {
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
    fn empty_results_render_in_the_scrollable_body_without_header_overflow(
        cx: &mut TestAppContext,
    ) {
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
    fn popovers_are_top_left_anchored_and_replace_does_not_displace_filter(
        cx: &mut TestAppContext,
    ) {
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
    fn long_page_names_scroll_horizontally_and_many_pages_scroll_vertically(
        cx: &mut TestAppContext,
    ) {
        let (host, cx) = setup(cx);
        let panel = panel(&host, cx);
        let pages = (1..=14)
            .map(|index| {
                let title = if index == 1 {
                    "This page name is intentionally far wider than the Pages panel viewport"
                        .to_owned()
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
        let long_row = bounds(cx, "pages-row-long-page-1");
        let scroll_handle = read_panel(&panel, cx, |panel| panel.pages_scroll_handle.clone());
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
        let scroll_handle = read_panel(&panel, cx, |panel| panel.results_scroll_handle.clone());
        assert!(scroll_handle.max_offset().height > px(0.));

        cx.simulate_event(ScrollWheelEvent {
            position: viewport.center(),
            delta: ScrollDelta::Pixels(point(px(0.), px(-160.))),
            ..Default::default()
        });
        assert!(scroll_handle.offset().y < px(0.));
    }
}
