//! Shared geometry scale for Fanta chrome.
//!
//! This module is GEOMETRY ONLY. Color, the body font, and surface radius come
//! from `gpui_component::Theme` (`radius`, `radius_lg`, `font_size`) — not from
//! here. A token that would restate a theme value does not belong in this file.
//!
//! Every constant is a bare `f32`, matching the existing
//! [`InspectorMetrics`](crate::molecules::InspectorMetrics) convention, so call
//! sites stay `px(RowHeight::FIELD)` and the text-scanning architecture tests
//! keep matching them. Hosts reach the scale at
//! `fanta_gpui::atoms::tokens::RowHeight::FIELD`.

/// Padding and gap steps shared by rows, sections, and panel chrome.
pub struct Space;

impl Space {
    pub const NONE: f32 = 0.;
    pub const XS: f32 = 4.;
    pub const SM: f32 = 8.;
    pub const MD: f32 = 12.;
    pub const LG: f32 = 16.;
}

/// Fixed heights for the crate's horizontal strips: inspector rows, list rows,
/// menu items, page entries, flyouts, and section headers.
pub struct RowHeight;

impl RowHeight {
    pub const FIELD: f32 = 24.;
    pub const LIST: f32 = 28.;
    /// Off-step by one: deliberately preserved shipping geometry, pending a
    /// post-ship polish pass.
    pub const MENU: f32 = 30.;
    pub const PAGE: f32 = 32.;
    /// Off-step by two: deliberately preserved shipping geometry, pending a
    /// post-ship polish pass.
    pub const FLYOUT: f32 = 38.;
    pub const SECTION_HEADER: f32 = 40.;
}

/// Square hit targets for controls: inline affordances inside a field, panel
/// chrome buttons, and toolbar tools.
pub struct ControlSize;

impl ControlSize {
    pub const INLINE: f32 = 16.;
    pub const CHROME: f32 = 28.;
    pub const TOOL: f32 = 32.;
}

/// Geometry specific to the UI3 button component families.
pub struct ButtonGeometry;

impl ButtonGeometry {
    pub const RADIUS: f32 = 5.;
    pub const WIDE_WIDTH: f32 = 256.;
    pub const SPLIT_SMALL_WIDTH: f32 = 41.;
    pub const SPLIT_LARGE_WIDTH: f32 = 49.;
    pub const SPLIT_GAP: f32 = 1.;
}

/// Geometry specific to the UI3 dropdown trigger.
pub struct DropdownGeometry;

impl DropdownGeometry {
    pub const WIDTH: f32 = 117.;
}

/// Geometry shared by the UI3 segmented-control family.
pub struct SegmentedControlGeometry;

impl SegmentedControlGeometry {
    pub const WIDTH: f32 = 200.;
    pub const RADIUS: f32 = ButtonGeometry::RADIUS;
}

/// Geometry shared by the UI3 radio-button family.
pub struct RadioButtonGeometry;

impl RadioButtonGeometry {
    pub const INDICATOR_SIZE: f32 = ControlSize::INLINE;
    pub const DOT_SIZE: f32 = Space::SM;
    pub const BUTTON_MIN_WIDTH: f32 = 88.;
}

/// Geometry shared by the UI3 tab and tabs component sets.
pub struct TabsGeometry;

impl TabsGeometry {
    pub const TAB_MIN_WIDTH: f32 = 72.;
    pub const TABS_WIDTH: f32 = 288.;
    pub const SINGLE_TAB_WIDTH: f32 = 104.;
    pub const BADGE_HEIGHT: f32 = ControlSize::INLINE;
    pub const BADGE_MIN_WIDTH: f32 = ControlSize::INLINE;
}

/// Geometry shared by UI3 tooltip surfaces and tooltip-link rows.
pub struct TooltipGeometry;

impl TooltipGeometry {
    pub const RADIUS: f32 = ButtonGeometry::RADIUS;
    pub const CARET_WIDTH: f32 = Space::SM;
    pub const CARET_HEIGHT: f32 = Space::XS;
    pub const LINK_WIDTH: f32 = MenuWidth::PICKER;
}

/// Geometry shared by the compact UI3 input families.
pub struct InputGeometry;

impl InputGeometry {
    pub const TEXT_WIDTH: f32 = 192.;
    pub const NUMERIC_WIDTH: f32 = 72.;
    pub const COLOR_WIDTH: f32 = 168.;
    pub const COMBO_WIDTH: f32 = 128.;
    pub const VARIABLE_CELL_WIDTH: f32 = 88.;
    pub const MULTI_CELL_WIDTH: f32 = 40.;
    pub const MULTILINE_HEIGHT: f32 = 64.;
    pub const CHIT_SMALL: f32 = RowHeight::FIELD;
    pub const CHIT_LARGE: f32 = 48.;
}

/// Corner radii owned by Fanta chrome. Surface radius that a host theme should
/// drive comes from `Theme::radius`/`radius_lg` instead.
pub struct Radius;

impl Radius {
    pub const CONTROL: f32 = 4.;
    pub const MENU: f32 = 12.;
}

/// Rendered glyph sizes for [`LucideIcon`](crate::atoms::LucideIcon) sites.
pub struct IconSize;

impl IconSize {
    pub const XS: f32 = 12.;
    /// Off-step by two: deliberately preserved shipping geometry, pending a
    /// post-ship polish pass.
    pub const SM: f32 = 14.;
    pub const MD: f32 = 16.;
}

/// Text sizes passed to `text_size(px(..))`. Body copy that should follow the
/// host theme uses `Theme::font_size` instead.
pub struct TypeScale;

impl TypeScale {
    pub const MICRO: f32 = 10.;
    pub const CAPTION: f32 = 11.;
    pub const BODY: f32 = 12.;
    pub const LABEL: f32 = 13.;
    /// Off-step by one: deliberately preserved shipping geometry, pending a
    /// post-ship polish pass.
    pub const TITLE: f32 = 14.;
    pub const DISPLAY: f32 = 20.;
}

/// Widths of transient surfaces: dropdown menus, popovers, and pickers.
pub struct MenuWidth;

impl MenuWidth {
    pub const NARROW: f32 = 168.;
    pub const STANDARD: f32 = 224.;
    pub const POPOVER: f32 = 248.;
    pub const PICKER: f32 = 280.;
    pub const PICKER_WIDE: f32 = 304.;
}

/// Geometry shared by the UI3 menu row families.
pub struct MenuGeometry;

impl MenuGeometry {
    pub const ROW_RADIUS: f32 = ButtonGeometry::RADIUS;
    pub const INDICATOR_SIZE: f32 = IconSize::SM;
    pub const AVATAR_SIZE: f32 = ControlSize::INLINE;
    pub const TOGGLE_WIDTH: f32 = RowHeight::FIELD;
    pub const TOGGLE_HEIGHT: f32 = IconSize::SM;
    pub const TOGGLE_KNOB: f32 = Space::SM;
    pub const BADGE_HEIGHT: f32 = ControlSize::INLINE;
    pub const DIVIDER_HEIGHT: f32 = 1.;
}

/// Container widths at which a panel switches layout density.
pub struct Breakpoint;

impl Breakpoint {
    pub const COMPACT: f32 = 360.;
    pub const WIDE: f32 = 440.;
}

/// Shared slider rail, thumb travel, and gradient-stop geometry.
pub struct SliderGeometry;
impl SliderGeometry {
    pub const TRACK_HEIGHT: f32 = ControlSize::INLINE;
    pub const HANDLE_SIZE: f32 = 12.;
    pub const INSET: f32 = Space::SM;
    pub const MIN_WIDTH: f32 = 100.;
    pub const STOP_SIZE: f32 = RowHeight::FIELD;
}
