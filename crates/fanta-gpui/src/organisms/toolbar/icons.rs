//! Lucide icon mapping for toolbar tools and modes.
//!
//! This module contains semantic mappings only. Geometry is owned by the
//! pinned Lucide catalog and rendered by the shared atom.

use gpui::{AnyElement, Hsla, IntoElement as _, Styled as _, px};
use gpui_component::{Icon, IconName, Sizable as _};

use crate::atoms::{LucideIcon, render_lucide_icon};

use super::{ToolbarCommand, ToolbarMode, ToolbarTool};

pub(crate) fn render_tool_icon(tool: ToolbarTool, color: Hsla, size: f32) -> AnyElement {
    render_lucide_icon(tool_icon(tool), color, size)
}

pub(crate) fn render_mode_icon(mode: ToolbarMode, color: Hsla, size: f32) -> AnyElement {
    render_lucide_icon(mode_icon(mode), color, size)
}

/// Renders one of gpui-component's bundled Lucide assets.
pub(crate) fn render_icon_asset(icon: IconName, color: Hsla, size: f32) -> AnyElement {
    Icon::new(icon)
        .with_size(px(size))
        .text_color(color)
        .into_any_element()
}

pub(crate) const fn mode_icon(mode: ToolbarMode) -> LucideIcon {
    match mode {
        ToolbarMode::Design => LucideIcon::SquareDashedMousePointer,
        ToolbarMode::Motion => LucideIcon::Clapperboard,
        ToolbarMode::Draw => LucideIcon::Brush,
        ToolbarMode::Dev => LucideIcon::CodeXml,
    }
}

pub(crate) const fn tool_icon(tool: ToolbarTool) -> LucideIcon {
    match tool {
        ToolbarTool::Brush => LucideIcon::Brush,
        ToolbarTool::Eraser => LucideIcon::Eraser,
        ToolbarTool::RectangleSelect => LucideIcon::SquareDashed,
        ToolbarTool::EllipseSelect => LucideIcon::CircleDashed,
        ToolbarTool::Lasso => LucideIcon::Lasso,
        ToolbarTool::PolygonalLasso => LucideIcon::LassoSelect,
        ToolbarTool::MagicWand => LucideIcon::Wand,
        ToolbarTool::Crop => LucideIcon::Crop,
        ToolbarTool::Move => LucideIcon::MousePointer2,
        ToolbarTool::Hand => LucideIcon::Hand,
        ToolbarTool::Scale => LucideIcon::Scaling,
        ToolbarTool::Frame => LucideIcon::Frame,
        ToolbarTool::Section => LucideIcon::PanelTop,
        ToolbarTool::Slice => LucideIcon::ScanLine,
        ToolbarTool::Rectangle => LucideIcon::Square,
        ToolbarTool::Line => LucideIcon::Slash,
        ToolbarTool::Arrow => LucideIcon::ArrowUpRight,
        ToolbarTool::Ellipse => LucideIcon::Circle,
        ToolbarTool::Polygon => LucideIcon::Pentagon,
        ToolbarTool::Star => LucideIcon::Star,
        ToolbarTool::ImageVideo => LucideIcon::Image,
        ToolbarTool::Pen => LucideIcon::PenTool,
        ToolbarTool::Pencil => LucideIcon::Pencil,
        ToolbarTool::PathSelect => LucideIcon::MousePointerClick,
        ToolbarTool::NodeEdit => LucideIcon::Spline,
        ToolbarTool::Text => LucideIcon::TypeIcon,
        ToolbarTool::TextPath => LucideIcon::Baseline,
        ToolbarTool::Comment => LucideIcon::MessageSquare,
        ToolbarTool::Annotation => LucideIcon::StickyNote,
        ToolbarTool::Measure => LucideIcon::Ruler,
        ToolbarTool::Resources => LucideIcon::LayoutDashboard,
        ToolbarTool::Actions => LucideIcon::Sparkles,
        ToolbarTool::Inspect => LucideIcon::Search,
        ToolbarTool::ColorPicker => LucideIcon::Pipette,
        ToolbarTool::Code => LucideIcon::CodeXml,
        ToolbarTool::Variables => LucideIcon::Variable,
        ToolbarTool::ReadyForDev => LucideIcon::CircleCheck,
        ToolbarTool::MotionSelect => LucideIcon::MousePointer2,
        ToolbarTool::AddKeyframe => LucideIcon::DiamondPlus,
        ToolbarTool::MotionPath => LucideIcon::Route,
        ToolbarTool::AnimationStyle => LucideIcon::WandSparkles,
        ToolbarTool::TimeComment => LucideIcon::Clock,
        ToolbarTool::AutoKeyframe => LucideIcon::RefreshCw,
        ToolbarTool::PlayPreview => LucideIcon::CirclePlay,
    }
}

/// One distinct, semantic icon for each searchable command.
pub(crate) const fn command_icon(command: ToolbarCommand) -> LucideIcon {
    match command {
        ToolbarCommand::Undo => LucideIcon::Undo2,
        ToolbarCommand::Redo => LucideIcon::Redo2,
        ToolbarCommand::Cut => LucideIcon::Scissors,
        ToolbarCommand::Copy => LucideIcon::Copy,
        ToolbarCommand::Paste => LucideIcon::ClipboardPaste,
        ToolbarCommand::Duplicate => LucideIcon::CopyPlus,
        ToolbarCommand::Delete => LucideIcon::Trash,
        ToolbarCommand::SelectAll => LucideIcon::SquareDashedMousePointer,
        ToolbarCommand::DeselectAll => LucideIcon::SquareDashed,
        ToolbarCommand::Group => LucideIcon::Group,
        ToolbarCommand::Ungroup => LucideIcon::Ungroup,
        ToolbarCommand::FrameSelection => LucideIcon::Frame,
        ToolbarCommand::AddAutoLayout => LucideIcon::PanelsTopLeft,
        ToolbarCommand::CreateComponent => LucideIcon::Component,
        ToolbarCommand::DetachInstance => LucideIcon::Unplug,
        ToolbarCommand::FindAndReplace => LucideIcon::Replace,
        ToolbarCommand::ToggleRulers => LucideIcon::Ruler,
        ToolbarCommand::ToggleLayoutGrids => LucideIcon::Grid2x2,
        ToolbarCommand::ToggleOutlines => LucideIcon::ScanLine,
        ToolbarCommand::TogglePixelPreview => LucideIcon::Grid3x3,
        ToolbarCommand::ToggleUi => LucideIcon::PanelsLeftBottom,
        ToolbarCommand::MinimizeUi => LucideIcon::Minimize2,
        ToolbarCommand::ZoomToFit => LucideIcon::Scan,
        ToolbarCommand::ZoomToSelection => LucideIcon::Focus,
        ToolbarCommand::Import => LucideIcon::Download,
        ToolbarCommand::Export => LucideIcon::Upload,
        ToolbarCommand::PlaceImageVideo => LucideIcon::ImagePlus,
        ToolbarCommand::OpenResources => LucideIcon::Library,
        ToolbarCommand::OpenPlugins => LucideIcon::Plug,
        ToolbarCommand::OpenWidgets => LucideIcon::Blocks,
        ToolbarCommand::OpenVariables => LucideIcon::Variable,
        ToolbarCommand::GenerateDesign => LucideIcon::LayoutTemplate,
        ToolbarCommand::ReplaceContent => LucideIcon::TextCursorInput,
        ToolbarCommand::RewriteText => LucideIcon::TextInitial,
        ToolbarCommand::TranslateText => LucideIcon::Languages,
        ToolbarCommand::RenameLayers => LucideIcon::ListOrdered,
        ToolbarCommand::RemoveBackground => LucideIcon::ImageMinus,
        ToolbarCommand::GenerateImage => LucideIcon::Image,
        ToolbarCommand::GenerateVideo => LucideIcon::Video,
        ToolbarCommand::GenerateVector => LucideIcon::PenTool,
        ToolbarCommand::GenerateMasks => LucideIcon::VenetianMask,
        ToolbarCommand::MakePrototype => LucideIcon::Workflow,
        ToolbarCommand::OpenDesignMode => LucideIcon::SquarePen,
        ToolbarCommand::OpenMotionMode => LucideIcon::Clapperboard,
        ToolbarCommand::OpenDrawMode => LucideIcon::Brush,
        ToolbarCommand::OpenDevMode => LucideIcon::CodeXml,
        ToolbarCommand::ViewVersionHistory => LucideIcon::RotateCcw,
        ToolbarCommand::CopyLink => LucideIcon::Link,
        ToolbarCommand::KeyboardShortcuts => LucideIcon::Keyboard,
        ToolbarCommand::Preferences => LucideIcon::Settings2,
        ToolbarCommand::Present => LucideIcon::Presentation,
        ToolbarCommand::Share => LucideIcon::Share2,
    }
}

pub(crate) const fn secondary_icon(control: super::ToolbarSecondaryControl) -> LucideIcon {
    use super::ToolbarSecondaryControl as C;
    match control {
        C::DevInspect => LucideIcon::Search,
        C::DevAnnotate => LucideIcon::StickyNote,
        C::DevMeasure => LucideIcon::Ruler,
        C::DevReadyForDevelopment => LucideIcon::CircleCheck,
        C::MotionPlayPause => LucideIcon::Play,
        C::MotionLoop => LucideIcon::Repeat,
        C::MotionAutoKeyframe => LucideIcon::Diamond,
        C::MotionAddKeyframe => LucideIcon::DiamondPlus,
        C::MotionAnimationStyle => LucideIcon::WandSparkles,
        C::MotionTimeline => LucideIcon::ChartGantt,
        C::MotionTimeComment => LucideIcon::MessageSquare,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_searchable_action_has_a_distinct_icon() {
        let icons: std::collections::HashSet<_> = ToolbarCommand::ALL
            .iter()
            .map(|command| command_icon(*command).svg_str())
            .collect();
        assert_eq!(icons.len(), ToolbarCommand::ALL.len());
    }

    #[test]
    fn every_toolbar_item_maps_to_an_official_lucide_icon() {
        for &tool in ToolbarTool::ALL {
            assert!(!tool_icon(tool).svg_str().is_empty());
        }
        for &mode in ToolbarMode::ALL {
            assert!(!mode_icon(mode).svg_str().is_empty());
        }
    }
}
