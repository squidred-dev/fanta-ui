#![allow(deprecated)]

use std::collections::HashSet;

use gpui::SharedString;

mod export;
mod media;

pub use export::*;
pub use media::*;

/// A Figma Design-panel selection kind.
///
/// These variants model inspector capabilities, not a Fanta document schema.
/// Hosts can map their own node kinds onto the closest visual contract.
///
/// `Image`, `Video`, `Arrow`, `Mask`, `Table`, `Pen`, `Pencil`, and
/// `MultipleSelection` are retained as source-compatible legacy adapters. They
/// are intentionally absent from [`Self::ALL`]: in Figma Design, image/video
/// are paint payloads, arrows are lines with an arrow cap, masks are an
/// orthogonal scene-node flag, Pen/Pencil are creation tools that yield vector
/// nodes, and multiple selection is inspection context rather than a node.
/// Tables belong to FigJam.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DesignPanelNodeKind {
    Frame,
    Group,
    TransformGroup,
    Section,
    Component,
    ComponentSet,
    Instance,
    Slot,
    Widget,
    Text,
    TextPath,
    Image,
    Video,
    Rectangle,
    Ellipse,
    Polygon,
    Star,
    Line,
    Arrow,
    Vector,
    BooleanOperation,
    Slice,
    Mask,
    Table,
    Pen,
    Pencil,
    MultipleSelection,
    Other,
}

impl DesignPanelNodeKind {
    /// Canonical Figma Design node kinds exposed by node-preset surfaces.
    pub const ALL: [Self; 20] = [
        Self::Frame,
        Self::Group,
        Self::TransformGroup,
        Self::Section,
        Self::Component,
        Self::ComponentSet,
        Self::Instance,
        Self::Slot,
        Self::Text,
        Self::TextPath,
        Self::Rectangle,
        Self::Ellipse,
        Self::Polygon,
        Self::Star,
        Self::Line,
        Self::Vector,
        Self::BooleanOperation,
        Self::Slice,
        Self::Widget,
        Self::Other,
    ];

    /// Complete compatibility set for hosts migrating from the original
    /// inspector taxonomy. New preset UIs should iterate [`Self::ALL`].
    pub const COMPATIBILITY_ALL: [Self; 28] = [
        Self::Frame,
        Self::Group,
        Self::TransformGroup,
        Self::Section,
        Self::Component,
        Self::ComponentSet,
        Self::Instance,
        Self::Slot,
        Self::Text,
        Self::TextPath,
        Self::Image,
        Self::Video,
        Self::Rectangle,
        Self::Ellipse,
        Self::Polygon,
        Self::Star,
        Self::Line,
        Self::Arrow,
        Self::Vector,
        Self::BooleanOperation,
        Self::Slice,
        Self::Mask,
        Self::Table,
        Self::Pen,
        Self::Pencil,
        Self::MultipleSelection,
        Self::Widget,
        Self::Other,
    ];

    /// Whether this is a compatibility-only taxonomy value.
    pub const fn is_compatibility_alias(self) -> bool {
        matches!(
            self,
            Self::Image
                | Self::Video
                | Self::Arrow
                | Self::Mask
                | Self::Table
                | Self::Pen
                | Self::Pencil
                | Self::MultipleSelection
        )
    }

    /// Closest canonical Design-node kind for a legacy adapter value.
    ///
    /// Hosts should retain their real underlying node kind whenever it is
    /// available; this mapping exists for incremental migration only.
    pub const fn canonical_kind(self) -> Self {
        match self {
            Self::Image | Self::Video => Self::Rectangle,
            Self::Arrow => Self::Line,
            Self::Mask | Self::Pen | Self::Pencil => Self::Vector,
            Self::Table => Self::Frame,
            Self::MultipleSelection => Self::Other,
            kind => kind,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Frame => "Frame",
            Self::Group => "Group",
            Self::TransformGroup => "Transform group",
            Self::Section => "Section",
            Self::Component => "Component",
            Self::ComponentSet => "Component set",
            Self::Instance => "Instance",
            Self::Slot => "Slot",
            Self::Widget => "Widget",
            Self::Text => "Text",
            Self::TextPath => "Text path",
            Self::Image => "Image",
            Self::Video => "Video",
            Self::Rectangle => "Rectangle",
            Self::Ellipse => "Ellipse",
            Self::Polygon => "Polygon",
            Self::Star => "Star",
            Self::Line => "Line",
            Self::Arrow => "Arrow",
            Self::Vector => "Vector",
            Self::BooleanOperation => "Boolean operation",
            Self::Slice => "Slice",
            Self::Mask => "Mask",
            Self::Table => "Table",
            Self::Pen => "Pen path",
            Self::Pencil => "Pencil path",
            Self::MultipleSelection => "Multiple selection",
            Self::Other => "Other",
        }
    }

    pub const fn glyph(self) -> &'static str {
        match self {
            Self::Frame => "#",
            Self::Group => "⌗",
            Self::TransformGroup => "⌖",
            Self::Section => "▣",
            Self::Component => "◆",
            Self::ComponentSet => "⁙",
            Self::Instance => "◇",
            Self::Slot => "◈",
            Self::Widget => "W",
            Self::Text => "T",
            Self::TextPath => "T⌁",
            Self::Image => "▧",
            Self::Video => "▶",
            Self::Rectangle => "□",
            Self::Ellipse => "○",
            Self::Polygon => "△",
            Self::Star => "☆",
            Self::Line => "—",
            Self::Arrow => "↗",
            Self::Vector => "⌁",
            Self::BooleanOperation => "∩",
            Self::Slice => "◩",
            Self::Mask => "◐",
            Self::Table => "▦",
            Self::Pen => "⌒",
            Self::Pencil => "∿",
            Self::MultipleSelection => "⁘",
            Self::Other => "•",
        }
    }

    /// Compatibility alias for the universal dimensions surface.
    ///
    /// Use [`Self::supports_auto_layout_container`] when deciding whether to
    /// show direction, padding, gap, and container-alignment controls.
    pub const fn supports_layout(self) -> bool {
        self.supports_dimensions()
    }

    /// Whether the layer exposes width and height in Figma's Layout section.
    ///
    /// This deliberately includes lines, slices, sections, and aggregate
    /// selections. Owning an auto-layout flow is a separate capability.
    pub const fn supports_dimensions(self) -> bool {
        true
    }

    /// Whether the layer exposes Figma's scene-node visibility field.
    ///
    /// Visibility is independent from opacity/blend/effects. In particular,
    /// Section, Slice, and Widget nodes all inherit writable `visible` even
    /// though their other Appearance capabilities differ.
    pub const fn supports_visibility(self) -> bool {
        true
    }

    /// Whether the Position section exposes writable X/Y coordinates.
    pub const fn supports_position_coordinates(self) -> bool {
        true
    }

    /// Whether selection alignment, distribution, and tidy actions apply.
    pub const fn supports_arrange(self) -> bool {
        !matches!(self, Self::Widget)
    }

    /// Whether direct rotation and flip transforms apply.
    pub const fn supports_transforms(self) -> bool {
        !matches!(self, Self::Section | Self::Widget)
    }

    /// Whether the Layout section exposes the aspect-ratio lock.
    pub const fn supports_aspect_ratio_lock(self) -> bool {
        !matches!(self, Self::Widget)
    }

    /// Whether parent auto-layout participation exposes child controls.
    pub const fn supports_auto_layout_child(self) -> bool {
        !matches!(self, Self::Section | Self::Widget)
    }

    /// Whether an eligible selection may request Figma's Add auto layout
    /// structural operation.
    pub const fn supports_add_auto_layout(self) -> bool {
        !matches!(self, Self::Section | Self::Widget)
    }

    /// Whether the layer can own and configure an auto-layout flow.
    ///
    /// Other layer kinds still participate in layout as children and expose
    /// sizing/positioning controls, but they must not inherit the container
    /// direction, padding, gap, or alignment controls.
    pub const fn supports_auto_layout_container(self) -> bool {
        matches!(
            self,
            Self::Frame | Self::Component | Self::ComponentSet | Self::Instance | Self::Slot
        )
    }

    /// Whether Figma accepts Grid as this auto-layout owner's flow.
    ///
    /// Slot nodes support horizontal and vertical auto layout, but the Plugin
    /// API rejects applying Grid to a Slot.
    pub const fn supports_grid_auto_layout(self) -> bool {
        matches!(
            self,
            Self::Frame | Self::Component | Self::ComponentSet | Self::Instance
        )
    }

    pub const fn supports_resize_to_fit(self) -> bool {
        matches!(
            self,
            Self::Frame
                | Self::Group
                | Self::TransformGroup
                | Self::Component
                | Self::ComponentSet
                | Self::Instance
                | Self::Slot
                | Self::Table
        )
    }

    pub const fn supports_clip_content(self) -> bool {
        matches!(
            self,
            Self::Frame | Self::Component | Self::ComponentSet | Self::Instance | Self::Slot
        )
    }

    pub const fn supports_fill(self) -> bool {
        !matches!(
            self,
            Self::Group
                | Self::TransformGroup
                | Self::Slice
                | Self::Line
                | Self::Arrow
                | Self::Widget
                | Self::MultipleSelection
        )
    }

    pub const fn supports_stroke(self) -> bool {
        !matches!(
            self,
            Self::Group
                | Self::TransformGroup
                | Self::Slice
                | Self::Widget
                | Self::MultipleSelection
        )
    }

    /// Whether the layer exposes Figma's opacity, blend-mode, and effects
    /// appearance contract.
    ///
    /// Sections have fills, strokes, and corners, but do not implement
    /// Figma's BlendMixin/EffectsMixin. Slice likewise has no appearance
    /// controls.
    pub const fn supports_layer_appearance(self) -> bool {
        !matches!(self, Self::Section | Self::Slice | Self::Widget)
    }

    /// Whether Figma exposes the container-only `Pass through` blend mode.
    ///
    /// Leaf shapes and text use the 18 compositing modes beginning with
    /// `Normal`; structural containers may preserve child compositing with
    /// `Pass through`.
    pub const fn supports_pass_through_blend(self) -> bool {
        matches!(
            self,
            Self::Frame
                | Self::Group
                | Self::TransformGroup
                | Self::Component
                | Self::ComponentSet
                | Self::Instance
                | Self::Slot
                | Self::Table
        )
    }

    pub const fn supports_effects(self) -> bool {
        self.supports_layer_appearance()
    }

    pub const fn supports_corner_radius(self) -> bool {
        matches!(
            self,
            Self::Frame
                | Self::Section
                | Self::Component
                | Self::ComponentSet
                | Self::Instance
                | Self::Slot
                | Self::Rectangle
                | Self::Image
                | Self::Video
                | Self::Ellipse
                | Self::Polygon
                | Self::Star
                | Self::Vector
                | Self::BooleanOperation
        )
    }

    pub const fn supports_constraints(self) -> bool {
        matches!(
            self,
            Self::Frame
                | Self::Group
                | Self::TransformGroup
                | Self::Component
                | Self::ComponentSet
                | Self::Instance
                | Self::Slot
                | Self::Text
                | Self::TextPath
                | Self::Image
                | Self::Video
                | Self::Rectangle
                | Self::Ellipse
                | Self::Polygon
                | Self::Star
                | Self::Line
                | Self::Arrow
                | Self::Vector
                | Self::Mask
                | Self::Pen
                | Self::Pencil
        )
    }
}

/// A host-provided color represented in display-space channels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesignColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl DesignColor {
    pub const BLACK: Self = Self::rgb(0x1e, 0x1e, 0x1e);
    pub const WHITE: Self = Self::rgb(0xff, 0xff, 0xff);
    pub const BLUE: Self = Self::rgb(0x4c, 0x86, 0xf7);
    pub const PURPLE: Self = Self::rgb(0x97, 0x5f, 0xf7);

    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: u8::MAX,
        }
    }

    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub fn hex(self) -> SharedString {
        if self.alpha == u8::MAX {
            format!("{:02X}{:02X}{:02X}", self.red, self.green, self.blue).into()
        } else {
            format!(
                "{:02X}{:02X}{:02X}{:02X}",
                self.red, self.green, self.blue, self.alpha
            )
            .into()
        }
    }
}

/// Figma's six top-level paint classifications.
///
/// Gradient direction remains represented by [`DesignPaintKind`], while this
/// type is suitable for the first level of a paint-type picker.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignPaintType {
    Solid,
    Gradient,
    Pattern,
    Image,
    Video,
    Shader,
    /// A host paint the built-in picker cannot edit without losing data.
    Unsupported,
}

impl DesignPaintType {
    /// Paint types directly creatable by the built-in picker.
    ///
    /// Shader creation starts by choosing a host-supplied definition; the
    /// picker never manufactures a shader id.
    pub const ALL: [Self; 6] = [
        Self::Solid,
        Self::Gradient,
        Self::Pattern,
        Self::Image,
        Self::Video,
        Self::Shader,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Solid => "Solid",
            Self::Gradient => "Gradient",
            Self::Pattern => "Pattern",
            Self::Image => "Image",
            Self::Video => "Video",
            Self::Shader => "Shader",
            Self::Unsupported => "Unsupported",
        }
    }

    /// Returns the representative concrete kind for a newly selected type.
    ///
    /// Existing gradient paints keep their current subtype; Linear is only the
    /// default when creating a gradient from the top-level classification.
    pub const fn default_kind(self) -> DesignPaintKind {
        match self {
            Self::Solid => DesignPaintKind::Solid,
            Self::Gradient => DesignPaintKind::LinearGradient,
            Self::Pattern => DesignPaintKind::Pattern,
            Self::Image => DesignPaintKind::Image,
            Self::Video => DesignPaintKind::Video,
            Self::Shader => DesignPaintKind::Shader,
            Self::Unsupported => DesignPaintKind::Unsupported,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignPaintKind {
    Solid,
    LinearGradient,
    RadialGradient,
    AngularGradient,
    DiamondGradient,
    Pattern,
    Image,
    Video,
    Shader,
    Unsupported,
}

impl DesignPaintKind {
    /// Paint kinds directly creatable by the built-in picker.
    pub const ALL: [Self; 9] = [
        Self::Solid,
        Self::LinearGradient,
        Self::RadialGradient,
        Self::AngularGradient,
        Self::DiamondGradient,
        Self::Pattern,
        Self::Image,
        Self::Video,
        Self::Shader,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Solid => "Solid",
            Self::LinearGradient => "Linear",
            Self::RadialGradient => "Radial",
            Self::AngularGradient => "Angular",
            Self::DiamondGradient => "Diamond",
            Self::Pattern => "Pattern",
            Self::Image => "Image",
            Self::Video => "Video",
            Self::Shader => "Shader",
            Self::Unsupported => "Unsupported",
        }
    }

    pub const fn is_gradient(self) -> bool {
        matches!(
            self,
            Self::LinearGradient
                | Self::RadialGradient
                | Self::AngularGradient
                | Self::DiamondGradient
        )
    }

    /// Collapses concrete gradient subtypes into Figma's top-level paint type.
    pub const fn paint_type(self) -> DesignPaintType {
        match self {
            Self::Solid => DesignPaintType::Solid,
            Self::LinearGradient
            | Self::RadialGradient
            | Self::AngularGradient
            | Self::DiamondGradient => DesignPaintType::Gradient,
            Self::Pattern => DesignPaintType::Pattern,
            Self::Image => DesignPaintType::Image,
            Self::Video => DesignPaintType::Video,
            Self::Shader => DesignPaintType::Shader,
            Self::Unsupported => DesignPaintType::Unsupported,
        }
    }
}

impl From<DesignPaintKind> for DesignPaintType {
    fn from(kind: DesignPaintKind) -> Self {
        kind.paint_type()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignGradientStop {
    /// Stable opaque host identity. Empty retains legacy index targeting.
    pub id: SharedString,
    pub position: f32,
    pub color: DesignColor,
    pub binding: Option<DesignPaintBinding>,
}

impl DesignGradientStop {
    pub fn new(position: f32, color: DesignColor) -> Self {
        Self {
            id: "".into(),
            position,
            color,
            binding: None,
        }
    }

    pub fn with_id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_binding(mut self, binding: DesignPaintBinding) -> Self {
        self.binding = Some(binding);
        self
    }
}

/// Opaque color-variable binding metadata retained at one paint color leaf.
///
/// This is deliberately not a Paint-style binding. Figma stores `fillStyleId`
/// and `strokeStyleId` on the whole ordered paint collection; those identities
/// are represented by [`DesignPaintStyleBinding`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignPaintBinding {
    pub variable_id: SharedString,
    pub variable_name: SharedString,
    pub collection_name: Option<SharedString>,
}

impl DesignPaintBinding {
    pub fn new(
        variable_id: impl Into<SharedString>,
        variable_name: impl Into<SharedString>,
    ) -> Self {
        Self {
            variable_id: variable_id.into(),
            variable_name: variable_name.into(),
            collection_name: None,
        }
    }

    pub fn with_collection(mut self, collection_name: impl Into<SharedString>) -> Self {
        self.collection_name = Some(collection_name.into());
        self
    }
}

/// Compatibility-only host-supplied leaf color preset.
///
/// New hosts should supply [`DesignPaintVariableViewData`] for color-variable
/// rows and [`DesignPaintStyleViewData`] for whole Fill/Stroke styles. This
/// older shape remains available so existing adapters can migrate without
/// conflating the two binding levels.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignColorStyle {
    pub id: SharedString,
    pub name: SharedString,
    pub color: DesignColor,
    pub binding: Option<DesignPaintBinding>,
}

impl DesignColorStyle {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        color: DesignColor,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            color,
            binding: None,
        }
    }

    pub fn with_binding(mut self, binding: DesignPaintBinding) -> Self {
        self.binding = Some(binding);
        self
    }
}

/// A named host-supplied library and its available color styles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignColorStyleLibrary {
    pub id: SharedString,
    pub name: SharedString,
    pub styles: Vec<DesignColorStyle>,
}

impl DesignColorStyleLibrary {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        styles: impl IntoIterator<Item = DesignColorStyle>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            styles: styles.into_iter().collect(),
        }
    }
}

/// Immutable color-style snapshot supplied by a Design-panel host.
///
/// Page styles populate the Custom tab's "On this page" section. Libraries
/// populate the Libraries tab in their supplied order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DesignColorStyleViewData {
    pub page_styles: Vec<DesignColorStyle>,
    pub libraries: Vec<DesignColorStyleLibrary>,
}

/// One solid-color sample exposed by a page or library Color style.
///
/// Sampling this value in the paint picker changes only the active solid
/// color or gradient-stop leaf. It does not attach the style to the complete
/// Fill/Stroke collection and it never creates a Color-variable binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignColorStyleSample {
    pub id: SharedString,
    pub name: SharedString,
    pub color: DesignColor,
    pub disabled_reason: Option<SharedString>,
}

impl DesignColorStyleSample {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        color: DesignColor,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            color,
            disabled_reason: None,
        }
    }

    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.disabled_reason = Some(reason.into());
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignColorStyleSampleLibrary {
    pub id: SharedString,
    pub name: SharedString,
    pub samples: Vec<DesignColorStyleSample>,
}

impl DesignColorStyleSampleLibrary {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        samples: impl IntoIterator<Item = DesignColorStyleSample>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            samples: samples.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DesignColorStyleSampleSource {
    Page,
    Library { library_id: SharedString },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DesignColorStyleSampleSelection {
    pub source: DesignColorStyleSampleSource,
    pub sample_id: SharedString,
}

impl DesignColorStyleSampleSelection {
    pub fn page(sample_id: impl Into<SharedString>) -> Self {
        Self {
            source: DesignColorStyleSampleSource::Page,
            sample_id: sample_id.into(),
        }
    }

    pub fn library(
        library_id: impl Into<SharedString>,
        sample_id: impl Into<SharedString>,
    ) -> Self {
        Self {
            source: DesignColorStyleSampleSource::Library {
                library_id: library_id.into(),
            },
            sample_id: sample_id.into(),
        }
    }
}

/// Host-filtered Color-style samples shown beside Color variables in the
/// retained paint picker.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DesignColorStyleSampleViewData {
    pub page_samples: Vec<DesignColorStyleSample>,
    pub libraries: Vec<DesignColorStyleSampleLibrary>,
}

impl DesignColorStyleSampleViewData {
    pub fn new(
        page_samples: impl IntoIterator<Item = DesignColorStyleSample>,
        libraries: impl IntoIterator<Item = DesignColorStyleSampleLibrary>,
    ) -> Self {
        Self {
            page_samples: page_samples.into_iter().collect(),
            libraries: libraries.into_iter().collect(),
        }
    }

    pub fn sample(
        &self,
        selection: &DesignColorStyleSampleSelection,
    ) -> Option<&DesignColorStyleSample> {
        match &selection.source {
            DesignColorStyleSampleSource::Page => self
                .page_samples
                .iter()
                .find(|sample| sample.id == selection.sample_id),
            DesignColorStyleSampleSource::Library { library_id } => self
                .libraries
                .iter()
                .find(|library| library.id == *library_id)?
                .samples
                .iter()
                .find(|sample| sample.id == selection.sample_id),
        }
    }
}

/// Import state for one Figma Paint style.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DesignPaintStyleImportState {
    #[default]
    Imported,
    Available,
}

/// One complete ordered Figma Paint style snapshot.
///
/// A style may contain several solid, gradient, image, pattern, video, or
/// shader paints. Applying it replaces the selected Fill or Stroke paint
/// collection as one host-owned operation; it never targets a single color
/// leaf.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignPaintStyle {
    pub id: SharedString,
    pub name: SharedString,
    pub paints: Vec<DesignPaint>,
    pub import_state: DesignPaintStyleImportState,
}

impl DesignPaintStyle {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        paints: impl IntoIterator<Item = DesignPaint>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            paints: paints.into_iter().collect(),
            import_state: DesignPaintStyleImportState::Imported,
        }
    }

    pub const fn with_import_state(mut self, import_state: DesignPaintStyleImportState) -> Self {
        self.import_state = import_state;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignPaintStyleLibrary {
    pub id: SharedString,
    pub name: SharedString,
    pub styles: Vec<DesignPaintStyle>,
}

impl DesignPaintStyleLibrary {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        styles: impl IntoIterator<Item = DesignPaintStyle>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            styles: styles.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DesignPaintStyleSource {
    Page,
    Library { library_id: SharedString },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DesignPaintStyleSelection {
    pub source: DesignPaintStyleSource,
    pub style_id: SharedString,
}

impl DesignPaintStyleSelection {
    pub fn page(style_id: impl Into<SharedString>) -> Self {
        Self {
            source: DesignPaintStyleSource::Page,
            style_id: style_id.into(),
        }
    }

    pub fn library(library_id: impl Into<SharedString>, style_id: impl Into<SharedString>) -> Self {
        Self {
            source: DesignPaintStyleSource::Library {
                library_id: library_id.into(),
            },
            style_id: style_id.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesignPaintStyleViewData {
    pub page_styles: Vec<DesignPaintStyle>,
    pub libraries: Vec<DesignPaintStyleLibrary>,
}

impl DesignPaintStyleViewData {
    pub fn new(
        page_styles: impl IntoIterator<Item = DesignPaintStyle>,
        libraries: impl IntoIterator<Item = DesignPaintStyleLibrary>,
    ) -> Self {
        Self {
            page_styles: page_styles.into_iter().collect(),
            libraries: libraries.into_iter().collect(),
        }
    }

    pub fn style(&self, selection: &DesignPaintStyleSelection) -> Option<&DesignPaintStyle> {
        match &selection.source {
            DesignPaintStyleSource::Page => self
                .page_styles
                .iter()
                .find(|style| style.id == selection.style_id),
            DesignPaintStyleSource::Library { library_id } => self
                .libraries
                .iter()
                .find(|library| library.id == *library_id)?
                .styles
                .iter()
                .find(|style| style.id == selection.style_id),
        }
    }

    pub fn style_mut(
        &mut self,
        selection: &DesignPaintStyleSelection,
    ) -> Option<&mut DesignPaintStyle> {
        match &selection.source {
            DesignPaintStyleSource::Page => self
                .page_styles
                .iter_mut()
                .find(|style| style.id == selection.style_id),
            DesignPaintStyleSource::Library { library_id } => self
                .libraries
                .iter_mut()
                .find(|library| library.id == *library_id)?
                .styles
                .iter_mut()
                .find(|style| style.id == selection.style_id),
        }
    }
}

/// Whole-collection `fillStyleId` or `strokeStyleId` binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignPaintStyleBinding {
    pub selection: DesignPaintStyleSelection,
    pub name: SharedString,
    pub can_detach: bool,
}

impl DesignPaintStyleBinding {
    pub fn new(selection: DesignPaintStyleSelection, name: impl Into<SharedString>) -> Self {
        Self {
            selection,
            name: name.into(),
            can_detach: true,
        }
    }

    pub const fn detachable(mut self, can_detach: bool) -> Self {
        self.can_detach = can_detach;
        self
    }
}

impl DesignColorStyleViewData {
    pub fn new(
        page_styles: impl IntoIterator<Item = DesignColorStyle>,
        libraries: impl IntoIterator<Item = DesignColorStyleLibrary>,
    ) -> Self {
        Self {
            page_styles: page_styles.into_iter().collect(),
            libraries: libraries.into_iter().collect(),
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.page_styles.is_empty() && self.libraries.is_empty()
    }
}

/// Stable scope for a color-style application intent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignColorStyleSource {
    Page,
    Library { library_id: SharedString },
}

/// Opaque style selection returned to the host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignColorStyleSelection {
    pub source: DesignColorStyleSource,
    pub style_id: SharedString,
}

impl DesignColorStyleSelection {
    pub fn page(style_id: impl Into<SharedString>) -> Self {
        Self {
            source: DesignColorStyleSource::Page,
            style_id: style_id.into(),
        }
    }

    pub fn library(library_id: impl Into<SharedString>, style_id: impl Into<SharedString>) -> Self {
        Self {
            source: DesignColorStyleSource::Library {
                library_id: library_id.into(),
            },
            style_id: style_id.into(),
        }
    }
}

/// The exact color leaf within a paint that receives a selected style.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignPaintColorTarget {
    Solid,
    GradientStop { stop_id: SharedString, index: usize },
}

/// Display-space affine transform used by gradients and source paints.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesignPaintTransform {
    pub m11: f32,
    pub m12: f32,
    pub m21: f32,
    pub m22: f32,
    pub tx: f32,
    pub ty: f32,
}

impl DesignPaintTransform {
    pub const IDENTITY: Self = Self {
        m11: 1.,
        m12: 0.,
        m21: 0.,
        m22: 1.,
        tx: 0.,
        ty: 0.,
    };

    pub fn rotated(self, degrees: f32) -> Self {
        let radians = degrees.to_radians();
        let cosine = radians.cos();
        let sine = radians.sin();
        self.prepend(Self {
            m11: cosine,
            m12: sine,
            m21: -sine,
            m22: cosine,
            tx: 0.,
            ty: 0.,
        })
    }

    pub fn flipped_horizontal(self) -> Self {
        self.prepend(Self {
            m11: -1.,
            m12: 0.,
            m21: 0.,
            m22: 1.,
            tx: 1.,
            ty: 0.,
        })
    }

    pub fn flipped_vertical(self) -> Self {
        self.prepend(Self {
            m11: 1.,
            m12: 0.,
            m21: 0.,
            m22: -1.,
            tx: 0.,
            ty: 1.,
        })
    }

    fn prepend(self, operation: Self) -> Self {
        Self {
            m11: operation.m11 * self.m11 + operation.m21 * self.m12,
            m12: operation.m12 * self.m11 + operation.m22 * self.m12,
            m21: operation.m11 * self.m21 + operation.m21 * self.m22,
            m22: operation.m12 * self.m21 + operation.m22 * self.m22,
            tx: operation.m11 * self.tx + operation.m21 * self.ty + operation.tx,
            ty: operation.m12 * self.tx + operation.m22 * self.ty + operation.ty,
        }
    }
}

impl Default for DesignPaintTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

fn paint_transform_is_finite(transform: DesignPaintTransform) -> bool {
    [
        transform.m11,
        transform.m12,
        transform.m21,
        transform.m22,
        transform.tx,
        transform.ty,
    ]
    .into_iter()
    .all(f32::is_finite)
}

/// Figma `PatternPaint.tileType`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignPatternTileType {
    Rectangular,
    HorizontalHexagonal,
    VerticalHexagonal,
}

impl DesignPatternTileType {
    pub const ALL: [Self; 3] = [
        Self::Rectangular,
        Self::HorizontalHexagonal,
        Self::VerticalHexagonal,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Rectangular => "Rectangular",
            Self::HorizontalHexagonal => "Horizontal hex",
            Self::VerticalHexagonal => "Vertical hex",
        }
    }
}

/// Figma `PatternPaint.horizontalAlignment`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignPatternHorizontalAlignment {
    Start,
    Center,
    End,
}

impl DesignPatternHorizontalAlignment {
    pub const ALL: [Self; 3] = [Self::Start, Self::Center, Self::End];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::Center => "Center",
            Self::End => "End",
        }
    }
}

/// Figma `Vector` spacing for a pattern tile.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DesignPatternSpacing {
    pub x: f32,
    pub y: f32,
}

impl DesignPatternSpacing {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignSolidPaint {
    pub color: DesignColor,
    pub binding: Option<DesignPaintBinding>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignGradientPaint {
    pub kind: DesignPaintKind,
    pub stops: Vec<DesignGradientStop>,
    pub transform: DesignPaintTransform,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignPatternPaint {
    /// Exact Figma scene-node id used by `setFillsAsync`/`setStrokesAsync`.
    pub source_node_id: SharedString,
    pub tile_type: DesignPatternTileType,
    pub scaling_factor: f32,
    pub spacing: DesignPatternSpacing,
    pub horizontal_alignment: DesignPatternHorizontalAlignment,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignImagePaint {
    pub source: DesignPaintSource,
    pub placement: DesignMediaPaintPlacement,
    pub filters: DesignImageFilters,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignVideoPaint {
    pub source: DesignPaintSource,
    pub placement: DesignMediaPaintPlacement,
    pub filters: DesignImageFilters,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignShaderPropertyDefinition {
    /// Definition id, not the author-facing property name.
    pub id: SharedString,
    pub name: SharedString,
    pub kind: DesignShaderPropertyKind,
    pub default_value: Option<DesignShaderPropertyValue>,
    pub description: Option<SharedString>,
}

impl DesignShaderPropertyDefinition {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        kind: DesignShaderPropertyKind,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind,
            default_value: None,
            description: None,
        }
    }

    pub fn with_default(mut self, value: DesignShaderPropertyValue) -> Self {
        debug_assert!(value.is_compatible_with(self.kind));
        self.default_value = Some(value);
        self
    }

    pub fn with_description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignShaderPropertyAssignment {
    pub definition_id: SharedString,
    pub value: DesignShaderPropertyValue,
}

impl DesignShaderPropertyAssignment {
    pub fn new(definition_id: impl Into<SharedString>, value: DesignShaderPropertyValue) -> Self {
        Self {
            definition_id: definition_id.into(),
            value,
        }
    }
}

/// One fill shader returned by the host's shader discovery/import layer.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignShaderDefinition {
    pub id: SharedString,
    pub name: SharedString,
    pub imported: bool,
    /// Present after import. Empty while an available library shader is only
    /// discoverable, matching Figma's `listAvailableShaders` contract.
    pub property_definitions: Vec<DesignShaderPropertyDefinition>,
}

impl DesignShaderDefinition {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        imported: bool,
        property_definitions: impl IntoIterator<Item = DesignShaderPropertyDefinition>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            imported,
            property_definitions: property_definitions.into_iter().collect(),
        }
    }

    pub fn property(&self, definition_id: &str) -> Option<&DesignShaderPropertyDefinition> {
        self.property_definitions
            .iter()
            .find(|definition| definition.id == definition_id)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignShaderLibrary {
    pub id: SharedString,
    pub name: SharedString,
    pub shaders: Vec<DesignShaderDefinition>,
}

impl DesignShaderLibrary {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        shaders: impl IntoIterator<Item = DesignShaderDefinition>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            shaders: shaders.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesignShaderViewData {
    pub page_shaders: Vec<DesignShaderDefinition>,
    pub libraries: Vec<DesignShaderLibrary>,
}

impl DesignShaderViewData {
    pub fn new(
        page_shaders: impl IntoIterator<Item = DesignShaderDefinition>,
        libraries: impl IntoIterator<Item = DesignShaderLibrary>,
    ) -> Self {
        Self {
            page_shaders: page_shaders.into_iter().collect(),
            libraries: libraries.into_iter().collect(),
        }
    }

    pub fn shader(&self, selection: &DesignShaderSelection) -> Option<&DesignShaderDefinition> {
        match &selection.source {
            DesignShaderSource::Page => self
                .page_shaders
                .iter()
                .find(|shader| shader.id == selection.shader_id),
            DesignShaderSource::Library { library_id } => self
                .libraries
                .iter()
                .find(|library| library.id == *library_id)?
                .shaders
                .iter()
                .find(|shader| shader.id == selection.shader_id),
        }
    }

    pub fn definition(&self, shader_id: &str) -> Option<&DesignShaderDefinition> {
        self.page_shaders
            .iter()
            .chain(
                self.libraries
                    .iter()
                    .flat_map(|library| library.shaders.iter()),
            )
            .find(|shader| shader.id == shader_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignShaderSource {
    Page,
    Library { library_id: SharedString },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignShaderSelection {
    pub source: DesignShaderSource,
    pub shader_id: SharedString,
}

impl DesignShaderSelection {
    pub fn page(shader_id: impl Into<SharedString>) -> Self {
        Self {
            source: DesignShaderSource::Page,
            shader_id: shader_id.into(),
        }
    }

    pub fn library(
        library_id: impl Into<SharedString>,
        shader_id: impl Into<SharedString>,
    ) -> Self {
        Self {
            source: DesignShaderSource::Library {
                library_id: library_id.into(),
            },
            shader_id: shader_id.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignShaderPaint {
    /// Shader id returned by Figma discovery/import, distinct from the stable
    /// identity of the paint row that owns this payload.
    pub shader_id: SharedString,
    pub name: SharedString,
    /// Current assignments keyed by definition id.
    pub properties: Vec<DesignShaderPropertyAssignment>,
}

impl DesignShaderPaint {
    pub fn new(shader_id: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self {
            shader_id: shader_id.into(),
            name: name.into(),
            properties: Vec::new(),
        }
    }

    pub fn with_properties(
        mut self,
        properties: impl IntoIterator<Item = DesignShaderPropertyAssignment>,
    ) -> Self {
        self.properties = properties.into_iter().collect();
        self
    }

    /// Builds the paint assignments Figma exposes after applying an imported
    /// shader, including every author default keyed by definition id.
    pub fn from_definition(definition: &DesignShaderDefinition) -> Option<Self> {
        if !definition.imported {
            return None;
        }
        Some(
            Self::new(definition.id.clone(), definition.name.clone()).with_properties(
                definition
                    .property_definitions
                    .iter()
                    .filter_map(|property| {
                        property.default_value.clone().map(|value| {
                            DesignShaderPropertyAssignment::new(property.id.clone(), value)
                        })
                    }),
            ),
        )
    }

    pub fn property(&self, definition_id: &str) -> Option<&DesignShaderPropertyValue> {
        self.properties
            .iter()
            .find(|assignment| assignment.definition_id == definition_id)
            .map(|assignment| &assignment.value)
    }

    pub fn property_assignment(
        &self,
        definition_id: &str,
    ) -> Option<&DesignShaderPropertyAssignment> {
        self.properties
            .iter()
            .find(|assignment| assignment.definition_id == definition_id)
    }
}

/// Lossless payload for paint kinds unknown to the built-in picker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignOpaquePaint {
    pub type_name: SharedString,
    pub payload: SharedString,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DesignPaintPayload {
    Solid(DesignSolidPaint),
    Gradient(DesignGradientPaint),
    Pattern(DesignPatternPaint),
    Image(DesignImagePaint),
    Video(DesignVideoPaint),
    Shader(DesignShaderPaint),
    Unsupported(DesignOpaquePaint),
}

impl DesignPaintPayload {
    pub const fn kind(&self) -> DesignPaintKind {
        match self {
            Self::Solid(_) => DesignPaintKind::Solid,
            Self::Gradient(gradient) => gradient.kind,
            Self::Pattern(_) => DesignPaintKind::Pattern,
            Self::Image(_) => DesignPaintKind::Image,
            Self::Video(_) => DesignPaintKind::Video,
            Self::Shader(_) => DesignPaintKind::Shader,
            Self::Unsupported(_) => DesignPaintKind::Unsupported,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignPaint {
    /// Stable opaque host identity. Empty retains legacy index targeting.
    pub id: SharedString,
    /// Compatibility projection of [`Self::payload`].
    pub kind: DesignPaintKind,
    /// Compatibility projection of the solid color or first gradient stop.
    pub color: DesignColor,
    pub opacity: f32,
    pub visible: bool,
    pub blend_mode: DesignBlendMode,
    /// Compatibility projection of gradient payload stops.
    pub gradient_stops: Vec<DesignGradientStop>,
    /// Canonical type-specific paint data.
    pub payload: DesignPaintPayload,
    /// Host-declared read-only state independent of document permissions.
    pub read_only: bool,
}

impl DesignPaint {
    pub const fn paint_type(&self) -> DesignPaintType {
        self.kind.paint_type()
    }

    pub fn solid(color: DesignColor) -> Self {
        Self {
            id: "".into(),
            kind: DesignPaintKind::Solid,
            color,
            opacity: 100.,
            visible: true,
            blend_mode: DesignBlendMode::Normal,
            gradient_stops: Vec::new(),
            payload: DesignPaintPayload::Solid(DesignSolidPaint {
                color,
                binding: None,
            }),
            read_only: false,
        }
    }

    pub fn gradient(kind: DesignPaintKind, stops: Vec<DesignGradientStop>) -> Self {
        debug_assert!(kind.is_gradient());
        let mut paint = Self {
            id: "".into(),
            kind,
            color: stops
                .first()
                .map(|stop| stop.color)
                .unwrap_or(DesignColor::BLACK),
            opacity: 100.,
            visible: true,
            blend_mode: DesignBlendMode::Normal,
            gradient_stops: stops.clone(),
            payload: DesignPaintPayload::Gradient(DesignGradientPaint {
                kind,
                stops,
                transform: DesignPaintTransform::IDENTITY,
            }),
            read_only: false,
        };
        paint.sync_legacy_projection();
        paint
    }

    pub fn pattern(source_node_id: impl Into<SharedString>) -> Self {
        Self::from_payload(DesignPaintPayload::Pattern(DesignPatternPaint {
            source_node_id: source_node_id.into(),
            tile_type: DesignPatternTileType::Rectangular,
            scaling_factor: 1.,
            spacing: DesignPatternSpacing::default(),
            horizontal_alignment: DesignPatternHorizontalAlignment::Center,
        }))
    }

    pub fn image(source: DesignPaintSource) -> Self {
        Self::from_payload(DesignPaintPayload::Image(DesignImagePaint {
            source,
            placement: DesignMediaPaintPlacement::default(),
            filters: DesignImageFilters::default(),
        }))
    }

    pub fn video(source: DesignPaintSource) -> Self {
        Self::from_payload(DesignPaintPayload::Video(DesignVideoPaint {
            source,
            placement: DesignMediaPaintPlacement::default(),
            filters: DesignImageFilters::default(),
        }))
    }

    pub fn shader(shader_id: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self::from_payload(DesignPaintPayload::Shader(DesignShaderPaint::new(
            shader_id, name,
        )))
    }

    pub fn unsupported(
        type_name: impl Into<SharedString>,
        payload: impl Into<SharedString>,
    ) -> Self {
        Self::from_payload(DesignPaintPayload::Unsupported(DesignOpaquePaint {
            type_name: type_name.into(),
            payload: payload.into(),
        }))
    }

    pub fn from_payload(payload: DesignPaintPayload) -> Self {
        let mut paint = Self {
            id: "".into(),
            kind: payload.kind(),
            color: DesignColor::BLACK,
            opacity: 100.,
            visible: true,
            blend_mode: DesignBlendMode::Normal,
            gradient_stops: Vec::new(),
            payload,
            read_only: false,
        };
        paint.sync_legacy_projection();
        paint
    }

    pub fn with_id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_blend_mode(mut self, blend_mode: DesignBlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    pub const fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn is_bound(&self) -> bool {
        match &self.payload {
            DesignPaintPayload::Solid(solid) => solid.binding.is_some(),
            DesignPaintPayload::Gradient(gradient) => {
                gradient.stops.iter().any(|stop| stop.binding.is_some())
            }
            DesignPaintPayload::Shader(shader) => shader
                .properties
                .iter()
                .any(|property| property.value.variable_alias_id().is_some()),
            DesignPaintPayload::Pattern(_)
            | DesignPaintPayload::Image(_)
            | DesignPaintPayload::Video(_)
            | DesignPaintPayload::Unsupported(_) => false,
        }
    }

    pub fn selected_color_is_bound(&self, stop_index: usize) -> bool {
        match &self.payload {
            DesignPaintPayload::Solid(solid) => solid.binding.is_some(),
            DesignPaintPayload::Gradient(gradient) => gradient
                .stops
                .get(stop_index.min(gradient.stops.len().saturating_sub(1)))
                .is_some_and(|stop| stop.binding.is_some()),
            _ => false,
        }
    }

    pub fn sync_legacy_projection(&mut self) {
        self.kind = self.payload.kind();
        match &self.payload {
            DesignPaintPayload::Solid(solid) => {
                self.color = solid.color;
                self.gradient_stops.clear();
            }
            DesignPaintPayload::Gradient(gradient) => {
                self.gradient_stops = gradient.stops.clone();
                self.color = gradient
                    .stops
                    .first()
                    .map_or(DesignColor::BLACK, |stop| stop.color);
            }
            DesignPaintPayload::Pattern(_)
            | DesignPaintPayload::Image(_)
            | DesignPaintPayload::Video(_)
            | DesignPaintPayload::Shader(_)
            | DesignPaintPayload::Unsupported(_) => {
                self.gradient_stops.clear();
            }
        }
    }

    pub fn apply_edit(&mut self, edit: &DesignPaintEdit) -> bool {
        let changed = match (&edit.property, &edit.value) {
            (DesignPaintProperty::Opacity, DesignPaintValue::Number(value)) => {
                self.opacity = value.clamp(0., 100.);
                true
            }
            (DesignPaintProperty::Visible, DesignPaintValue::Bool(value)) => {
                self.visible = *value;
                true
            }
            (DesignPaintProperty::BlendMode, DesignPaintValue::BlendMode(value)) => {
                self.blend_mode = *value;
                true
            }
            (DesignPaintProperty::Payload, DesignPaintValue::Payload(payload)) => {
                let mut payload = payload.clone();
                let valid = match &mut payload {
                    DesignPaintPayload::Image(image) => {
                        if let Some(filters) = image.filters.normalized() {
                            image.filters = filters;
                            image.placement.is_valid()
                        } else {
                            false
                        }
                    }
                    DesignPaintPayload::Video(video) => {
                        if let Some(filters) = video.filters.normalized() {
                            video.filters = filters;
                            video.placement.is_valid()
                        } else {
                            false
                        }
                    }
                    _ => true,
                };
                if valid {
                    self.payload = payload;
                }
                valid
            }
            (DesignPaintProperty::Color, DesignPaintValue::Color(color)) => match &mut self.payload
            {
                DesignPaintPayload::Solid(solid) if solid.binding.is_none() => {
                    solid.color = *color;
                    true
                }
                _ => false,
            },
            (DesignPaintProperty::GradientKind, DesignPaintValue::PaintKind(kind))
                if kind.is_gradient() =>
            {
                match &mut self.payload {
                    DesignPaintPayload::Gradient(gradient) => {
                        gradient.kind = *kind;
                        true
                    }
                    _ => false,
                }
            }
            (DesignPaintProperty::GradientTransform, DesignPaintValue::Transform(transform)) => {
                match &mut self.payload {
                    DesignPaintPayload::Gradient(gradient) => {
                        gradient.transform = *transform;
                        true
                    }
                    _ => false,
                }
            }
            (
                DesignPaintProperty::GradientStopColor { stop_id, index },
                DesignPaintValue::Color(color),
            ) => match &mut self.payload {
                DesignPaintPayload::Gradient(gradient) => {
                    if let Some(stop) = find_gradient_stop_mut(&mut gradient.stops, stop_id, *index)
                        && stop.binding.is_none()
                    {
                        stop.color = *color;
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            (
                DesignPaintProperty::GradientStopPosition { stop_id, index },
                DesignPaintValue::Number(position),
            ) => match &mut self.payload {
                DesignPaintPayload::Gradient(gradient) => {
                    if let Some(stop) = find_gradient_stop_mut(&mut gradient.stops, stop_id, *index)
                    {
                        stop.position = position.clamp(0., 1.);
                        gradient.stops.sort_by(|left, right| {
                            left.position
                                .partial_cmp(&right.position)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            (DesignPaintProperty::GradientStopAdd, DesignPaintValue::GradientStop(stop)) => {
                match &mut self.payload {
                    DesignPaintPayload::Gradient(gradient) => {
                        let mut stop = stop.clone();
                        stop.position = stop.position.clamp(0., 1.);
                        gradient.stops.push(stop);
                        gradient.stops.sort_by(|left, right| {
                            left.position
                                .partial_cmp(&right.position)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });
                        true
                    }
                    _ => false,
                }
            }
            (
                DesignPaintProperty::GradientStopRemove { stop_id, index },
                DesignPaintValue::None,
            ) => match &mut self.payload {
                DesignPaintPayload::Gradient(gradient) if gradient.stops.len() > 2 => {
                    if let Some(index) = find_gradient_stop_index(&gradient.stops, stop_id, *index)
                    {
                        gradient.stops.remove(index);
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            (DesignPaintProperty::Source, DesignPaintValue::Source(source)) => {
                match &mut self.payload {
                    DesignPaintPayload::Image(image) => {
                        image.source = source.clone();
                        true
                    }
                    DesignPaintPayload::Video(video) => {
                        video.source = source.clone();
                        true
                    }
                    _ => false,
                }
            }
            (
                DesignPaintProperty::PatternSourceNode,
                DesignPaintValue::PatternSourceNode(source_node_id),
            ) => match &mut self.payload {
                DesignPaintPayload::Pattern(pattern) => {
                    pattern.source_node_id = source_node_id.clone();
                    true
                }
                _ => false,
            },
            (DesignPaintProperty::MediaScaleMode, DesignPaintValue::MediaScaleMode(mode)) => {
                match &mut self.payload {
                    DesignPaintPayload::Image(image) => {
                        image.placement = image.placement.with_mode(*mode);
                        true
                    }
                    DesignPaintPayload::Video(video) => {
                        video.placement = video.placement.with_mode(*mode);
                        true
                    }
                    _ => false,
                }
            }
            (DesignPaintProperty::MediaCropTransform, DesignPaintValue::Transform(transform))
                if paint_transform_is_finite(*transform) =>
            {
                match &mut self.payload {
                    DesignPaintPayload::Image(DesignImagePaint {
                        placement: DesignMediaPaintPlacement::Crop { transform: current },
                        ..
                    }) => {
                        *current = *transform;
                        true
                    }
                    DesignPaintPayload::Video(DesignVideoPaint {
                        placement: DesignMediaPaintPlacement::Crop { transform: current },
                        ..
                    }) => {
                        *current = *transform;
                        true
                    }
                    _ => false,
                }
            }
            (
                DesignPaintProperty::MediaTileScalingFactor,
                DesignPaintValue::Number(scaling_factor),
            ) if scaling_factor.is_finite() && *scaling_factor > 0. => match &mut self.payload {
                DesignPaintPayload::Image(DesignImagePaint {
                    placement:
                        DesignMediaPaintPlacement::Tile {
                            scaling_factor: current,
                            ..
                        },
                    ..
                }) => {
                    *current = *scaling_factor;
                    true
                }
                DesignPaintPayload::Video(DesignVideoPaint {
                    placement:
                        DesignMediaPaintPlacement::Tile {
                            scaling_factor: current,
                            ..
                        },
                    ..
                }) => {
                    *current = *scaling_factor;
                    true
                }
                _ => false,
            },
            (
                DesignPaintProperty::MediaQuarterTurn,
                DesignPaintValue::MediaQuarterTurn(rotation),
            ) => match &mut self.payload {
                DesignPaintPayload::Image(DesignImagePaint {
                    placement:
                        DesignMediaPaintPlacement::Fill { rotation: current }
                        | DesignMediaPaintPlacement::Fit { rotation: current }
                        | DesignMediaPaintPlacement::Tile {
                            rotation: current, ..
                        },
                    ..
                }) => {
                    *current = *rotation;
                    true
                }
                DesignPaintPayload::Video(DesignVideoPaint {
                    placement:
                        DesignMediaPaintPlacement::Fill { rotation: current }
                        | DesignMediaPaintPlacement::Fit { rotation: current }
                        | DesignMediaPaintPlacement::Tile {
                            rotation: current, ..
                        },
                    ..
                }) => {
                    *current = *rotation;
                    true
                }
                _ => false,
            },
            (DesignPaintProperty::MediaFilter(filter), DesignPaintValue::Number(value))
                if value.is_finite() =>
            {
                match &mut self.payload {
                    DesignPaintPayload::Image(image) => image.filters.set_value(*filter, *value),
                    DesignPaintPayload::Video(video) => video.filters.set_value(*filter, *value),
                    _ => false,
                }
            }
            (
                DesignPaintProperty::PatternTileType,
                DesignPaintValue::PatternTileType(tile_type),
            ) => match &mut self.payload {
                DesignPaintPayload::Pattern(pattern) => {
                    pattern.tile_type = *tile_type;
                    true
                }
                _ => false,
            },
            (DesignPaintProperty::PatternScalingFactor, DesignPaintValue::Number(scale)) => {
                match &mut self.payload {
                    DesignPaintPayload::Pattern(pattern) => {
                        pattern.scaling_factor = scale.max(0.);
                        true
                    }
                    _ => false,
                }
            }
            (DesignPaintProperty::PatternSpacing, DesignPaintValue::PatternSpacing(spacing)) => {
                match &mut self.payload {
                    DesignPaintPayload::Pattern(pattern) => {
                        pattern.spacing = *spacing;
                        true
                    }
                    _ => false,
                }
            }
            (
                DesignPaintProperty::PatternHorizontalAlignment,
                DesignPaintValue::PatternHorizontalAlignment(alignment),
            ) => match &mut self.payload {
                DesignPaintPayload::Pattern(pattern) => {
                    pattern.horizontal_alignment = *alignment;
                    true
                }
                _ => false,
            },
            (
                DesignPaintProperty::ShaderProperty { definition_id },
                DesignPaintValue::ShaderProperty(value),
            ) => match &mut self.payload {
                DesignPaintPayload::Shader(shader) => {
                    if let Some(assignment) = shader
                        .properties
                        .iter_mut()
                        .find(|assignment| assignment.definition_id == *definition_id)
                    {
                        assignment.value = value.clone();
                    } else {
                        shader.properties.push(DesignShaderPropertyAssignment::new(
                            definition_id.clone(),
                            value.clone(),
                        ));
                    }
                    true
                }
                _ => false,
            },
            _ => false,
        };
        if changed {
            self.sync_legacy_projection();
        }
        changed
    }
}

fn find_gradient_stop_index(
    stops: &[DesignGradientStop],
    stop_id: &SharedString,
    fallback_index: usize,
) -> Option<usize> {
    if !stop_id.is_empty() {
        return stops.iter().position(|stop| stop.id == *stop_id);
    }
    (fallback_index < stops.len()).then_some(fallback_index)
}

fn find_gradient_stop_mut<'a>(
    stops: &'a mut [DesignGradientStop],
    stop_id: &SharedString,
    fallback_index: usize,
) -> Option<&'a mut DesignGradientStop> {
    let index = find_gradient_stop_index(stops, stop_id, fallback_index)?;
    stops.get_mut(index)
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DesignPaintProperty {
    Payload,
    Opacity,
    Visible,
    BlendMode,
    Color,
    GradientKind,
    GradientTransform,
    GradientStopColor { stop_id: SharedString, index: usize },
    GradientStopPosition { stop_id: SharedString, index: usize },
    GradientStopAdd,
    GradientStopRemove { stop_id: SharedString, index: usize },
    Source,
    MediaScaleMode,
    MediaCropTransform,
    MediaTileScalingFactor,
    MediaQuarterTurn,
    MediaFilter(DesignImageFilter),
    PatternSourceNode,
    PatternTileType,
    PatternScalingFactor,
    PatternSpacing,
    PatternHorizontalAlignment,
    ShaderProperty { definition_id: SharedString },
}

#[derive(Clone, Debug, PartialEq)]
pub enum DesignPaintValue {
    None,
    Bool(bool),
    Number(f32),
    Color(DesignColor),
    BlendMode(DesignBlendMode),
    PaintKind(DesignPaintKind),
    Transform(DesignPaintTransform),
    GradientStop(DesignGradientStop),
    Source(DesignPaintSource),
    MediaScaleMode(DesignMediaPaintScaleMode),
    MediaQuarterTurn(DesignMediaQuarterTurn),
    PatternSourceNode(SharedString),
    PatternTileType(DesignPatternTileType),
    PatternSpacing(DesignPatternSpacing),
    PatternHorizontalAlignment(DesignPatternHorizontalAlignment),
    ShaderProperty(DesignShaderPropertyValue),
    Payload(DesignPaintPayload),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignPaintEdit {
    pub property: DesignPaintProperty,
    pub value: DesignPaintValue,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignLayoutMode {
    None,
    Horizontal,
    Vertical,
    Grid,
}

impl DesignLayoutMode {
    pub const ALL: [Self; 4] = [Self::None, Self::Horizontal, Self::Vertical, Self::Grid];

    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "Freeform",
            Self::Horizontal => "Horizontal",
            Self::Vertical => "Vertical",
            Self::Grid => "Grid",
        }
    }
}

/// Host-controlled availability for a frame-preset catalog, group, or leaf.
///
/// Figma changes its device and asset templates independently of the document
/// schema. Keeping availability in the read model lets a host preserve the
/// exact current list and explain why a known option cannot be applied.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum DesignFramePresetAvailability {
    #[default]
    Available,
    Disabled {
        reason: SharedString,
    },
}

impl DesignFramePresetAvailability {
    pub const fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }

    pub const fn disabled_reason(&self) -> Option<&SharedString> {
        match self {
            Self::Available => None,
            Self::Disabled { reason } => Some(reason),
        }
    }

    pub fn disabled(reason: impl Into<SharedString>) -> Self {
        Self::Disabled {
            reason: reason.into(),
        }
    }
}

/// One exact frame size supplied by the host.
///
/// `id` is stable only within its group. The pair represented by
/// [`DesignFramePresetSelection`] is the canonical identity returned to the
/// host; labels and dimensions remain presentation/application data.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignFramePreset {
    pub id: SharedString,
    pub label: SharedString,
    pub width: f32,
    pub height: f32,
    pub availability: DesignFramePresetAvailability,
}

impl DesignFramePreset {
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        width: f32,
        height: f32,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            width,
            height,
            availability: DesignFramePresetAvailability::Available,
        }
    }

    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.availability = DesignFramePresetAvailability::disabled(reason);
        self
    }

    pub fn is_valid(&self) -> bool {
        !self.id.is_empty()
            && !self.label.is_empty()
            && self.width.is_finite()
            && self.width > 0.
            && self.height.is_finite()
            && self.height > 0.
            && self
                .availability
                .disabled_reason()
                .is_none_or(|reason| !reason.is_empty())
    }
}

/// Stable identity of one frame preset.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DesignFramePresetSelection {
    pub group_id: SharedString,
    pub preset_id: SharedString,
}

impl DesignFramePresetSelection {
    pub fn new(group_id: impl Into<SharedString>, preset_id: impl Into<SharedString>) -> Self {
        Self {
            group_id: group_id.into(),
            preset_id: preset_id.into(),
        }
    }
}

/// One ordered Figma-style frame-preset section such as Phone or Desktop.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignFramePresetGroup {
    pub id: SharedString,
    pub label: SharedString,
    pub presets: Vec<DesignFramePreset>,
    pub availability: DesignFramePresetAvailability,
}

impl DesignFramePresetGroup {
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        presets: impl IntoIterator<Item = DesignFramePreset>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            presets: presets.into_iter().collect(),
            availability: DesignFramePresetAvailability::Available,
        }
    }

    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.availability = DesignFramePresetAvailability::disabled(reason);
        self
    }

    pub fn selection(&self, preset_id: impl Into<SharedString>) -> DesignFramePresetSelection {
        DesignFramePresetSelection::new(self.id.clone(), preset_id)
    }

    pub fn is_valid(&self) -> bool {
        if self.id.is_empty()
            || self.label.is_empty()
            || self.presets.is_empty()
            || self
                .availability
                .disabled_reason()
                .is_some_and(|reason| reason.is_empty())
        {
            return false;
        }
        let mut preset_ids = HashSet::with_capacity(self.presets.len());
        self.presets
            .iter()
            .all(|preset| preset.is_valid() && preset_ids.insert(preset.id.clone()))
    }
}

/// Exact grouped frame-preset catalog for one selected Frame.
///
/// Binding the catalog to `target_node_id` prevents a late asynchronous
/// response from attaching device sizes to a different selection.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesignFramePresetViewData {
    pub target_node_id: SharedString,
    pub groups: Vec<DesignFramePresetGroup>,
    pub availability: DesignFramePresetAvailability,
}

impl DesignFramePresetViewData {
    pub fn new(
        target_node_id: impl Into<SharedString>,
        groups: impl IntoIterator<Item = DesignFramePresetGroup>,
    ) -> Self {
        Self {
            target_node_id: target_node_id.into(),
            groups: groups.into_iter().collect(),
            availability: DesignFramePresetAvailability::Available,
        }
    }

    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.availability = DesignFramePresetAvailability::disabled(reason);
        self
    }

    pub fn is_valid(&self) -> bool {
        if self.target_node_id.is_empty()
            || self.groups.is_empty()
            || self
                .availability
                .disabled_reason()
                .is_some_and(|reason| reason.is_empty())
        {
            return false;
        }
        let mut group_ids = HashSet::with_capacity(self.groups.len());
        self.groups
            .iter()
            .all(|group| group.is_valid() && group_ids.insert(group.id.clone()))
    }

    pub fn preset(
        &self,
        selection: &DesignFramePresetSelection,
    ) -> Option<(&DesignFramePresetGroup, &DesignFramePreset)> {
        let group = self
            .groups
            .iter()
            .find(|group| group.id == selection.group_id)?;
        let preset = group
            .presets
            .iter()
            .find(|preset| preset.id == selection.preset_id)?;
        Some((group, preset))
    }

    pub fn effective_disabled_reason(
        &self,
        selection: &DesignFramePresetSelection,
    ) -> Option<&SharedString> {
        if let Some(reason) = self.availability.disabled_reason() {
            return Some(reason);
        }
        let (group, preset) = self.preset(selection)?;
        group
            .availability
            .disabled_reason()
            .or_else(|| preset.availability.disabled_reason())
    }

    pub fn can_apply(&self, selection: &DesignFramePresetSelection) -> bool {
        self.is_valid()
            && self.preset(selection).is_some()
            && self.effective_disabled_reason(selection).is_none()
    }

    pub fn matching_dimensions(
        &self,
        width: f32,
        height: f32,
    ) -> Option<DesignFramePresetSelection> {
        self.groups.iter().find_map(|group| {
            group
                .presets
                .iter()
                .find(|preset| preset.width == width && preset.height == height)
                .map(|preset| group.selection(preset.id.clone()))
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignSizingMode {
    Fixed,
    Hug,
    Fill,
}

/// One axis in Figma's Width/Height resizing and min/max disclosure.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignLayoutDimensionAxis {
    Width,
    Height,
}

impl DesignLayoutDimensionAxis {
    pub const ALL: [Self; 2] = [Self::Width, Self::Height];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Width => "Width",
            Self::Height => "Height",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DesignAutoLayoutAlignment {
    pub x: u8,
    pub y: u8,
}

impl DesignAutoLayoutAlignment {
    pub const fn new(x: u8, y: u8) -> Self {
        Self {
            x: if x > 2 { 2 } else { x },
            y: if y > 2 { 2 } else { y },
        }
    }
}

impl DesignSizingMode {
    pub const ALL: [Self; 3] = [Self::Fixed, Self::Hug, Self::Fill];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Fixed => "Fixed",
            Self::Hug => "Hug",
            Self::Fill => "Fill",
        }
    }
}

/// How the primary-axis space between auto-layout children is distributed.
///
/// `Auto` corresponds to Figma's "Auto" gap (space between); [`DesignLayout::gap`]
/// remains the concrete gap used by `Fixed`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignItemSpacingMode {
    Fixed,
    Auto,
}

impl DesignItemSpacingMode {
    pub const ALL: [Self; 2] = [Self::Fixed, Self::Auto];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Fixed => "Fixed",
            Self::Auto => "Auto",
        }
    }
}

/// How wrapped tracks consume free space on the counter axis.
///
/// This maps directly to Figma's `counterAxisAlignContent` values. It is only
/// applicable to horizontal auto-layout with Wrap enabled.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignCounterAxisAlignContent {
    Auto,
    SpaceBetween,
}

impl DesignCounterAxisAlignContent {
    pub const ALL: [Self; 2] = [Self::Auto, Self::SpaceBetween];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::SpaceBetween => "Space between",
        }
    }
}

/// Whether a direct child participates in its parent's auto-layout flow.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignLayoutPositioning {
    InFlow,
    Absolute,
}

impl DesignLayoutPositioning {
    pub const ALL: [Self; 2] = [Self::InFlow, Self::Absolute];

    pub const fn label(self) -> &'static str {
        match self {
            Self::InFlow => "In auto layout",
            Self::Absolute => "Ignore auto layout",
        }
    }
}

/// Per-child cross-axis sizing inside an auto-layout parent.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignLayoutAlignSelf {
    Inherit,
    Stretch,
}

impl DesignLayoutAlignSelf {
    pub const ALL: [Self; 2] = [Self::Inherit, Self::Stretch];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Inherit => "Auto",
            Self::Stretch => "Stretch",
        }
    }
}

/// Canvas paint order for children whose bounds overlap.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignStackingOrder {
    LastOnTop,
    FirstOnTop,
}

impl DesignStackingOrder {
    pub const ALL: [Self; 2] = [Self::LastOnTop, Self::FirstOnTop];

    pub const fn label(self) -> &'static str {
        match self {
            Self::LastOnTop => "Last on top",
            Self::FirstOnTop => "First on top",
        }
    }
}

/// Whether horizontal auto-layout children align by their bounds or text baseline.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignBaselineAlignment {
    Bounds,
    Baseline,
}

impl DesignBaselineAlignment {
    pub const ALL: [Self; 2] = [Self::Bounds, Self::Baseline];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Bounds => "Bounds",
            Self::Baseline => "Baseline",
        }
    }
}

/// One explicit track axis in a grid auto-layout.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignGridTrackAxis {
    Column,
    Row,
}

impl DesignGridTrackAxis {
    pub const ALL: [Self; 2] = [Self::Column, Self::Row];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Column => "Column",
            Self::Row => "Row",
        }
    }
}

/// Positive row and column counts for one Grid auto-layout container.
///
/// A dimensions change is intentionally atomic: the panel never decomposes a
/// 2D selector choice into a sequence of per-track insertions or deletions.
/// The host owns track creation, removal, sizing, content relocation, and the
/// complete authoritative vectors echoed through [`DesignLayout`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DesignGridDimensions {
    pub columns: usize,
    pub rows: usize,
}

impl DesignGridDimensions {
    pub const fn new(columns: usize, rows: usize) -> Option<Self> {
        if columns == 0 || rows == 0 {
            None
        } else {
            Some(Self { columns, rows })
        }
    }

    pub const fn is_valid(self) -> bool {
        self.columns > 0 && self.rows > 0
    }
}

/// Whether Figma automatically creates and removes empty grid rows.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignGridAutoTracks {
    None,
    Rows,
}

impl DesignGridAutoTracks {
    pub const ALL: [Self; 2] = [Self::None, Self::Rows];

    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "Manual rows",
            Self::Rows => "Auto rows",
        }
    }
}

/// The sizing algorithm for one row or column in a grid auto-layout.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignGridTrackSizing {
    Fixed,
    Fraction,
    Hug,
}

impl DesignGridTrackSizing {
    pub const ALL: [Self; 3] = [Self::Fixed, Self::Fraction, Self::Hug];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Fixed => "Fixed",
            Self::Fraction => "Fraction",
            Self::Hug => "Hug",
        }
    }
}

/// A grid auto-layout track.
///
/// `value` is pixels for [`DesignGridTrackSizing::Fixed`], fractional units for
/// [`DesignGridTrackSizing::Fraction`], and ignored for
/// [`DesignGridTrackSizing::Hug`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesignGridTrack {
    pub sizing: DesignGridTrackSizing,
    pub value: f32,
}

impl DesignGridTrack {
    pub const fn fixed(pixels: f32) -> Self {
        Self {
            sizing: DesignGridTrackSizing::Fixed,
            value: pixels,
        }
    }

    pub const fn fraction(units: f32) -> Self {
        Self {
            sizing: DesignGridTrackSizing::Fraction,
            value: units,
        }
    }

    pub const fn hug() -> Self {
        Self {
            sizing: DesignGridTrackSizing::Hug,
            value: 0.,
        }
    }

    pub fn is_valid(self) -> bool {
        self.sizing == DesignGridTrackSizing::Hug || (self.value.is_finite() && self.value > 0.)
    }
}

impl Default for DesignGridTrack {
    fn default() -> Self {
        Self::hug()
    }
}

/// How children are assigned to grid cells.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignGridItemsPositioning {
    Manual,
    RowAutoFlow,
}

impl DesignGridItemsPositioning {
    pub const ALL: [Self; 2] = [Self::Manual, Self::RowAutoFlow];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Manual => "Manual",
            Self::RowAutoFlow => "Auto flow",
        }
    }
}

/// Per-item alignment within a grid auto-layout cell.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignGridItemAlignment {
    Auto,
    Start,
    Center,
    End,
}

impl DesignGridItemAlignment {
    pub const ALL: [Self; 4] = [Self::Auto, Self::Start, Self::Center, Self::End];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Start => "Start",
            Self::Center => "Center",
            Self::End => "End",
        }
    }
}

/// Properties that belong to a selected node as an item of an auto-layout parent.
///
/// Grid row and column indices are zero-based, matching Figma's plugin API.
/// Spans are always expected to be at least one.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignAutoLayoutItem {
    pub positioning: DesignLayoutPositioning,
    pub align_self: DesignLayoutAlignSelf,
    pub layout_grow: f32,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    /// For a Text/TextPath item, a numeric Max height is mutually exclusive
    /// with [`DesignTypography::max_lines`].
    pub max_height: Option<f32>,
    pub grid_row_index: usize,
    pub grid_column_index: usize,
    pub grid_row_span: u16,
    pub grid_column_span: u16,
    pub grid_horizontal_alignment: DesignGridItemAlignment,
    pub grid_vertical_alignment: DesignGridItemAlignment,
}

impl Default for DesignAutoLayoutItem {
    fn default() -> Self {
        Self {
            positioning: DesignLayoutPositioning::InFlow,
            align_self: DesignLayoutAlignSelf::Inherit,
            layout_grow: 0.,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            grid_row_index: 0,
            grid_column_index: 0,
            grid_row_span: 1,
            grid_column_span: 1,
            grid_horizontal_alignment: DesignGridItemAlignment::Auto,
            grid_vertical_alignment: DesignGridItemAlignment::Auto,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignLayout {
    pub mode: DesignLayoutMode,
    pub horizontal_sizing: DesignSizingMode,
    pub vertical_sizing: DesignSizingMode,
    pub alignment_x: u8,
    pub alignment_y: u8,
    pub wrap: bool,
    pub gap: f32,
    pub item_spacing_mode: DesignItemSpacingMode,
    /// Figma's `counterAxisSpacing`: `None` links wrapped-track spacing to the
    /// primary gap, while `Some` stores a positive custom spacing.
    ///
    /// Grid currently reuses this field for its explicit row gap and therefore
    /// always supplies `Some`.
    pub counter_axis_gap: Option<f32>,
    pub counter_axis_align_content: DesignCounterAxisAlignContent,
    /// Top, right, bottom, left.
    pub padding: [f32; 4],
    pub clip_content: bool,
    pub include_strokes: bool,
    pub stacking_order: DesignStackingOrder,
    pub baseline_alignment: DesignBaselineAlignment,
    pub grid_columns: Vec<DesignGridTrack>,
    pub grid_rows: Vec<DesignGridTrack>,
    pub grid_auto_tracks: DesignGridAutoTracks,
    pub grid_items_positioning: DesignGridItemsPositioning,
    pub item: DesignAutoLayoutItem,
}

impl Default for DesignLayout {
    fn default() -> Self {
        Self {
            mode: DesignLayoutMode::None,
            horizontal_sizing: DesignSizingMode::Fixed,
            vertical_sizing: DesignSizingMode::Fixed,
            alignment_x: 0,
            alignment_y: 0,
            wrap: false,
            gap: 8.,
            item_spacing_mode: DesignItemSpacingMode::Fixed,
            counter_axis_gap: None,
            counter_axis_align_content: DesignCounterAxisAlignContent::Auto,
            padding: [16.; 4],
            clip_content: true,
            include_strokes: false,
            stacking_order: DesignStackingOrder::LastOnTop,
            baseline_alignment: DesignBaselineAlignment::Bounds,
            grid_columns: Vec::new(),
            grid_rows: Vec::new(),
            grid_auto_tracks: DesignGridAutoTracks::None,
            grid_items_positioning: DesignGridItemsPositioning::Manual,
            item: DesignAutoLayoutItem::default(),
        }
    }
}

impl DesignLayout {
    /// Applies a flow only when the inspected node supports it.
    ///
    /// Returns `false` without changing the current flow when Figma would
    /// reject the operation (for example Grid on a Slot).
    pub fn set_mode_for_kind(&mut self, kind: DesignPanelNodeKind, mode: DesignLayoutMode) -> bool {
        self.set_mode_for_capabilities(
            kind.supports_auto_layout_container(),
            kind.supports_grid_auto_layout(),
            mode,
        )
    }

    /// Applies a flow against an exact host-authored node capability snapshot.
    pub fn set_mode_for_capabilities(
        &mut self,
        supports_auto_layout: bool,
        supports_grid: bool,
        mode: DesignLayoutMode,
    ) -> bool {
        if mode != DesignLayoutMode::None && !supports_auto_layout {
            return false;
        }
        if mode == DesignLayoutMode::Grid && !supports_grid {
            return false;
        }
        let leaving_grid = self.mode == DesignLayoutMode::Grid && mode != DesignLayoutMode::Grid;
        let entering_grid = self.mode != DesignLayoutMode::Grid && mode == DesignLayoutMode::Grid;
        self.mode = mode;
        if mode != DesignLayoutMode::Horizontal {
            self.wrap = false;
            if mode != DesignLayoutMode::Grid {
                self.counter_axis_align_content = DesignCounterAxisAlignContent::Auto;
                self.counter_axis_gap = None;
            }
        }
        if leaving_grid {
            self.counter_axis_align_content = DesignCounterAxisAlignContent::Auto;
            self.counter_axis_gap = None;
        }
        if entering_grid {
            self.horizontal_sizing = DesignSizingMode::Hug;
            self.vertical_sizing = DesignSizingMode::Hug;
            self.counter_axis_gap = Some(
                self.counter_axis_gap
                    .filter(|gap| gap.is_finite() && *gap >= 0.)
                    .unwrap_or_else(|| {
                        if self.gap.is_finite() {
                            self.gap.max(0.)
                        } else {
                            8.
                        }
                    }),
            );
            self.grid_columns = vec![DesignGridTrack::hug()];
            self.grid_rows = vec![DesignGridTrack::hug()];
            self.grid_auto_tracks = DesignGridAutoTracks::Rows;
            self.grid_items_positioning = DesignGridItemsPositioning::RowAutoFlow;
        }
        true
    }

    /// Applies Figma's horizontal-only Wrap invariant.
    pub fn set_wrap(&mut self, wrap: bool) {
        self.wrap = self.mode == DesignLayoutMode::Horizontal && wrap;
        if !self.wrap && self.mode != DesignLayoutMode::Grid {
            self.counter_axis_align_content = DesignCounterAxisAlignContent::Auto;
            self.counter_axis_gap = None;
        }
    }

    /// Applies wrapped-track distribution and clears spacing that Figma ignores.
    pub fn set_counter_axis_align_content(
        &mut self,
        alignment: DesignCounterAxisAlignContent,
    ) -> bool {
        if self.mode != DesignLayoutMode::Horizontal || !self.wrap {
            return false;
        }
        self.counter_axis_align_content = alignment;
        if alignment == DesignCounterAxisAlignContent::SpaceBetween {
            self.counter_axis_gap = None;
        }
        true
    }

    /// Applies Figma's nullable, positive wrapped-track spacing.
    pub fn set_counter_axis_gap(&mut self, spacing: Option<f32>) -> bool {
        if self.mode != DesignLayoutMode::Horizontal
            || !self.wrap
            || self.counter_axis_align_content != DesignCounterAxisAlignContent::Auto
            || spacing.is_some_and(|spacing| !spacing.is_finite() || spacing <= 0.)
        {
            return false;
        }
        self.counter_axis_gap = spacing;
        true
    }

    /// Canonicalizes imported host data to the node's Figma capabilities.
    pub fn normalize_for_kind(&mut self, kind: DesignPanelNodeKind) {
        if !kind.supports_auto_layout_container()
            || (self.mode == DesignLayoutMode::Grid && !kind.supports_grid_auto_layout())
        {
            self.mode = DesignLayoutMode::None;
        }
        if self.mode != DesignLayoutMode::Horizontal {
            self.wrap = false;
        }
        if self.mode == DesignLayoutMode::Horizontal && self.wrap {
            if self.counter_axis_align_content == DesignCounterAxisAlignContent::SpaceBetween
                || self
                    .counter_axis_gap
                    .is_some_and(|spacing| !spacing.is_finite() || spacing <= 0.)
            {
                self.counter_axis_gap = None;
            }
        } else if self.mode != DesignLayoutMode::Grid {
            self.counter_axis_align_content = DesignCounterAxisAlignContent::Auto;
            self.counter_axis_gap = None;
        }
        if self.mode == DesignLayoutMode::Grid {
            self.counter_axis_gap = Some(
                self.counter_axis_gap
                    .filter(|gap| gap.is_finite() && *gap >= 0.)
                    .unwrap_or_else(|| {
                        if self.gap.is_finite() {
                            self.gap.max(0.)
                        } else {
                            8.
                        }
                    }),
            );
            if self.grid_columns.is_empty() {
                self.grid_columns.push(DesignGridTrack::hug());
            }
            if self.grid_rows.is_empty() {
                self.grid_rows.push(DesignGridTrack::hug());
            }
            for track in &mut self.grid_columns {
                if !track.is_valid()
                    || (self.horizontal_sizing == DesignSizingMode::Hug
                        && track.sizing == DesignGridTrackSizing::Fraction)
                {
                    *track = DesignGridTrack::hug();
                }
            }
            for track in &mut self.grid_rows {
                if !track.is_valid()
                    || (self.vertical_sizing == DesignSizingMode::Hug
                        && track.sizing == DesignGridTrackSizing::Fraction)
                {
                    *track = DesignGridTrack::hug();
                }
            }
        }
    }

    pub fn grid_tracks(&self, axis: DesignGridTrackAxis) -> &[DesignGridTrack] {
        match axis {
            DesignGridTrackAxis::Column => &self.grid_columns,
            DesignGridTrackAxis::Row => &self.grid_rows,
        }
    }

    pub fn grid_track_count(&self, axis: DesignGridTrackAxis) -> usize {
        self.grid_tracks(axis).len()
    }

    pub fn grid_dimensions(&self) -> Option<DesignGridDimensions> {
        (self.mode == DesignLayoutMode::Grid)
            .then(|| DesignGridDimensions::new(self.grid_columns.len(), self.grid_rows.len()))
            .flatten()
    }

    pub fn grid_track_count_is_editable(&self, axis: DesignGridTrackAxis) -> bool {
        self.mode == DesignLayoutMode::Grid
            && !(axis == DesignGridTrackAxis::Row
                && self.grid_auto_tracks == DesignGridAutoTracks::Rows)
    }

    pub fn can_delete_grid_track(&self, axis: DesignGridTrackAxis) -> bool {
        self.grid_track_count_is_editable(axis) && self.grid_track_count(axis) > 1
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignConstraint {
    Left,
    Right,
    LeftAndRight,
    Center,
    Scale,
    Top,
    Bottom,
    TopAndBottom,
}

impl DesignConstraint {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
            Self::LeftAndRight => "Left & right",
            Self::Center => "Center",
            Self::Scale => "Scale",
            Self::Top => "Top",
            Self::Bottom => "Bottom",
            Self::TopAndBottom => "Top & bottom",
        }
    }
}

/// Figma mask evaluation mode for a scene node with `isMask = true`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignMaskType {
    Alpha,
    Vector,
    Luminance,
}

impl DesignMaskType {
    pub const ALL: [Self; 3] = [Self::Alpha, Self::Vector, Self::Luminance];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Alpha => "Alpha",
            Self::Vector => "Vector",
            Self::Luminance => "Luminance",
        }
    }

    pub fn parse_compatibility_label(label: &str) -> Option<Self> {
        match label.trim().to_ascii_lowercase().as_str() {
            "alpha" => Some(Self::Alpha),
            "vector" => Some(Self::Vector),
            "luminance" => Some(Self::Luminance),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignBlendMode {
    PassThrough,
    Normal,
    Darken,
    Multiply,
    LinearBurn,
    ColorBurn,
    Lighten,
    Screen,
    LinearDodge,
    ColorDodge,
    Overlay,
    SoftLight,
    HardLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl DesignBlendMode {
    /// The complete current Figma `BlendMode` set, in menu order.
    pub const ALL: [Self; 19] = [
        Self::PassThrough,
        Self::Normal,
        Self::Darken,
        Self::Multiply,
        Self::LinearBurn,
        Self::ColorBurn,
        Self::Lighten,
        Self::Screen,
        Self::LinearDodge,
        Self::ColorDodge,
        Self::Overlay,
        Self::SoftLight,
        Self::HardLight,
        Self::Difference,
        Self::Exclusion,
        Self::Hue,
        Self::Saturation,
        Self::Color,
        Self::Luminosity,
    ];

    /// Blend choices for paints and effects, where Pass through is not valid.
    pub const NON_PASS_THROUGH: [Self; 18] = [
        Self::Normal,
        Self::Darken,
        Self::Multiply,
        Self::LinearBurn,
        Self::ColorBurn,
        Self::Lighten,
        Self::Screen,
        Self::LinearDodge,
        Self::ColorDodge,
        Self::Overlay,
        Self::SoftLight,
        Self::HardLight,
        Self::Difference,
        Self::Exclusion,
        Self::Hue,
        Self::Saturation,
        Self::Color,
        Self::Luminosity,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::PassThrough => "Pass through",
            Self::Normal => "Normal",
            Self::Darken => "Darken",
            Self::Multiply => "Multiply",
            Self::LinearBurn => "Plus darker",
            Self::ColorBurn => "Color burn",
            Self::Lighten => "Lighten",
            Self::Screen => "Screen",
            Self::LinearDodge => "Plus lighter",
            Self::ColorDodge => "Color dodge",
            Self::Overlay => "Overlay",
            Self::SoftLight => "Soft light",
            Self::HardLight => "Hard light",
            Self::Difference => "Difference",
            Self::Exclusion => "Exclusion",
            Self::Hue => "Hue",
            Self::Saturation => "Saturation",
            Self::Color => "Color",
            Self::Luminosity => "Luminosity",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignStrokeAlign {
    Inside,
    Center,
    Outside,
}

impl DesignStrokeAlign {
    pub const ALL: [Self; 3] = [Self::Inside, Self::Center, Self::Outside];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Inside => "Inside",
            Self::Center => "Center",
            Self::Outside => "Outside",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignStrokeCap {
    None,
    Round,
    Square,
    LineArrow,
    TriangleArrow,
    ReverseTriangle,
    Diamond,
    Circle,
}

impl DesignStrokeCap {
    /// The complete Figma `StrokeCap` set.
    pub const ALL: [Self; 8] = [
        Self::None,
        Self::Round,
        Self::Square,
        Self::LineArrow,
        Self::TriangleArrow,
        Self::ReverseTriangle,
        Self::Diamond,
        Self::Circle,
    ];

    /// Compatibility alias for the former abbreviated line-arrow variant.
    #[allow(non_upper_case_globals)]
    #[deprecated(note = "use DesignStrokeCap::LineArrow")]
    pub const Arrow: Self = Self::LineArrow;

    /// Compatibility alias for the former abbreviated outward triangle variant.
    #[allow(non_upper_case_globals)]
    #[deprecated(note = "use DesignStrokeCap::TriangleArrow")]
    pub const Triangle: Self = Self::TriangleArrow;

    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Round => "Round",
            Self::Square => "Square",
            Self::LineArrow => "Line arrow",
            Self::TriangleArrow => "Triangle arrow",
            Self::ReverseTriangle => "Reverse triangle",
            Self::Diamond => "Diamond arrow",
            Self::Circle => "Circle",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignStrokeJoin {
    Miter,
    Bevel,
    Round,
}

impl DesignStrokeJoin {
    pub const ALL: [Self; 3] = [Self::Miter, Self::Bevel, Self::Round];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Miter => "Miter",
            Self::Bevel => "Bevel",
            Self::Round => "Rounded",
        }
    }
}

/// Which stroke-side editor Figma exposes while retaining all four weights.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignStrokeWeightMode {
    All,
    Top,
    Bottom,
    Left,
    Right,
    Custom,
}

impl DesignStrokeWeightMode {
    pub const ALL: [Self; 6] = [
        Self::All,
        Self::Top,
        Self::Bottom,
        Self::Left,
        Self::Right,
        Self::Custom,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Top => "Top",
            Self::Bottom => "Bottom",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Custom => "Custom",
        }
    }
}

/// Canonical top/right/bottom/left stroke weights.
///
/// The editor mode is presentation metadata; the four side values remain the
/// source of truth, including while the uniform `All` editor is active.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesignStrokeWeights {
    pub mode: DesignStrokeWeightMode,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl DesignStrokeWeights {
    pub const fn uniform(weight: f32) -> Self {
        Self {
            mode: DesignStrokeWeightMode::All,
            top: weight,
            right: weight,
            bottom: weight,
            left: weight,
        }
    }

    pub const fn custom(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            mode: DesignStrokeWeightMode::Custom,
            top,
            right,
            bottom,
            left,
        }
    }

    pub const fn active(self) -> f32 {
        match self.mode {
            DesignStrokeWeightMode::All
            | DesignStrokeWeightMode::Top
            | DesignStrokeWeightMode::Custom => self.top,
            DesignStrokeWeightMode::Bottom => self.bottom,
            DesignStrokeWeightMode::Left => self.left,
            DesignStrokeWeightMode::Right => self.right,
        }
    }

    pub fn set_uniform(&mut self, weight: f32) {
        self.top = weight;
        self.right = weight;
        self.bottom = weight;
        self.left = weight;
    }

    pub fn set_mode(&mut self, mode: DesignStrokeWeightMode) {
        let active = self.active();
        self.mode = mode;
        if mode == DesignStrokeWeightMode::All {
            self.set_uniform(active);
        }
    }

    pub fn set_active(&mut self, weight: f32) {
        match self.mode {
            DesignStrokeWeightMode::All => self.set_uniform(weight),
            DesignStrokeWeightMode::Top => self.top = weight,
            DesignStrokeWeightMode::Bottom => self.bottom = weight,
            DesignStrokeWeightMode::Left => self.left = weight,
            DesignStrokeWeightMode::Right => self.right = weight,
            DesignStrokeWeightMode::Custom => self.top = weight,
        }
    }

    pub const fn is_uniform(self) -> bool {
        self.top == self.right && self.top == self.bottom && self.top == self.left
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignStrokeDashMode {
    Solid,
    Dashed,
    Custom,
}

impl DesignStrokeDashMode {
    pub const ALL: [Self; 3] = [Self::Solid, Self::Dashed, Self::Custom];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Solid => "Solid",
            Self::Dashed => "Dashed",
            Self::Custom => "Custom",
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::Solid => Self::Dashed,
            Self::Dashed => Self::Custom,
            Self::Custom => Self::Solid,
        }
    }
}

/// Canonical active dash data plus the explicit Design-panel presentation mode.
///
/// Figma persists only an ordered `strokeDashes` array. The explicit mode keeps
/// an empty array selectable as Solid while still exposing a transition into
/// the two-value Dashed editor or the arbitrary ordered Custom editor.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignStrokeDashes {
    pub mode: DesignStrokeDashMode,
    pub pattern: Vec<f32>,
}

impl DesignStrokeDashes {
    pub const fn solid() -> Self {
        Self {
            mode: DesignStrokeDashMode::Solid,
            pattern: Vec::new(),
        }
    }

    pub fn dashed(dash: f32, gap: f32) -> Self {
        Self {
            mode: DesignStrokeDashMode::Dashed,
            pattern: vec![dash, gap],
        }
    }

    pub fn custom(pattern: impl IntoIterator<Item = f32>) -> Self {
        Self {
            mode: DesignStrokeDashMode::Custom,
            pattern: pattern.into_iter().collect(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.pattern
            .iter()
            .all(|value| value.is_finite() && *value >= 0.)
            && match self.mode {
                DesignStrokeDashMode::Solid => self.pattern.is_empty(),
                DesignStrokeDashMode::Dashed => self.pattern.len() == 2,
                DesignStrokeDashMode::Custom => !self.pattern.is_empty(),
            }
    }

    pub fn transition(&mut self, mode: DesignStrokeDashMode) {
        self.mode = mode;
        match mode {
            DesignStrokeDashMode::Solid => self.pattern.clear(),
            DesignStrokeDashMode::Dashed => {
                self.pattern = match self.pattern.as_slice() {
                    [dash, gap, ..]
                        if dash.is_finite() && *dash >= 0. && gap.is_finite() && *gap >= 0. =>
                    {
                        vec![*dash, *gap]
                    }
                    _ => vec![4., 4.],
                };
            }
            DesignStrokeDashMode::Custom => {
                if self.pattern.is_empty()
                    || !self
                        .pattern
                        .iter()
                        .all(|value| value.is_finite() && *value >= 0.)
                {
                    self.pattern = vec![4., 4., 1., 4.];
                }
            }
        }
    }

    pub fn set_pattern(&mut self, pattern: Vec<f32>) -> bool {
        let candidate = Self {
            mode: self.mode,
            pattern,
        };
        if !candidate.is_valid() {
            return false;
        }
        *self = candidate;
        true
    }

    pub fn is_solid(&self) -> bool {
        self.mode == DesignStrokeDashMode::Solid
    }
}

impl Default for DesignStrokeDashes {
    fn default() -> Self {
        Self::solid()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignVariableWidthPreset {
    Uniform,
    Wedge,
    Taper,
    QuarterTaper,
    Eye,
    MirroredTaper,
}

impl DesignVariableWidthPreset {
    pub const ALL: [Self; 6] = [
        Self::Uniform,
        Self::Wedge,
        Self::Taper,
        Self::QuarterTaper,
        Self::Eye,
        Self::MirroredTaper,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Uniform => "Uniform",
            Self::Wedge => "Wedge",
            Self::Taper => "Taper",
            Self::QuarterTaper => "Quarter taper",
            Self::Eye => "Eye",
            Self::MirroredTaper => "Mirrored taper",
        }
    }

    pub const fn api_name(self) -> &'static str {
        match self {
            Self::Uniform => "UNIFORM",
            Self::Wedge => "WEDGE",
            Self::Taper => "TAPER",
            Self::QuarterTaper => "QUARTER_TAPER",
            Self::Eye => "EYE",
            Self::MirroredTaper => "MIRRORED_TAPER",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesignVariableWidthPoint {
    pub position: f32,
    pub width: f32,
}

impl DesignVariableWidthPoint {
    pub const fn new(position: f32, width: f32) -> Self {
        Self { position, width }
    }

    pub fn is_valid(self) -> bool {
        self.position.is_finite()
            && (0. ..=1.).contains(&self.position)
            && self.width.is_finite()
            && self.width >= 0.
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DesignVariableWidthStroke {
    Preset(DesignVariableWidthPreset),
    Custom {
        /// Host order is canonical and is never sorted by the panel.
        points: Vec<DesignVariableWidthPoint>,
    },
}

impl DesignVariableWidthStroke {
    pub fn custom(points: impl IntoIterator<Item = DesignVariableWidthPoint>) -> Self {
        Self::Custom {
            points: points.into_iter().collect(),
        }
    }

    pub const fn label(&self) -> &'static str {
        match self {
            Self::Preset(preset) => preset.label(),
            Self::Custom { .. } => "Custom",
        }
    }

    pub fn points(&self) -> Option<&[DesignVariableWidthPoint]> {
        match self {
            Self::Preset(_) => None,
            Self::Custom { points } => Some(points),
        }
    }

    pub fn is_valid(&self) -> bool {
        match self {
            Self::Preset(_) => true,
            Self::Custom { points } => {
                !points.is_empty() && points.iter().all(|point| point.is_valid())
            }
        }
    }

    pub fn set_point_position(&mut self, index: usize, position: f32) -> bool {
        let Self::Custom { points } = self else {
            return false;
        };
        let Some(point) = points.get_mut(index) else {
            return false;
        };
        let candidate = DesignVariableWidthPoint { position, ..*point };
        if !candidate.is_valid() {
            return false;
        }
        *point = candidate;
        true
    }

    pub fn set_point_width(&mut self, index: usize, width: f32) -> bool {
        let Self::Custom { points } = self else {
            return false;
        };
        let Some(point) = points.get_mut(index) else {
            return false;
        };
        let candidate = DesignVariableWidthPoint { width, ..*point };
        if !candidate.is_valid() {
            return false;
        }
        *point = candidate;
        true
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignStrokeType {
    Basic,
    StretchBrush,
    ScatterBrush,
    Dynamic,
    Opaque,
}

impl DesignStrokeType {
    pub const EDITABLE: [Self; 4] = [
        Self::Basic,
        Self::StretchBrush,
        Self::ScatterBrush,
        Self::Dynamic,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::StretchBrush => "Stretch brush",
            Self::ScatterBrush => "Scatter brush",
            Self::Dynamic => "Dynamic",
            Self::Opaque => "Custom",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignStrokeBrushDirection {
    Forward,
    Backward,
}

impl DesignStrokeBrushDirection {
    pub const ALL: [Self; 2] = [Self::Forward, Self::Backward];

    #[allow(non_upper_case_globals)]
    #[deprecated(note = "use DesignStrokeBrushDirection::Backward")]
    pub const Reverse: Self = Self::Backward;

    pub const fn label(self) -> &'static str {
        match self {
            Self::Forward => "Forward",
            Self::Backward => "Backward",
        }
    }

    pub const fn api_name(self) -> &'static str {
        match self {
            Self::Forward => "FORWARD",
            Self::Backward => "BACKWARD",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignStretchBrushName {
    Heist,
    Blockbuster,
    Grindhouse,
    Biopic,
    SpaghettiWestern,
    Slasher,
    Hardboiled,
    Verite,
    Epic,
    Screwball,
    RomCom,
    Noir,
    Propaganda,
    Melodrama,
    NewWave,
}

impl DesignStretchBrushName {
    pub const ALL: [Self; 15] = [
        Self::Heist,
        Self::Blockbuster,
        Self::Grindhouse,
        Self::Biopic,
        Self::SpaghettiWestern,
        Self::Slasher,
        Self::Hardboiled,
        Self::Verite,
        Self::Epic,
        Self::Screwball,
        Self::RomCom,
        Self::Noir,
        Self::Propaganda,
        Self::Melodrama,
        Self::NewWave,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Heist => "Heist",
            Self::Blockbuster => "Blockbuster",
            Self::Grindhouse => "Grindhouse",
            Self::Biopic => "Biopic",
            Self::SpaghettiWestern => "Spaghetti western",
            Self::Slasher => "Slasher",
            Self::Hardboiled => "Hardboiled",
            Self::Verite => "Verite",
            Self::Epic => "Epic",
            Self::Screwball => "Screwball",
            Self::RomCom => "Rom com",
            Self::Noir => "Noir",
            Self::Propaganda => "Propaganda",
            Self::Melodrama => "Melodrama",
            Self::NewWave => "New wave",
        }
    }

    pub const fn api_name(self) -> &'static str {
        match self {
            Self::Heist => "HEIST",
            Self::Blockbuster => "BLOCKBUSTER",
            Self::Grindhouse => "GRINDHOUSE",
            Self::Biopic => "BIOPIC",
            Self::SpaghettiWestern => "SPAGHETTI_WESTERN",
            Self::Slasher => "SLASHER",
            Self::Hardboiled => "HARDBOILED",
            Self::Verite => "VERITE",
            Self::Epic => "EPIC",
            Self::Screwball => "SCREWBALL",
            Self::RomCom => "ROM_COM",
            Self::Noir => "NOIR",
            Self::Propaganda => "PROPAGANDA",
            Self::Melodrama => "MELODRAMA",
            Self::NewWave => "NEW_WAVE",
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::Heist => Self::Blockbuster,
            Self::Blockbuster => Self::Grindhouse,
            Self::Grindhouse => Self::Biopic,
            Self::Biopic => Self::SpaghettiWestern,
            Self::SpaghettiWestern => Self::Slasher,
            Self::Slasher => Self::Hardboiled,
            Self::Hardboiled => Self::Verite,
            Self::Verite => Self::Epic,
            Self::Epic => Self::Screwball,
            Self::Screwball => Self::RomCom,
            Self::RomCom => Self::Noir,
            Self::Noir => Self::Propaganda,
            Self::Propaganda => Self::Melodrama,
            Self::Melodrama => Self::NewWave,
            Self::NewWave => Self::Heist,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignScatterBrushName {
    Bubblegum,
    WitchHouse,
    Shoegaze,
    HonkyTonk,
    Screamo,
    Drone,
    DooWop,
    SpokenWord,
    Vaporwave,
    Oi,
}

impl DesignScatterBrushName {
    pub const ALL: [Self; 10] = [
        Self::Bubblegum,
        Self::WitchHouse,
        Self::Shoegaze,
        Self::HonkyTonk,
        Self::Screamo,
        Self::Drone,
        Self::DooWop,
        Self::SpokenWord,
        Self::Vaporwave,
        Self::Oi,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Bubblegum => "Bubblegum",
            Self::WitchHouse => "Witch house",
            Self::Shoegaze => "Shoegaze",
            Self::HonkyTonk => "Honky tonk",
            Self::Screamo => "Screamo",
            Self::Drone => "Drone",
            Self::DooWop => "Doo wop",
            Self::SpokenWord => "Spoken word",
            Self::Vaporwave => "Vaporwave",
            Self::Oi => "Oi",
        }
    }

    pub const fn api_name(self) -> &'static str {
        match self {
            Self::Bubblegum => "BUBBLEGUM",
            Self::WitchHouse => "WITCH_HOUSE",
            Self::Shoegaze => "SHOEGAZE",
            Self::HonkyTonk => "HONKY_TONK",
            Self::Screamo => "SCREAMO",
            Self::Drone => "DRONE",
            Self::DooWop => "DOO_WOP",
            Self::SpokenWord => "SPOKEN_WORD",
            Self::Vaporwave => "VAPORWAVE",
            Self::Oi => "OI",
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::Bubblegum => Self::WitchHouse,
            Self::WitchHouse => Self::Shoegaze,
            Self::Shoegaze => Self::HonkyTonk,
            Self::HonkyTonk => Self::Screamo,
            Self::Screamo => Self::Drone,
            Self::Drone => Self::DooWop,
            Self::DooWop => Self::SpokenWord,
            Self::SpokenWord => Self::Vaporwave,
            Self::Vaporwave => Self::Oi,
            Self::Oi => Self::Bubblegum,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DesignStretchBrushStroke {
    pub brush: DesignStretchBrushName,
    pub direction: DesignStrokeBrushDirection,
}

impl Default for DesignStretchBrushStroke {
    fn default() -> Self {
        Self {
            brush: DesignStretchBrushName::Heist,
            direction: DesignStrokeBrushDirection::Forward,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesignScatterBrushStroke {
    pub brush: DesignScatterBrushName,
    pub gap: f32,
    pub wiggle: f32,
    pub size_jitter: f32,
    pub angular_jitter: f32,
    pub rotation: f32,
}

impl DesignScatterBrushStroke {
    pub fn is_valid(self) -> bool {
        self.gap.is_finite()
            && self.gap >= 0.25
            && self.wiggle.is_finite()
            && self.wiggle >= 0.
            && self.size_jitter.is_finite()
            && (0. ..=3.).contains(&self.size_jitter)
            && self.angular_jitter.is_finite()
            && (-180. ..=180.).contains(&self.angular_jitter)
            && self.rotation.is_finite()
            && (-180. ..=180.).contains(&self.rotation)
    }
}

impl Default for DesignScatterBrushStroke {
    fn default() -> Self {
        Self {
            brush: DesignScatterBrushName::Bubblegum,
            gap: 0.25,
            wiggle: 0.,
            size_jitter: 0.,
            angular_jitter: 0.,
            rotation: 0.,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesignDynamicStroke {
    pub frequency: f32,
    pub wiggle: f32,
    pub smoothen: f32,
}

impl DesignDynamicStroke {
    pub fn is_valid(self) -> bool {
        self.frequency.is_finite()
            && (0.01..=20.).contains(&self.frequency)
            && self.wiggle.is_finite()
            && self.wiggle >= 0.
            && self.smoothen.is_finite()
            && (0. ..=1.).contains(&self.smoothen)
    }
}

impl Default for DesignDynamicStroke {
    fn default() -> Self {
        Self {
            frequency: 1.,
            wiggle: 0.,
            smoothen: 0.5,
        }
    }
}

/// Lossless host payload for custom brushes and future complex-stroke forms.
///
/// Figma returns `brushName: "CUSTOM"` but does not allow plugins to set that
/// brush. The panel therefore displays this payload without emitting edits.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignOpaqueComplexStroke {
    pub type_name: SharedString,
    pub label: SharedString,
    pub raw: SharedString,
}

impl DesignOpaqueComplexStroke {
    pub fn new(
        type_name: impl Into<SharedString>,
        label: impl Into<SharedString>,
        raw: impl Into<SharedString>,
    ) -> Self {
        Self {
            type_name: type_name.into(),
            label: label.into(),
            raw: raw.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum DesignComplexStroke {
    #[default]
    Basic,
    StretchBrush(DesignStretchBrushStroke),
    ScatterBrush(DesignScatterBrushStroke),
    Dynamic(DesignDynamicStroke),
    Opaque(DesignOpaqueComplexStroke),
}

impl DesignComplexStroke {
    pub const fn kind(&self) -> DesignStrokeType {
        match self {
            Self::Basic => DesignStrokeType::Basic,
            Self::StretchBrush(_) => DesignStrokeType::StretchBrush,
            Self::ScatterBrush(_) => DesignStrokeType::ScatterBrush,
            Self::Dynamic(_) => DesignStrokeType::Dynamic,
            Self::Opaque(_) => DesignStrokeType::Opaque,
        }
    }

    pub fn label(&self) -> SharedString {
        match self {
            Self::Opaque(opaque) => opaque.label.clone(),
            _ => self.kind().label().into(),
        }
    }

    pub const fn is_basic(&self) -> bool {
        matches!(self, Self::Basic)
    }

    pub const fn is_dynamic(&self) -> bool {
        matches!(self, Self::Dynamic(_))
    }

    pub const fn is_opaque(&self) -> bool {
        matches!(self, Self::Opaque(_))
    }

    pub fn editable_default(kind: DesignStrokeType) -> Option<Self> {
        match kind {
            DesignStrokeType::Basic => Some(Self::Basic),
            DesignStrokeType::StretchBrush => {
                Some(Self::StretchBrush(DesignStretchBrushStroke::default()))
            }
            DesignStrokeType::ScatterBrush => {
                Some(Self::ScatterBrush(DesignScatterBrushStroke::default()))
            }
            DesignStrokeType::Dynamic => Some(Self::Dynamic(DesignDynamicStroke::default())),
            DesignStrokeType::Opaque => None,
        }
    }

    pub fn is_valid(&self) -> bool {
        match self {
            Self::Basic | Self::StretchBrush(_) | Self::Opaque(_) => true,
            Self::ScatterBrush(stroke) => stroke.is_valid(),
            Self::Dynamic(stroke) => stroke.is_valid(),
        }
    }
}

/// Topology supplied by the document host for endpoint-sensitive controls.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignStrokePathTopology {
    Closed,
    Open,
    Branching,
}

/// Vector-edit context needed to place endpoint controls correctly.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DesignStrokeEditContext {
    pub topology: DesignStrokePathTopology,
    pub endpoint_count: u16,
    pub selected_vertices: u16,
    pub selected_endpoint_vertices: u16,
}

impl DesignStrokeEditContext {
    pub const fn closed() -> Self {
        Self {
            topology: DesignStrokePathTopology::Closed,
            endpoint_count: 0,
            selected_vertices: 0,
            selected_endpoint_vertices: 0,
        }
    }

    pub const fn open(endpoint_count: u16) -> Self {
        Self {
            topology: DesignStrokePathTopology::Open,
            endpoint_count,
            selected_vertices: 0,
            selected_endpoint_vertices: 0,
        }
    }

    pub const fn branching(endpoint_count: u16) -> Self {
        Self {
            topology: DesignStrokePathTopology::Branching,
            endpoint_count,
            selected_vertices: 0,
            selected_endpoint_vertices: 0,
        }
    }

    /// Compatibility shorthand for a selection made entirely of endpoints.
    pub const fn with_selected_vertices(mut self, selected_vertices: u16) -> Self {
        self.selected_vertices = selected_vertices;
        self.selected_endpoint_vertices = selected_vertices;
        self
    }

    /// Supplies the exact vector-edit selection and how many selected vertices
    /// are endpoints. Interior-only selections must not expose endpoint caps.
    pub const fn with_vertex_selection(
        mut self,
        selected_vertices: u16,
        selected_endpoint_vertices: u16,
    ) -> Self {
        self.selected_vertices = selected_vertices;
        self.selected_endpoint_vertices = if selected_endpoint_vertices > selected_vertices {
            selected_vertices
        } else {
            selected_endpoint_vertices
        };
        self
    }

    pub const fn endpoint_control(self) -> DesignStrokeEndpointControl {
        if self.endpoint_count == 0 {
            DesignStrokeEndpointControl::None
        } else if self.selected_endpoint_vertices > 0 {
            DesignStrokeEndpointControl::SelectedVertices
        } else if self.selected_vertices > 0 {
            DesignStrokeEndpointControl::None
        } else if matches!(self.topology, DesignStrokePathTopology::Open)
            && self.endpoint_count == 2
        {
            DesignStrokeEndpointControl::StartAndEnd
        } else {
            DesignStrokeEndpointControl::Aggregate
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignStrokeEndpointControl {
    None,
    StartAndEnd,
    Aggregate,
    SelectedVertices,
}

/// Host-declared support matrix for geometry controls.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DesignStrokeCapabilities {
    pub position: bool,
    pub individual_weights: bool,
    pub variable_width: bool,
    pub joins: bool,
    pub complex_stroke: bool,
}

impl DesignStrokeCapabilities {
    pub const fn for_node_kind(kind: DesignPanelNodeKind) -> Self {
        Self {
            // Figma fixes open Line strokes to Center and does not expose the
            // Inside/Center/Outside position control. `Arrow` is the
            // compatibility alias for a Line with an arrow endpoint.
            position: !matches!(kind, DesignPanelNodeKind::Line | DesignPanelNodeKind::Arrow),
            individual_weights: matches!(
                kind,
                DesignPanelNodeKind::Rectangle
                    | DesignPanelNodeKind::Frame
                    | DesignPanelNodeKind::Component
                    | DesignPanelNodeKind::ComponentSet
                    | DesignPanelNodeKind::Instance
                    | DesignPanelNodeKind::Slot
            ),
            variable_width: !matches!(kind, DesignPanelNodeKind::Pencil),
            joins: !matches!(kind, DesignPanelNodeKind::Line),
            complex_stroke: !matches!(
                kind,
                DesignPanelNodeKind::Image | DesignPanelNodeKind::Video
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignStroke {
    pub paints: Vec<DesignPaint>,
    pub weights: DesignStrokeWeights,
    pub align: DesignStrokeAlign,
    pub start_cap: DesignStrokeCap,
    pub end_cap: DesignStrokeCap,
    pub endpoint_cap: DesignStrokeCap,
    pub dashes: DesignStrokeDashes,
    pub dash_cap: DesignStrokeCap,
    pub join: DesignStrokeJoin,
    pub miter_angle: f32,
    pub variable_width: Option<DesignVariableWidthStroke>,
    pub complex_stroke: DesignComplexStroke,
    pub capabilities: DesignStrokeCapabilities,
    pub edit_context: DesignStrokeEditContext,
}

impl DesignStroke {
    pub fn for_node(
        kind: DesignPanelNodeKind,
        paint: DesignPaint,
        weight: f32,
        align: DesignStrokeAlign,
    ) -> Self {
        let capabilities = DesignStrokeCapabilities::for_node_kind(kind);
        let edit_context = if matches!(
            kind,
            DesignPanelNodeKind::Line
                | DesignPanelNodeKind::Arrow
                | DesignPanelNodeKind::Vector
                | DesignPanelNodeKind::Pen
                | DesignPanelNodeKind::Pencil
        ) {
            DesignStrokeEditContext::open(2)
        } else {
            DesignStrokeEditContext::closed()
        };
        Self {
            paints: vec![paint],
            weights: DesignStrokeWeights::uniform(weight),
            align: if capabilities.position {
                align
            } else {
                DesignStrokeAlign::Center
            },
            start_cap: DesignStrokeCap::None,
            end_cap: DesignStrokeCap::None,
            endpoint_cap: DesignStrokeCap::None,
            dashes: DesignStrokeDashes::solid(),
            dash_cap: DesignStrokeCap::None,
            join: DesignStrokeJoin::Miter,
            miter_angle: 90.,
            variable_width: None,
            complex_stroke: DesignComplexStroke::default(),
            capabilities,
            edit_context,
        }
    }

    /// Compatibility constructor for hosts that formerly supplied one paint.
    pub fn from_paint(paint: DesignPaint, weight: f32, align: DesignStrokeAlign) -> Self {
        Self::for_node(DesignPanelNodeKind::Rectangle, paint, weight, align)
    }

    pub fn alignment_options(&self) -> &'static [DesignStrokeAlign] {
        const CENTER_ONLY: &[DesignStrokeAlign] = &[DesignStrokeAlign::Center];
        const ALL: &[DesignStrokeAlign] = &DesignStrokeAlign::ALL;
        if self.capabilities.position && self.complex_stroke.is_basic() {
            ALL
        } else {
            CENTER_ONLY
        }
    }

    pub fn supports_variable_width(&self) -> bool {
        self.capabilities.variable_width
            && !self.complex_stroke.is_dynamic()
            && !self.complex_stroke.is_opaque()
            && self.edit_context.topology != DesignStrokePathTopology::Branching
    }

    pub fn set_type(&mut self, kind: DesignStrokeType) -> bool {
        if kind == DesignStrokeType::Opaque {
            return false;
        }
        if self.complex_stroke.kind() == kind {
            return true;
        }
        let Some(complex_stroke) = DesignComplexStroke::editable_default(kind) else {
            return false;
        };
        self.complex_stroke = complex_stroke;
        if kind != DesignStrokeType::Basic {
            self.align = DesignStrokeAlign::Center;
            self.dashes = DesignStrokeDashes::solid();
        }
        if kind == DesignStrokeType::Dynamic {
            self.variable_width = None;
        }
        true
    }

    pub fn set_dash_mode(&mut self, mode: DesignStrokeDashMode) -> bool {
        if !self.complex_stroke.is_basic() {
            return false;
        }
        self.dashes.transition(mode);
        true
    }

    pub fn set_dash_pattern(&mut self, dash_pattern: Vec<f32>) -> bool {
        self.complex_stroke.is_basic() && self.dashes.set_pattern(dash_pattern)
    }

    pub fn set_variable_width(
        &mut self,
        variable_width: Option<DesignVariableWidthStroke>,
    ) -> bool {
        if !self.supports_variable_width()
            || variable_width
                .as_ref()
                .is_some_and(|properties| !properties.is_valid())
        {
            return false;
        }
        self.variable_width = variable_width;
        true
    }

    pub fn add_paint(&mut self, paint: DesignPaint) {
        self.paints.push(paint);
    }

    pub fn remove_paint(&mut self, index: usize) -> Option<DesignPaint> {
        (index < self.paints.len()).then(|| self.paints.remove(index))
    }

    pub fn move_paint(&mut self, from: usize, to: usize) -> bool {
        if from >= self.paints.len() || to >= self.paints.len() || from == to {
            return from == to && from < self.paints.len();
        }
        let paint = self.paints.remove(from);
        self.paints.insert(to, paint);
        true
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignEffectKind {
    DropShadow,
    InnerShadow,
    LayerBlur,
    BackgroundBlur,
    Noise,
    Texture,
    Glass,
    Shader,
    /// A future effect type that this version of the inspector cannot edit.
    ///
    /// Hosts retain the original type and payload in
    /// [`DesignEffectSettings::Opaque`] so a newer document never loses data.
    Unsupported,
}

impl DesignEffectKind {
    /// Effect kinds that can be selected in the current Design panel.
    ///
    /// `Unsupported` is intentionally omitted: it is a read-only round-trip
    /// fallback for future document data, not a type that users can create.
    pub const ALL: [Self; 8] = [
        Self::DropShadow,
        Self::InnerShadow,
        Self::LayerBlur,
        Self::BackgroundBlur,
        Self::Noise,
        Self::Texture,
        Self::Glass,
        Self::Shader,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::DropShadow => "Drop shadow",
            Self::InnerShadow => "Inner shadow",
            Self::LayerBlur => "Layer blur",
            Self::BackgroundBlur => "Background blur",
            Self::Noise => "Noise",
            Self::Texture => "Texture",
            Self::Glass => "Glass",
            Self::Shader => "Shader",
            Self::Unsupported => "Unsupported effect",
        }
    }

    /// Maximum instances of this kind that Figma permits on one layer.
    pub const fn maximum_per_node(self) -> u8 {
        match self {
            Self::DropShadow | Self::InnerShadow => 8,
            Self::Noise => 2,
            Self::LayerBlur | Self::BackgroundBlur | Self::Texture | Self::Glass => 1,
            // Figma currently documents no per-node shader limit. The panel
            // therefore delegates any lower product limit to host
            // availability while retaining a finite collection guard.
            Self::Shader => u8::MAX,
            Self::Unsupported => 0,
        }
    }
}

/// Host-authored availability for one effect kind.
///
/// This is intentionally separate from Figma's documented per-kind count
/// limits. A host may need to disable Shader while no compatible shader is
/// imported, or disable a beta effect for a particular document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignEffectKindAvailability {
    pub kind: DesignEffectKind,
    pub available: bool,
    pub disabled_reason: Option<SharedString>,
}

impl DesignEffectKindAvailability {
    pub const fn available(kind: DesignEffectKind) -> Self {
        Self {
            kind,
            available: true,
            disabled_reason: None,
        }
    }

    pub fn unavailable(kind: DesignEffectKind, reason: impl Into<SharedString>) -> Self {
        Self {
            kind,
            available: false,
            disabled_reason: Some(reason.into()),
        }
    }
}

/// Exact node-context capabilities for effects whose applicability cannot be
/// inferred safely by a reusable inspector.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignEffectCapabilities {
    pub kind_availability: Vec<DesignEffectKindAvailability>,
    /// Whether Drop/Inner shadow spread is legal for this exact node.
    pub shadow_spread: bool,
    /// Whether Figma's "Show behind transparent areas" control is applicable
    /// for a Drop shadow on this exact node.
    pub show_shadow_behind_transparent_areas: bool,
}

impl DesignEffectCapabilities {
    pub fn for_node_kind(kind: DesignPanelNodeKind) -> Self {
        Self {
            kind_availability: DesignEffectKind::ALL
                .into_iter()
                .map(|effect_kind| {
                    if effect_kind == DesignEffectKind::Shader {
                        DesignEffectKindAvailability::unavailable(
                            effect_kind,
                            "No compatible shader is imported",
                        )
                    } else {
                        DesignEffectKindAvailability::available(effect_kind)
                    }
                })
                .collect(),
            shadow_spread: matches!(
                kind,
                DesignPanelNodeKind::Rectangle | DesignPanelNodeKind::Ellipse
            ),
            // Applicability depends on the host's resolved fills, strokes,
            // opacity, and blend modes; conservative is the only safe preset.
            show_shadow_behind_transparent_areas: false,
        }
    }

    pub fn availability(&self, kind: DesignEffectKind) -> Option<&DesignEffectKindAvailability> {
        self.kind_availability
            .iter()
            .find(|availability| availability.kind == kind)
    }

    pub fn kind_is_available(&self, kind: DesignEffectKind) -> bool {
        self.availability(kind)
            .is_some_and(|availability| availability.available)
    }

    pub fn set_kind_availability(&mut self, availability: DesignEffectKindAvailability) {
        if let Some(current) = self
            .kind_availability
            .iter_mut()
            .find(|current| current.kind == availability.kind)
        {
            *current = availability;
        } else {
            self.kind_availability.push(availability);
        }
    }
}

/// A two-axis value used by effect controls.
///
/// Blur control points use normalized object coordinates. Noise and texture
/// sizes use the same display-space units supplied by the host.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DesignEffectVector {
    pub x: f32,
    pub y: f32,
}

impl DesignEffectVector {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn splat(value: f32) -> Self {
        Self { x: value, y: value }
    }
}

/// Figma effect leaves that can be bound through
/// `setBoundVariableForEffect`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignEffectVariableField {
    Radius,
    Color,
    Spread,
    OffsetX,
    OffsetY,
}

impl DesignEffectVariableField {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Radius => "Blur",
            Self::Color => "Color",
            Self::Spread => "Spread",
            Self::OffsetX => "Offset X",
            Self::OffsetY => "Offset Y",
        }
    }

    pub const fn value_kind(self) -> DesignEffectVariableKind {
        match self {
            Self::Color => DesignEffectVariableKind::Color,
            Self::Radius | Self::Spread | Self::OffsetX | Self::OffsetY => {
                DesignEffectVariableKind::Float
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignEffectVariableKind {
    Float,
    Color,
}

/// One host-supplied variable shown for a bindable effect leaf.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignEffectVariable {
    pub id: SharedString,
    pub name: SharedString,
    pub collection_name: SharedString,
    pub kind: DesignEffectVariableKind,
    pub remote: bool,
}

impl DesignEffectVariable {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        collection_name: impl Into<SharedString>,
        kind: DesignEffectVariableKind,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            collection_name: collection_name.into(),
            kind,
            remote: false,
        }
    }

    pub const fn remote(mut self, remote: bool) -> Self {
        self.remote = remote;
        self
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DesignEffectVariableViewData {
    pub variables: Vec<DesignEffectVariable>,
}

impl DesignEffectVariableViewData {
    pub fn new(variables: impl IntoIterator<Item = DesignEffectVariable>) -> Self {
        Self {
            variables: variables.into_iter().collect(),
        }
    }

    pub fn variable(&self, variable_id: &str) -> Option<&DesignEffectVariable> {
        self.variables
            .iter()
            .find(|variable| variable.id.as_ref() == variable_id)
    }

    pub fn compatible(
        &self,
        field: DesignEffectVariableField,
    ) -> impl Iterator<Item = &DesignEffectVariable> {
        self.variables
            .iter()
            .filter(move |variable| variable.kind == field.value_kind())
    }
}

/// Current binding metadata for one effect leaf.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignEffectVariableBinding {
    pub field: DesignEffectVariableField,
    pub variable_id: SharedString,
    pub variable_name: SharedString,
    pub collection_name: SharedString,
    pub can_detach: bool,
}

impl DesignEffectVariableBinding {
    pub fn new(
        field: DesignEffectVariableField,
        variable_id: impl Into<SharedString>,
        variable_name: impl Into<SharedString>,
        collection_name: impl Into<SharedString>,
    ) -> Self {
        Self {
            field,
            variable_id: variable_id.into(),
            variable_name: variable_name.into(),
            collection_name: collection_name.into(),
            can_detach: true,
        }
    }

    pub const fn detachable(mut self, can_detach: bool) -> Self {
        self.can_detach = can_detach;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignShaderPropertyKind {
    Boolean,
    Text,
    Number,
    Image,
    InstanceSwap,
    Slot,
    Color,
    Point,
    Line,
    Circle,
    CirclePoint,
    ColorPoint,
    Gradient,
    Unsupported,
}

impl DesignShaderPropertyKind {
    pub const ALL: [Self; 14] = [
        Self::Boolean,
        Self::Text,
        Self::Number,
        Self::Image,
        Self::InstanceSwap,
        Self::Slot,
        Self::Color,
        Self::Point,
        Self::Line,
        Self::Circle,
        Self::CirclePoint,
        Self::ColorPoint,
        Self::Gradient,
        Self::Unsupported,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Boolean => "Boolean",
            Self::Text => "Text",
            Self::Number => "Number",
            Self::Image => "Image",
            Self::InstanceSwap => "Instance swap",
            Self::Slot => "Slot",
            Self::Color => "Color",
            Self::Point => "Point",
            Self::Line => "Line",
            Self::Circle => "Circle",
            Self::CirclePoint => "Circle point",
            Self::ColorPoint => "Color point",
            Self::Gradient => "Gradient",
            Self::Unsupported => "Unsupported",
        }
    }
}

/// Exact value leaf targeted by a host-owned Shader-property resource editor.
///
/// The effect and property-definition IDs on the surrounding intent remain
/// authoritative. A gradient-stop index is an ordering hint because Figma's
/// Shader API does not expose a stable stop ID.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignShaderPropertyEditorTarget {
    Value,
    ColorPointColor,
    GradientStopColor(usize),
}

/// Host surface requested for a Shader-property value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignShaderPropertyEditorKind {
    Resource,
    Variable,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignShaderGradientStop {
    pub position: f32,
    pub color: DesignColor,
    pub variable_id: Option<SharedString>,
}

impl DesignShaderGradientStop {
    pub const fn new(position: f32, color: DesignColor) -> Self {
        Self {
            position,
            color,
            variable_id: None,
        }
    }
}

/// A structured value for one shader property-definition ID.
///
/// The opaque case preserves newly introduced value shapes without requiring
/// this inspector version to understand or rewrite them.
#[derive(Clone, Debug, PartialEq)]
pub enum DesignShaderPropertyValue {
    Boolean(bool),
    Text(SharedString),
    Number(f32),
    AssetId(SharedString),
    Color(DesignColor),
    Point(DesignEffectVector),
    Line {
        start: DesignEffectVector,
        end: DesignEffectVector,
    },
    Circle {
        center: DesignEffectVector,
        radius: f32,
    },
    CirclePoint {
        center: DesignEffectVector,
        radius: f32,
        angle: f32,
    },
    ColorPoint {
        point: DesignEffectVector,
        color: DesignColor,
        variable_id: Option<SharedString>,
    },
    Gradient(Vec<DesignShaderGradientStop>),
    VariableAlias {
        variable_id: SharedString,
    },
    Opaque {
        type_name: SharedString,
        payload: SharedString,
    },
}

impl DesignShaderPropertyValue {
    pub const fn is_compatible_with(&self, kind: DesignShaderPropertyKind) -> bool {
        matches!(
            (self, kind),
            (Self::Boolean(_), DesignShaderPropertyKind::Boolean)
                | (Self::Text(_), DesignShaderPropertyKind::Text)
                | (Self::Number(_), DesignShaderPropertyKind::Number)
                | (
                    Self::AssetId(_),
                    DesignShaderPropertyKind::Image
                        | DesignShaderPropertyKind::InstanceSwap
                        | DesignShaderPropertyKind::Slot
                )
                | (Self::Color(_), DesignShaderPropertyKind::Color)
                | (Self::Point(_), DesignShaderPropertyKind::Point)
                | (Self::Line { .. }, DesignShaderPropertyKind::Line)
                | (Self::Circle { .. }, DesignShaderPropertyKind::Circle)
                | (
                    Self::CirclePoint { .. },
                    DesignShaderPropertyKind::CirclePoint
                )
                | (
                    Self::ColorPoint { .. },
                    DesignShaderPropertyKind::ColorPoint
                )
                | (Self::Gradient(_), DesignShaderPropertyKind::Gradient)
                | (Self::VariableAlias { .. }, _)
                | (Self::Opaque { .. }, DesignShaderPropertyKind::Unsupported)
        )
    }

    pub const fn variable_alias_id(&self) -> Option<&SharedString> {
        match self {
            Self::VariableAlias { variable_id } => Some(variable_id),
            _ => None,
        }
    }

    pub fn summary(&self) -> SharedString {
        match self {
            Self::Boolean(value) => value.to_string().into(),
            Self::Text(value) | Self::AssetId(value) => value.clone(),
            Self::Number(value) => value.to_string().into(),
            Self::Color(color) => format!("#{}", color.hex()).into(),
            Self::Point(point) => format!("{}, {}", point.x, point.y).into(),
            Self::Line { .. } => "Line".into(),
            Self::Circle { radius, .. } => format!("Circle · {radius}").into(),
            Self::CirclePoint { radius, angle, .. } => {
                format!("Circle · {radius} · {angle}°").into()
            }
            Self::ColorPoint { color, .. } => format!("Point · #{}", color.hex()).into(),
            Self::Gradient(stops) => format!("Gradient · {} stops", stops.len()).into(),
            Self::VariableAlias { variable_id } => format!("Variable · {variable_id}").into(),
            Self::Opaque { type_name, .. } => type_name.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignShaderProperty {
    pub definition_id: SharedString,
    pub name: SharedString,
    pub kind: DesignShaderPropertyKind,
    pub value: DesignShaderPropertyValue,
    pub description: Option<SharedString>,
    pub read_only: bool,
}

impl DesignShaderProperty {
    pub fn new(
        definition_id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        kind: DesignShaderPropertyKind,
        value: DesignShaderPropertyValue,
    ) -> Self {
        Self {
            definition_id: definition_id.into(),
            name: name.into(),
            kind,
            value,
            description: None,
            read_only: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignShaderEffect {
    pub shader_id: SharedString,
    pub name: SharedString,
    pub imported: bool,
    pub properties: Vec<DesignShaderProperty>,
}

impl DesignShaderEffect {
    pub fn new(
        shader_id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        properties: impl IntoIterator<Item = DesignShaderProperty>,
    ) -> Self {
        Self {
            shader_id: shader_id.into(),
            name: name.into(),
            imported: true,
            properties: properties.into_iter().collect(),
        }
    }
}

impl Default for DesignShaderEffect {
    fn default() -> Self {
        Self {
            shader_id: SharedString::default(),
            name: "Choose shader".into(),
            imported: false,
            properties: Vec::new(),
        }
    }
}

/// Lossless host payload for an effect type introduced after this inspector.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignOpaqueEffect {
    pub type_name: SharedString,
    pub summary: SharedString,
    pub payload: SharedString,
}

impl DesignOpaqueEffect {
    pub fn new(
        type_name: impl Into<SharedString>,
        summary: impl Into<SharedString>,
        payload: impl Into<SharedString>,
    ) -> Self {
        Self {
            type_name: type_name.into(),
            summary: summary.into(),
            payload: payload.into(),
        }
    }
}

impl Default for DesignOpaqueEffect {
    fn default() -> Self {
        Self::new("UNKNOWN", "Unsupported effect", "")
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignBlurType {
    Normal,
    Progressive,
}

impl DesignBlurType {
    pub const ALL: [Self; 2] = [Self::Normal, Self::Progressive];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Progressive => "Progressive",
        }
    }
}

/// Settings shared by layer blur and background blur.
///
/// Progressive blur positions are normalized to the effected node's bounds.
#[derive(Clone, Debug, PartialEq)]
pub enum DesignBlurEffect {
    Normal {
        radius: f32,
    },
    Progressive {
        start_radius: f32,
        end_radius: f32,
        start_offset: DesignEffectVector,
        end_offset: DesignEffectVector,
    },
}

impl DesignBlurEffect {
    pub const fn normal(radius: f32) -> Self {
        Self::Normal { radius }
    }

    pub const fn progressive(
        start_radius: f32,
        end_radius: f32,
        start_offset: DesignEffectVector,
        end_offset: DesignEffectVector,
    ) -> Self {
        Self::Progressive {
            start_radius,
            end_radius,
            start_offset,
            end_offset,
        }
    }

    pub const fn blur_type(&self) -> DesignBlurType {
        match self {
            Self::Normal { .. } => DesignBlurType::Normal,
            Self::Progressive { .. } => DesignBlurType::Progressive,
        }
    }

    pub const fn end_radius(&self) -> f32 {
        match self {
            Self::Normal { radius } => *radius,
            Self::Progressive { end_radius, .. } => *end_radius,
        }
    }

    pub fn set_end_radius(&mut self, radius: f32) {
        match self {
            Self::Normal {
                radius: current_radius,
            } => *current_radius = radius,
            Self::Progressive { end_radius, .. } => *end_radius = radius,
        }
    }

    /// Switches normal/progressive UI modes while preserving the end radius.
    pub fn set_blur_type(&mut self, blur_type: DesignBlurType) {
        if self.blur_type() == blur_type {
            return;
        }
        let end_radius = self.end_radius();
        *self = match blur_type {
            DesignBlurType::Normal => Self::normal(end_radius),
            DesignBlurType::Progressive => Self::progressive(
                0.,
                end_radius,
                DesignEffectVector::new(0.5, 0.),
                DesignEffectVector::new(0.5, 1.),
            ),
        };
    }
}

impl Default for DesignBlurEffect {
    fn default() -> Self {
        Self::normal(4.)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignNoiseType {
    Monotone,
    Duotone,
    Multitone,
}

impl DesignNoiseType {
    pub const ALL: [Self; 3] = [Self::Monotone, Self::Duotone, Self::Multitone];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Monotone => "Mono",
            Self::Duotone => "Duo",
            Self::Multitone => "Multi",
        }
    }
}

/// The color controls exposed for one of Figma's three noise modes.
#[derive(Clone, Debug, PartialEq)]
pub enum DesignNoiseColors {
    Monotone {
        color: DesignColor,
    },
    Duotone {
        color: DesignColor,
        secondary_color: DesignColor,
    },
    Multitone {
        opacity: f32,
    },
}

impl DesignNoiseColors {
    pub const fn noise_type(&self) -> DesignNoiseType {
        match self {
            Self::Monotone { .. } => DesignNoiseType::Monotone,
            Self::Duotone { .. } => DesignNoiseType::Duotone,
            Self::Multitone { .. } => DesignNoiseType::Multitone,
        }
    }

    pub const fn primary_color(&self) -> Option<DesignColor> {
        match self {
            Self::Monotone { color } | Self::Duotone { color, .. } => Some(*color),
            Self::Multitone { .. } => None,
        }
    }

    /// Switches the number-of-colors mode with representative values while
    /// preserving the existing primary color when one is available.
    pub fn set_noise_type(&mut self, noise_type: DesignNoiseType) {
        if self.noise_type() == noise_type {
            return;
        }
        let primary = self.primary_color().unwrap_or(DesignColor::BLACK);
        *self = match noise_type {
            DesignNoiseType::Monotone => Self::Monotone { color: primary },
            DesignNoiseType::Duotone => Self::Duotone {
                color: primary,
                secondary_color: DesignColor::WHITE,
            },
            DesignNoiseType::Multitone => Self::Multitone { opacity: 1. },
        };
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignDropShadowEffect {
    pub color: DesignColor,
    pub offset: DesignEffectVector,
    pub radius: f32,
    pub spread: f32,
    pub blend_mode: DesignBlendMode,
    pub show_behind_node: bool,
}

impl Default for DesignDropShadowEffect {
    fn default() -> Self {
        Self {
            color: DesignColor::rgba(0, 0, 0, 0x40),
            offset: DesignEffectVector::new(0., 4.),
            radius: 4.,
            spread: 0.,
            blend_mode: DesignBlendMode::Normal,
            show_behind_node: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignInnerShadowEffect {
    pub color: DesignColor,
    pub offset: DesignEffectVector,
    pub radius: f32,
    pub spread: f32,
    pub blend_mode: DesignBlendMode,
}

impl Default for DesignInnerShadowEffect {
    fn default() -> Self {
        Self {
            color: DesignColor::rgba(0, 0, 0, 0x40),
            offset: DesignEffectVector::new(0., 2.),
            radius: 4.,
            spread: 0.,
            blend_mode: DesignBlendMode::Normal,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignNoiseEffect {
    pub colors: DesignNoiseColors,
    pub size: DesignEffectVector,
    pub density: f32,
    pub blend_mode: DesignBlendMode,
}

impl Default for DesignNoiseEffect {
    fn default() -> Self {
        Self {
            colors: DesignNoiseColors::Monotone {
                color: DesignColor::BLACK,
            },
            size: DesignEffectVector::splat(1.),
            density: 0.5,
            blend_mode: DesignBlendMode::Normal,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignTextureEffect {
    pub size: DesignEffectVector,
    pub radius: f32,
    pub clip_to_shape: bool,
}

impl Default for DesignTextureEffect {
    fn default() -> Self {
        Self {
            size: DesignEffectVector::splat(1.),
            radius: 1.,
            clip_to_shape: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignGlassEffect {
    /// Normalized intensity in the inclusive 0–1 range.
    pub light_intensity: f32,
    /// Light direction in degrees.
    pub light_angle: f32,
    /// Normalized refraction amount in the inclusive 0–1 range.
    pub refraction: f32,
    /// Refraction depth. Figma requires this value to be at least one.
    pub depth: f32,
    /// Normalized chromatic dispersion in the inclusive 0–1 range.
    pub dispersion: f32,
    /// Frost radius.
    pub frost: f32,
    /// Highlight spread exposed by the current Design panel.
    pub splay: f32,
}

impl Default for DesignGlassEffect {
    fn default() -> Self {
        Self {
            light_intensity: 0.5,
            light_angle: -45.,
            refraction: 0.5,
            depth: 10.,
            dispersion: 0.,
            frost: 0.,
            splay: 0.5,
        }
    }
}

/// Effect-specific settings stored by a host-controlled inspector node.
///
/// The discriminant deliberately carries the effect kind so fields that Figma
/// does not expose for a particular kind cannot accidentally leak into its UI.
#[derive(Clone, Debug, PartialEq)]
pub enum DesignEffectSettings {
    DropShadow(DesignDropShadowEffect),
    InnerShadow(DesignInnerShadowEffect),
    LayerBlur(DesignBlurEffect),
    BackgroundBlur(DesignBlurEffect),
    Noise(DesignNoiseEffect),
    Texture(DesignTextureEffect),
    Glass(DesignGlassEffect),
    Shader(DesignShaderEffect),
    Opaque(DesignOpaqueEffect),
}

impl DesignEffectSettings {
    pub const fn kind(&self) -> DesignEffectKind {
        match self {
            Self::DropShadow(_) => DesignEffectKind::DropShadow,
            Self::InnerShadow(_) => DesignEffectKind::InnerShadow,
            Self::LayerBlur(_) => DesignEffectKind::LayerBlur,
            Self::BackgroundBlur(_) => DesignEffectKind::BackgroundBlur,
            Self::Noise(_) => DesignEffectKind::Noise,
            Self::Texture(_) => DesignEffectKind::Texture,
            Self::Glass(_) => DesignEffectKind::Glass,
            Self::Shader(_) => DesignEffectKind::Shader,
            Self::Opaque(_) => DesignEffectKind::Unsupported,
        }
    }

    pub fn default_for_kind(kind: DesignEffectKind) -> Self {
        match kind {
            DesignEffectKind::DropShadow => Self::DropShadow(DesignDropShadowEffect::default()),
            DesignEffectKind::InnerShadow => Self::InnerShadow(DesignInnerShadowEffect::default()),
            DesignEffectKind::LayerBlur => Self::LayerBlur(DesignBlurEffect::default()),
            DesignEffectKind::BackgroundBlur => Self::BackgroundBlur(DesignBlurEffect::default()),
            DesignEffectKind::Noise => Self::Noise(DesignNoiseEffect::default()),
            DesignEffectKind::Texture => Self::Texture(DesignTextureEffect::default()),
            DesignEffectKind::Glass => Self::Glass(DesignGlassEffect::default()),
            DesignEffectKind::Shader => Self::Shader(DesignShaderEffect::default()),
            DesignEffectKind::Unsupported => Self::Opaque(DesignOpaqueEffect::default()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignEffect {
    /// Stable host-owned identity. Empty is a compatibility fallback to index.
    pub id: SharedString,
    /// Compatibility summary used by the original compact effects row.
    pub kind: DesignEffectKind,
    pub visible: bool,
    pub color: DesignColor,
    pub blur: f32,
    pub spread: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    /// Authoritative effect-specific settings for the expanded settings UI.
    pub settings: DesignEffectSettings,
    /// Exact variable aliases currently bound to bindable fields on this
    /// effect. The host echoes this list after apply/detach intents.
    pub variable_bindings: Vec<DesignEffectVariableBinding>,
}

impl DesignEffect {
    pub fn new(kind: DesignEffectKind) -> Self {
        Self::from_settings(true, DesignEffectSettings::default_for_kind(kind))
    }

    pub fn drop_shadow(
        color: DesignColor,
        blur: f32,
        spread: f32,
        offset_x: f32,
        offset_y: f32,
    ) -> Self {
        Self::from_settings(
            true,
            DesignEffectSettings::DropShadow(DesignDropShadowEffect {
                color,
                offset: DesignEffectVector::new(offset_x, offset_y),
                radius: blur,
                spread,
                ..DesignDropShadowEffect::default()
            }),
        )
    }

    pub fn from_settings(visible: bool, settings: DesignEffectSettings) -> Self {
        let mut effect = Self {
            id: SharedString::default(),
            kind: settings.kind(),
            visible,
            color: DesignColor::BLACK,
            blur: 0.,
            spread: 0.,
            offset_x: 0.,
            offset_y: 0.,
            settings,
            variable_bindings: Vec::new(),
        };
        effect.sync_compatibility_summary();
        effect
    }

    pub fn with_id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_variable_binding(mut self, binding: DesignEffectVariableBinding) -> Self {
        self.set_variable_binding(binding);
        self
    }

    pub fn variable_binding(
        &self,
        field: DesignEffectVariableField,
    ) -> Option<&DesignEffectVariableBinding> {
        self.variable_bindings
            .iter()
            .find(|binding| binding.field == field)
    }

    pub fn set_variable_binding(&mut self, binding: DesignEffectVariableBinding) {
        if let Some(current) = self
            .variable_bindings
            .iter_mut()
            .find(|current| current.field == binding.field)
        {
            *current = binding;
        } else {
            self.variable_bindings.push(binding);
        }
    }

    pub fn remove_variable_binding(
        &mut self,
        field: DesignEffectVariableField,
    ) -> Option<DesignEffectVariableBinding> {
        self.variable_bindings
            .iter()
            .position(|binding| binding.field == field)
            .map(|index| self.variable_bindings.remove(index))
    }

    pub fn variable_fields(&self, spread_supported: bool) -> &'static [DesignEffectVariableField] {
        const SHADOW: &[DesignEffectVariableField] = &[
            DesignEffectVariableField::Radius,
            DesignEffectVariableField::Color,
            DesignEffectVariableField::Spread,
            DesignEffectVariableField::OffsetX,
            DesignEffectVariableField::OffsetY,
        ];
        const SHADOW_WITHOUT_SPREAD: &[DesignEffectVariableField] = &[
            DesignEffectVariableField::Radius,
            DesignEffectVariableField::Color,
            DesignEffectVariableField::OffsetX,
            DesignEffectVariableField::OffsetY,
        ];
        const BLUR: &[DesignEffectVariableField] = &[DesignEffectVariableField::Radius];
        const NONE: &[DesignEffectVariableField] = &[];
        match &self.settings {
            DesignEffectSettings::DropShadow(_) | DesignEffectSettings::InnerShadow(_) => {
                if spread_supported {
                    SHADOW
                } else {
                    SHADOW_WITHOUT_SPREAD
                }
            }
            DesignEffectSettings::LayerBlur(_) | DesignEffectSettings::BackgroundBlur(_) => BLUR,
            DesignEffectSettings::Noise(_)
            | DesignEffectSettings::Texture(_)
            | DesignEffectSettings::Glass(_)
            | DesignEffectSettings::Shader(_)
            | DesignEffectSettings::Opaque(_) => NONE,
        }
    }

    /// Replaces an effect kind with its representative settings while
    /// preserving the collection row's visibility.
    pub fn set_kind(&mut self, kind: DesignEffectKind) {
        self.settings = DesignEffectSettings::default_for_kind(kind);
        self.variable_bindings.clear();
        self.sync_compatibility_summary();
    }

    pub fn set_settings(&mut self, settings: DesignEffectSettings) {
        if self.settings.kind() != settings.kind() {
            self.variable_bindings.clear();
        }
        self.settings = settings;
        self.sync_compatibility_summary();
    }

    /// Updates the legacy compact blur field and its corresponding typed leaf.
    pub fn set_blur(&mut self, blur: f32) {
        self.blur = blur;
        match &mut self.settings {
            DesignEffectSettings::DropShadow(settings) => settings.radius = blur,
            DesignEffectSettings::InnerShadow(settings) => settings.radius = blur,
            DesignEffectSettings::LayerBlur(settings)
            | DesignEffectSettings::BackgroundBlur(settings) => settings.set_end_radius(blur),
            DesignEffectSettings::Texture(settings) => settings.radius = blur,
            DesignEffectSettings::Glass(settings) => settings.frost = blur,
            DesignEffectSettings::Noise(_)
            | DesignEffectSettings::Shader(_)
            | DesignEffectSettings::Opaque(_) => {}
        }
    }

    pub fn set_spread(&mut self, spread: f32) {
        self.spread = spread;
        match &mut self.settings {
            DesignEffectSettings::DropShadow(settings) => settings.spread = spread,
            DesignEffectSettings::InnerShadow(settings) => settings.spread = spread,
            _ => {}
        }
    }

    pub fn set_offset_x(&mut self, offset_x: f32) {
        self.offset_x = offset_x;
        match &mut self.settings {
            DesignEffectSettings::DropShadow(settings) => settings.offset.x = offset_x,
            DesignEffectSettings::InnerShadow(settings) => settings.offset.x = offset_x,
            _ => {}
        }
    }

    pub fn set_offset_y(&mut self, offset_y: f32) {
        self.offset_y = offset_y;
        match &mut self.settings {
            DesignEffectSettings::DropShadow(settings) => settings.offset.y = offset_y,
            DesignEffectSettings::InnerShadow(settings) => settings.offset.y = offset_y,
            _ => {}
        }
    }

    /// Rebuilds compatibility fields after a host changes typed settings.
    pub fn sync_compatibility_summary(&mut self) {
        self.kind = self.settings.kind();
        let (color, blur, spread, offset) = match &self.settings {
            DesignEffectSettings::DropShadow(settings) => (
                settings.color,
                settings.radius,
                settings.spread,
                settings.offset,
            ),
            DesignEffectSettings::InnerShadow(settings) => (
                settings.color,
                settings.radius,
                settings.spread,
                settings.offset,
            ),
            DesignEffectSettings::LayerBlur(settings)
            | DesignEffectSettings::BackgroundBlur(settings) => (
                DesignColor::BLACK,
                settings.end_radius(),
                0.,
                DesignEffectVector::default(),
            ),
            DesignEffectSettings::Noise(settings) => {
                let color = match &settings.colors {
                    DesignNoiseColors::Monotone { color }
                    | DesignNoiseColors::Duotone { color, .. } => *color,
                    DesignNoiseColors::Multitone { .. } => DesignColor::BLACK,
                };
                (color, 0., 0., DesignEffectVector::default())
            }
            DesignEffectSettings::Texture(settings) => (
                DesignColor::BLACK,
                settings.radius,
                0.,
                DesignEffectVector::default(),
            ),
            DesignEffectSettings::Glass(settings) => (
                DesignColor::WHITE,
                settings.frost,
                0.,
                DesignEffectVector::default(),
            ),
            DesignEffectSettings::Shader(_) | DesignEffectSettings::Opaque(_) => {
                (DesignColor::BLACK, 0., 0., DesignEffectVector::default())
            }
        };
        self.color = color;
        self.blur = blur;
        self.spread = spread;
        self.offset_x = offset.x;
        self.offset_y = offset.y;
    }
}

/// One host-supplied page or library Effect style.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignEffectStyle {
    pub id: SharedString,
    pub name: SharedString,
    /// Ordered preview summary. Applying remains host-owned and does not copy
    /// these values into the panel's controlled node.
    pub effect_kinds: Vec<DesignEffectKind>,
}

impl DesignEffectStyle {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        effect_kinds: impl IntoIterator<Item = DesignEffectKind>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            effect_kinds: effect_kinds.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignEffectStyleLibrary {
    pub id: SharedString,
    pub name: SharedString,
    pub styles: Vec<DesignEffectStyle>,
}

impl DesignEffectStyleLibrary {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        styles: impl IntoIterator<Item = DesignEffectStyle>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            styles: styles.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DesignEffectStyleSource {
    Page,
    Library { library_id: SharedString },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DesignEffectStyleSelection {
    pub source: DesignEffectStyleSource,
    pub style_id: SharedString,
}

impl DesignEffectStyleSelection {
    pub fn page(style_id: impl Into<SharedString>) -> Self {
        Self {
            source: DesignEffectStyleSource::Page,
            style_id: style_id.into(),
        }
    }

    pub fn library(library_id: impl Into<SharedString>, style_id: impl Into<SharedString>) -> Self {
        Self {
            source: DesignEffectStyleSource::Library {
                library_id: library_id.into(),
            },
            style_id: style_id.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesignEffectStyleViewData {
    pub page_styles: Vec<DesignEffectStyle>,
    pub libraries: Vec<DesignEffectStyleLibrary>,
}

impl DesignEffectStyleViewData {
    pub fn new(
        page_styles: impl IntoIterator<Item = DesignEffectStyle>,
        libraries: impl IntoIterator<Item = DesignEffectStyleLibrary>,
    ) -> Self {
        Self {
            page_styles: page_styles.into_iter().collect(),
            libraries: libraries.into_iter().collect(),
        }
    }

    pub fn style(&self, selection: &DesignEffectStyleSelection) -> Option<&DesignEffectStyle> {
        match &selection.source {
            DesignEffectStyleSource::Page => self
                .page_styles
                .iter()
                .find(|style| style.id == selection.style_id),
            DesignEffectStyleSource::Library { library_id } => self
                .libraries
                .iter()
                .find(|library| library.id == *library_id)?
                .styles
                .iter()
                .find(|style| style.id == selection.style_id),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignEffectStyleBinding {
    pub selection: DesignEffectStyleSelection,
    pub name: SharedString,
    pub can_detach: bool,
}

impl DesignEffectStyleBinding {
    pub fn new(selection: DesignEffectStyleSelection, name: impl Into<SharedString>) -> Self {
        Self {
            selection,
            name: name.into(),
            can_detach: true,
        }
    }

    pub const fn detachable(mut self, can_detach: bool) -> Self {
        self.can_detach = can_detach;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignGridKind {
    Uniform,
    Columns,
    Rows,
}

impl DesignGridKind {
    pub const ALL: [Self; 3] = [Self::Uniform, Self::Columns, Self::Rows];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Uniform => "Grid",
            Self::Columns => "Columns",
            Self::Rows => "Rows",
        }
    }
}

/// Count used by row and column layout guides.
///
/// `Auto` asks the host to derive as many tracks as fit the selected frame.
/// A concrete count is always at least one; constructors normalize zero.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignLayoutGridCount {
    Auto,
    Number(u16),
}

impl DesignLayoutGridCount {
    pub const fn number(count: u16) -> Self {
        Self::Number(if count == 0 { 1 } else { count })
    }

    pub const fn numeric(self) -> Option<u16> {
        match self {
            Self::Auto => None,
            Self::Number(count) => Some(count),
        }
    }

    pub fn label(self) -> SharedString {
        match self {
            Self::Auto => "Auto".into(),
            Self::Number(count) => count.to_string().into(),
        }
    }

    pub const fn is_auto(self) -> bool {
        matches!(self, Self::Auto)
    }
}

/// Exact Figma field on one layout guide that can retain a Number variable.
///
/// `offset` is shared by the fixed-guide Offset control and the stretch-guide
/// Margin control. [`DesignLayoutGridVariableTarget::property`] preserves
/// which inspector leaf was visible when an intent was emitted.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignLayoutGridVariableField {
    SectionSize,
    Count,
    Offset,
    GutterSize,
}

impl DesignLayoutGridVariableField {
    pub const fn api_name(self) -> &'static str {
        match self {
            Self::SectionSize => "sectionSize",
            Self::Count => "count",
            Self::Offset => "offset",
            Self::GutterSize => "gutterSize",
        }
    }
}

/// Stable, lossless target for one variable-bindable layout-guide leaf.
///
/// A non-empty `guide_id` is authoritative. `index` and the index embedded in
/// `property` are compatibility hints that hosts resolve again against their
/// latest ordered guide snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignLayoutGridVariableTarget {
    pub guide_id: SharedString,
    pub index: usize,
    pub property: DesignPanelProperty,
    pub field: DesignLayoutGridVariableField,
}

impl DesignLayoutGridVariableTarget {
    pub fn new(
        guide_id: impl Into<SharedString>,
        index: usize,
        property: DesignPanelProperty,
        field: DesignLayoutGridVariableField,
    ) -> Self {
        Self {
            guide_id: guide_id.into(),
            index,
            property: property.with_layout_grid_index(index),
            field,
        }
    }

    pub const fn with_index(mut self, index: usize) -> Self {
        self.index = index;
        self.property = self.property.with_layout_grid_index(index);
        self
    }
}

/// Typed resolved value used to validate and create layout-guide variables.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DesignLayoutGridVariableValue {
    Number(f32),
    Count(DesignLayoutGridCount),
}

impl DesignLayoutGridVariableValue {
    pub fn label(self) -> SharedString {
        match self {
            Self::Number(value) => {
                let rounded = value.round();
                if (value - rounded).abs() <= f32::EPSILON {
                    format!("{rounded:.0}").into()
                } else {
                    format!("{value:.2}").into()
                }
            }
            Self::Count(value) => value.label(),
        }
    }

    /// Returns a host- or type-authored reason that this Number variable
    /// cannot be applied to the exact layout-guide leaf.
    pub fn compatibility_reason(self, variable: &DesignVariable) -> Option<SharedString> {
        if let Some(reason) = &variable.disabled_reason {
            return Some(reason.clone());
        }
        if variable.resolved_type != DesignVariableResolvedType::Float {
            return Some("Layout guides require a Number variable".into());
        }
        let Some(DesignVariableResolvedValue::Float(resolved)) = variable.resolved_value else {
            return Some("A resolved Number value is required for this guide".into());
        };
        match self {
            Self::Number(_) if resolved.is_finite() && resolved >= 0. => None,
            Self::Number(_) => Some("This guide requires a finite non-negative number".into()),
            Self::Count(DesignLayoutGridCount::Auto)
                if resolved.is_infinite() && resolved.is_sign_positive() =>
            {
                None
            }
            Self::Count(DesignLayoutGridCount::Auto) => {
                Some("This guide requires an Auto count variable".into())
            }
            Self::Count(DesignLayoutGridCount::Number(_))
                if resolved.is_finite()
                    && resolved >= 1.
                    && resolved <= u16::MAX as f32
                    && resolved.fract().abs() <= f32::EPSILON =>
            {
                None
            }
            Self::Count(DesignLayoutGridCount::Number(_)) => {
                Some("This guide requires a positive whole-number count variable".into())
            }
        }
    }

    pub fn is_compatible(self, variable: &DesignVariable) -> bool {
        self.compatibility_reason(variable).is_none()
    }
}

/// Whether the host exposes variable creation for the active guide target.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum DesignLayoutGridVariableCreateState {
    #[default]
    Hidden,
    Enabled,
    Disabled {
        reason: SharedString,
    },
}

impl DesignLayoutGridVariableCreateState {
    pub const fn is_visible(&self) -> bool {
        !matches!(self, Self::Hidden)
    }

    pub const fn is_enabled(&self) -> bool {
        matches!(self, Self::Enabled)
    }

    pub const fn disabled_reason(&self) -> Option<&SharedString> {
        match self {
            Self::Disabled { reason } => Some(reason),
            Self::Hidden | Self::Enabled => None,
        }
    }
}

/// Host-controlled Number-variable catalog shared by every bindable layout
/// guide leaf. Source/import state, collection identity, search metadata, and
/// disabled reasons remain the lossless [`DesignVariable`] contract.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesignLayoutGridVariableViewData {
    pub variables: Vec<DesignVariable>,
    pub create_state: DesignLayoutGridVariableCreateState,
}

impl DesignLayoutGridVariableViewData {
    pub fn new(variables: impl IntoIterator<Item = DesignVariable>) -> Self {
        Self {
            variables: variables
                .into_iter()
                .filter(|variable| {
                    variable.resolved_type == DesignVariableResolvedType::Float
                        && matches!(
                            variable.resolved_value.as_ref(),
                            None | Some(DesignVariableResolvedValue::Float(_))
                        )
                })
                .collect(),
            create_state: DesignLayoutGridVariableCreateState::Hidden,
        }
    }

    pub fn with_create_state(mut self, create_state: DesignLayoutGridVariableCreateState) -> Self {
        self.create_state = create_state;
        self
    }

    pub fn variable(&self, variable_id: &str) -> Option<&DesignVariable> {
        self.variables.iter().find(|variable| {
            variable.id.as_ref() == variable_id
                && variable.resolved_type == DesignVariableResolvedType::Float
                && matches!(
                    variable.resolved_value.as_ref(),
                    None | Some(DesignVariableResolvedValue::Float(_))
                )
        })
    }

    pub fn variable_mut(&mut self, variable_id: &str) -> Option<&mut DesignVariable> {
        self.variables.iter_mut().find(|variable| {
            variable.id.as_ref() == variable_id
                && variable.resolved_type == DesignVariableResolvedType::Float
                && matches!(
                    variable.resolved_value.as_ref(),
                    None | Some(DesignVariableResolvedValue::Float(_))
                )
        })
    }
}

/// Compatibility-only variable candidate for a row/column guide count.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignLayoutGridCountVariable {
    pub id: SharedString,
    pub name: SharedString,
    pub collection_name: SharedString,
    pub resolved_count: DesignLayoutGridCount,
    /// Host-authored reason that prevents applying this variable.
    pub disabled_reason: Option<SharedString>,
}

impl DesignLayoutGridCountVariable {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        collection_name: impl Into<SharedString>,
        resolved_count: DesignLayoutGridCount,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            collection_name: collection_name.into(),
            resolved_count,
            disabled_reason: None,
        }
    }

    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.disabled_reason = Some(reason.into());
        self
    }

    /// Count variables preserve Figma's Number/Auto discriminant exactly.
    /// A numeric count cannot silently accept an Auto variable or vice versa.
    pub fn compatibility_reason(
        &self,
        current_count: DesignLayoutGridCount,
    ) -> Option<SharedString> {
        if let Some(reason) = &self.disabled_reason {
            return Some(reason.clone());
        }
        if self.resolved_count.is_auto() == current_count.is_auto() {
            None
        } else if current_count.is_auto() {
            Some("This guide requires an Auto count variable".into())
        } else {
            Some("This guide requires a numeric count variable".into())
        }
    }

    pub fn is_compatible(&self, current_count: DesignLayoutGridCount) -> bool {
        self.compatibility_reason(current_count).is_none()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesignLayoutGridCountVariableViewData {
    pub variables: Vec<DesignLayoutGridCountVariable>,
}

impl DesignLayoutGridCountVariableViewData {
    pub fn new(variables: impl IntoIterator<Item = DesignLayoutGridCountVariable>) -> Self {
        Self {
            variables: variables.into_iter().collect(),
        }
    }

    pub fn variable(&self, variable_id: &str) -> Option<&DesignLayoutGridCountVariable> {
        self.variables
            .iter()
            .find(|variable| variable.id.as_ref() == variable_id)
    }

    pub fn generalized(&self) -> DesignLayoutGridVariableViewData {
        DesignLayoutGridVariableViewData::new(self.variables.iter().cloned().map(|variable| {
            let resolved_value = match variable.resolved_count {
                DesignLayoutGridCount::Auto => f32::INFINITY,
                DesignLayoutGridCount::Number(count) => f32::from(count),
            };
            let mut value = DesignVariable::page(
                variable.id,
                variable.name,
                variable.collection_name.clone(),
                variable.collection_name,
                DesignVariableResolvedType::Float,
            )
            .with_resolved_value(DesignVariableResolvedValue::Float(resolved_value));
            value.disabled_reason = variable.disabled_reason;
            value
        }))
    }
}

/// Opaque variable identity for one Number-valued layout-guide field.
///
/// The panel displays the host-resolved raw value but prevents a direct edit
/// until the host accepts a detach intent. `read_only_reason` and
/// `can_detach` let a host distinguish an inspectable binding from one the
/// current user may replace or detach.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignLayoutGridVariableBinding {
    pub variable_id: SharedString,
    pub variable_name: SharedString,
    pub collection_name: Option<SharedString>,
    pub can_detach: bool,
    pub read_only_reason: Option<SharedString>,
}

impl DesignLayoutGridVariableBinding {
    pub fn new(
        variable_id: impl Into<SharedString>,
        variable_name: impl Into<SharedString>,
    ) -> Self {
        Self {
            variable_id: variable_id.into(),
            variable_name: variable_name.into(),
            collection_name: None,
            can_detach: true,
            read_only_reason: None,
        }
    }

    pub fn with_collection(mut self, collection_name: impl Into<SharedString>) -> Self {
        self.collection_name = Some(collection_name.into());
        self
    }

    pub const fn detachable(mut self, can_detach: bool) -> Self {
        self.can_detach = can_detach;
        self
    }

    pub fn read_only(mut self, reason: impl Into<SharedString>) -> Self {
        self.can_detach = false;
        self.read_only_reason = Some(reason.into());
        self
    }
}

/// One exact Figma field binding on a host-owned layout guide.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignLayoutGridBoundVariable {
    pub field: DesignLayoutGridVariableField,
    pub binding: DesignLayoutGridVariableBinding,
}

impl DesignLayoutGridBoundVariable {
    pub const fn new(
        field: DesignLayoutGridVariableField,
        binding: DesignLayoutGridVariableBinding,
    ) -> Self {
        Self { field, binding }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DesignLayoutGridStyleSource {
    Page,
    Library { library_id: SharedString },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DesignLayoutGridStyleSelection {
    pub source: DesignLayoutGridStyleSource,
    pub style_id: SharedString,
}

impl DesignLayoutGridStyleSelection {
    pub fn page(style_id: impl Into<SharedString>) -> Self {
        Self {
            source: DesignLayoutGridStyleSource::Page,
            style_id: style_id.into(),
        }
    }

    pub fn library(library_id: impl Into<SharedString>, style_id: impl Into<SharedString>) -> Self {
        Self {
            source: DesignLayoutGridStyleSource::Library {
                library_id: library_id.into(),
            },
            style_id: style_id.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DesignLayoutGridStyleImportState {
    #[default]
    Imported,
    Available,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignLayoutGridStyle {
    pub id: SharedString,
    pub name: SharedString,
    /// Complete ordered guide snapshot stored by Figma's `GridStyle`.
    pub layout_grids: Vec<DesignLayoutGrid>,
    pub import_state: DesignLayoutGridStyleImportState,
}

impl DesignLayoutGridStyle {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        layout_grids: impl IntoIterator<Item = DesignLayoutGrid>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            layout_grids: layout_grids.into_iter().collect(),
            import_state: DesignLayoutGridStyleImportState::Imported,
        }
    }

    pub const fn available(mut self) -> Self {
        self.import_state = DesignLayoutGridStyleImportState::Available;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignLayoutGridStyleLibrary {
    pub id: SharedString,
    pub name: SharedString,
    pub styles: Vec<DesignLayoutGridStyle>,
}

impl DesignLayoutGridStyleLibrary {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        styles: impl IntoIterator<Item = DesignLayoutGridStyle>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            styles: styles.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesignLayoutGridStyleViewData {
    pub page_styles: Vec<DesignLayoutGridStyle>,
    pub libraries: Vec<DesignLayoutGridStyleLibrary>,
}

impl DesignLayoutGridStyleViewData {
    pub fn new(
        page_styles: impl IntoIterator<Item = DesignLayoutGridStyle>,
        libraries: impl IntoIterator<Item = DesignLayoutGridStyleLibrary>,
    ) -> Self {
        Self {
            page_styles: page_styles.into_iter().collect(),
            libraries: libraries.into_iter().collect(),
        }
    }

    pub fn style(
        &self,
        selection: &DesignLayoutGridStyleSelection,
    ) -> Option<&DesignLayoutGridStyle> {
        match &selection.source {
            DesignLayoutGridStyleSource::Page => self
                .page_styles
                .iter()
                .find(|style| style.id == selection.style_id),
            DesignLayoutGridStyleSource::Library { library_id } => self
                .libraries
                .iter()
                .find(|library| library.id == *library_id)?
                .styles
                .iter()
                .find(|style| style.id == selection.style_id),
        }
    }
}

/// Opaque node-level layout-guide style identity.
///
/// Figma exposes one `gridStyleId` for the node's complete ordered
/// `layoutGrids` array. Style application remains host-owned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignLayoutGridStyleBinding {
    pub style_id: SharedString,
    pub style_name: SharedString,
    pub source: DesignLayoutGridStyleSource,
    pub can_detach: bool,
}

impl DesignLayoutGridStyleBinding {
    pub fn new(style_id: impl Into<SharedString>, style_name: impl Into<SharedString>) -> Self {
        Self {
            style_id: style_id.into(),
            style_name: style_name.into(),
            source: DesignLayoutGridStyleSource::Page,
            can_detach: true,
        }
    }

    pub fn library(
        library_id: impl Into<SharedString>,
        style_id: impl Into<SharedString>,
        style_name: impl Into<SharedString>,
    ) -> Self {
        Self {
            style_id: style_id.into(),
            style_name: style_name.into(),
            source: DesignLayoutGridStyleSource::Library {
                library_id: library_id.into(),
            },
            can_detach: true,
        }
    }

    pub fn selection(&self) -> DesignLayoutGridStyleSelection {
        DesignLayoutGridStyleSelection {
            source: self.source.clone(),
            style_id: self.style_id.clone(),
        }
    }

    pub const fn with_detach_allowed(mut self, can_detach: bool) -> Self {
        self.can_detach = can_detach;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignColumnGridAlignment {
    Left,
    Center,
    Right,
    Stretch,
}

impl DesignColumnGridAlignment {
    pub const ALL: [Self; 4] = [Self::Left, Self::Center, Self::Right, Self::Stretch];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Center => "Center",
            Self::Right => "Right",
            Self::Stretch => "Stretch",
        }
    }

    pub const fn is_stretch(self) -> bool {
        matches!(self, Self::Stretch)
    }

    /// Centered fixed guides do not expose an edge offset in Figma.
    pub const fn supports_offset(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignRowGridAlignment {
    Top,
    Center,
    Bottom,
    Stretch,
}

impl DesignRowGridAlignment {
    pub const ALL: [Self; 4] = [Self::Top, Self::Center, Self::Bottom, Self::Stretch];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Top => "Top",
            Self::Center => "Center",
            Self::Bottom => "Bottom",
            Self::Stretch => "Stretch",
        }
    }

    pub const fn is_stretch(self) -> bool {
        matches!(self, Self::Stretch)
    }

    /// Centered fixed guides do not expose an edge offset in Figma.
    pub const fn supports_offset(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
}

/// Uniform square-grid guide settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesignUniformLayoutGrid {
    pub size: f32,
}

impl Default for DesignUniformLayoutGrid {
    fn default() -> Self {
        Self { size: 10. }
    }
}

/// Column guide settings.
///
/// `size` and `offset` apply to Left/Center/Right fixed guides. Stretch
/// columns have an implicit Auto size and instead use `margin`; all alignment
/// modes use `gutter`.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignColumnLayoutGrid {
    pub alignment: DesignColumnGridAlignment,
    pub count: DesignLayoutGridCount,
    pub count_binding: Option<DesignLayoutGridVariableBinding>,
    pub size: f32,
    pub offset: f32,
    pub gutter: f32,
    pub margin: f32,
}

impl Default for DesignColumnLayoutGrid {
    fn default() -> Self {
        Self {
            alignment: DesignColumnGridAlignment::Stretch,
            count: DesignLayoutGridCount::number(12),
            count_binding: None,
            size: 72.,
            offset: 0.,
            gutter: 24.,
            margin: 32.,
        }
    }
}

/// Row guide settings.
///
/// `size` and `offset` apply to Top/Center/Bottom fixed guides. Stretch rows
/// have an implicit Auto size and instead use `margin`; all alignment modes
/// use `gutter`.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignRowLayoutGrid {
    pub alignment: DesignRowGridAlignment,
    pub count: DesignLayoutGridCount,
    pub count_binding: Option<DesignLayoutGridVariableBinding>,
    pub size: f32,
    pub offset: f32,
    pub gutter: f32,
    pub margin: f32,
}

impl Default for DesignRowLayoutGrid {
    fn default() -> Self {
        Self {
            alignment: DesignRowGridAlignment::Stretch,
            count: DesignLayoutGridCount::number(8),
            count_binding: None,
            size: 8.,
            offset: 0.,
            gutter: 8.,
            margin: 24.,
        }
    }
}

/// Settings discriminated by Figma layout-guide type.
#[derive(Clone, Debug, PartialEq)]
pub enum DesignLayoutGridSettings {
    Uniform(DesignUniformLayoutGrid),
    Columns(DesignColumnLayoutGrid),
    Rows(DesignRowLayoutGrid),
}

impl DesignLayoutGridSettings {
    pub const fn kind(&self) -> DesignGridKind {
        match self {
            Self::Uniform(_) => DesignGridKind::Uniform,
            Self::Columns(_) => DesignGridKind::Columns,
            Self::Rows(_) => DesignGridKind::Rows,
        }
    }
}

impl From<DesignGridKind> for DesignLayoutGridSettings {
    fn from(kind: DesignGridKind) -> Self {
        match kind {
            DesignGridKind::Uniform => Self::Uniform(DesignUniformLayoutGrid::default()),
            DesignGridKind::Columns => Self::Columns(DesignColumnLayoutGrid::default()),
            DesignGridKind::Rows => Self::Rows(DesignRowLayoutGrid::default()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignLayoutGrid {
    /// Stable opaque host identity. Empty retains the legacy index fallback.
    pub id: SharedString,
    pub visible: bool,
    pub settings: DesignLayoutGridSettings,
    pub color: DesignColor,
    /// Guide opacity as a percentage in the inclusive 0–100 range.
    pub opacity: f32,
    /// Exact Number-variable bindings keyed by Figma layout-grid field.
    ///
    /// The legacy row/column `count_binding` remains a compatibility fallback
    /// when no canonical Count entry is present here.
    pub variable_bindings: Vec<DesignLayoutGridBoundVariable>,
}

impl DesignLayoutGrid {
    pub fn uniform(size: f32, color: DesignColor) -> Self {
        Self {
            id: "".into(),
            visible: true,
            settings: DesignLayoutGridSettings::Uniform(DesignUniformLayoutGrid { size }),
            color,
            opacity: 10.,
            variable_bindings: Vec::new(),
        }
    }

    pub fn columns(settings: DesignColumnLayoutGrid, color: DesignColor) -> Self {
        Self {
            id: "".into(),
            visible: true,
            settings: DesignLayoutGridSettings::Columns(settings),
            color,
            opacity: 10.,
            variable_bindings: Vec::new(),
        }
    }

    pub fn rows(settings: DesignRowLayoutGrid, color: DesignColor) -> Self {
        Self {
            id: "".into(),
            visible: true,
            settings: DesignLayoutGridSettings::Rows(settings),
            color,
            opacity: 10.,
            variable_bindings: Vec::new(),
        }
    }

    pub const fn kind(&self) -> DesignGridKind {
        self.settings.kind()
    }

    pub fn with_id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0., 100.);
        self
    }

    pub fn with_variable_binding(
        mut self,
        field: DesignLayoutGridVariableField,
        binding: DesignLayoutGridVariableBinding,
    ) -> Self {
        self.set_variable_binding(field, Some(binding));
        self
    }

    pub fn set_variable_binding(
        &mut self,
        field: DesignLayoutGridVariableField,
        binding: Option<DesignLayoutGridVariableBinding>,
    ) {
        if field == DesignLayoutGridVariableField::Count {
            match &mut self.settings {
                DesignLayoutGridSettings::Columns(settings) => settings.count_binding = None,
                DesignLayoutGridSettings::Rows(settings) => settings.count_binding = None,
                DesignLayoutGridSettings::Uniform(_) => {}
            }
        }
        self.variable_bindings
            .retain(|current| current.field != field);
        if let Some(binding) = binding {
            self.variable_bindings
                .push(DesignLayoutGridBoundVariable::new(field, binding));
        }
    }

    pub fn variable_binding(
        &self,
        field: DesignLayoutGridVariableField,
    ) -> Option<&DesignLayoutGridVariableBinding> {
        self.variable_bindings
            .iter()
            .find(|current| current.field == field)
            .map(|current| &current.binding)
            .or_else(|| {
                (field == DesignLayoutGridVariableField::Count)
                    .then_some(match &self.settings {
                        DesignLayoutGridSettings::Columns(settings) => {
                            settings.count_binding.as_ref()
                        }
                        DesignLayoutGridSettings::Rows(settings) => settings.count_binding.as_ref(),
                        DesignLayoutGridSettings::Uniform(_) => None,
                    })
                    .flatten()
            })
    }

    /// Resolves one visible inspector property onto the exact Figma
    /// variable-bindable guide field and its current typed value.
    pub fn variable_target(
        &self,
        index: usize,
        property: DesignPanelProperty,
    ) -> Option<(
        DesignLayoutGridVariableTarget,
        DesignLayoutGridVariableValue,
    )> {
        let property = property.with_layout_grid_index(index);
        let (field, value) = match (&self.settings, property) {
            (
                DesignLayoutGridSettings::Uniform(settings),
                DesignPanelProperty::LayoutGridSize(_),
            ) => (
                DesignLayoutGridVariableField::SectionSize,
                DesignLayoutGridVariableValue::Number(settings.size),
            ),
            (
                DesignLayoutGridSettings::Columns(settings),
                DesignPanelProperty::LayoutGridCount(_),
            ) => (
                DesignLayoutGridVariableField::Count,
                DesignLayoutGridVariableValue::Count(settings.count),
            ),
            (DesignLayoutGridSettings::Rows(settings), DesignPanelProperty::LayoutGridCount(_)) => {
                (
                    DesignLayoutGridVariableField::Count,
                    DesignLayoutGridVariableValue::Count(settings.count),
                )
            }
            (
                DesignLayoutGridSettings::Columns(settings),
                DesignPanelProperty::LayoutGridSize(_),
            ) if !settings.alignment.is_stretch() => (
                DesignLayoutGridVariableField::SectionSize,
                DesignLayoutGridVariableValue::Number(settings.size),
            ),
            (DesignLayoutGridSettings::Rows(settings), DesignPanelProperty::LayoutGridSize(_))
                if !settings.alignment.is_stretch() =>
            {
                (
                    DesignLayoutGridVariableField::SectionSize,
                    DesignLayoutGridVariableValue::Number(settings.size),
                )
            }
            (
                DesignLayoutGridSettings::Columns(settings),
                DesignPanelProperty::LayoutGridOffset(_),
            ) if settings.alignment.supports_offset() => (
                DesignLayoutGridVariableField::Offset,
                DesignLayoutGridVariableValue::Number(settings.offset),
            ),
            (
                DesignLayoutGridSettings::Rows(settings),
                DesignPanelProperty::LayoutGridOffset(_),
            ) if settings.alignment.supports_offset() => (
                DesignLayoutGridVariableField::Offset,
                DesignLayoutGridVariableValue::Number(settings.offset),
            ),
            (
                DesignLayoutGridSettings::Columns(settings),
                DesignPanelProperty::LayoutGridMargin(_),
            ) if settings.alignment.is_stretch() => (
                DesignLayoutGridVariableField::Offset,
                DesignLayoutGridVariableValue::Number(settings.margin),
            ),
            (
                DesignLayoutGridSettings::Rows(settings),
                DesignPanelProperty::LayoutGridMargin(_),
            ) if settings.alignment.is_stretch() => (
                DesignLayoutGridVariableField::Offset,
                DesignLayoutGridVariableValue::Number(settings.margin),
            ),
            (
                DesignLayoutGridSettings::Columns(settings),
                DesignPanelProperty::LayoutGridGutter(_),
            ) => (
                DesignLayoutGridVariableField::GutterSize,
                DesignLayoutGridVariableValue::Number(settings.gutter),
            ),
            (
                DesignLayoutGridSettings::Rows(settings),
                DesignPanelProperty::LayoutGridGutter(_),
            ) => (
                DesignLayoutGridVariableField::GutterSize,
                DesignLayoutGridVariableValue::Number(settings.gutter),
            ),
            _ => return None,
        };
        Some((
            DesignLayoutGridVariableTarget::new(self.id.clone(), index, property, field),
            value,
        ))
    }
}

impl Default for DesignLayoutGrid {
    fn default() -> Self {
        Self::uniform(10., DesignColor::BLUE)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignTextResize {
    AutoWidth,
    AutoHeight,
    Fixed,
}

impl DesignTextResize {
    pub const ALL: [Self; 3] = [Self::AutoWidth, Self::AutoHeight, Self::Fixed];

    pub const fn label(self) -> &'static str {
        match self {
            Self::AutoWidth => "Auto width",
            Self::AutoHeight => "Auto height",
            Self::Fixed => "Fixed size",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum DesignLineHeight {
    #[default]
    Auto,
    Pixels(f32),
    Percent(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DesignLetterSpacing {
    Pixels(f32),
    Percent(f32),
}

impl Default for DesignLetterSpacing {
    fn default() -> Self {
        Self::Pixels(0.)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignTextHorizontalAlignment {
    Left,
    Center,
    Right,
    Justified,
}

impl DesignTextHorizontalAlignment {
    pub const ALL: [Self; 4] = [Self::Left, Self::Center, Self::Right, Self::Justified];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Center => "Center",
            Self::Right => "Right",
            Self::Justified => "Justified",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignTextVerticalAlignment {
    Top,
    Center,
    Bottom,
}

impl DesignTextVerticalAlignment {
    pub const ALL: [Self; 3] = [Self::Top, Self::Center, Self::Bottom];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Top => "Top",
            Self::Center => "Center",
            Self::Bottom => "Bottom",
        }
    }
}

/// Figma's `LeadingTrim` value, surfaced as Vertical trim in Type settings.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignTextLeadingTrim {
    None,
    CapHeight,
}

impl DesignTextLeadingTrim {
    pub const ALL: [Self; 2] = [Self::None, Self::CapHeight];

    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::CapHeight => "Cap height",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignTextCase {
    Original,
    Uppercase,
    Lowercase,
    TitleCase,
    SmallCaps,
    SmallCapsForced,
}

impl DesignTextCase {
    pub const ALL: [Self; 6] = [
        Self::Original,
        Self::Uppercase,
        Self::Lowercase,
        Self::TitleCase,
        Self::SmallCaps,
        Self::SmallCapsForced,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Original => "Original",
            Self::Uppercase => "Uppercase",
            Self::Lowercase => "Lowercase",
            Self::TitleCase => "Title Case",
            Self::SmallCaps => "Small Caps",
            Self::SmallCapsForced => "Small Caps Forced",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignTextDecoration {
    None,
    Underline,
    Strikethrough,
}

/// Exact writable `TextNode.textDecorationStyle` values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignTextDecorationStyle {
    Solid,
    Wavy,
    Dotted,
}

impl DesignTextDecorationStyle {
    pub const ALL: [Self; 3] = [Self::Solid, Self::Wavy, Self::Dotted];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Solid => "Solid",
            Self::Wavy => "Wavy",
            Self::Dotted => "Dotted",
        }
    }
}

/// Lossless writable value for `textDecorationOffset` and
/// `textDecorationThickness`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DesignTextDecorationMetric {
    Auto,
    Pixels(f32),
    Percent(f32),
}

impl DesignTextDecorationMetric {
    pub const fn is_finite(self) -> bool {
        match self {
            Self::Auto => true,
            Self::Pixels(value) | Self::Percent(value) => value.is_finite(),
        }
    }

    pub const fn is_non_negative(self) -> bool {
        match self {
            Self::Auto => true,
            Self::Pixels(value) | Self::Percent(value) => value.is_finite() && value >= 0.,
        }
    }
}

/// Writable `TextNode.textDecorationColor`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DesignTextDecorationColor {
    Auto,
    Solid(DesignColor),
}

/// Advanced underline/strikethrough fields exposed by the current Text API.
///
/// Hosts set this to `None` when no decoration is active or when these fields
/// are unavailable for the current selection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesignTextDecorationDetails {
    pub style: DesignTextDecorationStyle,
    pub offset: DesignTextDecorationMetric,
    pub thickness: DesignTextDecorationMetric,
    pub color: DesignTextDecorationColor,
    pub skip_ink: bool,
}

impl Default for DesignTextDecorationDetails {
    fn default() -> Self {
        Self {
            style: DesignTextDecorationStyle::Solid,
            offset: DesignTextDecorationMetric::Auto,
            thickness: DesignTextDecorationMetric::Auto,
            color: DesignTextDecorationColor::Auto,
            skip_ink: true,
        }
    }
}

impl DesignTextDecoration {
    pub const ALL: [Self; 3] = [Self::None, Self::Underline, Self::Strikethrough];

    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Underline => "Underline",
            Self::Strikethrough => "Strikethrough",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignTextList {
    None,
    Bulleted,
    Numbered,
}

impl DesignTextList {
    pub const ALL: [Self; 3] = [Self::None, Self::Bulleted, Self::Numbered];

    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Bulleted => "Bulleted list",
            Self::Numbered => "Numbered list",
        }
    }
}

/// Four-character registered OpenType tags remain distinguishable from
/// opaque future tags supplied by a host.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DesignOpenTypeFeatureTag {
    Registered(SharedString),
    Opaque(SharedString),
}

impl DesignOpenTypeFeatureTag {
    /// Builds a registered OpenType tag only when the exact UTF-8 value is
    /// four ASCII characters. Unknown four-character tags are still valid.
    pub fn registered(tag: impl Into<SharedString>) -> Option<Self> {
        let tag = tag.into();
        (tag.len() == 4 && tag.is_ascii()).then_some(Self::Registered(tag))
    }

    /// Preserves a host-defined tag without normalizing or interpreting it.
    pub fn opaque(tag: impl Into<SharedString>) -> Self {
        Self::Opaque(tag.into())
    }

    pub const fn api_name(&self) -> &SharedString {
        match self {
            Self::Registered(tag) | Self::Opaque(tag) => tag,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignOpenTypeFeatureAvailability {
    Available,
    Unavailable { reason: SharedString },
}

impl DesignOpenTypeFeatureAvailability {
    pub const fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }
}

/// One exact feature record from the selected font.
///
/// Unlike `TextNode.openTypeFeatures`, this record keeps both the font default
/// and current value so the panel never infers omitted-default semantics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignOpenTypeFeature {
    pub tag: DesignOpenTypeFeatureTag,
    pub name: SharedString,
    pub default_enabled: bool,
    pub enabled: bool,
    pub availability: DesignOpenTypeFeatureAvailability,
    pub preview: Option<SharedString>,
}

impl DesignOpenTypeFeature {
    pub fn new(
        tag: DesignOpenTypeFeatureTag,
        name: impl Into<SharedString>,
        default_enabled: bool,
        enabled: bool,
    ) -> Self {
        Self {
            tag,
            name: name.into(),
            default_enabled,
            enabled,
            availability: DesignOpenTypeFeatureAvailability::Available,
            preview: None,
        }
    }

    pub fn unavailable(mut self, reason: impl Into<SharedString>) -> Self {
        self.availability = DesignOpenTypeFeatureAvailability::Unavailable {
            reason: reason.into(),
        };
        self
    }

    pub fn with_preview(mut self, preview: impl Into<SharedString>) -> Self {
        self.preview = Some(preview.into());
        self
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DesignFontSource {
    Local,
    Library {
        library_id: SharedString,
        library_name: SharedString,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DesignFontSelection {
    pub source: DesignFontSource,
    pub family_id: SharedString,
    pub style_id: SharedString,
}

impl DesignFontSelection {
    pub fn local(family_id: impl Into<SharedString>, style_id: impl Into<SharedString>) -> Self {
        Self {
            source: DesignFontSource::Local,
            family_id: family_id.into(),
            style_id: style_id.into(),
        }
    }

    pub fn library(
        library_id: impl Into<SharedString>,
        library_name: impl Into<SharedString>,
        family_id: impl Into<SharedString>,
        style_id: impl Into<SharedString>,
    ) -> Self {
        Self {
            source: DesignFontSource::Library {
                library_id: library_id.into(),
                library_name: library_name.into(),
            },
            family_id: family_id.into(),
            style_id: style_id.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignFontAvailability {
    /// The font can be loaded and applied immediately.
    Imported,
    /// A library font must be imported before it can be applied.
    Available,
    Missing {
        reason: SharedString,
    },
    Unavailable {
        reason: SharedString,
    },
}

impl DesignFontAvailability {
    pub const fn can_apply(&self) -> bool {
        matches!(self, Self::Imported)
    }

    pub const fn can_import(&self) -> bool {
        matches!(self, Self::Available)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignFontStyle {
    pub id: SharedString,
    pub name: SharedString,
    pub weight: Option<u16>,
    pub italic: bool,
    pub availability: DesignFontAvailability,
    pub preview: Option<SharedString>,
}

impl DesignFontStyle {
    pub fn imported(id: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            weight: None,
            italic: false,
            availability: DesignFontAvailability::Imported,
            preview: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignFontFamily {
    pub id: SharedString,
    pub name: SharedString,
    pub source: DesignFontSource,
    pub styles: Vec<DesignFontStyle>,
}

impl DesignFontFamily {
    pub fn local(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        styles: impl IntoIterator<Item = DesignFontStyle>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            source: DesignFontSource::Local,
            styles: styles.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum DesignFontCatalogState {
    #[default]
    Loading,
    Ready,
    Unavailable {
        reason: SharedString,
    },
}

/// Immutable result of the host's font enumeration/loading adapter.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesignFontViewData {
    pub state: DesignFontCatalogState,
    pub families: Vec<DesignFontFamily>,
}

impl DesignFontViewData {
    pub fn ready(families: impl IntoIterator<Item = DesignFontFamily>) -> Self {
        Self {
            state: DesignFontCatalogState::Ready,
            families: families.into_iter().collect(),
        }
    }

    pub fn font(
        &self,
        selection: &DesignFontSelection,
    ) -> Option<(&DesignFontFamily, &DesignFontStyle)> {
        let family = self
            .families
            .iter()
            .find(|family| family.id == selection.family_id && family.source == selection.source)?;
        let style = family
            .styles
            .iter()
            .find(|style| style.id == selection.style_id)?;
        Some((family, style))
    }

    pub fn matching(
        &self,
        query: &str,
    ) -> impl Iterator<Item = (&DesignFontFamily, &DesignFontStyle)> {
        let query = query.trim().to_ascii_lowercase();
        self.families.iter().flat_map(move |family| {
            let query = query.clone();
            family
                .styles
                .iter()
                .filter(move |style| {
                    query.is_empty()
                        || family.name.to_ascii_lowercase().contains(&query)
                        || style.name.to_ascii_lowercase().contains(&query)
                })
                .map(move |style| (family, style))
        })
    }
}

/// One immutable host-supplied text style shown in the Typography picker.
///
/// `id` is opaque and stable within its page or library scope. The font
/// summary is presentation data and lets the picker describe a style without
/// synthesizing or applying typography values itself.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignTypographyStyle {
    pub id: SharedString,
    pub name: SharedString,
    pub family: SharedString,
    pub font_style: SharedString,
    pub size: f32,
}

impl DesignTypographyStyle {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        family: impl Into<SharedString>,
        font_style: impl Into<SharedString>,
        size: f32,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            family: family.into(),
            font_style: font_style.into(),
            size,
        }
    }
}

/// A named host-supplied library and its ordered text styles.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignTypographyStyleLibrary {
    pub id: SharedString,
    pub name: SharedString,
    pub styles: Vec<DesignTypographyStyle>,
}

impl DesignTypographyStyleLibrary {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        styles: impl IntoIterator<Item = DesignTypographyStyle>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            styles: styles.into_iter().collect(),
        }
    }
}

/// Stable scope for a text-style selection.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DesignTypographyStyleSource {
    Page,
    Library { library_id: SharedString },
}

/// Opaque text-style identity returned to the host.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DesignTypographyStyleSelection {
    pub source: DesignTypographyStyleSource,
    pub style_id: SharedString,
}

impl DesignTypographyStyleSelection {
    pub fn page(style_id: impl Into<SharedString>) -> Self {
        Self {
            source: DesignTypographyStyleSource::Page,
            style_id: style_id.into(),
        }
    }

    pub fn library(library_id: impl Into<SharedString>, style_id: impl Into<SharedString>) -> Self {
        Self {
            source: DesignTypographyStyleSource::Library {
                library_id: library_id.into(),
            },
            style_id: style_id.into(),
        }
    }
}

/// Immutable text-style snapshot supplied by a Design-panel host.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesignTypographyStyleViewData {
    pub page_styles: Vec<DesignTypographyStyle>,
    pub libraries: Vec<DesignTypographyStyleLibrary>,
}

impl DesignTypographyStyleViewData {
    pub fn new(
        page_styles: impl IntoIterator<Item = DesignTypographyStyle>,
        libraries: impl IntoIterator<Item = DesignTypographyStyleLibrary>,
    ) -> Self {
        Self {
            page_styles: page_styles.into_iter().collect(),
            libraries: libraries.into_iter().collect(),
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.page_styles.is_empty() && self.libraries.is_empty()
    }

    pub fn style(
        &self,
        selection: &DesignTypographyStyleSelection,
    ) -> Option<&DesignTypographyStyle> {
        match &selection.source {
            DesignTypographyStyleSource::Page => self
                .page_styles
                .iter()
                .find(|style| style.id == selection.style_id),
            DesignTypographyStyleSource::Library { library_id } => self
                .libraries
                .iter()
                .find(|library| library.id == *library_id)?
                .styles
                .iter()
                .find(|style| style.id == selection.style_id),
        }
    }
}

/// The text style currently bound to a controlled typography snapshot.
///
/// The display name remains available even when the host no longer exposes
/// the style in the current page/library browser.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignTypographyStyleBinding {
    pub selection: DesignTypographyStyleSelection,
    pub name: SharedString,
    pub can_detach: bool,
}

impl DesignTypographyStyleBinding {
    pub fn new(selection: DesignTypographyStyleSelection, name: impl Into<SharedString>) -> Self {
        Self {
            selection,
            name: name.into(),
            can_detach: true,
        }
    }

    pub fn detachable(mut self, can_detach: bool) -> Self {
        self.can_detach = can_detach;
        self
    }
}

/// Whether one host-enumerated variable-font axis can be changed.
///
/// `ReadOnly` keeps a valid, inspectable axis distinct from an axis the
/// current font/style cannot provide. Hosts may include a reason in either
/// state so disabled controls never need to infer policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignFontAxisAvailability {
    Available,
    ReadOnly { reason: Option<SharedString> },
    Unavailable { reason: SharedString },
}

impl DesignFontAxisAvailability {
    pub const fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }

    pub const fn disabled_reason(&self) -> Option<&SharedString> {
        match self {
            Self::Available => None,
            Self::ReadOnly { reason } => reason.as_ref(),
            Self::Unavailable { reason } => Some(reason),
        }
    }
}

/// Optional variable provenance for an axis represented by a bindable
/// typography leaf such as the registered `wght`/font-weight axis.
///
/// Variable-font axes and Figma variables are different features. This
/// metadata therefore never implies that every custom axis is bindable. When
/// present, the axis remains inspectable but direct axis editing is disabled;
/// the host exposes binding replacement/detachment through its ordinary
/// typography-variable surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignFontAxisBinding {
    pub variable_id: SharedString,
    pub variable_name: SharedString,
    pub collection_name: Option<SharedString>,
}

impl DesignFontAxisBinding {
    pub fn new(
        variable_id: impl Into<SharedString>,
        variable_name: impl Into<SharedString>,
    ) -> Self {
        Self {
            variable_id: variable_id.into(),
            variable_name: variable_name.into(),
            collection_name: None,
        }
    }

    pub fn with_collection(mut self, collection_name: impl Into<SharedString>) -> Self {
        self.collection_name = Some(collection_name.into());
        self
    }
}

/// One exact axis from the active variable-font face.
///
/// `tag` is the stable OpenType axis identity and is authoritative across
/// host echoes and row reorderings. `default` and `step` are authored by the
/// host/font adapter; the panel does not guess registered-axis semantics.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignFontAxis {
    pub tag: SharedString,
    pub name: SharedString,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub step: f32,
    pub availability: DesignFontAxisAvailability,
    pub binding: Option<DesignFontAxisBinding>,
}

impl DesignFontAxis {
    pub fn new(
        tag: impl Into<SharedString>,
        name: impl Into<SharedString>,
        value: f32,
        min: f32,
        max: f32,
        default: f32,
    ) -> Self {
        Self {
            tag: tag.into(),
            name: name.into(),
            value,
            min,
            max,
            default,
            step: 1.,
            availability: DesignFontAxisAvailability::Available,
            binding: None,
        }
    }

    pub const fn with_step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    pub fn read_only(mut self) -> Self {
        self.availability = DesignFontAxisAvailability::ReadOnly { reason: None };
        self
    }

    pub fn read_only_with_reason(mut self, reason: impl Into<SharedString>) -> Self {
        self.availability = DesignFontAxisAvailability::ReadOnly {
            reason: Some(reason.into()),
        };
        self
    }

    pub fn unavailable(mut self, reason: impl Into<SharedString>) -> Self {
        self.availability = DesignFontAxisAvailability::Unavailable {
            reason: reason.into(),
        };
        self
    }

    pub fn with_binding(mut self, binding: DesignFontAxisBinding) -> Self {
        self.binding = Some(binding);
        self
    }

    /// OpenType variation-axis tags are exactly four printable ASCII bytes.
    pub fn has_valid_tag(&self) -> bool {
        self.tag.len() == 4 && self.tag.bytes().all(|byte| (0x20..=0x7e).contains(&byte))
    }

    pub fn has_valid_range(&self) -> bool {
        self.value.is_finite()
            && self.min.is_finite()
            && self.max.is_finite()
            && self.default.is_finite()
            && self.step.is_finite()
            && self.min < self.max
            && self.step > 0.
            && (self.min..=self.max).contains(&self.value)
            && (self.min..=self.max).contains(&self.default)
    }

    pub fn is_editable(&self) -> bool {
        self.has_valid_tag()
            && self.has_valid_range()
            && self.availability.is_available()
            && self.binding.is_none()
    }

    pub fn disabled_reason(&self) -> Option<SharedString> {
        if !self.has_valid_tag() {
            return Some("Variable-font axis tags must be four printable ASCII characters".into());
        }
        if !self.has_valid_range() {
            return Some("The host supplied an invalid variable-font axis range".into());
        }
        if let Some(binding) = &self.binding {
            return Some(format!("Bound to variable {}", binding.variable_name).into());
        }
        self.availability.disabled_reason().cloned().or_else(|| {
            matches!(
                self.availability,
                DesignFontAxisAvailability::ReadOnly { .. }
            )
            .then(|| SharedString::from("This variable-font axis is read-only"))
        })
    }
}

/// Exact `TextPathNode.textPathStartData` value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesignTextPathStartData {
    /// Zero-based vector segment index.
    pub segment: u32,
    /// Normalized position along `segment`.
    pub position: f32,
}

impl DesignTextPathStartData {
    pub const DEFAULT: Self = Self {
        segment: 0,
        position: 0.,
    };

    pub fn new(segment: u32, position: f32) -> Option<Self> {
        (position.is_finite() && (0. ..=1.).contains(&position))
            .then_some(Self { segment, position })
    }

    pub const fn with_segment(self, segment: u32) -> Self {
        Self { segment, ..self }
    }

    pub fn with_position(self, position: f32) -> Option<Self> {
        Self::new(self.segment, position)
    }
}

impl Default for DesignTextPathStartData {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Host-owned projection of which side of a path currently carries its text.
///
/// Figma exposes a native "Flip text orientation" command in the Typography
/// section, but the public Plugin API does not currently expose a matching
/// orientation property. The inspector therefore treats this as opaque view
/// data and emits a command instead of deriving or mutating vector geometry.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DesignTextPathOrientation {
    #[default]
    Default,
    Flipped,
}

impl DesignTextPathOrientation {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Flipped => "Flipped",
        }
    }

    pub const fn toggled(self) -> Self {
        match self {
            Self::Default => Self::Flipped,
            Self::Flipped => Self::Default,
        }
    }
}

/// Exact host-authored native TextPath inspector state.
///
/// `show_start_data_debug_controls` is deliberately separate from native
/// orientation support. `textPathStartData` is useful to API hosts and
/// diagnostics, but Figma's native sidebar positions text with a canvas handle
/// rather than exposing segment/normalized-position number fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesignTextPathViewData {
    pub orientation: DesignTextPathOrientation,
    pub can_flip_orientation: bool,
    pub show_start_data_debug_controls: bool,
}

impl DesignTextPathViewData {
    pub const fn new(orientation: DesignTextPathOrientation) -> Self {
        Self {
            orientation,
            can_flip_orientation: true,
            show_start_data_debug_controls: false,
        }
    }

    pub const fn flippable(mut self, can_flip_orientation: bool) -> Self {
        self.can_flip_orientation = can_flip_orientation;
        self
    }

    pub const fn with_start_data_debug_controls(mut self, show: bool) -> Self {
        self.show_start_data_debug_controls = show;
        self
    }
}

impl Default for DesignTextPathViewData {
    fn default() -> Self {
        Self::new(DesignTextPathOrientation::Default)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignTypography {
    pub style_binding: Option<DesignTypographyStyleBinding>,
    pub family: SharedString,
    pub style: SharedString,
    /// Figma's numeric `fontWeight` value. This remains independent from the
    /// string-valued `fontStyle` field so either leaf can carry its own
    /// variable binding.
    pub weight: f32,
    pub size: f32,
    pub line_height: DesignLineHeight,
    pub letter_spacing: DesignLetterSpacing,
    pub leading_trim: DesignTextLeadingTrim,
    pub paragraph_spacing: f32,
    pub paragraph_indent: f32,
    pub list_spacing: f32,
    pub hanging_punctuation: bool,
    pub hanging_lists: bool,
    pub horizontal_alignment: DesignTextHorizontalAlignment,
    pub vertical_alignment: DesignTextVerticalAlignment,
    pub resize: DesignTextResize,
    /// Whether Figma's ending truncation is enabled for this text layer.
    pub truncate: bool,
    /// Figma's node-level `maxLines`: `None` is native Auto, numeric values
    /// are at least one, and are applicable only under
    /// [`Self::text_max_lines_are_available`].
    pub max_lines: Option<u32>,
    pub decoration: DesignTextDecoration,
    pub decoration_details: Option<DesignTextDecorationDetails>,
    pub case: DesignTextCase,
    pub list: DesignTextList,
    pub open_type_features: Vec<DesignOpenTypeFeature>,
    pub variable_axes: Vec<DesignFontAxis>,
}

impl Default for DesignTypography {
    fn default() -> Self {
        Self {
            style_binding: None,
            family: "Inter".into(),
            style: "Regular".into(),
            weight: 400.,
            size: 16.,
            line_height: DesignLineHeight::Auto,
            letter_spacing: DesignLetterSpacing::Pixels(0.),
            leading_trim: DesignTextLeadingTrim::None,
            paragraph_spacing: 0.,
            paragraph_indent: 0.,
            list_spacing: 0.,
            hanging_punctuation: false,
            hanging_lists: false,
            horizontal_alignment: DesignTextHorizontalAlignment::Left,
            vertical_alignment: DesignTextVerticalAlignment::Top,
            resize: DesignTextResize::AutoHeight,
            truncate: false,
            max_lines: None,
            decoration: DesignTextDecoration::None,
            decoration_details: None,
            case: DesignTextCase::Original,
            list: DesignTextList::None,
            open_type_features: Vec::new(),
            variable_axes: Vec::new(),
        }
    }
}

impl DesignTypography {
    /// Whether Figma exposes the Max lines control for this text-resize state.
    ///
    /// Parent auto-layout sizing is a node-level concern and is intentionally
    /// checked by [`DesignPanelNode::text_max_lines_are_available`].
    pub const fn text_max_lines_are_available(&self) -> bool {
        self.truncate
            && matches!(
                self.resize,
                DesignTextResize::AutoWidth | DesignTextResize::AutoHeight
            )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignComponentPropertyKind {
    Variant,
    Text,
    Boolean,
    InstanceSwap,
    Slot,
}

impl DesignComponentPropertyKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Variant => "Variant",
            Self::Text => "Text",
            Self::Boolean => "Boolean",
            Self::InstanceSwap => "Instance swap",
            Self::Slot => "Slot",
        }
    }
}

/// Figma keeps Variant definitions ahead of all other component-property
/// definitions. Reorder intents carry this partition explicitly so a host can
/// reject stale cross-partition moves without relying on a display index.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignComponentPropertyPartition {
    Variant,
    Regular,
}

impl DesignComponentPropertyPartition {
    pub const fn for_kind(kind: DesignComponentPropertyKind) -> Self {
        if matches!(kind, DesignComponentPropertyKind::Variant) {
            Self::Variant
        } else {
            Self::Regular
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Variant => "Variant properties",
            Self::Regular => "Component properties",
        }
    }
}

/// Per-definition authoring permissions supplied by the host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesignComponentPropertyDefinitionCapabilities {
    pub rename: bool,
    pub edit_metadata: bool,
    pub edit_default_value: bool,
    pub edit_preferred_values: bool,
    pub edit_slot_settings: bool,
    pub delete: bool,
    pub reorder: bool,
    pub edit_variant_options: bool,
}

impl DesignComponentPropertyDefinitionCapabilities {
    pub const FULL: Self = Self {
        rename: true,
        edit_metadata: true,
        edit_default_value: true,
        edit_preferred_values: true,
        edit_slot_settings: true,
        delete: true,
        reorder: true,
        edit_variant_options: true,
    };

    pub const READ_ONLY: Self = Self {
        rename: false,
        edit_metadata: false,
        edit_default_value: false,
        edit_preferred_values: false,
        edit_slot_settings: false,
        delete: false,
        reorder: false,
        edit_variant_options: false,
    };
}

/// Stable host identity and permissions for one option in a Variant property.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignComponentVariantOptionAuthoring {
    pub id: SharedString,
    pub name: SharedString,
    pub can_rename: bool,
    pub can_delete: bool,
    pub can_reorder: bool,
}

impl DesignComponentVariantOptionAuthoring {
    pub fn editable(id: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            can_rename: true,
            can_delete: true,
            can_reorder: true,
        }
    }
}

/// Authoring projection for one exact component-property definition.
///
/// The canonical definition and current/default values remain in
/// [`DesignComponentProperty`]. This record contributes only host-authorized
/// authoring controls and stable Variant-option identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignComponentPropertyDefinitionAuthoring {
    pub property_id: SharedString,
    pub capabilities: DesignComponentPropertyDefinitionCapabilities,
    pub variant_options: Vec<DesignComponentVariantOptionAuthoring>,
}

impl DesignComponentPropertyDefinitionAuthoring {
    pub fn editable(property_id: impl Into<SharedString>) -> Self {
        Self {
            property_id: property_id.into(),
            capabilities: DesignComponentPropertyDefinitionCapabilities::FULL,
            variant_options: Vec::new(),
        }
    }

    pub fn with_variant_options(
        mut self,
        options: impl IntoIterator<Item = DesignComponentVariantOptionAuthoring>,
    ) -> Self {
        self.variant_options = options.into_iter().collect();
        self
    }
}

/// Stable property choice used by selected-sublayer application controls and
/// nested-property exposure candidates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignComponentPropertyChoice {
    pub property_id: SharedString,
    pub property_name: SharedString,
    pub kind: DesignComponentPropertyKind,
}

impl DesignComponentPropertyChoice {
    pub fn new(
        property_id: impl Into<SharedString>,
        property_name: impl Into<SharedString>,
        kind: DesignComponentPropertyKind,
    ) -> Self {
        Self {
            property_id: property_id.into(),
            property_name: property_name.into(),
            kind,
        }
    }
}

/// Figma surface beside which an applied component-property pill is rendered.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignComponentPropertyApplicationSurface {
    Appearance,
    Text,
    NestedInstance,
}

impl DesignComponentPropertyApplicationSurface {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
            Self::Text => "Text",
            Self::NestedInstance => "Nested instance",
        }
    }
}

/// Host-authorized operations for one selected-sublayer property pill.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesignAppliedComponentPropertyCapabilities {
    pub apply: bool,
    pub switch: bool,
    pub detach: bool,
}

impl DesignAppliedComponentPropertyCapabilities {
    pub const FULL: Self = Self {
        apply: true,
        switch: true,
        detach: true,
    };

    pub const READ_ONLY: Self = Self {
        apply: false,
        switch: false,
        detach: false,
    };
}

/// One host-controlled purple-pill control for the selected layer inside a
/// main component or component set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignAppliedComponentPropertyControl {
    pub control_id: SharedString,
    pub layer_id: SharedString,
    pub layer_name: SharedString,
    pub surface: DesignComponentPropertyApplicationSurface,
    pub candidates: Vec<DesignComponentPropertyChoice>,
    pub applied_property_id: Option<SharedString>,
    pub capabilities: DesignAppliedComponentPropertyCapabilities,
}

impl DesignAppliedComponentPropertyControl {
    pub fn new(
        control_id: impl Into<SharedString>,
        layer_id: impl Into<SharedString>,
        layer_name: impl Into<SharedString>,
        surface: DesignComponentPropertyApplicationSurface,
        candidates: impl IntoIterator<Item = DesignComponentPropertyChoice>,
    ) -> Self {
        Self {
            control_id: control_id.into(),
            layer_id: layer_id.into(),
            layer_name: layer_name.into(),
            surface,
            candidates: candidates.into_iter().collect(),
            applied_property_id: None,
            capabilities: DesignAppliedComponentPropertyCapabilities::FULL,
        }
    }

    pub fn applied_to(mut self, property_id: impl Into<SharedString>) -> Self {
        self.applied_property_id = Some(property_id.into());
        self
    }

    pub fn candidate(&self, property_id: &str) -> Option<&DesignComponentPropertyChoice> {
        self.candidates
            .iter()
            .find(|candidate| candidate.property_id.as_ref() == property_id)
    }

    pub fn applied_property(&self) -> Option<&DesignComponentPropertyChoice> {
        self.applied_property_id
            .as_deref()
            .and_then(|property_id| self.candidate(property_id))
    }
}

/// Host-authorized operations for one nested-property exposure row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesignNestedComponentPropertyExposureCapabilities {
    pub expose: bool,
    pub unexpose: bool,
    pub preview: bool,
}

impl DesignNestedComponentPropertyExposureCapabilities {
    pub const FULL: Self = Self {
        expose: true,
        unexpose: true,
        preview: true,
    };

    pub const READ_ONLY: Self = Self {
        expose: false,
        unexpose: false,
        preview: false,
    };
}

/// One stable nested component-property candidate which may be exposed on the
/// selected main component/component set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignNestedComponentPropertyExposureCandidate {
    pub candidate_id: SharedString,
    pub nested_instance_id: SharedString,
    pub nested_instance_name: SharedString,
    pub nested_property: DesignComponentPropertyChoice,
    pub exposed_property_id: Option<SharedString>,
    pub capabilities: DesignNestedComponentPropertyExposureCapabilities,
}

impl DesignNestedComponentPropertyExposureCandidate {
    pub fn new(
        candidate_id: impl Into<SharedString>,
        nested_instance_id: impl Into<SharedString>,
        nested_instance_name: impl Into<SharedString>,
        nested_property: DesignComponentPropertyChoice,
    ) -> Self {
        Self {
            candidate_id: candidate_id.into(),
            nested_instance_id: nested_instance_id.into(),
            nested_instance_name: nested_instance_name.into(),
            nested_property,
            exposed_property_id: None,
            capabilities: DesignNestedComponentPropertyExposureCapabilities::FULL,
        }
    }

    pub fn exposed_as(mut self, property_id: impl Into<SharedString>) -> Self {
        self.exposed_property_id = Some(property_id.into());
        self
    }
}

/// Complete host-controlled component-authoring projection.
///
/// The panel owns only focus/hover presentation. Definition application,
/// deletion, reordering, exposure, and generated identities remain host
/// operations and are reflected only after a fresh snapshot is supplied.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DesignComponentAuthoringViewData {
    pub create_kinds: Vec<DesignComponentPropertyKind>,
    pub definitions: Vec<DesignComponentPropertyDefinitionAuthoring>,
    pub applied_properties: Vec<DesignAppliedComponentPropertyControl>,
    pub exposure_candidates: Vec<DesignNestedComponentPropertyExposureCandidate>,
}

impl DesignComponentAuthoringViewData {
    pub fn definition(
        &self,
        property_id: &str,
    ) -> Option<&DesignComponentPropertyDefinitionAuthoring> {
        self.definitions
            .iter()
            .find(|definition| definition.property_id.as_ref() == property_id)
    }

    pub fn applied_control(
        &self,
        control_id: &str,
    ) -> Option<&DesignAppliedComponentPropertyControl> {
        self.applied_properties
            .iter()
            .find(|control| control.control_id.as_ref() == control_id)
    }

    pub fn exposure_candidate(
        &self,
        candidate_id: &str,
    ) -> Option<&DesignNestedComponentPropertyExposureCandidate> {
        self.exposure_candidates
            .iter()
            .find(|candidate| candidate.candidate_id.as_ref() == candidate_id)
    }

    /// Validates Figma's leading-Variant definition partition without
    /// interpreting any host identifier.
    pub fn preserves_variant_partition(&self, properties: &[DesignComponentProperty]) -> bool {
        let mut reached_regular = false;
        for property in properties {
            match DesignComponentPropertyPartition::for_kind(property.definition.kind()) {
                DesignComponentPropertyPartition::Variant if reached_regular => return false,
                DesignComponentPropertyPartition::Variant => {}
                DesignComponentPropertyPartition::Regular => reached_regular = true,
            }
        }
        true
    }
}

/// Documentation attached to a component, property, or library reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignDocumentationLink {
    pub label: SharedString,
    pub url: SharedString,
}

impl DesignDocumentationLink {
    pub fn new(label: impl Into<SharedString>, url: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            url: url.into(),
        }
    }
}

/// Where a component reference is defined.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignComponentOrigin {
    Local,
    Remote { library_name: SharedString },
}

impl DesignComponentOrigin {
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Local => "Local component",
            Self::Remote { .. } => "Library component",
        }
    }
}

/// Whether a referenced main component can currently be resolved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignComponentAvailability {
    Available,
    Missing,
    Unavailable { reason: SharedString },
}

impl DesignComponentAvailability {
    pub const fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }

    pub const fn label(&self) -> &'static str {
        match self {
            Self::Available => "Available",
            Self::Missing => "Missing",
            Self::Unavailable { .. } => "Unavailable",
        }
    }
}

/// Stable identity used by main-component, instance-swap, and slot controls.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignComponentReference {
    pub id: SharedString,
    pub name: SharedString,
    pub origin: DesignComponentOrigin,
    pub availability: DesignComponentAvailability,
    pub description: Option<SharedString>,
    pub documentation_links: Vec<DesignDocumentationLink>,
}

impl DesignComponentReference {
    pub fn local(id: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            origin: DesignComponentOrigin::Local,
            availability: DesignComponentAvailability::Available,
            description: None,
            documentation_links: Vec::new(),
        }
    }

    pub fn remote(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        library_name: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            origin: DesignComponentOrigin::Remote {
                library_name: library_name.into(),
            },
            availability: DesignComponentAvailability::Available,
            description: None,
            documentation_links: Vec::new(),
        }
    }

    pub fn with_availability(mut self, availability: DesignComponentAvailability) -> Self {
        self.availability = availability;
        self
    }
}

/// Exact Figma asset discriminant retained by an instance-swap browser row.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignComponentAssetKind {
    Component,
    ComponentSet,
}

impl DesignComponentAssetKind {
    pub const fn api_name(self) -> &'static str {
        match self {
            Self::Component => "COMPONENT",
            Self::ComponentSet => "COMPONENT_SET",
        }
    }
}

/// Stable page or library origin for a component browser result.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DesignComponentSource {
    Page {
        page_id: SharedString,
        page_name: SharedString,
    },
    Library {
        library_id: SharedString,
        library_name: SharedString,
    },
}

impl DesignComponentSource {
    pub fn page(page_id: impl Into<SharedString>, page_name: impl Into<SharedString>) -> Self {
        Self::Page {
            page_id: page_id.into(),
            page_name: page_name.into(),
        }
    }

    pub fn library(
        library_id: impl Into<SharedString>,
        library_name: impl Into<SharedString>,
    ) -> Self {
        Self::Library {
            library_id: library_id.into(),
            library_name: library_name.into(),
        }
    }

    pub const fn label(&self) -> &SharedString {
        match self {
            Self::Page { page_name, .. } => page_name,
            Self::Library { library_name, .. } => library_name,
        }
    }
}

/// Whether a browser result is already usable or still needs importing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignComponentImportState {
    Local,
    Imported,
    Available,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DesignComponentSearchMetadata {
    pub path: Vec<SharedString>,
    pub description: Option<SharedString>,
    pub keywords: Vec<SharedString>,
}

impl DesignComponentSearchMetadata {
    pub fn new(
        path: impl IntoIterator<Item = impl Into<SharedString>>,
        keywords: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        Self {
            path: path.into_iter().map(Into::into).collect(),
            description: None,
            keywords: keywords.into_iter().map(Into::into).collect(),
        }
    }

    pub fn described(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Stable selection returned by instance-swap browser intents.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DesignComponentSwapSelection {
    pub component_id: SharedString,
    pub component_key: SharedString,
    pub asset_kind: DesignComponentAssetKind,
    pub source: DesignComponentSource,
}

/// One lossless local or library result in the all-components swap browser.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignComponentSwapCandidate {
    pub reference: DesignComponentReference,
    /// Figma library identity used by `importComponentByKeyAsync`.
    pub component_key: SharedString,
    pub asset_kind: DesignComponentAssetKind,
    pub source: DesignComponentSource,
    pub import_state: DesignComponentImportState,
    pub search: DesignComponentSearchMetadata,
    pub disabled_reason: Option<SharedString>,
}

impl DesignComponentSwapCandidate {
    pub fn local(
        reference: DesignComponentReference,
        component_key: impl Into<SharedString>,
        page_id: impl Into<SharedString>,
        page_name: impl Into<SharedString>,
    ) -> Self {
        Self {
            reference,
            component_key: component_key.into(),
            asset_kind: DesignComponentAssetKind::Component,
            source: DesignComponentSource::page(page_id, page_name),
            import_state: DesignComponentImportState::Local,
            search: DesignComponentSearchMetadata::default(),
            disabled_reason: None,
        }
    }

    pub fn library(
        reference: DesignComponentReference,
        component_key: impl Into<SharedString>,
        library_id: impl Into<SharedString>,
        library_name: impl Into<SharedString>,
    ) -> Self {
        Self {
            reference,
            component_key: component_key.into(),
            asset_kind: DesignComponentAssetKind::Component,
            source: DesignComponentSource::library(library_id, library_name),
            import_state: DesignComponentImportState::Available,
            search: DesignComponentSearchMetadata::default(),
            disabled_reason: None,
        }
    }

    pub const fn with_asset_kind(mut self, asset_kind: DesignComponentAssetKind) -> Self {
        self.asset_kind = asset_kind;
        self
    }

    pub const fn with_import_state(mut self, import_state: DesignComponentImportState) -> Self {
        self.import_state = import_state;
        self
    }

    pub fn with_search(mut self, search: DesignComponentSearchMetadata) -> Self {
        self.search = search;
        self
    }

    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.disabled_reason = Some(reason.into());
        self
    }

    pub fn selection(&self) -> DesignComponentSwapSelection {
        DesignComponentSwapSelection {
            component_id: self.reference.id.clone(),
            component_key: self.component_key.clone(),
            asset_kind: self.asset_kind,
            source: self.source.clone(),
        }
    }

    pub fn can_apply(&self) -> bool {
        self.import_state != DesignComponentImportState::Available
            && self.reference.availability.is_available()
            && self.disabled_reason.is_none()
    }

    pub fn can_import(&self) -> bool {
        self.import_state == DesignComponentImportState::Available
            && self.reference.availability.is_available()
            && self.disabled_reason.is_none()
    }

    pub fn matches_search(&self, query: &str) -> bool {
        let tokens = query
            .split_whitespace()
            .map(str::to_ascii_lowercase)
            .collect::<Vec<_>>();
        if tokens.is_empty() {
            return true;
        }
        let mut haystack = format!(
            "{} {} {} {} {}",
            self.reference.id,
            self.reference.name,
            self.component_key,
            self.source.label(),
            self.asset_kind.api_name(),
        )
        .to_ascii_lowercase();
        for segment in &self.search.path {
            haystack.push(' ');
            haystack.push_str(&segment.to_ascii_lowercase());
        }
        if let Some(description) = &self.search.description {
            haystack.push(' ');
            haystack.push_str(&description.to_ascii_lowercase());
        }
        for keyword in &self.search.keywords {
            haystack.push(' ');
            haystack.push_str(&keyword.to_ascii_lowercase());
        }
        tokens.iter().all(|token| haystack.contains(token))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DesignComponentSwapViewData {
    pub candidates: Vec<DesignComponentSwapCandidate>,
}

impl DesignComponentSwapViewData {
    pub fn new(candidates: impl IntoIterator<Item = DesignComponentSwapCandidate>) -> Self {
        Self {
            candidates: candidates.into_iter().collect(),
        }
    }

    pub fn candidate(
        &self,
        selection: &DesignComponentSwapSelection,
    ) -> Option<&DesignComponentSwapCandidate> {
        self.candidates.iter().find(|candidate| {
            candidate.reference.id == selection.component_id
                && candidate.component_key == selection.component_key
                && candidate.asset_kind == selection.asset_kind
                && candidate.source == selection.source
        })
    }

    pub fn matching<'a>(
        &'a self,
        query: &'a str,
    ) -> impl Iterator<Item = &'a DesignComponentSwapCandidate> + 'a {
        self.candidates
            .iter()
            .filter(move |candidate| candidate.matches_search(query))
    }
}

/// The selected object's component-system role.
///
/// This is deliberately independent of [`DesignPanelNodeKind`]: a host may
/// inspect a variant child or a slot instance while mapping both onto its own
/// frame-like node kind.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignComponentRole {
    StandaloneMain,
    VariantChild,
    ComponentSet,
    Instance,
    SlotDefinition,
    SlotInstance,
}

impl DesignComponentRole {
    pub const fn label(self) -> &'static str {
        match self {
            Self::StandaloneMain => "Main component",
            Self::VariantChild => "Variant",
            Self::ComponentSet => "Component set",
            Self::Instance => "Instance",
            Self::SlotDefinition => "Slot definition",
            Self::SlotInstance => "Slot instance",
        }
    }

    pub const fn uses_instance_section(self) -> bool {
        matches!(self, Self::Instance | Self::SlotInstance)
    }

    pub const fn can_edit_property_value(self) -> bool {
        !matches!(self, Self::SlotDefinition)
    }

    pub const fn can_configure_slot(self) -> bool {
        matches!(self, Self::SlotDefinition)
    }

    pub const fn can_reset_instance_overrides(self) -> bool {
        matches!(self, Self::Instance | Self::SlotInstance)
    }

    pub const fn can_detach_instance(self) -> bool {
        matches!(self, Self::Instance)
    }

    pub const fn can_modify_slot_instances(self) -> bool {
        matches!(self, Self::Instance | Self::SlotInstance)
    }

    pub const fn can_author_component_properties(self) -> bool {
        matches!(self, Self::StandaloneMain | Self::ComponentSet)
    }
}

/// Host-controlled reset availability for an override or slot value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignComponentResetState {
    NotApplicable,
    Clean,
    Resettable,
    Unavailable { reason: SharedString },
}

impl DesignComponentResetState {
    pub const fn can_reset(&self) -> bool {
        matches!(self, Self::Resettable)
    }

    pub const fn is_overridden(&self) -> bool {
        matches!(self, Self::Resettable | Self::Unavailable { .. })
    }
}

/// Aggregate instance-override information shown above its properties.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignComponentOverrideSummary {
    pub overridden_property_count: u32,
    pub nested_override_count: u32,
    pub reset_state: DesignComponentResetState,
}

impl Default for DesignComponentOverrideSummary {
    fn default() -> Self {
        Self {
            overridden_property_count: 0,
            nested_override_count: 0,
            reset_state: DesignComponentResetState::Clean,
        }
    }
}

/// Component-level context shared by the property rows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignComponentContext {
    pub role: DesignComponentRole,
    pub main_component: Option<DesignComponentReference>,
    pub description: Option<SharedString>,
    pub documentation_links: Vec<DesignDocumentationLink>,
    pub overrides: DesignComponentOverrideSummary,
    /// Present only while the host exposes component-definition authoring for
    /// this exact selection.
    pub authoring: Option<DesignComponentAuthoringViewData>,
}

impl DesignComponentContext {
    pub fn new(role: DesignComponentRole) -> Self {
        Self {
            role,
            main_component: None,
            description: None,
            documentation_links: Vec::new(),
            overrides: DesignComponentOverrideSummary::default(),
            authoring: None,
        }
    }

    pub fn with_main_component(mut self, main_component: DesignComponentReference) -> Self {
        self.main_component = Some(main_component);
        self
    }

    pub fn with_authoring(mut self, authoring: DesignComponentAuthoringViewData) -> Self {
        self.authoring = Some(authoring);
        self
    }
}

/// Per-child actions that a host allows from an instance's Slot surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesignSlotChildCapabilities {
    pub select: bool,
    pub remove: bool,
    pub reorder: bool,
    pub replace_instance: bool,
}

impl DesignSlotChildCapabilities {
    pub const EDITABLE_LAYER: Self = Self {
        select: true,
        remove: true,
        reorder: true,
        replace_instance: false,
    };

    pub const EDITABLE_INSTANCE: Self = Self {
        replace_instance: true,
        ..Self::EDITABLE_LAYER
    };

    pub const READ_ONLY: Self = Self {
        select: true,
        remove: false,
        reorder: false,
        replace_instance: false,
    };
}

/// One arbitrary scene-node child contained by a Slot.
///
/// Slots accept instances, text, images, vectors, frames, and other layers.
/// `main_component` is populated only for instance children and is never used
/// as the stable identity of the inserted layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignSlotChild {
    pub node_id: SharedString,
    pub name: SharedString,
    pub kind: DesignPanelNodeKind,
    pub main_component: Option<DesignComponentReference>,
    pub capabilities: DesignSlotChildCapabilities,
}

impl DesignSlotChild {
    pub fn layer(
        node_id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        kind: DesignPanelNodeKind,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            name: name.into(),
            kind,
            main_component: None,
            capabilities: DesignSlotChildCapabilities::EDITABLE_LAYER,
        }
    }

    pub fn instance(node_id: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self {
            capabilities: DesignSlotChildCapabilities::EDITABLE_INSTANCE,
            ..Self::layer(node_id, name, DesignPanelNodeKind::Instance)
        }
    }

    /// Compatibility constructor for the original instance-only Slot model.
    #[deprecated(note = "use DesignSlotChild::instance or DesignSlotChild::layer")]
    pub fn new(node_id: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self::instance(node_id, name)
    }

    pub const fn is_instance(&self) -> bool {
        matches!(self.kind, DesignPanelNodeKind::Instance)
    }
}

/// Compatibility alias for hosts migrating from the instance-only Slot model.
#[deprecated(note = "Slots accept arbitrary layers; use DesignSlotChild")]
pub type DesignSlotInstance = DesignSlotChild;

/// Ordered, host-controlled value of a slot component property.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DesignSlotValue {
    pub children: Vec<DesignSlotChild>,
}

/// Settings authored on a slot definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignSlotSettings {
    /// Native `stretchChildOnInsert`: inserted items fill the Slot's
    /// counter-axis by default.
    pub stretch_child_on_insert: bool,
    /// Native `displayEmptyByDefault`.
    pub display_empty: bool,
    /// Native `minChildren`; `None` preserves Figma's explicit `null`.
    pub minimum_children: Option<u32>,
    /// Native `maxChildren`; `None` preserves Figma's explicit `null`.
    pub maximum_children: Option<u32>,
    pub preferred_values_only: bool,
    pub preferred_values: Vec<DesignComponentReference>,
}

impl Default for DesignSlotSettings {
    fn default() -> Self {
        Self {
            stretch_child_on_insert: true,
            display_empty: true,
            minimum_children: None,
            maximum_children: None,
            preferred_values_only: false,
            preferred_values: Vec::new(),
        }
    }
}

impl DesignSlotSettings {
    pub const fn has_valid_child_range(&self) -> bool {
        match (self.minimum_children, self.maximum_children) {
            (Some(minimum), Some(maximum)) => minimum <= maximum,
            _ => true,
        }
    }
}

/// A reason the current slot contents do not satisfy their definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignSlotViolation {
    BelowMinimum {
        minimum: u32,
        actual: u32,
    },
    AboveMaximum {
        maximum: u32,
        actual: u32,
    },
    NonPreferredValue {
        instance_id: SharedString,
        component_name: SharedString,
    },
    NonPreferredChild {
        child_id: SharedString,
        child_name: SharedString,
    },
    MissingMainComponent {
        instance_id: SharedString,
    },
}

impl DesignSlotViolation {
    pub fn label(&self) -> SharedString {
        match self {
            Self::BelowMinimum { minimum, actual } => {
                format!("Needs at least {minimum} layers; contains {actual}").into()
            }
            Self::AboveMaximum { maximum, actual } => {
                format!("Recommended maximum is {maximum} layers; currently {actual}").into()
            }
            Self::NonPreferredValue { component_name, .. } => {
                format!("{component_name} is not a preferred value").into()
            }
            Self::NonPreferredChild { child_name, .. } => {
                format!("{child_name} is not a preferred instance").into()
            }
            Self::MissingMainComponent { .. } => {
                "An inserted instance has no main component".into()
            }
        }
    }
}

/// Validation and reset state supplied for a slot instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignSlotState {
    pub violations: Vec<DesignSlotViolation>,
    pub reset_state: DesignComponentResetState,
}

impl Default for DesignSlotState {
    fn default() -> Self {
        Self {
            violations: Vec::new(),
            reset_state: DesignComponentResetState::Clean,
        }
    }
}

/// Which exact Figma component-property field receives a variable alias.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignComponentPropertyVariableField {
    DefinitionDefaultValue,
    InstanceValue,
}

impl DesignComponentPropertyVariableField {
    pub const fn api_name(self) -> &'static str {
        match self {
            Self::DefinitionDefaultValue => "defaultValue",
            Self::InstanceValue => "value",
        }
    }
}

/// Lossless bind target carried by component-property variable intents.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignComponentPropertyVariableTarget {
    pub property_id: SharedString,
    pub property_name: SharedString,
    pub property_kind: DesignComponentPropertyKind,
    pub field: DesignComponentPropertyVariableField,
    pub resolved_type: DesignVariableResolvedType,
}

/// Current variable alias retained even when it is absent from the browser.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignComponentPropertyVariableBinding {
    pub variable_id: SharedString,
    pub variable_name: SharedString,
    pub resolved_value: DesignVariableResolvedValue,
    pub can_detach: bool,
}

impl DesignComponentPropertyVariableBinding {
    pub fn new(
        variable_id: impl Into<SharedString>,
        variable_name: impl Into<SharedString>,
        resolved_value: DesignVariableResolvedValue,
    ) -> Self {
        Self {
            variable_id: variable_id.into(),
            variable_name: variable_name.into(),
            resolved_value,
            can_detach: true,
        }
    }

    pub const fn detachable(mut self, can_detach: bool) -> Self {
        self.can_detach = can_detach;
        self
    }
}

/// Whether a row is authored on the selected component or surfaced from one
/// exact nested instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignComponentPropertyOrigin {
    SelectedNode,
    NestedInstance {
        instance_id: SharedString,
        instance_name: SharedString,
        main_component: Option<DesignComponentReference>,
    },
}

impl DesignComponentPropertyOrigin {
    pub const fn nested_instance_id(&self) -> Option<&SharedString> {
        match self {
            Self::SelectedNode => None,
            Self::NestedInstance { instance_id, .. } => Some(instance_id),
        }
    }
}

/// Type-specific definition of a component property.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignComponentPropertyDefinition {
    Boolean {
        default_value: bool,
    },
    Text {
        default_value: SharedString,
        multiline: bool,
    },
    InstanceSwap {
        default_value: Option<DesignComponentReference>,
        preferred_values: Vec<DesignComponentReference>,
    },
    Variant {
        default_value: SharedString,
        options: Vec<SharedString>,
    },
    Slot {
        default_value: DesignSlotValue,
        settings: DesignSlotSettings,
    },
}

impl DesignComponentPropertyDefinition {
    pub const fn kind(&self) -> DesignComponentPropertyKind {
        match self {
            Self::Boolean { .. } => DesignComponentPropertyKind::Boolean,
            Self::Text { .. } => DesignComponentPropertyKind::Text,
            Self::InstanceSwap { .. } => DesignComponentPropertyKind::InstanceSwap,
            Self::Variant { .. } => DesignComponentPropertyKind::Variant,
            Self::Slot { .. } => DesignComponentPropertyKind::Slot,
        }
    }

    pub fn option_labels(&self) -> Vec<SharedString> {
        match self {
            Self::Boolean { .. } => vec!["True".into(), "False".into()],
            Self::Variant { options, .. } => options.clone(),
            Self::InstanceSwap {
                preferred_values, ..
            } => preferred_values
                .iter()
                .filter(|reference| reference.availability.is_available())
                .map(|reference| reference.name.clone())
                .collect(),
            Self::Text { .. } | Self::Slot { .. } => Vec::new(),
        }
    }

    pub fn default_value(&self) -> DesignComponentPropertyValue {
        match self {
            Self::Boolean { default_value } => {
                DesignComponentPropertyValue::Boolean(*default_value)
            }
            Self::Text { default_value, .. } => {
                DesignComponentPropertyValue::Text(default_value.clone())
            }
            Self::InstanceSwap { default_value, .. } => {
                DesignComponentPropertyValue::InstanceSwap(default_value.clone())
            }
            Self::Variant { default_value, .. } => {
                DesignComponentPropertyValue::Variant(default_value.clone())
            }
            Self::Slot { default_value, .. } => {
                DesignComponentPropertyValue::Slot(default_value.clone())
            }
        }
    }
}

/// Type-safe current value of a component property.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignComponentPropertyValue {
    Boolean(bool),
    Text(SharedString),
    InstanceSwap(Option<DesignComponentReference>),
    Variant(SharedString),
    Slot(DesignSlotValue),
}

impl DesignComponentPropertyValue {
    pub const fn kind(&self) -> DesignComponentPropertyKind {
        match self {
            Self::Boolean(_) => DesignComponentPropertyKind::Boolean,
            Self::Text(_) => DesignComponentPropertyKind::Text,
            Self::InstanceSwap(_) => DesignComponentPropertyKind::InstanceSwap,
            Self::Variant(_) => DesignComponentPropertyKind::Variant,
            Self::Slot(_) => DesignComponentPropertyKind::Slot,
        }
    }

    pub fn display_value(&self) -> SharedString {
        match self {
            Self::Boolean(value) => {
                if *value {
                    "True".into()
                } else {
                    "False".into()
                }
            }
            Self::Text(value) | Self::Variant(value) => value.clone(),
            Self::InstanceSwap(Some(reference)) => reference.name.clone(),
            Self::InstanceSwap(None) => "None".into(),
            Self::Slot(value) => {
                let count = value.children.len();
                format!("{count} layer{}", if count == 1 { "" } else { "s" }).into()
            }
        }
    }
}

/// Whether an individual component property differs from its main value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignComponentPropertyOverrideState {
    Default,
    Overridden,
    Mixed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DesignComponentProperty {
    /// Stable property identity. Hosts should not use the display name as a key.
    pub id: SharedString,
    pub name: SharedString,
    pub description: Option<SharedString>,
    pub documentation_links: Vec<DesignDocumentationLink>,
    pub definition: DesignComponentPropertyDefinition,
    pub resolved_value: DesignComponentPropertyValue,
    /// Alias on `componentPropertyDefinitions[property].defaultValue`.
    pub default_value_binding: Option<DesignComponentPropertyVariableBinding>,
    /// Alias on `componentProperties[property].value`.
    pub resolved_value_binding: Option<DesignComponentPropertyVariableBinding>,
    pub origin: DesignComponentPropertyOrigin,
    pub override_state: DesignComponentPropertyOverrideState,
    pub reset_state: DesignComponentResetState,
    pub slot_state: Option<DesignSlotState>,
    /// Compatibility display projection used by early story adapters.
    pub value: SharedString,
    /// Compatibility projection of [`Self::definition`].
    pub kind: DesignComponentPropertyKind,
    /// Compatibility display projection of typed variant/instance options.
    pub preferred_values: Vec<SharedString>,
}

impl DesignComponentProperty {
    fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        definition: DesignComponentPropertyDefinition,
        resolved_value: DesignComponentPropertyValue,
    ) -> Self {
        debug_assert_eq!(definition.kind(), resolved_value.kind());
        let kind = definition.kind();
        let preferred_values = definition.option_labels();
        let value = resolved_value.display_value();
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            documentation_links: Vec::new(),
            definition,
            resolved_value,
            default_value_binding: None,
            resolved_value_binding: None,
            origin: DesignComponentPropertyOrigin::SelectedNode,
            override_state: DesignComponentPropertyOverrideState::Default,
            reset_state: DesignComponentResetState::NotApplicable,
            slot_state: None,
            value,
            kind,
            preferred_values,
        }
    }

    pub fn boolean(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        default_value: bool,
        value: bool,
    ) -> Self {
        Self::new(
            id,
            name,
            DesignComponentPropertyDefinition::Boolean { default_value },
            DesignComponentPropertyValue::Boolean(value),
        )
    }

    pub fn text(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        default_value: impl Into<SharedString>,
        value: impl Into<SharedString>,
    ) -> Self {
        Self::new(
            id,
            name,
            DesignComponentPropertyDefinition::Text {
                default_value: default_value.into(),
                multiline: false,
            },
            DesignComponentPropertyValue::Text(value.into()),
        )
    }

    pub fn variant(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        default_value: impl Into<SharedString>,
        value: impl Into<SharedString>,
        options: Vec<SharedString>,
    ) -> Self {
        Self::new(
            id,
            name,
            DesignComponentPropertyDefinition::Variant {
                default_value: default_value.into(),
                options,
            },
            DesignComponentPropertyValue::Variant(value.into()),
        )
    }

    pub fn instance_swap(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        default_value: Option<DesignComponentReference>,
        value: Option<DesignComponentReference>,
        preferred_values: Vec<DesignComponentReference>,
    ) -> Self {
        Self::new(
            id,
            name,
            DesignComponentPropertyDefinition::InstanceSwap {
                default_value,
                preferred_values,
            },
            DesignComponentPropertyValue::InstanceSwap(value),
        )
    }

    pub fn slot(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        default_value: DesignSlotValue,
        value: DesignSlotValue,
        settings: DesignSlotSettings,
        state: DesignSlotState,
    ) -> Self {
        let mut property = Self::new(
            id,
            name,
            DesignComponentPropertyDefinition::Slot {
                default_value,
                settings,
            },
            DesignComponentPropertyValue::Slot(value),
        );
        property.slot_state = Some(state);
        property
    }

    pub fn with_description(mut self, description: impl Into<SharedString>) -> Self {
        if matches!(
            self.definition,
            DesignComponentPropertyDefinition::Slot { .. }
        ) {
            self.description = Some(description.into());
        }
        self
    }

    pub fn with_multiline(mut self, multiline: bool) -> Self {
        if let DesignComponentPropertyDefinition::Text {
            multiline: current, ..
        } = &mut self.definition
        {
            *current = multiline;
        }
        self
    }

    pub fn with_default_value_binding(
        mut self,
        binding: DesignComponentPropertyVariableBinding,
    ) -> Self {
        self.default_value_binding = Some(binding);
        self
    }

    pub fn with_resolved_value_binding(
        mut self,
        binding: DesignComponentPropertyVariableBinding,
    ) -> Self {
        self.resolved_value_binding = Some(binding);
        self
    }

    pub fn from_nested_instance(
        mut self,
        instance_id: impl Into<SharedString>,
        instance_name: impl Into<SharedString>,
        main_component: Option<DesignComponentReference>,
    ) -> Self {
        self.origin = DesignComponentPropertyOrigin::NestedInstance {
            instance_id: instance_id.into(),
            instance_name: instance_name.into(),
            main_component,
        };
        self
    }

    /// Returns the exact bindable API field for this row in the selected
    /// component-system role. Definition defaults and instance values have
    /// intentionally different support matrices.
    pub fn variable_target(
        &self,
        role: DesignComponentRole,
    ) -> Option<DesignComponentPropertyVariableTarget> {
        let field = match role {
            DesignComponentRole::StandaloneMain
            | DesignComponentRole::VariantChild
            | DesignComponentRole::ComponentSet => {
                DesignComponentPropertyVariableField::DefinitionDefaultValue
            }
            DesignComponentRole::Instance | DesignComponentRole::SlotInstance => {
                DesignComponentPropertyVariableField::InstanceValue
            }
            DesignComponentRole::SlotDefinition => return None,
        };
        let resolved_type = match (field, self.definition.kind()) {
            (_, DesignComponentPropertyKind::Boolean) => DesignVariableResolvedType::Boolean,
            (
                DesignComponentPropertyVariableField::DefinitionDefaultValue,
                DesignComponentPropertyKind::Text | DesignComponentPropertyKind::InstanceSwap,
            )
            | (
                DesignComponentPropertyVariableField::InstanceValue,
                DesignComponentPropertyKind::Text
                | DesignComponentPropertyKind::InstanceSwap
                | DesignComponentPropertyKind::Variant,
            ) => DesignVariableResolvedType::String,
            (
                DesignComponentPropertyVariableField::DefinitionDefaultValue,
                DesignComponentPropertyKind::Variant | DesignComponentPropertyKind::Slot,
            )
            | (
                DesignComponentPropertyVariableField::InstanceValue,
                DesignComponentPropertyKind::Slot,
            ) => return None,
        };
        Some(DesignComponentPropertyVariableTarget {
            property_id: self.id.clone(),
            property_name: self.name.clone(),
            property_kind: self.definition.kind(),
            field,
            resolved_type,
        })
    }

    pub const fn variable_binding(
        &self,
        field: DesignComponentPropertyVariableField,
    ) -> Option<&DesignComponentPropertyVariableBinding> {
        match field {
            DesignComponentPropertyVariableField::DefinitionDefaultValue => {
                self.default_value_binding.as_ref()
            }
            DesignComponentPropertyVariableField::InstanceValue => {
                self.resolved_value_binding.as_ref()
            }
        }
    }

    pub fn active_variable_binding(
        &self,
        role: DesignComponentRole,
    ) -> Option<&DesignComponentPropertyVariableBinding> {
        self.variable_target(role)
            .and_then(|target| self.variable_binding(target.field))
    }

    pub fn with_reset_state(mut self, reset_state: DesignComponentResetState) -> Self {
        self.override_state = if reset_state.is_overridden() {
            DesignComponentPropertyOverrideState::Overridden
        } else {
            DesignComponentPropertyOverrideState::Default
        };
        self.reset_state = reset_state;
        self
    }

    /// Reads typed view data while accepting the original story adapter's
    /// direct mutation of the compatibility `value` projection.
    pub fn effective_value(&self) -> DesignComponentPropertyValue {
        if self.value == self.resolved_value.display_value() {
            return self.resolved_value.clone();
        }
        self.value_from_display(self.value.clone())
            .unwrap_or_else(|| self.resolved_value.clone())
    }

    pub fn value_from_display(
        &self,
        value: impl Into<SharedString>,
    ) -> Option<DesignComponentPropertyValue> {
        let value = value.into();
        match &self.definition {
            DesignComponentPropertyDefinition::Boolean { .. } => {
                if value.eq_ignore_ascii_case("true") {
                    Some(DesignComponentPropertyValue::Boolean(true))
                } else if value.eq_ignore_ascii_case("false") {
                    Some(DesignComponentPropertyValue::Boolean(false))
                } else {
                    None
                }
            }
            DesignComponentPropertyDefinition::Text { .. } => {
                Some(DesignComponentPropertyValue::Text(value))
            }
            DesignComponentPropertyDefinition::Variant { options, .. } => options
                .contains(&value)
                .then_some(DesignComponentPropertyValue::Variant(value)),
            DesignComponentPropertyDefinition::InstanceSwap {
                preferred_values, ..
            } => {
                if value.eq_ignore_ascii_case("none") || value.is_empty() {
                    return Some(DesignComponentPropertyValue::InstanceSwap(None));
                }
                preferred_values
                    .iter()
                    .find(|reference| reference.id == value || reference.name == value)
                    .cloned()
                    .map(Some)
                    .map(DesignComponentPropertyValue::InstanceSwap)
            }
            DesignComponentPropertyDefinition::Slot { .. } => None,
        }
    }

    pub fn slot_settings(&self) -> Option<&DesignSlotSettings> {
        match &self.definition {
            DesignComponentPropertyDefinition::Slot { settings, .. } => Some(settings),
            _ => None,
        }
    }

    /// Accepts a host-applied typed value and synchronizes compatibility
    /// projections. Returns false instead of coercing a mismatched kind.
    pub fn set_resolved_value(&mut self, value: DesignComponentPropertyValue) -> bool {
        if value.kind() != self.definition.kind() {
            return false;
        }
        self.value = value.display_value();
        self.resolved_value = value;
        self.kind = self.definition.kind();
        self.preferred_values = self.definition.option_labels();
        true
    }

    /// Restores the typed authored default and clears property-level override
    /// state. The surrounding host still owns undo and document mutation.
    pub fn reset_to_default(&mut self) {
        let default_value = self.definition.default_value();
        let _ = self.set_resolved_value(default_value);
        self.override_state = DesignComponentPropertyOverrideState::Default;
        self.reset_state = DesignComponentResetState::Clean;
        if let Some(slot_state) = self.slot_state.as_mut() {
            slot_state.reset_state = DesignComponentResetState::Clean;
        }
        self.refresh_slot_violations();
    }

    pub fn mark_overridden(&mut self) {
        self.override_state = DesignComponentPropertyOverrideState::Overridden;
        self.reset_state = DesignComponentResetState::Resettable;
        if let Some(slot_state) = self.slot_state.as_mut() {
            slot_state.reset_state = DesignComponentResetState::Resettable;
        }
    }

    /// Applies a host-accepted settings intent to this view-data adapter.
    ///
    /// The inspector itself never calls this method; it is provided for mock
    /// hosts and adapters that want one canonical projection update path.
    pub fn apply_slot_settings_change(&mut self, change: &DesignSlotSettingsChange) -> bool {
        let DesignComponentPropertyDefinition::Slot { settings, .. } = &mut self.definition else {
            return false;
        };
        match change {
            DesignSlotSettingsChange::StretchChildOnInsert(value) => {
                settings.stretch_child_on_insert = *value;
            }
            DesignSlotSettingsChange::DisplayEmpty(value) => {
                settings.display_empty = *value;
            }
            DesignSlotSettingsChange::MinimumInstances(value) => {
                settings.minimum_children = *value;
            }
            DesignSlotSettingsChange::MaximumInstances(value) => {
                settings.maximum_children = *value;
            }
            DesignSlotSettingsChange::PreferredValuesOnly(value) => {
                settings.preferred_values_only = *value;
            }
            DesignSlotSettingsChange::PreferredValues(values) => {
                settings.preferred_values = values.clone();
            }
            DesignSlotSettingsChange::Replace(replacement) => {
                *settings = replacement.clone();
            }
        }
        self.preferred_values = self.definition.option_labels();
        self.refresh_slot_violations();
        true
    }

    /// Recomputes current slot guidance from the typed value and definition.
    pub fn refresh_slot_violations(&mut self) {
        let DesignComponentPropertyDefinition::Slot { settings, .. } = &self.definition else {
            return;
        };
        let DesignComponentPropertyValue::Slot(value) = &self.resolved_value else {
            return;
        };
        let actual = u32::try_from(value.children.len()).unwrap_or(u32::MAX);
        let mut violations = Vec::new();
        if let Some(minimum) = settings.minimum_children
            && actual < minimum
        {
            violations.push(DesignSlotViolation::BelowMinimum { minimum, actual });
        }
        if settings
            .maximum_children
            .is_some_and(|maximum| actual > maximum)
        {
            violations.push(DesignSlotViolation::AboveMaximum {
                maximum: settings.maximum_children.unwrap_or_default(),
                actual,
            });
        }
        if settings.preferred_values_only {
            for child in &value.children {
                match child.main_component.as_ref() {
                    Some(main)
                        if !settings
                            .preferred_values
                            .iter()
                            .any(|preferred| preferred.id == main.id) =>
                    {
                        violations.push(DesignSlotViolation::NonPreferredValue {
                            instance_id: child.node_id.clone(),
                            component_name: main.name.clone(),
                        });
                    }
                    None if child.is_instance() => {
                        violations.push(DesignSlotViolation::MissingMainComponent {
                            instance_id: child.node_id.clone(),
                        });
                    }
                    Some(_) => {}
                    None => violations.push(DesignSlotViolation::NonPreferredChild {
                        child_id: child.node_id.clone(),
                        child_name: child.name.clone(),
                    }),
                }
            }
        }
        self.slot_state
            .get_or_insert_with(DesignSlotState::default)
            .violations = violations;
    }
}

/// One typed mutation to slot-definition settings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignSlotSettingsChange {
    StretchChildOnInsert(bool),
    DisplayEmpty(bool),
    MinimumInstances(Option<u32>),
    MaximumInstances(Option<u32>),
    PreferredValuesOnly(bool),
    PreferredValues(Vec<DesignComponentReference>),
    /// Atomic replacement used by Figma's Edit Slot property modal.
    Replace(DesignSlotSettings),
}

/// Host-declared availability of Figma's corner controls.
///
/// Shape topology matters more than a coarse node kind: a closed vector or a
/// Boolean operation containing rectangles can expose radius controls while
/// an open vector cannot. Hosts can therefore override this preset without
/// changing the node classification.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DesignCornerCapabilities {
    pub uniform_radius: bool,
    pub independent_radii: bool,
    pub smoothing: bool,
}

impl DesignCornerCapabilities {
    pub const NONE: Self = Self {
        uniform_radius: false,
        independent_radii: false,
        smoothing: false,
    };

    pub const fn for_node_kind(kind: DesignPanelNodeKind) -> Self {
        let uniform_radius = kind.supports_corner_radius();
        Self {
            uniform_radius,
            independent_radii: matches!(
                kind,
                DesignPanelNodeKind::Frame
                    | DesignPanelNodeKind::Section
                    | DesignPanelNodeKind::Component
                    | DesignPanelNodeKind::ComponentSet
                    | DesignPanelNodeKind::Instance
                    | DesignPanelNodeKind::Slot
                    | DesignPanelNodeKind::Rectangle
                    | DesignPanelNodeKind::Image
                    | DesignPanelNodeKind::Video
            ),
            smoothing: uniform_radius,
        }
    }

    pub const fn has_any(self) -> bool {
        self.uniform_radius || self.independent_radii || self.smoothing
    }
}

/// Canonical Figma `ArcData`.
///
/// Angles are stored in radians and `inner_radius` is a unit interval. The
/// Design panel converts angles to degrees and the radius to a percentage for
/// display and editing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesignArcData {
    pub starting_angle: f32,
    pub ending_angle: f32,
    pub inner_radius: f32,
}

impl DesignArcData {
    pub const FULL_CIRCLE: Self = Self {
        starting_angle: 0.,
        ending_angle: std::f32::consts::TAU,
        inner_radius: 0.,
    };

    pub fn new(starting_angle: f32, ending_angle: f32, inner_radius: f32) -> Self {
        Self {
            starting_angle,
            ending_angle,
            inner_radius: inner_radius.clamp(0., 1.),
        }
    }

    pub fn starting_degrees(self) -> f32 {
        self.starting_angle.to_degrees()
    }

    pub fn sweep_angle(self) -> f32 {
        self.ending_angle - self.starting_angle
    }

    pub fn sweep_degrees(self) -> f32 {
        self.sweep_angle().to_degrees()
    }

    pub fn ending_degrees(self) -> f32 {
        self.ending_angle.to_degrees()
    }

    /// Whether Figma's current Design panel exposes the Arc controls.
    ///
    /// A plain full ellipse has no Arc section in the current UI. The controls
    /// appear after the ellipse becomes a partial arc or a ring.
    pub fn has_inspector_controls(self) -> bool {
        let sweep = self.sweep_angle().abs();
        self.inner_radius > f32::EPSILON || (sweep - std::f32::consts::TAU).abs() > f32::EPSILON
    }
}

impl Default for DesignArcData {
    fn default() -> Self {
        Self::FULL_CIRCLE
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignBooleanOperation {
    Union,
    Subtract,
    Intersect,
    Exclude,
}

impl DesignBooleanOperation {
    pub const ALL: [Self; 4] = [Self::Union, Self::Subtract, Self::Intersect, Self::Exclude];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Union => "Union selection",
            Self::Subtract => "Subtract selection",
            Self::Intersect => "Intersect selection",
            Self::Exclude => "Exclude selection",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesignPolygonGeometry {
    pub point_count: u16,
}

impl DesignPolygonGeometry {
    pub const fn new(point_count: u16) -> Self {
        Self {
            point_count: if point_count < 3 {
                3
            } else if point_count > 60 {
                60
            } else {
                point_count
            },
        }
    }
}

impl Default for DesignPolygonGeometry {
    fn default() -> Self {
        Self::new(3)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesignStarGeometry {
    pub point_count: u16,
    /// Figma `innerRadius`, normalized to `0..=1`.
    pub inner_radius: f32,
}

impl DesignStarGeometry {
    pub fn new(point_count: u16, inner_radius: f32) -> Self {
        Self {
            point_count: point_count.clamp(3, 60),
            inner_radius: inner_radius.clamp(0., 1.),
        }
    }
}

impl Default for DesignStarGeometry {
    fn default() -> Self {
        Self::new(5, 0.38)
    }
}

/// Read-only counts shown when a host inspects a FigJam table through this
/// reusable surface. Figma Design does not provide table-structure mutations
/// in its Design properties panel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DesignTableGeometry {
    pub row_count: u16,
    pub column_count: u16,
}

impl DesignTableGeometry {
    pub const fn new(row_count: u16, column_count: u16) -> Self {
        Self {
            row_count,
            column_count,
        }
    }
}

impl Default for DesignTableGeometry {
    fn default() -> Self {
        Self::new(4, 3)
    }
}

/// One mutually-exclusive native shape payload.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum DesignShapeGeometry {
    #[default]
    None,
    Polygon(DesignPolygonGeometry),
    Star(DesignStarGeometry),
    Ellipse(DesignArcData),
    Boolean(DesignBooleanOperation),
    Table(DesignTableGeometry),
}

impl DesignShapeGeometry {
    pub const fn for_node_kind(kind: DesignPanelNodeKind) -> Self {
        match kind {
            DesignPanelNodeKind::Polygon => Self::Polygon(DesignPolygonGeometry::new(3)),
            DesignPanelNodeKind::Star => Self::Star(DesignStarGeometry {
                point_count: 5,
                inner_radius: 0.38,
            }),
            DesignPanelNodeKind::Ellipse => Self::Ellipse(DesignArcData::FULL_CIRCLE),
            DesignPanelNodeKind::BooleanOperation => Self::Boolean(DesignBooleanOperation::Union),
            DesignPanelNodeKind::Table => Self::Table(DesignTableGeometry::new(4, 3)),
            _ => Self::None,
        }
    }

    pub fn has_controls(self) -> bool {
        match self {
            Self::None => false,
            Self::Ellipse(arc) => arc.has_inspector_controls(),
            Self::Polygon(_) | Self::Star(_) | Self::Boolean(_) | Self::Table(_) => true,
        }
    }

    /// Whether this payload contributes native rows to Figma Design's
    /// Appearance section.
    ///
    /// Boolean operations live in the selected-node header and Tables are a
    /// FigJam compatibility projection, so neither is canonical Appearance
    /// content.
    pub fn has_appearance_controls(self) -> bool {
        match self {
            Self::Polygon(_) | Self::Star(_) => true,
            Self::Ellipse(arc) => arc.has_inspector_controls(),
            Self::None | Self::Boolean(_) | Self::Table(_) => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignSectionDevStatusKind {
    ReadyForDev,
    Completed,
}

impl DesignSectionDevStatusKind {
    pub const ALL: [Self; 2] = [Self::ReadyForDev, Self::Completed];

    pub const fn label(self) -> &'static str {
        match self {
            Self::ReadyForDev => "Ready for dev",
            Self::Completed => "Completed",
        }
    }
}

/// Current host-resolved Dev Mode status for a section.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignSectionDevStatus {
    pub kind: DesignSectionDevStatusKind,
    pub description: Option<SharedString>,
    /// Figma's UI-only Changed indicator. The plugin API does not currently
    /// expose this bit, so a host can leave it false when unavailable.
    pub changed: bool,
}

impl DesignSectionDevStatus {
    pub const fn new(kind: DesignSectionDevStatusKind) -> Self {
        Self {
            kind,
            description: None,
            changed: false,
        }
    }

    pub fn with_description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub const fn changed(mut self, changed: bool) -> Self {
        self.changed = changed;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DesignSectionCapabilities {
    pub share: bool,
    pub set_dev_status: bool,
    pub completed_status: bool,
    pub resolve_changed_status: bool,
}

impl Default for DesignSectionCapabilities {
    fn default() -> Self {
        Self {
            share: true,
            set_dev_status: true,
            completed_status: true,
            resolve_changed_status: true,
        }
    }
}

/// Section-only properties from Figma's current inspector contract.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DesignSectionProperties {
    pub contents_hidden: bool,
    pub dev_status: Option<DesignSectionDevStatus>,
    pub capabilities: DesignSectionCapabilities,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignRepeatAxis {
    Horizontal,
    Vertical,
}

impl DesignRepeatAxis {
    pub const ALL: [Self; 2] = [Self::Horizontal, Self::Vertical];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Horizontal => "Horizontal",
            Self::Vertical => "Vertical",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignRepeatType {
    Linear,
    Radial,
}

impl DesignRepeatType {
    pub const ALL: [Self; 2] = [Self::Linear, Self::Radial];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Linear => "Linear repeat",
            Self::Radial => "Radial repeat",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignRepeatMode {
    Linear(DesignRepeatAxis),
    Radial,
}

impl DesignRepeatMode {
    pub const fn repeat_type(self) -> DesignRepeatType {
        match self {
            Self::Linear(_) => DesignRepeatType::Linear,
            Self::Radial => DesignRepeatType::Radial,
        }
    }

    pub const fn axis(self) -> Option<DesignRepeatAxis> {
        match self {
            Self::Linear(axis) => Some(axis),
            Self::Radial => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignTransformUnit {
    Relative,
    Pixels,
}

impl DesignTransformUnit {
    pub const ALL: [Self; 2] = [Self::Relative, Self::Pixels];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Relative => "Relative",
            Self::Pixels => "Pixels",
        }
    }

    pub const fn suffix(self) -> &'static str {
        match self {
            Self::Relative => "%",
            Self::Pixels => " px",
        }
    }
}

/// Figma's currently-supported `REPEAT` transform modifier.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignRepeatModifier {
    /// Stable host identity retained while the modifier array is reordered.
    pub id: SharedString,
    pub mode: DesignRepeatMode,
    pub count: u32,
    pub unit: DesignTransformUnit,
    pub offset: f32,
}

impl DesignRepeatModifier {
    pub fn linear(id: impl Into<SharedString>, axis: DesignRepeatAxis) -> Self {
        Self {
            id: id.into(),
            mode: DesignRepeatMode::Linear(axis),
            count: 5,
            unit: DesignTransformUnit::Relative,
            offset: 100.,
        }
    }

    pub fn radial(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            mode: DesignRepeatMode::Radial,
            count: 8,
            unit: DesignTransformUnit::Pixels,
            offset: 96.,
        }
    }
}

/// One host-applied mutation to a repeat modifier.
///
/// Mode changes are fully discriminated so switching a radial modifier back
/// to linear also carries the axis Figma requires.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DesignTransformModifierChange {
    Mode(DesignRepeatMode),
    Count(u32),
    Unit(DesignTransformUnit),
    Offset(f32),
}

/// Legacy node-level media projection retained for source compatibility.
///
/// New hosts should model image/video through
/// [`DesignPaintPayload::Image`] and [`DesignPaintPayload::Video`]. The Design
/// panel no longer renders this duplicate snapshot or emits
/// `ReplaceMediaRequested`; media source and filter edits are paint intents.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignMedia {
    pub crop_mode: SharedString,
    pub exposure: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub temperature: f32,
    pub tint: f32,
    pub highlights: f32,
    pub shadows: f32,
}

impl Default for DesignMedia {
    fn default() -> Self {
        Self {
            crop_mode: "Fill".into(),
            exposure: 0.,
            contrast: 0.,
            saturation: 0.,
            temperature: 0.,
            tint: 0.,
            highlights: 0.,
            shadows: 0.,
        }
    }
}

impl DesignMedia {
    /// Adapts the legacy crop label to the canonical media-paint value.
    pub fn paint_scale_mode(&self) -> DesignMediaPaintScaleMode {
        match self.crop_mode.as_ref().to_ascii_lowercase().as_str() {
            "fit" => DesignMediaPaintScaleMode::Fit,
            "crop" => DesignMediaPaintScaleMode::Crop,
            "tile" => DesignMediaPaintScaleMode::Tile,
            _ => DesignMediaPaintScaleMode::Fill,
        }
    }

    /// Adapts legacy adjustment fields without inventing source/crop data.
    pub const fn image_filters(&self) -> DesignImageFilters {
        DesignImageFilters {
            exposure: self.exposure,
            contrast: self.contrast,
            saturation: self.saturation,
            temperature: self.temperature,
            tint: self.tint,
            highlights: self.highlights,
            shadows: self.shadows,
        }
    }
}

/// Paint collection containing one occurrence in a Selection colors aggregate.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignSelectionPaintCollection {
    Fill,
    Stroke,
}

impl DesignSelectionPaintCollection {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Fill => "Fill",
            Self::Stroke => "Stroke",
        }
    }
}

/// Stable identity for one underlying use of an aggregate selection paint.
///
/// `paint_index` and `gradient_stop_index` are compatibility fallbacks only;
/// hosts should preserve `paint_id` and `gradient_stop_id` across reorders.
/// Canonical getSelectionColors-like rows target the complete Solid/Gradient
/// paint and leave the gradient-stop fields empty. The optional stop identity
/// is retained for older leaf-color adapters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignSelectionPaintReference {
    pub node_id: SharedString,
    pub collection: DesignSelectionPaintCollection,
    pub paint_id: SharedString,
    pub paint_index: usize,
    pub gradient_stop_id: Option<SharedString>,
    pub gradient_stop_index: Option<usize>,
}

impl DesignSelectionPaintReference {
    pub fn paint(
        node_id: impl Into<SharedString>,
        collection: DesignSelectionPaintCollection,
        paint_id: impl Into<SharedString>,
        paint_index: usize,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            collection,
            paint_id: paint_id.into(),
            paint_index,
            gradient_stop_id: None,
            gradient_stop_index: None,
        }
    }

    pub fn gradient_stop(mut self, stop_id: impl Into<SharedString>, stop_index: usize) -> Self {
        self.gradient_stop_id = Some(stop_id.into());
        self.gradient_stop_index = Some(stop_index);
        self
    }
}

/// One deduplicated Solid/Gradient paint row across a multiple selection.
///
/// `paint` retains the representative Figma `Paint` rather than projecting
/// the row down to RGBA. `color` is a compatibility projection used by older
/// color-only hosts. `style_paints` is the complete ordered Fill/Stroke
/// collection used by whole Paint-style creation, while `style_binding` and
/// `binding` deliberately model the collection-level Paint style and the
/// Solid paint's Color-variable binding as separate identities. Gradient
/// rows do not synthesize one aggregate Color-variable binding from their
/// independently bindable stops.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignSelectionColor {
    /// Stable opaque aggregate-row identity.
    pub id: SharedString,
    /// Representative host paint for this aggregate row.
    pub paint: DesignPaint,
    pub color: DesignColor,
    /// Number shown by Figma when the color occurs more than once.
    pub occurrence_count: u32,
    /// Stable references the host will edit when accepting an edit.
    pub paint_references: Vec<DesignSelectionPaintReference>,
    /// Complete ordered collection snapshot for whole Paint-style operations.
    ///
    /// Empty means the compatibility adapter did not supply collection
    /// context, so the panel must not offer Paint-style creation.
    pub style_paints: Vec<DesignPaint>,
    /// Uniform collection-level `fillStyleId`/`strokeStyleId`, if every
    /// occurrence represented by this row shares the same binding.
    pub style_binding: Option<DesignPaintStyleBinding>,
    /// Uniform Solid-color variable binding.
    pub binding: Option<DesignPaintBinding>,
    pub read_only: bool,
}

impl DesignSelectionColor {
    pub fn new(
        id: impl Into<SharedString>,
        color: DesignColor,
        paint_references: impl IntoIterator<Item = DesignSelectionPaintReference>,
    ) -> Self {
        let paint_references = paint_references.into_iter().collect::<Vec<_>>();
        Self {
            id: id.into(),
            paint: DesignPaint::solid(color),
            color,
            occurrence_count: u32::try_from(paint_references.len()).unwrap_or(u32::MAX),
            paint_references,
            style_paints: Vec::new(),
            style_binding: None,
            binding: None,
            read_only: false,
        }
    }

    /// Sets a host-computed count for occurrences that cannot all be expanded
    /// into references at the inspector boundary.
    pub fn with_occurrence_count(mut self, occurrence_count: u32) -> Self {
        self.occurrence_count = occurrence_count;
        self
    }

    pub fn with_binding(mut self, binding: DesignPaintBinding) -> Self {
        self.binding = Some(binding);
        self
    }

    /// Retains the exact representative paint for getSelectionColors-like
    /// aggregation. Hosts should omit occurrence-specific IDs from their
    /// deduplication key while preserving all paint semantics.
    pub fn with_paint(mut self, paint: DesignPaint) -> Self {
        self.paint = paint;
        self
    }

    /// Supplies the complete ordered Paint-style context independently of the
    /// selected color leaf.
    pub fn with_style_context(
        mut self,
        paints: impl IntoIterator<Item = DesignPaint>,
        binding: Option<DesignPaintStyleBinding>,
    ) -> Self {
        self.style_paints = paints.into_iter().collect();
        self.style_binding = binding;
        self
    }

    pub const fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
}

/// Canonical host-owned aggregate behind Figma's Selection colors section.
///
/// Despite the historical type name, each canonical row represents one full
/// normal Solid/Gradient paint, not one row per gradient stop.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesignSelectionColors {
    pub colors: Vec<DesignSelectionColor>,
}

impl DesignSelectionColors {
    pub fn new(colors: impl IntoIterator<Item = DesignSelectionColor>) -> Self {
        Self {
            colors: colors.into_iter().collect(),
        }
    }

    pub fn total_occurrences(&self) -> u32 {
        self.colors.iter().fold(0_u32, |total, color| {
            total.saturating_add(color.occurrence_count)
        })
    }

    pub fn color(&self, id: &str) -> Option<&DesignSelectionColor> {
        self.colors.iter().find(|color| color.id.as_ref() == id)
    }
}

/// Exact writable values of Figma's `HandleMirroring` Plugin API type.
///
/// Mixed selections are represented by [`DesignVectorSelectionValue::Mixed`],
/// not by an invented writable value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignHandleMirroring {
    None,
    Angle,
    AngleAndLength,
}

impl DesignHandleMirroring {
    pub const ALL: [Self; 3] = [Self::None, Self::Angle, Self::AngleAndLength];

    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Angle => "Mirror angle",
            Self::AngleAndLength => "Mirror angle & length",
        }
    }

    pub const fn api_name(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Angle => "ANGLE",
            Self::AngleAndLength => "ANGLE_AND_LENGTH",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignVectorCoordinateAxis {
    X,
    Y,
}

/// Host-resolved state for one property across the selected vector vertices.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignVectorSelectionValue<T> {
    Unset,
    Uniform(T),
    Mixed,
}

impl<T> DesignVectorSelectionValue<T> {
    pub const fn is_unset(&self) -> bool {
        matches!(self, Self::Unset)
    }

    pub const fn is_mixed(&self) -> bool {
        matches!(self, Self::Mixed)
    }

    pub const fn uniform(&self) -> Option<&T> {
        match self {
            Self::Uniform(value) => Some(value),
            Self::Unset | Self::Mixed => None,
        }
    }
}

impl<T: PartialEq> DesignVectorSelectionValue<T> {
    fn from_values(values: impl IntoIterator<Item = T>) -> Self {
        let mut values = values.into_iter();
        let Some(first) = values.next() else {
            return Self::Unset;
        };
        if values.all(|value| value == first) {
            Self::Uniform(first)
        } else {
            Self::Mixed
        }
    }
}

/// Topology of one stable host vertex in a vector network.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignVectorVertexTopology {
    Endpoint,
    Interior,
    Branch,
}

impl DesignVectorVertexTopology {
    /// Branch vertices can join more than two segments, so Figma's single
    /// tangent-mirroring and per-vertex rounding controls are ambiguous there.
    pub const fn supports_single_path_controls(self) -> bool {
        !matches!(self, Self::Branch)
    }
}

/// Immutable coordinate supplied by the host for one vector vertex.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DesignVectorPoint {
    pub x: f32,
    pub y: f32,
}

impl DesignVectorPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn is_valid(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

/// One stable, host-owned vertex exposed while editing a Vector or TextPath.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignVectorVertexViewData {
    /// Opaque identity authored by the host; never inferred from list order.
    pub id: SharedString,
    pub position: DesignVectorPoint,
    /// `None` means this vertex does not expose per-vertex rounding.
    pub corner_radius: Option<f32>,
    /// `None` means this vertex has no editable tangent mirroring control.
    pub handle_mirroring: Option<DesignHandleMirroring>,
    pub topology: DesignVectorVertexTopology,
    pub selected: bool,
    pub read_only: bool,
}

impl DesignVectorVertexViewData {
    pub fn new(id: impl Into<SharedString>, x: f32, y: f32) -> Self {
        Self {
            id: id.into(),
            position: DesignVectorPoint::new(x, y),
            corner_radius: Some(0.),
            handle_mirroring: Some(DesignHandleMirroring::None),
            topology: DesignVectorVertexTopology::Interior,
            selected: false,
            read_only: false,
        }
    }

    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub const fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub const fn with_topology(mut self, topology: DesignVectorVertexTopology) -> Self {
        self.topology = topology;
        self
    }

    pub const fn with_corner_radius(mut self, corner_radius: Option<f32>) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    pub const fn with_handle_mirroring(
        mut self,
        handle_mirroring: Option<DesignHandleMirroring>,
    ) -> Self {
        self.handle_mirroring = handle_mirroring;
        self
    }

    pub fn is_valid(&self) -> bool {
        !self.id.is_empty()
            && self.position.is_valid()
            && self
                .corner_radius
                .is_none_or(|radius| radius.is_finite() && radius >= 0.)
    }

    pub const fn supports_corner_radius(&self) -> bool {
        self.topology.supports_single_path_controls() && self.corner_radius.is_some()
    }

    pub const fn supports_handle_mirroring(&self) -> bool {
        self.topology.supports_single_path_controls() && self.handle_mirroring.is_some()
    }
}

/// Host-controlled sub-selection presented while a Vector or TextPath is in
/// vector-edit mode.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesignVectorEditViewData {
    pub vertices: Vec<DesignVectorVertexViewData>,
    pub read_only: bool,
    pub disabled_reason: Option<SharedString>,
}

impl DesignVectorEditViewData {
    pub fn new(vertices: impl IntoIterator<Item = DesignVectorVertexViewData>) -> Self {
        Self {
            vertices: vertices.into_iter().collect(),
            read_only: false,
            disabled_reason: None,
        }
    }

    pub const fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn with_disabled_reason(mut self, reason: impl Into<SharedString>) -> Self {
        self.disabled_reason = Some(reason.into());
        self
    }

    pub fn is_valid(&self) -> bool {
        let mut ids = HashSet::with_capacity(self.vertices.len());
        self.vertices
            .iter()
            .all(|vertex| vertex.is_valid() && ids.insert(vertex.id.clone()))
    }

    pub fn vertex(&self, vertex_id: &str) -> Option<&DesignVectorVertexViewData> {
        (!vertex_id.is_empty()).then_some(())?;
        self.vertices
            .iter()
            .find(|vertex| vertex.id.as_ref() == vertex_id)
    }

    pub fn selected_vertices(&self) -> impl Iterator<Item = &DesignVectorVertexViewData> {
        self.vertices.iter().filter(|vertex| vertex.selected)
    }

    pub fn selected_vertex_ids(&self) -> Vec<SharedString> {
        self.selected_vertices()
            .map(|vertex| vertex.id.clone())
            .collect()
    }

    pub fn has_selection(&self) -> bool {
        self.selected_vertices().next().is_some()
    }

    pub fn selected_x(&self) -> DesignVectorSelectionValue<f32> {
        DesignVectorSelectionValue::from_values(
            self.selected_vertices().map(|vertex| vertex.position.x),
        )
    }

    pub fn selected_y(&self) -> DesignVectorSelectionValue<f32> {
        DesignVectorSelectionValue::from_values(
            self.selected_vertices().map(|vertex| vertex.position.y),
        )
    }

    pub fn selected_corner_radius(&self) -> DesignVectorSelectionValue<f32> {
        if !self
            .selected_vertices()
            .all(DesignVectorVertexViewData::supports_corner_radius)
        {
            return DesignVectorSelectionValue::Unset;
        }
        DesignVectorSelectionValue::from_values(
            self.selected_vertices()
                .filter_map(|vertex| vertex.corner_radius),
        )
    }

    pub fn selected_handle_mirroring(&self) -> DesignVectorSelectionValue<DesignHandleMirroring> {
        if !self
            .selected_vertices()
            .all(DesignVectorVertexViewData::supports_handle_mirroring)
        {
            return DesignVectorSelectionValue::Unset;
        }
        DesignVectorSelectionValue::from_values(
            self.selected_vertices()
                .filter_map(|vertex| vertex.handle_mirroring),
        )
    }

    pub fn can_select_vertices(&self) -> bool {
        self.is_valid() && !self.read_only
    }

    pub fn can_edit_coordinates(&self) -> bool {
        self.can_edit_selected(|_| true)
    }

    pub fn can_edit_corner_radius(&self) -> bool {
        self.can_edit_selected(DesignVectorVertexViewData::supports_corner_radius)
    }

    pub fn can_edit_handle_mirroring(&self) -> bool {
        self.can_edit_selected(DesignVectorVertexViewData::supports_handle_mirroring)
    }

    pub fn contains_exact_vertices(&self, vertex_ids: &[SharedString]) -> bool {
        !vertex_ids.is_empty()
            && vertex_ids.iter().all(|id| self.vertex(id).is_some())
            && vertex_ids.iter().collect::<HashSet<_>>().len() == vertex_ids.len()
    }

    fn can_edit_selected(&self, supports: impl Fn(&DesignVectorVertexViewData) -> bool) -> bool {
        self.is_valid()
            && !self.read_only
            && self.has_selection()
            && self
                .selected_vertices()
                .all(|vertex| !vertex.read_only && supports(vertex))
    }
}

/// Immutable data displayed by [`super::DesignPanel`].
#[derive(Clone, Debug, PartialEq)]
pub struct DesignPanelNode {
    pub id: SharedString,
    pub name: SharedString,
    pub kind: DesignPanelNodeKind,
    /// Optional authoritative inspector capabilities for this exact node.
    ///
    /// When absent, the compatibility presets on [`DesignPanelNodeKind`] are
    /// used. Hosts should provide this for `Other` and for future node kinds
    /// whose inspector surface cannot be inferred losslessly from the coarse
    /// compatibility taxonomy.
    pub capabilities: Option<DesignPanelNodeCapabilities>,
    /// Host-owned scene-node visibility shown by the Appearance eye control.
    pub visible: bool,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub lock_aspect_ratio: bool,
    /// Whether this layer is nested below a component instance.
    ///
    /// Figma keeps width and height editable for these descendants but does
    /// not expose the aspect-ratio setting; it must be changed on the
    /// corresponding layer in the main component.
    pub is_component_instance_child: bool,
    pub horizontal_constraint: DesignConstraint,
    pub vertical_constraint: DesignConstraint,
    pub layout: Option<DesignLayout>,
    pub opacity: f32,
    pub blend_mode: DesignBlendMode,
    pub corner_radii: [f32; 4],
    pub independent_corners: bool,
    /// Figma `cornerSmoothing`, normalized to `0..=1`.
    pub corner_smoothing: f32,
    pub corner_capabilities: DesignCornerCapabilities,
    /// Node-level Figma `fillStyleId` binding for the complete ordered Fill
    /// paint collection.
    pub fill_style_binding: Option<DesignPaintStyleBinding>,
    pub fills: Vec<DesignPaint>,
    /// Host-controlled native Frame-fill export visibility.
    ///
    /// `None` means the inspected host/node does not expose Figma's
    /// "Show in exports" control. The public Plugin API has no corresponding
    /// field, so the inspector deliberately keeps this capability opaque.
    pub fill_shows_in_exports: Option<bool>,
    /// One shared stroke geometry/style with zero or more indexed paint fills.
    pub stroke: Option<DesignStroke>,
    /// Node-level Figma `strokeStyleId` binding for the complete ordered Stroke
    /// paint collection.
    pub stroke_style_binding: Option<DesignPaintStyleBinding>,
    pub effects: Vec<DesignEffect>,
    pub effect_capabilities: DesignEffectCapabilities,
    pub effect_style_binding: Option<DesignEffectStyleBinding>,
    pub layout_grids: Vec<DesignLayoutGrid>,
    /// Node-level Figma `gridStyleId` binding for the complete ordered guide
    /// collection.
    pub layout_grid_style_binding: Option<DesignLayoutGridStyleBinding>,
    pub export_settings: Vec<DesignExportSetting>,
    pub typography: Option<DesignTypography>,
    /// Present only when the host exposes native TextPath orientation state.
    pub text_path: Option<DesignTextPathViewData>,
    /// Present only for a TextPath node.
    pub text_path_start_data: Option<DesignTextPathStartData>,
    /// Present only while the host supplies Vector/TextPath sub-selection data.
    pub vector_edit: Option<DesignVectorEditViewData>,
    pub component_context: Option<DesignComponentContext>,
    pub component_properties: Vec<DesignComponentProperty>,
    /// Compatibility-only duplicate of image/video paint data.
    pub media: Option<DesignMedia>,
    pub shape_geometry: DesignShapeGeometry,
    pub section: Option<DesignSectionProperties>,
    pub transform_modifiers: Vec<DesignRepeatModifier>,
    /// Canonical orthogonal mask flag, independent of [`Self::kind`].
    pub is_mask: bool,
    /// Typed mask evaluation mode used when [`Self::is_mask`] is true.
    pub mask_mode: DesignMaskType,
    /// Legacy string projection retained for source compatibility. New hosts
    /// should use [`Self::is_mask`] and [`Self::mask_mode`].
    pub mask_type: Option<SharedString>,
    /// Canonical multiple-selection color aggregate.
    pub selection_color_aggregate: DesignSelectionColors,
    /// Legacy aggregate encoded as fake paints. New hosts should populate
    /// [`Self::selection_color_aggregate`].
    pub selection_colors: Vec<DesignPaint>,
}

impl DesignPanelNode {
    /// Creates a representative controlled inspector read model for a node kind.
    ///
    /// Hosts normally populate these fields from their own document read model;
    /// the preset is also useful for stories and integration smoke tests.
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        kind: DesignPanelNodeKind,
    ) -> Self {
        let mut node = Self {
            id: id.into(),
            name: name.into(),
            kind,
            capabilities: None,
            visible: true,
            x: 120.,
            y: 96.,
            width: 320.,
            height: 180.,
            rotation: 0.,
            lock_aspect_ratio: false,
            is_component_instance_child: false,
            horizontal_constraint: DesignConstraint::Left,
            vertical_constraint: DesignConstraint::Top,
            layout: kind.supports_dimensions().then(DesignLayout::default),
            opacity: 100.,
            blend_mode: if kind.supports_pass_through_blend() {
                DesignBlendMode::PassThrough
            } else {
                DesignBlendMode::Normal
            },
            corner_radii: [0.; 4],
            independent_corners: false,
            corner_smoothing: 0.,
            corner_capabilities: DesignCornerCapabilities::for_node_kind(kind),
            fill_style_binding: None,
            fills: if kind.supports_fill() {
                vec![DesignPaint::solid(DesignColor::WHITE)]
            } else {
                Vec::new()
            },
            fill_shows_in_exports: (kind == DesignPanelNodeKind::Frame).then_some(true),
            stroke: None,
            stroke_style_binding: None,
            effects: Vec::new(),
            effect_capabilities: DesignEffectCapabilities::for_node_kind(kind),
            effect_style_binding: None,
            layout_grids: Vec::new(),
            layout_grid_style_binding: None,
            export_settings: Vec::new(),
            typography: matches!(
                kind,
                DesignPanelNodeKind::Text | DesignPanelNodeKind::TextPath
            )
            .then(DesignTypography::default),
            text_path: (kind == DesignPanelNodeKind::TextPath)
                .then_some(DesignTextPathViewData::default()),
            text_path_start_data: (kind == DesignPanelNodeKind::TextPath)
                .then_some(DesignTextPathStartData::DEFAULT),
            vector_edit: None,
            component_context: None,
            component_properties: Vec::new(),
            media: None,
            shape_geometry: DesignShapeGeometry::for_node_kind(kind),
            section: (kind == DesignPanelNodeKind::Section).then(DesignSectionProperties::default),
            transform_modifiers: if kind == DesignPanelNodeKind::TransformGroup {
                vec![DesignRepeatModifier::linear(
                    "repeat-0",
                    DesignRepeatAxis::Horizontal,
                )]
            } else {
                Vec::new()
            },
            is_mask: kind == DesignPanelNodeKind::Mask,
            mask_mode: DesignMaskType::Alpha,
            mask_type: (kind == DesignPanelNodeKind::Mask).then(|| "Alpha".into()),
            selection_color_aggregate: DesignSelectionColors::default(),
            selection_colors: Vec::new(),
        };
        node.apply_kind_preset();
        node.assign_preset_paint_ids();
        node
    }

    fn assign_preset_paint_ids(&mut self) {
        let node_id = self.id.clone();
        let assign = |paints: &mut [DesignPaint], collection: &str| {
            for (index, paint) in paints.iter_mut().enumerate() {
                if paint.id.is_empty() {
                    paint.id = format!("{node_id}-{collection}-{index}").into();
                }
                if let DesignPaintPayload::Gradient(gradient) = &mut paint.payload {
                    for (stop_index, stop) in gradient.stops.iter_mut().enumerate() {
                        if stop.id.is_empty() {
                            stop.id = format!("{}-stop-{stop_index}", paint.id).into();
                        }
                    }
                    paint.sync_legacy_projection();
                }
            }
        };
        assign(&mut self.fills, "fill");
        assign(&mut self.selection_colors, "selection-color");
        if let Some(stroke) = self.stroke.as_mut() {
            assign(&mut stroke.paints, "stroke");
        }
    }

    pub fn effect_count(&self, kind: DesignEffectKind) -> usize {
        self.effects
            .iter()
            .filter(|effect| effect.settings.kind() == kind)
            .count()
    }

    /// Whether changing/adding one row to `kind` is valid for the exact
    /// host-authored snapshot. `replacing_index` excludes the current row from
    /// the count when the user changes its kind.
    pub fn can_use_effect_kind(
        &self,
        kind: DesignEffectKind,
        replacing_index: Option<usize>,
    ) -> bool {
        if kind == DesignEffectKind::Unsupported
            || !self.effect_capabilities.kind_is_available(kind)
        {
            return false;
        }
        let replacing_same_kind = replacing_index
            .and_then(|index| self.effects.get(index))
            .is_some_and(|effect| effect.settings.kind() == kind);
        let count = self
            .effect_count(kind)
            .saturating_sub(usize::from(replacing_same_kind));
        count < usize::from(kind.maximum_per_node())
    }

    pub fn first_addable_effect_kind(&self) -> Option<DesignEffectKind> {
        DesignEffectKind::ALL
            .into_iter()
            .find(|kind| self.can_use_effect_kind(*kind, None))
    }

    pub fn effect_index_by_id(&self, effect_id: &str) -> Option<usize> {
        (!effect_id.is_empty()).then_some(())?;
        self.effects
            .iter()
            .position(|effect| effect.id.as_ref() == effect_id)
    }

    fn apply_kind_preset(&mut self) {
        match self.kind {
            DesignPanelNodeKind::Frame => {
                self.layout_grids.push(DesignLayoutGrid::columns(
                    DesignColumnLayoutGrid::default(),
                    DesignColor::rgb(0xf2, 0x48, 0x22),
                ));
            }
            DesignPanelNodeKind::Component => {
                let arrow = DesignComponentReference::local("icon-arrow-right", "Arrow right");
                let plus = DesignComponentReference::local("icon-plus", "Plus");
                let avatar = DesignComponentReference::remote(
                    "avatar-user",
                    "User avatar",
                    "Product foundations",
                );
                self.component_context = Some(DesignComponentContext {
                    role: DesignComponentRole::StandaloneMain,
                    main_component: Some(DesignComponentReference::local(
                        self.id.clone(),
                        self.name.clone(),
                    )),
                    description: Some(
                        "Primary action component with label, icon, and content-slot APIs.".into(),
                    ),
                    documentation_links: vec![DesignDocumentationLink::new(
                        "Button usage",
                        "https://example.com/components/button",
                    )],
                    overrides: DesignComponentOverrideSummary::default(),
                    authoring: None,
                });
                self.component_properties = vec![
                    DesignComponentProperty::text("label", "Label", "Button", "Button")
                        .with_multiline(true),
                    DesignComponentProperty::boolean("show-icon", "Show icon", true, true),
                    DesignComponentProperty::instance_swap(
                        "leading-icon",
                        "Leading icon",
                        Some(arrow.clone()),
                        Some(arrow.clone()),
                        vec![arrow, plus],
                    ),
                    DesignComponentProperty::slot(
                        "content",
                        "Content",
                        DesignSlotValue::default(),
                        DesignSlotValue::default(),
                        DesignSlotSettings {
                            preferred_values: vec![avatar],
                            ..DesignSlotSettings::default()
                        },
                        DesignSlotState::default(),
                    )
                    .with_description("Optional content inserted into the button"),
                ];
            }
            DesignPanelNodeKind::ComponentSet => {
                self.component_context = Some(DesignComponentContext {
                    role: DesignComponentRole::ComponentSet,
                    main_component: Some(DesignComponentReference::local(
                        self.id.clone(),
                        self.name.clone(),
                    )),
                    description: Some("Interactive button variants.".into()),
                    documentation_links: Vec::new(),
                    overrides: DesignComponentOverrideSummary::default(),
                    authoring: None,
                });
                self.component_properties = vec![
                    DesignComponentProperty::variant(
                        "state",
                        "State",
                        "Default",
                        "Default",
                        vec![
                            "Default".into(),
                            "Hover".into(),
                            "Pressed".into(),
                            "Disabled".into(),
                        ],
                    ),
                    DesignComponentProperty::variant(
                        "size",
                        "Size",
                        "Medium",
                        "Medium",
                        vec!["Small".into(), "Medium".into(), "Large".into()],
                    ),
                ];
            }
            DesignPanelNodeKind::Instance => {
                let arrow = DesignComponentReference::local("icon-arrow-right", "Arrow right");
                let plus = DesignComponentReference::local("icon-plus", "Plus");
                let check =
                    DesignComponentReference::remote("icon-check", "Check", "Product foundations");
                self.component_context = Some(DesignComponentContext {
                    role: DesignComponentRole::Instance,
                    main_component: Some(DesignComponentReference::remote(
                        "primary-button",
                        "Primary button",
                        "Product foundations",
                    )),
                    description: Some("Instance properties and nested overrides.".into()),
                    documentation_links: Vec::new(),
                    overrides: DesignComponentOverrideSummary {
                        overridden_property_count: 2,
                        nested_override_count: 1,
                        reset_state: DesignComponentResetState::Resettable,
                    },
                    authoring: None,
                });
                self.component_properties = vec![
                    DesignComponentProperty::variant(
                        "state",
                        "State",
                        "Default",
                        "Default",
                        vec!["Default".into(), "Hover".into(), "Pressed".into()],
                    ),
                    DesignComponentProperty::text("label", "Label", "Button", "Continue")
                        .with_reset_state(DesignComponentResetState::Resettable),
                    DesignComponentProperty::boolean("show-icon", "Show icon", true, true),
                    DesignComponentProperty::instance_swap(
                        "leading-icon",
                        "Leading icon",
                        Some(arrow.clone()),
                        Some(arrow.clone()),
                        vec![arrow, plus, check],
                    )
                    .with_reset_state(DesignComponentResetState::Resettable),
                ];
            }
            DesignPanelNodeKind::Text | DesignPanelNodeKind::TextPath => {
                self.width = 280.;
                self.height = 48.;
                self.fills = vec![DesignPaint::solid(DesignColor::BLACK)];
            }
            DesignPanelNodeKind::TransformGroup => {
                self.width = 280.;
                self.height = 180.;
                self.rotation = 8.;
            }
            DesignPanelNodeKind::Slot => {
                self.width = 240.;
                self.height = 120.;
                self.corner_radii = [8.; 4];
                let card = DesignComponentReference::local("content-card", "Content card");
                let empty = DesignComponentReference::local("empty-state", "Empty state");
                self.component_context = Some(DesignComponentContext {
                    role: DesignComponentRole::SlotDefinition,
                    main_component: Some(DesignComponentReference::local(
                        self.id.clone(),
                        self.name.clone(),
                    )),
                    description: Some(
                        "Defines which component instances can be inserted into this slot.".into(),
                    ),
                    documentation_links: Vec::new(),
                    overrides: DesignComponentOverrideSummary::default(),
                    authoring: None,
                });
                self.component_properties = vec![DesignComponentProperty::slot(
                    "content",
                    "Content",
                    DesignSlotValue::default(),
                    DesignSlotValue::default(),
                    DesignSlotSettings {
                        stretch_child_on_insert: true,
                        display_empty: true,
                        minimum_children: None,
                        maximum_children: Some(4),
                        preferred_values_only: false,
                        preferred_values: vec![card, empty],
                    },
                    DesignSlotState::default(),
                )];
            }
            DesignPanelNodeKind::Image | DesignPanelNodeKind::Video => {
                let media_paint = if self.kind == DesignPanelNodeKind::Image {
                    DesignPaint::image(DesignPaintSource::new("preset-image", "Image source"))
                } else {
                    DesignPaint::video(DesignPaintSource::new("preset-video", "Video source"))
                };
                self.fills = vec![media_paint.with_id("media-fill")];
                self.corner_radii = [12.; 4];
            }
            DesignPanelNodeKind::Rectangle => {
                self.corner_radii = [12.; 4];
                self.fills = vec![DesignPaint::gradient(
                    DesignPaintKind::LinearGradient,
                    vec![
                        DesignGradientStop::new(0., DesignColor::PURPLE),
                        DesignGradientStop::new(1., DesignColor::BLUE),
                    ],
                )];
            }
            DesignPanelNodeKind::Line | DesignPanelNodeKind::Arrow => {
                self.width = 240.;
                self.height = 0.;
                let mut stroke = DesignStroke::for_node(
                    self.kind,
                    DesignPaint::solid(DesignColor::BLACK),
                    2.,
                    DesignStrokeAlign::Inside,
                );
                stroke.end_cap = if self.kind == DesignPanelNodeKind::Arrow {
                    DesignStrokeCap::LineArrow
                } else {
                    DesignStrokeCap::None
                };
                self.stroke = Some(stroke);
            }
            DesignPanelNodeKind::Vector
            | DesignPanelNodeKind::BooleanOperation
            | DesignPanelNodeKind::Pen
            | DesignPanelNodeKind::Pencil => {
                self.fills = vec![DesignPaint::solid(DesignColor::PURPLE)];
                let mut stroke = DesignStroke::for_node(
                    self.kind,
                    DesignPaint::solid(DesignColor::BLACK),
                    1.,
                    DesignStrokeAlign::Center,
                );
                stroke.start_cap = DesignStrokeCap::Round;
                stroke.end_cap = DesignStrokeCap::Round;
                stroke.endpoint_cap = DesignStrokeCap::Round;
                self.stroke = Some(stroke);
            }
            DesignPanelNodeKind::MultipleSelection => {
                self.name = "3 layers selected".into();
                self.selection_color_aggregate = DesignSelectionColors::new([
                    DesignSelectionColor::new("purple", DesignColor::PURPLE, [])
                        .with_occurrence_count(1),
                    DesignSelectionColor::new("blue", DesignColor::BLUE, [])
                        .with_occurrence_count(1),
                    DesignSelectionColor::new("white", DesignColor::WHITE, [])
                        .with_occurrence_count(1),
                ]);
                self.selection_colors = vec![
                    DesignPaint::solid(DesignColor::PURPLE),
                    DesignPaint::solid(DesignColor::BLUE),
                    DesignPaint::solid(DesignColor::WHITE),
                ];
                self.fills.clear();
            }
            _ => {}
        }
    }

    /// Selection-color rows in canonical form.
    ///
    /// The legacy fake-paint projection is adapted only when the canonical
    /// aggregate is empty, allowing old hosts to migrate without teaching the
    /// panel to treat aggregate colors as normal fills.
    pub fn resolved_selection_colors(&self) -> Vec<DesignSelectionColor> {
        if !self.selection_color_aggregate.colors.is_empty() {
            return self.selection_color_aggregate.colors.clone();
        }
        self.selection_colors
            .iter()
            .enumerate()
            .map(|(index, paint)| {
                let binding = match &paint.payload {
                    DesignPaintPayload::Solid(solid) => solid.binding.clone(),
                    DesignPaintPayload::Gradient(gradient) => {
                        gradient.stops.first().and_then(|stop| stop.binding.clone())
                    }
                    DesignPaintPayload::Pattern(_)
                    | DesignPaintPayload::Image(_)
                    | DesignPaintPayload::Video(_)
                    | DesignPaintPayload::Shader(_)
                    | DesignPaintPayload::Unsupported(_) => None,
                };
                let mut color = DesignSelectionColor::new(
                    if paint.id.is_empty() {
                        format!("{}-legacy-selection-color-{index}", self.id).into()
                    } else {
                        paint.id.clone()
                    },
                    paint.color,
                    [],
                )
                .with_paint(paint.clone())
                .read_only(paint.read_only);
                color.binding = binding;
                color
            })
            .collect()
    }

    /// Canonical mask state with a compatibility fallback for hosts still
    /// populating the original string-only `mask_type` field.
    pub fn effective_is_mask(&self) -> bool {
        self.is_mask
            || self
                .mask_type
                .as_deref()
                .and_then(|label| DesignMaskType::parse_compatibility_label(label.as_ref()))
                .is_some()
    }

    pub fn effective_mask_type(&self) -> Option<DesignMaskType> {
        if self.is_mask {
            Some(self.mask_mode)
        } else {
            self.mask_type
                .as_deref()
                .and_then(|label| DesignMaskType::parse_compatibility_label(label.as_ref()))
        }
    }

    pub fn with_capabilities(mut self, capabilities: DesignPanelNodeCapabilities) -> Self {
        self.capabilities = Some(capabilities);
        self
    }

    pub const fn with_component_instance_child(mut self, is_child: bool) -> Self {
        self.is_component_instance_child = is_child;
        self
    }

    pub fn supports_dimensions(&self) -> bool {
        self.capabilities
            .as_ref()
            .map_or_else(|| self.kind.supports_dimensions(), |value| value.dimensions)
    }

    pub fn supports_visibility(&self) -> bool {
        self.capabilities
            .as_ref()
            .map_or_else(|| self.kind.supports_visibility(), |value| value.visibility)
    }

    pub fn supports_position_coordinates(&self) -> bool {
        self.capabilities.as_ref().map_or_else(
            || self.kind.supports_position_coordinates(),
            |value| value.position_coordinates,
        )
    }

    pub fn supports_arrange(&self) -> bool {
        self.capabilities
            .as_ref()
            .map_or_else(|| self.kind.supports_arrange(), |value| value.arrange)
    }

    pub fn supports_transforms(&self) -> bool {
        self.capabilities
            .as_ref()
            .map_or_else(|| self.kind.supports_transforms(), |value| value.transforms)
    }

    pub fn supports_aspect_ratio_lock(&self) -> bool {
        self.capabilities.as_ref().map_or_else(
            || self.kind.supports_aspect_ratio_lock(),
            |value| value.aspect_ratio_lock,
        )
    }

    pub fn supports_auto_layout_child(&self) -> bool {
        self.capabilities.as_ref().map_or_else(
            || self.kind.supports_auto_layout_child(),
            |value| value.auto_layout_child,
        )
    }

    pub fn supports_add_auto_layout(&self) -> bool {
        self.capabilities.as_ref().map_or_else(
            || self.kind.supports_add_auto_layout(),
            |value| value.add_auto_layout,
        )
    }

    pub fn supports_auto_layout_container(&self) -> bool {
        self.capabilities.as_ref().map_or_else(
            || self.kind.supports_auto_layout_container(),
            |value| value.auto_layout_container,
        )
    }

    /// Whether the current host snapshot can expose a numeric Max lines value.
    ///
    /// Figma additionally requires a text layer inside auto layout to use
    /// vertical Hug sizing. `inside_auto_layout` comes from the host's exact
    /// parent-layout context rather than being inferred from this node.
    pub fn text_max_lines_are_available(&self, inside_auto_layout: bool) -> bool {
        self.typography
            .as_ref()
            .is_some_and(DesignTypography::text_max_lines_are_available)
            && (!inside_auto_layout
                || self
                    .layout
                    .as_ref()
                    .is_some_and(|layout| layout.vertical_sizing == DesignSizingMode::Hug))
    }

    /// Applies a host-accepted resize value and restores the nullable Max
    /// lines value to Auto when the new mode cannot support a line limit.
    pub fn set_text_resize(&mut self, resize: DesignTextResize) -> bool {
        let Some(typography) = self.typography.as_mut() else {
            return false;
        };
        typography.resize = resize;
        if resize == DesignTextResize::Fixed {
            typography.max_lines = None;
        }
        true
    }

    /// Applies host-accepted truncation and restores Max lines to Auto when
    /// ending truncation is disabled.
    pub fn set_text_truncation(&mut self, truncate: bool) -> bool {
        let Some(typography) = self.typography.as_mut() else {
            return false;
        };
        typography.truncate = truncate;
        if !truncate {
            typography.max_lines = None;
        }
        true
    }

    /// Applies Figma's node-level nullable `maxLines` contract.
    ///
    /// A numeric line limit removes Max height atomically. `None` is the
    /// native Auto value and leaves an independently supplied Max height
    /// untouched.
    pub fn set_text_max_lines(&mut self, max_lines: Option<u32>, inside_auto_layout: bool) -> bool {
        if max_lines.is_some_and(|lines| lines == 0)
            || (max_lines.is_some() && !self.text_max_lines_are_available(inside_auto_layout))
        {
            return false;
        }
        let Some(typography) = self.typography.as_mut() else {
            return false;
        };
        typography.max_lines = max_lines;
        if max_lines.is_some()
            && let Some(layout) = self.layout.as_mut()
        {
            layout.item.max_height = None;
        }
        true
    }

    /// Applies a nullable Max height and atomically resets numeric Max lines
    /// to Auto when Figma's mutually-exclusive height limit becomes active.
    pub fn set_layout_max_height(&mut self, max_height: Option<f32>) -> bool {
        if max_height.is_some_and(|height| !height.is_finite() || height <= 0.) {
            return false;
        }
        let Some(layout) = self.layout.as_mut() else {
            return false;
        };
        layout.item.max_height = max_height;
        if max_height.is_some()
            && let Some(typography) = self.typography.as_mut()
        {
            typography.max_lines = None;
        }
        true
    }

    /// Revalidates a previously supplied numeric Max lines value after a
    /// resize or parent-layout change. Invalid values become native Auto.
    pub fn normalize_text_max_lines(&mut self, inside_auto_layout: bool) {
        if !self.text_max_lines_are_available(inside_auto_layout)
            && let Some(typography) = self.typography.as_mut()
        {
            typography.max_lines = None;
        }
    }

    pub fn supports_grid_auto_layout(&self) -> bool {
        self.capabilities.as_ref().map_or_else(
            || self.kind.supports_grid_auto_layout(),
            |value| value.grid_auto_layout,
        )
    }

    pub fn supports_resize_to_fit(&self) -> bool {
        self.capabilities.as_ref().map_or_else(
            || self.kind.supports_resize_to_fit(),
            |value| value.resize_to_fit,
        )
    }

    pub fn supports_clip_content(&self) -> bool {
        self.capabilities.as_ref().map_or_else(
            || self.kind.supports_clip_content(),
            |value| value.clip_content,
        )
    }

    pub fn supports_fill(&self) -> bool {
        self.capabilities
            .as_ref()
            .map_or_else(|| self.kind.supports_fill(), |value| value.fill)
    }

    pub fn supports_stroke(&self) -> bool {
        self.capabilities
            .as_ref()
            .map_or_else(|| self.kind.supports_stroke(), |value| value.stroke)
    }

    pub fn supports_layer_appearance(&self) -> bool {
        self.capabilities.as_ref().map_or_else(
            || self.kind.supports_layer_appearance(),
            |value| value.layer_appearance,
        )
    }

    pub fn supports_pass_through_blend(&self) -> bool {
        self.capabilities.as_ref().map_or_else(
            || self.kind.supports_pass_through_blend(),
            |value| value.pass_through_blend,
        )
    }

    pub fn supports_effects(&self) -> bool {
        self.capabilities
            .as_ref()
            .map_or_else(|| self.kind.supports_effects(), |value| value.effects)
    }

    pub fn supports_constraints(&self) -> bool {
        self.capabilities.as_ref().map_or_else(
            || self.kind.supports_constraints(),
            |value| value.constraints,
        )
    }

    pub fn supports_layout_guides(&self) -> bool {
        self.capabilities.as_ref().map_or_else(
            || {
                matches!(
                    self.kind,
                    DesignPanelNodeKind::Frame
                        | DesignPanelNodeKind::Component
                        | DesignPanelNodeKind::ComponentSet
                        | DesignPanelNodeKind::Instance
                        | DesignPanelNodeKind::Slot
                )
            },
            |value| value.layout_guides,
        )
    }

    /// Whether an exact known section belongs to this node's inspector.
    ///
    /// An explicit capability snapshot preserves the host's order and
    /// visibility. The semantic checks keep contradictory snapshots from
    /// exposing controls whose intents the same snapshot forbids.
    pub fn supports_section(&self, section: DesignPanelSection) -> bool {
        if let Some(capabilities) = &self.capabilities {
            return capabilities.sections.contains(&section)
                && match section {
                    DesignPanelSection::Layout => capabilities.dimensions,
                    DesignPanelSection::Layer => {
                        capabilities.visibility
                            || capabilities.layer_appearance
                            || self.corner_capabilities.has_any()
                            || self.shape_geometry.has_appearance_controls()
                    }
                    DesignPanelSection::Fill => capabilities.fill,
                    DesignPanelSection::Stroke => capabilities.stroke,
                    DesignPanelSection::Effects => capabilities.effects,
                    DesignPanelSection::LayoutGrid => capabilities.layout_guides,
                    DesignPanelSection::Constraints => capabilities.constraints,
                    DesignPanelSection::Selection
                    | DesignPanelSection::Component
                    | DesignPanelSection::Instance
                    | DesignPanelSection::Position
                    | DesignPanelSection::Section
                    | DesignPanelSection::Transform
                    | DesignPanelSection::Geometry
                    | DesignPanelSection::Typography
                    | DesignPanelSection::Media
                    | DesignPanelSection::Export => true,
                    DesignPanelSection::Mask => self.effective_is_mask(),
                };
        }

        match section {
            DesignPanelSection::Selection => {
                !self.selection_color_aggregate.colors.is_empty()
                    || !self.selection_colors.is_empty()
            }
            DesignPanelSection::Component => self
                .component_role()
                .is_some_and(|role| !role.uses_instance_section()),
            DesignPanelSection::Instance => self
                .component_role()
                .is_some_and(DesignComponentRole::uses_instance_section),
            DesignPanelSection::Position => true,
            DesignPanelSection::Layout => self.supports_dimensions(),
            DesignPanelSection::Constraints => self.supports_constraints(),
            DesignPanelSection::Layer => {
                self.kind != DesignPanelNodeKind::Slice
                    && (self.supports_visibility()
                        || self.supports_layer_appearance()
                        || self.corner_capabilities.has_any()
                        || self.shape_geometry.has_appearance_controls())
            }
            DesignPanelSection::Section => self.section.is_some(),
            DesignPanelSection::Transform => self.kind == DesignPanelNodeKind::TransformGroup,
            DesignPanelSection::Geometry => {
                matches!(self.shape_geometry, DesignShapeGeometry::Table(_))
            }
            DesignPanelSection::Mask => self.effective_is_mask(),
            DesignPanelSection::Typography => self.typography.is_some(),
            DesignPanelSection::Media => false,
            DesignPanelSection::Fill => self.supports_fill(),
            DesignPanelSection::Stroke => self.supports_stroke(),
            DesignPanelSection::Effects => self.supports_effects(),
            DesignPanelSection::LayoutGrid => self.supports_layout_guides(),
            DesignPanelSection::Export => true,
        }
    }

    pub fn with_layout_mode(mut self, mode: DesignLayoutMode) -> Self {
        let supports_auto_layout = self.supports_auto_layout_container();
        let supports_grid = self.supports_grid_auto_layout();
        if let Some(layout) = self.layout.as_mut()
            && layout.set_mode_for_capabilities(supports_auto_layout, supports_grid, mode)
            && mode != DesignLayoutMode::None
        {
            layout.horizontal_sizing = DesignSizingMode::Hug;
            layout.vertical_sizing = DesignSizingMode::Hug;
        }
        self
    }

    pub fn with_corner_capabilities(mut self, capabilities: DesignCornerCapabilities) -> Self {
        self.corner_capabilities = capabilities;
        self
    }

    pub fn with_shape_geometry(mut self, geometry: DesignShapeGeometry) -> Self {
        self.shape_geometry = geometry;
        self
    }

    pub fn with_component_context(mut self, context: DesignComponentContext) -> Self {
        self.component_context = Some(context);
        self
    }

    /// Returns explicit host context first, then a compatibility inference for
    /// the original node-kind-only adapter.
    pub fn component_role(&self) -> Option<DesignComponentRole> {
        self.component_context
            .as_ref()
            .map(|context| context.role)
            .or(match self.kind {
                DesignPanelNodeKind::Component => Some(DesignComponentRole::StandaloneMain),
                DesignPanelNodeKind::ComponentSet => Some(DesignComponentRole::ComponentSet),
                DesignPanelNodeKind::Instance => Some(DesignComponentRole::Instance),
                DesignPanelNodeKind::Slot => Some(DesignComponentRole::SlotDefinition),
                _ if !self.component_properties.is_empty() => {
                    Some(DesignComponentRole::StandaloneMain)
                }
                _ => None,
            })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignPanelSection {
    Selection,
    Component,
    Instance,
    Position,
    Layout,
    Constraints,
    Layer,
    Section,
    Transform,
    /// Compatibility-only shape section. Canonical shape controls compose
    /// inside [`Self::Layer`] (Appearance); FigJam Table and exact legacy host
    /// snapshots may still request this identifier.
    Geometry,
    Mask,
    Typography,
    /// Compatibility-only section; media controls now belong to Fill paints.
    Media,
    Fill,
    Stroke,
    Effects,
    LayoutGrid,
    Export,
}

impl DesignPanelSection {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Selection => "Selection colors",
            Self::Component => "Component properties",
            Self::Instance => "Instance",
            Self::Position => "Position",
            Self::Layout => "Layout",
            Self::Constraints => "Constraints",
            Self::Layer => "Appearance",
            Self::Section => "Section",
            Self::Transform => "Transform",
            Self::Geometry => "Geometry",
            Self::Mask => "Mask",
            Self::Typography => "Typography",
            Self::Media => "Media",
            Self::Fill => "Fill",
            Self::Stroke => "Stroke",
            Self::Effects => "Effects",
            Self::LayoutGrid => "Layout guides",
            Self::Export => "Export",
        }
    }
}

/// Host-authored inspector capabilities for one exact scene node.
///
/// `sections` is the authoritative ordered set of known Design-panel sections.
/// The remaining fields gate the semantics inside those sections and the
/// corresponding intents. This separation is intentional: for example,
/// Section nodes can expose corner controls in `Layer` without implementing
/// layer opacity/blend/effects, while a future host node may expose layout
/// guides without owning auto layout.
///
/// A host can start from [`Self::for_node_kind`] and replace `sections` or
/// individual semantic flags. [`DesignPanelNode`] uses kind-derived behavior
/// only when its optional capability snapshot is absent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignPanelNodeCapabilities {
    pub sections: Vec<DesignPanelSection>,
    pub dimensions: bool,
    pub visibility: bool,
    pub position_coordinates: bool,
    pub arrange: bool,
    pub transforms: bool,
    pub aspect_ratio_lock: bool,
    pub auto_layout_child: bool,
    pub add_auto_layout: bool,
    pub auto_layout_container: bool,
    pub grid_auto_layout: bool,
    pub resize_to_fit: bool,
    pub clip_content: bool,
    pub fill: bool,
    pub stroke: bool,
    pub layer_appearance: bool,
    pub pass_through_blend: bool,
    pub effects: bool,
    pub constraints: bool,
    pub layout_guides: bool,
}

impl DesignPanelNodeCapabilities {
    /// Compatibility capability snapshot for a coarse node kind.
    ///
    /// Type-specific sections reflect the kind's representative preset.
    /// Hosts that attach additional controlled type data should replace the
    /// ordered `sections` list explicitly.
    pub fn for_node_kind(kind: DesignPanelNodeKind) -> Self {
        let dimensions = kind.supports_dimensions();
        let visibility = kind.supports_visibility();
        let fill = kind.supports_fill();
        let stroke = kind.supports_stroke();
        let layer_appearance = kind.supports_layer_appearance();
        let effects = kind.supports_effects();
        let layout_guides = matches!(
            kind,
            DesignPanelNodeKind::Frame
                | DesignPanelNodeKind::Component
                | DesignPanelNodeKind::ComponentSet
                | DesignPanelNodeKind::Instance
                | DesignPanelNodeKind::Slot
        );
        let mut sections = vec![DesignPanelSection::Position];
        if dimensions {
            sections.push(DesignPanelSection::Layout);
        }
        if kind != DesignPanelNodeKind::Slice
            && (visibility
                || layer_appearance
                || DesignCornerCapabilities::for_node_kind(kind).has_any()
                || DesignShapeGeometry::for_node_kind(kind).has_appearance_controls())
        {
            sections.push(DesignPanelSection::Layer);
        }
        if kind == DesignPanelNodeKind::MultipleSelection {
            sections.push(DesignPanelSection::Selection);
        }
        match kind {
            DesignPanelNodeKind::Component | DesignPanelNodeKind::ComponentSet => {
                sections.push(DesignPanelSection::Component);
            }
            DesignPanelNodeKind::Instance => sections.push(DesignPanelSection::Instance),
            DesignPanelNodeKind::Slot => sections.push(DesignPanelSection::Component),
            _ => {}
        }
        if kind == DesignPanelNodeKind::Section {
            sections.push(DesignPanelSection::Section);
        }
        if kind == DesignPanelNodeKind::TransformGroup {
            sections.push(DesignPanelSection::Transform);
        }
        if kind == DesignPanelNodeKind::Table {
            sections.push(DesignPanelSection::Geometry);
        }
        if kind == DesignPanelNodeKind::Mask {
            sections.push(DesignPanelSection::Mask);
        }
        if matches!(
            kind,
            DesignPanelNodeKind::Text | DesignPanelNodeKind::TextPath
        ) {
            sections.push(DesignPanelSection::Typography);
        }
        if fill {
            sections.push(DesignPanelSection::Fill);
        }
        if stroke {
            sections.push(DesignPanelSection::Stroke);
        }
        if effects {
            sections.push(DesignPanelSection::Effects);
        }
        if layout_guides {
            sections.push(DesignPanelSection::LayoutGrid);
        }
        sections.push(DesignPanelSection::Export);

        Self {
            sections,
            dimensions,
            visibility,
            position_coordinates: kind.supports_position_coordinates(),
            arrange: kind.supports_arrange(),
            transforms: kind.supports_transforms(),
            aspect_ratio_lock: kind.supports_aspect_ratio_lock(),
            auto_layout_child: kind.supports_auto_layout_child(),
            add_auto_layout: kind.supports_add_auto_layout(),
            auto_layout_container: kind.supports_auto_layout_container(),
            grid_auto_layout: kind.supports_grid_auto_layout(),
            resize_to_fit: kind.supports_resize_to_fit(),
            clip_content: kind.supports_clip_content(),
            fill,
            stroke,
            layer_appearance,
            pass_through_blend: kind.supports_pass_through_blend(),
            effects,
            constraints: kind.supports_constraints(),
            layout_guides,
        }
    }

    pub fn with_sections(mut self, sections: impl IntoIterator<Item = DesignPanelSection>) -> Self {
        self.sections = sections.into_iter().collect();
        self
    }

    pub const fn with_dimensions(mut self, dimensions: bool) -> Self {
        self.dimensions = dimensions;
        self
    }

    pub const fn with_visibility(mut self, supported: bool) -> Self {
        self.visibility = supported;
        self
    }

    pub const fn with_position_coordinates(mut self, supported: bool) -> Self {
        self.position_coordinates = supported;
        self
    }

    pub const fn with_arrange(mut self, supported: bool) -> Self {
        self.arrange = supported;
        self
    }

    pub const fn with_transforms(mut self, supported: bool) -> Self {
        self.transforms = supported;
        self
    }

    pub const fn with_aspect_ratio_lock(mut self, supported: bool) -> Self {
        self.aspect_ratio_lock = supported;
        self
    }

    pub const fn with_auto_layout_child(mut self, supported: bool) -> Self {
        self.auto_layout_child = supported;
        self
    }

    pub const fn with_add_auto_layout(mut self, supported: bool) -> Self {
        self.add_auto_layout = supported;
        self
    }

    pub const fn with_auto_layout_container(mut self, supported: bool) -> Self {
        self.auto_layout_container = supported;
        self
    }

    pub const fn with_grid_auto_layout(mut self, supported: bool) -> Self {
        self.grid_auto_layout = supported;
        self
    }

    pub const fn with_resize_to_fit(mut self, supported: bool) -> Self {
        self.resize_to_fit = supported;
        self
    }

    pub const fn with_clip_content(mut self, supported: bool) -> Self {
        self.clip_content = supported;
        self
    }

    pub const fn with_fill(mut self, supported: bool) -> Self {
        self.fill = supported;
        self
    }

    pub const fn with_stroke(mut self, supported: bool) -> Self {
        self.stroke = supported;
        self
    }

    pub const fn with_layer_appearance(mut self, supported: bool) -> Self {
        self.layer_appearance = supported;
        self
    }

    pub const fn with_pass_through_blend(mut self, supported: bool) -> Self {
        self.pass_through_blend = supported;
        self
    }

    pub const fn with_effects(mut self, supported: bool) -> Self {
        self.effects = supported;
        self
    }

    pub const fn with_constraints(mut self, supported: bool) -> Self {
        self.constraints = supported;
        self
    }

    pub const fn with_layout_guides(mut self, supported: bool) -> Self {
        self.layout_guides = supported;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignPanelCollection {
    Fill,
    Stroke,
    Effect,
    LayoutGrid,
    Export,
}

impl DesignPanelCollection {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Fill => "Fill",
            Self::Stroke => "Stroke",
            Self::Effect => "Effect",
            Self::LayoutGrid => "Layout guide",
            Self::Export => "Export",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignPanelProperty {
    Visible,
    X,
    Y,
    /// Horizontal Space between for an exact multiple-selection snapshot.
    SmartSelectionHorizontalSpacing,
    /// Vertical Space between for an exact multiple-selection snapshot.
    SmartSelectionVerticalSpacing,
    /// X coordinate for the exact host-selected vector vertices.
    VectorVertexX,
    /// Y coordinate for the exact host-selected vector vertices.
    VectorVertexY,
    /// Per-vertex radius for the exact host-selected vector vertices.
    VectorVertexCornerRadius,
    /// Tangent mirroring for the exact host-selected vector vertices.
    VectorHandleMirroring,
    Width,
    Height,
    Rotation,
    LockAspectRatio,
    HorizontalConstraint,
    VerticalConstraint,
    LayoutMode,
    HorizontalSizing,
    VerticalSizing,
    AutoLayoutAlignment,
    /// Compatibility-only per-axis auto-layout field.
    #[deprecated(note = "use DesignPanelProperty::AutoLayoutAlignment")]
    AlignmentX,
    /// Compatibility-only per-axis auto-layout field.
    #[deprecated(note = "use DesignPanelProperty::AutoLayoutAlignment")]
    AlignmentY,
    Wrap,
    Gap,
    ItemSpacingMode,
    CounterAxisAlignContent,
    CounterAxisGap,
    PaddingVertical,
    PaddingHorizontal,
    PaddingShorthand,
    PaddingTop,
    PaddingRight,
    PaddingBottom,
    PaddingLeft,
    ClipContent,
    IncludeStrokes,
    StackingOrder,
    BaselineAlignment,
    MinWidth,
    MaxWidth,
    MinHeight,
    MaxHeight,
    LayoutPositioning,
    LayoutAlignSelf,
    LayoutGrow,
    GridAutoTracks,
    GridItemsPositioning,
    GridColumnCount,
    GridRowCount,
    GridColumnTrack(usize),
    GridRowTrack(usize),
    GridColumnTrackValue(usize),
    GridRowTrackValue(usize),
    GridRowIndex,
    GridColumnIndex,
    GridRowSpan,
    GridColumnSpan,
    GridHorizontalAlignment,
    GridVerticalAlignment,
    Opacity,
    BlendMode,
    CornerRadius,
    CornerRadiusTopLeft,
    CornerRadiusTopRight,
    CornerRadiusBottomRight,
    CornerRadiusBottomLeft,
    IndependentCorners,
    CornerSmoothing,
    FillShowsInExports,
    TypographyStyle,
    FontFamily,
    FontStyle,
    FontWeight,
    FontSize,
    LineHeight,
    LetterSpacing,
    TextLeadingTrim,
    ParagraphSpacing,
    ParagraphIndent,
    ListSpacing,
    TextHangingPunctuation,
    TextHangingLists,
    HorizontalTextAlignment,
    VerticalTextAlignment,
    TextResize,
    TextTruncate,
    TextMaxLines,
    TextDecoration,
    TextDecorationStyle,
    TextDecorationOffset,
    TextDecorationThickness,
    TextDecorationColor,
    TextDecorationSkipInk,
    TextCase,
    TextList,
    TextPathStartSegment,
    TextPathStartPosition,
    ComponentProperty(usize),
    SlotStretchChildOnInsert(usize),
    SlotDisplayEmpty(usize),
    SlotMinimumInstances(usize),
    SlotMaximumInstances(usize),
    SlotPreferredValuesOnly(usize),
    /// Compatibility-only node media property.
    #[deprecated(note = "use PaintEditRequested with a typed Image/Video paint edit")]
    MediaCropMode,
    /// Compatibility-only node media property.
    #[deprecated(note = "use PaintEditRequested with DesignPaintProperty::MediaFilter")]
    MediaExposure,
    /// Compatibility-only node media property.
    #[deprecated(note = "use PaintEditRequested with DesignPaintProperty::MediaFilter")]
    MediaContrast,
    /// Compatibility-only node media property.
    #[deprecated(note = "use PaintEditRequested with DesignPaintProperty::MediaFilter")]
    MediaSaturation,
    /// Compatibility-only node media property.
    #[deprecated(note = "use PaintEditRequested with DesignPaintProperty::MediaFilter")]
    MediaTemperature,
    /// Compatibility-only node media property.
    #[deprecated(note = "use PaintEditRequested with DesignPaintProperty::MediaFilter")]
    MediaTint,
    /// Compatibility-only node media property.
    #[deprecated(note = "use PaintEditRequested with DesignPaintProperty::MediaFilter")]
    MediaHighlights,
    /// Compatibility-only node media property.
    #[deprecated(note = "use PaintEditRequested with DesignPaintProperty::MediaFilter")]
    MediaShadows,
    PolygonCount,
    StarPointCount,
    StarInnerRadius,
    ArcStartingAngle,
    ArcSweep,
    /// Compatibility-only absolute `ArcData.endingAngle` editor.
    ArcEndingAngle,
    ArcInnerRadius,
    BooleanOperation,
    IsMask,
    MaskType,
    TableRows,
    TableColumns,
    SectionContentsHidden,
    SectionDevStatus,
    TransformRepeatType(usize),
    TransformRepeatAxis(usize),
    TransformRepeatCount(usize),
    TransformRepeatUnit(usize),
    TransformRepeatOffset(usize),
    /// Canonical aggregate color row; not a Fill collection index.
    SelectionColor(usize),
    PaintOpacity {
        collection: DesignPanelCollection,
        index: usize,
    },
    PaintVisible {
        collection: DesignPanelCollection,
        index: usize,
    },
    StrokeWeight,
    StrokeWeightMode,
    StrokeWeightTop,
    StrokeWeightRight,
    StrokeWeightBottom,
    StrokeWeightLeft,
    StrokeAlign,
    StrokeStartCap,
    StrokeEndCap,
    StrokeEndpointCap,
    StrokeDashMode,
    StrokeDashPattern,
    StrokeDashCap,
    StrokeJoin,
    StrokeMiterAngle,
    StrokeVariableWidth,
    StrokeVariableWidthPointPosition(usize),
    StrokeVariableWidthPointWidth(usize),
    StrokeType,
    StrokeStretchBrush,
    StrokeBrushDirection,
    StrokeScatterBrush,
    StrokeScatterGap,
    StrokeScatterWiggle,
    StrokeScatterSizeJitter,
    StrokeScatterAngularJitter,
    StrokeScatterRotation,
    StrokeDynamicFrequency,
    StrokeDynamicWiggle,
    StrokeDynamicSmoothen,
    EffectKind(usize),
    EffectVisible(usize),
    /// Compatibility-only aggregate effect payload.
    #[deprecated(note = "use the typed Effect* leaf properties with EffectEditRequested")]
    EffectSettings(usize),
    EffectShadowColor(usize),
    EffectShadowBlendMode(usize),
    EffectShadowBlur(usize),
    EffectShadowSpread(usize),
    EffectShadowOffsetX(usize),
    EffectShadowOffsetY(usize),
    EffectDropShadowShowBehindNode(usize),
    EffectBlurType(usize),
    EffectBlurRadius(usize),
    EffectProgressiveBlurStartRadius(usize),
    EffectProgressiveBlurEndRadius(usize),
    EffectProgressiveBlurStartOffsetX(usize),
    EffectProgressiveBlurStartOffsetY(usize),
    EffectProgressiveBlurEndOffsetX(usize),
    EffectProgressiveBlurEndOffsetY(usize),
    EffectNoiseType(usize),
    EffectNoisePrimaryColor(usize),
    EffectNoiseSecondaryColor(usize),
    EffectNoiseOpacity(usize),
    EffectNoiseBlendMode(usize),
    EffectNoiseSizeX(usize),
    EffectNoiseSizeY(usize),
    EffectNoiseDensity(usize),
    EffectTextureSizeX(usize),
    EffectTextureSizeY(usize),
    EffectTextureRadius(usize),
    EffectTextureClipToShape(usize),
    EffectGlassLightIntensity(usize),
    EffectGlassLightAngle(usize),
    EffectGlassRefraction(usize),
    EffectGlassDepth(usize),
    EffectGlassDispersion(usize),
    EffectGlassFrost(usize),
    EffectGlassSplay(usize),
    /// One Shader property-definition by stable row identity plus a
    /// compatibility property index. The typed edit intent also carries the
    /// definition ID so hosts never have to trust this index after an echo.
    EffectShaderProperty(usize, usize),
    /// Compatibility alias used by the original compact effects row.
    #[deprecated(note = "use EffectShadowBlur or EffectBlurRadius")]
    EffectBlur(usize),
    /// Compatibility alias used by the original compact effects row.
    #[deprecated(note = "use EffectShadowSpread")]
    EffectSpread(usize),
    /// Compatibility alias used by the original compact effects row.
    #[deprecated(note = "use EffectShadowOffsetX")]
    EffectOffsetX(usize),
    /// Compatibility alias used by the original compact effects row.
    #[deprecated(note = "use EffectShadowOffsetY")]
    EffectOffsetY(usize),
    LayoutGridKind(usize),
    LayoutGridVisible(usize),
    LayoutGridAlignment(usize),
    LayoutGridCount(usize),
    LayoutGridSize(usize),
    LayoutGridOffset(usize),
    LayoutGridGutter(usize),
    LayoutGridMargin(usize),
    LayoutGridColor(usize),
    LayoutGridOpacity(usize),
    ExportSizing(usize),
    /// Compatibility alias for scale-only export rows.
    #[deprecated(note = "use DesignPanelProperty::ExportSizing")]
    ExportScale(usize),
    ExportSuffix(usize),
    ExportFormat(usize),
    /// Compatibility-only auto-layout alignment-cell property.
    #[deprecated(note = "use DesignPanelProperty::AutoLayoutAlignment")]
    AlignSelection,
    /// Compatibility-only selection distribution property.
    #[deprecated(note = "use DesignPanelAction::ArrangeRequested")]
    DistributeSelection,
}

/// Compatibility-only Design-panel adapter paths and their canonical
/// replacements.
///
/// These paths remain in the public enums so older hosts continue to compile,
/// but the reusable panel and Storybook do not emit or exercise them as
/// canonical behavior.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignPanelCompatibilityPath {
    AutoLayoutAxisAlignment,
    AggregateEffectSettings,
    AutoLayoutAlignmentCell,
    SelectionDistribution,
    NodeMediaProperty,
    CompactEffectLeaf,
    ScaleOnlyExportProperty,
    IndexedLayoutGridPropertyAction,
    IndexedLayoutGridRemoveAction,
    CountOnlyLayoutGridVariableAction,
    WholePaintAction,
    ConflatedColorStyleAction,
    NodeMediaReplaceAction,
    SingleExportAction,
}

impl DesignPanelCompatibilityPath {
    /// The canonical host contract replacing this compatibility path.
    pub const fn replacement(self) -> &'static str {
        match self {
            Self::AutoLayoutAxisAlignment | Self::AutoLayoutAlignmentCell => {
                "DesignPanelProperty::AutoLayoutAlignment"
            }
            Self::AggregateEffectSettings => {
                "typed Effect* leaf properties through DesignPanelAction::EffectEditRequested"
            }
            Self::SelectionDistribution => "DesignPanelAction::ArrangeRequested",
            Self::NodeMediaProperty => {
                "DesignPanelAction::PaintEditRequested with a typed Image/Video paint edit"
            }
            Self::CompactEffectLeaf => {
                "EffectShadow* or EffectBlurRadius through DesignPanelAction::EffectEditRequested"
            }
            Self::ScaleOnlyExportProperty => "DesignPanelProperty::ExportSizing",
            Self::IndexedLayoutGridPropertyAction => {
                "DesignPanelAction::LayoutGridPropertyEditRequested"
            }
            Self::IndexedLayoutGridRemoveAction => "DesignPanelAction::LayoutGridRemoveRequested",
            Self::CountOnlyLayoutGridVariableAction => {
                "DesignPanelAction::LayoutGridVariableApplyRequested or LayoutGridVariableDetachRequested"
            }
            Self::WholePaintAction => "DesignPanelAction::PaintEditRequested",
            Self::ConflatedColorStyleAction => {
                "PaintStyle*Requested for whole styles or PaintColorVariable*Requested for leaf variables"
            }
            Self::NodeMediaReplaceAction => "DesignPanelAction::PaintMediaSourceActionRequested",
            Self::SingleExportAction => "DesignPanelAction::ExportAllRequested",
        }
    }
}

#[allow(deprecated)]
impl DesignPanelProperty {
    /// Returns the compatibility classification for non-canonical properties.
    pub const fn compatibility_path(self) -> Option<DesignPanelCompatibilityPath> {
        match self {
            Self::AlignmentX | Self::AlignmentY => {
                Some(DesignPanelCompatibilityPath::AutoLayoutAxisAlignment)
            }
            Self::EffectSettings(_) => Some(DesignPanelCompatibilityPath::AggregateEffectSettings),
            Self::AlignSelection => Some(DesignPanelCompatibilityPath::AutoLayoutAlignmentCell),
            Self::DistributeSelection => Some(DesignPanelCompatibilityPath::SelectionDistribution),
            Self::MediaCropMode
            | Self::MediaExposure
            | Self::MediaContrast
            | Self::MediaSaturation
            | Self::MediaTemperature
            | Self::MediaTint
            | Self::MediaHighlights
            | Self::MediaShadows => Some(DesignPanelCompatibilityPath::NodeMediaProperty),
            Self::EffectBlur(_)
            | Self::EffectSpread(_)
            | Self::EffectOffsetX(_)
            | Self::EffectOffsetY(_) => Some(DesignPanelCompatibilityPath::CompactEffectLeaf),
            Self::ExportScale(_) => Some(DesignPanelCompatibilityPath::ScaleOnlyExportProperty),
            _ => None,
        }
    }

    /// Returns the compatibility index carried by an ordinary layout-guide
    /// property.
    pub const fn layout_grid_index(self) -> Option<usize> {
        match self {
            Self::LayoutGridKind(index)
            | Self::LayoutGridVisible(index)
            | Self::LayoutGridAlignment(index)
            | Self::LayoutGridCount(index)
            | Self::LayoutGridSize(index)
            | Self::LayoutGridOffset(index)
            | Self::LayoutGridGutter(index)
            | Self::LayoutGridMargin(index)
            | Self::LayoutGridColor(index)
            | Self::LayoutGridOpacity(index) => Some(index),
            _ => None,
        }
    }

    /// Rewrites only the compatibility index of an ordinary layout-guide
    /// property.
    pub const fn with_layout_grid_index(self, index: usize) -> Self {
        match self {
            Self::LayoutGridKind(_) => Self::LayoutGridKind(index),
            Self::LayoutGridVisible(_) => Self::LayoutGridVisible(index),
            Self::LayoutGridAlignment(_) => Self::LayoutGridAlignment(index),
            Self::LayoutGridCount(_) => Self::LayoutGridCount(index),
            Self::LayoutGridSize(_) => Self::LayoutGridSize(index),
            Self::LayoutGridOffset(_) => Self::LayoutGridOffset(index),
            Self::LayoutGridGutter(_) => Self::LayoutGridGutter(index),
            Self::LayoutGridMargin(_) => Self::LayoutGridMargin(index),
            Self::LayoutGridColor(_) => Self::LayoutGridColor(index),
            Self::LayoutGridOpacity(_) => Self::LayoutGridOpacity(index),
            property => property,
        }
    }

    pub const fn effect_index(self) -> Option<usize> {
        match self {
            Self::EffectKind(index)
            | Self::EffectVisible(index)
            | Self::EffectSettings(index)
            | Self::EffectShadowColor(index)
            | Self::EffectShadowBlendMode(index)
            | Self::EffectShadowBlur(index)
            | Self::EffectShadowSpread(index)
            | Self::EffectShadowOffsetX(index)
            | Self::EffectShadowOffsetY(index)
            | Self::EffectDropShadowShowBehindNode(index)
            | Self::EffectBlurType(index)
            | Self::EffectBlurRadius(index)
            | Self::EffectProgressiveBlurStartRadius(index)
            | Self::EffectProgressiveBlurEndRadius(index)
            | Self::EffectProgressiveBlurStartOffsetX(index)
            | Self::EffectProgressiveBlurStartOffsetY(index)
            | Self::EffectProgressiveBlurEndOffsetX(index)
            | Self::EffectProgressiveBlurEndOffsetY(index)
            | Self::EffectNoiseType(index)
            | Self::EffectNoisePrimaryColor(index)
            | Self::EffectNoiseSecondaryColor(index)
            | Self::EffectNoiseOpacity(index)
            | Self::EffectNoiseBlendMode(index)
            | Self::EffectNoiseSizeX(index)
            | Self::EffectNoiseSizeY(index)
            | Self::EffectNoiseDensity(index)
            | Self::EffectTextureSizeX(index)
            | Self::EffectTextureSizeY(index)
            | Self::EffectTextureRadius(index)
            | Self::EffectTextureClipToShape(index)
            | Self::EffectGlassLightIntensity(index)
            | Self::EffectGlassLightAngle(index)
            | Self::EffectGlassRefraction(index)
            | Self::EffectGlassDepth(index)
            | Self::EffectGlassDispersion(index)
            | Self::EffectGlassFrost(index)
            | Self::EffectGlassSplay(index)
            | Self::EffectBlur(index)
            | Self::EffectSpread(index)
            | Self::EffectOffsetX(index)
            | Self::EffectOffsetY(index)
            | Self::EffectShaderProperty(index, _) => Some(index),
            _ => None,
        }
    }

    pub const fn with_effect_index(self, index: usize) -> Self {
        match self {
            Self::EffectKind(_) => Self::EffectKind(index),
            Self::EffectVisible(_) => Self::EffectVisible(index),
            Self::EffectSettings(_) => Self::EffectSettings(index),
            Self::EffectShadowColor(_) => Self::EffectShadowColor(index),
            Self::EffectShadowBlendMode(_) => Self::EffectShadowBlendMode(index),
            Self::EffectShadowBlur(_) => Self::EffectShadowBlur(index),
            Self::EffectShadowSpread(_) => Self::EffectShadowSpread(index),
            Self::EffectShadowOffsetX(_) => Self::EffectShadowOffsetX(index),
            Self::EffectShadowOffsetY(_) => Self::EffectShadowOffsetY(index),
            Self::EffectDropShadowShowBehindNode(_) => Self::EffectDropShadowShowBehindNode(index),
            Self::EffectBlurType(_) => Self::EffectBlurType(index),
            Self::EffectBlurRadius(_) => Self::EffectBlurRadius(index),
            Self::EffectProgressiveBlurStartRadius(_) => {
                Self::EffectProgressiveBlurStartRadius(index)
            }
            Self::EffectProgressiveBlurEndRadius(_) => Self::EffectProgressiveBlurEndRadius(index),
            Self::EffectProgressiveBlurStartOffsetX(_) => {
                Self::EffectProgressiveBlurStartOffsetX(index)
            }
            Self::EffectProgressiveBlurStartOffsetY(_) => {
                Self::EffectProgressiveBlurStartOffsetY(index)
            }
            Self::EffectProgressiveBlurEndOffsetX(_) => {
                Self::EffectProgressiveBlurEndOffsetX(index)
            }
            Self::EffectProgressiveBlurEndOffsetY(_) => {
                Self::EffectProgressiveBlurEndOffsetY(index)
            }
            Self::EffectNoiseType(_) => Self::EffectNoiseType(index),
            Self::EffectNoisePrimaryColor(_) => Self::EffectNoisePrimaryColor(index),
            Self::EffectNoiseSecondaryColor(_) => Self::EffectNoiseSecondaryColor(index),
            Self::EffectNoiseOpacity(_) => Self::EffectNoiseOpacity(index),
            Self::EffectNoiseBlendMode(_) => Self::EffectNoiseBlendMode(index),
            Self::EffectNoiseSizeX(_) => Self::EffectNoiseSizeX(index),
            Self::EffectNoiseSizeY(_) => Self::EffectNoiseSizeY(index),
            Self::EffectNoiseDensity(_) => Self::EffectNoiseDensity(index),
            Self::EffectTextureSizeX(_) => Self::EffectTextureSizeX(index),
            Self::EffectTextureSizeY(_) => Self::EffectTextureSizeY(index),
            Self::EffectTextureRadius(_) => Self::EffectTextureRadius(index),
            Self::EffectTextureClipToShape(_) => Self::EffectTextureClipToShape(index),
            Self::EffectGlassLightIntensity(_) => Self::EffectGlassLightIntensity(index),
            Self::EffectGlassLightAngle(_) => Self::EffectGlassLightAngle(index),
            Self::EffectGlassRefraction(_) => Self::EffectGlassRefraction(index),
            Self::EffectGlassDepth(_) => Self::EffectGlassDepth(index),
            Self::EffectGlassDispersion(_) => Self::EffectGlassDispersion(index),
            Self::EffectGlassFrost(_) => Self::EffectGlassFrost(index),
            Self::EffectGlassSplay(_) => Self::EffectGlassSplay(index),
            Self::EffectShaderProperty(_, property_index) => {
                Self::EffectShaderProperty(index, property_index)
            }
            Self::EffectBlur(_) => Self::EffectBlur(index),
            Self::EffectSpread(_) => Self::EffectSpread(index),
            Self::EffectOffsetX(_) => Self::EffectOffsetX(index),
            Self::EffectOffsetY(_) => Self::EffectOffsetY(index),
            property => property,
        }
    }

    pub const fn is_typography(self) -> bool {
        matches!(
            self,
            Self::TypographyStyle
                | Self::FontFamily
                | Self::FontStyle
                | Self::FontWeight
                | Self::FontSize
                | Self::LineHeight
                | Self::LetterSpacing
                | Self::TextLeadingTrim
                | Self::ParagraphSpacing
                | Self::ParagraphIndent
                | Self::ListSpacing
                | Self::TextHangingPunctuation
                | Self::TextHangingLists
                | Self::HorizontalTextAlignment
                | Self::VerticalTextAlignment
                | Self::TextResize
                | Self::TextTruncate
                | Self::TextMaxLines
                | Self::TextDecoration
                | Self::TextDecorationStyle
                | Self::TextDecorationOffset
                | Self::TextDecorationThickness
                | Self::TextDecorationColor
                | Self::TextDecorationSkipInk
                | Self::TextCase
                | Self::TextList
        )
    }

    /// Whether a typography property can target the active character range.
    ///
    /// Resizing, truncation, maximum lines, and vertical alignment describe
    /// the text layer's container and must remain whole-layer operations even
    /// while the host is editing characters.
    pub const fn supports_selected_text_range(self) -> bool {
        self.is_typography()
            && !matches!(
                self,
                Self::VerticalTextAlignment
                    | Self::TextResize
                    | Self::TextTruncate
                    | Self::TextMaxLines
            )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DesignPanelValue {
    Number(f32),
    /// A canonical radian value edited and displayed as degrees.
    AngleRadians(f32),
    /// A canonical `0..=1` ratio edited and displayed as a percentage.
    Ratio(f32),
    OptionalNumber(Option<f32>),
    Integer(i64),
    Bool(bool),
    Text(SharedString),
    TypographyStyle(Option<DesignTypographyStyleSelection>),
    LayoutMode(DesignLayoutMode),
    SizingMode(DesignSizingMode),
    AutoLayoutAlignment(DesignAutoLayoutAlignment),
    ItemSpacingMode(DesignItemSpacingMode),
    CounterAxisAlignContent(DesignCounterAxisAlignContent),
    LayoutPositioning(DesignLayoutPositioning),
    LayoutAlignSelf(DesignLayoutAlignSelf),
    StackingOrder(DesignStackingOrder),
    BaselineAlignment(DesignBaselineAlignment),
    GridAutoTracks(DesignGridAutoTracks),
    GridItemsPositioning(DesignGridItemsPositioning),
    GridTrack(DesignGridTrack),
    GridItemAlignment(DesignGridItemAlignment),
    Constraint(DesignConstraint),
    BooleanOperation(DesignBooleanOperation),
    MaskType(DesignMaskType),
    SectionDevStatus(Option<DesignSectionDevStatusKind>),
    RepeatType(DesignRepeatType),
    RepeatAxis(DesignRepeatAxis),
    TransformUnit(DesignTransformUnit),
    HandleMirroring(DesignHandleMirroring),
    BlendMode(DesignBlendMode),
    TextResize(DesignTextResize),
    LineHeight(DesignLineHeight),
    LetterSpacing(DesignLetterSpacing),
    TextHorizontalAlignment(DesignTextHorizontalAlignment),
    TextVerticalAlignment(DesignTextVerticalAlignment),
    TextLeadingTrim(DesignTextLeadingTrim),
    TextDecoration(DesignTextDecoration),
    TextDecorationStyle(DesignTextDecorationStyle),
    TextDecorationMetric(DesignTextDecorationMetric),
    TextDecorationColor(DesignTextDecorationColor),
    TextCase(DesignTextCase),
    TextList(DesignTextList),
    StrokeAlign(DesignStrokeAlign),
    StrokeCap(DesignStrokeCap),
    StrokeWeightMode(DesignStrokeWeightMode),
    StrokeJoin(DesignStrokeJoin),
    StrokeDashMode(DesignStrokeDashMode),
    StrokeVariableWidth(Option<DesignVariableWidthStroke>),
    StrokeType(DesignStrokeType),
    StrokeStretchBrush(DesignStretchBrushName),
    StrokeBrushDirection(DesignStrokeBrushDirection),
    StrokeScatterBrush(DesignScatterBrushName),
    EffectKind(DesignEffectKind),
    EffectSettings(DesignEffectSettings),
    EffectBlurType(DesignBlurType),
    EffectNoiseType(DesignNoiseType),
    ShaderProperty(DesignShaderPropertyValue),
    Color(DesignColor),
    GridKind(DesignGridKind),
    LayoutGridCount(DesignLayoutGridCount),
    ColumnGridAlignment(DesignColumnGridAlignment),
    RowGridAlignment(DesignRowGridAlignment),
    ExportSizing(DesignExportSizing),
    ExportFormat(DesignExportFormat),
    NumberList(Vec<f32>),
}

/// Lifecycle phase for a continuous inspector edit.
///
/// Hosts can use these phases to group keyboard stepping, typing, and pointer
/// scrubbing into one undoable document operation while the panel keeps only a
/// transient draft. A host should snapshot the controlled value at `Begin` and
/// restore that snapshot at `Cancel`; cancellation does not require the
/// accompanying candidate value to be the original.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesignPanelEditPhase {
    Begin,
    Preview,
    Commit,
    Cancel,
}

/// Scope of a text-property intent.
///
/// Text edit mode targets the host's active character range; object mode
/// targets the complete text layer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignTypographyTarget {
    WholeLayer,
    /// Compatibility target for hosts that do not yet identify text ranges.
    SelectedTextRange,
    /// Exact host-owned text-range revision.
    ///
    /// Hosts increment or otherwise replace this opaque revision whenever the
    /// active character range changes, even when the selected text node and
    /// Text edit mode remain unchanged.
    SelectedTextRangeRevision(u64),
}

impl DesignTypographyTarget {
    pub const fn is_selected_text_range(self) -> bool {
        matches!(
            self,
            Self::SelectedTextRange | Self::SelectedTextRangeRevision(_)
        )
    }

    pub const fn selected_text_range_revision(self) -> Option<u64> {
        match self {
            Self::SelectedTextRangeRevision(revision) => Some(revision),
            Self::WholeLayer | Self::SelectedTextRange => None,
        }
    }
}

/// Scope of a Fill/Stroke paint intent.
///
/// Fill controls follow Figma's text-edit semantics: while characters are
/// selected on a Text or TextPath node, the host applies the operation to that
/// exact character range. Stroke controls always target the complete layer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignPaintTarget {
    WholeLayer,
    /// Compatibility target for hosts that do not identify text ranges.
    SelectedTextRange,
    /// Exact host-owned text-range revision captured when the interaction
    /// starts.
    ///
    /// Hosts must reject this target when it no longer matches their active
    /// character range, even when the selected node is unchanged.
    SelectedTextRangeRevision(u64),
}

impl DesignPaintTarget {
    pub const fn is_selected_text_range(self) -> bool {
        matches!(
            self,
            Self::SelectedTextRange | Self::SelectedTextRangeRevision(_)
        )
    }

    pub const fn selected_text_range_revision(self) -> Option<u64> {
        match self {
            Self::SelectedTextRangeRevision(revision) => Some(revision),
            Self::WholeLayer | Self::SelectedTextRange => None,
        }
    }
}

/// Fully resolved type of a Figma variable after following aliases.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignVariableResolvedType {
    Boolean,
    Color,
    Float,
    String,
}

impl DesignVariableResolvedType {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Boolean => "Boolean",
            Self::Color => "Color",
            Self::Float => "Number",
            Self::String => "String",
        }
    }
}

/// Figma's variable-picker scopes.
///
/// `Opaque` preserves a scope introduced after this crate version without
/// silently widening it to `AllScopes`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DesignVariableScope {
    AllScopes,
    TextContent,
    CornerRadius,
    WidthHeight,
    Gap,
    AllFills,
    FrameFill,
    ShapeFill,
    TextFill,
    StrokeColor,
    EffectColor,
    StrokeFloat,
    EffectFloat,
    Opacity,
    FontFamily,
    FontStyle,
    FontWeight,
    FontSize,
    LineHeight,
    LetterSpacing,
    ParagraphSpacing,
    ParagraphIndent,
    Opaque(SharedString),
}

impl DesignVariableScope {
    pub fn api_name(&self) -> &str {
        match self {
            Self::AllScopes => "ALL_SCOPES",
            Self::TextContent => "TEXT_CONTENT",
            Self::CornerRadius => "CORNER_RADIUS",
            Self::WidthHeight => "WIDTH_HEIGHT",
            Self::Gap => "GAP",
            Self::AllFills => "ALL_FILLS",
            Self::FrameFill => "FRAME_FILL",
            Self::ShapeFill => "SHAPE_FILL",
            Self::TextFill => "TEXT_FILL",
            Self::StrokeColor => "STROKE_COLOR",
            Self::EffectColor => "EFFECT_COLOR",
            Self::StrokeFloat => "STROKE_FLOAT",
            Self::EffectFloat => "EFFECT_FLOAT",
            Self::Opacity => "OPACITY",
            Self::FontFamily => "FONT_FAMILY",
            Self::FontStyle => "FONT_STYLE",
            Self::FontWeight => "FONT_WEIGHT",
            Self::FontSize => "FONT_SIZE",
            Self::LineHeight => "LINE_HEIGHT",
            Self::LetterSpacing => "LETTER_SPACING",
            Self::ParagraphSpacing => "PARAGRAPH_SPACING",
            Self::ParagraphIndent => "PARAGRAPH_INDENT",
            Self::Opaque(value) => value.as_ref(),
        }
    }
}

/// Exact Figma `VariableBindableNodeField` names.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignVariableBindableNodeField {
    Height,
    Width,
    Characters,
    ItemSpacing,
    PaddingLeft,
    PaddingRight,
    PaddingTop,
    PaddingBottom,
    Visible,
    CornerRadius,
    TopLeftRadius,
    TopRightRadius,
    BottomLeftRadius,
    BottomRightRadius,
    MinWidth,
    MaxWidth,
    MinHeight,
    MaxHeight,
    CounterAxisSpacing,
    StrokeWeight,
    StrokeTopWeight,
    StrokeRightWeight,
    StrokeBottomWeight,
    StrokeLeftWeight,
    Opacity,
    GridRowGap,
    GridColumnGap,
}

impl DesignVariableBindableNodeField {
    pub const fn api_name(self) -> &'static str {
        match self {
            Self::Height => "height",
            Self::Width => "width",
            Self::Characters => "characters",
            Self::ItemSpacing => "itemSpacing",
            Self::PaddingLeft => "paddingLeft",
            Self::PaddingRight => "paddingRight",
            Self::PaddingTop => "paddingTop",
            Self::PaddingBottom => "paddingBottom",
            Self::Visible => "visible",
            Self::CornerRadius => "cornerRadius",
            Self::TopLeftRadius => "topLeftRadius",
            Self::TopRightRadius => "topRightRadius",
            Self::BottomLeftRadius => "bottomLeftRadius",
            Self::BottomRightRadius => "bottomRightRadius",
            Self::MinWidth => "minWidth",
            Self::MaxWidth => "maxWidth",
            Self::MinHeight => "minHeight",
            Self::MaxHeight => "maxHeight",
            Self::CounterAxisSpacing => "counterAxisSpacing",
            Self::StrokeWeight => "strokeWeight",
            Self::StrokeTopWeight => "strokeTopWeight",
            Self::StrokeRightWeight => "strokeRightWeight",
            Self::StrokeBottomWeight => "strokeBottomWeight",
            Self::StrokeLeftWeight => "strokeLeftWeight",
            Self::Opacity => "opacity",
            Self::GridRowGap => "gridRowGap",
            Self::GridColumnGap => "gridColumnGap",
        }
    }
}

/// Exact Figma `VariableBindableTextField` names.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignVariableBindableTextField {
    FontFamily,
    FontSize,
    FontStyle,
    FontWeight,
    LetterSpacing,
    LineHeight,
    ParagraphSpacing,
    ParagraphIndent,
}

impl DesignVariableBindableTextField {
    pub const fn api_name(self) -> &'static str {
        match self {
            Self::FontFamily => "fontFamily",
            Self::FontSize => "fontSize",
            Self::FontStyle => "fontStyle",
            Self::FontWeight => "fontWeight",
            Self::LetterSpacing => "letterSpacing",
            Self::LineHeight => "lineHeight",
            Self::ParagraphSpacing => "paragraphSpacing",
            Self::ParagraphIndent => "paragraphIndent",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignVariableBindableField {
    Node(DesignVariableBindableNodeField),
    Text(DesignVariableBindableTextField),
}

impl DesignVariableBindableField {
    pub const fn api_name(self) -> &'static str {
        match self {
            Self::Node(field) => field.api_name(),
            Self::Text(field) => field.api_name(),
        }
    }
}

/// Exact property-to-API-field target carried by variable intents.
///
/// Independent-corner nodes intentionally carry four fields for the uniform
/// Corner radius property, matching how Figma records that binding in
/// `boundVariables`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignPropertyVariableTarget {
    pub property: DesignPanelProperty,
    pub fields: Vec<DesignVariableBindableField>,
    pub resolved_type: DesignVariableResolvedType,
    pub compatible_scope: Option<DesignVariableScope>,
    pub typography_target: Option<DesignTypographyTarget>,
}

impl DesignPropertyVariableTarget {
    fn node(
        property: DesignPanelProperty,
        fields: impl IntoIterator<Item = DesignVariableBindableNodeField>,
        resolved_type: DesignVariableResolvedType,
        compatible_scope: Option<DesignVariableScope>,
    ) -> Self {
        Self {
            property,
            fields: fields
                .into_iter()
                .map(DesignVariableBindableField::Node)
                .collect(),
            resolved_type,
            compatible_scope,
            typography_target: None,
        }
    }

    fn text(
        property: DesignPanelProperty,
        field: DesignVariableBindableTextField,
        resolved_type: DesignVariableResolvedType,
        compatible_scope: DesignVariableScope,
    ) -> Self {
        Self {
            property,
            fields: vec![DesignVariableBindableField::Text(field)],
            resolved_type,
            compatible_scope: Some(compatible_scope),
            typography_target: Some(DesignTypographyTarget::WholeLayer),
        }
    }

    pub const fn with_typography_target(mut self, target: DesignTypographyTarget) -> Self {
        if self.typography_target.is_some() {
            self.typography_target = Some(target);
        }
        self
    }
}

/// Whether a variable lives in this page or a published library.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DesignVariableSource {
    Page {
        page_id: SharedString,
        page_name: SharedString,
    },
    Library {
        library_id: SharedString,
        library_name: SharedString,
    },
}

impl DesignVariableSource {
    pub fn page(page_id: impl Into<SharedString>, page_name: impl Into<SharedString>) -> Self {
        Self::Page {
            page_id: page_id.into(),
            page_name: page_name.into(),
        }
    }

    pub fn library(
        library_id: impl Into<SharedString>,
        library_name: impl Into<SharedString>,
    ) -> Self {
        Self::Library {
            library_id: library_id.into(),
            library_name: library_name.into(),
        }
    }

    pub const fn is_remote(&self) -> bool {
        matches!(self, Self::Library { .. })
    }

    pub const fn label(&self) -> &SharedString {
        match self {
            Self::Page { page_name, .. } => page_name,
            Self::Library { library_name, .. } => library_name,
        }
    }
}

/// Import state is separate from source so imported library variables retain
/// their remote identity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignVariableImportState {
    Local,
    Imported,
    Available,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DesignVariableSearchMetadata {
    pub path: Vec<SharedString>,
    pub description: Option<SharedString>,
    pub keywords: Vec<SharedString>,
}

impl DesignVariableSearchMetadata {
    pub fn new(
        path: impl IntoIterator<Item = impl Into<SharedString>>,
        keywords: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        Self {
            path: path.into_iter().map(Into::into).collect(),
            description: None,
            keywords: keywords.into_iter().map(Into::into).collect(),
        }
    }

    pub fn described(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Host-resolved preview value for the active variable mode.
#[derive(Clone, Debug, PartialEq)]
pub enum DesignVariableResolvedValue {
    Boolean(bool),
    Color(DesignColor),
    Float(f32),
    String(SharedString),
}

impl DesignVariableResolvedValue {
    pub const fn resolved_type(&self) -> DesignVariableResolvedType {
        match self {
            Self::Boolean(_) => DesignVariableResolvedType::Boolean,
            Self::Color(_) => DesignVariableResolvedType::Color,
            Self::Float(_) => DesignVariableResolvedType::Float,
            Self::String(_) => DesignVariableResolvedType::String,
        }
    }

    pub fn panel_value(&self) -> DesignPanelValue {
        match self {
            Self::Boolean(value) => DesignPanelValue::Bool(*value),
            Self::Color(value) => DesignPanelValue::Color(*value),
            Self::Float(value) => DesignPanelValue::Number(*value),
            Self::String(value) => DesignPanelValue::Text(value.clone()),
        }
    }
}

/// One lossless host-supplied variable picker row.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignVariable {
    pub id: SharedString,
    pub name: SharedString,
    pub collection_id: SharedString,
    pub collection_name: SharedString,
    pub source: DesignVariableSource,
    pub resolved_type: DesignVariableResolvedType,
    pub scopes: Vec<DesignVariableScope>,
    pub import_state: DesignVariableImportState,
    pub resolved_value: Option<DesignVariableResolvedValue>,
    pub search: DesignVariableSearchMetadata,
    pub disabled_reason: Option<SharedString>,
}

impl DesignVariable {
    pub fn page(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        collection_id: impl Into<SharedString>,
        collection_name: impl Into<SharedString>,
        resolved_type: DesignVariableResolvedType,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            collection_id: collection_id.into(),
            collection_name: collection_name.into(),
            source: DesignVariableSource::page("current-page", "This page"),
            resolved_type,
            scopes: vec![DesignVariableScope::AllScopes],
            import_state: DesignVariableImportState::Local,
            resolved_value: None,
            search: DesignVariableSearchMetadata::default(),
            disabled_reason: None,
        }
    }

    pub fn with_source(mut self, source: DesignVariableSource) -> Self {
        self.source = source;
        self
    }

    pub fn with_scopes(mut self, scopes: impl IntoIterator<Item = DesignVariableScope>) -> Self {
        self.scopes = scopes.into_iter().collect();
        self
    }

    pub const fn with_import_state(mut self, import_state: DesignVariableImportState) -> Self {
        self.import_state = import_state;
        self
    }

    pub fn with_resolved_value(mut self, value: DesignVariableResolvedValue) -> Self {
        debug_assert_eq!(value.resolved_type(), self.resolved_type);
        self.resolved_value = Some(value);
        self
    }

    pub fn with_search(mut self, search: DesignVariableSearchMetadata) -> Self {
        self.search = search;
        self
    }

    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.disabled_reason = Some(reason.into());
        self
    }

    pub fn is_compatible_with(&self, target: &DesignPropertyVariableTarget) -> bool {
        if self.resolved_type != target.resolved_type {
            return false;
        }
        let Some(scope) = &target.compatible_scope else {
            return true;
        };
        self.scopes.is_empty()
            || self.scopes.contains(&DesignVariableScope::AllScopes)
            || self.scopes.contains(scope)
    }

    pub fn matches_search(&self, query: &str) -> bool {
        let tokens = query
            .split_whitespace()
            .map(str::to_ascii_lowercase)
            .collect::<Vec<_>>();
        if tokens.is_empty() {
            return true;
        }
        let mut haystack = format!(
            "{} {} {} {} {}",
            self.id,
            self.name,
            self.collection_id,
            self.collection_name,
            self.source.label()
        )
        .to_ascii_lowercase();
        for segment in &self.search.path {
            haystack.push(' ');
            haystack.push_str(&segment.to_ascii_lowercase());
        }
        if let Some(description) = &self.search.description {
            haystack.push(' ');
            haystack.push_str(&description.to_ascii_lowercase());
        }
        for keyword in &self.search.keywords {
            haystack.push(' ');
            haystack.push_str(&keyword.to_ascii_lowercase());
        }
        tokens.iter().all(|token| haystack.contains(token))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesignVariableViewData {
    pub variables: Vec<DesignVariable>,
}

impl DesignVariableViewData {
    pub fn new(variables: impl IntoIterator<Item = DesignVariable>) -> Self {
        Self {
            variables: variables.into_iter().collect(),
        }
    }

    pub fn variable(&self, variable_id: &str) -> Option<&DesignVariable> {
        self.variables
            .iter()
            .find(|variable| variable.id.as_ref() == variable_id)
    }

    pub fn compatible<'a>(
        &'a self,
        target: &'a DesignPropertyVariableTarget,
        query: &'a str,
    ) -> impl Iterator<Item = &'a DesignVariable> + 'a {
        self.variables.iter().filter(move |variable| {
            variable.is_compatible_with(target) && variable.matches_search(query)
        })
    }
}

/// Host-filtered color variables available to solid paints and gradient stops.
///
/// The host supplies only candidates compatible with the active document
/// context. The panel additionally rejects non-Color values and never derives
/// a variable identity from a resolved color. Import and apply remain separate
/// operations because an available library variable cannot be bound yet.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesignPaintVariableViewData {
    pub variables: Vec<DesignVariable>,
}

impl DesignPaintVariableViewData {
    pub fn new(variables: impl IntoIterator<Item = DesignVariable>) -> Self {
        Self {
            variables: variables
                .into_iter()
                .filter(|variable| {
                    variable.resolved_type == DesignVariableResolvedType::Color
                        && matches!(
                            variable.resolved_value.as_ref(),
                            None | Some(DesignVariableResolvedValue::Color(_))
                        )
                })
                .collect(),
        }
    }

    pub fn variable(&self, variable_id: &str) -> Option<&DesignVariable> {
        self.variables.iter().find(|variable| {
            variable.id.as_ref() == variable_id
                && variable.resolved_type == DesignVariableResolvedType::Color
                && matches!(
                    variable.resolved_value.as_ref(),
                    None | Some(DesignVariableResolvedValue::Color(_))
                )
        })
    }

    pub fn variable_mut(&mut self, variable_id: &str) -> Option<&mut DesignVariable> {
        self.variables.iter_mut().find(|variable| {
            variable.id.as_ref() == variable_id
                && variable.resolved_type == DesignVariableResolvedType::Color
                && matches!(
                    variable.resolved_value.as_ref(),
                    None | Some(DesignVariableResolvedValue::Color(_))
                )
        })
    }
}

/// Native audience classification used by Figma's color-contrast checker.
///
/// `Auto` is a transient picker choice. Every host snapshot also supplies the
/// concrete category that Auto resolves to for the exact inspected paint
/// occurrence.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DesignColorContrastCategory {
    #[default]
    Auto,
    LargeText,
    NormalText,
    Graphics,
}

impl DesignColorContrastCategory {
    pub const ALL: [Self; 4] = [
        Self::Auto,
        Self::LargeText,
        Self::NormalText,
        Self::Graphics,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::LargeText => "Large text",
            Self::NormalText => "Normal text",
            Self::Graphics => "Graphics",
        }
    }

    pub const fn is_concrete(self) -> bool {
        !matches!(self, Self::Auto)
    }
}

/// WCAG conformance level selected in Figma's color-contrast checker.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DesignColorContrastLevel {
    #[default]
    Aa,
    Aaa,
}

impl DesignColorContrastLevel {
    pub const ALL: [Self; 2] = [Self::Aa, Self::Aaa];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Aa => "AA",
            Self::Aaa => "AAA",
        }
    }
}

/// One host-computed nearest compliant color for an exact checker mode.
///
/// The panel intentionally does not guess Figma's correction algorithm. It
/// emits the supplied color through the ordinary phased paint-edit contract
/// and waits for the host to echo the accepted paint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesignColorContrastCorrection {
    pub category: DesignColorContrastCategory,
    pub level: DesignColorContrastLevel,
    pub color: DesignColor,
}

impl DesignColorContrastCorrection {
    pub const fn new(
        category: DesignColorContrastCategory,
        level: DesignColorContrastLevel,
        color: DesignColor,
    ) -> Self {
        Self {
            category,
            level,
            color,
        }
    }

    pub const fn is_valid(self) -> bool {
        self.category.is_concrete()
            && !(matches!(self.category, DesignColorContrastCategory::Graphics)
                && matches!(self.level, DesignColorContrastLevel::Aaa))
    }
}

/// Host-owned contrast result for one Solid color or one stable gradient stop.
///
/// `ratio` is authoritative. Supplying it, rather than recomputing from a
/// guessed white/black canvas, lets the host account for the exact effective
/// background, opacity, blending, and nested scene context.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignColorContrastLeafViewData {
    pub color_target: DesignPaintColorTarget,
    pub effective_background: DesignColor,
    pub ratio: f32,
    pub automatic_category: DesignColorContrastCategory,
    pub corrections: Vec<DesignColorContrastCorrection>,
    pub disabled_reason: Option<SharedString>,
}

impl DesignColorContrastLeafViewData {
    pub fn new(
        color_target: DesignPaintColorTarget,
        effective_background: DesignColor,
        ratio: f32,
        automatic_category: DesignColorContrastCategory,
    ) -> Self {
        Self {
            color_target,
            effective_background,
            ratio,
            automatic_category,
            corrections: Vec::new(),
            disabled_reason: None,
        }
    }

    pub fn with_corrections(
        mut self,
        corrections: impl IntoIterator<Item = DesignColorContrastCorrection>,
    ) -> Self {
        self.corrections = corrections.into_iter().collect();
        self
    }

    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.disabled_reason = Some(reason.into());
        self
    }

    pub fn is_valid(&self) -> bool {
        if !self.ratio.is_finite()
            || !(1. ..=21.).contains(&self.ratio)
            || !self.automatic_category.is_concrete()
        {
            return false;
        }
        let mut modes = HashSet::with_capacity(self.corrections.len());
        self.corrections.iter().all(|correction| {
            correction.is_valid() && modes.insert((correction.category, correction.level))
        })
    }

    pub fn correction(
        &self,
        category: DesignColorContrastCategory,
        level: DesignColorContrastLevel,
    ) -> Option<DesignColor> {
        self.corrections
            .iter()
            .find(|correction| correction.category == category && correction.level == level)
            .map(|correction| correction.color)
    }
}

/// Stable document target for one paint represented by contrast view data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignColorContrastPaintTarget {
    Paint {
        node_id: SharedString,
        collection: DesignPanelCollection,
        paint_id: SharedString,
        index: usize,
    },
    SelectionColor {
        target: DesignPanelTarget,
        selection_color_id: SharedString,
    },
}

impl DesignColorContrastPaintTarget {
    pub fn paint(
        node_id: impl Into<SharedString>,
        collection: DesignPanelCollection,
        paint_id: impl Into<SharedString>,
        index: usize,
    ) -> Self {
        Self::Paint {
            node_id: node_id.into(),
            collection,
            paint_id: paint_id.into(),
            index,
        }
    }

    pub fn selection_color(
        target: DesignPanelTarget,
        selection_color_id: impl Into<SharedString>,
    ) -> Self {
        Self::SelectionColor {
            target,
            selection_color_id: selection_color_id.into(),
        }
    }
}

/// Every contrast leaf for one stable paint occurrence.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignColorContrastPaintViewData {
    pub target: DesignColorContrastPaintTarget,
    pub leaves: Vec<DesignColorContrastLeafViewData>,
}

impl DesignColorContrastPaintViewData {
    pub fn new(
        target: DesignColorContrastPaintTarget,
        leaves: impl IntoIterator<Item = DesignColorContrastLeafViewData>,
    ) -> Self {
        Self {
            target,
            leaves: leaves.into_iter().collect(),
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.leaves.is_empty() || !self.leaves.iter().all(|leaf| leaf.is_valid()) {
            return false;
        }
        let mut solid = false;
        let mut stop_ids = HashSet::with_capacity(self.leaves.len());
        self.leaves.iter().all(|leaf| match &leaf.color_target {
            DesignPaintColorTarget::Solid => {
                if solid || self.leaves.len() != 1 {
                    false
                } else {
                    solid = true;
                    true
                }
            }
            DesignPaintColorTarget::GradientStop { stop_id, index } => {
                let identity = if stop_id.is_empty() {
                    format!("#{index}").into()
                } else {
                    stop_id.clone()
                };
                !solid && stop_ids.insert(identity)
            }
        })
    }

    pub fn leaf(
        &self,
        color_target: &DesignPaintColorTarget,
    ) -> Option<&DesignColorContrastLeafViewData> {
        self.leaves.iter().find(|leaf| {
            leaf.color_target == *color_target
                || matches!(
                    (&leaf.color_target, color_target),
                    (
                        DesignPaintColorTarget::GradientStop {
                            stop_id: left_id,
                            index: left_index,
                        },
                        DesignPaintColorTarget::GradientStop {
                            stop_id: right_id,
                            index: right_index,
                        },
                    ) if (!left_id.is_empty() && left_id == right_id)
                        || (left_id.is_empty() && right_id.is_empty() && left_index == right_index)
                )
        })
    }
}

/// Host-owned contrast catalog scoped to the current inspector selection.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DesignColorContrastViewData {
    pub paints: Vec<DesignColorContrastPaintViewData>,
}

impl DesignColorContrastViewData {
    pub fn new(paints: impl IntoIterator<Item = DesignColorContrastPaintViewData>) -> Self {
        let mut view_data = Self {
            paints: paints.into_iter().collect(),
        };
        view_data
            .paints
            .retain(DesignColorContrastPaintViewData::is_valid);
        view_data
    }

    pub fn paint(
        &self,
        target: &DesignColorContrastPaintTarget,
    ) -> Option<&DesignColorContrastPaintViewData> {
        self.paints.iter().find(|paint| paint.target == *target)
    }
}

impl DesignPanelNode {
    /// Maps one visible inspector property to the exact Figma variable fields
    /// that a host must bind. Composite padding and independent-corner
    /// controls deliberately return every affected leaf.
    pub fn property_variable_target(
        &self,
        property: DesignPanelProperty,
    ) -> Option<DesignPropertyVariableTarget> {
        use DesignVariableBindableNodeField as NodeField;
        use DesignVariableBindableTextField as TextField;
        use DesignVariableResolvedType as Type;
        use DesignVariableScope as Scope;

        macro_rules! node {
            ($fields:expr, $resolved_type:expr, $scope:expr $(,)?) => {
                DesignPropertyVariableTarget::node(property, $fields, $resolved_type, $scope)
            };
        }
        let text = |field, resolved_type, scope| {
            DesignPropertyVariableTarget::text(property, field, resolved_type, scope)
        };
        let capability_allows_property = match property {
            DesignPanelProperty::Visible => {
                self.supports_section(DesignPanelSection::Layer) && self.supports_visibility()
            }
            DesignPanelProperty::Opacity => {
                self.supports_section(DesignPanelSection::Layer) && self.supports_layer_appearance()
            }
            DesignPanelProperty::Width | DesignPanelProperty::Height => {
                self.supports_section(DesignPanelSection::Layout) && self.supports_dimensions()
            }
            DesignPanelProperty::Gap
            | DesignPanelProperty::CounterAxisGap
            | DesignPanelProperty::PaddingVertical
            | DesignPanelProperty::PaddingHorizontal
            | DesignPanelProperty::PaddingShorthand
            | DesignPanelProperty::PaddingTop
            | DesignPanelProperty::PaddingRight
            | DesignPanelProperty::PaddingBottom
            | DesignPanelProperty::PaddingLeft => {
                self.supports_section(DesignPanelSection::Layout)
                    && self.supports_auto_layout_container()
            }
            DesignPanelProperty::MinWidth
            | DesignPanelProperty::MaxWidth
            | DesignPanelProperty::MinHeight
            | DesignPanelProperty::MaxHeight => {
                self.supports_section(DesignPanelSection::Layout)
                    && self.supports_dimensions()
                    && (self.supports_auto_layout_container() || self.supports_auto_layout_child())
            }
            DesignPanelProperty::CornerRadius
            | DesignPanelProperty::CornerRadiusTopLeft
            | DesignPanelProperty::CornerRadiusTopRight
            | DesignPanelProperty::CornerRadiusBottomRight
            | DesignPanelProperty::CornerRadiusBottomLeft => {
                self.supports_section(DesignPanelSection::Layer)
            }
            DesignPanelProperty::StrokeWeight
            | DesignPanelProperty::StrokeWeightTop
            | DesignPanelProperty::StrokeWeightRight
            | DesignPanelProperty::StrokeWeightBottom
            | DesignPanelProperty::StrokeWeightLeft => {
                self.supports_section(DesignPanelSection::Stroke) && self.supports_stroke()
            }
            property if property.is_typography() => {
                self.supports_section(DesignPanelSection::Typography)
            }
            _ => true,
        };
        if !capability_allows_property {
            return None;
        }
        match property {
            DesignPanelProperty::Visible if self.supports_visibility() => {
                Some(node!([NodeField::Visible], Type::Boolean, None,))
            }
            DesignPanelProperty::Width if self.supports_dimensions() => Some(node!(
                [NodeField::Width],
                Type::Float,
                Some(Scope::WidthHeight),
            )),
            DesignPanelProperty::Height if self.supports_dimensions() => Some(node!(
                [NodeField::Height],
                Type::Float,
                Some(Scope::WidthHeight),
            )),
            DesignPanelProperty::Gap => {
                let layout = self.layout.as_ref()?;
                match layout.mode {
                    DesignLayoutMode::Horizontal | DesignLayoutMode::Vertical => Some(node!(
                        [NodeField::ItemSpacing],
                        Type::Float,
                        Some(Scope::Gap),
                    )),
                    DesignLayoutMode::Grid => Some(node!(
                        [NodeField::GridColumnGap],
                        Type::Float,
                        Some(Scope::Gap),
                    )),
                    DesignLayoutMode::None => None,
                }
            }
            DesignPanelProperty::CounterAxisGap => {
                let layout = self.layout.as_ref()?;
                match layout.mode {
                    DesignLayoutMode::Grid => Some(node!(
                        [NodeField::GridRowGap],
                        Type::Float,
                        Some(Scope::Gap),
                    )),
                    DesignLayoutMode::Horizontal if layout.wrap => Some(node!(
                        [NodeField::CounterAxisSpacing],
                        Type::Float,
                        Some(Scope::Gap),
                    )),
                    DesignLayoutMode::None
                    | DesignLayoutMode::Horizontal
                    | DesignLayoutMode::Vertical => None,
                }
            }
            DesignPanelProperty::PaddingVertical
                if self.layout.as_ref()?.mode != DesignLayoutMode::None =>
            {
                Some(node!(
                    [NodeField::PaddingTop, NodeField::PaddingBottom],
                    Type::Float,
                    Some(Scope::Gap),
                ))
            }
            DesignPanelProperty::PaddingHorizontal
                if self.layout.as_ref()?.mode != DesignLayoutMode::None =>
            {
                Some(node!(
                    [NodeField::PaddingLeft, NodeField::PaddingRight],
                    Type::Float,
                    Some(Scope::Gap),
                ))
            }
            DesignPanelProperty::PaddingShorthand
                if self.layout.as_ref()?.mode != DesignLayoutMode::None =>
            {
                Some(node!(
                    [
                        NodeField::PaddingTop,
                        NodeField::PaddingRight,
                        NodeField::PaddingBottom,
                        NodeField::PaddingLeft,
                    ],
                    Type::Float,
                    Some(Scope::Gap),
                ))
            }
            DesignPanelProperty::PaddingTop
                if self.layout.as_ref()?.mode != DesignLayoutMode::None =>
            {
                Some(node!(
                    [NodeField::PaddingTop],
                    Type::Float,
                    Some(Scope::Gap),
                ))
            }
            DesignPanelProperty::PaddingRight
                if self.layout.as_ref()?.mode != DesignLayoutMode::None =>
            {
                Some(node!(
                    [NodeField::PaddingRight],
                    Type::Float,
                    Some(Scope::Gap),
                ))
            }
            DesignPanelProperty::PaddingBottom
                if self.layout.as_ref()?.mode != DesignLayoutMode::None =>
            {
                Some(node!(
                    [NodeField::PaddingBottom],
                    Type::Float,
                    Some(Scope::Gap),
                ))
            }
            DesignPanelProperty::PaddingLeft
                if self.layout.as_ref()?.mode != DesignLayoutMode::None =>
            {
                Some(node!(
                    [NodeField::PaddingLeft],
                    Type::Float,
                    Some(Scope::Gap),
                ))
            }
            DesignPanelProperty::MinWidth if self.layout.is_some() => Some(node!(
                [NodeField::MinWidth],
                Type::Float,
                Some(Scope::WidthHeight),
            )),
            DesignPanelProperty::MaxWidth if self.layout.is_some() => Some(node!(
                [NodeField::MaxWidth],
                Type::Float,
                Some(Scope::WidthHeight),
            )),
            DesignPanelProperty::MinHeight if self.layout.is_some() => Some(node!(
                [NodeField::MinHeight],
                Type::Float,
                Some(Scope::WidthHeight),
            )),
            DesignPanelProperty::MaxHeight if self.layout.is_some() => Some(node!(
                [NodeField::MaxHeight],
                Type::Float,
                Some(Scope::WidthHeight),
            )),
            DesignPanelProperty::Opacity if self.supports_layer_appearance() => Some(node!(
                [NodeField::Opacity],
                Type::Float,
                Some(Scope::Opacity),
            )),
            DesignPanelProperty::CornerRadius if self.corner_capabilities.uniform_radius => {
                let fields = if self.corner_capabilities.independent_radii {
                    vec![
                        NodeField::TopLeftRadius,
                        NodeField::TopRightRadius,
                        NodeField::BottomRightRadius,
                        NodeField::BottomLeftRadius,
                    ]
                } else {
                    vec![NodeField::CornerRadius]
                };
                Some(node!(fields, Type::Float, Some(Scope::CornerRadius)))
            }
            DesignPanelProperty::CornerRadiusTopLeft
                if self.corner_capabilities.independent_radii =>
            {
                Some(node!(
                    [NodeField::TopLeftRadius],
                    Type::Float,
                    Some(Scope::CornerRadius),
                ))
            }
            DesignPanelProperty::CornerRadiusTopRight
                if self.corner_capabilities.independent_radii =>
            {
                Some(node!(
                    [NodeField::TopRightRadius],
                    Type::Float,
                    Some(Scope::CornerRadius),
                ))
            }
            DesignPanelProperty::CornerRadiusBottomRight
                if self.corner_capabilities.independent_radii =>
            {
                Some(node!(
                    [NodeField::BottomRightRadius],
                    Type::Float,
                    Some(Scope::CornerRadius),
                ))
            }
            DesignPanelProperty::CornerRadiusBottomLeft
                if self.corner_capabilities.independent_radii =>
            {
                Some(node!(
                    [NodeField::BottomLeftRadius],
                    Type::Float,
                    Some(Scope::CornerRadius),
                ))
            }
            DesignPanelProperty::StrokeWeight if self.stroke.is_some() => Some(node!(
                [NodeField::StrokeWeight],
                Type::Float,
                Some(Scope::StrokeFloat),
            )),
            DesignPanelProperty::StrokeWeightTop
                if self
                    .stroke
                    .as_ref()
                    .is_some_and(|stroke| stroke.capabilities.individual_weights) =>
            {
                Some(node!(
                    [NodeField::StrokeTopWeight],
                    Type::Float,
                    Some(Scope::StrokeFloat),
                ))
            }
            DesignPanelProperty::StrokeWeightRight
                if self
                    .stroke
                    .as_ref()
                    .is_some_and(|stroke| stroke.capabilities.individual_weights) =>
            {
                Some(node!(
                    [NodeField::StrokeRightWeight],
                    Type::Float,
                    Some(Scope::StrokeFloat),
                ))
            }
            DesignPanelProperty::StrokeWeightBottom
                if self
                    .stroke
                    .as_ref()
                    .is_some_and(|stroke| stroke.capabilities.individual_weights) =>
            {
                Some(node!(
                    [NodeField::StrokeBottomWeight],
                    Type::Float,
                    Some(Scope::StrokeFloat),
                ))
            }
            DesignPanelProperty::StrokeWeightLeft
                if self
                    .stroke
                    .as_ref()
                    .is_some_and(|stroke| stroke.capabilities.individual_weights) =>
            {
                Some(node!(
                    [NodeField::StrokeLeftWeight],
                    Type::Float,
                    Some(Scope::StrokeFloat),
                ))
            }
            DesignPanelProperty::FontFamily if self.typography.is_some() => {
                Some(text(TextField::FontFamily, Type::String, Scope::FontFamily))
            }
            DesignPanelProperty::FontStyle if self.typography.is_some() => {
                Some(text(TextField::FontStyle, Type::String, Scope::FontStyle))
            }
            DesignPanelProperty::FontWeight if self.typography.is_some() => {
                Some(text(TextField::FontWeight, Type::Float, Scope::FontWeight))
            }
            DesignPanelProperty::FontSize if self.typography.is_some() => {
                Some(text(TextField::FontSize, Type::Float, Scope::FontSize))
            }
            DesignPanelProperty::LineHeight if self.typography.is_some() => {
                Some(text(TextField::LineHeight, Type::Float, Scope::LineHeight))
            }
            DesignPanelProperty::LetterSpacing if self.typography.is_some() => Some(text(
                TextField::LetterSpacing,
                Type::Float,
                Scope::LetterSpacing,
            )),
            DesignPanelProperty::ParagraphSpacing if self.typography.is_some() => Some(text(
                TextField::ParagraphSpacing,
                Type::Float,
                Scope::ParagraphSpacing,
            )),
            DesignPanelProperty::ParagraphIndent if self.typography.is_some() => Some(text(
                TextField::ParagraphIndent,
                Type::Float,
                Scope::ParagraphIndent,
            )),
            _ => None,
        }
    }
}

/// Stable host target for commands that may affect a page or many selected
/// nodes. Direct target-aware commands carry this value themselves; ordinary
/// multi-selection leaves carry it through
/// [`DesignPanelAction::TargetedNodeActionRequested`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignPanelTarget {
    Page { page_id: SharedString },
    Nodes { node_ids: Vec<SharedString> },
}

/// Host-owned Draw appearance policy for one exact ordered node target.
///
/// Opacity always uses the fixed `0..=100` percentage range. Corner radius is
/// intentionally target-bound because its useful slider maximum depends on
/// host geometry and product policy. For a multiple selection, the host also
/// supplies explicit uniform Width and Height property states; mixed or
/// unresolved geometry cannot safely share one corner-radius range. Stale,
/// Page, empty, or duplicate targets remain stored but never render or emit.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignDrawAppearanceViewData {
    pub target: DesignPanelTarget,
    pub corner_radius_range: DesignDrawSliderRange,
}

impl DesignDrawAppearanceViewData {
    pub const fn new(
        target: DesignPanelTarget,
        corner_radius_range: DesignDrawSliderRange,
    ) -> Self {
        Self {
            target,
            corner_radius_range,
        }
    }

    pub fn is_valid(&self) -> bool {
        let DesignPanelTarget::Nodes { node_ids } = &self.target else {
            return false;
        };
        let unique_ids = node_ids
            .iter()
            .map(SharedString::as_ref)
            .collect::<HashSet<_>>();
        !node_ids.is_empty()
            && unique_ids.len() == node_ids.len()
            && node_ids.iter().all(|node_id| !node_id.trim().is_empty())
    }
}

/// One immutable Figma inspector-menu hover candidate.
///
/// The payload deliberately repeats the original and candidate values on the
/// matching [`DesignMenuPreviewPhase::End`] event. Hosts can therefore unwind
/// exactly the preview that began even after a controlled selection, effect,
/// or paint echo has invalidated the panel's current indices.
#[derive(Clone, Debug, PartialEq)]
pub enum DesignMenuPreview {
    /// A node property resolved against the exact ordered page selection.
    NodeProperty {
        target: DesignPanelTarget,
        property: DesignPanelProperty,
        original: DesignPanelValue,
        candidate: DesignPanelValue,
    },
    /// An effect property resolved by stable effect ID with an index fallback
    /// for legacy hosts that do not yet supply IDs.
    EffectProperty {
        node_id: SharedString,
        effect_id: SharedString,
        index: usize,
        property: DesignPanelProperty,
        original: DesignPanelValue,
        candidate: DesignPanelValue,
    },
    /// A paint property resolved by exact text/object scope, stable paint ID,
    /// collection, and compatibility index.
    PaintProperty {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        property: DesignPaintProperty,
        original: DesignPaintValue,
        candidate: DesignPaintValue,
    },
}

/// Balanced lifecycle for an inspector-menu hover preview.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignMenuPreviewPhase {
    Begin,
    End,
}

/// Host-owned availability for Figma's native “Add auto layout” operation.
///
/// Structural eligibility cannot be inferred from an aggregate inspector node:
/// groups and arbitrary layer selections may be eligible even though neither
/// owns an auto-layout flow yet. The target binds an asynchronous projection to
/// the exact ordered selection that was evaluated. `disabled_reason` keeps an
/// otherwise eligible affordance visible but inert, for example while one
/// selected layer is locked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignAddAutoLayoutViewData {
    pub target: DesignPanelTarget,
    pub structurally_eligible: bool,
    pub disabled_reason: Option<SharedString>,
}

impl DesignAddAutoLayoutViewData {
    pub const fn eligible(target: DesignPanelTarget) -> Self {
        Self {
            target,
            structurally_eligible: true,
            disabled_reason: None,
        }
    }

    pub const fn ineligible(target: DesignPanelTarget) -> Self {
        Self {
            target,
            structurally_eligible: false,
            disabled_reason: None,
        }
    }

    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.disabled_reason = Some(reason.into());
        self
    }

    pub fn is_valid(&self) -> bool {
        let DesignPanelTarget::Nodes { node_ids } = &self.target else {
            return false;
        };
        let unique_ids = node_ids
            .iter()
            .map(SharedString::as_ref)
            .collect::<HashSet<_>>();
        !node_ids.is_empty()
            && unique_ids.len() == node_ids.len()
            && node_ids.iter().all(|node_id| !node_id.is_empty())
            && self
                .disabled_reason
                .as_ref()
                .is_none_or(|reason| !reason.is_empty())
    }

    pub const fn can_request(&self) -> bool {
        self.structurally_eligible && self.disabled_reason.is_none()
    }
}

/// Color/code representation selected for a view-only Properties section.
///
/// Figma exposes these five representations for Borders. The selected value
/// remains host owned because switching representations changes the exact
/// displayed and copied section summary.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignViewerColorRepresentation {
    Css,
    Hex,
    Rgb,
    Hsl,
    Hsb,
}

impl DesignViewerColorRepresentation {
    pub const ALL: [Self; 5] = [Self::Css, Self::Hex, Self::Rgb, Self::Hsl, Self::Hsb];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Css => "CSS",
            Self::Hex => "Hex",
            Self::Rgb => "RGB",
            Self::Hsl => "HSL",
            Self::Hsb => "HSB",
        }
    }
}

/// One exact host-formatted readout in the view-only Properties projection.
///
/// When `property` is present, activating the row reuses
/// [`DesignPanelAction::PropertyCopyRequested`]. Rows without a canonical
/// [`DesignPanelProperty`] remain inspectable and are copied through their
/// containing section's explicit copy action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignViewerPropertyRow {
    pub id: SharedString,
    pub label: SharedString,
    pub displayed_value: SharedString,
    pub property: Option<DesignPanelProperty>,
}

impl DesignViewerPropertyRow {
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        displayed_value: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            displayed_value: displayed_value.into(),
            property: None,
        }
    }

    pub const fn with_property(mut self, property: DesignPanelProperty) -> Self {
        self.property = Some(property);
        self
    }
}

/// One stable, host-owned section in Figma's view-only Properties projection.
///
/// `summary` supports multiline content such as a Text layer's characters.
/// `copy_value` is the exact payload a host should place on the clipboard; it
/// intentionally need not equal any one displayed row. When
/// `color_representation` is present, the panel offers all five Figma viewer
/// representations and requests a host echo before changing the section.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignViewerPropertySection {
    pub id: SharedString,
    pub title: SharedString,
    pub summary: Option<SharedString>,
    pub rows: Vec<DesignViewerPropertyRow>,
    pub copy_label: SharedString,
    pub copy_value: Option<SharedString>,
    pub color_representation: Option<DesignViewerColorRepresentation>,
}

impl DesignViewerPropertySection {
    pub fn new(
        id: impl Into<SharedString>,
        title: impl Into<SharedString>,
        rows: impl IntoIterator<Item = DesignViewerPropertyRow>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: None,
            rows: rows.into_iter().collect(),
            copy_label: "Copy".into(),
            copy_value: None,
            color_representation: None,
        }
    }

    pub fn text_content(id: impl Into<SharedString>, content: impl Into<SharedString>) -> Self {
        let content = content.into();
        Self::new(id, "Content", [])
            .with_summary(content.clone())
            .with_copy_value(content)
    }

    pub fn with_summary(mut self, summary: impl Into<SharedString>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    pub fn with_copy_value(mut self, copy_value: impl Into<SharedString>) -> Self {
        self.copy_value = Some(copy_value.into());
        self
    }

    pub fn with_copy_label(mut self, copy_label: impl Into<SharedString>) -> Self {
        self.copy_label = copy_label.into();
        self
    }

    pub fn with_copy_all(self) -> Self {
        self.with_copy_label("Copy all")
    }

    pub const fn with_color_representation(
        mut self,
        representation: DesignViewerColorRepresentation,
    ) -> Self {
        self.color_representation = Some(representation);
        self
    }

    pub fn row(&self, row_id: &str) -> Option<&DesignViewerPropertyRow> {
        self.rows.iter().find(|row| row.id.as_ref() == row_id)
    }
}

/// Exact viewer-only Properties projection for one ordered selection target.
///
/// The panel never derives Content, CSS, or color conversions from editor
/// fields. Hosts supply the strings they want displayed and copied, then echo
/// a fresh snapshot after a representation request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignViewerPropertiesViewData {
    pub target: DesignPanelTarget,
    pub sections: Vec<DesignViewerPropertySection>,
}

impl DesignViewerPropertiesViewData {
    pub fn new(
        target: DesignPanelTarget,
        sections: impl IntoIterator<Item = DesignViewerPropertySection>,
    ) -> Self {
        Self {
            target,
            sections: sections.into_iter().collect(),
        }
    }

    pub fn section(&self, section_id: &str) -> Option<&DesignViewerPropertySection> {
        self.sections
            .iter()
            .find(|section| section.id.as_ref() == section_id)
    }

    pub fn is_valid(&self) -> bool {
        let section_ids = self
            .sections
            .iter()
            .map(|section| section.id.as_ref())
            .collect::<HashSet<_>>();
        section_ids.len() == self.sections.len()
            && self.sections.iter().all(|section| {
                !section.id.is_empty()
                    && !section.title.is_empty()
                    && !section.copy_label.is_empty()
                    && section
                        .rows
                        .iter()
                        .map(|row| row.id.as_ref())
                        .collect::<HashSet<_>>()
                        .len()
                        == section.rows.len()
                    && section
                        .rows
                        .iter()
                        .all(|row| !row.id.is_empty() && !row.label.is_empty())
            })
    }
}

/// Top-level resource browser shown in the Page/no-selection inspector.
#[deprecated(
    note = "use DesignPageLocalStylesViewData; canonical Page styles are current-file-only"
)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignLocalResourceCategory {
    Styles,
    VariableCollections,
}

impl DesignLocalResourceCategory {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Styles => "Local styles",
            Self::VariableCollections => "Local variables",
        }
    }
}

/// One concrete Figma resource kind that can be created from the Page panel.
#[deprecated(note = "use DesignLocalStyleKind for canonical current-file Page styles")]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignLocalResourceKind {
    PaintStyle,
    TextStyle,
    EffectStyle,
    GridStyle,
    VariableCollection,
}

impl DesignLocalResourceKind {
    pub const fn category(self) -> DesignLocalResourceCategory {
        match self {
            Self::PaintStyle | Self::TextStyle | Self::EffectStyle | Self::GridStyle => {
                DesignLocalResourceCategory::Styles
            }
            Self::VariableCollection => DesignLocalResourceCategory::VariableCollections,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::PaintStyle => "Paint style",
            Self::TextStyle => "Text style",
            Self::EffectStyle => "Effect style",
            Self::GridStyle => "Grid style",
            Self::VariableCollection => "Variable collection",
        }
    }
}

/// Stable origin of a local/importable style or variable collection.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DesignLocalResourceSource {
    Local,
    Library {
        library_id: SharedString,
        library_name: SharedString,
    },
}

impl DesignLocalResourceSource {
    pub fn library(
        library_id: impl Into<SharedString>,
        library_name: impl Into<SharedString>,
    ) -> Self {
        Self::Library {
            library_id: library_id.into(),
            library_name: library_name.into(),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Local => "This file",
            Self::Library { library_name, .. } => library_name.as_ref(),
        }
    }
}

/// Whether a resource can be opened now, imported, or only inspected.
#[deprecated(note = "the canonical Page local-style tree does not browse or import libraries")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DesignLocalResourceAvailability {
    Imported,
    Available,
    Unavailable { reason: SharedString },
}

impl DesignLocalResourceAvailability {
    pub const fn can_open(&self) -> bool {
        matches!(self, Self::Imported)
    }

    pub const fn can_import(&self) -> bool {
        matches!(self, Self::Available)
    }

    pub const fn disabled_reason(&self) -> Option<&SharedString> {
        match self {
            Self::Unavailable { reason } => Some(reason),
            Self::Imported | Self::Available => None,
        }
    }
}

#[deprecated(note = "use DesignLocalStyleItem")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignLocalResource {
    pub id: SharedString,
    pub name: SharedString,
    pub kind: DesignLocalResourceKind,
    pub availability: DesignLocalResourceAvailability,
}

impl DesignLocalResource {
    pub fn local(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        kind: DesignLocalResourceKind,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind,
            availability: DesignLocalResourceAvailability::Imported,
        }
    }

    pub fn available(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        kind: DesignLocalResourceKind,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind,
            availability: DesignLocalResourceAvailability::Available,
        }
    }

    pub fn unavailable(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        kind: DesignLocalResourceKind,
        reason: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind,
            availability: DesignLocalResourceAvailability::Unavailable {
                reason: reason.into(),
            },
        }
    }
}

#[deprecated(note = "use DesignLocalStyleTarget")]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DesignLocalResourceSelection {
    pub group_id: SharedString,
    pub source: DesignLocalResourceSource,
    pub resource_id: SharedString,
}

#[deprecated(note = "use nested DesignLocalStyleEntry folders")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignLocalResourceGroup {
    pub id: SharedString,
    pub name: SharedString,
    pub source: DesignLocalResourceSource,
    pub resources: Vec<DesignLocalResource>,
}

impl DesignLocalResourceGroup {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        source: DesignLocalResourceSource,
        resources: impl IntoIterator<Item = DesignLocalResource>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            source,
            resources: resources.into_iter().collect(),
        }
    }

    pub fn selection(&self, resource_id: impl Into<SharedString>) -> DesignLocalResourceSelection {
        DesignLocalResourceSelection {
            group_id: self.id.clone(),
            source: self.source.clone(),
            resource_id: resource_id.into(),
        }
    }
}

#[deprecated(note = "use DesignPageLocalStylesViewData")]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DesignLocalResourceViewData {
    pub groups: Vec<DesignLocalResourceGroup>,
}

impl DesignLocalResourceViewData {
    pub fn new(groups: impl IntoIterator<Item = DesignLocalResourceGroup>) -> Self {
        Self {
            groups: groups.into_iter().collect(),
        }
    }

    pub fn resource(
        &self,
        selection: &DesignLocalResourceSelection,
    ) -> Option<&DesignLocalResource> {
        self.groups
            .iter()
            .find(|group| group.id == selection.group_id && group.source == selection.source)?
            .resources
            .iter()
            .find(|resource| resource.id == selection.resource_id)
    }
}

/// Canonical current-file style families shown by Figma's Page/no-selection
/// Design inspector.
///
/// The order is deliberate and matches the native Page surface. Library
/// discovery and importing belong to the contextual style pickers, not this
/// current-file tree.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignLocalStyleKind {
    Text,
    Color,
    Effect,
    LayoutGuide,
}

impl DesignLocalStyleKind {
    pub const ALL: [Self; 4] = [Self::Text, Self::Color, Self::Effect, Self::LayoutGuide];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Text => "Text styles",
            Self::Color => "Color styles",
            Self::Effect => "Effect styles",
            Self::LayoutGuide => "Layout guide styles",
        }
    }
}

/// Host-authored visual summary for one local style.
///
/// These snapshots are display-only. Activating a row emits an exact style
/// target and never applies the preview data to a document node.
#[derive(Clone, Debug, PartialEq)]
pub enum DesignLocalStylePreview {
    Text(DesignTypographyStyle),
    Color(Vec<DesignPaint>),
    Effect(Vec<DesignEffect>),
    LayoutGuide(Vec<DesignLayoutGrid>),
}

impl DesignLocalStylePreview {
    pub const fn kind(&self) -> DesignLocalStyleKind {
        match self {
            Self::Text(_) => DesignLocalStyleKind::Text,
            Self::Color(_) => DesignLocalStyleKind::Color,
            Self::Effect(_) => DesignLocalStyleKind::Effect,
            Self::LayoutGuide(_) => DesignLocalStyleKind::LayoutGuide,
        }
    }
}

/// One current-file style leaf in the Page inspector.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignLocalStyleItem {
    pub id: SharedString,
    pub name: SharedString,
    pub description: SharedString,
    pub preview: DesignLocalStylePreview,
    pub disabled_reason: Option<SharedString>,
}

impl DesignLocalStyleItem {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        preview: DesignLocalStylePreview,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: SharedString::default(),
            preview,
            disabled_reason: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = description.into();
        self
    }

    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.disabled_reason = Some(reason.into());
        self
    }
}

/// One recursively nested current-file style tree entry.
#[derive(Clone, Debug, PartialEq)]
pub enum DesignLocalStyleEntry {
    Folder {
        id: SharedString,
        name: SharedString,
        disabled_reason: Option<SharedString>,
        entries: Vec<DesignLocalStyleEntry>,
    },
    Style(DesignLocalStyleItem),
}

impl DesignLocalStyleEntry {
    pub fn folder(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        entries: impl IntoIterator<Item = DesignLocalStyleEntry>,
    ) -> Self {
        Self::Folder {
            id: id.into(),
            name: name.into(),
            disabled_reason: None,
            entries: entries.into_iter().collect(),
        }
    }

    pub fn style(item: DesignLocalStyleItem) -> Self {
        Self::Style(item)
    }

    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        match &mut self {
            Self::Folder {
                disabled_reason, ..
            } => *disabled_reason = Some(reason.into()),
            Self::Style(item) => item.disabled_reason = Some(reason.into()),
        }
        self
    }

    pub const fn id(&self) -> &SharedString {
        match self {
            Self::Folder { id, .. } => id,
            Self::Style(item) => &item.id,
        }
    }

    pub const fn disabled_reason(&self) -> Option<&SharedString> {
        match self {
            Self::Folder {
                disabled_reason, ..
            } => disabled_reason.as_ref(),
            Self::Style(item) => item.disabled_reason.as_ref(),
        }
    }
}

/// One canonical style-family section in a Page local-style tree.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignLocalStyleSection {
    pub kind: DesignLocalStyleKind,
    pub entries: Vec<DesignLocalStyleEntry>,
}

impl DesignLocalStyleSection {
    pub fn new(
        kind: DesignLocalStyleKind,
        entries: impl IntoIterator<Item = DesignLocalStyleEntry>,
    ) -> Self {
        Self {
            kind,
            entries: entries.into_iter().collect(),
        }
    }
}

/// Exact identity of one style at its current location.
///
/// `expected_index` counts both folders and styles in the direct parent. It
/// makes commands safe against a host echo that moved the same stable style
/// ID before the pointer event was delivered.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DesignLocalStyleTarget {
    pub page_id: SharedString,
    pub kind: DesignLocalStyleKind,
    pub style_id: SharedString,
    pub parent_folder_id: Option<SharedString>,
    pub expected_index: usize,
}

impl DesignLocalStyleTarget {
    pub fn new(
        page_id: impl Into<SharedString>,
        kind: DesignLocalStyleKind,
        style_id: impl Into<SharedString>,
        parent_folder_id: Option<SharedString>,
        expected_index: usize,
    ) -> Self {
        Self {
            page_id: page_id.into(),
            kind,
            style_id: style_id.into(),
            parent_folder_id,
            expected_index,
        }
    }
}

/// One exact insertion point in a current-file style tree.
///
/// Both neighbors are carried because an index alone can silently retarget
/// after a concurrent host reorder.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DesignLocalStyleInsertion {
    pub kind: DesignLocalStyleKind,
    pub parent_folder_id: Option<SharedString>,
    pub expected_index: usize,
    pub expected_before_id: Option<SharedString>,
    pub expected_after_id: Option<SharedString>,
}

impl DesignLocalStyleInsertion {
    pub fn new(
        kind: DesignLocalStyleKind,
        parent_folder_id: Option<SharedString>,
        expected_index: usize,
        expected_before_id: Option<SharedString>,
        expected_after_id: Option<SharedString>,
    ) -> Self {
        Self {
            kind,
            parent_folder_id,
            expected_index,
            expected_before_id,
            expected_after_id,
        }
    }
}

/// Commands available for one exact local style.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignLocalStyleCommand {
    Edit,
    GoToDefinition,
    Copy,
    Duplicate,
}

impl DesignLocalStyleCommand {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Edit => "Edit style",
            Self::GoToDefinition => "Go to definition",
            Self::Copy => "Copy style",
            Self::Duplicate => "Duplicate style",
        }
    }
}

/// Host-owned canonical current-file style projection for one exact Page.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignPageLocalStylesViewData {
    pub target: DesignPanelTarget,
    pub sections: Vec<DesignLocalStyleSection>,
    pub create_disabled_reason: Option<SharedString>,
}

impl DesignPageLocalStylesViewData {
    pub fn new(
        target: DesignPanelTarget,
        sections: impl IntoIterator<Item = DesignLocalStyleSection>,
    ) -> Self {
        Self {
            target,
            sections: sections.into_iter().collect(),
            create_disabled_reason: None,
        }
    }

    pub fn for_page(
        page_id: impl Into<SharedString>,
        sections: impl IntoIterator<Item = DesignLocalStyleSection>,
    ) -> Self {
        Self::new(
            DesignPanelTarget::Page {
                page_id: page_id.into(),
            },
            sections,
        )
    }

    pub fn with_create_disabled_reason(mut self, reason: impl Into<SharedString>) -> Self {
        self.create_disabled_reason = Some(reason.into());
        self
    }

    pub fn page_id(&self) -> Option<&SharedString> {
        match &self.target {
            DesignPanelTarget::Page { page_id } if !page_id.is_empty() => Some(page_id),
            DesignPanelTarget::Page { .. } | DesignPanelTarget::Nodes { .. } => None,
        }
    }

    pub fn section(&self, kind: DesignLocalStyleKind) -> Option<&DesignLocalStyleSection> {
        self.sections.iter().find(|section| section.kind == kind)
    }

    /// Returns supplied sections in Figma's canonical family order.
    pub fn ordered_sections(&self) -> impl Iterator<Item = &DesignLocalStyleSection> {
        DesignLocalStyleKind::ALL
            .into_iter()
            .filter_map(|kind| self.section(kind))
    }

    pub fn entries(
        &self,
        kind: DesignLocalStyleKind,
        parent_folder_id: Option<&str>,
    ) -> Option<&[DesignLocalStyleEntry]> {
        let section = self.section(kind)?;
        match parent_folder_id {
            None => Some(section.entries.as_slice()),
            Some(folder_id) => {
                fn find<'a>(
                    entries: &'a [DesignLocalStyleEntry],
                    folder_id: &str,
                ) -> Option<&'a [DesignLocalStyleEntry]> {
                    for entry in entries {
                        if let DesignLocalStyleEntry::Folder {
                            id,
                            entries: children,
                            ..
                        } = entry
                        {
                            if id.as_ref() == folder_id {
                                return Some(children.as_slice());
                            }
                            if let Some(found) = find(children, folder_id) {
                                return Some(found);
                            }
                        }
                    }
                    None
                }
                find(&section.entries, folder_id)
            }
        }
    }

    pub fn folder(
        &self,
        kind: DesignLocalStyleKind,
        folder_id: &str,
    ) -> Option<&DesignLocalStyleEntry> {
        fn find<'a>(
            entries: &'a [DesignLocalStyleEntry],
            folder_id: &str,
        ) -> Option<&'a DesignLocalStyleEntry> {
            for entry in entries {
                if let DesignLocalStyleEntry::Folder {
                    id,
                    entries: children,
                    ..
                } = entry
                {
                    if id.as_ref() == folder_id {
                        return Some(entry);
                    }
                    if let Some(found) = find(children, folder_id) {
                        return Some(found);
                    }
                }
            }
            None
        }
        find(&self.section(kind)?.entries, folder_id)
    }

    /// Whether an entry and every containing folder are enabled.
    pub fn entry_path_is_enabled(&self, kind: DesignLocalStyleKind, entry_id: &str) -> bool {
        fn find(
            entries: &[DesignLocalStyleEntry],
            entry_id: &str,
            ancestors_enabled: bool,
        ) -> Option<bool> {
            for entry in entries {
                let enabled = ancestors_enabled && entry.disabled_reason().is_none();
                if entry.id().as_ref() == entry_id {
                    return Some(enabled);
                }
                if let DesignLocalStyleEntry::Folder {
                    entries: children, ..
                } = entry
                    && let Some(found) = find(children, entry_id, enabled)
                {
                    return Some(found);
                }
            }
            None
        }
        self.section(kind)
            .and_then(|section| find(&section.entries, entry_id, true))
            .unwrap_or(false)
    }

    pub fn resolve_style(&self, target: &DesignLocalStyleTarget) -> Option<&DesignLocalStyleItem> {
        if self.page_id()? != &target.page_id {
            return None;
        }
        let entries = self.entries(
            target.kind,
            target.parent_folder_id.as_ref().map(|id| id.as_ref()),
        )?;
        match entries.get(target.expected_index)? {
            DesignLocalStyleEntry::Style(item) if item.id == target.style_id => Some(item),
            DesignLocalStyleEntry::Folder { .. } | DesignLocalStyleEntry::Style(_) => None,
        }
    }

    /// Validates target shape, globally unique stable IDs, unique family
    /// sections, and exact preview/family agreement.
    pub fn is_valid(&self) -> bool {
        if self.page_id().is_none() {
            return false;
        }
        let section_kinds = self
            .sections
            .iter()
            .map(|section| section.kind)
            .collect::<HashSet<_>>();
        if section_kinds.len() != self.sections.len() {
            return false;
        }

        fn validate_entries(
            entries: &[DesignLocalStyleEntry],
            kind: DesignLocalStyleKind,
            ids: &mut HashSet<SharedString>,
        ) -> bool {
            entries.iter().all(|entry| {
                if entry.id().is_empty() || !ids.insert(entry.id().clone()) {
                    return false;
                }
                match entry {
                    DesignLocalStyleEntry::Folder {
                        entries: children, ..
                    } => validate_entries(children, kind, ids),
                    DesignLocalStyleEntry::Style(item) => item.preview.kind() == kind,
                }
            })
        }

        let mut ids = HashSet::new();
        self.sections
            .iter()
            .all(|section| validate_entries(&section.entries, section.kind, &mut ids))
    }
}

/// Whether the historic Variables shortcut is projected in the right
/// sidebar. Figma's canonical default exposes Variables from the navigation
/// bar, so the Page surface omits a variables row unless a host explicitly
/// requests the compatibility entry point.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum DesignVariablesEntryPoint {
    #[default]
    NavigationBarOnly,
    LegacyRightSidebar {
        disabled_reason: Option<SharedString>,
    },
}

/// Canonical single solid paint exposed by `PageNode.backgrounds`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignPageBackground {
    pub color: DesignColor,
    pub read_only: bool,
    pub disabled_reason: Option<SharedString>,
}

impl DesignPageBackground {
    pub const fn new(color: DesignColor) -> Self {
        Self {
            color,
            read_only: false,
            disabled_reason: None,
        }
    }

    pub fn read_only(mut self, reason: impl Into<SharedString>) -> Self {
        self.read_only = true;
        self.disabled_reason = Some(reason.into());
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignPageViewData {
    pub page_id: SharedString,
    pub background: DesignPageBackground,
    #[deprecated(note = "set DesignPageLocalStylesViewData on DesignPanel instead")]
    pub local_resources: DesignLocalResourceViewData,
}

impl DesignPageViewData {
    pub fn new(
        page_id: impl Into<SharedString>,
        background: DesignPageBackground,
        local_resources: DesignLocalResourceViewData,
    ) -> Self {
        Self {
            page_id: page_id.into(),
            background,
            local_resources,
        }
    }

    /// Canonical Page projection without the deprecated resource browser.
    pub fn canonical(page_id: impl Into<SharedString>, background: DesignPageBackground) -> Self {
        Self::new(page_id, background, DesignLocalResourceViewData::default())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignVariableMode {
    pub id: SharedString,
    pub name: SharedString,
    pub disabled_reason: Option<SharedString>,
}

impl DesignVariableMode {
    pub fn new(id: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            disabled_reason: None,
        }
    }

    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.disabled_reason = Some(reason.into());
        self
    }
}

/// One variable collection's default, resolved, and explicitly selected mode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignVariableModeCollection {
    pub id: SharedString,
    pub name: SharedString,
    pub source: DesignLocalResourceSource,
    pub modes: Vec<DesignVariableMode>,
    pub default_mode_id: SharedString,
    pub resolved_mode_id: SharedString,
    pub explicit_mode_id: Option<SharedString>,
    pub disabled_reason: Option<SharedString>,
}

impl DesignVariableModeCollection {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        source: DesignLocalResourceSource,
        modes: impl IntoIterator<Item = DesignVariableMode>,
        default_mode_id: impl Into<SharedString>,
        resolved_mode_id: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            source,
            modes: modes.into_iter().collect(),
            default_mode_id: default_mode_id.into(),
            resolved_mode_id: resolved_mode_id.into(),
            explicit_mode_id: None,
            disabled_reason: None,
        }
    }

    pub fn explicit(mut self, mode_id: impl Into<SharedString>) -> Self {
        let mode_id = mode_id.into();
        self.resolved_mode_id = mode_id.clone();
        self.explicit_mode_id = Some(mode_id);
        self
    }

    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.disabled_reason = Some(reason.into());
        self
    }

    pub fn mode(&self, mode_id: &str) -> Option<&DesignVariableMode> {
        self.modes.iter().find(|mode| mode.id.as_ref() == mode_id)
    }

    pub fn resolved_mode(&self) -> Option<&DesignVariableMode> {
        self.mode(self.resolved_mode_id.as_ref())
    }

    pub fn explicit_mode(&self) -> Option<&DesignVariableMode> {
        self.explicit_mode_id
            .as_ref()
            .and_then(|mode_id| self.mode(mode_id.as_ref()))
    }

    pub fn mode_disabled_reason(&self, mode_id: &str) -> Option<SharedString> {
        if let Some(reason) = &self.disabled_reason {
            return Some(reason.clone());
        }
        self.mode(mode_id)
            .and_then(|mode| mode.disabled_reason.clone())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignVariableModeViewData {
    pub target: DesignPanelTarget,
    pub collections: Vec<DesignVariableModeCollection>,
}

impl DesignVariableModeViewData {
    pub fn new(
        target: DesignPanelTarget,
        collections: impl IntoIterator<Item = DesignVariableModeCollection>,
    ) -> Self {
        Self {
            target,
            collections: collections.into_iter().collect(),
        }
    }

    pub fn collection(&self, collection_id: &str) -> Option<&DesignVariableModeCollection> {
        self.collections
            .iter()
            .find(|collection| collection.id.as_ref() == collection_id)
    }
}

/// One leaf operation exposed by the selected-node header.
///
/// Header controls are intentionally commands rather than document mutations:
/// the host decides how a command changes selection, edit mode, or document
/// data and then echoes a fresh inspection context.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DesignSelectionHeaderCommand {
    SelectMatchingLayers,
    CreateLink,
    ApplyTextContentVariable,
    CreateComponent,
    UseAsMask,
    Boolean(DesignBooleanOperation),
    Flatten,
    EditObject,
    TitleMenuItem { item_id: SharedString },
    HostDefined { command_id: SharedString },
}

impl DesignSelectionHeaderCommand {
    /// Default permission class for this command.
    ///
    /// Host-authored controls and menu items can override this default with
    /// [`DesignSelectionHeaderControl::with_access`] or
    /// [`DesignSelectionHeaderMenuItem::with_access`]. Built-in constructors
    /// retain these semantics.
    pub const fn default_access(&self) -> DesignSelectionHeaderCommandAccess {
        match self {
            Self::SelectMatchingLayers => DesignSelectionHeaderCommandAccess::ViewerSafe,
            Self::CreateLink
            | Self::ApplyTextContentVariable
            | Self::CreateComponent
            | Self::UseAsMask
            | Self::Boolean(_)
            | Self::Flatten
            | Self::EditObject
            | Self::TitleMenuItem { .. }
            | Self::HostDefined { .. } => DesignSelectionHeaderCommandAccess::EditRequired,
        }
    }

    /// Whether the command can change document data or enter an editing mode.
    ///
    /// Selecting matching layers is the only built-in command that remains
    /// available to a viewer. Hosts can still disable it explicitly.
    pub const fn requires_edit(&self) -> bool {
        self.default_access().requires_edit()
    }
}

/// Permission class for one selected-node header command leaf.
///
/// The default remains edit-required. Hosts should opt a custom direct
/// control, title-menu item, or menu leaf into `ViewerSafe` only when invoking
/// it cannot mutate the document or enter an editing mode.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DesignSelectionHeaderCommandAccess {
    ViewerSafe,
    #[default]
    EditRequired,
}

impl DesignSelectionHeaderCommandAccess {
    pub const fn requires_edit(self) -> bool {
        matches!(self, Self::EditRequired)
    }
}

/// Visual/semantic role of an ordered selected-node header control.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignSelectionHeaderControlKind {
    SelectMatchingLayers,
    CreateLink,
    ApplyTextContentVariable,
    CreateComponent,
    UseAsMask,
    BooleanFlattenMenu,
    EditObject,
    HostDefined,
}

impl DesignSelectionHeaderControlKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::SelectMatchingLayers => "Select matching layers",
            Self::CreateLink => "Create link",
            Self::ApplyTextContentVariable => "Apply variable to text content",
            Self::CreateComponent => "Create component",
            Self::UseAsMask => "Use as mask",
            Self::BooleanFlattenMenu => "Boolean operations and flatten",
            Self::EditObject => "Edit object",
            Self::HostDefined => "More action",
        }
    }

    pub const fn is_menu(self) -> bool {
        matches!(self, Self::BooleanFlattenMenu)
    }
}

/// One enabled or disabled leaf in a selected-node header menu.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignSelectionHeaderMenuItem {
    pub id: SharedString,
    pub label: SharedString,
    pub command: DesignSelectionHeaderCommand,
    pub access: DesignSelectionHeaderCommandAccess,
    pub enabled: bool,
    pub disabled_reason: Option<SharedString>,
}

impl DesignSelectionHeaderMenuItem {
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        command: DesignSelectionHeaderCommand,
    ) -> Self {
        let access = command.default_access();
        Self {
            id: id.into(),
            label: label.into(),
            command,
            access,
            enabled: true,
            disabled_reason: None,
        }
    }

    pub const fn with_access(mut self, access: DesignSelectionHeaderCommandAccess) -> Self {
        self.access = access;
        self
    }

    pub const fn viewer_safe(self) -> Self {
        self.with_access(DesignSelectionHeaderCommandAccess::ViewerSafe)
    }

    pub const fn edit_required(self) -> Self {
        self.with_access(DesignSelectionHeaderCommandAccess::EditRequired)
    }

    /// Access used by the panel after preserving built-in command semantics.
    pub const fn effective_access(&self) -> DesignSelectionHeaderCommandAccess {
        match &self.command {
            DesignSelectionHeaderCommand::TitleMenuItem { .. }
            | DesignSelectionHeaderCommand::HostDefined { .. } => self.access,
            _ => self.command.default_access(),
        }
    }

    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.enabled = false;
        self.disabled_reason = Some(reason.into());
        self
    }
}

/// Ordered menu presented by the selected-node title.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DesignSelectionHeaderMenu {
    pub id: SharedString,
    pub items: Vec<DesignSelectionHeaderMenuItem>,
}

impl DesignSelectionHeaderMenu {
    pub fn new(
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = DesignSelectionHeaderMenuItem>,
    ) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().collect(),
        }
    }
}

/// Host-controlled icon presentation for one selected-node header control.
///
/// `Default` preserves the icon implied by the built-in control kind. The
/// remaining variants let an integration reproduce contextual or plugin
/// controls without pretending they are one of the built-in commands.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum DesignSelectionHeaderControlIcon {
    #[default]
    Default,
    Ellipsis,
    Plus,
    Minus,
    Check,
    Search,
    Settings,
    File,
    Inspector,
    Layout,
    /// A compact host-supplied glyph such as a plugin monogram.
    Glyph(SharedString),
}

/// One ordered primary or overflow control in the selected-node header.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignSelectionHeaderControl {
    pub id: SharedString,
    pub kind: DesignSelectionHeaderControlKind,
    pub icon: DesignSelectionHeaderControlIcon,
    pub tooltip: SharedString,
    /// Permission class for this control's direct command. Menu-bearing
    /// controls use the access class on each menu item instead.
    pub access: DesignSelectionHeaderCommandAccess,
    pub enabled: bool,
    pub disabled_reason: Option<SharedString>,
    /// Leaf items for menu-bearing controls. Direct controls leave this empty.
    pub menu_items: Vec<DesignSelectionHeaderMenuItem>,
}

impl DesignSelectionHeaderControl {
    pub fn new(id: impl Into<SharedString>, kind: DesignSelectionHeaderControlKind) -> Self {
        let access = match kind {
            DesignSelectionHeaderControlKind::SelectMatchingLayers => {
                DesignSelectionHeaderCommandAccess::ViewerSafe
            }
            DesignSelectionHeaderControlKind::CreateLink
            | DesignSelectionHeaderControlKind::ApplyTextContentVariable
            | DesignSelectionHeaderControlKind::CreateComponent
            | DesignSelectionHeaderControlKind::UseAsMask
            | DesignSelectionHeaderControlKind::BooleanFlattenMenu
            | DesignSelectionHeaderControlKind::EditObject
            | DesignSelectionHeaderControlKind::HostDefined => {
                DesignSelectionHeaderCommandAccess::EditRequired
            }
        };
        Self {
            id: id.into(),
            kind,
            icon: DesignSelectionHeaderControlIcon::Default,
            tooltip: kind.label().into(),
            access,
            enabled: true,
            disabled_reason: None,
            menu_items: Vec::new(),
        }
    }

    pub fn with_tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = tooltip.into();
        self
    }

    pub fn with_icon(mut self, icon: DesignSelectionHeaderControlIcon) -> Self {
        self.icon = icon;
        self
    }

    pub const fn with_access(mut self, access: DesignSelectionHeaderCommandAccess) -> Self {
        self.access = access;
        self
    }

    pub const fn viewer_safe(self) -> Self {
        self.with_access(DesignSelectionHeaderCommandAccess::ViewerSafe)
    }

    pub const fn edit_required(self) -> Self {
        self.with_access(DesignSelectionHeaderCommandAccess::EditRequired)
    }

    pub fn with_menu_items(
        mut self,
        items: impl IntoIterator<Item = DesignSelectionHeaderMenuItem>,
    ) -> Self {
        self.menu_items = items.into_iter().collect();
        self
    }

    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.enabled = false;
        self.disabled_reason = Some(reason.into());
        self
    }

    pub fn command(&self) -> Option<DesignSelectionHeaderCommand> {
        if self.is_menu() {
            return None;
        }
        match self.kind {
            DesignSelectionHeaderControlKind::SelectMatchingLayers => {
                Some(DesignSelectionHeaderCommand::SelectMatchingLayers)
            }
            DesignSelectionHeaderControlKind::CreateLink => {
                Some(DesignSelectionHeaderCommand::CreateLink)
            }
            DesignSelectionHeaderControlKind::ApplyTextContentVariable => {
                Some(DesignSelectionHeaderCommand::ApplyTextContentVariable)
            }
            DesignSelectionHeaderControlKind::CreateComponent => {
                Some(DesignSelectionHeaderCommand::CreateComponent)
            }
            DesignSelectionHeaderControlKind::UseAsMask => {
                Some(DesignSelectionHeaderCommand::UseAsMask)
            }
            DesignSelectionHeaderControlKind::BooleanFlattenMenu => None,
            DesignSelectionHeaderControlKind::EditObject => {
                Some(DesignSelectionHeaderCommand::EditObject)
            }
            DesignSelectionHeaderControlKind::HostDefined => {
                Some(DesignSelectionHeaderCommand::HostDefined {
                    command_id: self.id.clone(),
                })
            }
        }
    }

    /// Whether this control opens a menu instead of emitting a direct command.
    ///
    /// Built-in Boolean/Flatten controls are always menus. Any host-defined
    /// control becomes a menu as soon as the host supplies menu items.
    pub fn is_menu(&self) -> bool {
        self.kind.is_menu() || !self.menu_items.is_empty()
    }

    /// Access used by the panel after preserving built-in control semantics.
    pub fn effective_access(&self) -> DesignSelectionHeaderCommandAccess {
        if self.kind == DesignSelectionHeaderControlKind::HostDefined {
            self.access
        } else {
            self.command().as_ref().map_or(
                DesignSelectionHeaderCommandAccess::EditRequired,
                |command| command.default_access(),
            )
        }
    }

    pub fn direct(kind: DesignSelectionHeaderControlKind) -> Self {
        let id = match kind {
            DesignSelectionHeaderControlKind::SelectMatchingLayers => "select-matching-layers",
            DesignSelectionHeaderControlKind::CreateLink => "create-link",
            DesignSelectionHeaderControlKind::ApplyTextContentVariable => {
                "apply-text-content-variable"
            }
            DesignSelectionHeaderControlKind::CreateComponent => "create-component",
            DesignSelectionHeaderControlKind::UseAsMask => "use-as-mask",
            DesignSelectionHeaderControlKind::BooleanFlattenMenu => "boolean-flatten",
            DesignSelectionHeaderControlKind::EditObject => "edit-object",
            DesignSelectionHeaderControlKind::HostDefined => "host-defined",
        };
        Self::new(id, kind)
    }

    pub fn boolean_flatten_menu() -> Self {
        Self::direct(DesignSelectionHeaderControlKind::BooleanFlattenMenu).with_menu_items(
            DesignBooleanOperation::ALL
                .into_iter()
                .map(|operation| {
                    DesignSelectionHeaderMenuItem::new(
                        format!(
                            "boolean-{}",
                            operation.label().to_ascii_lowercase().replace(' ', "-")
                        ),
                        operation.label(),
                        DesignSelectionHeaderCommand::Boolean(operation),
                    )
                })
                .chain([DesignSelectionHeaderMenuItem::new(
                    "flatten",
                    "Flatten",
                    DesignSelectionHeaderCommand::Flatten,
                )]),
        )
    }
}

/// Complete host-owned selected-node header presentation.
///
/// The vectors retain host order. Supplying this value is authoritative:
/// the panel does not add kind-derived controls or move controls between the
/// primary row and the transient More menu.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesignSelectionHeaderViewData {
    pub title: SharedString,
    pub title_menu: Option<DesignSelectionHeaderMenu>,
    pub primary_controls: Vec<DesignSelectionHeaderControl>,
    pub overflow_controls: Vec<DesignSelectionHeaderControl>,
}

impl DesignSelectionHeaderViewData {
    pub fn new(
        title: impl Into<SharedString>,
        primary_controls: impl IntoIterator<Item = DesignSelectionHeaderControl>,
    ) -> Self {
        Self {
            title: title.into(),
            title_menu: None,
            primary_controls: primary_controls.into_iter().collect(),
            overflow_controls: Vec::new(),
        }
    }

    pub fn with_title_menu(mut self, menu: DesignSelectionHeaderMenu) -> Self {
        self.title_menu = Some(menu);
        self
    }

    pub fn with_overflow_controls(
        mut self,
        controls: impl IntoIterator<Item = DesignSelectionHeaderControl>,
    ) -> Self {
        self.overflow_controls = controls.into_iter().collect();
        self
    }

    /// Conservative compatibility header for an exact multiple selection.
    ///
    /// A multiple selection is not represented by any one member's node kind.
    /// In particular, borrowing the first member's preset can expose a Text
    /// content-variable command or a Frame title menu for an unrelated mixed
    /// selection. Hosts should supply exact target-bound data whenever they
    /// can resolve more contextual commands. This baseline intentionally keeps
    /// only Create component, which applies to arbitrary selected layers.
    pub fn for_multiple_selection(count: usize) -> Self {
        Self::new(
            format!("{count} layers"),
            [DesignSelectionHeaderControl::direct(
                DesignSelectionHeaderControlKind::CreateComponent,
            )],
        )
    }

    /// A representative fallback for stories and incremental integrations.
    ///
    /// Real Figma headers are contextual (for example Select matching layers
    /// depends on the current page). Hosts should supply exact view data when
    /// sibling availability, plugins, or document state affect the matrix.
    pub fn for_node_kind(kind: DesignPanelNodeKind) -> Self {
        use DesignSelectionHeaderControlKind as Kind;

        let direct = DesignSelectionHeaderControl::direct;
        let boolean = DesignSelectionHeaderControl::boolean_flatten_menu;
        let title = kind.label();
        match kind {
            DesignPanelNodeKind::Text | DesignPanelNodeKind::TextPath => Self::new(
                title,
                [
                    direct(Kind::SelectMatchingLayers),
                    direct(Kind::CreateLink),
                    direct(Kind::ApplyTextContentVariable),
                    direct(Kind::CreateComponent),
                ],
            )
            .with_overflow_controls([direct(Kind::UseAsMask), direct(Kind::EditObject)]),
            DesignPanelNodeKind::Frame => Self::new(
                title,
                [
                    direct(Kind::SelectMatchingLayers),
                    direct(Kind::CreateComponent),
                    direct(Kind::UseAsMask),
                    boolean(),
                ],
            )
            .with_title_menu(DesignSelectionHeaderMenu::new(
                "frame-type",
                ["frame", "group", "section"].into_iter().map(|id| {
                    let label = match id {
                        "frame" => "Frame",
                        "group" => "Group",
                        "section" => "Section",
                        _ => unreachable!(),
                    };
                    DesignSelectionHeaderMenuItem::new(
                        id,
                        label,
                        DesignSelectionHeaderCommand::TitleMenuItem { item_id: id.into() },
                    )
                }),
            )),
            DesignPanelNodeKind::Arrow | DesignPanelNodeKind::Line => Self::new(
                title,
                [
                    direct(Kind::SelectMatchingLayers),
                    direct(Kind::CreateComponent),
                    direct(Kind::UseAsMask),
                    boolean(),
                ],
            )
            .with_overflow_controls([direct(Kind::EditObject)]),
            DesignPanelNodeKind::Ellipse => Self::new(
                title,
                [
                    direct(Kind::CreateComponent),
                    direct(Kind::UseAsMask),
                    boolean(),
                    direct(Kind::EditObject),
                ],
            ),
            DesignPanelNodeKind::Widget => Self::new(title, []),
            _ => Self::new(
                title,
                [
                    direct(Kind::SelectMatchingLayers),
                    direct(Kind::CreateComponent),
                    direct(Kind::UseAsMask),
                    boolean(),
                    direct(Kind::EditObject),
                ],
            ),
        }
    }
}

/// Geometry class of an exact multiple selection for Figma Smart Selection.
///
/// `None` means the selection is not currently evenly spaced. A host can
/// still expose Tidy up or distribution controls that create a Smart
/// Selection. The other variants determine which Space between fields are
/// valid.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DesignSmartSelectionKind {
    #[default]
    None,
    Horizontal,
    Vertical,
    TwoDimensional,
}

impl DesignSmartSelectionKind {
    pub const fn supports_axis(self, axis: DesignSmartSelectionAxis) -> bool {
        matches!(
            (self, axis),
            (
                Self::Horizontal | Self::TwoDimensional,
                DesignSmartSelectionAxis::Horizontal
            ) | (
                Self::Vertical | Self::TwoDimensional,
                DesignSmartSelectionAxis::Vertical
            )
        )
    }
}

/// Axis of one native Smart Selection Space between field.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignSmartSelectionAxis {
    Horizontal,
    Vertical,
}

impl DesignSmartSelectionAxis {
    pub const ALL: [Self; 2] = [Self::Horizontal, Self::Vertical];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Horizontal => "Horizontal space between",
            Self::Vertical => "Vertical space between",
        }
    }

    pub const fn glyph(self) -> &'static str {
        match self {
            Self::Horizontal => "↔",
            Self::Vertical => "↕",
        }
    }
}

/// Host-owned value shown by one Smart Selection spacing field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DesignSmartSelectionSpacingValue {
    Mixed,
    Uniform(f32),
}

impl DesignSmartSelectionSpacingValue {
    pub const fn is_mixed(self) -> bool {
        matches!(self, Self::Mixed)
    }

    pub fn uniform(self) -> Option<f32> {
        match self {
            Self::Mixed => None,
            Self::Uniform(value) if value.is_finite() => Some(value),
            Self::Uniform(_) => None,
        }
    }

    pub fn is_valid(self) -> bool {
        matches!(self, Self::Mixed) || self.uniform().is_some()
    }
}

/// Host-authored availability and disabled explanation for a Smart Selection
/// field or command.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum DesignSmartSelectionAvailability {
    #[default]
    Available,
    Disabled {
        reason: SharedString,
    },
}

impl DesignSmartSelectionAvailability {
    pub const fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }

    pub const fn disabled_reason(&self) -> Option<&SharedString> {
        match self {
            Self::Available => None,
            Self::Disabled { reason } => Some(reason),
        }
    }

    pub fn disabled(reason: impl Into<SharedString>) -> Self {
        Self::Disabled {
            reason: reason.into(),
        }
    }

    fn is_valid(&self) -> bool {
        self.disabled_reason()
            .is_none_or(|reason| !reason.trim().is_empty())
    }
}

/// One host-controlled Smart Selection spacing readout.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignSmartSelectionSpacingViewData {
    pub value: DesignSmartSelectionSpacingValue,
    pub availability: DesignSmartSelectionAvailability,
}

impl DesignSmartSelectionSpacingViewData {
    pub const fn new(value: DesignSmartSelectionSpacingValue) -> Self {
        Self {
            value,
            availability: DesignSmartSelectionAvailability::Available,
        }
    }

    pub const fn uniform(value: f32) -> Self {
        Self::new(DesignSmartSelectionSpacingValue::Uniform(value))
    }

    pub const fn mixed() -> Self {
        Self::new(DesignSmartSelectionSpacingValue::Mixed)
    }

    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.availability = DesignSmartSelectionAvailability::disabled(reason);
        self
    }

    pub fn is_valid(&self) -> bool {
        self.value.is_valid() && self.availability.is_valid()
    }
}

/// Arrange command whose eligibility is resolved against one exact Smart
/// Selection snapshot.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignSmartSelectionOperation {
    DistributeHorizontal,
    DistributeVertical,
    TidyUp,
}

impl DesignSmartSelectionOperation {
    pub const ALL: [Self; 3] = [
        Self::DistributeHorizontal,
        Self::DistributeVertical,
        Self::TidyUp,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::DistributeHorizontal => "Distribute horizontal spacing",
            Self::DistributeVertical => "Distribute vertical spacing",
            Self::TidyUp => "Tidy up",
        }
    }
}

/// Exact ordered multiple-selection projection used by Smart Selection.
///
/// The target order is part of the snapshot identity. Optional command
/// availability controls whether a native arrange/tidy control is present;
/// a present disabled value remains visible and explains why it cannot run.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignSmartSelectionViewData {
    pub target: DesignPanelTarget,
    pub kind: DesignSmartSelectionKind,
    pub horizontal_spacing: Option<DesignSmartSelectionSpacingViewData>,
    pub vertical_spacing: Option<DesignSmartSelectionSpacingViewData>,
    pub distribute_horizontal: Option<DesignSmartSelectionAvailability>,
    pub distribute_vertical: Option<DesignSmartSelectionAvailability>,
    pub tidy_up: Option<DesignSmartSelectionAvailability>,
    pub read_only_reason: Option<SharedString>,
}

impl DesignSmartSelectionViewData {
    pub fn new(target: DesignPanelTarget, kind: DesignSmartSelectionKind) -> Self {
        Self {
            target,
            kind,
            horizontal_spacing: None,
            vertical_spacing: None,
            distribute_horizontal: None,
            distribute_vertical: None,
            tidy_up: None,
            read_only_reason: None,
        }
    }

    pub fn horizontal(
        target: DesignPanelTarget,
        spacing: DesignSmartSelectionSpacingViewData,
    ) -> Self {
        let mut view_data = Self::new(target, DesignSmartSelectionKind::Horizontal);
        view_data.horizontal_spacing = Some(spacing);
        view_data
    }

    pub fn vertical(
        target: DesignPanelTarget,
        spacing: DesignSmartSelectionSpacingViewData,
    ) -> Self {
        let mut view_data = Self::new(target, DesignSmartSelectionKind::Vertical);
        view_data.vertical_spacing = Some(spacing);
        view_data
    }

    pub fn two_dimensional(
        target: DesignPanelTarget,
        horizontal_spacing: DesignSmartSelectionSpacingViewData,
        vertical_spacing: DesignSmartSelectionSpacingViewData,
    ) -> Self {
        let mut view_data = Self::new(target, DesignSmartSelectionKind::TwoDimensional);
        view_data.horizontal_spacing = Some(horizontal_spacing);
        view_data.vertical_spacing = Some(vertical_spacing);
        view_data
    }

    pub fn with_operation(
        mut self,
        operation: DesignSmartSelectionOperation,
        availability: DesignSmartSelectionAvailability,
    ) -> Self {
        *self.operation_availability_mut(operation) = Some(availability);
        self
    }

    pub fn read_only(mut self, reason: impl Into<SharedString>) -> Self {
        self.read_only_reason = Some(reason.into());
        self
    }

    pub const fn spacing(
        &self,
        axis: DesignSmartSelectionAxis,
    ) -> Option<&DesignSmartSelectionSpacingViewData> {
        match axis {
            DesignSmartSelectionAxis::Horizontal => self.horizontal_spacing.as_ref(),
            DesignSmartSelectionAxis::Vertical => self.vertical_spacing.as_ref(),
        }
    }

    pub const fn operation_availability(
        &self,
        operation: DesignSmartSelectionOperation,
    ) -> Option<&DesignSmartSelectionAvailability> {
        match operation {
            DesignSmartSelectionOperation::DistributeHorizontal => {
                self.distribute_horizontal.as_ref()
            }
            DesignSmartSelectionOperation::DistributeVertical => self.distribute_vertical.as_ref(),
            DesignSmartSelectionOperation::TidyUp => self.tidy_up.as_ref(),
        }
    }

    fn operation_availability_mut(
        &mut self,
        operation: DesignSmartSelectionOperation,
    ) -> &mut Option<DesignSmartSelectionAvailability> {
        match operation {
            DesignSmartSelectionOperation::DistributeHorizontal => &mut self.distribute_horizontal,
            DesignSmartSelectionOperation::DistributeVertical => &mut self.distribute_vertical,
            DesignSmartSelectionOperation::TidyUp => &mut self.tidy_up,
        }
    }

    pub fn spacing_is_editable(&self, axis: DesignSmartSelectionAxis) -> bool {
        self.read_only_reason.is_none()
            && self
                .spacing(axis)
                .is_some_and(|spacing| spacing.availability.is_available())
    }

    pub fn operation_is_available(&self, operation: DesignSmartSelectionOperation) -> bool {
        self.read_only_reason.is_none()
            && self
                .operation_availability(operation)
                .is_some_and(DesignSmartSelectionAvailability::is_available)
    }

    pub fn is_valid(&self) -> bool {
        let DesignPanelTarget::Nodes { node_ids } = &self.target else {
            return false;
        };
        let mut unique_ids = HashSet::with_capacity(node_ids.len());
        if node_ids.len() < 2
            || node_ids
                .iter()
                .any(|node_id| node_id.trim().is_empty() || !unique_ids.insert(node_id))
            || self
                .read_only_reason
                .as_ref()
                .is_some_and(|reason| reason.trim().is_empty())
        {
            return false;
        }

        for axis in DesignSmartSelectionAxis::ALL {
            let spacing = self.spacing(axis);
            if self.kind.supports_axis(axis) != spacing.is_some()
                || spacing.is_some_and(|spacing| !spacing.is_valid())
            {
                return false;
            }
        }
        DesignSmartSelectionOperation::ALL
            .into_iter()
            .filter_map(|operation| self.operation_availability(operation))
            .all(DesignSmartSelectionAvailability::is_valid)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignArrangeOperation {
    AlignLeft,
    AlignHorizontalCenter,
    AlignRight,
    AlignTop,
    AlignVerticalCenter,
    AlignBottom,
    DistributeHorizontal,
    DistributeVertical,
    TidyUp,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignTransformOperation {
    RotateClockwise90,
    FlipHorizontal,
    FlipVertical,
}

/// Host-controlled cross-file preference for keyboard nudging in numeric
/// inspector fields.
///
/// Figma defaults to one resolution-independent point for a normal arrow key
/// and ten points while Shift is held. Construction rejects zero, negative,
/// NaN, and infinite values so a panel never has to repair host preferences.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesignNudgeSettings {
    small: f32,
    big: f32,
}

impl DesignNudgeSettings {
    pub const DEFAULT_SMALL: f32 = 1.;
    pub const DEFAULT_BIG: f32 = 10.;

    pub fn new(small: f32, big: f32) -> Option<Self> {
        (small.is_finite() && small > 0. && big.is_finite() && big > 0.)
            .then_some(Self { small, big })
    }

    pub const fn small(self) -> f32 {
        self.small
    }

    pub const fn big(self) -> f32 {
        self.big
    }

    pub const fn amount(self, big: bool) -> f32 {
        if big { self.big } else { self.small }
    }
}

impl Default for DesignNudgeSettings {
    fn default() -> Self {
        Self {
            small: Self::DEFAULT_SMALL,
            big: Self::DEFAULT_BIG,
        }
    }
}

/// Host-controlled workspace projection for the editable right sidebar.
///
/// This is deliberately orthogonal to [`DesignPanelSurface`] and
/// [`DesignPanelEditMode`](super::DesignPanelEditMode). Entering Draw changes
/// which inspector presentation is mounted, but it does not replace the
/// host's accepted Design/Prototype surface or the active canvas edit context.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DesignPanelWorkspaceMode {
    #[default]
    Design,
    Draw,
}

impl DesignPanelWorkspaceMode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Design => "Design",
            Self::Draw => "Draw",
        }
    }
}

/// One validated linear range used by a Draw-only appearance slider.
///
/// Figma documents Draw's slider-first presentation but does not publish a
/// universal corner-radius maximum. The host therefore supplies that range
/// for the exact current target instead of the component inventing one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesignDrawSliderRange {
    min: f32,
    max: f32,
    step: f32,
}

impl DesignDrawSliderRange {
    pub fn new(min: f32, max: f32, step: f32) -> Option<Self> {
        (min.is_finite() && max.is_finite() && step.is_finite() && min < max && step > 0.)
            .then_some(Self { min, max, step })
    }

    pub const fn min(self) -> f32 {
        self.min
    }

    pub const fn max(self) -> f32 {
        self.max
    }

    pub const fn step(self) -> f32 {
        self.step
    }

    pub fn clamp_and_snap(self, value: f32) -> f32 {
        if !value.is_finite() {
            return self.min;
        }
        let snapped = self.min + ((value - self.min) / self.step).round() * self.step;
        snapped.clamp(self.min, self.max)
    }
}

/// Vertical displacement at or beyond 80 px toward the screen top for 2x.
pub const DESIGN_SCRUB_DOUBLE_SPEED_Y_THRESHOLD: f32 = -80.;
/// Vertical displacement at or beyond 80 px toward the screen bottom for 1/2.
pub const DESIGN_SCRUB_HALF_SPEED_Y_THRESHOLD: f32 = 80.;
/// Vertical displacement at or beyond 160 px toward the screen bottom for 1/4.
pub const DESIGN_SCRUB_QUARTER_SPEED_Y_THRESHOLD: f32 = 160.;

/// Figma's four numeric-field scrub speed notifications.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DesignScrubSpeed {
    Double,
    #[default]
    Normal,
    Half,
    Quarter,
}

impl DesignScrubSpeed {
    /// Resolves a deterministic band from pointer Y minus the gesture origin.
    ///
    /// Figma documents the direction and four values but not exact screen
    /// distances. Fanta uses −80 px for 2x, +80 px for 1/2, and +160 px for
    /// 1/4; the interval between −80 and +80 remains 1x.
    pub fn from_vertical_displacement(displacement_y: f32) -> Self {
        if !displacement_y.is_finite() {
            Self::Normal
        } else if displacement_y <= DESIGN_SCRUB_DOUBLE_SPEED_Y_THRESHOLD {
            Self::Double
        } else if displacement_y >= DESIGN_SCRUB_QUARTER_SPEED_Y_THRESHOLD {
            Self::Quarter
        } else if displacement_y >= DESIGN_SCRUB_HALF_SPEED_Y_THRESHOLD {
            Self::Half
        } else {
            Self::Normal
        }
    }

    pub const fn multiplier(self) -> f32 {
        match self {
            Self::Double => 2.,
            Self::Normal => 1.,
            Self::Half => 0.5,
            Self::Quarter => 0.25,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Double => "2x",
            Self::Normal => "1x",
            Self::Half => "1/2",
            Self::Quarter => "1/4",
        }
    }

    /// Width of the transient status cue, mirroring Figma's wider cursor at
    /// faster bands without replacing the platform resize cursor.
    pub const fn cue_width(self) -> f32 {
        match self {
            Self::Double => 28.,
            Self::Normal => 22.,
            Self::Half => 16.,
            Self::Quarter => 10.,
        }
    }
}

/// One host-controlled right-sidebar surface.
///
/// Editor and viewer permissions expose different, exhaustive surface sets.
/// The reusable panel renders the Design and Properties projections; Prototype
/// and Comment remain host-owned surfaces behind the same navigation contract.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DesignPanelSurface {
    #[default]
    Design,
    Prototype,
    Comment,
    Properties,
}

impl DesignPanelSurface {
    pub const EDITOR: [Self; 2] = [Self::Design, Self::Prototype];
    pub const VIEWER: [Self; 2] = [Self::Comment, Self::Properties];

    pub const fn available(can_edit: bool) -> &'static [Self] {
        if can_edit {
            &Self::EDITOR
        } else {
            &Self::VIEWER
        }
    }

    pub const fn is_available(self, can_edit: bool) -> bool {
        matches!(
            (can_edit, self),
            (true, Self::Design | Self::Prototype) | (false, Self::Comment | Self::Properties)
        )
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Design => "Design",
            Self::Prototype => "Prototype",
            Self::Comment => "Comment",
            Self::Properties => "Properties",
        }
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Self::Design => "design",
            Self::Prototype => "prototype",
            Self::Comment => "comment",
            Self::Properties => "properties",
        }
    }
}

/// Typed intent emitted by [`super::DesignPanel`].
#[derive(Clone, Debug, PartialEq)]
pub enum DesignPanelAction {
    /// Compatibility commit-only replacement for `PageNode.backgrounds`.
    ///
    /// The retained color picker emits [`Self::PageBackgroundEditRequested`].
    PageBackgroundChangeRequested {
        page_id: SharedString,
        color: DesignColor,
    },
    /// Phased replacement for the one solid paint exposed by
    /// `PageNode.backgrounds`.
    PageBackgroundEditRequested {
        page_id: SharedString,
        color: DesignColor,
        phase: DesignPanelEditPhase,
    },
    /// Opens one host-supplied Page resource category. Browsing is
    /// non-mutating and remains available in view-only contexts.
    #[deprecated(note = "canonical Page styles render DesignPageLocalStylesViewData directly")]
    LocalResourceBrowseRequested {
        page_id: SharedString,
        category: DesignLocalResourceCategory,
    },
    /// Opens an already-local style or variable collection by stable source
    /// and resource ID.
    #[deprecated(note = "use LocalStyleCommandRequested")]
    LocalResourceOpenRequested {
        page_id: SharedString,
        resource: DesignLocalResourceSelection,
    },
    /// Creates one exact local style or variable-collection kind.
    #[deprecated(note = "use LocalStyleCreateRequested")]
    LocalResourceCreateRequested {
        page_id: SharedString,
        kind: DesignLocalResourceKind,
    },
    /// Imports one exact discoverable library resource.
    #[deprecated(note = "library importing is not part of the canonical Page Styles surface")]
    LocalResourceImportRequested {
        page_id: SharedString,
        resource: DesignLocalResourceSelection,
    },
    /// Runs one command against a style whose Page, family, direct parent,
    /// stable ID, and current sibling index all still match the host snapshot.
    LocalStyleCommandRequested {
        target: DesignLocalStyleTarget,
        command: DesignLocalStyleCommand,
    },
    /// Creates one local style in the exact current-file family/folder.
    LocalStyleCreateRequested {
        page_id: SharedString,
        kind: DesignLocalStyleKind,
        parent_folder_id: Option<SharedString>,
    },
    /// Creates a folder and optionally places an exact ordered style
    /// selection into it. An empty selection requests an empty folder.
    LocalStyleFolderCreateRequested {
        page_id: SharedString,
        kind: DesignLocalStyleKind,
        selected: Vec<DesignLocalStyleTarget>,
    },
    /// Deletes one or more exact current style leaves atomically.
    LocalStylesDeleteRequested {
        targets: Vec<DesignLocalStyleTarget>,
    },
    /// Moves exact current style leaves to an exact, neighbor-validated
    /// insertion point.
    LocalStylesMoveRequested {
        targets: Vec<DesignLocalStyleTarget>,
        destination: DesignLocalStyleInsertion,
    },
    /// Opens the host's Variables surface from the explicitly enabled legacy
    /// right-sidebar entry point. This is navigation only; it never imports or
    /// creates a collection.
    VariablesViewOpenRequested {
        page_id: SharedString,
    },
    /// Sets an explicit mode for one collection on the exact page or
    /// single-node target. The panel never updates its resolved snapshot.
    VariableModeApplyRequested {
        target: DesignPanelTarget,
        collection_id: SharedString,
        mode_id: SharedString,
    },
    /// Clears the exact current explicit mode. Carrying the old mode ID lets
    /// hosts reject stale intents after a concurrent variable-mode echo.
    VariableModeClearRequested {
        target: DesignPanelTarget,
        collection_id: SharedString,
        explicit_mode_id: SharedString,
    },
    /// Requests a host-controlled right-sidebar navigation change.
    ///
    /// This is a non-document intent. The panel keeps rendering `current`
    /// until the host validates `requested` and echoes it through
    /// `DesignPanel::set_active_surface`.
    SurfaceChangeRequested {
        current: DesignPanelSurface,
        requested: DesignPanelSurface,
    },
    SelectionHeaderCommandRequested {
        target: DesignPanelTarget,
        command: DesignSelectionHeaderCommand,
    },
    /// Requests Figma's structural “Add auto layout” operation for the exact
    /// ordered current selection.
    ///
    /// The host may convert a group/frame-like node or create a new wrapping
    /// frame around arbitrary selected layers. The reusable panel never
    /// predicts a wrapper identity, node kind, layout direction, or selection;
    /// all of those arrive in a fresh inspection-context echo.
    AddAutoLayoutRequested {
        target: DesignPanelTarget,
    },
    /// Carries the exact ordered node selection for one ordinary inspector
    /// action whose leaf payload historically named only the aggregate node.
    ///
    /// `target` is authoritative. The nested action retains its single
    /// `node_id` compatibility hint so existing single-node host reducers can
    /// be reused per target while migrating to an atomic multi-node operation.
    /// The panel emits this envelope whenever an ordinary document-facing
    /// action is triggered for a multiple selection.
    TargetedNodeActionRequested {
        target: DesignPanelTarget,
        action: Box<DesignPanelAction>,
    },
    TypographyPropertyChangeRequested {
        node_id: SharedString,
        target: DesignTypographyTarget,
        property: DesignPanelProperty,
        value: DesignPanelValue,
    },
    TypographyPropertyEditRequested {
        node_id: SharedString,
        target: DesignTypographyTarget,
        property: DesignPanelProperty,
        value: DesignPanelValue,
        phase: DesignPanelEditPhase,
    },
    /// Changes one exact host-enumerated variable-font axis.
    ///
    /// `tag` is authoritative; a row index is intentionally absent so a host
    /// may reorder its axis snapshot while an edit is in flight. Continuous
    /// slider, keyboard, and numeric-input interactions use the standard
    /// Begin/Preview/Commit/Cancel lifecycle.
    TypographyVariableAxisEditRequested {
        node_id: SharedString,
        target: DesignTypographyTarget,
        tag: SharedString,
        value: f32,
        phase: DesignPanelEditPhase,
    },
    /// Applies one exact host-supplied page or library text style. The host
    /// resolves the stable selection and echoes a fresh typography snapshot.
    TypographyStyleApplyRequested {
        node_id: SharedString,
        target: DesignTypographyTarget,
        style: DesignTypographyStyleSelection,
    },
    /// Detaches the exact current style binding while preserving its resolved
    /// typography values in host-owned document state.
    TypographyStyleDetachRequested {
        node_id: SharedString,
        target: DesignTypographyTarget,
        style: DesignTypographyStyleSelection,
    },
    /// Applies an exact host-enumerated font family/style pair.
    TypographyFontApplyRequested {
        node_id: SharedString,
        target: DesignTypographyTarget,
        font: DesignFontSelection,
    },
    /// Imports an exact discoverable font. Import and apply are separate
    /// operations so the host can echo loading or failure state.
    TypographyFontImportRequested {
        node_id: SharedString,
        target: DesignTypographyTarget,
        font: DesignFontSelection,
    },
    /// Changes one exact host-supplied OpenType feature tag.
    TypographyOpenTypeFeatureChangeRequested {
        node_id: SharedString,
        target: DesignTypographyTarget,
        tag: DesignOpenTypeFeatureTag,
        enabled: bool,
    },
    /// Requests Figma's native whole-node "Flip text orientation" command.
    ///
    /// Orientation is intentionally not predicted. The host resolves the
    /// command for this exact TextPath identity and echoes fresh
    /// [`DesignTextPathViewData`].
    TextPathFlipOrientationRequested {
        node_id: SharedString,
    },
    /// Replaces the complete `TextPathNode.textPathStartData` record.
    TextPathStartChangeRequested {
        node_id: SharedString,
        data: DesignTextPathStartData,
        phase: DesignPanelEditPhase,
    },
    /// Replaces the host-owned vector sub-selection by opaque vertex identity.
    VectorVertexSelectionEditRequested {
        node_id: SharedString,
        selected_vertex_ids: Vec<SharedString>,
        phase: DesignPanelEditPhase,
    },
    /// Sets one coordinate for every exact selected vertex identity.
    VectorVertexPositionEditRequested {
        node_id: SharedString,
        vertex_ids: Vec<SharedString>,
        axis: DesignVectorCoordinateAxis,
        value: f32,
        phase: DesignPanelEditPhase,
    },
    /// Sets Figma's per-vertex `cornerRadius` for exact selected identities.
    VectorVertexCornerRadiusEditRequested {
        node_id: SharedString,
        vertex_ids: Vec<SharedString>,
        radius: f32,
        phase: DesignPanelEditPhase,
    },
    /// Sets Figma's writable `HandleMirroring` for exact selected identities.
    VectorHandleMirroringEditRequested {
        node_id: SharedString,
        vertex_ids: Vec<SharedString>,
        mirroring: DesignHandleMirroring,
        phase: DesignPanelEditPhase,
    },
    PropertyChangeRequested {
        node_id: SharedString,
        property: DesignPanelProperty,
        value: DesignPanelValue,
    },
    PropertyEditRequested {
        node_id: SharedString,
        property: DesignPanelProperty,
        value: DesignPanelValue,
        phase: DesignPanelEditPhase,
    },
    /// Starts or ends Figma's on-canvas min/max bounds preview for one axis.
    ///
    /// The values are the exact immutable host snapshot shown by the
    /// inspector. `preview=false` carries the same captured values as the
    /// matching start so a host can safely unwind a preview after a selection
    /// or property echo.
    DimensionLimitsPreviewRequested {
        node_id: SharedString,
        axis: DesignLayoutDimensionAxis,
        minimum: Option<f32>,
        maximum: Option<f32>,
        preview: bool,
    },
    /// Starts or ends a non-mutating preview for one exact inspector-menu
    /// candidate. A matching `End` always repeats the immutable `Begin`
    /// payload; committed document state changes only through the ordinary
    /// property, paint, or effect action emitted after preview cleanup.
    MenuPreviewRequested {
        preview: DesignMenuPreview,
        phase: DesignMenuPreviewPhase,
    },
    /// Requests that the host copy one generic inspector readout exactly as
    /// displayed. This is a non-mutating viewer action: the reusable panel
    /// never writes to the clipboard itself.
    PropertyCopyRequested {
        target: DesignPanelTarget,
        property: DesignPanelProperty,
        displayed_value: SharedString,
    },
    /// Requests that the host copy one exact view-only Properties section.
    ///
    /// The reusable panel never writes to the clipboard. `copy_value` is
    /// supplied by the host projection so CSS and color conversions stay
    /// lossless and host controlled.
    ViewerSectionCopyRequested {
        target: DesignPanelTarget,
        section_id: SharedString,
        copy_value: SharedString,
    },
    /// Requests a different host-owned representation for one view-only
    /// Properties section. The visible rows do not change until the host
    /// echoes a fresh [`DesignViewerPropertiesViewData`] snapshot.
    ViewerSectionRepresentationChangeRequested {
        target: DesignPanelTarget,
        section_id: SharedString,
        representation: DesignViewerColorRepresentation,
    },
    /// Applies an imported/page variable to every exact API field represented
    /// by one inspector property. The panel never changes its controlled
    /// property snapshot while waiting for the host echo.
    PropertyVariableApplyRequested {
        node_id: SharedString,
        target: DesignPropertyVariableTarget,
        variable_id: SharedString,
    },
    /// Requests import of one discoverable library variable. Import and apply
    /// are intentionally separate host operations.
    PropertyVariableImportRequested {
        node_id: SharedString,
        target: DesignPropertyVariableTarget,
        variable_id: SharedString,
    },
    /// Detaches the exact current variable from all API fields represented by
    /// the property while preserving the host-resolved raw value.
    PropertyVariableDetachRequested {
        node_id: SharedString,
        target: DesignPropertyVariableTarget,
        variable_id: SharedString,
    },
    SectionShareRequested {
        node_id: SharedString,
    },
    SectionResolveChangedStatusRequested {
        node_id: SharedString,
    },
    TransformModifierAddRequested {
        node_id: SharedString,
        repeat_type: DesignRepeatType,
    },
    TransformModifierRemoveRequested {
        node_id: SharedString,
        modifier_id: SharedString,
        index: usize,
    },
    TransformModifierChangeRequested {
        node_id: SharedString,
        modifier_id: SharedString,
        index: usize,
        change: DesignTransformModifierChange,
        phase: DesignPanelEditPhase,
    },
    ApplyTransformModifiersRequested {
        node_id: SharedString,
    },
    /// Creates a new definition at the end of Figma's Variant or regular
    /// component-property partition.
    ///
    /// The host owns the generated property identity. `expected_property_order`
    /// and `after_property_id` are stable stale-intent guards captured when the
    /// modal is submitted; neither value is a parsed document identifier.
    ComponentPropertyDefinitionCreateRequested {
        node_id: SharedString,
        kind: DesignComponentPropertyKind,
        name: SharedString,
        /// Slot descriptions are authored in the create modal. Other kinds
        /// currently carry `None`, matching Figma's property schema.
        description: Option<SharedString>,
        documentation_links: Vec<DesignDocumentationLink>,
        definition: DesignComponentPropertyDefinition,
        /// Stable variable catalog identity for Boolean/Text default values.
        default_variable_id: Option<SharedString>,
        partition: DesignComponentPropertyPartition,
        expected_property_order: Vec<SharedString>,
        after_property_id: Option<SharedString>,
    },
    /// Edits one exact definition name through the panel's inline editor.
    ///
    /// `original_name` is the value at `Begin`; `expected_name` is the latest
    /// controlled host echo. Together with `phase`, these fields let a host
    /// balance preview/commit/cancel without accepting a stale overwrite.
    ComponentPropertyDefinitionRenameRequested {
        node_id: SharedString,
        property_id: SharedString,
        original_name: SharedString,
        expected_name: SharedString,
        name: SharedString,
        phase: DesignPanelEditPhase,
    },
    /// Commits one exact description from the Edit property modal.
    ComponentPropertyDefinitionMetadataEditRequested {
        node_id: SharedString,
        property_id: SharedString,
        expected_description: Option<SharedString>,
        description: Option<SharedString>,
        expected_documentation_links: Vec<DesignDocumentationLink>,
        documentation_links: Vec<DesignDocumentationLink>,
    },
    /// Atomically commits Figma's Edit property modal for one exact property.
    ///
    /// Metadata and the typed definition share one stale guard and one host
    /// transaction so a combined confirmation cannot be partially accepted or
    /// split across undo entries. Slot settings live inside `definition`.
    ComponentPropertyDefinitionEditRequested {
        node_id: SharedString,
        property_id: SharedString,
        expected_description: Option<SharedString>,
        description: Option<SharedString>,
        expected_documentation_links: Vec<DesignDocumentationLink>,
        documentation_links: Vec<DesignDocumentationLink>,
        expected_definition: DesignComponentPropertyDefinition,
        definition: DesignComponentPropertyDefinition,
    },
    ComponentPropertyDefinitionDeleteRequested {
        node_id: SharedString,
        property_id: SharedString,
        expected_name: SharedString,
    },
    /// Reorders within one Figma definition partition.
    ///
    /// `before_property_id` is `None` when moving to the end. The original and
    /// expected stable-ID orders keep a drag cancelable and reject stale or
    /// cross-partition moves without relying on display indexes.
    ComponentPropertyDefinitionReorderRequested {
        node_id: SharedString,
        property_id: SharedString,
        partition: DesignComponentPropertyPartition,
        original_property_order: Vec<SharedString>,
        expected_property_order: Vec<SharedString>,
        before_property_id: Option<SharedString>,
        phase: DesignPanelEditPhase,
    },
    ComponentVariantOptionCreateRequested {
        node_id: SharedString,
        property_id: SharedString,
        name: SharedString,
        expected_option_order: Vec<SharedString>,
        after_option_id: Option<SharedString>,
    },
    ComponentVariantOptionRenameRequested {
        node_id: SharedString,
        property_id: SharedString,
        option_id: SharedString,
        original_name: SharedString,
        expected_name: SharedString,
        name: SharedString,
        phase: DesignPanelEditPhase,
    },
    ComponentVariantOptionDeleteRequested {
        node_id: SharedString,
        property_id: SharedString,
        option_id: SharedString,
        expected_name: SharedString,
    },
    ComponentVariantOptionReorderRequested {
        node_id: SharedString,
        property_id: SharedString,
        option_id: SharedString,
        original_option_order: Vec<SharedString>,
        expected_option_order: Vec<SharedString>,
        before_option_id: Option<SharedString>,
        phase: DesignPanelEditPhase,
    },
    ComponentPropertyApplyToLayerRequested {
        node_id: SharedString,
        control_id: SharedString,
        layer_id: SharedString,
        surface: DesignComponentPropertyApplicationSurface,
        property_id: SharedString,
    },
    ComponentPropertySwitchOnLayerRequested {
        node_id: SharedString,
        control_id: SharedString,
        layer_id: SharedString,
        surface: DesignComponentPropertyApplicationSurface,
        from_property_id: SharedString,
        to_property_id: SharedString,
    },
    ComponentPropertyDetachFromLayerRequested {
        node_id: SharedString,
        control_id: SharedString,
        layer_id: SharedString,
        surface: DesignComponentPropertyApplicationSurface,
        property_id: SharedString,
    },
    NestedComponentPropertyExposeRequested {
        node_id: SharedString,
        candidate_id: SharedString,
        nested_instance_id: SharedString,
        nested_property_id: SharedString,
    },
    NestedComponentPropertyUnexposeRequested {
        node_id: SharedString,
        candidate_id: SharedString,
        nested_instance_id: SharedString,
        nested_property_id: SharedString,
        exposed_property_id: SharedString,
    },
    /// `preview=true` highlights one nested source; `false` restores the
    /// host's prior canvas presentation without changing the document.
    NestedComponentPropertyPreviewRequested {
        node_id: SharedString,
        candidate_id: SharedString,
        nested_instance_id: SharedString,
        nested_property_id: SharedString,
        preview: bool,
    },
    ComponentPropertyChangeRequested {
        node_id: SharedString,
        property_id: SharedString,
        value: DesignComponentPropertyValue,
    },
    ComponentPropertyEditRequested {
        node_id: SharedString,
        property_id: SharedString,
        value: DesignComponentPropertyValue,
        phase: DesignPanelEditPhase,
    },
    ComponentPropertyResetRequested {
        node_id: SharedString,
        property_id: SharedString,
    },
    /// Applies one page/imported variable to the exact `defaultValue` or
    /// instance `value` field represented by `target`.
    ComponentPropertyVariableApplyRequested {
        node_id: SharedString,
        target: DesignComponentPropertyVariableTarget,
        variable_id: SharedString,
    },
    ComponentPropertyVariableImportRequested {
        node_id: SharedString,
        target: DesignComponentPropertyVariableTarget,
        variable_id: SharedString,
    },
    ComponentPropertyVariableDetachRequested {
        node_id: SharedString,
        target: DesignComponentPropertyVariableTarget,
        variable_id: SharedString,
    },
    /// Accepts one exact all-components browser selection. `None` clears an
    /// optional instance-swap property.
    ComponentSwapApplyRequested {
        node_id: SharedString,
        property_id: SharedString,
        selection: Option<DesignComponentSwapSelection>,
    },
    ComponentSwapImportRequested {
        node_id: SharedString,
        property_id: SharedString,
        selection: DesignComponentSwapSelection,
    },
    /// `Some` starts/changes hover preview and `None` restores host state.
    ComponentSwapPreviewRequested {
        node_id: SharedString,
        property_id: SharedString,
        selection: Option<DesignComponentSwapSelection>,
    },
    ComponentPropertyNestedInstanceSelectRequested {
        node_id: SharedString,
        property_id: SharedString,
        instance_id: SharedString,
    },
    ComponentPropertyNestedInstanceGoToMainRequested {
        node_id: SharedString,
        property_id: SharedString,
        instance_id: SharedString,
        main_component_id: SharedString,
    },
    SlotSettingsChangeRequested {
        node_id: SharedString,
        property_id: SharedString,
        expected_settings: DesignSlotSettings,
        change: DesignSlotSettingsChange,
        phase: DesignPanelEditPhase,
    },
    SlotResetRequested {
        node_id: SharedString,
        property_id: SharedString,
    },
    SlotClearRequested {
        node_id: SharedString,
        property_id: SharedString,
    },
    SlotAddInstanceRequested {
        node_id: SharedString,
        property_id: SharedString,
        preferred_component: Option<DesignComponentReference>,
    },
    /// Selects/navigates to one stable layer contained by a Slot.
    SlotChildSelectRequested {
        node_id: SharedString,
        property_id: SharedString,
        child_node_id: SharedString,
    },
    /// Selects every current layer that violates a Slot's preferred-instance
    /// guideline.
    ///
    /// The ordered IDs are an exact stale-intent guard captured from the
    /// host-controlled Slot value and violation snapshot. Hosts must reject
    /// the request if the Slot contents or violations no longer resolve to
    /// this same ordered set.
    SlotLimitLayersSelectRequested {
        node_id: SharedString,
        property_id: SharedString,
        child_node_ids: Vec<SharedString>,
    },
    /// Removes one stable Slot child. `index` is only an ordering hint.
    SlotChildRemoveRequested {
        node_id: SharedString,
        property_id: SharedString,
        child_node_id: SharedString,
        index: usize,
    },
    /// Reorders one stable Slot child within the Slot's back-to-front list.
    SlotChildReorderRequested {
        node_id: SharedString,
        property_id: SharedString,
        child_node_id: SharedString,
        from_index: usize,
        to_index: usize,
    },
    /// Replaces an instance child while preserving its stable Slot position.
    SlotChildReplaceRequested {
        node_id: SharedString,
        property_id: SharedString,
        child_node_id: SharedString,
        replacement: DesignComponentReference,
    },
    CollectionItemAddRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
    },
    CollectionItemRemoveRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        index: usize,
    },
    /// Adds the first exact effect kind selected by the panel from the host's
    /// availability and Figma's per-kind count limits.
    EffectAddRequested {
        node_id: SharedString,
        kind: DesignEffectKind,
    },
    /// Removes one stable effect row. `index` is a compatibility hint only.
    EffectRemoveRequested {
        node_id: SharedString,
        effect_id: SharedString,
        index: usize,
    },
    /// Reorders one effect without invalidating its anchored settings popup.
    EffectReorderRequested {
        node_id: SharedString,
        effect_id: SharedString,
        from_index: usize,
        to_index: usize,
    },
    /// Edits one stable effect row. Shader leaves additionally carry their
    /// property-definition ID so the host never resolves a stale property
    /// index after a shader metadata echo.
    EffectEditRequested {
        node_id: SharedString,
        effect_id: SharedString,
        index: usize,
        property: DesignPanelProperty,
        shader_property_id: Option<SharedString>,
        value: DesignPanelValue,
        phase: DesignPanelEditPhase,
    },
    /// Opens the host's imported/available shader chooser.
    EffectShaderChooseRequested {
        node_id: SharedString,
        effect_id: SharedString,
        index: usize,
    },
    /// Opens a host-owned resource or variable chooser for one stable Shader
    /// property-definition. The panel never invents asset or variable IDs.
    EffectShaderPropertyEditorRequested {
        node_id: SharedString,
        effect_id: SharedString,
        index: usize,
        shader_property_id: SharedString,
        property_index: usize,
        property_kind: DesignShaderPropertyKind,
        target: DesignShaderPropertyEditorTarget,
        editor: DesignShaderPropertyEditorKind,
        current_value: DesignShaderPropertyValue,
    },
    /// Detaches an echoed Shader-property variable binding. `variable_id`
    /// lets a host reject a stale request.
    EffectShaderPropertyVariableDetachRequested {
        node_id: SharedString,
        effect_id: SharedString,
        index: usize,
        shader_property_id: SharedString,
        property_index: usize,
        target: DesignShaderPropertyEditorTarget,
        variable_id: SharedString,
    },
    EffectStyleApplyRequested {
        node_id: SharedString,
        style: DesignEffectStyleSelection,
    },
    EffectStyleCreateRequested {
        node_id: SharedString,
        effects: Vec<DesignEffect>,
    },
    EffectStyleDetachRequested {
        node_id: SharedString,
        style: DesignEffectStyleSelection,
    },
    EffectVariableApplyRequested {
        node_id: SharedString,
        effect_id: SharedString,
        index: usize,
        field: DesignEffectVariableField,
        variable_id: SharedString,
    },
    EffectVariableDetachRequested {
        node_id: SharedString,
        effect_id: SharedString,
        index: usize,
        field: DesignEffectVariableField,
        variable_id: SharedString,
    },
    /// Atomically requests positive column and row counts for one exact Grid
    /// container. The host decides how existing tracks and children relocate,
    /// then echoes complete authoritative track vectors.
    ///
    /// When rows are host-derived through [`DesignGridAutoTracks::Rows`], the
    /// panel preserves the echoed row count and emits column-only changes with
    /// that exact current row count.
    GridDimensionsEditRequested {
        node_id: SharedString,
        dimensions: DesignGridDimensions,
        phase: DesignPanelEditPhase,
    },
    /// Inserts one explicit Grid track. The host owns the resulting track
    /// vector and echoes it through [`DesignPanelNode`].
    GridTrackAddRequested {
        node_id: SharedString,
        axis: DesignGridTrackAxis,
        insertion_index: usize,
    },
    /// Deletes one explicit Grid track. The panel suppresses this intent when
    /// it would remove the final track or an auto-managed row.
    GridTrackDeleteRequested {
        node_id: SharedString,
        axis: DesignGridTrackAxis,
        index: usize,
    },
    /// Reorders explicit Grid tracks using Figma's `fromIndices` and
    /// `insertionIndex` semantics.
    GridTracksReorderRequested {
        node_id: SharedString,
        axis: DesignGridTrackAxis,
        from_indices: Vec<usize>,
        insertion_index: usize,
    },
    LayoutGridStyleApplyRequested {
        node_id: SharedString,
        style: DesignLayoutGridStyleSelection,
    },
    LayoutGridStyleCreateRequested {
        node_id: SharedString,
        layout_grids: Vec<DesignLayoutGrid>,
    },
    LayoutGridStyleDetachRequested {
        node_id: SharedString,
        style: DesignLayoutGridStyleSelection,
    },
    LayoutGridStyleImportRequested {
        node_id: SharedString,
        style: DesignLayoutGridStyleSelection,
    },
    /// Edits one ordinary layout guide by stable host identity.
    ///
    /// `index` is only a compatibility hint for legacy guides whose `id` is
    /// empty. Hosts must resolve non-empty `guide_id` values against their
    /// latest ordered guide snapshot on every phase.
    LayoutGridPropertyEditRequested {
        node_id: SharedString,
        guide_id: SharedString,
        index: usize,
        property: DesignPanelProperty,
        value: DesignPanelValue,
        phase: DesignPanelEditPhase,
    },
    /// Removes one ordinary layout guide by stable host identity.
    ///
    /// `index` is only a compatibility hint when `guide_id` is empty.
    LayoutGridRemoveRequested {
        node_id: SharedString,
        guide_id: SharedString,
        index: usize,
    },
    /// Applies one already-local Number variable to an exact, stable
    /// layout-guide leaf.
    LayoutGridVariableApplyRequested {
        node_id: SharedString,
        target: DesignLayoutGridVariableTarget,
        variable_id: SharedString,
    },
    /// Imports one available library Number variable. Import and apply remain
    /// separate host operations.
    LayoutGridVariableImportRequested {
        node_id: SharedString,
        target: DesignLayoutGridVariableTarget,
        variable_id: SharedString,
    },
    /// Detaches the exact echoed binding while preserving the resolved raw
    /// guide value.
    LayoutGridVariableDetachRequested {
        node_id: SharedString,
        target: DesignLayoutGridVariableTarget,
        variable_id: SharedString,
    },
    /// Opens the host's Number-variable creation flow for one exact guide
    /// field. The panel never manufactures a collection or variable ID.
    LayoutGridVariableCreateRequested {
        node_id: SharedString,
        target: DesignLayoutGridVariableTarget,
        value: DesignLayoutGridVariableValue,
    },
    #[deprecated(note = "use LayoutGridVariableApplyRequested with a Count target")]
    LayoutGridCountVariableApplyRequested {
        node_id: SharedString,
        guide_id: SharedString,
        index: usize,
        variable_id: SharedString,
    },
    #[deprecated(note = "use LayoutGridVariableDetachRequested with a Count target")]
    LayoutGridCountVariableDetachRequested {
        node_id: SharedString,
        guide_id: SharedString,
        index: usize,
        variable_id: SharedString,
    },
    #[deprecated(note = "use DesignPanelAction::PaintEditRequested")]
    PaintChangeRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        index: usize,
        paint: DesignPaint,
    },
    /// Typed paint edit keyed by stable paint identity with a legacy index
    /// fallback. The host applies the edit and echoes a fresh paint snapshot.
    PaintEditRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        edit: DesignPaintEdit,
        phase: DesignPanelEditPhase,
    },
    /// Reorders one paint without invalidating an open picker for a stable ID.
    PaintReorderRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        from_index: usize,
        to_index: usize,
    },
    /// Applies an imported/page Paint style to the complete ordered Fill or
    /// Stroke paint collection.
    PaintStyleApplyRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        style: DesignPaintStyleSelection,
    },
    /// Imports one discoverable library Paint style. Import and apply are
    /// intentionally separate host operations.
    PaintStyleImportRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        style: DesignPaintStyleSelection,
    },
    /// Opens the host's Paint-style creation flow with the exact ordered
    /// collection snapshot.
    PaintStyleCreateRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paints: Vec<DesignPaint>,
    },
    /// Detaches the exact current `fillStyleId`/`strokeStyleId` while
    /// preserving the host-resolved ordered paints.
    PaintStyleDetachRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        style: DesignPaintStyleSelection,
    },
    /// Opens the host's source-node chooser for a Pattern paint.
    PaintSourceReplaceRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
    },
    /// Opens one exact host-owned image/video source workflow. `source_id`
    /// snapshots the currently controlled source so an asynchronous chooser,
    /// generation, or editing result cannot be applied to a subsequently
    /// replaced source.
    PaintMediaSourceActionRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        source_id: SharedString,
        action: DesignMediaSourceAction,
    },
    /// Replaces one exact image/video source from a validated external file.
    ///
    /// `expected_source_id` and `expected_media_kind` snapshot the controlled
    /// media payload. The host rejects the intent if either changed before it
    /// resolves the stable paint identity. The reusable panel never opens,
    /// reads, uploads, or retains the file.
    PaintMediaSourceDropRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        expected_source_id: SharedString,
        expected_media_kind: DesignMediaKind,
        file: DesignMediaDroppedFile,
    },
    /// Drives a host-controlled crop-tool session for one stable image/video
    /// paint. Preview/commit payloads carry the candidate affine crop without
    /// mutating the controlled paint snapshot in the panel.
    PaintMediaCropActionRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        action: DesignMediaCropAction,
    },
    /// Controls Design-tab video preview only. Playback never becomes a paint
    /// document property.
    PaintVideoPreviewActionRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        action: DesignVideoPreviewAction,
    },
    /// Imports one discoverable fill shader into the file. The host performs
    /// the async import and echoes refreshed [`DesignShaderViewData`].
    PaintShaderImportRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        shader: DesignShaderSelection,
    },
    /// Applies one already-imported fill shader to this stable paint row.
    /// The host resolves author defaults and echoes the resulting payload.
    PaintShaderApplyRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        shader: DesignShaderSelection,
    },
    /// Opens the host's variable picker for one shader property-definition id.
    PaintShaderPropertyBindRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        definition_id: SharedString,
    },
    /// Opens the host's type-appropriate editor for a complex shader value
    /// (resource, geometry, gradient, or author-specific text control).
    PaintShaderPropertyEditorRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        definition_id: SharedString,
    },
    /// Detaches the exact current variable alias from one shader property.
    PaintShaderPropertyDetachRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        definition_id: SharedString,
        variable_id: SharedString,
    },
    /// Applies one imported/page Color variable to a solid paint or a specific
    /// stable gradient stop.
    PaintColorVariableApplyRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        color_target: DesignPaintColorTarget,
        variable_id: SharedString,
    },
    /// Imports one discoverable Color variable. Import and apply are separate
    /// so an available library value never appears bound before a host echo.
    PaintColorVariableImportRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        color_target: DesignPaintColorTarget,
        variable_id: SharedString,
    },
    /// Detaches the exact current Color-variable identity from one paint leaf
    /// while preserving its host-resolved color.
    PaintColorVariableDetachRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        color_target: DesignPaintColorTarget,
        variable_id: SharedString,
    },
    /// Opens the host's Color-variable creation flow using the resolved leaf
    /// value; the panel does not choose a collection or manufacture an ID.
    PaintColorVariableCreateRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        color_target: DesignPaintColorTarget,
        color: DesignColor,
    },
    /// Samples one host-supplied Color-style value into the exact active color
    /// leaf without attaching a whole Fill/Stroke Paint style or Color
    /// variable binding.
    PaintColorStyleSampleRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        color_target: DesignPaintColorTarget,
        sample: DesignColorStyleSampleSelection,
    },
    /// Compatibility-only leaf color preset action.
    #[deprecated(
        note = "use PaintStyleApplyRequested for whole styles or PaintColorVariableApplyRequested for leaf variables"
    )]
    PaintColorStyleApplyRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        color_target: DesignPaintColorTarget,
        style: DesignColorStyleSelection,
    },
    /// Compatibility-only leaf color preset creation action.
    #[deprecated(note = "use PaintColorVariableCreateRequested")]
    PaintColorStyleCreateRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        color_target: DesignPaintColorTarget,
        color: DesignColor,
    },
    /// Opens the host's eyedropper for an editable, unbound solid color or
    /// stable gradient stop. Applying a sampled value remains host-owned.
    PaintEyedropperRequested {
        node_id: SharedString,
        collection: DesignPanelCollection,
        target: DesignPaintTarget,
        paint_id: SharedString,
        index: usize,
        color_target: DesignPaintColorTarget,
    },
    #[deprecated(note = "use DesignPanelAction::ExportAllRequested")]
    ExportRequested {
        node_id: SharedString,
        index: usize,
    },
    ExportConfigurationAddRequested {
        target: DesignPanelTarget,
    },
    ExportConfigurationRemoveRequested {
        target: DesignPanelTarget,
        configuration_id: SharedString,
    },
    ExportConfigurationChangeRequested {
        target: DesignPanelTarget,
        configuration_id: SharedString,
        change: DesignExportConfigurationChange,
        phase: DesignPanelEditPhase,
    },
    ExportModeChangeRequested {
        target: DesignPanelTarget,
        mode: DesignExportMode,
    },
    AnimatedExportChangeRequested {
        target: DesignPanelTarget,
        change: DesignAnimatedExportChange,
        phase: DesignPanelEditPhase,
    },
    AnimatedExportRequested {
        target: DesignPanelTarget,
        settings: DesignAnimatedExportSettings,
    },
    ExportAllRequested {
        target: DesignPanelTarget,
    },
    ExportPreviewRequested {
        target: DesignPanelTarget,
    },
    /// Continuously updates one axis of the exact ordered Smart Selection.
    ///
    /// The panel retains only the draft. The host snapshots spacing at Begin,
    /// previews/commits the candidate atomically for the complete target, and
    /// restores its snapshot at Cancel.
    SmartSelectionSpacingEditRequested {
        target: DesignPanelTarget,
        axis: DesignSmartSelectionAxis,
        value: f32,
        phase: DesignPanelEditPhase,
    },
    /// Runs one host-authorized distribute or Tidy up operation for the exact
    /// ordered Smart Selection snapshot.
    SmartSelectionArrangeRequested {
        target: DesignPanelTarget,
        operation: DesignSmartSelectionOperation,
    },
    ArrangeRequested {
        target: DesignPanelTarget,
        operation: DesignArrangeOperation,
    },
    TransformRequested {
        target: DesignPanelTarget,
        operation: DesignTransformOperation,
    },
    ResizeToFitRequested {
        target: DesignPanelTarget,
    },
    /// Applies one exact host-enumerated Frame preset.
    ///
    /// Stable group/preset identity lets the host reject a stale catalog
    /// activation. Dimensions are the exact immutable values displayed by the
    /// triggering row and let reducers validate that identity against their
    /// latest catalog before resizing the Frame.
    FramePresetApplyRequested {
        node_id: SharedString,
        selection: DesignFramePresetSelection,
        width: f32,
        height: f32,
    },
    /// Atomically exchanges the start and end cap of an open line/path.
    ///
    /// The host owns the stroke and echoes the resulting cap values.
    SwapStrokeEndpointsRequested {
        node_id: SharedString,
    },
    /// Recolors every stable paint occurrence represented by one Selection
    /// colors aggregate row.
    SelectionColorEditRequested {
        target: DesignPanelTarget,
        selection_color_id: SharedString,
        color: DesignColor,
        paint_references: Vec<DesignSelectionPaintReference>,
        phase: DesignPanelEditPhase,
    },
    /// Applies one typed normal-paint edit to every exact occurrence behind a
    /// stable Selection-colors aggregate row.
    ///
    /// The ordered selection and occurrence references are authoritative.
    /// `DesignPaintEdit` retains the same typed property/value contract used
    /// by ordinary Fill and Stroke paint editors.
    SelectionColorPaintEditRequested {
        target: DesignPanelTarget,
        selection_color_id: SharedString,
        paint_references: Vec<DesignSelectionPaintReference>,
        edit: DesignPaintEdit,
        phase: DesignPanelEditPhase,
    },
    /// Asks the host to select every exact paint occurrence represented by one
    /// aggregate row. This changes selection only and remains available to
    /// viewers when every occurrence has a stable supplied reference.
    SelectionColorOccurrencesSelectRequested {
        target: DesignPanelTarget,
        selection_color_id: SharedString,
        paint_references: Vec<DesignSelectionPaintReference>,
    },
    /// Applies one imported/page Paint style to every exact Fill/Stroke
    /// collection represented by a stable Selection-colors aggregate row.
    SelectionColorPaintStyleApplyRequested {
        target: DesignPanelTarget,
        selection_color_id: SharedString,
        paint_references: Vec<DesignSelectionPaintReference>,
        style: DesignPaintStyleSelection,
    },
    /// Imports one discoverable library Paint style without applying it.
    SelectionColorPaintStyleImportRequested {
        target: DesignPanelTarget,
        selection_color_id: SharedString,
        paint_references: Vec<DesignSelectionPaintReference>,
        style: DesignPaintStyleSelection,
    },
    /// Opens whole Paint-style creation with the row's exact ordered
    /// collection snapshot.
    SelectionColorPaintStyleCreateRequested {
        target: DesignPanelTarget,
        selection_color_id: SharedString,
        paint_references: Vec<DesignSelectionPaintReference>,
        paints: Vec<DesignPaint>,
    },
    /// Detaches the exact uniform collection-level style binding while
    /// preserving its resolved paints.
    SelectionColorPaintStyleDetachRequested {
        target: DesignPanelTarget,
        selection_color_id: SharedString,
        paint_references: Vec<DesignSelectionPaintReference>,
        style: DesignPaintStyleSelection,
    },
    /// Applies one imported/page Color variable to every exact selected leaf.
    SelectionColorVariableApplyRequested {
        target: DesignPanelTarget,
        selection_color_id: SharedString,
        paint_references: Vec<DesignSelectionPaintReference>,
        variable_id: SharedString,
    },
    /// Imports one available library Color variable without binding it.
    SelectionColorVariableImportRequested {
        target: DesignPanelTarget,
        selection_color_id: SharedString,
        paint_references: Vec<DesignSelectionPaintReference>,
        variable_id: SharedString,
    },
    /// Opens Color-variable creation for the row's current resolved color.
    SelectionColorVariableCreateRequested {
        target: DesignPanelTarget,
        selection_color_id: SharedString,
        paint_references: Vec<DesignSelectionPaintReference>,
        color: DesignColor,
    },
    /// Detaches the exact uniform leaf-level variable binding.
    SelectionColorVariableDetachRequested {
        target: DesignPanelTarget,
        selection_color_id: SharedString,
        paint_references: Vec<DesignSelectionPaintReference>,
        variable_id: SharedString,
    },
    /// Compatibility intent retained for old node-level media adapters.
    ///
    /// The panel no longer emits this variant. Hosts should handle
    /// [`Self::PaintMediaSourceActionRequested`] instead.
    #[deprecated(note = "use DesignPanelAction::PaintMediaSourceActionRequested")]
    ReplaceMediaRequested {
        node_id: SharedString,
    },
    ResetInstanceOverridesRequested {
        node_id: SharedString,
    },
    GoToMainComponentRequested {
        node_id: SharedString,
    },
    DetachInstanceRequested {
        node_id: SharedString,
    },
}

#[allow(deprecated)]
impl DesignPanelAction {
    /// Wraps a legacy single-node leaf payload in its authoritative ordered
    /// selection target.
    ///
    /// Hosts should treat the target as one atomic operation. The nested
    /// `node_id` is only a compatibility hint and must not be used to discard
    /// the remaining selected IDs.
    pub fn for_selection_target(target: DesignPanelTarget, action: Self) -> Self {
        Self::TargetedNodeActionRequested {
            target,
            action: Box::new(action),
        }
    }

    /// Returns the authoritative ordered target and nested leaf payload for a
    /// targeted ordinary inspector action.
    pub fn targeted_node_action(&self) -> Option<(&DesignPanelTarget, &Self)> {
        match self {
            Self::TargetedNodeActionRequested { target, action } => Some((target, action.as_ref())),
            _ => None,
        }
    }

    /// Returns the exact Fill/Stroke scope captured by a paint interaction.
    ///
    /// Hosts can compare a selected-range target with their current text
    /// selection before resolving the stable paint ID. Import-only actions
    /// retain the same target so an asynchronous chooser cannot be replayed
    /// against a later range.
    pub const fn paint_target(&self) -> Option<(DesignPanelCollection, DesignPaintTarget)> {
        match self {
            Self::CollectionItemAddRequested {
                collection, target, ..
            }
            | Self::CollectionItemRemoveRequested {
                collection, target, ..
            }
            | Self::PaintChangeRequested {
                collection, target, ..
            }
            | Self::PaintEditRequested {
                collection, target, ..
            }
            | Self::PaintReorderRequested {
                collection, target, ..
            }
            | Self::PaintStyleApplyRequested {
                collection, target, ..
            }
            | Self::PaintStyleImportRequested {
                collection, target, ..
            }
            | Self::PaintStyleCreateRequested {
                collection, target, ..
            }
            | Self::PaintStyleDetachRequested {
                collection, target, ..
            }
            | Self::PaintSourceReplaceRequested {
                collection, target, ..
            }
            | Self::PaintMediaSourceActionRequested {
                collection, target, ..
            }
            | Self::PaintMediaSourceDropRequested {
                collection, target, ..
            }
            | Self::PaintMediaCropActionRequested {
                collection, target, ..
            }
            | Self::PaintVideoPreviewActionRequested {
                collection, target, ..
            }
            | Self::PaintShaderImportRequested {
                collection, target, ..
            }
            | Self::PaintShaderApplyRequested {
                collection, target, ..
            }
            | Self::PaintShaderPropertyBindRequested {
                collection, target, ..
            }
            | Self::PaintShaderPropertyEditorRequested {
                collection, target, ..
            }
            | Self::PaintShaderPropertyDetachRequested {
                collection, target, ..
            }
            | Self::PaintColorVariableApplyRequested {
                collection, target, ..
            }
            | Self::PaintColorVariableImportRequested {
                collection, target, ..
            }
            | Self::PaintColorVariableDetachRequested {
                collection, target, ..
            }
            | Self::PaintColorVariableCreateRequested {
                collection, target, ..
            }
            | Self::PaintColorStyleSampleRequested {
                collection, target, ..
            }
            | Self::PaintColorStyleApplyRequested {
                collection, target, ..
            }
            | Self::PaintColorStyleCreateRequested {
                collection, target, ..
            }
            | Self::PaintEyedropperRequested {
                collection, target, ..
            } => Some((*collection, *target)),
            _ => None,
        }
    }

    /// Returns a copy of a legacy leaf payload with its compatibility
    /// `node_id` replaced.
    ///
    /// This is primarily useful to small reference hosts that intentionally
    /// replay one atomic targeted action through an existing single-node
    /// reducer. Production hosts can consume [`Self::targeted_node_action`]
    /// directly and apply one multi-node document operation.
    pub fn retargeted_legacy_node_action(&self, node_id: impl Into<SharedString>) -> Option<Self> {
        let mut action = self.clone();
        *action.legacy_node_id_mut()? = node_id.into();
        Some(action)
    }

    pub(crate) fn legacy_node_id_mut(&mut self) -> Option<&mut SharedString> {
        match self {
            Self::TypographyPropertyChangeRequested { node_id, .. }
            | Self::TypographyPropertyEditRequested { node_id, .. }
            | Self::TypographyVariableAxisEditRequested { node_id, .. }
            | Self::TypographyStyleApplyRequested { node_id, .. }
            | Self::TypographyStyleDetachRequested { node_id, .. }
            | Self::TypographyFontApplyRequested { node_id, .. }
            | Self::TypographyFontImportRequested { node_id, .. }
            | Self::TypographyOpenTypeFeatureChangeRequested { node_id, .. }
            | Self::TextPathFlipOrientationRequested { node_id }
            | Self::TextPathStartChangeRequested { node_id, .. }
            | Self::VectorVertexSelectionEditRequested { node_id, .. }
            | Self::VectorVertexPositionEditRequested { node_id, .. }
            | Self::VectorVertexCornerRadiusEditRequested { node_id, .. }
            | Self::VectorHandleMirroringEditRequested { node_id, .. }
            | Self::PropertyChangeRequested { node_id, .. }
            | Self::PropertyEditRequested { node_id, .. }
            | Self::DimensionLimitsPreviewRequested { node_id, .. }
            | Self::PropertyVariableApplyRequested { node_id, .. }
            | Self::PropertyVariableImportRequested { node_id, .. }
            | Self::PropertyVariableDetachRequested { node_id, .. }
            | Self::SectionShareRequested { node_id }
            | Self::SectionResolveChangedStatusRequested { node_id }
            | Self::TransformModifierAddRequested { node_id, .. }
            | Self::TransformModifierRemoveRequested { node_id, .. }
            | Self::TransformModifierChangeRequested { node_id, .. }
            | Self::ApplyTransformModifiersRequested { node_id }
            | Self::ComponentPropertyDefinitionCreateRequested { node_id, .. }
            | Self::ComponentPropertyDefinitionRenameRequested { node_id, .. }
            | Self::ComponentPropertyDefinitionMetadataEditRequested { node_id, .. }
            | Self::ComponentPropertyDefinitionEditRequested { node_id, .. }
            | Self::ComponentPropertyDefinitionDeleteRequested { node_id, .. }
            | Self::ComponentPropertyDefinitionReorderRequested { node_id, .. }
            | Self::ComponentVariantOptionCreateRequested { node_id, .. }
            | Self::ComponentVariantOptionRenameRequested { node_id, .. }
            | Self::ComponentVariantOptionDeleteRequested { node_id, .. }
            | Self::ComponentVariantOptionReorderRequested { node_id, .. }
            | Self::ComponentPropertyApplyToLayerRequested { node_id, .. }
            | Self::ComponentPropertySwitchOnLayerRequested { node_id, .. }
            | Self::ComponentPropertyDetachFromLayerRequested { node_id, .. }
            | Self::NestedComponentPropertyExposeRequested { node_id, .. }
            | Self::NestedComponentPropertyUnexposeRequested { node_id, .. }
            | Self::NestedComponentPropertyPreviewRequested { node_id, .. }
            | Self::ComponentPropertyChangeRequested { node_id, .. }
            | Self::ComponentPropertyEditRequested { node_id, .. }
            | Self::ComponentPropertyResetRequested { node_id, .. }
            | Self::ComponentPropertyVariableApplyRequested { node_id, .. }
            | Self::ComponentPropertyVariableImportRequested { node_id, .. }
            | Self::ComponentPropertyVariableDetachRequested { node_id, .. }
            | Self::ComponentSwapApplyRequested { node_id, .. }
            | Self::ComponentSwapImportRequested { node_id, .. }
            | Self::ComponentSwapPreviewRequested { node_id, .. }
            | Self::ComponentPropertyNestedInstanceSelectRequested { node_id, .. }
            | Self::ComponentPropertyNestedInstanceGoToMainRequested { node_id, .. }
            | Self::SlotSettingsChangeRequested { node_id, .. }
            | Self::SlotResetRequested { node_id, .. }
            | Self::SlotClearRequested { node_id, .. }
            | Self::SlotAddInstanceRequested { node_id, .. }
            | Self::SlotChildSelectRequested { node_id, .. }
            | Self::SlotLimitLayersSelectRequested { node_id, .. }
            | Self::SlotChildRemoveRequested { node_id, .. }
            | Self::SlotChildReorderRequested { node_id, .. }
            | Self::SlotChildReplaceRequested { node_id, .. }
            | Self::CollectionItemAddRequested { node_id, .. }
            | Self::CollectionItemRemoveRequested { node_id, .. }
            | Self::EffectAddRequested { node_id, .. }
            | Self::EffectRemoveRequested { node_id, .. }
            | Self::EffectReorderRequested { node_id, .. }
            | Self::EffectEditRequested { node_id, .. }
            | Self::EffectShaderChooseRequested { node_id, .. }
            | Self::EffectShaderPropertyEditorRequested { node_id, .. }
            | Self::EffectShaderPropertyVariableDetachRequested { node_id, .. }
            | Self::EffectStyleApplyRequested { node_id, .. }
            | Self::EffectStyleCreateRequested { node_id, .. }
            | Self::EffectStyleDetachRequested { node_id, .. }
            | Self::EffectVariableApplyRequested { node_id, .. }
            | Self::EffectVariableDetachRequested { node_id, .. }
            | Self::GridDimensionsEditRequested { node_id, .. }
            | Self::GridTrackAddRequested { node_id, .. }
            | Self::GridTrackDeleteRequested { node_id, .. }
            | Self::GridTracksReorderRequested { node_id, .. }
            | Self::LayoutGridStyleApplyRequested { node_id, .. }
            | Self::LayoutGridStyleCreateRequested { node_id, .. }
            | Self::LayoutGridStyleDetachRequested { node_id, .. }
            | Self::LayoutGridStyleImportRequested { node_id, .. }
            | Self::LayoutGridPropertyEditRequested { node_id, .. }
            | Self::LayoutGridRemoveRequested { node_id, .. }
            | Self::LayoutGridVariableApplyRequested { node_id, .. }
            | Self::LayoutGridVariableImportRequested { node_id, .. }
            | Self::LayoutGridVariableDetachRequested { node_id, .. }
            | Self::LayoutGridVariableCreateRequested { node_id, .. }
            | Self::LayoutGridCountVariableApplyRequested { node_id, .. }
            | Self::LayoutGridCountVariableDetachRequested { node_id, .. }
            | Self::PaintChangeRequested { node_id, .. }
            | Self::PaintEditRequested { node_id, .. }
            | Self::PaintReorderRequested { node_id, .. }
            | Self::PaintStyleApplyRequested { node_id, .. }
            | Self::PaintStyleImportRequested { node_id, .. }
            | Self::PaintStyleCreateRequested { node_id, .. }
            | Self::PaintStyleDetachRequested { node_id, .. }
            | Self::PaintSourceReplaceRequested { node_id, .. }
            | Self::PaintMediaSourceActionRequested { node_id, .. }
            | Self::PaintMediaSourceDropRequested { node_id, .. }
            | Self::PaintMediaCropActionRequested { node_id, .. }
            | Self::PaintVideoPreviewActionRequested { node_id, .. }
            | Self::PaintShaderImportRequested { node_id, .. }
            | Self::PaintShaderApplyRequested { node_id, .. }
            | Self::PaintShaderPropertyBindRequested { node_id, .. }
            | Self::PaintShaderPropertyEditorRequested { node_id, .. }
            | Self::PaintShaderPropertyDetachRequested { node_id, .. }
            | Self::PaintColorVariableApplyRequested { node_id, .. }
            | Self::PaintColorVariableImportRequested { node_id, .. }
            | Self::PaintColorVariableDetachRequested { node_id, .. }
            | Self::PaintColorVariableCreateRequested { node_id, .. }
            | Self::PaintColorStyleSampleRequested { node_id, .. }
            | Self::PaintColorStyleApplyRequested { node_id, .. }
            | Self::PaintColorStyleCreateRequested { node_id, .. }
            | Self::PaintEyedropperRequested { node_id, .. }
            | Self::ExportRequested { node_id, .. }
            | Self::FramePresetApplyRequested { node_id, .. }
            | Self::SwapStrokeEndpointsRequested { node_id }
            | Self::ReplaceMediaRequested { node_id }
            | Self::ResetInstanceOverridesRequested { node_id }
            | Self::GoToMainComponentRequested { node_id }
            | Self::DetachInstanceRequested { node_id } => Some(node_id),
            Self::PageBackgroundChangeRequested { .. }
            | Self::PageBackgroundEditRequested { .. }
            | Self::LocalResourceBrowseRequested { .. }
            | Self::LocalResourceOpenRequested { .. }
            | Self::LocalResourceCreateRequested { .. }
            | Self::LocalResourceImportRequested { .. }
            | Self::LocalStyleCommandRequested { .. }
            | Self::LocalStyleCreateRequested { .. }
            | Self::LocalStyleFolderCreateRequested { .. }
            | Self::LocalStylesDeleteRequested { .. }
            | Self::LocalStylesMoveRequested { .. }
            | Self::VariablesViewOpenRequested { .. }
            | Self::VariableModeApplyRequested { .. }
            | Self::VariableModeClearRequested { .. }
            | Self::SurfaceChangeRequested { .. }
            | Self::SelectionHeaderCommandRequested { .. }
            | Self::AddAutoLayoutRequested { .. }
            | Self::TargetedNodeActionRequested { .. }
            | Self::MenuPreviewRequested { .. }
            | Self::PropertyCopyRequested { .. }
            | Self::ViewerSectionCopyRequested { .. }
            | Self::ViewerSectionRepresentationChangeRequested { .. }
            | Self::ExportConfigurationAddRequested { .. }
            | Self::ExportConfigurationRemoveRequested { .. }
            | Self::ExportConfigurationChangeRequested { .. }
            | Self::ExportModeChangeRequested { .. }
            | Self::AnimatedExportChangeRequested { .. }
            | Self::AnimatedExportRequested { .. }
            | Self::ExportAllRequested { .. }
            | Self::ExportPreviewRequested { .. }
            | Self::SmartSelectionSpacingEditRequested { .. }
            | Self::SmartSelectionArrangeRequested { .. }
            | Self::ArrangeRequested { .. }
            | Self::TransformRequested { .. }
            | Self::ResizeToFitRequested { .. }
            | Self::SelectionColorEditRequested { .. }
            | Self::SelectionColorPaintEditRequested { .. }
            | Self::SelectionColorOccurrencesSelectRequested { .. }
            | Self::SelectionColorPaintStyleApplyRequested { .. }
            | Self::SelectionColorPaintStyleImportRequested { .. }
            | Self::SelectionColorPaintStyleCreateRequested { .. }
            | Self::SelectionColorPaintStyleDetachRequested { .. }
            | Self::SelectionColorVariableApplyRequested { .. }
            | Self::SelectionColorVariableImportRequested { .. }
            | Self::SelectionColorVariableCreateRequested { .. }
            | Self::SelectionColorVariableDetachRequested { .. } => None,
        }
    }

    /// Returns the compatibility classification for actions the current panel
    /// never emits as canonical behavior.
    pub const fn compatibility_path(&self) -> Option<DesignPanelCompatibilityPath> {
        match self {
            Self::PaintChangeRequested { .. } => {
                Some(DesignPanelCompatibilityPath::WholePaintAction)
            }
            Self::PaintColorStyleApplyRequested { .. }
            | Self::PaintColorStyleCreateRequested { .. } => {
                Some(DesignPanelCompatibilityPath::ConflatedColorStyleAction)
            }
            Self::TargetedNodeActionRequested { .. } => None,
            Self::ReplaceMediaRequested { .. } => {
                Some(DesignPanelCompatibilityPath::NodeMediaReplaceAction)
            }
            Self::ExportRequested { .. } => Some(DesignPanelCompatibilityPath::SingleExportAction),
            Self::PropertyChangeRequested { property, .. }
            | Self::PropertyEditRequested { property, .. }
                if property.layout_grid_index().is_some() =>
            {
                Some(DesignPanelCompatibilityPath::IndexedLayoutGridPropertyAction)
            }
            Self::CollectionItemRemoveRequested {
                collection: DesignPanelCollection::LayoutGrid,
                ..
            } => Some(DesignPanelCompatibilityPath::IndexedLayoutGridRemoveAction),
            Self::LayoutGridCountVariableApplyRequested { .. }
            | Self::LayoutGridCountVariableDetachRequested { .. } => {
                Some(DesignPanelCompatibilityPath::CountOnlyLayoutGridVariableAction)
            }
            Self::PropertyChangeRequested { property, .. }
            | Self::PropertyEditRequested { property, .. }
            | Self::EffectEditRequested { property, .. } => property.compatibility_path(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
