//! Media paint sources, drops, cropping, and video preview
//! transactions.

use super::*;

pub(crate) fn reduce(
    screen: &mut DesignScreen,
    action: &DesignPanelAction,
    node_index: usize,
    media_drop_capabilities: DesignMediaPaintCapabilities,
    _cx: &mut Context<Storybook>,
) -> Option<NodeOutcome> {
    let node = &mut screen.host.nodes[node_index];
    match action {
        DesignPanelAction::PaintMediaSourceActionRequested {
            node_id,
            collection,
            paint_id,
            index,
            source_id,
            action,
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
                let current_source_id = match &paint.payload {
                    DesignPaintPayload::Image(image) => &image.source.id,
                    DesignPaintPayload::Video(video) => &video.source.id,
                    _ => return false,
                };
                if current_source_id != source_id || !action.is_applicable_to(paint.paint_type()) {
                    return false;
                }
                let source = DesignPaintSource::new(
                    format!("{node_id}-{paint_id}-{}", action.slug()),
                    match action {
                        DesignMediaSourceAction::Upload => "Uploaded source",
                        DesignMediaSourceAction::MakeImage => "Generated image",
                        DesignMediaSourceAction::EditImage => "Edited image",
                    },
                );
                paint.apply_edit(&DesignPaintEdit {
                    property: DesignPaintProperty::Source,
                    value: DesignPaintValue::Source(source),
                })
            });
            screen.harness.last_action = if accepted {
                format!(
                    "Host completed {} for {} paint {} on {node_id}",
                    action.label(),
                    collection.label(),
                    if paint_id.is_empty() {
                        format!("#{index}")
                    } else {
                        paint_id.to_string()
                    }
                )
                .into()
            } else {
                format!(
                    "Host rejected stale {} output for {} paint {} on {node_id}",
                    action.label(),
                    collection.label(),
                    if paint_id.is_empty() {
                        format!("#{index}")
                    } else {
                        paint_id.to_string()
                    }
                )
                .into()
            };
        }
        DesignPanelAction::PaintMediaSourceDropRequested {
            node_id,
            collection,
            paint_id,
            index,
            expected_source_id,
            expected_media_kind,
            file,
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
                apply_story_media_source_drop(
                    paints,
                    paint_id,
                    *index,
                    expected_source_id,
                    *expected_media_kind,
                    file,
                    media_drop_capabilities,
                    &mut screen.host.next_media_source_id,
                )
            });
            screen.harness.last_action = format!(
                "Host {} {} file drop for {} paint {} on {node_id}",
                if accepted { "accepted" } else { "rejected" },
                file.kind.label(),
                collection.label(),
                if paint_id.is_empty() {
                    format!("#{index}")
                } else {
                    paint_id.to_string()
                }
            )
            .into();
        }
        DesignPanelAction::PaintMediaCropActionRequested {
            node_id,
            collection,
            paint_id,
            index,
            action,
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
            let paint = paints.and_then(|paints| {
                let resolved = if paint_id.is_empty() {
                    (*index < paints.len()).then_some(*index)
                } else {
                    paints.iter().position(|paint| paint.id == *paint_id)
                }?;
                paints.get_mut(resolved)
            });
            let committed_transform = paint.as_ref().and_then(|paint| match &paint.payload {
                DesignPaintPayload::Image(image) => image.placement.crop_transform(),
                DesignPaintPayload::Video(video) => video.placement.crop_transform(),
                _ => None,
            });
            let view = screen
                .host
                .media_paint_views
                .entry(node_id.clone())
                .or_default()
                .paints
                .iter_mut()
                .find(|view| view.matches(*collection, paint_id, *index));
            if let Some(view) = view {
                match action {
                    DesignMediaCropAction::Begin => {
                        view.crop_tool.active = true;
                        if let Some(transform) = committed_transform {
                            view.crop_tool.transform = transform;
                        }
                    }
                    DesignMediaCropAction::Preview {
                        transform,
                        zoom,
                        aspect_ratio,
                    } => {
                        view.crop_tool = DesignMediaCropToolState {
                            active: true,
                            transform: *transform,
                            zoom: *zoom,
                            aspect_ratio: *aspect_ratio,
                        };
                    }
                    DesignMediaCropAction::Commit {
                        transform,
                        zoom,
                        aspect_ratio,
                    } => {
                        view.crop_tool = DesignMediaCropToolState {
                            active: false,
                            transform: *transform,
                            zoom: *zoom,
                            aspect_ratio: *aspect_ratio,
                        };
                        if let Some(paint) = paint {
                            match &mut paint.payload {
                                DesignPaintPayload::Image(image) => {
                                    image.placement = DesignMediaPaintPlacement::Crop {
                                        transform: *transform,
                                    };
                                }
                                DesignPaintPayload::Video(video) => {
                                    video.placement = DesignMediaPaintPlacement::Crop {
                                        transform: *transform,
                                    };
                                }
                                _ => {}
                            }
                            paint.sync_legacy_projection();
                        }
                    }
                    DesignMediaCropAction::Cancel => {
                        view.crop_tool.active = false;
                        if let Some(transform) = committed_transform {
                            view.crop_tool.transform = transform;
                        }
                    }
                    DesignMediaCropAction::ResizeToFit => {
                        view.crop_tool.active = false;
                        view.crop_tool.transform = DesignPaintTransform::IDENTITY;
                        if let Some(paint) = paint {
                            match &mut paint.payload {
                                DesignPaintPayload::Image(image) => {
                                    image.placement = DesignMediaPaintPlacement::Crop {
                                        transform: DesignPaintTransform::IDENTITY,
                                    };
                                }
                                DesignPaintPayload::Video(video) => {
                                    video.placement = DesignMediaPaintPlacement::Crop {
                                        transform: DesignPaintTransform::IDENTITY,
                                    };
                                }
                                _ => {}
                            }
                            paint.sync_legacy_projection();
                        }
                    }
                }
            }
            screen.harness.last_action =
                format!("Host handled crop {action:?} for {paint_id} on {node_id}").into();
        }
        DesignPanelAction::PaintVideoPreviewActionRequested {
            node_id,
            collection,
            target,
            paint_id,
            index,
            action,
            ..
        } => {
            let edit_target = StoryPaintEditTarget::new(
                node_id.clone(),
                *collection,
                *target,
                paint_id.clone(),
                *index,
            );
            let scrub_snapshots = &mut screen.edits.video_scrub_snapshots;
            let preview = screen
                .host
                .media_paint_views
                .entry(node_id.clone())
                .or_default()
                .paints
                .iter_mut()
                .find(|view| view.matches(*collection, paint_id, *index))
                .and_then(|view| view.video_preview.as_mut());
            if let Some(preview) = preview
                && matches!(preview.status, DesignVideoPreviewStatus::Ready)
            {
                match action {
                    DesignVideoPreviewAction::Play => preview.playing = true,
                    DesignVideoPreviewAction::Pause => preview.playing = false,
                    DesignVideoPreviewAction::Seek { seconds } => {
                        preview.current_seconds = seconds.clamp(0., preview.duration_seconds);
                    }
                    DesignVideoPreviewAction::Scrub { seconds, phase } => match phase {
                        DesignPanelEditPhase::Begin => {
                            scrub_snapshots
                                .entry(edit_target)
                                .or_insert(preview.current_seconds);
                        }
                        DesignPanelEditPhase::Preview => {
                            preview.current_seconds = seconds.clamp(0., preview.duration_seconds);
                        }
                        DesignPanelEditPhase::Commit => {
                            preview.current_seconds = seconds.clamp(0., preview.duration_seconds);
                            scrub_snapshots.remove(&edit_target);
                        }
                        DesignPanelEditPhase::Cancel => {
                            if let Some(original) = scrub_snapshots.remove(&edit_target) {
                                preview.current_seconds = original;
                            }
                        }
                    },
                }
            }
            screen.harness.last_action =
                format!("Host handled video preview {action:?} for {paint_id} on {node_id}").into();
        }
        _ => return None,
    }
    Some(NodeOutcome::Applied)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_story_media_source_drop(
    paints: &mut [DesignPaint],
    paint_id: &SharedString,
    index: usize,
    expected_source_id: &SharedString,
    expected_media_kind: DesignMediaKind,
    file: &DesignMediaDroppedFile,
    capabilities: DesignMediaPaintCapabilities,
    next_media_source_id: &mut usize,
) -> bool {
    if DesignMediaFileKind::from_path(&file.path) != Some(file.kind)
        || !capabilities.allows_file_drop(file.kind)
    {
        return false;
    }
    let resolved_index = if paint_id.is_empty() {
        (index < paints.len()).then_some(index)
    } else {
        paints.iter().position(|paint| paint.id == *paint_id)
    };
    let Some(paint) = resolved_index.and_then(|index| paints.get_mut(index)) else {
        return false;
    };
    if paint.read_only {
        return false;
    }
    let (current_media_kind, current_source_id, placement, filters) = match &paint.payload {
        DesignPaintPayload::Image(image) => (
            DesignMediaKind::Image,
            image.source.id.clone(),
            image.placement,
            image.filters,
        ),
        DesignPaintPayload::Video(video) => (
            DesignMediaKind::Video,
            video.source.id.clone(),
            video.placement,
            video.filters,
        ),
        _ => return false,
    };
    if current_media_kind != expected_media_kind || current_source_id != *expected_source_id {
        return false;
    }

    let source_name = file
        .path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| file.kind.label().to_owned());
    let mut source = DesignPaintSource::new(
        allocate_story_media_source_id(next_media_source_id),
        source_name,
    );
    source.mime_type = Some(file.kind.mime_type().into());

    let mut replacement = match file.media_kind() {
        DesignMediaKind::Image => DesignPaint::image(source),
        DesignMediaKind::Video => DesignPaint::video(source),
    };
    match &mut replacement.payload {
        DesignPaintPayload::Image(image) => {
            image.placement = placement;
            image.filters = filters;
        }
        DesignPaintPayload::Video(video) => {
            video.placement = placement;
            video.filters = filters;
        }
        _ => unreachable!("the replacement constructors are media-only"),
    }
    replacement.id = paint.id.clone();
    replacement.opacity = paint.opacity;
    replacement.visible = paint.visible;
    replacement.blend_mode = paint.blend_mode;
    replacement.read_only = paint.read_only;
    *paint = replacement;
    true
}

pub(crate) fn allocate_story_media_source_id(next_media_source_id: &mut usize) -> SharedString {
    let source_id = format!("storybook-media-source-{}", *next_media_source_id).into();
    *next_media_source_id = (*next_media_source_id)
        .checked_add(1)
        .expect("the Storybook media source ID space is exhausted");
    source_id
}
