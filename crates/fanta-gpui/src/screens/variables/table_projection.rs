//! Derived presentation indices, rebuilt only when data or filters change.
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use super::{VariableKind, VariableModeValue, VariablesScreen, VariablesViewData};
use gpui::SharedString;

pub(super) struct TableProjection {
    names: HashMap<SharedString, SharedString>,
    row_indices: HashMap<SharedString, usize>,
    search_names: Vec<String>,
    filter_key: Option<(String, [bool; 4])>,
    pub visible_indices: Rc<Vec<usize>>,
    pub group_count: usize,
}

impl TableProjection {
    pub fn new(data: &VariablesViewData) -> Self {
        Self {
            names: data
                .variables
                .iter()
                .map(|row| (row.id.clone(), row.name.clone()))
                .collect(),
            row_indices: data
                .variables
                .iter()
                .enumerate()
                .map(|(index, row)| (row.id.clone(), index))
                .collect(),
            search_names: data
                .variables
                .iter()
                .map(|row| row.name.to_lowercase())
                .collect(),
            filter_key: None,
            visible_indices: Rc::default(),
            group_count: 0,
        }
    }

    pub fn alias_name(&self, id: &SharedString) -> SharedString {
        self.names.get(id).unwrap_or(id).clone()
    }

    /// Resolves a linked variable in the same mode using only the supplied
    /// snapshot. Invalid host data (missing targets/modes, mismatched kinds,
    /// or a cycle) has no resolved value; the source's literal fallback is
    /// retained for a future unlink but never presented as the linked value.
    pub fn resolve_alias_value<'a>(
        &self,
        data: &'a VariablesViewData,
        target_id: &SharedString,
        mode_id: &SharedString,
        kind: VariableKind,
    ) -> Option<&'a VariableModeValue> {
        let mut current_id = target_id.clone();
        let mut visited = HashSet::new();
        loop {
            let index = *self.row_indices.get(&current_id)?;
            if !visited.insert(index) {
                return None;
            }
            let variable = data.variables.get(index)?;
            if variable.kind != kind {
                return None;
            }
            let value = variable
                .values
                .iter()
                .find(|value| value.mode_id == *mode_id)?;
            if let Some(next_id) = &value.alias_id {
                current_id = next_id.clone();
            } else {
                return Some(value);
            }
        }
    }

    pub fn filter(&mut self, data: &VariablesViewData, query: &str, kinds: [bool; 4]) {
        if self
            .filter_key
            .as_ref()
            .is_some_and(|(previous, previous_kinds)| previous == query && *previous_kinds == kinds)
        {
            return;
        }
        self.filter_key = Some((query.to_owned(), kinds));
        let query = query.to_lowercase();
        let aggregate = data
            .groups
            .iter()
            .any(|group| group.id == data.selected_group_id && group.is_aggregate);
        self.group_count = 0;
        self.visible_indices = Rc::new(
            data.variables
                .iter()
                .enumerate()
                .filter_map(|(index, row)| {
                    if !aggregate && row.group_id != data.selected_group_id {
                        return None;
                    }
                    self.group_count += 1;
                    (kinds[VariablesScreen::kind_index(row.kind)]
                        && self.search_names[index].contains(&query))
                    .then_some(index)
                })
                .collect(),
        );
    }
}
