#![forbid(unsafe_code)]

use fanta_gpui::prelude::{PagesPanel, PagesPanelAction, PagesPanelItem};
use gpui::{
    App, AppContext as _, Application, Bounds, Context, IntoElement, ParentElement as _, Render,
    SharedString, Styled as _, TitlebarOptions, Window, WindowBounds, WindowOptions, div, px, size,
};
use gpui_component::{ActiveTheme as _, Root, StyledExt as _, Theme, ThemeMode, h_flex, v_flex};
use gpui_component_assets::Assets;

struct Storybook {
    pages_expanded: bool,
    active_page: SharedString,
    last_action: SharedString,
}

impl Storybook {
    fn new() -> Self {
        Self {
            pages_expanded: true,
            active_page: "page-5".into(),
            last_action: "Ready".into(),
        }
    }

    fn pages() -> Vec<PagesPanelItem> {
        vec![
            PagesPanelItem::new("page-1", "Page 1"),
            PagesPanelItem::new("page-3", "Page 3"),
            PagesPanelItem::new("page-4", "Page 4"),
            PagesPanelItem::new("page-2", "Page 2"),
            PagesPanelItem::new("page-5", "Page 5"),
        ]
    }
}

impl Render for Storybook {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let panel = PagesPanel::new("storybook-pages", Self::pages())
            .expanded(self.pages_expanded)
            .selected_page(self.active_page.clone())
            .on_action(
                cx.listener(|story, action: &PagesPanelAction, _window, cx| {
                    match action {
                        PagesPanelAction::Toggle { expanded } => {
                            story.pages_expanded = *expanded;
                            story.last_action = if *expanded {
                                "Expanded Pages".into()
                            } else {
                                "Collapsed Pages".into()
                            };
                        }
                        PagesPanelAction::Select { page_id } => {
                            story.active_page = page_id.clone();
                            story.last_action = format!("Selected {page_id}").into();
                        }
                        PagesPanelAction::Search => {
                            story.last_action = "Search requested".into();
                        }
                        PagesPanelAction::Add => {
                            story.last_action = "Add requested".into();
                        }
                    }
                    cx.notify();
                }),
            );

        h_flex()
            .size_full()
            .items_start()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                v_flex()
                    .w(px(283.))
                    .h_full()
                    .p_4()
                    .gap_3()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("COMPONENT"),
                    )
                    .child(panel),
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
                            .max_w(px(520.))
                            .text_color(cx.theme().muted_foreground)
                            .child(
                                "Controlled navigation chrome. Collapse it, select a page, or \
                                 trigger the action buttons; the story owns all resulting state.",
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Last intent:"),
                            )
                            .child(div().text_sm().child(self.last_action.clone())),
                    ),
            )
    }
}

fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        gpui_component::init(cx);
        Theme::change(ThemeMode::Dark, None, cx);
        cx.activate(true);

        let bounds = Bounds::centered(None, size(px(860.), px(560.)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_min_size: Some(size(px(640.), px(420.))),
            titlebar: Some(TitlebarOptions {
                title: Some("Fanta GPUI Storybook".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            window.activate_window();
            let storybook = cx.new(|_| Storybook::new());
            cx.new(|cx| Root::new(storybook, window, cx))
        })
        .expect("failed to open the Storybook window");
    });
}
