// Bundled theme JSON uses the editor settings schema, but this decoder must remain
// usable without linking the editor's settings service. The editor keeps a parity
// test over every bundled theme file so these wire-format fields stay aligned.
use std::{fmt::Display, sync::Arc};

use collections::IndexMap;
use gpui::{HighlightStyle, Hsla, Refineable as _};
use palette::FromColor;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::{
    AccentColors, Appearance, AppearanceContent, PlayerColor, PlayerColors, StatusColors,
    StatusColorsRefinement, SyntaxTheme, SystemColors, Theme, ThemeColors, ThemeColorsRefinement,
    ThemeFamily, ThemeStyles, default_color_scales,
};

#[derive(Debug, Deserialize)]
struct ThemeFamilyContent {
    name: String,
    author: String,
    themes: Vec<ThemeContent>,
}

#[derive(Debug, Deserialize)]
struct ThemeContent {
    name: String,
    appearance: AppearanceContent,
    style: ThemeStyleContent,
}

const LIGHT_DIFF_HUNK_FILLED_OPACITY: f32 = 0.16;
const LIGHT_DIFF_HUNK_HOLLOW_BACKGROUND_OPACITY: f32 = 0.08;
const LIGHT_DIFF_HUNK_HOLLOW_BORDER_OPACITY: f32 = 0.48;
const DARK_DIFF_HUNK_FILLED_OPACITY: f32 = 0.12;
const DARK_DIFF_HUNK_HOLLOW_BACKGROUND_OPACITY: f32 = 0.06;
const DARK_DIFF_HUNK_HOLLOW_BORDER_OPACITY: f32 = 0.36;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ThemeStyleContent {
    #[serde(rename = "background.appearance")]
    pub window_background_appearance: Option<WindowBackgroundContent>,

    #[serde(default)]
    pub accents: Vec<AccentContent>,

    #[serde(flatten, default)]
    pub colors: ThemeColorsContent,

    #[serde(flatten, default)]
    pub status: StatusColorsContent,

    #[serde(default)]
    pub players: Vec<PlayerColorContent>,

    /// The styles for syntax nodes.
    #[serde(default)]
    pub syntax: IndexMap<String, HighlightStyleContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccentContent(pub Option<String>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerColorContent {
    pub cursor: Option<String>,
    pub background: Option<String>,
    pub selection: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ThemeColorsContent {
    /// Border color. Used for most borders, is usually a high contrast color.
    #[serde(rename = "border")]
    pub border: Option<String>,

    /// Border color. Used for deemphasized borders, like a visual divider between two sections
    #[serde(rename = "border.variant")]
    pub border_variant: Option<String>,

    /// Border color. Used for focused elements, like keyboard focused list item.
    #[serde(rename = "border.focused")]
    pub border_focused: Option<String>,

    /// Border color. Used for selected elements, like an active search filter or selected checkbox.
    #[serde(rename = "border.selected")]
    pub border_selected: Option<String>,

    /// Border color. Used for transparent borders. Used for placeholder borders when an element gains a border on state change.
    #[serde(rename = "border.transparent")]
    pub border_transparent: Option<String>,

    /// Border color. Used for disabled elements, like a disabled input or button.
    #[serde(rename = "border.disabled")]
    pub border_disabled: Option<String>,

    /// Background color. Used for elevated surfaces, like a context menu, popup, or dialog.
    #[serde(rename = "elevated_surface.background")]
    pub elevated_surface_background: Option<String>,

    /// Background Color. Used for grounded surfaces like a panel or tab.
    #[serde(rename = "surface.background")]
    pub surface_background: Option<String>,

    /// Background Color. Used for the app background and blank panels or windows.
    #[serde(rename = "background")]
    pub background: Option<String>,

    /// Background Color. Used for the background of an element that should have a different background than the surface it's on.
    ///
    /// Elements might include: Buttons, Inputs, Checkboxes, Radio Buttons...
    ///
    /// For an element that should have the same background as the surface it's on, use `ghost_element_background`.
    #[serde(rename = "element.background")]
    pub element_background: Option<String>,

    /// Background Color. Used for the hover state of an element that should have a different background than the surface it's on.
    ///
    /// Hover states are triggered by the mouse entering an element, or a finger touching an element on a touch screen.
    #[serde(rename = "element.hover")]
    pub element_hover: Option<String>,

    /// Background Color. Used for the active state of an element that should have a different background than the surface it's on.
    ///
    /// Active states are triggered by the mouse button being pressed down on an element, or the Return button or other activator being pressed.
    #[serde(rename = "element.active")]
    pub element_active: Option<String>,

    /// Background Color. Used for the selected state of an element that should have a different background than the surface it's on.
    ///
    /// Selected states are triggered by the element being selected (or "activated") by the user.
    ///
    /// This could include a selected checkbox, a toggleable button that is toggled on, etc.
    #[serde(rename = "element.selected")]
    pub element_selected: Option<String>,

    /// Background Color. Used for the disabled state of an element that should have a different background than the surface it's on.
    ///
    /// Disabled states are shown when a user cannot interact with an element, like a disabled button or input.
    #[serde(rename = "element.disabled")]
    pub element_disabled: Option<String>,

    /// Background Color. Used for the background of selections in a UI element.
    #[serde(rename = "element.selection_background")]
    pub element_selection_background: Option<String>,

    /// Background Color. Used for the area that shows where a dragged element will be dropped.
    #[serde(rename = "drop_target.background")]
    pub drop_target_background: Option<String>,

    /// Border Color. Used for the border that shows where a dragged element will be dropped.
    #[serde(rename = "drop_target.border")]
    pub drop_target_border: Option<String>,

    /// Used for the background of a ghost element that should have the same background as the surface it's on.
    ///
    /// Elements might include: Buttons, Inputs, Checkboxes, Radio Buttons...
    ///
    /// For an element that should have a different background than the surface it's on, use `element_background`.
    #[serde(rename = "ghost_element.background")]
    pub ghost_element_background: Option<String>,

    /// Background Color. Used for the hover state of a ghost element that should have the same background as the surface it's on.
    ///
    /// Hover states are triggered by the mouse entering an element, or a finger touching an element on a touch screen.
    #[serde(rename = "ghost_element.hover")]
    pub ghost_element_hover: Option<String>,

    /// Background Color. Used for the active state of a ghost element that should have the same background as the surface it's on.
    ///
    /// Active states are triggered by the mouse button being pressed down on an element, or the Return button or other activator being pressed.
    #[serde(rename = "ghost_element.active")]
    pub ghost_element_active: Option<String>,

    /// Background Color. Used for the selected state of a ghost element that should have the same background as the surface it's on.
    ///
    /// Selected states are triggered by the element being selected (or "activated") by the user.
    ///
    /// This could include a selected checkbox, a toggleable button that is toggled on, etc.
    #[serde(rename = "ghost_element.selected")]
    pub ghost_element_selected: Option<String>,

    /// Background Color. Used for the disabled state of a ghost element that should have the same background as the surface it's on.
    ///
    /// Disabled states are shown when a user cannot interact with an element, like a disabled button or input.
    #[serde(rename = "ghost_element.disabled")]
    pub ghost_element_disabled: Option<String>,

    /// Text Color. Default text color used for most text.
    #[serde(rename = "text")]
    pub text: Option<String>,

    /// Text Color. Color of muted or deemphasized text. It is a subdued version of the standard text color.
    #[serde(rename = "text.muted")]
    pub text_muted: Option<String>,

    /// Text Color. Color of the placeholder text typically shown in input fields to guide the user to enter valid data.
    #[serde(rename = "text.placeholder")]
    pub text_placeholder: Option<String>,

    /// Text Color. Color used for text denoting disabled elements. Typically, the color is faded or grayed out to emphasize the disabled state.
    #[serde(rename = "text.disabled")]
    pub text_disabled: Option<String>,

    /// Text Color. Color used for emphasis or highlighting certain text, like an active filter or a matched character in a search.
    #[serde(rename = "text.accent")]
    pub text_accent: Option<String>,

    /// Fill Color. Used for the default fill color of an icon.
    #[serde(rename = "icon")]
    pub icon: Option<String>,

    /// Fill Color. Used for the muted or deemphasized fill color of an icon.
    ///
    /// This might be used to show an icon in an inactive pane, or to deemphasize a series of icons to give them less visual weight.
    #[serde(rename = "icon.muted")]
    pub icon_muted: Option<String>,

    /// Fill Color. Used for the disabled fill color of an icon.
    ///
    /// Disabled states are shown when a user cannot interact with an element, like a icon button.
    #[serde(rename = "icon.disabled")]
    pub icon_disabled: Option<String>,

    /// Fill Color. Used for the placeholder fill color of an icon.
    ///
    /// This might be used to show an icon in an input that disappears when the user enters text.
    #[serde(rename = "icon.placeholder")]
    pub icon_placeholder: Option<String>,

    /// Fill Color. Used for the accent fill color of an icon.
    ///
    /// This might be used to show when a toggleable icon button is selected.
    #[serde(rename = "icon.accent")]
    pub icon_accent: Option<String>,

    /// Color used to accent some of the debuggers elements
    /// Only accent breakpoint & breakpoint related symbols right now
    #[serde(rename = "debugger.accent")]
    pub debugger_accent: Option<String>,

    #[serde(rename = "status_bar.background")]
    pub status_bar_background: Option<String>,

    #[serde(rename = "title_bar.background")]
    pub title_bar_background: Option<String>,

    #[serde(rename = "title_bar.inactive_background")]
    pub title_bar_inactive_background: Option<String>,

    #[serde(rename = "toolbar.background")]
    pub toolbar_background: Option<String>,

    #[serde(rename = "tab_bar.background")]
    pub tab_bar_background: Option<String>,

    #[serde(rename = "tab.inactive_background")]
    pub tab_inactive_background: Option<String>,

    #[serde(rename = "tab.active_background")]
    pub tab_active_background: Option<String>,

    #[serde(rename = "search.match_background")]
    pub search_match_background: Option<String>,

    #[serde(rename = "search.active_match_background")]
    pub search_active_match_background: Option<String>,

    #[serde(rename = "panel.background")]
    pub panel_background: Option<String>,

    #[serde(rename = "panel.focused_border")]
    pub panel_focused_border: Option<String>,

    #[serde(rename = "panel.indent_guide")]
    pub panel_indent_guide: Option<String>,

    #[serde(rename = "panel.indent_guide_hover")]
    pub panel_indent_guide_hover: Option<String>,

    #[serde(rename = "panel.indent_guide_active")]
    pub panel_indent_guide_active: Option<String>,

    #[serde(rename = "panel.overlay_background")]
    pub panel_overlay_background: Option<String>,

    #[serde(rename = "panel.overlay_hover")]
    pub panel_overlay_hover: Option<String>,

    #[serde(rename = "pane.focused_border")]
    pub pane_focused_border: Option<String>,

    #[serde(rename = "pane_group.border")]
    pub pane_group_border: Option<String>,

    /// The deprecated version of `scrollbar.thumb.background`.
    ///
    /// Don't use this field.
    #[serde(rename = "scrollbar_thumb.background", skip_serializing)]
    pub deprecated_scrollbar_thumb_background: Option<String>,

    /// The color of the scrollbar thumb.
    #[serde(rename = "scrollbar.thumb.background")]
    pub scrollbar_thumb_background: Option<String>,

    /// The color of the scrollbar thumb when hovered over.
    #[serde(rename = "scrollbar.thumb.hover_background")]
    pub scrollbar_thumb_hover_background: Option<String>,

    /// The color of the scrollbar thumb whilst being actively dragged.
    #[serde(rename = "scrollbar.thumb.active_background")]
    pub scrollbar_thumb_active_background: Option<String>,

    /// The border color of the scrollbar thumb.
    #[serde(rename = "scrollbar.thumb.border")]
    pub scrollbar_thumb_border: Option<String>,

    /// The background color of the scrollbar track.
    #[serde(rename = "scrollbar.track.background")]
    pub scrollbar_track_background: Option<String>,

    /// The border color of the scrollbar track.
    #[serde(rename = "scrollbar.track.border")]
    pub scrollbar_track_border: Option<String>,

    /// The color of the minimap thumb.
    #[serde(rename = "minimap.thumb.background")]
    pub minimap_thumb_background: Option<String>,

    /// The color of the minimap thumb when hovered over.
    #[serde(rename = "minimap.thumb.hover_background")]
    pub minimap_thumb_hover_background: Option<String>,

    /// The color of the minimap thumb whilst being actively dragged.
    #[serde(rename = "minimap.thumb.active_background")]
    pub minimap_thumb_active_background: Option<String>,

    /// The border color of the minimap thumb.
    #[serde(rename = "minimap.thumb.border")]
    pub minimap_thumb_border: Option<String>,

    #[serde(rename = "editor.foreground")]
    pub editor_foreground: Option<String>,

    #[serde(rename = "editor.background")]
    pub editor_background: Option<String>,

    #[serde(rename = "editor.gutter.background")]
    pub editor_gutter_background: Option<String>,

    #[serde(rename = "editor.subheader.background")]
    pub editor_subheader_background: Option<String>,

    #[serde(rename = "editor.active_line.background")]
    pub editor_active_line_background: Option<String>,

    #[serde(rename = "editor.highlighted_line.background")]
    pub editor_highlighted_line_background: Option<String>,

    /// Background of active line of debugger
    #[serde(rename = "editor.debugger_active_line.background")]
    pub editor_debugger_active_line_background: Option<String>,

    /// Text Color. Used for the text of the line number in the editor gutter.
    #[serde(rename = "editor.line_number")]
    pub editor_line_number: Option<String>,

    /// Text Color. Used for the text of the line number in the editor gutter when the line is highlighted.
    #[serde(rename = "editor.active_line_number")]
    pub editor_active_line_number: Option<String>,

    /// Text Color. Used for the text of the line number in the editor gutter when the line is hovered over.
    #[serde(rename = "editor.hover_line_number")]
    pub editor_hover_line_number: Option<String>,

    /// Text Color. Used to mark invisible characters in the editor.
    ///
    /// Example: spaces, tabs, carriage returns, etc.
    #[serde(rename = "editor.invisible")]
    pub editor_invisible: Option<String>,

    #[serde(rename = "editor.wrap_guide")]
    pub editor_wrap_guide: Option<String>,

    #[serde(rename = "editor.active_wrap_guide")]
    pub editor_active_wrap_guide: Option<String>,

    #[serde(rename = "editor.indent_guide")]
    pub editor_indent_guide: Option<String>,

    #[serde(rename = "editor.indent_guide_active")]
    pub editor_indent_guide_active: Option<String>,

    /// Read-access of a symbol, like reading a variable.
    ///
    /// A document highlight is a range inside a text document which deserves
    /// special attention. Usually a document highlight is visualized by changing
    /// the background color of its range.
    #[serde(rename = "editor.document_highlight.read_background")]
    pub editor_document_highlight_read_background: Option<String>,

    /// Read-access of a symbol, like reading a variable.
    ///
    /// A document highlight is a range inside a text document which deserves
    /// special attention. Usually a document highlight is visualized by changing
    /// the background color of its range.
    #[serde(rename = "editor.document_highlight.write_background")]
    pub editor_document_highlight_write_background: Option<String>,

    /// Highlighted brackets background color.
    ///
    /// Matching brackets in the cursor scope are highlighted with this background color.
    #[serde(rename = "editor.document_highlight.bracket_background")]
    pub editor_document_highlight_bracket_background: Option<String>,

    /// Filled background color for added diff hunk row highlights in the editor.
    #[serde(rename = "editor.diff_hunk.added.background")]
    pub editor_diff_hunk_added_background: Option<String>,

    /// Hollow background color for added diff hunk row highlights in the editor.
    #[serde(rename = "editor.diff_hunk.added.hollow_background")]
    pub editor_diff_hunk_added_hollow_background: Option<String>,

    /// Hollow border color for added diff hunk row highlights in the editor.
    #[serde(rename = "editor.diff_hunk.added.hollow_border")]
    pub editor_diff_hunk_added_hollow_border: Option<String>,

    /// Filled background color for deleted diff hunk row highlights in the editor.
    #[serde(rename = "editor.diff_hunk.deleted.background")]
    pub editor_diff_hunk_deleted_background: Option<String>,

    /// Hollow background color for deleted diff hunk row highlights in the editor.
    #[serde(rename = "editor.diff_hunk.deleted.hollow_background")]
    pub editor_diff_hunk_deleted_hollow_background: Option<String>,

    /// Hollow border color for deleted diff hunk row highlights in the editor.
    #[serde(rename = "editor.diff_hunk.deleted.hollow_border")]
    pub editor_diff_hunk_deleted_hollow_border: Option<String>,

    /// Terminal background color.
    #[serde(rename = "terminal.background")]
    pub terminal_background: Option<String>,

    /// Terminal foreground color.
    #[serde(rename = "terminal.foreground")]
    pub terminal_foreground: Option<String>,

    /// Terminal ANSI background color.
    #[serde(rename = "terminal.ansi.background")]
    pub terminal_ansi_background: Option<String>,

    /// Bright terminal foreground color.
    #[serde(rename = "terminal.bright_foreground")]
    pub terminal_bright_foreground: Option<String>,

    /// Dim terminal foreground color.
    #[serde(rename = "terminal.dim_foreground")]
    pub terminal_dim_foreground: Option<String>,

    /// Black ANSI terminal color.
    #[serde(rename = "terminal.ansi.black")]
    pub terminal_ansi_black: Option<String>,

    /// Bright black ANSI terminal color.
    #[serde(rename = "terminal.ansi.bright_black")]
    pub terminal_ansi_bright_black: Option<String>,

    /// Dim black ANSI terminal color.
    #[serde(rename = "terminal.ansi.dim_black")]
    pub terminal_ansi_dim_black: Option<String>,

    /// Red ANSI terminal color.
    #[serde(rename = "terminal.ansi.red")]
    pub terminal_ansi_red: Option<String>,

    /// Bright red ANSI terminal color.
    #[serde(rename = "terminal.ansi.bright_red")]
    pub terminal_ansi_bright_red: Option<String>,

    /// Dim red ANSI terminal color.
    #[serde(rename = "terminal.ansi.dim_red")]
    pub terminal_ansi_dim_red: Option<String>,

    /// Green ANSI terminal color.
    #[serde(rename = "terminal.ansi.green")]
    pub terminal_ansi_green: Option<String>,

    /// Bright green ANSI terminal color.
    #[serde(rename = "terminal.ansi.bright_green")]
    pub terminal_ansi_bright_green: Option<String>,

    /// Dim green ANSI terminal color.
    #[serde(rename = "terminal.ansi.dim_green")]
    pub terminal_ansi_dim_green: Option<String>,

    /// Yellow ANSI terminal color.
    #[serde(rename = "terminal.ansi.yellow")]
    pub terminal_ansi_yellow: Option<String>,

    /// Bright yellow ANSI terminal color.
    #[serde(rename = "terminal.ansi.bright_yellow")]
    pub terminal_ansi_bright_yellow: Option<String>,

    /// Dim yellow ANSI terminal color.
    #[serde(rename = "terminal.ansi.dim_yellow")]
    pub terminal_ansi_dim_yellow: Option<String>,

    /// Blue ANSI terminal color.
    #[serde(rename = "terminal.ansi.blue")]
    pub terminal_ansi_blue: Option<String>,

    /// Bright blue ANSI terminal color.
    #[serde(rename = "terminal.ansi.bright_blue")]
    pub terminal_ansi_bright_blue: Option<String>,

    /// Dim blue ANSI terminal color.
    #[serde(rename = "terminal.ansi.dim_blue")]
    pub terminal_ansi_dim_blue: Option<String>,

    /// Magenta ANSI terminal color.
    #[serde(rename = "terminal.ansi.magenta")]
    pub terminal_ansi_magenta: Option<String>,

    /// Bright magenta ANSI terminal color.
    #[serde(rename = "terminal.ansi.bright_magenta")]
    pub terminal_ansi_bright_magenta: Option<String>,

    /// Dim magenta ANSI terminal color.
    #[serde(rename = "terminal.ansi.dim_magenta")]
    pub terminal_ansi_dim_magenta: Option<String>,

    /// Cyan ANSI terminal color.
    #[serde(rename = "terminal.ansi.cyan")]
    pub terminal_ansi_cyan: Option<String>,

    /// Bright cyan ANSI terminal color.
    #[serde(rename = "terminal.ansi.bright_cyan")]
    pub terminal_ansi_bright_cyan: Option<String>,

    /// Dim cyan ANSI terminal color.
    #[serde(rename = "terminal.ansi.dim_cyan")]
    pub terminal_ansi_dim_cyan: Option<String>,

    /// White ANSI terminal color.
    #[serde(rename = "terminal.ansi.white")]
    pub terminal_ansi_white: Option<String>,

    /// Bright white ANSI terminal color.
    #[serde(rename = "terminal.ansi.bright_white")]
    pub terminal_ansi_bright_white: Option<String>,

    /// Dim white ANSI terminal color.
    #[serde(rename = "terminal.ansi.dim_white")]
    pub terminal_ansi_dim_white: Option<String>,

    #[serde(rename = "link_text.hover")]
    pub link_text_hover: Option<String>,

    /// Added version control color.
    #[serde(rename = "version_control.added")]
    pub version_control_added: Option<String>,

    /// Deleted version control color.
    #[serde(rename = "version_control.deleted")]
    pub version_control_deleted: Option<String>,

    /// Modified version control color.
    #[serde(rename = "version_control.modified")]
    pub version_control_modified: Option<String>,

    /// Renamed version control color.
    #[serde(rename = "version_control.renamed")]
    pub version_control_renamed: Option<String>,

    /// Conflict version control color.
    #[serde(rename = "version_control.conflict")]
    pub version_control_conflict: Option<String>,

    /// Ignored version control color.
    #[serde(rename = "version_control.ignored")]
    pub version_control_ignored: Option<String>,

    /// Color for added words in word diffs.
    #[serde(rename = "version_control.word_added")]
    pub version_control_word_added: Option<String>,

    /// Color for deleted words in word diffs.
    #[serde(rename = "version_control.word_deleted")]
    pub version_control_word_deleted: Option<String>,

    /// Background color for row highlights of "ours" regions in merge conflicts.
    #[serde(rename = "version_control.conflict_marker.ours")]
    pub version_control_conflict_marker_ours: Option<String>,

    /// Background color for row highlights of "theirs" regions in merge conflicts.
    #[serde(rename = "version_control.conflict_marker.theirs")]
    pub version_control_conflict_marker_theirs: Option<String>,

    /// Deprecated in favor of `version_control_conflict_marker_ours`.
    #[deprecated]
    pub version_control_conflict_ours_background: Option<String>,

    /// Deprecated in favor of `version_control_conflict_marker_theirs`.
    #[deprecated]
    pub version_control_conflict_theirs_background: Option<String>,

    /// Background color for Vim Normal mode indicator.
    #[serde(rename = "vim.normal.background")]
    pub vim_normal_background: Option<String>,
    /// Background color for Vim Insert mode indicator.
    #[serde(rename = "vim.insert.background")]
    pub vim_insert_background: Option<String>,
    /// Background color for Vim Replace mode indicator.
    #[serde(rename = "vim.replace.background")]
    pub vim_replace_background: Option<String>,
    /// Background color for Vim Visual mode indicator.
    #[serde(rename = "vim.visual.background")]
    pub vim_visual_background: Option<String>,
    /// Background color for Vim Visual Line mode indicator.
    #[serde(rename = "vim.visual_line.background")]
    pub vim_visual_line_background: Option<String>,
    /// Background color for Vim Visual Block mode indicator.
    #[serde(rename = "vim.visual_block.background")]
    pub vim_visual_block_background: Option<String>,
    /// Background color for Vim yank highlight.
    #[serde(rename = "vim.yank.background")]
    pub vim_yank_background: Option<String>,
    /// Foreground color for Helix jump labels.
    #[serde(rename = "vim.helix_jump_label.foreground")]
    pub vim_helix_jump_label_foreground: Option<String>,
    /// Background color for Vim Helix Normal mode indicator.
    #[serde(rename = "vim.helix_normal.background")]
    pub vim_helix_normal_background: Option<String>,
    /// Background color for Vim Helix Select mode indicator.
    #[serde(rename = "vim.helix_select.background")]
    pub vim_helix_select_background: Option<String>,
    /// Background color for Vim Normal mode indicator.
    #[serde(rename = "vim.normal.foreground")]
    pub vim_normal_foreground: Option<String>,
    /// Foreground color for Vim Insert mode indicator.
    #[serde(rename = "vim.insert.foreground")]
    pub vim_insert_foreground: Option<String>,
    /// Foreground color for Vim Replace mode indicator.
    #[serde(rename = "vim.replace.foreground")]
    pub vim_replace_foreground: Option<String>,
    /// Foreground color for Vim Visual mode indicator.
    #[serde(rename = "vim.visual.foreground")]
    pub vim_visual_foreground: Option<String>,
    /// Foreground color for Vim Visual Line mode indicator.
    #[serde(rename = "vim.visual_line.foreground")]
    pub vim_visual_line_foreground: Option<String>,
    /// Foreground color for Vim Visual Block mode indicator.
    #[serde(rename = "vim.visual_block.foreground")]
    pub vim_visual_block_foreground: Option<String>,
    /// Foreground color for Vim Helix Normal mode indicator.
    #[serde(rename = "vim.helix_normal.foreground")]
    pub vim_helix_normal_foreground: Option<String>,
    /// Foreground color for Vim Helix Select mode indicator.
    #[serde(rename = "vim.helix_select.foreground")]
    pub vim_helix_select_foreground: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct HighlightStyleContent {
    pub color: Option<String>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "treat_error_as_none"
    )]
    pub background_color: Option<String>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "treat_error_as_none"
    )]
    pub font_style: Option<FontStyleContent>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "treat_error_as_none"
    )]
    pub font_weight: Option<FontWeightContent>,
}

fn treat_error_as_none<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    let value: Value = Deserialize::deserialize(deserializer)?;
    Ok(T::deserialize(value).ok())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct StatusColorsContent {
    /// Indicates some kind of conflict, like a file changed on disk while it was open, or
    /// merge conflicts in a Git repository.
    #[serde(rename = "conflict")]
    pub conflict: Option<String>,

    #[serde(rename = "conflict.background")]
    pub conflict_background: Option<String>,

    #[serde(rename = "conflict.border")]
    pub conflict_border: Option<String>,

    /// Indicates something new, like a new file added to a Git repository.
    #[serde(rename = "created")]
    pub created: Option<String>,

    #[serde(rename = "created.background")]
    pub created_background: Option<String>,

    #[serde(rename = "created.border")]
    pub created_border: Option<String>,

    /// Indicates that something no longer exists, like a deleted file.
    #[serde(rename = "deleted")]
    pub deleted: Option<String>,

    #[serde(rename = "deleted.background")]
    pub deleted_background: Option<String>,

    #[serde(rename = "deleted.border")]
    pub deleted_border: Option<String>,

    /// Indicates a system error, a failed operation or a diagnostic error.
    #[serde(rename = "error")]
    pub error: Option<String>,

    #[serde(rename = "error.background")]
    pub error_background: Option<String>,

    #[serde(rename = "error.border")]
    pub error_border: Option<String>,

    /// Represents a hidden status, such as a file being hidden in a file tree.
    #[serde(rename = "hidden")]
    pub hidden: Option<String>,

    #[serde(rename = "hidden.background")]
    pub hidden_background: Option<String>,

    #[serde(rename = "hidden.border")]
    pub hidden_border: Option<String>,

    /// Indicates a hint or some kind of additional information.
    #[serde(rename = "hint")]
    pub hint: Option<String>,

    #[serde(rename = "hint.background")]
    pub hint_background: Option<String>,

    #[serde(rename = "hint.border")]
    pub hint_border: Option<String>,

    /// Indicates that something is deliberately ignored, such as a file or operation ignored by Git.
    #[serde(rename = "ignored")]
    pub ignored: Option<String>,

    #[serde(rename = "ignored.background")]
    pub ignored_background: Option<String>,

    #[serde(rename = "ignored.border")]
    pub ignored_border: Option<String>,

    /// Represents informational status updates or messages.
    #[serde(rename = "info")]
    pub info: Option<String>,

    #[serde(rename = "info.background")]
    pub info_background: Option<String>,

    #[serde(rename = "info.border")]
    pub info_border: Option<String>,

    /// Indicates a changed or altered status, like a file that has been edited.
    #[serde(rename = "modified")]
    pub modified: Option<String>,

    #[serde(rename = "modified.background")]
    pub modified_background: Option<String>,

    #[serde(rename = "modified.border")]
    pub modified_border: Option<String>,

    /// Indicates something that is predicted, like automatic code completion, or generated code.
    #[serde(rename = "predictive")]
    pub predictive: Option<String>,

    #[serde(rename = "predictive.background")]
    pub predictive_background: Option<String>,

    #[serde(rename = "predictive.border")]
    pub predictive_border: Option<String>,

    /// Represents a renamed status, such as a file that has been renamed.
    #[serde(rename = "renamed")]
    pub renamed: Option<String>,

    #[serde(rename = "renamed.background")]
    pub renamed_background: Option<String>,

    #[serde(rename = "renamed.border")]
    pub renamed_border: Option<String>,

    /// Indicates a successful operation or task completion.
    #[serde(rename = "success")]
    pub success: Option<String>,

    #[serde(rename = "success.background")]
    pub success_background: Option<String>,

    #[serde(rename = "success.border")]
    pub success_border: Option<String>,

    /// Indicates some kind of unreachable status, like a block of code that can never be reached.
    #[serde(rename = "unreachable")]
    pub unreachable: Option<String>,

    #[serde(rename = "unreachable.background")]
    pub unreachable_background: Option<String>,

    #[serde(rename = "unreachable.border")]
    pub unreachable_border: Option<String>,

    /// Represents a warning status, like an operation that is about to fail.
    #[serde(rename = "warning")]
    pub warning: Option<String>,

    #[serde(rename = "warning.background")]
    pub warning_background: Option<String>,

    #[serde(rename = "warning.border")]
    pub warning_border: Option<String>,
}

/// The background appearance of the window.
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowBackgroundContent {
    Opaque,
    Transparent,
    Blurred,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FontStyleContent {
    Normal,
    Italic,
    Oblique,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FontWeightContent(pub f32);

impl Display for FontWeightContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<f32> for FontWeightContent {
    fn from(weight: f32) -> Self {
        FontWeightContent(weight)
    }
}

impl Default for FontWeightContent {
    fn default() -> Self {
        Self::NORMAL
    }
}

impl FontWeightContent {
    pub const NORMAL: FontWeightContent = FontWeightContent(400.0);
}

pub fn status_colors_refinement(colors: &StatusColorsContent) -> StatusColorsRefinement {
    StatusColorsRefinement {
        conflict: colors
            .conflict
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        conflict_background: colors
            .conflict_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        conflict_border: colors
            .conflict_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        created: colors
            .created
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        created_background: colors
            .created_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        created_border: colors
            .created_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        deleted: colors
            .deleted
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        deleted_background: colors
            .deleted_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        deleted_border: colors
            .deleted_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        error: colors
            .error
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        error_background: colors
            .error_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        error_border: colors
            .error_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        hidden: colors
            .hidden
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        hidden_background: colors
            .hidden_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        hidden_border: colors
            .hidden_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        hint: colors
            .hint
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        hint_background: colors
            .hint_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        hint_border: colors
            .hint_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        ignored: colors
            .ignored
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        ignored_background: colors
            .ignored_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        ignored_border: colors
            .ignored_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        info: colors
            .info
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        info_background: colors
            .info_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        info_border: colors
            .info_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        modified: colors
            .modified
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        modified_background: colors
            .modified_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        modified_border: colors
            .modified_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        predictive: colors
            .predictive
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        predictive_background: colors
            .predictive_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        predictive_border: colors
            .predictive_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        renamed: colors
            .renamed
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        renamed_background: colors
            .renamed_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        renamed_border: colors
            .renamed_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        success: colors
            .success
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        success_background: colors
            .success_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        success_border: colors
            .success_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        unreachable: colors
            .unreachable
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        unreachable_background: colors
            .unreachable_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        unreachable_border: colors
            .unreachable_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        warning: colors
            .warning
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        warning_background: colors
            .warning_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        warning_border: colors
            .warning_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
    }
}

pub fn theme_colors_refinement(
    this: &ThemeColorsContent,
    status_colors: &StatusColorsRefinement,
    is_light: bool,
) -> ThemeColorsRefinement {
    let border = this
        .border
        .as_ref()
        .and_then(|color| try_parse_color(color).ok());
    let editor_document_highlight_read_background = this
        .editor_document_highlight_read_background
        .as_ref()
        .and_then(|color| try_parse_color(color).ok());
    let scrollbar_thumb_background = this
        .scrollbar_thumb_background
        .as_ref()
        .and_then(|color| try_parse_color(color).ok())
        .or_else(|| {
            this.deprecated_scrollbar_thumb_background
                .as_ref()
                .and_then(|color| try_parse_color(color).ok())
        });
    let scrollbar_thumb_hover_background = this
        .scrollbar_thumb_hover_background
        .as_ref()
        .and_then(|color| try_parse_color(color).ok());
    let scrollbar_thumb_active_background = this
        .scrollbar_thumb_active_background
        .as_ref()
        .and_then(|color| try_parse_color(color).ok())
        .or(scrollbar_thumb_background);
    let scrollbar_thumb_border = this
        .scrollbar_thumb_border
        .as_ref()
        .and_then(|color| try_parse_color(color).ok());
    let element_hover = this
        .element_hover
        .as_ref()
        .and_then(|color| try_parse_color(color).ok());
    let panel_background = this
        .panel_background
        .as_ref()
        .and_then(|color| try_parse_color(color).ok());
    let search_match_background = this
        .search_match_background
        .as_ref()
        .and_then(|color| try_parse_color(color).ok());
    let search_active_match_background = this
        .search_active_match_background
        .as_ref()
        .and_then(|color| try_parse_color(color).ok())
        .or(search_match_background);
    let version_control_added = this
        .version_control_added
        .as_ref()
        .and_then(|color| try_parse_color(color).ok())
        .or(status_colors.created);
    let version_control_deleted = this
        .version_control_deleted
        .as_ref()
        .and_then(|color| try_parse_color(color).ok())
        .or(status_colors.deleted);
    let (hunk_fill, hunk_hollow_bg, hunk_hollow_border) = if is_light {
        (
            LIGHT_DIFF_HUNK_FILLED_OPACITY,
            LIGHT_DIFF_HUNK_HOLLOW_BACKGROUND_OPACITY,
            LIGHT_DIFF_HUNK_HOLLOW_BORDER_OPACITY,
        )
    } else {
        (
            DARK_DIFF_HUNK_FILLED_OPACITY,
            DARK_DIFF_HUNK_HOLLOW_BACKGROUND_OPACITY,
            DARK_DIFF_HUNK_HOLLOW_BORDER_OPACITY,
        )
    };
    ThemeColorsRefinement {
        border,
        border_variant: this
            .border_variant
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        border_focused: this
            .border_focused
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        border_selected: this
            .border_selected
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        border_transparent: this
            .border_transparent
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        border_disabled: this
            .border_disabled
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        elevated_surface_background: this
            .elevated_surface_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        surface_background: this
            .surface_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        background: this
            .background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        element_background: this
            .element_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        element_hover,
        element_active: this
            .element_active
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        element_selected: this
            .element_selected
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        element_disabled: this
            .element_disabled
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        element_selection_background: this
            .element_selection_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        drop_target_background: this
            .drop_target_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        drop_target_border: this
            .drop_target_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        ghost_element_background: this
            .ghost_element_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        ghost_element_hover: this
            .ghost_element_hover
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        ghost_element_active: this
            .ghost_element_active
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        ghost_element_selected: this
            .ghost_element_selected
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        ghost_element_disabled: this
            .ghost_element_disabled
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        text: this
            .text
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        text_muted: this
            .text_muted
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        text_placeholder: this
            .text_placeholder
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        text_disabled: this
            .text_disabled
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        text_accent: this
            .text_accent
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        icon: this
            .icon
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        icon_muted: this
            .icon_muted
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        icon_disabled: this
            .icon_disabled
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        icon_placeholder: this
            .icon_placeholder
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        icon_accent: this
            .icon_accent
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        debugger_accent: this
            .debugger_accent
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        status_bar_background: this
            .status_bar_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        title_bar_background: this
            .title_bar_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        title_bar_inactive_background: this
            .title_bar_inactive_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        toolbar_background: this
            .toolbar_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        tab_bar_background: this
            .tab_bar_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        tab_inactive_background: this
            .tab_inactive_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        tab_active_background: this
            .tab_active_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        search_match_background,
        search_active_match_background,
        panel_background,
        panel_focused_border: this
            .panel_focused_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        panel_indent_guide: this
            .panel_indent_guide
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        panel_indent_guide_hover: this
            .panel_indent_guide_hover
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        panel_indent_guide_active: this
            .panel_indent_guide_active
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        panel_overlay_background: this
            .panel_overlay_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or(panel_background.map(ensure_opaque)),
        panel_overlay_hover: this
            .panel_overlay_hover
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or(panel_background
                .zip(element_hover)
                .map(|(panel_bg, hover_bg)| panel_bg.blend(hover_bg))
                .map(ensure_opaque)),
        pane_focused_border: this
            .pane_focused_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        pane_group_border: this
            .pane_group_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or(border),
        scrollbar_thumb_background,
        scrollbar_thumb_hover_background,
        scrollbar_thumb_active_background,
        scrollbar_thumb_border,
        scrollbar_track_background: this
            .scrollbar_track_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        scrollbar_track_border: this
            .scrollbar_track_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        minimap_thumb_background: this
            .minimap_thumb_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or(scrollbar_thumb_background.map(ensure_non_opaque)),
        minimap_thumb_hover_background: this
            .minimap_thumb_hover_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or(scrollbar_thumb_hover_background.map(ensure_non_opaque)),
        minimap_thumb_active_background: this
            .minimap_thumb_active_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or(scrollbar_thumb_active_background.map(ensure_non_opaque)),
        minimap_thumb_border: this
            .minimap_thumb_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or(scrollbar_thumb_border),
        editor_foreground: this
            .editor_foreground
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_background: this
            .editor_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_gutter_background: this
            .editor_gutter_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_subheader_background: this
            .editor_subheader_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_active_line_background: this
            .editor_active_line_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_highlighted_line_background: this
            .editor_highlighted_line_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_debugger_active_line_background: this
            .editor_debugger_active_line_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_line_number: this
            .editor_line_number
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_hover_line_number: this
            .editor_hover_line_number
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_active_line_number: this
            .editor_active_line_number
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_invisible: this
            .editor_invisible
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_wrap_guide: this
            .editor_wrap_guide
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_active_wrap_guide: this
            .editor_active_wrap_guide
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_indent_guide: this
            .editor_indent_guide
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_indent_guide_active: this
            .editor_indent_guide_active
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_document_highlight_read_background,
        editor_document_highlight_write_background: this
            .editor_document_highlight_write_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        editor_document_highlight_bracket_background: this
            .editor_document_highlight_bracket_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or(editor_document_highlight_read_background),
        editor_diff_hunk_added_background: this
            .editor_diff_hunk_added_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or_else(|| version_control_added.map(|c| c.opacity(hunk_fill))),
        editor_diff_hunk_added_hollow_background: this
            .editor_diff_hunk_added_hollow_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or_else(|| version_control_added.map(|c| c.opacity(hunk_hollow_bg))),
        editor_diff_hunk_added_hollow_border: this
            .editor_diff_hunk_added_hollow_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or_else(|| version_control_added.map(|c| c.opacity(hunk_hollow_border))),
        editor_diff_hunk_deleted_background: this
            .editor_diff_hunk_deleted_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or_else(|| version_control_deleted.map(|c| c.opacity(hunk_fill))),
        editor_diff_hunk_deleted_hollow_background: this
            .editor_diff_hunk_deleted_hollow_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or_else(|| version_control_deleted.map(|c| c.opacity(hunk_hollow_bg))),
        editor_diff_hunk_deleted_hollow_border: this
            .editor_diff_hunk_deleted_hollow_border
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or_else(|| version_control_deleted.map(|c| c.opacity(hunk_hollow_border))),
        terminal_background: this
            .terminal_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_background: this
            .terminal_ansi_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_foreground: this
            .terminal_foreground
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_bright_foreground: this
            .terminal_bright_foreground
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_dim_foreground: this
            .terminal_dim_foreground
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_black: this
            .terminal_ansi_black
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_bright_black: this
            .terminal_ansi_bright_black
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_dim_black: this
            .terminal_ansi_dim_black
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_red: this
            .terminal_ansi_red
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_bright_red: this
            .terminal_ansi_bright_red
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_dim_red: this
            .terminal_ansi_dim_red
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_green: this
            .terminal_ansi_green
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_bright_green: this
            .terminal_ansi_bright_green
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_dim_green: this
            .terminal_ansi_dim_green
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_yellow: this
            .terminal_ansi_yellow
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_bright_yellow: this
            .terminal_ansi_bright_yellow
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_dim_yellow: this
            .terminal_ansi_dim_yellow
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_blue: this
            .terminal_ansi_blue
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_bright_blue: this
            .terminal_ansi_bright_blue
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_dim_blue: this
            .terminal_ansi_dim_blue
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_magenta: this
            .terminal_ansi_magenta
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_bright_magenta: this
            .terminal_ansi_bright_magenta
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_dim_magenta: this
            .terminal_ansi_dim_magenta
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_cyan: this
            .terminal_ansi_cyan
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_bright_cyan: this
            .terminal_ansi_bright_cyan
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_dim_cyan: this
            .terminal_ansi_dim_cyan
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_white: this
            .terminal_ansi_white
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_bright_white: this
            .terminal_ansi_bright_white
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        terminal_ansi_dim_white: this
            .terminal_ansi_dim_white
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        link_text_hover: this
            .link_text_hover
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        version_control_added,
        version_control_deleted,
        version_control_modified: this
            .version_control_modified
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or(status_colors.modified),
        version_control_renamed: this
            .version_control_renamed
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or(status_colors.modified),
        version_control_conflict: this
            .version_control_conflict
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or(status_colors.ignored),
        version_control_ignored: this
            .version_control_ignored
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or(status_colors.ignored),
        version_control_word_added: this
            .version_control_word_added
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        version_control_word_deleted: this
            .version_control_word_deleted
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        #[allow(deprecated)]
        version_control_conflict_marker_ours: this
            .version_control_conflict_marker_ours
            .as_ref()
            .or(this.version_control_conflict_ours_background.as_ref())
            .and_then(|color| try_parse_color(color).ok()),
        #[allow(deprecated)]
        version_control_conflict_marker_theirs: this
            .version_control_conflict_marker_theirs
            .as_ref()
            .or(this.version_control_conflict_theirs_background.as_ref())
            .and_then(|color| try_parse_color(color).ok()),
        vim_normal_background: this
            .vim_normal_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        vim_insert_background: this
            .vim_insert_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        vim_replace_background: this
            .vim_replace_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        vim_visual_background: this
            .vim_visual_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        vim_visual_line_background: this
            .vim_visual_line_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        vim_visual_block_background: this
            .vim_visual_block_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        vim_yank_background: this
            .vim_yank_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or(editor_document_highlight_read_background),
        vim_helix_jump_label_foreground: this
            .vim_helix_jump_label_foreground
            .as_ref()
            .and_then(|color| try_parse_color(color).ok())
            .or(status_colors.error),
        vim_helix_normal_background: this
            .vim_helix_normal_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        vim_helix_select_background: this
            .vim_helix_select_background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        vim_normal_foreground: this
            .vim_normal_foreground
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        vim_insert_foreground: this
            .vim_insert_foreground
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        vim_replace_foreground: this
            .vim_replace_foreground
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        vim_visual_foreground: this
            .vim_visual_foreground
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        vim_visual_line_foreground: this
            .vim_visual_line_foreground
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        vim_visual_block_foreground: this
            .vim_visual_block_foreground
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        vim_helix_normal_foreground: this
            .vim_helix_normal_foreground
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
        vim_helix_select_foreground: this
            .vim_helix_select_foreground
            .as_ref()
            .and_then(|color| try_parse_color(color).ok()),
    }
}

fn ensure_non_opaque(color: Hsla) -> Hsla {
    const MAXIMUM_OPACITY: f32 = 0.7;
    if color.a <= MAXIMUM_OPACITY {
        color
    } else {
        Hsla {
            a: MAXIMUM_OPACITY,
            ..color
        }
    }
}

fn ensure_opaque(color: Hsla) -> Hsla {
    Hsla { a: 1.0, ..color }
}

fn try_parse_color(color: &str) -> anyhow::Result<Hsla> {
    let rgba = gpui::Rgba::try_from(color)?;
    let rgba = palette::rgb::Srgba::from_components((rgba.r, rgba.g, rgba.b, rgba.a));
    let hsla = palette::Hsla::from_color(rgba);

    let hsla = gpui::hsla(
        hsla.hue.into_positive_degrees() / 360.,
        hsla.saturation,
        hsla.lightness,
        hsla.alpha,
    );

    Ok(hsla)
}

trait IntoGpui {
    type Output;
    fn into_gpui(self) -> Self::Output;
}

impl IntoGpui for FontStyleContent {
    type Output = gpui::FontStyle;

    fn into_gpui(self) -> Self::Output {
        match self {
            Self::Normal => gpui::FontStyle::Normal,
            Self::Italic => gpui::FontStyle::Italic,
            Self::Oblique => gpui::FontStyle::Oblique,
        }
    }
}

impl IntoGpui for FontWeightContent {
    type Output = gpui::FontWeight;

    fn into_gpui(self) -> Self::Output {
        gpui::FontWeight(self.0.clamp(100., 950.))
    }
}

impl IntoGpui for WindowBackgroundContent {
    type Output = gpui::WindowBackgroundAppearance;

    fn into_gpui(self) -> Self::Output {
        match self {
            Self::Opaque => gpui::WindowBackgroundAppearance::Opaque,
            Self::Transparent => gpui::WindowBackgroundAppearance::Transparent,
            Self::Blurred => gpui::WindowBackgroundAppearance::Blurred,
        }
    }
}

/// Decodes a bundled Zed theme family independently of editor settings.
pub fn decode_bundled_theme(bytes: &[u8]) -> anyhow::Result<ThemeFamily> {
    let content: ThemeFamilyContent = serde_json::from_slice(bytes)?;
    Ok(refine_theme_family(content))
}

fn refine_theme_family(theme_family_content: ThemeFamilyContent) -> ThemeFamily {
    let id = uuid::Uuid::new_v4().to_string();
    let name = theme_family_content.name.clone();
    let author = theme_family_content.author.clone();

    let themes: Vec<Theme> = theme_family_content
        .themes
        .iter()
        .map(refine_theme)
        .collect();

    ThemeFamily {
        id,
        name: name.into(),
        author: author.into(),
        themes,
        scales: default_color_scales(),
    }
}

/// Refines a [`ThemeContent`] into a [`Theme`].
fn refine_theme(theme: &ThemeContent) -> Theme {
    let appearance = match theme.appearance {
        AppearanceContent::Light => Appearance::Light,
        AppearanceContent::Dark => Appearance::Dark,
    };

    let mut refined_status_colors = match theme.appearance {
        AppearanceContent::Light => StatusColors::light(),
        AppearanceContent::Dark => StatusColors::dark(),
    };
    let mut status_colors_refinement = status_colors_refinement(&theme.style.status);
    crate::apply_status_color_defaults(&mut status_colors_refinement);
    refined_status_colors.refine(&status_colors_refinement);

    let mut refined_player_colors = match theme.appearance {
        AppearanceContent::Light => PlayerColors::light(),
        AppearanceContent::Dark => PlayerColors::dark(),
    };
    merge_player_colors(&mut refined_player_colors, &theme.style.players);

    let mut refined_theme_colors = match theme.appearance {
        AppearanceContent::Light => ThemeColors::light(),
        AppearanceContent::Dark => ThemeColors::dark(),
    };
    let mut theme_colors_refinement = theme_colors_refinement(
        &theme.style.colors,
        &status_colors_refinement,
        theme.appearance == AppearanceContent::Light,
    );
    crate::apply_theme_color_defaults(&mut theme_colors_refinement, &refined_player_colors);
    refined_theme_colors.refine(&theme_colors_refinement);

    let mut refined_accent_colors = match theme.appearance {
        AppearanceContent::Light => AccentColors::light(),
        AppearanceContent::Dark => AccentColors::dark(),
    };
    merge_accent_colors(&mut refined_accent_colors, &theme.style.accents);

    let syntax_highlights = theme.style.syntax.iter().map(|(syntax_token, highlight)| {
        (
            syntax_token.clone(),
            HighlightStyle {
                color: highlight
                    .color
                    .as_ref()
                    .and_then(|color| try_parse_color(color).ok()),
                background_color: highlight
                    .background_color
                    .as_ref()
                    .and_then(|color| try_parse_color(color).ok()),
                font_style: highlight.font_style.map(|s| s.into_gpui()),
                font_weight: highlight.font_weight.map(|w| w.into_gpui()),
                ..Default::default()
            },
        )
    });
    let syntax_theme = Arc::new(SyntaxTheme::new(syntax_highlights));

    let window_background_appearance = theme
        .style
        .window_background_appearance
        .map(|w| w.into_gpui())
        .unwrap_or_default();

    Theme {
        id: uuid::Uuid::new_v4().to_string(),
        name: theme.name.clone().into(),
        appearance,
        styles: ThemeStyles {
            system: SystemColors::default(),
            window_background_appearance,
            accents: refined_accent_colors,
            colors: refined_theme_colors,
            status: refined_status_colors,
            player: refined_player_colors,
            syntax: syntax_theme,
        },
    }
}

/// Merges player color overrides into the given [`PlayerColors`].
fn merge_player_colors(
    player_colors: &mut PlayerColors,
    user_player_colors: &[PlayerColorContent],
) {
    if user_player_colors.is_empty() {
        return;
    }

    for (idx, player) in user_player_colors.iter().enumerate() {
        let cursor = player
            .cursor
            .as_ref()
            .and_then(|color| try_parse_color(color).ok());
        let background = player
            .background
            .as_ref()
            .and_then(|color| try_parse_color(color).ok());
        let selection = player
            .selection
            .as_ref()
            .and_then(|color| try_parse_color(color).ok());

        if let Some(player_color) = player_colors.0.get_mut(idx) {
            *player_color = PlayerColor {
                cursor: cursor.unwrap_or(player_color.cursor),
                background: background.unwrap_or(player_color.background),
                selection: selection.unwrap_or(player_color.selection),
            };
        } else {
            player_colors.0.push(PlayerColor {
                cursor: cursor.unwrap_or_default(),
                background: background.unwrap_or_default(),
                selection: selection.unwrap_or_default(),
            });
        }
    }
}

/// Merges accent color overrides into the given [`AccentColors`].
fn merge_accent_colors(accent_colors: &mut AccentColors, user_accent_colors: &[AccentContent]) {
    if user_accent_colors.is_empty() {
        return;
    }

    let colors = user_accent_colors
        .iter()
        .filter_map(|accent_color| {
            accent_color
                .0
                .as_ref()
                .and_then(|color| try_parse_color(color).ok())
        })
        .collect::<Vec<_>>();

    if !colors.is_empty() {
        accent_colors.0 = Arc::from(colors);
    }
}

#[cfg(test)]
mod tests {
    use gpui::AssetSource as _;

    use super::*;

    #[test]
    fn every_bundled_theme_decodes_without_editor_settings() {
        let paths = zed_ui_assets::Assets
            .list("themes/")
            .expect("bundled theme list");
        let mut theme_names = Vec::new();
        for path in paths.into_iter().filter(|path| path.ends_with(".json")) {
            let bytes = zed_ui_assets::Assets
                .load(&path)
                .expect("bundled theme load")
                .expect("bundled theme exists");
            let family = decode_bundled_theme(&bytes)
                .unwrap_or_else(|error| panic!("could not decode {path}: {error}"));
            theme_names.extend(
                family
                    .themes
                    .into_iter()
                    .map(|theme| theme.name.to_string()),
            );
        }
        assert!(theme_names.iter().any(|name| name == "One Light"));
        assert!(theme_names.iter().any(|name| name == "One Dark"));
    }
}
