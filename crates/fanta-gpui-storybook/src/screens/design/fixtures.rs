//! Seed fixtures for the Design story: the 74-node inventory behind the
//! preset matrix plus every host-side view-data seed. The inventory is the
//! single table both the preset matrix and scenario resolution iterate.

use super::*;

pub(crate) fn seed_design_nodes() -> Vec<DesignPanelNode> {
    let mut nodes = DesignPanelNodeKind::ALL
        .into_iter()
        .enumerate()
        .map(|(index, kind)| {
            let id = match kind {
                DesignPanelNodeKind::Widget => "reference-widget".to_owned(),
                // Widget is inserted immediately before Other in `ALL`;
                // retain Other's original selector identity.
                DesignPanelNodeKind::Other => "design-node-18".to_owned(),
                _ => format!("design-node-{index}"),
            };
            DesignPanelNode::new(id, format!("{} example", kind.label()), kind)
        })
        .collect::<Vec<_>>();

    // Keep the canonical matrix keyed by `ALL`, then append explicit
    // migration fixtures for compatibility-only taxonomy aliases that do
    // not already have a richer reference fixture below. Multiple
    // selection remains an inspection scenario, never a fake node.
    nodes.extend([
        DesignPanelNode::new(
            "compatibility-mask",
            "Compatibility alias · Mask",
            DesignPanelNodeKind::Mask,
        ),
        DesignPanelNode::new(
            "compatibility-table",
            "Compatibility alias · Table (FigJam)",
            DesignPanelNodeKind::Table,
        ),
        DesignPanelNode::new(
            "compatibility-pen",
            "Compatibility alias · Pen path",
            DesignPanelNodeKind::Pen,
        ),
        DesignPanelNode::new(
            "compatibility-pencil",
            "Compatibility alias · Pencil path",
            DesignPanelNodeKind::Pencil,
        ),
    ]);

    if let Some(frame) = nodes.first_mut() {
        frame.name = "Desktop frame · Freeform".into();
        frame.fills = vec![
            DesignPaint::solid(DesignColor::rgb(0x0d, 0x99, 0xff))
                .with_id("desktop-frame-style-fill"),
        ];
        frame.fill_style_binding = Some(DesignPaintStyleBinding::new(
            DesignPaintStyleSelection::page("paint-style-brand-surface"),
            "Brand / Surface",
        ));
        frame.effects.push(
            DesignEffect::drop_shadow(DesignColor::rgba(0x00, 0x00, 0x00, 0x33), 24., 0., 0., 8.)
                .with_id("desktop-frame-shadow"),
        );
        let mut stroke = DesignStroke::for_node(
            DesignPanelNodeKind::Frame,
            DesignPaint::solid(DesignColor::BLACK),
            1.,
            DesignStrokeAlign::Inside,
        );
        stroke.add_paint(DesignPaint::solid(DesignColor::PURPLE));
        stroke.weights.mode = DesignStrokeWeightMode::Custom;
        stroke.weights.top = 1.;
        stroke.weights.right = 2.;
        stroke.weights.bottom = 4.;
        stroke.weights.left = 2.;
        stroke.join = DesignStrokeJoin::Miter;
        stroke.miter_angle = 72.;
        frame.stroke = Some(stroke);
    }

    if let Some(group) = nodes
        .iter_mut()
        .find(|node| node.kind == DesignPanelNodeKind::Group)
    {
        group.id = "add-auto-layout-group".into();
        group.name = "Navigation cluster · Group".into();
    }

    if let Some(other) = nodes
        .iter_mut()
        .find(|node| node.kind == DesignPanelNodeKind::Other)
    {
        other.name = "Plugin node · Host capabilities".into();
        other.capabilities = Some(
            DesignPanelNodeCapabilities::for_node_kind(DesignPanelNodeKind::Other)
                .with_sections([
                    DesignPanelSection::Position,
                    DesignPanelSection::Layout,
                    DesignPanelSection::Layer,
                    DesignPanelSection::Stroke,
                    DesignPanelSection::LayoutGrid,
                    DesignPanelSection::Export,
                ])
                .with_fill(false)
                .with_effects(false)
                .with_constraints(true)
                .with_layout_guides(true),
        );
        other.fills.clear();
        other.effects.clear();
    }

    if let Some(widget) = nodes
        .iter_mut()
        .find(|node| node.kind == DesignPanelNodeKind::Widget)
    {
        widget.name = "Widget · Opaque object".into();
        widget.width = 320.;
        widget.height = 180.;
        widget.capabilities = Some(DesignPanelNodeCapabilities::for_node_kind(
            DesignPanelNodeKind::Widget,
        ));
    }

    if let Some(text) = nodes
        .iter_mut()
        .find(|node| node.kind == DesignPanelNodeKind::Text)
        && let Some(typography) = text.typography.as_mut()
    {
        text.name = "Variable text · Truncated".into();
        typography.weight = 520.;
        typography.style_binding = Some(DesignTypographyStyleBinding::new(
            DesignTypographyStyleSelection::page("page-body-default"),
            "Body / Default",
        ));
        typography.resize = DesignTextResize::AutoHeight;
        typography.truncate = true;
        typography.max_lines = Some(3);
        typography.list = DesignTextList::Bulleted;
        typography.list_spacing = 8.;
        typography.decoration = DesignTextDecoration::Underline;
        typography.decoration_details = Some(DesignTextDecorationDetails {
            style: DesignTextDecorationStyle::Wavy,
            offset: DesignTextDecorationMetric::Pixels(2.),
            thickness: DesignTextDecorationMetric::Percent(110.),
            color: DesignTextDecorationColor::Solid(DesignColor::BLUE),
            skip_ink: true,
        });
        typography.open_type_features = vec![
            DesignOpenTypeFeature::new(
                DesignOpenTypeFeatureTag::registered("liga")
                    .expect("liga is an exact registered tag"),
                "Standard ligatures",
                true,
                true,
            )
            .with_preview("fi fl ffi"),
            DesignOpenTypeFeature::new(
                DesignOpenTypeFeatureTag::registered("zero")
                    .expect("zero is an exact registered tag"),
                "Slashed zero",
                false,
                true,
            )
            .with_preview("0 O"),
            DesignOpenTypeFeature {
                tag: DesignOpenTypeFeatureTag::opaque("FUTURE:round-dots"),
                name: "Future round dots".into(),
                default_enabled: false,
                enabled: false,
                availability: DesignOpenTypeFeatureAvailability::Unavailable {
                    reason: "The active style does not provide this feature".into(),
                },
                preview: Some("i j :".into()),
            },
        ];
        typography.variable_axes = vec![
            DesignFontAxis::new("wght", "Weight", 520., 100., 900., 400.),
            DesignFontAxis::new("wdth", "Width", 96.5, 75., 125., 100.).with_step(0.1),
            DesignFontAxis::new("opsz", "Optical size", 18., 9., 144., 14.).with_step(0.1),
            DesignFontAxis::new("slnt", "Slant", -6., -12., 0., 0.).with_step(0.1),
            DesignFontAxis::new("GRAD", "Grade", 25., -100., 150., 0.),
            DesignFontAxis::new("ital", "Italic", 0., 0., 1., 0.)
                .read_only_with_reason("The active face controls italic as a named style"),
            DesignFontAxis::new("XTRA", "Counter width", 468., 323., 603., 468.).with_binding(
                DesignFontAxisBinding::new("variable-counter-width", "Counter width")
                    .with_collection("Type scale"),
            ),
        ];
    }
    if let Some(text_path) = nodes
        .iter_mut()
        .find(|node| node.kind == DesignPanelNodeKind::TextPath)
        && let Some(typography) = text_path.typography.as_mut()
    {
        text_path.name = "Text path · Percent units".into();
        typography.line_height = DesignLineHeight::Percent(125.);
        typography.letter_spacing = DesignLetterSpacing::Percent(2.);
        if let Some(view_data) = text_path.text_path.as_mut() {
            view_data.orientation = DesignTextPathOrientation::Default;
            view_data.can_flip_orientation = true;
            view_data.show_start_data_debug_controls = false;
        }
        text_path.text_path_start_data = DesignTextPathStartData::new(2, 0.35);
        text_path.vector_edit = Some(DesignVectorEditViewData::new([
            DesignVectorVertexViewData::new("text-path-start", 8., 28.)
                .with_topology(DesignVectorVertexTopology::Endpoint)
                .with_corner_radius(Some(0.))
                .with_handle_mirroring(Some(DesignHandleMirroring::Angle))
                .selected(true),
            DesignVectorVertexViewData::new("text-path-curve", 72., 44.)
                .with_corner_radius(Some(10.))
                .with_handle_mirroring(Some(DesignHandleMirroring::AngleAndLength))
                .selected(true),
            DesignVectorVertexViewData::new("text-path-end", 144., 20.)
                .with_topology(DesignVectorVertexTopology::Endpoint)
                .with_corner_radius(Some(0.))
                .with_handle_mirroring(Some(DesignHandleMirroring::None)),
        ]));
    }
    let mut text_path_debug = DesignPanelNode::new(
        "text-path-api-debug",
        "Text path · API start-data debug",
        DesignPanelNodeKind::TextPath,
    );
    text_path_debug.text_path = Some(
        fanta_gpui::prelude::DesignTextPathViewData::new(DesignTextPathOrientation::Flipped)
            .flippable(false)
            .with_start_data_debug_controls(true),
    );
    text_path_debug.text_path_start_data = DesignTextPathStartData::new(4, 0.625);
    nodes.push(text_path_debug);

    if let Some(vector) = nodes
        .iter_mut()
        .find(|node| node.kind == DesignPanelNodeKind::Vector)
    {
        vector.vector_edit = Some(DesignVectorEditViewData::new([
            DesignVectorVertexViewData::new("network-left", 16., 24.)
                .with_topology(DesignVectorVertexTopology::Endpoint)
                .with_corner_radius(Some(4.))
                .with_handle_mirroring(Some(DesignHandleMirroring::None))
                .selected(true),
            DesignVectorVertexViewData::new("network-curve", 64., 24.)
                .with_corner_radius(Some(12.))
                .with_handle_mirroring(Some(DesignHandleMirroring::Angle))
                .selected(true),
            DesignVectorVertexViewData::new("network-branch", 96., 72.)
                .with_topology(DesignVectorVertexTopology::Branch)
                .with_corner_radius(None)
                .with_handle_mirroring(None),
            DesignVectorVertexViewData::new("network-end", 156., 56.)
                .with_topology(DesignVectorVertexTopology::Endpoint)
                .with_corner_radius(Some(0.))
                .with_handle_mirroring(Some(DesignHandleMirroring::AngleAndLength)),
        ]));
    }

    if let Some(vector) = nodes
        .iter_mut()
        .find(|node| node.kind == DesignPanelNodeKind::Vector)
        && let Some(stroke) = vector.stroke.as_mut()
    {
        vector.name = "Vector network · Branching".into();
        stroke.edit_context = DesignStrokeEditContext::branching(4);
        stroke.endpoint_cap = DesignStrokeCap::Diamond;
        stroke.add_paint(DesignPaint::solid(DesignColor::BLUE));
    }

    if let Some(pen) = nodes
        .iter_mut()
        .find(|node| node.kind == DesignPanelNodeKind::Pen)
        && let Some(stroke) = pen.stroke.as_mut()
    {
        pen.name = "Pen path · Stretch brush".into();
        stroke.complex_stroke = DesignComplexStroke::StretchBrush(DesignStretchBrushStroke {
            brush: DesignStretchBrushName::Hardboiled,
            direction: DesignStrokeBrushDirection::Backward,
        });
    }

    if let Some(pencil) = nodes
        .iter_mut()
        .find(|node| node.kind == DesignPanelNodeKind::Pencil)
        && let Some(stroke) = pencil.stroke.as_mut()
    {
        pencil.name = "Pencil path · Dynamic".into();
        stroke.complex_stroke = DesignComplexStroke::Dynamic(DesignDynamicStroke {
            frequency: 6.4,
            wiggle: 0.38,
            smoothen: 0.72,
        });
    }

    let horizontal = DesignPanelNode::new(
        "layout-horizontal",
        "Card row · Horizontal",
        DesignPanelNodeKind::Frame,
    )
    .with_layout_mode(DesignLayoutMode::Horizontal);
    let mut vertical = DesignPanelNode::new(
        "layout-vertical",
        "Settings · Vertical",
        DesignPanelNodeKind::Frame,
    )
    .with_layout_mode(DesignLayoutMode::Vertical);
    if let Some(layout) = vertical.layout.as_mut() {
        layout.padding = [12., 24., 16., 20.];
    }
    let mut wrapped = DesignPanelNode::new(
        "layout-wrap",
        "Chips · Horizontal wrap",
        DesignPanelNodeKind::Frame,
    )
    .with_layout_mode(DesignLayoutMode::Horizontal);
    if let Some(layout) = wrapped.layout.as_mut() {
        layout.wrap = true;
        layout.gap = 6.;
        layout.counter_axis_gap = Some(6.);
        layout.padding = [8., 12., 8., 12.];
    }
    let mut wrapped_linked = DesignPanelNode::new(
        "layout-wrap-linked",
        "Tags · Wrap with linked row spacing",
        DesignPanelNodeKind::Frame,
    )
    .with_layout_mode(DesignLayoutMode::Horizontal);
    if let Some(layout) = wrapped_linked.layout.as_mut() {
        layout.set_wrap(true);
        layout.gap = 10.;
        layout.counter_axis_gap = None;
    }
    let mut wrapped_space_between = DesignPanelNode::new(
        "layout-wrap-space-between",
        "Cards · Wrap rows space between",
        DesignPanelNodeKind::Frame,
    )
    .with_layout_mode(DesignLayoutMode::Horizontal);
    if let Some(layout) = wrapped_space_between.layout.as_mut() {
        layout.set_wrap(true);
        let _ = layout.set_counter_axis_align_content(DesignCounterAxisAlignContent::SpaceBetween);
    }
    let mut grid =
        DesignPanelNode::new("layout-grid", "Gallery · Grid", DesignPanelNodeKind::Frame)
            .with_layout_mode(DesignLayoutMode::Grid);
    if let Some(layout) = grid.layout.as_mut() {
        layout.horizontal_sizing = DesignSizingMode::Fixed;
        layout.grid_auto_tracks = DesignGridAutoTracks::None;
        layout.grid_items_positioning = DesignGridItemsPositioning::Manual;
        layout.gap = 16.;
        layout.counter_axis_gap = Some(24.);
        layout.grid_columns = vec![
            DesignGridTrack::fraction(1.),
            DesignGridTrack::fraction(1.),
            DesignGridTrack::fixed(160.),
        ];
        layout.grid_rows = vec![DesignGridTrack::hug(), DesignGridTrack::fixed(120.)];
    }
    let mut auto_spacing = DesignPanelNode::new(
        "layout-auto-spacing",
        "Toolbar · Auto spacing and first-on-top",
        DesignPanelNodeKind::Frame,
    )
    .with_layout_mode(DesignLayoutMode::Horizontal);
    if let Some(layout) = auto_spacing.layout.as_mut() {
        layout.horizontal_sizing = DesignSizingMode::Fixed;
        layout.vertical_sizing = DesignSizingMode::Hug;
        layout.item_spacing_mode = DesignItemSpacingMode::Auto;
        layout.gap = 0.;
        layout.padding = [12., 20., 12., 20.];
        layout.clip_content = false;
        layout.include_strokes = true;
        layout.stacking_order = DesignStackingOrder::FirstOnTop;
        layout.baseline_alignment = DesignBaselineAlignment::Baseline;
    }
    let mut responsive_limits = DesignPanelNode::new(
        "layout-responsive-limits",
        "Card · Auto layout min/max disclosures",
        DesignPanelNodeKind::Frame,
    )
    .with_layout_mode(DesignLayoutMode::Vertical);
    if let Some(layout) = responsive_limits.layout.as_mut() {
        layout.horizontal_sizing = DesignSizingMode::Hug;
        layout.vertical_sizing = DesignSizingMode::Hug;
        layout.gap = 12.;
        layout.padding = [20., 24., 20., 24.];
        layout.item.min_width = Some(240.);
        layout.item.max_width = Some(960.);
        layout.item.min_height = Some(120.);
        layout.item.max_height = Some(720.);
    }
    let mut vertical_auto_spacing = DesignPanelNode::new(
        "layout-vertical-auto-spacing",
        "List · Vertical auto spacing",
        DesignPanelNodeKind::Frame,
    )
    .with_layout_mode(DesignLayoutMode::Vertical);
    if let Some(layout) = vertical_auto_spacing.layout.as_mut() {
        layout.horizontal_sizing = DesignSizingMode::Hug;
        layout.vertical_sizing = DesignSizingMode::Fixed;
        layout.alignment_x = 1;
        layout.item_spacing_mode = DesignItemSpacingMode::Auto;
        layout.gap = 0.;
        layout.padding = [8., 16., 24., 32.];
    }
    let mut grid_auto_flow = DesignPanelNode::new(
        "layout-grid-auto-flow",
        "Gallery · Grid auto flow",
        DesignPanelNodeKind::Frame,
    )
    .with_layout_mode(DesignLayoutMode::Grid);
    if let Some(layout) = grid_auto_flow.layout.as_mut() {
        layout.horizontal_sizing = DesignSizingMode::Fixed;
        layout.vertical_sizing = DesignSizingMode::Fixed;
        layout.grid_columns = vec![
            DesignGridTrack::fraction(2.),
            DesignGridTrack::hug(),
            DesignGridTrack::fixed(96.),
        ];
        layout.grid_rows = vec![DesignGridTrack::hug(), DesignGridTrack::fraction(1.)];
        layout.grid_auto_tracks = DesignGridAutoTracks::Rows;
        layout.grid_items_positioning = DesignGridItemsPositioning::RowAutoFlow;
        layout.gap = 12.;
        layout.counter_axis_gap = Some(20.);
    }
    let mut horizontal_item = DesignPanelNode::new(
        "layout-child-horizontal",
        "Card · Horizontal auto-layout child",
        DesignPanelNodeKind::Rectangle,
    );
    if let Some(layout) = horizontal_item.layout.as_mut() {
        layout.horizontal_sizing = DesignSizingMode::Fill;
        layout.vertical_sizing = DesignSizingMode::Fixed;
        layout.item.positioning = DesignLayoutPositioning::InFlow;
        layout.item.layout_grow = 1.;
        layout.item.min_width = Some(160.);
        layout.item.max_width = Some(480.);
        layout.item.max_height = Some(96.);
    }
    let mut absolute_item = DesignPanelNode::new(
        "layout-child-absolute",
        "Card · Absolute stretched child",
        DesignPanelNodeKind::Rectangle,
    );
    if let Some(layout) = absolute_item.layout.as_mut() {
        layout.horizontal_sizing = DesignSizingMode::Fill;
        layout.vertical_sizing = DesignSizingMode::Hug;
        layout.item.positioning = DesignLayoutPositioning::Absolute;
        layout.item.align_self = DesignLayoutAlignSelf::Stretch;
        layout.item.layout_grow = 1.;
        layout.item.min_width = Some(120.);
        layout.item.max_width = Some(640.);
        layout.item.min_height = Some(44.);
        layout.item.max_height = Some(240.);
    }
    let mut vertical_item = DesignPanelNode::new(
        "layout-child-vertical",
        "Card · Vertical auto-layout child",
        DesignPanelNodeKind::Rectangle,
    );
    if let Some(layout) = vertical_item.layout.as_mut() {
        layout.horizontal_sizing = DesignSizingMode::Fill;
        layout.vertical_sizing = DesignSizingMode::Fixed;
        layout.item.positioning = DesignLayoutPositioning::InFlow;
        layout.item.align_self = DesignLayoutAlignSelf::Stretch;
    }
    let mut grid_item = DesignPanelNode::new(
        "layout-grid-child-placement",
        "Card · Grid child placement",
        DesignPanelNodeKind::Rectangle,
    );
    if let Some(layout) = grid_item.layout.as_mut() {
        layout.horizontal_sizing = DesignSizingMode::Fill;
        layout.vertical_sizing = DesignSizingMode::Fill;
        layout.item.grid_row_index = 2;
        layout.item.grid_column_index = 1;
        layout.item.grid_row_span = 2;
        layout.item.grid_column_span = 3;
        layout.item.grid_horizontal_alignment = DesignGridItemAlignment::End;
        layout.item.grid_vertical_alignment = DesignGridItemAlignment::Center;
        layout.item.min_width = Some(96.);
        layout.item.max_height = Some(320.);
    }
    nodes.splice(
        1..1,
        [
            horizontal,
            vertical,
            wrapped,
            wrapped_linked,
            wrapped_space_between,
            grid,
            auto_spacing,
            responsive_limits,
            vertical_auto_spacing,
            grid_auto_flow,
            horizontal_item,
            absolute_item,
            vertical_item,
            grid_item,
        ],
    );

    let gradient_kinds = [
        DesignPaintKind::LinearGradient,
        DesignPaintKind::RadialGradient,
        DesignPaintKind::AngularGradient,
        DesignPaintKind::DiamondGradient,
    ];
    for (index, kind) in gradient_kinds.into_iter().enumerate() {
        let mut gradient = DesignPanelNode::new(
            format!("gradient-{index}"),
            format!("{} gradient", kind.label()),
            DesignPanelNodeKind::Rectangle,
        );
        gradient.fills = vec![DesignPaint::gradient(
            kind,
            vec![
                DesignGradientStop::new(0., DesignColor::PURPLE),
                DesignGradientStop::new(0.5, DesignColor::BLUE),
                DesignGradientStop::new(1., DesignColor::rgb(0x14, 0xae, 0x5c)),
            ],
        )];
        nodes.push(gradient);
    }

    if let Some(slice) = nodes
        .iter_mut()
        .find(|node| node.kind == DesignPanelNodeKind::Slice)
    {
        slice.export_settings = vec![
            DesignExportSetting {
                scale: 1.,
                suffix: "".into(),
                format: DesignExportFormat::Png,
            },
            DesignExportSetting {
                scale: 1.,
                suffix: "-vector".into(),
                format: DesignExportFormat::Svg,
            },
        ];
    }
    if let Some(component) = nodes
        .iter_mut()
        .find(|node| node.kind == DesignPanelNodeKind::Component)
    {
        component.component_properties.insert(
            0,
            DesignComponentProperty::variant(
                "state",
                "State",
                "Default",
                "Default",
                vec!["Default".into(), "Hover".into(), "Pressed".into()],
            ),
        );
        component.layout = Some({
            let mut layout = component.layout.clone().unwrap_or_default();
            layout.mode = DesignLayoutMode::Horizontal;
            layout.gap = 8.;
            layout.padding = [10., 16., 10., 16.];
            layout
        });
        if let Some(property) = component
            .component_properties
            .iter_mut()
            .find(|property| property.id == "show-icon")
        {
            property.default_value_binding = Some(DesignComponentPropertyVariableBinding::new(
                "layer-visible",
                "Layer visible",
                DesignVariableResolvedValue::Boolean(true),
            ));
        }
        if let Some(property) = component
            .component_properties
            .iter_mut()
            .find(|property| property.id == "label")
        {
            property.description = Some("Action copy shown inside the button.".into());
            property.documentation_links = vec![DesignDocumentationLink::new(
                "Button content guidelines",
                "https://example.com/components/button/content",
            )];
        }

        let choices = component
            .component_properties
            .iter()
            .map(|property| {
                DesignComponentPropertyChoice::new(
                    property.id.clone(),
                    property.name.clone(),
                    property.definition.kind(),
                )
            })
            .collect::<Vec<_>>();
        let choice = |property_id: &str| {
            choices
                .iter()
                .find(|choice| choice.property_id.as_ref() == property_id)
                .expect("storybook component property choice")
                .clone()
        };
        let definitions = component
            .component_properties
            .iter()
            .map(|property| {
                let mut definition =
                    DesignComponentPropertyDefinitionAuthoring::editable(property.id.clone());
                if property.definition.kind() == DesignComponentPropertyKind::Variant {
                    definition = definition.with_variant_options(
                        property
                            .preferred_values
                            .iter()
                            .enumerate()
                            .map(|(index, name)| {
                                DesignComponentVariantOptionAuthoring::editable(
                                    format!("{}-option-{index}", property.id),
                                    name.clone(),
                                )
                            }),
                    );
                }
                definition
            })
            .collect::<Vec<_>>();
        let applied_properties = vec![
            DesignAppliedComponentPropertyControl::new(
                "appearance-visible-property",
                "button-icon-layer",
                "Leading icon",
                DesignComponentPropertyApplicationSurface::Appearance,
                [choice("show-icon"), choice("state")],
            )
            .applied_to("show-icon"),
            DesignAppliedComponentPropertyControl::new(
                "text-content-property",
                "button-label-layer",
                "Label",
                DesignComponentPropertyApplicationSurface::Text,
                [choice("label"), choice("state")],
            )
            .applied_to("label"),
            DesignAppliedComponentPropertyControl::new(
                "nested-swap-property",
                "button-icon-instance",
                "Icon instance",
                DesignComponentPropertyApplicationSurface::NestedInstance,
                [choice("leading-icon"), choice("content")],
            )
            .applied_to("leading-icon"),
            DesignAppliedComponentPropertyControl::new(
                "nested-variant-property",
                "button-state-layer",
                "State layer",
                DesignComponentPropertyApplicationSurface::NestedInstance,
                [choice("state"), choice("show-icon")],
            )
            .applied_to("state"),
            DesignAppliedComponentPropertyControl::new(
                "nested-slot-property",
                "button-content-layer",
                "Content layer",
                DesignComponentPropertyApplicationSurface::NestedInstance,
                [choice("content"), choice("leading-icon")],
            )
            .applied_to("content"),
        ];
        let exposure_candidates = [
            (
                "nested-expose-state",
                "nested-button",
                "Nested button",
                "state",
            ),
            (
                "nested-expose-label",
                "nested-button",
                "Nested button",
                "label",
            ),
            (
                "nested-expose-visible",
                "nested-icon",
                "Nested icon",
                "show-icon",
            ),
            (
                "nested-expose-swap",
                "nested-icon",
                "Nested icon",
                "leading-icon",
            ),
            (
                "nested-expose-slot",
                "nested-content",
                "Nested content",
                "content",
            ),
        ]
        .into_iter()
        .map(|(candidate_id, instance_id, instance_name, property_id)| {
            DesignNestedComponentPropertyExposureCandidate::new(
                candidate_id,
                instance_id,
                instance_name,
                choice(property_id),
            )
        })
        .collect::<Vec<_>>();
        if let Some(context) = component.component_context.as_mut() {
            context.description =
                Some("Primary action component with label, icon, and content-slot APIs.".into());
            context.documentation_links = vec![
                DesignDocumentationLink::new(
                    "Button usage",
                    "https://example.com/components/button",
                ),
                DesignDocumentationLink::new(
                    "Button accessibility",
                    "https://example.com/components/button/accessibility",
                ),
            ];
            context.authoring = Some(DesignComponentAuthoringViewData {
                create_kinds: vec![
                    DesignComponentPropertyKind::Variant,
                    DesignComponentPropertyKind::Boolean,
                    DesignComponentPropertyKind::Text,
                    DesignComponentPropertyKind::InstanceSwap,
                    DesignComponentPropertyKind::Slot,
                ],
                definitions,
                applied_properties,
                exposure_candidates,
            });
        }
    }

    if let Some(component_set) = nodes
        .iter_mut()
        .find(|node| node.kind == DesignPanelNodeKind::ComponentSet)
    {
        let definitions = component_set
            .component_properties
            .iter()
            .map(|property| {
                DesignComponentPropertyDefinitionAuthoring::editable(property.id.clone())
                    .with_variant_options(property.preferred_values.iter().enumerate().map(
                        |(index, name)| {
                            DesignComponentVariantOptionAuthoring::editable(
                                format!("{}-option-{index}", property.id),
                                name.clone(),
                            )
                        },
                    ))
            })
            .collect();
        if let Some(context) = component_set.component_context.as_mut() {
            context.description =
                Some("Button family with ordered State and Size variant axes.".into());
            context.documentation_links = vec![
                DesignDocumentationLink::new(
                    "Button variants",
                    "https://example.com/components/button/variants",
                ),
                DesignDocumentationLink::new(
                    "Variant naming",
                    "https://example.com/components/button/variant-naming",
                ),
            ];
            context.authoring = Some(DesignComponentAuthoringViewData {
                create_kinds: vec![
                    DesignComponentPropertyKind::Variant,
                    DesignComponentPropertyKind::Boolean,
                    DesignComponentPropertyKind::Text,
                    DesignComponentPropertyKind::InstanceSwap,
                    DesignComponentPropertyKind::Slot,
                ],
                definitions,
                applied_properties: Vec::new(),
                exposure_candidates: Vec::new(),
            });
        }
    }

    if let Some(component_source) = nodes
        .iter()
        .find(|node| node.kind == DesignPanelNodeKind::Component)
        .cloned()
    {
        let mut selected_text = DesignPanelNode::new(
            "component-authoring-selected-text",
            "Component authoring · Selected text layer",
            DesignPanelNodeKind::Text,
        );
        selected_text.component_properties = component_source.component_properties;
        selected_text.component_context = component_source.component_context;
        if let Some(context) = selected_text.component_context.as_mut() {
            context.description =
                Some("Selected text sublayer inside the authored main component.".into());
            if let Some(authoring) = context.authoring.as_mut() {
                authoring.applied_properties.retain(|control| {
                    control.surface == DesignComponentPropertyApplicationSurface::Text
                });
                authoring.exposure_candidates.clear();
            }
        }
        nodes.push(selected_text);
    }

    if let Some(instance) = nodes
        .iter_mut()
        .find(|node| node.kind == DesignPanelNodeKind::Instance)
    {
        if let Some(context) = instance.component_context.as_mut() {
            context.description =
                Some("Button instance with local property and nested override examples.".into());
            context.documentation_links = vec![
                DesignDocumentationLink::new(
                    "Button instance guidance",
                    "https://example.com/components/button/instances",
                ),
                DesignDocumentationLink::new(
                    "Override behavior",
                    "https://example.com/components/button/overrides",
                ),
            ];
            if let Some(main) = context.main_component.as_mut() {
                main.description =
                    Some("Published primary action from Product foundations.".into());
                main.documentation_links = vec![DesignDocumentationLink::new(
                    "Primary button API",
                    "https://example.com/libraries/product-foundations/primary-button",
                )];
            }
        }
        if let Some(property) = instance
            .component_properties
            .iter_mut()
            .find(|property| property.id == "label")
        {
            *property = property.clone().with_multiline(true);
        }
        if let Some(property) = instance
            .component_properties
            .iter_mut()
            .find(|property| property.id == "show-icon")
        {
            property.resolved_value_binding = Some(DesignComponentPropertyVariableBinding::new(
                "layer-visible",
                "Layer visible",
                DesignVariableResolvedValue::Boolean(true),
            ));
        }

        let badge_main =
            DesignComponentReference::local("nested-status-badge-main", "Status badge");
        instance.component_properties.extend([
            DesignComponentProperty::text(
                "nested-badge-label",
                "Badge label",
                "New",
                "Ready\nfor review",
            )
            .with_multiline(true)
            .with_reset_state(DesignComponentResetState::Resettable)
            .from_nested_instance(
                "nested-status-badge",
                "Status badge",
                Some(badge_main.clone()),
            ),
            DesignComponentProperty::boolean("nested-badge-visible", "Show badge", true, true)
                .from_nested_instance("nested-status-badge", "Status badge", Some(badge_main)),
        ]);
    }

    if let Some(slot) = nodes
        .iter_mut()
        .find(|node| node.kind == DesignPanelNodeKind::Slot)
        && let Some(context) = slot.component_context.as_mut()
    {
        context.description =
            Some("Slot definition for optional card content with preferred insertions.".into());
        context.documentation_links = vec![
            DesignDocumentationLink::new(
                "Slot authoring",
                "https://example.com/components/slots/authoring",
            ),
            DesignDocumentationLink::new(
                "Preferred slot content",
                "https://example.com/components/slots/preferred-content",
            ),
        ];
    }

    let mut instance_child = DesignPanelNode::new(
        "component-instance-child",
        "Nested layer · Aspect ratio inherited from main",
        DesignPanelNodeKind::Rectangle,
    );
    instance_child.is_component_instance_child = true;
    instance_child.width = 240.;
    instance_child.height = 135.;
    nodes.push(instance_child);

    let mut reference_rectangle = DesignPanelNode::new(
        "reference-rectangle",
        "Rectangle 2",
        DesignPanelNodeKind::Rectangle,
    );
    reference_rectangle.x = 0.;
    reference_rectangle.y = 437.;
    reference_rectangle.width = 393.;
    reference_rectangle.height = 215.;
    reference_rectangle.corner_radii = [0.; 4];
    reference_rectangle.fills = vec![
        DesignPaint::solid(DesignColor::rgb(0xd9, 0xd9, 0xd9)).with_id("reference-rectangle-fill"),
    ];
    nodes.push(reference_rectangle);

    let mut reference_frame =
        DesignPanelNode::new("reference-frame", "Frame 2", DesignPanelNodeKind::Frame);
    reference_frame.x = 1_709.;
    reference_frame.y = 424.;
    reference_frame.width = 548.;
    reference_frame.height = 250.;
    reference_frame.corner_radii = [0.; 4];
    reference_frame.fills.clear();
    reference_frame.stroke = None;
    reference_frame.effects.clear();
    reference_frame.layout_grids.clear();
    if let Some(layout) = reference_frame.layout.as_mut() {
        layout.gap = 0.;
        layout.counter_axis_gap = None;
        layout.clip_content = false;
    }
    nodes.push(reference_frame);

    let mut reference_text =
        DesignPanelNode::new("reference-text", "hello", DesignPanelNodeKind::Text);
    reference_text.x = 1_256.;
    reference_text.y = 1_415.;
    reference_text.width = 27.;
    reference_text.height = 15.;
    reference_text.fills =
        vec![DesignPaint::solid(DesignColor::WHITE).with_id("reference-text-fill")];
    if let Some(typography) = reference_text.typography.as_mut() {
        typography.family = "Inter".into();
        typography.style = "Regular".into();
        typography.weight = 400.;
        typography.size = 12.;
        typography.resize = DesignTextResize::AutoWidth;
        typography.letter_spacing = DesignLetterSpacing::Percent(0.);
    }
    nodes.push(reference_text);

    let mut reference_ellipse = DesignPanelNode::new(
        "reference-ellipse",
        "Ellipse 1",
        DesignPanelNodeKind::Ellipse,
    );
    reference_ellipse.x = 4_880.;
    reference_ellipse.y = -337.;
    reference_ellipse.width = 100.;
    reference_ellipse.height = 100.;
    reference_ellipse.corner_radii = [0.; 4];
    reference_ellipse.fills = vec![
        DesignPaint::solid(DesignColor::rgb(0xd9, 0xd9, 0xd9)).with_id("reference-ellipse-fill"),
    ];
    reference_ellipse.shape_geometry = DesignShapeGeometry::Ellipse(DesignArcData::FULL_CIRCLE);
    nodes.push(reference_ellipse);

    let mut reference_image = DesignPanelNode::new(
        "reference-image",
        "Screenshot 2026-07-25",
        DesignPanelNodeKind::Image,
    );
    reference_image.x = 1_337.;
    reference_image.y = 817.;
    reference_image.width = 207.;
    reference_image.height = 335.;
    reference_image.corner_radii = [0.; 4];
    nodes.push(reference_image);

    let mut reference_arrow =
        DesignPanelNode::new("reference-arrow", "Arrow 6", DesignPanelNodeKind::Arrow);
    reference_arrow.x = 1_481.;
    reference_arrow.y = 897.;
    reference_arrow.width = 205.43;
    reference_arrow.height = 0.;
    reference_arrow.rotation = 14.37;
    if let Some(stroke) = reference_arrow.stroke.as_mut() {
        stroke.paints =
            vec![DesignPaint::solid(DesignColor::WHITE).with_id("reference-arrow-stroke")];
        stroke.align = DesignStrokeAlign::Inside;
        stroke.weights.set_uniform(2.);
        stroke.start_cap = DesignStrokeCap::None;
        stroke.end_cap = DesignStrokeCap::LineArrow;
    }
    nodes.push(reference_arrow);

    nodes.push(DesignPanelNode::new(
        "reference-video",
        "Product preview",
        DesignPanelNodeKind::Video,
    ));

    for alignment_y in 0_u8..3 {
        for alignment_x in 0_u8..3 {
            let mut alignment = DesignPanelNode::new(
                format!("layout-alignment-{alignment_x}-{alignment_y}"),
                format!(
                    "Auto layout · Alignment {}",
                    usize::from(alignment_y) * 3 + usize::from(alignment_x) + 1
                ),
                DesignPanelNodeKind::Frame,
            )
            .with_layout_mode(DesignLayoutMode::Horizontal);
            if let Some(layout) = alignment.layout.as_mut() {
                layout.alignment_x = alignment_x;
                layout.alignment_y = alignment_y;
                layout.gap = 12.;
                layout.padding = [16.; 4];
            }
            nodes.push(alignment);
        }
    }

    let mut partial_arc = DesignPanelNode::new(
        "ellipse-partial-arc",
        "Ellipse · Partial arc",
        DesignPanelNodeKind::Ellipse,
    );
    partial_arc.shape_geometry = DesignShapeGeometry::Ellipse(DesignArcData::new(
        30_f32.to_radians(),
        280_f32.to_radians(),
        0.,
    ));
    nodes.push(partial_arc);

    let mut ring = DesignPanelNode::new(
        "ellipse-ring",
        "Ellipse · Ring",
        DesignPanelNodeKind::Ellipse,
    );
    ring.shape_geometry =
        DesignShapeGeometry::Ellipse(DesignArcData::new(0., std::f32::consts::TAU, 0.62));
    nodes.push(ring);

    for operation in DesignBooleanOperation::ALL {
        if operation == DesignBooleanOperation::Union {
            continue;
        }
        let mut boolean = DesignPanelNode::new(
            format!("boolean-{:?}", operation).to_ascii_lowercase(),
            format!("Boolean · {}", operation.label()),
            DesignPanelNodeKind::BooleanOperation,
        );
        boolean.shape_geometry = DesignShapeGeometry::Boolean(operation);
        nodes.push(boolean);
    }

    for mask_mode in DesignMaskType::ALL {
        let mut mask = DesignPanelNode::new(
            format!("mask-{:?}", mask_mode).to_ascii_lowercase(),
            format!("Vector · {} mask", mask_mode.label()),
            DesignPanelNodeKind::Vector,
        );
        mask.is_mask = true;
        mask.mask_mode = mask_mode;
        mask.mask_type = Some(mask_mode.label().into());
        nodes.push(mask);
    }

    if let Some(section) = nodes
        .iter_mut()
        .find(|node| node.kind == DesignPanelNodeKind::Section)
        .and_then(|node| node.section.as_mut())
    {
        section.dev_status = Some(
            DesignSectionDevStatus::new(DesignSectionDevStatusKind::ReadyForDev)
                .with_description("Implementation notes are available in Dev Mode.")
                .changed(true),
        );
    }
    let mut hidden_section = DesignPanelNode::new(
        "section-hidden",
        "Section · Hidden contents",
        DesignPanelNodeKind::Section,
    );
    if let Some(section) = hidden_section.section.as_mut() {
        section.contents_hidden = true;
    }
    nodes.push(hidden_section);
    let mut completed_section = DesignPanelNode::new(
        "section-completed",
        "Section · Completed",
        DesignPanelNodeKind::Section,
    );
    if let Some(section) = completed_section.section.as_mut() {
        section.dev_status = Some(
            DesignSectionDevStatus::new(DesignSectionDevStatusKind::Completed)
                .with_description("Reviewed and implemented."),
        );
    }
    nodes.push(completed_section);

    let mut repeat_vertical = DesignPanelNode::new(
        "transform-repeat-vertical",
        "Transform group · Vertical pixels",
        DesignPanelNodeKind::TransformGroup,
    );
    let mut vertical_modifier =
        DesignRepeatModifier::linear("repeat-vertical", DesignRepeatAxis::Vertical);
    vertical_modifier.unit = DesignTransformUnit::Pixels;
    vertical_modifier.count = 6;
    vertical_modifier.offset = 24.;
    repeat_vertical.transform_modifiers = vec![vertical_modifier];
    nodes.push(repeat_vertical);

    let mut repeat_radial = DesignPanelNode::new(
        "transform-repeat-radial",
        "Transform group · Radial",
        DesignPanelNodeKind::TransformGroup,
    );
    let mut radial_modifier = DesignRepeatModifier::radial("repeat-radial");
    radial_modifier.unit = DesignTransformUnit::Relative;
    radial_modifier.count = 12;
    radial_modifier.offset = 18.;
    repeat_radial.transform_modifiers = vec![radial_modifier];
    nodes.push(repeat_radial);

    let mut repeat_stack = DesignPanelNode::new(
        "transform-repeat-stack",
        "Transform group · Modifier stack",
        DesignPanelNodeKind::TransformGroup,
    );
    let mut stack_radial = DesignRepeatModifier::radial("repeat-stack-radial");
    stack_radial.count = 8;
    repeat_stack.transform_modifiers = vec![
        DesignRepeatModifier::linear("repeat-stack-horizontal", DesignRepeatAxis::Horizontal),
        DesignRepeatModifier::linear("repeat-stack-vertical", DesignRepeatAxis::Vertical),
        stack_radial,
    ];
    nodes.push(repeat_stack);

    let mut repeat_empty = DesignPanelNode::new(
        "transform-repeat-empty",
        "Transform group · No modifiers",
        DesignPanelNodeKind::TransformGroup,
    );
    repeat_empty.transform_modifiers.clear();
    nodes.push(repeat_empty);

    let mut repeat_horizontal_pixels = DesignPanelNode::new(
        "transform-repeat-horizontal-pixels",
        "Transform group · Horizontal pixels",
        DesignPanelNodeKind::TransformGroup,
    );
    let mut horizontal_pixels =
        DesignRepeatModifier::linear("repeat-horizontal-pixels", DesignRepeatAxis::Horizontal);
    horizontal_pixels.unit = DesignTransformUnit::Pixels;
    horizontal_pixels.count = 9;
    horizontal_pixels.offset = 36.;
    repeat_horizontal_pixels.transform_modifiers = vec![horizontal_pixels];
    nodes.push(repeat_horizontal_pixels);

    let mut paint_plain = DesignPanelNode::new(
        "paint-solid",
        "Paint · Solid",
        DesignPanelNodeKind::Rectangle,
    );
    paint_plain.fills =
        vec![DesignPaint::solid(DesignColor::rgb(0x7c, 0x3a, 0xed)).with_id("plain-solid-fill")];
    nodes.push(paint_plain);

    let mut contrast_checker = DesignPanelNode::new(
        "paint-contrast",
        "Paint · Contrast checker",
        DesignPanelNodeKind::Rectangle,
    );
    contrast_checker.fills = vec![
        DesignPaint::solid(DesignColor::rgb(0xa0, 0xae, 0xc0)).with_id("contrast-checker-fill"),
    ];
    nodes.push(contrast_checker);

    let mut paint_solid = DesignPanelNode::new(
        "paint-solid-bound",
        "Paint · Bound and read-only solids",
        DesignPanelNodeKind::Rectangle,
    );
    let mut bound_solid =
        DesignPaint::solid(DesignColor::rgb(0x0d, 0x99, 0xff)).with_id("bound-solid");
    if let DesignPaintPayload::Solid(solid) = &mut bound_solid.payload {
        solid.binding = Some(
            DesignPaintBinding::new("brand-primary", "Brand / Primary").with_collection("Theme"),
        );
    }
    bound_solid.sync_legacy_projection();
    paint_solid.fills = vec![
        bound_solid,
        DesignPaint::solid(DesignColor::rgb(0xff, 0xc7, 0x00))
            .with_id("readonly-solid")
            .with_read_only(true),
    ];
    nodes.push(paint_solid);

    for (index, tile_type) in DesignPatternTileType::ALL.into_iter().enumerate() {
        let mut paint_pattern = DesignPanelNode::new(
            format!("paint-pattern-{index}"),
            format!("Paint · Pattern {}", tile_type.label()),
            DesignPanelNodeKind::Rectangle,
        );
        let mut paint = DesignPaint::pattern(format!("pattern-source-node-{}", index + 1))
            .with_id(format!("pattern-{index}-fill"));
        if let DesignPaintPayload::Pattern(pattern) = &mut paint.payload {
            pattern.tile_type = tile_type;
            pattern.scaling_factor = 0.5 + index as f32 * 0.25;
            pattern.spacing =
                DesignPatternSpacing::new(0.08 + index as f32 * 0.04, 0.12 + index as f32 * 0.04);
            pattern.horizontal_alignment = [
                DesignPatternHorizontalAlignment::Start,
                DesignPatternHorizontalAlignment::Center,
                DesignPatternHorizontalAlignment::End,
            ][index];
        }
        paint.sync_legacy_projection();
        paint_pattern.fills = vec![paint];
        nodes.push(paint_pattern);
    }

    for (index, scale_mode) in DesignMediaPaintScaleMode::ALL.into_iter().enumerate() {
        let source_access = [
            "All source actions",
            "Upload + Edit",
            "Properties only",
            "Make only",
        ][index];
        let mut paint_image = DesignPanelNode::new(
            format!("paint-image-{index}"),
            format!("Paint · Image {} · {source_access}", scale_mode.label()),
            DesignPanelNodeKind::Rectangle,
        );
        let source = DesignPaintSource {
            id: format!("image-source-{index}").into(),
            name: format!("Hero photograph {}", index + 1).into(),
            mime_type: Some("image/jpeg".into()),
            reference: Some(format!("asset://image/{index}").into()),
        };
        let mut paint = DesignPaint::image(source).with_id(format!("image-{index}-fill"));
        if let DesignPaintPayload::Image(image) = &mut paint.payload {
            let rotation = DesignMediaQuarterTurn::ALL[index];
            image.placement = match scale_mode {
                DesignMediaPaintScaleMode::Fill => DesignMediaPaintPlacement::Fill { rotation },
                DesignMediaPaintScaleMode::Fit => DesignMediaPaintPlacement::Fit { rotation },
                DesignMediaPaintScaleMode::Crop => DesignMediaPaintPlacement::Crop {
                    transform: DesignPaintTransform::IDENTITY
                        .rotated(17.)
                        .flipped_vertical(),
                },
                DesignMediaPaintScaleMode::Tile => DesignMediaPaintPlacement::Tile {
                    scaling_factor: 0.75 + index as f32 * 0.25,
                    rotation,
                },
            };
            image.filters = DesignImageFilters {
                exposure: -0.12 + index as f32 * 0.08,
                contrast: 0.18,
                saturation: 0.24 - index as f32 * 0.04,
                temperature: 0.08,
                tint: -0.06,
                highlights: -0.20,
                shadows: 0.16,
            };
        }
        paint.sync_legacy_projection();
        paint_image.fills = vec![paint];
        nodes.push(paint_image);
    }

    for (index, scale_mode) in DesignMediaPaintScaleMode::ALL.into_iter().enumerate() {
        let mut paint_video = DesignPanelNode::new(
            format!("paint-video-{index}"),
            format!("Paint · Video {}", scale_mode.label()),
            DesignPanelNodeKind::Rectangle,
        );
        let source = DesignPaintSource {
            id: format!("video-source-{index}").into(),
            name: format!("Product demo {}", index + 1).into(),
            mime_type: Some("video/mp4".into()),
            reference: Some(format!("asset://video/{index}").into()),
        };
        let mut paint = DesignPaint::video(source).with_id(format!("video-{index}-fill"));
        if let DesignPaintPayload::Video(video) = &mut paint.payload {
            let rotation = DesignMediaQuarterTurn::ALL[index];
            video.placement = match scale_mode {
                DesignMediaPaintScaleMode::Fill => DesignMediaPaintPlacement::Fill { rotation },
                DesignMediaPaintScaleMode::Fit => DesignMediaPaintPlacement::Fit { rotation },
                DesignMediaPaintScaleMode::Crop => DesignMediaPaintPlacement::Crop {
                    transform: DesignPaintTransform::IDENTITY.rotated(11.),
                },
                DesignMediaPaintScaleMode::Tile => DesignMediaPaintPlacement::Tile {
                    scaling_factor: 1. + index as f32 * 0.25,
                    rotation,
                },
            };
            video.filters = DesignImageFilters {
                exposure: 0.04,
                contrast: 0.12,
                saturation: 0.10,
                temperature: -0.08,
                tint: 0.05,
                highlights: -0.16,
                shadows: 0.22,
            };
        }
        paint.sync_legacy_projection();
        paint_video.fills = vec![paint];
        nodes.push(paint_video);
    }

    let mut paint_opaque = DesignPanelNode::new(
        "paint-host-opaque",
        "Paint · Shader and unsupported",
        DesignPanelNodeKind::Rectangle,
    );
    paint_opaque.fills = vec![
        DesignPaint::from_payload(DesignPaintPayload::Shader(
            DesignShaderPaint::from_definition(&story_fractal_shader_definition())
                .expect("the Storybook Fractal noise shader is imported"),
        ))
        .with_id("shader-fill"),
        DesignPaint::unsupported("PLUGIN_PAINT", "{\"opaque\":true}")
            .with_id("unsupported-fill")
            .with_read_only(true),
    ];
    nodes.push(paint_opaque);

    let mut effects_lab = DesignPanelNode::new(
        "effects-all",
        "Effects · Every native type",
        DesignPanelNodeKind::Rectangle,
    );
    effects_lab.effects = DesignEffectKind::ALL
        .into_iter()
        .enumerate()
        .map(|(index, kind)| {
            DesignEffect::new(kind).with_id(format!("effects-all-{index}-{}", kind.label()))
        })
        .collect();
    effects_lab
        .effect_capabilities
        .show_shadow_behind_transparent_areas = true;
    effects_lab
        .effect_capabilities
        .set_kind_availability(DesignEffectKindAvailability::available(
            DesignEffectKind::Shader,
        ));
    if let Some(drop_shadow) = effects_lab.effects.first_mut() {
        drop_shadow.set_variable_binding(DesignEffectVariableBinding::new(
            DesignEffectVariableField::Color,
            "effect-shadow-color",
            "Shadow / Default",
            "Semantic colors",
        ));
    }
    if let Some(shader) = effects_lab
        .effects
        .iter_mut()
        .find(|effect| effect.kind == DesignEffectKind::Shader)
    {
        shader.set_settings(DesignEffectSettings::Shader(DesignShaderEffect::new(
            "shader:effect:refraction",
            "Refraction study",
            [
                DesignShaderProperty::new(
                    "enabled",
                    "Enabled",
                    DesignShaderPropertyKind::Boolean,
                    DesignShaderPropertyValue::Boolean(true),
                ),
                DesignShaderProperty::new(
                    "label",
                    "Label",
                    DesignShaderPropertyKind::Text,
                    DesignShaderPropertyValue::Text("Glass surface".into()),
                ),
                DesignShaderProperty::new(
                    "scale",
                    "Scale",
                    DesignShaderPropertyKind::Number,
                    DesignShaderPropertyValue::Number(4.),
                ),
                DesignShaderProperty::new(
                    "image",
                    "Environment image",
                    DesignShaderPropertyKind::Image,
                    DesignShaderPropertyValue::AssetId("image:environment".into()),
                ),
                DesignShaderProperty::new(
                    "instance",
                    "Surface component",
                    DesignShaderPropertyKind::InstanceSwap,
                    DesignShaderPropertyValue::AssetId("component:surface".into()),
                ),
                DesignShaderProperty::new(
                    "slot",
                    "Content slot",
                    DesignShaderPropertyKind::Slot,
                    DesignShaderPropertyValue::AssetId("slot:content".into()),
                ),
                DesignShaderProperty::new(
                    "tint",
                    "Tint",
                    DesignShaderPropertyKind::Color,
                    DesignShaderPropertyValue::Color(DesignColor::PURPLE),
                ),
                DesignShaderProperty::new(
                    "center",
                    "Center",
                    DesignShaderPropertyKind::Point,
                    DesignShaderPropertyValue::Point(fanta_gpui::design::DesignEffectVector::new(
                        0.5, 0.5,
                    )),
                ),
                DesignShaderProperty::new(
                    "ray",
                    "Refraction line",
                    DesignShaderPropertyKind::Line,
                    DesignShaderPropertyValue::Line {
                        start: fanta_gpui::design::DesignEffectVector::new(0.1, 0.2),
                        end: fanta_gpui::design::DesignEffectVector::new(0.9, 0.8),
                    },
                ),
                DesignShaderProperty::new(
                    "lens",
                    "Lens",
                    DesignShaderPropertyKind::Circle,
                    DesignShaderPropertyValue::Circle {
                        center: fanta_gpui::design::DesignEffectVector::new(0.5, 0.5),
                        radius: 0.35,
                    },
                ),
                DesignShaderProperty::new(
                    "orbit",
                    "Highlight orbit",
                    DesignShaderPropertyKind::CirclePoint,
                    DesignShaderPropertyValue::CirclePoint {
                        center: fanta_gpui::design::DesignEffectVector::new(0.5, 0.5),
                        radius: 0.3,
                        angle: 45.,
                    },
                ),
                DesignShaderProperty::new(
                    "color-point",
                    "Chromatic focus",
                    DesignShaderPropertyKind::ColorPoint,
                    DesignShaderPropertyValue::ColorPoint {
                        point: fanta_gpui::design::DesignEffectVector::new(0.35, 0.6),
                        color: DesignColor::BLUE,
                        variable_id: Some("variable:brand:accent".into()),
                    },
                ),
                DesignShaderProperty::new(
                    "gradient",
                    "Dispersion gradient",
                    DesignShaderPropertyKind::Gradient,
                    DesignShaderPropertyValue::Gradient(vec![
                        fanta_gpui::design::DesignShaderGradientStop::new(0., DesignColor::PURPLE),
                        {
                            let mut stop = fanta_gpui::design::DesignShaderGradientStop::new(
                                0.5,
                                DesignColor::BLUE,
                            );
                            stop.variable_id = Some("variable:brand:accent".into());
                            stop
                        },
                        fanta_gpui::design::DesignShaderGradientStop::new(1., DesignColor::WHITE),
                    ]),
                ),
                DesignShaderProperty::new(
                    "alias",
                    "Bound amount",
                    DesignShaderPropertyKind::Number,
                    DesignShaderPropertyValue::VariableAlias {
                        variable_id: "variable:shader:amount".into(),
                    },
                ),
                DesignShaderProperty::new(
                    "future",
                    "Future payload",
                    DesignShaderPropertyKind::Unsupported,
                    DesignShaderPropertyValue::Opaque {
                        type_name: "MESH_PATCH".into(),
                        payload: "{\"patches\":4,\"mode\":\"future\"}".into(),
                    },
                ),
            ],
        )));
    }
    effects_lab.effects.push(
        DesignEffect::from_settings(
            true,
            DesignEffectSettings::Opaque(fanta_gpui::design::DesignOpaqueEffect::new(
                "FUTURE_EFFECT",
                "Future effect",
                "{\"version\":2}",
            )),
        )
        .with_id("effects-all-future"),
    );
    nodes.push(effects_lab);

    let mut effects_style_bound = DesignPanelNode::new(
        "effects-style-bound",
        "Effects · Bound style",
        DesignPanelNodeKind::Rectangle,
    );
    effects_style_bound.effects = vec![
        DesignEffect::new(DesignEffectKind::BackgroundBlur).with_id("style-blur"),
        DesignEffect::new(DesignEffectKind::DropShadow).with_id("style-shadow"),
    ];
    effects_style_bound.effect_style_binding = Some(DesignEffectStyleBinding::new(
        DesignEffectStyleSelection::library("fanta-effects", "effect-library-overlay"),
        "Overlay / Raised",
    ));
    nodes.push(effects_style_bound);

    let mut arrow_line = DesignPanelNode::new(
        "stroke-line-arrow",
        "Line · Arrow cap",
        DesignPanelNodeKind::Line,
    );
    if let Some(stroke) = arrow_line.stroke.as_mut() {
        stroke.end_cap = DesignStrokeCap::LineArrow;
    }
    nodes.push(arrow_line);

    let mut dashed_line = DesignPanelNode::new(
        "stroke-dashed-arrow",
        "Line · Dashed triangle arrow",
        DesignPanelNodeKind::Line,
    );
    if let Some(stroke) = dashed_line.stroke.as_mut() {
        stroke.set_dash_mode(DesignStrokeDashMode::Custom);
        stroke.set_dash_pattern(vec![12., 6., 2., 6.]);
        stroke.dash_cap = DesignStrokeCap::Round;
        stroke.start_cap = DesignStrokeCap::Circle;
        stroke.end_cap = DesignStrokeCap::TriangleArrow;
    }
    nodes.push(dashed_line);

    let mut simple_dashed_line = DesignPanelNode::new(
        "stroke-dashed-simple",
        "Line · Dashed",
        DesignPanelNodeKind::Line,
    );
    if let Some(stroke) = simple_dashed_line.stroke.as_mut() {
        stroke.set_dash_mode(DesignStrokeDashMode::Dashed);
        stroke.set_dash_pattern(vec![8., 4.]);
    }
    nodes.push(simple_dashed_line);

    let mut brush_vector = DesignPanelNode::new(
        "stroke-brush",
        "Vector · Brush stroke",
        DesignPanelNodeKind::Vector,
    );
    if let Some(stroke) = brush_vector.stroke.as_mut() {
        stroke.complex_stroke = DesignComplexStroke::StretchBrush(DesignStretchBrushStroke {
            brush: DesignStretchBrushName::Noir,
            direction: DesignStrokeBrushDirection::Backward,
        });
        stroke.variable_width = Some(DesignVariableWidthStroke::Preset(
            DesignVariableWidthPreset::Taper,
        ));
    }
    nodes.push(brush_vector);

    let mut scatter_vector = DesignPanelNode::new(
        "stroke-scatter-brush",
        "Vector · Scatter brush",
        DesignPanelNodeKind::Vector,
    );
    if let Some(stroke) = scatter_vector.stroke.as_mut() {
        stroke.complex_stroke = DesignComplexStroke::ScatterBrush(DesignScatterBrushStroke {
            brush: DesignScatterBrushName::Vaporwave,
            gap: 0.75,
            wiggle: 1.25,
            size_jitter: 1.5,
            angular_jitter: -30.,
            rotation: 45.,
        });
    }
    nodes.push(scatter_vector);

    let mut dynamic_vector = DesignPanelNode::new(
        "stroke-dynamic",
        "Vector · Dynamic stroke",
        DesignPanelNodeKind::Vector,
    );
    if let Some(stroke) = dynamic_vector.stroke.as_mut() {
        stroke.complex_stroke = DesignComplexStroke::Dynamic(DesignDynamicStroke {
            frequency: 12.,
            wiggle: 3.8,
            smoothen: 0.72,
        });
    }
    nodes.push(dynamic_vector);

    for preset in DesignVariableWidthPreset::ALL {
        let mut variable_vector = DesignPanelNode::new(
            format!("stroke-variable-{:?}", preset).to_ascii_lowercase(),
            format!("Vector · Variable width · {}", preset.label()),
            DesignPanelNodeKind::Vector,
        );
        if let Some(stroke) = variable_vector.stroke.as_mut() {
            stroke.variable_width = Some(DesignVariableWidthStroke::Preset(preset));
        }
        nodes.push(variable_vector);
    }

    let mut custom_width_vector = DesignPanelNode::new(
        "stroke-variable-custom",
        "Vector · Variable width · Ordered custom points",
        DesignPanelNodeKind::Vector,
    );
    if let Some(stroke) = custom_width_vector.stroke.as_mut() {
        stroke.variable_width = Some(DesignVariableWidthStroke::custom([
            DesignVariableWidthPoint::new(0., 0.2),
            DesignVariableWidthPoint::new(0.35, 1.4),
            DesignVariableWidthPoint::new(1., 0.5),
        ]));
    }
    nodes.push(custom_width_vector);

    let mut opaque_stroke = DesignPanelNode::new(
        "stroke-opaque-custom",
        "Vector · Read-only custom brush",
        DesignPanelNodeKind::Vector,
    );
    if let Some(stroke) = opaque_stroke.stroke.as_mut() {
        stroke.complex_stroke = DesignComplexStroke::Opaque(DesignOpaqueComplexStroke::new(
            "CUSTOM",
            "Imported custom brush",
            "{\"brushName\":\"CUSTOM\",\"source\":\"host\"}",
        ));
    }
    nodes.push(opaque_stroke);

    let mut corner_lab = DesignPanelNode::new(
        "corners-independent",
        "Rectangle · Independent smooth corners",
        DesignPanelNodeKind::Rectangle,
    );
    corner_lab.independent_corners = true;
    corner_lab.corner_radii = [4., 12., 24., 32.];
    corner_lab.corner_smoothing = 0.6;
    nodes.push(corner_lab);

    for resize in DesignTextResize::ALL {
        let mut text = DesignPanelNode::new(
            format!("text-resize-{:?}", resize).to_ascii_lowercase(),
            format!("Text · {}", resize.label()),
            DesignPanelNodeKind::Text,
        );
        if let Some(typography) = text.typography.as_mut() {
            typography.resize = resize;
            typography.truncate = true;
            typography.max_lines = (resize == DesignTextResize::AutoWidth).then_some(2);
        }
        nodes.push(text);
    }

    let mut vertical_hug_text = DesignPanelNode::new(
        "text-max-lines-vertical-hug",
        "Text · Auto layout · Vertical Hug · 4 lines",
        DesignPanelNodeKind::Text,
    );
    if let Some(typography) = vertical_hug_text.typography.as_mut() {
        typography.resize = DesignTextResize::AutoHeight;
        typography.truncate = true;
        typography.max_lines = Some(4);
    }
    vertical_hug_text
        .layout
        .as_mut()
        .expect("Text fixtures support dimensions")
        .vertical_sizing = DesignSizingMode::Hug;
    nodes.push(vertical_hug_text);

    let mut vertical_fixed_text = DesignPanelNode::new(
        "text-max-lines-vertical-fixed",
        "Text · Auto layout · Vertical Fixed · Max lines unavailable",
        DesignPanelNodeKind::Text,
    );
    if let Some(typography) = vertical_fixed_text.typography.as_mut() {
        typography.resize = DesignTextResize::AutoHeight;
        typography.truncate = true;
        typography.max_lines = None;
    }
    vertical_fixed_text
        .layout
        .as_mut()
        .expect("Text fixtures support dimensions")
        .vertical_sizing = DesignSizingMode::Fixed;
    nodes.push(vertical_fixed_text);

    let mut max_height_text = DesignPanelNode::new(
        "text-max-lines-max-height",
        "Text · Max height 96 · Max lines Auto",
        DesignPanelNodeKind::Text,
    );
    if let Some(typography) = max_height_text.typography.as_mut() {
        typography.resize = DesignTextResize::AutoWidth;
        typography.truncate = true;
        typography.max_lines = None;
    }
    max_height_text
        .layout
        .as_mut()
        .expect("Text fixtures support dimensions")
        .item
        .max_height = Some(96.);
    nodes.push(max_height_text);

    let guide_color = DesignColor::rgb(0xf2, 0x48, 0x22);
    let mut uniform_guides = DesignPanelNode::new(
        "guides-uniform",
        "Layout guides · Uniform",
        DesignPanelNodeKind::Frame,
    );
    uniform_guides.layout_grids = vec![
        DesignLayoutGrid::uniform(8., guide_color)
            .with_opacity(18.)
            .with_id("guides-uniform-grid"),
    ];
    uniform_guides.layout_grid_style_binding = Some(DesignLayoutGridStyleBinding::new(
        "grid-style-8pt",
        "Foundations / 8 pt grid",
    ));
    nodes.push(uniform_guides);
    let mut variable_uniform_guides = DesignPanelNode::new(
        "guides-uniform-variable",
        "Layout guides · Uniform variable",
        DesignPanelNodeKind::Frame,
    );
    variable_uniform_guides.layout_grids = vec![
        DesignLayoutGrid::uniform(8., guide_color)
            .with_opacity(18.)
            .with_id("guides-uniform-variable-grid")
            .with_variable_binding(
                DesignLayoutGridVariableField::SectionSize,
                DesignLayoutGridVariableBinding::new("spacing-8", "Spacing / 8")
                    .with_collection("Spacing"),
            ),
    ];
    nodes.push(variable_uniform_guides);

    for (index, alignment) in DesignColumnGridAlignment::ALL.into_iter().enumerate() {
        let settings = DesignColumnLayoutGrid {
            alignment,
            count: if index == 0 {
                DesignLayoutGridCount::Auto
            } else {
                DesignLayoutGridCount::number((index as u16 + 2) * 3)
            },
            size: 72. + index as f32 * 8.,
            offset: 16. + index as f32 * 4.,
            gutter: 20. + index as f32 * 2.,
            margin: 24. + index as f32 * 4.,
            ..DesignColumnLayoutGrid::default()
        };
        let mut guide = DesignPanelNode::new(
            format!("guides-columns-{:?}", alignment).to_ascii_lowercase(),
            format!("Layout guides · Columns {}", alignment.label()),
            DesignPanelNodeKind::Frame,
        );
        let mut grid = DesignLayoutGrid::columns(settings, guide_color)
            .with_opacity(12.)
            .with_id(format!("guides-columns-{index}"));
        if index == 0 {
            grid = grid
                .with_variable_binding(
                    DesignLayoutGridVariableField::Count,
                    DesignLayoutGridVariableBinding::new("auto-track-count", "Auto tracks")
                        .with_collection("Responsive"),
                )
                .with_variable_binding(
                    DesignLayoutGridVariableField::SectionSize,
                    DesignLayoutGridVariableBinding::new("grid-section-64", "Grid section / 64")
                        .with_collection("Spacing"),
                )
                .with_variable_binding(
                    DesignLayoutGridVariableField::Offset,
                    DesignLayoutGridVariableBinding::new("spacing-8", "Spacing / 8")
                        .with_collection("Spacing"),
                )
                .with_variable_binding(
                    DesignLayoutGridVariableField::GutterSize,
                    DesignLayoutGridVariableBinding::new("grid-gutter-24", "Grid gutter / 24")
                        .with_collection("Spacing"),
                );
        } else if index == 2 {
            grid = grid.with_variable_binding(
                DesignLayoutGridVariableField::Count,
                DesignLayoutGridVariableBinding::new("desktop-column-count", "Desktop columns")
                    .with_collection("Responsive"),
            );
        } else if index == 3 {
            grid = grid.with_variable_binding(
                DesignLayoutGridVariableField::Offset,
                DesignLayoutGridVariableBinding::new("spacing-8", "Spacing / 8")
                    .with_collection("Spacing"),
            );
        }
        guide.layout_grids = vec![grid];
        nodes.push(guide);
    }

    for (index, alignment) in DesignRowGridAlignment::ALL.into_iter().enumerate() {
        let settings = DesignRowLayoutGrid {
            alignment,
            count: if index == 0 {
                DesignLayoutGridCount::Auto
            } else {
                DesignLayoutGridCount::number((index as u16 + 1) * 4)
            },
            size: 48. + index as f32 * 8.,
            offset: 12. + index as f32 * 4.,
            gutter: 12. + index as f32 * 2.,
            margin: 20. + index as f32 * 4.,
            ..DesignRowLayoutGrid::default()
        };
        let mut guide = DesignPanelNode::new(
            format!("guides-rows-{:?}", alignment).to_ascii_lowercase(),
            format!("Layout guides · Rows {}", alignment.label()),
            DesignPanelNodeKind::Frame,
        );
        let mut grid = DesignLayoutGrid::rows(settings, DesignColor::BLUE)
            .with_opacity(12.)
            .with_id(format!("guides-rows-{index}"));
        if index == 0 {
            grid = grid.with_variable_binding(
                DesignLayoutGridVariableField::Count,
                DesignLayoutGridVariableBinding::new("auto-track-count", "Auto tracks")
                    .with_collection("Responsive"),
            );
        } else if index == 2 {
            grid = grid.with_variable_binding(
                DesignLayoutGridVariableField::Count,
                DesignLayoutGridVariableBinding::new("desktop-row-count", "Desktop rows")
                    .with_collection("Responsive"),
            );
        }
        guide.layout_grids = vec![grid];
        if index == 3 {
            guide.layout_grid_style_binding = Some(DesignLayoutGridStyleBinding::new(
                "grid-style-rows",
                "Responsive / Desktop rows",
            ));
        }
        nodes.push(guide);
    }

    let mut hidden_guides = DesignPanelNode::new(
        "guides-hidden",
        "Layout guides · Hidden",
        DesignPanelNodeKind::Frame,
    );
    let mut hidden_grid = DesignLayoutGrid::uniform(4., DesignColor::BLUE)
        .with_opacity(8.)
        .with_id("guides-hidden-grid");
    hidden_grid.visible = false;
    hidden_guides.layout_grids = vec![hidden_grid];
    nodes.push(hidden_guides);

    let mut locked_style_guides = DesignPanelNode::new(
        "guides-style-locked",
        "Layout guides · Non-detachable style",
        DesignPanelNodeKind::Frame,
    );
    locked_style_guides.layout_grids = vec![
        DesignLayoutGrid::columns(DesignColumnLayoutGrid::default(), guide_color)
            .with_opacity(10.)
            .with_id("guides-locked-columns"),
    ];
    locked_style_guides.layout_grid_style_binding = Some(
        DesignLayoutGridStyleBinding::library(
            "fanta-grid-library",
            "grid-style-locked",
            "Library / Locked desktop grid",
        )
        .with_detach_allowed(false),
    );
    nodes.push(locked_style_guides);

    let mut guide_stack = DesignPanelNode::new(
        "guides-stack",
        "Layout guides · Three-guide stack",
        DesignPanelNodeKind::Frame,
    );
    guide_stack.layout_grids = vec![
        DesignLayoutGrid::uniform(8., DesignColor::BLUE)
            .with_opacity(8.)
            .with_id("guides-stack-uniform"),
        DesignLayoutGrid::columns(DesignColumnLayoutGrid::default(), guide_color)
            .with_opacity(12.)
            .with_id("guides-stack-columns"),
        DesignLayoutGrid::rows(DesignRowLayoutGrid::default(), DesignColor::PURPLE)
            .with_opacity(10.)
            .with_id("guides-stack-rows"),
    ];
    nodes.push(guide_stack);

    let missing_variant_main = DesignComponentReference::local("component-set-button", "Button")
        .with_availability(DesignComponentAvailability::Missing);
    let mut variant_context = DesignComponentContext::new(DesignComponentRole::VariantChild)
        .with_main_component(missing_variant_main);
    variant_context.description =
        Some("Variant child with an unresolved component-set reference.".into());
    variant_context.overrides = DesignComponentOverrideSummary {
        overridden_property_count: 2,
        nested_override_count: 0,
        reset_state: DesignComponentResetState::Unavailable {
            reason: "The source component set is missing.".into(),
        },
    };
    let mut mixed_variant_property = DesignComponentProperty::variant(
        "variant-child-state",
        "State",
        "Default",
        "Hover",
        vec!["Default".into(), "Hover".into(), "Pressed".into()],
    )
    .with_reset_state(DesignComponentResetState::Resettable);
    mixed_variant_property.override_state = DesignComponentPropertyOverrideState::Mixed;
    let unavailable_property =
        DesignComponentProperty::text("variant-child-label", "Label", "Button", "Continue")
            .with_reset_state(DesignComponentResetState::Unavailable {
                reason: "The source property cannot be resolved.".into(),
            });
    let mut variant_child = DesignPanelNode::new(
        "component-variant-child-missing",
        "Component · Variant child · Missing main",
        DesignPanelNodeKind::Component,
    )
    .with_component_context(variant_context);
    variant_child.component_properties = vec![mixed_variant_property, unavailable_property];
    nodes.push(variant_child);

    let preferred_card = DesignComponentReference::remote(
        "slot-content-card",
        "Content card",
        "Product foundations",
    );
    let preferred_empty = DesignComponentReference::local("slot-empty-state", "Empty state");
    let nonpreferred_banner = DesignComponentReference::local("slot-promo-banner", "Promo banner");
    let available_slot_main = DesignComponentReference::local("slot-shell", "Content slot");
    let unavailable_slot_main = DesignComponentReference::remote(
        "slot-shell-library",
        "Content slot",
        "Unavailable library",
    )
    .with_availability(DesignComponentAvailability::Unavailable {
        reason: "The component library is not loaded.".into(),
    });

    let make_slot_layer =
        |node_id: &'static str,
         name: &'static str,
         main_component: Option<DesignComponentReference>| {
            let mut instance = DesignSlotChild::instance(node_id, name);
            instance.main_component = main_component;
            instance
        };
    let make_slot_story =
        |id: &'static str,
         name: &'static str,
         main_component: DesignComponentReference,
         property: DesignComponentProperty,
         aggregate_reset_state: DesignComponentResetState| {
            let overridden_property_count =
                u32::from(property.override_state != DesignComponentPropertyOverrideState::Default);
            let mut context = DesignComponentContext::new(DesignComponentRole::SlotInstance)
                .with_main_component(main_component);
            context.description =
                Some("Slot-instance contents are supplied by the Storybook host.".into());
            context.overrides = DesignComponentOverrideSummary {
                overridden_property_count,
                nested_override_count: 0,
                reset_state: aggregate_reset_state,
            };
            let mut node = DesignPanelNode::new(id, name, DesignPanelNodeKind::Instance)
                .with_component_context(context);
            node.component_properties = vec![property];
            node
        };

    let populated_value = DesignSlotValue {
        children: vec![
            make_slot_layer(
                "slot-populated-card",
                "Content card",
                Some(preferred_card.clone()),
            ),
            DesignSlotChild::layer(
                "slot-populated-copy",
                "Supporting copy",
                DesignPanelNodeKind::Text,
            ),
            DesignSlotChild::layer(
                "slot-populated-image",
                "Hero image",
                DesignPanelNodeKind::Rectangle,
            ),
        ],
    };
    let populated_property = DesignComponentProperty::slot(
        "slot-populated-content",
        "Content",
        DesignSlotValue::default(),
        populated_value,
        DesignSlotSettings {
            stretch_child_on_insert: true,
            display_empty: true,
            minimum_children: None,
            maximum_children: Some(3),
            preferred_values_only: false,
            preferred_values: vec![preferred_card.clone(), preferred_empty.clone()],
        },
        DesignSlotState {
            violations: Vec::new(),
            reset_state: DesignComponentResetState::Resettable,
        },
    )
    .with_reset_state(DesignComponentResetState::Resettable);
    nodes.push(make_slot_story(
        "component-slot-instance-populated",
        "Instance · Slot · Populated",
        available_slot_main.clone(),
        populated_property,
        DesignComponentResetState::Resettable,
    ));

    let exact_capacity_value = DesignSlotValue {
        children: vec![
            make_slot_layer(
                "slot-capacity-card",
                "Content card",
                Some(preferred_card.clone()),
            ),
            make_slot_layer(
                "slot-capacity-empty",
                "Empty state",
                Some(preferred_empty.clone()),
            ),
        ],
    };
    let exact_capacity_property = DesignComponentProperty::slot(
        "slot-exact-capacity-content",
        "Content",
        DesignSlotValue::default(),
        exact_capacity_value,
        DesignSlotSettings {
            stretch_child_on_insert: false,
            display_empty: true,
            minimum_children: Some(1),
            maximum_children: Some(2),
            preferred_values_only: true,
            preferred_values: vec![preferred_card.clone(), preferred_empty.clone()],
        },
        DesignSlotState {
            violations: Vec::new(),
            reset_state: DesignComponentResetState::Resettable,
        },
    )
    .with_reset_state(DesignComponentResetState::Resettable);
    nodes.push(make_slot_story(
        "component-slot-instance-exact-capacity",
        "Instance · Slot · Exact capacity",
        unavailable_slot_main,
        exact_capacity_property,
        DesignComponentResetState::Unavailable {
            reason: "Overrides cannot be reset while the library is unavailable.".into(),
        },
    ));

    let below_minimum_property = DesignComponentProperty::slot(
        "slot-below-minimum-content",
        "Required content",
        DesignSlotValue::default(),
        DesignSlotValue::default(),
        DesignSlotSettings {
            stretch_child_on_insert: true,
            display_empty: true,
            minimum_children: Some(2),
            maximum_children: Some(4),
            preferred_values_only: false,
            preferred_values: vec![preferred_card.clone()],
        },
        DesignSlotState {
            violations: vec![DesignSlotViolation::BelowMinimum {
                minimum: 2,
                actual: 0,
            }],
            reset_state: DesignComponentResetState::Clean,
        },
    )
    .with_reset_state(DesignComponentResetState::Clean);
    nodes.push(make_slot_story(
        "component-slot-instance-below-minimum",
        "Instance · Slot · Below minimum",
        available_slot_main.clone(),
        below_minimum_property,
        DesignComponentResetState::Clean,
    ));

    let above_maximum_value = DesignSlotValue {
        children: vec![
            make_slot_layer(
                "slot-above-card-1",
                "Content card 1",
                Some(preferred_card.clone()),
            ),
            make_slot_layer(
                "slot-above-card-2",
                "Content card 2",
                Some(preferred_card.clone()),
            ),
            make_slot_layer(
                "slot-above-card-3",
                "Content card 3",
                Some(preferred_card.clone()),
            ),
        ],
    };
    let above_maximum_property = DesignComponentProperty::slot(
        "slot-above-maximum-content",
        "Content",
        DesignSlotValue::default(),
        above_maximum_value,
        DesignSlotSettings {
            stretch_child_on_insert: true,
            display_empty: true,
            minimum_children: None,
            maximum_children: Some(2),
            preferred_values_only: true,
            preferred_values: vec![preferred_card.clone()],
        },
        DesignSlotState {
            violations: vec![DesignSlotViolation::AboveMaximum {
                maximum: 2,
                actual: 3,
            }],
            reset_state: DesignComponentResetState::Resettable,
        },
    )
    .with_reset_state(DesignComponentResetState::Resettable);
    nodes.push(make_slot_story(
        "component-slot-instance-above-maximum",
        "Instance · Slot · Above maximum",
        available_slot_main.clone(),
        above_maximum_property,
        DesignComponentResetState::Resettable,
    ));

    let nonpreferred_value = DesignSlotValue {
        children: vec![make_slot_layer(
            "slot-nonpreferred-banner",
            "Promo banner",
            Some(nonpreferred_banner.clone()),
        )],
    };
    let nonpreferred_property = DesignComponentProperty::slot(
        "slot-nonpreferred-content",
        "Content",
        DesignSlotValue::default(),
        nonpreferred_value,
        DesignSlotSettings {
            stretch_child_on_insert: true,
            display_empty: true,
            minimum_children: None,
            maximum_children: Some(2),
            preferred_values_only: true,
            preferred_values: vec![preferred_card.clone(), preferred_empty],
        },
        DesignSlotState {
            violations: vec![DesignSlotViolation::NonPreferredValue {
                instance_id: "slot-nonpreferred-banner".into(),
                component_name: nonpreferred_banner.name.clone(),
            }],
            reset_state: DesignComponentResetState::Resettable,
        },
    )
    .with_reset_state(DesignComponentResetState::Resettable);
    nodes.push(make_slot_story(
        "component-slot-instance-nonpreferred",
        "Instance · Slot · Nonpreferred value",
        available_slot_main.clone(),
        nonpreferred_property,
        DesignComponentResetState::Resettable,
    ));

    let missing_inserted_main_value = DesignSlotValue {
        children: vec![make_slot_layer(
            "slot-missing-inserted-main",
            "Detached content",
            None,
        )],
    };
    let missing_inserted_main_property = DesignComponentProperty::slot(
        "slot-missing-main-content",
        "Content",
        DesignSlotValue::default(),
        missing_inserted_main_value,
        DesignSlotSettings {
            stretch_child_on_insert: true,
            display_empty: true,
            minimum_children: None,
            maximum_children: Some(2),
            preferred_values_only: true,
            preferred_values: vec![preferred_card],
        },
        DesignSlotState {
            violations: vec![DesignSlotViolation::MissingMainComponent {
                instance_id: "slot-missing-inserted-main".into(),
            }],
            reset_state: DesignComponentResetState::Resettable,
        },
    )
    .with_reset_state(DesignComponentResetState::Resettable);
    nodes.push(make_slot_story(
        "component-slot-instance-missing-main",
        "Instance · Slot · Missing inserted main",
        available_slot_main,
        missing_inserted_main_property,
        DesignComponentResetState::Resettable,
    ));

    nodes.extend(story_homogeneous_multiple_nodes());
    assign_story_paint_ids(&mut nodes);
    nodes
}

pub(crate) fn assign_story_paint_ids(nodes: &mut [DesignPanelNode]) {
    for node in nodes {
        let node_id = node.id.clone();
        let assign_collection = |paints: &mut [DesignPaint], collection: &str| {
            for (paint_index, paint) in paints.iter_mut().enumerate() {
                if paint.id.is_empty() {
                    paint.id = format!("{node_id}-{collection}-{paint_index}").into();
                }
                if let DesignPaintPayload::Gradient(gradient) = &mut paint.payload {
                    for (stop_index, stop) in gradient.stops.iter_mut().enumerate() {
                        if stop.id.is_empty() {
                            stop.id = format!("{}-stop-{stop_index}", paint.id).into();
                        }
                    }
                    paint.sync_legacy_projection();
                }
            }
        };

        assign_collection(&mut node.fills, "fill");
        assign_collection(&mut node.selection_colors, "selection-color");
        if let Some(stroke) = node.stroke.as_mut() {
            assign_collection(&mut stroke.paints, "stroke");
        }
    }
}

pub(crate) fn seed_design_color_contrast(nodes: &[DesignPanelNode]) -> DesignColorContrastViewData {
    let mut views = Vec::new();
    for node in nodes {
        for (collection, paints) in [
            (DesignPanelCollection::Fill, node.fills.as_slice()),
            (
                DesignPanelCollection::Stroke,
                node.stroke
                    .as_ref()
                    .map_or(&[] as &[DesignPaint], |stroke| stroke.paints.as_slice()),
            ),
        ] {
            for (index, paint) in paints.iter().enumerate() {
                let automatic_category = if matches!(
                    node.kind,
                    DesignPanelNodeKind::Text | DesignPanelNodeKind::TextPath
                ) {
                    DesignColorContrastCategory::NormalText
                } else {
                    DesignColorContrastCategory::Graphics
                };
                let effective_background = DesignColor::WHITE;
                let make_leaf = |color_target, color: DesignColor| {
                    let foreground = story_color_with_opacity(color, paint.opacity);
                    let ratio = story_contrast_ratio(foreground, effective_background);
                    let mut corrections = Vec::new();
                    for category in [
                        DesignColorContrastCategory::LargeText,
                        DesignColorContrastCategory::NormalText,
                        DesignColorContrastCategory::Graphics,
                    ] {
                        for level in DesignColorContrastLevel::ALL {
                            let Some(threshold) = story_contrast_threshold(category, level) else {
                                continue;
                            };
                            let mut black =
                                story_color_with_opacity(DesignColor::rgb(0, 0, 0), paint.opacity);
                            let mut white =
                                story_color_with_opacity(DesignColor::WHITE, paint.opacity);
                            black.alpha = color.alpha;
                            white.alpha = color.alpha;
                            let black_ratio = story_contrast_ratio(black, effective_background);
                            let white_ratio = story_contrast_ratio(white, effective_background);
                            let (candidate, candidate_ratio) = if black_ratio >= white_ratio {
                                (black, black_ratio)
                            } else {
                                (white, white_ratio)
                            };
                            if candidate_ratio >= threshold {
                                corrections.push(DesignColorContrastCorrection::new(
                                    category, level, candidate,
                                ));
                            }
                        }
                    }
                    DesignColorContrastLeafViewData::new(
                        color_target,
                        effective_background,
                        ratio,
                        automatic_category,
                    )
                    .with_corrections(corrections)
                };
                let leaves = match &paint.payload {
                    DesignPaintPayload::Solid(solid) => {
                        vec![make_leaf(DesignPaintColorTarget::Solid, solid.color)]
                    }
                    DesignPaintPayload::Gradient(gradient) => gradient
                        .stops
                        .iter()
                        .enumerate()
                        .map(|(stop_index, stop)| {
                            make_leaf(
                                DesignPaintColorTarget::GradientStop {
                                    stop_id: stop.id.clone(),
                                    index: stop_index,
                                },
                                stop.color,
                            )
                        })
                        .collect(),
                    DesignPaintPayload::Pattern(_)
                    | DesignPaintPayload::Image(_)
                    | DesignPaintPayload::Video(_)
                    | DesignPaintPayload::Shader(_)
                    | DesignPaintPayload::Unsupported(_) => Vec::new(),
                };
                if !leaves.is_empty() {
                    views.push(DesignColorContrastPaintViewData::new(
                        DesignColorContrastPaintTarget::paint(
                            node.id.clone(),
                            collection,
                            paint.id.clone(),
                            index,
                        ),
                        leaves,
                    ));
                }
            }
        }
    }
    DesignColorContrastViewData::new(views)
}

pub(crate) fn seed_design_shaders() -> DesignShaderViewData {
    let fractal = story_fractal_shader_definition();
    let mesh = DesignShaderDefinition::new(
        "shader:page:mesh",
        "Mesh distortion",
        true,
        [
            DesignShaderPropertyDefinition::new(
                "def:strength",
                "Strength",
                DesignShaderPropertyKind::Number,
            )
            .with_default(DesignShaderPropertyValue::Number(0.5)),
            DesignShaderPropertyDefinition::new(
                "def:source",
                "Source image",
                DesignShaderPropertyKind::Image,
            )
            .with_default(DesignShaderPropertyValue::AssetId("image:hero".into())),
        ],
    );
    let library = DesignShaderLibrary::new(
        "shader-library:material",
        "Material shaders",
        [
            DesignShaderDefinition::new(
                "shader:library:chromatic-glass",
                "Chromatic glass",
                true,
                [
                    DesignShaderPropertyDefinition::new(
                        "def:refraction",
                        "Refraction",
                        DesignShaderPropertyKind::Number,
                    )
                    .with_default(DesignShaderPropertyValue::Number(0.8)),
                    DesignShaderPropertyDefinition::new(
                        "def:enabled",
                        "Enabled",
                        DesignShaderPropertyKind::Boolean,
                    )
                    .with_default(DesignShaderPropertyValue::Boolean(true)),
                ],
            ),
            DesignShaderDefinition::new("shader:library:liquid-metal", "Liquid metal", false, []),
        ],
    );
    DesignShaderViewData::new([fractal, mesh], [library])
}

// The Page view fixture intentionally exercises the deprecated
// local-resource read model until the Design story migrates off it.
#[allow(deprecated)]
pub(crate) fn seed_design_page_view_data() -> DesignPageViewData {
    let brand_source = DesignLocalResourceSource::library("library-brand", "Brand system");
    let local_styles = DesignLocalResourceGroup::new(
        "page-local-styles",
        "Styles",
        DesignLocalResourceSource::Local,
        [
            DesignLocalResource::local(
                "style-brand-primary",
                "Brand / Primary",
                DesignLocalResourceKind::PaintStyle,
            ),
            DesignLocalResource::local(
                "style-body",
                "Typography / Body",
                DesignLocalResourceKind::TextStyle,
            ),
            DesignLocalResource::local(
                "style-soft-shadow",
                "Effects / Soft shadow",
                DesignLocalResourceKind::EffectStyle,
            ),
            DesignLocalResource::local(
                "style-grid-desktop",
                "Layout / Desktop columns",
                DesignLocalResourceKind::GridStyle,
            ),
        ],
    );
    let local_variables = DesignLocalResourceGroup::new(
        "page-local-variables",
        "Variable collections",
        DesignLocalResourceSource::Local,
        [DesignLocalResource::local(
            "collection-theme",
            "Theme",
            DesignLocalResourceKind::VariableCollection,
        )],
    );
    let library_resources = DesignLocalResourceGroup::new(
        "brand-library-resources",
        "Brand library",
        brand_source,
        [
            DesignLocalResource::available(
                "style-marketing-accent",
                "Marketing / Accent",
                DesignLocalResourceKind::PaintStyle,
            ),
            DesignLocalResource::unavailable(
                "style-licensed-display",
                "Typography / Licensed display",
                DesignLocalResourceKind::TextStyle,
                "The library font is not available",
            ),
            DesignLocalResource::available(
                "collection-density",
                "Density",
                DesignLocalResourceKind::VariableCollection,
            ),
        ],
    );
    DesignPageViewData::new(
        "storybook-page",
        DesignPageBackground::new(DesignColor::rgb(0xf5, 0xf5, 0xf5)),
        DesignLocalResourceViewData::new([local_styles, local_variables, library_resources]),
    )
}

pub(crate) fn seed_design_page_local_styles() -> DesignPageLocalStylesViewData {
    let text = DesignLocalStyleSection::new(
        DesignLocalStyleKind::Text,
        [DesignLocalStyleEntry::folder(
            "folder-typography",
            "Typography",
            [
                DesignLocalStyleEntry::style(
                    DesignLocalStyleItem::new(
                        "style-text-body",
                        "Body",
                        DesignLocalStylePreview::Text(DesignTypographyStyle::new(
                            "style-text-body",
                            "Body",
                            "Inter",
                            "Regular",
                            16.,
                        )),
                    )
                    .with_description("Inter Regular · 16"),
                ),
                DesignLocalStyleEntry::folder(
                    "folder-typography-display",
                    "Display",
                    [DesignLocalStyleEntry::style(
                        DesignLocalStyleItem::new(
                            "style-text-hero",
                            "Hero",
                            DesignLocalStylePreview::Text(DesignTypographyStyle::new(
                                "style-text-hero",
                                "Hero",
                                "Inter",
                                "Bold",
                                64.,
                            )),
                        )
                        .with_description("Inter Bold · 64"),
                    )],
                ),
            ],
        )],
    );
    let color = DesignLocalStyleSection::new(
        DesignLocalStyleKind::Color,
        [
            DesignLocalStyleEntry::style(
                DesignLocalStyleItem::new(
                    "style-color-brand",
                    "Brand / Primary",
                    DesignLocalStylePreview::Color(vec![
                        DesignPaint::solid(DesignColor::rgb(80, 70, 229))
                            .with_id("style-color-brand-paint"),
                    ]),
                )
                .with_description("Primary product color"),
            ),
            DesignLocalStyleEntry::folder(
                "folder-semantic-colors",
                "Semantic",
                [DesignLocalStyleEntry::style(
                    DesignLocalStyleItem::new(
                        "style-color-success",
                        "Success",
                        DesignLocalStylePreview::Color(vec![
                            DesignPaint::solid(DesignColor::rgb(20, 160, 90))
                                .with_id("style-color-success-paint"),
                        ]),
                    )
                    .with_description("Positive status"),
                )],
            ),
        ],
    );
    let effect = DesignLocalStyleSection::new(
        DesignLocalStyleKind::Effect,
        [DesignLocalStyleEntry::style(
            DesignLocalStyleItem::new(
                "style-effect-soft-shadow",
                "Soft shadow",
                DesignLocalStylePreview::Effect(vec![
                    DesignEffect::drop_shadow(DesignColor::rgba(0, 0, 0, 64), 16., 0., 0., 8.)
                        .with_id("style-effect-soft-shadow-effect"),
                ]),
            )
            .with_description("0 8 16 · 25%"),
        )],
    );
    let layout_guide = DesignLocalStyleSection::new(
        DesignLocalStyleKind::LayoutGuide,
        [DesignLocalStyleEntry::style(
            DesignLocalStyleItem::new(
                "style-layout-desktop",
                "Desktop columns",
                DesignLocalStylePreview::LayoutGuide(vec![
                    DesignLayoutGrid::columns(
                        fanta_gpui::prelude::DesignColumnLayoutGrid::default(),
                        DesignColor::rgb(255, 0, 128),
                    )
                    .with_id("style-layout-desktop-grid"),
                ]),
            )
            .with_description("12 columns"),
        )],
    );
    DesignPageLocalStylesViewData::for_page("storybook-page", [text, color, effect, layout_guide])
}

pub(crate) fn seed_design_variable_mode_views(
    nodes: &[DesignPanelNode],
) -> HashMap<SharedString, DesignVariableModeViewData> {
    fn theme_collection(explicit_mode: Option<&str>) -> DesignVariableModeCollection {
        let collection = DesignVariableModeCollection::new(
            "collection-theme",
            "Theme",
            DesignLocalResourceSource::Local,
            [
                DesignVariableMode::new("theme-light", "Light"),
                DesignVariableMode::new("theme-dark", "Dark"),
                DesignVariableMode::new("theme-high-contrast", "High contrast"),
            ],
            "theme-light",
            explicit_mode.unwrap_or("theme-light").to_owned(),
        );
        explicit_mode.map_or(collection.clone(), |mode| {
            collection.explicit(mode.to_owned())
        })
    }

    fn density_collection(resolved_mode: &str) -> DesignVariableModeCollection {
        DesignVariableModeCollection::new(
            "collection-density",
            "Density",
            DesignLocalResourceSource::library("library-brand", "Brand system"),
            [
                DesignVariableMode::new("density-comfortable", "Comfortable"),
                DesignVariableMode::new("density-compact", "Compact"),
                DesignVariableMode::new("density-touch", "Touch")
                    .disabled("Touch density is unavailable for this target"),
            ],
            "density-comfortable",
            resolved_mode.to_owned(),
        )
    }

    let mut views = HashMap::new();
    let page_target = DesignPanelTarget::Page {
        page_id: "storybook-page".into(),
    };
    views.insert(
        "storybook-page".into(),
        DesignVariableModeViewData::new(
            page_target,
            [
                theme_collection(Some("theme-dark")),
                density_collection("density-compact"),
            ],
        ),
    );
    for (index, node) in nodes.iter().enumerate() {
        let target = DesignPanelTarget::Nodes {
            node_ids: vec![node.id.clone()],
        };
        let theme = if index % 3 == 0 {
            theme_collection(Some("theme-dark"))
        } else {
            theme_collection(None)
        };
        let density = density_collection(if index % 2 == 0 {
            "density-comfortable"
        } else {
            "density-compact"
        });
        views.insert(
            node.id.clone(),
            DesignVariableModeViewData::new(target, [theme, density]),
        );
    }
    views
}

pub(crate) fn seed_design_viewer_properties(
    nodes: &[DesignPanelNode],
) -> HashMap<SharedString, DesignViewerPropertiesViewData> {
    nodes
        .iter()
        .map(|node| {
            (
                node.id.clone(),
                viewer_properties_for_node(node, DesignViewerColorRepresentation::Css),
            )
        })
        .collect()
}

pub(crate) fn viewer_properties_for_node(
    node: &DesignPanelNode,
    border_representation: DesignViewerColorRepresentation,
) -> DesignViewerPropertiesViewData {
    let mut sections = Vec::new();
    if let Some(context) = node.component_context.as_ref() {
        sections.push(story_viewer_component_section(context));
    }
    sections.push(DesignViewerPropertySection::new(
        "layout",
        "Layout",
        [
            DesignViewerPropertyRow::new("x", "X", story_viewer_number(node.x))
                .with_property(DesignPanelProperty::X),
            DesignViewerPropertyRow::new("y", "Y", story_viewer_number(node.y))
                .with_property(DesignPanelProperty::Y),
            DesignViewerPropertyRow::new("width", "Width", story_viewer_px(node.width))
                .with_property(DesignPanelProperty::Width),
            DesignViewerPropertyRow::new("height", "Height", story_viewer_px(node.height))
                .with_property(DesignPanelProperty::Height),
        ],
    ));

    if let Some(typography) = &node.typography {
        let content = if node.kind == DesignPanelNodeKind::TextPath {
            "Design follows the path"
        } else {
            "Build thoughtful products faster with a shared design system."
        };
        sections.push(DesignViewerPropertySection::text_content(
            "content", content,
        ));
        let line_height = match typography.line_height {
            DesignLineHeight::Auto => "Auto".to_owned(),
            DesignLineHeight::Pixels(value) => story_viewer_px(value),
            DesignLineHeight::Percent(value) => format!("{}%", story_viewer_number(value)),
        };
        let letter_spacing = match typography.letter_spacing {
            DesignLetterSpacing::Pixels(value) => story_viewer_px(value),
            DesignLetterSpacing::Percent(value) => {
                format!("{}%", story_viewer_number(value))
            }
        };
        let typography_copy = format!(
            "font-family: \"{}\";\nfont-style: {};\nfont-weight: {};\nfont-size: {};\nline-height: {};\nletter-spacing: {};",
            typography.family,
            typography.style,
            story_viewer_number(typography.weight),
            story_viewer_px(typography.size),
            line_height,
            letter_spacing,
        );
        sections.push(
            DesignViewerPropertySection::new(
                "typography",
                "Typography",
                [
                    DesignViewerPropertyRow::new("font-family", "Font", typography.family.clone())
                        .with_property(DesignPanelProperty::FontFamily),
                    DesignViewerPropertyRow::new("font-style", "Style", typography.style.clone())
                        .with_property(DesignPanelProperty::FontStyle),
                    DesignViewerPropertyRow::new(
                        "font-weight",
                        "Weight",
                        story_viewer_number(typography.weight),
                    )
                    .with_property(DesignPanelProperty::FontWeight),
                    DesignViewerPropertyRow::new(
                        "font-size",
                        "Size",
                        story_viewer_px(typography.size),
                    )
                    .with_property(DesignPanelProperty::FontSize),
                    DesignViewerPropertyRow::new("line-height", "Line height", line_height)
                        .with_property(DesignPanelProperty::LineHeight),
                    DesignViewerPropertyRow::new(
                        "letter-spacing",
                        "Letter spacing",
                        letter_spacing,
                    )
                    .with_property(DesignPanelProperty::LetterSpacing),
                ],
            )
            .with_copy_value(typography_copy)
            .with_copy_all(),
        );
    }

    if let Some(fill) = node.fills.iter().find(|paint| paint.visible) {
        let summary = story_viewer_color(fill.color, DesignViewerColorRepresentation::Hex);
        sections.push(
            DesignViewerPropertySection::new("fills", "Fills", [])
                .with_summary(summary.clone())
                .with_copy_value(summary),
        );
    }

    if let Some(stroke) = &node.stroke {
        let color = stroke
            .paints
            .iter()
            .find(|paint| paint.visible)
            .map_or(DesignColor::BLACK, |paint| paint.color);
        let color_summary = story_viewer_color(color, border_representation);
        let weight = story_viewer_px(stroke.weights.active());
        let css_style = if stroke.dashes.mode == fanta_gpui::prelude::DesignStrokeDashMode::Solid {
            "solid"
        } else {
            "dashed"
        };
        let copy_value = if border_representation == DesignViewerColorRepresentation::Css {
            if stroke.weights.mode == DesignStrokeWeightMode::Custom {
                format!(
                    "border-style: {css_style};\nborder-width: {} {} {} {};\nborder-color: {};",
                    story_viewer_px(stroke.weights.top),
                    story_viewer_px(stroke.weights.right),
                    story_viewer_px(stroke.weights.bottom),
                    story_viewer_px(stroke.weights.left),
                    story_viewer_color(color, DesignViewerColorRepresentation::Css),
                )
            } else {
                format!(
                    "border: {weight} {css_style} {};",
                    story_viewer_color(color, DesignViewerColorRepresentation::Css),
                )
            }
        } else {
            color_summary.clone()
        };
        sections.push(
            DesignViewerPropertySection::new(
                "borders",
                "Borders",
                [
                    DesignViewerPropertyRow::new("color", "Color", color_summary),
                    DesignViewerPropertyRow::new("weight", "Weight", weight)
                        .with_property(DesignPanelProperty::StrokeWeight),
                    DesignViewerPropertyRow::new("position", "Position", stroke.align.label())
                        .with_property(DesignPanelProperty::StrokeAlign),
                ],
            )
            .with_summary(copy_value.clone())
            .with_copy_value(copy_value)
            .with_color_representation(border_representation),
        );
    }

    if !node.effects.is_empty() {
        let summary = node
            .effects
            .iter()
            .map(|effect| effect.kind.label())
            .collect::<Vec<_>>()
            .join(", ");
        sections.push(
            DesignViewerPropertySection::new("effects", "Effects", [])
                .with_summary(summary.clone())
                .with_copy_value(summary),
        );
    }

    DesignViewerPropertiesViewData::new(
        DesignPanelTarget::Nodes {
            node_ids: vec![node.id.clone()],
        },
        sections,
    )
}

pub(crate) fn seed_design_color_styles() -> DesignColorStyleViewData {
    let page_styles = [
        DesignColorStyle::new(
            "page-brand-primary",
            "Brand / Primary",
            DesignColor::rgb(0x0d, 0x99, 0xff),
        )
        .with_binding(
            DesignPaintBinding::new("brand-primary", "Brand / Primary")
                .with_collection("Page variables"),
        ),
        DesignColorStyle::new(
            "page-surface-canvas",
            "Surface / Canvas",
            DesignColor::rgb(0xf5, 0xf5, 0xf5),
        ),
        DesignColorStyle::new(
            "page-ink-primary",
            "Ink / Primary",
            DesignColor::rgb(0x1e, 0x1e, 0x1e),
        ),
        DesignColorStyle::new(
            "page-accent-purple",
            "Accent / Purple",
            DesignColor::rgb(0x97, 0x47, 0xff),
        ),
    ];
    let libraries = [
        DesignColorStyleLibrary::new(
            "fanta-foundations",
            "Fanta foundations",
            [
                DesignColorStyle::new(
                    "foundation-blue-500",
                    "Blue / 500",
                    DesignColor::rgb(0x0d, 0x99, 0xff),
                )
                .with_binding(
                    DesignPaintBinding::new("blue-500", "Blue / 500").with_collection("Primitives"),
                ),
                DesignColorStyle::new(
                    "foundation-green-500",
                    "Green / 500",
                    DesignColor::rgb(0x14, 0xae, 0x5c),
                )
                .with_binding(
                    DesignPaintBinding::new("green-500", "Green / 500")
                        .with_collection("Primitives"),
                ),
                DesignColorStyle::new(
                    "foundation-red-500",
                    "Red / 500",
                    DesignColor::rgb(0xf2, 0x48, 0x22),
                )
                .with_binding(
                    DesignPaintBinding::new("red-500", "Red / 500").with_collection("Primitives"),
                ),
            ],
        ),
        DesignColorStyleLibrary::new(
            "marketing-theme",
            "Marketing theme",
            [
                DesignColorStyle::new(
                    "marketing-sun",
                    "Campaign / Sun",
                    DesignColor::rgb(0xff, 0xc7, 0x00),
                ),
                DesignColorStyle::new(
                    "marketing-orchid",
                    "Campaign / Orchid",
                    DesignColor::rgb(0xd7, 0x32, 0xa8),
                ),
            ],
        ),
    ];
    DesignColorStyleViewData::new(page_styles, libraries)
}

pub(crate) fn seed_design_color_style_samples() -> DesignColorStyleSampleViewData {
    DesignColorStyleSampleViewData::new(
        [
            DesignColorStyleSample::new(
                "sample-page-brand",
                "Brand / Primary",
                DesignColor::rgb(0x0d, 0x99, 0xff),
            ),
            DesignColorStyleSample::new(
                "sample-page-ink",
                "Ink / Primary",
                DesignColor::rgb(0x1e, 0x1e, 0x1e),
            ),
            DesignColorStyleSample::new(
                "sample-page-overlay",
                "Overlay / Soft",
                DesignColor::rgba(0x97, 0x47, 0xff, 0x80),
            ),
        ],
        [
            DesignColorStyleSampleLibrary::new(
                "sample-library-foundations",
                "Fanta foundations",
                [
                    DesignColorStyleSample::new(
                        "sample-library-blue",
                        "Blue / 500",
                        DesignColor::rgb(0x0d, 0x99, 0xff),
                    ),
                    DesignColorStyleSample::new(
                        "sample-library-green",
                        "Green / 500",
                        DesignColor::rgb(0x14, 0xae, 0x5c),
                    ),
                    DesignColorStyleSample::new(
                        "sample-library-red",
                        "Red / 500",
                        DesignColor::rgb(0xf2, 0x48, 0x22),
                    ),
                ],
            ),
            DesignColorStyleSampleLibrary::new(
                "sample-library-marketing",
                "Marketing theme",
                [
                    DesignColorStyleSample::new(
                        "sample-library-sun",
                        "Campaign / Sun",
                        DesignColor::rgb(0xff, 0xc7, 0x00),
                    ),
                    DesignColorStyleSample::new(
                        "sample-library-orchid",
                        "Campaign / Orchid",
                        DesignColor::rgb(0xd7, 0x32, 0xa8),
                    )
                    .disabled("The Marketing library is view-only"),
                ],
            ),
        ],
    )
}

pub(crate) fn seed_design_paint_styles() -> DesignPaintStyleViewData {
    let page_styles = [
        DesignPaintStyle::new(
            "paint-style-brand-surface",
            "Brand / Surface",
            [DesignPaint::solid(DesignColor::rgb(0x0d, 0x99, 0xff))
                .with_id("style-brand-surface-solid")],
        ),
        DesignPaintStyle::new(
            "paint-style-hero-gradient",
            "Marketing / Hero gradient",
            [
                DesignPaint::gradient(
                    DesignPaintKind::LinearGradient,
                    vec![
                        DesignGradientStop::new(0., DesignColor::PURPLE)
                            .with_id("style-hero-gradient-start"),
                        DesignGradientStop::new(1., DesignColor::BLUE)
                            .with_id("style-hero-gradient-end"),
                    ],
                )
                .with_id("style-hero-gradient"),
                DesignPaint::solid(DesignColor::rgba(0xff, 0xff, 0xff, 0x24))
                    .with_id("style-hero-highlight"),
            ],
        ),
    ];
    let libraries = [DesignPaintStyleLibrary::new(
        "paint-library-foundations",
        "Fanta foundations",
        [
            DesignPaintStyle::new(
                "paint-style-library-ink",
                "Ink / Primary",
                [DesignPaint::solid(DesignColor::rgb(0x1e, 0x1e, 0x1e))
                    .with_id("style-library-ink-solid")],
            ),
            DesignPaintStyle::new(
                "paint-style-library-campaign",
                "Campaign / Aurora",
                [DesignPaint::gradient(
                    DesignPaintKind::RadialGradient,
                    vec![
                        DesignGradientStop::new(0., DesignColor::rgb(0xff, 0xc7, 0x00))
                            .with_id("style-campaign-start"),
                        DesignGradientStop::new(1., DesignColor::rgb(0xd7, 0x32, 0xa8))
                            .with_id("style-campaign-end"),
                    ],
                )
                .with_id("style-campaign-gradient")],
            )
            .with_import_state(DesignPaintStyleImportState::Available),
        ],
    )];
    DesignPaintStyleViewData::new(page_styles, libraries)
}

pub(crate) fn seed_design_paint_variables() -> DesignPaintVariableViewData {
    let page = DesignVariableSource::page("page-5", "Page 5");
    let library = DesignVariableSource::library("paint-library-foundations", "Fanta foundations");
    DesignPaintVariableViewData::new([
        DesignVariable::page(
            "color-brand-primary",
            "Brand / Primary",
            "semantic-colors",
            "Semantic colors",
            DesignVariableResolvedType::Color,
        )
        .with_source(page)
        .with_scopes([DesignVariableScope::AllFills])
        .with_resolved_value(DesignVariableResolvedValue::Color(DesignColor::rgb(
            0x0d, 0x99, 0xff,
        ))),
        DesignVariable::page(
            "color-foundation-green",
            "Green / 500",
            "primitive-colors",
            "Primitive colors",
            DesignVariableResolvedType::Color,
        )
        .with_source(library.clone())
        .with_scopes([
            DesignVariableScope::AllFills,
            DesignVariableScope::StrokeColor,
        ])
        .with_import_state(DesignVariableImportState::Imported)
        .with_resolved_value(DesignVariableResolvedValue::Color(DesignColor::rgb(
            0x14, 0xae, 0x5c,
        ))),
        DesignVariable::page(
            "color-foundation-red",
            "Red / 500",
            "primitive-colors",
            "Primitive colors",
            DesignVariableResolvedType::Color,
        )
        .with_source(library)
        .with_scopes([
            DesignVariableScope::AllFills,
            DesignVariableScope::StrokeColor,
        ])
        .with_import_state(DesignVariableImportState::Available)
        .with_resolved_value(DesignVariableResolvedValue::Color(DesignColor::rgb(
            0xf2, 0x48, 0x22,
        ))),
    ])
}

pub(crate) fn seed_design_typography_styles() -> DesignTypographyStyleViewData {
    let page_styles = [
        DesignTypographyStyle::new("page-display-hero", "Display / Hero", "Inter", "Bold", 64.),
        DesignTypographyStyle::new(
            "page-heading-section",
            "Heading / Section",
            "Inter",
            "Semi Bold",
            32.,
        ),
        DesignTypographyStyle::new(
            "page-body-default",
            "Body / Default",
            "Inter",
            "Regular",
            16.,
        ),
        DesignTypographyStyle::new("page-label-small", "Label / Small", "Inter", "Medium", 12.),
    ];
    let libraries = [
        DesignTypographyStyleLibrary::new(
            "fanta-type",
            "Fanta type",
            [
                DesignTypographyStyle::new(
                    "fanta-title-large",
                    "Title / Large",
                    "Inter",
                    "Bold",
                    40.,
                ),
                DesignTypographyStyle::new(
                    "fanta-body-compact",
                    "Body / Compact",
                    "Inter",
                    "Regular",
                    14.,
                ),
                DesignTypographyStyle::new(
                    "fanta-code",
                    "Code / Default",
                    "Roboto Mono",
                    "Regular",
                    13.,
                ),
            ],
        ),
        DesignTypographyStyleLibrary::new(
            "marketing-type",
            "Marketing type",
            [
                DesignTypographyStyle::new(
                    "marketing-campaign",
                    "Campaign / Display",
                    "Archivo",
                    "Black",
                    72.,
                ),
                DesignTypographyStyle::new(
                    "marketing-deck-title",
                    "Deck / Title",
                    "Archivo",
                    "Semi Bold",
                    36.,
                ),
                DesignTypographyStyle::new(
                    "marketing-caption",
                    "Caption / Editorial",
                    "Source Serif 4",
                    "Italic",
                    13.,
                ),
            ],
        ),
    ];
    DesignTypographyStyleViewData::new(page_styles, libraries)
}

pub(crate) fn seed_design_fonts() -> DesignFontViewData {
    let unavailable_style = |id: &'static str,
                             name: &'static str,
                             availability: DesignFontAvailability|
     -> DesignFontStyle {
        DesignFontStyle {
            id: id.into(),
            name: name.into(),
            weight: None,
            italic: false,
            availability,
            preview: Some("The quick brown fox · 0123456789".into()),
        }
    };
    let families = [
        DesignFontFamily::local(
            "inter",
            "Inter",
            [
                DesignFontStyle {
                    weight: Some(400),
                    preview: Some("Inter Regular · Aa Bb 0123".into()),
                    ..DesignFontStyle::imported("regular", "Regular")
                },
                DesignFontStyle {
                    weight: Some(500),
                    preview: Some("Inter Medium · Aa Bb 0123".into()),
                    ..DesignFontStyle::imported("medium", "Medium")
                },
                DesignFontStyle {
                    weight: Some(700),
                    preview: Some("Inter Bold · Aa Bb 0123".into()),
                    ..DesignFontStyle::imported("bold", "Bold")
                },
            ],
        ),
        DesignFontFamily {
            id: "archivo".into(),
            name: "Archivo".into(),
            source: DesignFontSource::Library {
                library_id: "marketing-type".into(),
                library_name: "Marketing type".into(),
            },
            styles: vec![
                unavailable_style("semi-bold", "Semi Bold", DesignFontAvailability::Available),
                unavailable_style("black", "Black", DesignFontAvailability::Available),
            ],
        },
        DesignFontFamily::local(
            "retired-brand",
            "Retired Brand Sans",
            [
                unavailable_style(
                    "regular",
                    "Regular",
                    DesignFontAvailability::Missing {
                        reason: "Font file is not installed".into(),
                    },
                ),
                unavailable_style(
                    "restricted",
                    "Restricted",
                    DesignFontAvailability::Unavailable {
                        reason: "Your organization has not enabled this font".into(),
                    },
                ),
            ],
        ),
    ];
    match std::env::var("FANTA_FONT_CATALOG_STATE")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "loading" => DesignFontViewData::default(),
        "unavailable" => DesignFontViewData {
            state: fanta_gpui::prelude::DesignFontCatalogState::Unavailable {
                reason: "The Storybook host denied font enumeration".into(),
            },
            families: Vec::new(),
        },
        _ => DesignFontViewData::ready(families),
    }
}

pub(crate) fn seed_design_effect_styles() -> DesignEffectStyleViewData {
    DesignEffectStyleViewData::new(
        [
            DesignEffectStyle::new(
                "effect-page-elevation-1",
                "Elevation / 1",
                [DesignEffectKind::DropShadow],
            ),
            DesignEffectStyle::new(
                "effect-page-frosted",
                "Surface / Frosted",
                [
                    DesignEffectKind::BackgroundBlur,
                    DesignEffectKind::DropShadow,
                ],
            ),
        ],
        [DesignEffectStyleLibrary::new(
            "fanta-effects",
            "Fanta effects",
            [
                DesignEffectStyle::new(
                    "effect-library-overlay",
                    "Overlay / Raised",
                    [DesignEffectKind::DropShadow, DesignEffectKind::InnerShadow],
                ),
                DesignEffectStyle::new(
                    "effect-library-glass",
                    "Material / Glass",
                    [DesignEffectKind::Glass],
                ),
            ],
        )],
    )
}

pub(crate) fn seed_design_layout_grid_styles() -> DesignLayoutGridStyleViewData {
    let guide_color = DesignColor::rgb(0xf2, 0x48, 0x22);
    DesignLayoutGridStyleViewData::new(
        [
            DesignLayoutGridStyle::new(
                "grid-style-8pt",
                "Foundations / 8 pt grid",
                [DesignLayoutGrid::uniform(8., guide_color)
                    .with_opacity(18.)
                    .with_id("style-8pt-grid")],
            ),
            DesignLayoutGridStyle::new(
                "grid-style-columns",
                "Responsive / Desktop columns",
                [DesignLayoutGrid::columns(
                    DesignColumnLayoutGrid {
                        count: DesignLayoutGridCount::number(12),
                        gutter: 24.,
                        margin: 32.,
                        ..DesignColumnLayoutGrid::default()
                    },
                    guide_color,
                )
                .with_opacity(12.)
                .with_id("style-desktop-columns")],
            ),
            DesignLayoutGridStyle::new(
                "grid-style-rows",
                "Responsive / Desktop rows",
                [DesignLayoutGrid::rows(
                    DesignRowLayoutGrid {
                        count: DesignLayoutGridCount::number(8),
                        gutter: 16.,
                        margin: 24.,
                        ..DesignRowLayoutGrid::default()
                    },
                    DesignColor::BLUE,
                )
                .with_opacity(12.)
                .with_id("style-desktop-rows")],
            ),
        ],
        [DesignLayoutGridStyleLibrary::new(
            "fanta-grid-library",
            "Fanta layout grids",
            [
                DesignLayoutGridStyle::new(
                    "grid-style-locked",
                    "Library / Locked desktop grid",
                    [
                        DesignLayoutGrid::columns(DesignColumnLayoutGrid::default(), guide_color)
                            .with_id("style-locked-columns"),
                    ],
                ),
                DesignLayoutGridStyle::new(
                    "grid-style-editorial",
                    "Editorial / Baseline + columns",
                    [
                        DesignLayoutGrid::uniform(4., DesignColor::BLUE)
                            .with_opacity(8.)
                            .with_id("style-editorial-baseline"),
                        DesignLayoutGrid::columns(
                            DesignColumnLayoutGrid {
                                count: DesignLayoutGridCount::number(6),
                                gutter: 24.,
                                margin: 40.,
                                ..DesignColumnLayoutGrid::default()
                            },
                            guide_color,
                        )
                        .with_opacity(12.)
                        .with_id("style-editorial-columns"),
                    ],
                )
                .available(),
            ],
        )],
    )
}

pub(crate) fn seed_design_layout_grid_variables() -> DesignLayoutGridVariableViewData {
    DesignLayoutGridVariableViewData::new([
        DesignVariable::page(
            "desktop-column-count",
            "Desktop columns",
            "responsive",
            "Responsive",
            DesignVariableResolvedType::Float,
        )
        .with_resolved_value(DesignVariableResolvedValue::Float(12.)),
        DesignVariable::page(
            "desktop-row-count",
            "Desktop rows",
            "responsive",
            "Responsive",
            DesignVariableResolvedType::Float,
        )
        .with_resolved_value(DesignVariableResolvedValue::Float(8.)),
        DesignVariable::page(
            "auto-track-count",
            "Auto tracks",
            "responsive",
            "Responsive",
            DesignVariableResolvedType::Float,
        )
        .with_resolved_value(DesignVariableResolvedValue::Float(f32::INFINITY)),
        DesignVariable::page(
            "spacing-8",
            "Spacing / 8",
            "spacing",
            "Spacing",
            DesignVariableResolvedType::Float,
        )
        .with_resolved_value(DesignVariableResolvedValue::Float(8.)),
        DesignVariable::page(
            "grid-gutter-24",
            "Grid gutter / 24",
            "spacing",
            "Spacing",
            DesignVariableResolvedType::Float,
        )
        .with_source(DesignVariableSource::library(
            "layout-library",
            "Layout foundations",
        ))
        .with_import_state(DesignVariableImportState::Imported)
        .with_resolved_value(DesignVariableResolvedValue::Float(24.)),
        DesignVariable::page(
            "grid-section-64",
            "Grid section / 64",
            "spacing",
            "Spacing",
            DesignVariableResolvedType::Float,
        )
        .with_source(DesignVariableSource::library(
            "layout-library",
            "Layout foundations",
        ))
        .with_import_state(DesignVariableImportState::Available)
        .with_resolved_value(DesignVariableResolvedValue::Float(64.)),
        DesignVariable::page(
            "restricted-grid-number",
            "Restricted grid number",
            "library-variables",
            "Library variables",
            DesignVariableResolvedType::Float,
        )
        .with_resolved_value(DesignVariableResolvedValue::Float(8.))
        .disabled("Unavailable in the active variable mode"),
    ])
    .with_create_state(DesignLayoutGridVariableCreateState::Enabled)
}

pub(crate) fn seed_design_frame_presets(
    nodes: &[DesignPanelNode],
) -> HashMap<SharedString, DesignFramePresetViewData> {
    let groups = vec![
        DesignFramePresetGroup::new(
            "phone",
            "Phone",
            [
                DesignFramePreset::new("iphone-16-pro", "iPhone 16 Pro", 402., 874.),
                DesignFramePreset::new("iphone-14-15-pro", "iPhone 14 & 15 Pro", 393., 852.),
                DesignFramePreset::new("android-small", "Android Small", 360., 800.)
                    .disabled("Disabled by the Storybook host"),
            ],
        ),
        DesignFramePresetGroup::new(
            "tablet",
            "Tablet",
            [
                DesignFramePreset::new("ipad-pro-11", "iPad Pro 11″", 834., 1194.),
                DesignFramePreset::new("surface-pro-8", "Surface Pro 8", 1440., 960.),
            ],
        ),
        DesignFramePresetGroup::new(
            "desktop",
            "Desktop",
            [
                DesignFramePreset::new("desktop", "Desktop", 1440., 1024.),
                DesignFramePreset::new("macbook-air", "MacBook Air", 1280., 832.),
            ],
        ),
        DesignFramePresetGroup::new(
            "presentation",
            "Presentation",
            [
                DesignFramePreset::new("slide-16-9", "Slide 16:9", 1920., 1080.),
                DesignFramePreset::new("slide-4-3", "Slide 4:3", 1024., 768.),
            ],
        ),
        DesignFramePresetGroup::new(
            "watch",
            "Watch",
            [DesignFramePreset::new(
                "apple-watch-45",
                "Apple Watch 45 mm",
                396.,
                484.,
            )],
        ),
        DesignFramePresetGroup::new(
            "paper",
            "Paper",
            [
                DesignFramePreset::new("a4", "A4", 595., 842.),
                DesignFramePreset::new("us-letter", "US Letter", 612., 792.),
            ],
        ),
        DesignFramePresetGroup::new(
            "social-media",
            "Social Media",
            [
                DesignFramePreset::new("instagram-post", "Instagram Post", 1080., 1080.),
                DesignFramePreset::new("youtube", "YouTube", 1280., 720.),
            ],
        ),
        DesignFramePresetGroup::new(
            "figma-community",
            "Figma Community",
            [DesignFramePreset::new(
                "community-cover",
                "Community cover",
                1920.,
                960.,
            )],
        )
        .disabled("Connect a Community catalog to use these presets"),
        DesignFramePresetGroup::new(
            "archive",
            "Archive",
            [
                DesignFramePreset::new("iphone-8", "iPhone 8", 375., 667.),
                DesignFramePreset::new("ipad-mini", "iPad mini", 768., 1024.),
            ],
        ),
    ];

    nodes
        .iter()
        .filter(|node| node.kind == DesignPanelNodeKind::Frame)
        .map(|node| {
            (
                node.id.clone(),
                DesignFramePresetViewData::new(node.id.clone(), groups.clone()),
            )
        })
        .collect()
}

pub(crate) fn seed_design_effect_variables() -> DesignEffectVariableViewData {
    DesignEffectVariableViewData::new([
        DesignEffectVariable::new(
            "effect-radius-md",
            "Radius / Medium",
            "Effects",
            DesignEffectVariableKind::Float,
        ),
        DesignEffectVariable::new(
            "effect-offset-sm",
            "Offset / Small",
            "Effects",
            DesignEffectVariableKind::Float,
        ),
        DesignEffectVariable::new(
            "effect-shadow-color",
            "Shadow / Default",
            "Semantic colors",
            DesignEffectVariableKind::Color,
        )
        .remote(true),
    ])
}

pub(crate) fn seed_design_property_variables() -> DesignVariableViewData {
    let page = DesignVariableSource::page("page-5", "Page 5");
    let library = DesignVariableSource::library("fanta-foundations", "Fanta foundations");
    DesignVariableViewData::new([
        DesignVariable::page(
            "size-card",
            "Card",
            "layout-size",
            "Layout / Size",
            DesignVariableResolvedType::Float,
        )
        .with_source(page.clone())
        .with_scopes([DesignVariableScope::WidthHeight])
        .with_resolved_value(DesignVariableResolvedValue::Float(320.))
        .with_search(
            DesignVariableSearchMetadata::new(
                ["layout", "dimensions"],
                ["width", "height", "card"],
            )
            .described("Default card width and height"),
        ),
        DesignVariable::page(
            "space-200",
            "200",
            "spacing",
            "Foundations / Spacing",
            DesignVariableResolvedType::Float,
        )
        .with_source(page.clone())
        .with_scopes([DesignVariableScope::Gap])
        .with_resolved_value(DesignVariableResolvedValue::Float(8.))
        .with_search(DesignVariableSearchMetadata::new(
            ["foundations", "spacing"],
            ["gap", "padding", "8"],
        )),
        DesignVariable::page(
            "surface-opacity",
            "Surface",
            "appearance",
            "Appearance",
            DesignVariableResolvedType::Float,
        )
        .with_source(page.clone())
        .with_scopes([DesignVariableScope::Opacity])
        .with_resolved_value(DesignVariableResolvedValue::Float(88.))
        .with_search(DesignVariableSearchMetadata::new(
            ["appearance"],
            ["opacity", "surface"],
        )),
        DesignVariable::page(
            "layer-visible",
            "Layer visible",
            "behavior",
            "Behavior",
            DesignVariableResolvedType::Boolean,
        )
        .with_source(page.clone())
        .with_resolved_value(DesignVariableResolvedValue::Boolean(true))
        .with_search(DesignVariableSearchMetadata::new(
            ["behavior"],
            ["visible", "visibility"],
        )),
        DesignVariable::page(
            "font-body-family",
            "Body family",
            "typography",
            "Typography",
            DesignVariableResolvedType::String,
        )
        .with_source(library.clone())
        .with_scopes([DesignVariableScope::FontFamily])
        .with_import_state(DesignVariableImportState::Imported)
        .with_resolved_value(DesignVariableResolvedValue::String("Inter".into()))
        .with_search(DesignVariableSearchMetadata::new(
            ["typography", "body"],
            ["font", "family", "inter"],
        )),
        DesignVariable::page(
            "font-body-style",
            "Body style",
            "typography",
            "Typography",
            DesignVariableResolvedType::String,
        )
        .with_source(page.clone())
        .with_scopes([DesignVariableScope::FontStyle])
        .with_resolved_value(DesignVariableResolvedValue::String("Medium".into()))
        .with_search(DesignVariableSearchMetadata::new(
            ["typography", "body"],
            ["font", "style", "medium"],
        )),
        DesignVariable::page(
            "font-body-weight",
            "Body weight",
            "typography",
            "Typography",
            DesignVariableResolvedType::Float,
        )
        .with_source(page.clone())
        .with_scopes([DesignVariableScope::FontWeight])
        .with_resolved_value(DesignVariableResolvedValue::Float(500.))
        .with_search(DesignVariableSearchMetadata::new(
            ["typography", "body"],
            ["font", "weight", "500"],
        )),
        DesignVariable::page(
            "font-display-weight",
            "Display weight",
            "typography",
            "Typography",
            DesignVariableResolvedType::Float,
        )
        .with_source(library.clone())
        .with_scopes([DesignVariableScope::FontWeight])
        .with_import_state(DesignVariableImportState::Available)
        .with_resolved_value(DesignVariableResolvedValue::Float(700.))
        .with_search(DesignVariableSearchMetadata::new(
            ["typography", "display"],
            ["font", "weight", "bold", "700"],
        )),
        DesignVariable::page(
            "component-label-campaign",
            "Campaign label",
            "component-copy",
            "Components / Copy",
            DesignVariableResolvedType::String,
        )
        .with_source(library.clone())
        .with_import_state(DesignVariableImportState::Available)
        .with_resolved_value(DesignVariableResolvedValue::String(
            "Start free trial\nNo credit card required".into(),
        ))
        .with_search(DesignVariableSearchMetadata::new(
            ["components", "copy"],
            ["label", "button", "campaign"],
        )),
        DesignVariable::page(
            "radius-container",
            "Container",
            "radius",
            "Foundations / Radius",
            DesignVariableResolvedType::Float,
        )
        .with_source(library)
        .with_scopes([DesignVariableScope::CornerRadius])
        .with_import_state(DesignVariableImportState::Available)
        .with_resolved_value(DesignVariableResolvedValue::Float(16.))
        .with_search(DesignVariableSearchMetadata::new(
            ["foundations", "radius"],
            ["corner", "container", "16"],
        )),
        DesignVariable::page(
            "brand-accent",
            "Accent",
            "semantic-colors",
            "Semantic / Color",
            DesignVariableResolvedType::Color,
        )
        .with_source(page)
        .with_scopes([DesignVariableScope::AllFills])
        .with_resolved_value(DesignVariableResolvedValue::Color(DesignColor::BLUE)),
    ])
}

pub(crate) fn seed_design_component_swaps() -> DesignComponentSwapViewData {
    DesignComponentSwapViewData::new([
        DesignComponentSwapCandidate::local(
            DesignComponentReference::local("icon-arrow-right", "Arrow right"),
            "local-key-arrow-right",
            "page-5",
            "Page 5",
        )
        .with_search(DesignComponentSearchMetadata::new(
            ["icons", "arrows"],
            ["arrow", "next", "direction"],
        )),
        DesignComponentSwapCandidate::local(
            DesignComponentReference::local("icon-plus", "Plus"),
            "local-key-plus",
            "page-5",
            "Page 5",
        )
        .with_search(DesignComponentSearchMetadata::new(
            ["icons", "actions"],
            ["plus", "add", "create"],
        )),
        DesignComponentSwapCandidate::local(
            DesignComponentReference::local("icon-status-set", "Status icons"),
            "local-key-status-set",
            "page-5",
            "Page 5",
        )
        .with_asset_kind(DesignComponentAssetKind::ComponentSet)
        .with_search(DesignComponentSearchMetadata::new(
            ["icons", "sets"],
            ["status", "check", "warning"],
        )),
        DesignComponentSwapCandidate::library(
            DesignComponentReference::remote("icon-check", "Check", "Product foundations"),
            "library-key-icon-check",
            "product-foundations",
            "Product foundations",
        )
        .with_import_state(DesignComponentImportState::Imported)
        .with_search(DesignComponentSearchMetadata::new(
            ["icons", "actions"],
            ["check", "success", "complete"],
        )),
        DesignComponentSwapCandidate::library(
            DesignComponentReference::remote("icon-sparkle", "Sparkle", "Marketing components"),
            "library-key-icon-sparkle",
            "marketing-components",
            "Marketing components",
        )
        .with_search(
            DesignComponentSearchMetadata::new(["icons", "decorative"], ["sparkle", "ai", "magic"])
                .described("Available from the Marketing components library"),
        ),
        DesignComponentSwapCandidate::library(
            DesignComponentReference::remote("icon-legacy", "Legacy icon", "Archived components")
                .with_availability(DesignComponentAvailability::Unavailable {
                    reason: "Library access is required".into(),
                }),
            "library-key-icon-legacy",
            "archived-components",
            "Archived components",
        )
        .disabled("Library access is required"),
    ])
}

pub(crate) fn seed_design_media_paint_views(
    nodes: &[DesignPanelNode],
) -> HashMap<SharedString, DesignMediaPaintViewData> {
    nodes
        .iter()
        .filter_map(|node| {
            let paints = node
                .fills
                .iter()
                .enumerate()
                .filter_map(|(index, paint)| match &paint.payload {
                    DesignPaintPayload::Image(image) => {
                        let transform = image.placement.crop_transform().unwrap_or_default();
                        let capabilities = match paint.id.as_ref() {
                            "image-1-fill" => DesignMediaPaintCapabilities::editor()
                                .with_source_actions(true, false, true)
                                .with_accepted_drop_file_kinds(
                                    DesignMediaFileKinds::STANDARD | DesignMediaFileKinds::TIFF,
                                ),
                            "image-2-fill" => DesignMediaPaintCapabilities::property_editor_only(),
                            "image-3-fill" => DesignMediaPaintCapabilities::editor()
                                .with_source_actions(false, true, false),
                            _ => DesignMediaPaintCapabilities::editor(),
                        };
                        Some(
                            DesignMediaPaintView::new(
                                DesignPanelCollection::Fill,
                                paint.id.clone(),
                                index,
                            )
                            .with_capabilities(capabilities)
                            .with_crop_tool(DesignMediaCropToolState {
                                active: paint.id.as_ref() == "image-2-fill",
                                transform,
                                zoom: 1.25,
                                aspect_ratio: DesignMediaCropAspectRatio::Original,
                            }),
                        )
                    }
                    DesignPaintPayload::Video(video) => {
                        let transform = video.placement.crop_transform().unwrap_or_default();
                        let preview = match paint.id.as_ref() {
                            "video-0-fill" => DesignVideoPreviewState::ready(18., 3.5, false),
                            "video-1-fill" => DesignVideoPreviewState::ready(31., 12., true),
                            "video-2-fill" => DesignVideoPreviewState::loading(),
                            _ => DesignVideoPreviewState::error(
                                "The host could not decode this preview",
                            ),
                        };
                        let capabilities = if paint.id.as_ref() == "video-0-fill" {
                            DesignMediaPaintCapabilities::property_editor_only()
                        } else {
                            DesignMediaPaintCapabilities::editor()
                        };
                        Some(
                            DesignMediaPaintView::new(
                                DesignPanelCollection::Fill,
                                paint.id.clone(),
                                index,
                            )
                            .with_capabilities(capabilities)
                            .with_crop_tool(DesignMediaCropToolState {
                                active: false,
                                transform,
                                zoom: 1.,
                                aspect_ratio: DesignMediaCropAspectRatio::Free,
                            })
                            .with_video_preview(preview),
                        )
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            (!paints.is_empty()).then(|| (node.id.clone(), DesignMediaPaintViewData::new(paints)))
        })
        .collect()
}

pub(crate) fn seed_design_export_configurations(
    nodes: &[DesignPanelNode],
) -> HashMap<SharedString, Vec<DesignExportConfiguration>> {
    let mut configurations = nodes
        .iter()
        .map(|node| {
            let rows = if node.export_settings.is_empty() {
                Vec::new()
            } else {
                node.export_settings
                    .iter()
                    .enumerate()
                    .map(|(index, setting)| {
                        DesignExportConfiguration::from_legacy(
                            format!("{}-export-{index}", node.id),
                            setting,
                        )
                    })
                    .collect()
            };
            (node.id.clone(), rows)
        })
        .collect::<HashMap<_, _>>();

    if let Some(slice) = nodes
        .iter()
        .find(|node| node.kind == DesignPanelNodeKind::Slice)
    {
        let mut scale = DesignExportConfiguration::new("slice-png-scale", DesignExportFormat::Png);
        scale.sizing = DesignExportSizing::Scale(2.);
        scale.common.suffix = "@2x".into();

        let mut height =
            DesignExportConfiguration::new("slice-png-height", DesignExportFormat::Png);
        height.sizing = DesignExportSizing::Height(800.);
        height.common.suffix = "-800h".into();

        let mut width = DesignExportConfiguration::new("slice-jpg-width", DesignExportFormat::Jpg);
        width.sizing = DesignExportSizing::Width(1440.);
        width.common.suffix = "-1440w".into();

        let mut svg = DesignExportConfiguration::new("slice-svg", DesignExportFormat::Svg);
        svg.common.suffix = "-vector".into();
        let mut pdf = DesignExportConfiguration::new("slice-pdf", DesignExportFormat::Pdf);
        pdf.common.suffix = "-print".into();
        configurations.insert(slice.id.clone(), vec![scale, height, width, svg, pdf]);
    }

    for reference_id in ["reference-text", "reference-arrow", "reference-frame"] {
        if let Some(node) = nodes.iter().find(|node| node.id == reference_id) {
            let format = if reference_id == "reference-frame" {
                DesignExportFormat::Png
            } else {
                DesignExportFormat::Svg
            };
            configurations.insert(
                node.id.clone(),
                vec![DesignExportConfiguration::new_for_capabilities(
                    format!("{reference_id}-export"),
                    format,
                    static_export_capabilities(node),
                )],
            );
        }
    }

    configurations.insert(
        "storybook-page".into(),
        vec![DesignExportConfiguration::new(
            "page-export-pdf",
            DesignExportFormat::Pdf,
        )],
    );
    configurations
}

pub(crate) fn static_export_capabilities(node: &DesignPanelNode) -> DesignStaticExportCapabilities {
    story_static_export_capabilities(node)
}

pub(crate) fn seed_design_animated_exports(
    nodes: &[DesignPanelNode],
) -> HashMap<SharedString, DesignAnimatedExportViewData> {
    nodes
        .iter()
        .map(|node| {
            let (capability, settings) = match node.id.as_ref() {
                "reference-frame" => (
                    DesignAnimatedExportCapability::eligible(1920, 1080),
                    DesignAnimatedExportSettings::default(),
                ),
                "layout-horizontal" => {
                    let mut capability = DesignAnimatedExportCapability::eligible(1920, 1080);
                    capability.high_resolution_allowed = false;
                    capability.high_resolution_reason =
                        Some("Upgrade to export above 1080p or 30 FPS".into());
                    (
                        capability,
                        DesignAnimatedExportSettings::Mp4 {
                            sizing: DesignExportSizing::Scale(1.),
                            fps: DesignVideoExportFps::Fps60,
                            quality: fanta_gpui::prelude::DesignExportImageQuality::High,
                        },
                    )
                }
                "layout-vertical" => (
                    DesignAnimatedExportCapability::disabled(
                        "Video export requires a top-level animated frame",
                    ),
                    DesignAnimatedExportSettings::for_format(DesignAnimatedExportFormat::WebM),
                ),
                "reference-rectangle" => (
                    DesignAnimatedExportCapability::disabled(
                        "Add motion to a top-level frame to export animation",
                    ),
                    DesignAnimatedExportSettings::for_format(DesignAnimatedExportFormat::Gif),
                ),
                "reference-arrow" => (
                    DesignAnimatedExportCapability::disabled(
                        "Only top-level animated frames can be exported",
                    ),
                    DesignAnimatedExportSettings::Svg {
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
                    },
                ),
                _ => (
                    DesignAnimatedExportCapability::disabled(
                        "Add motion to a top-level frame to export animation",
                    ),
                    DesignAnimatedExportSettings::default(),
                ),
            };
            (
                node.id.clone(),
                DesignAnimatedExportViewData::new(capability, settings),
            )
        })
        .collect()
}

pub(crate) fn seed_design_export_previews(
    nodes: &[DesignPanelNode],
) -> HashMap<SharedString, DesignExportPreviewState> {
    nodes
        .iter()
        .map(|node| {
            let state = match node.id.as_ref() {
                "reference-frame" => DesignExportPreviewState::Ready(
                    DesignExportPreview::new(1920, 1080)
                        .with_thumbnail("storybook-frame-preview")
                        .with_estimated_output("1.8 MB"),
                ),
                "reference-rectangle" => DesignExportPreviewState::Loading,
                "reference-text" => DesignExportPreviewState::Error {
                    message: "Preview could not be generated".into(),
                },
                _ => DesignExportPreviewState::Idle,
            };
            (node.id.clone(), state)
        })
        .collect()
}

pub(crate) fn story_homogeneous_multiple_nodes() -> [DesignPanelNode; 2] {
    let make_fills = |prefix: &str| {
        let mut accent =
            DesignPaint::solid(DesignColor::PURPLE).with_id(format!("{prefix}-fill-accent"));
        accent.opacity = 35.;
        vec![
            DesignPaint::solid(DesignColor::BLUE).with_id(format!("{prefix}-fill-surface")),
            accent,
        ]
    };
    let make_stroke = |prefix: &str| {
        let mut paint =
            DesignPaint::solid(DesignColor::BLACK).with_id(format!("{prefix}-stroke-outline"));
        paint.opacity = 80.;
        DesignStroke::for_node(
            DesignPanelNodeKind::Rectangle,
            paint,
            3.,
            DesignStrokeAlign::Inside,
        )
    };

    let mut first = DesignPanelNode::new(
        STORY_HOMOGENEOUS_MULTIPLE_NODE_IDS[0],
        "Common paints · Card A",
        DesignPanelNodeKind::Rectangle,
    );
    first.x = 48.;
    first.y = 64.;
    first.width = 240.;
    first.corner_radii = [16.; 4];
    first.fills = make_fills("homogeneous-first");
    first.stroke = Some(make_stroke("homogeneous-first"));
    first.effects.push(
        DesignEffect::drop_shadow(DesignColor::rgba(0x00, 0x00, 0x00, 0x33), 16., 0., 0., 4.)
            .with_id("homogeneous-first-shadow"),
    );

    let mut second = DesignPanelNode::new(
        STORY_HOMOGENEOUS_MULTIPLE_NODE_IDS[1],
        "Common paints · Card B",
        DesignPanelNodeKind::Rectangle,
    );
    second.x = 336.;
    second.y = 112.;
    second.width = 320.;
    second.corner_radii = [28.; 4];
    second.fills = make_fills("homogeneous-second");
    second.stroke = Some(make_stroke("homogeneous-second"));
    second
        .effects
        .push(DesignEffect::new(DesignEffectKind::LayerBlur).with_id("homogeneous-second-blur"));

    [first, second]
}

/// How an inspection scenario selects its required node in the seeded
/// inventory. `Current` keeps whichever preset is already selected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScenarioNodeSelector {
    Current,
    Id(&'static str),
    Kind(DesignPanelNodeKind),
}

/// The single scenario-to-node table. Scenario activation and the launch
/// environment resolve their required preset through this table so the
/// node inventory has exactly one lookup vocabulary.
pub(crate) fn scenario_node_selector(scenario: DesignInspectionScenario) -> ScenarioNodeSelector {
    match scenario {
        DesignInspectionScenario::AddAutoLayoutGroup => {
            ScenarioNodeSelector::Id("add-auto-layout-group")
        }
        DesignInspectionScenario::AddAutoLayoutMultiple => {
            ScenarioNodeSelector::Kind(DesignPanelNodeKind::Rectangle)
        }
        DesignInspectionScenario::HomogeneousMultiple => {
            ScenarioNodeSelector::Id(STORY_HOMOGENEOUS_MULTIPLE_NODE_IDS[0])
        }
        DesignInspectionScenario::AutoLayoutChild => {
            ScenarioNodeSelector::Id("layout-child-horizontal")
        }
        DesignInspectionScenario::AutoLayoutIgnored => {
            ScenarioNodeSelector::Id("layout-child-absolute")
        }
        DesignInspectionScenario::AutoLayoutVerticalChild => {
            ScenarioNodeSelector::Id("layout-child-vertical")
        }
        DesignInspectionScenario::GridChild => {
            ScenarioNodeSelector::Id("layout-grid-child-placement")
        }
        DesignInspectionScenario::TextEdit | DesignInspectionScenario::PropertyStates => {
            ScenarioNodeSelector::Kind(DesignPanelNodeKind::Text)
        }
        DesignInspectionScenario::VectorEdit => {
            ScenarioNodeSelector::Kind(DesignPanelNodeKind::Vector)
        }
        DesignInspectionScenario::Page
        | DesignInspectionScenario::ViewOnlyPage
        | DesignInspectionScenario::RestrictedPage
        | DesignInspectionScenario::EditableSingle
        | DesignInspectionScenario::CanvasSingle
        | DesignInspectionScenario::EditableMultiple
        | DesignInspectionScenario::ViewOnlyMultiple
        | DesignInspectionScenario::RestrictedMultiple
        | DesignInspectionScenario::SmartSelectionNone
        | DesignInspectionScenario::SmartSelectionHorizontal
        | DesignInspectionScenario::SmartSelectionVertical
        | DesignInspectionScenario::SmartSelectionTwoDimensional
        | DesignInspectionScenario::SmartSelectionReadOnly
        | DesignInspectionScenario::ViewOnlySingle
        | DesignInspectionScenario::RestrictedSingle => ScenarioNodeSelector::Current,
    }
}

pub(crate) fn node_index_for_selector(
    nodes: &[DesignPanelNode],
    selector: ScenarioNodeSelector,
) -> Option<usize> {
    match selector {
        ScenarioNodeSelector::Current => None,
        ScenarioNodeSelector::Id(id) => nodes.iter().position(|node| node.id.as_ref() == id),
        ScenarioNodeSelector::Kind(kind) => nodes.iter().position(|node| node.kind == kind),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_scenario_selector_resolves_in_the_seeded_inventory() {
        let nodes = seed_design_nodes();
        for scenario in DesignInspectionScenario::ALL {
            match scenario_node_selector(scenario) {
                ScenarioNodeSelector::Current => {}
                selector => {
                    assert!(
                        node_index_for_selector(&nodes, selector).is_some(),
                        "{scenario:?} must resolve {selector:?} against the seeded inventory"
                    );
                }
            }
        }
    }
}
