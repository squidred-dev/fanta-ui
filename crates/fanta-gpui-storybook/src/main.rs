#![forbid(unsafe_code)]

mod configuration;
mod design_host;
mod fixtures;
mod gallery;
mod screens;
mod themes;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use configuration::*;
use design_host::*;
use fanta_gpui::prelude::*;
use fixtures::*;
use gallery::SidebarPin;
use gpui::{
    AnyElement, App, AppContext as _, Bounds, ClipboardItem, Context, Entity, Focusable as _,
    InteractiveElement as _, IntoElement, KeyDownEvent, Menu, MenuItem, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement as _, Render, ScrollHandle,
    SharedString, StatefulInteractiveElement as _, Styled as _, Subscription, TitlebarOptions,
    Window, WindowBounds, WindowId, WindowOptions, canvas, div, prelude::FluentBuilder as _, px,
    rgba, size,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, Root, Selectable as _, Sizable as _, StyledExt as _, Theme,
    ThemeConfig, ThemeMode,
    button::{Button, ButtonCustomVariant, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    menu::AppMenuBar,
    resizable::{h_resizable, resizable_panel},
    sidebar::{Sidebar, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
    v_flex,
};
use screens::{
    ButtonsScreen, CheckboxStory, DesignScreen, DropdownStory, FieldsScreen, FileInspectorScreen,
    IconsScreen, InputsStory, LabelsScreen, LayersScreen, ListRowsScreen, MenusScreen,
    OverlaysScreen, PagesScreen, PopupsScreen, PrototypeScreen, PseudoEditorScreen,
    RadioButtonStory, SegmentedControlStory, StructureScreen, TabsStory, TimelineScreen,
    TokensScreen, ToolbarScreen, TooltipsStory, VariablesStory, WelcomeScreen,
    viewport::StoryViewport,
};
use themes::{apply_zed_theme, initial_zed_theme_index, zed_themes};

gpui::actions!(fanta_storybook, [OpenStoryWindow, ToggleGalleryTheme]);

const STORYBOOK_KEY_CONTEXT: &str = "FantaStorybook";

struct Storybook {
    active_story: StoryKind,
    launch_mode: StorybookLaunchMode,
    story_windows: HashMap<WindowId, StoryKind>,
    gallery_theme_mode: ThemeMode,
    gallery_theme_index: usize,
    gallery_themes: Vec<Rc<ThemeConfig>>,
    gallery_search_input: Entity<InputState>,
    gallery_menu_bar: Entity<AppMenuBar>,
    gallery_story_scroll_handle: ScrollHandle,
    story_viewport: StoryViewport,
    keyboard_help_visible: bool,
    /// The user's standing sidebar choice; `Auto` follows the window width.
    sidebar_pin: SidebarPin,
    /// The user's standing knobs-section choice; `None` follows the
    /// window width (expanded when wide, collapsed when narrow).
    knobs_user_expanded: Option<bool>,
    welcome_screen: WelcomeScreen,
    buttons_screen: ButtonsScreen,
    checkbox_story: CheckboxStory,
    dropdown_story: DropdownStory,
    inputs_story: InputsStory,
    labels_screen: LabelsScreen,
    icons_screen: IconsScreen,
    tokens_screen: TokensScreen,
    menus_screen: MenusScreen,
    radio_button_story: RadioButtonStory,
    segmented_control_story: SegmentedControlStory,
    tabs_story: TabsStory,
    tooltips_story: TooltipsStory,
    list_rows_screen: ListRowsScreen,
    popups_screen: PopupsScreen,
    fields_screen: FieldsScreen,
    structure_screen: StructureScreen,
    overlays_screen: OverlaysScreen,
    variables_screen: VariablesStory,
    sliders_screen: screens::sliders::SlidersStory,
    color_picker_screen: screens::color_picker::ColorPickerStory,
    prototype_screen: PrototypeScreen,
    timeline_screen: TimelineScreen,
    file_inspector_screen: FileInspectorScreen,
    pseudo_screen: PseudoEditorScreen,
    pages_screen: PagesScreen,
    layers_screen: LayersScreen,
    toolbar_screen: ToolbarScreen,
    design_screen: DesignScreen,
    _subscriptions: Vec<Subscription>,
}

impl Storybook {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let launch = storybook_launch_from_env()
            .expect("the storybook launch environment is validated in main()");
        let gallery_search_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Search components..."));
        let gallery_menu_bar = AppMenuBar::new(window, cx);
        let gallery_story_scroll_handle = ScrollHandle::new();
        let welcome_screen = WelcomeScreen::new(cx);
        let buttons_screen = ButtonsScreen::new(cx);
        let checkbox_story = CheckboxStory::new(cx);
        let dropdown_story = DropdownStory::new(cx);
        let inputs_story = InputsStory::new(window, cx);
        let labels_screen = LabelsScreen::new(window, cx);
        let icons_screen = IconsScreen::new(window, cx);
        let tokens_screen = TokensScreen::new(cx);
        let menus_screen = MenusScreen::new(cx);
        let radio_button_story = RadioButtonStory::new(cx);
        let segmented_control_story = SegmentedControlStory::new(cx);
        let tabs_story = TabsStory::new(cx);
        let tooltips_story = TooltipsStory::new(cx);
        let list_rows_screen = ListRowsScreen::new(cx);
        let popups_screen = PopupsScreen::new(cx);
        let fields_screen = FieldsScreen::new(cx);
        let structure_screen = StructureScreen::new(cx);
        let overlays_screen = OverlaysScreen::new(cx);
        let variables_screen = VariablesStory::new(window, cx);
        let sliders_screen = screens::sliders::SlidersStory::new(cx);
        let color_picker_screen = screens::color_picker::ColorPickerStory::new(window, cx);
        let prototype_screen = PrototypeScreen::new(cx);
        let timeline_screen = TimelineScreen::new(cx);
        let pages_screen = PagesScreen::new(window, cx);
        let layers_screen = LayersScreen::new(window, cx);
        let file_inspector_screen = FileInspectorScreen::new(window, cx);
        let design_screen = DesignScreen::new(window, cx);
        let toolbar_screen = ToolbarScreen::new(window, cx);

        let pseudo_screen = PseudoEditorScreen::new(
            PseudoEditorChildren {
                pages: pages_screen.panel.clone(),
                layers: layers_screen.panel.clone(),
                design: design_screen.panel.clone(),
                prototype: prototype_screen.panel.clone(),
                timeline: timeline_screen.timeline.clone(),
                toolbar: toolbar_screen.toolbar.clone(),
                variables: variables_screen.screen.clone(),
            },
            cx,
        );

        let mut subscriptions = vec![
            cx.subscribe(
                &file_inspector_screen.pages.panel,
                |story, panel, action: &PagesPanelAction, cx| {
                    story
                        .file_inspector_screen
                        .pages
                        .handle_action(panel, action, cx);
                },
            ),
            cx.subscribe(
                &file_inspector_screen.layers.panel,
                |story, panel, action: &LayersPanelAction, cx| {
                    story
                        .file_inspector_screen
                        .layers
                        .handle_action(panel, action, cx);
                },
            ),
            cx.subscribe(
                &variables_screen.screen,
                |story, _, action: &fanta_gpui::variables::VariablesContextAction, cx| {
                    story.variables_screen.handle_context_action(action, cx);
                },
            ),
            cx.subscribe_in(
                &color_picker_screen.picker,
                window,
                |story, _, action: &ColorPickerAction, window, cx| {
                    story.color_picker_screen.handle_action(action, window, cx);
                },
            ),
            cx.subscribe(&gallery_search_input, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            }),
            cx.subscribe(
                &variables_screen.screen,
                |story, screen, action: &VariablesAction, cx| {
                    story.variables_screen.handle_action(screen, action, cx);
                },
            ),
            cx.subscribe(
                &prototype_screen.panel,
                |story, panel, action: &PrototypePanelAction, cx| {
                    story.prototype_screen.handle_action(panel, action, cx);
                },
            ),
            cx.subscribe(
                &timeline_screen.timeline,
                |story, timeline, action: &TimelineAction, cx| {
                    story.timeline_screen.handle_action(timeline, action, cx);
                },
            ),
            cx.subscribe(
                &pseudo_screen.editor,
                |story, editor, action: &PseudoEditorAction, cx| {
                    story.pseudo_screen.handle_action(editor, action, cx);
                },
            ),
            cx.subscribe(
                &pages_screen.panel,
                |story, panel, action: &PagesPanelAction, cx| {
                    story.pages_screen.handle_action(panel, action, cx);
                },
            ),
            cx.subscribe(
                &layers_screen.panel,
                |story, panel, action: &LayersPanelAction, cx| {
                    story.layers_screen.handle_action(panel, action, cx);
                },
            ),
            cx.subscribe(
                &design_screen.panel,
                |story, panel, action: &DesignPanelAction, cx| {
                    story.design_screen.handle_action(panel, action, cx);
                },
            ),
            cx.subscribe(
                &toolbar_screen.toolbar,
                |story, toolbar, action: &ToolbarAction, cx| {
                    story.handle_toolbar_action(toolbar, action, cx);
                },
            ),
            cx.subscribe(
                &labels_screen.truncation,
                |story, panel, action: &LayersPanelAction, cx| {
                    story
                        .labels_screen
                        .handle_truncation_action(panel, action, cx);
                },
            ),
        ];
        let storybook = cx.weak_entity();
        subscriptions.push(cx.on_window_closed(move |cx, _window_id| {
            let open_window_ids = cx
                .windows()
                .into_iter()
                .map(|window| window.window_id())
                .collect::<HashSet<_>>();
            if let Some(storybook) = storybook.upgrade() {
                storybook.update(cx, |storybook, cx| {
                    let previous_count = storybook.story_windows.len();
                    storybook
                        .story_windows
                        .retain(|window_id, _| open_window_ids.contains(window_id));
                    if storybook.story_windows.len() != previous_count {
                        cx.notify();
                    }
                });
            }
        }));

        let gallery_themes = zed_themes();
        let gallery_theme_index = gallery_themes
            .iter()
            .position(|theme| theme.name == *Theme::global(cx).theme_name())
            .unwrap_or_else(|| initial_zed_theme_index(Theme::global(cx).mode));
        for slider in &sliders_screen.sliders {
            subscriptions.push(
                cx.subscribe(slider, |story, slider, action: &SliderAction, cx| {
                    if action.phase == SliderPhase::Commit {
                        slider.update(cx, |slider, cx| slider.set_value(action.value, cx));
                    }
                    story.sliders_screen.last_action =
                        format!("{:?}: {:.0}%", action.phase, action.value * 100.).into();
                    cx.notify();
                }),
            );
        }
        let storybook = Self {
            active_story: launch.story,
            launch_mode: launch.mode,
            story_windows: HashMap::new(),
            gallery_theme_mode: Theme::global(cx).mode,
            gallery_theme_index,
            gallery_themes,
            gallery_search_input,
            gallery_menu_bar,
            gallery_story_scroll_handle,
            story_viewport: StoryViewport::for_story(launch.story),
            keyboard_help_visible: false,
            sidebar_pin: SidebarPin::Auto,
            knobs_user_expanded: None,
            welcome_screen,
            buttons_screen,
            checkbox_story,
            dropdown_story,
            inputs_story,
            labels_screen,
            icons_screen,
            tokens_screen,
            menus_screen,
            radio_button_story,
            segmented_control_story,
            tabs_story,
            tooltips_story,
            list_rows_screen,
            popups_screen,
            fields_screen,
            structure_screen,
            overlays_screen,
            variables_screen,
            sliders_screen,
            color_picker_screen,
            prototype_screen,
            timeline_screen,
            file_inspector_screen,
            pseudo_screen,
            pages_screen,
            layers_screen,
            toolbar_screen,
            design_screen,
            _subscriptions: subscriptions,
        };
        storybook.focus_story(storybook.active_story, window, cx);
        storybook
    }

    fn focus_story(&self, kind: StoryKind, window: &mut Window, cx: &mut Context<Self>) {
        (kind.descriptor().focus)(self, window, cx);
    }

    fn activate_gallery_story(
        &mut self,
        kind: StoryKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.active_story = kind;
        // Carry the measured story area across the switch so a fluid
        // story lays out correctly on its first frame.
        let available = self.story_viewport.available;
        self.story_viewport = StoryViewport::for_story(kind);
        self.story_viewport.available = available;
        self.gallery_story_scroll_handle
            .set_offset(Default::default());
        self.focus_story(kind, window, cx);
        cx.notify();
    }

    fn last_action_for_story(&self, kind: StoryKind) -> SharedString {
        (kind.descriptor().last_action)(self)
    }

    fn render_story_component(&self, story: StoryKind, cx: &mut Context<Self>) -> AnyElement {
        (story.descriptor().render_story)(self, cx)
    }

    fn render_gallery_story_component(&self, cx: &mut Context<Self>) -> AnyElement {
        let descriptor = self.active_story.descriptor();
        match descriptor.render_gallery {
            Some(render_gallery) => render_gallery(self, cx),
            None => (descriptor.render_story)(self, cx),
        }
    }
}

impl Render for Storybook {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(story) = self
            .story_windows
            .get(&window.window_handle().window_id())
            .copied()
        {
            return self.render_story_window(story, cx);
        }

        if self.launch_mode == StorybookLaunchMode::Gallery {
            return self.render_gallery_shell(window, cx);
        }

        (self.active_story.descriptor().render_reference)(self, cx)
    }
}

fn apply_figma_ui3_storybook_theme(cx: &mut App) {
    let theme = Theme::global_mut(cx);
    theme.background = rgba(0x2c2c2cff).into();
    theme.foreground = rgba(0xffffffff).into();
    theme.secondary = rgba(0x383838ff).into();
    theme.secondary_hover = rgba(0x444444ff).into();
    theme.secondary_active = rgba(0x4d4d4dff).into();
    theme.accent = rgba(0x444444ff).into();
    theme.accent_foreground = rgba(0xffffffff).into();
    theme.border = rgba(0x444444ff).into();
    theme.input = rgba(0x444444ff).into();
    theme.muted = rgba(0x383838ff).into();
    theme.muted_foreground = rgba(0xbfbfbfff).into();
    theme.popover = rgba(0x2c2c2cff).into();
    theme.popover_foreground = rgba(0xffffffff).into();
    theme.selection = rgba(0x0c8ce9ff).into();
    theme.sidebar = rgba(0x2c2c2cff).into();
    theme.sidebar_border = rgba(0x444444ff).into();
    theme.sidebar_foreground = rgba(0xffffffff).into();
    theme.tab = rgba(0x2c2c2cff).into();
    theme.tab_active = rgba(0x383838ff).into();
    theme.tab_active_foreground = rgba(0xffffffff).into();
    theme.title_bar = rgba(0x2c2c2cff).into();
    theme.title_bar_border = rgba(0x444444ff).into();
    theme.radius = px(4.);
}

fn main() {
    let launch = match storybook_launch_from_env() {
        Ok(launch) => launch,
        Err(error) => {
            eprintln!("fanta-gpui-storybook: {error}");
            std::process::exit(1);
        }
    };
    gpui_platform::application()
        .with_assets(StorybookAssets)
        .run(move |cx: &mut App| {
            gpui_component::init(cx);
            fanta_gpui::init(cx);
            match launch.mode {
                StorybookLaunchMode::Gallery => {
                    let index = initial_zed_theme_index(storybook_theme_from_env());
                    let themes = zed_themes();
                    apply_zed_theme(&themes[index], cx);
                }
                StorybookLaunchMode::ReferenceFixture => {
                    Theme::change(ThemeMode::Dark, None, cx);
                    apply_figma_ui3_storybook_theme(cx);
                }
            }
            cx.set_menus(vec![
                Menu::new("Fanta GPUI")
                    .items([MenuItem::action("Toggle Theme", ToggleGalleryTheme)]),
                Menu::new("Edit"),
                Menu::new("Window").items([MenuItem::action("Open Story Window", OpenStoryWindow)]),
                Menu::new("Help"),
            ]);
            cx.activate(true);

            let initial_story = launch.story;
            let (reference_width, reference_height) = if launch.mode == StorybookLaunchMode::Gallery
            {
                (1240., 820.)
            } else {
                initial_story.reference_window_size()
            };
            let window_width = storybook_window_dimension("FANTA_STORYBOOK_WIDTH", reference_width);
            let window_height =
                storybook_window_dimension("FANTA_STORYBOOK_HEIGHT", reference_height);
            let bounds = Bounds::centered(None, size(px(window_width), px(window_height)), cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                // The registered window minimum: the gallery shell reflows
                // (collapsing its sidebar and knobs) instead of clipping.
                window_min_size: Some(size(px(320.), px(240.))),
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

#[cfg(test)]
mod tests;
