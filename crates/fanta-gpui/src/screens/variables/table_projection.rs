//! Derived presentation indices, rebuilt only when data or filters change.
use std::{collections::HashMap, rc::Rc};

use super::{VariablesScreen, VariablesViewData};
use gpui::SharedString;

pub(super) struct TableProjection {
    names: HashMap<SharedString, SharedString>,
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
