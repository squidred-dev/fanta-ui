//! The shared story viewport harness.
//!
//! Generalizes the Design story's width resizer into chrome every Gallery
//! story mounts: width *and* height scrub handles with a live pixel
//! readout, plus per-story viewport preset chips sourced from the
//! registry. The harness owns viewport state only; the story inside the
//! surface reflows to whatever size the viewport currently has.

use crate::*;

use super::knobs;

pub(crate) const VIEWPORT_MIN_WIDTH: f32 = 200.;
pub(crate) const VIEWPORT_MAX_WIDTH: f32 = 2400.;
pub(crate) const VIEWPORT_MIN_HEIGHT: f32 = 160.;
pub(crate) const VIEWPORT_MAX_HEIGHT: f32 = 1600.;
const VIEWPORT_KEYBOARD_STEP: f32 = 8.;
const VIEWPORT_KEYBOARD_COARSE_STEP: f32 = 32.;
/// Padding of the gallery story canvas; the area probe subtracts it so a
/// fluid surface fills exactly the visible story area.
pub(crate) const GALLERY_CANVAS_PADDING: f32 = 16.;
/// Thickness of the width/height scrub handles beside the story surface.
pub(crate) const VIEWPORT_HANDLE_THICKNESS: f32 = 10.;
/// Two measurements within this distance count as unchanged, so prepaint
/// probes never notify in a loop over sub-pixel layout jitter.
const MEASURE_EPSILON: f32 = 0.5;

/// Records a prepaint measurement; returns whether it meaningfully changed.
fn record_dimensions(slot: &mut Option<(f32, f32)>, next: (f32, f32)) -> bool {
    if let Some((width, height)) = *slot
        && (width - next.0).abs() <= MEASURE_EPSILON
        && (height - next.1).abs() <= MEASURE_EPSILON
    {
        return false;
    }
    *slot = Some(next);
    true
}

/// One named viewport size a story registers as a natural component size.
pub(crate) struct ViewportPreset {
    pub(crate) label: &'static str,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl ViewportPreset {
    pub(crate) const fn new(label: &'static str, width: f32, height: f32) -> Self {
        Self {
            label,
            width,
            height,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ViewportAxis {
    Width,
    Height,
}

pub(crate) fn clamp_viewport_dimension(axis: ViewportAxis, value: f32) -> f32 {
    let (min, max, fallback) = match axis {
        ViewportAxis::Width => (VIEWPORT_MIN_WIDTH, VIEWPORT_MAX_WIDTH, VIEWPORT_MIN_WIDTH),
        ViewportAxis::Height => (
            VIEWPORT_MIN_HEIGHT,
            VIEWPORT_MAX_HEIGHT,
            VIEWPORT_MIN_HEIGHT,
        ),
    };
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        fallback
    }
}

/// Keyboard scrubbing on a viewport handle: the width handle answers
/// left/right, the height handle up/down, and both answer +/− (Shift for
/// the coarse step).
pub(crate) fn viewport_dimension_after_key(
    axis: ViewportAxis,
    value: f32,
    key: &str,
    shift: bool,
) -> Option<f32> {
    let step = if shift {
        VIEWPORT_KEYBOARD_COARSE_STEP
    } else {
        VIEWPORT_KEYBOARD_STEP
    };
    let delta = match (axis, key) {
        (ViewportAxis::Width, "left") | (ViewportAxis::Height, "up") => -step,
        (ViewportAxis::Width, "right") | (ViewportAxis::Height, "down") => step,
        (_, "-") | (_, "_") => -step,
        (_, "+") | (_, "=") => step,
        _ => return None,
    };
    Some(clamp_viewport_dimension(axis, value + delta))
}

/// A live scrub on one viewport handle. The handles sit on the right and
/// bottom edges, so dragging right or down grows the surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ViewportDrag {
    pub(crate) axis: ViewportAxis,
    start_pointer: f32,
    start_value: f32,
}

impl ViewportDrag {
    pub(crate) fn new(axis: ViewportAxis, start_pointer: f32, start_value: f32) -> Self {
        Self {
            axis,
            start_pointer,
            start_value: clamp_viewport_dimension(axis, start_value),
        }
    }

    pub(crate) fn value_at(self, pointer: f32) -> f32 {
        clamp_viewport_dimension(self.axis, self.start_value + pointer - self.start_pointer)
    }
}

/// The Gallery's current story viewport. Reset to the story's registered
/// default whenever a story is activated.
pub(crate) struct StoryViewport {
    pub(crate) width: f32,
    pub(crate) height: f32,
    /// Track the measured story area on both axes instead of holding a
    /// fixed size. Scrubbing or picking a preset pins the viewport to a
    /// fixed size.
    pub(crate) fluid: bool,
    pub(crate) drag: Option<ViewportDrag>,
    /// The story area available to a fluid surface, measured by the
    /// gallery's bounds probe during prepaint.
    pub(crate) available: Option<(f32, f32)>,
    /// The story's smallest registered viewport: a fluid surface never
    /// shrinks below it, so the canvas scrolls instead of clipping the
    /// component below its designed minimum.
    floor: (f32, f32),
}

impl StoryViewport {
    pub(crate) fn for_story(kind: StoryKind) -> Self {
        let descriptor = kind.descriptor();
        let (width, height) = descriptor.gallery_surface_size;
        Self {
            width: clamp_viewport_dimension(ViewportAxis::Width, width),
            height: clamp_viewport_dimension(ViewportAxis::Height, height),
            fluid: descriptor.gallery_fluid_width,
            drag: None,
            available: None,
            floor: descriptor.min_story_size(),
        }
    }

    /// Records the measured story area; returns whether it changed.
    pub(crate) fn record_available(&mut self, width: f32, height: f32) -> bool {
        record_dimensions(&mut self.available, (width, height))
    }

    /// The size the surface renders at. A pinned viewport holds its
    /// authored size; a fluid viewport fills the measured story area up
    /// to the scrub handles, floored at the story's smallest registered
    /// viewport (falling back to the registered size before the first
    /// measurement).
    pub(crate) fn surface_size(&self) -> (f32, f32) {
        if !self.fluid {
            return (self.width, self.height);
        }
        match self.available {
            Some((width, height)) => (
                (width - VIEWPORT_HANDLE_THICKNESS).max(self.floor.0),
                (height - VIEWPORT_HANDLE_THICKNESS).max(self.floor.1),
            ),
            None => (self.width, self.height),
        }
    }

    /// A fluid surface has no authored size to scrub from; pin it at the
    /// size it renders at so an interaction continues from the screen.
    fn pin_fluid(&mut self) {
        if !self.fluid {
            return;
        }
        let (width, height) = self.surface_size();
        self.width = clamp_viewport_dimension(ViewportAxis::Width, width);
        self.height = clamp_viewport_dimension(ViewportAxis::Height, height);
        self.fluid = false;
    }

    pub(crate) fn apply_preset(&mut self, width: f32, height: f32) {
        self.drag = None;
        self.fluid = false;
        self.width = clamp_viewport_dimension(ViewportAxis::Width, width);
        self.height = clamp_viewport_dimension(ViewportAxis::Height, height);
    }

    pub(crate) fn set_fluid(&mut self) {
        self.drag = None;
        self.fluid = true;
    }

    pub(crate) fn matches_preset(&self, preset: &ViewportPreset) -> bool {
        !self.fluid
            && (self.width - preset.width).abs() < f32::EPSILON
            && (self.height - preset.height).abs() < f32::EPSILON
    }

    pub(crate) fn begin_drag(&mut self, axis: ViewportAxis, pointer: f32) {
        // Scrubbing either handle pins a fluid viewport at its rendered
        // size first, so the scrub continues from what is on screen.
        self.pin_fluid();
        let value = match axis {
            ViewportAxis::Width => self.width,
            ViewportAxis::Height => self.height,
        };
        self.drag = Some(ViewportDrag::new(axis, pointer, value));
    }

    /// Applies a drag update; returns whether the viewport changed.
    pub(crate) fn update_drag(&mut self, pointer_x: f32, pointer_y: f32) -> bool {
        let Some(drag) = self.drag else {
            return false;
        };
        let (target, pointer) = match drag.axis {
            ViewportAxis::Width => (&mut self.width, pointer_x),
            ViewportAxis::Height => (&mut self.height, pointer_y),
        };
        let value = drag.value_at(pointer);
        if (*target - value).abs() < f32::EPSILON {
            return false;
        }
        *target = value;
        true
    }

    pub(crate) fn finish_drag(&mut self) -> bool {
        self.drag.take().is_some()
    }

    /// Applies a keyboard scrub on one handle; returns whether it handled
    /// the key.
    pub(crate) fn apply_key(&mut self, axis: ViewportAxis, key: &str, shift: bool) -> bool {
        // A fluid viewport steps from its rendered size, so pinning via
        // the keyboard is as continuous as pinning via a drag.
        let (surface_width, surface_height) = self.surface_size();
        let value = match axis {
            ViewportAxis::Width => surface_width,
            ViewportAxis::Height => surface_height,
        };
        let Some(value) = viewport_dimension_after_key(axis, value, key, shift) else {
            return false;
        };
        self.pin_fluid();
        match axis {
            ViewportAxis::Width => self.width = value,
            ViewportAxis::Height => self.height = value,
        }
        self.drag = None;
        true
    }

    fn readout(&self) -> String {
        let (width, height) = self.surface_size();
        if self.fluid {
            format!("fluid · {width:.0} × {height:.0} px")
        } else {
            format!("{width:.0} × {height:.0} px")
        }
    }
}

impl Storybook {
    /// An absolute prepaint probe recording the visible story area (the
    /// gallery canvas region minus its padding) for fluid surfaces. Layer
    /// it as the last child of a `relative()` wrapper around the canvas.
    pub(crate) fn render_story_area_probe(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        canvas(
            move |bounds, _, app| {
                let inset = 2. * GALLERY_CANVAS_PADDING;
                let changed = entity.update(app, |storybook, _| {
                    storybook.story_viewport.record_available(
                        (f32::from(bounds.size.width) - inset).max(0.),
                        (f32::from(bounds.size.height) - inset).max(0.),
                    )
                });
                if changed {
                    // Notifies issued mid-draw are swallowed by the window
                    // invalidator, so schedule the re-render after this
                    // draw completes instead.
                    app.defer(move |app| {
                        entity.update(app, |_, cx| cx.notify());
                    });
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
    }

    fn render_viewport_handle(&self, axis: ViewportAxis, cx: &mut Context<Self>) -> AnyElement {
        let active = self
            .story_viewport
            .drag
            .is_some_and(|drag| drag.axis == axis);
        let (id, selector, tooltip) = match axis {
            ViewportAxis::Width => (
                "story-viewport-width-handle",
                "story-viewport-width-handle",
                format!(
                    "Resize the story viewport width · {} · drag or use ←/→ and +/− (Shift for 32 px)",
                    self.story_viewport.readout()
                ),
            ),
            ViewportAxis::Height => (
                "story-viewport-height-handle",
                "story-viewport-height-handle",
                format!(
                    "Resize the story viewport height · {} · drag or use ↑/↓ and +/− (Shift for 32 px)",
                    self.story_viewport.readout()
                ),
            ),
        };
        let grip = match axis {
            ViewportAxis::Width => div().w(px(2.)).h(px(36.)),
            ViewportAxis::Height => div().w(px(36.)).h(px(2.)),
        };
        Button::new(id)
            .debug_selector(move || selector.to_owned())
            .tooltip(tooltip)
            .xsmall()
            .compact()
            .selected(active)
            .p_0()
            .map(|handle| match axis {
                ViewportAxis::Width => handle
                    .w(px(VIEWPORT_HANDLE_THICKNESS))
                    .h_full()
                    .cursor_col_resize(),
                ViewportAxis::Height => handle
                    .h(px(VIEWPORT_HANDLE_THICKNESS))
                    .w_full()
                    .cursor_row_resize(),
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                    let pointer = match axis {
                        ViewportAxis::Width => f32::from(event.position.x),
                        ViewportAxis::Height => f32::from(event.position.y),
                    };
                    this.story_viewport.begin_drag(axis, pointer);
                    cx.notify();
                }),
            )
            .on_key_down(cx.listener(move |this, event: &KeyDownEvent, _, cx| {
                if this.story_viewport.apply_key(
                    axis,
                    event.keystroke.key.as_str(),
                    event.keystroke.modifiers.shift,
                ) {
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .child(grip.rounded_full().bg(if active {
                cx.theme().selection
            } else {
                cx.theme().border
            }))
            .into_any_element()
    }

    /// The preset-chip and readout row rendered by the gallery shell just
    /// above the story canvas, so it never eats into the measured area a
    /// fluid surface fills.
    pub(crate) fn render_viewport_meta_row(&self, cx: &mut Context<Self>) -> AnyElement {
        let descriptor = self.active_story.descriptor();
        let mut chips = h_flex().gap_2().flex_wrap();
        if descriptor.gallery_fluid_width {
            chips = chips.child(knobs::knob_chip(
                "story-viewport-fluid".into(),
                "Fluid".into(),
                self.story_viewport.fluid,
                |this, _, cx| {
                    this.story_viewport.set_fluid();
                    cx.notify();
                },
                cx,
            ));
        }
        for (index, preset) in descriptor.viewport_presets.iter().enumerate() {
            let (width, height) = (preset.width, preset.height);
            chips = chips.child(knobs::knob_chip(
                SharedString::from(format!("story-viewport-preset-{index}")),
                SharedString::from(format!(
                    "{} · {:.0}×{:.0}",
                    preset.label, preset.width, preset.height
                )),
                self.story_viewport.matches_preset(preset),
                move |this, _, cx| {
                    this.story_viewport.apply_preset(width, height);
                    cx.notify();
                },
                cx,
            ));
        }

        h_flex()
            .w_full()
            .flex_none()
            .items_center()
            .justify_between()
            .gap_4()
            .pb_2()
            .child(chips)
            .child(
                div()
                    .id("story-viewport-readout")
                    .debug_selector(|| "story-viewport-readout".to_owned())
                    .flex_none()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.story_viewport.readout()),
            )
            .into_any_element()
    }

    /// The Gallery story surface: the sized story surface plus the
    /// width/height scrub handles. A fluid viewport derives its size from
    /// the measured story area every frame (both axes); a pinned viewport
    /// holds its authored size and the canvas scrolls to whatever does
    /// not fit.
    pub(crate) fn render_story_viewport(&self, cx: &mut Context<Self>) -> AnyElement {
        let (width, height) = self.story_viewport.surface_size();

        let surface = div()
            .id("storybook-gallery-story-surface")
            .debug_selector(|| "storybook-gallery-story-surface".to_owned())
            .relative()
            .flex_none()
            .w(px(width))
            .h(px(height))
            .overflow_hidden()
            .bg(cx.theme().background)
            .child(self.render_gallery_story_component(cx));

        v_flex()
            .id("storybook-story-viewport")
            .debug_selector(|| "storybook-story-viewport".to_owned())
            .relative()
            .w_full()
            .child(
                h_flex()
                    .w_full()
                    .items_start()
                    .child(
                        v_flex()
                            .flex_none()
                            .child(surface)
                            .child(self.render_viewport_handle(ViewportAxis::Height, cx)),
                    )
                    .child(
                        div()
                            .flex_none()
                            .h(px(height))
                            .child(self.render_viewport_handle(ViewportAxis::Width, cx)),
                    ),
            )
            // While a handle is being scrubbed, an occluding shield keeps
            // the drag events away from the story's own hitboxes.
            .when(self.story_viewport.drag.is_some(), |wrapper| {
                wrapper.child(
                    div()
                        .id("story-viewport-drag-shield")
                        .absolute()
                        .inset_0()
                        .occlude()
                        .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                            if event.dragging()
                                && this.story_viewport.update_drag(
                                    f32::from(event.position.x),
                                    f32::from(event.position.y),
                                )
                            {
                                cx.notify();
                            }
                        }))
                        .on_mouse_up(
                            MouseButton::Left,
                            cx.listener(|this, _: &MouseUpEvent, _, cx| {
                                if this.story_viewport.finish_drag() {
                                    cx.notify();
                                }
                            }),
                        )
                        .on_mouse_up_out(
                            MouseButton::Left,
                            cx.listener(|this, _: &MouseUpEvent, _, cx| {
                                if this.story_viewport.finish_drag() {
                                    cx.notify();
                                }
                            }),
                        ),
                )
            })
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_dimensions_clamp_to_the_harness_bounds() {
        assert_eq!(
            clamp_viewport_dimension(ViewportAxis::Width, 100.),
            VIEWPORT_MIN_WIDTH
        );
        assert_eq!(
            clamp_viewport_dimension(ViewportAxis::Width, 9_000.),
            VIEWPORT_MAX_WIDTH
        );
        assert_eq!(clamp_viewport_dimension(ViewportAxis::Width, 337.), 337.);
        assert_eq!(
            clamp_viewport_dimension(ViewportAxis::Height, 10.),
            VIEWPORT_MIN_HEIGHT
        );
        assert_eq!(
            clamp_viewport_dimension(ViewportAxis::Height, f32::NAN),
            VIEWPORT_MIN_HEIGHT
        );
    }

    #[test]
    fn viewport_keyboard_scrub_is_axis_specific_and_clamped() {
        assert_eq!(
            viewport_dimension_after_key(ViewportAxis::Width, 337., "right", false),
            Some(345.)
        );
        assert_eq!(
            viewport_dimension_after_key(ViewportAxis::Width, 337., "left", true),
            Some(305.)
        );
        assert_eq!(
            viewport_dimension_after_key(ViewportAxis::Width, 337., "up", false),
            None,
            "the width handle must ignore the height handle's keys"
        );
        assert_eq!(
            viewport_dimension_after_key(ViewportAxis::Height, 716., "down", false),
            Some(724.)
        );
        assert_eq!(
            viewport_dimension_after_key(ViewportAxis::Height, 716., "left", false),
            None
        );
        assert_eq!(
            viewport_dimension_after_key(ViewportAxis::Height, 716., "+", true),
            Some(748.)
        );
        assert_eq!(
            viewport_dimension_after_key(ViewportAxis::Width, VIEWPORT_MIN_WIDTH, "left", true),
            Some(VIEWPORT_MIN_WIDTH)
        );
    }

    #[test]
    fn viewport_drag_grows_toward_the_pointer_and_clamps() {
        let drag = ViewportDrag::new(ViewportAxis::Width, 500., 337.);
        assert_eq!(drag.value_at(564.), 401.);
        assert_eq!(drag.value_at(420.), 257.);
        assert_eq!(drag.value_at(-9_000.), VIEWPORT_MIN_WIDTH);
        assert_eq!(drag.value_at(90_000.), VIEWPORT_MAX_WIDTH);

        let drag = ViewportDrag::new(ViewportAxis::Height, 600., 716.);
        assert_eq!(drag.value_at(632.), 748.);
    }

    #[test]
    fn story_viewports_seed_from_the_registry_and_pin_on_interaction() {
        for descriptor in crate::screens::registry() {
            let viewport = StoryViewport::for_story(descriptor.kind);
            assert_eq!(
                viewport.fluid, descriptor.gallery_fluid_width,
                "{} must seed its registered fluid mode",
                descriptor.title
            );
            assert_eq!(
                viewport.width,
                clamp_viewport_dimension(ViewportAxis::Width, descriptor.gallery_surface_size.0)
            );
            assert_eq!(
                viewport.height,
                clamp_viewport_dimension(ViewportAxis::Height, descriptor.gallery_surface_size.1)
            );
        }

        let mut viewport = StoryViewport::for_story(StoryKind::PseudoEditor);
        assert!(viewport.fluid);
        viewport.apply_preset(1240., 820.);
        assert!(!viewport.fluid);
        assert_eq!((viewport.width, viewport.height), (1240., 820.));
        viewport.set_fluid();
        assert!(viewport.fluid);

        // A keyboard scrub on either axis pins a fluid viewport. Without
        // a measurement yet, it steps from the stored size.
        viewport.apply_key(ViewportAxis::Height, "down", false);
        assert!(!viewport.fluid, "a height scrub must also pin fluid mode");
        assert_eq!(viewport.height, 828.);
        viewport.apply_key(ViewportAxis::Width, "right", false);
        assert_eq!(viewport.width, 1248.);
    }

    #[test]
    fn fluid_scrubs_pin_the_viewport_at_its_rendered_size() {
        let mut viewport = StoryViewport::for_story(StoryKind::PseudoEditor);
        assert!(viewport.fluid);
        assert!(viewport.record_available(1310., 742.));
        assert!(
            !viewport.record_available(1310.2, 741.9),
            "sub-pixel jitter must not count as a new measurement"
        );
        assert_eq!(viewport.surface_size(), (1300., 732.));

        // Dragging a handle pins both axes at the rendered size, then
        // scrubs the grabbed axis from there.
        viewport.begin_drag(ViewportAxis::Height, 600.);
        assert!(!viewport.fluid);
        assert_eq!((viewport.width, viewport.height), (1300., 732.));
        assert!(viewport.update_drag(0., 632.));
        assert_eq!(viewport.height, 764.);
        assert_eq!(viewport.width, 1300.);

        // A keyboard scrub pins from the rendered size the same way.
        let mut viewport = StoryViewport::for_story(StoryKind::PseudoEditor);
        viewport.record_available(1310., 742.);
        assert!(viewport.apply_key(ViewportAxis::Width, "right", false));
        assert!(!viewport.fluid);
        assert_eq!((viewport.width, viewport.height), (1308., 732.));

        // An unhandled key must leave the fluid mode untouched.
        let mut viewport = StoryViewport::for_story(StoryKind::PseudoEditor);
        viewport.record_available(1310., 742.);
        assert!(!viewport.apply_key(ViewportAxis::Width, "up", false));
        assert!(viewport.fluid);
    }

    #[test]
    fn fluid_surface_sizes_and_readouts_track_the_measured_area() {
        let mut viewport = StoryViewport::for_story(StoryKind::PseudoEditor);
        assert!(viewport.fluid);
        // Before the first measurement the surface falls back to the
        // story's registered size.
        assert_eq!(viewport.surface_size(), (1440., 900.));
        assert_eq!(viewport.readout(), "fluid · 1440 × 900 px");

        // Fluid tracks the measured area on both axes, minus the handles.
        assert!(viewport.record_available(1188., 640.));
        assert_eq!(viewport.surface_size(), (1178., 630.));
        assert_eq!(viewport.readout(), "fluid · 1178 × 630 px");

        // A degenerate story area floors at the story's smallest
        // registered viewport instead of crushing the component; the
        // canvas scrolls to the rest.
        let (floor_width, floor_height) = StoryKind::PseudoEditor.descriptor().min_story_size();
        assert!(viewport.record_available(600., 120.));
        assert_eq!(
            viewport.surface_size(),
            (590f32.max(floor_width), 110f32.max(floor_height))
        );

        // Presets pin an authored size; fluid derivation stops.
        viewport.apply_preset(900., 620.);
        assert_eq!(viewport.surface_size(), (900., 620.));
        assert_eq!(viewport.readout(), "900 × 620 px");
    }

    #[test]
    fn story_viewport_drags_update_only_their_axis() {
        let mut viewport = StoryViewport::for_story(StoryKind::Pages);
        // Pages seeds fluid like every story now; grabbing a handle pins
        // it at the registered size before the scrub applies.
        assert!(viewport.fluid);
        viewport.begin_drag(ViewportAxis::Width, 400.);
        assert!(!viewport.fluid);
        assert!(viewport.update_drag(432., 999.));
        assert_eq!(viewport.width, 369.);
        assert_eq!(viewport.height, 716.);
        assert!(!viewport.update_drag(432., 0.), "same position is a no-op");
        assert!(viewport.finish_drag());
        assert!(!viewport.finish_drag());

        viewport.begin_drag(ViewportAxis::Height, 100.);
        assert!(viewport.update_drag(0., 60.));
        assert_eq!(viewport.height, 676.);
        assert_eq!(viewport.width, 369.);
    }
}
