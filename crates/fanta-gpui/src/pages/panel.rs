use std::rc::Rc;

use gpui::{
    App, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme as _, IconName, Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};

/// Read-only page data displayed by [`PagesPanel`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PagesPanelItem {
    /// Opaque host identifier. The component returns it unchanged in actions.
    pub id: SharedString,
    /// User-facing page name.
    pub title: SharedString,
}

impl PagesPanelItem {
    /// Creates a page row.
    pub fn new(id: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
        }
    }
}

/// User intent emitted by [`PagesPanel`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PagesPanelAction {
    /// Requests a new controlled expanded state.
    Toggle { expanded: bool },
    /// Requests that the host select the identified page.
    Select { page_id: SharedString },
    /// Requests that the host show page search.
    Search,
    /// Requests that the host create a page through its domain operation path.
    Add,
}

type ActionHandler = Rc<dyn Fn(&PagesPanelAction, &mut Window, &mut App)>;

/// A controlled, collapsible page navigation panel.
///
/// The panel never owns selection or domain mutations. Re-render it with the
/// new `expanded`, `selected_page`, and `pages` values after handling an
/// action.
#[derive(IntoElement)]
pub struct PagesPanel {
    id: SharedString,
    pages: Vec<PagesPanelItem>,
    expanded: bool,
    selected_page: Option<SharedString>,
    on_action: Option<ActionHandler>,
}

impl PagesPanel {
    /// Creates a panel with no selected page.
    pub fn new(id: impl Into<SharedString>, pages: Vec<PagesPanelItem>) -> Self {
        Self {
            id: id.into(),
            pages,
            expanded: true,
            selected_page: None,
            on_action: None,
        }
    }

    /// Sets the controlled expanded state.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Sets the controlled selected page.
    pub fn selected_page(mut self, page_id: impl Into<SharedString>) -> Self {
        self.selected_page = Some(page_id.into());
        self
    }

    /// Registers one typed intent handler for all panel interactions.
    pub fn on_action(
        mut self,
        handler: impl Fn(&PagesPanelAction, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_action = Some(Rc::new(handler));
        self
    }

    fn collapsed_title(&self) -> SharedString {
        self.selected_page
            .as_ref()
            .and_then(|selected| self.pages.iter().find(|page| &page.id == selected))
            .map(|page| page.title.clone())
            .unwrap_or_else(|| "Pages".into())
    }
}

impl RenderOnce for PagesPanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let root_id = self.id.clone();
        let expanded = self.expanded;
        let selected_page = self.selected_page.clone();
        let collapsed_title = self.collapsed_title();
        let on_action = self.on_action.clone();

        let toggle_id = SharedString::from(format!("{root_id}-toggle"));
        let toggle_handler = on_action.clone();
        let toggle = h_flex()
            .id(toggle_id)
            .h_full()
            .flex_1()
            .gap_1()
            .px_2()
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().sidebar_accent.opacity(0.55)))
            .on_click(move |_, window, cx| {
                if let Some(handler) = &toggle_handler {
                    handler(
                        &PagesPanelAction::Toggle {
                            expanded: !expanded,
                        },
                        window,
                        cx,
                    );
                }
            })
            .child(
                gpui_component::Icon::new(if expanded {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronRight
                })
                .xsmall(),
            )
            .child(if expanded {
                SharedString::from("Pages")
            } else {
                collapsed_title
            });

        let search_handler = on_action.clone();
        let add_handler = on_action.clone();
        let header = h_flex()
            .h(px(40.))
            .w_full()
            .flex_shrink_0()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(toggle)
            .when(expanded, |header| {
                header
                    .child(
                        Button::new(SharedString::from(format!("{root_id}-search")))
                            .ghost()
                            .xsmall()
                            .compact()
                            .icon(IconName::Search)
                            .tooltip("Search pages")
                            .on_click(move |_, window, cx| {
                                if let Some(handler) = &search_handler {
                                    handler(&PagesPanelAction::Search, window, cx);
                                }
                            }),
                    )
                    .child(
                        Button::new(SharedString::from(format!("{root_id}-add")))
                            .ghost()
                            .xsmall()
                            .compact()
                            .icon(IconName::Plus)
                            .tooltip("Add page")
                            .on_click(move |_, window, cx| {
                                if let Some(handler) = &add_handler {
                                    handler(&PagesPanelAction::Add, window, cx);
                                }
                            }),
                    )
                    .child(div().w(px(4.)))
            });

        let rows =
            v_flex()
                .w_full()
                .p_2()
                .gap_1()
                .children(self.pages.into_iter().enumerate().map(|(index, page)| {
                    let is_selected = selected_page.as_ref() == Some(&page.id);
                    let page_id = page.id.clone();
                    let select_handler = on_action.clone();

                    h_flex()
                        .id(SharedString::from(format!("{root_id}-page-{index}")))
                        .h(px(32.))
                        .w_full()
                        .px_2()
                        .rounded(px(4.))
                        .text_sm()
                        .cursor_pointer()
                        .hover(|style| style.bg(cx.theme().sidebar_accent.opacity(0.55)))
                        .when(is_selected, |row| {
                            row.bg(cx.theme().sidebar_accent)
                                .text_color(cx.theme().sidebar_accent_foreground)
                        })
                        .on_click(move |_, window, cx| {
                            if let Some(handler) = &select_handler {
                                handler(
                                    &PagesPanelAction::Select {
                                        page_id: page_id.clone(),
                                    },
                                    window,
                                    cx,
                                );
                            }
                        })
                        .child(page.title)
                }));

        v_flex()
            .id(root_id)
            .w_full()
            .overflow_hidden()
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().sidebar_foreground)
            .border_1()
            .border_color(cx.theme().border)
            .child(header)
            .when(expanded, |panel| panel.child(rows))
    }
}

#[cfg(test)]
mod tests {
    use super::{PagesPanel, PagesPanelItem};

    #[test]
    fn collapsed_title_uses_the_selected_page() {
        let panel = PagesPanel::new(
            "pages",
            vec![
                PagesPanelItem::new("page-1", "Page 1"),
                PagesPanelItem::new("page-2", "Page 2"),
            ],
        )
        .selected_page("page-2");

        assert_eq!(panel.collapsed_title().as_ref(), "Page 2");
    }

    #[test]
    fn collapsed_title_falls_back_to_pages() {
        let panel = PagesPanel::new("pages", vec![PagesPanelItem::new("page-1", "Page 1")])
            .selected_page("missing");

        assert_eq!(panel.collapsed_title().as_ref(), "Pages");
    }
}
