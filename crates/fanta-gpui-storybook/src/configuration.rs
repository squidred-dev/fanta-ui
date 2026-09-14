use gpui_component::ThemeMode;

/// Story identity. Every per-story attribute and dispatch hook lives in
/// the screen registry (`screens::registry()`); this enum only names the
/// story.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum StoryKind {
    Welcome,
    Buttons,
    Labels,
    Icons,
    Menus,
    ListRows,
    Popups,
    Toolbar,
    Pages,
    Layers,
    Design,
    Variables,
    Assets,
    Prototype,
    Timeline,
    FileInspector,
    PseudoEditor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StorybookLaunchMode {
    Gallery,
    ReferenceFixture,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct StorybookLaunch {
    pub(super) story: StoryKind,
    pub(super) mode: StorybookLaunchMode,
}

fn story_for_launch_value(env_name: &str, value: &str) -> Result<StoryKind, String> {
    crate::screens::story_from_name(value).ok_or_else(|| {
        format!(
            "unknown story {value:?} in {env_name}; valid story ids: {}",
            crate::screens::registry()
                .iter()
                .map(|descriptor| descriptor.id)
                .collect::<Vec<_>>()
                .join(", ")
        )
    })
}

pub(super) fn parse_storybook_launch(
    reference_story_env: Option<&str>,
    gallery_story_env: Option<&str>,
) -> Result<StorybookLaunch, String> {
    match reference_story_env {
        Some(story) => Ok(StorybookLaunch {
            story: story_for_launch_value("FANTA_STORYBOOK_STORY", story)?,
            mode: StorybookLaunchMode::ReferenceFixture,
        }),
        None => Ok(StorybookLaunch {
            story: match gallery_story_env {
                Some(story) => story_for_launch_value("FANTA_STORYBOOK_GALLERY_STORY", story)?,
                None => StoryKind::PseudoEditor,
            },
            mode: StorybookLaunchMode::Gallery,
        }),
    }
}

pub(super) fn storybook_launch_from_env() -> Result<StorybookLaunch, String> {
    let reference_story = std::env::var_os("FANTA_STORYBOOK_STORY");
    let gallery_story = std::env::var_os("FANTA_STORYBOOK_GALLERY_STORY");
    parse_storybook_launch(
        reference_story
            .as_deref()
            .map(|value| value.to_string_lossy())
            .as_deref(),
        gallery_story
            .as_deref()
            .map(|value| value.to_string_lossy())
            .as_deref(),
    )
}

pub(super) fn parse_storybook_theme(value: Option<&str>) -> ThemeMode {
    if value.is_some_and(|value| value.eq_ignore_ascii_case("dark")) {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    }
}

pub(super) fn storybook_theme_from_env() -> ThemeMode {
    let value = std::env::var_os("FANTA_STORYBOOK_THEME");
    parse_storybook_theme(
        value
            .as_deref()
            .map(|value| value.to_string_lossy())
            .as_deref(),
    )
}

pub(super) fn storybook_window_dimension(name: &str, fallback: f32) -> f32 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .filter(|value| value.is_finite() && *value >= 240.)
        .unwrap_or(fallback)
}
