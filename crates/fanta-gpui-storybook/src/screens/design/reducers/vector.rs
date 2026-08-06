//! Text-path orientation and start, vector vertex and handle
//! edits, and stroke-endpoint swaps.

use super::*;

pub(crate) fn reduce(
    screen: &mut DesignScreen,
    action: &DesignPanelAction,
    node_index: usize,
    _cx: &mut Context<Storybook>,
) -> Option<NodeOutcome> {
    let node = &mut screen.nodes[node_index];
    match action {
        DesignPanelAction::TextPathFlipOrientationRequested { node_id } => {
            let flipped = apply_story_text_path_flip_orientation(node, node_id);
            screen.last_action = if flipped {
                let orientation = node
                    .text_path
                    .as_ref()
                    .map_or("Unknown", |text_path| text_path.orientation.label());
                format!("Host flipped text orientation to {orientation} on {node_id}").into()
            } else {
                format!("Host rejected unavailable or stale text-orientation flip on {node_id}")
                    .into()
            };
        }
        DesignPanelAction::TextPathStartChangeRequested {
            node_id,
            data,
            phase,
        } => {
            apply_story_text_path_edit_phase(
                node,
                &mut screen.text_path_edit_snapshots,
                node_id,
                *data,
                *phase,
            );
            screen.last_action =
                format!("Host observed {phase:?} for textPathStartData {data:?} in {node_id}")
                    .into();
        }
        DesignPanelAction::VectorVertexSelectionEditRequested {
            node_id,
            selected_vertex_ids,
            phase,
        } => {
            apply_story_vector_edit_phase(
                node,
                &mut screen.vector_edit_snapshots,
                node_id,
                StoryVectorEditChange::Selection(selected_vertex_ids.clone()),
                *phase,
            );
            screen.last_action = format!(
                "Host observed {phase:?} vector selection {selected_vertex_ids:?} in {node_id}"
            )
            .into();
        }
        DesignPanelAction::VectorVertexPositionEditRequested {
            node_id,
            vertex_ids,
            axis,
            value,
            phase,
        } => {
            apply_story_vector_edit_phase(
                node,
                &mut screen.vector_edit_snapshots,
                node_id,
                StoryVectorEditChange::Position {
                    vertex_ids: vertex_ids.clone(),
                    axis: *axis,
                    value: *value,
                },
                *phase,
            );
            screen.last_action = format!(
                "Host observed {phase:?} vector {axis:?} = {value} for {vertex_ids:?} in {node_id}"
            )
            .into();
        }
        DesignPanelAction::VectorVertexCornerRadiusEditRequested {
            node_id,
            vertex_ids,
            radius,
            phase,
        } => {
            apply_story_vector_edit_phase(
                node,
                &mut screen.vector_edit_snapshots,
                node_id,
                StoryVectorEditChange::CornerRadius {
                    vertex_ids: vertex_ids.clone(),
                    radius: *radius,
                },
                *phase,
            );
            screen.last_action = format!(
                "Host observed {phase:?} vertex radius {radius} for {vertex_ids:?} in {node_id}"
            )
            .into();
        }
        DesignPanelAction::VectorHandleMirroringEditRequested {
            node_id,
            vertex_ids,
            mirroring,
            phase,
        } => {
            apply_story_vector_edit_phase(
                node,
                &mut screen.vector_edit_snapshots,
                node_id,
                StoryVectorEditChange::HandleMirroring {
                    vertex_ids: vertex_ids.clone(),
                    mirroring: *mirroring,
                },
                *phase,
            );
            screen.last_action = format!(
                    "Host observed {phase:?} handle mirroring {mirroring:?} for {vertex_ids:?} in {node_id}"
                )
                .into();
        }
        DesignPanelAction::SwapStrokeEndpointsRequested { node_id } => {
            if let Some(stroke) = node.stroke.as_mut() {
                std::mem::swap(&mut stroke.start_cap, &mut stroke.end_cap);
                screen.last_action = format!("Host swapped stroke endpoints for {node_id}").into();
            } else {
                screen.last_action = format!("Ignored stroke endpoint swap for {node_id}").into();
            }
        }
        _ => return None,
    }
    Some(NodeOutcome::Applied)
}

pub(crate) fn apply_story_text_path_flip_orientation(
    node: &mut DesignPanelNode,
    node_id: &SharedString,
) -> bool {
    if node.id != *node_id || node.kind != DesignPanelNodeKind::TextPath {
        return false;
    }
    let Some(text_path) = node.text_path.as_mut() else {
        return false;
    };
    if !text_path.can_flip_orientation {
        return false;
    }
    text_path.orientation = text_path.orientation.toggled();
    true
}

pub(crate) fn apply_story_text_path_edit_phase(
    node: &mut DesignPanelNode,
    snapshots: &mut HashMap<SharedString, Option<DesignTextPathStartData>>,
    node_id: &SharedString,
    data: DesignTextPathStartData,
    phase: DesignPanelEditPhase,
) {
    match phase {
        DesignPanelEditPhase::Begin => {
            snapshots
                .entry(node_id.clone())
                .or_insert(node.text_path_start_data);
        }
        DesignPanelEditPhase::Preview => {
            node.text_path_start_data = Some(data);
        }
        DesignPanelEditPhase::Commit => {
            node.text_path_start_data = Some(data);
            snapshots.remove(node_id);
        }
        DesignPanelEditPhase::Cancel => {
            if let Some(original) = snapshots.remove(node_id) {
                node.text_path_start_data = original;
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum StoryVectorEditChange {
    Selection(Vec<SharedString>),
    Position {
        vertex_ids: Vec<SharedString>,
        axis: DesignVectorCoordinateAxis,
        value: f32,
    },
    CornerRadius {
        vertex_ids: Vec<SharedString>,
        radius: f32,
    },
    HandleMirroring {
        vertex_ids: Vec<SharedString>,
        mirroring: DesignHandleMirroring,
    },
}

pub(crate) fn apply_story_vector_edit_phase(
    node: &mut DesignPanelNode,
    snapshots: &mut HashMap<SharedString, Option<DesignVectorEditViewData>>,
    node_id: &SharedString,
    change: StoryVectorEditChange,
    phase: DesignPanelEditPhase,
) -> bool {
    match phase {
        DesignPanelEditPhase::Begin => {
            let Some(mut candidate) = node.vector_edit.clone() else {
                return false;
            };
            if !apply_story_vector_edit_change(&mut candidate, &change) {
                return false;
            }
            snapshots
                .entry(node_id.clone())
                .or_insert_with(|| node.vector_edit.clone());
            true
        }
        DesignPanelEditPhase::Preview => node
            .vector_edit
            .as_mut()
            .is_some_and(|view_data| apply_story_vector_edit_change(view_data, &change)),
        DesignPanelEditPhase::Commit => {
            let applied = node
                .vector_edit
                .as_mut()
                .is_some_and(|view_data| apply_story_vector_edit_change(view_data, &change));
            snapshots.remove(node_id);
            applied
        }
        DesignPanelEditPhase::Cancel => snapshots.remove(node_id).is_some_and(|original| {
            node.vector_edit = original;
            true
        }),
    }
}

pub(crate) fn apply_story_vector_edit_change(
    view_data: &mut DesignVectorEditViewData,
    change: &StoryVectorEditChange,
) -> bool {
    if !view_data.is_valid() || view_data.read_only {
        return false;
    }
    match change {
        StoryVectorEditChange::Selection(selected_vertex_ids) => {
            if selected_vertex_ids.iter().enumerate().any(|(index, id)| {
                selected_vertex_ids[index + 1..].contains(id)
                    || view_data.vertex(id.as_ref()).is_none()
            }) {
                return false;
            }
            for vertex in &mut view_data.vertices {
                vertex.selected = selected_vertex_ids.contains(&vertex.id);
            }
            true
        }
        StoryVectorEditChange::Position {
            vertex_ids,
            axis,
            value,
        } => {
            if !value.is_finite()
                || !view_data.can_edit_coordinates()
                || !story_vector_targets_current_selection(view_data, vertex_ids)
            {
                return false;
            }
            for vertex in &mut view_data.vertices {
                if vertex_ids.contains(&vertex.id) {
                    match axis {
                        DesignVectorCoordinateAxis::X => vertex.position.x = *value,
                        DesignVectorCoordinateAxis::Y => vertex.position.y = *value,
                    }
                }
            }
            true
        }
        StoryVectorEditChange::CornerRadius { vertex_ids, radius } => {
            if !radius.is_finite()
                || *radius < 0.
                || !view_data.can_edit_corner_radius()
                || !story_vector_targets_current_selection(view_data, vertex_ids)
            {
                return false;
            }
            for vertex in &mut view_data.vertices {
                if vertex_ids.contains(&vertex.id) {
                    vertex.corner_radius = Some(*radius);
                }
            }
            true
        }
        StoryVectorEditChange::HandleMirroring {
            vertex_ids,
            mirroring,
        } => {
            if !view_data.can_edit_handle_mirroring()
                || !story_vector_targets_current_selection(view_data, vertex_ids)
            {
                return false;
            }
            for vertex in &mut view_data.vertices {
                if vertex_ids.contains(&vertex.id) {
                    vertex.handle_mirroring = Some(*mirroring);
                }
            }
            true
        }
    }
}

pub(crate) fn story_vector_targets_current_selection(
    view_data: &DesignVectorEditViewData,
    vertex_ids: &[SharedString],
) -> bool {
    let selected = view_data.selected_vertex_ids();
    view_data.contains_exact_vertices(vertex_ids)
        && selected.len() == vertex_ids.len()
        && selected.iter().all(|id| vertex_ids.contains(id))
}
