use std::{
    any::Any,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicU8, Ordering},
    },
};

use gpui::{
    AnyElement, App, AppContext as _, AssetSource as _, FocusHandle, Focusable as _, Font,
    IntoElement as _, Pixels, SharedString, Subscription, Window, font, px,
};
use gpui_component::input::{Input, InputEvent, InputState};
use theme::{ThemeSettingsProvider, UiDensity};
use ui as _;
use ui_input::{ErasedEditor, ErasedEditorEvent};

use crate::fixtures::StorybookAssets;

struct StorybookThemeSettings {
    ui_font: Font,
    buffer_font: Font,
    density: Arc<AtomicU8>,
}

static UI_DENSITY: OnceLock<Arc<AtomicU8>> = OnceLock::new();

pub(super) fn set_density(index: u8) {
    if let Some(density) = UI_DENSITY.get() {
        density.store(index.min(2), Ordering::Relaxed);
    }
}

impl ThemeSettingsProvider for StorybookThemeSettings {
    fn ui_font<'a>(&'a self, _cx: &'a App) -> &'a Font {
        &self.ui_font
    }

    fn buffer_font<'a>(&'a self, _cx: &'a App) -> &'a Font {
        &self.buffer_font
    }

    fn ui_font_size(&self, _cx: &App) -> Pixels {
        px(16.)
    }

    fn buffer_font_size(&self, _cx: &App) -> Pixels {
        px(15.)
    }

    fn ui_density(&self, _cx: &App) -> UiDensity {
        match self.density.load(Ordering::Relaxed) {
            0 => UiDensity::Compact,
            2 => UiDensity::Comfortable,
            _ => UiDensity::Default,
        }
    }
}

struct StorybookEditor(gpui::Entity<InputState>);

impl ErasedEditor for StorybookEditor {
    fn text(&self, cx: &App) -> String {
        self.0.read(cx).value().to_string()
    }

    fn set_text(&self, text: &str, window: &mut Window, cx: &mut App) {
        self.0
            .update(cx, |input, cx| input.set_value(text, window, cx));
    }

    fn clear(&self, window: &mut Window, cx: &mut App) {
        self.set_text("", window, cx);
    }

    fn set_placeholder_text(&self, text: &str, window: &mut Window, cx: &mut App) {
        self.0
            .update(cx, |input, cx| input.set_placeholder(text, window, cx));
    }

    fn move_selection_to_end(&self, _window: &mut Window, cx: &mut App) {
        self.0
            .update(cx, |input, cx| input.move_selection_to_end(cx));
    }

    fn select_all(&self, _window: &mut Window, cx: &mut App) {
        self.0.update(cx, |input, cx| input.select_all_text(cx));
    }

    fn set_masked(&self, masked: bool, window: &mut Window, cx: &mut App) {
        self.0
            .update(cx, |input, cx| input.set_masked(masked, window, cx));
    }

    fn set_read_only(&self, read_only: bool, cx: &mut App) {
        self.0
            .update(cx, |input, cx| input.set_read_only(read_only, cx));
    }

    fn set_multiline(&self, max_lines: Option<usize>, _window: &mut Window, cx: &mut App) {
        self.0
            .update(cx, |input, cx| input.set_multiline(max_lines, cx));
    }

    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.0.read(cx).focus_handle(cx)
    }

    fn subscribe(
        &self,
        mut callback: Box<dyn FnMut(ErasedEditorEvent, &mut Window, &mut App) + 'static>,
        window: &mut Window,
        cx: &mut App,
    ) -> Subscription {
        window.subscribe(&self.0, cx, move |_, event: &InputEvent, window, cx| {
            let event = match event {
                InputEvent::Change => ErasedEditorEvent::BufferEdited,
                InputEvent::Blur => ErasedEditorEvent::Blurred,
                _ => return,
            };
            callback(event, window, cx);
        })
    }

    fn render(&self, _window: &mut Window, _cx: &App) -> AnyElement {
        Input::new(&self.0)
            .appearance(false)
            .bordered(false)
            .into_any_element()
    }

    fn as_any(&self) -> &dyn Any {
        &self.0
    }
}

fn create_editor(window: &mut Window, cx: &mut App) -> Arc<dyn ErasedEditor> {
    Arc::new(StorybookEditor(cx.new(|cx| InputState::new(window, cx))))
}

pub(super) fn init(cx: &mut App) {
    theme::init(theme::LoadThemes::All(Box::new(StorybookAssets)), cx);
    let density = UI_DENSITY
        .get_or_init(|| Arc::new(AtomicU8::new(1)))
        .clone();
    set_density(1);
    theme::set_theme_settings_provider(
        Box::new(StorybookThemeSettings {
            ui_font: font(".ZedSans"),
            buffer_font: font(".ZedMono"),
            density,
        }),
        cx,
    );
    ui_input::ERASED_EDITOR_FACTORY.get_or_init(|| create_editor);
    component::init();
    for path in asset_paths("fonts/") {
        if !path.ends_with(".ttf") {
            continue;
        }
        match StorybookAssets.load(&path) {
            Ok(Some(bytes)) => {
                if let Err(error) = cx.text_system().add_fonts(vec![bytes]) {
                    eprintln!("failed to load bundled font {path}: {error}");
                }
            }
            Ok(None) => eprintln!("bundled font missing: {path}"),
            Err(error) => eprintln!("failed to read bundled font {path}: {error}"),
        }
    }
    let registry = theme::ThemeRegistry::global(cx);
    for path in asset_paths("themes/") {
        if !path.ends_with(".json") {
            continue;
        }
        match StorybookAssets
            .load(&path)
            .and_then(|bytes| bytes.ok_or_else(|| anyhow::anyhow!("bundled theme missing: {path}")))
        {
            Ok(bytes) => match theme::decode_bundled_theme(&bytes) {
                Ok(family) => registry.insert_theme_families([family]),
                Err(error) => eprintln!("failed to decode bundled theme {path}: {error}"),
            },
            Err(error) => eprintln!("failed to read bundled theme {path}: {error}"),
        }
    }
    // This gallery-only theme is also offered by `zed_themes()`. Register it
    // with the baseline so selecting it updates both theme systems together.
    match theme::decode_bundled_theme(include_bytes!("../assets/themes/figma-ui3.json")) {
        Ok(family) => registry.insert_theme_families([family]),
        Err(error) => eprintln!("failed to decode bundled Figma UI3 theme: {error}"),
    }
}

fn asset_paths(prefix: &str) -> Vec<SharedString> {
    match StorybookAssets.list(prefix) {
        Ok(paths) => paths,
        Err(error) => {
            eprintln!("failed to list bundled assets under {prefix}: {error}");
            Vec::new()
        }
    }
}
