//! Paint edits and lifecycle preflights, paint shaders, paint
//! styles, and color variables and styles.

use super::*;

pub(crate) fn rejects_stale_paint_target(
    screen: &mut DesignScreen,
    action: &DesignPanelAction,
    action_node_id: &SharedString,
    cx: &mut Context<Storybook>,
) -> bool {
    if let Some((collection, target)) = action.paint_target() {
        let expected = screen.paint_target_for(action_node_id, collection);
        let stale_cancel_is_active = match action {
            DesignPanelAction::PaintEditRequested {
                node_id,
                collection,
                target,
                paint_id,
                index,
                phase: DesignPanelEditPhase::Cancel,
                ..
            } => screen
                .edits
                .paint_edit_snapshots
                .contains_key(&StoryPaintEditTarget::new(
                    node_id.clone(),
                    *collection,
                    *target,
                    paint_id.clone(),
                    *index,
                )),
            DesignPanelAction::PaintMediaCropActionRequested {
                node_id,
                collection,
                target,
                paint_id,
                index,
                action: DesignMediaCropAction::Cancel,
            } => screen
                .edits
                .media_crop_targets
                .contains(&StoryPaintEditTarget::new(
                    node_id.clone(),
                    *collection,
                    *target,
                    paint_id.clone(),
                    *index,
                )),
            DesignPanelAction::PaintVideoPreviewActionRequested {
                node_id,
                collection,
                target,
                paint_id,
                index,
                action:
                    DesignVideoPreviewAction::Scrub {
                        phase: DesignPanelEditPhase::Cancel,
                        ..
                    },
            } => screen
                .edits
                .video_scrub_snapshots
                .contains_key(&StoryPaintEditTarget::new(
                    node_id.clone(),
                    *collection,
                    *target,
                    paint_id.clone(),
                    *index,
                )),
            _ => false,
        };
        if target != expected && !stale_cancel_is_active {
            screen.harness.last_action = format!(
                    "Host rejected stale {target:?} paint intent for {action_node_id}; current target is {expected:?}"
                )
                .into();
            cx.notify();
            return true;
        }
    }
    false
}

pub(crate) fn rejects_out_of_order_lifecycle(
    screen: &mut DesignScreen,
    action: &DesignPanelAction,
    action_node_id: &SharedString,
    cx: &mut Context<Storybook>,
) -> bool {
    let paint_lifecycle_is_valid = match action {
        DesignPanelAction::PaintEditRequested {
            node_id,
            collection,
            target,
            paint_id,
            index,
            phase,
            ..
        } => {
            let target = StoryPaintEditTarget::new(
                node_id.clone(),
                *collection,
                *target,
                paint_id.clone(),
                *index,
            );
            match phase {
                DesignPanelEditPhase::Begin => {
                    !screen.edits.paint_edit_snapshots.contains_key(&target)
                }
                DesignPanelEditPhase::Preview | DesignPanelEditPhase::Cancel => {
                    screen.edits.paint_edit_snapshots.contains_key(&target)
                }
                // Picker buttons and toggles are allowed to emit one
                // atomic Commit without opening a transaction.
                DesignPanelEditPhase::Commit => true,
            }
        }
        DesignPanelAction::PaintMediaCropActionRequested {
            node_id,
            collection,
            target,
            paint_id,
            index,
            action,
        } => {
            let target = StoryPaintEditTarget::new(
                node_id.clone(),
                *collection,
                *target,
                paint_id.clone(),
                *index,
            );
            match action {
                DesignMediaCropAction::Begin => screen.edits.media_crop_targets.insert(target),
                DesignMediaCropAction::Preview { .. } => {
                    screen.edits.media_crop_targets.contains(&target)
                }
                DesignMediaCropAction::Commit { .. } | DesignMediaCropAction::Cancel => {
                    screen.edits.media_crop_targets.remove(&target)
                }
                DesignMediaCropAction::ResizeToFit => true,
            }
        }
        DesignPanelAction::PaintVideoPreviewActionRequested {
            node_id,
            collection,
            target,
            paint_id,
            index,
            action: DesignVideoPreviewAction::Scrub { phase, .. },
        } => {
            let target = StoryPaintEditTarget::new(
                node_id.clone(),
                *collection,
                *target,
                paint_id.clone(),
                *index,
            );
            match phase {
                DesignPanelEditPhase::Begin => {
                    !screen.edits.video_scrub_snapshots.contains_key(&target)
                }
                DesignPanelEditPhase::Preview
                | DesignPanelEditPhase::Commit
                | DesignPanelEditPhase::Cancel => {
                    screen.edits.video_scrub_snapshots.contains_key(&target)
                }
            }
        }
        _ => true,
    };
    if !paint_lifecycle_is_valid {
        screen.harness.last_action =
            format!("Host rejected an out-of-order paint transaction for {action_node_id}").into();
        cx.notify();
        return true;
    }
    false
}

#[allow(deprecated)]
pub(crate) fn reduce(
    screen: &mut DesignScreen,
    action: &DesignPanelAction,
    node_index: usize,
    _cx: &mut Context<Storybook>,
) -> Option<NodeOutcome> {
    let node = &mut screen.host.nodes[node_index];
    match action {
        DesignPanelAction::PaintEditRequested {
            node_id,
            collection,
            target,
            paint_id,
            index,
            edit,
            phase,
            ..
        } => {
            let edit_target = StoryPaintEditTarget::new(
                node_id.clone(),
                *collection,
                *target,
                paint_id.clone(),
                *index,
            );
            let paints = match collection {
                DesignPanelCollection::Fill
                    if node.kind == DesignPanelNodeKind::MultipleSelection =>
                {
                    Some(&mut node.selection_colors)
                }
                DesignPanelCollection::Fill => Some(&mut node.fills),
                DesignPanelCollection::Stroke => {
                    node.stroke.as_mut().map(|stroke| &mut stroke.paints)
                }
                DesignPanelCollection::Effect
                | DesignPanelCollection::LayoutGrid
                | DesignPanelCollection::Export => None,
            };
            if let Some(paints) = paints {
                let resolved_index = if paint_id.is_empty() {
                    (*index < paints.len()).then_some(*index)
                } else {
                    paints.iter().position(|paint| paint.id == *paint_id)
                };
                if let Some(paint) = resolved_index.and_then(|index| paints.get_mut(index)) {
                    apply_story_paint_edit_phase(
                        paint,
                        &mut screen.edits.paint_edit_snapshots,
                        edit_target,
                        edit,
                        *phase,
                    );
                }
            }
            screen.harness.last_action = format!(
                "Host observed {phase:?} for {:?} on {} paint {} in {node_id}",
                edit.property,
                collection.label(),
                if paint_id.is_empty() {
                    format!("#{index}")
                } else {
                    paint_id.to_string()
                }
            )
            .into();
        }
        DesignPanelAction::PaintReorderRequested {
            node_id,
            collection,
            paint_id,
            from_index,
            to_index,
            ..
        } => {
            let paints = match collection {
                DesignPanelCollection::Fill
                    if node.kind == DesignPanelNodeKind::MultipleSelection =>
                {
                    Some(&mut node.selection_colors)
                }
                DesignPanelCollection::Fill => Some(&mut node.fills),
                DesignPanelCollection::Stroke => {
                    node.stroke.as_mut().map(|stroke| &mut stroke.paints)
                }
                DesignPanelCollection::Effect
                | DesignPanelCollection::LayoutGrid
                | DesignPanelCollection::Export => None,
            };
            if let Some(paints) = paints {
                let resolved_from = if paint_id.is_empty() {
                    (*from_index < paints.len()).then_some(*from_index)
                } else {
                    paints.iter().position(|paint| paint.id == *paint_id)
                };
                if let Some(resolved_from) = resolved_from
                    && *to_index < paints.len()
                {
                    let paint = paints.remove(resolved_from);
                    paints.insert(*to_index, paint);
                }
            }
            screen.harness.last_action = format!(
                "Host moved {} paint {} from {from_index} to {to_index} on {node_id}",
                collection.label(),
                if paint_id.is_empty() {
                    "legacy-index".to_owned()
                } else {
                    paint_id.to_string()
                }
            )
            .into();
        }
        DesignPanelAction::PaintSourceReplaceRequested {
            node_id,
            collection,
            paint_id,
            index,
            ..
        } => {
            let paints = match collection {
                DesignPanelCollection::Fill
                    if node.kind == DesignPanelNodeKind::MultipleSelection =>
                {
                    Some(&mut node.selection_colors)
                }
                DesignPanelCollection::Fill => Some(&mut node.fills),
                DesignPanelCollection::Stroke => {
                    node.stroke.as_mut().map(|stroke| &mut stroke.paints)
                }
                DesignPanelCollection::Effect
                | DesignPanelCollection::LayoutGrid
                | DesignPanelCollection::Export => None,
            };
            let accepted = paints.is_some_and(|paints| {
                let resolved_index = if paint_id.is_empty() {
                    (*index < paints.len()).then_some(*index)
                } else {
                    paints.iter().position(|paint| paint.id == *paint_id)
                };
                let Some(paint) = resolved_index.and_then(|index| paints.get_mut(index)) else {
                    return false;
                };
                if !matches!(&paint.payload, DesignPaintPayload::Pattern(_)) {
                    return false;
                }
                paint.apply_edit(&DesignPaintEdit {
                    property: DesignPaintProperty::PatternSourceNode,
                    value: DesignPaintValue::PatternSourceNode(
                        format!("{node_id}-replacement-pattern-node").into(),
                    ),
                })
            });
            screen.harness.last_action = format!(
                "Host {} Pattern source for {} paint {} on {node_id}",
                if accepted { "replaced" } else { "rejected" },
                collection.label(),
                if paint_id.is_empty() {
                    format!("#{index}")
                } else {
                    paint_id.to_string()
                }
            )
            .into();
        }
        DesignPanelAction::PaintShaderImportRequested {
            node_id,
            collection,
            paint_id,
            index,
            shader,
            ..
        } => {
            let imported = resolve_story_shader_mut(&mut screen.host.shaders, shader).is_some_and(
                |definition| {
                    if definition.imported {
                        return false;
                    }
                    definition.imported = true;
                    if definition.property_definitions.is_empty() {
                        definition.property_definitions =
                            imported_story_shader_properties(&definition.id);
                    }
                    true
                },
            );
            screen.harness.last_action = if imported {
                format!(
                    "Host imported shader {} for {} paint {} on {node_id}",
                    shader.shader_id,
                    collection.label(),
                    if paint_id.is_empty() {
                        format!("#{index}")
                    } else {
                        paint_id.to_string()
                    }
                )
                .into()
            } else {
                format!("Host ignored an already-imported or stale shader on {node_id}").into()
            };
        }
        DesignPanelAction::PaintShaderApplyRequested {
            node_id,
            collection,
            paint_id,
            index,
            shader,
            ..
        } => {
            let payload = screen
                .host
                .shaders
                .shader(shader)
                .and_then(DesignShaderPaint::from_definition);
            let applied = payload
                .zip(story_paint_mut(node, *collection, paint_id, *index))
                .is_some_and(|(payload, paint)| {
                    paint.payload = DesignPaintPayload::Shader(payload);
                    paint.sync_legacy_projection();
                    true
                });
            screen.harness.last_action = if applied {
                format!(
                    "Host applied shader {} to {} paint {} on {node_id}",
                    shader.shader_id,
                    collection.label(),
                    if paint_id.is_empty() {
                        format!("#{index}")
                    } else {
                        paint_id.to_string()
                    }
                )
                .into()
            } else {
                format!("Host rejected an unavailable shader on {node_id}").into()
            };
        }
        DesignPanelAction::PaintShaderPropertyBindRequested {
            node_id,
            collection,
            paint_id,
            index,
            definition_id,
            ..
        } => {
            let bound = story_paint_mut(node, *collection, paint_id, *index).is_some_and(|paint| {
                paint.apply_edit(&DesignPaintEdit {
                    property: DesignPaintProperty::ShaderProperty {
                        definition_id: definition_id.clone(),
                    },
                    value: DesignPaintValue::ShaderProperty(
                        DesignShaderPropertyValue::VariableAlias {
                            variable_id: format!("storybook-variable:{definition_id}").into(),
                        },
                    ),
                })
            });
            screen.harness.last_action = if bound {
                format!(
                    "Host bound shader property {definition_id} in {} paint on {node_id}",
                    collection.label()
                )
                .into()
            } else {
                format!("Host rejected a stale shader binding on {node_id}").into()
            };
        }
        DesignPanelAction::PaintShaderPropertyEditorRequested {
            node_id,
            collection,
            paint_id,
            index,
            definition_id,
            ..
        } => {
            let edited = apply_story_paint_shader_property_editor(
                node,
                *collection,
                paint_id,
                *index,
                definition_id,
            );
            screen.harness.last_action = if edited {
                format!("Host edited shader property {definition_id} on {node_id}").into()
            } else {
                format!("Host rejected a stale shader property editor on {node_id}").into()
            };
        }
        DesignPanelAction::PaintShaderPropertyDetachRequested {
            node_id,
            collection,
            paint_id,
            index,
            definition_id,
            variable_id,
            ..
        } => {
            let fallback = story_paint_mut(node, *collection, paint_id, *index)
                .and_then(|paint| {
                    let DesignPaintPayload::Shader(shader) = &paint.payload else {
                        return None;
                    };
                    let current_matches = shader
                        .property(definition_id)
                        .and_then(DesignShaderPropertyValue::variable_alias_id)
                        .is_some_and(|current| current == variable_id);
                    current_matches.then(|| shader.shader_id.clone())
                })
                .and_then(|shader_id| {
                    screen
                        .host
                        .shaders
                        .definition(&shader_id)
                        .and_then(|shader| shader.property(definition_id))
                        .and_then(|property| property.default_value.clone())
                });
            let detached = fallback
                .zip(story_paint_mut(node, *collection, paint_id, *index))
                .is_some_and(|(fallback, paint)| {
                    paint.apply_edit(&DesignPaintEdit {
                        property: DesignPaintProperty::ShaderProperty {
                            definition_id: definition_id.clone(),
                        },
                        value: DesignPaintValue::ShaderProperty(fallback),
                    })
                });
            screen.harness.last_action = if detached {
                format!(
                        "Host detached variable {variable_id} from shader property {definition_id} on {node_id}"
                    )
                    .into()
            } else {
                format!("Host rejected a stale shader detach on {node_id}").into()
            };
        }
        DesignPanelAction::PaintStyleApplyRequested {
            node_id,
            collection,
            style,
            ..
        } => {
            let applied =
                screen
                    .host
                    .paint_styles
                    .style(style)
                    .cloned()
                    .is_some_and(|style_data| {
                        apply_story_paint_style(node, *collection, style, style_data)
                    });
            screen.harness.last_action = if applied {
                format!(
                    "Host applied Paint style {} to {} on {node_id}",
                    style.style_id,
                    collection.label()
                )
                .into()
            } else {
                format!("Host rejected unavailable Paint style on {node_id}").into()
            };
        }
        DesignPanelAction::PaintStyleImportRequested {
            node_id,
            collection,
            style,
            ..
        } => {
            let imported = screen
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
                    "Host imported Paint style {} for {}; choose it again to apply",
                    style.style_id,
                    collection.label()
                )
                .into()
            } else {
                format!("Host rejected stale Paint-style import on {node_id}").into()
            };
        }
        DesignPanelAction::PaintStyleCreateRequested {
            node_id,
            collection,
            paints,
            ..
        } => {
            screen.harness.last_action = format!(
                "Host opened Paint-style creation for {} ordered {} paint{} on {node_id}",
                paints.len(),
                collection.label(),
                if paints.len() == 1 { "" } else { "s" }
            )
            .into();
        }
        DesignPanelAction::PaintStyleDetachRequested {
            node_id,
            collection,
            style,
            ..
        } => {
            let detached = match collection {
                DesignPanelCollection::Fill => node
                    .fill_style_binding
                    .as_ref()
                    .is_some_and(|binding| binding.can_detach && binding.selection == *style)
                    .then(|| node.fill_style_binding = None)
                    .is_some(),
                DesignPanelCollection::Stroke => node
                    .stroke_style_binding
                    .as_ref()
                    .is_some_and(|binding| binding.can_detach && binding.selection == *style)
                    .then(|| node.stroke_style_binding = None)
                    .is_some(),
                DesignPanelCollection::Effect
                | DesignPanelCollection::LayoutGrid
                | DesignPanelCollection::Export => false,
            };
            screen.harness.last_action = if detached {
                format!(
                    "Host detached Paint style {} from {} on {node_id}",
                    style.style_id,
                    collection.label()
                )
                .into()
            } else {
                format!("Host rejected stale Paint-style detach on {node_id}").into()
            };
        }
        DesignPanelAction::PaintColorVariableApplyRequested {
            node_id,
            collection,
            paint_id,
            index,
            color_target,
            variable_id,
            ..
        } => {
            let variable = screen
                .host
                .paint_variables
                .variable(variable_id.as_ref())
                .cloned();
            let applied = variable
                .filter(|variable| {
                    variable.disabled_reason.is_none()
                        && matches!(
                            variable.import_state,
                            DesignVariableImportState::Local | DesignVariableImportState::Imported
                        )
                })
                .is_some_and(|variable| {
                    story_paint_mut(node, *collection, paint_id, *index).is_some_and(|paint| {
                        apply_story_paint_variable(paint, color_target, &variable)
                    })
                });
            screen.harness.last_action = if applied {
                format!(
                    "Host bound Color variable {variable_id} to {:?} in {} paint {} on {node_id}",
                    color_target,
                    collection.label(),
                    if paint_id.is_empty() {
                        format!("#{index}")
                    } else {
                        paint_id.to_string()
                    }
                )
                .into()
            } else {
                format!("Host rejected unavailable Color variable on {node_id}").into()
            };
        }
        DesignPanelAction::PaintColorVariableImportRequested {
            node_id,
            collection,
            variable_id,
            ..
        } => {
            let imported = screen
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
                    "Host imported Color variable {variable_id} for {}; choose it again to bind",
                    collection.label()
                )
                .into()
            } else {
                format!("Host rejected stale Color-variable import on {node_id}").into()
            };
        }
        DesignPanelAction::PaintColorVariableDetachRequested {
            node_id,
            collection,
            paint_id,
            index,
            color_target,
            variable_id,
            ..
        } => {
            let detached = story_paint_mut(node, *collection, paint_id, *index)
                .is_some_and(|paint| detach_story_paint_variable(paint, color_target, variable_id));
            screen.harness.last_action = if detached {
                format!(
                    "Host detached Color variable {variable_id} from {:?} on {node_id}",
                    color_target
                )
                .into()
            } else {
                format!("Host rejected stale Color-variable detach on {node_id}").into()
            };
        }
        DesignPanelAction::PaintColorVariableCreateRequested {
            node_id,
            collection,
            color_target,
            color,
            ..
        } => {
            screen.harness.last_action = format!(
                "Host opened Color-variable creation for #{} at {:?} in {} on {node_id}",
                color.hex(),
                color_target,
                collection.label()
            )
            .into();
        }
        DesignPanelAction::PaintColorStyleSampleRequested {
            node_id,
            collection,
            paint_id,
            index,
            color_target,
            sample,
            ..
        } => {
            let resolved_sample =
                resolve_story_color_style_sample(&screen.host.color_style_samples, sample).cloned();
            let applied = resolved_sample
                .filter(|sample| sample.disabled_reason.is_none())
                .is_some_and(|sample| {
                    story_paint_mut(node, *collection, paint_id, *index).is_some_and(|paint| {
                        apply_story_color_style_sample(paint, color_target, sample.color)
                    })
                });
            screen.harness.last_action = if applied {
                format!(
                    "Host sampled Color style {} into {:?} in {} paint {} on {node_id}",
                    sample.sample_id,
                    color_target,
                    collection.label(),
                    if paint_id.is_empty() {
                        format!("#{index}")
                    } else {
                        paint_id.to_string()
                    }
                )
                .into()
            } else {
                format!("Host rejected unavailable Color-style sample on {node_id}").into()
            };
        }
        DesignPanelAction::PaintColorStyleApplyRequested {
            node_id,
            collection,
            paint_id,
            index,
            color_target,
            style,
            ..
        } => {
            let resolved_style =
                resolve_story_color_style(&screen.host.color_styles, style).cloned();
            let paints = match collection {
                DesignPanelCollection::Fill
                    if node.kind == DesignPanelNodeKind::MultipleSelection =>
                {
                    Some(&mut node.selection_colors)
                }
                DesignPanelCollection::Fill => Some(&mut node.fills),
                DesignPanelCollection::Stroke => {
                    node.stroke.as_mut().map(|stroke| &mut stroke.paints)
                }
                DesignPanelCollection::Effect
                | DesignPanelCollection::LayoutGrid
                | DesignPanelCollection::Export => None,
            };
            let applied = paints.is_some_and(|paints| {
                let resolved_index = if paint_id.is_empty() {
                    (*index < paints.len()).then_some(*index)
                } else {
                    paints.iter().position(|paint| paint.id == *paint_id)
                };
                resolved_index
                    .and_then(|index| paints.get_mut(index))
                    .zip(resolved_style.as_ref())
                    .is_some_and(|(paint, style)| {
                        apply_story_color_style(paint, color_target, style)
                    })
            });
            screen.harness.last_action = if applied {
                format!(
                    "Host applied color style {} to {} paint {} on {node_id}",
                    style.style_id,
                    collection.label(),
                    if paint_id.is_empty() {
                        format!("#{index}")
                    } else {
                        paint_id.to_string()
                    }
                )
                .into()
            } else {
                format!("Host rejected unavailable color style on {node_id}").into()
            };
        }
        DesignPanelAction::PaintColorStyleCreateRequested {
            node_id,
            collection,
            paint_id,
            index,
            color_target,
            color,
            ..
        } => {
            screen.harness.last_action = format!(
                "Host opened color-style creation for #{} on {:?} in {} paint {} on {node_id}",
                color.hex(),
                color_target,
                collection.label(),
                if paint_id.is_empty() {
                    format!("#{index}")
                } else {
                    paint_id.to_string()
                }
            )
            .into();
        }
        DesignPanelAction::PaintEyedropperRequested {
            node_id,
            collection,
            paint_id,
            index,
            color_target,
            ..
        } => {
            screen.harness.last_action = format!(
                "Host opened the eyedropper for {:?} in {} paint {} on {node_id}",
                color_target,
                collection.label(),
                if paint_id.is_empty() {
                    format!("#{index}")
                } else {
                    paint_id.to_string()
                }
            )
            .into();
        }
        _ => return None,
    }
    Some(NodeOutcome::Applied)
}

pub(crate) fn story_whole_layer_paints(
    node: &DesignPanelNode,
    collection: DesignPanelCollection,
) -> Option<&[DesignPaint]> {
    match collection {
        DesignPanelCollection::Fill => Some(&node.fills),
        DesignPanelCollection::Stroke => {
            node.stroke.as_ref().map(|stroke| stroke.paints.as_slice())
        }
        DesignPanelCollection::Effect
        | DesignPanelCollection::LayoutGrid
        | DesignPanelCollection::Export => None,
    }
}

pub(crate) fn story_whole_layer_paints_mut(
    node: &mut DesignPanelNode,
    collection: DesignPanelCollection,
) -> Option<&mut [DesignPaint]> {
    match collection {
        DesignPanelCollection::Fill => Some(&mut node.fills),
        DesignPanelCollection::Stroke => node
            .stroke
            .as_mut()
            .map(|stroke| stroke.paints.as_mut_slice()),
        DesignPanelCollection::Effect
        | DesignPanelCollection::LayoutGrid
        | DesignPanelCollection::Export => None,
    }
}

pub(crate) fn apply_story_paint_edit_phase(
    paint: &mut DesignPaint,
    snapshots: &mut HashMap<StoryPaintEditTarget, DesignPaint>,
    target: StoryPaintEditTarget,
    edit: &DesignPaintEdit,
    phase: DesignPanelEditPhase,
) {
    match phase {
        DesignPanelEditPhase::Begin => {
            snapshots.entry(target).or_insert_with(|| paint.clone());
        }
        DesignPanelEditPhase::Preview => {
            paint.apply_edit(edit);
        }
        DesignPanelEditPhase::Commit => {
            paint.apply_edit(edit);
            snapshots.remove(&target);
        }
        DesignPanelEditPhase::Cancel => {
            // Paint Cancel does not promise that `edit` contains the original
            // value. Restore the mock host snapshot captured at Begin.
            if let Some(original) = snapshots.remove(&target) {
                *paint = original;
            }
        }
    }
}

pub(crate) fn story_paint_mut<'a>(
    node: &'a mut DesignPanelNode,
    collection: DesignPanelCollection,
    paint_id: &SharedString,
    index: usize,
) -> Option<&'a mut DesignPaint> {
    let paints = match collection {
        DesignPanelCollection::Fill if node.kind == DesignPanelNodeKind::MultipleSelection => {
            Some(&mut node.selection_colors)
        }
        DesignPanelCollection::Fill => Some(&mut node.fills),
        DesignPanelCollection::Stroke => node.stroke.as_mut().map(|stroke| &mut stroke.paints),
        DesignPanelCollection::Effect
        | DesignPanelCollection::LayoutGrid
        | DesignPanelCollection::Export => None,
    }?;
    let resolved = if paint_id.is_empty() {
        (index < paints.len()).then_some(index)
    } else {
        paints.iter().position(|paint| paint.id == *paint_id)
    }?;
    paints.get_mut(resolved)
}

pub(crate) fn story_paint_shader_editor_value(
    current: &DesignShaderPropertyValue,
) -> Option<DesignShaderPropertyValue> {
    let vector = fanta_gpui::design::DesignEffectVector::new;
    Some(match current {
        DesignShaderPropertyValue::Boolean(value) => DesignShaderPropertyValue::Boolean(!value),
        DesignShaderPropertyValue::Text(value) => {
            DesignShaderPropertyValue::Text(format!("{value} · edited").into())
        }
        DesignShaderPropertyValue::Number(value) => DesignShaderPropertyValue::Number(value + 0.25),
        DesignShaderPropertyValue::AssetId(_) => {
            DesignShaderPropertyValue::AssetId("image:replacement".into())
        }
        DesignShaderPropertyValue::Color(_) => DesignShaderPropertyValue::Color(DesignColor::BLUE),
        DesignShaderPropertyValue::Point(_) => DesignShaderPropertyValue::Point(vector(0.25, 0.75)),
        DesignShaderPropertyValue::Line { .. } => DesignShaderPropertyValue::Line {
            start: vector(0.2, 0.8),
            end: vector(0.8, 0.2),
        },
        DesignShaderPropertyValue::Circle { .. } => DesignShaderPropertyValue::Circle {
            center: vector(0.4, 0.6),
            radius: 0.42,
        },
        DesignShaderPropertyValue::CirclePoint { .. } => DesignShaderPropertyValue::CirclePoint {
            center: vector(0.45, 0.55),
            radius: 0.38,
            angle: 135.,
        },
        DesignShaderPropertyValue::ColorPoint { .. } => DesignShaderPropertyValue::ColorPoint {
            point: vector(0.65, 0.35),
            color: DesignColor::PURPLE,
            variable_id: None,
        },
        DesignShaderPropertyValue::Gradient(_) => DesignShaderPropertyValue::Gradient(vec![
            fanta_gpui::design::DesignShaderGradientStop::new(0., DesignColor::BLUE),
            fanta_gpui::design::DesignShaderGradientStop::new(0.5, DesignColor::WHITE),
            fanta_gpui::design::DesignShaderGradientStop::new(1., DesignColor::PURPLE),
        ]),
        DesignShaderPropertyValue::Opaque { type_name, .. } => DesignShaderPropertyValue::Opaque {
            type_name: type_name.clone(),
            payload: "{\"storybookEdited\":true}".into(),
        },
        DesignShaderPropertyValue::VariableAlias { .. } => return None,
    })
}

pub(crate) fn apply_story_paint_shader_property_editor(
    node: &mut DesignPanelNode,
    collection: DesignPanelCollection,
    paint_id: &SharedString,
    index: usize,
    definition_id: &SharedString,
) -> bool {
    let Some(paint) = story_paint_mut(node, collection, paint_id, index) else {
        return false;
    };
    let DesignPaintPayload::Shader(shader) = &paint.payload else {
        return false;
    };
    let Some(next) = shader
        .property(definition_id)
        .and_then(story_paint_shader_editor_value)
    else {
        return false;
    };
    paint.apply_edit(&DesignPaintEdit {
        property: DesignPaintProperty::ShaderProperty {
            definition_id: definition_id.clone(),
        },
        value: DesignPaintValue::ShaderProperty(next),
    })
}

pub(crate) fn apply_story_paint_style(
    node: &mut DesignPanelNode,
    collection: DesignPanelCollection,
    selection: &DesignPaintStyleSelection,
    style: DesignPaintStyle,
) -> bool {
    if style.import_state != DesignPaintStyleImportState::Imported || style.paints.is_empty() {
        return false;
    }
    let mut paints = style.paints;
    assign_story_applied_style_paint_ids(&node.id, collection, &selection.style_id, &mut paints);
    let binding = DesignPaintStyleBinding::new(selection.clone(), style.name);
    match collection {
        DesignPanelCollection::Fill => {
            node.fills = paints;
            node.fill_style_binding = Some(binding);
            true
        }
        DesignPanelCollection::Stroke => {
            if node.stroke.is_none() {
                let first_paint = paints
                    .first()
                    .cloned()
                    .expect("non-empty Paint style checked above");
                node.stroke = Some(DesignStroke::for_node(
                    node.kind,
                    first_paint,
                    1.,
                    DesignStrokeAlign::Center,
                ));
            }
            node.stroke
                .as_mut()
                .expect("stroke was created above")
                .paints = paints;
            node.stroke_style_binding = Some(binding);
            true
        }
        DesignPanelCollection::Effect
        | DesignPanelCollection::LayoutGrid
        | DesignPanelCollection::Export => false,
    }
}

pub(crate) fn assign_story_applied_style_paint_ids(
    node_id: &SharedString,
    collection: DesignPanelCollection,
    style_id: &SharedString,
    paints: &mut [DesignPaint],
) {
    for (paint_index, paint) in paints.iter_mut().enumerate() {
        paint.id = format!(
            "{node_id}-{}-{style_id}-{paint_index}",
            collection.label().to_ascii_lowercase()
        )
        .into();
        if let DesignPaintPayload::Gradient(gradient) = &mut paint.payload {
            for (stop_index, stop) in gradient.stops.iter_mut().enumerate() {
                stop.id = format!("{}-stop-{stop_index}", paint.id).into();
            }
            paint.sync_legacy_projection();
        }
    }
}

pub(crate) fn apply_story_paint_variable(
    paint: &mut DesignPaint,
    target: &DesignPaintColorTarget,
    variable: &DesignVariable,
) -> bool {
    let Some(DesignVariableResolvedValue::Color(color)) = variable.resolved_value.as_ref() else {
        return false;
    };
    let mut binding = DesignPaintBinding::new(variable.id.clone(), variable.name.clone());
    binding.collection_name = Some(variable.collection_name.clone());
    let applied = match (&mut paint.payload, target) {
        (DesignPaintPayload::Solid(solid), DesignPaintColorTarget::Solid) => {
            solid.color = *color;
            solid.binding = Some(binding);
            true
        }
        (
            DesignPaintPayload::Gradient(gradient),
            DesignPaintColorTarget::GradientStop { stop_id, index },
        ) => {
            let stop = if stop_id.is_empty() {
                gradient.stops.get_mut(*index)
            } else {
                gradient.stops.iter_mut().find(|stop| stop.id == *stop_id)
            };
            stop.is_some_and(|stop| {
                stop.color = *color;
                stop.binding = Some(binding);
                true
            })
        }
        _ => false,
    };
    if applied {
        paint.sync_legacy_projection();
    }
    applied
}

pub(crate) fn detach_story_paint_variable(
    paint: &mut DesignPaint,
    target: &DesignPaintColorTarget,
    variable_id: &SharedString,
) -> bool {
    let detached = match (&mut paint.payload, target) {
        (DesignPaintPayload::Solid(solid), DesignPaintColorTarget::Solid) => {
            let matches = solid
                .binding
                .as_ref()
                .is_some_and(|binding| binding.variable_id == *variable_id);
            if matches {
                solid.binding = None;
            }
            matches
        }
        (
            DesignPaintPayload::Gradient(gradient),
            DesignPaintColorTarget::GradientStop { stop_id, index },
        ) => {
            let stop = if stop_id.is_empty() {
                gradient.stops.get_mut(*index)
            } else {
                gradient.stops.iter_mut().find(|stop| stop.id == *stop_id)
            };
            stop.is_some_and(|stop| {
                let matches = stop
                    .binding
                    .as_ref()
                    .is_some_and(|binding| binding.variable_id == *variable_id);
                if matches {
                    stop.binding = None;
                }
                matches
            })
        }
        _ => false,
    };
    if detached {
        paint.sync_legacy_projection();
    }
    detached
}

pub(crate) fn resolve_story_shader_mut<'a>(
    view_data: &'a mut DesignShaderViewData,
    selection: &DesignShaderSelection,
) -> Option<&'a mut DesignShaderDefinition> {
    match &selection.source {
        DesignShaderSource::Page => view_data
            .page_shaders
            .iter_mut()
            .find(|shader| shader.id == selection.shader_id),
        DesignShaderSource::Library { library_id } => view_data
            .libraries
            .iter_mut()
            .find(|library| library.id == *library_id)?
            .shaders
            .iter_mut()
            .find(|shader| shader.id == selection.shader_id),
    }
}

pub(crate) fn story_fractal_shader_definition() -> DesignShaderDefinition {
    let vector = fanta_gpui::design::DesignEffectVector::new;
    DesignShaderDefinition::new(
        "shader:page:fractal-noise",
        "Fractal noise",
        true,
        [
            DesignShaderPropertyDefinition::new(
                "def:scale",
                "Scale",
                DesignShaderPropertyKind::Number,
            )
            .with_default(DesignShaderPropertyValue::Number(4.))
            .with_description("Frequency multiplier"),
            DesignShaderPropertyDefinition::new(
                "def:tint",
                "Tint",
                DesignShaderPropertyKind::Color,
            )
            .with_default(DesignShaderPropertyValue::Color(DesignColor::PURPLE)),
            DesignShaderPropertyDefinition::new(
                "def:animate",
                "Animate",
                DesignShaderPropertyKind::Boolean,
            )
            .with_default(DesignShaderPropertyValue::Boolean(true)),
            DesignShaderPropertyDefinition::new(
                "def:center",
                "Center",
                DesignShaderPropertyKind::Point,
            )
            .with_default(DesignShaderPropertyValue::Point(vector(0.5, 0.5))),
            DesignShaderPropertyDefinition::new(
                "def:ray",
                "Refraction line",
                DesignShaderPropertyKind::Line,
            )
            .with_default(DesignShaderPropertyValue::Line {
                start: vector(0.1, 0.25),
                end: vector(0.9, 0.75),
            }),
            DesignShaderPropertyDefinition::new(
                "def:lens",
                "Lens",
                DesignShaderPropertyKind::Circle,
            )
            .with_default(DesignShaderPropertyValue::Circle {
                center: vector(0.45, 0.5),
                radius: 0.32,
            }),
            DesignShaderPropertyDefinition::new(
                "def:orbit",
                "Highlight orbit",
                DesignShaderPropertyKind::CirclePoint,
            )
            .with_default(DesignShaderPropertyValue::CirclePoint {
                center: vector(0.5, 0.5),
                radius: 0.4,
                angle: 40.,
            }),
            DesignShaderPropertyDefinition::new(
                "def:color-point",
                "Chromatic focus",
                DesignShaderPropertyKind::ColorPoint,
            )
            .with_default(DesignShaderPropertyValue::ColorPoint {
                point: vector(0.35, 0.65),
                color: DesignColor::BLUE,
                variable_id: None,
            }),
            DesignShaderPropertyDefinition::new(
                "def:gradient",
                "Dispersion gradient",
                DesignShaderPropertyKind::Gradient,
            )
            .with_default(DesignShaderPropertyValue::Gradient(vec![
                fanta_gpui::design::DesignShaderGradientStop::new(0., DesignColor::PURPLE),
                fanta_gpui::design::DesignShaderGradientStop::new(0.55, DesignColor::BLUE),
                fanta_gpui::design::DesignShaderGradientStop::new(1., DesignColor::WHITE),
            ])),
            DesignShaderPropertyDefinition::new(
                "def:future",
                "Future payload",
                DesignShaderPropertyKind::Unsupported,
            )
            .with_default(DesignShaderPropertyValue::Opaque {
                type_name: "MESH_PATCH".into(),
                payload: "{\"patches\":4,\"mode\":\"future\"}".into(),
            }),
        ],
    )
}

pub(crate) fn imported_story_shader_properties(
    shader_id: &str,
) -> Vec<DesignShaderPropertyDefinition> {
    match shader_id {
        "shader:library:liquid-metal" => vec![
            DesignShaderPropertyDefinition::new(
                "def:flow",
                "Flow",
                DesignShaderPropertyKind::Number,
            )
            .with_default(DesignShaderPropertyValue::Number(0.65)),
            DesignShaderPropertyDefinition::new(
                "def:color",
                "Metal color",
                DesignShaderPropertyKind::Color,
            )
            .with_default(DesignShaderPropertyValue::Color(DesignColor::WHITE)),
        ],
        _ => Vec::new(),
    }
}

pub(crate) fn story_shader_property_fallback(
    kind: DesignShaderPropertyKind,
) -> DesignShaderPropertyValue {
    let origin = fanta_gpui::design::DesignEffectVector::new(0., 0.);
    match kind {
        DesignShaderPropertyKind::Boolean => DesignShaderPropertyValue::Boolean(false),
        DesignShaderPropertyKind::Text => DesignShaderPropertyValue::Text(SharedString::default()),
        DesignShaderPropertyKind::Number => DesignShaderPropertyValue::Number(0.),
        DesignShaderPropertyKind::Image => {
            DesignShaderPropertyValue::AssetId("image:storybook-default".into())
        }
        DesignShaderPropertyKind::InstanceSwap => {
            DesignShaderPropertyValue::AssetId("component:storybook-default".into())
        }
        DesignShaderPropertyKind::Slot => {
            DesignShaderPropertyValue::AssetId("slot:storybook-default".into())
        }
        DesignShaderPropertyKind::Color => DesignShaderPropertyValue::Color(DesignColor::BLACK),
        DesignShaderPropertyKind::Point => DesignShaderPropertyValue::Point(origin),
        DesignShaderPropertyKind::Line => DesignShaderPropertyValue::Line {
            start: origin,
            end: fanta_gpui::design::DesignEffectVector::new(1., 1.),
        },
        DesignShaderPropertyKind::Circle => DesignShaderPropertyValue::Circle {
            center: origin,
            radius: 0.,
        },
        DesignShaderPropertyKind::CirclePoint => DesignShaderPropertyValue::CirclePoint {
            center: origin,
            radius: 0.,
            angle: 0.,
        },
        DesignShaderPropertyKind::ColorPoint => DesignShaderPropertyValue::ColorPoint {
            point: origin,
            color: DesignColor::BLACK,
            variable_id: None,
        },
        DesignShaderPropertyKind::Gradient => DesignShaderPropertyValue::Gradient(vec![
            fanta_gpui::design::DesignShaderGradientStop::new(0., DesignColor::BLACK),
            fanta_gpui::design::DesignShaderGradientStop::new(1., DesignColor::WHITE),
        ]),
        DesignShaderPropertyKind::Unsupported => DesignShaderPropertyValue::Opaque {
            type_name: "Unsupported".into(),
            payload: "Restored by the Storybook host".into(),
        },
    }
}

pub(crate) fn resolve_story_color_style_sample<'a>(
    view_data: &'a DesignColorStyleSampleViewData,
    selection: &DesignColorStyleSampleSelection,
) -> Option<&'a DesignColorStyleSample> {
    match &selection.source {
        DesignColorStyleSampleSource::Page => view_data
            .page_samples
            .iter()
            .find(|sample| sample.id == selection.sample_id),
        DesignColorStyleSampleSource::Library { library_id } => view_data
            .libraries
            .iter()
            .find(|library| library.id == *library_id)?
            .samples
            .iter()
            .find(|sample| sample.id == selection.sample_id),
    }
}

pub(crate) fn apply_story_color_style_sample(
    paint: &mut DesignPaint,
    target: &DesignPaintColorTarget,
    color: DesignColor,
) -> bool {
    let applied = match (&mut paint.payload, target) {
        (DesignPaintPayload::Solid(solid), DesignPaintColorTarget::Solid)
            if solid.binding.is_none() =>
        {
            solid.color = DesignColor::rgb(color.red, color.green, color.blue);
            paint.opacity = f32::from(color.alpha) / 255. * 100.;
            true
        }
        (
            DesignPaintPayload::Gradient(gradient),
            DesignPaintColorTarget::GradientStop { stop_id, index },
        ) => {
            let stop = if stop_id.is_empty() {
                gradient.stops.get_mut(*index)
            } else {
                gradient.stops.iter_mut().find(|stop| stop.id == *stop_id)
            };
            stop.is_some_and(|stop| {
                if stop.binding.is_some() {
                    return false;
                }
                stop.color = color;
                true
            })
        }
        _ => false,
    };
    if applied {
        paint.sync_legacy_projection();
    }
    applied
}

pub(crate) fn resolve_story_color_style<'a>(
    view_data: &'a DesignColorStyleViewData,
    selection: &DesignColorStyleSelection,
) -> Option<&'a DesignColorStyle> {
    match &selection.source {
        DesignColorStyleSource::Page => view_data
            .page_styles
            .iter()
            .find(|style| style.id == selection.style_id),
        DesignColorStyleSource::Library { library_id } => view_data
            .libraries
            .iter()
            .find(|library| library.id == *library_id)
            .and_then(|library| {
                library
                    .styles
                    .iter()
                    .find(|style| style.id == selection.style_id)
            }),
    }
}

pub(crate) fn apply_story_color_style(
    paint: &mut DesignPaint,
    target: &DesignPaintColorTarget,
    style: &DesignColorStyle,
) -> bool {
    let applied = match (&mut paint.payload, target) {
        (DesignPaintPayload::Solid(solid), DesignPaintColorTarget::Solid) => {
            solid.color = style.color;
            solid.binding = style.binding.clone();
            true
        }
        (
            DesignPaintPayload::Gradient(gradient),
            DesignPaintColorTarget::GradientStop { stop_id, index },
        ) => {
            let stop = if stop_id.is_empty() {
                gradient.stops.get_mut(*index)
            } else {
                gradient.stops.iter_mut().find(|stop| stop.id == *stop_id)
            };
            stop.is_some_and(|stop| {
                stop.color = style.color;
                stop.binding = style.binding.clone();
                true
            })
        }
        _ => false,
    };
    if applied {
        paint.sync_legacy_projection();
    }
    applied
}
