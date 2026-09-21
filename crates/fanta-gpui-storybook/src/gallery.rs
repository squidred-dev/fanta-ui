use super::*;
use fanta_gpui::atoms::TypographyExt as _;

/// Below this window width the gallery runs its narrow layout: the
/// sidebar auto-collapses and story knobs default to collapsed.
pub(crate) const GALLERY_NARROW_WINDOW_WIDTH: f32 = 880.;

/// The user's standing choice for the gallery sidebar, kept on the
/// Storybook entity so it survives story switches.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SidebarPin {
    /// Follow the window width: expanded when wide, collapsed when narrow.
    Auto,
    /// Explicitly collapsed; stays collapsed at any window width.
    Collapsed,
    /// Explicitly reopened while the window was narrow.
    Expanded,
}

pub(crate) fn window_is_narrow(window_width: f32) -> bool {
    window_width < GALLERY_NARROW_WINDOW_WIDTH
}

/// Whether the sidebar is collapsed at the given window width.
pub(crate) fn sidebar_collapsed(window_width: f32, pin: SidebarPin) -> bool {
    match pin {
        SidebarPin::Collapsed => true,
        SidebarPin::Expanded => false,
        SidebarPin::Auto => window_is_narrow(window_width),
    }
}

/// The pin state after the user presses the sidebar toggle. Collapsing is
/// always an explicit pin; reopening at a wide width returns to Auto so
/// the sidebar keeps auto-collapsing when the window later shrinks.
pub(crate) fn toggled_sidebar_pin(pin: SidebarPin, window_width: f32) -> SidebarPin {
    if sidebar_collapsed(window_width, pin) {
        if window_is_narrow(window_width) {
            SidebarPin::Expanded
        } else {
            SidebarPin::Auto
        }
    } else {
        SidebarPin::Collapsed
    }
}

/// Whether a story's knobs section renders its body: the user's explicit
/// choice wins over the narrow-window default.
pub(crate) fn knobs_expanded(narrow: bool, user_choice: Option<bool>) -> bool {
    user_choice.unwrap_or(!narrow)
}

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
        self.gallery_theme_index = (self.gallery_theme_index + 1) % self.gallery_themes.len();
        apply_zed_theme(&self.gallery_themes[self.gallery_theme_index], cx);
        self.gallery_theme_mode = Theme::global(cx).mode;
        cx.refresh_windows();
        cx.notify();
    }

    fn render_gallery_sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        let query = self
            .gallery_search_input
            .read(cx)
            .value()
            .trim()
            .to_ascii_lowercase();
        let mut groups = Vec::new();
        for section in screens::StorySection::ALL {
            let items = screens::registry()
                .iter()
                .filter(|descriptor| descriptor.section == section)
                .filter(|descriptor| descriptor.matches_query(&query))
                .map(|descriptor| {
                    let story = descriptor.kind;
                    SidebarMenuItem::new(descriptor.title)
                        .active(self.active_story == story)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.activate_gallery_story(story, window, cx);
                        }))
                })
                .collect::<Vec<_>>();
            if !items.is_empty() {
                groups.push(
                    SidebarGroup::new(section.label()).child(SidebarMenu::new().children(items)),
                );
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
                                        .bg(fanta_gpui::atoms::SemanticColor::BackgroundBrand
                                            .resolve(cx))
                                        .text_color(
                                            fanta_gpui::atoms::SemanticColor::TextOnBrand
                                                .resolve(cx),
                                        )
                                        .child(Icon::new(IconName::GalleryVerticalEnd).size_4()),
                                )
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .min_w(px(0.))
                                        .gap_0()
                                        .line_height(gpui::relative(1.25))
                                        .child(
                                            div()
                                                .typography(
                                                    fanta_gpui::atoms::TypographyToken::BodyLarge,
                                                )
                                                .font_semibold()
                                                .child("Fanta GPUI"),
                                        )
                                        .child(
                                            div()
                                                .typography(
                                                    fanta_gpui::atoms::TypographyToken::BodyMedium,
                                                )
                                                .text_color(
                                                    fanta_gpui::atoms::SemanticColor::TextTertiary
                                                        .resolve(cx),
                                                )
                                                .child("Component gallery"),
                                        ),
                                ),
                        ),
                    )
                    .child(
                        div()
                            .w_full()
                            .rounded(cx.theme().radius)
                            .bg(fanta_gpui::atoms::SemanticColor::BackgroundToolbarHover
                                .resolve(cx))
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

    fn toggle_keyboard_help(&mut self, cx: &mut Context<Self>) {
        self.keyboard_help_visible = !self.keyboard_help_visible;
        cx.notify();
    }

    pub(super) fn toggle_gallery_sidebar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let window_width = f32::from(window.viewport_size().width);
        self.sidebar_pin = toggled_sidebar_pin(self.sidebar_pin, window_width);
        cx.notify();
    }

    fn render_keyboard_help_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let descriptor = self.active_story.descriptor();
        let hint_row = |hint: &screens::KeyboardHint, cx: &mut Context<Self>| {
            h_flex()
                .w_full()
                .items_start()
                .gap_3()
                .child(
                    // Shrinks to its floor before the description column,
                    // so the panel stays readable at narrow window widths.
                    div()
                        .w(px(210.))
                        .min_w(px(96.))
                        .font_semibold()
                        .child(hint.keys),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                        .child(hint.action),
                )
                .into_any_element()
        };

        let mut rows = Vec::new();
        for hint in screens::SHARED_KEYBOARD_HINTS {
            rows.push(hint_row(hint, cx));
        }
        if !descriptor.keyboard_hints.is_empty() {
            rows.push(
                div()
                    .mt_1()
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child(format!("{} bindings", descriptor.title))
                    .into_any_element(),
            );
            for hint in descriptor.keyboard_hints {
                rows.push(hint_row(hint, cx));
            }
        }

        v_flex()
            .id("storybook-keyboard-help-panel")
            .debug_selector(|| "storybook-keyboard-help-panel".to_owned())
            .absolute()
            .left(px(12.))
            .right(px(12.))
            .bottom(px(36.))
            .max_w(px(460.))
            .p_3()
            .gap_2()
            .rounded(px(8.))
            .border_1()
            .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
            .bg(fanta_gpui::atoms::SemanticColor::BackgroundMenu
                .resolve(cx)
                .opacity(0.98))
            .shadow_lg()
            .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
            .child(
                div()
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child("KEYBOARD"),
            )
            .children(rows)
            .into_any_element()
    }

    pub(super) fn render_gallery_shell(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let active_story = self.active_story;
        let last_action = self.last_action_for_story(active_story);
        let window_width = f32::from(window.viewport_size().width);
        let narrow = window_is_narrow(window_width);
        let sidebar_hidden = sidebar_collapsed(window_width, self.sidebar_pin);

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
            .relative()
            .size_full()
            .min_h(px(0.))
            .overflow_hidden()
            .bg(fanta_gpui::atoms::SemanticColor::Background.resolve(cx))
            .text_color(fanta_gpui::atoms::SemanticColor::Text.resolve(cx))
            .child(
                h_flex()
                    .h(px(34.))
                    .w_full()
                    .flex_none()
                    .border_b_1()
                    .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                    .bg(fanta_gpui::atoms::SemanticColor::BackgroundToolbar.resolve(cx))
                    .child(
                        h_flex().h_full().pl_1().flex_none().items_center().child(
                            fanta_gpui::atoms::ui_button("storybook-sidebar-toggle")
                                .debug_selector(|| "storybook-sidebar-toggle".to_owned())
                                .ghost()
                                .small()
                                .icon(
                                    Icon::new(if sidebar_hidden {
                                        IconName::PanelLeftOpen
                                    } else {
                                        IconName::PanelLeftClose
                                    })
                                    .size_4(),
                                )
                                .tooltip(if sidebar_hidden {
                                    "Show the component sidebar"
                                } else {
                                    "Hide the component sidebar"
                                })
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.toggle_gallery_sidebar(window, cx);
                                })),
                        ),
                    )
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
                            .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                            .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                            .child(
                                fanta_gpui::atoms::ui_button("storybook-toggle-gallery-theme")
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
                                    .label(self.gallery_themes[self.gallery_theme_index].name.clone())
                                    .tooltip("Switch to the next bundled Zed theme")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.toggle_gallery_theme(cx);
                                    })),
                            )
                            .when(!narrow, |bar| {
                                bar.child(Icon::new(IconName::CircleCheck).size_4())
                                    .child("Interactive")
                            }),
                    ),
            )
            .child(
                div().flex_1().min_h(px(0.)).child({
                    let story_column = v_flex()
                        .size_full()
                        .min_w(px(0.))
                        .min_h(px(0.))
                        .child(
                            h_flex()
                                .w_full()
                                .min_h(px(86.))
                                .flex_none()
                                .flex_wrap()
                                .items_start()
                                .justify_between()
                                .gap_3()
                                .px_4()
                                .py_4()
                                .border_b_1()
                                .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                                .child(
                                    v_flex()
                                        .min_w(px(0.))
                                        .gap_1()
                                        .child(
                                            div()
                                                .typography(fanta_gpui::atoms::TypographyToken::HeadingLarge)
                                                .font_semibold()
                                                .child(active_story.title()),
                                        )
                                        .child(
                                            div()
                                                .max_w(px(780.))
                                                .typography(fanta_gpui::atoms::TypographyToken::BodyLarge)
                                                .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                                                .child(active_story.description()),
                                        ),
                                )
                                .child(
                                    fanta_gpui::atoms::ui_button("storybook-open-story-window")
                                        .outline()
                                        .small()
                                        .icon(Icon::new(IconName::ExternalLink).size_4())
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
                            // The viewport meta row lives above the scrolled
                            // canvas so preset chips stay reachable and never
                            // eat into the measured story area.
                            div()
                                .w_full()
                                .flex_none()
                                .px_4()
                                .pt_3()
                                .child(self.render_viewport_meta_row(cx)),
                        )
                        .child(
                            // The relative wrapper carries the area probe so
                            // fluid viewports can fill the visible canvas.
                            div()
                                .flex_1()
                                .min_h(px(0.))
                                .relative()
                                .child(
                                    div()
                                        .id("storybook-gallery-canvas")
                                        .debug_selector(|| "storybook-gallery-canvas".to_owned())
                                        .size_full()
                                        .overflow_scroll()
                                        .track_scroll(&self.gallery_story_scroll_handle)
                                        .bg(fanta_gpui::atoms::SemanticColor::Background.resolve(cx))
                                        .p(px(screens::viewport::GALLERY_CANVAS_PADDING))
                                        .child(
                                            v_flex()
                                                .w_full()
                                                .child(self.render_story_viewport(cx))
                                                .when_some(
                                                    active_story.descriptor().render_knobs,
                                                    |canvas, knobs| {
                                                        canvas.child(
                                                            self.render_gallery_knobs_section(
                                                                narrow, knobs, cx,
                                                            ),
                                                        )
                                                    },
                                                ),
                                        ),
                                )
                                .child(self.render_story_area_probe(cx)),
                        );
                    if sidebar_hidden {
                        story_column.into_any_element()
                    } else {
                        h_resizable("storybook-gallery-layout")
                            .child(
                                resizable_panel()
                                    .size(px(255.))
                                    .size_range(px(200.)..px(320.))
                                    .child(
                                        div()
                                            .id("storybook-gallery-sidebar")
                                            .debug_selector(|| {
                                                "storybook-gallery-sidebar".to_owned()
                                            })
                                            .size_full()
                                            .child(self.render_gallery_sidebar(cx)),
                                    ),
                            )
                            .child(resizable_panel().child(story_column))
                            .into_any_element()
                    }
                }),
            )
            .child(
                h_flex()
                    .h(px(28.))
                    .w_full()
                    .flex_none()
                    .px_3()
                    .gap_2()
                    .overflow_hidden()
                    .border_t_1()
                    .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                    .bg(fanta_gpui::atoms::SemanticColor::BackgroundToolbar.resolve(cx))
                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                    .child(
                        fanta_gpui::atoms::ui_button("storybook-keyboard-help-toggle")
                            .debug_selector(|| "storybook-keyboard-help-toggle".to_owned())
                            .ghost()
                            .xsmall()
                            .compact()
                            .label("?")
                            .selected(self.keyboard_help_visible)
                            .tooltip("Show this story's key bindings")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_keyboard_help(cx);
                            })),
                    )
                    .child(div().size(px(7.)).rounded_full().bg(fanta_gpui::atoms::SemanticColor::BackgroundSuccess.resolve(cx)))
                    .child(div().font_medium().child(active_story.title()))
                    .child(
                        div()
                            .max_w(px(720.))
                            .truncate()
                            .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                            .child(last_action),
                    )
                    .child(div().flex_1())
                    .when(!narrow, |footer| {
                        footer.child(
                            div()
                                .flex_none()
                                .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                                .child(format!(
                                    "{} stories · {} · typed host intents",
                                    screens::registry().len(),
                                    self.gallery_themes[self.gallery_theme_index].name
                                )),
                        )
                    }),
            )
            .when(self.keyboard_help_visible, |shell| {
                shell.child(self.render_keyboard_help_panel(cx))
            })
            .into_any_element()
    }

    /// The collapsible wrapper around a story's runtime knobs. Narrow
    /// windows collapse the section by default so the knobs never crowd
    /// out the story surface; the user's toggle wins at any width and
    /// persists across story switches.
    fn render_gallery_knobs_section(
        &self,
        narrow: bool,
        render_knobs: fn(&Storybook, &mut Context<Storybook>) -> AnyElement,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let expanded = knobs_expanded(narrow, self.knobs_user_expanded);
        v_flex()
            .w_full()
            .mt_3()
            .child(
                h_flex().w_full().child(
                    fanta_gpui::atoms::ui_button("storybook-knobs-toggle")
                        .debug_selector(|| "storybook-knobs-toggle".to_owned())
                        .outline()
                        .xsmall()
                        .icon(
                            Icon::new(if expanded {
                                IconName::ChevronDown
                            } else {
                                IconName::ChevronRight
                            })
                            .size_4(),
                        )
                        .label(if expanded { "Hide knobs" } else { "Show knobs" })
                        .tooltip("Collapse or expand this story's runtime knobs")
                        .on_click(cx.listener(|this, _, window, cx| {
                            let narrow = window_is_narrow(f32::from(window.viewport_size().width));
                            let expanded = knobs_expanded(narrow, this.knobs_user_expanded);
                            this.knobs_user_expanded = Some(!expanded);
                            cx.notify();
                        })),
                ),
            )
            .when(expanded, |section| section.child(render_knobs(self, cx)))
            .into_any_element()
    }

    pub(super) fn render_story_window(
        &self,
        story: StoryKind,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Below the story's smallest registered viewport the window
        // scrolls instead of clipping the component.
        let (min_width, min_height) = story.descriptor().min_story_size();
        div()
            .id("storybook-story-window")
            .debug_selector(|| "storybook-story-window".to_owned())
            .size_full()
            .min_w(px(0.))
            .min_h(px(0.))
            .overflow_scroll()
            .bg(fanta_gpui::atoms::SemanticColor::Background.resolve(cx))
            .text_color(fanta_gpui::atoms::SemanticColor::Text.resolve(cx))
            .child(
                div()
                    .id("storybook-story-window-content")
                    .debug_selector(|| "storybook-story-window-content".to_owned())
                    .size_full()
                    .min_w(px(min_width))
                    .min_h(px(min_height))
                    .child(self.render_story_component(story, cx)),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sidebar_auto_collapses_only_below_the_narrow_threshold() {
        assert!(!sidebar_collapsed(1440., SidebarPin::Auto));
        assert!(!sidebar_collapsed(
            GALLERY_NARROW_WINDOW_WIDTH,
            SidebarPin::Auto
        ));
        assert!(sidebar_collapsed(
            GALLERY_NARROW_WINDOW_WIDTH - 1.,
            SidebarPin::Auto
        ));
        assert!(sidebar_collapsed(320., SidebarPin::Auto));

        // Explicit pins win at any width.
        assert!(sidebar_collapsed(1440., SidebarPin::Collapsed));
        assert!(sidebar_collapsed(320., SidebarPin::Collapsed));
        assert!(!sidebar_collapsed(1440., SidebarPin::Expanded));
        assert!(!sidebar_collapsed(320., SidebarPin::Expanded));
    }

    #[test]
    fn sidebar_toggles_pin_collapses_and_release_wide_reopens_to_auto() {
        // Collapsing is always an explicit pin, so later window growth
        // does not reopen it.
        assert_eq!(
            toggled_sidebar_pin(SidebarPin::Auto, 1440.),
            SidebarPin::Collapsed
        );
        assert_eq!(
            toggled_sidebar_pin(SidebarPin::Expanded, 320.),
            SidebarPin::Collapsed
        );

        // Reopening while narrow pins the sidebar open; reopening once the
        // window is wide returns to Auto so it keeps auto-collapsing.
        assert_eq!(
            toggled_sidebar_pin(SidebarPin::Collapsed, 320.),
            SidebarPin::Expanded
        );
        assert_eq!(
            toggled_sidebar_pin(SidebarPin::Collapsed, 1440.),
            SidebarPin::Auto
        );
        assert_eq!(
            toggled_sidebar_pin(SidebarPin::Auto, 320.),
            SidebarPin::Expanded,
            "an auto-collapsed sidebar reopens pinned while the window stays narrow"
        );
    }

    #[test]
    fn knobs_default_by_window_width_and_the_user_choice_wins() {
        assert!(knobs_expanded(false, None));
        assert!(!knobs_expanded(true, None));
        assert!(knobs_expanded(true, Some(true)));
        assert!(!knobs_expanded(false, Some(false)));
    }
}
