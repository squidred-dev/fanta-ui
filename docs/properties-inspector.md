# Properties inspector

`PropertiesInspector` is the right-sidebar layout. Its header embeds the shared
`ZoomControls` molecule directly; it does not nest a floating `ZoomBar`. Collapse
keeps a compact themed zoom card and the reopen button. Outer sidebar borders
belong to the host. Only internal section and tab separators are drawn.

All six modes use a single horizontal Zed tab bar in this order: Design,
Motion, Draw, Code, Prototype, Comments. Zed's `ui::TabBar` and `ui::Tab`
provide rectangular active/inactive surfaces, joined borders, and UI labels.
Tabs keep their natural widths and 14 px typography; the strip scrolls at
compact panel widths. Previous/next controls expose every mode by pointer,
and selecting a mode brings its tab into view. Left/Right cycle all six modes;
Up/Down are equivalent, while Home/End select the first/last mode. Focus follows
selection so Enter activates the current tab. Pointer activation mounts that
mode's supplied child view. Resizing the strip keeps the active tab visible while ordinary renders preserve
manual scrolling. Hosts without both a Zed theme and its settings provider use
the same tab-strip geometry through the gpui-component fallback.

Nested paint, media, shader, type-settings, and export content navigation uses
the same rectangular Zed tabs. Labels align left, tabs retain natural widths,
and a narrow strip scrolls to the selected tab. Arrow keys and Home/End move
both selection and focus; Enter and Space activate the focused tab. Property
choices such as repeat layout and alignment remain segmented controls.

Inspector chrome follows the active Zed panel and elevated-surface colors when
the host installs Zed's theme, with the gpui-component theme as a standalone
fallback. Shared panel typography uses Zed's rem-scaled 14 px UI copy, a 20 px
line height, and 12 px captions. Fields, collection rows, disclosure headers,
menus, and action buttons share the same compact geometry and Lucide icons.
Section headers are 32 px, side insets are 16 px, body bottom padding is 12 px,
attached captions use 4 px gaps, field rows use 8 px gaps, and distinct groups
use 12 px gaps. Section bodies
start directly below the header; an explicit 8 px inset is used for standalone
content that needs separation.

The layout accepts six independent `AnyView` children: Design, Motion, Draw,
Code, Prototype, and Comments. Tab/collapse changes are presentation state and
emit `PropertiesInspectorAction`; hosts can synchronize them with setters.
Zoom requests carry `ZoomControlsAction`, and accepted zoom is echoed through
`set_zoom`. The layout never handles document edits from a child.

Prototype presentation uses an icon-only play control in the shared header,
visible only on the Prototype tab. Hosts call `set_can_present` with their
accepted presentation availability (initially `false`) and handle
`PropertiesInspectorAction::PresentRequested`. This availability is independent
of document edit permission, so read-only prototypes can still be played.
`PrototypeInspector` places the same play icon in its header when mounted
independently. Composed hosts call `set_show_presentation_action(false)` on that
child to keep one presentation action in the shared header. `PseudoEditor`
canvas and top-bar chrome do not add a Present control.

## Composition

- `DesignInspector` composes `DesignPropertyPanel` entities for the current
  selection. Alignment, Layout, Appearance, Fill, Stroke, Effects, Export,
  Typography and the contextual component/geometry/page sections share a single
  `DesignPanel` edit controller. The old inspector shell and navigation are not
  mounted. Hosts subscribe to the controller's existing `DesignPanelAction` and
  keep supplying its canonical snapshot; resource catalogs, permissions,
  transactional edits, menus and paint pickers retain their existing behavior.
- `MotionInspector` receives animation timing, easing, playback and property
  projections and emits explicit motion requests.
- `DrawInspector` consumes the same controlled brush options as the toolbar.
  One compact tip selector exposes the host's brush catalog; arrow keys,
  Enter, and Escape support choosing and dismissing it. A small round/flat tip
  sample reflects accepted size, hardness, color, and opacity, scaled to fit.
  Textured or custom tips keep their host-supplied names without inventing a
  texture preview. Size/hardness and opacity/flow use paired exact fields with
  captions above, unit suffixes, and thin slider rails. Smoothing remains an
  exact field and slider; pressure and anti-aliasing use capability-dependent
  checkboxes. Controls share 24 px heights, 8 px row gaps, and 12 px group gaps.
  The inspector does not render or modify document content. This grouping follows
  Photoshop's [Brush Settings](https://helpx.adobe.com/photoshop/desktop/apply-painting-techniques/brushes-presets/display-brush-panel-brush-options.html)
  and [painting controls](https://helpx.adobe.com/ca/photoshop/using/painting-tools.html).
- `CodeInspector` displays host-generated code and property data. It requests
  language changes and copying; it does not generate application code itself.
- `PrototypeInspector` displays flows, connections and presentation settings.
- `CommentsInspector` receives threads and permissions; writing, replying and
  resolving emit typed requests. No messages are sent by the component.

Each organism can be mounted outside this layout. The layout has no dependency
on Fanta document or engine types. Each organism uses shared fields, theme tokens and menu primitives. Before
hiding or unmounting Design, hosts call `DesignInspector::deactivate(window, cx)`
to cancel active edits and close overlays without changing the accepted snapshot.
This applies to programmatic tab/collapse setters as well as user clicks.
Other tab organisms may preserve unsubmitted drafts while switching tabs.

Adding a Fill, Stroke, Effect, Layout guide, or Export configuration expands its
collapsed section immediately. The host still owns the collection; the echoed
new paint or effect opens its editor. Effect adjustments stay inline below
their row while editing and after host echoes, follow the stable effect ID
through reordering, and close when that effect disappears or becomes style
bound. Variable pickers have one opaque elevated surface, window-clamped
placement, and a bounded scrolling result list independent of the small
trigger's size.

## Storybook

The Properties inspector layout, five additional tab organisms and eight small
Design panel stories are independently registered. Inspection contexts and node
fixtures are in **Knobs**, outside the preview. The mock hosts accept and echo
edits, so inspecting a panel exercises its actual request path. The existing
Design story now mounts the composed Design inspector.

The design reference is Figma's [properties sidebar](https://help.figma.com/hc/en-us/articles/360039832014-Design-prototype-and-explore-layer-properties-in-the-right-sidebar)
and [Dev Mode inspection](https://help.figma.com/hc/en-us/articles/15023124644247-Guide-to-Dev-Mode).
Fanta exposes its requested six modes together; the host remains responsible for
code generation, prototype navigation, persistence and animation evaluation.

The compact default selection header groups secondary commands in More.
Frame, Group, and Section expose typed
`DesignSelectionHeaderCommand::ChangeLayerType` requests. A host applies a
permitted conversion and echoes its new inspection snapshot; clicking a menu
never mutates the supplied layer. Host-authored headers remain authoritative
and may offer this command in their own enabled menus.

## Paint editor

Fill, Stroke, selection colors, page backgrounds, and auxiliary color controls
mount the shared `paint_picker::PaintPicker` organism with a single surface.
It is also exported independently with `PaintPickerAction` and
`PaintPickerTarget`, and demonstrated in the **Paint picker** story. See
ARCHITECTURE §20 for the controlled edit and dismissal contract.

Hosts populate `DesignMediaPaintViewData::assets` with existing image/video
sources to enable reuse in the Assets destination. Upload is a separate explicit
source action. Switching the paint type only requests a paint edit, so a host
must not start a file chooser from that edit. Asset selection emits a Source edit
and retains the paint's placement and adjustment values. Media thumbnails and
shader previews remain supplied by the application renderer.
