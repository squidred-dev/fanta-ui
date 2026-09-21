//! The storybook's screen tier: one module per story plus the story
//! registry, the knob framework, and the shared reference-fixture chrome.
//!
//! Adding a story means adding one screen module here and one
//! `StoryDescriptor` entry to [`REGISTRY`]. The registry is the single
//! source for sidebar grouping, render/focus dispatch, env-name parsing,
//! and window sizing; nothing else may hold a per-story match.

pub(crate) mod buttons;
pub(crate) mod checkbox;
pub(crate) mod color_picker;
pub(crate) mod color_system;
pub(crate) mod design;
pub(crate) mod dropdown;
pub(crate) mod fields;
pub(crate) mod file_inspector;
pub(crate) mod harness;
pub(crate) mod icons;
pub(crate) mod inputs;
pub(crate) mod intent;
pub(crate) mod knobs;
pub(crate) mod labels;
pub(crate) mod layers;
pub(crate) mod list_rows;
pub(crate) mod menus;
pub(crate) mod overlays;
pub(crate) mod pages;
pub(crate) mod popups;
pub(crate) mod prototype;
pub(crate) mod pseudo_editor;
pub(crate) mod radio_button;
pub(crate) mod segmented_control;
pub(crate) mod sliders;
pub(crate) mod spec;
pub(crate) mod specimen;
pub(crate) mod structure;
pub(crate) mod tabs;
pub(crate) mod timeline;
pub(crate) mod tokens;
pub(crate) mod toolbar;
pub(crate) mod tooltips;
pub(crate) mod typography;
pub(crate) mod variables;
pub(crate) mod viewport;
pub(crate) mod welcome;

pub(crate) use buttons::ButtonsScreen;
pub(crate) use checkbox::CheckboxStory;
pub(crate) use design::DesignScreen;
pub(crate) use dropdown::DropdownStory;
pub(crate) use fields::FieldsScreen;
pub(crate) use file_inspector::FileInspectorScreen;
pub(crate) use icons::IconsScreen;
pub(crate) use inputs::InputsStory;
pub(crate) use labels::LabelsScreen;
pub(crate) use layers::LayersScreen;
pub(crate) use list_rows::ListRowsScreen;
pub(crate) use menus::MenusScreen;
pub(crate) use overlays::OverlaysScreen;
pub(crate) use pages::PagesScreen;
pub(crate) use popups::PopupsScreen;
pub(crate) use prototype::PrototypeScreen;
pub(crate) use pseudo_editor::PseudoEditorScreen;
pub(crate) use radio_button::RadioButtonStory;
pub(crate) use segmented_control::SegmentedControlStory;
pub(crate) use structure::StructureScreen;
pub(crate) use tabs::TabsStory;
pub(crate) use timeline::TimelineScreen;
pub(crate) use tokens::TokensScreen;
pub(crate) use toolbar::ToolbarScreen;
pub(crate) use tooltips::TooltipsStory;
pub(crate) use variables::VariablesStory;
pub(crate) use viewport::ViewportPreset;
pub(crate) use welcome::WelcomeScreen;

use crate::*;

/// The sidebar sections, mirroring the library's atomic design tiers
/// (ARCHITECTURE.md §16). Sections render in [`StorySection::ALL`] order:
/// Getting started, then Atoms → Molecules → Organisms → Layouts → Screens.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StorySection {
    GettingStarted,
    Atoms,
    Molecules,
    Organisms,
    Layouts,
    Screens,
}

impl StorySection {
    pub(crate) const ALL: [Self; 6] = [
        Self::GettingStarted,
        Self::Atoms,
        Self::Molecules,
        Self::Organisms,
        Self::Layouts,
        Self::Screens,
    ];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::GettingStarted => "Getting started",
            Self::Atoms => "Atoms",
            Self::Molecules => "Molecules",
            Self::Organisms => "Organisms",
            Self::Layouts => "Layouts",
            Self::Screens => "Screens",
        }
    }
}

/// One entry in a story's keyboard-help listing.
pub(crate) struct KeyboardHint {
    pub(crate) keys: &'static str,
    pub(crate) action: &'static str,
}

impl KeyboardHint {
    pub(crate) const fn new(keys: &'static str, action: &'static str) -> Self {
        Self { keys, action }
    }
}

/// Keyboard affordances every story shares, listed ahead of the
/// story-specific hints by the gallery help panel.
pub(crate) const SHARED_KEYBOARD_HINTS: &[KeyboardHint] = &[
    KeyboardHint::new("Tab / Shift-Tab", "Move keyboard focus"),
    KeyboardHint::new("Enter / Space", "Activate the focused control"),
    KeyboardHint::new(
        "← → ↑ ↓ / + − on the viewport handles",
        "Resize the story viewport (Shift: 32 px)",
    ),
];

/// Everything the development host needs to know about one story.
pub(crate) struct StoryDescriptor {
    pub(crate) kind: StoryKind,
    /// The canonical launch id accepted by `FANTA_STORYBOOK_STORY`.
    pub(crate) id: &'static str,
    /// Additional accepted launch names for the same story.
    pub(crate) aliases: &'static [&'static str],
    pub(crate) title: &'static str,
    /// Short label used by the reference-fixture navigation row.
    pub(crate) nav_label: &'static str,
    pub(crate) description: &'static str,
    pub(crate) section: StorySection,
    pub(crate) reference_window_size: (f32, f32),
    pub(crate) gallery_surface_size: (f32, f32),
    pub(crate) gallery_fluid_width: bool,
    /// The component's natural sizes, offered as viewport preset chips by
    /// the shared story viewport harness.
    pub(crate) viewport_presets: &'static [ViewportPreset],
    /// Story-specific key bindings listed by the gallery keyboard help.
    /// [`SHARED_KEYBOARD_HINTS`] is always shown ahead of these.
    pub(crate) keyboard_hints: &'static [KeyboardHint],
    /// The bare story component, used by shared story windows.
    pub(crate) render_story: fn(&Storybook, &mut Context<Storybook>) -> AnyElement,
    /// Gallery surface override when the story mounts a richer harness.
    pub(crate) render_gallery: Option<fn(&Storybook, &mut Context<Storybook>) -> AnyElement>,
    /// The full-window reference fixture behind `FANTA_STORYBOOK_STORY`.
    pub(crate) render_reference: fn(&Storybook, &mut Context<Storybook>) -> AnyElement,
    /// Runtime knob rows mounted under the Gallery surface.
    pub(crate) render_knobs: Option<fn(&Storybook, &mut Context<Storybook>) -> AnyElement>,
    pub(crate) focus: fn(&Storybook, &mut Window, &mut Context<Storybook>),
    pub(crate) last_action: fn(&Storybook) -> SharedString,
}

impl StoryDescriptor {
    pub(crate) fn matches_query(&self, query: &str) -> bool {
        query.is_empty()
            || self.title.to_ascii_lowercase().contains(query)
            || self.description.to_ascii_lowercase().contains(query)
    }

    /// The smallest registered viewport for this story. Story windows use
    /// it as their scroll floor: below it the window scrolls instead of
    /// clipping the story.
    pub(crate) fn min_story_size(&self) -> (f32, f32) {
        self.viewport_presets
            .iter()
            .fold(self.gallery_surface_size, |(width, height), preset| {
                (width.min(preset.width), height.min(preset.height))
            })
    }
}

pub(crate) fn registry() -> &'static [StoryDescriptor] {
    &REGISTRY
}

pub(crate) fn story_from_name(value: &str) -> Option<StoryKind> {
    let value = value.to_ascii_lowercase();
    registry()
        .iter()
        .find(|descriptor| descriptor.id == value || descriptor.aliases.contains(&value.as_str()))
        .map(|descriptor| descriptor.kind)
}

impl StoryKind {
    pub(crate) fn descriptor(self) -> &'static StoryDescriptor {
        registry()
            .iter()
            .find(|descriptor| descriptor.kind == self)
            .expect("every story registers exactly one descriptor")
    }

    pub(crate) fn title(self) -> &'static str {
        self.descriptor().title
    }

    pub(crate) fn description(self) -> &'static str {
        self.descriptor().description
    }

    pub(crate) fn reference_window_size(self) -> (f32, f32) {
        self.descriptor().reference_window_size
    }

    pub(crate) fn gallery_surface_size(self) -> (f32, f32) {
        self.descriptor().gallery_surface_size
    }
}

static REGISTRY: [StoryDescriptor; 34] = [
    StoryDescriptor {
        kind: StoryKind::Typography,
        id: "typography",
        aliases: &["type", "type-tokens"],
        title: "Typography",
        nav_label: "Typography",
        description: "Figma UI3 display, heading, and body roles applied through reusable semantic typography tokens.",
        section: StorySection::Atoms,
        reference_window_size: (1240., 900.),
        gallery_surface_size: (900., 880.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Sheet", 720., 760.),
            ViewportPreset::new("Default", 900., 880.),
        ],
        keyboard_hints: &[],
        render_story: |story, cx| story.render_typography_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_typography_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| story.tokens_screen.focus_handle.focus(window, cx),
        last_action: |_| "Review semantic typography roles".into(),
    },
    StoryDescriptor {
        kind: StoryKind::ColorSystem,
        id: "color-system",
        aliases: &["semantic-colors", "colors"],
        title: "Color system",
        nav_label: "Color system",
        description: "Figma UI3 semantic border, background, icon, and text roles mapped live to the active GPUI theme.",
        section: StorySection::Atoms,
        reference_window_size: (1240., 900.),
        gallery_surface_size: (900., 880.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Sheet", 720., 760.),
            ViewportPreset::new("Default", 900., 880.),
            ViewportPreset::new("Wide", 1120., 900.),
        ],
        keyboard_hints: &[],
        render_story: |story, cx| story.render_color_system_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_color_system_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| story.tokens_screen.focus_handle.focus(window, cx),
        last_action: |_| "Review semantic colors from the active theme".into(),
    },
    StoryDescriptor {
        kind: StoryKind::Sliders,
        id: "slider",
        aliases: &[],
        title: "Slider",
        nav_label: "Slider",
        description: "Theme-aware slider family: states, variants, pointer and keyboard interactions.",
        section: StorySection::Molecules,
        reference_window_size: (720., 800.),
        gallery_surface_size: (440., 640.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Compact", 320., 640.),
            ViewportPreset::new("Default", 440., 640.),
        ],
        keyboard_hints: &[
            KeyboardHint::new("Arrows / Shift+Arrows", "Adjust value / coarse step"),
            KeyboardHint::new("Home / End", "Minimum / maximum"),
        ],
        render_story: |story, cx| story.render_sliders_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_sliders_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| {
            story.sliders_screen.sliders[0]
                .focus_handle(cx)
                .focus(window, cx)
        },
        last_action: |story| story.sliders_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::SliderBackgrounds,
        id: "slider-background",
        aliases: &[],
        title: "Slider background",
        nav_label: "Slider background",
        description: "Theme-aware slider family: states, variants, pointer and keyboard interactions.",
        section: StorySection::Atoms,
        reference_window_size: (720., 800.),
        gallery_surface_size: (440., 640.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Compact", 320., 640.),
            ViewportPreset::new("Default", 440., 640.),
        ],
        keyboard_hints: &[
            KeyboardHint::new("Arrows / Shift+Arrows", "Adjust value / coarse step"),
            KeyboardHint::new("Home / End", "Minimum / maximum"),
        ],
        render_story: |story, cx| story.render_slider_backgrounds_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_slider_backgrounds_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| {
            story.sliders_screen.sliders[5]
                .focus_handle(cx)
                .focus(window, cx)
        },
        last_action: |story| story.sliders_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::SliderHandles,
        id: "slider-handle",
        aliases: &[],
        title: "Slider handle",
        nav_label: "Slider handle",
        description: "Theme-aware slider family: states, variants, pointer and keyboard interactions.",
        section: StorySection::Atoms,
        reference_window_size: (720., 800.),
        gallery_surface_size: (440., 640.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Compact", 320., 640.),
            ViewportPreset::new("Default", 440., 640.),
        ],
        keyboard_hints: &[
            KeyboardHint::new("Arrows / Shift+Arrows", "Adjust value / coarse step"),
            KeyboardHint::new("Home / End", "Minimum / maximum"),
        ],
        render_story: |story, cx| story.render_slider_handles_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_slider_handles_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| {
            story.sliders_screen.sliders[0]
                .focus_handle(cx)
                .focus(window, cx)
        },
        last_action: |story| story.sliders_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::SliderStops,
        id: "slider-gradient-stop",
        aliases: &[],
        title: "Slider gradient stop",
        nav_label: "Slider gradient stop",
        description: "Theme-aware slider family: states, variants, pointer and keyboard interactions.",
        section: StorySection::Atoms,
        reference_window_size: (720., 800.),
        gallery_surface_size: (440., 640.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Compact", 320., 640.),
            ViewportPreset::new("Default", 440., 640.),
        ],
        keyboard_hints: &[
            KeyboardHint::new("Arrows / Shift+Arrows", "Adjust value / coarse step"),
            KeyboardHint::new("Home / End", "Minimum / maximum"),
        ],
        render_story: |story, cx| story.render_slider_stops_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_slider_stops_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| {
            story.sliders_screen.sliders[0]
                .focus_handle(cx)
                .focus(window, cx)
        },
        last_action: |story| story.sliders_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::ColorPicker,
        id: "color-picker",
        aliases: &["color"],
        title: "Color picker",
        nav_label: "Color",
        description: "The inspector’s shared RGBA editor with spectrum, hue, opacity, and Hex/RGB/HSL/HSB inputs.",
        section: StorySection::Organisms,
        reference_window_size: (720., 720.),
        gallery_surface_size: (600., 620.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Compact", 320., 560.),
            ViewportPreset::new("Default", 600., 620.),
        ],
        keyboard_hints: &[
            KeyboardHint::new("Arrow keys", "Adjust focused spectrum, hue or alpha"),
            KeyboardHint::new("Enter / Escape", "Commit / cancel color text edits"),
        ],
        render_story: |story, cx| story.render_color_picker_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_color_picker_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| {
            story
                .color_picker_screen
                .picker
                .focus_handle(cx)
                .focus(window, cx)
        },
        last_action: |story| story.color_picker_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Welcome,
        id: "welcome",
        aliases: &["overview", "getting-started"],
        title: "Welcome",
        nav_label: "Welcome",
        description: "The atomic-design map of the library: what each tier owns, with every \
                      story listed on its tier card.",
        section: StorySection::GettingStarted,
        reference_window_size: (1240., 820.),
        gallery_surface_size: (1200., 860.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Narrow", 480., 360.),
            ViewportPreset::new("Default", 1200., 860.),
        ],
        keyboard_hints: &[],
        render_story: |story, cx| story.render_welcome_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_welcome_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| {
            story.welcome_screen.focus_handle.focus(window, cx);
        },
        last_action: |story| story.welcome_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Buttons,
        id: "buttons",
        aliases: &["icon-button", "activation"],
        title: "Buttons & activation",
        nav_label: "Buttons",
        description: "The complete UI3 button taxonomy: nine text-button variants, three sizes, \
                      leading-icon alignment, icon/toggle/dialog/split families, all interaction \
                      states, and the shared pointer/keyboard activation path.",
        section: StorySection::Atoms,
        reference_window_size: (1240., 820.),
        gallery_surface_size: (900., 640.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Compact", 700., 560.),
            ViewportPreset::new("Default", 900., 640.),
        ],
        keyboard_hints: &[KeyboardHint::new(
            "Tab, then Enter / Space",
            "Activate the focused specimen through the shared command path",
        )],
        render_story: |story, cx| story.render_buttons_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_buttons_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| {
            story.buttons_screen.focus_handle.focus(window, cx);
        },
        last_action: |story| story.buttons_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Checkbox,
        id: "checkbox",
        aliases: &["checkboxes", "mixed-checkbox"],
        title: "Checkbox",
        nav_label: "Checkbox",
        description: "The UI3 checkbox taxonomy: checked, unchecked, and mixed values with \
                      focused, disabled, muted, and ghost presentation axes.",
        section: StorySection::Atoms,
        reference_window_size: (1000., 820.),
        gallery_surface_size: (760., 720.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Compact", 520., 640.),
            ViewportPreset::new("Default", 760., 720.),
        ],
        keyboard_hints: &[KeyboardHint::new(
            "Tab, then Enter / Space",
            "Cycle the live controlled checkbox",
        )],
        render_story: |story, cx| story.render_checkbox_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_checkbox_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| story.checkbox_story.focus_handle.focus(window, cx),
        last_action: |story| story.checkbox_story.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Dropdown,
        id: "dropdown",
        aliases: &["select", "dropdown-trigger"],
        title: "Dropdown",
        nav_label: "Dropdown",
        description: "The UI3 dropdown trigger taxonomy: default and large sizes, \
                      default/focused/active/disabled states, optional stroke, and optional \
                      leading icon.",
        section: StorySection::Atoms,
        reference_window_size: (1000., 820.),
        gallery_surface_size: (760., 720.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Compact", 520., 640.),
            ViewportPreset::new("Default", 760., 720.),
        ],
        keyboard_hints: &[KeyboardHint::new(
            "Tab, then Enter / Space",
            "Activate the live dropdown trigger",
        )],
        render_story: |story, cx| story.render_dropdown_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_dropdown_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| story.dropdown_story.focus_handle.focus(window, cx),
        last_action: |story| story.dropdown_story.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Inputs,
        id: "inputs",
        aliases: &[
            "input",
            "text-input",
            "numeric-input",
            "color-input",
            "combo-input",
        ],
        title: "Inputs",
        nav_label: "Inputs",
        description: "The UI3 input taxonomy: controlled text, numeric, grouped numeric, color, \
                      and combo inputs plus their variable cells, chips, dropdown segments, and \
                      24/48 px color chits.",
        section: StorySection::Atoms,
        reference_window_size: (1240., 900.),
        gallery_surface_size: (1000., 820.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Compact", 560., 700.),
            ViewportPreset::new("Default", 1000., 820.),
        ],
        keyboard_hints: &[
            KeyboardHint::new("Tab / Shift-Tab", "Move among editable input specimens"),
            KeyboardHint::new("Enter", "Commit the focused single-line value"),
        ],
        render_story: |story, cx| story.render_inputs_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_inputs_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| story.inputs_story.focus_handle(cx).focus(window, cx),
        last_action: |story| story.inputs_story.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Labels,
        id: "labels",
        aliases: &["truncation", "truncating-label"],
        title: "Labels & truncation",
        nav_label: "Labels",
        description: "truncating_label live at adjustable widths: scrub the story viewport \
                      and labels ellipsize instead of widening their rows.",
        section: StorySection::Atoms,
        reference_window_size: (1240., 820.),
        gallery_surface_size: (900., 620.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Narrow", 420., 620.),
            ViewportPreset::new("Default", 900., 620.),
        ],
        keyboard_hints: &[],
        render_story: |story, cx| story.render_labels_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_labels_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| {
            story
                .labels_screen
                .truncation
                .focus_handle(cx)
                .focus(window, cx);
        },
        last_action: |story| story.labels_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Icons,
        id: "icons",
        aliases: &[
            "icon",
            "icon-gallery",
            "icon-catalog",
            "icon-assets",
            "foundations",
            "atoms",
            "control-icons",
        ],
        title: "Lucide icons",
        nav_label: "Lucide icons",
        description: "The Lucide icons bundled by gpui-component, with canonical names and \
                      asset paths. Fanta-owned controls use the same pinned Lucide catalog.",
        section: StorySection::Atoms,
        reference_window_size: (1240., 820.),
        gallery_surface_size: (1200., 900.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Compact", 900., 700.),
            ViewportPreset::new("Default", 1200., 900.),
        ],
        keyboard_hints: &[],
        render_story: |story, _| story.icons_screen.gallery.clone().into_any_element(),
        render_gallery: None,
        render_reference: |story, _| story.icons_screen.gallery.clone().into_any_element(),
        render_knobs: None,
        focus: |story, window, cx| {
            story
                .icons_screen
                .gallery
                .focus_handle(cx)
                .focus(window, cx);
        },
        last_action: |_| "Browse or filter every bundled icon".into(),
    },
    StoryDescriptor {
        kind: StoryKind::Tokens,
        id: "tokens",
        aliases: &["design-tokens", "scale", "geometry"],
        title: "Design tokens",
        nav_label: "Design tokens",
        description: "The shared geometry scale drawn at size: every constant gets one line \
                      naming the path a host types, the number, and the geometry itself, so \
                      two tokens can be compared by eye instead of by arithmetic.",
        section: StorySection::Atoms,
        reference_window_size: (1240., 900.),
        gallery_surface_size: (900., 880.),
        gallery_fluid_width: true,
        viewport_presets: &[
            // Nothing on the sheet rescales, so the narrowest preset is the
            // width that still shows the widest specimen whole.
            ViewportPreset::new("Sheet", tokens::TOKENS_SHEET_MIN_WIDTH, 760.),
            ViewportPreset::new("Default", 900., 880.),
            ViewportPreset::new("Wide", 1120., 900.),
        ],
        keyboard_hints: &[],
        render_story: |story, cx| story.render_tokens_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_tokens_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| {
            story.tokens_screen.focus_handle.focus(window, cx);
        },
        last_action: |story| story.tokens_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Menus,
        id: "menus",
        // "molecules" survives as a launch name from the retired
        // Foundations catalog's alias set.
        aliases: &["molecules", "context-menu"],
        title: "Menus",
        nav_label: "Menus",
        description: "All 64 UI3 menu specimens: 57 row variants across the simple, complex, \
                      selection, toggle, toolbar, and structural families, plus 7 multi-select \
                      and composed variants and a live context-menu clamping demo.",
        section: StorySection::Molecules,
        reference_window_size: (1240., 820.),
        gallery_surface_size: (900., 620.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Narrow", 560., 560.),
            ViewportPreset::new("Default", 900., 620.),
        ],
        keyboard_hints: &[
            KeyboardHint::new(
                "Enter / Space on an opener",
                "Open the demo menu without a pointer",
            ),
            KeyboardHint::new("Esc", "Close the demo context menu"),
        ],
        render_story: |story, cx| story.render_menus_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_menus_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| {
            story.menus_screen.demo_focus.focus(window, cx);
        },
        last_action: |story| story.menus_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::SegmentedControl,
        id: "segmented-control",
        aliases: &["segmented", "segments"],
        title: "Segmented control",
        nav_label: "Segmented control",
        description: "All 46 linked UI3 specimens: 20 icon/label and count/state variants, 10 segment primitive states, 16 common inspector presets, and two controlled interactive examples.",
        section: StorySection::Molecules,
        reference_window_size: (1240., 900.),
        gallery_surface_size: (900., 880.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Compact", 560., 680.),
            ViewportPreset::new("Default", 900., 880.),
        ],
        keyboard_hints: &[
            KeyboardHint::new(
                "Tab, then Enter / Space",
                "Select the focused segment through the shared activation path",
            ),
            KeyboardHint::new(
                "← → ↑ ↓ / Home / End",
                "Move the controlled selection within a segmented control",
            ),
        ],
        render_story: |story, cx| story.render_segmented_control_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_segmented_control_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| {
            story.segmented_control_story.focus_handle.focus(window, cx);
        },
        last_action: |story| story.segmented_control_story.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::RadioButton,
        id: "radio-button",
        aliases: &["radio", "radio-buttons"],
        title: "Radio button",
        nav_label: "Radio button",
        description: "The 12 valid UI3 radio variants plus a controlled group with pointer and keyboard selection.",
        section: StorySection::Atoms,
        reference_window_size: (1240., 900.),
        gallery_surface_size: (900., 760.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Compact", 520., 620.),
            ViewportPreset::new("Default", 900., 760.),
        ],
        keyboard_hints: &[KeyboardHint::new(
            "Tab, then Enter / Space",
            "Select the focused radio option",
        )],
        render_story: |story, cx| story.render_radio_button_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_radio_button_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| story.radio_button_story.focus_handle.focus(window, cx),
        last_action: |story| story.radio_button_story.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Tabs,
        id: "tabs",
        aliases: &["tab"],
        title: "Tabs",
        nav_label: "Tabs",
        description: "All 4 Tabs count variants and all 7 valid _Tab primitive variants, backed by controlled selection intents.",
        section: StorySection::Molecules,
        reference_window_size: (1240., 900.),
        gallery_surface_size: (900., 780.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Compact", 520., 640.),
            ViewportPreset::new("Default", 900., 780.),
        ],
        keyboard_hints: &[
            KeyboardHint::new("Enter / Space", "Select the focused tab"),
            KeyboardHint::new("← → ↑ ↓ / Home / End", "Move the selected tab"),
        ],
        render_story: |story, cx| story.render_tabs_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_tabs_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| story.tabs_story.focus_handle.focus(window, cx),
        last_action: |story| story.tabs_story.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Tooltips,
        id: "tooltips",
        aliases: &["tooltip", "tooltip-link"],
        title: "Tooltips",
        nav_label: "Tooltips",
        description: "All 8 tooltip directions and 8 Tooltip link actions, including live hover, pointer, and keyboard examples.",
        section: StorySection::Molecules,
        reference_window_size: (1240., 900.),
        gallery_surface_size: (900., 840.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Compact", 520., 680.),
            ViewportPreset::new("Default", 900., 840.),
        ],
        keyboard_hints: &[KeyboardHint::new(
            "Enter / Space",
            "Show a focused tooltip or activate a tooltip link",
        )],
        render_story: |story, cx| story.render_tooltips_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_tooltips_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| story.tooltips_story.focus_handle.focus(window, cx),
        last_action: |story| story.tooltips_story.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::ListRows,
        id: "list-rows",
        aliases: &["rows"],
        title: "List rows",
        nav_label: "List rows",
        description: "list_row specimens with hover, selection, the reserved focus ring, a \
                      lock icon_button satellite, and the §9 roving-navigation hint.",
        section: StorySection::Molecules,
        reference_window_size: (1240., 820.),
        gallery_surface_size: (700., 620.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Narrow", 420., 560.),
            ViewportPreset::new("Default", 700., 620.),
        ],
        keyboard_hints: &[KeyboardHint::new(
            "Tab, then Enter / Space",
            "Select the focused row or toggle its lock satellite",
        )],
        render_story: |story, cx| story.render_list_rows_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_list_rows_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| {
            story.list_rows_screen.focus_handle.focus(window, cx);
        },
        last_action: |story| story.list_rows_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Popups,
        id: "popups",
        aliases: &["edge-fades", "anchored-popup"],
        title: "Popups & edge fades",
        nav_label: "Popups",
        description: "An anchored popup snapped inside the window per §12, and a scrollable \
                      chip row growing live edge fades from its tracked scroll state.",
        section: StorySection::Molecules,
        reference_window_size: (1240., 820.),
        gallery_surface_size: (900., 620.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Narrow", 460., 620.),
            ViewportPreset::new("Default", 900., 620.),
        ],
        keyboard_hints: &[KeyboardHint::new("Esc", "Close the anchored popup")],
        render_story: |story, cx| story.render_popups_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_popups_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| {
            story.popups_screen.focus_handle.focus(window, cx);
        },
        last_action: |story| story.popups_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Fields,
        id: "fields",
        aliases: &["inspector-fields", "property-fields"],
        title: "Inspector fields",
        nav_label: "Fields",
        description: "Every controlled field kind in one property grid, the value-state by \
                      access matrix they all obey, and a live edit log: drag the X field or \
                      the opacity track and watch Begin, Preview and Commit arrive.",
        section: StorySection::Molecules,
        reference_window_size: (1240., 900.),
        gallery_surface_size: (1000., 880.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Narrow", 620., 760.),
            ViewportPreset::new("Default", 1000., 880.),
            ViewportPreset::new("Wide", 1200., 900.),
        ],
        keyboard_hints: &[
            KeyboardHint::new(
                "↑ / ↓ on the live X field",
                "Nudge the host value by one unit",
            ),
            KeyboardHint::new(
                "Tab, then Enter / Space",
                "Reach the selection checkboxes and the log's Clear button",
            ),
        ],
        render_story: |story, cx| story.render_fields_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_fields_reference(cx),
        render_knobs: Some(|story, cx| story.render_fields_knobs(cx)),
        focus: |story, window, cx| {
            story.fields_screen.focus_handle.focus(window, cx);
        },
        last_action: |story| story.fields_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Structure,
        id: "structure",
        aliases: &["sections", "sections-and-grids", "inspector-structure"],
        title: "Sections & grids",
        nav_label: "Structure",
        description: "The inspector's structural recipes as surfaces: headers that disclose, \
                      a grid that hands every row its geometry, and the same frame mounted at \
                      320, 400 and 472 px so the responsive range is on screen at once.",
        section: StorySection::Molecules,
        reference_window_size: (1240., 860.),
        gallery_surface_size: (1040., 800.),
        gallery_fluid_width: true,
        viewport_presets: &[
            // The three widths the comparison card mounts: one under the
            // compact breakpoint, one between, one over the wide one.
            ViewportPreset::new("320 px", structure::STRUCTURE_WIDTHS[0].0, 760.),
            ViewportPreset::new("400 px", structure::STRUCTURE_WIDTHS[1].0, 760.),
            ViewportPreset::new("472 px", structure::STRUCTURE_WIDTHS[2].0, 760.),
            ViewportPreset::new("Default", 1040., 800.),
        ],
        keyboard_hints: &[KeyboardHint::new(
            "Tab, then Enter / Space",
            "Disclose the focused section header",
        )],
        render_story: |story, cx| story.render_structure_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_structure_reference(cx),
        render_knobs: Some(|story, cx| story.render_structure_knobs(cx)),
        focus: |story, window, cx| {
            story.structure_screen.focus_handle.focus(window, cx);
        },
        last_action: |story| story.structure_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Overlays,
        id: "overlays",
        aliases: &["inspector-overlays", "dismissal"],
        title: "Inspector overlays",
        nav_label: "Overlays",
        description: "An anchored popover and an anchored context menu sharing one placement: \
                      close either with Escape, with an outside click, or by committing, and \
                      the log names the cause that fired and where focus landed.",
        section: StorySection::Molecules,
        reference_window_size: (1240., 900.),
        gallery_surface_size: (960., 860.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Narrow", 600., 760.),
            ViewportPreset::new("Default", 960., 860.),
            ViewportPreset::new("Wide", 1180., 900.),
        ],
        keyboard_hints: &[
            KeyboardHint::new(
                "Enter / Space on a trigger",
                "Open the popover or the context menu without a pointer",
            ),
            KeyboardHint::new(
                "Esc",
                "Dismiss the open surface and hand focus back to its trigger",
            ),
        ],
        render_story: |story, cx| story.render_overlays_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_overlays_reference(cx),
        render_knobs: Some(|story, cx| story.render_overlays_knobs(cx)),
        focus: |story, window, cx| {
            story.overlays_screen.focus_handle.focus(window, cx);
        },
        last_action: |story| story.overlays_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Design,
        id: "design",
        aliases: &[],
        title: "Design inspector",
        nav_label: "Design",
        description: "A host-controlled inspector covering layout, appearance, typography, variables, and export.",
        section: StorySection::Organisms,
        reference_window_size: (1240., 820.),
        gallery_surface_size: (1200., 760.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new(
                "Minimum",
                design::DESIGN_STORY_MIN_WIDTH,
                design::DESIGN_STORY_MIN_HEIGHT,
            ),
            ViewportPreset::new("Default", 1200., 760.),
            ViewportPreset::new("Desktop", 1440., 900.),
        ],
        keyboard_hints: &[
            KeyboardHint::new(
                "← → ↑ ↓ / + − on the divider",
                "Resize the inspector (Shift: 32 px)",
            ),
            KeyboardHint::new("↑ / ↓ on numeric fields", "Nudge by the Small / Big step"),
            KeyboardHint::new("Esc", "Dismiss open pickers and menus"),
        ],
        render_story: |story, _| story.design_screen.panel.clone().into_any_element(),
        render_gallery: Some(|story, cx| story.render_design_story_harness(cx)),
        render_reference: |story, cx| story.render_design_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| {
            story.design_screen.panel.focus_handle(cx).focus(window, cx);
        },
        last_action: |story| story.design_screen.harness.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Layers,
        id: "layers",
        aliases: &[],
        title: "Layers panel",
        nav_label: "Layers",
        description: "A controlled layer tree with selection, expansion, reordering, and contextual actions.",
        section: StorySection::Organisms,
        reference_window_size: (1240., 820.),
        gallery_surface_size: (337., 716.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Narrow", LAYERS_PANEL_MIN_WIDTH, LAYERS_PANEL_MIN_HEIGHT),
            ViewportPreset::new("Default", 337., 716.),
            ViewportPreset::new("Wide", 472., 716.),
        ],
        keyboard_hints: &[
            KeyboardHint::new("↑ / ↓", "Focus the previous / next layer"),
            KeyboardHint::new("← / →", "Collapse / expand the focused layer"),
            KeyboardHint::new("Esc", "Close the layer context menu"),
        ],
        render_story: |story, _| story.layers_screen.panel.clone().into_any_element(),
        render_gallery: None,
        render_reference: |story, cx| story.render_layers_reference(cx),
        render_knobs: Some(|story, cx| story.render_layers_knobs(cx)),
        focus: |story, window, cx| {
            story.layers_screen.panel.focus_handle(cx).focus(window, cx);
        },
        last_action: |story| story.layers_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Pages,
        id: "pages",
        aliases: &[],
        title: "Pages panel",
        nav_label: "Pages",
        description: "Interactive page navigation, search, replace, filters, and typed host intents.",
        section: StorySection::Organisms,
        reference_window_size: (1240., 820.),
        gallery_surface_size: (337., 716.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Narrow", PAGES_PANEL_MIN_WIDTH, PAGES_PANEL_MIN_HEIGHT),
            ViewportPreset::new("Default", 337., 716.),
            ViewportPreset::new("Wide", 472., 716.),
        ],
        keyboard_hints: &[
            KeyboardHint::new("⌘F", "Find in pages"),
            KeyboardHint::new("↑ / ↓", "Focus the previous / next page"),
            KeyboardHint::new("Home / End", "Jump to the first / last page"),
            KeyboardHint::new("Esc", "Close search, then open menus"),
        ],
        render_story: |story, _| story.pages_screen.panel.clone().into_any_element(),
        render_gallery: None,
        render_reference: |story, cx| story.render_pages_reference(cx),
        render_knobs: Some(|story, cx| story.render_pages_knobs(cx)),
        focus: |story, window, cx| {
            story.pages_screen.panel.focus_handle(cx).focus(window, cx);
        },
        last_action: |story| story.pages_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Prototype,
        id: "prototype",
        aliases: &["prototype-panel"],
        title: "Prototype panel",
        nav_label: "Prototype",
        description: "Prototype settings and connection guidance presented as a controlled inspector.",
        section: StorySection::Organisms,
        reference_window_size: (473., 716.),
        gallery_surface_size: (473., 716.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new(
                "Narrow",
                PROTOTYPE_PANEL_MIN_WIDTH,
                PROTOTYPE_PANEL_MIN_HEIGHT,
            ),
            ViewportPreset::new("Default", 473., 716.),
            ViewportPreset::new("Wide", 560., 716.),
        ],
        keyboard_hints: &[],
        render_story: |story, _| story.prototype_screen.panel.clone().into_any_element(),
        render_gallery: None,
        render_reference: |story, cx| story.render_prototype_reference(cx),
        render_knobs: Some(|story, cx| story.render_prototype_knobs(cx)),
        focus: |story, window, cx| {
            story
                .prototype_screen
                .panel
                .focus_handle(cx)
                .focus(window, cx);
        },
        last_action: |story| story.prototype_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Timeline,
        id: "timeline",
        aliases: &["motion-timeline"],
        title: "Timeline",
        nav_label: "Timeline",
        description: "Motion controls, a time ruler, seeking, zoom, keyframes, and an empty-state agent prompt.",
        section: StorySection::Organisms,
        reference_window_size: (1728., 333.),
        gallery_surface_size: (1200., 333.),
        gallery_fluid_width: true,
        viewport_presets: &[
            // The floor: floored rails, the minimum ruler, and the icon-only
            // zoom cluster, with room for the empty-state card below.
            ViewportPreset::new("Minimum", TIMELINE_MIN_WIDTH, 220.),
            ViewportPreset::new("Compact", 900., 260.),
            ViewportPreset::new("Default", 1200., 333.),
            ViewportPreset::new("Full", 1728., 333.),
        ],
        keyboard_hints: &[],
        render_story: |story, _| story.timeline_screen.timeline.clone().into_any_element(),
        render_gallery: None,
        render_reference: |story, cx| story.render_timeline_reference(cx),
        render_knobs: Some(|story, cx| story.render_timeline_knobs(cx)),
        focus: |story, window, cx| {
            story
                .timeline_screen
                .timeline
                .focus_handle(cx)
                .focus(window, cx);
        },
        last_action: |story| story.timeline_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Toolbar,
        id: "toolbar",
        aliases: &["editor-toolbar"],
        title: "Editor toolbar",
        nav_label: "Toolbar",
        description: "A mode-aware Figma-style toolbar with tool groups, actions, zoom, and agent overlays.",
        section: StorySection::Organisms,
        reference_window_size: (1240., 820.),
        gallery_surface_size: (1180., 720.),
        gallery_fluid_width: true,
        viewport_presets: &[
            // The floor: the dock sheds its zoom steppers and rides the
            // percent-only tier, so the whole dock still fits the surface.
            ViewportPreset::new("Minimum", 360., 520.),
            ViewportPreset::new("Narrow", 900., 640.),
            ViewportPreset::new("Default", 1180., 720.),
            ViewportPreset::new("Laptop", 1240., 820.),
        ],
        keyboard_hints: &[
            KeyboardHint::new("⌘K or ⌘/", "Open toolbar Actions"),
            KeyboardHint::new("↑ / ↓", "Move through Actions commands"),
            KeyboardHint::new("← / →", "Adjust the focused toolbar control"),
            KeyboardHint::new("V H F R O P T …", "Select tools directly"),
            KeyboardHint::new("Esc", "Close the Actions or Agent overlay"),
        ],
        render_story: |story, cx| story.render_toolbar_story(cx),
        render_gallery: None,
        render_reference: |story, cx| story.render_toolbar_reference(cx),
        render_knobs: Some(|story, cx| story.render_toolbar_knobs(cx)),
        focus: |story, window, cx| {
            story
                .toolbar_screen
                .toolbar
                .focus_handle(cx)
                .focus(window, cx);
        },
        last_action: |story| story.toolbar_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::Variables,
        id: "variables",
        aliases: &["variable", "variables-screen", "variables-page"],
        title: "Variables screen",
        nav_label: "Variables",
        description: "A full variables workspace with collections, groups, modes, values, and creation intents.",
        section: StorySection::Screens,
        reference_window_size: (1677., 1048.),
        gallery_surface_size: (1200., 760.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new(
                "Minimum",
                VARIABLES_SCREEN_MIN_WIDTH,
                VARIABLES_SCREEN_MIN_HEIGHT,
            ),
            ViewportPreset::new("Compact", 720., 520.),
            ViewportPreset::new("Default", 1200., 760.),
        ],
        keyboard_hints: &[],
        render_story: |story, _| story.variables_screen.screen.clone().into_any_element(),
        render_gallery: None,
        render_reference: |story, cx| story.render_variables_reference(cx),
        render_knobs: Some(|story, cx| story.render_variables_knobs(cx)),
        focus: |story, window, cx| {
            story
                .variables_screen
                .screen
                .focus_handle(cx)
                .focus(window, cx);
        },
        last_action: |story| story.variables_screen.last_action.clone(),
    },
    StoryDescriptor {
        kind: StoryKind::FileInspector,
        id: "file-inspector",
        aliases: &["file-sidebar"],
        title: "File inspector",
        nav_label: "File inspector",
        description: "One unified sidebar containing the Pages and Layers panels.",
        section: StorySection::Layouts,
        reference_window_size: (1240., 820.),
        gallery_surface_size: (337., 716.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new(
                "Minimum",
                FILE_INSPECTOR_MIN_WIDTH,
                FILE_INSPECTOR_MIN_HEIGHT,
            ),
            ViewportPreset::new("Default", 337., 716.),
            ViewportPreset::new("Wide", 472., 820.),
        ],
        keyboard_hints: &[
            KeyboardHint::new("⌘F", "Find in pages"),
            KeyboardHint::new("↑ / ↓", "Move through the focused Pages or Layers list"),
            KeyboardHint::new("Esc", "Close the active child-panel overlay"),
        ],
        render_story: |story, _| {
            story
                .file_inspector_screen
                .sidebar
                .clone()
                .into_any_element()
        },
        render_gallery: None,
        render_reference: |story, cx| story.render_file_inspector_reference(cx),
        render_knobs: None,
        focus: |story, window, cx| {
            story
                .file_inspector_screen
                .sidebar
                .focus_handle(cx)
                .focus(window, cx);
        },
        last_action: |story| story.file_inspector_last_action(),
    },
    StoryDescriptor {
        kind: StoryKind::PseudoEditor,
        id: "pseudo-editor",
        aliases: &["pseudo", "editor"],
        title: "Pseudo editor",
        nav_label: "Pseudo editor",
        description: "All Fanta GPUI surfaces assembled into one simulated editor workspace.",
        section: StorySection::Layouts,
        reference_window_size: (1440., 900.),
        gallery_surface_size: (1440., 900.),
        gallery_fluid_width: true,
        viewport_presets: &[
            ViewportPreset::new("Minimum", PSEUDO_EDITOR_MIN_WIDTH, PSEUDO_EDITOR_MIN_HEIGHT),
            ViewportPreset::new("Laptop", PSEUDO_EDITOR_PREFERRED_WIDTH, 820.),
            ViewportPreset::new("Desktop", 1440., 900.),
        ],
        keyboard_hints: &[KeyboardHint::new(
            "Esc",
            "Close the active overlay or Variables manager",
        )],
        render_story: |story, _| story.pseudo_screen.editor.clone().into_any_element(),
        render_gallery: None,
        render_reference: |story, cx| story.render_pseudo_editor_reference(cx),
        render_knobs: Some(|story, cx| story.render_pseudo_editor_knobs(cx)),
        focus: |story, window, cx| {
            story
                .pseudo_screen
                .editor
                .focus_handle(cx)
                .focus(window, cx);
        },
        last_action: |story| story.pseudo_screen.last_action.clone(),
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_ids_aliases_and_kinds_are_unique_and_resolvable() {
        let mut names = std::collections::HashSet::new();
        let mut kinds = std::collections::HashSet::new();
        for descriptor in registry() {
            assert!(
                kinds.insert(descriptor.kind),
                "{:?} must register exactly one descriptor",
                descriptor.kind
            );
            for name in std::iter::once(descriptor.id).chain(descriptor.aliases.iter().copied()) {
                assert!(
                    names.insert(name),
                    "story name {name:?} must resolve to exactly one story"
                );
                assert_eq!(
                    story_from_name(name),
                    Some(descriptor.kind),
                    "{name:?} must resolve to its registering story"
                );
                assert_eq!(
                    name,
                    name.to_ascii_lowercase(),
                    "story names are matched case-insensitively and must register lowercase"
                );
            }
        }
        assert_eq!(registry().len(), 34);
        assert_eq!(
            story_from_name("color-picker"),
            Some(StoryKind::ColorPicker)
        );
        assert_eq!(story_from_name("TOOLBAR"), Some(StoryKind::Toolbar));
        // Launch names of the retired Foundations catalog keep resolving to
        // the specimen stories that absorbed it.
        assert_eq!(story_from_name("foundations"), Some(StoryKind::Icons));
        assert_eq!(story_from_name("atoms"), Some(StoryKind::Icons));
        assert_eq!(story_from_name("molecules"), Some(StoryKind::Menus));
        assert_eq!(story_from_name("not-a-story"), None);
    }

    #[test]
    fn every_section_owns_at_least_one_story() {
        for section in StorySection::ALL {
            assert!(
                registry()
                    .iter()
                    .any(|descriptor| descriptor.section == section),
                "{} must list at least one story",
                section.label()
            );
        }
    }

    #[test]
    fn sections_mirror_the_atomic_design_tiers() {
        assert_eq!(
            StorySection::ALL.map(StorySection::label),
            [
                "Getting started",
                "Atoms",
                "Molecules",
                "Organisms",
                "Layouts",
                "Screens",
            ],
            "the sidebar sections name the §16 tiers in fixed order"
        );
        // Feature stories sit in their matching reusable tiers.
        for (kind, section) in [
            (StoryKind::Design, StorySection::Organisms),
            (StoryKind::Layers, StorySection::Organisms),
            (StoryKind::Pages, StorySection::Organisms),
            (StoryKind::Prototype, StorySection::Organisms),
            (StoryKind::Timeline, StorySection::Organisms),
            (StoryKind::Toolbar, StorySection::Organisms),
            (StoryKind::FileInspector, StorySection::Layouts),
            (StoryKind::Variables, StorySection::Screens),
            (StoryKind::PseudoEditor, StorySection::Layouts),
            (StoryKind::Icons, StorySection::Atoms),
            (StoryKind::Tokens, StorySection::Atoms),
            (StoryKind::Inputs, StorySection::Atoms),
            (StoryKind::Fields, StorySection::Molecules),
            (StoryKind::Structure, StorySection::Molecules),
            (StoryKind::Overlays, StorySection::Molecules),
        ] {
            assert_eq!(
                kind.descriptor().section,
                section,
                "{kind:?} must sit in the {} section",
                section.label()
            );
        }
    }

    #[test]
    fn every_story_registers_clamped_viewport_presets() {
        for descriptor in registry() {
            assert!(
                !descriptor.viewport_presets.is_empty(),
                "{} must offer at least one viewport preset",
                descriptor.title
            );
            let mut labels = std::collections::HashSet::new();
            for preset in descriptor.viewport_presets {
                assert!(
                    labels.insert(preset.label),
                    "{} preset labels must stay unique",
                    descriptor.title
                );
                assert_eq!(
                    preset.width,
                    viewport::clamp_viewport_dimension(viewport::ViewportAxis::Width, preset.width),
                    "{} preset {} width must survive the harness clamp",
                    descriptor.title,
                    preset.label
                );
                assert_eq!(
                    preset.height,
                    viewport::clamp_viewport_dimension(
                        viewport::ViewportAxis::Height,
                        preset.height
                    ),
                    "{} preset {} height must survive the harness clamp",
                    descriptor.title,
                    preset.label
                );
            }
            let (min_width, min_height) = descriptor.min_story_size();
            assert!(min_width <= descriptor.gallery_surface_size.0);
            assert!(min_height <= descriptor.gallery_surface_size.1);
        }
    }

    #[test]
    fn keyboard_hints_and_the_shared_ladder_are_well_formed() {
        assert!(!SHARED_KEYBOARD_HINTS.is_empty());
        for hint in SHARED_KEYBOARD_HINTS {
            assert!(!hint.keys.is_empty());
            assert!(!hint.action.is_empty());
        }
        for descriptor in registry() {
            for hint in descriptor.keyboard_hints {
                assert!(
                    !hint.keys.is_empty() && !hint.action.is_empty(),
                    "{} hints must name both keys and effect",
                    descriptor.title
                );
            }
        }
        // Stories with overlay surfaces document their Escape ladder.
        for kind in [
            StoryKind::Toolbar,
            StoryKind::Pages,
            StoryKind::PseudoEditor,
            StoryKind::Menus,
            StoryKind::Popups,
            StoryKind::Overlays,
        ] {
            assert!(
                kind.descriptor()
                    .keyboard_hints
                    .iter()
                    .any(|hint| hint.keys.contains("Esc")),
                "{:?} must document its Escape ladder",
                kind
            );
        }
    }
}
