#![forbid(unsafe_code)]

use gpui::{App, Context, IntoElement, Render, Window, WindowOptions, div, prelude::*};

struct Smoke;

impl Render for Smoke {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .p_4()
            .child("Fanta GPUI registry consumer")
            .child(gpui_component::button::Button::new("smoke").label("Component ready"))
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(gpui_component_assets::Assets)
        .run(|cx: &mut App| {
            gpui_component::init(cx);
            fanta_gpui::init(cx);
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|_| Smoke);
                cx.new(|cx| gpui_component::Root::new(view, window, cx))
            })
            .expect("open smoke window");
            cx.activate(true);
            if std::env::var_os("FANTA_SMOKE_AUTO_QUIT").is_some() {
                let timer = cx
                    .background_executor()
                    .timer(std::time::Duration::from_secs(2));
                cx.spawn(async move |cx| {
                    timer.await;
                    cx.update(|cx| cx.quit());
                })
                .detach();
            }
        });
}

#[cfg(test)]
#[gpui::test]
fn framework_alias_supports_test_macro(cx: &mut gpui::TestAppContext) {
    let entity = cx.new(|_| 42);
    assert_eq!(entity.read_with(cx, |value, _| *value), 42);
}

#[cfg(test)]
#[gpui::property_test]
fn property_macro_uses_reexported_proptest(value: u16) {
    assert_eq!(value, value.saturating_add(0));
}

#[cfg(test)]
#[derive(ui_macros::RegisterComponent)]
struct SmokeComponent;

#[cfg(test)]
impl component::Component for SmokeComponent {
    fn description() -> &'static str {
        "Registry alias smoke test"
    }

    fn preview(_: &mut Window, _: &mut App) -> gpui::AnyElement {
        div().into_any_element()
    }
}

#[cfg(test)]
#[test]
fn extracted_zed_baseline_aliases_link() -> Result<(), Box<dyn std::error::Error>> {
    use std::any::TypeId;

    menu::init();
    let _icon = icons::IconName::Check;
    let _component = component::ComponentId("smoke");
    let _ = TypeId::of::<file_icons::FileIcons>();
    let _ = TypeId::of::<syntax_theme::SyntaxTheme>();
    let _ = TypeId::of::<theme::Theme>();
    let _ = TypeId::of::<ui::Button>();
    let _ = TypeId::of::<ui_input::InputField>();
    let _ = TypeId::of::<SmokeComponent>();
    let bytes = zed_ui_assets::bundled_asset("themes/one/one.json")
        .ok_or("missing bundled One theme")?;
    let family = theme::decode_bundled_theme(&bytes)?;
    assert!(family.themes.iter().any(|theme| theme.name == "One Light"));
    Ok(())
}
