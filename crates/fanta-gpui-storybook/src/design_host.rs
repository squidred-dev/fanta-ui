use std::collections::{HashMap, HashSet};

use fanta_gpui::prelude::*;
use gpui::SharedString;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) struct StoryPaintEditTarget {
    node_id: SharedString,
    collection: DesignPanelCollection,
    target: DesignPaintTarget,
    paint_id: SharedString,
    legacy_index: Option<usize>,
}

impl StoryPaintEditTarget {
    pub(super) fn new(
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
    ) -> Self {
        let legacy_index = paint_id.is_empty().then_some(index);
        Self {
            node_id,
            collection,
            target,
            paint_id,
            legacy_index,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) struct StoryLayoutGridEditTarget {
    node_id: SharedString,
    pub(super) guide_id: SharedString,
    pub(super) legacy_index: Option<usize>,
    pub(super) property: DesignPanelProperty,
}

impl StoryLayoutGridEditTarget {
    pub(super) fn new(
        node_id: SharedString,
        guide_id: SharedString,
        index: usize,
        property: DesignPanelProperty,
    ) -> Self {
        Self {
            node_id,
            legacy_index: guide_id.is_empty().then_some(index),
            guide_id,
            property: property.with_layout_grid_index(0),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) struct StoryNodeEditTarget {
    node_id: SharedString,
    transaction_id: SharedString,
}

impl StoryNodeEditTarget {
    pub(super) fn new(node_id: SharedString, transaction_id: impl Into<SharedString>) -> Self {
        Self {
            node_id,
            transaction_id: transaction_id.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) struct StoryExportEditTarget {
    target_id: SharedString,
    configuration_id: SharedString,
}

impl StoryExportEditTarget {
    pub(super) fn new(target: &DesignPanelTarget, configuration_id: SharedString) -> Self {
        Self {
            target_id: format!("{target:?}").into(),
            configuration_id,
        }
    }
}

type StoryExportConfigurationSnapshot = Vec<(SharedString, Option<Vec<DesignExportConfiguration>>)>;
pub(super) type StoryExportEditSnapshots =
    HashMap<StoryExportEditTarget, StoryExportConfigurationSnapshot>;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) struct StorySelectionColorEditTarget {
    target: SharedString,
    selection_color_id: SharedString,
}

impl StorySelectionColorEditTarget {
    pub(super) fn new(target: &DesignPanelTarget, selection_color_id: SharedString) -> Self {
        Self {
            target: format!("{target:?}").into(),
            selection_color_id,
        }
    }
}

pub(super) fn storybook_plugin_header_controls() -> [DesignSelectionHeaderControl; 2] {
    let menu = DesignSelectionHeaderControl::new(
        "storybook-plugin-menu",
        DesignSelectionHeaderControlKind::HostDefined,
    )
    .with_icon(DesignSelectionHeaderControlIcon::Lucide(
        LucideIcon::Sparkles,
    ))
    .with_tooltip("Plugin actions")
    .with_menu_items([
        DesignSelectionHeaderMenuItem::new(
            "inspect-plugin-metadata",
            "Inspect plugin metadata",
            DesignSelectionHeaderCommand::HostDefined {
                command_id: "inspect-plugin-metadata".into(),
            },
        )
        .viewer_safe(),
        DesignSelectionHeaderMenuItem::new(
            "apply-plugin-transform",
            "Apply plugin transform",
            DesignSelectionHeaderCommand::HostDefined {
                command_id: "apply-plugin-transform".into(),
            },
        ),
    ]);
    let direct = DesignSelectionHeaderControl::new(
        "storybook-plugin-action",
        DesignSelectionHeaderControlKind::HostDefined,
    )
    .with_icon(DesignSelectionHeaderControlIcon::Lucide(
        LucideIcon::Sparkles,
    ))
    .with_tooltip("Inspect host plugin state")
    .viewer_safe();
    [menu, direct]
}

pub(super) fn story_property_copy_status(
    target: &fanta_gpui::prelude::DesignPanelTarget,
    property: DesignPanelProperty,
    displayed_value: &SharedString,
) -> SharedString {
    format!("Host copied {property:?} = “{displayed_value}” from {target:?}").into()
}

pub(super) fn story_viewer_number(value: f32) -> String {
    if (value - value.round()).abs() < 0.001 {
        format!("{value:.0}")
    } else {
        let mut formatted = format!("{value:.2}");
        while formatted.ends_with('0') {
            formatted.pop();
        }
        formatted
    }
}

pub(super) fn story_viewer_px(value: f32) -> String {
    format!("{}px", story_viewer_number(value))
}

pub(super) fn story_viewer_component_section(
    context: &DesignComponentContext,
) -> DesignViewerPropertySection {
    let main = context.main_component.as_ref();
    let main_name: SharedString = main
        .map(|main| main.name.clone())
        .unwrap_or_else(|| "No main component".into());
    let origin: SharedString = match main.map(|main| &main.origin) {
        Some(DesignComponentOrigin::Local) => "Local".into(),
        Some(DesignComponentOrigin::Remote { library_name }) => {
            format!("Library · {library_name}").into()
        }
        None => "Not available".into(),
    };
    let availability: SharedString = match main.map(|main| &main.availability) {
        Some(DesignComponentAvailability::Available) => "Available".into(),
        Some(DesignComponentAvailability::Missing) | None => "Missing".into(),
        Some(DesignComponentAvailability::Unavailable { reason }) if reason.is_empty() => {
            "Unavailable".into()
        }
        Some(DesignComponentAvailability::Unavailable { reason }) => {
            format!("Unavailable · {reason}").into()
        }
    };
    let description = context
        .description
        .clone()
        .or_else(|| main.and_then(|main| main.description.clone()))
        .unwrap_or_else(|| "No description".into());

    let mut rows = vec![
        DesignViewerPropertyRow::new("role", "Role", context.role.label()),
        DesignViewerPropertyRow::new("main-component", "Main component", main_name),
        DesignViewerPropertyRow::new("main-component-origin", "Origin", origin),
        DesignViewerPropertyRow::new("main-component-availability", "Availability", availability),
        DesignViewerPropertyRow::new("description", "Description", description),
    ];
    let mut links = context.documentation_links.iter().collect::<Vec<_>>();
    if let Some(main) = main {
        links.extend(main.documentation_links.iter());
    }
    rows.extend(links.into_iter().enumerate().map(|(index, link)| {
        DesignViewerPropertyRow::new(
            format!("documentation-{index}"),
            link.label.clone(),
            link.url.clone(),
        )
    }));

    let copy_value = rows
        .iter()
        .map(|row| format!("{}: {}", row.label, row.displayed_value))
        .collect::<Vec<_>>()
        .join("\n");
    let (section_id, section_title) = if context.role.uses_instance_section() {
        ("instance", "Instance")
    } else {
        ("component", "Component")
    };
    DesignViewerPropertySection::new(section_id, section_title, rows)
        .with_copy_value(copy_value)
        .with_copy_all()
}

pub(super) fn story_viewer_color(
    color: DesignColor,
    representation: DesignViewerColorRepresentation,
) -> String {
    let alpha = f32::from(color.alpha) / 255.;
    let alpha_suffix = if color.alpha != u8::MAX {
        format!(" / {}%", story_viewer_number(alpha * 100.))
    } else {
        String::new()
    };
    match representation {
        DesignViewerColorRepresentation::Css | DesignViewerColorRepresentation::Rgb => {
            if color.alpha == u8::MAX {
                format!("rgb({}, {}, {})", color.red, color.green, color.blue)
            } else {
                format!(
                    "rgba({}, {}, {}, {})",
                    color.red,
                    color.green,
                    color.blue,
                    story_viewer_number(alpha)
                )
            }
        }
        DesignViewerColorRepresentation::Hex => format!("#{}", color.hex()),
        DesignViewerColorRepresentation::Hsl => {
            let (hue, saturation, lightness) = story_rgb_to_hsl(color);
            format!(
                "hsl({}, {}%, {}%){alpha_suffix}",
                story_viewer_number(hue),
                story_viewer_number(saturation * 100.),
                story_viewer_number(lightness * 100.)
            )
        }
        DesignViewerColorRepresentation::Hsb => {
            let (hue, saturation, brightness) = story_rgb_to_hsb(color);
            format!(
                "hsb({}, {}%, {}%){alpha_suffix}",
                story_viewer_number(hue),
                story_viewer_number(saturation * 100.),
                story_viewer_number(brightness * 100.)
            )
        }
    }
}

fn story_rgb_to_hsl(color: DesignColor) -> (f32, f32, f32) {
    let red = f32::from(color.red) / 255.;
    let green = f32::from(color.green) / 255.;
    let blue = f32::from(color.blue) / 255.;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let lightness = (max + min) / 2.;
    let delta = max - min;
    if delta <= f32::EPSILON {
        return (0., 0., lightness);
    }
    let saturation = delta / (1. - (2. * lightness - 1.).abs());
    (
        story_color_hue(red, green, blue, max, delta),
        saturation,
        lightness,
    )
}

fn story_rgb_to_hsb(color: DesignColor) -> (f32, f32, f32) {
    let red = f32::from(color.red) / 255.;
    let green = f32::from(color.green) / 255.;
    let blue = f32::from(color.blue) / 255.;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;
    let saturation = if max <= f32::EPSILON { 0. } else { delta / max };
    let hue = if delta <= f32::EPSILON {
        0.
    } else {
        story_color_hue(red, green, blue, max, delta)
    };
    (hue, saturation, max)
}

fn story_color_hue(red: f32, green: f32, blue: f32, max: f32, delta: f32) -> f32 {
    let sector = if (max - red).abs() <= f32::EPSILON {
        ((green - blue) / delta).rem_euclid(6.)
    } else if (max - green).abs() <= f32::EPSILON {
        (blue - red) / delta + 2.
    } else {
        (red - green) / delta + 4.
    };
    sector * 60.
}

pub(super) fn story_color_with_opacity(mut color: DesignColor, opacity: f32) -> DesignColor {
    let alpha = f32::from(color.alpha) / 255. * (opacity / 100.).clamp(0., 1.);
    color.alpha = (alpha * 255.).round().clamp(0., 255.) as u8;
    color
}

fn story_composite_color(foreground: DesignColor, background: DesignColor) -> DesignColor {
    let foreground_alpha = f32::from(foreground.alpha) / 255.;
    let background_alpha = f32::from(background.alpha) / 255.;
    let output_alpha = foreground_alpha + background_alpha * (1. - foreground_alpha);
    if output_alpha <= f32::EPSILON {
        return DesignColor::rgba(0, 0, 0, 0);
    }
    let channel = |foreground: u8, background: u8| {
        ((f32::from(foreground) * foreground_alpha
            + f32::from(background) * background_alpha * (1. - foreground_alpha))
            / output_alpha)
            .round()
            .clamp(0., 255.) as u8
    };
    DesignColor::rgba(
        channel(foreground.red, background.red),
        channel(foreground.green, background.green),
        channel(foreground.blue, background.blue),
        (output_alpha * 255.).round().clamp(0., 255.) as u8,
    )
}

fn story_relative_luminance(color: DesignColor) -> f32 {
    let linear = |channel: u8| {
        let encoded = f32::from(channel) / 255.;
        if encoded <= 0.04045 {
            encoded / 12.92
        } else {
            ((encoded + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(color.red) + 0.7152 * linear(color.green) + 0.0722 * linear(color.blue)
}

pub(super) fn story_contrast_ratio(foreground: DesignColor, background: DesignColor) -> f32 {
    let background = story_composite_color(background, DesignColor::WHITE);
    let foreground = story_composite_color(foreground, background);
    let foreground_luminance = story_relative_luminance(foreground);
    let background_luminance = story_relative_luminance(background);
    let lighter = foreground_luminance.max(background_luminance);
    let darker = foreground_luminance.min(background_luminance);
    (lighter + 0.05) / (darker + 0.05)
}

pub(super) fn story_contrast_threshold(
    category: DesignColorContrastCategory,
    level: DesignColorContrastLevel,
) -> Option<f32> {
    match (category, level) {
        (DesignColorContrastCategory::NormalText, DesignColorContrastLevel::Aa) => Some(4.5),
        (DesignColorContrastCategory::NormalText, DesignColorContrastLevel::Aaa) => Some(7.),
        (DesignColorContrastCategory::LargeText, DesignColorContrastLevel::Aa) => Some(3.),
        (DesignColorContrastCategory::LargeText, DesignColorContrastLevel::Aaa) => Some(4.5),
        (DesignColorContrastCategory::Graphics, DesignColorContrastLevel::Aa) => Some(3.),
        (DesignColorContrastCategory::Auto, _)
        | (DesignColorContrastCategory::Graphics, DesignColorContrastLevel::Aaa) => None,
    }
}

pub(super) fn apply_story_component_authoring_name_edit<K>(
    current_name: &mut SharedString,
    sessions: &mut HashMap<K, SharedString>,
    key: K,
    original_name: &SharedString,
    expected_name: &SharedString,
    candidate_name: &SharedString,
    phase: DesignPanelEditPhase,
) -> bool
where
    K: Clone + Eq + std::hash::Hash,
{
    match phase {
        DesignPanelEditPhase::Begin => {
            if current_name != expected_name
                || expected_name != original_name
                || candidate_name != original_name
                || sessions.contains_key(&key)
            {
                return false;
            }
            sessions.insert(key, original_name.clone());
            true
        }
        DesignPanelEditPhase::Preview | DesignPanelEditPhase::Commit => {
            if current_name != expected_name
                || candidate_name.trim().is_empty()
                || sessions.get(&key) != Some(original_name)
            {
                return false;
            }
            *current_name = candidate_name.clone();
            if phase == DesignPanelEditPhase::Commit {
                sessions.remove(&key);
            }
            true
        }
        DesignPanelEditPhase::Cancel => {
            if current_name != expected_name || sessions.get(&key) != Some(original_name) {
                return false;
            }
            *current_name = sessions
                .remove(&key)
                .expect("a matched component-authoring name session has a snapshot");
            true
        }
    }
}

fn story_component_authoring_orders_have_same_unique_members(
    left: &[SharedString],
    right: &[SharedString],
) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let left_ids = left
        .iter()
        .map(SharedString::as_ref)
        .collect::<HashSet<_>>();
    let right_ids = right
        .iter()
        .map(SharedString::as_ref)
        .collect::<HashSet<_>>();
    left_ids.len() == left.len() && right_ids.len() == right.len() && left_ids == right_ids
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_story_component_authoring_reorder<K>(
    current_order: &[SharedString],
    sessions: &mut HashMap<K, Vec<SharedString>>,
    key: K,
    moved_id: &SharedString,
    original_order: &[SharedString],
    expected_order: &[SharedString],
    before_id: Option<&SharedString>,
    phase: DesignPanelEditPhase,
) -> Option<Vec<SharedString>>
where
    K: Clone + Eq + std::hash::Hash,
{
    if current_order != expected_order
        || !story_component_authoring_orders_have_same_unique_members(original_order, current_order)
        || !current_order.contains(moved_id)
    {
        return None;
    }
    match phase {
        DesignPanelEditPhase::Begin => {
            if original_order != expected_order || sessions.contains_key(&key) {
                return None;
            }
            sessions.insert(key, original_order.to_vec());
            Some(current_order.to_vec())
        }
        DesignPanelEditPhase::Preview | DesignPanelEditPhase::Commit => {
            if sessions.get(&key).map(Vec::as_slice) != Some(original_order)
                || before_id == Some(moved_id)
                || before_id.is_some_and(|before_id| !current_order.contains(before_id))
            {
                return None;
            }
            let mut reordered = current_order.to_vec();
            let from = reordered
                .iter()
                .position(|candidate| candidate == moved_id)?;
            let moved = reordered.remove(from);
            let to = before_id
                .and_then(|before_id| {
                    reordered
                        .iter()
                        .position(|candidate| candidate == before_id)
                })
                .unwrap_or(reordered.len());
            reordered.insert(to, moved);
            if phase == DesignPanelEditPhase::Commit {
                sessions.remove(&key);
            }
            Some(reordered)
        }
        DesignPanelEditPhase::Cancel => {
            if sessions.get(&key).map(Vec::as_slice) != Some(original_order) {
                return None;
            }
            sessions.remove(&key)
        }
    }
}

fn first_unused_story_component_authoring_id<'a>(
    prefix: &str,
    used_ids: impl IntoIterator<Item = &'a str>,
) -> SharedString {
    let used_ids = used_ids.into_iter().collect::<HashSet<_>>();
    let mut sequence = 0;
    loop {
        let candidate = format!("{prefix}-{sequence}");
        if !used_ids.contains(candidate.as_str()) {
            return candidate.into();
        }
        sequence += 1;
    }
}

pub(super) fn next_story_component_property_id(
    node: &DesignPanelNode,
    kind: DesignComponentPropertyKind,
) -> SharedString {
    let prefix = format!("storybook-{}", kind.label().to_ascii_lowercase());
    let property_ids = node
        .component_properties
        .iter()
        .map(|property| property.id.as_ref());
    let authored_ids = node
        .component_context
        .as_ref()
        .and_then(|context| context.authoring.as_ref())
        .into_iter()
        .flat_map(|authoring| {
            authoring
                .definitions
                .iter()
                .map(|definition| definition.property_id.as_ref())
        });
    first_unused_story_component_authoring_id(&prefix, property_ids.chain(authored_ids))
}

pub(super) fn next_story_component_variant_option_id(
    node: &DesignPanelNode,
    property_id: &str,
) -> SharedString {
    let prefix = format!("{property_id}-option");
    let option_ids = node
        .component_context
        .as_ref()
        .and_then(|context| context.authoring.as_ref())
        .and_then(|authoring| authoring.definition(property_id))
        .into_iter()
        .flat_map(|definition| {
            definition
                .variant_options
                .iter()
                .map(|option| option.id.as_ref())
        });
    first_unused_story_component_authoring_id(&prefix, option_ids)
}

pub(super) fn invalidate_story_component_property_reorders(
    sessions: &mut HashMap<(SharedString, SharedString), Vec<SharedString>>,
    node_id: &SharedString,
    deleted_property_id: &SharedString,
) {
    sessions.retain(|(active_node_id, _), original_order| {
        active_node_id != node_id || !original_order.contains(deleted_property_id)
    });
}

pub(super) fn invalidate_story_component_variant_option_reorders(
    sessions: &mut HashMap<(SharedString, SharedString, SharedString), Vec<SharedString>>,
    node_id: &SharedString,
    property_id: &SharedString,
) {
    sessions.retain(|(active_node_id, active_property_id, _), _| {
        active_node_id != node_id || active_property_id != property_id
    });
}

pub(super) fn echo_story_component_property_order(
    node: &mut DesignPanelNode,
    partition: DesignComponentPropertyPartition,
    desired_order: &[SharedString],
) -> bool {
    let current = node
        .component_properties
        .iter()
        .filter(|property| {
            DesignComponentPropertyPartition::for_kind(property.definition.kind()) == partition
        })
        .map(|property| property.id.clone())
        .collect::<Vec<_>>();
    if !story_component_authoring_orders_have_same_unique_members(&current, desired_order) {
        return false;
    }
    let Some(partition_properties) = desired_order
        .iter()
        .map(|property_id| {
            node.component_properties
                .iter()
                .find(|property| &property.id == property_id)
                .filter(|property| {
                    DesignComponentPropertyPartition::for_kind(property.definition.kind())
                        == partition
                })
                .cloned()
        })
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let other_properties = node
        .component_properties
        .iter()
        .filter(|property| {
            DesignComponentPropertyPartition::for_kind(property.definition.kind()) != partition
        })
        .cloned()
        .collect::<Vec<_>>();
    node.component_properties = match partition {
        DesignComponentPropertyPartition::Variant => partition_properties
            .into_iter()
            .chain(other_properties)
            .collect(),
        DesignComponentPropertyPartition::Regular => other_properties
            .into_iter()
            .chain(partition_properties)
            .collect(),
    };
    true
}

pub(super) fn echo_story_component_variant_option_order(
    node: &mut DesignPanelNode,
    property_id: &str,
    desired_order: &[SharedString],
) -> bool {
    let Some((definition_index, definition)) = node
        .component_context
        .as_ref()
        .and_then(|context| context.authoring.as_ref())
        .and_then(|authoring| {
            authoring
                .definitions
                .iter()
                .enumerate()
                .find(|(_, definition)| definition.property_id.as_ref() == property_id)
        })
    else {
        return false;
    };
    let current = definition
        .variant_options
        .iter()
        .map(|option| option.id.clone())
        .collect::<Vec<_>>();
    if !story_component_authoring_orders_have_same_unique_members(&current, desired_order) {
        return false;
    }
    let Some(property_index) = node
        .component_properties
        .iter()
        .position(|property| property.id.as_ref() == property_id)
    else {
        return false;
    };
    let DesignComponentPropertyDefinition::Variant {
        options: backing_options,
        ..
    } = &node.component_properties[property_index].definition
    else {
        return false;
    };
    let authored_names = definition
        .variant_options
        .iter()
        .map(|option| option.name.clone())
        .collect::<Vec<_>>();
    if backing_options != &authored_names {
        return false;
    }
    let Some(ordered_options) = desired_order
        .iter()
        .map(|option_id| {
            definition
                .variant_options
                .iter()
                .find(|option| &option.id == option_id)
                .cloned()
        })
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let ordered_names = ordered_options
        .iter()
        .map(|option| option.name.clone())
        .collect::<Vec<_>>();
    let mut next_node = node.clone();
    let Some(next_definition) = next_node
        .component_context
        .as_mut()
        .and_then(|context| context.authoring.as_mut())
        .and_then(|authoring| authoring.definitions.get_mut(definition_index))
    else {
        return false;
    };
    next_definition.variant_options = ordered_options;
    let Some(next_property) = next_node.component_properties.get_mut(property_index) else {
        return false;
    };
    let DesignComponentPropertyDefinition::Variant { options, .. } = &mut next_property.definition
    else {
        return false;
    };
    *options = ordered_names;
    next_property.preferred_values = next_property.definition.option_labels();
    *node = next_node;
    true
}

#[allow(clippy::too_many_arguments)]
pub(super) fn story_component_property_create_is_current(
    node: &DesignPanelNode,
    kind: DesignComponentPropertyKind,
    name: &SharedString,
    description: &Option<SharedString>,
    documentation_links: &[DesignDocumentationLink],
    definition: &DesignComponentPropertyDefinition,
    partition: DesignComponentPropertyPartition,
    expected_order: &[SharedString],
    after_property_id: Option<&SharedString>,
) -> bool {
    let current_order = node
        .component_properties
        .iter()
        .filter(|property| {
            DesignComponentPropertyPartition::for_kind(property.definition.kind()) == partition
        })
        .map(|property| property.id.clone())
        .collect::<Vec<_>>();
    current_order == expected_order
        && after_property_id == current_order.last()
        && DesignComponentPropertyPartition::for_kind(kind) == partition
        && definition.kind() == kind
        && !name.trim().is_empty()
        && description
            .as_ref()
            .is_none_or(|description| !description.trim().is_empty())
        && (kind == DesignComponentPropertyKind::Slot || description.is_none())
        && (kind == DesignComponentPropertyKind::Slot || documentation_links.is_empty())
        && documentation_links
            .iter()
            .all(|link| !link.label.trim().is_empty() && !link.url.trim().is_empty())
        && match definition {
            DesignComponentPropertyDefinition::Slot { settings, .. } => {
                settings.has_valid_child_range()
            }
            _ => true,
        }
}

pub(super) fn story_component_references_are_current(
    catalog: &DesignComponentSwapViewData,
    references: &[DesignComponentReference],
) -> bool {
    let mut ids = HashSet::new();
    references.iter().all(|reference| {
        ids.insert(reference.id.clone())
            && catalog
                .candidates
                .iter()
                .any(|candidate| candidate.can_apply() && candidate.reference == *reference)
    })
}

pub(super) fn story_component_definition_references_are_current(
    catalog: &DesignComponentSwapViewData,
    definition: &DesignComponentPropertyDefinition,
) -> bool {
    match definition {
        DesignComponentPropertyDefinition::InstanceSwap {
            default_value,
            preferred_values,
        } => {
            default_value.as_ref().is_none_or(|reference| {
                story_component_references_are_current(catalog, std::slice::from_ref(reference))
            }) && story_component_references_are_current(catalog, preferred_values)
        }
        DesignComponentPropertyDefinition::Slot { settings, .. } => {
            story_component_references_are_current(catalog, &settings.preferred_values)
        }
        DesignComponentPropertyDefinition::Boolean { .. }
        | DesignComponentPropertyDefinition::Text { .. }
        | DesignComponentPropertyDefinition::Variant { .. } => true,
    }
}

pub(super) fn story_component_default_variable(
    variables: &DesignVariableViewData,
    kind: DesignComponentPropertyKind,
    variable_id: Option<&SharedString>,
) -> Option<Option<DesignComponentPropertyVariableBinding>> {
    let Some(variable_id) = variable_id else {
        return Some(None);
    };
    let expected_type = match kind {
        DesignComponentPropertyKind::Boolean => DesignVariableResolvedType::Boolean,
        DesignComponentPropertyKind::Text => DesignVariableResolvedType::String,
        DesignComponentPropertyKind::Variant
        | DesignComponentPropertyKind::InstanceSwap
        | DesignComponentPropertyKind::Slot => return None,
    };
    let variable = variables
        .variable(variable_id.as_ref())
        .filter(|variable| {
            variable.resolved_type == expected_type
                && variable.disabled_reason.is_none()
                && variable.import_state != DesignVariableImportState::Available
        })?;
    Some(Some(DesignComponentPropertyVariableBinding::new(
        variable.id.clone(),
        format!("{} / {}", variable.collection_name, variable.name),
        variable.resolved_value.clone()?,
    )))
}

#[allow(clippy::too_many_arguments)]
fn story_component_property_definition_edit_is_current(
    node: &DesignPanelNode,
    component_catalog: &DesignComponentSwapViewData,
    property_id: &SharedString,
    expected_description: &Option<SharedString>,
    description: &Option<SharedString>,
    expected_documentation_links: &[DesignDocumentationLink],
    documentation_links: &[DesignDocumentationLink],
    expected_definition: &DesignComponentPropertyDefinition,
    definition: &DesignComponentPropertyDefinition,
) -> bool {
    let Some(property) = node
        .component_properties
        .iter()
        .find(|property| property.id == *property_id)
    else {
        return false;
    };
    let Some(authoring_definition) = node
        .component_context
        .as_ref()
        .and_then(|context| context.authoring.as_ref())
        .and_then(|authoring| authoring.definition(property_id.as_ref()))
    else {
        return false;
    };
    if property.description != *expected_description
        || property.documentation_links != expected_documentation_links
        || property.definition != *expected_definition
        || definition.kind() != expected_definition.kind()
    {
        return false;
    }
    let metadata_changed =
        description != expected_description || documentation_links != expected_documentation_links;
    let definition_changed = definition != expected_definition;
    if !metadata_changed && !definition_changed {
        return false;
    }
    if metadata_changed
        && (!authoring_definition.capabilities.edit_metadata
            || description
                .as_ref()
                .is_some_and(|description| description.trim().is_empty())
            || documentation_links
                .iter()
                .any(|link| link.label.trim().is_empty() || link.url.trim().is_empty()))
    {
        return false;
    }
    if !definition_changed {
        return true;
    }
    match (expected_definition, definition) {
        (
            DesignComponentPropertyDefinition::Boolean { .. },
            DesignComponentPropertyDefinition::Boolean { .. },
        ) => authoring_definition.capabilities.edit_default_value,
        (
            DesignComponentPropertyDefinition::Text {
                multiline: expected_multiline,
                ..
            },
            DesignComponentPropertyDefinition::Text {
                multiline: next_multiline,
                ..
            },
        ) => {
            authoring_definition.capabilities.edit_default_value
                && expected_multiline == next_multiline
        }
        (
            DesignComponentPropertyDefinition::InstanceSwap {
                default_value: expected_default,
                preferred_values: expected_preferred,
            },
            DesignComponentPropertyDefinition::InstanceSwap {
                default_value,
                preferred_values,
            },
        ) => {
            (default_value == expected_default
                || authoring_definition.capabilities.edit_default_value)
                && (preferred_values == expected_preferred
                    || authoring_definition.capabilities.edit_preferred_values)
                && story_component_definition_references_are_current(component_catalog, definition)
        }
        (
            DesignComponentPropertyDefinition::Slot {
                default_value: expected_default,
                ..
            },
            DesignComponentPropertyDefinition::Slot {
                default_value,
                settings,
            },
        ) => {
            authoring_definition.capabilities.edit_slot_settings
                && default_value == expected_default
                && settings.has_valid_child_range()
                && story_component_definition_references_are_current(component_catalog, definition)
        }
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_story_component_property_definition_edit(
    node: &mut DesignPanelNode,
    component_catalog: &DesignComponentSwapViewData,
    property_id: &SharedString,
    expected_description: &Option<SharedString>,
    description: &Option<SharedString>,
    expected_documentation_links: &[DesignDocumentationLink],
    documentation_links: &[DesignDocumentationLink],
    expected_definition: &DesignComponentPropertyDefinition,
    definition: &DesignComponentPropertyDefinition,
) -> bool {
    if !story_component_property_definition_edit_is_current(
        node,
        component_catalog,
        property_id,
        expected_description,
        description,
        expected_documentation_links,
        documentation_links,
        expected_definition,
        definition,
    ) {
        return false;
    }
    let Some(property) = node
        .component_properties
        .iter_mut()
        .find(|property| property.id == *property_id)
    else {
        return false;
    };
    let mut replacement = property.clone();
    let follows_authored_default =
        replacement.resolved_value == expected_definition.default_value();
    let resolved_value = if follows_authored_default {
        definition.default_value()
    } else {
        replacement.resolved_value.clone()
    };
    replacement.description = description.clone();
    replacement.documentation_links = documentation_links.to_vec();
    replacement.definition = definition.clone();
    if !replacement.set_resolved_value(resolved_value) {
        return false;
    }
    replacement.refresh_slot_violations();
    *property = replacement;
    true
}

pub(super) fn story_component_property_delete_is_current(
    node: &DesignPanelNode,
    property_id: &str,
    expected_name: &str,
) -> bool {
    node.component_properties
        .iter()
        .find(|property| property.id.as_ref() == property_id)
        .is_some_and(|property| property.name.as_ref() == expected_name)
}

pub(super) fn story_component_variant_option_create_is_current(
    node: &DesignPanelNode,
    property_id: &str,
    name: &SharedString,
    expected_order: &[SharedString],
    after_option_id: Option<&SharedString>,
) -> bool {
    let Some(definition) = node
        .component_context
        .as_ref()
        .and_then(|context| context.authoring.as_ref())
        .and_then(|authoring| authoring.definition(property_id))
    else {
        return false;
    };
    let current_order = definition
        .variant_options
        .iter()
        .map(|option| option.id.clone())
        .collect::<Vec<_>>();
    current_order == expected_order
        && after_option_id == current_order.last()
        && !name.trim().is_empty()
}

pub(super) fn story_component_variant_option_delete_is_current(
    node: &DesignPanelNode,
    property_id: &str,
    option_id: &str,
    expected_name: &str,
) -> bool {
    node.component_context
        .as_ref()
        .and_then(|context| context.authoring.as_ref())
        .and_then(|authoring| authoring.definition(property_id))
        .and_then(|definition| {
            definition
                .variant_options
                .iter()
                .find(|option| option.id.as_ref() == option_id)
        })
        .is_some_and(|option| option.name.as_ref() == expected_name)
}
