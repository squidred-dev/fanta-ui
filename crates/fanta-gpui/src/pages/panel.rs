use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, AnyElement, App, AppContext as _, ClickEvent, Context, ElementId,
    Entity, EventEmitter, InteractiveElement as _, IntoElement, ParentElement as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Subscription, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _,
    animation::cubic_bezier,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState, SelectAll},
    v_flex,
};

use super::{
    PagesPanelAction, PagesPanelElementKind, PagesPanelItem, PagesPanelResultDirection,
    PagesPanelSearchRequest, PagesPanelSearchResults, PagesPanelSearchScope,
};

const HEADER_HEIGHT: f32 = 40.;
const PAGE_ROW_HEIGHT: f32 = 32.;
const PAGE_ROW_GAP: f32 = 4.;
const PAGE_PADDING: f32 = 8.;
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
    results: PagesPanelSearchResults,
    active_result: Option<usize>,
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
            results: PagesPanelSearchResults::default(),
            active_result: None,
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

    fn set_scope(&mut self, scope: PagesPanelSearchScope, cx: &mut Context<Self>) {
        self.search_scope = scope;
        self.scope_menu_open = false;
        self.emit_search_request(cx);
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
        let next = if len == 0 {
            None
        } else {
            Some(match (self.active_result, direction) {
                (None, _) => 0,
                (Some(0), PagesPanelResultDirection::Previous) => len - 1,
                (Some(index), PagesPanelResultDirection::Previous) => index - 1,
                (Some(index), PagesPanelResultDirection::Next) => (index + 1) % len,
            })
        };
        self.active_result = next;
        let result_id = next.map(|index| self.results.items[index].id.clone());
        cx.emit(PagesPanelAction::NavigateResults {
            direction,
            result_id,
        });
        cx.notify();
    }

    fn request_replace(&mut self, all: bool, cx: &mut Context<Self>) {
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
        PAGE_PADDING * 2.
            + count as f32 * PAGE_ROW_HEIGHT
            + count.saturating_sub(1) as f32 * PAGE_ROW_GAP
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

        h_flex()
            .h(px(HEADER_HEIGHT))
            .w_full()
            .flex_shrink_0()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .id(SharedString::from(format!("{}-toggle", self.id)))
                    .h_full()
                    .flex_1()
                    .gap_1()
                    .px_2()
                    .cursor_pointer()
                    .hover(|style| style.bg(cx.theme().sidebar_accent.opacity(0.55)))
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_expanded(cx)))
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
                Button::new(SharedString::from(format!("{}-search", self.id)))
                    .ghost()
                    .xsmall()
                    .compact()
                    .icon(IconName::Search)
                    .tooltip("Find elements")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.open_search(window, cx);
                    })),
            )
            .child(
                Button::new(SharedString::from(format!("{}-add", self.id)))
                    .ghost()
                    .xsmall()
                    .compact()
                    .icon(IconName::Plus)
                    .tooltip("Add page")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.begin_new_page(window, cx);
                    })),
            )
            .child(div().w(px(4.)))
            .into_any_element()
    }

    fn render_pages(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let selected_page = self.selected_page.clone();
        let editing = self.editing.clone();
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
            .p_2()
            .gap_1()
            .children(rows)
            .when(self.editing_is_new(), |content| {
                content.child(self.render_page_editor(format!("{}-new-page", self.id)))
            });

        let expanded = self.expanded;
        let height = self.page_reveal_height();
        div()
            .w_full()
            .h(px(if expanded { height } else { 0. }))
            .overflow_hidden()
            .child(content)
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

    fn render_page_editor(&self, id: impl Into<SharedString>) -> AnyElement {
        h_flex()
            .id(id.into())
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
        h_flex()
            .id(SharedString::from(format!("{}-page-{index}", self.id)))
            .h(px(PAGE_ROW_HEIGHT))
            .w_full()
            .px_2()
            .rounded(px(4.))
            .text_sm()
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().sidebar_accent.opacity(0.65)))
            .when(is_selected, |row| {
                row.bg(cx.theme().sidebar_accent)
                    .text_color(cx.theme().sidebar_accent_foreground)
                    .font_semibold()
            })
            .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                if event.click_count() >= 2 {
                    this.begin_rename(page_id.clone(), page_title.clone(), window, cx);
                } else {
                    cx.emit(PagesPanelAction::SelectRequested {
                        page_id: page_id.clone(),
                    });
                }
            }))
            .child(page.title)
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
                        Button::new(SharedString::from(format!("{}-filters", self.id)))
                            .ghost()
                            .small()
                            .compact()
                            .icon(IconName::Settings2)
                            .tooltip("Search options")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.filter_menu_open = !this.filter_menu_open;
                                this.scope_menu_open = false;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new(SharedString::from(format!("{}-close-search", self.id)))
                            .ghost()
                            .small()
                            .compact()
                            .icon(IconName::Close)
                            .tooltip("Close search")
                            .on_click(cx.listener(|this, _, _, cx| this.close_search(cx))),
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
                                Button::new(SharedString::from(format!("{}-replace-one", self.id)))
                                    .label("Replace")
                                    .small()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.request_replace(false, cx);
                                    })),
                            )
                            .child(
                                Button::new(SharedString::from(format!("{}-replace-all", self.id)))
                                    .label("Replace all")
                                    .small()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.request_replace(true, cx);
                                    })),
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
                            .h(px(28.))
                            .gap_1()
                            .px_2()
                            .rounded(px(6.))
                            .border_1()
                            .border_color(cx.theme().border)
                            .cursor_pointer()
                            .hover(|style| style.bg(cx.theme().accent))
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
        let result_label = if self.results.total == 0 {
            format!("No results on {}", self.search_scope.label().to_lowercase())
        } else {
            format!("{} results", self.results.total)
        };
        h_flex()
            .h(px(48.))
            .w_full()
            .px_3()
            .gap_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(div().text_sm().child(result_label))
            .child(div().text_sm().child("·"))
            .child(
                h_flex()
                    .id(SharedString::from(format!("{}-scope", self.id)))
                    .gap_1()
                    .text_sm()
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.scope_menu_open = !this.scope_menu_open;
                        this.filter_menu_open = false;
                        cx.notify();
                    }))
                    .child(self.search_scope.label())
                    .child(Icon::new(IconName::ChevronDown).xsmall()),
            )
            .child(div().flex_1())
            .child(
                Button::new(SharedString::from(format!("{}-previous", self.id)))
                    .ghost()
                    .xsmall()
                    .compact()
                    .icon(IconName::ChevronUp)
                    .tooltip("Previous result")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.navigate_results(PagesPanelResultDirection::Previous, cx);
                    })),
            )
            .child(
                Button::new(SharedString::from(format!("{}-next", self.id)))
                    .ghost()
                    .xsmall()
                    .compact()
                    .icon(IconName::ChevronDown)
                    .tooltip("Next result")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.navigate_results(PagesPanelResultDirection::Next, cx);
                    })),
            )
            .into_any_element()
    }

    fn render_results(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let active = self.active_result;
        let replacement = self.replace_input.read(cx).value();
        let replace_mode = self.mode == PanelMode::Replace;

        v_flex()
            .w_full()
            .children(
                self.results
                    .items
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(index, result)| {
                        let is_active = active == Some(index);
                        h_flex()
                            .id(SharedString::from(format!("{}-result-{index}", self.id)))
                            .min_h(px(52.))
                            .w_full()
                            .gap_2()
                            .px_4()
                            .py_2()
                            .cursor_pointer()
                            .hover(|style| style.bg(cx.theme().accent))
                            .when(is_active, |row| {
                                row.bg(cx.theme().list_active)
                                    .border_l_2()
                                    .border_color(cx.theme().list_active_border)
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
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
                                                            text.text_color(
                                                                cx.theme().muted_foreground,
                                                            )
                                                            .line_through()
                                                        },
                                                    )
                                                    .child(result.title),
                                            )
                                            .when(
                                                replace_mode && !replacement.is_empty(),
                                                |line| line.child(replacement.clone()),
                                            ),
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
        let counts = self.results.element_counts.clone();
        let all_active = self.active_filters.is_empty();
        let mode = self.mode;

        v_flex()
            .absolute()
            .top(px(if mode == PanelMode::Replace {
                144.
            } else {
                58.
            }))
            .right(px(34.))
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
                    .h(px(32.))
                    .mx_2()
                    .px_2()
                    .gap_2()
                    .rounded(px(5.))
                    .cursor_pointer()
                    .hover(|style| style.bg(cx.theme().accent))
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
            .child(self.render_option_item("Whole words", self.whole_words, false, cx))
            .into_any_element()
    }

    fn render_mode_item(
        &mut self,
        item_mode: PanelMode,
        label: &'static str,
        active_mode: PanelMode,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        h_flex()
            .id(SharedString::from(format!("{}-mode-{label}", self.id)))
            .h(px(32.))
            .mx_2()
            .px_2()
            .gap_2()
            .rounded(px(5.))
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().accent))
            .when(item_mode == active_mode, |row| row.bg(cx.theme().accent))
            .on_click(cx.listener(move |this, _, window, cx| {
                this.mode = item_mode;
                this.filter_menu_open = false;
                if item_mode == PanelMode::Replace {
                    this.replace_input.update(cx, |input, cx| {
                        input.focus(window, cx);
                    });
                } else {
                    this.search_input.update(cx, |input, cx| {
                        input.focus(window, cx);
                    });
                }
                cx.notify();
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
            .h(px(32.))
            .mx_2()
            .px_2()
            .gap_2()
            .rounded(px(5.))
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().accent))
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
        v_flex()
            .absolute()
            .top(px(if self.mode == PanelMode::Replace {
                220.
            } else {
                108.
            }))
            .left(px(72.))
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
                    h_flex()
                        .id(SharedString::from(format!(
                            "{}-scope-{}",
                            self.id,
                            scope.label()
                        )))
                        .h(px(36.))
                        .mx_2()
                        .px_2()
                        .gap_2()
                        .rounded(px(5.))
                        .cursor_pointer()
                        .hover(|style| style.bg(cx.theme().accent))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.set_scope(scope, cx);
                        }))
                        .child(div().w(px(14.)).when(scope == self.search_scope, |slot| {
                            slot.child(Icon::new(IconName::Check).xsmall())
                        }))
                        .child(scope.label())
                }),
            )
            .into_any_element()
    }

    fn render_search(&mut self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .relative()
            .size_full()
            .min_h(px(280.))
            .bg(cx.theme().sidebar)
            .child(self.render_search_toolbar(cx))
            .child(self.render_results_header(cx))
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(self.render_results(cx)),
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

impl Render for PagesPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let id = self.id.clone();
        if self.mode == PanelMode::Pages {
            v_flex()
                .id(id)
                .w_full()
                .overflow_hidden()
                .bg(cx.theme().sidebar)
                .text_color(cx.theme().sidebar_foreground)
                .border_1()
                .border_color(cx.theme().border)
                .child(self.render_header(cx))
                .child(self.render_pages(cx))
                .into_any_element()
        } else {
            v_flex()
                .id(id)
                .size_full()
                .overflow_hidden()
                .bg(cx.theme().sidebar)
                .text_color(cx.theme().sidebar_foreground)
                .border_1()
                .border_color(cx.theme().border)
                .child(self.render_search(cx))
                .into_any_element()
        }
    }
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
    use super::next_page_title;
    use crate::pages::PagesPanelItem;

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
}
