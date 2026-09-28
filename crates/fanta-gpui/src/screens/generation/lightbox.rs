//! Full-screen creation preview. The selected output remains host data; only
//! whether this presentation layer is open is owned by `GenerationScreen`.

use gpui::{
    AnyElement, App, Context, InteractiveElement as _, IntoElement as _, KeyDownEvent, MouseButton,
    ObjectFit, ParentElement as _, Role, SharedString, StatefulInteractiveElement as _,
    Styled as _, StyledImage as _, Window, div, img, px,
};

use crate::atoms::{CONTROL_KEY_CONTEXT, ControlExt as _};

use super::{GenerationAction, GenerationKind, GenerationOutput, GenerationOutputStatus};
use super::{GenerationScreen, GenerationStyle};

impl GenerationScreen {
    /// Render a window-filling preview above the generation workspace. The
    /// host still supplies and selects the output; this layer owns no media.
    pub(super) fn render_lightbox(
        &self,
        colors: GenerationStyle,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let active_id = self.lightbox_output_id.as_ref()?;
        let outputs: Vec<&GenerationOutput> = self
            .data
            .outputs
            .iter()
            .filter(|output| output.kind == self.kind)
            .collect();
        let index = outputs.iter().position(|output| &output.id == active_id)?;
        let output = outputs[index];
        let screen = cx.entity();

        let backdrop = div()
            .id(SharedString::from(format!("{}-lightbox-backdrop", self.id)))
            .debug_selector(|| "generation-lightbox-backdrop".to_owned())
            .absolute()
            .inset_0()
            .bg(colors.background.opacity(0.97))
            .on_mouse_down(MouseButton::Left, {
                let screen = screen.clone();
                move |_, window, cx| {
                    cx.stop_propagation();
                    screen.update(cx, |this, cx| this.close_lightbox(window, cx));
                }
            });

        let close = self.lightbox_button("close", "Close preview", true, colors, {
            let screen = screen.clone();
            move |window, cx| {
                screen.update(cx, |this, cx| this.close_lightbox(window, cx));
            }
        });
        let previous = self.lightbox_button("previous", "Previous", index > 0, colors, {
            let screen = screen.clone();
            move |_, cx| {
                screen.update(cx, |this, cx| this.step_lightbox(-1, cx));
            }
        });
        let next = self.lightbox_button("next", "Next", index + 1 < outputs.len(), colors, {
            let screen = screen.clone();
            move |_, cx| {
                screen.update(cx, |this, cx| this.step_lightbox(1, cx));
            }
        });

        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(18.))
            .pb(px(14.))
            .border_b_1()
            .border_color(colors.border)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .min_w_0()
                    .child(
                        div()
                            .text_size(px(10.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(colors.accent)
                            .child("YOUR CREATIONS"),
                    )
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(colors.text)
                            .child(output.title.clone()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .flex_none()
                    .gap(px(8.))
                    .child(
                        div()
                            .px(px(10.))
                            .text_size(px(11.))
                            .text_color(colors.muted)
                            .child(format!("{} / {}", index + 1, outputs.len())),
                    )
                    .child(previous)
                    .child(next)
                    .child(close),
            );

        let panel = div()
            .id(SharedString::from(format!("{}-lightbox-dialog", self.id)))
            .role(Role::Dialog)
            .aria_label(format!("Preview: {}", output.title))
            .debug_selector(|| "generation-lightbox-dialog".to_owned())
            .key_context("FantaGenerationLightbox")
            .tab_group()
            .tab_index(0)
            .tab_stop(false)
            .track_focus(&self.lightbox_focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "escape" => this.close_lightbox(window, cx),
                    "left" => this.step_lightbox(-1, cx),
                    "right" => this.step_lightbox(1, cx),
                    _ => return,
                }
                cx.stop_propagation();
            }))
            .absolute()
            .top(px(16.))
            .right(px(16.))
            .bottom(px(16.))
            .left(px(16.))
            .occlude()
            .flex()
            .flex_col()
            .p(px(18.))
            .rounded_sm()
            .border_1()
            .border_color(colors.border)
            .bg(colors.panel)
            .child(header)
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .gap(px(16.))
                    .pt(px(16.))
                    .child(self.lightbox_stage(output, colors))
                    .child(self.lightbox_details(output, colors, cx)),
            );

        Some(
            div()
                .id(SharedString::from(format!("{}-lightbox", self.id)))
                .debug_selector(|| "generation-lightbox".to_owned())
                .absolute()
                .inset_0()
                .occlude()
                .child(backdrop)
                .child(panel)
                .into_any_element(),
        )
    }

    fn lightbox_stage(&self, output: &GenerationOutput, colors: GenerationStyle) -> AnyElement {
        let stage = div()
            .id(SharedString::from(format!("{}-lightbox-stage", self.id)))
            .debug_selector(|| "generation-lightbox-stage".to_owned())
            .flex_1()
            .min_w_0()
            .h_full()
            .rounded_sm()
            .border_1()
            .border_color(colors.border)
            .bg(colors.background)
            .overflow_hidden()
            .flex()
            .items_center()
            .justify_center();
        if matches!(output.kind, GenerationKind::Video | GenerationKind::Audio)
            && let Some(player) = &output.playback_view
        {
            return stage.child(player.clone()).into_any_element();
        }
        if let Some(preview) = &output.preview {
            stage
                .child(
                    img(preview.clone())
                        .w_full()
                        .h_full()
                        .object_fit(ObjectFit::Contain),
                )
                .into_any_element()
        } else {
            stage
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.))
                        .text_color(colors.muted)
                        .child(output.status.label())
                        .child(format!("{} preview unavailable", output.kind.label())),
                )
                .into_any_element()
        }
    }

    fn lightbox_details(
        &self,
        output: &GenerationOutput,
        colors: GenerationStyle,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let model_label = self
            .data
            .models
            .iter()
            .find(|model| model.id == output.model_id)
            .map(|model| model.label.clone())
            .unwrap_or_else(|| output.model_id.clone());
        let mut actions = div().flex().flex_wrap().gap(px(8.));
        let screen = cx.entity();
        if output.status == GenerationOutputStatus::Succeeded {
            if output.can_add_to_project {
                let id = output.id.clone();
                let screen = screen.clone();
                actions = actions.child(self.lightbox_button(
                    "add-to-project",
                    "Add to project",
                    true,
                    colors,
                    move |_, cx| {
                        screen.update(cx, |_, cx| {
                            cx.emit(GenerationAction::AddToProjectRequested { id: id.clone() });
                        });
                    },
                ));
            }
            if matches!(output.kind, GenerationKind::Video | GenerationKind::Audio)
                && output.playback_view.is_none()
            {
                let id = output.id.clone();
                let screen = screen.clone();
                actions = actions.child(self.lightbox_button(
                    "play",
                    "Play",
                    true,
                    colors,
                    move |_, cx| {
                        screen.update(cx, |_, cx| {
                            cx.emit(GenerationAction::PlayRequested { id: id.clone() });
                        });
                    },
                ));
            }
            let id = output.id.clone();
            let screen = screen.clone();
            actions = actions.child(self.lightbox_button(
                "download",
                "Download",
                true,
                colors,
                move |_, cx| {
                    screen.update(cx, |_, cx| {
                        cx.emit(GenerationAction::DownloadRequested { id: id.clone() });
                    });
                },
            ));
        }
        if output.prompt.is_some() {
            let id = output.id.clone();
            actions = actions.child(self.lightbox_button(
                "reuse-prompt",
                "Use prompt",
                true,
                colors,
                move |window, cx| {
                    screen.update(cx, |this, cx| {
                        this.reuse_prompt(id.clone(), window, cx);
                        this.close_lightbox(window, cx);
                    });
                },
            ));
        }

        let mut details = div()
            .id(SharedString::from(format!("{}-lightbox-details", self.id)))
            .debug_selector(|| "generation-lightbox-details".to_owned())
            .w(px(274.))
            .flex_none()
            .h_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap(px(18.))
            .p(px(16.))
            .rounded_sm()
            .border_1()
            .border_color(colors.border)
            .bg(colors.surface)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(10.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(colors.muted)
                            .child("DETAILS"),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(if output.status == GenerationOutputStatus::Succeeded {
                                colors.accent
                            } else {
                                colors.muted
                            })
                            .child(output.status.label()),
                    ),
            )
            .child(Self::lightbox_info("MODEL", model_label, colors))
            .child(Self::lightbox_info(
                "CREATED",
                output.created_at.clone(),
                colors,
            ));
        if !output.detail.is_empty() {
            details = details.child(Self::lightbox_info("OUTPUT", output.detail.clone(), colors));
        }
        if let Some(prompt) = &output.prompt {
            details = details.child(Self::lightbox_info("PROMPT", prompt.clone(), colors));
        }
        details
            .child(div().pt(px(6.)).child(actions))
            .into_any_element()
    }

    fn lightbox_info(
        label: &'static str,
        value: SharedString,
        colors: GenerationStyle,
    ) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(
                div()
                    .text_size(px(10.))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors.muted)
                    .child(label),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(colors.text)
                    .child(value),
            )
            .into_any_element()
    }

    fn lightbox_button(
        &self,
        suffix: &'static str,
        label: &'static str,
        enabled: bool,
        colors: GenerationStyle,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> AnyElement {
        let button = div()
            .id(SharedString::from(format!(
                "{}-lightbox-{}",
                self.id, suffix
            )))
            .role(Role::Button)
            .aria_label(label)
            .debug_selector(move || format!("generation-lightbox-{suffix}"))
            .key_context(CONTROL_KEY_CONTEXT)
            .tab_index(if enabled { 0 } else { -1 })
            .min_h(px(32.))
            .px(px(12.))
            .flex()
            .items_center()
            .justify_center()
            .rounded_sm()
            .border_1()
            .border_color(colors.border)
            .bg(colors.panel)
            .text_size(px(11.))
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(if enabled { colors.text } else { colors.muted })
            .child(label);
        if enabled {
            button
                .cursor_pointer()
                .hover(|style| style.bg(colors.hover))
                .focus(|style| style.border_color(colors.focus))
                .on_activate(move |_, window, cx| handler(window, cx))
                .into_any_element()
        } else {
            button.into_any_element()
        }
    }
}
