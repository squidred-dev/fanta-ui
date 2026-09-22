//! The Pages story: mock host data, fixtures, search, reducer, and knobs.

use crate::*;

use super::harness;
use super::knobs::{self, KnobOption};

#[derive(Clone)]
pub(crate) struct MockElement {
    pub(crate) result: PagesPanelSearchResult,
    pub(crate) page_id: SharedString,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PagesNamedState {
    Default,
    Empty,
    ManyPages,
    SearchActive,
}

impl PagesNamedState {
    pub(crate) const ALL: [Self; 4] = [
        Self::Default,
        Self::Empty,
        Self::ManyPages,
        Self::SearchActive,
    ];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Empty => "Empty",
            Self::ManyPages => "Many pages",
            Self::SearchActive => "Search active",
        }
    }
}

pub(crate) fn seed_pages() -> Vec<PagesPanelItem> {
    vec![
        PagesPanelItem::new("page-1", "Page 1"),
        PagesPanelItem::new("page-3", "Page 3"),
        PagesPanelItem::new("page-4", "Page 4"),
        PagesPanelItem::new("page-2", "Page 2"),
        PagesPanelItem::new("page-5", "Page 5"),
    ]
}

pub(crate) fn many_pages() -> Vec<PagesPanelItem> {
    (1..=40)
        .map(|ordinal| PagesPanelItem::new(format!("page-{ordinal}"), format!("Page {ordinal}")))
        .collect()
}

pub(crate) fn seed_elements() -> Vec<MockElement> {
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

pub(crate) struct PagesScreen {
    pub(crate) panel: Entity<PagesPanel>,
    pub(crate) pages: Vec<PagesPanelItem>,
    pub(crate) active_page: SharedString,
    pub(crate) elements: Vec<MockElement>,
    pub(crate) last_action: SharedString,
    pub(crate) next_page_id: usize,
    pub(crate) named_state: PagesNamedState,
}

impl PagesScreen {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Storybook>) -> Self {
        let pages = seed_pages();
        let active_page: SharedString = "page-5".into();
        let panel = cx.new(|cx| PagesPanel::new("storybook-pages", pages.clone(), window, cx));
        panel.update(cx, |panel, cx| {
            panel.set_selected_page(Some(active_page.clone()), cx);
        });
        Self {
            panel,
            pages,
            active_page,
            elements: seed_elements(),
            last_action: "Ready — try every Pages control".into(),
            next_page_id: 6,
            named_state: PagesNamedState::Default,
        }
    }

    fn fixture(state: PagesNamedState) -> (Vec<PagesPanelItem>, Option<SharedString>, usize) {
        match state {
            PagesNamedState::Default | PagesNamedState::SearchActive => {
                (seed_pages(), Some("page-5".into()), 6)
            }
            PagesNamedState::Empty => (Vec::new(), None, 1),
            PagesNamedState::ManyPages => (many_pages(), Some("page-1".into()), 41),
        }
    }

    pub(crate) fn apply_named_state(
        &mut self,
        state: PagesNamedState,
        window: &mut Window,
        cx: &mut Context<Storybook>,
    ) {
        self.named_state = state;
        let (pages, active_page, next_page_id) = Self::fixture(state);
        self.pages = pages;
        self.active_page = active_page.clone().unwrap_or_default();
        self.elements = if state == PagesNamedState::Empty {
            Vec::new()
        } else {
            seed_elements()
        };
        self.next_page_id = next_page_id;
        let pages = self.pages.clone();
        self.panel.update(cx, |panel, cx| {
            panel.set_pages(pages, cx);
            panel.set_selected_page(active_page, cx);
        });
        if state == PagesNamedState::SearchActive {
            self.panel.focus_handle(cx).focus(window, cx);
            window.dispatch_action(Box::new(FindInPages), cx);
        }
        self.last_action = format!("Story applied the {} Pages state", state.label()).into();
        cx.notify();
    }

    pub(crate) fn search(&self, request: &PagesPanelSearchRequest) -> PagesPanelSearchResults {
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

    pub(crate) fn handle_action(
        &mut self,
        panel: Entity<PagesPanel>,
        action: &PagesPanelAction,
        cx: &mut Context<Storybook>,
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
            PagesPanelAction::MoveRequested { page_id, direction } => {
                if let Some(index) = self.pages.iter().position(|page| page.id == *page_id)
                    && let Some(target) = direction.destination(index, self.pages.len())
                {
                    let page = self.pages.remove(index);
                    self.pages.insert(target, page);
                    panel.update(cx, |panel, cx| panel.set_pages(self.pages.clone(), cx));
                    self.last_action = format!("Moved {page_id} {direction:?}").into();
                }
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
}

impl Storybook {
    pub(crate) fn render_pages_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_story_shell(
            "storybook-reference-pages",
            harness::ReferenceStoryCopy {
                eyebrow: "PAGES PANEL",
                title: "Pages panel",
                description: "Collapse it, add and rename pages, secondary-click a page, or \
                              open Find to exercise filters, Replace, result scope, and \
                              navigation.",
                adapter_description: "The story owns page and result data, listens to \
                                      PagesPanelAction, applies mock mutations, and feeds \
                                      fresh read models back into the component.",
            },
            self.pages_screen.last_action.clone(),
            self.pages_screen.panel.clone().into_any_element(),
            cx,
        )
    }

    pub(crate) fn render_pages_knobs(&self, cx: &mut Context<Self>) -> AnyElement {
        knobs::knobs_panel(
            "pages-story-knobs",
            vec![knobs::enum_knob_row(
                "pages-knob-state",
                "NAMED STATE",
                PagesNamedState::ALL.map(|state| KnobOption::new(state, state.label())),
                self.pages_screen.named_state,
                |this, state, window, cx| {
                    this.pages_screen.apply_named_state(state, window, cx);
                },
                cx,
            )],
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_named_states_return_reseedable_fixtures() {
        let (default_pages, default_active, next_id) =
            PagesScreen::fixture(PagesNamedState::Default);
        assert_eq!(default_pages.len(), 5);
        assert_eq!(default_active.as_deref(), Some("page-5"));
        assert_eq!(next_id, 6);

        let (empty, active, next_id) = PagesScreen::fixture(PagesNamedState::Empty);
        assert!(empty.is_empty());
        assert!(active.is_none());
        assert_eq!(next_id, 1);

        let (many, active, next_id) = PagesScreen::fixture(PagesNamedState::ManyPages);
        assert_eq!(many.len(), 40);
        assert_eq!(active.as_deref(), Some("page-1"));
        assert_eq!(
            next_id, 41,
            "created pages must not collide with the fixture ids"
        );
        let ids: std::collections::HashSet<_> = many.iter().map(|page| page.id.clone()).collect();
        assert_eq!(ids.len(), many.len(), "page ids must stay unique");
    }
}
