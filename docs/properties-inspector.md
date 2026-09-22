# Properties inspector

`PropertiesInspector` is the right-sidebar layout. Its header embeds the shared
`ZoomControls` molecule directly; it does not nest a floating `ZoomBar`. Collapse
keeps a compact themed zoom card and the reopen button. Outer sidebar borders
belong to the host. Only internal section and tab separators are drawn.

The layout accepts six independent `AnyView` children: Design, Motion, Draw,
Code, Prototype, and Comments. Tab/collapse changes are presentation state and
emit `PropertiesInspectorAction`; hosts can synchronize them with setters.
Zoom requests carry `ZoomControlsAction`, and accepted zoom is echoed through
`set_zoom`. The layout never handles document edits from a child.

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

## Paint editor

Fill, Stroke, selection colors, page backgrounds, and auxiliary color controls
mount the shared `paint_picker::PaintPicker` organism with a single surface.
It is also exported independently with `PaintPickerAction` and
`PaintPickerTarget`, and demonstrated in the **Paint picker** story. See
ARCHITECTURE §20 for the controlled edit and dismissal contract.
