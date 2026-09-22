//! Individually mountable property panels sharing one controlled edit session.
//!
//! `DesignPanel` is the compatibility controller and action source. These
//! entities render feature sections directly; they never mount its legacy
//! shell, workspace header, or surface tabs.

use super::*;

mod controller;
mod kind;

pub(super) use controller::DesignPanelCompositionController;
pub use kind::DesignPropertyPanelKind;

/// An independently composable property panel backed by a shared edit
/// controller. It renders only `kind`, with its complete existing fields,
/// catalogs, overlays, permission checks, and edit lifecycle.
///
/// Subscribe to [`DesignPanelAction`] on [`Self::controller`]. A composition
/// shares one controller so it also shares one action stream and one overlay
/// coordinator; it does not create a new full inspector per section.
pub struct DesignPropertyPanel {
    id: SharedString,
    kind: DesignPropertyPanelKind,
    controller: Entity<DesignPanel>,
    standalone: bool,
    _observation: Subscription,
}

impl DesignPropertyPanel {
    pub fn new(
        id: impl Into<SharedString>,
        kind: DesignPropertyPanelKind,
        controller: Entity<DesignPanel>,
        cx: &mut Context<Self>,
    ) -> Self {
        let observation = cx.observe(&controller, |_, _, cx| cx.notify());
        Self {
            id: id.into(),
            kind,
            controller,
            standalone: true,
            _observation: observation,
        }
    }

    pub fn controller(&self) -> &Entity<DesignPanel> {
        &self.controller
    }

    /// Call before the host hides or unmounts this inspector. Active edits
    /// receive one terminal Cancel and overlays close without changing the
    /// accepted host snapshot, navigation, or property values.
    pub fn deactivate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.controller.update(cx, |controller, cx| {
            controller.deactivate_composed_panels(window, cx);
        });
    }

    pub const fn kind(&self) -> DesignPropertyPanelKind {
        self.kind
    }

    /// Whether the current host selection, permissions, and workspace allow
    /// this panel. An inapplicable standalone panel takes no layout space.
    pub fn is_applicable(&self, cx: &App) -> bool {
        self.controller
            .read(cx)
            .composed_panel_kinds()
            .contains(&self.kind)
    }
}

impl Focusable for DesignPropertyPanel {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.controller.read(cx).focus_handle.clone()
    }
}

impl Render for DesignPropertyPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let kind = self.kind;
        let id = self.id.clone();
        let standalone = self.standalone;
        self.controller.update(cx, |controller, cx| {
            if standalone {
                controller.prepare_composed_panels(window, cx);
            }
            let applicable = controller.composed_panel_kinds().contains(&kind);
            if !applicable {
                return div().into_any_element();
            }
            let content = controller.render_composed_panel(kind, cx);
            if standalone {
                controller
                    .composed_interaction_root(id, cx)
                    .w_full()
                    .flex_none()
                    .child(content)
                    .when_some(controller.render_scrub_speed_cue(cx), |root, cue| {
                        root.child(cue)
                    })
                    .into_any_element()
            } else {
                div()
                    .id(id.clone())
                    .debug_selector(move || id.to_string())
                    .w_full()
                    .flex_none()
                    .child(content)
                    .into_any_element()
            }
        })
    }
}

/// The Design or Draw tab's default composition of individual property panels.
///
/// This organism owns only scrolling and retained panel entities. The passed
/// controller owns transient editing continuity and accepts authoritative
/// [`DesignPanelViewData`] from the host. Layouts place their own navigation
/// above this organism; there are no duplicate surface tabs inside it.
pub struct DesignInspector {
    id: SharedString,
    controller: Entity<DesignPanel>,
    panels: Vec<(DesignPropertyPanelKind, Entity<DesignPropertyPanel>)>,
    _observation: Subscription,
}

impl DesignInspector {
    pub fn new(
        id: impl Into<SharedString>,
        controller: Entity<DesignPanel>,
        cx: &mut Context<Self>,
    ) -> Self {
        let id = id.into();
        let panels = DesignPropertyPanelKind::ALL
            .iter()
            .copied()
            .map(|kind| {
                let panel_id = SharedString::from(format!("{}-{}", id, kind.slug()));
                let panel = cx.new(|cx| {
                    let mut panel =
                        DesignPropertyPanel::new(panel_id, kind, controller.clone(), cx);
                    panel.standalone = false;
                    panel
                });
                (kind, panel)
            })
            .collect();
        let observation = cx.observe(&controller, |_, _, cx| cx.notify());
        Self {
            id,
            controller,
            panels,
            _observation: observation,
        }
    }

    pub fn controller(&self) -> &Entity<DesignPanel> {
        &self.controller
    }

    /// Call before the host hides or unmounts this inspector. Active edits
    /// receive one terminal Cancel and overlays close without changing the
    /// accepted host snapshot, navigation, or property values.
    pub fn deactivate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.controller.update(cx, |controller, cx| {
            controller.deactivate_composed_panels(window, cx);
        });
    }

    /// The exact ordered panel composition for the current host context.
    pub fn visible_panel_kinds(&self, cx: &App) -> Vec<DesignPropertyPanelKind> {
        self.controller.read(cx).composed_panel_kinds()
    }
}

impl Focusable for DesignInspector {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.controller.read(cx).focus_handle.clone()
    }
}

impl Render for DesignInspector {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let id = self.id.clone();
        let visible = self.visible_panel_kinds(cx);
        let panels: Vec<_> = visible
            .into_iter()
            .filter_map(|kind| {
                self.panels
                    .iter()
                    .find(|(candidate, _)| *candidate == kind)
                    .map(|(_, panel)| panel.clone())
            })
            .collect();
        self.controller.update(cx, |controller, cx| {
            controller.prepare_composed_panels(window, cx);
            let scroll_handle = controller.shell.scroll_handle.clone();
            let summary = sections::header::render_selection_summary(
                &controller.header_projection(),
                sections::header::HeaderEventSink::new(cx.entity()),
                cx,
            );
            controller
                .composed_interaction_root(id.clone(), cx)
                .size_full()
                .min_h_0()
                .child(summary)
                .child(
                    div()
                        .relative()
                        .flex_1()
                        .min_h_0()
                        .child(
                            v_flex()
                                .id(SharedString::from(format!("{id}-panels")))
                                .size_full()
                                .min_h_0()
                                .overflow_y_scroll()
                                .track_scroll(&scroll_handle)
                                .children(panels),
                        )
                        .child(
                            div().absolute().top_0().right_0().h_full().child(
                                Scrollbar::new(&scroll_handle).axis(ScrollbarAxis::Vertical),
                            ),
                        )
                        .when_some(controller.render_scrub_speed_cue(cx), |root, cue| {
                            root.child(cue)
                        }),
                )
                .into_any_element()
        })
    }
}

#[cfg(test)]
mod tests;
