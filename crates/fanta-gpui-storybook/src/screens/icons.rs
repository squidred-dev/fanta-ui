use fanta_gpui::atoms::TypographyExt as _;
use gpui::{
    AppContext as _, Context, Entity, FocusHandle, Focusable, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _,
    Subscription, Window, div, prelude::FluentBuilder as _,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, IconNamed as _, StyledExt as _, h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};

pub(crate) const ICON_COUNT: usize = 86;

#[cfg(test)]
pub(crate) fn icon_count() -> usize {
    icon_specs().len()
}

#[cfg(test)]
pub(crate) fn icon_names() -> Vec<&'static str> {
    icon_specs().into_iter().map(|spec| spec.name).collect()
}

struct IconSpec {
    name: &'static str,
    icon: IconName,
}

pub(crate) struct IconGallery {
    search_input: Entity<InputState>,
    _search_subscription: Subscription,
}

impl IconGallery {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_input = cx
            .new(|cx| InputState::new(window, cx).placeholder("Filter by icon name or asset path"));
        let _search_subscription = cx.subscribe(&search_input, |_, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        });

        Self {
            search_input,
            _search_subscription,
        }
    }

    fn query(&self, cx: &Context<Self>) -> String {
        self.search_input
            .read(cx)
            .value()
            .trim()
            .to_ascii_lowercase()
    }
}

impl Focusable for IconGallery {
    fn focus_handle(&self, cx: &gpui::App) -> FocusHandle {
        self.search_input.focus_handle(cx)
    }
}

impl Render for IconGallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let query = self.query(cx);
        let icons = icon_specs()
            .into_iter()
            .filter(|spec| {
                query.is_empty()
                    || spec.name.to_ascii_lowercase().contains(&query)
                    || spec
                        .icon
                        .clone()
                        .path()
                        .as_ref()
                        .to_ascii_lowercase()
                        .contains(&query)
            })
            .collect::<Vec<_>>();
        let visible_count = icons.len();
        let mut grid = h_flex()
            .id("storybook-icon-grid")
            .debug_selector(|| "storybook-icon-grid".to_owned())
            .w_full()
            .items_start()
            .content_start()
            .gap_3()
            .flex_wrap();

        for spec in icons {
            let path = spec.icon.clone().path();
            grid = grid.child(
                v_flex()
                    .id(SharedString::from(format!(
                        "storybook-icon-{}",
                        spec.name.to_ascii_lowercase()
                    )))
                    .w(gpui::px(168.))
                    .h(gpui::px(124.))
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .px_3()
                    .rounded(cx.theme().radius_lg)
                    .border_1()
                    .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                    .bg(fanta_gpui::atoms::SemanticColor::BackgroundSecondary.resolve(cx))
                    .hover(|style| {
                        style
                            .border_color(
                                fanta_gpui::atoms::SemanticColor::BackgroundBrand
                                    .resolve(cx)
                                    .opacity(0.5),
                            )
                            .bg(fanta_gpui::atoms::SemanticColor::BackgroundHover.resolve(cx))
                    })
                    .child(
                        div()
                            .size_10()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_lg()
                            .bg(fanta_gpui::atoms::SemanticColor::BackgroundSecondary.resolve(cx))
                            .text_color(fanta_gpui::atoms::SemanticColor::Text.resolve(cx))
                            .child(Icon::new(spec.icon).size_6()),
                    )
                    .child(
                        v_flex()
                            .w_full()
                            .min_w(gpui::px(0.))
                            .items_center()
                            .gap_0p5()
                            .child(
                                div()
                                    .typography(fanta_gpui::atoms::TypographyToken::BodyLarge)
                                    .font_medium()
                                    .child(spec.name),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .truncate()
                                    .text_center()
                                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                                    .text_color(
                                        fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx),
                                    )
                                    .child(path),
                            ),
                    ),
            );
        }

        v_flex()
            .id("storybook-icon-gallery")
            .debug_selector(|| "storybook-icon-gallery".to_owned())
            .size_full()
            .min_h(gpui::px(0.))
            .overflow_hidden()
            .bg(fanta_gpui::atoms::SemanticColor::Background.resolve(cx))
            .text_color(fanta_gpui::atoms::SemanticColor::Text.resolve(cx))
            .child(
                h_flex()
                    .w_full()
                    .flex_none()
                    .items_end()
                    .justify_between()
                    .gap_6()
                    .px_6()
                    .py_5()
                    .border_b_1()
                    .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                    .child(
                        v_flex()
                            .min_w(gpui::px(0.))
                            .gap_1()
                            .child(
                                div()
                                    .typography(fanta_gpui::atoms::TypographyToken::HeadingLarge)
                                    .font_semibold()
                                    .child("Icon catalog"),
                            )
                            .child(
                                div()
                                    .typography(fanta_gpui::atoms::TypographyToken::BodyLarge)
                                    .text_color(
                                        fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx),
                                    )
                                    .child(format!(
                                        "{visible_count} of {ICON_COUNT} bundled icons"
                                    )),
                            ),
                    )
                    .child(
                        div().w(gpui::px(360.)).max_w(gpui::relative(0.45)).child(
                            Input::new(&self.search_input)
                                .prefix(Icon::new(IconName::Search).size_4())
                                .cleanable(true),
                        ),
                    ),
            )
            .child(
                div()
                    .id("storybook-icon-scroll")
                    .flex_1()
                    .min_h(gpui::px(0.))
                    .overflow_scroll()
                    .p_6()
                    .when(visible_count == 0, |body| {
                        body.child(
                            v_flex()
                                .w_full()
                                .h(gpui::px(240.))
                                .items_center()
                                .justify_center()
                                .gap_2()
                                .text_color(
                                    fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx),
                                )
                                .child(Icon::new(IconName::Search).size_6())
                                .child("No icons match this filter"),
                        )
                    })
                    .when(visible_count > 0, |body| body.child(grid)),
            )
    }
}

fn icon_specs() -> Vec<IconSpec> {
    vec![
        IconSpec {
            name: "ALargeSmall",
            icon: IconName::ALargeSmall,
        },
        IconSpec {
            name: "ArrowDown",
            icon: IconName::ArrowDown,
        },
        IconSpec {
            name: "ArrowLeft",
            icon: IconName::ArrowLeft,
        },
        IconSpec {
            name: "ArrowRight",
            icon: IconName::ArrowRight,
        },
        IconSpec {
            name: "ArrowUp",
            icon: IconName::ArrowUp,
        },
        IconSpec {
            name: "Asterisk",
            icon: IconName::Asterisk,
        },
        IconSpec {
            name: "Bell",
            icon: IconName::Bell,
        },
        IconSpec {
            name: "BookOpen",
            icon: IconName::BookOpen,
        },
        IconSpec {
            name: "Bot",
            icon: IconName::Bot,
        },
        IconSpec {
            name: "Building2",
            icon: IconName::Building2,
        },
        IconSpec {
            name: "Calendar",
            icon: IconName::Calendar,
        },
        IconSpec {
            name: "CaseSensitive",
            icon: IconName::CaseSensitive,
        },
        IconSpec {
            name: "ChartPie",
            icon: IconName::ChartPie,
        },
        IconSpec {
            name: "Check",
            icon: IconName::Check,
        },
        IconSpec {
            name: "ChevronDown",
            icon: IconName::ChevronDown,
        },
        IconSpec {
            name: "ChevronLeft",
            icon: IconName::ChevronLeft,
        },
        IconSpec {
            name: "ChevronRight",
            icon: IconName::ChevronRight,
        },
        IconSpec {
            name: "ChevronsUpDown",
            icon: IconName::ChevronsUpDown,
        },
        IconSpec {
            name: "ChevronUp",
            icon: IconName::ChevronUp,
        },
        IconSpec {
            name: "CircleCheck",
            icon: IconName::CircleCheck,
        },
        IconSpec {
            name: "CircleUser",
            icon: IconName::CircleUser,
        },
        IconSpec {
            name: "CircleX",
            icon: IconName::CircleX,
        },
        IconSpec {
            name: "Close",
            icon: IconName::Close,
        },
        IconSpec {
            name: "Copy",
            icon: IconName::Copy,
        },
        IconSpec {
            name: "Dash",
            icon: IconName::Dash,
        },
        IconSpec {
            name: "Delete",
            icon: IconName::Delete,
        },
        IconSpec {
            name: "Ellipsis",
            icon: IconName::Ellipsis,
        },
        IconSpec {
            name: "EllipsisVertical",
            icon: IconName::EllipsisVertical,
        },
        IconSpec {
            name: "ExternalLink",
            icon: IconName::ExternalLink,
        },
        IconSpec {
            name: "Eye",
            icon: IconName::Eye,
        },
        IconSpec {
            name: "EyeOff",
            icon: IconName::EyeOff,
        },
        IconSpec {
            name: "File",
            icon: IconName::File,
        },
        IconSpec {
            name: "Folder",
            icon: IconName::Folder,
        },
        IconSpec {
            name: "FolderClosed",
            icon: IconName::FolderClosed,
        },
        IconSpec {
            name: "FolderOpen",
            icon: IconName::FolderOpen,
        },
        IconSpec {
            name: "Frame",
            icon: IconName::Frame,
        },
        IconSpec {
            name: "GalleryVerticalEnd",
            icon: IconName::GalleryVerticalEnd,
        },
        IconSpec {
            name: "GitHub",
            icon: IconName::GitHub,
        },
        IconSpec {
            name: "Globe",
            icon: IconName::Globe,
        },
        IconSpec {
            name: "Heart",
            icon: IconName::Heart,
        },
        IconSpec {
            name: "HeartOff",
            icon: IconName::HeartOff,
        },
        IconSpec {
            name: "Inbox",
            icon: IconName::Inbox,
        },
        IconSpec {
            name: "Info",
            icon: IconName::Info,
        },
        IconSpec {
            name: "Inspector",
            icon: IconName::Inspector,
        },
        IconSpec {
            name: "LayoutDashboard",
            icon: IconName::LayoutDashboard,
        },
        IconSpec {
            name: "Loader",
            icon: IconName::Loader,
        },
        IconSpec {
            name: "LoaderCircle",
            icon: IconName::LoaderCircle,
        },
        IconSpec {
            name: "Map",
            icon: IconName::Map,
        },
        IconSpec {
            name: "Maximize",
            icon: IconName::Maximize,
        },
        IconSpec {
            name: "Menu",
            icon: IconName::Menu,
        },
        IconSpec {
            name: "Minimize",
            icon: IconName::Minimize,
        },
        IconSpec {
            name: "Minus",
            icon: IconName::Minus,
        },
        IconSpec {
            name: "Moon",
            icon: IconName::Moon,
        },
        IconSpec {
            name: "Palette",
            icon: IconName::Palette,
        },
        IconSpec {
            name: "PanelBottom",
            icon: IconName::PanelBottom,
        },
        IconSpec {
            name: "PanelBottomOpen",
            icon: IconName::PanelBottomOpen,
        },
        IconSpec {
            name: "PanelLeft",
            icon: IconName::PanelLeft,
        },
        IconSpec {
            name: "PanelLeftClose",
            icon: IconName::PanelLeftClose,
        },
        IconSpec {
            name: "PanelLeftOpen",
            icon: IconName::PanelLeftOpen,
        },
        IconSpec {
            name: "PanelRight",
            icon: IconName::PanelRight,
        },
        IconSpec {
            name: "PanelRightClose",
            icon: IconName::PanelRightClose,
        },
        IconSpec {
            name: "PanelRightOpen",
            icon: IconName::PanelRightOpen,
        },
        IconSpec {
            name: "Plus",
            icon: IconName::Plus,
        },
        IconSpec {
            name: "Redo",
            icon: IconName::Redo,
        },
        IconSpec {
            name: "Redo2",
            icon: IconName::Redo2,
        },
        IconSpec {
            name: "Replace",
            icon: IconName::Replace,
        },
        IconSpec {
            name: "ResizeCorner",
            icon: IconName::ResizeCorner,
        },
        IconSpec {
            name: "Search",
            icon: IconName::Search,
        },
        IconSpec {
            name: "Settings",
            icon: IconName::Settings,
        },
        IconSpec {
            name: "Settings2",
            icon: IconName::Settings2,
        },
        IconSpec {
            name: "SortAscending",
            icon: IconName::SortAscending,
        },
        IconSpec {
            name: "SortDescending",
            icon: IconName::SortDescending,
        },
        IconSpec {
            name: "SquareTerminal",
            icon: IconName::SquareTerminal,
        },
        IconSpec {
            name: "Star",
            icon: IconName::Star,
        },
        IconSpec {
            name: "StarOff",
            icon: IconName::StarOff,
        },
        IconSpec {
            name: "Sun",
            icon: IconName::Sun,
        },
        IconSpec {
            name: "ThumbsDown",
            icon: IconName::ThumbsDown,
        },
        IconSpec {
            name: "ThumbsUp",
            icon: IconName::ThumbsUp,
        },
        IconSpec {
            name: "TriangleAlert",
            icon: IconName::TriangleAlert,
        },
        IconSpec {
            name: "Undo",
            icon: IconName::Undo,
        },
        IconSpec {
            name: "Undo2",
            icon: IconName::Undo2,
        },
        IconSpec {
            name: "User",
            icon: IconName::User,
        },
        IconSpec {
            name: "WindowClose",
            icon: IconName::WindowClose,
        },
        IconSpec {
            name: "WindowMaximize",
            icon: IconName::WindowMaximize,
        },
        IconSpec {
            name: "WindowMinimize",
            icon: IconName::WindowMinimize,
        },
        IconSpec {
            name: "WindowRestore",
            icon: IconName::WindowRestore,
        },
    ]
}

/// The Icons foundation screen: the catalog entity plus registry hooks.
pub(crate) struct IconsScreen {
    pub(crate) gallery: Entity<IconGallery>,
}

impl IconsScreen {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<crate::Storybook>) -> Self {
        Self {
            gallery: cx.new(|cx| IconGallery::new(window, cx)),
        }
    }
}
