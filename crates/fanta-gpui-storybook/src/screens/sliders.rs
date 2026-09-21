//! Slider family: live host-controlled values and theme-aware primitive matrices.
use crate::*;
use fanta_gpui::atoms::TypographyExt as _;

pub(crate) struct SlidersStory {
    pub(crate) sliders: Vec<Entity<Slider>>,
    pub(crate) last_action: SharedString,
    selected_stop: usize,
    disabled: bool,
    stop_positions: [f32; 3],
    stop_focus: [gpui::FocusHandle; 3],
    stop_bounds: gpui::Bounds<gpui::Pixels>,
    stop_drag: Option<(usize, [f32; 3])>,
}
impl SlidersStory {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        let variants = [
            SliderVariant::Range,
            SliderVariant::Stepper { intervals: 6 },
            SliderVariant::Centered,
            SliderVariant::CornerRadius { marker: 0.7 },
            SliderVariant::ColorRange,
            SliderVariant::Fill,
            SliderVariant::Range,
        ];
        let sliders = variants
            .into_iter()
            .enumerate()
            .map(|(i, variant)| {
                cx.new(|cx| {
                    let mut slider = Slider::new(format!("slider-story-{i}"), 0.35, cx);
                    let background = if variant == SliderVariant::Fill {
                        SliderBackground::Opacity(
                            fanta_gpui::atoms::SemanticColor::BackgroundBrand.resolve(cx),
                        )
                    } else {
                        SliderBackground::Default
                    };
                    slider.configure(variant, background, i == 6, cx);
                    slider
                })
            })
            .collect();
        Self {
            sliders,
            last_action: "Ready — drag or use arrows, Home and End; Shift makes larger steps"
                .into(),
            selected_stop: 0,
            disabled: false,
            stop_positions: [0., 0.5, 1.],
            stop_focus: std::array::from_fn(|_| cx.focus_handle()),
            stop_bounds: gpui::Bounds::default(),
            stop_drag: None,
        }
    }
}
const LABELS: [&str; 7] = [
    "Range",
    "Stepper · 7 positions",
    "Centered · neutral at 50%",
    "Corner radius · reference mark",
    "Color range · hue",
    "Fill · opacity",
    "Disabled",
];
fn row(label: impl Into<SharedString>, content: impl IntoElement) -> AnyElement {
    v_flex()
        .gap_2()
        .w_full()
        .child(
            div()
                .typography(fanta_gpui::atoms::TypographyToken::BodyLarge)
                .child(label.into()),
        )
        .child(content)
        .into_any_element()
}
impl Storybook {
    pub(crate) fn render_sliders_story(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .w_full()
            .max_w(px(440.))
            .h_full()
            .p_4()
            .gap_4()
            .child(
                div()
                    .typography(fanta_gpui::atoms::TypographyToken::BodyLarge)
                    .child("Drag a handle or focus a rail and use the arrow keys."),
            )
            .children(
                self.sliders_screen
                    .sliders
                    .iter()
                    .enumerate()
                    .map(|(i, slider)| {
                        row(
                            format!("{} · {:.0}%", LABELS[i], slider.read(cx).value() * 100.),
                            slider.clone(),
                        )
                    }),
            )
            .child(self.render_gradient_stop_demo(cx))
            .into_any_element()
    }
    pub(crate) fn render_slider_backgrounds_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let color = fanta_gpui::atoms::SemanticColor::BackgroundBrand.resolve(cx);
        v_flex().w_full().max_w(px(440.)).h_full().p_4().gap_4()
            .child(div().typography(fanta_gpui::atoms::TypographyToken::BodyLarge).child("Backgrounds inherit the active theme. Color rails keep their color meaning in both themes."))
            .children([
                ("Default",SliderBackground::Default,SliderState::Default),
                ("Focused",SliderBackground::Default,SliderState::Focused),
                ("Disabled",SliderBackground::Default,SliderState::Disabled),
                ("Gradient · hue spectrum",SliderBackground::Hue,SliderState::Default),
                ("Fill · transparency",SliderBackground::Opacity(color),SliderState::Default),
                ("Custom gradient · three stops",SliderBackground::Gradient(vec![(0.,color.opacity(0.)),(0.4,color),(1.,fanta_gpui::atoms::SemanticColor::Text.resolve(cx))]),SliderState::Default),
            ].into_iter().map(|(label,bg,state)|row(label,bg.render(state,cx))))
            .child(row("Live fill preview",self.sliders_screen.sliders[5].clone())).into_any_element()
    }
    pub(crate) fn render_slider_handles_story(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex().w_full().max_w(px(440.)).h_full().p_4().gap_4()
            .child(div().typography(fanta_gpui::atoms::TypographyToken::BodyLarge).child("Default / Focused / Disabled. Tab to the live slider to check its actual focus ring."))
            .children([(SliderHandleVariant::Fill,"Fill"),(SliderHandleVariant::Stroke,"Stroke · modified"),(SliderHandleVariant::Chit,"Chit · color")].into_iter().map(|(variant,label)| {
                row(label,h_flex().gap_6().children([SliderState::Default,SliderState::Focused,SliderState::Disabled].into_iter().map(|state|SliderHandle{variant,state,color:Some(fanta_gpui::atoms::SemanticColor::BackgroundBrand.resolve(cx))}.render(cx))))
            }))
            .child(row("Live handle",self.sliders_screen.sliders[0].clone())).into_any_element()
    }
    fn move_story_stop(&mut self, position: gpui::Point<gpui::Pixels>, cx: &mut Context<Self>) {
        let Some((index, _)) = self.sliders_screen.stop_drag else {
            return;
        };
        let bounds = self.sliders_screen.stop_bounds;
        let inset = fanta_gpui::atoms::tokens::SliderGeometry::STOP_SIZE / 2.;
        let width = f32::from(bounds.size.width) - 2. * inset;
        if width <= 0. {
            return;
        }
        self.sliders_screen.stop_positions[index] =
            ((f32::from(position.x - bounds.left()) - inset) / width).clamp(0., 1.);
        self.sliders_screen.last_action = format!(
            "Stop {} · {:.0}%",
            index + 1,
            self.sliders_screen.stop_positions[index] * 100.
        )
        .into();
        cx.notify();
    }
    fn render_gradient_stop_demo(&self, cx: &mut Context<Self>) -> AnyElement {
        use fanta_gpui::atoms::tokens::SliderGeometry;
        use gpui::{KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
        let colors = [
            fanta_gpui::atoms::SemanticColor::BackgroundBrand
                .resolve(cx)
                .opacity(0.25),
            fanta_gpui::atoms::SemanticColor::BackgroundBrand.resolve(cx),
            fanta_gpui::atoms::SemanticColor::Text.resolve(cx),
        ];
        let positions = self.sliders_screen.stop_positions;
        let disabled = self.sliders_screen.disabled;
        let selected = self.sliders_screen.selected_stop;
        let rail = div()
            .id("slider-gradient-demo")
            .debug_selector(|| "slider-gradient-demo".to_owned())
            .relative()
            .w_full()
            .h(px(SliderGeometry::STOP_SIZE + SliderGeometry::TRACK_HEIGHT))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if event.dragging() {
                    this.move_story_stop(event.position, cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| {
                    this.sliders_screen.stop_drag = None;
                    cx.notify();
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| {
                    this.sliders_screen.stop_drag = None;
                    cx.notify();
                }),
            )
            .child(track_bounds(cx.entity(), |this, bounds| {
                this.sliders_screen.stop_bounds = bounds
            }))
            .child(
                div().absolute().left_0().right_0().bottom_0().child(
                    SliderBackground::Gradient(
                        colors
                            .into_iter()
                            .enumerate()
                            .map(|(i, c)| (positions[i], c))
                            .collect(),
                    )
                    .render(
                        if disabled {
                            SliderState::Disabled
                        } else {
                            SliderState::Default
                        },
                        cx,
                    ),
                ),
            )
            .child(
                div()
                    .absolute()
                    .left(px(SliderGeometry::STOP_SIZE / 2.))
                    .right(px(SliderGeometry::STOP_SIZE / 2.))
                    .top_0()
                    .bottom_0()
                    .children(colors.into_iter().enumerate().map(|(i, color)| {
                        SliderGradientStop {
                            color,
                            selected: selected == i,
                            disabled,
                        }
                        .render(SharedString::from(format!("slider-stop-story-{i}")), cx)
                        .debug_selector(move || format!("slider-stop-story-{i}"))
                        .track_focus(
                            &self.sliders_screen.stop_focus[i]
                                .clone()
                                .tab_index(0)
                                .tab_stop(!disabled),
                        )
                        .absolute()
                        .left(gpui::relative(positions[i]))
                        .ml(px(-SliderGeometry::STOP_SIZE / 2.))
                        .top_0()
                        .on_activate(cx.listener(move |this, _, _, cx| {
                            if !this.sliders_screen.disabled {
                                this.sliders_screen.selected_stop = i;
                                cx.notify();
                            }
                        }))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                                if this.sliders_screen.disabled {
                                    return;
                                }
                                this.sliders_screen.stop_focus[i].focus(window, cx);
                                this.sliders_screen.selected_stop = i;
                                this.sliders_screen.stop_drag =
                                    Some((i, this.sliders_screen.stop_positions));
                                this.move_story_stop(event.position, cx);
                                cx.stop_propagation();
                            }),
                        )
                        .on_key_down(cx.listener(
                            move |this, event: &KeyDownEvent, window, cx| {
                                if this.sliders_screen.disabled {
                                    return;
                                }
                                let step = if event.keystroke.modifiers.shift {
                                    0.1
                                } else {
                                    0.01
                                };
                                let value = this.sliders_screen.stop_positions[i];
                                let next = match event.keystroke.key.as_str() {
                                    "left" => value - step,
                                    "right" => value + step,
                                    "home" => 0.,
                                    "end" => 1.,
                                    "escape" => {
                                        if let Some((_, original)) =
                                            this.sliders_screen.stop_drag.take()
                                        {
                                            this.sliders_screen.stop_positions = original;
                                        }
                                        value
                                    }
                                    _ => return,
                                };
                                if event.keystroke.key != "escape" {
                                    this.sliders_screen.stop_positions[i] = next.clamp(0., 1.);
                                }
                                window.prevent_default();
                                cx.stop_propagation();
                                cx.notify();
                            },
                        ))
                    })),
            );
        v_flex()
            .gap_2()
            .w_full()
            .child(
                div()
                    .typography(fanta_gpui::atoms::TypographyToken::BodyLarge)
                    .child(format!(
                        "Gradient · stop {} at {:.0}%",
                        selected + 1,
                        positions[selected] * 100.
                    )),
            )
            .child(rail)
            .child(
                div()
                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child("Drag a stop, or use Left/Right and Home/End."),
            )
            .into_any_element()
    }
    pub(crate) fn render_slider_stops_story(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex().w_full().max_w(px(440.)).h_full().p_4().gap_4()
            .child(div().typography(fanta_gpui::atoms::TypographyToken::BodyLarge).child("Selected stops use our selection color. Unselected stops retain compact field chrome and show transparency."))
            .child(self.render_gradient_stop_demo(cx))
            .child(fanta_gpui::atoms::ui_button("slider-stops-disable").label(if self.sliders_screen.disabled {"Enable stops"}else{"Disable stops"}).xsmall().compact()
                .on_activate(cx.listener(|this,_,_,cx|{this.sliders_screen.disabled = !this.sliders_screen.disabled;cx.notify();})))
            .into_any_element()
    }
    pub(crate) fn render_sliders_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-sliders-reference",
            self.render_sliders_story(cx),
            self.sliders_screen.last_action.clone(),
            cx,
        )
    }
    pub(crate) fn render_slider_backgrounds_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-slider-backgrounds-reference",
            self.render_slider_backgrounds_story(cx),
            self.sliders_screen.last_action.clone(),
            cx,
        )
    }
    pub(crate) fn render_slider_handles_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-slider-handles-reference",
            self.render_slider_handles_story(cx),
            self.sliders_screen.last_action.clone(),
            cx,
        )
    }
    pub(crate) fn render_slider_stops_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-slider-stops-reference",
            self.render_slider_stops_story(cx),
            self.sliders_screen.last_action.clone(),
            cx,
        )
    }
}
