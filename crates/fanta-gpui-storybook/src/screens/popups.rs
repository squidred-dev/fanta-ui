//! The Popups & edge fades molecule story.
//!
//! An anchored-popup specimen built from `popup_surface` + `anchored_popup`
//! (trigger-attached, deferred, snapped inside the window per §12), and a
//! horizontally scrollable chip row whose clipped edges grow live gradient
//! fades through `track_horizontal_edge_fades` + `horizontal_fade_overlays`.
//! Both demos emit intents into the shared intent log.

use gpui::{Anchor, FocusHandle, point};

use crate::*;

use super::specimen::{specimen_card, specimen_row, specimen_rows, specimen_story_root};

/// How many chips the fade row renders; wide enough to overflow the
/// Narrow viewport preset so both fades appear.
pub(crate) const FADE_ROW_CHIP_COUNT: usize = 16;

pub(crate) struct PopupsScreen {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) popup_open: bool,
    pub(crate) fades: EdgeFades,
    pub(crate) fade_scroll_handle: ScrollHandle,
    pub(crate) last_action: SharedString,
}

impl PopupsScreen {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            popup_open: false,
            fades: EdgeFades::default(),
            fade_scroll_handle: ScrollHandle::new(),
            last_action: "Ready — open the anchored popup, or scroll the chip row".into(),
        }
    }

    pub(crate) fn set_popup_open(&mut self, open: bool, reason: &str) {
        if self.popup_open != open {
            self.popup_open = open;
            self.last_action = if open {
                format!("Opened the anchored popup ({reason})").into()
            } else {
                format!("Closed the anchored popup ({reason})").into()
            };
        }
    }
}

impl Storybook {
    fn render_popup_demo(&self, cx: &mut Context<Self>) -> AnyElement {
        let open = self.popups_screen.popup_open;
        let trigger = Button::new("popups-trigger")
            .debug_selector(|| "popups-trigger".to_owned())
            .outline()
            .small()
            .label(if open {
                "Close the popup"
            } else {
                "Open the popup"
            })
            .selected(open)
            .on_activate(cx.listener(|this, event: &ActivateEvent, _, cx| {
                let open = !this.popups_screen.popup_open;
                this.popups_screen.set_popup_open(
                    open,
                    if event.keyboard {
                        "trigger via Enter/Space"
                    } else {
                        "trigger via pointer"
                    },
                );
                cx.notify();
            }));

        let mut demo = v_flex().items_start().child(trigger);
        if open {
            let surface = popup_surface("popups-demo-popup", px(8.), cx)
                .debug_selector(|| "popups-demo-popup".to_owned())
                .w(px(280.))
                .max_h(px(220.))
                .p_3()
                .gap_2()
                .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                    this.popups_screen.set_popup_open(false, "clicked outside");
                    cx.notify();
                }))
                .child(div().text_sm().font_semibold().child("Anchored popup"))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(
                            "popup_surface owns the chrome; anchored_popup renders it \
                             in a deferred layer snapped inside the window with the \
                             8 px §12 margin. Hosts size real popups with popup_width \
                             and popup_height, which clamp to the window.",
                        ),
                )
                .child(
                    Button::new("popups-demo-intent")
                        .debug_selector(|| "popups-demo-intent".to_owned())
                        .outline()
                        .small()
                        .label("Emit a demo intent")
                        .on_activate(cx.listener(|this, event: &ActivateEvent, _, cx| {
                            this.popups_screen.last_action = format!(
                                "Demo intent from inside the popup · via {}",
                                if event.keyboard {
                                    "Enter/Space"
                                } else {
                                    "pointer"
                                }
                            )
                            .into();
                            cx.notify();
                        })),
                );
            demo = demo.child(anchored_popup(
                Anchor::TopLeft,
                point(px(0.), px(6.)),
                60,
                surface,
            ));
        }
        demo.into_any_element()
    }

    fn render_fade_demo(&self, cx: &mut Context<Self>) -> AnyElement {
        let background = cx.theme().background;
        let mut chip_row = h_flex()
            .id("popups-fade-row")
            .debug_selector(|| "popups-fade-row".to_owned())
            .w_full()
            .overflow_x_scroll()
            .track_scroll(&self.popups_screen.fade_scroll_handle)
            .gap_2()
            .p_2();
        for index in 0..FADE_ROW_CHIP_COUNT {
            chip_row = chip_row.child(
                div()
                    .flex_none()
                    .px_3()
                    .py_1()
                    .rounded(px(4.))
                    .border_1()
                    .border_color(cx.theme().border)
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("Chip {:02}", index + 1)),
            );
        }
        div()
            .id("popups-fade-wrapper")
            .debug_selector(|| "popups-fade-wrapper".to_owned())
            .relative()
            .w_full()
            .rounded(px(6.))
            .border_1()
            .border_color(cx.theme().border)
            .bg(background)
            .child(chip_row)
            .child(track_horizontal_edge_fades(
                cx.entity(),
                self.popups_screen.fade_scroll_handle.clone(),
                |story| story.popups_screen.fades,
                |story, fades| story.popups_screen.fades = fades,
            ))
            .children(horizontal_fade_overlays(
                self.popups_screen.fades,
                px(28.),
                background,
                "popups-demo",
            ))
            .into_any_element()
    }

    pub(crate) fn render_popups_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let popup_demo = self.render_popup_demo(cx);
        let fade_demo = self.render_fade_demo(cx);
        specimen_story_root("storybook-popups")
            .track_focus(&self.popups_screen.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if event.keystroke.key == "escape" && this.popups_screen.popup_open {
                    this.popups_screen.set_popup_open(false, "Escape");
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .child(specimen_card(
                "popups-anchored-demo",
                "popup_surface / anchored_popup · trigger-attached surface",
                "The trigger is a gpui Button registered through \
                 ButtonControlExt::on_activate, so pointer and Enter/Space share \
                 one handler. The open surface lives in a deferred layer anchored \
                 to the trigger and snaps inside the window on both axes.",
                specimen_rows(vec![specimen_row(
                    "popups-anchored-row",
                    "Open the popup, then drag the window small — the surface \
                     stays inside",
                    vec![popup_demo],
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "popups-edge-fades",
                "EdgeFades · horizontally scrollable row",
                "Rows that overflow_x_scroll clip silently, so the molecule layers \
                 pointer-transparent gradients over the clipped edges as the \
                 tracked scroll state changes.",
                specimen_rows(vec![specimen_row(
                    "popups-fades-row",
                    "Scroll the chips — or pick the Narrow preset — and watch the \
                     start/end fades",
                    vec![fade_demo],
                    cx,
                )]),
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn render_popups_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-popups",
            self.render_popups_story(cx),
            self.popups_screen.last_action.clone(),
            cx,
        )
    }
}
