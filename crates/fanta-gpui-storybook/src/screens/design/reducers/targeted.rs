//! The §11 targeted multi-node preflight: validate every target
//! member, reject complete stale targets, then replay the single-node
//! reducer in the target's original order.

use super::*;

pub(crate) fn replay_preflighted_target(
    screen: &mut DesignScreen,
    panel: &Entity<DesignPanel>,
    action: &DesignPanelAction,
    cx: &mut Context<Storybook>,
) -> bool {
    if let Some((target, node_action)) = action.targeted_node_action() {
        let (current_target, can_edit) = {
            let panel = panel.read(cx);
            (
                story_design_target(panel.inspection_context()),
                panel.inspection_context().permissions().can_edit(),
            )
        };
        if let DesignPanelAction::PaintEditRequested {
            collection,
            target: paint_target,
            paint_id,
            index,
            edit,
            phase,
            ..
        } = node_action
        {
            let accepted = apply_story_targeted_common_paint_edit(
                &mut screen.host.nodes,
                &mut screen.edits.paint_edit_snapshots,
                current_target.as_ref(),
                target,
                can_edit,
                *collection,
                *paint_target,
                paint_id,
                *index,
                edit,
                *phase,
            );
            screen.harness.last_action = if accepted {
                format!(
                        "Host atomically applied {phase:?} for {:?} across the common {} collection on {target:?}",
                        edit.property,
                        collection.label(),
                    )
                    .into()
            } else {
                format!(
                    "Host rejected stale, reordered, or non-common {} paint edit for {target:?}",
                    collection.label(),
                )
                .into()
            };
            if accepted {
                screen.apply_inspection_context(panel, cx);
            }
            cx.notify();
            return true;
        }
        if let DesignPanelAction::EffectAddRequested { kind, .. } = node_action {
            let accepted = apply_story_targeted_effect_add(
                &mut screen.host.nodes,
                current_target.as_ref(),
                target,
                can_edit,
                *kind,
            );
            screen.harness.last_action = if accepted {
                format!(
                    "Host atomically added {} to the exact target {target:?}",
                    kind.label()
                )
                .into()
            } else {
                format!(
                        "Host rejected stale, permission-invalid, or partially unavailable {} for {target:?}",
                        kind.label()
                    )
                    .into()
            };
            if accepted {
                screen.apply_inspection_context(panel, cx);
            }
            cx.notify();
            return true;
        }
        let Some(retargeted_actions) = story_targeted_node_actions(
            &screen.host.nodes,
            current_target.as_ref(),
            target,
            can_edit,
            node_action,
        ) else {
            screen.harness.last_action =
                format!("Ignored invalid or stale targeted Design action for {target:?}").into();
            cx.notify();
            return true;
        };
        for retargeted_action in &retargeted_actions {
            screen.handle_action(panel.clone(), retargeted_action, cx);
        }
        screen.harness.last_action =
            format!("Host replayed one fully preflighted Design action across {target:?}").into();
        cx.notify();
        return true;
    }
    false
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_story_targeted_common_paint_edit(
    nodes: &mut [DesignPanelNode],
    snapshots: &mut HashMap<StoryPaintEditTarget, DesignPaint>,
    current_target: Option<&DesignPanelTarget>,
    requested_target: &DesignPanelTarget,
    can_edit: bool,
    collection: DesignPanelCollection,
    paint_target: DesignPaintTarget,
    paint_id: &SharedString,
    fallback_index: usize,
    edit: &DesignPaintEdit,
    phase: DesignPanelEditPhase,
) -> bool {
    if paint_target != DesignPaintTarget::WholeLayer
        || !matches!(
            collection,
            DesignPanelCollection::Fill | DesignPanelCollection::Stroke
        )
    {
        return false;
    }
    let Some(node_indices) =
        story_exact_editable_node_indices(nodes, current_target, requested_target, can_edit)
    else {
        return false;
    };
    let Some(reference_index) = node_indices.first().copied() else {
        return false;
    };
    let reference = &nodes[reference_index];
    let collection_is_unbound = match collection {
        DesignPanelCollection::Fill => reference.fill_style_binding.is_none(),
        DesignPanelCollection::Stroke => reference.stroke_style_binding.is_none(),
        DesignPanelCollection::Effect
        | DesignPanelCollection::LayoutGrid
        | DesignPanelCollection::Export => false,
    };
    let Some(reference_paints) = story_whole_layer_paints(reference, collection) else {
        return false;
    };
    let resolved_index = if paint_id.is_empty() {
        (fallback_index < reference_paints.len()).then_some(fallback_index)
    } else {
        reference_paints
            .iter()
            .position(|paint| paint.id == *paint_id)
    };
    let Some(resolved_index) = resolved_index else {
        return false;
    };
    let collections_remain_common = node_indices.iter().all(|node_index| {
        let node = &nodes[*node_index];
        let supported = match collection {
            DesignPanelCollection::Fill => {
                node.supports_fill()
                    && node.supports_section(DesignPanelSection::Fill)
                    && node.fill_style_binding.is_none()
            }
            DesignPanelCollection::Stroke => {
                node.supports_stroke()
                    && node.supports_section(DesignPanelSection::Stroke)
                    && node.stroke_style_binding.is_none()
            }
            DesignPanelCollection::Effect
            | DesignPanelCollection::LayoutGrid
            | DesignPanelCollection::Export => false,
        };
        let common = match collection {
            DesignPanelCollection::Fill => story_fill_collection_is_common(reference, node),
            DesignPanelCollection::Stroke => story_stroke_collection_is_common(reference, node),
            DesignPanelCollection::Effect
            | DesignPanelCollection::LayoutGrid
            | DesignPanelCollection::Export => false,
        };
        supported && common
    });
    if !collection_is_unbound || !collections_remain_common {
        return false;
    }

    let mut edits = Vec::with_capacity(node_indices.len());
    for node_index in node_indices {
        let node = &nodes[node_index];
        let Some(paint) = story_whole_layer_paints(node, collection)
            .and_then(|paints| paints.get(resolved_index))
        else {
            return false;
        };
        if paint.read_only {
            return false;
        }
        let key = StoryPaintEditTarget::new(
            node.id.clone(),
            collection,
            paint_target,
            paint.id.clone(),
            resolved_index,
        );
        let mut candidate = paint.clone();
        if phase != DesignPanelEditPhase::Cancel && !candidate.apply_edit(edit) {
            return false;
        }
        edits.push((node_index, key, paint.clone(), candidate));
    }

    let snapshot_count = edits
        .iter()
        .filter(|(_, key, _, _)| snapshots.contains_key(key))
        .count();
    let lifecycle_is_valid = match phase {
        DesignPanelEditPhase::Begin => snapshot_count == 0,
        DesignPanelEditPhase::Preview | DesignPanelEditPhase::Cancel => {
            snapshot_count == edits.len()
        }
        DesignPanelEditPhase::Commit => snapshot_count == 0 || snapshot_count == edits.len(),
    };
    if !lifecycle_is_valid {
        return false;
    }

    if phase == DesignPanelEditPhase::Begin {
        for (_, key, original, _) in edits {
            snapshots.insert(key, original);
        }
        return true;
    }

    let replacements = edits
        .iter()
        .map(|(node_index, key, _, candidate)| {
            let replacement = if phase == DesignPanelEditPhase::Cancel {
                snapshots
                    .get(key)
                    .cloned()
                    .expect("the complete Cancel snapshot set was preflighted")
            } else {
                candidate.clone()
            };
            (*node_index, key.clone(), replacement)
        })
        .collect::<Vec<_>>();
    for (node_index, _, replacement) in &replacements {
        story_whole_layer_paints_mut(&mut nodes[*node_index], collection)
            .expect("the common collection was preflighted")[resolved_index] = replacement.clone();
    }
    if matches!(
        phase,
        DesignPanelEditPhase::Commit | DesignPanelEditPhase::Cancel
    ) {
        for (_, key, _) in replacements {
            snapshots.remove(&key);
        }
    }
    true
}

pub(crate) fn story_targeted_node_actions(
    nodes: &[DesignPanelNode],
    current_target: Option<&DesignPanelTarget>,
    target: &DesignPanelTarget,
    can_edit: bool,
    action: &DesignPanelAction,
) -> Option<Vec<DesignPanelAction>> {
    let node_indices = story_exact_editable_node_indices(nodes, current_target, target, can_edit)?;
    if !node_indices
        .iter()
        .all(|index| story_targeted_node_action_is_applicable(&nodes[*index], action))
    {
        return None;
    }
    let DesignPanelTarget::Nodes { node_ids } = target else {
        unreachable!("exact node-target validation rejects Page targets")
    };
    node_ids
        .iter()
        .map(|node_id| action.retargeted_legacy_node_action(node_id.clone()))
        .collect()
}

pub(crate) fn story_exact_editable_node_indices(
    nodes: &[DesignPanelNode],
    current_target: Option<&DesignPanelTarget>,
    requested_target: &DesignPanelTarget,
    can_edit: bool,
) -> Option<Vec<usize>> {
    if !can_edit || current_target != Some(requested_target) {
        return None;
    }
    let DesignPanelTarget::Nodes { node_ids } = requested_target else {
        return None;
    };
    let unique_ids = node_ids
        .iter()
        .map(SharedString::as_ref)
        .collect::<HashSet<_>>();
    if node_ids.is_empty()
        || unique_ids.len() != node_ids.len()
        || node_ids.iter().any(|node_id| node_id.trim().is_empty())
    {
        return None;
    }
    node_ids
        .iter()
        .map(|node_id| {
            let mut matches = nodes
                .iter()
                .enumerate()
                .filter(|(_, node)| node.id == *node_id);
            let (index, _) = matches.next()?;
            matches.next().is_none().then_some(index)
        })
        .collect()
}

pub(crate) fn story_targeted_node_action_is_applicable(
    node: &DesignPanelNode,
    action: &DesignPanelAction,
) -> bool {
    match action {
        DesignPanelAction::PropertyChangeRequested {
            property, value, ..
        }
        | DesignPanelAction::PropertyEditRequested {
            property, value, ..
        } => story_multiple_property_edit_is_applicable(node, *property, value),
        DesignPanelAction::CollectionItemAddRequested {
            collection, target, ..
        } => {
            *target == DesignPaintTarget::WholeLayer
                && story_collection_add_is_applicable(node, *collection)
        }
        DesignPanelAction::EffectAddRequested { kind, .. } => {
            story_effect_add_is_applicable(node, *kind)
        }
        // The Storybook intentionally rejects multi-node leaves whose
        // sub-resource identity or shared-catalog side effects cannot be
        // preflighted by this compact mock reducer. A production host may
        // support more leaves through one native document transaction.
        _ => false,
    }
}

pub(crate) fn story_multiple_property_edit_is_applicable(
    node: &DesignPanelNode,
    property: DesignPanelProperty,
    value: &DesignPanelValue,
) -> bool {
    match (property, value) {
        (DesignPanelProperty::X | DesignPanelProperty::Y, DesignPanelValue::Number(value)) => {
            value.is_finite()
                && node.supports_position_coordinates()
                && node.supports_section(DesignPanelSection::Position)
        }
        (DesignPanelProperty::Rotation, DesignPanelValue::Number(value)) => {
            value.is_finite()
                && node.supports_transforms()
                && node.supports_section(DesignPanelSection::Position)
        }
        (
            DesignPanelProperty::Width | DesignPanelProperty::Height,
            DesignPanelValue::Number(value),
        ) => {
            value.is_finite()
                && *value >= 0.
                && node.supports_dimensions()
                && node.supports_section(DesignPanelSection::Layout)
        }
        (DesignPanelProperty::LockAspectRatio, DesignPanelValue::Bool(_)) => {
            !node.is_component_instance_child
                && node.supports_aspect_ratio_lock()
                && node.supports_dimensions()
                && node.supports_section(DesignPanelSection::Layout)
        }
        (
            DesignPanelProperty::HorizontalConstraint | DesignPanelProperty::VerticalConstraint,
            DesignPanelValue::Constraint(_),
        ) => node.supports_constraints() && node.supports_section(DesignPanelSection::Position),
        (DesignPanelProperty::Visible, DesignPanelValue::Bool(_)) => {
            node.supports_visibility() && node.supports_section(DesignPanelSection::Layer)
        }
        (DesignPanelProperty::BlendMode, DesignPanelValue::BlendMode(_)) => {
            node.supports_layer_appearance() && node.supports_section(DesignPanelSection::Layer)
        }
        (DesignPanelProperty::Opacity, DesignPanelValue::Number(value)) => {
            value.is_finite()
                && (0. ..=100.).contains(value)
                && node.supports_layer_appearance()
                && node.supports_section(DesignPanelSection::Layer)
        }
        (DesignPanelProperty::CornerRadius, DesignPanelValue::Number(value)) => {
            value.is_finite()
                && *value >= 0.
                && node.corner_capabilities.uniform_radius
                && node.supports_section(DesignPanelSection::Layer)
        }
        (
            DesignPanelProperty::CornerRadiusTopLeft
            | DesignPanelProperty::CornerRadiusTopRight
            | DesignPanelProperty::CornerRadiusBottomRight
            | DesignPanelProperty::CornerRadiusBottomLeft,
            DesignPanelValue::Number(value),
        ) => {
            value.is_finite()
                && *value >= 0.
                && node.corner_capabilities.independent_radii
                && node.supports_section(DesignPanelSection::Layer)
        }
        (DesignPanelProperty::IndependentCorners, DesignPanelValue::Bool(_)) => {
            node.corner_capabilities.independent_radii
                && node.supports_section(DesignPanelSection::Layer)
        }
        (DesignPanelProperty::CornerSmoothing, DesignPanelValue::Ratio(value)) => {
            value.is_finite()
                && (0. ..=1.).contains(value)
                && node.corner_capabilities.smoothing
                && node.supports_section(DesignPanelSection::Layer)
        }
        _ => false,
    }
}

pub(crate) fn story_collection_add_is_applicable(
    node: &DesignPanelNode,
    collection: DesignPanelCollection,
) -> bool {
    match collection {
        DesignPanelCollection::Fill => {
            node.supports_fill()
                && node.supports_section(DesignPanelSection::Fill)
                && node.fill_style_binding.is_none()
        }
        DesignPanelCollection::Stroke => {
            node.supports_stroke()
                && node.supports_section(DesignPanelSection::Stroke)
                && node.stroke_style_binding.is_none()
        }
        DesignPanelCollection::Effect => {
            story_effect_add_is_applicable(node, DesignEffectKind::DropShadow)
        }
        DesignPanelCollection::LayoutGrid => {
            node.supports_layout_guides()
                && node.supports_section(DesignPanelSection::LayoutGrid)
                && node.layout_grid_style_binding.is_none()
        }
        // Multi-node export has a dedicated exact-target reducer.
        DesignPanelCollection::Export => false,
    }
}

pub(crate) fn story_effect_add_is_applicable(
    node: &DesignPanelNode,
    kind: DesignEffectKind,
) -> bool {
    node.supports_effects()
        && node.supports_section(DesignPanelSection::Effects)
        && node.effect_style_binding.is_none()
        && node.can_use_effect_kind(kind, None)
}

pub(crate) fn apply_story_targeted_effect_add(
    nodes: &mut [DesignPanelNode],
    current_target: Option<&DesignPanelTarget>,
    requested_target: &DesignPanelTarget,
    can_edit: bool,
    kind: DesignEffectKind,
) -> bool {
    let Some(node_indices) =
        story_exact_editable_node_indices(nodes, current_target, requested_target, can_edit)
    else {
        return false;
    };
    if !node_indices
        .iter()
        .all(|index| story_effect_add_is_applicable(&nodes[*index], kind))
    {
        return false;
    }
    for index in node_indices {
        let node = &mut nodes[index];
        let effect_id = format!(
            "{}-effect-{}-{}",
            node.id,
            kind.label().to_ascii_lowercase().replace(' ', "-"),
            node.effects.len()
        );
        node.effects
            .push(DesignEffect::new(kind).with_id(effect_id));
    }
    true
}

pub(crate) fn story_common_property_binding<'a>(
    bindings: &'a HashMap<
        (SharedString, DesignPanelProperty),
        DesignPanelPropertyBinding<DesignPanelValue>,
    >,
    node_ids: &[SharedString],
    property: DesignPanelProperty,
) -> Option<&'a DesignPanelPropertyBinding<DesignPanelValue>> {
    let first_id = node_ids.first()?;
    let binding = bindings.get(&(first_id.clone(), property))?;
    node_ids
        .iter()
        .skip(1)
        .all(|node_id| bindings.get(&(node_id.clone(), property)) == Some(binding))
        .then_some(binding)
}

pub(crate) fn story_multiple_inspection_context(
    first: &DesignPanelNode,
    second: &DesignPanelNode,
    permissions: DesignPanelPermissions,
) -> (
    DesignPanelInspectionContext,
    Vec<(
        DesignPanelProperty,
        DesignPanelPropertyValueState<DesignPanelValue>,
    )>,
) {
    let (aggregate, property_states) = aggregate_story_multiple_selection(first, second);
    let selection = DesignPanelMultipleSelection::new(aggregate, second.clone());
    (
        DesignPanelInspectionContext::multiple(
            selection,
            DesignPanelParentLayout::Mixed,
            permissions,
        ),
        property_states,
    )
}

pub(crate) fn story_widget_dimension_property_states(
    width: f32,
    height: f32,
) -> Vec<(
    DesignPanelProperty,
    DesignPanelPropertyValueState<DesignPanelValue>,
)> {
    [
        (DesignPanelProperty::Width, width),
        (DesignPanelProperty::Height, height),
    ]
    .into_iter()
    .map(|(property, value)| {
        (
            property,
            DesignPanelPropertyValueState::Uniform(DesignPanelValue::Number(value))
                .read_only_with_reason("Widget dimensions are read-only in the Plugin API"),
        )
    })
    .collect()
}

pub(crate) fn story_paint_semantic_projection(paint: &DesignPaint) -> DesignPaint {
    let mut projected = paint.clone();
    projected.id = SharedString::default();
    if let DesignPaintPayload::Gradient(gradient) = &mut projected.payload {
        for stop in &mut gradient.stops {
            stop.id = SharedString::default();
        }
    }
    projected.sync_legacy_projection();
    projected
}

pub(crate) fn story_paint_collections_are_semantically_equal(
    first: &[DesignPaint],
    second: &[DesignPaint],
) -> bool {
    first.len() == second.len()
        && first.iter().zip(second).all(|(first, second)| {
            story_paint_semantic_projection(first) == story_paint_semantic_projection(second)
        })
}

pub(crate) fn story_strokes_are_semantically_equal(
    first: Option<&DesignStroke>,
    second: Option<&DesignStroke>,
) -> bool {
    match (first, second) {
        (None, None) => true,
        (Some(first), Some(second)) => {
            let mut first = first.clone();
            let mut second = second.clone();
            for paint in &mut first.paints {
                *paint = story_paint_semantic_projection(paint);
            }
            for paint in &mut second.paints {
                *paint = story_paint_semantic_projection(paint);
            }
            first == second
        }
        (None, Some(_)) | (Some(_), None) => false,
    }
}

pub(crate) fn story_fill_collection_is_common(
    first: &DesignPanelNode,
    second: &DesignPanelNode,
) -> bool {
    first.fill_style_binding == second.fill_style_binding
        && story_paint_collections_are_semantically_equal(&first.fills, &second.fills)
}

pub(crate) fn story_stroke_collection_is_common(
    first: &DesignPanelNode,
    second: &DesignPanelNode,
) -> bool {
    first.stroke_style_binding == second.stroke_style_binding
        && story_strokes_are_semantically_equal(first.stroke.as_ref(), second.stroke.as_ref())
}

pub(crate) fn aggregate_story_multiple_selection(
    first: &DesignPanelNode,
    second: &DesignPanelNode,
) -> (
    DesignPanelNode,
    Vec<(
        DesignPanelProperty,
        DesignPanelPropertyValueState<DesignPanelValue>,
    )>,
) {
    let corner_capabilities = DesignCornerCapabilities {
        uniform_radius: first.corner_capabilities.uniform_radius
            && second.corner_capabilities.uniform_radius,
        independent_radii: first.corner_capabilities.independent_radii
            && second.corner_capabilities.independent_radii,
        smoothing: first.corner_capabilities.smoothing && second.corner_capabilities.smoothing,
    };
    let dimensions = first.supports_dimensions() && second.supports_dimensions();
    let visibility = first.supports_visibility() && second.supports_visibility();
    let position_coordinates =
        first.supports_position_coordinates() && second.supports_position_coordinates();
    let arrange = first.supports_arrange() && second.supports_arrange();
    let transforms = first.supports_transforms() && second.supports_transforms();
    let aspect_ratio_lock =
        first.supports_aspect_ratio_lock() && second.supports_aspect_ratio_lock();
    let auto_layout_child =
        first.supports_auto_layout_child() && second.supports_auto_layout_child();
    let add_auto_layout = first.supports_add_auto_layout() && second.supports_add_auto_layout();
    let fill = first.supports_fill() && second.supports_fill();
    let stroke = first.supports_stroke() && second.supports_stroke();
    let layer_appearance = first.supports_layer_appearance() && second.supports_layer_appearance();
    let effects = first.supports_effects() && second.supports_effects();
    let layout_guides = first.supports_layout_guides() && second.supports_layout_guides();
    let constraints = first.supports_constraints() && second.supports_constraints();
    let common_fills = fill && story_fill_collection_is_common(first, second);
    let common_stroke = stroke && story_stroke_collection_is_common(first, second);
    let selection_color_aggregate = aggregate_story_selection_colors_for_collections(
        [first, second],
        !common_fills,
        !common_stroke,
    );
    let mut sections = vec![DesignPanelSection::Position];
    if dimensions {
        sections.push(DesignPanelSection::Layout);
    }
    if visibility || layer_appearance || corner_capabilities.has_any() {
        sections.push(DesignPanelSection::Layer);
    }
    if !selection_color_aggregate.colors.is_empty() {
        sections.push(DesignPanelSection::Selection);
    }
    if fill {
        sections.push(DesignPanelSection::Fill);
    }
    if stroke {
        sections.push(DesignPanelSection::Stroke);
    }
    if effects {
        sections.push(DesignPanelSection::Effects);
    }
    if layout_guides {
        sections.push(DesignPanelSection::LayoutGrid);
    }
    sections.push(DesignPanelSection::Export);

    let mut effect_capabilities = DesignEffectCapabilities {
        kind_availability: Vec::new(),
        progressive_blur: first.effect_capabilities.progressive_blur
            && second.effect_capabilities.progressive_blur,
        shadow_blend_mode: first.effect_capabilities.shadow_blend_mode
            && second.effect_capabilities.shadow_blend_mode,
        shadow_spread: first.effect_capabilities.shadow_spread
            && second.effect_capabilities.shadow_spread,
        show_shadow_behind_transparent_areas: first
            .effect_capabilities
            .show_shadow_behind_transparent_areas
            && second
                .effect_capabilities
                .show_shadow_behind_transparent_areas,
    };
    for kind in DesignEffectKind::ALL {
        let available = first.effect_style_binding.is_none()
            && second.effect_style_binding.is_none()
            && first.can_use_effect_kind(kind, None)
            && second.can_use_effect_kind(kind, None);
        effect_capabilities.set_kind_availability(if available {
            DesignEffectKindAvailability::available(kind)
        } else {
            DesignEffectKindAvailability::unavailable(
                kind,
                "Unavailable or at its limit for part of the selection",
            )
        });
    }

    let mut aggregate = first.clone();
    aggregate.name = "2 layers selected".into();
    aggregate.is_component_instance_child =
        first.is_component_instance_child || second.is_component_instance_child;
    aggregate.capabilities = Some(DesignPanelNodeCapabilities {
        sections,
        dimensions,
        visibility,
        position_coordinates,
        arrange,
        transforms,
        aspect_ratio_lock,
        auto_layout_child,
        add_auto_layout,
        auto_layout_container: false,
        grid_auto_layout: false,
        resize_to_fit: first.supports_resize_to_fit() && second.supports_resize_to_fit(),
        clip_content: false,
        fill,
        stroke,
        layer_appearance,
        pass_through_blend: first.supports_pass_through_blend()
            && second.supports_pass_through_blend(),
        effects,
        constraints,
        layout_guides,
    });
    aggregate.layout = None;
    aggregate.corner_capabilities = corner_capabilities;
    if common_fills {
        if first.fill_shows_in_exports != second.fill_shows_in_exports {
            aggregate.fill_shows_in_exports = None;
        }
    } else {
        aggregate.fill_style_binding = None;
        aggregate.fills.clear();
        aggregate.fill_shows_in_exports = None;
    }
    if !common_stroke {
        aggregate.stroke = None;
        aggregate.stroke_style_binding = None;
    }
    aggregate.effects.clear();
    aggregate.effect_capabilities = effect_capabilities;
    aggregate.effect_style_binding = None;
    aggregate.layout_grids.clear();
    aggregate.layout_grid_style_binding = None;
    aggregate.export_settings.clear();
    aggregate.typography = None;
    aggregate.text_path = None;
    aggregate.text_path_start_data = None;
    aggregate.vector_edit = None;
    aggregate.component_context = None;
    aggregate.component_properties.clear();
    aggregate.media = None;
    aggregate.shape_geometry = DesignShapeGeometry::None;
    aggregate.section = None;
    aggregate.transform_modifiers.clear();
    aggregate.is_mask = false;
    aggregate.mask_type = None;
    aggregate.selection_color_aggregate = selection_color_aggregate;
    aggregate.selection_colors.clear();

    let state = |same, value| {
        if same {
            DesignPanelPropertyValueState::Uniform(value)
        } else {
            DesignPanelPropertyValueState::Mixed
        }
    };
    let property_states = vec![
        (
            DesignPanelProperty::Visible,
            state(
                first.visible == second.visible,
                DesignPanelValue::Bool(first.visible),
            ),
        ),
        (
            DesignPanelProperty::X,
            state(first.x == second.x, DesignPanelValue::Number(first.x)),
        ),
        (
            DesignPanelProperty::Y,
            state(first.y == second.y, DesignPanelValue::Number(first.y)),
        ),
        (
            DesignPanelProperty::Width,
            state(
                first.width == second.width,
                DesignPanelValue::Number(first.width),
            ),
        ),
        (
            DesignPanelProperty::Height,
            state(
                first.height == second.height,
                DesignPanelValue::Number(first.height),
            ),
        ),
        (
            DesignPanelProperty::Rotation,
            state(
                first.rotation == second.rotation,
                DesignPanelValue::Number(first.rotation),
            ),
        ),
        (
            DesignPanelProperty::LockAspectRatio,
            state(
                first.lock_aspect_ratio == second.lock_aspect_ratio,
                DesignPanelValue::Bool(first.lock_aspect_ratio),
            ),
        ),
        (
            DesignPanelProperty::HorizontalConstraint,
            state(
                first.horizontal_constraint == second.horizontal_constraint,
                DesignPanelValue::Constraint(first.horizontal_constraint),
            ),
        ),
        (
            DesignPanelProperty::VerticalConstraint,
            state(
                first.vertical_constraint == second.vertical_constraint,
                DesignPanelValue::Constraint(first.vertical_constraint),
            ),
        ),
        (
            DesignPanelProperty::Opacity,
            state(
                first.opacity == second.opacity,
                DesignPanelValue::Number(first.opacity),
            ),
        ),
        (
            DesignPanelProperty::BlendMode,
            state(
                first.blend_mode == second.blend_mode,
                DesignPanelValue::BlendMode(first.blend_mode),
            ),
        ),
        (
            DesignPanelProperty::CornerRadius,
            state(
                first.corner_radii == second.corner_radii,
                DesignPanelValue::Number(first.corner_radii[0]),
            ),
        ),
        (
            DesignPanelProperty::CornerRadiusTopLeft,
            state(
                first.corner_radii[0] == second.corner_radii[0],
                DesignPanelValue::Number(first.corner_radii[0]),
            ),
        ),
        (
            DesignPanelProperty::CornerRadiusTopRight,
            state(
                first.corner_radii[1] == second.corner_radii[1],
                DesignPanelValue::Number(first.corner_radii[1]),
            ),
        ),
        (
            DesignPanelProperty::CornerRadiusBottomRight,
            state(
                first.corner_radii[2] == second.corner_radii[2],
                DesignPanelValue::Number(first.corner_radii[2]),
            ),
        ),
        (
            DesignPanelProperty::CornerRadiusBottomLeft,
            state(
                first.corner_radii[3] == second.corner_radii[3],
                DesignPanelValue::Number(first.corner_radii[3]),
            ),
        ),
        (
            DesignPanelProperty::IndependentCorners,
            state(
                first.independent_corners == second.independent_corners,
                DesignPanelValue::Bool(first.independent_corners),
            ),
        ),
        (
            DesignPanelProperty::CornerSmoothing,
            state(
                first.corner_smoothing == second.corner_smoothing,
                DesignPanelValue::Ratio(first.corner_smoothing),
            ),
        ),
    ];

    (aggregate, property_states)
}
