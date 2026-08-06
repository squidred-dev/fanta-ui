//! The Prototype story: mock host state, reducer, and knobs.

use crate::*;

use super::knobs::{self, KnobOption};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PrototypeNamedState {
    Default,
}

impl PrototypeNamedState {
    pub(crate) const ALL: [Self; 1] = [Self::Default];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
        }
    }
}

pub(crate) struct PrototypeScreen {
    pub(crate) panel: Entity<PrototypePanel>,
    pub(crate) view_data: PrototypeViewData,
    pub(crate) last_action: SharedString,
    pub(crate) named_state: PrototypeNamedState,
}

impl PrototypeScreen {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        let view_data = PrototypeViewData::default();
        let panel = cx.new(|cx| PrototypePanel::new("storybook-prototype", view_data.clone(), cx));
        Self {
            panel,
            view_data,
            last_action: "Ready — switch surfaces, inspect settings, or dismiss either hint".into(),
            named_state: PrototypeNamedState::Default,
        }
    }

    fn fixture(state: PrototypeNamedState) -> PrototypeViewData {
        match state {
            PrototypeNamedState::Default => PrototypeViewData::default(),
        }
    }

    pub(crate) fn apply_named_state(
        &mut self,
        state: PrototypeNamedState,
        cx: &mut Context<Storybook>,
    ) {
        self.named_state = state;
        self.view_data = Self::fixture(state);
        let view_data = self.view_data.clone();
        self.panel
            .update(cx, |panel, cx| panel.set_view_data(view_data, cx));
        self.last_action = format!("Story applied the {} Prototype state", state.label()).into();
        cx.notify();
    }

    pub(crate) fn handle_action(
        &mut self,
        panel: Entity<PrototypePanel>,
        action: &PrototypePanelAction,
        cx: &mut Context<Storybook>,
    ) {
        match action {
            PrototypePanelAction::SurfaceChangeRequested { surface } => {
                self.view_data.surface = *surface;
                self.last_action = format!("Selected the {surface:?} surface").into();
            }
            PrototypePanelAction::ZoomMenuRequested => {
                self.view_data.zoom_percent = match self.view_data.zoom_percent {
                    0..=12 => 25,
                    13..=25 => 50,
                    26..=50 => 100,
                    _ => 12,
                };
                self.last_action = format!(
                    "Host selected {}% prototype zoom",
                    self.view_data.zoom_percent
                )
                .into();
            }
            PrototypePanelAction::DeviceMenuRequested => {
                self.view_data.device_name = if self.view_data.device_name.as_ref() == "No device" {
                    "iPhone 16 Pro".into()
                } else {
                    "No device".into()
                };
                self.last_action = format!(
                    "Host selected prototype device {}",
                    self.view_data.device_name
                )
                .into();
            }
            PrototypePanelAction::BackgroundEditRequested => {
                self.view_data.background_hex =
                    if self.view_data.background_hex.as_ref() == "000000" {
                        "1E1E1E".into()
                    } else {
                        "000000".into()
                    };
                self.last_action = format!(
                    "Host changed prototype background to #{}",
                    self.view_data.background_hex
                )
                .into();
            }
            PrototypePanelAction::HintDismissed { hint } => {
                self.last_action = format!("Dismissed the {hint:?} hint").into();
            }
        }
        panel.update(cx, |panel, cx| {
            panel.set_view_data(self.view_data.clone(), cx);
        });
        cx.notify();
    }
}

impl Storybook {
    pub(crate) fn render_prototype_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-prototype",
            self.prototype_screen.panel.clone().into_any_element(),
            self.prototype_screen.last_action.clone(),
            cx,
        )
    }

    pub(crate) fn render_prototype_knobs(&self, cx: &mut Context<Self>) -> AnyElement {
        knobs::knobs_panel(
            "prototype-story-knobs",
            vec![knobs::enum_knob_row(
                "prototype-knob-state",
                "NAMED STATE",
                PrototypeNamedState::ALL.map(|state| KnobOption::new(state, state.label())),
                self.prototype_screen.named_state,
                |this, state, _, cx| this.prototype_screen.apply_named_state(state, cx),
                cx,
            )],
            cx,
        )
    }
}
