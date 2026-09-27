#[cfg(test)]
mod tests {
    use component::Component;
    use gpui::{AnyElement, App, IntoElement as _, Window, div};
    use ui_macros::RegisterComponent;

    #[derive(RegisterComponent)]
    struct RegisteredButton;

    impl Component for RegisteredButton {
        fn description() -> &'static str {
            "Unpublished macro registration test"
        }

        fn preview(_window: &mut Window, _cx: &mut App) -> AnyElement {
            div().into_any_element()
        }
    }

    #[test]
    fn derive_registers_in_the_component_inventory() {
        let _ = std::any::TypeId::of::<ui::Button>();
        component::init();
        assert!(
            component::components()
                .get(&component::ComponentId(std::any::type_name::<
                    RegisteredButton,
                >()))
                .is_some()
        );
    }
}
