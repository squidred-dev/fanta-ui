//! Standalone, host-controlled color picker story.
use crate::*;
use fanta_gpui::atoms::TypographyExt as _;

pub(crate) struct ColorPickerStory {
    pub(crate) picker: Entity<ColorPicker>,
    pub(crate) color: PickerColor,
    pub(crate) last_action: SharedString,
}
impl ColorPickerStory {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Storybook>) -> Self {
        let color = PickerColor::rgba(0x0d, 0x99, 0xff, 255);
        let picker = cx.new(|cx| ColorPicker::new("storybook-color-picker", color, window, cx));
        Self {
            picker,
            color,
            last_action: "Ready — edit the spectrum, hue, alpha, or color fields".into(),
        }
    }
    pub(crate) fn handle_action(
        &mut self,
        action: &ColorPickerAction,
        window: &mut Window,
        cx: &mut Context<Storybook>,
    ) {
        match action {
            ColorPickerAction::Edit {
                color,
                phase: ColorPickerPhase::Commit,
            } => {
                self.color = *color;
                self.picker
                    .update(cx, |picker, cx| picker.set_color(*color, window, cx));
            }
            ColorPickerAction::EyedropperRequested => {
                // Storybook stands in for the application canvas sampler.
                let sampled = PickerColor::rgba(0x16, 0xb8, 0xa6, 255);
                self.color = sampled;
                self.picker
                    .update(cx, |picker, cx| picker.set_color(sampled, window, cx));
                self.last_action = "Canvas sample accepted: 16B8A6".into();
                cx.notify();
                return;
            }
            _ => {}
        }
        self.last_action = format!("{action:?}").into();
        cx.notify();
    }
}
impl Storybook {
    pub(crate) fn render_color_picker_story(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .size_full()
            .p_3()
            .gap_3()
            .items_center()
            .child(
                h_flex().gap_2().children(
                    [
                        ("Blue", PickerColor::rgba(13, 153, 255, 255)),
                        ("Alpha", PickerColor::rgba(240, 50, 80, 128)),
                        ("White", PickerColor::rgba(255, 255, 255, 255)),
                    ]
                    .into_iter()
                    .map(|(label, color)| {
                        fanta_gpui::atoms::ui_button(SharedString::from(format!(
                            "color-fixture-{label}"
                        )))
                        .label(label)
                        .xsmall()
                        .compact()
                        .on_activate(cx.listener(
                            move |this, _, window, cx| {
                                this.color_picker_screen.picker.update(cx, |picker, cx| {
                                    picker.cancel(window, cx);
                                    picker.set_color(color, window, cx);
                                });
                                this.color_picker_screen.color = color;
                                cx.notify();
                            },
                        ))
                    }),
                ),
            )
            .child(self.color_picker_screen.picker.clone())
            .child(
                div()
                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                    .child(format!(
                        "Accepted RGBA: {}",
                        self.color_picker_screen.color.hex()
                    )),
            )
            .into_any_element()
    }
    pub(crate) fn render_color_picker_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-color-picker-reference",
            self.render_color_picker_story(cx),
            self.color_picker_screen.last_action.clone(),
            cx,
        )
    }
}
