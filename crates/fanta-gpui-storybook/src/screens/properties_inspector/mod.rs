//! Mock-host composition; inspection fixtures live exclusively in Knobs.
use super::knobs::{self, KnobOption};
use super::properties_tabs::PropertiesTabsScreen;
use crate::*;
use fanta_gpui::design::{DesignInspector, DesignPropertyPanel, DesignPropertyPanelKind};

pub(crate) struct PropertiesInspectorScreen {
    pub(crate) inspector: Entity<PropertiesInspector>,
    pub(crate) design: Entity<DesignInspector>,
    panels: Vec<(DesignPropertyPanelKind, Entity<DesignPropertyPanel>)>,
    pub(crate) last_action: SharedString,
    pub(crate) typography_host: DesignScreen,
}
impl PropertiesInspectorScreen {
    pub(crate) fn new(
        controller: Entity<DesignPanel>,
        tabs: &PropertiesTabsScreen,
        window: &mut Window,
        cx: &mut Context<Storybook>,
    ) -> Self {
        let design = cx
            .new(|cx| DesignInspector::new("storybook-design-properties", controller.clone(), cx));
        let mut typography_host = DesignScreen::new(window, cx);
        typography_host
            .activate_inspection_scenario(super::design::DesignInspectionScenario::TextEdit, cx);
        let panels = DesignPropertyPanelKind::ALL
            .iter()
            .copied()
            .map(|kind| {
                let panel_controller = if kind == DesignPropertyPanelKind::Typography {
                    typography_host.panel.clone()
                } else {
                    controller.clone()
                };
                let panel = cx.new(|cx| {
                    DesignPropertyPanel::new(
                        format!("storybook-property-{kind:?}"),
                        kind,
                        panel_controller,
                        cx,
                    )
                });
                (kind, panel)
            })
            .collect();
        let inspector = cx.new(|cx| {
            PropertiesInspector::new(
                "storybook-properties-inspector",
                PropertiesInspectorChildren {
                    design: design.clone().into(),
                    motion: tabs.motion.clone().into(),
                    draw: tabs.draw.clone().into(),
                    code: tabs.code.clone().into(),
                    prototype: tabs.prototype.clone().into(),
                    comments: tabs.comments.clone().into(),
                },
                100,
                cx,
            )
        });
        Self {
            inspector,
            design,
            panels,
            typography_host,
            last_action: "Ready".into(),
        }
    }
    pub(crate) fn panel(&self, kind: DesignPropertyPanelKind) -> Entity<DesignPropertyPanel> {
        self.panels
            .iter()
            .find(|(k, _)| *k == kind)
            .unwrap()
            .1
            .clone()
    }
    pub(crate) fn handle_action(
        &mut self,
        action: &PropertiesInspectorAction,
        window: &mut Window,
        cx: &mut Context<Storybook>,
    ) {
        self.last_action = format!("{action:?}").into();
        if matches!(action, PropertiesInspectorAction::TabChanged { tab } if *tab != PropertiesInspectorTab::Design)
            || matches!(
                action,
                PropertiesInspectorAction::CollapsedChanged { collapsed: true }
            )
        {
            self.design
                .update(cx, |design, cx| design.deactivate(window, cx));
        }
        if let PropertiesInspectorAction::Zoom(action) = action {
            use fanta_gpui::molecules::ZoomControlsAction;
            let percent = match action {
                ZoomControlsAction::ZoomChangeRequested { percent } => *percent,
                ZoomControlsAction::FitToViewRequested => 75,
                ZoomControlsAction::FitToSelectionRequested => 150,
            };
            self.inspector
                .update(cx, |sidebar, cx| sidebar.set_zoom(percent, cx));
        }
        cx.notify();
    }
}
impl Storybook {
    pub(crate) fn render_properties_inspector_knobs(&self, cx: &mut Context<Self>) -> AnyElement {
        let tab = self
            .properties_inspector_screen
            .inspector
            .read(cx)
            .active_tab();
        let collapsed = self
            .properties_inspector_screen
            .inspector
            .read(cx)
            .is_collapsed();
        v_flex()
            .gap_2()
            .child(knobs::knobs_panel(
                "properties-inspector-knobs",
                vec![
                    knobs::enum_knob_row(
                        "properties-tab-knob",
                        "TAB",
                        PropertiesInspectorTab::ALL.map(|tab| KnobOption::new(tab, tab.label())),
                        tab,
                        |story, tab, window, cx| {
                            story
                                .properties_inspector_screen
                                .inspector
                                .update(cx, |sidebar, cx| sidebar.set_active_tab(tab, cx));
                            story.properties_inspector_screen.handle_action(
                                &PropertiesInspectorAction::TabChanged { tab },
                                window,
                                cx,
                            );
                        },
                        cx,
                    ),
                    knobs::bool_knob_row(
                        "properties-collapse-knob",
                        "SIDEBAR",
                        "Collapsed",
                        collapsed,
                        |story, value, window, cx| {
                            story
                                .properties_inspector_screen
                                .inspector
                                .update(cx, |sidebar, cx| sidebar.set_collapsed(value, cx));
                            story.properties_inspector_screen.handle_action(
                                &PropertiesInspectorAction::CollapsedChanged { collapsed: value },
                                window,
                                cx,
                            );
                        },
                        cx,
                    ),
                ],
                cx,
            ))
            .child(if tab == PropertiesInspectorTab::Design {
                self.render_design_knobs(cx)
            } else {
                self.render_properties_tabs_knobs(cx)
            })
            .into_any_element()
    }
}
