#![forbid(unsafe_code)]

use fanta_gpui::prelude::{
    PagesPanel, PagesPanelAction, PagesPanelElementCount, PagesPanelElementKind, PagesPanelItem,
    PagesPanelSearchRequest, PagesPanelSearchResult, PagesPanelSearchResults,
    PagesPanelSearchScope,
};
use gpui::{
    App, AppContext as _, Application, Bounds, ClipboardItem, Context, Entity, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, Subscription, TitlebarOptions, Window,
    WindowBounds, WindowOptions, div, px, size,
};
use gpui_component::{ActiveTheme as _, Root, StyledExt as _, Theme, ThemeMode, h_flex, v_flex};
use gpui_component_assets::Assets;

#[derive(Clone)]
struct MockElement {
    result: PagesPanelSearchResult,
    page_id: SharedString,
}

struct Storybook {
    panel: Entity<PagesPanel>,
    pages: Vec<PagesPanelItem>,
    active_page: SharedString,
    elements: Vec<MockElement>,
    last_action: SharedString,
    next_page_id: usize,
    _subscriptions: Vec<Subscription>,
}

impl Storybook {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let pages = Self::seed_pages();
        let active_page: SharedString = "page-5".into();
        let panel = cx.new(|cx| PagesPanel::new("storybook-pages", pages.clone(), window, cx));
        panel.update(cx, |panel, cx| {
            panel.set_selected_page(Some(active_page.clone()), cx);
        });

        let subscriptions =
            vec![
                cx.subscribe(&panel, |story, panel, action: &PagesPanelAction, cx| {
                    story.handle_action(panel, action, cx);
                }),
            ];

        Self {
            panel,
            pages,
            active_page,
            elements: Self::seed_elements(),
            last_action: "Ready — try every control in the panel".into(),
            next_page_id: 6,
            _subscriptions: subscriptions,
        }
    }

    fn seed_pages() -> Vec<PagesPanelItem> {
        vec![
            PagesPanelItem::new("page-1", "Page 1"),
            PagesPanelItem::new("page-3", "Page 3"),
            PagesPanelItem::new("page-4", "Page 4"),
            PagesPanelItem::new("page-2", "Page 2"),
            PagesPanelItem::new("page-5", "Page 5"),
        ]
    }

    fn seed_elements() -> Vec<MockElement> {
        use PagesPanelElementKind::{Component, FrameGroup, Image, Instance, Shape, Text};

        [
            ("image-1", "Homepage hero", "Hero frame", Image, "page-5"),
            ("text-1", "Hello designers", "Hero frame", Text, "page-5"),
            ("frame-1", "Header", "Homepage", FrameGroup, "page-5"),
            (
                "component-1",
                "Primary button",
                "Components",
                Component,
                "page-5",
            ),
            (
                "instance-1",
                "Primary button instance",
                "Header",
                Instance,
                "page-5",
            ),
            ("text-2", "Hello again", "Footer", Text, "page-5"),
            ("shape-1", "Background glow", "Hero frame", Shape, "page-5"),
            ("image-2", "Team portrait", "About section", Image, "page-5"),
            ("text-3", "Page title", "Frame 2", Text, "page-1"),
            (
                "image-3",
                "Screenshot 2026-07-25 at 23.54.15",
                "Frame 2",
                Image,
                "page-1",
            ),
            (
                "image-4",
                "Screenshot 2026-07-25 at 23.54.31",
                "Frame 2",
                Image,
                "page-3",
            ),
            ("text-4", "Pricing heading", "Pricing", Text, "page-4"),
        ]
        .into_iter()
        .map(|(id, title, parent, kind, page_id)| MockElement {
            result: PagesPanelSearchResult::new(id, title, kind).parent(parent),
            page_id: page_id.into(),
        })
        .collect()
    }

    fn handle_action(
        &mut self,
        panel: Entity<PagesPanel>,
        action: &PagesPanelAction,
        cx: &mut Context<Self>,
    ) {
        match action {
            PagesPanelAction::ExpansionChanged { expanded } => {
                self.last_action = if *expanded {
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
                self.last_action = format!("Selected {page_id}").into();
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
                self.last_action = format!("Created {title} through the host adapter").into();
            }
            PagesPanelAction::RenameRequested { page_id, title } => {
                if let Some(page) = self.pages.iter_mut().find(|page| page.id == *page_id) {
                    page.title = title.clone();
                }
                panel.update(cx, |panel, cx| {
                    panel.set_pages(self.pages.clone(), cx);
                });
                self.last_action = format!("Renamed {page_id} to {title}").into();
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
                    self.last_action =
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
                self.last_action = format!("Deleted {page_id} through the host adapter").into();
            }
            PagesPanelAction::CopyLinkRequested { page_id } => {
                let link = format!("fanta://pages/{page_id}");
                cx.write_to_clipboard(ClipboardItem::new_string(link.clone()));
                self.last_action = format!("Copied {link}").into();
            }
            PagesPanelAction::SearchRequested(request) => {
                let results = self.search(request);
                panel.update(cx, |panel, cx| {
                    panel.set_search_results(results, cx);
                });
                self.last_action = format!("Host searched for “{}”", request.query).into();
            }
            PagesPanelAction::SearchClosed => {
                self.last_action = "Closed search and returned to Pages".into();
            }
            PagesPanelAction::SearchResultSelected { result_id } => {
                self.last_action = format!("Selected result {result_id}").into();
            }
            PagesPanelAction::NavigateResults {
                direction,
                result_id,
            } => {
                self.last_action = format!("Navigated {direction:?} to {result_id:?}").into();
            }
            PagesPanelAction::ReplaceRequested {
                request,
                result_id,
                replacement,
            } => {
                self.last_action = format!(
                    "Replace “{}” with “{}” in {result_id:?}",
                    request.query, replacement
                )
                .into();
            }
            PagesPanelAction::ReplaceAllRequested {
                request,
                replacement,
            } => {
                self.last_action = format!(
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
        let query_matches_element = |element: &&MockElement| query_matches(&element.result.title);

        let query_results = self
            .elements
            .iter()
            .filter(in_scope)
            .filter(query_matches_element)
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
}

impl Render for Storybook {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .size_full()
            .items_start()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                v_flex()
                    .w(px(340.))
                    .h_full()
                    .p_4()
                    .gap_3()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("PAGES PANEL"),
                    )
                    .child(div().flex_1().w_full().child(self.panel.clone())),
            )
            .child(
                v_flex()
                    .flex_1()
                    .h_full()
                    .border_l_1()
                    .border_color(cx.theme().border)
                    .p_8()
                    .gap_4()
                    .child(div().text_2xl().font_semibold().child("Pages panel"))
                    .child(
                        div()
                            .max_w(px(560.))
                            .text_color(cx.theme().muted_foreground)
                            .child(
                                "Interactive headless component. Collapse it, add and rename \
                                 pages, secondary-click a page for its context menu, or open Find \
                                 to exercise element filters, Replace, result scope, and \
                                 navigation.",
                            ),
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
                            .child(div().text_sm().child(self.last_action.clone())),
                    )
                    .child(
                        div()
                            .mt_4()
                            .max_w(px(560.))
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(
                                "The story acts like a Fanta host: it owns pages and result data, \
                                 listens to PagesPanelAction, applies mock mutations, and feeds the \
                                 updated read models back into the component.",
                            ),
                    ),
            )
    }
}

fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        gpui_component::init(cx);
        Theme::change(ThemeMode::Dark, None, cx);
        cx.activate(true);

        let bounds = Bounds::centered(None, size(px(980.), px(680.)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_min_size: Some(size(px(760.), px(520.))),
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
