use super::super::DesignPanelSection;

/// A complete property section that may be mounted independently or included
/// in [`super::DesignInspector`]. Alignment includes position and constraints;
/// a separate Constraints kind preserves explicit host section projections.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DesignPropertyPanelKind {
    Alignment,
    Layout,
    Appearance,
    SelectionColors,
    Component,
    Section,
    Transform,
    Geometry,
    Mask,
    Typography,
    Fill,
    Stroke,
    Effects,
    LayoutGuides,
    Export,
    Constraints,
    Page,
    Viewer,
}

impl DesignPropertyPanelKind {
    pub const ALL: &'static [Self] = &[
        Self::Alignment,
        Self::Layout,
        Self::Appearance,
        Self::SelectionColors,
        Self::Component,
        Self::Section,
        Self::Transform,
        Self::Geometry,
        Self::Mask,
        Self::Typography,
        Self::Fill,
        Self::Stroke,
        Self::Effects,
        Self::LayoutGuides,
        Self::Export,
        Self::Constraints,
        Self::Page,
        Self::Viewer,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Alignment => "Alignment and position",
            Self::Layout => "Layout",
            Self::Appearance => "Appearance",
            Self::SelectionColors => "Selection colors",
            Self::Component => "Component",
            Self::Section => "Section",
            Self::Transform => "Transform",
            Self::Geometry => "Geometry",
            Self::Mask => "Mask",
            Self::Typography => "Typography",
            Self::Fill => "Fill",
            Self::Stroke => "Stroke",
            Self::Effects => "Effects",
            Self::LayoutGuides => "Layout guides",
            Self::Export => "Export",
            Self::Constraints => "Constraints",
            Self::Page => "Page",
            Self::Viewer => "Properties",
        }
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Self::Alignment => "alignment",
            Self::Layout => "layout",
            Self::Appearance => "appearance",
            Self::SelectionColors => "selection-colors",
            Self::Component => "component",
            Self::Section => "section",
            Self::Transform => "transform",
            Self::Geometry => "geometry",
            Self::Mask => "mask",
            Self::Typography => "typography",
            Self::Fill => "fill",
            Self::Stroke => "stroke",
            Self::Effects => "effects",
            Self::LayoutGuides => "layout-guides",
            Self::Export => "export",
            Self::Constraints => "constraints",
            Self::Page => "page",
            Self::Viewer => "viewer",
        }
    }

    pub(super) const fn from_section(section: DesignPanelSection) -> Option<Self> {
        Some(match section {
            DesignPanelSection::Position => Self::Alignment,
            DesignPanelSection::Layout => Self::Layout,
            DesignPanelSection::Layer => Self::Appearance,
            DesignPanelSection::Selection => Self::SelectionColors,
            DesignPanelSection::Component | DesignPanelSection::Instance => Self::Component,
            DesignPanelSection::Section => Self::Section,
            DesignPanelSection::Transform => Self::Transform,
            DesignPanelSection::Geometry => Self::Geometry,
            DesignPanelSection::Mask => Self::Mask,
            DesignPanelSection::Typography => Self::Typography,
            DesignPanelSection::Fill => Self::Fill,
            DesignPanelSection::Stroke => Self::Stroke,
            DesignPanelSection::Effects => Self::Effects,
            DesignPanelSection::LayoutGrid => Self::LayoutGuides,
            DesignPanelSection::Export => Self::Export,
            DesignPanelSection::Constraints => Self::Constraints,
            // Media settings are already part of the paint picker.
            DesignPanelSection::Media => return None,
        })
    }
}
