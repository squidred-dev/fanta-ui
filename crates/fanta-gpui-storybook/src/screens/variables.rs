//! The Variables story: mock host state, fixtures, reducer, and knobs.

use crate::*;

use super::knobs::{self, KnobOption};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VariablesNamedState {
    Default,
    Empty,
}

impl VariablesNamedState {
    pub(crate) const ALL: [Self; 2] = [Self::Default, Self::Empty];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Empty => "Empty",
        }
    }
}

pub(crate) fn seed_variables_view_data() -> VariablesViewData {
    VariablesViewData {
        document_name: "Untitled".into(),
        collections: vec![
            VariablesCollection::new("collection-1", "Collection 1", 1),
            VariablesCollection::new("collection-2", "Collection 2", 0),
        ],
        selected_collection_id: "collection-1".into(),
        groups: vec![VariablesGroup::new("all", "All", 1).aggregate()],
        selected_group_id: "all".into(),
        modes: vec![
            VariablesMode::new("mode-1", "Mode 1"),
            VariablesMode::new("mode-2", "Mode 2"),
        ],
        variables: vec![VariableRow::new(
            "color",
            "Color",
            "all",
            VariableKind::Color,
            [
                VariableModeValue::new("mode-1", "FFFFFF").color("FFFFFF"),
                VariableModeValue::new("mode-2", "FFFFFF").color("FFFFFF"),
            ],
        )],
    }
}

pub(crate) fn empty_variables_view_data() -> VariablesViewData {
    VariablesViewData {
        document_name: "Untitled".into(),
        collections: vec![VariablesCollection::new("collection-1", "Collection 1", 0)],
        selected_collection_id: "collection-1".into(),
        groups: vec![VariablesGroup::new("all", "All", 0).aggregate()],
        selected_group_id: "all".into(),
        modes: vec![VariablesMode::new("mode-1", "Mode 1")],
        variables: Vec::new(),
    }
}

pub(crate) struct VariablesScreen {
    pub(crate) page: Entity<VariablesPage>,
    pub(crate) view_data: VariablesViewData,
    pub(crate) last_action: SharedString,
    pub(crate) named_state: VariablesNamedState,
}

impl VariablesScreen {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Storybook>) -> Self {
        let view_data = seed_variables_view_data();
        let page =
            cx.new(|cx| VariablesPage::new("storybook-variables", view_data.clone(), window, cx));
        Self {
            page,
            view_data,
            last_action: "Ready — switch collections and groups, edit values, or add a mode".into(),
            named_state: VariablesNamedState::Default,
        }
    }

    fn fixture(state: VariablesNamedState) -> VariablesViewData {
        match state {
            VariablesNamedState::Default => seed_variables_view_data(),
            VariablesNamedState::Empty => empty_variables_view_data(),
        }
    }

    pub(crate) fn apply_named_state(
        &mut self,
        state: VariablesNamedState,
        cx: &mut Context<Storybook>,
    ) {
        self.named_state = state;
        self.view_data = Self::fixture(state);
        let view_data = self.view_data.clone();
        self.page
            .update(cx, |page, cx| page.set_view_data(view_data, cx));
        self.last_action = format!("Story applied the {} Variables state", state.label()).into();
        cx.notify();
    }

    pub(crate) fn handle_action(
        &mut self,
        page: Entity<VariablesPage>,
        action: &VariablesAction,
        cx: &mut Context<Storybook>,
    ) {
        match action {
            VariablesAction::CollectionSelected { collection_id } => {
                self.view_data.selected_collection_id = collection_id.clone();
                self.view_data.selected_group_id = "all".into();
                self.last_action = format!("Selected collection {collection_id}").into();
            }
            VariablesAction::CollectionRenameRequested {
                collection_id,
                name,
            } => {
                if let Some(collection) = self
                    .view_data
                    .collections
                    .iter_mut()
                    .find(|collection| collection.id == *collection_id)
                {
                    collection.name = name.clone();
                }
                self.last_action = format!("Renamed collection to {name}").into();
            }
            VariablesAction::GroupSelected { group_id } => {
                self.view_data.selected_group_id = group_id.clone();
                self.last_action = format!("Selected variable group {group_id}").into();
            }
            VariablesAction::SearchQueryChanged { query } => {
                self.last_action = if query.is_empty() {
                    "Cleared the variables search".into()
                } else {
                    format!("Filtered variables by “{query}”").into()
                };
            }
            VariablesAction::SearchOptionsRequested => {
                self.last_action = "Host opened Variables search options".into();
            }
            VariablesAction::CreateCollectionRequested => {
                let ordinal = self.view_data.collections.len() + 1;
                let collection_id: SharedString = format!("collection-{ordinal}").into();
                self.view_data.collections.push(VariablesCollection::new(
                    collection_id.clone(),
                    format!("Collection {ordinal}"),
                    self.view_data.variables.len(),
                ));
                self.view_data.selected_collection_id = collection_id.clone();
                self.last_action =
                    format!("Created and selected {collection_id} through the host adapter").into();
            }
            VariablesAction::CreateVariableRequested => {
                let ordinal = self.view_data.variables.len() + 1;
                let values = self
                    .view_data
                    .modes
                    .iter()
                    .map(|mode| VariableModeValue::new(mode.id.clone(), "0"))
                    .collect::<Vec<_>>();
                self.view_data.variables.push(VariableRow::new(
                    format!("spacing-{ordinal}"),
                    format!("Spacing {ordinal}"),
                    "all",
                    VariableKind::Number,
                    values,
                ));
                let variable_count = self.view_data.variables.len();
                if let Some(collection) = self
                    .view_data
                    .collections
                    .iter_mut()
                    .find(|collection| collection.id == self.view_data.selected_collection_id)
                {
                    collection.variable_count = variable_count;
                }
                if let Some(group) = self
                    .view_data
                    .groups
                    .iter_mut()
                    .find(|group| group.id.as_ref() == "all")
                {
                    group.variable_count = variable_count;
                }
                self.last_action =
                    format!("Created variable Spacing {ordinal} through the host adapter").into();
            }
            VariablesAction::ImportVariablesRequested => {
                let mode_id = self
                    .view_data
                    .modes
                    .first()
                    .map_or_else(|| "mode-1".into(), |mode| mode.id.clone());
                self.view_data.variables.push(VariableRow::new(
                    "imported-variable",
                    "Imported variable",
                    "all",
                    VariableKind::String,
                    [VariableModeValue::new(mode_id, "Imported")],
                ));
                self.last_action = "Imported variables through the host adapter".into();
            }
            VariablesAction::AddModeRequested => {
                let ordinal = self.view_data.modes.len() + 1;
                let mode_id: SharedString = format!("mode-{ordinal}").into();
                self.view_data.modes.push(VariablesMode::new(
                    mode_id.clone(),
                    format!("Mode {ordinal}"),
                ));
                for variable in &mut self.view_data.variables {
                    let value = match variable.kind {
                        VariableKind::Color => {
                            VariableModeValue::new(mode_id.clone(), "FFFFFF").color("FFFFFF")
                        }
                        VariableKind::Number => VariableModeValue::new(mode_id.clone(), "0"),
                        VariableKind::String => VariableModeValue::new(mode_id.clone(), "Text"),
                        VariableKind::Boolean => VariableModeValue::new(mode_id.clone(), "False"),
                    };
                    variable.values.push(value);
                }
                self.last_action = format!("Added Mode {ordinal} through the host adapter").into();
            }
            VariablesAction::ValueEditRequested {
                variable_id,
                mode_id,
            } => {
                if let Some(variable) = self
                    .view_data
                    .variables
                    .iter_mut()
                    .find(|variable| variable.id == *variable_id)
                {
                    let kind = variable.kind;
                    if let Some(value) = variable
                        .values
                        .iter_mut()
                        .find(|value| value.mode_id == *mode_id)
                    {
                        match kind {
                            VariableKind::Color => {
                                let next = if value.value.as_ref() == "FFFFFF" {
                                    "0D99FF"
                                } else {
                                    "FFFFFF"
                                };
                                value.value = next.into();
                                value.color_hex = Some(next.into());
                            }
                            VariableKind::Number => {
                                value.value = if value.value.as_ref() == "0" {
                                    "8".into()
                                } else {
                                    "0".into()
                                };
                            }
                            VariableKind::String => value.value = "Edited".into(),
                            VariableKind::Boolean => {
                                value.value = if value.value.as_ref() == "True" {
                                    "False".into()
                                } else {
                                    "True".into()
                                };
                            }
                        }
                    }
                }
                self.last_action =
                    format!("Edited {variable_id} in {mode_id} through the host adapter").into();
            }
            VariablesAction::VariableSettingsRequested { variable_id } => {
                self.last_action = format!("Host opened settings for {variable_id}").into();
            }
            VariablesAction::HelpRequested => {
                self.last_action = "Host opened Variables help".into();
            }
        }
        page.update(cx, |page, cx| {
            page.set_view_data(self.view_data.clone(), cx);
        });
        cx.notify();
    }
}

impl Storybook {
    pub(crate) fn render_variables_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-variables",
            self.variables_screen.page.clone().into_any_element(),
            self.variables_screen.last_action.clone(),
            cx,
        )
    }

    pub(crate) fn render_variables_knobs(&self, cx: &mut Context<Self>) -> AnyElement {
        knobs::knobs_panel(
            "variables-story-knobs",
            vec![knobs::enum_knob_row(
                "variables-knob-state",
                "NAMED STATE",
                VariablesNamedState::ALL.map(|state| KnobOption::new(state, state.label())),
                self.variables_screen.named_state,
                |this, state, _, cx| this.variables_screen.apply_named_state(state, cx),
                cx,
            )],
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variables_named_states_return_reseedable_fixtures() {
        let default = VariablesScreen::fixture(VariablesNamedState::Default);
        assert!(!default.variables.is_empty());
        let empty = VariablesScreen::fixture(VariablesNamedState::Empty);
        assert!(empty.variables.is_empty());
        assert!(
            empty
                .collections
                .iter()
                .any(|collection| collection.id == empty.selected_collection_id),
            "the empty state must keep a valid selected collection"
        );
    }
}
