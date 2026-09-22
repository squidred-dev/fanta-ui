//! Controlled Draw-mode context strip. Inputs and sliders hold drafts only;
//! accepted paint/selection settings are always echoed by the host.
use super::{EditorToolbar, ToolbarOverlay};
use crate::atoms::{
    ActivateControl, ControlExt as _, LucideIcon, SemanticColor as Color, TypographyExt as _,
    TypographyToken, icon_button, render_lucide_icon, tokens,
};
use crate::molecules::{menu_item, popup_max_height, popup_surface, popup_width};
use crate::toolbar::{
    DrawSelectionOperation, DrawToolbarAction, DrawToolbarOptions, ToolbarAction, ToolbarTool,
};
use gpui::{prelude::FluentBuilder as _, *};
use gpui_component::{Icon, IconName, Sizable as _, h_flex, input::Input, tooltip::Tooltip};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DrawNumber {
    Size,
    Hardness,
    Opacity,
    Flow,
    Smoothing,
    Feather,
    Tolerance,
}
impl DrawNumber {
    fn label(self) -> &'static str {
        match self {
            Self::Size => "Size",
            Self::Hardness => "Hardness",
            Self::Opacity => "Opacity",
            Self::Flow => "Flow",
            Self::Smoothing => "Smooth",
            Self::Feather => "Feather",
            Self::Tolerance => "Tolerance",
        }
    }
    fn suffix(self) -> &'static str {
        match self {
            Self::Size | Self::Feather => "px",
            Self::Tolerance => "",
            _ => "%",
        }
    }
    fn range(self) -> (u16, u16) {
        match self {
            Self::Size => (1, 5000),
            Self::Feather => (0, 1000),
            Self::Tolerance => (0, 255),
            _ => (0, 100),
        }
    }
    fn get(self, o: &DrawToolbarOptions) -> u16 {
        match self {
            Self::Size => o.size,
            Self::Hardness => o.hardness.into(),
            Self::Opacity => o.opacity.into(),
            Self::Flow => o.flow.into(),
            Self::Smoothing => o.smoothing.into(),
            Self::Feather => o.feather,
            Self::Tolerance => o.tolerance.into(),
        }
    }
    fn set(self, o: &mut DrawToolbarOptions, value: u16) {
        let (min, max) = self.range();
        let value = value.clamp(min, max);
        match self {
            Self::Size => o.size = value,
            Self::Hardness => o.hardness = value as u8,
            Self::Opacity => o.opacity = value as u8,
            Self::Flow => o.flow = value as u8,
            Self::Smoothing => o.smoothing = value as u8,
            Self::Feather => o.feather = value,
            Self::Tolerance => o.tolerance = value as u8,
        }
    }
    fn normalized(self, value: u16) -> f32 {
        let (min, max) = self.range();
        let f = (value.clamp(min, max) - min) as f32 / (max - min) as f32;
        if self == Self::Size { f.sqrt() } else { f }
    }
    pub(super) fn value_at(self, value: f32) -> u16 {
        let (min, max) = self.range();
        let value = if self == Self::Size {
            value * value
        } else {
            value
        };
        min + (value * (max - min) as f32).round() as u16
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DrawChoice {
    BrushTip,
    CropRatio,
}

fn chip(id: SharedString, label: impl Into<SharedString>, cx: &App) -> Stateful<Div> {
    icon_button(
        id,
        px(tokens::ControlSize::CHROME),
        px(tokens::Radius::CONTROL),
        cx,
    )
    .w_auto()
    .px_2()
    .gap_1()
    .flex_none()
    .bg(Color::BackgroundSecondary.resolve(cx))
    .typography(TypographyToken::BodyMedium)
    .text_color(Color::Text.resolve(cx))
    .child(label.into())
}
impl EditorToolbar {
    pub fn draw_options(&self) -> &DrawToolbarOptions {
        &self.draw_options
    }
    pub fn set_draw_options(&mut self, options: DrawToolbarOptions, cx: &mut Context<Self>) {
        self.draw_options = options.normalized();
        if let Some(ToolbarOverlay::DrawNumber(field)) = self.overlay {
            self.draw_slider.update(cx, |slider, cx| {
                slider.set_value(field.normalized(field.get(&self.draw_options)), cx)
            });
        }
        cx.notify();
    }
    pub(super) fn request_draw_number(
        &mut self,
        field: DrawNumber,
        value: u16,
        cx: &mut Context<Self>,
    ) {
        let mut options = self.draw_options.clone();
        field.set(&mut options, value);
        cx.emit(ToolbarAction::DrawOptionsChangeRequested { options });
    }
    pub(super) fn commit_draw_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(ToolbarOverlay::DrawNumber(field)) = self.overlay
            && let Ok(value) = self.draw_input.read(cx).value().trim().parse::<u16>()
        {
            self.request_draw_number(field, value, cx);
            self.dismiss_overlay(window, cx);
        }
    }
    fn toggle_draw_number(
        &mut self,
        field: DrawNumber,
        was_open: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if was_open {
            self.dismiss_overlay(window, cx);
            return;
        }
        let value = field.get(&self.draw_options);
        self.draw_slider.update(cx, |slider, cx| {
            slider.set_value(field.normalized(value), cx)
        });
        self.draw_input.update(cx, |input, cx| {
            input.set_value(value.to_string(), window, cx);
            input.focus(window, cx);
        });
        self.set_overlay(Some(ToolbarOverlay::DrawNumber(field)), cx);
    }
    pub(super) fn draw_choices(&self, choice: DrawChoice) -> &[SharedString] {
        match choice {
            DrawChoice::BrushTip => &self.draw_options.brush_tips,
            DrawChoice::CropRatio => &self.draw_options.crop_ratios,
        }
    }
    fn current_draw_choice(&self, choice: DrawChoice) -> &SharedString {
        match choice {
            DrawChoice::BrushTip => &self.draw_options.brush_tip,
            DrawChoice::CropRatio => &self.draw_options.crop_ratio,
        }
    }
    pub(super) fn choose_draw_choice(
        &mut self,
        choice: DrawChoice,
        index: usize,
        cx: &mut Context<Self>,
    ) {
        let Some(value) = self.draw_choices(choice).get(index).cloned() else {
            return;
        };
        let mut options = self.draw_options.clone();
        match choice {
            DrawChoice::BrushTip => options.brush_tip = value,
            DrawChoice::CropRatio => options.crop_ratio = value,
        }
        self.set_overlay(None, cx);
        cx.emit(ToolbarAction::DrawOptionsChangeRequested { options });
    }
    fn toggle_draw_choice(&mut self, choice: DrawChoice, was_open: bool, cx: &mut Context<Self>) {
        self.reset_menu_cursor(
            self.draw_choices(choice)
                .iter()
                .position(|v| v == self.current_draw_choice(choice))
                .unwrap_or(0),
        );
        self.set_overlay(
            (!was_open).then_some(ToolbarOverlay::DrawChoice(choice)),
            cx,
        );
    }
    fn render_draw_number(
        &self,
        field: DrawNumber,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let open = self.overlay == Some(ToolbarOverlay::DrawNumber(field));
        let value = field.get(&self.draw_options);
        let trigger = chip(format!("{}-draw-{:?}", self.id, field).into(), "", cx)
            // Reserve room for the entire supported range, including 5000px.
            .w(px(tokens::InputGeometry::NUMERIC_WIDTH
                + if matches!(field, DrawNumber::Size | DrawNumber::Feather) {
                    tokens::Space::SM
                } else {
                    tokens::Space::NONE
                }))
            .child(render_lucide_icon(
                match field {
                    DrawNumber::Size => LucideIcon::CircleDot,
                    DrawNumber::Hardness => LucideIcon::CircleDashed,
                    DrawNumber::Opacity => LucideIcon::Droplet,
                    DrawNumber::Flow => LucideIcon::Droplets,
                    DrawNumber::Smoothing => LucideIcon::Spline,
                    DrawNumber::Feather => LucideIcon::Feather,
                    DrawNumber::Tolerance => LucideIcon::Wand,
                },
                Color::TextSecondary.resolve(cx),
                tokens::IconSize::SM,
            ))
            .child(format!("{}{}", value, field.suffix()))
            .debug_selector(move || format!("toolbar-draw-{}", field.label().to_lowercase()))
            .relative()
            .tooltip(move |window, cx| {
                Tooltip::new(format!(
                    "{} · click to enter a value or drag the slider",
                    field.label()
                ))
                .build(window, cx)
            })
            // Press activation preserves the focused input and outside-dismiss contract.
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    window.prevent_default();
                    this.toggle_draw_number(field, open, window, cx);
                }),
            )
            .on_action(cx.listener(move |this, _: &ActivateControl, window, cx| {
                this.toggle_draw_number(field, open, window, cx)
            }));
        h_flex()
            .relative()
            .flex_none()
            .items_start()
            .child(trigger)
            .when(open, |trigger| {
                let popup = popup_surface(
                    format!("{}-draw-number-popup", self.id),
                    px(tokens::Radius::MENU),
                    cx,
                )
                .debug_selector(|| "toolbar-draw-number-popup".to_owned())
                .w(popup_width(window, 240.))
                .max_h(popup_max_height(window))
                .p_3()
                .gap_3()
                .typography(TypographyToken::BodyMedium)
                .occlude()
                .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                .on_mouse_down_out(
                    cx.listener(|this, _, _, cx| this.dismiss_overlay_for_pointer(cx)),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(div().flex_1().child(field.label()))
                        .child(
                            div().w(px(80.)).child(
                                Input::new(&self.draw_input)
                                    .small()
                                    .typography(TypographyToken::BodyMedium),
                            ),
                        )
                        .child(field.suffix()),
                )
                .child(self.draw_slider.clone())
                .child(
                    div()
                        .text_color(Color::TextTertiary.resolve(cx))
                        .child(format!(
                            "{}–{}{} · Enter to apply",
                            field.range().0,
                            field.range().1,
                            field.suffix()
                        )),
                );
                trigger.child(self.dock_popup(
                    popup,
                    popup_width(window, 240.),
                    !matches!(field, DrawNumber::Size | DrawNumber::Hardness),
                    11,
                ))
            })
            .into_any_element()
    }
    fn render_draw_choice(
        &self,
        choice: DrawChoice,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let open = self.overlay == Some(ToolbarOverlay::DrawChoice(choice));
        let label = self.current_draw_choice(choice).clone();
        let mut trigger = chip(format!("{}-draw-choice-{choice:?}", self.id).into(), "", cx)
            .w(px(tokens::DropdownGeometry::WIDTH))
            .debug_selector(move || format!("toolbar-draw-choice-{choice:?}"))
            .relative()
            .child(crate::atoms::truncating_label(label.clone()))
            .child(Icon::new(IconName::ChevronDown).xsmall())
            .tooltip(move |window, cx| Tooltip::new(label.clone()).build(window, cx))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| this.toggle_draw_choice(choice, open, cx)),
            )
            .on_action(cx.listener(move |this, _: &ActivateControl, window, cx| {
                if !(open && this.commit_open_menu_entry(window, cx)) {
                    this.toggle_draw_choice(choice, open, cx);
                }
            }));
        if open {
            let mut menu = popup_surface(
                format!("{}-draw-choice-menu", self.id),
                px(tokens::Radius::MENU),
                cx,
            )
            .w(popup_width(window, 220.))
            .max_h(popup_max_height(window))
            .p_1()
            .typography(TypographyToken::BodyMedium)
            .occlude()
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .on_mouse_down_out(cx.listener(|this, _, _, cx| this.dismiss_overlay_for_pointer(cx)));
            if self.draw_choices(choice).is_empty() {
                menu = menu.child(
                    div()
                        .px_2()
                        .py_1()
                        .text_color(Color::TextTertiary.resolve(cx))
                        .child("No options available"),
                );
            }
            for (index, value) in self.draw_choices(choice).iter().enumerate() {
                menu = menu.child(
                    menu_item(
                        format!("{}-draw-choice-{index}", self.id),
                        px(tokens::RowHeight::MENU),
                        cx,
                    )
                    .debug_selector(move || format!("toolbar-draw-choice-item-{index}"))
                    .when(index == self.menu_cursor, |row| {
                        row.bg(Color::BackgroundSelected.resolve(cx))
                    })
                    .on_hover(cx.listener(move |this, hovered, _, cx| {
                        if *hovered {
                            this.menu_cursor = index;
                            cx.notify();
                        }
                    }))
                    .on_activate(cx.listener(move |this, _, window, cx| {
                        this.focus_handle.focus(window, cx);
                        this.choose_draw_choice(choice, index, cx);
                    }))
                    .child(div().flex_1().child(value.clone()))
                    .when(value == self.current_draw_choice(choice), |row| {
                        row.child(Icon::new(IconName::Check).xsmall())
                    }),
                );
            }
            trigger = trigger.child(self.dock_popup(menu, popup_width(window, 220.), false, 11));
        }
        trigger.into_any_element()
    }
    fn draw_toggle(
        &self,
        id: &'static str,
        label: &'static str,
        icon: LucideIcon,
        active: bool,
        change: impl Fn(&mut DrawToolbarOptions) + 'static,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        icon_button(
            format!("{}-{id}", self.id),
            px(tokens::ControlSize::CHROME),
            px(tokens::Radius::CONTROL),
            cx,
        )
        .debug_selector(move || format!("toolbar-draw-{id}"))
        .flex_none()
        .when(active, |button| {
            button.bg(Color::BackgroundSelected.resolve(cx))
        })
        .tooltip(move |window, cx| Tooltip::new(label).build(window, cx))
        .on_activate(cx.listener(move |this, _, _, cx| {
            let mut options = this.draw_options.clone();
            change(&mut options);
            cx.emit(ToolbarAction::DrawOptionsChangeRequested { options });
        }))
        .child(render_lucide_icon(icon, Color::Text.resolve(cx), 16.))
        .into_any_element()
    }
    fn draw_action(
        &self,
        action: DrawToolbarAction,
        label: &'static str,
        icon: LucideIcon,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        chip(format!("{}-{action:?}", self.id).into(), label, cx)
            .debug_selector(move || format!("toolbar-draw-action-{action:?}"))
            .child(render_lucide_icon(icon, Color::Text.resolve(cx), 16.))
            .on_activate(
                cx.listener(move |_, _, _, cx| {
                    cx.emit(ToolbarAction::DrawActionInvoked { action })
                }),
            )
            .into_any_element()
    }
    pub(super) fn render_draw_secondary(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut row = h_flex()
            .h(px(tokens::RowHeight::SECTION_HEADER))
            .px_1()
            .gap_1()
            .flex_none();
        let o = &self.draw_options;
        match self.active_tool {
            ToolbarTool::Brush | ToolbarTool::Pencil | ToolbarTool::Eraser => {
                row = row.child(self.render_draw_choice(DrawChoice::BrushTip, window, cx));
                for field in [
                    DrawNumber::Size,
                    DrawNumber::Hardness,
                    DrawNumber::Opacity,
                    DrawNumber::Flow,
                    DrawNumber::Smoothing,
                ] {
                    row = row.child(self.render_draw_number(field, window, cx));
                }
                row = row.child(self.draw_toggle(
                    "pressure",
                    "Use pen pressure",
                    LucideIcon::PenTool,
                    o.pressure,
                    |o| o.pressure = !o.pressure,
                    cx,
                ));
            }
            ToolbarTool::RectangleSelect
            | ToolbarTool::EllipseSelect
            | ToolbarTool::Lasso
            | ToolbarTool::PolygonalLasso
            | ToolbarTool::MagicWand => {
                for op in DrawSelectionOperation::ALL {
                    let (id, icon) = match op {
                        DrawSelectionOperation::Replace => ("replace", LucideIcon::SquareDashed),
                        DrawSelectionOperation::Add => ("add", LucideIcon::SquarePlus),
                        DrawSelectionOperation::Subtract => ("subtract", LucideIcon::SquareMinus),
                        DrawSelectionOperation::Intersect => ("intersect", LucideIcon::Combine),
                    };
                    row = row.child(self.draw_toggle(
                        id,
                        op.label(),
                        icon,
                        o.selection_operation == op,
                        move |o| o.selection_operation = op,
                        cx,
                    ));
                }
                row = row
                    .child(self.render_draw_number(DrawNumber::Feather, window, cx))
                    .child(self.draw_toggle(
                        "anti-alias",
                        "Anti-alias selection edges",
                        LucideIcon::Blend,
                        o.anti_alias,
                        |o| o.anti_alias = !o.anti_alias,
                        cx,
                    ));
                if self.active_tool == ToolbarTool::MagicWand {
                    row = row
                        .child(self.render_draw_number(DrawNumber::Tolerance, window, cx))
                        .child(self.draw_toggle(
                            "contiguous",
                            "Select contiguous pixels",
                            LucideIcon::Scan,
                            o.contiguous,
                            |o| o.contiguous = !o.contiguous,
                            cx,
                        ));
                }
                row = row
                    .child(self.draw_action(
                        DrawToolbarAction::InvertSelection,
                        "Invert",
                        LucideIcon::Contrast,
                        cx,
                    ))
                    .child(self.draw_action(
                        DrawToolbarAction::Deselect,
                        "Deselect",
                        LucideIcon::SquareDashedMousePointer,
                        cx,
                    ));
            }
            ToolbarTool::Crop => {
                row = row
                    .child(self.render_draw_choice(DrawChoice::CropRatio, window, cx))
                    .child(self.draw_toggle(
                        "delete-cropped",
                        "Delete cropped pixels",
                        LucideIcon::Trash,
                        o.delete_cropped_pixels,
                        |o| o.delete_cropped_pixels = !o.delete_cropped_pixels,
                        cx,
                    ))
                    .child(self.draw_action(
                        DrawToolbarAction::CancelCrop,
                        "Cancel",
                        LucideIcon::X,
                        cx,
                    ))
                    .child(self.draw_action(
                        DrawToolbarAction::ApplyCrop,
                        "Apply crop",
                        LucideIcon::Check,
                        cx,
                    ));
            }
            ToolbarTool::PathSelect | ToolbarTool::NodeEdit | ToolbarTool::Pen => {
                row = row
                    .child(self.draw_action(
                        DrawToolbarAction::ClosePath,
                        "Close path",
                        LucideIcon::Pentagon,
                        cx,
                    ))
                    .child(self.draw_action(
                        DrawToolbarAction::JoinPaths,
                        "Join paths",
                        LucideIcon::Merge,
                        cx,
                    ))
                    .child(self.draw_action(
                        DrawToolbarAction::SimplifyPath,
                        "Simplify",
                        LucideIcon::Spline,
                        cx,
                    ));
            }
            _ => {
                row = row.child(
                    div()
                        .px_2()
                        .typography(TypographyToken::BodyMedium)
                        .text_color(Color::TextSecondary.resolve(cx))
                        .child(if self.active_tool == ToolbarTool::Hand {
                            "Pan the canvas · choose a brush or selection tool to edit"
                        } else {
                            "Sample a color from the canvas"
                        }),
                );
            }
        }
        row.into_any_element()
    }
}
