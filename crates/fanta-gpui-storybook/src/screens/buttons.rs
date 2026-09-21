//! The Buttons & activation atom story.
//!
//! An `icon_button` specimen matrix on the shared cell grid — default,
//! hover, the real reserved focus ring, a live selected toggle, the atom's
//! pressed treatment, and the disabled look — at the sizes the organisms
//! actually mount (24/32/40, plus one compact 18 px row). An "In context"
//! section then assembles the same atoms into a mini toolbar strip and a
//! mini panel-header strip so the button is seen where it lives. Every live
//! specimen registers exactly one `ControlExt::on_activate` handler, so
//! clicking or pressing Enter/Space on a focused specimen demonstrates the
//! single §9 activation path in the shared intent log, including which
//! input path fired.

use fanta_gpui::atoms::TypographyExt as _;
use gpui::{Div, FocusHandle, Stateful};

use crate::*;

use super::specimen::{
    specimen_card, specimen_cell, specimen_row, specimen_rows, specimen_story_root,
};

/// The matrix rows: selector id, left-aligned row title, and control size.
/// The standard sizes organisms mount read smallest to largest; the compact
/// 18 px size closes the matrix in its own row.
pub(crate) const BUTTON_ROW_SPECS: [(&str, &str, f32); 4] = [
    ("buttons-row-24", "24 px · list-row satellite", 24.),
    ("buttons-row-32", "32 px · toolbar tool", 32.),
    ("buttons-row-40", "40 px · primary action", 40.),
    (
        "buttons-row-compact",
        "Compact · 18 px",
        BUTTON_COMPACT_SIZE,
    ),
];

/// The compact size shown once, in its own row at the end of the matrix.
pub(crate) const BUTTON_COMPACT_SIZE: f32 = 18.;

/// The mini toolbar strip's live tools: slug, icon, and intent label.
const CONTEXT_TOOLBAR_TOOLS: [(&str, LucideIcon, &str); 5] = [
    ("play", LucideIcon::Play, "Play"),
    ("pause", LucideIcon::Pause, "Pause"),
    ("keyframe", LucideIcon::Diamond, "Keyframe"),
    ("agent", LucideIcon::Sparkles, "Agent"),
    ("help", LucideIcon::CircleQuestionMark, "Help"),
];

pub(crate) struct ButtonsScreen {
    pub(crate) focus_handle: FocusHandle,
    /// Shared state behind every "selected" specimen: activating one
    /// toggles the selected treatment for the column.
    pub(crate) selected: bool,
    /// The mini toolbar strip's Loop tool doubles as a live selected
    /// specimen in context.
    pub(crate) context_looping: bool,
    pub(crate) activation_count: usize,
    pub(crate) last_action: SharedString,
}

impl ButtonsScreen {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            selected: true,
            context_looping: true,
            activation_count: 0,
            last_action: "Ready — click a specimen, or Tab to one and press Enter or Space".into(),
        }
    }

    pub(crate) fn record_activation(&mut self, description: String, keyboard: bool) {
        self.activation_count += 1;
        self.last_action = format!(
            "{description} via {} (activation #{})",
            if keyboard { "Enter/Space" } else { "pointer" },
            self.activation_count
        )
        .into();
    }
}

/// One live matrix specimen: the atom plus one logging activation handler.
fn live_button_specimen(
    slug: &'static str,
    description: &'static str,
    size: f32,
    cx: &mut Context<Storybook>,
) -> Stateful<Div> {
    let id = SharedString::from(format!("buttons-{slug}-{size:.0}"));
    let selector = id.to_string();
    icon_button(id, px(size), px(4.), cx)
        .debug_selector(move || selector.clone())
        .on_activate(cx.listener(move |this, event: &ActivateEvent, _, cx| {
            this.buttons_screen.record_activation(
                format!("Activated the {size:.0} px {description} specimen"),
                event.keyboard,
            );
            cx.notify();
        }))
}

impl Storybook {
    fn render_semantic_button_matrix(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut rows = Vec::new();
        for variant in SemanticButtonVariant::ALL {
            let label = variant.label();
            let slug = label.to_ascii_lowercase().replace(' ', "-");
            let mut cells = Vec::new();
            for state in SemanticButtonState::ALL {
                let id = SharedString::from(format!("semantic-button-{slug}-{state:?}"));
                let button = semantic_button(id, variant, SemanticButtonSize::Default)
                    .label(label)
                    .preview_state(state)
                    .on_activate(cx.listener(move |this, event: &ActivateEvent, _, cx| {
                        this.buttons_screen.record_activation(
                            format!("Activated {label} ({})", state.label()),
                            event.keyboard,
                        );
                        cx.notify();
                    }))
                    .into_any_element();
                cells.push(specimen_cell(state.label(), None, button, cx));
            }
            rows.push(specimen_row(
                SharedString::from(format!("semantic-button-row-{slug}")),
                label,
                cells,
                cx,
            ));
        }
        specimen_rows(rows)
    }

    fn render_semantic_button_sizes(&self, cx: &mut Context<Self>) -> AnyElement {
        let examples = [
            semantic_button(
                "semantic-button-size-default",
                SemanticButtonVariant::Primary,
                SemanticButtonSize::Default,
            )
            .label("Default")
            .into_any_element(),
            semantic_button(
                "semantic-button-size-large",
                SemanticButtonVariant::Primary,
                SemanticButtonSize::Large,
            )
            .label("Large")
            .into_any_element(),
            semantic_button(
                "semantic-button-icon-left",
                SemanticButtonVariant::Secondary,
                SemanticButtonSize::Default,
            )
            .label("Left icon")
            .icon(LucideIcon::Plus, SemanticButtonIconAlignment::Left)
            .into_any_element(),
            semantic_button(
                "semantic-button-icon-center",
                SemanticButtonVariant::Secondary,
                SemanticButtonSize::Default,
            )
            .label("Hidden")
            .icon(LucideIcon::Plus, SemanticButtonIconAlignment::Center)
            .into_any_element(),
        ];
        specimen_rows(vec![
            specimen_row(
                "semantic-button-sizes",
                "Size and leading-icon alignment",
                examples
                    .into_iter()
                    .enumerate()
                    .map(|(index, example)| {
                        specimen_cell(
                            [
                                "Default · 24 px",
                                "Large · 32 px",
                                "Left-aligned",
                                "Center-aligned",
                            ][index],
                            None,
                            example,
                            cx,
                        )
                    })
                    .collect(),
                cx,
            ),
            specimen_row(
                "semantic-button-wide",
                "Wide · 256 × 24 px",
                vec![
                    semantic_button(
                        "semantic-button-size-wide",
                        SemanticButtonVariant::Primary,
                        SemanticButtonSize::Wide,
                    )
                    .label("Wide button")
                    .into_any_element(),
                ],
                cx,
            ),
        ])
    }

    fn render_semantic_icon_button_matrix(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut rows = Vec::new();
        for (kind, title, icon) in [
            (
                SemanticIconButtonKind::Button,
                "Button icon",
                LucideIcon::Plus,
            ),
            (
                SemanticIconButtonKind::Toggle,
                "Button icon toggle · on",
                LucideIcon::Eye,
            ),
            (
                SemanticIconButtonKind::DialogToggle,
                "Button icon dialog toggle · on",
                LucideIcon::SlidersHorizontal,
            ),
        ] {
            let slug = title.to_ascii_lowercase().replace([' ', '·'], "-");
            let cells = SemanticButtonState::ALL
                .into_iter()
                .map(|state| {
                    let button = semantic_icon_button(
                        SharedString::from(format!("{slug}-{state:?}")),
                        icon,
                        kind,
                    )
                    .on(kind != SemanticIconButtonKind::Button)
                    .highlighted(kind == SemanticIconButtonKind::Toggle)
                    .secondary(kind == SemanticIconButtonKind::DialogToggle)
                    .preview_state(state)
                    .into_any_element();
                    specimen_cell(state.label(), None, button, cx)
                })
                .collect();
            rows.push(specimen_row(
                SharedString::from(format!("semantic-{slug}")),
                title,
                cells,
                cx,
            ));
        }

        let split_cells = SemanticSplitButtonState::ALL
            .into_iter()
            .map(|state| {
                specimen_cell(
                    state.label(),
                    None,
                    semantic_icon_button(
                        SharedString::from(format!("split-{state:?}")),
                        LucideIcon::Plus,
                        SemanticIconButtonKind::Split,
                    )
                    .split_state(state)
                    .into_any_element(),
                    cx,
                )
            })
            .collect();
        rows.push(specimen_row(
            "semantic-split-button",
            "Button icon split · 41 × 24 px",
            split_cells,
            cx,
        ));
        rows.push(specimen_row(
            "semantic-split-button-large",
            "Button icon split · large · 49 × 32 px",
            vec![specimen_cell(
                "Large",
                None,
                semantic_icon_button(
                    "split-large",
                    LucideIcon::Plus,
                    SemanticIconButtonKind::Split,
                )
                .large(true)
                .into_any_element(),
                cx,
            )],
            cx,
        ));
        specimen_rows(rows)
    }

    /// One titled matrix row: the six state cells at one control size.
    fn render_button_specimen_row(
        &self,
        row_id: &'static str,
        title: &'static str,
        size: f32,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let icon_size = (size * 0.55).clamp(10., 18.);
        let selected = self.buttons_screen.selected;

        let default_cell = {
            let specimen = live_button_specimen("default", "default", size, cx)
                .child(render_lucide_icon(
                    LucideIcon::Play,
                    fanta_gpui::atoms::SemanticColor::Text.resolve(cx),
                    icon_size,
                ))
                .into_any_element();
            specimen_cell("default", None, specimen, cx)
        };
        let hover_cell = {
            let specimen = live_button_specimen("hover", "hover", size, cx)
                .child(render_lucide_icon(
                    LucideIcon::Sparkles,
                    fanta_gpui::atoms::SemanticColor::Text.resolve(cx),
                    icon_size,
                ))
                .into_any_element();
            specimen_cell("hover", Some("hover me"), specimen, cx)
        };
        let focus_cell = {
            // The real focus ring, with its color forced visible: a Tab
            // stop shows the same ring without layout shift because the
            // atom reserves the border width while unfocused.
            let specimen = live_button_specimen("focus", "focus-ring", size, cx)
                .border_color(fanta_gpui::atoms::SemanticColor::BackgroundSelected.resolve(cx))
                .child(render_lucide_icon(
                    LucideIcon::Diamond,
                    fanta_gpui::atoms::SemanticColor::Text.resolve(cx),
                    icon_size,
                ))
                .into_any_element();
            specimen_cell("focus ring", Some("reserved border"), specimen, cx)
        };
        let selected_cell = {
            let id = SharedString::from(format!("buttons-selected-{size:.0}"));
            let selector = id.to_string();
            let specimen = icon_button(id, px(size), px(4.), cx)
                .debug_selector(move || selector.clone())
                .when(selected, |button| {
                    button.bg(fanta_gpui::atoms::SemanticColor::BackgroundHover.resolve(cx))
                })
                .on_activate(cx.listener(move |this, event: &ActivateEvent, _, cx| {
                    this.buttons_screen.selected = !this.buttons_screen.selected;
                    this.buttons_screen.record_activation(
                        format!(
                            "Toggled the {size:.0} px selected specimen {}",
                            if this.buttons_screen.selected {
                                "on"
                            } else {
                                "off"
                            }
                        ),
                        event.keyboard,
                    );
                    cx.notify();
                }))
                .child(render_lucide_icon(
                    if selected {
                        LucideIcon::Eye
                    } else {
                        LucideIcon::EyeOff
                    },
                    fanta_gpui::atoms::SemanticColor::Text.resolve(cx),
                    icon_size,
                ))
                .into_any_element();
            specimen_cell(
                if selected {
                    "selected · on"
                } else {
                    "selected · off"
                },
                Some("click toggles"),
                specimen,
                cx,
            )
        };
        let pressed_cell = {
            // The atom's new pressed treatment, previewed statically so
            // the column reads at a glance; the specimen is also live, so
            // holding any other cell down shows the same deepened fill.
            let specimen = live_button_specimen("pressed", "pressed", size, cx)
                .bg(fanta_gpui::atoms::SemanticColor::BackgroundHover.resolve(cx))
                .child(render_lucide_icon(
                    LucideIcon::Repeat2,
                    fanta_gpui::atoms::SemanticColor::Text.resolve(cx),
                    icon_size,
                ))
                .into_any_element();
            specimen_cell("pressed", Some("press & hold"), specimen, cx)
        };
        let disabled_cell = {
            // The atom has no disabled slot: hosts style the look and
            // withhold the activation handler, so this cell is the one
            // deliberate non-control in the matrix.
            let specimen = div()
                .id(SharedString::from(format!("buttons-disabled-{size:.0}")))
                .size(px(size))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(4.))
                .border_1()
                .border_color(cx.theme().transparent)
                .opacity(0.4)
                .child(render_lucide_icon(
                    LucideIcon::Lock,
                    fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx),
                    icon_size,
                ))
                .into_any_element();
            specimen_cell("disabled", Some("no handler"), specimen, cx)
        };

        specimen_row(
            row_id,
            title,
            vec![
                default_cell,
                hover_cell,
                focus_cell,
                selected_cell,
                pressed_cell,
                disabled_cell,
            ],
            cx,
        )
    }

    /// One live strip control shared by the in-context section.
    fn context_strip_button(
        &self,
        strip: &'static str,
        slug: &'static str,
        description: &'static str,
        icon: LucideIcon,
        size: f32,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let icon_size = (size * 0.55).clamp(10., 18.);
        let id = SharedString::from(format!("buttons-context-{strip}-{slug}"));
        let selector = id.to_string();
        icon_button(id, px(size), px(4.), cx)
            .debug_selector(move || selector.clone())
            .on_activate(cx.listener(move |this, event: &ActivateEvent, _, cx| {
                this.buttons_screen.record_activation(
                    format!("Activated {description} in the {strip} strip"),
                    event.keyboard,
                );
                cx.notify();
            }))
            .child(render_lucide_icon(
                icon,
                fanta_gpui::atoms::SemanticColor::Text.resolve(cx),
                icon_size,
            ))
            .into_any_element()
    }

    /// A mini toolbar strip assembled from the same atoms at real sizes on
    /// the secondary surface color.
    fn render_context_toolbar_strip(&self, cx: &mut Context<Self>) -> AnyElement {
        let looping = self.buttons_screen.context_looping;
        let mut strip = h_flex()
            .id("buttons-context-toolbar")
            .debug_selector(|| "buttons-context-toolbar".to_owned())
            .flex_none()
            .items_center()
            .gap_1()
            .px_1p5()
            .py_1()
            .rounded(px(8.))
            .border_1()
            .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
            .bg(fanta_gpui::atoms::SemanticColor::BackgroundSecondary.resolve(cx));
        for (index, (slug, icon, description)) in CONTEXT_TOOLBAR_TOOLS.into_iter().enumerate() {
            if index == 2 {
                // The live Loop toggle sits between transport and
                // annotation tools, exactly like a real mode control.
                strip = strip.child(
                    icon_button("buttons-context-toolbar-loop", px(32.), px(4.), cx)
                        .debug_selector(|| "buttons-context-toolbar-loop".to_owned())
                        .when(looping, |button| {
                            button.bg(fanta_gpui::atoms::SemanticColor::BackgroundHover.resolve(cx))
                        })
                        .on_activate(cx.listener(|this, event: &ActivateEvent, _, cx| {
                            this.buttons_screen.context_looping =
                                !this.buttons_screen.context_looping;
                            this.buttons_screen.record_activation(
                                format!(
                                    "Toggled Loop {} in the toolbar strip",
                                    if this.buttons_screen.context_looping {
                                        "on"
                                    } else {
                                        "off"
                                    }
                                ),
                                event.keyboard,
                            );
                            cx.notify();
                        }))
                        .child(render_lucide_icon(
                            LucideIcon::Repeat2,
                            if looping {
                                fanta_gpui::atoms::SemanticColor::Text.resolve(cx)
                            } else {
                                fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx)
                            },
                            17.,
                        )),
                );
                strip = strip.child(
                    div()
                        .flex_none()
                        .w(px(1.))
                        .h(px(18.))
                        .mx_1()
                        .bg(fanta_gpui::atoms::SemanticColor::Border.resolve(cx)),
                );
            }
            strip =
                strip.child(self.context_strip_button("toolbar", slug, description, icon, 32., cx));
        }
        strip.into_any_element()
    }

    /// A mini panel-header strip: a truncating title with compact chrome
    /// satellites, the way Pages/Layers/Design headers mount the atom.
    fn render_context_panel_header(&self, cx: &mut Context<Self>) -> AnyElement {
        h_flex()
            .id("buttons-context-header")
            .debug_selector(|| "buttons-context-header".to_owned())
            .flex_none()
            .w(px(280.))
            .max_w_full()
            .h(px(40.))
            .items_center()
            .gap_1()
            .pl_3()
            .pr_1p5()
            .rounded(px(8.))
            .border_1()
            .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
            .bg(fanta_gpui::atoms::SemanticColor::BackgroundSecondary.resolve(cx))
            .child(
                truncating_label("Design panel")
                    .typography(fanta_gpui::atoms::TypographyToken::BodyLarge)
                    .font_medium(),
            )
            .child(self.context_strip_button(
                "header",
                "jump",
                "Jump to selection",
                LucideIcon::CornerUpRight,
                24.,
                cx,
            ))
            .child(self.context_strip_button(
                "header",
                "help",
                "Help",
                LucideIcon::CircleQuestionMark,
                24.,
                cx,
            ))
            .child(self.context_strip_button(
                "header",
                "close",
                "Close panel",
                LucideIcon::X,
                24.,
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn render_buttons_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let matrix = specimen_rows(
            BUTTON_ROW_SPECS
                .into_iter()
                .map(|(row_id, title, size)| {
                    self.render_button_specimen_row(row_id, title, size, cx)
                })
                .collect(),
        );
        let in_context = specimen_rows(vec![
            specimen_row(
                "buttons-context-toolbar-row",
                "Toolbar strip · 32 px tools with a live Loop toggle",
                vec![self.render_context_toolbar_strip(cx)],
                cx,
            ),
            specimen_row(
                "buttons-context-header-row",
                "Panel header · truncating title with 24 px chrome satellites",
                vec![self.render_context_panel_header(cx)],
                cx,
            ),
        ]);
        specimen_story_root("storybook-buttons")
            .track_focus(&self.buttons_screen.focus_handle)
            .child(specimen_card(
                "buttons-semantic-matrix",
                "Button · 9 variants × interaction states",
                "The UI3 Button taxonomy is preserved directly. Each row is a visual intent; each column previews default, hover, active, focused, and disabled behavior using Fanta semantic colors and typography.",
                self.render_semantic_button_matrix(cx),
                cx,
            ))
            .child(specimen_card(
                "buttons-semantic-sizes",
                "Button · size and icon properties",
                "Default is 24 px high, large is 32 px, and wide is 256 × 24 px. Leading icons can be left-aligned with a label or centered as the button's only visible content.",
                self.render_semantic_button_sizes(cx),
                cx,
            ))
            .child(specimen_card(
                "buttons-semantic-icon-families",
                "Icon button families",
                "Button icon, icon toggle, dialog toggle, and split button remain separate component families. Toggles expose on/highlighted properties; split buttons preserve primary and secondary active/focus states.",
                self.render_semantic_icon_button_matrix(cx),
                cx,
            ))
            .child(specimen_card(
                "buttons-activation-matrix",
                "Low-level icon_button · activation contract",
                "Every live specimen registers one ControlExt::on_activate handler: \
                 pointer clicks and Enter/Space on a focused specimen run the same \
                 code, and the intent log names which input path fired. Hover any \
                 cell for the hover fill, hold it down for the pressed fill, and \
                 Tab through the matrix for the reserved, non-shifting focus ring.",
                matrix,
                cx,
            ))
            .child(specimen_card(
                "buttons-in-context",
                "In context · the same atoms where they live",
                "The strips below are assembled from icon_button and the vector \
                 icons at real sizes on the secondary surface color — no bespoke \
                 chrome. Every control is live and reports into the intent log.",
                in_context,
                cx,
            ))
            .child(specimen_card(
                "buttons-activation-copy",
                "What the atom owns",
                "icon_button owns the complete §9 control recipe: id, the \
                 FantaControl key context, a tab stop, hover and pressed fills, \
                 pointer cursor, and the focus ring with its border width reserved \
                 while unfocused. Callers add children, selected styling, and \
                 exactly one on_activate handler — never hand-wired \
                 on_click/on_action pairs.",
                div()
                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child(format!(
                        "Activations received so far: {}",
                        self.buttons_screen.activation_count
                    ))
                    .into_any_element(),
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn render_buttons_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-buttons",
            self.render_buttons_story(cx),
            self.buttons_screen.last_action.clone(),
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specimen_rows_read_ascending_with_the_compact_row_last() {
        let standard = &BUTTON_ROW_SPECS[..BUTTON_ROW_SPECS.len() - 1];
        assert!(
            standard.windows(2).all(|pair| pair[0].2 < pair[1].2),
            "the standard rows read smallest to largest"
        );
        let (compact_id, _, compact_size) = BUTTON_ROW_SPECS[BUTTON_ROW_SPECS.len() - 1];
        assert_eq!(compact_id, "buttons-row-compact");
        assert_eq!(compact_size, BUTTON_COMPACT_SIZE);
        assert!(
            standard.iter().all(|(_, _, size)| compact_size < *size),
            "the compact row is the one below-standard size"
        );
        let ids: std::collections::HashSet<_> =
            BUTTON_ROW_SPECS.iter().map(|(id, _, _)| *id).collect();
        assert_eq!(ids.len(), BUTTON_ROW_SPECS.len(), "row ids stay unique");
    }
}
