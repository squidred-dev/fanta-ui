//! Collection items, layout grids, grid dimensions and tracks,
//! and layout-grid styles and variables.

use super::*;

#[allow(deprecated)]
pub(crate) fn reduce(
    screen: &mut DesignScreen,
    action: &DesignPanelAction,
    node_index: usize,
    _cx: &mut Context<Storybook>,
) -> Option<NodeOutcome> {
    let node = &mut screen.nodes[node_index];
    match action {
        DesignPanelAction::CollectionItemAddRequested {
            node_id,
            collection,
            ..
        } => {
            match collection {
                DesignPanelCollection::Fill => {
                    let paint_id = format!("{node_id}-fill-host-{}", node.fills.len());
                    node.fills
                        .push(DesignPaint::solid(DesignColor::BLUE).with_id(paint_id));
                }
                DesignPanelCollection::Stroke => {
                    let paint_count = node.stroke.as_ref().map_or(0, |stroke| stroke.paints.len());
                    let paint = DesignPaint::solid(DesignColor::BLACK)
                        .with_id(format!("{node_id}-stroke-host-{paint_count}"));
                    if let Some(stroke) = node.stroke.as_mut() {
                        stroke.add_paint(paint);
                    } else {
                        node.stroke = Some(DesignStroke::for_node(
                            node.kind,
                            paint,
                            1.,
                            DesignStrokeAlign::Inside,
                        ));
                    }
                }
                DesignPanelCollection::Effect => node.effects.push(
                    DesignEffect::drop_shadow(
                        DesignColor::rgba(0x00, 0x00, 0x00, 0x33),
                        16.,
                        0.,
                        0.,
                        4.,
                    )
                    .with_id(format!("{node_id}-legacy-effect-{}", node.effects.len())),
                ),
                DesignPanelCollection::LayoutGrid => {
                    if node.layout_grid_style_binding.is_none() {
                        let guide_id = format!("{node_id}-guide-host-{}", node.layout_grids.len());
                        node.layout_grids
                            .push(DesignLayoutGrid::default().with_id(guide_id));
                    }
                }
                DesignPanelCollection::Export => {
                    node.export_settings.push(DesignExportSetting {
                        scale: 1.,
                        suffix: "".into(),
                        format: DesignExportFormat::Png,
                    });
                }
            }
            screen.last_action = format!("Host added {} to {node_id}", collection.label()).into();
        }
        DesignPanelAction::CollectionItemRemoveRequested {
            node_id,
            collection,
            index,
            ..
        } => {
            match collection {
                DesignPanelCollection::Fill => {
                    if node.kind == DesignPanelNodeKind::MultipleSelection {
                        remove_index(&mut node.selection_colors, *index);
                    } else {
                        remove_index(&mut node.fills, *index);
                    }
                }
                DesignPanelCollection::Stroke => {
                    if let Some(stroke) = node.stroke.as_mut() {
                        stroke.remove_paint(*index);
                    }
                }
                DesignPanelCollection::Effect => remove_index(&mut node.effects, *index),
                DesignPanelCollection::LayoutGrid => {
                    unreachable!("indexed layout-guide removal returns before reducer dispatch")
                }
                DesignPanelCollection::Export => {
                    remove_index(&mut node.export_settings, *index);
                }
            }
            screen.last_action = format!(
                "Host removed {} #{index} from {node_id}",
                collection.label()
            )
            .into();
        }
        DesignPanelAction::LayoutGridPropertyEditRequested {
            node_id,
            guide_id,
            index,
            property,
            value,
            phase,
        } => {
            let target = StoryLayoutGridEditTarget::new(
                node_id.clone(),
                guide_id.clone(),
                *index,
                *property,
            );
            let applied = apply_story_layout_grid_edit_phase(
                node,
                &mut screen.layout_grid_edit_snapshots,
                target,
                *property,
                value,
                *phase,
            );
            screen.last_action = if applied {
                format!("Host observed {phase:?} for {property:?} on guide {guide_id} in {node_id}")
                    .into()
            } else {
                format!("Host rejected a stale layout-guide edit on {node_id}").into()
            };
        }
        DesignPanelAction::LayoutGridRemoveRequested {
            node_id,
            guide_id,
            index,
        } => {
            let resolved_index = story_layout_grid_index(
                node,
                &StoryLayoutGridEditTarget::new(
                    node_id.clone(),
                    guide_id.clone(),
                    *index,
                    DesignPanelProperty::LayoutGridVisible(*index),
                ),
            );
            let removed = node.layout_grid_style_binding.is_none()
                && resolved_index.is_some_and(|index| {
                    node.layout_grids.remove(index);
                    true
                });
            screen.last_action = if removed {
                format!("Host removed guide {guide_id} from {node_id}").into()
            } else {
                format!("Host rejected a stale layout-guide removal on {node_id}").into()
            };
        }
        DesignPanelAction::GridDimensionsEditRequested {
            node_id,
            dimensions,
            phase,
        } => {
            let applied = apply_story_grid_dimensions_edit_phase(
                node,
                &mut screen.grid_dimensions_edit_snapshots,
                node_id,
                *dimensions,
                *phase,
            );
            screen.last_action = if applied {
                format!(
                    "Host observed {phase:?} for Grid dimensions {} × {} on {node_id}",
                    dimensions.columns, dimensions.rows
                )
                .into()
            } else {
                format!("Host rejected stale or invalid Grid dimensions on {node_id}").into()
            };
        }
        DesignPanelAction::GridTrackAddRequested {
            node_id,
            axis,
            insertion_index,
        } => {
            let applied = node.layout.as_mut().is_some_and(|layout| {
                if layout.mode != DesignLayoutMode::Grid
                    || !layout.grid_track_count_is_editable(*axis)
                {
                    return false;
                }
                let tracks = match axis {
                    DesignGridTrackAxis::Column => &mut layout.grid_columns,
                    DesignGridTrackAxis::Row => &mut layout.grid_rows,
                };
                if *insertion_index > tracks.len() {
                    return false;
                }
                tracks.insert(*insertion_index, DesignGridTrack::hug());
                true
            });
            screen.last_action = if applied {
                format!(
                    "Host inserted a Hug {} track at {insertion_index} on {node_id}",
                    axis.label().to_ascii_lowercase()
                )
                .into()
            } else {
                format!("Host rejected an invalid Grid track insertion on {node_id}").into()
            };
        }
        DesignPanelAction::GridTrackDeleteRequested {
            node_id,
            axis,
            index,
        } => {
            let applied = node.layout.as_mut().is_some_and(|layout| {
                if !layout.can_delete_grid_track(*axis) {
                    return false;
                }
                let tracks = match axis {
                    DesignGridTrackAxis::Column => &mut layout.grid_columns,
                    DesignGridTrackAxis::Row => &mut layout.grid_rows,
                };
                if *index >= tracks.len() {
                    return false;
                }
                tracks.remove(*index);
                true
            });
            screen.last_action = if applied {
                format!(
                    "Host deleted {} track {index} on {node_id}",
                    axis.label().to_ascii_lowercase()
                )
                .into()
            } else {
                format!("Host preserved the required final Grid track on {node_id}").into()
            };
        }
        DesignPanelAction::GridTracksReorderRequested {
            node_id,
            axis,
            from_indices,
            insertion_index,
        } => {
            let applied = node.layout.as_mut().is_some_and(|layout| {
                if layout.mode != DesignLayoutMode::Grid {
                    return false;
                }
                let tracks = match axis {
                    DesignGridTrackAxis::Column => &mut layout.grid_columns,
                    DesignGridTrackAxis::Row => &mut layout.grid_rows,
                };
                reorder_story_grid_tracks(tracks, from_indices, *insertion_index)
            });
            screen.last_action = if applied {
                format!(
                    "Host reordered {} tracks {from_indices:?} at {insertion_index} on {node_id}",
                    axis.label().to_ascii_lowercase()
                )
                .into()
            } else {
                format!("Host rejected an invalid Grid track reorder on {node_id}").into()
            };
        }
        DesignPanelAction::LayoutGridStyleApplyRequested { node_id, style } => {
            let resolved = screen
                .layout_grid_styles
                .style(style)
                .filter(|resource| {
                    resource.import_state == DesignLayoutGridStyleImportState::Imported
                })
                .cloned();
            let applied = resolved.is_some_and(|resource| {
                node.layout_grids = resource.layout_grids;
                node.layout_grid_style_binding = Some(match &style.source {
                    DesignLayoutGridStyleSource::Page => {
                        DesignLayoutGridStyleBinding::new(style.style_id.clone(), resource.name)
                    }
                    DesignLayoutGridStyleSource::Library { library_id } => {
                        DesignLayoutGridStyleBinding::library(
                            library_id.clone(),
                            style.style_id.clone(),
                            resource.name,
                        )
                    }
                });
                true
            });
            screen.last_action = if applied {
                format!(
                    "Host atomically applied Grid style {} on {node_id}",
                    style.style_id
                )
                .into()
            } else {
                format!(
                    "Host rejected unavailable Grid style {} on {node_id}",
                    style.style_id
                )
                .into()
            };
        }
        DesignPanelAction::LayoutGridStyleCreateRequested {
            node_id,
            layout_grids,
        } => {
            let matches_current =
                node.layout_grid_style_binding.is_none() && node.layout_grids == *layout_grids;
            if matches_current {
                let style_number = screen
                    .layout_grid_styles
                    .page_styles
                    .len()
                    .saturating_add(1);
                let style_id: SharedString = format!("grid-style-page-{style_number}").into();
                let name: SharedString = format!("Grid / {style_number}").into();
                screen
                    .layout_grid_styles
                    .page_styles
                    .push(DesignLayoutGridStyle::new(
                        style_id.clone(),
                        name.clone(),
                        layout_grids.clone(),
                    ));
                node.layout_grid_style_binding =
                    Some(DesignLayoutGridStyleBinding::new(style_id.clone(), name));
                screen.last_action =
                    format!("Host created Grid style {style_id} on {node_id}").into();
            } else {
                screen.last_action =
                    format!("Host rejected a stale Grid style snapshot on {node_id}").into();
            }
        }
        DesignPanelAction::LayoutGridStyleDetachRequested { node_id, style } => {
            let detached = node
                .layout_grid_style_binding
                .as_ref()
                .is_some_and(|binding| binding.can_detach && binding.selection() == *style);
            if detached {
                node.layout_grid_style_binding = None;
            }
            screen.last_action = if detached {
                format!("Host detached Grid style {} on {node_id}", style.style_id).into()
            } else {
                format!("Host rejected a stale Grid style detach on {node_id}").into()
            };
        }
        DesignPanelAction::LayoutGridStyleImportRequested { node_id, style } => {
            let imported = match &style.source {
                DesignLayoutGridStyleSource::Page => false,
                DesignLayoutGridStyleSource::Library { library_id } => screen
                    .layout_grid_styles
                    .libraries
                    .iter_mut()
                    .find(|library| library.id == *library_id)
                    .and_then(|library| {
                        library
                            .styles
                            .iter_mut()
                            .find(|resource| resource.id == style.style_id)
                    })
                    .is_some_and(|resource| {
                        if resource.import_state != DesignLayoutGridStyleImportState::Available {
                            return false;
                        }
                        resource.import_state = DesignLayoutGridStyleImportState::Imported;
                        true
                    }),
            };
            screen.last_action = if imported {
                format!("Host imported Grid style {} for {node_id}", style.style_id).into()
            } else {
                format!(
                    "Host rejected Grid style import {} for {node_id}",
                    style.style_id
                )
                .into()
            };
        }
        DesignPanelAction::LayoutGridVariableApplyRequested {
            node_id,
            target,
            variable_id,
        } => {
            let variable = screen
                .layout_grid_variables
                .variable(variable_id.as_ref())
                .cloned();
            let applied = variable.is_some_and(|variable| {
                variable.import_state != DesignVariableImportState::Available
                    && apply_story_layout_grid_variable(node, target, &variable)
            });
            screen.last_action = if applied {
                format!(
                    "Host bound Number variable {variable_id} to {} on guide {} in {node_id}",
                    target.field.api_name(),
                    target.guide_id
                )
                .into()
            } else {
                format!("Host rejected incompatible Number variable {variable_id} on {node_id}")
                    .into()
            };
        }
        DesignPanelAction::LayoutGridVariableImportRequested {
            node_id,
            target,
            variable_id,
        } => {
            let compatible = screen
                .layout_grid_variables
                .variable(variable_id.as_ref())
                .is_some_and(|variable| {
                    variable.import_state == DesignVariableImportState::Available
                        && story_layout_grid_variable_target(node, target)
                            .is_some_and(|(_, value)| value.is_compatible(variable))
                        && node.layout_grid_style_binding.is_none()
                });
            let imported = compatible
                && screen
                    .layout_grid_variables
                    .variable_mut(variable_id.as_ref())
                    .is_some_and(|variable| {
                        variable.import_state = DesignVariableImportState::Imported;
                        true
                    });
            screen.last_action = if imported {
                format!("Host imported Number variable {variable_id} for {node_id}").into()
            } else {
                format!("Host rejected Number-variable import on {node_id}").into()
            };
        }
        DesignPanelAction::LayoutGridVariableDetachRequested {
            node_id,
            target,
            variable_id,
        } => {
            let detached = detach_story_layout_grid_variable(node, target, variable_id.as_ref());
            screen.last_action = if detached {
                format!(
                    "Host detached Number variable {variable_id} from {} on guide {} in {node_id}",
                    target.field.api_name(),
                    target.guide_id
                )
                .into()
            } else {
                format!("Host rejected a stale layout-guide variable detach on {node_id}").into()
            };
        }
        DesignPanelAction::LayoutGridVariableCreateRequested {
            node_id,
            target,
            value,
        } => {
            let variable_number = story_layout_grid_variable_float(*value);
            let variable_index = screen.layout_grid_variables.variables.len() + 1;
            let variable_id: SharedString =
                format!("storybook-layout-number-{variable_index}").into();
            let variable_name: SharedString = format!("Layout number {variable_index}").into();
            let variable = variable_number.map(|number| {
                DesignVariable::page(
                    variable_id.clone(),
                    variable_name,
                    "storybook-created",
                    "Created in Storybook",
                    DesignVariableResolvedType::Float,
                )
                .with_resolved_value(DesignVariableResolvedValue::Float(number))
            });
            let created = screen.layout_grid_variables.create_state.is_enabled()
                && variable.as_ref().is_some_and(|variable| {
                    apply_story_layout_grid_variable(node, target, variable)
                });
            if created && let Some(variable) = variable {
                screen.layout_grid_variables.variables.push(variable);
            }
            screen.last_action = if created {
                format!(
                    "Host created and bound Number variable {variable_id} for {} on {node_id}",
                    target.field.api_name()
                )
                .into()
            } else {
                format!("Host rejected layout-guide variable creation on {node_id}").into()
            };
        }
        DesignPanelAction::LayoutGridCountVariableApplyRequested { node_id, .. }
        | DesignPanelAction::LayoutGridCountVariableDetachRequested { node_id, .. } => {
            screen.last_action =
                format!("Host ignored a compatibility-only count-variable action on {node_id}")
                    .into();
        }
        _ => return None,
    }
    Some(NodeOutcome::Applied)
}

pub(crate) fn remove_index<T>(items: &mut Vec<T>, index: usize) {
    if index < items.len() {
        items.remove(index);
    }
}

pub(crate) fn story_layout_grid_index(
    node: &DesignPanelNode,
    target: &StoryLayoutGridEditTarget,
) -> Option<usize> {
    if target.guide_id.is_empty() {
        target
            .legacy_index
            .filter(|index| *index < node.layout_grids.len())
    } else {
        node.layout_grids
            .iter()
            .position(|guide| guide.id == target.guide_id)
    }
}

pub(crate) fn apply_story_layout_grid_edit_phase(
    node: &mut DesignPanelNode,
    snapshots: &mut HashMap<StoryLayoutGridEditTarget, DesignLayoutGrid>,
    target: StoryLayoutGridEditTarget,
    property: DesignPanelProperty,
    value: &DesignPanelValue,
    phase: DesignPanelEditPhase,
) -> bool {
    if property.layout_grid_index().is_none() || node.layout_grid_style_binding.is_some() {
        snapshots.remove(&target);
        return false;
    }
    let Some(index) = story_layout_grid_index(node, &target) else {
        snapshots.remove(&target);
        return false;
    };
    match phase {
        DesignPanelEditPhase::Begin => {
            let mut candidate = node.layout_grids[index].clone();
            if !apply_story_layout_grid_property(&mut candidate, property, value) {
                return false;
            }
            snapshots
                .entry(target)
                .or_insert_with(|| node.layout_grids[index].clone());
            true
        }
        DesignPanelEditPhase::Preview => {
            apply_story_layout_grid_property(&mut node.layout_grids[index], property, value)
        }
        DesignPanelEditPhase::Commit => {
            let applied =
                apply_story_layout_grid_property(&mut node.layout_grids[index], property, value);
            snapshots.remove(&target);
            applied
        }
        DesignPanelEditPhase::Cancel => {
            let Some(original) = snapshots.remove(&target) else {
                return false;
            };
            node.layout_grids[index] = original;
            true
        }
    }
}

pub(crate) fn story_layout_grid_variable_target(
    node: &DesignPanelNode,
    target: &DesignLayoutGridVariableTarget,
) -> Option<(usize, DesignLayoutGridVariableValue)> {
    let index = if target.guide_id.is_empty() {
        (target.index < node.layout_grids.len()).then_some(target.index)
    } else {
        node.layout_grids
            .iter()
            .position(|guide| guide.id == target.guide_id)
    }?;
    let (resolved, value) = node.layout_grids[index]
        .variable_target(index, target.property.with_layout_grid_index(index))?;
    (resolved.field == target.field
        && resolved.property.with_layout_grid_index(0) == target.property.with_layout_grid_index(0))
    .then_some((index, value))
}

pub(crate) fn story_layout_grid_variable_float(
    value: DesignLayoutGridVariableValue,
) -> Option<f32> {
    match value {
        DesignLayoutGridVariableValue::Number(value) if value.is_finite() && value >= 0. => {
            Some(value)
        }
        DesignLayoutGridVariableValue::Count(DesignLayoutGridCount::Auto) => Some(f32::INFINITY),
        DesignLayoutGridVariableValue::Count(DesignLayoutGridCount::Number(value)) => {
            Some(f32::from(value))
        }
        DesignLayoutGridVariableValue::Number(_) => None,
    }
}

pub(crate) fn apply_story_layout_grid_variable(
    node: &mut DesignPanelNode,
    target: &DesignLayoutGridVariableTarget,
    variable: &DesignVariable,
) -> bool {
    if node.layout_grid_style_binding.is_some()
        || variable.import_state == DesignVariableImportState::Available
    {
        return false;
    }
    let Some((index, current_value)) = story_layout_grid_variable_target(node, target) else {
        return false;
    };
    if !current_value.is_compatible(variable)
        || node.layout_grids[index]
            .variable_binding(target.field)
            .is_some_and(|binding| binding.read_only_reason.is_some())
    {
        return false;
    }
    let Some(DesignVariableResolvedValue::Float(resolved)) = variable.resolved_value else {
        return false;
    };
    let property = target.property.with_layout_grid_index(index);
    let applied = match (property, &mut node.layout_grids[index].settings) {
        (DesignPanelProperty::LayoutGridSize(_), DesignLayoutGridSettings::Uniform(settings))
            if resolved.is_finite() && resolved >= 0. =>
        {
            settings.size = resolved;
            true
        }
        (DesignPanelProperty::LayoutGridSize(_), DesignLayoutGridSettings::Columns(settings))
            if !settings.alignment.is_stretch() && resolved.is_finite() && resolved >= 0. =>
        {
            settings.size = resolved;
            true
        }
        (DesignPanelProperty::LayoutGridSize(_), DesignLayoutGridSettings::Rows(settings))
            if !settings.alignment.is_stretch() && resolved.is_finite() && resolved >= 0. =>
        {
            settings.size = resolved;
            true
        }
        (DesignPanelProperty::LayoutGridCount(_), DesignLayoutGridSettings::Columns(settings)) => {
            settings.count = if resolved.is_infinite() && resolved.is_sign_positive() {
                DesignLayoutGridCount::Auto
            } else {
                DesignLayoutGridCount::number(resolved as u16)
            };
            true
        }
        (DesignPanelProperty::LayoutGridCount(_), DesignLayoutGridSettings::Rows(settings)) => {
            settings.count = if resolved.is_infinite() && resolved.is_sign_positive() {
                DesignLayoutGridCount::Auto
            } else {
                DesignLayoutGridCount::number(resolved as u16)
            };
            true
        }
        (DesignPanelProperty::LayoutGridOffset(_), DesignLayoutGridSettings::Columns(settings))
            if settings.alignment.supports_offset() && resolved.is_finite() && resolved >= 0. =>
        {
            settings.offset = resolved;
            true
        }
        (DesignPanelProperty::LayoutGridOffset(_), DesignLayoutGridSettings::Rows(settings))
            if settings.alignment.supports_offset() && resolved.is_finite() && resolved >= 0. =>
        {
            settings.offset = resolved;
            true
        }
        (DesignPanelProperty::LayoutGridMargin(_), DesignLayoutGridSettings::Columns(settings))
            if settings.alignment.is_stretch() && resolved.is_finite() && resolved >= 0. =>
        {
            settings.margin = resolved;
            true
        }
        (DesignPanelProperty::LayoutGridMargin(_), DesignLayoutGridSettings::Rows(settings))
            if settings.alignment.is_stretch() && resolved.is_finite() && resolved >= 0. =>
        {
            settings.margin = resolved;
            true
        }
        (DesignPanelProperty::LayoutGridGutter(_), DesignLayoutGridSettings::Columns(settings))
            if resolved.is_finite() && resolved >= 0. =>
        {
            settings.gutter = resolved;
            true
        }
        (DesignPanelProperty::LayoutGridGutter(_), DesignLayoutGridSettings::Rows(settings))
            if resolved.is_finite() && resolved >= 0. =>
        {
            settings.gutter = resolved;
            true
        }
        _ => false,
    };
    if applied {
        node.layout_grids[index].set_variable_binding(
            target.field,
            Some(
                DesignLayoutGridVariableBinding::new(variable.id.clone(), variable.name.clone())
                    .with_collection(variable.collection_name.clone()),
            ),
        );
    }
    applied
}

pub(crate) fn detach_story_layout_grid_variable(
    node: &mut DesignPanelNode,
    target: &DesignLayoutGridVariableTarget,
    variable_id: &str,
) -> bool {
    if node.layout_grid_style_binding.is_some() {
        return false;
    }
    let Some((index, _)) = story_layout_grid_variable_target(node, target) else {
        return false;
    };
    let detachable = node.layout_grids[index]
        .variable_binding(target.field)
        .is_some_and(|binding| {
            binding.variable_id.as_ref() == variable_id
                && binding.can_detach
                && binding.read_only_reason.is_none()
        });
    if detachable {
        node.layout_grids[index].set_variable_binding(target.field, None);
    }
    detachable
}

pub(crate) fn apply_story_layout_grid_property(
    guide: &mut DesignLayoutGrid,
    property: DesignPanelProperty,
    value: &DesignPanelValue,
) -> bool {
    if guide
        .variable_target(0, property.with_layout_grid_index(0))
        .is_some_and(|(target, _)| guide.variable_binding(target.field).is_some())
    {
        return false;
    }
    match (property.with_layout_grid_index(0), value) {
        (DesignPanelProperty::LayoutGridKind(_), DesignPanelValue::GridKind(value)) => {
            guide.settings = match value {
                DesignGridKind::Uniform => {
                    DesignLayoutGridSettings::Uniform(DesignUniformLayoutGrid::default())
                }
                DesignGridKind::Columns => {
                    DesignLayoutGridSettings::Columns(DesignColumnLayoutGrid::default())
                }
                DesignGridKind::Rows => {
                    DesignLayoutGridSettings::Rows(DesignRowLayoutGrid::default())
                }
            };
            true
        }
        (DesignPanelProperty::LayoutGridVisible(_), DesignPanelValue::Bool(value)) => {
            guide.visible = *value;
            true
        }
        (
            DesignPanelProperty::LayoutGridAlignment(_),
            DesignPanelValue::ColumnGridAlignment(value),
        ) => {
            let DesignLayoutGridSettings::Columns(settings) = &mut guide.settings else {
                return false;
            };
            settings.alignment = *value;
            true
        }
        (
            DesignPanelProperty::LayoutGridAlignment(_),
            DesignPanelValue::RowGridAlignment(value),
        ) => {
            let DesignLayoutGridSettings::Rows(settings) = &mut guide.settings else {
                return false;
            };
            settings.alignment = *value;
            true
        }
        (DesignPanelProperty::LayoutGridCount(_), DesignPanelValue::LayoutGridCount(value)) => {
            match &mut guide.settings {
                DesignLayoutGridSettings::Columns(settings) => {
                    settings.count = *value;
                    true
                }
                DesignLayoutGridSettings::Rows(settings) => {
                    settings.count = *value;
                    true
                }
                DesignLayoutGridSettings::Uniform(_) => false,
            }
        }
        (DesignPanelProperty::LayoutGridSize(_), DesignPanelValue::Number(value))
            if value.is_finite() && *value >= 0. =>
        {
            match &mut guide.settings {
                DesignLayoutGridSettings::Uniform(settings) => settings.size = *value,
                DesignLayoutGridSettings::Columns(settings) if !settings.alignment.is_stretch() => {
                    settings.size = *value;
                }
                DesignLayoutGridSettings::Rows(settings) if !settings.alignment.is_stretch() => {
                    settings.size = *value;
                }
                DesignLayoutGridSettings::Columns(_) | DesignLayoutGridSettings::Rows(_) => {
                    return false;
                }
            }
            true
        }
        (DesignPanelProperty::LayoutGridOffset(_), DesignPanelValue::Number(value))
            if value.is_finite() && *value >= 0. =>
        {
            match &mut guide.settings {
                DesignLayoutGridSettings::Columns(settings)
                    if settings.alignment.supports_offset() =>
                {
                    settings.offset = *value;
                    true
                }
                DesignLayoutGridSettings::Rows(settings)
                    if settings.alignment.supports_offset() =>
                {
                    settings.offset = *value;
                    true
                }
                DesignLayoutGridSettings::Uniform(_)
                | DesignLayoutGridSettings::Columns(_)
                | DesignLayoutGridSettings::Rows(_) => false,
            }
        }
        (DesignPanelProperty::LayoutGridGutter(_), DesignPanelValue::Number(value))
            if value.is_finite() && *value >= 0. =>
        {
            match &mut guide.settings {
                DesignLayoutGridSettings::Columns(settings) => settings.gutter = *value,
                DesignLayoutGridSettings::Rows(settings) => settings.gutter = *value,
                DesignLayoutGridSettings::Uniform(_) => return false,
            }
            true
        }
        (DesignPanelProperty::LayoutGridMargin(_), DesignPanelValue::Number(value))
            if value.is_finite() && *value >= 0. =>
        {
            match &mut guide.settings {
                DesignLayoutGridSettings::Columns(settings) if settings.alignment.is_stretch() => {
                    settings.margin = *value;
                    true
                }
                DesignLayoutGridSettings::Rows(settings) if settings.alignment.is_stretch() => {
                    settings.margin = *value;
                    true
                }
                DesignLayoutGridSettings::Uniform(_)
                | DesignLayoutGridSettings::Columns(_)
                | DesignLayoutGridSettings::Rows(_) => false,
            }
        }
        (DesignPanelProperty::LayoutGridColor(_), DesignPanelValue::Color(value)) => {
            guide.color = *value;
            true
        }
        (DesignPanelProperty::LayoutGridOpacity(_), DesignPanelValue::Number(value))
            if value.is_finite() =>
        {
            guide.opacity = value.clamp(0., 100.);
            true
        }
        _ => false,
    }
}

pub(crate) fn reorder_story_grid_tracks(
    tracks: &mut Vec<DesignGridTrack>,
    from_indices: &[usize],
    insertion_index: usize,
) -> bool {
    if from_indices.is_empty() || insertion_index > tracks.len() {
        return false;
    }
    let mut selected = vec![false; tracks.len()];
    for index in from_indices {
        let Some(slot) = selected.get_mut(*index) else {
            return false;
        };
        if *slot {
            return false;
        }
        *slot = true;
    }
    let moved = tracks
        .iter()
        .copied()
        .zip(&selected)
        .filter_map(|(track, selected)| (*selected).then_some(track))
        .collect::<Vec<_>>();
    let removed_before_insertion = selected
        .iter()
        .take(insertion_index)
        .filter(|selected| **selected)
        .count();
    let adjusted_insertion = insertion_index.saturating_sub(removed_before_insertion);
    let mut remaining = tracks
        .iter()
        .copied()
        .zip(selected)
        .filter_map(|(track, selected)| (!selected).then_some(track))
        .collect::<Vec<_>>();
    if adjusted_insertion > remaining.len() {
        return false;
    }
    remaining.splice(adjusted_insertion..adjusted_insertion, moved);
    *tracks = remaining;
    true
}

pub(crate) fn apply_story_grid_dimensions_edit_phase(
    node: &mut DesignPanelNode,
    snapshots: &mut HashMap<SharedString, DesignLayout>,
    node_id: &SharedString,
    dimensions: DesignGridDimensions,
    phase: DesignPanelEditPhase,
) -> bool {
    if node.id != *node_id || !dimensions.is_valid() {
        return false;
    }
    if phase == DesignPanelEditPhase::Cancel {
        let Some(snapshot) = snapshots.remove(node_id) else {
            return false;
        };
        node.layout = Some(snapshot);
        return true;
    }
    let Some(layout) = node
        .layout
        .as_mut()
        .filter(|layout| layout.mode == DesignLayoutMode::Grid)
    else {
        return false;
    };
    if layout.grid_auto_tracks == DesignGridAutoTracks::Rows
        && dimensions.rows != layout.grid_rows.len()
    {
        return false;
    }
    match phase {
        DesignPanelEditPhase::Begin => {
            if snapshots.contains_key(node_id) {
                return false;
            }
            snapshots.insert(node_id.clone(), layout.clone());
        }
        DesignPanelEditPhase::Preview => {
            if !snapshots.contains_key(node_id) {
                return false;
            }
        }
        DesignPanelEditPhase::Commit => {}
        DesignPanelEditPhase::Cancel => unreachable!("Cancel returned before Grid validation"),
    }

    layout
        .grid_columns
        .resize(dimensions.columns, DesignGridTrack::hug());
    if layout.grid_auto_tracks == DesignGridAutoTracks::None {
        layout
            .grid_rows
            .resize(dimensions.rows, DesignGridTrack::hug());
    }
    if phase == DesignPanelEditPhase::Commit {
        snapshots.remove(node_id);
    }
    true
}
