//! The Sections & grids molecule story.
//!
//! The inspector's structural recipes shown as surfaces rather than as a
//! list of functions: a section group whose headers disclose, a field grid
//! whose rows and labels inherit one policy, and the same grouped frame
//! mounted at 320, 400 and 472 px so the responsive behaviour across the
//! inspector's supported range is on screen instead of described. The
//! density and label-placement knobs drive every specimen from the grid,
//! which is the contract: fields never choose their own geometry.

use fanta_gpui::atoms::TypographyExt as _;
use gpui::FocusHandle;

use crate::*;

use super::knobs::{self, KnobOption};
use super::spec::NamedState;
use super::specimen::{
    framed_panel, specimen_card, specimen_row, specimen_rows, specimen_story_root,
};

/// The inspector's supported panel widths, with what each one sits on
/// either side of. The library exposes the breakpoints as metrics; the
/// structural recipes never branch on them, which is exactly what the
/// side-by-side specimens let the reader check.
pub(crate) const STRUCTURE_WIDTHS: [(f32, &str); 3] = [
    (320., "below the 360 px compact breakpoint"),
    (400., "between the two breakpoints"),
    (472., "above the 440 px wide breakpoint"),
];

/// Height of the width-comparison panels: tall enough for three rows at
/// the most generous density and placement.
const WIDTH_PANEL_HEIGHT: f32 = 200.;

/// The disclosure specimens: one section per header state, both live.
pub(crate) const SECTION_SPECIMENS: [(&str, &str); 2] =
    [("position", "Position"), ("appearance", "Appearance")];

/// The disclosure states the story opens on: one header collapsed and one
/// expanded, so both halves of the contract are visible before any click.
pub(crate) const SECTION_SEED: [bool; SECTION_SPECIMENS.len()] = [false, true];

/// Density as a knob axis. The grid owns this choice for every row it
/// contains, so the story keeps it on the screen rather than the field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StructureDensity {
    Compact,
    Comfortable,
}

impl StructureDensity {
    pub(crate) const fn density(self) -> InspectorDensity {
        match self {
            Self::Compact => InspectorDensity::Compact,
            Self::Comfortable => InspectorDensity::Comfortable,
        }
    }
}

impl NamedState for StructureDensity {
    const ALL: &'static [Self] = &[Self::Compact, Self::Comfortable];

    fn label(self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Comfortable => "Comfortable",
        }
    }
}

/// Label placement as a knob axis, the grid's other inherited policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StructureLabelPlacement {
    Leading,
    Stacked,
}

impl StructureLabelPlacement {
    pub(crate) const fn placement(self) -> InspectorLabelPlacement {
        match self {
            Self::Leading => InspectorLabelPlacement::Leading,
            Self::Stacked => InspectorLabelPlacement::Stacked,
        }
    }
}

impl NamedState for StructureLabelPlacement {
    const ALL: &'static [Self] = &[Self::Leading, Self::Stacked];

    fn label(self) -> &'static str {
        match self {
            Self::Leading => "Leading",
            Self::Stacked => "Stacked",
        }
    }
}

pub(crate) struct StructureScreen {
    pub(crate) focus_handle: FocusHandle,
    /// One disclosure state per [`SECTION_SPECIMENS`] entry, seeded so the
    /// reader sees a collapsed and an expanded header at once.
    pub(crate) expanded: [bool; SECTION_SPECIMENS.len()],
    pub(crate) density: StructureDensity,
    pub(crate) label_placement: StructureLabelPlacement,
    pub(crate) last_action: SharedString,
}

impl StructureScreen {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            expanded: SECTION_SEED,
            density: StructureDensity::Compact,
            label_placement: StructureLabelPlacement::Leading,
            last_action: "Ready — toggle a section header, then compare the three widths".into(),
        }
    }

    /// The layout policy every specimen in the story inherits.
    pub(crate) fn layout(&self) -> InspectorGridLayout {
        InspectorGridLayout::new(
            InspectorMetrics::default(),
            self.density.density(),
            self.label_placement.placement(),
        )
    }

    pub(crate) fn toggle_section(&mut self, index: usize, keyboard: bool) {
        self.expanded[index] = !self.expanded[index];
        let (_, title) = SECTION_SPECIMENS[index];
        self.last_action = format!(
            "{} the {title} section via {}",
            if self.expanded[index] {
                "Expanded"
            } else {
                "Collapsed"
            },
            if keyboard { "Enter/Space" } else { "pointer" }
        )
        .into();
    }

    pub(crate) fn set_density(&mut self, density: StructureDensity) {
        self.density = density;
        self.last_action = format!(
            "Density · {} — every row took its height and gap from the grid",
            density.label()
        )
        .into();
    }

    pub(crate) fn set_label_placement(&mut self, placement: StructureLabelPlacement) {
        self.label_placement = placement;
        self.last_action = format!(
            "Label placement · {} — every row changed axis at once",
            placement.label()
        )
        .into();
    }
}

impl Storybook {
    /// One label-plus-control row, both halves following the grid's policy.
    fn render_structure_field_row(
        &self,
        label: &'static str,
        control: AnyElement,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let layout = self.structure_screen.layout();
        let leading = layout.label_placement == InspectorLabelPlacement::Leading;
        inspector_row_with_layout(layout)
            .child(
                inspector_field_label(layout)
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child(label),
            )
            .child(
                div()
                    .min_w(px(0.))
                    .when(leading, |slot| slot.flex_1())
                    .when(!leading, |slot| slot.w_full())
                    .child(control),
            )
            .into_any_element()
    }

    /// A standalone field frame: the seam owner when a field stands alone.
    fn render_structure_field(
        &self,
        selector: String,
        value: &'static str,
        presentation: &InspectorFieldPresentation,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = selector.clone();
        inspector_field_frame_with_presentation(
            SharedString::from(selector),
            presentation,
            InspectorMetrics::default(),
            cx,
        )
        .debug_selector(move || id.clone())
        .w_full()
        .child(div().flex_1().min_w(px(0.)).child(value))
        .into_any_element()
    }

    /// Two axis fields sharing one outline, the way X/Y and W/H are paired.
    fn render_structure_field_group(
        &self,
        key: &str,
        invalid: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let metrics = InspectorMetrics::default();
        let presentation =
            InspectorFieldPresentation::new(InspectorFieldAccess::Editable).invalid(invalid);
        let selector = format!("{key}-group");
        let id = selector.clone();
        let mut group = inspector_field_group(metrics, cx)
            .id(SharedString::from(selector))
            .debug_selector(move || id.clone());
        for (axis, value) in [("X", "0"), ("Y", "-24")] {
            group = group.child(
                inspector_grouped_field_frame(
                    SharedString::from(format!("{key}-group-{axis}")),
                    &presentation,
                    metrics,
                    cx,
                )
                .flex_1()
                .min_w(px(0.))
                .child(
                    div()
                        .flex_none()
                        .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                        .child(axis),
                )
                .child(div().flex_1().min_w(px(0.)).child(value)),
            );
        }
        group.into_any_element()
    }

    /// One disclosure section: the header, and the grid it discloses.
    fn render_structure_section(&self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        let (key, title) = SECTION_SPECIMENS[index];
        let expanded = self.structure_screen.expanded[index];
        let metrics = InspectorMetrics::default();
        let selector = format!("structure-section-{key}-header");
        let id = selector.clone();
        let header = inspector_section_header(SharedString::from(selector), metrics, cx)
            .debug_selector(move || id.clone())
            .on_activate(cx.listener(move |this, event: &ActivateEvent, _, cx| {
                this.structure_screen.toggle_section(index, event.keyboard);
                cx.notify();
            }))
            .child(render_lucide_icon(
                if expanded {
                    LucideIcon::ChevronDown
                } else {
                    LucideIcon::ChevronRight
                },
                fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx),
                14.,
            ))
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .typography(fanta_gpui::atoms::TypographyToken::BodyLarge)
                    .font_semibold()
                    .child(title),
            )
            .child(
                div()
                    .flex_none()
                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child(if expanded { "Expanded" } else { "Collapsed" }),
            );

        inspector_section(cx)
            .child(header)
            .when(expanded, |section| {
                section.child(self.render_structure_grid(&format!("structure-section-{key}"), cx))
            })
            .into_any_element()
    }

    /// The field grid mounted both inside a disclosed section and, on its
    /// own, inside each width specimen: one grouped pair and two
    /// standalone fields, all inheriting the grid's policy.
    fn render_structure_grid(&self, key: &str, cx: &mut Context<Self>) -> AnyElement {
        let metrics = InspectorMetrics::default();
        inspector_field_grid_with_layout(self.structure_screen.layout())
            .pb(metrics.control_gap)
            .child(self.render_structure_field_row(
                "Position",
                self.render_structure_field_group(key, false, cx),
                cx,
            ))
            .child(self.render_structure_field_row(
                "Opacity",
                self.render_structure_field(
                    format!("{key}-opacity"),
                    "100%",
                    &InspectorFieldPresentation::default(),
                    cx,
                ),
                cx,
            ))
            .child(self.render_structure_field_row(
                "Blend",
                self.render_structure_field(
                    format!("{key}-blend"),
                    "Normal",
                    &InspectorFieldPresentation::default(),
                    cx,
                ),
                cx,
            ))
            .into_any_element()
    }

    fn render_structure_sections_card(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut group = inspector_section_group();
        for index in 0..SECTION_SPECIMENS.len() {
            group = group.child(self.render_structure_section(index, cx));
        }
        specimen_card(
            "structure-sections",
            "Section header · collapsed and expanded",
            "Two sections inside one inspector_section_group: the first is collapsed, so its \
             header is all there is, and the second is expanded over the field grid it \
             discloses. Click either header — or Tab to it and press Enter — and it toggles. \
             Watch the seams while you do: each section draws its own bottom separator and the \
             group draws none, so collapsing never removes a line or doubles one.",
            specimen_rows(vec![specimen_row(
                "structure-sections-row",
                "One group, one collapsed header and one expanded",
                vec![framed_panel(320., 280., group.into_any_element(), cx)],
                cx,
            )]),
            cx,
        )
    }

    fn render_structure_fields_card(&self, cx: &mut Context<Self>) -> AnyElement {
        let read_only = InspectorFieldPresentation::new(InspectorFieldAccess::read_only(Some(
            SharedString::from("Bound to a token"),
        )));
        let disabled = InspectorFieldPresentation::new(InspectorFieldAccess::disabled(Some(
            SharedString::from("No shared value"),
        )));
        let grid = inspector_field_grid_with_layout(self.structure_screen.layout())
            .py(InspectorMetrics::default().control_gap)
            .child(self.render_structure_field_row(
                "Name",
                self.render_structure_field(
                    "structure-field-editable".to_owned(),
                    "Hero frame",
                    &InspectorFieldPresentation::default(),
                    cx,
                ),
                cx,
            ))
            .child(self.render_structure_field_row(
                "Corner",
                self.render_structure_field(
                    "structure-field-read-only".to_owned(),
                    "8",
                    &read_only,
                    cx,
                ),
                cx,
            ))
            .child(self.render_structure_field_row(
                "Constraint",
                self.render_structure_field(
                    "structure-field-disabled".to_owned(),
                    "Scale",
                    &disabled,
                    cx,
                ),
                cx,
            ))
            .child(self.render_structure_field_row(
                "Position",
                self.render_structure_field_group("structure-field", false, cx),
                cx,
            ))
            .child(self.render_structure_field_row(
                "Rotation",
                self.render_structure_field_group("structure-invalid", true, cx),
                cx,
            ));
        specimen_card(
            "structure-fields",
            "Field grid · rows, labels, and one shared outline",
            "One inspector_field_grid. The first three rows pair an inspector_field_label with \
             a standalone field frame — editable, then read-only, then disabled — and the last \
             two put a pair of children inside an inspector_field_group. The group owns the \
             only outline, so its children announce hover, focus and invalidity with \
             background alone: the invalid pair at the bottom changes colour without moving a \
             single edge. Every row takes its height, gap and axis from the grid, so flipping \
             a knob restyles all five at once and no field opts out.",
            specimen_rows(vec![specimen_row(
                "structure-fields-row",
                "Five rows, one inherited policy",
                vec![framed_panel(400., 320., grid.into_any_element(), cx)],
                cx,
            )]),
            cx,
        )
    }

    fn render_structure_widths_card(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut panels = Vec::with_capacity(STRUCTURE_WIDTHS.len());
        for (width, note) in STRUCTURE_WIDTHS {
            let key = format!("structure-width-{}", width as i32);
            let selector = key.clone();
            let body = div()
                .w_full()
                .pt(InspectorMetrics::default().control_gap)
                .child(self.render_structure_grid(&key, cx))
                .into_any_element();
            panels.push(
                v_flex()
                    .id(SharedString::from(key))
                    .debug_selector(move || selector.clone())
                    .flex_none()
                    .gap_1p5()
                    .child(framed_panel(width, WIDTH_PANEL_HEIGHT, body, cx))
                    .child(
                        div()
                            .w(px(width))
                            .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                            .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                            .child(SharedString::from(format!("{} px · {note}", width as i32))),
                    )
                    .into_any_element(),
            );
        }
        specimen_card(
            "structure-widths",
            "The same grouped frame at 320, 400 and 472 px",
            "The inspector's supported range side by side: three copies of one grid, identical \
             apart from the width they were given. Nothing in these recipes measures the panel, \
             so compare the X/Y pair across the three — the horizontal padding, the 96 px label \
             column and the control gap stay put, every pixel a wider panel gains goes to the \
             controls, and the narrow panel takes it back from them instead of clipping, \
             wrapping or growing a scrollbar. The metrics publish breakpoints that bracket this \
             run — 320 sits under the compact one, 472 over the wide one — and no recipe here \
             branches on either. Switch label placement to Stacked in the knobs and the label \
             column leaves all three at once, handing the controls the full width even at 320.",
            specimen_rows(vec![specimen_row(
                "structure-widths-row",
                "320 · 400 · 472 — the supported inspector range",
                panels,
                cx,
            )]),
            cx,
        )
    }

    pub(crate) fn render_structure_story(&self, cx: &mut Context<Self>) -> AnyElement {
        specimen_story_root("storybook-structure")
            .track_focus(&self.structure_screen.focus_handle)
            .child(self.render_structure_sections_card(cx))
            .child(self.render_structure_fields_card(cx))
            .child(self.render_structure_widths_card(cx))
            .into_any_element()
    }

    pub(crate) fn render_structure_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        let story = self.render_structure_story(cx);
        self.spec_reference("storybook-reference-structure", story, cx)
    }

    pub(crate) fn render_structure_knobs(&self, cx: &mut Context<Self>) -> AnyElement {
        knobs::knobs_panel(
            "structure-story-knobs",
            vec![
                knobs::enum_knob_row(
                    "structure-knob-density",
                    "DENSITY",
                    StructureDensity::ALL
                        .iter()
                        .copied()
                        .map(|density| KnobOption::new(density, density.label())),
                    self.structure_screen.density,
                    |this, density, _, cx| {
                        this.structure_screen.set_density(density);
                        cx.notify();
                    },
                    cx,
                ),
                knobs::enum_knob_row(
                    "structure-knob-label-placement",
                    "LABEL PLACEMENT",
                    StructureLabelPlacement::ALL
                        .iter()
                        .copied()
                        .map(|placement| KnobOption::new(placement, placement.label())),
                    self.structure_screen.label_placement,
                    |this, placement, _, cx| {
                        this.structure_screen.set_label_placement(placement);
                        cx.notify();
                    },
                    cx,
                ),
            ],
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specimen_widths_bracket_the_published_breakpoints() {
        let metrics = InspectorMetrics::default();
        let widths: Vec<f32> = STRUCTURE_WIDTHS.iter().map(|(width, _)| *width).collect();
        assert!(
            widths.windows(2).all(|pair| pair[0] < pair[1]),
            "the width specimens must read narrow to wide: {widths:?}"
        );
        let compact = metrics.compact_breakpoint.as_f32();
        let wide = metrics.wide_breakpoint.as_f32();
        assert!(
            widths[0] < compact,
            "the narrow specimen must sit below the compact breakpoint"
        );
        assert!(
            widths[1] > compact && widths[1] < wide,
            "the middle specimen must sit between the breakpoints"
        );
        assert!(
            widths[2] > wide,
            "the wide specimen must sit above the wide breakpoint"
        );
        assert!(
            widths[0] > metrics.label_width.as_f32() + metrics.horizontal_padding.as_f32() * 2.,
            "even the narrowest specimen must leave room for a control beside the label column"
        );
    }

    #[test]
    fn knob_axes_name_every_case_once() {
        let densities: Vec<&str> = StructureDensity::ALL
            .iter()
            .map(|density| density.label())
            .collect();
        assert_eq!(densities, ["Compact", "Comfortable"]);
        let placements: Vec<&str> = StructureLabelPlacement::ALL
            .iter()
            .map(|placement| placement.label())
            .collect();
        assert_eq!(placements, ["Leading", "Stacked"]);

        let metrics = InspectorMetrics::default();
        let compact = InspectorGridLayout::new(
            metrics,
            StructureDensity::Compact.density(),
            StructureLabelPlacement::Leading.placement(),
        );
        let comfortable = InspectorGridLayout::new(
            metrics,
            StructureDensity::Comfortable.density(),
            StructureLabelPlacement::Stacked.placement(),
        );
        assert!(
            comfortable.row_height() > compact.row_height(),
            "the two density chips must be visibly different specimens"
        );
        assert_ne!(compact.label_placement, comfortable.label_placement);
    }

    #[test]
    fn section_specimens_cover_both_disclosure_states() {
        assert_eq!(SECTION_SPECIMENS.len(), SECTION_SEED.len());
        assert!(
            SECTION_SEED.iter().any(|expanded| *expanded)
                && SECTION_SEED.iter().any(|expanded| !*expanded),
            "the story must open on one collapsed and one expanded header"
        );
        let keys: Vec<&str> = SECTION_SPECIMENS.iter().map(|(key, _)| *key).collect();
        assert_ne!(
            keys[0], keys[1],
            "each section needs its own element ids and debug selectors"
        );
    }
}
