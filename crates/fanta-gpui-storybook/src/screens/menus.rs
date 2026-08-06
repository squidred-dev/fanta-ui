//! The Menus molecule story.
//!
//! A live context-menu demo built from `menu_surface`, `menu_item`, and
//! `clamp_menu_origin`: right-click anywhere in the demo area (or use the
//! keyboard openers) and the menu clamps inside the area on both axes, so
//! anchoring near the right or bottom edge visibly flips it back inside.
//! The demo area's bounds come from the `track_bounds` atom; every item
//! emits a demo intent into the shared intent log.

use gpui::{Bounds, FocusHandle, Pixels, Point, point, size};

use crate::*;

use super::specimen::{specimen_card, specimen_row, specimen_rows, specimen_story_root};

/// Menu geometry shared by the render path and the clamping computation.
const MENU_WIDTH: f32 = 224.;
const MENU_ITEM_HEIGHT: f32 = 28.;
const MENU_MAX_HEIGHT: f32 = 240.;
/// Vertical padding the molecule's `py_2` adds around the item stack.
const MENU_VERTICAL_PADDING: f32 = 16.;

/// The demo menu's items.
pub(crate) const MENU_DEMO_ITEMS: [&str; 4] =
    ["Rename specimen", "Duplicate", "Copy link", "Delete"];

/// The demo menu's rendered size, as fed to `clamp_menu_origin`.
pub(crate) fn menu_demo_size() -> gpui::Size<gpui::Pixels> {
    size(
        px(MENU_WIDTH),
        px(
            (MENU_DEMO_ITEMS.len() as f32 * MENU_ITEM_HEIGHT + MENU_VERTICAL_PADDING)
                .min(MENU_MAX_HEIGHT),
        ),
    )
}

pub(crate) struct MenusScreen {
    pub(crate) demo_focus: FocusHandle,
    /// The demo area's window bounds, recorded by the `track_bounds` atom.
    pub(crate) demo_bounds: Bounds<Pixels>,
    /// The unclamped, demo-local anchor of the open menu.
    pub(crate) open_menu: Option<Point<Pixels>>,
    pub(crate) last_action: SharedString,
}

impl MenusScreen {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        Self {
            demo_focus: cx.focus_handle(),
            demo_bounds: Bounds::default(),
            open_menu: None,
            last_action: "Ready — right-click the demo area, or use a keyboard opener".into(),
        }
    }

    /// Opens the demo menu at a demo-local anchor; clamping happens at
    /// render time against the tracked demo bounds.
    pub(crate) fn open_menu_at(&mut self, anchor: Point<Pixels>, description: &str) {
        self.open_menu = Some(anchor);
        self.last_action = format!(
            "Opened the demo menu {description} at {:.0}, {:.0}",
            f32::from(anchor.x),
            f32::from(anchor.y)
        )
        .into();
    }

    pub(crate) fn close_menu(&mut self, reason: &str) {
        if self.open_menu.take().is_some() {
            self.last_action = format!("Closed the demo menu ({reason})").into();
        }
    }
}

impl Storybook {
    fn render_menus_demo_menu(&self, anchor: Point<Pixels>, cx: &mut Context<Self>) -> AnyElement {
        let menu_size = menu_demo_size();
        let origin = clamp_menu_origin(anchor, self.menus_screen.demo_bounds.size, menu_size);
        let mut surface = menu_surface(
            "menus-demo-menu",
            origin,
            menu_size.width,
            px(MENU_MAX_HEIGHT),
            px(8.),
            cx,
        )
        .debug_selector(|| "menus-demo-menu".to_owned())
        .on_mouse_down_out(cx.listener(|this, _, _, cx| {
            this.menus_screen.close_menu("clicked outside");
            cx.notify();
        }));
        for (index, label) in MENU_DEMO_ITEMS.into_iter().enumerate() {
            surface = surface.child(
                menu_item(
                    SharedString::from(format!("menus-demo-item-{index}")),
                    px(MENU_ITEM_HEIGHT),
                    cx,
                )
                .debug_selector(move || format!("menus-demo-item-{index}"))
                .on_activate(cx.listener(move |this, event: &ActivateEvent, _, cx| {
                    this.menus_screen.open_menu = None;
                    this.menus_screen.last_action = format!(
                        "Demo intent: {label} · via {}",
                        if event.keyboard {
                            "Enter/Space"
                        } else {
                            "pointer"
                        }
                    )
                    .into();
                    cx.notify();
                }))
                .child(truncating_label(label)),
            );
        }
        surface.into_any_element()
    }

    fn render_menus_demo_area(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .id("menus-demo-area")
            .debug_selector(|| "menus-demo-area".to_owned())
            .track_focus(&self.menus_screen.demo_focus)
            .relative()
            .w_full()
            .h(px(320.))
            .overflow_hidden()
            .rounded(px(8.))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().secondary.opacity(0.35))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    let local = point(
                        event.position.x - this.menus_screen.demo_bounds.origin.x,
                        event.position.y - this.menus_screen.demo_bounds.origin.y,
                    );
                    this.menus_screen.open_menu_at(local, "at the pointer");
                    cx.notify();
                }),
            )
            .child(
                // The center crosshair marks the demo surface as a click
                // target; the copy beneath it names the interaction.
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        v_flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .relative()
                                    .size(px(40.))
                                    .child(
                                        div()
                                            .absolute()
                                            .left(px(19.5))
                                            .top_0()
                                            .bottom_0()
                                            .w(px(1.))
                                            .bg(cx.theme().muted_foreground.opacity(0.4)),
                                    )
                                    .child(
                                        div()
                                            .absolute()
                                            .top(px(19.5))
                                            .left_0()
                                            .right_0()
                                            .h(px(1.))
                                            .bg(cx.theme().muted_foreground.opacity(0.4)),
                                    )
                                    .child(
                                        div()
                                            .absolute()
                                            .left(px(14.))
                                            .top(px(14.))
                                            .size(px(12.))
                                            .rounded_full()
                                            .border_1()
                                            .border_color(cx.theme().muted_foreground.opacity(0.4)),
                                    ),
                            )
                            .child(
                                div()
                                    .max_w(px(360.))
                                    .text_center()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(
                                        "Right-click anywhere — try the corners: the menu \
                                         clamps inside this frame on both axes",
                                    ),
                            ),
                    ),
            )
            .child(track_bounds(cx.entity(), |story, bounds| {
                story.menus_screen.demo_bounds = bounds;
            }))
            .when_some(self.menus_screen.open_menu, |area, anchor| {
                area.child(self.render_menus_demo_menu(anchor, cx))
            })
            .into_any_element()
    }

    pub(crate) fn render_menus_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let demo_size = menu_demo_size();
        let near_corner_anchor = point(
            (self.menus_screen.demo_bounds.size.width - px(24.)).max(px(0.)),
            (self.menus_screen.demo_bounds.size.height - px(24.)).max(px(0.)),
        );
        let openers = h_flex()
            .gap_2()
            .flex_wrap()
            .child(
                Button::new("menus-open-center")
                    .debug_selector(|| "menus-open-center".to_owned())
                    .outline()
                    .small()
                    .label("Open near center")
                    .on_activate(cx.listener(|this, _, _, cx| {
                        let bounds = this.menus_screen.demo_bounds;
                        this.menus_screen.open_menu_at(
                            point(bounds.size.width / 2., bounds.size.height / 2.),
                            "near the center",
                        );
                        cx.notify();
                    })),
            )
            .child(
                Button::new("menus-open-corner")
                    .debug_selector(|| "menus-open-corner".to_owned())
                    .outline()
                    .small()
                    .label("Open near the bottom-right corner")
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.menus_screen
                            .open_menu_at(near_corner_anchor, "near the corner");
                        cx.notify();
                    })),
            );

        specimen_story_root("storybook-menus")
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if event.keystroke.key == "escape" && this.menus_screen.open_menu.is_some() {
                    this.menus_screen.close_menu("Escape");
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .child(specimen_card(
                "menus-live-demo",
                "menu_surface / menu_item / clamp_menu_origin · live context menu",
                "The molecule owns the popover chrome, the row treatment, and \
                 two-axis clamping; the caller owns dismissal and the activation \
                 handlers. Every item routes through ControlExt::on_activate, so \
                 pointer clicks and Enter/Space on a focused item emit the same \
                 demo intent. The openers are gpui Buttons registered through \
                 ButtonControlExt::on_activate.",
                specimen_rows(vec![
                    specimen_row(
                        "menus-openers-row",
                        "Keyboard openers · Tab to one, then Enter or Space",
                        vec![openers.into_any_element()],
                        cx,
                    ),
                    specimen_row(
                        "menus-demo-row",
                        "Clamping demo area · anchor at the crosshair or any corner",
                        vec![self.render_menus_demo_area(cx)],
                        cx,
                    ),
                ]),
                cx,
            ))
            .child(specimen_card(
                "menus-clamping-copy",
                "Two-axis window clamping",
                "clamp_menu_origin clamps the anchor so the menu stays inside its \
                 container on both axes and floors at the top-left for oversized \
                 menus. §12 established the rule for toolbar popups; \
                 pointer-anchored context menus inherit it through this molecule, \
                 and the size passed to the clamp is the same one the surface \
                 renders at.",
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "Demo menu size: {:.0} × {:.0} px",
                        f32::from(demo_size.width),
                        f32::from(demo_size.height)
                    ))
                    .into_any_element(),
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn render_menus_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-menus",
            self.render_menus_story(cx),
            self.menus_screen.last_action.clone(),
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_menu_size_matches_its_item_stack_and_stays_clampable() {
        let menu = menu_demo_size();
        assert_eq!(menu.width, px(MENU_WIDTH));
        assert_eq!(
            menu.height,
            px(MENU_DEMO_ITEMS.len() as f32 * MENU_ITEM_HEIGHT + MENU_VERTICAL_PADDING)
        );
        // A corner anchor inside the 320 px demo area clamps on both axes.
        let clamped = clamp_menu_origin(point(px(890.), px(310.)), size(px(900.), px(320.)), menu);
        assert_eq!(clamped.x, px(900.) - menu.width);
        assert_eq!(clamped.y, px(320.) - menu.height);
    }
}
