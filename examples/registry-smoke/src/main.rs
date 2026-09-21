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
