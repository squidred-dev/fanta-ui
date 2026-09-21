//! UI3 menu taxonomy and live context-menu story.

use fanta_gpui::atoms::TypographyExt as _;
use gpui::{Bounds, FocusHandle, Pixels, Point, point, size};

use crate::*;

use super::specimen::{specimen_card, specimen_row, specimen_rows, specimen_story_root};

/// Menu geometry shared by the render path and the clamping computation.
const MENU_WIDTH: f32 = 224.;
const MENU_ITEM_HEIGHT: f32 = 28.;
const MENU_MAX_HEIGHT: f32 = 240.;
/// Vertical padding the molecule's `py_2` adds around the item stack.
const MENU_VERTICAL_PADDING: f32 = 16.;
const TAXONOMY_MENU_WIDTH: f32 = 224.;

const FIGMA_SIMPLE_VARIANTS: usize = 5;
const FIGMA_COMPLEX_VARIANTS: usize = 30;
const FIGMA_CHECKMARK_VARIANTS: usize = 7;
const FIGMA_TOGGLE_VARIANTS: usize = 4;
const FIGMA_TOOLBAR_VARIANTS: usize = 3;
const FIGMA_HEADING_VARIANTS: usize = 2;
const FIGMA_DIVIDER_VARIANTS: usize = 1;
const FIGMA_EXPAND_VARIANTS: usize = 4;
const FIGMA_FOOTER_VARIANTS: usize = 1;
const FIGMA_MULTI_SELECT_VARIANTS: usize = 4;
const FIGMA_COMPOSED_EXAMPLES: usize = 3;

const FIGMA_ROW_VARIANTS: usize = FIGMA_SIMPLE_VARIANTS
    + FIGMA_COMPLEX_VARIANTS
    + FIGMA_CHECKMARK_VARIANTS
    + FIGMA_TOGGLE_VARIANTS
    + FIGMA_TOOLBAR_VARIANTS
    + FIGMA_HEADING_VARIANTS
    + FIGMA_DIVIDER_VARIANTS
    + FIGMA_EXPAND_VARIANTS
    + FIGMA_FOOTER_VARIANTS;
const FIGMA_COMPOSED_VARIANTS: usize = FIGMA_MULTI_SELECT_VARIANTS + FIGMA_COMPOSED_EXAMPLES;
const FIGMA_MENU_SPECIMENS: usize = FIGMA_ROW_VARIANTS + FIGMA_COMPOSED_VARIANTS;

/// The demo menu's items.
pub(crate) const MENU_DEMO_ITEMS: [&str; 4] =
    ["Rename specimen", "Duplicate", "Copy link", "Delete"];

/// The demo menu's rendered size, as fed to `clamp_menu_origin`.
pub(crate) fn menu_demo_size() -> gpui::Size<gpui::Pixels> {
    size(
        px(MENU_WIDTH),
        px(
            (MENU_DEMO_ITEMS.len() as f32 * MENU_ITEM_HEIGHT + MENU_VERTICAL_PADDING)
                .min(MENU_MAX_HEIGHT),
        ),
    )
}

pub(crate) struct MenusScreen {
    pub(crate) demo_focus: FocusHandle,
    /// The demo area's window bounds, recorded by the `track_bounds` atom.
    pub(crate) demo_bounds: Bounds<Pixels>,
    /// The unclamped, demo-local anchor of the open menu.
    pub(crate) open_menu: Option<Point<Pixels>>,
    pub(crate) snap_to_pixel: bool,
    pub(crate) show_layout_grids: bool,
    pub(crate) selected_alignment: usize,
    pub(crate) expanded_more: bool,
    pub(crate) last_action: SharedString,
}

impl MenusScreen {
    pub(crate) fn new(cx: &mut Context<Storybook>) -> Self {
        Self {
            demo_focus: cx.focus_handle(),
            demo_bounds: Bounds::default(),
            open_menu: None,
            snap_to_pixel: true,
            show_layout_grids: false,
            selected_alignment: 0,
            expanded_more: false,
            last_action: "Ready — right-click the demo area, or use a keyboard opener".into(),
        }
    }

    /// Opens the demo menu at a demo-local anchor; clamping happens at
    /// render time against the tracked demo bounds.
    pub(crate) fn open_menu_at(&mut self, anchor: Point<Pixels>, description: &str) {
        self.open_menu = Some(anchor);
        self.last_action = format!(
            "Opened the demo menu {description} at {:.0}, {:.0}",
            f32::from(anchor.x),
            f32::from(anchor.y)
        )
        .into();
    }

    pub(crate) fn close_menu(&mut self, reason: &str) {
        if self.open_menu.take().is_some() {
            self.last_action = format!("Closed the demo menu ({reason})").into();
        }
    }
}

fn taxonomy_panel(
    id: impl Into<gpui::ElementId>,
    label: &'static str,
    rows: Vec<AnyElement>,
    cx: &mut Context<Storybook>,
) -> AnyElement {
    v_flex()
        .w(px(TAXONOMY_MENU_WIDTH))
        .flex_none()
        .gap_1p5()
        .child(menu_panel(id, px(TAXONOMY_MENU_WIDTH), cx).children(rows))
        .child(
            div()
                .w_full()
                .text_center()
                .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                .child(label),
        )
        .into_any_element()
}

impl Storybook {
    fn render_simple_menu_taxonomy(&self, cx: &mut Context<Self>) -> AnyElement {
        taxonomy_panel(
            "menus-simple-panel",
            "Menu row/Simple",
            vec![
                MenuSimpleRow::new("menu-simple-default", "Row text")
                    .shortcut("⌥⇧⌘O")
                    .into_any_element(),
                MenuSimpleRow::new("menu-simple-disabled", "Row text")
                    .shortcut("⌥⇧⌘O")
                    .preview_state(MenuRowState::Disabled)
                    .into_any_element(),
                MenuSimpleRow::new("menu-simple-hover", "Row text")
                    .shortcut("⌥⇧⌘O")
                    .preview_state(MenuRowState::Hover)
                    .into_any_element(),
                MenuSimpleRow::new("menu-simple-submenu", "Row text")
                    .submenu(true)
                    .into_any_element(),
                MenuSimpleRow::new("menu-simple-submenu-hover", "Row text")
                    .submenu(true)
                    .preview_state(MenuRowState::Hover)
                    .into_any_element(),
            ],
            cx,
        )
    }

    fn render_complex_menu_lead_taxonomy(
        &self,
        slug: &'static str,
        label: &'static str,
        lead: MenuLead,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let trails = [
            ("None", MenuTrail::None),
            ("Badge", MenuTrail::Badge("3".into())),
            ("Checkbox", MenuTrail::Checkbox(true)),
            ("Mixed", MenuTrail::Mixed),
            ("Shortcut", MenuTrail::Shortcut("250".into())),
        ];
        let mut rows = Vec::with_capacity(10);
        for (trail_label, trail) in trails {
            for state in [MenuRowState::Default, MenuRowState::Hover] {
                rows.push(
                    MenuComplexRow::new(
                        SharedString::from(format!("menu-complex-{slug}-{trail_label}-{state:?}")),
                        format!("{trail_label} · {}", state.label()),
                    )
                    .lead(lead.clone())
                    .trail(trail.clone())
                    .preview_state(state)
                    .into_any_element(),
                );
            }
        }
        taxonomy_panel(
            SharedString::from(format!("menus-complex-{slug}-panel")),
            label,
            rows,
            cx,
        )
    }

    fn render_checkmark_menu_taxonomy(&self, cx: &mut Context<Self>) -> AnyElement {
        taxonomy_panel(
            "menus-checkmark-panel",
            "Menu row/Checkmark · 7 variants",
            vec![
                MenuCheckRow::new("menu-check-default", "Check · Default")
                    .on(true)
                    .shortcut("⌥⇧⌘O")
                    .into_any_element(),
                MenuCheckRow::new("menu-check-disabled", "Check · Disabled")
                    .on(true)
                    .shortcut("⌥⇧⌘O")
                    .preview_state(MenuRowState::Disabled)
                    .into_any_element(),
                MenuCheckRow::new("menu-check-hover", "Check · Hover")
                    .on(true)
                    .shortcut("⌥⇧⌘O")
                    .preview_state(MenuRowState::Hover)
                    .into_any_element(),
                MenuCheckRow::new("menu-check-submenu", "Check · Submenu")
                    .on(true)
                    .submenu(true)
                    .into_any_element(),
                MenuCheckRow::new("menu-check-submenu-hover", "Check · Submenu · Hover")
                    .on(true)
                    .submenu(true)
                    .preview_state(MenuRowState::Hover)
                    .into_any_element(),
                MenuCheckRow::new("menu-dot-default", "Dot · Default")
                    .variant(MenuCheckVariant::Dot)
                    .on(true)
                    .shortcut("⌥⇧⌘O")
                    .into_any_element(),
                MenuCheckRow::new("menu-dot-hover", "Dot · Hover")
                    .variant(MenuCheckVariant::Dot)
                    .on(true)
                    .shortcut("⌥⇧⌘O")
                    .preview_state(MenuRowState::Hover)
                    .into_any_element(),
            ],
            cx,
        )
    }

    fn render_toggle_menu_taxonomy(&self, cx: &mut Context<Self>) -> AnyElement {
        taxonomy_panel(
            "menus-toggle-panel",
            "Menu row/Toggle · 4 variants",
            vec![
                MenuToggleRow::new("menu-toggle-on", "On · No icon")
                    .on(true)
                    .shortcut("⌥⇧⌘O")
                    .into_any_element(),
                MenuToggleRow::new("menu-toggle-off", "Off · No icon")
                    .shortcut("⌥⇧⌘O")
                    .into_any_element(),
                MenuToggleRow::new("menu-toggle-icon-on", "On · Icon")
                    .on(true)
                    .icon(LucideIcon::Square)
                    .shortcut("⌥⇧⌘O")
                    .into_any_element(),
                MenuToggleRow::new("menu-toggle-icon-off", "Off · Icon")
                    .icon(LucideIcon::Square)
                    .shortcut("⌥⇧⌘O")
                    .into_any_element(),
            ],
            cx,
        )
    }

    fn render_toolbar_menu_taxonomy(&self, cx: &mut Context<Self>) -> AnyElement {
        taxonomy_panel(
            "menus-toolbar-panel",
            "Menu row/Toolbar · 3 variants",
            vec![
                MenuToolbarRow::new("menu-tool-default", "Default", LucideIcon::Square)
                    .on(true)
                    .shortcut("R")
                    .into_any_element(),
                MenuToolbarRow::new("menu-tool-hover", "Hover", LucideIcon::Square)
                    .on(true)
                    .shortcut("R")
                    .preview_state(MenuRowState::Hover)
                    .into_any_element(),
                MenuToolbarRow::new("menu-tool-disabled", "Disabled", LucideIcon::Square)
                    .on(true)
                    .shortcut("R")
                    .preview_state(MenuRowState::Disabled)
                    .into_any_element(),
            ],
            cx,
        )
    }

    fn render_structure_menu_taxonomy(&self, cx: &mut Context<Self>) -> AnyElement {
        taxonomy_panel(
            "menus-structure-panel",
            "Heading 2 · Divider 1 · Expand 4 · Footer 1",
            vec![
                MenuHeading::new("Heading · Default").into_any_element(),
                MenuHeading::new("Heading · Toggle")
                    .alignment(MenuHeadingAlignment::Toggle)
                    .toggle_text("On")
                    .into_any_element(),
                MenuDivider.into_any_element(),
                MenuExpandRow::new("menu-expand-collapsed", "Collapsed · Default")
                    .into_any_element(),
                MenuExpandRow::new("menu-expand-expanded", "Expanded · Default")
                    .expanded(true)
                    .into_any_element(),
                MenuExpandRow::new("menu-expand-collapsed-hover", "Collapsed · Hover")
                    .preview_state(MenuRowState::Hover)
                    .into_any_element(),
                MenuExpandRow::new("menu-expand-expanded-hover", "Expanded · Hover")
                    .expanded(true)
                    .preview_state(MenuRowState::Hover)
                    .into_any_element(),
                MenuFooter::new("Footer · Default").into_any_element(),
            ],
            cx,
        )
    }

    fn render_live_menu_controls(&self, cx: &mut Context<Self>) -> AnyElement {
        let selected_alignment = self.menus_screen.selected_alignment;
        let snap_to_pixel = self.menus_screen.snap_to_pixel;
        let show_layout_grids = self.menus_screen.show_layout_grids;
        let expanded = self.menus_screen.expanded_more;
        let mut rows = vec![
            MenuHeading::new("Controlled values")
                .alignment(MenuHeadingAlignment::Toggle)
                .toggle_text("Live")
                .into_any_element(),
            MenuToggleRow::new("menu-live-snap", "Snap to pixel")
                .on(snap_to_pixel)
                .shortcut("⌘⇧'")
                .on_activate(cx.listener(|this, event: &ActivateEvent, _, cx| {
                    this.menus_screen.snap_to_pixel = !this.menus_screen.snap_to_pixel;
                    this.menus_screen.last_action = format!(
                        "Menu intent: Snap to pixel {} · {}",
                        if this.menus_screen.snap_to_pixel {
                            "on"
                        } else {
                            "off"
                        },
                        if event.keyboard {
                            "keyboard"
                        } else {
                            "pointer"
                        }
                    )
                    .into();
                    cx.notify();
                }))
                .into_any_element(),
            MenuToggleRow::new("menu-live-grid", "Layout grids")
                .on(show_layout_grids)
                .icon(LucideIcon::Grid2x2)
                .shortcut("⇧G")
                .on_activate(cx.listener(|this, _, _, cx| {
                    this.menus_screen.show_layout_grids = !this.menus_screen.show_layout_grids;
                    this.menus_screen.last_action = format!(
                        "Menu intent: Layout grids {}",
                        if this.menus_screen.show_layout_grids {
                            "on"
                        } else {
                            "off"
                        }
                    )
                    .into();
                    cx.notify();
                }))
                .into_any_element(),
            MenuDivider.into_any_element(),
            MenuCheckRow::new("menu-live-left", "Align left")
                .variant(MenuCheckVariant::Dot)
                .on(selected_alignment == 0)
                .on_activate(cx.listener(|this, _, _, cx| {
                    this.menus_screen.selected_alignment = 0;
                    this.menus_screen.last_action = "Menu intent: Align left".into();
                    cx.notify();
                }))
                .into_any_element(),
            MenuCheckRow::new("menu-live-center", "Align center")
                .variant(MenuCheckVariant::Dot)
                .on(selected_alignment == 1)
                .on_activate(cx.listener(|this, _, _, cx| {
                    this.menus_screen.selected_alignment = 1;
                    this.menus_screen.last_action = "Menu intent: Align center".into();
                    cx.notify();
                }))
                .into_any_element(),
            MenuCheckRow::new("menu-live-right", "Align right")
                .variant(MenuCheckVariant::Dot)
                .on(selected_alignment == 2)
                .on_activate(cx.listener(|this, _, _, cx| {
                    this.menus_screen.selected_alignment = 2;
                    this.menus_screen.last_action = "Menu intent: Align right".into();
                    cx.notify();
                }))
                .into_any_element(),
            MenuDivider.into_any_element(),
            MenuExpandRow::new("menu-live-expand", "More options")
                .expanded(expanded)
                .on_activate(cx.listener(|this, _, _, cx| {
                    this.menus_screen.expanded_more = !this.menus_screen.expanded_more;
                    this.menus_screen.last_action = format!(
                        "Menu intent: More options {}",
                        if this.menus_screen.expanded_more {
                            "expanded"
                        } else {
                            "collapsed"
                        }
                    )
                    .into();
                    cx.notify();
                }))
                .into_any_element(),
        ];
        if expanded {
            rows.push(
                MenuSimpleRow::new("menu-live-nested", "Nested option")
                    .submenu(true)
                    .into_any_element(),
            );
        }
        rows.push(MenuFooter::new("Host owns every value").into_any_element());
        taxonomy_panel(
            "menus-live-controls-panel",
            "Host-controlled behavior",
            rows,
            cx,
        )
    }

    fn render_multi_select_variant(
        &self,
        variant: MenuMultiSelectVariant,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let slug = match variant {
            MenuMultiSelectVariant::Default => "default",
            MenuMultiSelectVariant::MixedIcons => "mixed-icons",
            MenuMultiSelectVariant::Avatars => "avatars",
            MenuMultiSelectVariant::LabelOnly => "label-only",
        };
        if variant == MenuMultiSelectVariant::LabelOnly {
            return taxonomy_panel(
                "menus-multiselect-label-only",
                variant.label(),
                vec![
                    MenuComplexRow::new("menus-label-only-off", "Off")
                        .trail(MenuTrail::Checkbox(false))
                        .into_any_element(),
                    MenuComplexRow::new("menus-label-only-design", "Design")
                        .trail(MenuTrail::Checkbox(true))
                        .into_any_element(),
                    MenuComplexRow::new("menus-label-only-figjam", "FigJam")
                        .trail(MenuTrail::Checkbox(false))
                        .into_any_element(),
                    MenuComplexRow::new("menus-label-only-slides", "Slides")
                        .trail(MenuTrail::Checkbox(false))
                        .into_any_element(),
                    MenuFooter::new("Select all").into_any_element(),
                ],
                cx,
            );
        }
        let (projects_lead, teams_lead) = match variant {
            MenuMultiSelectVariant::Default | MenuMultiSelectVariant::LabelOnly => {
                (MenuLead::None, MenuLead::None)
            }
            MenuMultiSelectVariant::MixedIcons => (
                MenuLead::Icon(LucideIcon::Folder),
                MenuLead::Icon(LucideIcon::Users),
            ),
            MenuMultiSelectVariant::Avatars => {
                (MenuLead::Avatar("P".into()), MenuLead::Avatar("T".into()))
            }
        };
        taxonomy_panel(
            SharedString::from(format!("menus-multiselect-{slug}")),
            variant.label(),
            vec![
                MenuHeading::new("Projects").into_any_element(),
                MenuComplexRow::new(
                    SharedString::from(format!("menus-multiselect-{slug}-project")),
                    "Project A",
                )
                .lead(projects_lead)
                .trail(MenuTrail::Checkbox(true))
                .into_any_element(),
                MenuHeading::new("Teams").into_any_element(),
                MenuComplexRow::new(
                    SharedString::from(format!("menus-multiselect-{slug}-team")),
                    "Team A",
                )
                .lead(teams_lead)
                .trail(MenuTrail::Mixed)
                .into_any_element(),
                MenuFooter::new("Clear all").into_any_element(),
            ],
            cx,
        )
    }

    fn render_composed_menu_example(&self, cx: &mut Context<Self>) -> AnyElement {
        taxonomy_panel(
            "menus-composed-example",
            "Menu example · shortcuts and nested rows",
            vec![
                MenuSimpleRow::new("menu-example-copy", "Copy")
                    .shortcut("⌘C")
                    .preview_state(MenuRowState::Hover)
                    .into_any_element(),
                MenuSimpleRow::new("menu-example-paste", "Copy / Paste as")
                    .submenu(true)
                    .into_any_element(),
                MenuDivider.into_any_element(),
                MenuSimpleRow::new("menu-example-front", "Bring to front")
                    .shortcut("⌘]")
                    .into_any_element(),
                MenuSimpleRow::new("menu-example-back", "Send to back")
                    .shortcut("⌘[")
                    .into_any_element(),
                MenuDivider.into_any_element(),
                MenuSimpleRow::new("menu-example-flatten", "Flatten")
                    .shortcut("⌘E")
                    .into_any_element(),
            ],
            cx,
        )
    }

    fn render_composed_toggle_example(&self, cx: &mut Context<Self>) -> AnyElement {
        taxonomy_panel(
            "menus-composed-toggles",
            "Menu example toggles",
            vec![
                MenuHeading::new("View").into_any_element(),
                MenuToggleRow::new("menu-example-pixel-preview", "Pixel preview")
                    .on(true)
                    .shortcut("⌘Y")
                    .into_any_element(),
                MenuToggleRow::new("menu-example-pixel-grid", "Pixel grid")
                    .on(true)
                    .icon(LucideIcon::Grid2x2)
                    .into_any_element(),
                MenuToggleRow::new("menu-example-layout-grid", "Layout grids")
                    .icon(LucideIcon::LayoutGrid)
                    .into_any_element(),
                MenuDivider.into_any_element(),
                MenuCheckRow::new("menu-example-rulers", "Rulers")
                    .on(true)
                    .shortcut("⇧R")
                    .into_any_element(),
                MenuCheckRow::new("menu-example-outlines", "Outlines")
                    .shortcut("⇧O")
                    .into_any_element(),
            ],
            cx,
        )
    }

    fn render_composed_filter_example(&self, cx: &mut Context<Self>) -> AnyElement {
        taxonomy_panel(
            "menus-composed-filters",
            "Menu example filters",
            vec![
                MenuHeading::new("File type").into_any_element(),
                MenuCheckRow::new("menu-filter-design", "Design")
                    .on(true)
                    .into_any_element(),
                MenuCheckRow::new("menu-filter-figjam", "FigJam").into_any_element(),
                MenuCheckRow::new("menu-filter-slides", "Slides")
                    .on(true)
                    .into_any_element(),
                MenuDivider.into_any_element(),
                MenuHeading::new("Include").into_any_element(),
                MenuCheckRow::new("menu-filter-comments", "Comments")
                    .on(true)
                    .into_any_element(),
                MenuCheckRow::new("menu-filter-prototypes", "Prototypes").into_any_element(),
                MenuCheckRow::new("menu-filter-annotations", "Annotations")
                    .on(true)
                    .into_any_element(),
                MenuFooter::new("Clear all").into_any_element(),
            ],
            cx,
        )
    }

    fn render_menus_demo_menu(&self, anchor: Point<Pixels>, cx: &mut Context<Self>) -> AnyElement {
        let menu_size = menu_demo_size();
        let origin = clamp_menu_origin(anchor, self.menus_screen.demo_bounds.size, menu_size);
        let mut surface = menu_surface(
            "menus-demo-menu",
            origin,
            menu_size.width,
            px(MENU_MAX_HEIGHT),
            px(8.),
            cx,
        )
        .debug_selector(|| "menus-demo-menu".to_owned())
        .on_mouse_down_out(cx.listener(|this, _, _, cx| {
            this.menus_screen.close_menu("clicked outside");
            cx.notify();
        }));
        for (index, label) in MENU_DEMO_ITEMS.into_iter().enumerate() {
            surface = surface.child(
                menu_item(
                    SharedString::from(format!("menus-demo-item-{index}")),
                    px(MENU_ITEM_HEIGHT),
                    cx,
                )
                .debug_selector(move || format!("menus-demo-item-{index}"))
                .on_activate(cx.listener(move |this, event: &ActivateEvent, _, cx| {
                    this.menus_screen.open_menu = None;
                    this.menus_screen.last_action = format!(
                        "Demo intent: {label} · via {}",
                        if event.keyboard {
                            "Enter/Space"
                        } else {
                            "pointer"
                        }
                    )
                    .into();
                    cx.notify();
                }))
                .child(truncating_label(label)),
            );
        }
        surface.into_any_element()
    }

    fn render_menus_demo_area(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .id("menus-demo-area")
            .debug_selector(|| "menus-demo-area".to_owned())
            .track_focus(&self.menus_screen.demo_focus)
            .relative()
            .w_full()
            .h(px(320.))
            .overflow_hidden()
            .rounded(px(8.))
            .border_1()
            .border_color(fanta_gpui::atoms::SemanticColor::Border.resolve(cx))
            .bg(fanta_gpui::atoms::SemanticColor::BackgroundSecondary
                .resolve(cx)
                .opacity(0.35))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    let local = point(
                        event.position.x - this.menus_screen.demo_bounds.origin.x,
                        event.position.y - this.menus_screen.demo_bounds.origin.y,
                    );
                    this.menus_screen.open_menu_at(local, "at the pointer");
                    cx.notify();
                }),
            )
            .child(
                // The center crosshair marks the demo surface as a click
                // target; the copy beneath it names the interaction.
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        v_flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .relative()
                                    .size(px(40.))
                                    .child(
                                        div()
                                            .absolute()
                                            .left(px(19.5))
                                            .top_0()
                                            .bottom_0()
                                            .w(px(1.))
                                            .bg(fanta_gpui::atoms::SemanticColor::TextTertiary
                                                .resolve(cx)
                                                .opacity(0.4)),
                                    )
                                    .child(
                                        div()
                                            .absolute()
                                            .top(px(19.5))
                                            .left_0()
                                            .right_0()
                                            .h(px(1.))
                                            .bg(fanta_gpui::atoms::SemanticColor::TextTertiary
                                                .resolve(cx)
                                                .opacity(0.4)),
                                    )
                                    .child(
                                        div()
                                            .absolute()
                                            .left(px(14.))
                                            .top(px(14.))
                                            .size(px(12.))
                                            .rounded_full()
                                            .border_1()
                                            .border_color(
                                                fanta_gpui::atoms::SemanticColor::TextTertiary
                                                    .resolve(cx)
                                                    .opacity(0.4),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .max_w(px(360.))
                                    .text_center()
                                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                                    .text_color(
                                        fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx),
                                    )
                                    .child(
                                        "Right-click anywhere — try the corners: the menu \
                                         clamps inside this frame on both axes",
                                    ),
                            ),
                    ),
            )
            .child(track_bounds(cx.entity(), |story, bounds| {
                story.menus_screen.demo_bounds = bounds;
            }))
            .when_some(self.menus_screen.open_menu, |area, anchor| {
                area.child(self.render_menus_demo_menu(anchor, cx))
            })
            .into_any_element()
    }

    pub(crate) fn render_menus_story(&self, cx: &mut Context<Self>) -> AnyElement {
        let demo_size = menu_demo_size();
        let near_corner_anchor = point(
            (self.menus_screen.demo_bounds.size.width - px(24.)).max(px(0.)),
            (self.menus_screen.demo_bounds.size.height - px(24.)).max(px(0.)),
        );
        let openers = h_flex()
            .gap_2()
            .flex_wrap()
            .child(
                fanta_gpui::atoms::ui_button("menus-open-center")
                    .debug_selector(|| "menus-open-center".to_owned())
                    .outline()
                    .small()
                    .label("Open near center")
                    .on_activate(cx.listener(|this, _, _, cx| {
                        let bounds = this.menus_screen.demo_bounds;
                        this.menus_screen.open_menu_at(
                            point(bounds.size.width / 2., bounds.size.height / 2.),
                            "near the center",
                        );
                        cx.notify();
                    })),
            )
            .child(
                fanta_gpui::atoms::ui_button("menus-open-corner")
                    .debug_selector(|| "menus-open-corner".to_owned())
                    .outline()
                    .small()
                    .label("Open near the bottom-right corner")
                    .on_activate(cx.listener(move |this, _, _, cx| {
                        this.menus_screen
                            .open_menu_at(near_corner_anchor, "near the corner");
                        cx.notify();
                    })),
            );

        specimen_story_root("storybook-menus")
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if event.keystroke.key == "escape" && this.menus_screen.open_menu.is_some() {
                    this.menus_screen.close_menu("Escape");
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .child(specimen_card(
                "menus-coverage",
                "Figma coverage · 64 specimens",
                "Every component-set child represented by the linked Figma nodes is rendered below: \
                 57 row variants and 7 composed menu variants. The counts come from the actual \
                 component-set dimensions and variant properties, not from inferred combinations.",
                h_flex()
                    .gap_4()
                    .flex_wrap()
                    .child(format!("{FIGMA_MENU_SPECIMENS} total"))
                    .child(format!("{FIGMA_ROW_VARIANTS} row variants"))
                    .child(format!("{FIGMA_COMPOSED_VARIANTS} composed variants"))
                    .child(format!("{FIGMA_SIMPLE_VARIANTS} simple"))
                    .child(format!("{FIGMA_COMPLEX_VARIANTS} complex"))
                    .child(format!("{FIGMA_CHECKMARK_VARIANTS} checkmark"))
                    .child(format!("{FIGMA_TOGGLE_VARIANTS} toggle"))
                    .child(format!("{FIGMA_TOOLBAR_VARIANTS} toolbar"))
                    .child(format!("{FIGMA_HEADING_VARIANTS} heading"))
                    .child(format!("{FIGMA_DIVIDER_VARIANTS} divider"))
                    .child(format!("{FIGMA_EXPAND_VARIANTS} expand"))
                    .child(format!("{FIGMA_FOOTER_VARIANTS} footer"))
                    .child(format!("{FIGMA_MULTI_SELECT_VARIANTS} multi-select"))
                    .child(format!("{FIGMA_COMPOSED_EXAMPLES} composed examples"))
                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                    .text_color(fanta_gpui::atoms::SemanticColor::TextSecondary.resolve(cx))
                    .into_any_element(),
                cx,
            ))
            .child(specimen_card(
                "menus-row-families",
                "Simple and complex rows · 35 variants",
                "The Figma taxonomy separates a compact label/shortcut/submenu row from a row \
                 whose leading and trailing content vary independently. Complex is the complete \
                 3 lead × 5 trail × 2 state matrix.",
                specimen_rows(vec![specimen_row(
                    "menus-row-families-row",
                    "Simple 5 · Complex 30",
                    vec![
                        self.render_simple_menu_taxonomy(cx),
                        self.render_complex_menu_lead_taxonomy(
                            "none",
                            "Complex · Lead none · 10 variants",
                            MenuLead::None,
                            cx,
                        ),
                        self.render_complex_menu_lead_taxonomy(
                            "icon",
                            "Complex · Lead icon · 10 variants",
                            MenuLead::Icon(LucideIcon::Frame),
                            cx,
                        ),
                        self.render_complex_menu_lead_taxonomy(
                            "avatar",
                            "Complex · Lead avatar · 10 variants",
                            MenuLead::Avatar("JR".into()),
                            cx,
                        ),
                    ],
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "menus-selection-families",
                "Checkmark, toggle, and toolbar rows · 14 variants",
                "This is the exact 7 + 4 + 3 variant inventory from Figma. The separate live panel \
                 demonstrates that selection and toggle values remain host-controlled.",
                specimen_rows(vec![specimen_row(
                    "menus-selection-families-row",
                    "Checkmark 7 · Toggle 4 · Toolbar 3 · Live behavior",
                    vec![
                        self.render_checkmark_menu_taxonomy(cx),
                        self.render_toggle_menu_taxonomy(cx),
                        self.render_toolbar_menu_taxonomy(cx),
                        self.render_live_menu_controls(cx),
                    ],
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "menus-structure-family",
                "Structural rows · 8 variants",
                "Heading, divider, expand, and footer remain separate primitives. Both heading \
                 alignments and all four expanded/state combinations are displayed explicitly.",
                specimen_rows(vec![specimen_row(
                    "menus-structure-family-row",
                    "Heading 2 · Divider 1 · Expand 4 · Footer 1",
                    vec![self.render_structure_menu_taxonomy(cx)],
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "menus-multiselect-family",
                "Menu multi-select",
                "The four Figma variants share one menu composition. Only the leading identity \
                 changes between plain labels, mixed icons, and avatars; the checkbox and mixed \
                 selection trails keep the same layout.",
                specimen_rows(vec![specimen_row(
                    "menus-multiselect-family-row",
                    "Default · Mixed icons · Avatars · Label only",
                    MenuMultiSelectVariant::ALL
                        .into_iter()
                        .map(|variant| self.render_multi_select_variant(variant, cx))
                        .collect(),
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "menus-composed-examples",
                "Composed menu examples · 3 variants",
                "The three standalone Figma examples are represented separately: the shortcut and \
                 submenu menu, the toggle menu, and the filter menu.",
                specimen_rows(vec![specimen_row(
                    "menus-composed-examples-row",
                    "Menu example · Menu example toggles · Menu example filters",
                    vec![
                        self.render_composed_menu_example(cx),
                        self.render_composed_toggle_example(cx),
                        self.render_composed_filter_example(cx),
                    ],
                    cx,
                )]),
                cx,
            ))
            .child(specimen_card(
                "menus-live-demo",
                "menu_surface / menu_item / clamp_menu_origin · live context menu",
                "The molecule owns the popover chrome, the row treatment, and \
                 two-axis clamping; the caller owns dismissal and the activation \
                 handlers. Every item routes through ControlExt::on_activate, so \
                 pointer clicks and Enter/Space on a focused item emit the same \
                 demo intent. The openers are gpui Buttons registered through \
                 ButtonControlExt::on_activate.",
                specimen_rows(vec![
                    specimen_row(
                        "menus-openers-row",
                        "Keyboard openers · Tab to one, then Enter or Space",
                        vec![openers.into_any_element()],
                        cx,
                    ),
                    specimen_row(
                        "menus-demo-row",
                        "Clamping demo area · anchor at the crosshair or any corner",
                        vec![self.render_menus_demo_area(cx)],
                        cx,
                    ),
                ]),
                cx,
            ))
            .child(specimen_card(
                "menus-clamping-copy",
                "Two-axis window clamping",
                "clamp_menu_origin clamps the anchor so the menu stays inside its \
                 container on both axes and floors at the top-left for oversized \
                 menus. §12 established the rule for toolbar popups; \
                 pointer-anchored context menus inherit it through this molecule, \
                 and the size passed to the clamp is the same one the surface \
                 renders at.",
                div()
                    .typography(fanta_gpui::atoms::TypographyToken::BodyMedium)
                    .text_color(fanta_gpui::atoms::SemanticColor::TextTertiary.resolve(cx))
                    .child(format!(
                        "Demo menu size: {:.0} × {:.0} px",
                        f32::from(demo_size.width),
                        f32::from(demo_size.height)
                    ))
                    .into_any_element(),
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn render_menus_reference(&self, cx: &mut Context<Self>) -> AnyElement {
        self.render_reference_component_fixture(
            "storybook-reference-menus",
            self.render_menus_story(cx),
            self.menus_screen.last_action.clone(),
            cx,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figma_menu_story_covers_every_linked_variant() {
        assert_eq!(FIGMA_COMPLEX_VARIANTS, 3 * 5 * 2);
        assert_eq!(FIGMA_ROW_VARIANTS, 57);
        assert_eq!(FIGMA_COMPOSED_VARIANTS, 7);
        assert_eq!(FIGMA_MENU_SPECIMENS, 64);
    }

    #[test]
    fn demo_menu_size_matches_its_item_stack_and_stays_clampable() {
        let menu = menu_demo_size();
        assert_eq!(menu.width, px(MENU_WIDTH));
        assert_eq!(
            menu.height,
            px(MENU_DEMO_ITEMS.len() as f32 * MENU_ITEM_HEIGHT + MENU_VERTICAL_PADDING)
        );
        // A corner anchor inside the 320 px demo area clamps on both axes.
        let clamped = clamp_menu_origin(point(px(890.), px(310.)), size(px(900.), px(320.)), menu);
        assert_eq!(clamped.x, px(900.) - menu.width);
        assert_eq!(clamped.y, px(320.) - menu.height);
    }
}
