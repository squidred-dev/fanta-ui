//! Mock host for the public paint editor, independent of DesignPanel.
use super::knobs::{self, KnobOption};
use crate::*;

pub(crate) struct PaintPickerStory {
    pub(crate) picker: Entity<PaintPicker>,
    paint: DesignPaint,
    original: Option<DesignPaint>,
    open: bool,
    disabled: bool,
    shaders: DesignShaderViewData,
    pub(crate) last_action: SharedString,
}
impl PaintPickerStory {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Storybook>) -> Self {
        let paint = DesignPaint::solid(DesignColor::BLUE).with_id("sample-paint");
        let shaders = super::design::fixtures::seed_design_shaders();
        let picker = cx.new(|cx| {
            let mut picker = PaintPicker::new("storybook-paint-picker", window, cx);
            picker.set_target(
                "sample-layer",
                DesignPanelCollection::Fill,
                0,
                paint.clone(),
                window,
                cx,
            );
            picker.set_media_view_data(
                DesignMediaPaintViewData::new([])
                    .with_pattern_sources([
                        DesignPatternSource::new("sample-tile", "Rounded tile"),
                        DesignPatternSource::new("sample-symbol", "Leaf symbol"),
                        DesignPatternSource::new("sample-motif", "Abstract motif"),
                    ])
                    .with_assets(super::design::fixtures::seed_design_media_assets()),
                cx,
            );
            picker.set_shader_view_data(shaders.clone(), cx);
            picker
        });
        Self {
            picker,
            paint,
            original: None,
            open: true,
            disabled: false,
            shaders,
            last_action: "Ready — choose a paint type or edit the color".into(),
        }
    }
    fn sync(&self, window: &mut Window, cx: &mut Context<Storybook>) {
        self.picker.update(cx, |picker, cx| {
            picker.set_target(
                "sample-layer",
                DesignPanelCollection::Fill,
                0,
                self.paint.clone(),
                window,
                cx,
            );
            picker.set_disabled(self.disabled, cx);
        });
    }
    pub(crate) fn handle_action(
        &mut self,
        action: &PaintPickerAction,
        window: &mut Window,
        cx: &mut Context<Storybook>,
    ) {
        match action {
            PaintPickerAction::Edit { edit, phase, .. } => {
                match phase {
                    DesignPanelEditPhase::Begin => self.original = Some(self.paint.clone()),
                    DesignPanelEditPhase::Preview => {
                        self.paint.apply_edit(edit);
                    }
                    DesignPanelEditPhase::Commit => {
                        self.paint.apply_edit(edit);
                        self.original = None;
                    }
                    DesignPanelEditPhase::Cancel => {
                        if let Some(original) = self.original.take() {
                            self.paint = original;
                        }
                    }
                }
                self.sync(window, cx);
            }
            PaintPickerAction::CloseRequested => {
                if let Some(original) = self.original.take() {
                    self.paint = original;
                }
                self.picker
                    .update(cx, |picker, cx| picker.prepare_for_dismissal(cx));
                self.open = false;
            }
            PaintPickerAction::MediaSourceActionRequested { action, .. } => {
                self.paint.apply_edit(&DesignPaintEdit {
                    property: DesignPaintProperty::Source,
                    value: DesignPaintValue::Source(DesignPaintSource::new(
                        format!("sample-{}", action.slug()),
                        match action {
                            DesignMediaSourceAction::Upload => "Uploaded media",
                            DesignMediaSourceAction::MakeImage => "Generated image",
                            DesignMediaSourceAction::EditImage => "Edited image",
                        },
                    )),
                });
                self.sync(window, cx);
            }
            PaintPickerAction::ShaderApplyRequested { shader, .. }
            | PaintPickerAction::ShaderImportRequested { shader, .. } => {
                if let Some(mut definition) = self.shaders.shader(shader).cloned() {
                    definition.imported = true;
                    if let Some(current) = self
                        .shaders
                        .page_shaders
                        .iter_mut()
                        .chain(
                            self.shaders
                                .libraries
                                .iter_mut()
                                .flat_map(|library| &mut library.shaders),
                        )
                        .find(|current| current.id == definition.id)
                    {
                        *current = definition.clone();
                    }
                    if let Some(shader) = DesignShaderPaint::from_definition(&definition) {
                        self.paint.apply_edit(&DesignPaintEdit {
                            property: DesignPaintProperty::Payload,
                            value: DesignPaintValue::Payload(DesignPaintPayload::Shader(shader)),
                        });
                        self.picker.update(cx, |picker, cx| {
                            picker.set_shader_view_data(self.shaders.clone(), cx)
                        });
                        self.sync(window, cx);
                    }
                }
            }
            _ => {}
        }
        self.last_action = format!("{action:?}").into();
        cx.notify();
    }
}
impl Storybook {
    pub(crate) fn render_paint_picker_story(&self, _: &mut Context<Self>) -> AnyElement {
        v_flex()
            .size_full()
            .p_3()
            .items_center()
            .when(self.paint_picker_screen.open, |view| {
                view.child(self.paint_picker_screen.picker.clone())
            })
            .into_any_element()
    }
    pub(crate) fn render_paint_picker_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-paint-picker-reference",
            self.render_paint_picker_story(cx),
            self.paint_picker_screen.last_action.clone(),
            cx,
        )
    }
    pub(crate) fn render_paint_picker_knobs(&self, cx: &mut Context<Self>) -> AnyElement {
        knobs::knobs_panel(
            "paint-picker-knobs",
            vec![
                knobs::enum_knob_row(
                    "paint-fixture",
                    "PAINT",
                    [
                        KnobOption::new(DesignPaintKind::Solid, "Solid"),
                        KnobOption::new(DesignPaintKind::LinearGradient, "Linear"),
                        KnobOption::new(DesignPaintKind::RadialGradient, "Radial"),
                    ],
                    self.paint_picker_screen.paint.kind,
                    |story, kind, window, cx| {
                        let paint = if kind == DesignPaintKind::Solid {
                            DesignPaint::solid(DesignColor::BLUE)
                        } else {
                            DesignPaint::gradient(
                                kind,
                                vec![
                                    DesignGradientStop::new(0., DesignColor::PURPLE),
                                    DesignGradientStop::new(1., DesignColor::BLUE),
                                ],
                            )
                        };
                        story.paint_picker_screen.paint = paint.with_id("sample-paint");
                        story.paint_picker_screen.original = None;
                        story.paint_picker_screen.open = true;
                        story.paint_picker_screen.sync(window, cx);
                    },
                    cx,
                ),
                knobs::bool_knob_row(
                    "paint-open",
                    "PRESENTATION",
                    "Open",
                    self.paint_picker_screen.open,
                    |story, open, window, cx| {
                        if !open {
                            story.paint_picker_screen.handle_action(
                                &PaintPickerAction::CloseRequested,
                                window,
                                cx,
                            );
                        } else {
                            story.paint_picker_screen.open = true;
                            story.paint_picker_screen.sync(window, cx);
                        }
                    },
                    cx,
                ),
                knobs::bool_knob_row(
                    "paint-disabled",
                    "ACCESS",
                    "Read only",
                    self.paint_picker_screen.disabled,
                    |story, disabled, window, cx| {
                        story.paint_picker_screen.disabled = disabled;
                        story.paint_picker_screen.sync(window, cx);
                    },
                    cx,
                ),
            ],
            cx,
        )
    }
}
