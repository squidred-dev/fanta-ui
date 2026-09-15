//! Selected-node header commands and Selection colors:
//! occurrence edits, styles, and variables across the exact selection.

use super::*;

pub(crate) fn acknowledge_header_command(
    screen: &mut DesignScreen,
    panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    cx: &mut Context<Storybook>,
) -> bool {
    if let DesignPanelAction::SelectionHeaderCommandRequested { target, command } = action {
        screen.harness.last_action =
            format!("Host received selected-node command {command:?} for {target:?}").into();
        screen.apply_inspection_context(panel, cx);
        cx.notify();
        return true;
    }
    false
}

pub(crate) fn select_color_occurrences(
    screen: &mut DesignScreen,
    panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    cx: &mut Context<Storybook>,
) -> bool {
    if let DesignPanelAction::SelectionColorOccurrencesSelectRequested {
        target,
        selection_color_id,
        paint_references,
    } = action
    {
        if story_selection_row_is_current(
            &screen.host.nodes,
            target,
            selection_color_id,
            paint_references,
        ) {
            screen.harness.last_action = format!(
                    "Host selected all {} occurrences of selection paint {selection_color_id} for {target:?}",
                    paint_references.len()
                )
                .into();
        } else {
            screen.harness.last_action =
                format!("Ignored stale selection-paint occurrences for {target:?}").into();
        }
        screen.apply_inspection_context(panel, cx);
        cx.notify();
        return true;
    }
    false
}

pub(crate) fn edit_selection_color_paint(
    screen: &mut DesignScreen,
    panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    cx: &mut Context<Storybook>,
) -> bool {
    if let DesignPanelAction::SelectionColorPaintEditRequested {
        target,
        selection_color_id,
        paint_references,
        edit,
        phase,
    } = action
    {
        let edit_target = StorySelectionColorEditTarget::new(target, selection_color_id.clone());
        let references_are_current = story_selection_row_is_current(
            &screen.host.nodes,
            target,
            selection_color_id,
            paint_references,
        );
        match phase {
            DesignPanelEditPhase::Begin if references_are_current => {
                screen
                    .edits
                    .selection_color_edit_snapshots
                    .entry(edit_target.clone())
                    .or_insert_with(|| screen.host.nodes.clone());
            }
            DesignPanelEditPhase::Preview | DesignPanelEditPhase::Commit
                if references_are_current =>
            {
                let mut candidate_nodes = screen.host.nodes.clone();
                let applied = paint_references.iter().all(|reference| {
                    candidate_nodes
                        .iter_mut()
                        .find(|node| node.id == reference.node_id)
                        .and_then(|node| story_selection_reference_paint_mut(node, reference))
                        .is_some_and(|paint| {
                            let Some(edit) = story_selection_paint_edit_for_occurrence(paint, edit)
                            else {
                                return false;
                            };
                            paint.apply_edit(&edit)
                        })
                });
                if applied {
                    screen.host.nodes = candidate_nodes;
                }
            }
            DesignPanelEditPhase::Cancel => {
                if let Some(original) = screen
                    .edits
                    .selection_color_edit_snapshots
                    .remove(&edit_target)
                {
                    screen.host.nodes = original;
                }
            }
            DesignPanelEditPhase::Begin
            | DesignPanelEditPhase::Preview
            | DesignPanelEditPhase::Commit => {}
        }
        if *phase == DesignPanelEditPhase::Commit {
            screen
                .edits
                .selection_color_edit_snapshots
                .remove(&edit_target);
        }
        screen.harness.last_action = if references_are_current {
            format!(
                    "Host observed {phase:?} selection-paint {:?} for {selection_color_id} on {target:?}",
                    edit.property
                )
                .into()
        } else {
            format!("Ignored stale selection-paint edit for {target:?}").into()
        };
        screen.apply_inspection_context(panel, cx);
        cx.notify();
        return true;
    }
    false
}

pub(crate) fn edit_selection_color(
    screen: &mut DesignScreen,
    panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    cx: &mut Context<Storybook>,
) -> bool {
    if let DesignPanelAction::SelectionColorEditRequested {
        target,
        selection_color_id,
        color,
        paint_references,
        phase,
    } = action
    {
        let edit_target = StorySelectionColorEditTarget::new(target, selection_color_id.clone());
        match phase {
            DesignPanelEditPhase::Begin => {
                let original = screen.host.nodes.clone();
                screen
                    .edits
                    .selection_color_edit_snapshots
                    .entry(edit_target.clone())
                    .or_insert(original);
            }
            DesignPanelEditPhase::Cancel => {
                if let Some(original) = screen
                    .edits
                    .selection_color_edit_snapshots
                    .remove(&edit_target)
                {
                    screen.host.nodes = original;
                }
            }
            DesignPanelEditPhase::Preview | DesignPanelEditPhase::Commit => {}
        }
        if matches!(
            phase,
            DesignPanelEditPhase::Preview | DesignPanelEditPhase::Commit
        ) {
            for node in &mut screen.host.nodes {
                if let Some((index, selection_color)) = node
                    .selection_color_aggregate
                    .colors
                    .iter_mut()
                    .enumerate()
                    .find(|(_, selection_color)| selection_color.id == *selection_color_id)
                {
                    selection_color.color = *color;
                    if let Some(legacy_paint) = node.selection_colors.get_mut(index) {
                        legacy_paint.apply_edit(&DesignPaintEdit {
                            property: DesignPaintProperty::Color,
                            value: DesignPaintValue::Color(*color),
                        });
                    }
                }
            }

            for reference in paint_references {
                let Some(node) = screen
                    .host
                    .nodes
                    .iter_mut()
                    .find(|node| node.id == reference.node_id)
                else {
                    continue;
                };
                let paints = match reference.collection {
                    DesignSelectionPaintCollection::Fill => Some(&mut node.fills),
                    DesignSelectionPaintCollection::Stroke => {
                        node.stroke.as_mut().map(|stroke| &mut stroke.paints)
                    }
                };
                let Some(paints) = paints else {
                    continue;
                };
                let paint_index = if reference.paint_id.is_empty() {
                    (reference.paint_index < paints.len()).then_some(reference.paint_index)
                } else {
                    paints
                        .iter()
                        .position(|paint| paint.id == reference.paint_id)
                };
                let Some(paint) = paint_index.and_then(|index| paints.get_mut(index)) else {
                    continue;
                };
                let property = reference.gradient_stop_id.as_ref().map_or(
                    DesignPaintProperty::Color,
                    |stop_id| DesignPaintProperty::GradientStopColor {
                        stop_id: stop_id.clone(),
                        index: reference.gradient_stop_index.unwrap_or_default(),
                    },
                );
                paint.apply_edit(&DesignPaintEdit {
                    property,
                    value: DesignPaintValue::Color(*color),
                });
            }
        }
        if *phase == DesignPanelEditPhase::Commit {
            screen
                .edits
                .selection_color_edit_snapshots
                .remove(&edit_target);
        }

        screen.harness.last_action = format!(
            "Host observed {phase:?} for selection color {selection_color_id} = #{} on {target:?}",
            color.hex()
        )
        .into();
        screen.apply_inspection_context(panel, cx);
        cx.notify();
        return true;
    }
    false
}

pub(crate) fn reduce_color_resources(
    screen: &mut DesignScreen,
    panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    cx: &mut Context<Storybook>,
) -> bool {
    let selection_resource_handled = match action {
        DesignPanelAction::SelectionColorPaintStyleApplyRequested {
            target,
            selection_color_id,
            paint_references,
            style,
        } => {
            let style_data = screen.host.paint_styles.style(style).cloned();
            let collection_targets = story_selection_collection_targets(paint_references);
            let applicable = story_selection_references_are_current(
                &screen.host.nodes,
                target,
                paint_references,
            ) && style_data.as_ref().is_some_and(|style| {
                style.import_state == DesignPaintStyleImportState::Imported
                    && !style.paints.is_empty()
            }) && paint_references.iter().all(|reference| {
                screen
                    .host
                    .nodes
                    .iter()
                    .find(|node| node.id == reference.node_id)
                    .and_then(|node| story_selection_reference_paint(node, reference))
                    .is_some_and(|paint| !paint.read_only)
            });
            if applicable {
                let style_data = style_data.expect("applicable Selection Paint style is present");
                for (node_id, collection) in &collection_targets {
                    if let Some(node) = screen
                        .host
                        .nodes
                        .iter_mut()
                        .find(|node| node.id == *node_id)
                    {
                        apply_story_paint_style(
                            node,
                            story_design_collection(*collection),
                            style,
                            style_data.clone(),
                        );
                    }
                }
            }
            screen.harness.last_action = if applicable {
                format!(
                        "Host applied Paint style {} to {} exact Selection-color collection{} for {selection_color_id} on {target:?}",
                        style.style_id,
                        collection_targets.len(),
                        if collection_targets.len() == 1 { "" } else { "s" },
                    )
                    .into()
            } else {
                format!("Host rejected stale Selection Paint-style apply for {selection_color_id}")
                    .into()
            };
            true
        }
        DesignPanelAction::SelectionColorPaintStyleImportRequested {
            target,
            selection_color_id,
            paint_references,
            style,
        } => {
            let current = story_selection_references_are_current(
                &screen.host.nodes,
                target,
                paint_references,
            );
            let imported = current
                && screen
                    .host
                    .paint_styles
                    .style_mut(style)
                    .filter(|style| style.import_state == DesignPaintStyleImportState::Available)
                    .map(|style| {
                        style.import_state = DesignPaintStyleImportState::Imported;
                    })
                    .is_some();
            screen.harness.last_action = if imported {
                format!(
                    "Host imported Selection Paint style {} for {selection_color_id}",
                    style.style_id
                )
                .into()
            } else {
                format!("Host rejected stale Selection Paint-style import for {selection_color_id}")
                    .into()
            };
            true
        }
        DesignPanelAction::SelectionColorPaintStyleCreateRequested {
            target,
            selection_color_id,
            paint_references,
            paints,
        } => {
            let collection_targets = story_selection_collection_targets(paint_references);
            let creatable = !paints.is_empty()
                && story_selection_references_are_current(
                    &screen.host.nodes,
                    target,
                    paint_references,
                )
                && collection_targets.iter().all(|(node_id, collection)| {
                    screen
                        .host
                        .nodes
                        .iter()
                        .find(|node| node.id == *node_id)
                        .is_some_and(|node| {
                            story_selection_collection_style_binding(node, *collection).is_none()
                        })
                });
            screen.harness.last_action = if creatable {
                format!(
                        "Host opened Selection Paint-style creation with {} ordered paint{} for {selection_color_id} on {target:?}",
                        paints.len(),
                        if paints.len() == 1 { "" } else { "s" }
                    )
                    .into()
            } else {
                format!(
                    "Host rejected stale Selection Paint-style creation for {selection_color_id}"
                )
                .into()
            };
            true
        }
        DesignPanelAction::SelectionColorPaintStyleDetachRequested {
            target,
            selection_color_id,
            paint_references,
            style,
        } => {
            let collection_targets = story_selection_collection_targets(paint_references);
            let detachable = story_selection_references_are_current(
                &screen.host.nodes,
                target,
                paint_references,
            ) && collection_targets.iter().all(|(node_id, collection)| {
                screen
                    .host
                    .nodes
                    .iter()
                    .find(|node| node.id == *node_id)
                    .and_then(|node| story_selection_collection_style_binding(node, *collection))
                    .is_some_and(|binding| binding.can_detach && binding.selection == *style)
            });
            if detachable {
                for (node_id, collection) in &collection_targets {
                    if let Some(node) = screen
                        .host
                        .nodes
                        .iter_mut()
                        .find(|node| node.id == *node_id)
                    {
                        match collection {
                            DesignSelectionPaintCollection::Fill => {
                                node.fill_style_binding = None;
                            }
                            DesignSelectionPaintCollection::Stroke => {
                                node.stroke_style_binding = None;
                            }
                        }
                    }
                }
            }
            screen.harness.last_action = if detachable {
                format!(
                    "Host detached Selection Paint style {} for {selection_color_id} on {target:?}",
                    style.style_id
                )
                .into()
            } else {
                format!("Host rejected stale Selection Paint-style detach for {selection_color_id}")
                    .into()
            };
            true
        }
        DesignPanelAction::SelectionColorVariableApplyRequested {
            target,
            selection_color_id,
            paint_references,
            variable_id,
        } => {
            let variable = screen
                .host
                .paint_variables
                .variable(variable_id.as_ref())
                .cloned();
            let applicable = story_selection_references_are_current(
                &screen.host.nodes,
                target,
                paint_references,
            ) && variable.as_ref().is_some_and(|variable| {
                variable.disabled_reason.is_none()
                    && matches!(
                        variable.import_state,
                        DesignVariableImportState::Local | DesignVariableImportState::Imported
                    )
            }) && paint_references.iter().all(|reference| {
                screen
                    .host
                    .nodes
                    .iter()
                    .find(|node| node.id == reference.node_id)
                    .is_some_and(|node| {
                        story_selection_collection_style_binding(node, reference.collection)
                            .is_none()
                            && story_selection_reference_paint(node, reference)
                                .is_some_and(|paint| !paint.read_only)
                    })
            });
            if applicable {
                let variable = variable.expect("applicable Selection Color variable is present");
                for reference in paint_references {
                    if let Some(node) = screen
                        .host
                        .nodes
                        .iter_mut()
                        .find(|node| node.id == reference.node_id)
                        && let Some(paint) = story_selection_reference_paint_mut(node, reference)
                    {
                        apply_story_paint_variable(
                            paint,
                            &story_selection_color_target(reference),
                            &variable,
                        );
                    }
                }
            }
            screen.harness.last_action = if applicable {
                format!(
                        "Host bound Selection Color variable {variable_id} to {} exact occurrence{} for {selection_color_id} on {target:?}",
                        paint_references.len(),
                        if paint_references.len() == 1 { "" } else { "s" },
                    )
                    .into()
            } else {
                format!(
                    "Host rejected stale Selection Color-variable apply for {selection_color_id}"
                )
                .into()
            };
            true
        }
        DesignPanelAction::SelectionColorVariableImportRequested {
            target,
            selection_color_id,
            paint_references,
            variable_id,
        } => {
            let current = story_selection_references_are_current(
                &screen.host.nodes,
                target,
                paint_references,
            );
            let imported = current
                && screen
                    .host
                    .paint_variables
                    .variable_mut(variable_id.as_ref())
                    .filter(|variable| {
                        variable.disabled_reason.is_none()
                            && variable.import_state == DesignVariableImportState::Available
                    })
                    .map(|variable| {
                        variable.import_state = DesignVariableImportState::Imported;
                    })
                    .is_some();
            screen.harness.last_action = if imported {
                format!(
                    "Host imported Selection Color variable {variable_id} for {selection_color_id}"
                )
                .into()
            } else {
                format!(
                    "Host rejected stale Selection Color-variable import for {selection_color_id}"
                )
                .into()
            };
            true
        }
        DesignPanelAction::SelectionColorVariableCreateRequested {
            target,
            selection_color_id,
            paint_references,
            color,
        } => {
            let creatable = story_selection_references_are_current(
                &screen.host.nodes,
                target,
                paint_references,
            ) && paint_references.iter().all(|reference| {
                screen
                    .host
                    .nodes
                    .iter()
                    .find(|node| node.id == reference.node_id)
                    .is_some_and(|node| {
                        story_selection_collection_style_binding(node, reference.collection)
                            .is_none()
                            && story_selection_reference_paint(node, reference).is_some_and(
                                |paint| {
                                    !paint.read_only
                                        && match (&paint.payload, &reference.gradient_stop_id) {
                                            (DesignPaintPayload::Solid(solid), None) => {
                                                solid.binding.is_none()
                                            }
                                            (
                                                DesignPaintPayload::Gradient(gradient),
                                                Some(stop_id),
                                            ) => {
                                                let stop = if stop_id.is_empty() {
                                                    reference
                                                        .gradient_stop_index
                                                        .and_then(|index| gradient.stops.get(index))
                                                } else {
                                                    gradient
                                                        .stops
                                                        .iter()
                                                        .find(|stop| stop.id == *stop_id)
                                                };
                                                stop.is_some_and(|stop| stop.binding.is_none())
                                            }
                                            _ => false,
                                        }
                                },
                            )
                    })
            });
            screen.harness.last_action = if creatable {
                format!(
                        "Host opened Selection Color-variable creation for #{} and {selection_color_id} on {target:?}",
                        color.hex()
                    )
                    .into()
            } else {
                format!(
                    "Host rejected stale Selection Color-variable creation for {selection_color_id}"
                )
                .into()
            };
            true
        }
        DesignPanelAction::SelectionColorVariableDetachRequested {
            target,
            selection_color_id,
            paint_references,
            variable_id,
        } => {
            let detachable = story_selection_references_are_current(
                &screen.host.nodes,
                target,
                paint_references,
            ) && paint_references.iter().all(|reference| {
                screen
                    .host
                    .nodes
                    .iter()
                    .find(|node| node.id == reference.node_id)
                    .is_some_and(|node| {
                        story_selection_collection_style_binding(node, reference.collection)
                            .is_none()
                            && story_selection_reference_paint(node, reference).is_some_and(
                                |paint| {
                                    let target = story_selection_color_target(reference);
                                    match (&paint.payload, target) {
                                        (
                                            DesignPaintPayload::Solid(solid),
                                            DesignPaintColorTarget::Solid,
                                        ) => solid.binding.as_ref().is_some_and(|binding| {
                                            binding.variable_id == *variable_id
                                        }),
                                        (
                                            DesignPaintPayload::Gradient(gradient),
                                            DesignPaintColorTarget::GradientStop { stop_id, index },
                                        ) => {
                                            let stop = if stop_id.is_empty() {
                                                gradient.stops.get(index)
                                            } else {
                                                gradient
                                                    .stops
                                                    .iter()
                                                    .find(|stop| stop.id == stop_id)
                                            };
                                            stop.is_some_and(|stop| {
                                                stop.binding.as_ref().is_some_and(|binding| {
                                                    binding.variable_id == *variable_id
                                                })
                                            })
                                        }
                                        _ => false,
                                    }
                                },
                            )
                    })
            });
            if detachable {
                for reference in paint_references {
                    if let Some(node) = screen
                        .host
                        .nodes
                        .iter_mut()
                        .find(|node| node.id == reference.node_id)
                        && let Some(paint) = story_selection_reference_paint_mut(node, reference)
                    {
                        detach_story_paint_variable(
                            paint,
                            &story_selection_color_target(reference),
                            variable_id,
                        );
                    }
                }
            }
            screen.harness.last_action = if detachable {
                format!(
                        "Host detached Selection Color variable {variable_id} for {selection_color_id} on {target:?}"
                    )
                    .into()
            } else {
                format!(
                    "Host rejected stale Selection Color-variable detach for {selection_color_id}"
                )
                .into()
            };
            true
        }
        _ => false,
    };
    if selection_resource_handled {
        screen.apply_inspection_context(panel, cx);
        cx.notify();
        return true;
    }
    false
}

pub(crate) fn story_selection_reference_paint<'a>(
    node: &'a DesignPanelNode,
    reference: &DesignSelectionPaintReference,
) -> Option<&'a DesignPaint> {
    if node.id != reference.node_id {
        return None;
    }
    let paints = match reference.collection {
        DesignSelectionPaintCollection::Fill => Some(node.fills.as_slice()),
        DesignSelectionPaintCollection::Stroke => {
            node.stroke.as_ref().map(|stroke| stroke.paints.as_slice())
        }
    }?;
    let resolved = if reference.paint_id.is_empty() {
        (reference.paint_index < paints.len()).then_some(reference.paint_index)
    } else {
        paints
            .iter()
            .position(|paint| paint.id == reference.paint_id)
    }?;
    paints.get(resolved)
}

pub(crate) fn story_selection_reference_paint_mut<'a>(
    node: &'a mut DesignPanelNode,
    reference: &DesignSelectionPaintReference,
) -> Option<&'a mut DesignPaint> {
    if node.id != reference.node_id {
        return None;
    }
    let paints = match reference.collection {
        DesignSelectionPaintCollection::Fill => Some(&mut node.fills),
        DesignSelectionPaintCollection::Stroke => {
            node.stroke.as_mut().map(|stroke| &mut stroke.paints)
        }
    }?;
    let resolved = if reference.paint_id.is_empty() {
        (reference.paint_index < paints.len()).then_some(reference.paint_index)
    } else {
        paints
            .iter()
            .position(|paint| paint.id == reference.paint_id)
    }?;
    paints.get_mut(resolved)
}

pub(crate) fn story_selection_paint_edit_for_occurrence(
    paint: &DesignPaint,
    edit: &DesignPaintEdit,
) -> Option<DesignPaintEdit> {
    let mut edit = edit.clone();
    match &mut edit.property {
        DesignPaintProperty::GradientStopColor { stop_id, index }
        | DesignPaintProperty::GradientStopPosition { stop_id, index }
        | DesignPaintProperty::GradientStopRemove { stop_id, index } => {
            let DesignPaintPayload::Gradient(gradient) = &paint.payload else {
                return None;
            };
            let target_stop = gradient.stops.get(*index)?;
            *stop_id = target_stop.id.clone();
        }
        _ => {}
    }
    match (&edit.property, &mut edit.value) {
        (DesignPaintProperty::GradientStopAdd, DesignPaintValue::GradientStop(stop)) => {
            let DesignPaintPayload::Gradient(gradient) = &paint.payload else {
                return None;
            };
            stop.id = if paint.id.is_empty() {
                "".into()
            } else {
                format!("{}-selection-stop-{}", paint.id, gradient.stops.len()).into()
            };
        }
        (
            DesignPaintProperty::Payload,
            DesignPaintValue::Payload(DesignPaintPayload::Gradient(gradient)),
        ) => {
            for (index, stop) in gradient.stops.iter_mut().enumerate() {
                stop.id = if paint.id.is_empty() {
                    "".into()
                } else {
                    format!("{}-selection-stop-{index}", paint.id).into()
                };
            }
        }
        _ => {}
    }
    Some(edit)
}

pub(crate) fn story_selection_color_target(
    reference: &DesignSelectionPaintReference,
) -> DesignPaintColorTarget {
    reference
        .gradient_stop_id
        .as_ref()
        .map_or(DesignPaintColorTarget::Solid, |stop_id| {
            DesignPaintColorTarget::GradientStop {
                stop_id: stop_id.clone(),
                index: reference.gradient_stop_index.unwrap_or_default(),
            }
        })
}

pub(crate) fn story_selection_references_are_current(
    nodes: &[DesignPanelNode],
    target: &DesignPanelTarget,
    references: &[DesignSelectionPaintReference],
) -> bool {
    let DesignPanelTarget::Nodes { node_ids } = target else {
        return false;
    };
    !node_ids.is_empty()
        && !references.is_empty()
        && node_ids
            .iter()
            .all(|node_id| nodes.iter().any(|node| node.id == *node_id))
        && references.iter().all(|reference| {
            node_ids.contains(&reference.node_id)
                && nodes
                    .iter()
                    .find(|node| node.id == reference.node_id)
                    .and_then(|node| story_selection_reference_paint(node, reference))
                    .is_some_and(|paint| {
                        if let Some(stop_id) = &reference.gradient_stop_id {
                            let DesignPaintPayload::Gradient(gradient) = &paint.payload else {
                                return false;
                            };
                            if stop_id.is_empty() {
                                reference
                                    .gradient_stop_index
                                    .is_some_and(|index| index < gradient.stops.len())
                            } else {
                                gradient.stops.iter().any(|stop| stop.id == *stop_id)
                            }
                        } else {
                            matches!(
                                &paint.payload,
                                DesignPaintPayload::Solid(_) | DesignPaintPayload::Gradient(_)
                            )
                        }
                    })
        })
}

pub(crate) fn story_selection_row_is_current(
    nodes: &[DesignPanelNode],
    target: &DesignPanelTarget,
    selection_color_id: &SharedString,
    references: &[DesignSelectionPaintReference],
) -> bool {
    if !story_selection_references_are_current(nodes, target, references) {
        return false;
    }
    let DesignPanelTarget::Nodes { node_ids } = target else {
        return false;
    };
    let selected = node_ids
        .iter()
        .filter_map(|node_id| nodes.iter().find(|node| node.id == *node_id))
        .collect::<Vec<_>>();
    selected.len() == node_ids.len()
        && aggregate_story_selection_colors(selected)
            .color(selection_color_id.as_ref())
            .is_some_and(|row| row.paint_references == references)
}

pub(crate) fn story_selection_collection_targets(
    references: &[DesignSelectionPaintReference],
) -> Vec<(SharedString, DesignSelectionPaintCollection)> {
    let mut targets = Vec::new();
    for reference in references {
        let target = (reference.node_id.clone(), reference.collection);
        if !targets.contains(&target) {
            targets.push(target);
        }
    }
    targets
}

pub(crate) fn story_selection_collection_style_binding(
    node: &DesignPanelNode,
    collection: DesignSelectionPaintCollection,
) -> Option<&DesignPaintStyleBinding> {
    match collection {
        DesignSelectionPaintCollection::Fill => node.fill_style_binding.as_ref(),
        DesignSelectionPaintCollection::Stroke => node.stroke_style_binding.as_ref(),
    }
}

pub(crate) fn story_design_collection(
    collection: DesignSelectionPaintCollection,
) -> DesignPanelCollection {
    match collection {
        DesignSelectionPaintCollection::Fill => DesignPanelCollection::Fill,
        DesignSelectionPaintCollection::Stroke => DesignPanelCollection::Stroke,
    }
}

pub(crate) fn aggregate_story_selection_colors<'a>(
    nodes: impl IntoIterator<Item = &'a DesignPanelNode>,
) -> DesignSelectionColors {
    aggregate_story_selection_colors_for_collections(nodes, true, true)
}

pub(crate) fn aggregate_story_selection_colors_for_collections<'a>(
    nodes: impl IntoIterator<Item = &'a DesignPanelNode>,
    include_fills: bool,
    include_strokes: bool,
) -> DesignSelectionColors {
    let mut colors = Vec::<DesignSelectionColor>::new();
    for node in nodes {
        if include_fills {
            push_story_selection_paint_colors(
                &mut colors,
                node,
                DesignSelectionPaintCollection::Fill,
                &node.fills,
            );
        }
        if include_strokes && let Some(stroke) = node.stroke.as_ref() {
            push_story_selection_paint_colors(
                &mut colors,
                node,
                DesignSelectionPaintCollection::Stroke,
                &stroke.paints,
            );
        }
    }
    DesignSelectionColors::new(colors)
}

pub(crate) fn push_story_selection_paint_colors(
    colors: &mut Vec<DesignSelectionColor>,
    node: &DesignPanelNode,
    collection: DesignSelectionPaintCollection,
    paints: &[DesignPaint],
) {
    let style_binding = match collection {
        DesignSelectionPaintCollection::Fill => node.fill_style_binding.clone(),
        DesignSelectionPaintCollection::Stroke => node.stroke_style_binding.clone(),
    };
    for (paint_index, paint) in paints.iter().enumerate() {
        let paint_reference = || {
            DesignSelectionPaintReference::paint(
                node.id.clone(),
                collection,
                paint.id.clone(),
                paint_index,
            )
        };
        match &paint.payload {
            DesignPaintPayload::Solid(solid) => {
                push_story_selection_color(
                    colors,
                    solid.color,
                    paint_reference(),
                    paint,
                    paints,
                    style_binding.clone(),
                    solid.binding.clone(),
                    paint.read_only,
                );
            }
            DesignPaintPayload::Gradient(_) => push_story_selection_color(
                colors,
                paint.color,
                paint_reference(),
                paint,
                paints,
                style_binding.clone(),
                None,
                paint.read_only,
            ),
            DesignPaintPayload::Pattern(_)
            | DesignPaintPayload::Image(_)
            | DesignPaintPayload::Video(_)
            | DesignPaintPayload::Shader(_)
            | DesignPaintPayload::Unsupported(_) => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn push_story_selection_color(
    colors: &mut Vec<DesignSelectionColor>,
    color: DesignColor,
    reference: DesignSelectionPaintReference,
    paint: &DesignPaint,
    style_paints: &[DesignPaint],
    style_binding: Option<DesignPaintStyleBinding>,
    binding: Option<fanta_gpui::prelude::DesignPaintBinding>,
    read_only: bool,
) {
    if let Some(existing) = colors.iter_mut().find(|existing| {
        existing.color == color
            && story_selection_paint_semantics_equal(&existing.paint, paint)
            && story_selection_paint_collections_equal(&existing.style_paints, style_paints)
            && existing.style_binding.as_ref() == style_binding.as_ref()
            && existing.binding.as_ref() == binding.as_ref()
            && existing.read_only == read_only
    }) {
        existing.paint_references.push(reference);
        existing.occurrence_count = existing.occurrence_count.saturating_add(1);
        return;
    }

    let mut selection_color =
        DesignSelectionColor::new(story_selection_color_id(&reference), color, [reference])
            .with_paint(paint.clone())
            .with_style_context(style_paints.iter().cloned(), style_binding)
            .read_only(read_only);
    if let Some(binding) = binding {
        selection_color = selection_color.with_binding(binding);
    }
    colors.push(selection_color);
}

pub(crate) fn story_selection_color_id(reference: &DesignSelectionPaintReference) -> SharedString {
    let paint_identity = if reference.paint_id.is_empty() {
        format!("index-{}", reference.paint_index)
    } else {
        reference.paint_id.to_string()
    };
    format!(
        "selection-color-{}-{}-{paint_identity}",
        reference.node_id,
        reference.collection.label().to_ascii_lowercase(),
    )
    .into()
}

pub(crate) fn story_selection_paint_collections_equal(
    left: &[DesignPaint],
    right: &[DesignPaint],
) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| story_selection_paint_semantics_equal(left, right))
}

pub(crate) fn story_selection_paint_semantics_equal(
    left: &DesignPaint,
    right: &DesignPaint,
) -> bool {
    normalize_story_selection_paint(left) == normalize_story_selection_paint(right)
}

pub(crate) fn normalize_story_selection_paint(paint: &DesignPaint) -> DesignPaint {
    let mut paint = paint.clone();
    paint.id = "".into();
    for stop in &mut paint.gradient_stops {
        stop.id = "".into();
    }
    if let DesignPaintPayload::Gradient(gradient) = &mut paint.payload {
        for stop in &mut gradient.stops {
            stop.id = "".into();
        }
    }
    paint
}
