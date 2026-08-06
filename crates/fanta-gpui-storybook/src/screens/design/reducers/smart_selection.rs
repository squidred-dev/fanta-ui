//! Smart Selection spacing and arrange operations plus the
//! native Add auto layout preflight.

use super::*;

pub(crate) fn reduce_spacing_and_arrange(
    screen: &mut DesignScreen,
    panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    cx: &mut Context<Storybook>,
) -> bool {
    match action {
        DesignPanelAction::SmartSelectionSpacingEditRequested {
            target,
            axis,
            value,
            phase,
        } => {
            let (current_target, can_edit) = {
                let panel = panel.read(cx);
                (
                    story_design_target(panel.inspection_context()),
                    panel.inspection_context().permissions().can_edit(),
                )
            };
            let projection = current_target
                .and_then(|current_target| screen.smart_selection_view_data(current_target));
            let accepted = apply_story_smart_selection_spacing_edit(
                &mut screen.smart_selection_spacing,
                &mut screen.smart_selection_edit_snapshots,
                projection.as_ref(),
                can_edit,
                StorySmartSelectionSpacingEdit {
                    target,
                    axis: *axis,
                    value: *value,
                    phase: *phase,
                },
            );
            if accepted
                && matches!(
                    phase,
                    DesignPanelEditPhase::Commit | DesignPanelEditPhase::Cancel
                )
            {
                screen.apply_inspection_context(panel, cx);
            }
            screen.last_action = if accepted {
                format!(
                    "Host accepted {phase:?} for {} = {value} on exact target {target:?}",
                    axis.label()
                )
                .into()
            } else {
                format!(
                        "Host rejected stale, read-only, invalid, or out-of-order {} edit for {target:?}",
                        axis.label()
                    )
                    .into()
            };
            cx.notify();
            return true;
        }
        DesignPanelAction::SmartSelectionArrangeRequested { target, operation } => {
            let (current_target, can_edit) = {
                let panel = panel.read(cx);
                (
                    story_design_target(panel.inspection_context()),
                    panel.inspection_context().permissions().can_edit(),
                )
            };
            let projection = current_target
                .and_then(|current_target| screen.smart_selection_view_data(current_target));
            let accepted = story_smart_selection_operation_is_current(
                projection.as_ref(),
                can_edit,
                target,
                *operation,
            );
            screen.last_action = if accepted {
                format!(
                    "Host applied {} to exact Smart Selection target {target:?}",
                    operation.label()
                )
                .into()
            } else {
                format!(
                    "Host rejected stale or unavailable {} for {target:?}",
                    operation.label()
                )
                .into()
            };
            cx.notify();
            return true;
        }
        _ => {}
    }
    false
}

pub(crate) fn add_auto_layout(
    screen: &mut DesignScreen,
    panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    cx: &mut Context<Storybook>,
) -> bool {
    if let DesignPanelAction::AddAutoLayoutRequested { target } = action {
        let (current_target, can_edit, structurally_eligible) = {
            let panel = panel.read(cx);
            let context = panel.inspection_context();
            let structurally_eligible = match screen.inspection_scenario {
                DesignInspectionScenario::AddAutoLayoutGroup => {
                    context.selection().kind()
                        == fanta_gpui::prelude::DesignPanelSelectionKind::Single
                        && context
                            .selection()
                            .items()
                            .first()
                            .is_some_and(|node| node.kind == DesignPanelNodeKind::Group)
                }
                DesignInspectionScenario::AddAutoLayoutMultiple => {
                    context.selection().kind()
                        == fanta_gpui::prelude::DesignPanelSelectionKind::Multiple
                }
                _ => false,
            };
            (
                story_design_target(context),
                context.permissions().can_edit(),
                structurally_eligible,
            )
        };
        let echo = apply_story_add_auto_layout(
            &mut screen.nodes,
            target,
            current_target.as_ref(),
            can_edit && structurally_eligible,
            &mut screen.next_auto_layout_id,
        );
        if let Some((selected_index, echo)) = echo {
            screen.selected_node = selected_index;
            screen.inspection_scenario = DesignInspectionScenario::EditableSingle;
            screen.apply_inspection_context(panel, cx);
            screen.last_action = match echo {
                StoryAddAutoLayoutEcho::Converted { node_id } => {
                    format!("Host converted {node_id} to an auto-layout frame").into()
                }
                StoryAddAutoLayoutEcho::Wrapped {
                    wrapper_id,
                    child_ids,
                } => format!("Host wrapped ordered selection {child_ids:?} in {wrapper_id}").into(),
            };
        } else {
            screen.last_action =
                format!("Host rejected stale or ineligible Add auto layout target {target:?}")
                    .into();
        }
        cx.notify();
        return true;
    }
    false
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StoryAddAutoLayoutEcho {
    Converted {
        node_id: SharedString,
    },
    Wrapped {
        wrapper_id: SharedString,
        child_ids: Vec<SharedString>,
    },
}

#[derive(Clone, Copy)]
pub(crate) struct StorySmartSelectionSpacingEdit<'a> {
    pub(crate) target: &'a DesignPanelTarget,
    pub(crate) axis: DesignSmartSelectionAxis,
    pub(crate) value: f32,
    pub(crate) phase: DesignPanelEditPhase,
}

pub(crate) fn apply_story_smart_selection_spacing_edit(
    spacing_values: &mut HashMap<DesignSmartSelectionAxis, DesignSmartSelectionSpacingValue>,
    snapshots: &mut HashMap<
        DesignSmartSelectionAxis,
        (DesignPanelTarget, DesignSmartSelectionSpacingValue),
    >,
    projection: Option<&DesignSmartSelectionViewData>,
    can_edit: bool,
    edit: StorySmartSelectionSpacingEdit<'_>,
) -> bool {
    let StorySmartSelectionSpacingEdit {
        target: action_target,
        axis,
        value,
        phase,
    } = edit;
    let projection_is_current = projection.is_some_and(|view_data| {
        view_data.target == *action_target
            && view_data.spacing_is_editable(axis)
            && view_data.is_valid()
    });
    let active_snapshot = snapshots.get(&axis).cloned();
    let lifecycle_is_valid = match phase {
        DesignPanelEditPhase::Begin => active_snapshot.is_none(),
        DesignPanelEditPhase::Preview => active_snapshot
            .as_ref()
            .is_some_and(|(active_target, _)| active_target == action_target),
        DesignPanelEditPhase::Commit => active_snapshot
            .as_ref()
            .is_none_or(|(active_target, _)| active_target == action_target),
        DesignPanelEditPhase::Cancel => active_snapshot
            .as_ref()
            .is_some_and(|(active_target, _)| active_target == action_target),
    };
    if !value.is_finite()
        || !lifecycle_is_valid
        || (phase != DesignPanelEditPhase::Cancel && (!can_edit || !projection_is_current))
    {
        return false;
    }

    match phase {
        DesignPanelEditPhase::Begin => {
            let original = spacing_values
                .get(&axis)
                .copied()
                .unwrap_or(DesignSmartSelectionSpacingValue::Mixed);
            snapshots.insert(axis, (action_target.clone(), original));
        }
        DesignPanelEditPhase::Preview => {
            spacing_values.insert(axis, DesignSmartSelectionSpacingValue::Uniform(value));
        }
        DesignPanelEditPhase::Commit => {
            spacing_values.insert(axis, DesignSmartSelectionSpacingValue::Uniform(value));
            snapshots.remove(&axis);
        }
        DesignPanelEditPhase::Cancel => {
            if let Some((_, original)) = snapshots.remove(&axis) {
                spacing_values.insert(axis, original);
            }
        }
    }
    true
}

pub(crate) fn story_smart_selection_operation_is_current(
    projection: Option<&DesignSmartSelectionViewData>,
    can_edit: bool,
    action_target: &DesignPanelTarget,
    operation: DesignSmartSelectionOperation,
) -> bool {
    can_edit
        && projection.is_some_and(|view_data| {
            view_data.target == *action_target
                && view_data.operation_is_available(operation)
                && view_data.is_valid()
        })
}

pub(crate) fn apply_story_add_auto_layout(
    nodes: &mut Vec<DesignPanelNode>,
    action_target: &DesignPanelTarget,
    current_target: Option<&DesignPanelTarget>,
    can_edit: bool,
    next_wrapper_id: &mut usize,
) -> Option<(usize, StoryAddAutoLayoutEcho)> {
    if !can_edit || current_target != Some(action_target) {
        return None;
    }
    let DesignPanelTarget::Nodes { node_ids } = action_target else {
        return None;
    };
    let unique_ids = node_ids.iter().collect::<HashSet<_>>();
    if node_ids.is_empty()
        || unique_ids.len() != node_ids.len()
        || node_ids.iter().any(|node_id| node_id.is_empty())
        || node_ids
            .iter()
            .any(|node_id| !nodes.iter().any(|node| node.id == *node_id))
    {
        return None;
    }

    if node_ids.len() == 1 {
        let node_index = nodes.iter().position(|node| node.id == node_ids[0])?;
        let node = &mut nodes[node_index];
        let is_active_owner = node.supports_auto_layout_container()
            && node
                .layout
                .as_ref()
                .is_some_and(|layout| layout.mode != DesignLayoutMode::None);
        if is_active_owner {
            return None;
        }
        if node.kind == DesignPanelNodeKind::Group || node.supports_auto_layout_container() {
            if node.kind == DesignPanelNodeKind::Group {
                node.kind = DesignPanelNodeKind::Frame;
                node.capabilities = None;
            }
            let layout = node.layout.get_or_insert_with(DesignLayout::default);
            layout.mode = DesignLayoutMode::Vertical;
            layout.horizontal_sizing = DesignSizingMode::Hug;
            layout.vertical_sizing = DesignSizingMode::Hug;
            node.name = format!("{} · Auto layout", node.name).into();
            return Some((
                node_index,
                StoryAddAutoLayoutEcho::Converted {
                    node_id: node.id.clone(),
                },
            ));
        }
    }

    let selected = node_ids
        .iter()
        .filter_map(|node_id| nodes.iter().find(|node| node.id == *node_id))
        .collect::<Vec<_>>();
    let min_x = selected
        .iter()
        .map(|node| node.x)
        .fold(f32::INFINITY, f32::min);
    let min_y = selected
        .iter()
        .map(|node| node.y)
        .fold(f32::INFINITY, f32::min);
    let max_x = selected
        .iter()
        .map(|node| node.x + node.width)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = selected
        .iter()
        .map(|node| node.y + node.height)
        .fold(f32::NEG_INFINITY, f32::max);
    let flow = if max_x - min_x >= max_y - min_y {
        DesignLayoutMode::Horizontal
    } else {
        DesignLayoutMode::Vertical
    };
    let wrapper_id = loop {
        let candidate = SharedString::from(format!("storybook-auto-layout-{}", *next_wrapper_id));
        *next_wrapper_id += 1;
        if nodes.iter().all(|node| node.id != candidate) {
            break candidate;
        }
    };
    let mut wrapper = DesignPanelNode::new(
        wrapper_id.clone(),
        format!("Auto layout wrapper · {} layers", node_ids.len()),
        DesignPanelNodeKind::Frame,
    )
    .with_layout_mode(flow);
    wrapper.x = min_x;
    wrapper.y = min_y;
    wrapper.width = (max_x - min_x).max(1.);
    wrapper.height = (max_y - min_y).max(1.);
    nodes.push(wrapper);
    Some((
        nodes.len() - 1,
        StoryAddAutoLayoutEcho::Wrapped {
            wrapper_id,
            child_ids: node_ids.clone(),
        },
    ))
}
