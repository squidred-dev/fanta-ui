//! Mock host for the public paint editor, independent of DesignPanel.
use super::knobs::{self, KnobOption};
use crate::*;

pub(crate) struct PaintPickerStory {
    pub(crate) picker: Entity<PaintPicker>,
    paint: DesignPaint,
    original: Option<DesignPaint>,
    open: bool,
    disabled: bool,
    pub(crate) last_action: SharedString,
}
impl PaintPickerStory {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Storybook>) -> Self {
        let paint = DesignPaint::solid(DesignColor::BLUE).with_id("sample-paint");
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
            picker
        });
        Self {
            picker,
            paint,
            original: None,
            open: true,
            disabled: false,
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
