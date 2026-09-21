//! Reusable retained color editor shared with the Design inspector.
//!
//! Hosts supply RGBA colors and receive typed edit phases. Draft previews live
//! here until committed or canceled; accepted colors remain host-controlled.
pub(crate) mod paint;

use crate::design::{
    CancelDesignInteraction, DESIGN_PANEL_KEY_CONTEXT, DesignColor, DesignPaint,
    DesignPanelCollection, DesignPanelEditPhase,
};
use gpui::{
    App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString, Subscription,
    Window, div,
};
use paint::{PaintPicker, PaintPickerEvent};

/// Display-space RGBA color. Alpha is an eight-bit channel, like RGB.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PickerColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}
impl PickerColor {
    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
    pub fn hex(self) -> SharedString {
        DesignColor::rgba(self.red, self.green, self.blue, self.alpha).hex()
    }
    fn paint(self) -> DesignPaint {
        let mut paint =
            DesignPaint::solid(DesignColor::rgb(self.red, self.green, self.blue)).with_id("color");
        paint.opacity = f32::from(self.alpha) / 255. * 100.;
        paint
    }
    fn from_paint(paint: &DesignPaint) -> Self {
        Self::rgba(
            paint.color.red,
            paint.color.green,
            paint.color.blue,
            (f32::from(paint.color.alpha) * paint.opacity / 100.)
                .round()
                .clamp(0., 255.) as u8,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorPickerPhase {
    Begin,
    Preview,
    Commit,
    Cancel,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ColorPickerAction {
    Edit {
        color: PickerColor,
        phase: ColorPickerPhase,
    },
    CloseRequested,
    EyedropperRequested,
}

/// Full inline picker surface, including its header, spectrum and color fields.
/// Mount directly in a story or inside the host's anchored popup.
pub struct ColorPicker {
    editor: Entity<PaintPicker>,
    color: PickerColor,
    draft: Option<DesignPaint>,
    editing: bool,
    _subscription: Subscription,
}
impl EventEmitter<ColorPickerAction> for ColorPicker {}
impl ColorPicker {
    pub fn new(
        id: impl Into<SharedString>,
        color: PickerColor,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let editor = cx.new(|cx| {
            let mut editor = PaintPicker::new(id, window, cx);
            editor.set_color_only(Some("Color".into()), cx);
            editor.set_target(
                "color",
                DesignPanelCollection::Fill,
                0,
                color.paint(),
                window,
                cx,
            );
            editor
        });
        let subscription = cx.subscribe_in(
            &editor,
            window,
            |this, _, event: &PaintPickerEvent, window, cx| this.handle_edit(event, window, cx),
        );
        Self {
            editor,
            color,
            draft: None,
            editing: false,
            _subscription: subscription,
        }
    }
    /// Echo the host's accepted color without resetting an in-flight preview.
    pub fn set_color(&mut self, color: PickerColor, window: &mut Window, cx: &mut Context<Self>) {
        if self.color == color {
            return;
        }
        self.color = color;
        self.draft = None;
        self.editing = false;
        self.sync_editor(window, cx);
    }
    /// Whether a nested format menu should receive dismissal before its host popup.
    pub fn has_open_menu(&self, cx: &App) -> bool {
        self.editor.read(cx).has_open_menu()
    }

    pub fn color(&self) -> PickerColor {
        self.color
    }
    pub fn cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.editing {
            cx.emit(ColorPickerAction::Edit {
                color: self.color,
                phase: ColorPickerPhase::Cancel,
            });
        }
        self.draft = None;
        self.editing = false;
        self.editor
            .update(cx, |editor, cx| editor.prepare_for_dismissal(cx));
        self.sync_editor(window, cx);
    }
    fn sync_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paint = self.draft.clone().unwrap_or_else(|| self.color.paint());
        self.editor.update(cx, |editor, cx| {
            editor.set_target("color", DesignPanelCollection::Fill, 0, paint, window, cx)
        });
        cx.notify();
    }
    fn handle_edit(
        &mut self,
        event: &PaintPickerEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            PaintPickerEvent::Edit { edit, phase, .. } => {
                let phase = match phase {
                    DesignPanelEditPhase::Begin => ColorPickerPhase::Begin,
                    DesignPanelEditPhase::Preview => ColorPickerPhase::Preview,
                    DesignPanelEditPhase::Commit => ColorPickerPhase::Commit,
                    DesignPanelEditPhase::Cancel => ColorPickerPhase::Cancel,
                };
                let mut paint = self.draft.clone().unwrap_or_else(|| self.color.paint());
                if phase == ColorPickerPhase::Cancel {
                    paint = self.color.paint();
                } else if phase != ColorPickerPhase::Begin {
                    paint.apply_edit(edit);
                }
                let color = PickerColor::from_paint(&paint);
                self.editing = matches!(phase, ColorPickerPhase::Begin | ColorPickerPhase::Preview);
                self.draft = if phase == ColorPickerPhase::Cancel {
                    None
                } else {
                    Some(paint)
                };
                self.sync_editor(window, cx);
                cx.emit(ColorPickerAction::Edit { color, phase });
            }
            PaintPickerEvent::EyedropperRequested { .. } => {
                cx.emit(ColorPickerAction::EyedropperRequested)
            }
            _ => {}
        }
    }
}
impl Focusable for ColorPicker {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.editor.focus_handle(cx)
    }
}
impl Render for ColorPicker {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("fanta-color-picker")
            .key_context(DESIGN_PANEL_KEY_CONTEXT)
            .debug_selector(|| "fanta-color-picker".to_owned())
            // Single-line Input propagates Enter after emitting PressEnter.
            // Consume it synchronously so the platform never inserts a newline.
            .on_action(|_: &gpui_component::input::Enter, window, cx| {
                window.prevent_default();
                cx.stop_propagation();
            })
            .on_action(
                cx.listener(|this, _: &gpui_component::input::Escape, window, cx| {
                    this.cancel(window, cx);
                    cx.emit(ColorPickerAction::CloseRequested);
                    cx.stop_propagation();
                }),
            )
            .on_action(
                cx.listener(|this, _: &CancelDesignInteraction, window, cx| {
                    this.cancel(window, cx);
                    cx.emit(ColorPickerAction::CloseRequested);
                    cx.stop_propagation();
                }),
            )
            .child(self.editor.clone())
    }
}

#[cfg(test)]
mod tests;
