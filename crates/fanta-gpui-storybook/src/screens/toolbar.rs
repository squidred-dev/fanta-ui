//! The Toolbar story: the `EditorToolbar` dock in isolation, plus the mock
//! host options, env-seeded knob defaults, reducer, and the
//! mode/overlay/zoom knobs that drive it.
//!
//! `FANTA_TOOLBAR_MODE` and `FANTA_TOOLBAR_OVERLAY` stay supported as the
//! initial values of the runtime knobs, preserving the screenshot-CI
//! launch interface.

use crate::*;

use super::knobs::{self, KnobOption};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ToolbarOverlay {
    None,
    Actions,
}

impl ToolbarOverlay {
    pub(crate) const ALL: [Self; 2] = [Self::None, Self::Actions];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Actions => "Actions",
        }
    }
}

/// Horizontal breathing room between the story surface and the dock. The
/// floor test subtracts it to prove the dock still clears its own collapse
/// breakpoint at the smallest registered viewport.
const STORY_HORIZONTAL_PADDING: f32 = 16.;

/// The launch-mode order is also the knob's chip order.
pub(crate) const TOOLBAR_MODE_KNOB_ORDER: [ToolbarMode; 4] = [
    ToolbarMode::Design,
    ToolbarMode::Motion,
    ToolbarMode::Draw,
    ToolbarMode::Dev,
];

pub(crate) fn parse_toolbar_mode(value: Option<&str>) -> ToolbarMode {
    match value.unwrap_or_default().to_ascii_lowercase().as_str() {
        "motion" => ToolbarMode::Motion,
        "draw" => ToolbarMode::Draw,
        "dev" => ToolbarMode::Dev,
        _ => ToolbarMode::Design,
    }
}

pub(crate) fn parse_toolbar_overlay(value: Option<&str>) -> ToolbarOverlay {
    match value.unwrap_or_default().to_ascii_lowercase().as_str() {
        "actions" => ToolbarOverlay::Actions,
        _ => ToolbarOverlay::None,
    }
}

pub(crate) struct ToolbarScreen {
    pub(crate) toolbar: Entity<EditorToolbar>,
    pub(crate) mode: ToolbarMode,
    pub(crate) tool: ToolbarTool,
    pub(crate) zoom: u16,
    pub(crate) dev_options: DevToolbarOptions,
    pub(crate) motion_options: MotionToolbarOptions,
    pub(crate) overlay: ToolbarOverlay,
    pub(crate) last_action: SharedString,
}

impl ToolbarScreen {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Storybook>) -> Self {
        let mode = parse_toolbar_mode(std::env::var("FANTA_TOOLBAR_MODE").ok().as_deref());
        let tool = match mode {
            ToolbarMode::Design => ToolbarTool::Move,
            ToolbarMode::Motion => ToolbarTool::MotionSelect,
            ToolbarMode::Draw => ToolbarTool::Brush,
            ToolbarMode::Dev => ToolbarTool::Inspect,
        };
        let zoom = 100;
        // The mock host seeds every candidate the Motion option editor may
        // offer; the toolbar only presents these host values.
        let dev_options = DevToolbarOptions::default();
        let motion_options = MotionToolbarOptions {
            available_animation_styles: ["Fade in", "Spring", "Slide up", "Pop"]
                .into_iter()
                .map(Into::into)
                .collect(),
            ..MotionToolbarOptions::default()
        };
        let toolbar =
            cx.new(|cx| EditorToolbar::new("storybook-toolbar", mode, tool, zoom, window, cx));
        toolbar.update(cx, |toolbar, cx| {
            toolbar.set_motion_options(motion_options.clone(), cx);
            toolbar.set_dev_options(dev_options, cx);
        });
        toolbar.focus_handle(cx).focus(window, cx);
        let overlay = parse_toolbar_overlay(std::env::var("FANTA_TOOLBAR_OVERLAY").ok().as_deref());
        match overlay {
            ToolbarOverlay::Actions => toolbar.update(cx, |toolbar, cx| {
                toolbar.open_actions(window, cx);
            }),
            ToolbarOverlay::None => {}
        }
        Self {
            toolbar,
            mode,
            tool,
            zoom,
            dev_options,
            motion_options,
            overlay,
            last_action: "Ready — explore Design, Motion, Draw and Dev, tool flyouts, contextual settings and Actions"
                .into(),
        }
    }
}

impl Storybook {
    pub(crate) fn handle_toolbar_action(
        &mut self,
        toolbar: Entity<EditorToolbar>,
        action: &ToolbarAction,
        cx: &mut Context<Self>,
    ) {
        match action {
            ToolbarAction::ModeChangeRequested { mode } => {
                self.toolbar_screen.mode = *mode;
                self.toolbar_screen.tool = match mode {
                    ToolbarMode::Design => ToolbarTool::Move,
                    ToolbarMode::Motion => ToolbarTool::MotionSelect,
                    ToolbarMode::Draw => ToolbarTool::Brush,
                    ToolbarMode::Dev => ToolbarTool::Inspect,
                };
                toolbar.update(cx, |toolbar, cx| {
                    toolbar.set_mode(*mode, cx);
                    toolbar.set_active_tool(self.toolbar_screen.tool, cx);
                });
                self.toolbar_screen.last_action =
                    format!("Host switched to {} mode", mode.label()).into();
            }
            ToolbarAction::ToolChangeRequested { mode, tool } => {
                self.toolbar_screen.mode = *mode;
                self.toolbar_screen.tool = *tool;
                toolbar.update(cx, |toolbar, cx| {
                    toolbar.set_mode(*mode, cx);
                    toolbar.set_active_tool(*tool, cx);
                });
                self.toolbar_screen.last_action =
                    format!("Host selected {} in {}", tool.label(), mode.label()).into();
            }
            ToolbarAction::SecondaryControlInvoked { mode, control } => {
                let tool = match control {
                    ToolbarSecondaryControl::DevInspect => Some(ToolbarTool::Inspect),
                    ToolbarSecondaryControl::DevAnnotate => Some(ToolbarTool::Annotation),
                    ToolbarSecondaryControl::DevMeasure => Some(ToolbarTool::Measure),
                    ToolbarSecondaryControl::MotionAddKeyframe => {
                        self.toolbar_screen.last_action = "Host inserted a keyframe at 0.0s".into();
                        None
                    }
                    _ => None,
                };
                if let Some(tool) = tool {
                    self.toolbar_screen.tool = tool;
                    toolbar.update(cx, |toolbar, cx| {
                        toolbar.set_active_tool(tool, cx);
                    });
                }
                if !matches!(control, ToolbarSecondaryControl::MotionAddKeyframe) {
                    self.toolbar_screen.last_action =
                        format!("Host invoked {} in {}", control.label(), mode.label()).into();
                }
            }
            ToolbarAction::ControlChangeRequested {
                mode,
                control,
                value,
            } => {
                match (control, value) {
                    (
                        ToolbarSecondaryControl::DevReadyForDevelopment,
                        ToolbarControlValue::Toggle(value),
                    ) => self.toolbar_screen.dev_options.ready_for_development = *value,
                    (
                        ToolbarSecondaryControl::MotionPlayPause,
                        ToolbarControlValue::Toggle(value),
                    ) => self.toolbar_screen.motion_options.playing = *value,
                    (ToolbarSecondaryControl::MotionLoop, ToolbarControlValue::Toggle(value)) => {
                        self.toolbar_screen.motion_options.looping = *value
                    }
                    (
                        ToolbarSecondaryControl::MotionAutoKeyframe,
                        ToolbarControlValue::Toggle(value),
                    ) => self.toolbar_screen.motion_options.auto_keyframe = *value,
                    (
                        ToolbarSecondaryControl::MotionAnimationStyle,
                        ToolbarControlValue::Choice(value),
                    ) => self.toolbar_screen.motion_options.animation_style = value.clone(),
                    _ => {}
                }
                toolbar.update(cx, |toolbar, cx| {
                    toolbar.set_dev_options(self.toolbar_screen.dev_options, cx);
                    toolbar.set_motion_options(self.toolbar_screen.motion_options.clone(), cx);
                });
                self.toolbar_screen.last_action = format!(
                    "Host applied {} = {value:?} in {}",
                    control.label(),
                    mode.label()
                )
                .into();
            }
            ToolbarAction::CommandQueryChanged { query } => {
                self.toolbar_screen.last_action = if query.is_empty() {
                    "Actions search cleared".into()
                } else {
                    format!("Actions query: “{query}”").into()
                };
            }
            ToolbarAction::CommandInvoked { command } => {
                let requested_mode = match command {
                    ToolbarCommand::OpenDesignMode => Some(ToolbarMode::Design),
                    ToolbarCommand::OpenMotionMode => Some(ToolbarMode::Motion),
                    ToolbarCommand::OpenDrawMode => Some(ToolbarMode::Draw),
                    ToolbarCommand::OpenDevMode => Some(ToolbarMode::Dev),
                    _ => None,
                };
                if let Some(mode) = requested_mode {
                    self.toolbar_screen.mode = mode;
                    self.toolbar_screen.tool = match mode {
                        ToolbarMode::Design => ToolbarTool::Move,
                        ToolbarMode::Motion => ToolbarTool::MotionSelect,
                        ToolbarMode::Draw => ToolbarTool::Brush,
                        ToolbarMode::Dev => ToolbarTool::Inspect,
                    };
                    toolbar.update(cx, |toolbar, cx| {
                        toolbar.set_mode(mode, cx);
                        toolbar.set_active_tool(self.toolbar_screen.tool, cx);
                    });
                }
                self.toolbar_screen.last_action =
                    format!("Host ran {} · {}", command.category(), command.label()).into();
            }
            ToolbarAction::DrawOptionsChangeRequested { options } => {
                toolbar.update(cx, |toolbar, cx| {
                    toolbar.set_draw_options(options.clone(), cx)
                });
                self.toolbar_screen.last_action =
                    format!("Host applied Draw settings: {options:?}").into();
            }
            ToolbarAction::DrawActionInvoked { action } => {
                self.toolbar_screen.last_action =
                    format!("Host received Draw action: {action:?}").into();
            }
            ToolbarAction::ZoomChangeRequested { percent } => {
                self.toolbar_screen.zoom = *percent;
                toolbar.update(cx, |toolbar, cx| {
                    toolbar.set_zoom_percent(*percent, cx);
                });
                self.toolbar_screen.last_action =
                    format!("Host set canvas zoom to {percent}%").into();
            }
            ToolbarAction::ChromeControlInvoked { id } => {
                self.toolbar_screen.last_action =
                    format!("Host received chrome action {id}").into();
            }
        }
        cx.notify();
    }

    /// The overlay knob has no component intent to synthesize, so it
    /// drives the toolbar entity's overlay surface directly.
    pub(crate) fn apply_toolbar_overlay(
        &mut self,
        overlay: ToolbarOverlay,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toolbar_screen.overlay = overlay;
        self.toolbar_screen.toolbar.update(cx, |toolbar, cx| {
            toolbar.dismiss_overlay(window, cx);
            match overlay {
                ToolbarOverlay::Actions => toolbar.open_actions(window, cx),
                ToolbarOverlay::None => {}
            }
        });
        self.toolbar_screen.last_action =
            format!("Story set the {} toolbar overlay", overlay.label()).into();
        cx.notify();
    }

    /// The story is the toolbar and nothing else.
    ///
    /// `EditorToolbar` is a floating dock, so it keeps a neutral surface to
    /// sit on and rides near the bottom the way it does over a real canvas:
    /// that is where its tool overlays have room to open
    /// upward. No mock document, sidebar, or inspector — the gallery already
    /// reports the host's last accepted intent, and the knobs panel owns
    /// mode, overlay, and zoom.
    ///
    /// The surface takes its size from its parent rather than growing into
    /// it: the Gallery mounts a story inside a fixed-size `div`, which is
    /// `Display::Block`, so a `flex_1` child would collapse to the height of
    /// its in-flow content. The dock is placed by flex alignment for the
    /// same reason — an absolutely positioned child contributes no height,
    /// which would leave the surface empty and clip the dock out of view.
    pub(crate) fn render_toolbar_story(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .id("toolbar-story-surface")
            .debug_selector(|| "toolbar-story-surface".to_owned())
            .relative()
            .size_full()
            .min_h(px(0.))
            .min_w(px(0.))
            .justify_end()
            .items_center()
            .overflow_hidden()
            .bg(fanta_gpui::atoms::SemanticColor::BackgroundTertiary.resolve(cx))
            .child(
                h_flex()
                    .w_full()
                    .flex_none()
                    .pb(px(18.))
                    .px(px(STORY_HORIZONTAL_PADDING))
                    .justify_center()
                    .child(self.toolbar_screen.toolbar.clone()),
            )
            .into_any_element()
    }

    pub(crate) fn render_toolbar_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .id("storybook-reference-toolbar")
            .debug_selector(|| "storybook-reference-toolbar".to_owned())
            .size_full()
            .bg(fanta_gpui::atoms::SemanticColor::Background.resolve(cx))
            .text_color(fanta_gpui::atoms::SemanticColor::Text.resolve(cx))
            .child(self.render_reference_nav(cx))
            // The story surface fills its parent, so the reference window
            // gives it a flex row of its own below the nav.
            .child(
                v_flex()
                    .flex_1()
                    .min_h(px(0.))
                    .child(self.render_toolbar_story(cx)),
            )
            .into_any_element()
    }

    pub(crate) fn render_toolbar_knobs(&self, cx: &mut Context<Self>) -> AnyElement {
        knobs::knobs_panel(
            "toolbar-story-knobs",
            vec![
                knobs::enum_knob_row(
                    "toolbar-knob-mode",
                    "MODE",
                    TOOLBAR_MODE_KNOB_ORDER.map(|mode| KnobOption::new(mode, mode.label())),
                    self.toolbar_screen.mode,
                    |this, mode, _, cx| {
                        let toolbar = this.toolbar_screen.toolbar.clone();
                        this.handle_toolbar_action(
                            toolbar,
                            &ToolbarAction::ModeChangeRequested { mode },
                            cx,
                        );
                    },
                    cx,
                ),
                knobs::enum_knob_row(
                    "toolbar-knob-overlay",
                    "OVERLAY",
                    ToolbarOverlay::ALL.map(|overlay| KnobOption::new(overlay, overlay.label())),
                    self.toolbar_screen.overlay,
                    |this, overlay, window, cx| {
                        this.apply_toolbar_overlay(overlay, window, cx);
                    },
                    cx,
                ),
                knobs::numeric_knob_row(
                    "toolbar-knob-zoom",
                    "ZOOM",
                    [50u16, 100, 200]
                        .map(|percent| KnobOption::new(percent, format!("{percent}%"))),
                    self.toolbar_screen.zoom,
                    |this, percent, _, cx| {
                        let toolbar = this.toolbar_screen.toolbar.clone();
                        this.handle_toolbar_action(
                            toolbar,
                            &ToolbarAction::ZoomChangeRequested { percent },
                            cx,
                        );
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
    fn toolbar_env_vars_seed_the_runtime_knob_defaults() {
        assert_eq!(parse_toolbar_mode(None), ToolbarMode::Design);
        assert_eq!(
            parse_toolbar_mode(Some("draw")),
            ToolbarMode::Draw,
            "Draw mode is available in the compact mode selector"
        );
        assert_eq!(parse_toolbar_mode(Some("MOTION")), ToolbarMode::Motion);
        assert_eq!(parse_toolbar_mode(Some("dev")), ToolbarMode::Dev);
        assert_eq!(parse_toolbar_mode(Some("unknown")), ToolbarMode::Design);

        assert_eq!(parse_toolbar_overlay(None), ToolbarOverlay::None);
        assert_eq!(
            parse_toolbar_overlay(Some("actions")),
            ToolbarOverlay::Actions
        );
        assert_eq!(parse_toolbar_overlay(Some("AGENT")), ToolbarOverlay::None);
        assert_eq!(parse_toolbar_overlay(Some("off")), ToolbarOverlay::None);
    }

    #[test]
    fn toolbar_mode_knob_covers_every_mode_once() {
        let mut seen = std::collections::HashSet::new();
        for mode in TOOLBAR_MODE_KNOB_ORDER {
            assert!(seen.insert(mode.label()));
        }
        assert_eq!(seen.len(), ToolbarMode::ALL.len());
    }
}
