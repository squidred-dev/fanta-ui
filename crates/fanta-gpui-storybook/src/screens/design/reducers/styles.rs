//! Page background, frame presets, local resources and styles,
//! and variable-mode application on the exact page target.

use super::*;

#[allow(deprecated)]
pub(crate) fn reduce_page_and_local_styles(
    screen: &mut DesignScreen,
    panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    cx: &mut Context<Storybook>,
) -> bool {
    let (current_page_projection, legacy_variables_entry_is_enabled) = {
        let panel = panel.read(cx);
        let page_matches = story_page_local_styles_projection_is_current(
            panel.inspection_context().selection().kind()
                == fanta_gpui::prelude::DesignPanelSelectionKind::None,
            panel.page_view_data().map(|page| &page.page_id),
            panel.page_local_styles_view_data(),
            &screen.page_view_data.page_id,
            &screen.page_local_styles,
        );
        let legacy_variables_entry_is_enabled = matches!(
            panel.variables_entry_point(),
            DesignVariablesEntryPoint::LegacyRightSidebar {
                disabled_reason: None
            }
        );
        (page_matches, legacy_variables_entry_is_enabled)
    };
    let page_or_mode_handled = match action {
        DesignPanelAction::PageBackgroundEditRequested {
            page_id,
            color,
            phase,
        } => {
            if screen.page_view_data.page_id == *page_id {
                apply_story_page_background_edit_phase(
                    &mut screen.page_view_data.background.color,
                    &mut screen.page_background_edit_snapshots,
                    page_id,
                    *color,
                    *phase,
                );
                screen.last_action =
                    format!("Host {:?} Page background #{}", phase, color.hex()).into();
            } else {
                screen.page_background_edit_snapshots.remove(page_id);
                screen.last_action =
                    format!("Ignored stale Page background edit for {page_id}").into();
            }
            true
        }
        DesignPanelAction::PageBackgroundChangeRequested { page_id, color } => {
            if screen.page_view_data.page_id == *page_id {
                screen.page_view_data.background.color = *color;
                screen.last_action =
                    format!("Host changed Page background to #{}", color.hex()).into();
            } else {
                screen.last_action =
                    format!("Ignored stale Page background intent for {page_id}").into();
            }
            true
        }
        DesignPanelAction::FramePresetApplyRequested {
            node_id,
            selection,
            width,
            height,
        } => {
            let applied = apply_story_frame_preset(
                &mut screen.nodes,
                &screen.frame_presets,
                node_id,
                selection,
                *width,
                *height,
            );
            screen.last_action = if applied {
                format!(
                    "Host applied Frame preset {}/{} at {} × {}",
                    selection.group_id, selection.preset_id, width, height
                )
                .into()
            } else {
                format!(
                    "Ignored stale or unavailable Frame preset {}/{}",
                    selection.group_id, selection.preset_id
                )
                .into()
            };
            true
        }
        DesignPanelAction::LocalResourceBrowseRequested { page_id, category } => {
            screen.last_action = if screen.page_view_data.page_id == *page_id {
                format!("Host opened the {} browser", category.label()).into()
            } else {
                format!("Ignored stale resource browser intent for {page_id}").into()
            };
            true
        }
        DesignPanelAction::LocalResourceOpenRequested { page_id, resource } => {
            let opened = screen.page_view_data.page_id == *page_id
                && screen
                    .page_view_data
                    .local_resources
                    .resource(resource)
                    .is_some_and(|item| item.availability.can_open());
            screen.last_action = if opened {
                format!(
                    "Host opened Page resource {} from group {}",
                    resource.resource_id, resource.group_id
                )
                .into()
            } else {
                format!(
                    "Ignored stale or non-local resource {}",
                    resource.resource_id
                )
                .into()
            };
            true
        }
        DesignPanelAction::LocalResourceCreateRequested { page_id, kind } => {
            let accepted = if screen.page_view_data.page_id == *page_id {
                let group_id = match kind.category() {
                    DesignLocalResourceCategory::Styles => "page-local-styles",
                    DesignLocalResourceCategory::VariableCollections => "page-local-variables",
                };
                if let Some(group) = screen
                    .page_view_data
                    .local_resources
                    .groups
                    .iter_mut()
                    .find(|group| group.id.as_ref() == group_id)
                {
                    let resource_id = SharedString::from(format!(
                        "storybook-resource-{}",
                        screen.next_resource_id
                    ));
                    screen.next_resource_id += 1;
                    group.resources.push(DesignLocalResource::local(
                        resource_id,
                        format!("New {}", kind.label()),
                        *kind,
                    ));
                    true
                } else {
                    false
                }
            } else {
                false
            };
            screen.last_action = if accepted {
                format!("Host created a local {}", kind.label()).into()
            } else {
                format!("Ignored stale {} creation intent", kind.label()).into()
            };
            true
        }
        DesignPanelAction::LocalResourceImportRequested { page_id, resource } => {
            let imported = if screen.page_view_data.page_id == *page_id {
                screen
                    .page_view_data
                    .local_resources
                    .groups
                    .iter_mut()
                    .find(|group| group.id == resource.group_id && group.source == resource.source)
                    .and_then(|group| {
                        group
                            .resources
                            .iter_mut()
                            .find(|item| item.id == resource.resource_id)
                    })
                    .filter(|item| item.availability.can_import())
                    .map(|item| {
                        item.availability = DesignLocalResourceAvailability::Imported;
                    })
                    .is_some()
            } else {
                false
            };
            screen.last_action = if imported {
                format!("Host imported Page resource {}", resource.resource_id).into()
            } else {
                format!("Ignored unavailable resource {}", resource.resource_id).into()
            };
            true
        }
        DesignPanelAction::LocalStyleCommandRequested { target, command } => {
            let permissions = panel.read(cx).inspection_context().permissions();
            let permission_allows = match command {
                DesignLocalStyleCommand::Edit | DesignLocalStyleCommand::Duplicate => {
                    permissions.can_edit()
                }
                DesignLocalStyleCommand::Copy => permissions.can_copy(),
                DesignLocalStyleCommand::GoToDefinition => true,
            };
            let mut accepted = current_page_projection
                && permission_allows
                && story_local_style_targets_are_current(
                    &screen.page_local_styles,
                    std::slice::from_ref(target),
                    false,
                );
            if accepted && *command == DesignLocalStyleCommand::Duplicate {
                accepted = story_duplicate_local_style(
                    &mut screen.page_local_styles,
                    target,
                    screen.next_resource_id,
                );
                if accepted {
                    screen.next_resource_id += 1;
                }
            }
            screen.last_action = if accepted {
                format!("Host ran {} for {}", command.label(), target.style_id).into()
            } else {
                format!(
                    "Ignored stale or permission-invalid {} for {}",
                    command.label(),
                    target.style_id
                )
                .into()
            };
            true
        }
        DesignPanelAction::LocalStyleCreateRequested {
            page_id,
            kind,
            parent_folder_id,
        } => {
            let can_edit = panel.read(cx).inspection_context().permissions().can_edit();
            let accepted = current_page_projection
                && can_edit
                && story_create_local_style(
                    &mut screen.page_local_styles,
                    page_id,
                    *kind,
                    parent_folder_id.as_ref(),
                    screen.next_resource_id,
                );
            if accepted {
                screen.next_resource_id += 1;
            }
            screen.last_action = if accepted {
                format!("Host created a current-file {}", kind.label()).into()
            } else {
                format!("Ignored stale {} creation", kind.label()).into()
            };
            true
        }
        DesignPanelAction::LocalStyleFolderCreateRequested {
            page_id,
            kind,
            selected,
        } => {
            let can_edit = panel.read(cx).inspection_context().permissions().can_edit();
            let accepted = current_page_projection
                && can_edit
                && story_create_local_style_folder(
                    &mut screen.page_local_styles,
                    page_id,
                    *kind,
                    selected,
                    screen.next_resource_id,
                );
            if accepted {
                screen.next_resource_id += 1;
            }
            screen.last_action = if accepted {
                format!(
                    "Host created a {} folder around {} styles",
                    kind.label(),
                    selected.len()
                )
                .into()
            } else {
                format!("Ignored stale {} folder creation", kind.label()).into()
            };
            true
        }
        DesignPanelAction::LocalStylesDeleteRequested { targets } => {
            let can_edit = panel.read(cx).inspection_context().permissions().can_edit();
            let accepted = current_page_projection
                && can_edit
                && story_remove_local_style_targets(&mut screen.page_local_styles, targets)
                    .is_some();
            screen.last_action = if accepted {
                format!("Host deleted {} exact local styles", targets.len()).into()
            } else {
                "Ignored stale local-style deletion".into()
            };
            true
        }
        DesignPanelAction::LocalStylesMoveRequested {
            targets,
            destination,
        } => {
            let can_edit = panel.read(cx).inspection_context().permissions().can_edit();
            let accepted = current_page_projection
                && can_edit
                && story_move_local_styles(&mut screen.page_local_styles, targets, destination);
            screen.last_action = if accepted {
                format!(
                    "Host moved {} local styles to index {}",
                    targets.len(),
                    destination.expected_index
                )
                .into()
            } else {
                "Ignored stale local-style move".into()
            };
            true
        }
        DesignPanelAction::VariablesViewOpenRequested { page_id } => {
            screen.last_action = if current_page_projection
                && legacy_variables_entry_is_enabled
                && screen.page_view_data.page_id == *page_id
            {
                "Host opened Variables from the legacy right-sidebar entry".into()
            } else {
                format!("Ignored stale Variables navigation for {page_id}").into()
            };
            true
        }
        DesignPanelAction::VariableModeApplyRequested {
            target,
            collection_id,
            mode_id,
        } => {
            let key = match target {
                DesignPanelTarget::Page { page_id } => Some(page_id.clone()),
                DesignPanelTarget::Nodes { node_ids } if node_ids.len() == 1 => {
                    node_ids.first().cloned()
                }
                DesignPanelTarget::Nodes { .. } => None,
            };
            let applied = key
                .and_then(|key| screen.variable_mode_views.get_mut(&key))
                .filter(|view_data| view_data.target == *target)
                .and_then(|view_data| {
                    view_data
                        .collections
                        .iter_mut()
                        .find(|collection| collection.id == *collection_id)
                })
                .filter(|collection| {
                    collection
                        .mode(mode_id.as_ref())
                        .is_some_and(|mode| mode.disabled_reason.is_none())
                        && collection.disabled_reason.is_none()
                })
                .map(|collection| {
                    collection.resolved_mode_id = mode_id.clone();
                    collection.explicit_mode_id = Some(mode_id.clone());
                })
                .is_some();
            screen.last_action = if applied {
                format!("Host set explicit mode {mode_id} for collection {collection_id}").into()
            } else {
                format!("Ignored stale variable mode {mode_id}").into()
            };
            true
        }
        DesignPanelAction::VariableModeClearRequested {
            target,
            collection_id,
            explicit_mode_id,
        } => {
            let key = match target {
                DesignPanelTarget::Page { page_id } => Some(page_id.clone()),
                DesignPanelTarget::Nodes { node_ids } if node_ids.len() == 1 => {
                    node_ids.first().cloned()
                }
                DesignPanelTarget::Nodes { .. } => None,
            };
            let cleared = key
                .and_then(|key| screen.variable_mode_views.get_mut(&key))
                .filter(|view_data| view_data.target == *target)
                .and_then(|view_data| {
                    view_data
                        .collections
                        .iter_mut()
                        .find(|collection| collection.id == *collection_id)
                })
                .filter(|collection| {
                    collection.explicit_mode_id.as_ref() == Some(explicit_mode_id)
                        && collection.disabled_reason.is_none()
                })
                .map(|collection| {
                    collection.explicit_mode_id = None;
                    collection.resolved_mode_id = collection.default_mode_id.clone();
                })
                .is_some();
            screen.last_action = if cleared {
                format!("Host cleared explicit mode {explicit_mode_id} for {collection_id}").into()
            } else {
                format!("Ignored stale explicit mode {explicit_mode_id}").into()
            };
            true
        }
        _ => false,
    };
    if page_or_mode_handled {
        screen.apply_inspection_context(panel, cx);
        cx.notify();
        return true;
    }
    false
}

pub(crate) fn apply_story_frame_preset(
    nodes: &mut [DesignPanelNode],
    catalogs: &HashMap<SharedString, DesignFramePresetViewData>,
    node_id: &SharedString,
    selection: &DesignFramePresetSelection,
    width: f32,
    height: f32,
) -> bool {
    let valid_preset = catalogs
        .get(node_id)
        .filter(|view_data| view_data.target_node_id == *node_id && view_data.can_apply(selection))
        .and_then(|view_data| view_data.preset(selection))
        .is_some_and(|(_, preset)| preset.width == width && preset.height == height);
    valid_preset
        && nodes
            .iter_mut()
            .find(|node| node.id == *node_id && node.kind == DesignPanelNodeKind::Frame)
            .map(|node| {
                node.width = width;
                node.height = height;
            })
            .is_some()
}

pub(crate) fn apply_story_page_background_edit_phase(
    current: &mut DesignColor,
    snapshots: &mut HashMap<SharedString, DesignColor>,
    page_id: &SharedString,
    color: DesignColor,
    phase: DesignPanelEditPhase,
) {
    match phase {
        DesignPanelEditPhase::Begin => {
            snapshots.entry(page_id.clone()).or_insert(*current);
        }
        DesignPanelEditPhase::Preview => {
            snapshots.entry(page_id.clone()).or_insert(*current);
            *current = color;
        }
        DesignPanelEditPhase::Commit => {
            *current = color;
            snapshots.remove(page_id);
        }
        DesignPanelEditPhase::Cancel => {
            if let Some(original) = snapshots.remove(page_id) {
                *current = original;
            }
        }
    }
}

pub(crate) fn story_page_local_styles_projection_is_current(
    selection_is_page: bool,
    panel_page_id: Option<&SharedString>,
    panel_styles: Option<&DesignPageLocalStylesViewData>,
    host_page_id: &SharedString,
    host_styles: &DesignPageLocalStylesViewData,
) -> bool {
    selection_is_page
        && panel_page_id == Some(host_page_id)
        && panel_styles == Some(host_styles)
        && host_styles.target
            == DesignPanelTarget::Page {
                page_id: host_page_id.clone(),
            }
        && host_styles.is_valid()
}

pub(crate) fn story_local_style_entries_mut<'a>(
    view_data: &'a mut DesignPageLocalStylesViewData,
    kind: DesignLocalStyleKind,
    parent_folder_id: Option<&str>,
) -> Option<&'a mut Vec<DesignLocalStyleEntry>> {
    fn folder_entries_mut<'a>(
        entries: &'a mut [DesignLocalStyleEntry],
        folder_id: &str,
    ) -> Option<&'a mut Vec<DesignLocalStyleEntry>> {
        for entry in entries {
            if let DesignLocalStyleEntry::Folder {
                id,
                entries: children,
                ..
            } = entry
            {
                if id.as_ref() == folder_id {
                    return Some(children);
                }
                if let Some(found) = folder_entries_mut(children, folder_id) {
                    return Some(found);
                }
            }
        }
        None
    }

    let section = view_data
        .sections
        .iter_mut()
        .find(|section| section.kind == kind)?;
    match parent_folder_id {
        None => Some(&mut section.entries),
        Some(folder_id) => folder_entries_mut(&mut section.entries, folder_id),
    }
}

pub(crate) fn story_local_style_insertion_is_current(
    view_data: &DesignPageLocalStylesViewData,
    insertion: &DesignLocalStyleInsertion,
) -> bool {
    if !view_data.is_valid()
        || insertion
            .parent_folder_id
            .as_ref()
            .is_some_and(|folder_id| {
                !view_data.entry_path_is_enabled(insertion.kind, folder_id.as_ref())
            })
    {
        return false;
    }
    let Some(entries) = view_data.entries(
        insertion.kind,
        insertion
            .parent_folder_id
            .as_ref()
            .map(|folder_id| folder_id.as_ref()),
    ) else {
        return false;
    };
    if insertion.expected_index > entries.len() {
        return false;
    }
    let before = insertion
        .expected_index
        .checked_sub(1)
        .and_then(|index| entries.get(index))
        .map(DesignLocalStyleEntry::id);
    let after = entries
        .get(insertion.expected_index)
        .map(DesignLocalStyleEntry::id);
    before == insertion.expected_before_id.as_ref() && after == insertion.expected_after_id.as_ref()
}

pub(crate) fn story_local_style_targets_are_current(
    view_data: &DesignPageLocalStylesViewData,
    targets: &[DesignLocalStyleTarget],
    allow_empty: bool,
) -> bool {
    view_data.is_valid()
        && (!targets.is_empty() || allow_empty)
        && targets.iter().collect::<HashSet<_>>().len() == targets.len()
        && targets.iter().all(|target| {
            view_data.resolve_style(target).is_some()
                && view_data.entry_path_is_enabled(target.kind, target.style_id.as_ref())
        })
}

pub(crate) fn story_remove_local_style_targets(
    view_data: &mut DesignPageLocalStylesViewData,
    targets: &[DesignLocalStyleTarget],
) -> Option<Vec<DesignLocalStyleEntry>> {
    if !story_local_style_targets_are_current(view_data, targets, false) {
        return None;
    }
    let mut removed = Vec::with_capacity(targets.len());
    for target in targets {
        let entries = story_local_style_entries_mut(
            view_data,
            target.kind,
            target
                .parent_folder_id
                .as_ref()
                .map(|folder_id| folder_id.as_ref()),
        )?;
        let current_index = entries
            .iter()
            .position(|entry| entry.id() == &target.style_id)?;
        removed.push(entries.remove(current_index));
    }
    Some(removed)
}

pub(crate) fn story_default_local_style(
    kind: DesignLocalStyleKind,
    id: SharedString,
) -> DesignLocalStyleItem {
    let preview =
        match kind {
            DesignLocalStyleKind::Text => DesignLocalStylePreview::Text(
                DesignTypographyStyle::new(id.clone(), "New text style", "Inter", "Regular", 16.),
            ),
            DesignLocalStyleKind::Color => DesignLocalStylePreview::Color(vec![
                DesignPaint::solid(DesignColor::BLACK).with_id(format!("{id}-paint")),
            ]),
            DesignLocalStyleKind::Effect => DesignLocalStylePreview::Effect(vec![
                DesignEffect::new(DesignEffectKind::DropShadow).with_id(format!("{id}-effect")),
            ]),
            DesignLocalStyleKind::LayoutGuide => DesignLocalStylePreview::LayoutGuide(vec![
                DesignLayoutGrid::uniform(8., DesignColor::rgb(255, 0, 128))
                    .with_id(format!("{id}-guide")),
            ]),
        };
    DesignLocalStyleItem::new(id, "New style", preview)
}

pub(crate) fn story_create_local_style(
    view_data: &mut DesignPageLocalStylesViewData,
    page_id: &SharedString,
    kind: DesignLocalStyleKind,
    parent_folder_id: Option<&SharedString>,
    next_id: usize,
) -> bool {
    if view_data.page_id() != Some(page_id)
        || !view_data.is_valid()
        || view_data.create_disabled_reason.is_some()
        || parent_folder_id
            .as_ref()
            .is_some_and(|folder_id| !view_data.entry_path_is_enabled(kind, folder_id.as_ref()))
    {
        return false;
    }
    if view_data.section(kind).is_none() {
        if parent_folder_id.is_some() {
            return false;
        }
        view_data
            .sections
            .push(DesignLocalStyleSection::new(kind, []));
    }
    let Some(entries) = story_local_style_entries_mut(
        view_data,
        kind,
        parent_folder_id.map(|folder_id| folder_id.as_ref()),
    ) else {
        return false;
    };
    let id = SharedString::from(format!("storybook-local-style-{next_id}"));
    entries.push(DesignLocalStyleEntry::style(story_default_local_style(
        kind, id,
    )));
    true
}

pub(crate) fn story_duplicate_local_style(
    view_data: &mut DesignPageLocalStylesViewData,
    target: &DesignLocalStyleTarget,
    next_id: usize,
) -> bool {
    let Some(mut duplicate) = view_data.resolve_style(target).cloned() else {
        return false;
    };
    if !view_data.entry_path_is_enabled(target.kind, target.style_id.as_ref()) {
        return false;
    }
    duplicate.id = SharedString::from(format!("storybook-local-style-{next_id}"));
    duplicate.name = SharedString::from(format!("{} copy", duplicate.name));
    duplicate.disabled_reason = None;
    let Some(entries) = story_local_style_entries_mut(
        view_data,
        target.kind,
        target
            .parent_folder_id
            .as_ref()
            .map(|folder_id| folder_id.as_ref()),
    ) else {
        return false;
    };
    entries.insert(
        target.expected_index + 1,
        DesignLocalStyleEntry::style(duplicate),
    );
    true
}

pub(crate) fn story_create_local_style_folder(
    view_data: &mut DesignPageLocalStylesViewData,
    page_id: &SharedString,
    kind: DesignLocalStyleKind,
    selected: &[DesignLocalStyleTarget],
    next_id: usize,
) -> bool {
    if view_data.page_id() != Some(page_id)
        || !view_data.is_valid()
        || view_data.create_disabled_reason.is_some()
        || !story_local_style_targets_are_current(view_data, selected, true)
        || selected
            .iter()
            .any(|target| target.page_id != *page_id || target.kind != kind)
    {
        return false;
    }
    if view_data.section(kind).is_none() {
        if !selected.is_empty() {
            return false;
        }
        view_data
            .sections
            .push(DesignLocalStyleSection::new(kind, []));
    }
    let parent_folder_id = selected
        .first()
        .and_then(|target| target.parent_folder_id.clone());
    if selected
        .iter()
        .any(|target| target.parent_folder_id != parent_folder_id)
    {
        return false;
    }
    let insertion_index = selected
        .iter()
        .map(|target| target.expected_index)
        .min()
        .unwrap_or_else(|| {
            view_data
                .entries(
                    kind,
                    parent_folder_id
                        .as_ref()
                        .map(|folder_id| folder_id.as_ref()),
                )
                .map_or(0, <[DesignLocalStyleEntry]>::len)
        });
    let children = if selected.is_empty() {
        Vec::new()
    } else {
        let Some(children) = story_remove_local_style_targets(view_data, selected) else {
            return false;
        };
        children
    };
    let Some(entries) = story_local_style_entries_mut(
        view_data,
        kind,
        parent_folder_id
            .as_ref()
            .map(|folder_id| folder_id.as_ref()),
    ) else {
        return false;
    };
    entries.insert(
        insertion_index.min(entries.len()),
        DesignLocalStyleEntry::folder(
            format!("storybook-local-style-folder-{next_id}"),
            "New folder",
            children,
        ),
    );
    true
}

pub(crate) fn story_move_local_styles(
    view_data: &mut DesignPageLocalStylesViewData,
    targets: &[DesignLocalStyleTarget],
    destination: &DesignLocalStyleInsertion,
) -> bool {
    if !story_local_style_targets_are_current(view_data, targets, false)
        || targets.iter().any(|target| target.kind != destination.kind)
        || !story_local_style_insertion_is_current(view_data, destination)
    {
        return false;
    }
    let removed_before_destination = targets
        .iter()
        .filter(|target| {
            target.kind == destination.kind
                && target.parent_folder_id == destination.parent_folder_id
                && target.expected_index < destination.expected_index
        })
        .count();
    let insertion_index = destination
        .expected_index
        .saturating_sub(removed_before_destination);
    let Some(removed) = story_remove_local_style_targets(view_data, targets) else {
        return false;
    };
    let Some(entries) = story_local_style_entries_mut(
        view_data,
        destination.kind,
        destination
            .parent_folder_id
            .as_ref()
            .map(|folder_id| folder_id.as_ref()),
    ) else {
        return false;
    };
    if insertion_index > entries.len() {
        return false;
    }
    for (offset, entry) in removed.into_iter().enumerate() {
        entries.insert(insertion_index + offset, entry);
    }
    true
}
