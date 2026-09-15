//! Export configurations, export modes, animated exports, and
//! export previews, preflighted against the exact export target.

use super::*;

pub(crate) fn reduce(
    screen: &mut DesignScreen,
    panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    cx: &mut Context<Storybook>,
) -> bool {
    let (current_export_target, can_export) = {
        let panel = panel.read(cx);
        (
            story_export_target(panel.inspection_context()),
            panel.inspection_context().permissions().can_export(),
        )
    };
    let export_handled = match action {
        DesignPanelAction::ExportConfigurationAddRequested { target } => {
            let configuration_id =
                SharedString::from(format!("storybook-export-{}", screen.host.next_export_id));
            let capabilities = match target {
                DesignPanelTarget::Page { .. } => DesignStaticExportCapabilities::default(),
                DesignPanelTarget::Nodes { node_ids } => {
                    story_aggregate_static_export_capabilities(node_ids.iter().filter_map(
                        |node_id| screen.host.nodes.iter().find(|node| node.id == *node_id),
                    ))
                }
            };
            let accepted = apply_story_export_configuration_add(
                &screen.host.nodes,
                &mut screen.host.export_configurations,
                &current_export_target,
                target,
                can_export,
                DesignExportConfiguration::new_for_capabilities(
                    configuration_id.clone(),
                    DesignExportFormat::Png,
                    capabilities,
                ),
            );
            if accepted {
                screen.host.next_export_id += 1;
            }
            screen.harness.last_action = if accepted {
                format!("Host added export configuration {configuration_id} to {target:?}").into()
            } else {
                format!("Ignored stale or mixed export-add target {target:?}").into()
            };
            true
        }
        DesignPanelAction::ExportConfigurationRemoveRequested {
            target,
            configuration_id,
        } => {
            let accepted = apply_story_export_configuration_remove(
                &screen.host.nodes,
                &mut screen.host.export_configurations,
                &current_export_target,
                target,
                can_export,
                configuration_id,
            );
            screen.harness.last_action = if accepted {
                format!("Host removed export configuration {configuration_id} from {target:?}")
                    .into()
            } else {
                format!("Ignored stale or mixed export-remove {configuration_id} for {target:?}")
                    .into()
            };
            true
        }
        DesignPanelAction::ExportConfigurationChangeRequested {
            target,
            configuration_id,
            change,
            phase,
        } => {
            let accepted = apply_story_export_configuration_edit(
                &screen.host.nodes,
                &mut screen.host.export_configurations,
                &mut screen.edits.export_edit_snapshots,
                StoryExportConfigurationEdit {
                    current_target: &current_export_target,
                    requested_target: target,
                    can_export,
                    configuration_id,
                    change,
                    phase: *phase,
                },
            );
            screen.harness.last_action = if accepted {
                format!(
                        "Host observed {phase:?} for export {configuration_id} on exact {target:?}: {change:?}"
                    )
                    .into()
            } else {
                format!("Ignored stale or mixed {phase:?} export {configuration_id} for {target:?}")
                    .into()
            };
            true
        }
        DesignPanelAction::ExportModeChangeRequested { target, mode } => {
            let target_keys = story_export_target_keys(
                &screen.host.nodes,
                &current_export_target,
                target,
                can_export,
            );
            let accepted = target_keys
                .as_ref()
                .filter(|keys| keys.len() == 1)
                .and_then(|keys| keys.first())
                .filter(|key| screen.host.animated_exports.contains_key(*key))
                .cloned()
                .map(|key| screen.host.export_modes.insert(key, *mode))
                .is_some();
            screen.harness.last_action = if accepted {
                format!("Host switched export mode to {}", mode.label()).into()
            } else {
                format!("Ignored stale or multi-target export mode for {target:?}").into()
            };
            true
        }
        DesignPanelAction::AnimatedExportChangeRequested {
            target,
            change,
            phase,
        } => {
            let key = story_export_target_keys(
                &screen.host.nodes,
                &current_export_target,
                target,
                can_export,
            )
            .filter(|keys| keys.len() == 1)
            .and_then(|keys| keys.into_iter().next());
            let accepted = key.is_some_and(|key| match phase {
                DesignPanelEditPhase::Begin => {
                    let Some(original) = screen.host.animated_exports.get(&key).cloned() else {
                        return false;
                    };
                    screen
                        .edits
                        .animated_export_edit_snapshots
                        .entry(key)
                        .or_insert(Some(original));
                    true
                }
                DesignPanelEditPhase::Preview | DesignPanelEditPhase::Commit => {
                    let Some(animated) = screen.host.animated_exports.get_mut(&key) else {
                        return false;
                    };
                    let changed = if *change
                        == DesignAnimatedExportChange::Format(DesignAnimatedExportFormat::Svg)
                    {
                        animated.settings = DesignAnimatedExportSettings::Svg {
                            options: vec![
                                DesignAnimatedSvgOption::new(
                                    "precision",
                                    "Precision",
                                    "Balanced",
                                    ["Compact", "Balanced", "Exact"],
                                ),
                                DesignAnimatedSvgOption::new(
                                    "loop",
                                    "Loop",
                                    "Forever",
                                    ["Once", "Forever"],
                                ),
                            ],
                        };
                        true
                    } else {
                        animated.settings.apply_change(change.clone())
                    };
                    if changed && *phase == DesignPanelEditPhase::Commit {
                        screen.edits.animated_export_edit_snapshots.remove(&key);
                    }
                    changed
                }
                DesignPanelEditPhase::Cancel => {
                    let Some(original) = screen.edits.animated_export_edit_snapshots.remove(&key)
                    else {
                        return false;
                    };
                    if let Some(animated) = original {
                        screen.host.animated_exports.insert(key, animated);
                    } else {
                        screen.host.animated_exports.remove(&key);
                    }
                    true
                }
            });
            screen.harness.last_action = if accepted {
                format!("Host observed {phase:?} animated export change: {change:?}").into()
            } else {
                format!("Ignored stale or multi-target animated export for {target:?}").into()
            };
            true
        }
        DesignPanelAction::AnimatedExportRequested { target, settings } => {
            let accepted = story_export_target_keys(
                &screen.host.nodes,
                &current_export_target,
                target,
                can_export,
            )
            .filter(|keys| keys.len() == 1)
            .and_then(|keys| keys.into_iter().next())
            .and_then(|key| screen.host.animated_exports.get(&key))
            .is_some_and(|animated| {
                animated.settings == *settings && animated.capability.allows(settings)
            });
            screen.harness.last_action = if accepted {
                format!(
                    "Host exported {target:?} as animated {}",
                    settings.format().label()
                )
                .into()
            } else {
                format!("Ignored stale or multi-target animated export for {target:?}").into()
            };
            true
        }
        DesignPanelAction::ExportAllRequested { target } => {
            let target_keys = story_export_target_keys(
                &screen.host.nodes,
                &current_export_target,
                target,
                can_export,
            );
            let accepted = target_keys.as_ref().is_some_and(|target_keys| {
                story_uniform_export_configurations(&screen.host.export_configurations, target_keys)
                    .is_some_and(|configurations| !configurations.is_empty())
            });
            screen.harness.last_action = if accepted {
                format!("Host atomically exported every configured format for {target:?}").into()
            } else {
                format!("Ignored stale, empty, or mixed export target {target:?}").into()
            };
            true
        }
        DesignPanelAction::ExportPreviewRequested { target } => {
            let key = story_export_target_keys(
                &screen.host.nodes,
                &current_export_target,
                target,
                can_export,
            )
            .filter(|keys| keys.len() == 1)
            .and_then(|keys| keys.into_iter().next());
            let accepted = key
                .map(|key| {
                    screen.host.export_previews.insert(
                        key,
                        DesignExportPreviewState::Ready(
                            DesignExportPreview::new(1440, 900)
                                .with_thumbnail("storybook-requested-preview")
                                .with_estimated_output("824 KB"),
                        ),
                    );
                })
                .is_some();
            screen.harness.last_action = if accepted {
                format!("Host prepared an export preview for {target:?}").into()
            } else {
                format!("Ignored stale or multi-target export preview for {target:?}").into()
            };
            true
        }
        _ => false,
    };
    if export_handled {
        screen.apply_inspection_context(panel, cx);
        cx.notify();
        return true;
    }
    false
}

pub(crate) fn story_export_target(context: &DesignPanelInspectionContext) -> DesignPanelTarget {
    story_design_target(context).unwrap_or_else(|| DesignPanelTarget::Page {
        page_id: "storybook-page".into(),
    })
}

pub(crate) fn story_export_target_keys(
    nodes: &[DesignPanelNode],
    current_target: &DesignPanelTarget,
    requested_target: &DesignPanelTarget,
    can_export: bool,
) -> Option<Vec<SharedString>> {
    if !can_export || current_target != requested_target {
        return None;
    }
    match requested_target {
        DesignPanelTarget::Page { page_id } => Some(vec![page_id.clone()]),
        DesignPanelTarget::Nodes { node_ids } => {
            let unique = node_ids
                .iter()
                .map(SharedString::as_ref)
                .collect::<HashSet<_>>();
            (node_ids.len() == unique.len()
                && !node_ids.is_empty()
                && node_ids
                    .iter()
                    .all(|node_id| nodes.iter().any(|node| node.id == *node_id)))
            .then(|| node_ids.clone())
        }
    }
}

pub(crate) fn story_uniform_export_configurations(
    configurations: &HashMap<SharedString, Vec<DesignExportConfiguration>>,
    target_keys: &[SharedString],
) -> Option<Vec<DesignExportConfiguration>> {
    let first_key = target_keys.first()?;
    let first = configurations.get(first_key).cloned().unwrap_or_default();
    target_keys
        .iter()
        .skip(1)
        .all(|key| configurations.get(key).cloned().unwrap_or_default() == first)
        .then_some(first)
}

pub(crate) fn story_static_export_capabilities(
    node: &DesignPanelNode,
) -> DesignStaticExportCapabilities {
    let text = matches!(
        node.kind,
        DesignPanelNodeKind::Text | DesignPanelNodeKind::TextPath
    );
    DesignStaticExportCapabilities {
        can_include_text_bounding_box: text,
        can_include_svg_bounds: text,
        svg_outline_text_default: text,
        svg_simplify_stroke_default: !text,
    }
}

pub(crate) fn story_aggregate_static_export_capabilities<'a>(
    nodes: impl IntoIterator<Item = &'a DesignPanelNode>,
) -> DesignStaticExportCapabilities {
    let capabilities = nodes
        .into_iter()
        .map(story_static_export_capabilities)
        .collect::<Vec<_>>();
    if capabilities.is_empty() {
        return DesignStaticExportCapabilities::default();
    }
    DesignStaticExportCapabilities {
        can_include_text_bounding_box: capabilities
            .iter()
            .all(|capability| capability.can_include_text_bounding_box),
        can_include_svg_bounds: capabilities
            .iter()
            .all(|capability| capability.can_include_svg_bounds),
        svg_outline_text_default: capabilities
            .iter()
            .all(|capability| capability.svg_outline_text_default),
        svg_simplify_stroke_default: capabilities
            .iter()
            .all(|capability| capability.svg_simplify_stroke_default),
    }
}

pub(crate) fn story_export_projection(
    target: DesignPanelTarget,
    nodes: &[DesignPanelNode],
    configurations: &HashMap<SharedString, Vec<DesignExportConfiguration>>,
    modes: &HashMap<SharedString, DesignExportMode>,
    animated_exports: &HashMap<SharedString, DesignAnimatedExportViewData>,
    previews: &HashMap<SharedString, DesignExportPreviewState>,
) -> DesignExportViewData {
    match &target {
        DesignPanelTarget::Page { page_id } => DesignExportViewData {
            target: target.clone(),
            configurations: configurations.get(page_id).cloned().unwrap_or_default(),
            mode: modes.get(page_id).copied().unwrap_or_default(),
            static_capabilities: Default::default(),
            preview: previews.get(page_id).cloned(),
            animated: animated_exports.get(page_id).cloned(),
        },
        DesignPanelTarget::Nodes { node_ids } if node_ids.len() == 1 => {
            let node_id = &node_ids[0];
            let node = nodes.iter().find(|node| node.id == *node_id);
            DesignExportViewData {
                target: target.clone(),
                configurations: configurations.get(node_id).cloned().unwrap_or_default(),
                mode: modes.get(node_id).copied().unwrap_or_default(),
                static_capabilities: node
                    .map(story_static_export_capabilities)
                    .unwrap_or_default(),
                preview: previews.get(node_id).cloned(),
                animated: animated_exports.get(node_id).cloned(),
            }
        }
        DesignPanelTarget::Nodes { node_ids } => {
            let selected_nodes = node_ids
                .iter()
                .filter_map(|node_id| nodes.iter().find(|node| node.id == *node_id))
                .collect::<Vec<_>>();
            let exact_target = !node_ids.is_empty()
                && selected_nodes.len() == node_ids.len()
                && node_ids
                    .iter()
                    .map(SharedString::as_ref)
                    .collect::<HashSet<_>>()
                    .len()
                    == node_ids.len();
            DesignExportViewData {
                target: target.clone(),
                configurations: if exact_target {
                    story_uniform_export_configurations(configurations, node_ids)
                        .unwrap_or_default()
                } else {
                    Vec::new()
                },
                // Figma exposes multi-layer export as static export. Motion export,
                // previews, and the Static/Animated switch are single-target only.
                mode: DesignExportMode::Static,
                static_capabilities: if exact_target {
                    story_aggregate_static_export_capabilities(selected_nodes)
                } else {
                    DesignStaticExportCapabilities::default()
                },
                preview: None,
                animated: None,
            }
        }
    }
}

pub(crate) fn apply_story_export_configuration_add(
    nodes: &[DesignPanelNode],
    configurations: &mut HashMap<SharedString, Vec<DesignExportConfiguration>>,
    current_target: &DesignPanelTarget,
    requested_target: &DesignPanelTarget,
    can_export: bool,
    configuration: DesignExportConfiguration,
) -> bool {
    let Some(target_keys) =
        story_export_target_keys(nodes, current_target, requested_target, can_export)
    else {
        return false;
    };
    let Some(current) = story_uniform_export_configurations(configurations, &target_keys) else {
        return false;
    };
    if current
        .iter()
        .any(|candidate| candidate.id == configuration.id)
    {
        return false;
    }
    for key in target_keys {
        configurations
            .entry(key)
            .or_default()
            .push(configuration.clone());
    }
    true
}

pub(crate) fn apply_story_export_configuration_remove(
    nodes: &[DesignPanelNode],
    configurations: &mut HashMap<SharedString, Vec<DesignExportConfiguration>>,
    current_target: &DesignPanelTarget,
    requested_target: &DesignPanelTarget,
    can_export: bool,
    configuration_id: &SharedString,
) -> bool {
    let Some(target_keys) =
        story_export_target_keys(nodes, current_target, requested_target, can_export)
    else {
        return false;
    };
    let Some(current) = story_uniform_export_configurations(configurations, &target_keys) else {
        return false;
    };
    if !current
        .iter()
        .any(|configuration| configuration.id == *configuration_id)
    {
        return false;
    }
    for key in target_keys {
        configurations
            .entry(key)
            .or_default()
            .retain(|configuration| configuration.id != *configuration_id);
    }
    true
}

pub(crate) struct StoryExportConfigurationEdit<'a> {
    pub(crate) current_target: &'a DesignPanelTarget,
    pub(crate) requested_target: &'a DesignPanelTarget,
    pub(crate) can_export: bool,
    pub(crate) configuration_id: &'a SharedString,
    pub(crate) change: &'a DesignExportConfigurationChange,
    pub(crate) phase: DesignPanelEditPhase,
}

pub(crate) fn apply_story_export_configuration_edit(
    nodes: &[DesignPanelNode],
    configurations: &mut HashMap<SharedString, Vec<DesignExportConfiguration>>,
    snapshots: &mut StoryExportEditSnapshots,
    edit: StoryExportConfigurationEdit<'_>,
) -> bool {
    let StoryExportConfigurationEdit {
        current_target,
        requested_target,
        can_export,
        configuration_id,
        change,
        phase,
    } = edit;
    let Some(target_keys) =
        story_export_target_keys(nodes, current_target, requested_target, can_export)
    else {
        return false;
    };
    let edit_target = StoryExportEditTarget::new(requested_target, configuration_id.clone());
    if phase == DesignPanelEditPhase::Cancel {
        let Some(originals) = snapshots.remove(&edit_target) else {
            return false;
        };
        for (key, original) in originals {
            if let Some(original) = original {
                configurations.insert(key, original);
            } else {
                configurations.remove(&key);
            }
        }
        return true;
    }

    let Some(current) = story_uniform_export_configurations(configurations, &target_keys) else {
        return false;
    };
    if !current
        .iter()
        .any(|configuration| configuration.id == *configuration_id)
    {
        return false;
    }
    if phase == DesignPanelEditPhase::Begin {
        snapshots.entry(edit_target).or_insert_with(|| {
            target_keys
                .iter()
                .map(|key| (key.clone(), configurations.get(key).cloned()))
                .collect()
        });
        return true;
    }

    let target_capabilities = match requested_target {
        DesignPanelTarget::Page { .. } => DesignStaticExportCapabilities::default(),
        DesignPanelTarget::Nodes { node_ids } => story_aggregate_static_export_capabilities(
            node_ids
                .iter()
                .filter_map(|node_id| nodes.iter().find(|node| node.id == *node_id)),
        ),
    };
    for key in &target_keys {
        let Some(configuration) = configurations.get_mut(key).and_then(|configurations| {
            configurations
                .iter_mut()
                .find(|configuration| configuration.id == *configuration_id)
        }) else {
            // The complete target was validated above, so this can only be an
            // unexpected host-state race. Do not continue a partial write.
            return false;
        };
        configuration.apply_change(change.clone());
        if matches!(change, DesignExportConfigurationChange::Format(_)) {
            configuration.apply_target_defaults(target_capabilities);
        }
    }
    if phase == DesignPanelEditPhase::Commit {
        snapshots.remove(&edit_target);
    }
    true
}
