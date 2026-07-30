use super::*;

impl Storybook {
    pub(super) fn open_story_window(&mut self, cx: &mut Context<Self>) {
        if self.launch_mode != StorybookLaunchMode::Gallery {
            return;
        }

        let story = self.active_story;
        let storybook = cx.entity();
        let (story_width, story_height) = story.gallery_surface_size();
        cx.defer(move |cx| {
            let restore_size = size(
                px(story_width.clamp(960., 1600.)),
                px(story_height.clamp(720., 1000.)),
            );
            let bounds = Bounds::centered(None, restore_size, cx);
            #[cfg(not(test))]
            let window_bounds = WindowBounds::Maximized(bounds);
            #[cfg(test)]
            let window_bounds = WindowBounds::Windowed(bounds);
            let options = WindowOptions {
                window_bounds: Some(window_bounds),
                window_min_size: Some(size(px(320.), px(240.))),
                titlebar: Some(TitlebarOptions {
                    title: Some(format!("{} — Fanta GPUI", story.title()).into()),
                    ..Default::default()
                }),
                ..Default::default()
            };

            let result = cx.open_window(options, move |window, cx| {
                let window_id = window.window_handle().window_id();
                storybook.update(cx, |storybook, cx| {
                    storybook.story_windows.insert(window_id, story);
                    cx.notify();
                });
                window.activate_window();
                cx.new(|cx| Root::new(storybook.clone(), window, cx))
            });
            if let Err(error) = result {
                eprintln!("failed to open the {} story window: {error}", story.title());
            }
        });
    }

    pub(super) fn toggle_gallery_theme(&mut self, cx: &mut Context<Self>) {
        if self.launch_mode != StorybookLaunchMode::Gallery {
            return;
        }
        self.gallery_theme_mode = if self.gallery_theme_mode.is_dark() {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        };
        Theme::change(self.gallery_theme_mode, None, cx);
        cx.refresh_windows();
        cx.notify();
    }

    fn render_gallery_sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        const GETTING_STARTED: &[StoryKind] = &[StoryKind::PseudoEditor];
        const FOUNDATIONS: &[StoryKind] = &[StoryKind::Icons];
        const COMPONENTS: &[StoryKind] = &[
            StoryKind::Assets,
            StoryKind::Design,
            StoryKind::Layers,
            StoryKind::Pages,
            StoryKind::Prototype,
            StoryKind::Timeline,
            StoryKind::Toolbar,
            StoryKind::Variables,
        ];
        let query = self
            .gallery_search_input
            .read(cx)
            .value()
            .trim()
            .to_ascii_lowercase();
        let mut groups = Vec::new();
        for (label, stories) in [
            ("Getting Started", GETTING_STARTED),
            ("Foundations", FOUNDATIONS),
            ("Components", COMPONENTS),
        ] {
            let items = stories
                .iter()
                .copied()
                .filter(|story| story.matches_query(&query))
                .map(|story| {
                    SidebarMenuItem::new(story.title())
                        .active(self.active_story == story)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.activate_gallery_story(story, window, cx);
                        }))
                })
                .collect::<Vec<_>>();
            if !items.is_empty() {
                groups.push(SidebarGroup::new(label).child(SidebarMenu::new().children(items)));
            }
        }
        if groups.is_empty() {
            groups.push(SidebarGroup::new("Components").child(
                SidebarMenu::new().child(SidebarMenuItem::new("No matching stories").disable(true)),
            ));
        }

        Sidebar::left()
            .w(gpui::relative(1.))
            .h_full()
            .border_0()
            .collapsible(false)
            .header(
                v_flex()
                    .w_full()
                    .gap_3()
                    .child(
                        SidebarHeader::new().w_full().child(
                            h_flex()
                                .w_full()
                                .gap_2()
                                .child(
                                    div()
                                        .size_8()
                                        .flex_none()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .rounded_lg()
                                        .bg(cx.theme().primary)
                                        .text_color(cx.theme().primary_foreground)
                                        .child(Icon::new(IconName::GalleryVerticalEnd).size_4()),
                                )
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .min_w(px(0.))
                                        .gap_0()
                                        .line_height(gpui::relative(1.25))
                                        .child(div().text_sm().font_semibold().child("Fanta GPUI"))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("Component gallery"),
                                        ),
                                ),
                        ),
                    )
                    .child(
                        div()
                            .w_full()
                            .rounded(cx.theme().radius)
                            .bg(cx.theme().sidebar_accent)
                            .child(
                                Input::new(&self.gallery_search_input)
                                    .small()
                                    .prefix(Icon::new(IconName::Search).size_4())
                                    .cleanable(true),
                            ),
                    ),
            )
            .children(groups)
            .into_any_element()
    }

    fn render_gallery_story_surface(&self, cx: &mut Context<Self>) -> AnyElement {
        let (width, height) = self.active_story.gallery_surface_size();
        div()
            .id("storybook-gallery-story-surface")
            .debug_selector(|| "storybook-gallery-story-surface".to_owned())
            .relative()
            .flex_none()
            .h(px(height))
            .overflow_hidden()
            .bg(cx.theme().background)
            .when(self.active_story.gallery_uses_fluid_width(), |surface| {
                surface.w_full().min_w(px(width))
            })
            .when(!self.active_story.gallery_uses_fluid_width(), |surface| {
                surface.w(px(width))
            })
            .child(self.render_gallery_story_component(cx))
            .into_any_element()
    }

    pub(super) fn render_gallery_shell(&self, cx: &mut Context<Self>) -> AnyElement {
        let active_story = self.active_story;
        let last_action = self.last_action_for_story(active_story);

        v_flex()
            .id("storybook-gallery-shell")
            .debug_selector(|| "storybook-gallery-shell".to_owned())
            .key_context(STORYBOOK_KEY_CONTEXT)
            .on_action(cx.listener(|this, _: &OpenStoryWindow, _, cx| {
                this.open_story_window(cx);
            }))
            .on_action(cx.listener(|this, _: &ToggleGalleryTheme, _, cx| {
                this.toggle_gallery_theme(cx);
            }))
            .size_full()
            .min_h(px(0.))
            .overflow_hidden()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                h_flex()
                    .h(px(34.))
                    .w_full()
                    .flex_none()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().title_bar)
                    .child(
                        div()
                            .h_full()
                            .flex_1()
                            .min_w(px(0.))
                            .px_1()
                            .child(self.gallery_menu_bar.clone()),
                    )
                    .child(
                        h_flex()
                            .h_full()
                            .px_2()
                            .gap_2()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(
                                Button::new("storybook-toggle-gallery-theme")
                                    .ghost()
                                    .small()
                                    .icon(
                                        Icon::new(if self.gallery_theme_mode.is_dark() {
                                            IconName::Moon
                                        } else {
                                            IconName::Sun
                                        })
                                        .size_4(),
                                    )
                                    .label(if self.gallery_theme_mode.is_dark() {
                                        "Dark"
                                    } else {
                                        "Light"
                                    })
                                    .tooltip("Toggle the active Gallery theme")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.toggle_gallery_theme(cx);
                                    })),
                            )
                            .child(Icon::new(IconName::CircleCheck).size_4())
                            .child("Interactive"),
                    ),
            )
            .child(
                div().flex_1().min_h(px(0.)).child(
                    h_resizable("storybook-gallery-layout")
                        .child(
                            resizable_panel()
                                .size(px(255.))
                                .size_range(px(200.)..px(320.))
                                .child(self.render_gallery_sidebar(cx)),
                        )
                        .child(
                            resizable_panel().child(
                                v_flex()
                                    .size_full()
                                    .min_w(px(0.))
                                    .min_h(px(0.))
                                    .child(
                                        h_flex()
                                            .w_full()
                                            .min_h(px(86.))
                                            .flex_none()
                                            .items_start()
                                            .justify_between()
                                            .gap_6()
                                            .px_4()
                                            .py_4()
                                            .border_b_1()
                                            .border_color(cx.theme().border)
                                            .child(
                                                v_flex()
                                                    .min_w(px(0.))
                                                    .gap_1()
                                                    .child(
                                                        div()
                                                            .text_xl()
                                                            .font_semibold()
                                                            .child(active_story.title()),
                                                    )
                                                    .child(
                                                        div()
                                                            .max_w(px(780.))
                                                            .text_sm()
                                                            .text_color(
                                                                cx.theme().muted_foreground,
                                                            )
                                                            .child(active_story.description()),
                                                    ),
                                            )
                                            .child(
                                                Button::new("storybook-open-story-window")
                                                    .outline()
                                                    .small()
                                                    .icon(
                                                        Icon::new(IconName::ExternalLink).size_4(),
                                                    )
                                                    .label("Open window")
                                                    .tooltip(
                                                        "Open this story in a maximized window. Its data stays in the Gallery.",
                                                    )
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.open_story_window(cx);
                                                    })),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .id("storybook-gallery-canvas")
                                            .debug_selector(|| {
                                                "storybook-gallery-canvas".to_owned()
                                            })
                                            .flex_1()
                                            .min_h(px(0.))
                                            .overflow_scroll()
                                            .track_scroll(&self.gallery_story_scroll_handle)
                                            .bg(cx.theme().background)
                                            .p_4()
                                            .child(self.render_gallery_story_surface(cx)),
                                    ),
                            ),
                        ),
                ),
            )
            .child(
                h_flex()
                    .h(px(28.))
                    .w_full()
                    .flex_none()
                    .px_3()
                    .gap_2()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().title_bar)
                    .text_xs()
                    .child(div().size(px(7.)).rounded_full().bg(cx.theme().success))
                    .child(div().font_medium().child(active_story.title()))
                    .child(
                        div()
                            .max_w(px(720.))
                            .truncate()
                            .text_color(cx.theme().muted_foreground)
                            .child(last_action),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "{} stories · {} theme · typed host intents",
                                StoryKind::ALL.len(),
                                self.gallery_theme_mode.name()
                            )),
                    ),
            )
            .into_any_element()
    }

    pub(super) fn render_story_window(
        &self,
        story: StoryKind,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .id("storybook-story-window")
            .debug_selector(|| "storybook-story-window".to_owned())
            .size_full()
            .min_w(px(0.))
            .min_h(px(0.))
            .overflow_hidden()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_story_component(story, cx))
            .into_any_element()
    }
}
