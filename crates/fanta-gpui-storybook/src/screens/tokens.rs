//! The Design tokens atom story: the shared geometry scale, drawn at size.
//!
//! `fanta_gpui::atoms::tokens` is a wall of bare `f32` constants, so the one
//! thing a reader cannot check by reading it is how big any of them actually
//! are. This story is the scale sheet. Every constant in every group gets one
//! line — the path a host types, the number, and then the geometry itself
//! drawn at that number. A `Space` step is a bar that many pixels wide, a
//! `RowHeight` is a strip that tall, a `ControlSize` is a hit target that
//! square, a `Radius` is a corner turned at it, an `IconSize` is a real Lucide
//! glyph rendered at it, a `TypeScale` step is text set in it, and a
//! `MenuWidth` or `Breakpoint` is a surface that wide. Nothing on the sheet is
//! scaled, so any two tokens can be compared by eye.
//!
//! The sheet closes on the constants tokens.rs documents as deliberately
//! off-step shipping geometry: each one is drawn again between the two on-grid
//! neighbours it sits between, so the reader can see how far off-step it is
//! rather than taking the doc comment's word for it.
//!
//! This story is a reference sheet. It has no knobs, owns no mock host state,
//! and emits no intents.

use gpui::FocusHandle;

use fanta_gpui::atoms::tokens::{
    Breakpoint, ControlSize, IconSize, MenuWidth, Radius, RowHeight, Space, TypeScale,
};

use crate::*;

use super::spec::NamedState;
use super::specimen::{specimen_card, specimen_row, specimen_rows, specimen_story_root};

/// Width of the token-path column. Every specimen on the sheet starts at the
/// same x, which is what makes two tokens comparable by eye.
const NAME_COLUMN: f32 = 176.;

/// Width of the right-aligned value column.
const VALUE_COLUMN: f32 = 52.;

/// Gap between the three columns of a token line.
const COLUMN_GAP: f32 = Space::MD;

/// Width of the `RowHeight` strip specimen. Only its height is the token.
const ROW_STRIP_WIDTH: f32 = 220.;

/// Width and height of the `Radius` corner specimen. Only its corner is the
/// token.
const CORNER_SPECIMEN_WIDTH: f32 = 64.;
const CORNER_SPECIMEN_HEIGHT: f32 = 48.;

/// Height of the `Space` bar and the `Breakpoint` ruler. Only their widths are
/// the token.
const BAR_HEIGHT: f32 = 10.;

/// The card and story-root padding a line sits inside.
const SHEET_PADDING: f32 = 2. * Space::LG + 2. * Space::MD;

/// The narrowest story viewport that still shows the widest specimen — the
/// `Breakpoint::WIDE` ruler — without running past the card edge. Nothing on
/// this sheet rescales to fit, so the story should be registered no narrower
/// than this.
pub(crate) const TOKENS_SHEET_MIN_WIDTH: f32 =
    NAME_COLUMN + VALUE_COLUMN + 2. * COLUMN_GAP + Breakpoint::WIDE + SHEET_PADDING;

/// The string every `TypeScale` specimen is set in, so size is the only
/// variable down that group.
const TYPE_SAMPLE: &str = "Design tokens 0123";

/// One line of the scale sheet: the path a host types, the value, and the
/// geometry drawn at it.
#[derive(Clone, Copy, Debug)]
struct TokenSpec {
    /// The path a host types, spelled as `Group::CONSTANT`.
    name: &'static str,
    /// The constant itself, read straight from the library.
    value: f32,
    /// The site the group's own doc comment names for this step, carried only
    /// where that comment lists the sites in constant order.
    hint: Option<&'static str>,
    /// Set where tokens.rs documents the constant as deliberately preserved
    /// off-step shipping geometry.
    off_step: bool,
}

impl TokenSpec {
    fn new(name: &'static str, value: f32) -> Self {
        Self {
            name,
            value,
            hint: None,
            off_step: false,
        }
    }

    fn hint(mut self, hint: &'static str) -> Self {
        self.hint = Some(hint);
        self
    }

    fn off_step(mut self) -> Self {
        self.off_step = true;
        self
    }
}

/// The eight groups of the scale, in the order tokens.rs declares them.
///
/// The sheet draws one card per group in [`NamedState::ALL`] order, and the
/// module's tests walk the same list, so a group cannot be added to the
/// library and quietly missed here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TokenGroup {
    Space,
    RowHeight,
    ControlSize,
    Radius,
    IconSize,
    TypeScale,
    MenuWidth,
    Breakpoint,
}

impl NamedState for TokenGroup {
    const ALL: &'static [Self] = &[
        Self::Space,
        Self::RowHeight,
        Self::ControlSize,
        Self::Radius,
        Self::IconSize,
        Self::TypeScale,
        Self::MenuWidth,
        Self::Breakpoint,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Space => "Space",
            Self::RowHeight => "RowHeight",
            Self::ControlSize => "ControlSize",
            Self::Radius => "Radius",
            Self::IconSize => "IconSize",
            Self::TypeScale => "TypeScale",
            Self::MenuWidth => "MenuWidth",
            Self::Breakpoint => "Breakpoint",
        }
    }
}

impl TokenGroup {
    /// The card's element id and debug selector.
    const fn card_id(self) -> &'static str {
        match self {
            Self::Space => "tokens-space",
            Self::RowHeight => "tokens-row-height",
            Self::ControlSize => "tokens-control-size",
            Self::Radius => "tokens-radius",
            Self::IconSize => "tokens-icon-size",
            Self::TypeScale => "tokens-type-scale",
            Self::MenuWidth => "tokens-menu-width",
            Self::Breakpoint => "tokens-breakpoint",
        }
    }

    /// The specimen row's element id and debug selector.
    const fn row_id(self) -> &'static str {
        match self {
            Self::Space => "tokens-space-row",
            Self::RowHeight => "tokens-row-height-row",
            Self::ControlSize => "tokens-control-size-row",
            Self::Radius => "tokens-radius-row",
            Self::IconSize => "tokens-icon-size-row",
            Self::TypeScale => "tokens-type-scale-row",
            Self::MenuWidth => "tokens-menu-width-row",
            Self::Breakpoint => "tokens-breakpoint-row",
        }
    }

    const fn title(self) -> &'static str {
        match self {
            Self::Space => "Space · padding and gap steps",
            Self::RowHeight => "RowHeight · the fixed horizontal strips",
            Self::ControlSize => "ControlSize · square hit targets",
            Self::Radius => "Radius · the corners Fanta owns",
            Self::IconSize => "IconSize · rendered glyph sizes",
            Self::TypeScale => "TypeScale · text sizes",
            Self::MenuWidth => "MenuWidth · transient surface widths",
            Self::Breakpoint => "Breakpoint · where a panel changes density",
        }
    }

    const fn copy(self) -> &'static str {
        match self {
            Self::Space => {
                "Each bar is exactly as wide as its constant, so NONE plus the \
                 four spacing steps read as one 4 px ramp: XS is a quarter of LG. These are the \
                 paddings and gaps rows, sections, and panel chrome share — the \
                 bar's height means nothing."
            }
            Self::RowHeight => {
                "Each strip is drawn at its own height and at one shared width, \
                 so stacking them shows how little separates a field row from a \
                 list row and how much separates either from a section header. \
                 Two of the six sit off the 4 px grid and carry a badge."
            }
            Self::ControlSize => {
                "Each square is the hit target at full size, with a real glyph \
                 centered in it at the icon token that site would use — XS \
                 inside the inline affordance, MD inside the chrome button and \
                 the toolbar tool. Compare the squares, not the glyphs."
            }
            Self::Radius => {
                "The same box turned at each corner radius Fanta owns. Surface \
                 radius is not here: a host theme drives that through \
                 Theme::radius and radius_lg, and a token restating a theme \
                 value does not belong in the scale."
            }
            Self::IconSize => {
                "One Lucide glyph rendered three times, at each of the three \
                 sizes, so the only variable down the column is the token. SM \
                 sits between the other two off the grid and carries a badge."
            }
            Self::TypeScale => {
                "The same sample string set at each step. The ramp is a 1 px \
                 walk from MICRO to TITLE and then a jump to DISPLAY — body \
                 copy that should follow the host theme uses Theme::font_size \
                 instead of anything on this card."
            }
            Self::MenuWidth => {
                "Each surface is drawn at its full width, none of them scaled \
                 down, so the five widths a dropdown, popover, or picker can \
                 take are directly comparable. Each one is a menu row tall and \
                 turned at Radius::MENU."
            }
            Self::Breakpoint => {
                "The two container widths at which a panel switches layout \
                 density, drawn as rulers at full width — the widest specimens \
                 on the sheet. Below roughly 750 px of story viewport they run \
                 past the card edge rather than shrinking, because the value is \
                 the whole point of the sheet."
            }
        }
    }

    /// Every constant in the group, read from the library rather than
    /// retyped, in the order tokens.rs declares them.
    fn tokens(self) -> Vec<TokenSpec> {
        match self {
            Self::Space => vec![
                TokenSpec::new("Space::NONE", Space::NONE),
                TokenSpec::new("Space::XS", Space::XS),
                TokenSpec::new("Space::SM", Space::SM),
                TokenSpec::new("Space::MD", Space::MD),
                TokenSpec::new("Space::LG", Space::LG),
            ],
            Self::RowHeight => vec![
                TokenSpec::new("RowHeight::FIELD", RowHeight::FIELD).hint("inspector rows"),
                TokenSpec::new("RowHeight::LIST", RowHeight::LIST).hint("list rows"),
                TokenSpec::new("RowHeight::MENU", RowHeight::MENU)
                    .hint("menu items")
                    .off_step(),
                TokenSpec::new("RowHeight::PAGE", RowHeight::PAGE).hint("page entries"),
                TokenSpec::new("RowHeight::FLYOUT", RowHeight::FLYOUT)
                    .hint("flyouts")
                    .off_step(),
                TokenSpec::new("RowHeight::SECTION_HEADER", RowHeight::SECTION_HEADER)
                    .hint("section headers"),
            ],
            Self::ControlSize => vec![
                TokenSpec::new("ControlSize::INLINE", ControlSize::INLINE)
                    .hint("inline affordance inside a field"),
                TokenSpec::new("ControlSize::CHROME", ControlSize::CHROME)
                    .hint("panel chrome button"),
                TokenSpec::new("ControlSize::TOOL", ControlSize::TOOL).hint("toolbar tool"),
            ],
            Self::Radius => vec![
                TokenSpec::new("Radius::CONTROL", Radius::CONTROL),
                TokenSpec::new("Radius::MENU", Radius::MENU),
            ],
            Self::IconSize => vec![
                TokenSpec::new("IconSize::XS", IconSize::XS),
                TokenSpec::new("IconSize::SM", IconSize::SM).off_step(),
                TokenSpec::new("IconSize::MD", IconSize::MD),
            ],
            Self::TypeScale => vec![
                TokenSpec::new("TypeScale::MICRO", TypeScale::MICRO),
                TokenSpec::new("TypeScale::CAPTION", TypeScale::CAPTION),
                TokenSpec::new("TypeScale::BODY", TypeScale::BODY),
                TokenSpec::new("TypeScale::LABEL", TypeScale::LABEL),
                TokenSpec::new("TypeScale::TITLE", TypeScale::TITLE).off_step(),
                TokenSpec::new("TypeScale::DISPLAY", TypeScale::DISPLAY),
            ],
            Self::MenuWidth => vec![
                TokenSpec::new("MenuWidth::NARROW", MenuWidth::NARROW),
                TokenSpec::new("MenuWidth::STANDARD", MenuWidth::STANDARD),
                TokenSpec::new("MenuWidth::POPOVER", MenuWidth::POPOVER),
                TokenSpec::new("MenuWidth::PICKER", MenuWidth::PICKER),
                TokenSpec::new("MenuWidth::PICKER_WIDE", MenuWidth::PICKER_WIDE),
            ],
            Self::Breakpoint => vec![
                TokenSpec::new("Breakpoint::COMPACT", Breakpoint::COMPACT),
                TokenSpec::new("Breakpoint::WIDE", Breakpoint::WIDE),
            ],
        }
    }

    /// The constant in this group spelled `name`.
    fn token(self, name: &str) -> TokenSpec {
        self.tokens()
            .into_iter()
            .find(|token| token.name == name)
            .expect("every off-step comparison names a constant the group lists")
    }
}

/// The groups whose constants otherwise hold a 4 px grid, so a value that is
/// not a multiple of four is a deliberate deviation rather than a ramp of its
/// own. `TypeScale` is excluded: it walks in single pixels.
#[cfg(test)]
const FOUR_PX_GRID_GROUPS: &[TokenGroup] = &[
    TokenGroup::Space,
    TokenGroup::RowHeight,
    TokenGroup::ControlSize,
    TokenGroup::Radius,
    TokenGroup::IconSize,
    TokenGroup::MenuWidth,
    TokenGroup::Breakpoint,
];

/// Each off-step constant with the on-grid neighbour below it and the one
/// above it, so the closing card can draw the deviation instead of asserting
/// it: the element id, the group, and the three constants low → off-step →
/// high.
fn off_step_comparisons() -> [(&'static str, TokenGroup, [&'static str; 3]); 4] {
    [
        (
            "tokens-off-step-menu",
            TokenGroup::RowHeight,
            ["RowHeight::LIST", "RowHeight::MENU", "RowHeight::PAGE"],
        ),
        (
            "tokens-off-step-flyout",
            TokenGroup::RowHeight,
            [
                "RowHeight::PAGE",
                "RowHeight::FLYOUT",
                "RowHeight::SECTION_HEADER",
            ],
        ),
        (
            "tokens-off-step-icon",
            TokenGroup::IconSize,
            ["IconSize::XS", "IconSize::SM", "IconSize::MD"],
        ),
        (
            "tokens-off-step-title",
            TokenGroup::TypeScale,
            ["TypeScale::LABEL", "TypeScale::TITLE", "TypeScale::DISPLAY"],
        ),
    ]
}

/// `24` for a whole constant, `24.5` for one that ever stops being whole.
fn format_px(value: f32) -> String {
    if value.fract() == 0. {
        format!("{} px", value as i64)
    } else {
        format!("{value} px")
    }
}

/// The geometry itself, drawn at the token's value.
fn token_specimen(group: TokenGroup, value: f32, cx: &mut Context<Storybook>) -> AnyElement {
    match group {
        TokenGroup::Space => div()
            .w(px(value))
            .h(px(BAR_HEIGHT))
            .flex_none()
            .rounded(px(2.))
            .bg(fanta_gpui::atoms::SemanticColor::BackgroundBrand
                .resolve(cx)
                .opacity(0.7))
            .into_any_element(),
        TokenGroup::RowHeight => div()
            .w(px(ROW_STRIP_WIDTH))
            .h(px(value))
            .flex_none()
            .rounded(px(Radius::CONTROL))
            .border_1()
            .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
            .bg(fanta_gpui::atoms::SemanticColor::BackgroundSecondary
                .resolve(cx)
                .opacity(0.55))
            .into_any_element(),
        TokenGroup::ControlSize => {
            let glyph = if value <= ControlSize::INLINE {
                IconSize::XS
            } else {
                IconSize::MD
            };
            div()
                .size(px(value))
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(Radius::CONTROL))
                .border_1()
                .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
                .bg(fanta_gpui::atoms::SemanticColor::BackgroundSecondary
                    .resolve(cx)
                    .opacity(0.55))
                .child(render_lucide_icon(
                    LucideIcon::Settings2,
                    fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx),
                    glyph,
                ))
                .into_any_element()
        }
        TokenGroup::Radius => div()
            .w(px(CORNER_SPECIMEN_WIDTH))
            .h(px(CORNER_SPECIMEN_HEIGHT))
            .flex_none()
            .rounded(px(value))
            .border_1()
            .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
            .bg(fanta_gpui::atoms::SemanticColor::BackgroundSecondary
                .resolve(cx)
                .opacity(0.55))
            .into_any_element(),
        TokenGroup::IconSize => div()
            .flex_none()
            .child(render_lucide_icon(
                LucideIcon::Frame,
                fanta_gpui::atoms::SemanticColor::Text.resolve(cx),
                value,
            ))
            .into_any_element(),
        TokenGroup::TypeScale => div()
            .flex_none()
            .text_size(px(value))
            .text_color(fanta_gpui::atoms::SemanticColor::Text.resolve(cx))
            .child(TYPE_SAMPLE)
            .into_any_element(),
        TokenGroup::MenuWidth => div()
            .w(px(value))
            .h(px(RowHeight::MENU))
            .flex_none()
            .rounded(px(Radius::MENU))
            .border_1()
            .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
            .bg(fanta_gpui::atoms::SemanticColor::BackgroundMenu.resolve(cx))
            .into_any_element(),
        TokenGroup::Breakpoint => h_flex()
            .w(px(value))
            .h(px(BAR_HEIGHT))
            .flex_none()
            .items_center()
            .child(
                div().w(px(1.)).h(px(BAR_HEIGHT)).flex_none().bg(
                    fanta_gpui::atoms::SemanticColor::BackgroundBrand
                        .resolve(cx)
                        .opacity(0.7),
                ),
            )
            .child(
                div()
                    .flex_1()
                    .h(px(1.))
                    .bg(fanta_gpui::atoms::SemanticColor::Border.resolve(cx)),
            )
            .child(
                div().w(px(1.)).h(px(BAR_HEIGHT)).flex_none().bg(
                    fanta_gpui::atoms::SemanticColor::BackgroundBrand
                        .resolve(cx)
                        .opacity(0.7),
                ),
            )
            .into_any_element(),
    }
}

/// The badge marking a constant tokens.rs documents as deliberately off-step.
fn off_step_badge(cx: &mut Context<Storybook>) -> AnyElement {
    div()
        .flex_none()
        .px(px(Space::XS))
        .py(px(1.))
        .rounded(px(Radius::CONTROL))
        .border_1()
        .border_color(
            fanta_gpui::atoms::SemanticColor::BackgroundBrand
                .resolve(cx)
                .opacity(0.5),
        )
        .text_size(px(TypeScale::MICRO))
        .text_color(fanta_gpui::atoms::SemanticColor::BackgroundBrand.resolve(cx))
        .child("off-step")
        .into_any_element()
}

/// One line of the sheet: path, value, the geometry at that value, then the
/// off-step badge and the site the doc comment names, when there are any.
fn token_line(group: TokenGroup, token: TokenSpec, cx: &mut Context<Storybook>) -> AnyElement {
    let specimen = token_specimen(group, token.value, cx);
    h_flex()
        .w_full()
        .items_center()
        .gap(px(COLUMN_GAP))
        .child(
            div()
                .w(px(NAME_COLUMN))
                .flex_none()
                .text_size(px(TypeScale::CAPTION))
                .font_medium()
                .child(token.name),
        )
        .child(
            div()
                .w(px(VALUE_COLUMN))
                .flex_none()
                .text_right()
                .text_size(px(TypeScale::CAPTION))
                .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                .child(format_px(token.value)),
        )
        .child(specimen)
        .when(token.off_step, |line| line.child(off_step_badge(cx)))
        .when_some(token.hint, |line, hint| {
            line.child(
                div()
                    .flex_none()
                    .text_size(px(TypeScale::MICRO))
                    .text_color(
                        fanta_gpui::atoms::SemanticColor::TextTertiary
                            .resolve(cx)
                            .opacity(0.8),
                    )
                    .child(hint),
            )
        })
        .into_any_element()
}

/// The stack of lines inside one card, with the shared 8 px line gap.
fn token_lines(lines: Vec<AnyElement>) -> AnyElement {
    v_flex()
        .w_full()
        .gap(px(Space::SM))
        .children(lines)
        .into_any_element()
}

pub(crate) struct TokensScreen {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) last_action: SharedString,
}

impl TokensScreen {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            last_action: "Ready — every specimen below is drawn at its own pixel value".into(),
        }
    }
}

impl Storybook {
    /// The column legend: the three columns named once, at the widths every
    /// line below uses, plus the badge the off-step constants carry.
    fn render_tokens_legend(&self, cx: &mut Context<Self>) -> AnyElement {
        h_flex()
            .w_full()
            .items_center()
            .gap(px(COLUMN_GAP))
            .child(
                div()
                    .w(px(NAME_COLUMN))
                    .flex_none()
                    .text_size(px(TypeScale::MICRO))
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child("WHAT A HOST TYPES"),
            )
            .child(
                div()
                    .w(px(VALUE_COLUMN))
                    .flex_none()
                    .text_right()
                    .text_size(px(TypeScale::MICRO))
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child("VALUE"),
            )
            .child(
                div()
                    .flex_none()
                    .text_size(px(TypeScale::MICRO))
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child("DRAWN AT THAT VALUE"),
            )
            .child(off_step_badge(cx))
            .into_any_element()
    }

    /// One card per group: every constant it declares, one per line.
    fn render_tokens_group_card(&self, group: TokenGroup, cx: &mut Context<Self>) -> AnyElement {
        let lines = group
            .tokens()
            .into_iter()
            .map(|token| token_line(group, token, cx))
            .collect::<Vec<_>>();
        specimen_card(
            group.card_id(),
            group.title(),
            group.copy(),
            specimen_rows(vec![specimen_row(
                group.row_id(),
                format!(
                    "{} · {} constants, drawn at size",
                    group.label(),
                    lines.len()
                ),
                vec![token_lines(lines)],
                cx,
            )]),
            cx,
        )
    }

    /// The closing card: each off-step constant drawn between the two on-grid
    /// neighbours it sits between.
    fn render_tokens_off_step_card(&self, cx: &mut Context<Self>) -> AnyElement {
        let rows = off_step_comparisons()
            .into_iter()
            .map(|(row_id, group, names)| {
                let lines = names
                    .into_iter()
                    .map(|name| token_line(group, group.token(name), cx))
                    .collect::<Vec<_>>();
                let flagged = group.token(names[1]);
                specimen_row(
                    row_id,
                    format!("{} sits between its neighbours", flagged.name),
                    vec![token_lines(lines)],
                    cx,
                )
            })
            .collect::<Vec<_>>();
        specimen_card(
            "tokens-off-step",
            "Off the step, on purpose",
            "tokens.rs marks four constants as deliberately preserved shipping \
             geometry pending a post-ship polish pass, and each is drawn here \
             between the two neighbours it sits between. Three of them break a \
             4 px grid their group otherwise holds — RowHeight::MENU one short \
             of PAGE, RowHeight::FLYOUT two short of SECTION_HEADER, \
             IconSize::SM two past XS — and TypeScale::TITLE carries the same \
             note on the text ramp. Read them as geometry the host cutover \
             keeps, not as rounding to fix.",
            specimen_rows(rows),
            cx,
        )
    }

    pub(crate) fn render_tokens_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let legend = self.render_tokens_legend(cx);
        let legend_card = specimen_card(
            "tokens-legend",
            "How to read this sheet",
            "Every line is one constant of fanta_gpui::atoms::tokens: the path \
             a host types, the number, and then the geometry drawn at that \
             number. Nothing is scaled to fit, so two tokens can be compared by \
             eye. A note at the end of a line is the site that group's own doc \
             comment names for that step.",
            specimen_rows(vec![specimen_row(
                "tokens-legend-row",
                "The three columns every card below repeats",
                vec![legend],
                cx,
            )]),
            cx,
        );
        let mut sheet = specimen_story_root("storybook-tokens")
            // Nothing on the sheet rescales, so the columns hold their width
            // instead of compressing the specimens when the viewport narrows.
            .min_w(px(TOKENS_SHEET_MIN_WIDTH))
            .child(legend_card);
        for group in TokenGroup::ALL.iter().copied() {
            sheet = sheet.child(self.render_tokens_group_card(group, cx));
        }
        sheet
            .child(self.render_tokens_off_step_card(cx))
            .into_any_element()
    }

    pub(crate) fn render_tokens_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.spec_reference(
            "storybook-reference-tokens",
            self.render_tokens_story(cx),
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_group_lists_its_constants_in_ascending_order() {
        for group in TokenGroup::ALL.iter().copied() {
            let tokens = group.tokens();
            assert!(
                tokens.len() >= 2,
                "{} should be worth a card of its own",
                group.label()
            );
            for pair in tokens.windows(2) {
                assert!(
                    pair[0].value < pair[1].value,
                    "{} must be drawn below {} on the sheet",
                    pair[0].name,
                    pair[1].name
                );
            }
        }
    }

    #[test]
    fn every_token_name_is_the_path_a_host_types() {
        for group in TokenGroup::ALL.iter().copied() {
            let prefix = format!("{}::", group.label());
            for token in group.tokens() {
                assert!(
                    token.name.starts_with(&prefix),
                    "{} should be spelled as a {prefix} path",
                    token.name
                );
            }
        }
    }

    #[test]
    fn the_four_px_grid_breaks_only_where_tokens_rs_says_it_does() {
        let off_grid = FOUR_PX_GRID_GROUPS
            .iter()
            .copied()
            .flat_map(TokenGroup::tokens)
            .filter(|token| token.value % 4. != 0.)
            .map(|token| token.name)
            .collect::<Vec<_>>();
        assert_eq!(
            off_grid,
            vec!["RowHeight::MENU", "RowHeight::FLYOUT", "IconSize::SM"],
            "the sheet's 4 px grid claim must match the library's constants"
        );
    }

    #[test]
    fn the_badged_constants_are_the_ones_tokens_rs_documents() {
        let badged = TokenGroup::ALL
            .iter()
            .copied()
            .flat_map(TokenGroup::tokens)
            .filter(|token| token.off_step)
            .map(|token| token.name)
            .collect::<Vec<_>>();
        assert_eq!(
            badged,
            vec![
                "RowHeight::MENU",
                "RowHeight::FLYOUT",
                "IconSize::SM",
                "TypeScale::TITLE",
            ],
            "the off-step badge must follow the library's own doc comments"
        );
    }

    #[test]
    fn each_comparison_frames_its_off_step_constant() {
        for (_, group, names) in off_step_comparisons() {
            let [low, flagged, high] = names.map(|name| group.token(name));
            assert!(
                flagged.off_step,
                "{} is the constant the comparison exists to show",
                flagged.name
            );
            assert!(!low.off_step && !high.off_step);
            assert!(
                low.value < flagged.value && flagged.value < high.value,
                "{} must be drawn between {} and {}",
                flagged.name,
                low.name,
                high.name
            );
        }
    }

    #[test]
    fn the_sheet_is_registered_wide_enough_for_its_widest_specimen() {
        let widest = [Breakpoint::WIDE, MenuWidth::PICKER_WIDE, ROW_STRIP_WIDTH]
            .into_iter()
            .fold(0., f32::max);
        assert!(
            TOKENS_SHEET_MIN_WIDTH
                >= NAME_COLUMN + VALUE_COLUMN + 2. * COLUMN_GAP + widest + SHEET_PADDING,
            "nothing on the sheet rescales, so the story must be registered at \
             least {TOKENS_SHEET_MIN_WIDTH} px wide"
        );
    }
}
