//! The Inspector overlays molecule story.
//!
//! An anchored popover and an anchored context menu, both mounted through one
//! `InspectorOverlayPlacement`, with every close routed through
//! `InspectorOverlayDismissIntent`. The dismissal log names the
//! `InspectorOverlayDismissCause` that fired — Escape or OutsideClick — and a
//! commit is shown for what it is: an owner close that carries no cause at
//! all. Each entry also names the control focus landed on once the deferred
//! restore settled, which is the half of the contract a reader cannot see by
//! looking at the surfaces.

use fanta_gpui::atoms::TypographyExt as _;
use gpui::{Anchor, FocusHandle, point};

use crate::*;

use fanta_gpui::prelude::{
    InspectorFieldAccess, InspectorMetrics, InspectorOverlayDismissCause,
    InspectorOverlayDismissIntent, InspectorOverlayFocusTarget, InspectorOverlayPlacement,
    inspector_action_button, inspector_anchored_menu, inspector_anchored_menu_surface,
    inspector_anchored_overlay, inspector_menu_item, inspector_popover_surface,
};

use super::knobs::{self, KnobOption};
use super::spec::{NamedState, state_matrix};
use super::specimen::{specimen_card, specimen_row, specimen_rows, specimen_story_root};

/// Width of the popover specimen.
const POPOVER_WIDTH: f32 = 268.;
/// Width and height ceiling of the context-menu specimen.
const MENU_WIDTH: f32 = 208.;
const MENU_MAX_HEIGHT: f32 = 220.;
/// Deferred layer priority shared by both surfaces.
const OVERLAY_PRIORITY: usize = 60;
/// Height of the demo area the triggers and overlays live in.
const DEMO_AREA_HEIGHT: f32 = 300.;
/// How many dismissals the log keeps, newest first.
const DISMISSAL_LOG_LIMIT: usize = 6;
/// Placeholder an entry carries until the deferred focus restore settles.
const RESTORING: &str = "restoring…";

/// The committing rows of the context-menu specimen.
pub(crate) const MENU_COMMIT_ITEMS: [&str; 3] =
    ["Rename layer", "Duplicate layer", "Copy link to layer"];

/// The single placement both surfaces are mounted with: menus and popovers
/// deliberately share window snapping and deferred priority, and differ only
/// in content anatomy.
pub(crate) fn overlay_placement() -> InspectorOverlayPlacement {
    InspectorOverlayPlacement::new(Anchor::TopLeft, point(px(0.), px(6.)), OVERLAY_PRIORITY)
}

/// The two anchored surfaces the story mounts; also the overlay identity
/// carried by every [`InspectorOverlayDismissIntent`] it builds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OverlaySurface {
    Popover,
    Menu,
}

impl OverlaySurface {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Popover => "Popover",
            Self::Menu => "Context menu",
        }
    }
}

/// The story's named states: nothing open, the popover open, the menu open.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OverlaysNamedState {
    Closed,
    PopoverOpen,
    MenuOpen,
}

impl OverlaysNamedState {
    /// The surface this state has open, if any.
    pub(crate) const fn surface(self) -> Option<OverlaySurface> {
        match self {
            Self::Closed => None,
            Self::PopoverOpen => Some(OverlaySurface::Popover),
            Self::MenuOpen => Some(OverlaySurface::Menu),
        }
    }

    pub(crate) const fn for_surface(surface: OverlaySurface) -> Self {
        match surface {
            OverlaySurface::Popover => Self::PopoverOpen,
            OverlaySurface::Menu => Self::MenuOpen,
        }
    }
}

impl NamedState for OverlaysNamedState {
    const ALL: &'static [Self] = &[Self::Closed, Self::PopoverOpen, Self::MenuOpen];

    fn label(self) -> &'static str {
        match self {
            Self::Closed => "Closed",
            Self::PopoverOpen => "Popover open",
            Self::MenuOpen => "Menu open",
        }
    }
}

/// The three ways a reader can close an open surface.
///
/// Only two of them are dismissals: [`InspectorOverlayDismissCause`] has
/// exactly one variant for Escape and one for an outside click. A commit is
/// the owner closing its own surface, so no cause exists to report and the
/// owner restores focus from the retained target itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OverlayDismissRoute {
    Escape,
    OutsideClick,
    Commit,
}

impl OverlayDismissRoute {
    /// The cause this route reports to the retained owner.
    pub(crate) const fn cause(self) -> Option<InspectorOverlayDismissCause> {
        match self {
            Self::Escape => Some(InspectorOverlayDismissCause::Escape),
            Self::OutsideClick => Some(InspectorOverlayDismissCause::OutsideClick),
            Self::Commit => None,
        }
    }

    /// What the log prints in the cause column.
    pub(crate) const fn cause_label(self) -> &'static str {
        match self {
            Self::Escape => "InspectorOverlayDismissCause::Escape",
            Self::OutsideClick => "InspectorOverlayDismissCause::OutsideClick",
            Self::Commit => "no cause — the owner closed it",
        }
    }

    /// The short cause spelling the state matrix prints in a 96 px cell.
    pub(crate) const fn cause_badge(self) -> &'static str {
        match self {
            Self::Escape => "Escape",
            Self::OutsideClick => "OutsideClick",
            Self::Commit => "no cause",
        }
    }

    /// How a reader fires this route.
    pub(crate) const fn how(self) -> &'static str {
        match self {
            Self::Escape => "press Esc",
            Self::OutsideClick => "click outside the surface",
            Self::Commit => "activate a row or Apply",
        }
    }
}

impl NamedState for OverlayDismissRoute {
    const ALL: &'static [Self] = &[Self::Escape, Self::OutsideClick, Self::Commit];

    fn label(self) -> &'static str {
        match self {
            Self::Escape => "Escape",
            Self::OutsideClick => "Outside click",
            Self::Commit => "Commit",
        }
    }
}

/// One closed surface, as the log prints it.
pub(crate) struct OverlayDismissalRecord {
    pub(crate) surface: OverlaySurface,
    pub(crate) route: OverlayDismissRoute,
    /// What the reader did, in their own terms.
    pub(crate) detail: SharedString,
    pub(crate) cause: Option<InspectorOverlayDismissCause>,
    /// The opening generation the intent was tied to.
    pub(crate) generation: u64,
    /// What `restore_focus` reported: false when no target was retained.
    pub(crate) restored: bool,
    pub(crate) focus_before: SharedString,
    /// Filled in once the deferred restore has actually run.
    pub(crate) focus_after: SharedString,
}

pub(crate) struct OverlaysScreen {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) popover_trigger: FocusHandle,
    pub(crate) menu_trigger: FocusHandle,
    pub(crate) neighbor: FocusHandle,
    pub(crate) popover_focus: FocusHandle,
    pub(crate) menu_focus: FocusHandle,
    pub(crate) named_state: OverlaysNamedState,
    /// Bumped on every open so each intent names one opening.
    pub(crate) generation: u64,
    /// Knob: whether the owner retains a focus target to restore.
    pub(crate) retain_focus_target: bool,
    /// The control focus was last observed on, named for the reader.
    pub(crate) focused_control: SharedString,
    pub(crate) log: Vec<OverlayDismissalRecord>,
    pub(crate) last_action: SharedString,
}

impl OverlaysScreen {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            popover_trigger: cx.focus_handle(),
            menu_trigger: cx.focus_handle(),
            neighbor: cx.focus_handle(),
            popover_focus: cx.focus_handle(),
            menu_focus: cx.focus_handle(),
            named_state: OverlaysNamedState::Closed,
            generation: 0,
            retain_focus_target: true,
            focused_control: "not observed yet".into(),
            log: Vec::new(),
            last_action: "Ready — open a surface, then close it three different ways".into(),
        }
    }

    /// The trigger an overlay must hand focus back to.
    fn trigger(&self, surface: OverlaySurface) -> &FocusHandle {
        match surface {
            OverlaySurface::Popover => &self.popover_trigger,
            OverlaySurface::Menu => &self.menu_trigger,
        }
    }

    /// Names the control that currently holds focus, so the log can show
    /// where the deferred restore actually landed.
    fn focused_control_name(&self, window: &Window) -> SharedString {
        for (handle, name) in [
            (&self.popover_trigger, "Popover trigger"),
            (&self.menu_trigger, "Context-menu trigger"),
            (&self.neighbor, "Neighbor control"),
            (&self.popover_focus, "Popover surface"),
            (&self.menu_focus, "Context menu · first row"),
            (&self.focus_handle, "Story root"),
        ] {
            if handle.is_focused(window) {
                return name.into();
            }
        }
        "an unnamed control".into()
    }

    /// Reads focus back after the current frame's deferred work — including
    /// the overlay's own deferred focus restore — has run.
    fn observe_focus(&self, fills_record: bool, window: &mut Window, cx: &mut Context<Storybook>) {
        cx.defer_in(window, move |story, window, cx| {
            let focused = story.overlays_screen.focused_control_name(window);
            story.overlays_screen.focused_control = focused.clone();
            if fills_record
                && let Some(record) = story.overlays_screen.log.first_mut()
                && record.focus_after.as_ref() == RESTORING
            {
                record.focus_after = focused;
            }
            cx.notify();
        });
    }

    /// Opens one surface and moves focus into it, so the story reaches an
    /// open state without a pointer.
    pub(crate) fn open(
        &mut self,
        surface: OverlaySurface,
        detail: &str,
        window: &mut Window,
        cx: &mut Context<Storybook>,
    ) {
        self.generation += 1;
        self.named_state = OverlaysNamedState::for_surface(surface);
        match surface {
            OverlaySurface::Popover => self.popover_focus.focus(window, cx),
            OverlaySurface::Menu => self.menu_focus.focus(window, cx),
        }
        self.last_action = format!(
            "Opened the {} · {detail} · generation {}",
            surface.label().to_ascii_lowercase(),
            self.generation
        )
        .into();
        self.observe_focus(false, window, cx);
        cx.notify();
    }

    /// Closes one surface through `route`, recording the cause the retained
    /// owner was handed and where focus ended up.
    ///
    /// This is the shape `overlay.rs` documents: the owner clears its own
    /// open state first, then asks the intent to restore focus.
    pub(crate) fn dismiss(
        &mut self,
        surface: OverlaySurface,
        route: OverlayDismissRoute,
        detail: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Storybook>,
    ) {
        if self.named_state.surface() != Some(surface) {
            return;
        }
        let focus_before = self.focused_control_name(window);
        let generation = self.generation;
        let focus_return = self
            .retain_focus_target
            .then(|| InspectorOverlayFocusTarget::new(self.trigger(surface).clone()));
        self.named_state = OverlaysNamedState::Closed;

        let mut cause = None;
        let restored = match route.cause() {
            Some(reported) => {
                let intent = InspectorOverlayDismissIntent::new_versioned(
                    surface,
                    reported,
                    focus_return,
                    generation,
                );
                let restored = intent.restore_focus(window, cx);
                // Read back off the intent, so the log reports what the
                // retained owner was actually handed.
                cause = Some(intent.cause());
                restored
            }
            // A commit is not a dismissal, so there is no intent to build and
            // no cause to report; the owner restores focus itself.
            None => match focus_return {
                Some(target) => {
                    target.restore(window, cx);
                    true
                }
                None => false,
            },
        };

        self.log.insert(
            0,
            OverlayDismissalRecord {
                surface,
                route,
                detail: detail.into(),
                cause,
                generation,
                restored,
                focus_before,
                focus_after: RESTORING.into(),
            },
        );
        self.log.truncate(DISMISSAL_LOG_LIMIT);
        self.last_action = format!(
            "{} closed · {} · {}",
            surface.label(),
            route.label(),
            route.cause_label()
        )
        .into();
        self.observe_focus(true, window, cx);
        cx.notify();
    }

    /// Drives the story from its named state alone.
    pub(crate) fn apply_named_state(
        &mut self,
        state: OverlaysNamedState,
        window: &mut Window,
        cx: &mut Context<Storybook>,
    ) {
        match state.surface() {
            Some(surface) => self.open(surface, "named-state knob", window, cx),
            None => {
                // Closing from the knob is an owner close, exactly like a
                // commit: nothing hands the owner a dismiss cause.
                if let Some(open) = self.named_state.surface() {
                    self.dismiss(open, OverlayDismissRoute::Commit, "knob", window, cx);
                }
            }
        }
    }
}

impl Storybook {
    fn render_overlays_popover(&self, cx: &mut Context<Self>) -> AnyElement {
        let metrics = InspectorMetrics::default();
        let surface = inspector_popover_surface("overlays-popover-surface", metrics, cx)
            .debug_selector(|| "overlays-popover-surface".to_owned())
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(0)
            .track_focus(&self.overlays_screen.popover_focus)
            .w(px(POPOVER_WIDTH))
            .p_3()
            .gap_2()
            .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, window, cx| {
                this.overlays_screen.dismiss(
                    OverlaySurface::Popover,
                    OverlayDismissRoute::OutsideClick,
                    "pointer landed outside the surface",
                    window,
                    cx,
                );
            }))
            .child(
                div()
                    .typography(fanta_gpui::atoms::TypographyToken::BodyLarge)
                    .font_semibold()
                    .child("Corner radius"),
            )
            .child(
                div()
                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child(
                        "inspector_popover_surface owns this chrome; the placement \
                         comes from the shared InspectorOverlayPlacement, so the \
                         surface snaps inside the window exactly like the menu.",
                    ),
            )
            .child(
                inspector_action_button(
                    "overlays-popover-commit",
                    &InspectorFieldAccess::Editable,
                    metrics,
                    cx,
                )
                .debug_selector(|| "overlays-popover-commit".to_owned())
                .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                .on_activate(cx.listener(|this, event: &ActivateEvent, window, cx| {
                    this.overlays_screen.dismiss(
                        OverlaySurface::Popover,
                        OverlayDismissRoute::Commit,
                        if event.keyboard {
                            "Apply via Enter/Space"
                        } else {
                            "Apply via pointer"
                        },
                        window,
                        cx,
                    );
                }))
                .child("Apply"),
            );
        inspector_anchored_overlay(overlay_placement(), surface)
    }

    fn render_overlays_menu(&self, cx: &mut Context<Self>) -> AnyElement {
        let metrics = InspectorMetrics::default();
        let mut surface = inspector_anchored_menu_surface(
            "overlays-menu-surface",
            px(MENU_WIDTH),
            px(MENU_MAX_HEIGHT),
            metrics,
            cx,
        )
        .debug_selector(|| "overlays-menu-surface".to_owned())
        .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, window, cx| {
            this.overlays_screen.dismiss(
                OverlaySurface::Menu,
                OverlayDismissRoute::OutsideClick,
                "pointer landed outside the surface",
                window,
                cx,
            );
        }));
        for (index, label) in MENU_COMMIT_ITEMS.into_iter().enumerate() {
            let row = inspector_menu_item(
                SharedString::from(format!("overlays-menu-item-{index}")),
                &InspectorFieldAccess::Editable,
                metrics,
                cx,
            )
            .debug_selector(move || format!("overlays-menu-item-{index}"))
            .on_activate(cx.listener(move |this, _: &ActivateEvent, window, cx| {
                this.overlays_screen.dismiss(
                    OverlaySurface::Menu,
                    OverlayDismissRoute::Commit,
                    label,
                    window,
                    cx,
                );
            }))
            .child(truncating_label(label));
            // The first row carries menu focus continuity, so the menu is
            // reachable — and dismissable — without a pointer.
            surface = surface.child(if index == 0 {
                row.track_focus(&self.overlays_screen.menu_focus)
                    .into_any_element()
            } else {
                row.into_any_element()
            });
        }
        surface = surface.child(
            inspector_menu_item(
                "overlays-menu-disabled",
                &InspectorFieldAccess::disabled(Some("Not available on this selection".into())),
                metrics,
                cx,
            )
            .debug_selector(|| "overlays-menu-disabled".to_owned())
            .child(truncating_label("Delete layer")),
        );
        inspector_anchored_menu(overlay_placement(), surface)
    }

    fn render_overlays_trigger(
        &self,
        surface: OverlaySurface,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let metrics = InspectorMetrics::default();
        let open = self.overlays_screen.named_state.surface() == Some(surface);
        let (id, label, handle) = match surface {
            OverlaySurface::Popover => (
                "overlays-popover-trigger",
                "Open popover",
                &self.overlays_screen.popover_trigger,
            ),
            OverlaySurface::Menu => (
                "overlays-menu-trigger",
                "Open context menu",
                &self.overlays_screen.menu_trigger,
            ),
        };
        let trigger = inspector_action_button(id, &InspectorFieldAccess::Editable, metrics, cx)
            .debug_selector(move || id.to_owned())
            .border_color(if open {
                fanta_gpui::atoms::SemanticColor::BackgroundSelected.resolve(cx)
            } else {
                fanta_gpui::atoms::SemanticColor::Border.resolve(cx)
            })
            .track_focus(handle)
            .on_activate(cx.listener(move |this, event: &ActivateEvent, window, cx| {
                // Opening only: a pointer press on the trigger while the
                // surface is open is itself an outside click, and the log
                // shows that dismissal arriving first.
                if this.overlays_screen.named_state.surface() == Some(surface) {
                    return;
                }
                this.overlays_screen.open(
                    surface,
                    if event.keyboard {
                        "Enter/Space on the trigger"
                    } else {
                        "pointer on the trigger"
                    },
                    window,
                    cx,
                );
            }))
            .child(label);
        v_flex()
            .relative()
            .items_start()
            .child(trigger)
            .when(open, |wrapper| {
                wrapper.child(match surface {
                    OverlaySurface::Popover => self.render_overlays_popover(cx),
                    OverlaySurface::Menu => self.render_overlays_menu(cx),
                })
            })
            .into_any_element()
    }

    fn render_overlays_demo_area(&self, cx: &mut Context<Self>) -> AnyElement {
        let metrics = InspectorMetrics::default();
        let neighbor = inspector_action_button(
            "overlays-neighbor",
            &InspectorFieldAccess::Editable,
            metrics,
            cx,
        )
        .debug_selector(|| "overlays-neighbor".to_owned())
        .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
        .track_focus(&self.overlays_screen.neighbor)
        .on_activate(cx.listener(|this, _: &ActivateEvent, window, cx| {
            this.overlays_screen.last_action =
                "Neighbor control activated — it never opens a surface".into();
            this.overlays_screen.observe_focus(false, window, cx);
            cx.notify();
        }))
        .child("Neighbor control");

        div()
            .id("overlays-demo-area")
            .debug_selector(|| "overlays-demo-area".to_owned())
            .relative()
            .w_full()
            .h(px(DEMO_AREA_HEIGHT))
            .p_3()
            .rounded(px(8.))
            .border_1()
            .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
            .bg(fanta_gpui::atoms::SemanticColor::BackgroundSecondary
                .resolve(cx)
                .opacity(0.35))
            .child(
                h_flex()
                    .items_start()
                    .gap_3()
                    .flex_wrap()
                    .child(self.render_overlays_trigger(OverlaySurface::Popover, cx))
                    .child(self.render_overlays_trigger(OverlaySurface::Menu, cx))
                    .child(neighbor),
            )
            .child(
                v_flex()
                    .absolute()
                    .left(px(12.))
                    .bottom(px(12.))
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(10.))
                            .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                            .child("FOCUS NOW"),
                    )
                    .child(
                        div()
                            .id("overlays-focus-readout")
                            .debug_selector(|| "overlays-focus-readout".to_owned())
                            .typography(fanta_gpui::atoms::TypographyToken::BodyLarge)
                            .child(self.overlays_screen.focused_control.clone()),
                    ),
            )
            .into_any_element()
    }

    fn render_overlays_log(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut log = v_flex()
            .id("overlays-dismissal-log")
            .debug_selector(|| "overlays-dismissal-log".to_owned())
            .w_full()
            .gap_2();
        if self.overlays_screen.log.is_empty() {
            return log
                .child(
                    div()
                        .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                        .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                        .child(
                            "Nothing closed yet. Open a surface, then press Esc, click \
                             the neighbor control, or activate a row — each close adds \
                             one line here.",
                        ),
                )
                .into_any_element();
        }
        for (index, record) in self.overlays_screen.log.iter().enumerate() {
            let cause = match record.cause {
                Some(cause) => format!("InspectorOverlayDismissCause::{cause:?}"),
                None => "commit · no InspectorOverlayDismissCause".to_owned(),
            };
            log = log.child(
                v_flex()
                    .w_full()
                    .gap_1()
                    .px_2()
                    .py_1p5()
                    .rounded(px(6.))
                    .border_1()
                    .border_color(if index == 0 {
                        fanta_gpui::atoms::SemanticColor::BackgroundSelected.resolve(cx)
                    } else {
                        fanta_gpui::atoms::SemanticColor::Border.resolve(cx)
                    })
                    .bg(fanta_gpui::atoms::SemanticColor::Background.resolve(cx))
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .gap_2()
                            .flex_wrap()
                            .child(
                                div()
                                    .px_1()
                                    .py_0p5()
                                    .rounded(px(4.))
                                    .border_1()
                                    .border_color(if record.cause.is_some() {
                                        fanta_gpui::atoms::SemanticColor::BackgroundSelected
                                            .resolve(cx)
                                    } else {
                                        fanta_gpui::atoms::SemanticColor::Border.resolve(cx)
                                    })
                                    .text_size(px(11.))
                                    .child(cause),
                            )
                            .child(
                                div()
                                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                                    .text_color(
                                        fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx),
                                    )
                                    .child(format!(
                                        "{} · {} · {}",
                                        record.surface.label(),
                                        record.route.label(),
                                        record.detail
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                            .child(format!(
                                "focus {} → {} · restore_focus() → {} · generation {}",
                                record.focus_before,
                                record.focus_after,
                                record.restored,
                                record.generation
                            )),
                    ),
            );
        }
        log.into_any_element()
    }

    fn render_overlays_cause_matrix(&self, cx: &mut Context<Self>) -> AnyElement {
        state_matrix(
            OverlaysNamedState::ALL,
            OverlayDismissRoute::ALL,
            |state, route, cx| {
                let (text, reported) = match state.surface() {
                    None => ("nothing open", false),
                    Some(_) => (route.cause_badge(), route.cause().is_some()),
                };
                div()
                    .px_1()
                    .py_0p5()
                    .rounded(px(4.))
                    .border_1()
                    .border_color(if reported {
                        fanta_gpui::atoms::SemanticColor::BackgroundSelected.resolve(cx)
                    } else {
                        fanta_gpui::atoms::SemanticColor::Border.resolve(cx)
                    })
                    .text_size(px(10.))
                    .text_color(if reported {
                        fanta_gpui::atoms::SemanticColor::Text.resolve(cx)
                    } else {
                        fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx)
                    })
                    .child(text)
                    .into_any_element()
            },
            cx,
        )
    }

    pub(crate) fn render_overlays_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let demo_area = self.render_overlays_demo_area(cx);
        let log = self.render_overlays_log(cx);
        let matrix = self.render_overlays_cause_matrix(cx);
        let routes = OverlayDismissRoute::ALL
            .iter()
            .map(|route| format!("{} — {}", route.label(), route.how()))
            .collect::<Vec<_>>()
            .join("   ·   ");

        specimen_story_root("storybook-overlays")
            .track_focus(&self.overlays_screen.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape"
                    && let Some(surface) = this.overlays_screen.named_state.surface()
                {
                    this.overlays_screen.dismiss(
                        surface,
                        OverlayDismissRoute::Escape,
                        "Esc while the surface was open",
                        window,
                        cx,
                    );
                    cx.stop_propagation();
                }
            }))
            .child(specimen_card(
                "overlays-anchored-surfaces",
                "inspector_anchored_overlay · one placement, two anatomies",
                "The popover and the context menu are mounted with the same \
                 InspectorOverlayPlacement — same anchor, same offset, same \
                 deferred priority — so only their content anatomy differs. Open \
                 either trigger with the pointer or with Enter/Space; focus moves \
                 into the surface, so Esc and Tab work without ever touching the \
                 mouse. Clicking the trigger of an open surface counts as an \
                 outside click, and the log below shows that arriving first.",
                specimen_rows(vec![specimen_row(
                    "overlays-demo-row",
                    "Two triggers and a neighbor · the readout names the focused control",
                    vec![demo_area],
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "overlays-dismissal-log-card",
                "InspectorOverlayDismissIntent · which cause fired, and where focus went",
                "Every close routes through the owner: it clears its open state, \
                 then asks the intent to restore focus, which defers the focus \
                 call until the surface has left the tree. Each line names the \
                 cause the owner was handed, what restore_focus() reported, and — \
                 read after the deferred restore ran — the control focus actually \
                 landed on. Dismiss the popover by clicking the neighbor control: \
                 focus returns to the trigger, not to what you clicked. Turn the \
                 retained focus target off in the knobs and the same close leaves \
                 focus where it was.",
                specimen_rows(vec![specimen_row(
                    "overlays-log-row",
                    "Newest close first · the three routes and how to fire them",
                    vec![
                        v_flex()
                            .w_full()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(
                                        fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx),
                                    )
                                    .child(routes),
                            )
                            .child(log)
                            .into_any_element(),
                    ],
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "overlays-cause-matrix",
                "Commit is not a dismissal",
                "InspectorOverlayDismissCause has exactly two variants, so the \
                 Commit column reports none of them in any state: a committing \
                 row closes its own surface and restores focus from the retained \
                 target directly. The Closed row is empty for the same reason — \
                 with nothing open there is no intent to deliver at all.",
                matrix,
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn render_overlays_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.spec_reference(
            "storybook-reference-overlays",
            self.render_overlays_story(cx),
            cx,
        )
    }

    pub(crate) fn render_overlays_knobs(&self, cx: &mut Context<Self>) -> AnyElement {
        knobs::knobs_panel(
            "overlays-story-knobs",
            vec![
                knobs::enum_knob_row(
                    "overlays-knob-state",
                    "NAMED STATE",
                    OverlaysNamedState::ALL
                        .iter()
                        .copied()
                        .map(|state| KnobOption::new(state, state.label())),
                    self.overlays_screen.named_state,
                    |this, state, window, cx| {
                        this.overlays_screen.apply_named_state(state, window, cx);
                    },
                    cx,
                ),
                knobs::bool_knob_row(
                    "overlays-knob-focus-target",
                    "RETAINED FOCUS TARGET",
                    "Restore focus to the trigger",
                    self.overlays_screen.retain_focus_target,
                    |this, value, _, cx| {
                        this.overlays_screen.retain_focus_target = value;
                        this.overlays_screen.last_action = if value {
                            "Dismissals now retain a focus target".into()
                        } else {
                            "Dismissals now retain no focus target — restore_focus() reports false"
                                .into()
                        };
                        cx.notify();
                    },
                    cx,
                ),
            ],
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_is_the_only_route_without_a_dismiss_cause() {
        assert_eq!(
            OverlayDismissRoute::Escape.cause(),
            Some(InspectorOverlayDismissCause::Escape)
        );
        assert_eq!(
            OverlayDismissRoute::OutsideClick.cause(),
            Some(InspectorOverlayDismissCause::OutsideClick)
        );
        assert_eq!(OverlayDismissRoute::Commit.cause(), None);
        assert_eq!(
            OverlayDismissRoute::ALL
                .iter()
                .filter(|route| route.cause().is_none())
                .count(),
            1,
            "the matrix's empty column must stay the commit column"
        );
    }

    #[test]
    fn every_named_state_round_trips_through_its_surface() {
        assert_eq!(OverlaysNamedState::Closed.surface(), None);
        for state in OverlaysNamedState::ALL.iter().copied() {
            if let Some(surface) = state.surface() {
                assert_eq!(OverlaysNamedState::for_surface(surface), state);
            }
        }
        assert_eq!(
            OverlaysNamedState::ALL
                .iter()
                .filter(|state| state.surface().is_some())
                .count(),
            2,
            "both anchored surfaces must be reachable from a named state alone"
        );
    }

    #[test]
    fn both_surfaces_share_one_placement() {
        let placement = overlay_placement();
        assert_eq!(placement.anchor(), Anchor::TopLeft);
        assert_eq!(placement.priority(), OVERLAY_PRIORITY);
        assert_eq!(placement.offset(), point(px(0.), px(6.)));
    }
}
