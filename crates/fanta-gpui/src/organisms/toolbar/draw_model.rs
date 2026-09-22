use gpui::SharedString;

/// Host-owned paint and selection settings. Distances use document pixels.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrawToolbarOptions {
    pub brush_tip: SharedString,
    pub brush_tips: Vec<SharedString>,
    pub size: u16,
    pub hardness: u8,
    pub opacity: u8,
    pub flow: u8,
    pub smoothing: u8,
    pub pressure: bool,
    pub selection_operation: DrawSelectionOperation,
    pub feather: u16,
    pub anti_alias: bool,
    pub tolerance: u8,
    pub contiguous: bool,
    pub crop_ratio: SharedString,
    pub crop_ratios: Vec<SharedString>,
    pub delete_cropped_pixels: bool,
}
impl Default for DrawToolbarOptions {
    fn default() -> Self {
        Self {
            brush_tip: "Round".into(),
            brush_tips: ["Round", "Soft round", "Flat", "Pencil", "Ink", "Charcoal"]
                .into_iter()
                .map(Into::into)
                .collect(),
            size: 24,
            hardness: 100,
            opacity: 100,
            flow: 100,
            smoothing: 0,
            pressure: true,
            selection_operation: DrawSelectionOperation::Replace,
            feather: 0,
            anti_alias: true,
            tolerance: 32,
            contiguous: true,
            crop_ratio: "Freeform".into(),
            crop_ratios: ["Freeform", "Original", "1:1", "4:3", "3:2", "16:9"]
                .into_iter()
                .map(Into::into)
                .collect(),
            delete_cropped_pixels: false,
        }
    }
}
impl DrawToolbarOptions {
    pub fn normalized(mut self) -> Self {
        self.size = self.size.clamp(1, 5000);
        self.hardness = self.hardness.min(100);
        self.opacity = self.opacity.min(100);
        self.flow = self.flow.min(100);
        self.smoothing = self.smoothing.min(100);
        self.feather = self.feather.min(1000);
        self
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DrawSelectionOperation {
    #[default]
    Replace,
    Add,
    Subtract,
    Intersect,
}
impl DrawSelectionOperation {
    pub const ALL: [Self; 4] = [Self::Replace, Self::Add, Self::Subtract, Self::Intersect];
    pub const fn label(self) -> &'static str {
        match self {
            Self::Replace => "New selection",
            Self::Add => "Add to selection",
            Self::Subtract => "Subtract from selection",
            Self::Intersect => "Intersect selection",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawToolbarAction {
    SelectAll,
    Deselect,
    InvertSelection,
    ApplyCrop,
    CancelCrop,
    ClosePath,
    JoinPaths,
    SimplifyPath,
}
