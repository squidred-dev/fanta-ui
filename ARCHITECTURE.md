# Fanta GPUI Architecture

This is the design record for the reusable Fanta UI library. Section numbers
are stable; add new sections instead of renumbering existing ones.

## §1 Purpose and shape

`fanta-gpui` contains reusable GPUI components for Fanta hosts. It is not a
desktop application and it does not own documents, persistence, document/canvas
rendering, or editing behavior.

“Headless” in this repository means engine-decoupled and host-controlled, not
unstyled or renderer-agnostic. The crate deliberately owns themed GPUI
presentation, interaction continuity, and accessibility behavior.

The workspace has two layers:

```text
fanta-gpui-storybook ──► fanta-gpui ──► gpui-component ──► gpui
```

`fanta-gpui` is the facade consumed by a host. The storybook is a development
host and must not become a dependency of the library.

## §2 The engine seam

`fanta-gpui` must not depend on `fanta-engine` or any individual Fanta Engine
crate. The dependency direction in an application is:

```text
                     ┌──► fanta-engine
application host ────┤
                     └──► fanta-gpui
```

The host adapts engine identifiers and read models into component props, then
maps component intents to the engine's public operation surface. This keeps UI
release cadence independent from the document schema and avoids introducing a
windowing dependency into the engine.

## §3 Controlled domain data

Domain-facing component data is controlled:

- Input data is immutable for the duration of a render.
- Selection and expanded state are supplied by the host.
- User interaction emits typed intents.
- A component never mutates a Fanta document or silently invents domain state.
- A component entity may keep transient presentation state when interaction
  continuity requires it (focus, draft text, animation, popovers, filters,
  scroll position, or the active search-result cursor).

The Pages panel demonstrates the contract with `PagesPanelItem`,
`PagesPanelSearchResults`, `PagesPanelAction`, and `PagesPanel`. Committing a
new draft emits `CreateRequested`; it does not create a page itself. Committing
a rename emits `RenameRequested`; it does not update the supplied read model.
The host applies its operation and calls the appropriate setter with fresh
data.

## §4 Identity boundary

Engine IDs cross the UI boundary as opaque strings. Components compare and
return them unchanged. They do not parse ULIDs or assume a particular engine
identifier type.

This small adapter cost is deliberate: it prevents duplicate domain types and
lets another host reuse the component with its own model.

## §5 Visual system

Components use `gpui-component` primitives and its active theme. They should
prefer theme tokens over hard-coded colors. Fixed dimensions are acceptable
for intentional editor chrome geometry.

The library can reference the canonical `IconName` paths. Asset ownership stays
with the application because GPUI installs one application-level asset source.
The storybook uses `gpui-component-assets::Assets`; a production host may use
that source or compose those assets with its own.

Semantic toolbar icons use canonical or toolbar-owned vector rendering. A
Unicode character rendered through the current text font is not an icon source:
its outline, alignment, and availability vary by platform. Textual shortcut
hints may still use their conventional symbols.

## §6 Storybook

Every reusable component should have at least one interactive story covering
its meaningful visual states. Story state is local mock state only. Stories are
the place for visual tuning; domain workflows and Fanta operations belong in
the application host.

The Pages story is also a reference adapter: it subscribes to the component's
typed event stream, mutates mock host data, performs mock search, and supplies
the resulting page and search read models back to the component.

The Gallery keeps that mock host mounted in its primary window. **Open window**
creates a maximized component-only window backed by the same Storybook entity
and child component entities, so emitted intents still update one authoritative
mock-data owner. The opened window captures the selected story at activation;
later Gallery navigation does not retarget it. Window routing is development
host state and is not part of any reusable component contract.

The sidebar mirrors the library's atomic tiers (§16). The Atoms and
Molecules sections carry per-piece specimen stories built on the curated
public `fanta_gpui::atoms` / `fanta_gpui::molecules` API — the icon_button
activation matrix, the labeled ControlIcon grid, truncation, context menus
with two-axis clamping, list rows, and anchored popups with edge fades —
each wired into the shared intent log. The Atoms section also includes the
Icon assets page covering every `IconName` provided by the installed
`gpui-component` asset bundle; that catalog belongs to the Storybook because
it documents application-level assets rather than adding domain or document
state to `fanta-gpui`. The Getting started Welcome story renders the tier
map itself, and the screens tier is the storybook: every story is a screen
module feeding mock host data into the tiers above.

Stories are registered in one `StoryDescriptor` table
(`storybook/src/screens/mod.rs`). The registry is the single source for
sidebar grouping into the five fixed sections — Getting started, Atoms,
Molecules, Organisms, Layouts — plus render/focus/last-intent dispatch,
launch-name parsing, and
window sizing; `FANTA_STORYBOOK_STORY` rejects unknown names with the valid id
list instead of falling back. Each story is one screen module owning its mock
host state, seed and named-state fixtures, reducer, and knobs; the shared knob
framework and last-typed-intent chrome live beside the registry. Restart-style
environment variables (for example `FANTA_TOOLBAR_MODE`) survive as the
initial values of runtime knobs. Adding a story means adding one screen module
and one registry entry.

The Gallery is also a responsiveness and accessibility instrument. Every
story mounts the shared viewport harness (`screens/viewport.rs`): width and
height scrub handles with a live pixel readout plus per-story preset chips
sourced from the registry's `viewport_presets` (a component's natural sizes),
so stories reflow instead of clipping at one fixed surface. Story windows
scroll below the smallest registered viewport rather than clipping. The
registry's `keyboard_hints` feed the gallery footer's `?` help panel, and a
registry-driven storybook test walks every story's tab ring — proving a
focusable control exists, Tab traversal cycles without trapping, and Escape
is safe with and without an overlay open — so new stories are covered by
registering.

## §7 Safety and quality

- `unsafe` code is forbidden.
- Public intent and prop types should implement useful comparison/debug traits.
- `cargo fmt -p fanta-gpui -p fanta-gpui-storybook -- --check`,
  `cargo test --workspace`, and
  `cargo clippy --workspace --all-targets -- -D warnings` must pass. Scoped
  `fmt` is deliberate: `--all` would also reformat the patched sibling
  `fanta-edit` sources.
- New dependencies should preserve the seam in §2.

## §8 Pages panel integration contract

`PagesPanel` is a stateful GPUI entity because input focus, animation, and
popover continuity cannot be represented safely by reconstructing a
`RenderOnce` element each frame. This does not move domain ownership into the
component.

Host-controlled state:

- page identifiers and titles;
- selected page;
- expanded state when an external host changes it;
- search results and element counts;
- document mutations, selection changes, and replacement.

Component-owned presentation state:

- reveal animation and locally-triggered expanded state;
- the in-progress new/rename draft;
- Find/Replace mode and input values;
- open filter/scope menus, active filter pills, and search options;
- active result cursor.

The component emits a `PagesPanelAction` for every change that can affect the
document or another host surface. Opaque identifiers are returned unchanged.
Search requests contain only UI-level query fields and do not assume a Fanta
index implementation.

## §9 Commands and keyboard access

`fanta_gpui::init` registers reusable GPUI command actions and their default
key bindings. Hosts call it once after `gpui_component::init`. Controls route
pointer activation, Enter/Space activation, and command actions through the
same component methods, so keyboard access cannot bypass the headless intent
boundary.

Reloading or replacing a host keymap should rebuild the programmatic
`gpui-component` and Fanta bindings before applying user bindings. As a narrow
safety net for hosts that clear everything, focused toolbar, Pages, and Design
inputs recover their core editing commands and suppress broader canvas
bindings. A
Fanta surface key context must not shadow those editing actions.

Custom button-like controls share `ActivateControl` and the `FantaControl` key
context. A surface-specific command is warranted only when the action must also
be available outside the focused control. Controls register that convergence
through the shared activation builder (§16) rather than hand-wiring listener
pairs, so pointer/keyboard parity holds by construction and is proven once by
atom-level interaction tests.

Interactive controls participate in the GPUI tab-stop tree. Text inputs,
buttons, page and result rows, filter pills, search-scope controls, and menu
items must remain reachable with Tab and Shift-Tab. Dense lists and trees use
roving navigation instead of per-control stops: a row is one tab stop, Up/Down
(and Left/Right for tree disclosure, Home/End for list ends) move focus between
rows, and per-row satellite controls such as lock and visibility are
pointer-plus-command surfaces reachable through row-scoped key bindings that
emit the same typed intents. Focus styling is part of the component
presentation state — the shared atoms reserve their focus-ring border so
keyboard focus never shifts layout — and command actions remain public so a
host can replace key bindings or expose them in its own command palette.

## §10 Layers panel integration contract

`LayersPanel` follows the same engine seam as `PagesPanel`. A host supplies a
recursive `LayersPanelItem` read model and maps its own node types onto
`LayersPanelNodeKind`. Identifiers remain opaque strings; the component never
parses them or resolves document relationships outside the supplied tree.

Host-controlled state:

- node hierarchy, order, names, kinds, visibility, and lock state;
- current selection, including multiple selected identifiers;
- whole-panel expanded state when an external host replaces it;
- expanded identifiers when the host replaces or synchronizes tree state;
- document mutations, clipboard behavior, z-order, grouping, conversion,
  component operations, layout, plugins, widgets, motion, and layer reordering.

Component-owned presentation state:

- focused and hovered rows;
- the open contextual menu and its return focus;
- the in-progress rename draft;
- scroll position;
- an immediately updated whole-panel disclosure state;
- an immediately updated expansion list after a local disclosure interaction.

The Layers header emits `LayersPanelAction::PanelExpansionChanged` and can be
accepted or replaced through `set_expanded`. Tree-node expansion interactions
emit `LayersPanelAction::ExpansionChanged` and can be accepted or replaced
through `set_expanded_node_ids`. Selection, visibility, lock, rename, and
contextual menu operations always emit typed intents. They do not modify the
supplied node tree.

Dragging a row is presentation-only until it is dropped. A valid drop emits
`LayersPanelAction::MoveRequested` with opaque source and target identifiers
plus a before/inside/after placement. The host performs the reorder or
reparenting and supplies the resulting tree back through `set_nodes`. The
panel itself refuses only what the tree proves impossible (a node onto itself
or a descendant, `Inside` on a kind that cannot hold children — instances
included); every document rule beyond that (page roots, component masters,
recursive instances, locked parents, …) is the host's, supplied through
`set_drop_validator` so the drop highlight never promises a drop the host
would refuse. A refused target shows no highlight and emits nothing.

The tree is virtualized (`uniform_list` over the flattened visible rows): the
host tree is flattened once per `set_nodes` into a pre-order arena, the
visible-row list is recomputed only when the tree or the expansion set
changes, and each frame builds only the rows inside the viewport. Hosts with
tens of thousands of nodes per page therefore pay per-visible-row, never
per-node, and echo cost is bounded by `set_nodes` alone. `reveal_node` scrolls
a shown row to the viewport center (canvas selection → panel reveal); it does
not expand ancestors — the host owns expansion and echoes it through
`set_expanded_node_ids` first. `visible_row_ids` exposes the shown order for
host-side shift-click range selection over what the user actually sees.

Contextual menu contents are selected from UI capabilities associated with
`LayersPanelNodeKind`. This lets the component reproduce Figma-like menus
without importing a document schema or assuming that a host implements an
operation. Submenu-bearing actions are emitted as typed
`LayersPanelContextAction` values so the host can open its own deeper picker.

## §11 Design panel integration contract

`DesignPanel` is a controlled inspector surface. A host maps the active
selection, parent layout, permissions, and edit mode onto
`DesignPanelInspectionContext`, supplies an aggregate `DesignPanelNode`, and
applies `DesignPanelAction` intents through its own operation and undo system.
The component does not edit node properties or keep a second authoritative
selection.

Host-controlled state:

- no/single/multiple selection, parent layout, permissions, and edit mode;
- the active right-sidebar surface, echoed independently for the editable
  Design/Prototype and viewer Comment/Properties permission sets;
- the orthogonal editable Design/Draw workspace projection, plus the exact
  ordered target and validated corner-radius slider range used by Draw
  Appearance; changing workspace never replaces the accepted sidebar surface
  or canvas edit mode;
- the user-level cross-file Additional labels preference, supplied through a
  presentation-only setter and never represented as a document action;
- the user-level cross-file Small/Big numeric nudge preference, supplied as a
  validated `DesignNudgeSettings` value (default `1 / 10`) and never
  represented as a document action;
- selected-node header title, optional title menu, ordered primary/overflow
  controls, icon presentation, command availability, and disabled reasons;
- the selected node identifier, kind, name, visibility, position, size,
  aspect-ratio lock, whether it is nested below a component instance, and
  constraints;
- the exact ordered inspector sections and semantic node capabilities when a
  coarse compatibility kind cannot describe a host or future node losslessly;
- mixed, bound, unset, uniform, and read-only property states;
- exact ordered Smart Selection geometry, per-axis mixed/uniform spacing,
  arrange/tidy availability, and disabled or read-only reasons;
- auto-layout/grid mode, sizing, alignment, gaps, wrapping, padding, tracks,
  and selected-child participation;
- the exact ordered target, structural eligibility, and optional disabled
  reason for the native Add auto layout operation;
- the exact target-bound grouped Frame-preset catalog, dimensions,
  availability, and disabled reasons;
- component/instance properties, typography, discriminated shape geometry,
  Section properties, and Transform-group repeat modifiers;
- variable-font axis tags, ranges, defaults, steps, availability, binding
  provenance, and exact character-range targets;
- optional main-component/component-set authoring capabilities, stable
  property/Variant-option identities, selected-sublayer applied-property
  controls, and nested-property exposure candidates;
- the active character-range revision used by both typography and Fill
  intents while editing selected text; Stroke remains layer-wide;
- fills, strokes, effects, layout grids, selection colors, and exports;
- whole-collection Fill/Stroke Paint-style bindings and ordered style
  snapshots, independently from solid/gradient-stop Color-variable bindings;
- the Page/no-selection solid background plus the exact current-file nested
  Text, Color, Effect, and Layout-guide style tree;
- resolved and explicit variable modes for the exact Page or scene-node target;
- every document mutation and export operation.

Component-owned presentation state:

- the last supplied Design/Draw workspace value and retained slider entities;
  these project controlled host data and never become document state;
- the last supplied Additional labels value, defaulting off and retained
  across selection/context changes;
- the last supplied Small/Big nudge value plus the active scrub gesture's
  origin/current pointer Y and transient speed cue; these affect input
  interpretation only and never become document state;
- collapsed inspector sections;
- the inline Position-constraints disclosure state;
- scroll and focus continuity;
- an uncommitted numeric/text draft and select-menu highlight;
- the open Grid-dimensions popover and its uncommitted positive 2D candidate;
- the open Width/Height resizing menu and transiently disclosed min/max fields;
- the active on-canvas dimension-limit hover preview identity;
- one immutable active inspector-menu preview for node, stable-effect, or
  stable-paint options; its matching End payload survives target invalidation;
- the active fill/stroke or color-only picker, stable picker target, selected
  gradient stop, and uncommitted picker inputs;
- the open Type settings popover and its Basics/Details/Variable tab;
- the open selected-node title, Boolean/Flatten, or More popover;
- the open Appearance blend menu and advanced-corner disclosure;
- the open Page background picker, collapsed local-style folders, and anchored
  variable-mode browser;
- style-resource search text, page/library source filter, and list/grid view;
- the open grouped Frame-preset chooser;
- the native component-property Dialog plus its uncommitted typed create/edit
  draft, nullable Slot-limit inputs, and catalog selections;
- hover presentation for one host-supplied nested-property exposure candidate;
- expanded export advanced rows and the export-preview disclosure.

`DesignPanelNode::capabilities` is optional so existing integrations retain
their kind-derived presets. When supplied, `DesignPanelNodeCapabilities` is
authoritative: its ordered section list controls composition, while its
dimensions, visibility, X/Y coordinates, arrange actions, transforms,
aspect-ratio lock, auto-layout ownership/child/Add operations, Grid, resize,
clipping, fill, stroke, layer appearance, effects, constraints, and
Layout-guide flags gate both rendering and typed intents. This is the
forward-compatible boundary for `Other` and future host node types; stale data
in a disabled family never makes that family editable.

`Widget` is a canonical node kind and remains distinct from the generic
host-defined `Other` path. Its exact opaque profile follows Figma's
`WidgetNode`/`OpaqueNodeMixin` contract: Position exposes editable X/Y,
Appearance exposes visibility, Layout keeps inspectable read-only width and
height, and Export remains available. Arrange, rotation/flip, aspect lock,
auto-layout ownership/child/Add operations, resize, clipping, paints, opacity,
blend, effects, constraints, and Layout guides are absent. The Storybook
fixture `reference-widget` supplies explicit read-only width/height property
states rather than pretending the reusable component owns Widget sizing.

Adding auto layout is a structural host operation, not an ordinary layout
property edit. `DesignAddAutoLayoutViewData` binds host-resolved eligibility
and an optional disabled reason to the exact ordered current node target. The
panel renders **Add auto layout** in Layout only for a matching eligible
projection whose exact node capabilities allow the structural operation and
whose selection is not already an active auto-layout owner.
Activation emits `AddAutoLayoutRequested { target }`; viewers, disabled
projections, stale/reordered targets, and existing owners cannot emit. The
component never changes a Group to a Frame, invents a wrapper ID, chooses a
flow, or changes selection locally. The host performs the conversion or wraps
the selected layers and echoes a fresh inspection context. This matches
Figma's documented
[Add auto layout behavior](https://help.figma.com/hc/en-us/articles/5731482952599-Toggle-on-auto-layout-in-designs)
and
[one-or-more-layer workflow](https://help.figma.com/hc/en-us/articles/360040451373-Guide-to-auto-layout).

Smart Selection is a separate exact-target Position projection.
`DesignSmartSelectionViewData` describes whether the current ordered multiple
selection is not smart, horizontal, vertical, or two-dimensional; it supplies
only the applicable horizontal/vertical Space between fields. Each axis is
independently mixed or uniform and independently available or disabled.
Distribute-horizontal, distribute-vertical, and Tidy up visibility/access are
also host-authored, including disabled explanations. A projection is valid
only for at least two unique nonempty node IDs, and `[A, B]` never matches
`[B, A]`.

Spacing emits
`SmartSelectionSpacingEditRequested { target, axis, value, phase }`; arrange
controls emit `SmartSelectionArrangeRequested { target, operation }`. Both
retain the exact ordered target and are rejected when the projection is stale,
read-only, unavailable, or lacks edit permission. The panel owns only the
active field draft. It does not calculate spacing, move layers, tidy the
selection, or change its controlled readout, and it cancels the captured draft
when selection, permission, or host echo changes. This follows Figma's
documented
[Smart Selection workflow](https://help.figma.com/hc/en-us/articles/360040450233-Arrange-layers-with-Smart-selection).

Grid auto-layout dimensions use one atomic
`GridDimensionsEditRequested { node_id, dimensions, phase }` boundary. The
panel's retained right-sidebar selector and Number of columns/Number of rows
inputs both carry the exact stable node plus both positive counts; they never
decompose a 2D choice into per-track mutations. The host owns track/content
relocation and echoes complete authoritative row and column vectors. When
automatic rows are active, the echoed row count is derived and read-only while
column changes preserve it exactly. Arrow-key navigation changes only the
transient selector candidate until Enter/Space commit, and permissions,
capabilities, leaf read-only state, and stale identity gate every path. This
matches Figma's documented
[Grid picker and numeric fields](https://help.figma.com/hc/en-us/articles/31289469907863-Use-the-grid-auto-layout-flow).
An incomplete same-ID echo with an empty axis remains visible as exact raw
counts in a disabled trigger; the panel never fabricates tracks or weakens the
positive atomic dimensions contract.

Minimum and maximum dimensions remain nullable host fields on
`DesignLayout.item`. They are projected only for an active auto-layout owner
or a participating direct child. Width/Height menus expose resizing plus
separate Add min/Add max commands; adding a field changes presentation state
only, then the ordinary phased property editor emits the typed value. They are
the only Fixed/Hug/Fill controls in those valid contexts; no duplicate lower
Resizing row is projected. The combined Remove command emits
`OptionalNumber(None)` independently for every editable existing leaf, so a
bound or read-only sibling cannot be cleared accidentally. Existing host
values add the native flanked dimension icon; hover emits an exact start/end
preview pair without mutating the snapshot. Selection or permission changes
hide the transient fields and cancel that preview. This follows Figma's
documented
[minimum and maximum dimensions workflow](https://help.figma.com/hc/en-us/articles/360040451373-Explore-auto-layout-properties).

The selected-node header is a separate controlled command surface. Hosts
supply `DesignSelectionHeaderViewData`; the panel preserves its exact order
and does not infer additional controls when host data is present. Supplied
data is bound to the exact ordered selection target and is invalidated when
any selected node ID or the cardinality changes. Asynchronous integrations use
the explicit-target setter so an older response cannot attach itself to a new
selection. Direct controls and title/menu leaves emit
`SelectionHeaderCommandRequested { target, command }`, including the complete
multi-selection target. Opening More is presentation state only. View
permissions gate edit commands without preventing non-mutating selection
commands, while host-defined direct controls and individual menu leaves can
opt into viewer-safe access. A host-defined control with menu items is rendered
as a menu rather than as a direct command. Disabled reasons remain
host-authored.

Right-sidebar navigation uses one permission-neutral `DesignPanelSurface`
model. Editable contexts expose Design/Prototype and viewers expose
Comment/Properties. Button activation emits
`SurfaceChangeRequested { current, requested }`, a non-document intent; it
does not update the selected tab. The host rejects stale or permission-invalid
requests and echoes an accepted value through `set_active_surface`. Design and
Properties mount this component's inspector projections. Prototype and
Comment retain the shared header but mount an explicit host-owned empty
projection, so content from the previous surface cannot be misrepresented.
This follows Figma's documented
[right-sidebar surfaces](https://help.figma.com/hc/en-us/articles/360039832014-Design-prototype-and-explore-layer-properties-in-the-right-sidebar).

View-only permissions select a separate UI3 Properties projection rather than
reusing disabled editor controls. `DesignViewerPropertiesViewData` is bound to
the exact ordered `DesignPanelTarget`; hosts own its Content text, stable
section/row IDs, displayed strings, and exact copy payloads. Canonical leaf
rows reuse `PropertyCopyRequested`, while whole-section copy emits
`ViewerSectionCopyRequested`. Stroke is projected as Borders, whose active
CSS/Hex/RGB/HSL/HSB representation is host owned:
`ViewerSectionRepresentationChangeRequested` asks for a new representation and
the visible snapshot changes only after the host echoes it. Restricted viewers
remain inspectable but non-copyable. Crossing from editor to viewer permission
closes editor-only transient overlays.

Paint pickers present Solid, Gradient, Pattern, Image, Video, and Shader
classifications, with linear, radial, angular, and diamond gradient subtypes.
`DesignPaintPayload` retains the discriminated settings for each type,
including bindings, transforms, canonical pattern source-node/tiling data,
mode-discriminated media placement/filter data, and definition-ID-keyed shader
property values. Fill/Fit/Tile media rotations are quarter turns; Crop alone
owns an affine transform, and Tile alone owns a scaling factor. Only
the Unsupported case is opaque. Shader discovery/import and property resource
editing remain host intents. Each host-supplied paint ID is stable across a
collection reorder; an empty ID is the compatibility fallback to an index.
Picker and inline paint edits emit `PaintEditRequested` with a typed property,
value, and edit phase. Reordering emits `PaintReorderRequested`, and a host
echo resolves an open picker by paint ID rather than its previous index.
Hex accepts 3/4/6/8 digits and CSS accepts `rgba(...)`; explicit alpha maps to
solid-paint Opacity or the exact gradient-stop Color leaf. One focused text
session retains one Begin and one terminal even when an RGBA solid preview
crosses those two typed leaves. Per-paint blend mode remains a typed atomic
edit. WCAG contrast remains non-document view data, while its exact effective
background, ratio, Auto audience resolution, and nearest compliant colors are
host-owned `DesignColorContrastViewData`. The picker owns only its transient
category/level choice and emits a normal phased paint edit for a supplied
correction; it never predicts document mutation.
Source chooser buttons emit `PaintSourceReplaceRequested`; they never invent
or retain an application asset. Media-specific view data separately controls
property-edit and source-replacement capability, crop-tool lifecycle state,
and video preview state. Crop begin/preview/commit/cancel and video
play/pause/seek/scrub are stable-ID host intents; autoplay, loop, and poster
frame are not Design-tab paint document properties.
An Image/Video source preview and its nested Fill/Stroke swatch also accept one
external file when the exact paint capability permits it. Standard formats
are JPG/JPEG, PNG, HEIC, WebP, GIF, MP4, MOV, and WebM; TIFF is a deliberate
host opt-in. SVG, PDF, unknown, and multi-file drops are rejected atomically.
`PaintMediaSourceDropRequested` carries the exact collection/range target,
stable paint identity, resolved index, expected source ID, expected current
Image/Video kind, and classified path. The panel revalidates all of those
values plus permissions, style binding, read-only state, and accepted format
at drop time. The host still validates path readability and file content,
performs the import, and echoes a new source. The Storybook host allocates a
fresh monotonic source ID only after each drop passes that complete preflight;
the ID is independent from the node, paint, media kind, and local path.
Replacing Image with Video or Video with Image preserves paint identity/order,
common paint state, and media placement/crop/filter settings when the host
chooses to accept it.

The Appearance eye is a real controlled property, not decorative chrome.
Activating it emits `PropertyChangeRequested { property: Visible, ... }`; the
panel keeps displaying the host's `DesignPanelNode::visible` value until the
host echoes an accepted snapshot. `Pass through` is offered only for
structural container kinds, while leaf nodes expose the 18 non-pass-through
blend modes.

Page/no-selection inspection has its own controlled read model rather than
pretending the Page is a scene node. `DesignPageViewData` contains the stable
Page ID and the one solid `PageNode.backgrounds` color.
`DesignPageLocalStylesViewData` separately binds the exact current Page to a
current-file-only nested style tree in canonical Text, Color, Effect, then
Layout-guide order. Folder/style IDs are globally unique and opaque. Every
style command, create, folder-create, atomic delete, and move is re-resolved
against the current Page, kind, direct parent, index, and—when inserting—both
neighbor IDs. Style rows drag before another style, into a folder end, or to a
family-root end through that same move contract; drag state is presentation
only. Disabled ancestor folders disable the complete subtree. The panel never
mutates this snapshot; it owns only folder disclosure state and prunes stale
disclosure IDs after host echoes.

The default Variables entry point is navigation-bar-only, so Page inspection
does not render a Local variables browser. An explicit compatibility mode can
render one **Open variables** host-navigation action, never Browse/import.
The former grouped local/library resource types and `LocalResource*Requested`
actions remain deprecated API compatibility only; `DesignLocalResourceSource`
stays active because variable-mode collections use it.

Variable modes are a separate target projection shared by the Page header and
scene-node Appearance header. `DesignVariableModeViewData` preserves each
collection's stable ID/source, modes, default mode, resolved mode, and optional
explicit mode. A resolved mode may come from an ancestor; it must not be
mistaken for a target-owned override. Applying or clearing an explicit mode
emits a stable-ID intent for the exact Page or single-node target. Clear also
carries the currently echoed explicit mode ID so a host can reject a stale
request. Only popover openness is component-owned.
Sample-only Color-style rows are likewise controlled: hosts supply
`DesignColorStyleSampleViewData` with stable page and library identities, and
the picker emits `PaintColorStyleSampleRequested` for one solid-color or
stable gradient-stop leaf. This samples a resolved value only; it never writes
the whole Fill/Stroke Paint-style identity or a Color-variable binding.
Complete styles remain `DesignPaintStyleViewData`, while the older
`DesignColorStyleViewData` contract remains compatibility-only.
Image and Video are paint payloads rather than node kinds; the legacy
node-level `DesignMedia` snapshot and `ReplaceMediaRequested` intent are
compatibility-only and are not rendered or emitted by the panel.
Adding or removing fills, stroke paints, or grids likewise emits typed
collection intents. Effects use dedicated stable-ID add/edit/remove/reorder
intents because order changes rendering and an anchored settings popup must
survive host echoes. Effect styles, bindable effect leaves, Shader metadata,
and opaque future-effect payloads are all host-controlled. Effect Shader
properties retain their definition IDs through metadata reorder. Scalar and
geometry/gradient subleaves emit complete typed values with phased lifecycle;
asset and variable choosers are explicit stable-ID host intents. Variable
aliases detach by exact echoed ID, while opaque values remain inspectable and
read-only instead of emitting no-op edits. The older
whole-paint `PaintChangeRequested` remains an adapter surface for existing
hosts.
Export rows instead use stable configuration IDs with typed add, remove,
change, preview, and export-all intents; the host echoes
`DesignExportViewData`. Hosts accept a change by supplying updated inspection
context and view data. Continuous property editors additionally report
Begin/Preview/Commit/Cancel phases so the host can coalesce undo. Cancel is a
transaction boundary rather than a candidate value: the host restores the
authoritative target snapshot captured at Begin.

Every document-facing interaction preserves the exact ordered host selection.
Smart Selection spacing/arrange, general arrange/distribute, transform,
resize-to-fit, preview, selection-color, and export intents carry
`DesignPanelTarget` directly. Ordinary property,
typography, component, collection, paint, effect, style, and variable leaves
retain their compact single-node payload for a single selection. For a
multiple selection the panel emits that leaf inside
`TargetedNodeActionRequested { target, action }`; the outer target is
authoritative and the nested `node_id` is only a compatibility hint. Hosts
must apply the envelope as one multi-node document operation rather than
silently reducing it to the aggregate/first node.

Mixed, unset, bound, uniform, and read-only leaves remain a separate
host-controlled projection supplied through `set_property_value_states`.
Emitting a targeted candidate never changes those states. The Storybook
reference adapter validates every target member before reducer dispatch,
rejects the complete stale target instead of partially applying it, and then
replays its single-node mock reducer in the target's original order.
Page-level commands continue to use an explicit page target.

Native shape view data preserves API units and discriminants.
`DesignShapeGeometry` carries exactly one Polygon, Star, Ellipse, Boolean, or
read-only FigJam Table payload. Ellipse start/end angles remain radians and
inner radii remain `0..=1` ratios even though the panel displays degrees and
percentages. Figma's Start/Sweep/Ratio Appearance projection derives Sweep as
`ending_angle - starting_angle`; accepting a Sweep sets
`ending_angle = starting_angle + sweep`, while changing Start preserves the
current sweep. Polygon Count, Star Count/Ratio, and conditional Arc rows
compose inside Appearance in both Design and Draw. A default full ellipse
omits only its Arc rows. Boolean/Flatten is a selected-node header menu.
`DesignPanelSection::Geometry`, the absolute ending-angle property, and the
Boolean property editor remain explicit host-snapshot compatibility paths;
read-only FigJam Table counts also use that section. Canonical Figma Design
resolution never adds Geometry for a shape.

Mask state is orthogonal to node kind. An active mask resolves a dedicated
Mask section containing its typed Alpha/Vector/Luminance mode, while creating
or removing the mask remains a selected-node header command. A legacy
Geometry-only capability snapshot retains its old mask projection without
duplicating a simultaneously supplied Mask section. Hosts supply
`DesignCornerCapabilities` so uniform radius, independent radii, and smoothing
follow actual topology instead of a guessed node label.

Section state is a dedicated host-owned record containing contents visibility,
Dev status/description/Changed state, and feature capabilities. Sections share
scene visibility, fill, stroke, radius, and smoothing surfaces but
intentionally omit rotation/flip, layer opacity, blend, and effects. Share and
Changed-resolution controls emit typed requests only after revalidating the
exact node, feature capability, edit permission where required, and current
Changed state; the host performs external sharing or document changes and
echoes a fresh Section snapshot. Sections cannot participate in a parent
auto-layout flow or receive Add auto layout; those capability gates remain
false even when stale host context/view data claims eligibility.

Transform groups supply an ordered collection of stable-ID
`DesignRepeatModifier` values. Repeat mode is discriminated as linear
horizontal/vertical or radial, and count, Relative/Pixels unit, and offset are
typed. Add/remove/apply operations and phased modifier changes emit dedicated
intents. A modifier edit includes stable ID and current index; the panel
revalidates both together, along with permission and candidate validity, so
retained actions become inert across reorder and permission echoes. The
Storybook adapter mirrors those predicates before mutating mock state and
reports rejected stale actions rather than claiming success.

Typography remains host controlled. Resize mode, truncation, and nullable
maximum lines remain distinct fields, but their availability follows Figma's
node-level invariants: Max lines is exposed only for ending-truncated Auto
width/Auto height text, and an auto-layout child additionally requires
vertical Hug sizing. `None` is the native Auto value. Numeric Max lines and
Max height are mutually exclusive; the Storybook host reducer canonicalizes
either accepted operation atomically and echoes the complete node snapshot.
The reusable component never resolves that conflict locally. Max lines,
resize, and truncation remain whole-layer typography targets, while Max height
is a whole-node property. See Figma's
[text properties](https://help.figma.com/hc/en-us/articles/360039956634-Explore-text-properties),
[auto-layout sizing](https://help.figma.com/hc/en-us/articles/360040451373-Explore-auto-layout-properties),
and nullable
[`maxLines` API](https://developers.figma.com/docs/plugins/api/properties/nodes-maxlines/).
Line height and letter spacing preserve their units; alignment, case, list
spacing, and every decoration detail remain typed.
Font family and style remain independent string leaves, while numeric font
weight is a separate controlled leaf mapping exactly to Figma's Float
`fontWeight` text field and `FONT_WEIGHT` variable scope. The combined font
browser exposes separate Family and Style variable affordances, and Weight has
its own variable-aware continuous value editor. Mixed, unset, bound, and
read-only states therefore remain leaf-specific; a variable-bound Weight never
silently aliases or replaces the string-valued Style. All three variable
targets inherit the same WholeLayer versus selected-range identity as ordinary
typography edits.
Font catalogs and OpenType feature records are immutable host view data:
feature tags/defaults/availability are never synthesized, and unknown tags
stay opaque. Font import, font apply, and OpenType changes use separate typed
intents. Typography intents include whether object mode targets the whole text
layer or text edit mode targets the active character range.

TextPath native orientation is separate, opaque whole-node view data.
`DesignTextPathViewData` controls whether Figma's native **Flip text
orientation** command is available and carries the host's current
Default/Flipped readout. Activation emits
`TextPathFlipOrientationRequested` for the exact node; the component never
predicts the result or edits vector geometry. Figma's public TextPath API
currently exposes start data but no matching orientation field, so the host
echo is authoritative.

TextPath start placement remains separate whole-node API data. Segment and
normalized position controls appear only when the host explicitly enables the
start-data debug disclosure. They emit a complete typed `textPathStartData`
candidate and transaction phase; these diagnostic fields are not projected as
native Figma sidebar controls.

Vector and TextPath sub-selection is likewise host controlled.
`DesignVectorEditViewData` carries opaque stable vertex IDs, coordinates,
per-vertex corner radii, exact writable handle-mirroring values, topology,
selection, and read-only state. Contextual controls appear only for an editable
single selection in vector-edit mode with a host-selected vertex. Selection,
coordinate, radius, and handle-mirroring intents retain the exact vertex IDs
and `Begin`/`Preview`/`Commit`/`Cancel` phase. Mixed values remain a presentation
state, and branch topology suppresses ambiguous radius and tangent-mirroring
edits; the component never changes the supplied vector network.

Component inspection remains host controlled and is role-aware. A selected
main, variant child, component set, instance, slot definition, or slot instance
supplies typed Boolean/Text/Instance swap/Variant/Slot property definitions and
values, stable property and component identities, local/remote availability,
descriptions/documentation, and override/reset state. Slot definitions own no
document state in the panel: stretch/display/limit/preferred settings,
inserted-instance values, and limit violations are immutable view data.
Property, reset, slot-setting, clear, reset, and add-instance interactions emit
typed intents keyed by stable property ID. Slot contents are an ordered set of
arbitrary scene-node descriptors rather than instance-only rows, and child
select/remove/reorder/replace intents carry stable node IDs plus index hints.
Nullable minimum and maximum counts preserve the Plugin API's `null` state.
They are advisory guidance: the transient Limits disclosure presents met
guidelines with green checks and unmet guidelines with orange warnings, while
exceeding a maximum never disables insertion. Its preferred-instance **View
layers** action is an atomic `SlotLimitLayersSelectRequested` intent with the
exact violating child IDs in Slot order; the panel and host reject stale,
reordered, partial, or unselectable targets. Create/Edit drafts, Dialog
visibility, switches, Limits disclosure state, and catalog browsing are
transient presentation state. A confirmed definition edit carries its exact
expected and replacement metadata and typed definition in one
stable-property-ID intent, so the host can atomically reject stale
description/documentation/default/Slot-setting combinations and keep them in
one undo transaction. The host applies every operation and echoes a fresh
component snapshot.

Multiple selection is inspection context, not a synthetic node kind.
The first selection item is the host's aggregate visual model, not permission
to clone one real node wholesale. A heterogeneous aggregate supplies only
capabilities common to every exact target, clears type-specific and indexed
collection leaves that have no aggregate identity, and supplies
uniform/mixed/unset/bound property states explicitly. A binding is aggregate
`Bound` only when every selected target resolves the same binding identity.
When every selected node has a semantically identical complete Fill or Stroke
collection, the aggregate may retain that collection in its native section
while ignoring host-local paint and gradient-stop IDs. Those paints are
excluded from Selection colors, and edits are preflighted against the exact
ordered node target before any member is changed.
`DesignSelectionColors` is a host-owned, getSelectionColors-like aggregate
whose canonical rows retain one representative full Solid/Gradient Paint, the
exact ordered collection snapshot, occurrence counts, and stable node/paint
references. Optional gradient-stop references remain a compatibility adapter;
normal Gradient paints are not split into one row per stop. The
whole-collection Paint-style binding and Solid Color-variable binding are
independent identities. Same-RGBA projections with different Paint semantics,
Paint styles, Color variables, or read-only semantics remain distinct. Typed
full-paint edits, occurrence selection, Paint-style
apply/import/create/detach, and Color-variable
apply/import/create/detach intents all carry the stable row identity, exact
ordered references, and complete ordered multi-node target; they are not
routed through normal indexed Fill intents. Same-target host echoes re-resolve
an active picker by row ID, while a selection identity/order change cancels its
transaction. Style-bound rows gate paint/variable mutation until style detach;
a variable-bound Solid gates its color leaf without conflating paint opacity
or blend. Exact occurrence selection remains viewer-safe.

Stroke is modeled as one optional node-level geometry/style record containing
an indexed paint collection. Paint intents include both a stable paint ID and
the current compatibility index. Weight (including four retained side weights),
position, explicit Solid/Dashed/Custom dash mode, endpoint caps, join/miter,
lossless variable-width preset or ordered custom points, and discriminated
Basic/Stretch brush/Scatter brush/Dynamic/opaque complex-stroke data are
unindexed shared properties. Custom brush payloads remain host-preserved and
read-only because Figma exposes them but does not allow plugins to manufacture
them. The host also supplies stroke capabilities and path/edit topology so
invalid position, individual-side, endpoint, join, branching variable-width,
dynamic variable-width, and opaque-stroke controls are not exposed.

Layout guides use a discriminated Uniform/Columns/Rows view model. Columns and
rows retain axis-specific alignment, numeric-or-Auto counts, and only the
size/offset or Auto-size/margin/gutter fields valid for that alignment. Every
supported numeric leaf maps to one exact Figma variable field:
`sectionSize`, `count`, `offset`, or `gutterSize`; the stretch Margin and fixed
Offset controls deliberately share `offset` while their inspector-property
identity remains distinct. Color and opacity are independently editable.
Every guide has stable opaque identity with an index fallback for legacy
hosts. Ordinary field edits and removal use dedicated guide-ID intents;
continuous editors capture that identity at Begin and re-resolve it after host
echoes, so Preview/Commit/Cancel cannot drift to another guide after reorder.

Grid styles are node-level, matching Figma's single `gridStyleId` over the
complete ordered `layoutGrids` array. The host supplies the current binding,
page styles, and library styles (including import availability); the Layout
guides header browser emits atomic apply/create/detach/import intents and never
mutates the guide array. A bound Grid style makes the whole guide collection
read-only until the host accepts detach. The shared Number-variable catalog,
per-field bindings, import state, creation availability, detachability, and
read-only reasons are likewise host data. Each leaf browser preserves exact
Number-versus-Auto count compatibility and emits apply/import/detach/create
intents with a stable guide ID plus the exact property and API field.

Deprecated Design-panel properties and actions are isolated compatibility
adapters, never canonical renderer output. Their public
`compatibility_path()` classification names the replacement contract:
aggregate auto-layout alignment, typed effect leaves, typed Image/Video paint
edits, stable export configuration sizing/actions, stable layout-guide
edit/remove actions, typed paint edits, or stable paint-source replacement.
Storybook rejects compatibility-only paths before reducer dispatch.

## §12 Editor toolbar integration contract

`EditorToolbar` is a stateful, intrinsic editor-chrome dock. It renders its
persistent controls as one contained, content-sized surface; it does not claim
full-canvas bounds, choose viewport coordinates, or apply its own outer
positioning. A host supplies the active mode, selected tool, zoom, and
Draw/Dev/Motion option read models. The toolbar emits `ToolbarAction` intents
and never creates layers, changes a selection, runs a command, advances a
timeline, or invokes an AI service.

Host-controlled state:

- the active Draw, Design, Motion, or Dev mode;
- the selected primary tool and accepted mode-specific control values;
- canvas zoom, Draw stroke options, Dev readiness, and Motion transport state;
- every candidate the Draw and Motion option editors may offer: the swatch
  palette, weight and smoothing min/max/step ranges, brush styles, and
  animation styles — the toolbar presents these and never invents a value
  outside them;
- Agent context copy and suggestions, plus the allowed command subset/order;
- command execution, plugins, widgets, media placement, undo, present, share,
  generated content, document mutations, and timeline data.

Component-owned presentation state:

- the open split-tool, Actions, Agent, zoom, or chip option-editor overlay;
- Actions query and highlighted result, plus the menu highlight cursor;
- the unsent Agent prompt and focused suggestion/control;
- focus return, hover, pressed, tooltip, and flyout continuity;
- row scroll offsets and the overflow-fade visibility derived from them.

Each transient surface is anchored to the exact disclosure, Actions, Agent, or
zoom trigger that opened it. It may render in a deferred layer above the dock,
but it must not position itself against an unrelated full-canvas root or canvas
center. Component-owned popups snap inside the window with an 8 px edge margin.
Persistent primary, secondary, mode, zoom, and Agent controls remain inside the
dock; constrained hosts may let the dock's internal rows scroll.

Pointer controls, Enter/Space activation, and the command actions registered by
`fanta_gpui::init` converge on the same methods and typed intents. Hosts accept
an intent by calling `set_mode`, `set_active_tool`, `set_zoom_percent`, or the
appropriate options setter with fresh controlled data. The interactive
storybook is the mock host and is the only layer that applies those requests.
The host owns the dock's outer placement, canvas inset, and surrounding canvas
clipping; the toolbar owns collision handling for its transient surfaces.

## §13 Variables, assets, prototype, and timeline contracts

`VariablesPage`, `AssetsPanel`, `PrototypePanel`, and `Timeline` follow the
same controlled seam as the established editor surfaces. Hosts supply
collections, libraries, prototype settings, and transport values as immutable
view data. Every operation that can affect a document, external library,
prototype, or animation emits a typed intent.

The components may retain only presentation continuity: search drafts, scroll
position, dismissed educational hints, focus, and open/closed empty-state
guidance. They do not create variables, import libraries, change prototype
settings, add keyframes, seek, or run an agent.

## §14 Pseudo editor composition contract

`PseudoEditor` is a presentation-only integration shell. A host constructs and
subscribes to each child component, then passes those entities through
`PseudoEditorChildren`. The shell owns simulated surface tabs and variables
overlay visibility; it does not intercept or translate child intents.

The shell's canvas is the positioning context for the intrinsic toolbar. It
places a host-owned wrapper at bottom center and mounts `EditorToolbar` inside
that wrapper without asking the toolbar to fill the canvas. The standalone
Toolbar story uses the same canvas-relative bottom-center pattern. The shell or
storybook host, rather than the reusable toolbar, chooses the bottom inset and
adapts it when another host-owned surface such as the timeline occupies that
edge.

This makes the storybook composition useful for visual and integration testing
without turning it into a document host. Pages, layers, assets, design,
prototype, variables, timeline, and toolbar data remain controlled through
their individual component contracts.

## §15 Source boundaries and test levels

Crate entry points assemble modules; they do not own feature implementations.
Large surfaces should separate host-facing models, registered commands,
retained presentation state, render sections, and interaction tests whenever
those responsibilities can change independently. Storybook configuration,
fixture construction, mock reducers, and gallery rendering likewise belong in
focused modules rather than its binary entry point.

The public `design` module is the curated Design API boundary. The crate
prelude re-exports that boundary instead of maintaining a second hand-written
inventory of Design types.

Tests provide four complementary levels:

- pure model tests cover validation, compatibility, and typed intent payloads;
- GPUI interaction tests prove pointer and keyboard paths emit the same intent;
- composed host-flow tests render real components, apply emitted intents in a
  mock host, and echo controlled data back into the component;
- Storybook tests and reference fixtures cover visual assembly and mock-host
  behavior.

`tests/architecture_boundaries.rs` makes the dependency direction executable:
the reusable crate may not depend on any Fanta domain/application crate, may
not reference its Storybook host, and the Storybook must consume the library
through its public facade.

## §16 Atomic design tiers

The library source is organized by atomic design tier:

```text
crates/fanta-gpui/src/
  atoms/        activation, buttons, vector icons, truncation, bounds tracking
  molecules/    menu chrome + clamping, anchored popups, list rows, edge fades
  organisms/    assets, design, layers, pages, prototype, timeline, toolbar,
                variables — the host-facing feature surfaces
  layouts/      pseudo_editor — composition shells that arrange organisms
```

Organism and layout modules are re-exported at the crate root, so hosts import
`fanta_gpui::pages`, never a tier path. The atoms and molecules tiers are
curated public API: hosts (and the storybook) import `fanta_gpui::atoms` and
`fanta_gpui::molecules` — or the prelude — to build custom chrome that shares
the library's activation, focus-ring, icon, and clamping contracts, while
drawing internals such as the icon stroke-path builder stay crate-private.
The storybook mirrors the taxonomy
with one screen module per story. Feature surfaces compose the shared tiers
instead of re-implementing key contexts, focus rings, activation wiring,
icons, or menu chrome per control:

- `ControlExt::on_activate` registers one handler for both pointer clicks and
  the Enter/Space `ActivateControl` command, suppressing keyboard-synthesized
  clicks so a keystroke activates exactly once. A control may keep an explicit
  action/click pair only when the pointer path needs event data the activation
  payload lacks (for example `click_count` for double-click rename); such
  sites carry a contract comment. `ButtonControlExt` mirrors the same
  convergence for `gpui_component::Button` (which owns its focus handle and
  click slot), and its `on_keyboard_activate` covers Buttons whose pointer
  path is owned by a wrapping surface such as a `Popover` trigger — a Button
  tab stop without one of these registrations is a dead control.
- `icon_button` and `list_row` own the complete §9 control recipe: `id`, the
  `FantaControl` key context, a tab stop, hover treatment, and a non-shifting
  focus ring with the border width reserved while unfocused.
- `menu_surface`, `menu_item`, and `clamp_menu_origin` own popover-menu chrome
  and two-axis window clamping. Transient surfaces must stay inside the window
  on both axes (§12 established the rule for toolbar popups; pointer-anchored
  context menus follow it through this molecule).
- `vector_icon` renders theme-colored stroke paths on a 16-unit grid and is
  the sanctioned §5 icon source, shared with the toolbar's tool/mode drawings.
  Unicode characters and hand-assembled div art are not icons.
- `track_bounds` and `truncating_label` replace hand-rolled measurement
  canvases and character-count width heuristics; labels truncate rather than
  forcing horizontal scroll extents.
- `controls::test_support` mounts any intent-emitting component behind a
  recording probe host and asserts the §9 pointer/Enter/Space parity matrix
  against a control's debug selector. Every interactive control carries a
  systematic `debug_selector` so interaction tests target controls uniformly.

New feature work must consume this layer; adding a bespoke control stanza,
menu implementation, or glyph icon to a feature module is an architecture
violation unless this section records why the shared piece cannot serve it.
