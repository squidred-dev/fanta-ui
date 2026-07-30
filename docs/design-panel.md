# Design panel

`DesignPanel` reproduces the capability branches and interaction states of
Figma's Design inspector without coupling the UI library to a Fanta document
schema. It accepts immutable inspection context and node view data, then emits
typed `DesignPanelAction` intents.

## State ownership

The host owns selection, parent-layout context, permissions, committed property
values, mixed/bound/read-only states, collections, document mutations, undo,
and export. The component owns only scroll/focus continuity, collapsed
sections, select-menu state, uncommitted input drafts, and the currently open
paint picker or grouped Frame-preset chooser. Style-resource search text,
This-page/Library source filtering, and List/Grid presentation are likewise
transient inspector state; they never filter or rewrite the host's catalog.
The host also owns user-level inspector preferences that outlive a selection
or file. The panel retains only the most recently supplied presentation value.
Right-sidebar navigation is controlled independently from document data: the
host accepts a surface request, then echoes the active surface through the
panel setter.

```rust
let panel = cx.new(|cx| {
    DesignPanel::new(
        "inspector",
        DesignPanelNode::new("node-1", "Card", DesignPanelNodeKind::Frame),
        window,
        cx,
    )
});

cx.subscribe(&panel, |host, panel, action: &DesignPanelAction, cx| {
    host.apply_design_intent(action);
    panel.update(cx, |panel, cx| {
        panel.set_node(host.current_design_read_model(), cx);
    });
});
```

Use `DesignPanel::new_with_context` when the initial surface is a page,
multiple selection, viewer context, text edit, or vector edit. Existing panels
can switch context with `set_inspection_context` and can supply exact
`Unset`/`Uniform`/`Mixed`/`Bound`/`ReadOnly` leaf states through
`set_property_value_states`.

Continuous numeric and text edits emit `Begin`, zero or more `Preview`, and
exactly one `Commit` or `Cancel` phase. The panel never applies those values to
its node snapshot; the host echoes accepted view data back. A `Cancel` payload
is not required to repeat the original value: hosts snapshot the complete
controlled target at `Begin` and restore that snapshot at `Cancel`. The
Storybook mock follows this rule for typography, generic properties, transform
modifiers, component and Slot properties, effects, paints, TextPath/vector
edits, selection colors, media scrubbing, and static/animated export settings.
Uniform numeric readouts are also horizontally scrubbable. The gesture waits
for a small movement threshold before emitting `Begin`, sends deduplicated
`Preview` values while dragging, and commits once on pointer release. Shift
uses a 10× coarse delta and Alt/Option uses a 0.1× fine delta; combining them
returns to the normal delta. Pointer Y selects the documented `2x`, `1x`,
`1/2`, and `1/4` speed bands and a transient status chip mirrors the current
speed with a proportionally wide cursor cue. Figma documents the four speeds
and their vertical direction, but not pixel distances, so the panel uses
deterministic displacement thresholds of −80px, +80px, and +160px. The same
rules apply to variable-font-axis value scrubbing. Mixed, unset, bound,
read-only, and viewer values never start a scrub. See Figma's
[numeric-field adjustment guide](https://help.figma.com/hc/en-us/articles/360039956914-Adjust-alignment-rotation-position-and-dimensions).

### Design, Prototype, Comment, and Properties surfaces

`DesignPanelSurface` is one neutral public surface model with two exhaustive
permission sets: editable contexts expose `Design` and `Prototype`; viewer
contexts expose `Comment` and `Properties`. `available_surfaces()` reports the
current set and `active_surface()` reports the last host echo for that set.
Editor surfaces default to `Design`; viewer surfaces default to `Properties`.

The tabs are keyboard-reachable Buttons. Pointer, Enter, and Space activation
all emit the non-document
`SurfaceChangeRequested { current, requested }` intent. The panel does not
optimistically select the requested tab. A host should reject a stale
`current`, reject a `requested` surface outside the current permission set,
then call `set_active_surface(requested, cx)`. The setter performs the same
permission validation and returns whether the echo was accepted.

The reusable panel owns the complete Design and Properties projections. On
Prototype or Comment it retains the common surface/selection header and
renders an explicit host-owned empty projection; it never leaves Design
controls or viewer Properties readouts mounted under the wrong active tab.
The Storybook is the reference adapter for all four surfaces and demonstrates
stale and cross-permission rejection without touching its mock document.

This contract follows Figma's current
[right-sidebar surface model](https://help.figma.com/hc/en-us/articles/360039832014-Design-prototype-and-explore-layer-properties-in-the-right-sidebar).

### Design and Draw workspaces

`DesignPanelWorkspaceMode` is a separate host-controlled presentation axis
with `Design` and `Draw` values. It is not a fifth `DesignPanelSurface`, and it
is not a `DesignPanelEditMode`. A new panel defaults to `Design`.
`workspace_mode()` returns the current echo and
`set_workspace_mode(mode, cx)` changes it without emitting a document or
surface action.

Entering Draw preserves the host's accepted editable surface. For example, if
the saved surface is `Prototype`, Draw temporarily mounts its own inspector;
switching the workspace back to Design reveals the still-controlled Prototype
projection. Viewer contexts continue to use Comment/Properties and never
mount editable Draw controls. A workspace transition balances active phased
edits with `Cancel`, dismisses incompatible transient editors, resets the
projection scroll position, and preserves the controlled node, selection,
edit mode, section disclosures, and accepted surface.

Draw uses a deliberately conservative filter over the already-resolved
capability list. It keeps Position, Layout, Appearance, Transform, Mask,
Typography, Fill, Stroke, Effects, Export, and any explicitly supplied legacy
Geometry section. Position is mounted immediately below the node header without
a second section heading; its inline constraints remain available.
Component/Instance, Selection colors, Section, the legacy Media section, a
duplicate Constraints section, and Layout guides are suppressed. Layout moves
Resize to fit and Add auto layout into its header and labels the
None/Horizontal/Vertical/Grid selector **Flow**.

Appearance follows Draw's slider-first right sidebar: each property label has
its own line, followed by one row containing the numeric field, slider rail,
and the trailing visibility or independent-corners action. Opacity always uses
the fixed `0..=100` percentage range. Figma does not publish one universal
corner-radius slider maximum, so the host supplies a validated range for the
exact ordered selection:

```rust
let target = DesignPanelTarget::Nodes {
    node_ids: vec!["frame-1".into()],
};
let range = DesignDrawSliderRange::new(0., 320., 1.)
    .expect("finite increasing range with a positive step");
panel.update(cx, |panel, cx| {
    assert!(panel.set_draw_appearance_view_data(
        DesignDrawAppearanceViewData::new(target, range),
        cx,
    ));
    panel.set_workspace_mode(DesignPanelWorkspaceMode::Draw, cx);
});
```

Polygon Count, Star Count/Ratio, and conditional Ellipse Start/Sweep/Ratio rows
remain inside this Appearance section in both Design and Draw. A full ellipse
omits only those Arc rows.

Page, empty, duplicate, and blank targets are rejected. A valid but stale
target may remain cached, but its corner slider does not render or emit until
that exact ordered selection is current again. For a multiple selection the
range is usable only when the same host snapshot supplies finite, resolved
`Uniform` (or bound/read-only uniform) Width and Height states. Mixed or unset
geometry omits the corner slider even if target-bound range data is cached;
the panel never treats member one as the aggregate geometry. `[A, B]` and
`[B, A]` are distinct targets. Slider pointer gestures and arrow steps use the
same controlled `PropertyEditRequested`
`Begin`/`Preview`/`Commit`/`Cancel` contract as numeric fields; bound, mixed,
read-only, stale, and viewer states never consume an arrow key or begin a
transaction.

The Storybook's **Inspector workspace · Design / Draw** buttons demonstrate
the projection independently from the surface controls. Switching the editor
toolbar to Draw also echoes Draw into the inspector adapter.
`FANTA_DESIGN_WORKSPACE=draw` opens the Design story directly in this state
(and `FANTA_TOOLBAR_MODE=draw` is used as a fallback). Its adapter derives a
corner range for multiple selection only when every selected Width and Height
is exactly uniform; otherwise it supplies no projection.

This presentation follows Figma's current
[Figma Draw overview](https://help.figma.com/hc/en-us/articles/31440394517143-Explore-Figma-Draw),
including its streamlined slider-based right sidebar.

### Additional labels preference

`set_additional_labels(true, cx)` enables UI3's **Additional labels**
preference (formerly **Property labels**); `additional_labels()` reports the
current presentation value. A new panel defaults to `false`. The flag survives
node and inspection-context changes, adds short explanatory text to shared
numeric, icon-led, and select-backed compact controls, and truncates inside
the same row at the supported 320, 400, and 472px inspector widths.

The setter emits no `DesignPanelAction` and never changes a property value.
The application host is responsible for persisting and broadcasting the
cross-file user preference. The Storybook's keyboard-reachable
**Additional labels · On/Off** button demonstrates the adapter, and
`FANTA_DESIGN_ADDITIONAL_LABELS=1` enables it at launch for deterministic
visual QA.

This ownership follows Figma's current documentation: preferences apply
[across files](https://help.figma.com/hc/en-us/articles/30928303842327-FD4B-Set-your-file-preferences),
labels add context to
[right-sidebar settings](https://help.figma.com/hc/en-us/articles/360041065034-Adjust-your-zoom-and-view-options),
and the UI3 navigation rollout renames Property labels to
[Additional labels](https://help.figma.com/hc/en-us/articles/360039831974-View-layers-and-assets-in-the-Layers-Panel).

### Small and big nudge preference

`DesignNudgeSettings` is the validated host value for Figma's cross-file
Small/Big nudge preference. `DesignNudgeSettings::default()` is `1 / 10`;
`new(small, big)` rejects zero, negative, NaN, and infinite amounts.
`set_nudge_settings(settings, cx)` updates presentation/interaction policy
without emitting a document action, and `nudge_settings()` returns the current
echo.

Up/Down uses Small and Shift+Up/Down uses Big for every editable scalar numeric
inspector draft: ordinary numbers, optional numbers only when present, angles,
percentages, numeric layout-guide counts, static-export sizing, numeric Shader
leaves, and variable-font axes. Export `x`/`w`/`h` discriminators, Shader leaf
identity, guide/configuration IDs, clamps, integer rounding, and semantic
applicability are preserved. Auto, None, blank Mixed/Unset, invalid
expressions, Text, Color, and number-list drafts are not consumed and never
fall back to an original value. Valid changes still travel through
Begin/Preview/Commit/Cancel, leaving the controlled node snapshot untouched.

The Storybook offers keyboard-reachable **Default · 1 / 10** and
**Custom · 0.5 / 8** preference buttons. The ownership and defaults follow
Figma's
[Small and Big Nudge preference documentation](https://help.figma.com/hc/en-us/articles/4404575206295-Set-small-and-big-nudge-values).

## Exact node capabilities

`DesignPanelNodeKind` remains a compatibility preset, not a closed document
schema. Hosts can attach `DesignPanelNodeCapabilities` to one exact node,
especially `Other` or a node type introduced after this crate version. Its
`sections` list is authoritative and ordered; the semantic flags gate both the
controls inside those sections and every corresponding property/collection
intent. When `capabilities` is `None`, all existing kind-derived presets behave
as before.

```rust
node.capabilities = Some(
    DesignPanelNodeCapabilities::for_node_kind(DesignPanelNodeKind::Other)
        .with_sections([
            DesignPanelSection::Position,
            DesignPanelSection::Layout,
            DesignPanelSection::Layer,
            DesignPanelSection::Stroke,
            DesignPanelSection::LayoutGrid,
            DesignPanelSection::Export,
        ])
        .with_fill(false)
        .with_effects(false)
        .with_constraints(true)
        .with_layout_guides(true),
);
```

This example exposes freeform constraints and Layout guides for a host-defined
node while suppressing Fill and Effects, even if stale fill/effect data remains
in the immutable snapshot. Unsupported programmatic interactions are rejected;
the host never receives an intent for a capability the same snapshot disabled.
Dimensions, auto-layout ownership, Grid ownership, resize-to-fit, clipping,
visibility, X/Y coordinates, arrange actions, rotation/flip transforms,
aspect-ratio lock, auto-layout-child participation, Add auto layout, stroke,
layer appearance, pass-through blend, and the other listed semantic flags
follow the same rule.

`Widget` is canonical rather than an `Other` fallback. Figma's
[`WidgetNode`](https://raw.githubusercontent.com/figma/plugin-typings/master/plugin-api.d.ts)
is an opaque scene node: the panel exposes editable X/Y and visibility,
inspectable read-only width/height, and Export. It omits arrange,
rotation/flip, aspect lock, auto-layout ownership/child/Add operations,
resize, clipping, fills, strokes, opacity, blend, effects, constraints, and
Layout guides. The stable Story fixture `reference-widget` provides the
read-only dimension states. `Other` remains available for an unrelated
host-authored capability snapshot.

### Add auto layout

Creating an auto-layout container is a structural operation. A Group or other
non-frame selection does not become an auto-layout owner merely because the
inspector rendered a button. Supply a target-bound projection only after the
host has evaluated the exact current structure:

```rust
let target = DesignPanelTarget::Nodes {
    node_ids: vec!["12:34".into(), "12:56".into()],
};
panel.update(cx, |panel, cx| {
    panel.set_add_auto_layout_view_data(
        DesignAddAutoLayoutViewData::eligible(target.clone()),
        cx,
    );
});
```

`structurally_eligible` controls whether the native **Add auto layout**
affordance belongs in Layout. `disabled(reason)` keeps an otherwise eligible
affordance visible and inert with a host-authored explanation. The target is
the exact ordered selection: a projection for `[A, B]` is stale for `[B, A]`,
and invalid, duplicate, empty, or Page targets remain inert. The affordance is
also omitted once the selected node is already an active auto-layout owner.

Activation emits
`DesignPanelAction::AddAutoLayoutRequested { target }`. It is never rewritten
as a legacy first-node action. The host converts an eligible Group/frame-like
node or creates a wrapping Frame for arbitrary selected layers, chooses the
initial flow, updates selection, and echoes a complete
`DesignPanelInspectionContext`. Until that echo the panel retains the original
kind, layout mode, IDs, and selection. View-only permissions and a supplied
disabled reason cannot emit.

The Storybook scenarios **Add auto layout · Group** and
**Add auto layout · Multiple** demonstrate both host echoes. Its mock reducer
validates the complete current target (including order), rejects stale targets,
converts the Group or creates a uniquely identified wrapper, and then selects
the echoed Frame. This follows Figma's current
[toggle/add auto layout documentation](https://help.figma.com/hc/en-us/articles/5731482952599-Toggle-on-auto-layout-in-designs)
and
[guide for one or more selected layers](https://help.figma.com/hc/en-us/articles/360040451373-Guide-to-auto-layout).

The Appearance header follows the same controlled rule. Its eye reflects
`DesignPanelNode::visible` and emits a typed `Visible` property request; it
does not locally hide the node. Its droplet opens the blend menu. Structural
containers include `Pass through`; leaf nodes expose only the 18 compositing
modes from Normal through Luminosity. Opacity and corner radius remain on the
compact shared row, with nonuniform radii and smoothing behind the transient
advanced-corner action.

## Page/no-selection and variable modes

Page inspection uses dedicated host data instead of adapting a fake scene
node. Supply `DesignPageViewData` whenever the inspection context has no
selection:

```rust
panel.update(cx, |panel, cx| {
    panel.set_page_view_data(
        DesignPageViewData::canonical(
            "page-1",
            DesignPageBackground::new(DesignColor::rgb(0xf5, 0xf5, 0xf5)),
        ),
        cx,
    );
    panel.set_page_local_styles_view_data(
        DesignPageLocalStylesViewData::for_page("page-1", local_style_sections),
        cx,
    );
});
```

`DesignPageBackground` represents the single solid paint exposed by Figma's
`PageNode.backgrounds`. Its swatch opens the shared retained solid-color
editor, including HSV, hue, alpha, Hex/RGB/CSS/HSL/HSB, keyboard,
outside-dismiss, and Escape behavior. Editing emits
`PageBackgroundEditRequested { page_id, color, phase }`; Begin/Preview/Commit/
Cancel remain host-owned and never mutate the supplied color. The older
commit-only `PageBackgroundChangeRequested` is a compatibility adapter.

The canonical **Local styles** surface is current-file-only.
`DesignPageLocalStylesViewData` binds an exact Page target to ordered
Text, Color, Effect, and Layout-guide sections. Sections may contain nested
`DesignLocalStyleEntry::Folder` trees and style leaves with host-authored
typography, ordered Paint, Effect, or Layout-guide previews. Folder and style
IDs must be nonempty and globally unique, section kinds must be unique, and a
leaf's preview kind must match its section. Invalid, scene-node-targeted, or
stale Page projections do not render or emit.

Every leaf intent carries `DesignLocalStyleTarget { page_id, kind, style_id,
parent_folder_id, expected_index }`. Edit, Go to definition, Copy, and
Duplicate use `LocalStyleCommandRequested`; create, folder-create, atomic
delete, and move have dedicated intents. Move additionally carries a
`DesignLocalStyleInsertion` with the exact parent, index, and both expected
neighbor IDs. The panel re-resolves all identities, direct parents, indices,
neighbors, disabled ancestor folders, permissions, and the current Page before
emitting. Viewers can inspect and go to definitions; Copy follows copy
permission, while edit, duplicate, create, delete, and move require edit
permission. A style can be dragged before another style, to the end of a
folder, or to the end of its family root; each drop is converted to the same
neighbor-validated move intent. The compact up/down buttons exercise the same
boundary in the Storybook. Folder disclosure is transient presentation state
and is pruned when a host echo removes the stable folder.

Figma's canonical Variables entry point is outside the right sidebar, so
`DesignVariablesEntryPoint::NavigationBarOnly` is the default and no “Local
variables” row is rendered. A host can explicitly opt into
`LegacyRightSidebar` to show one **Open variables** navigation action; it emits
only `VariablesViewOpenRequested` and never opens an import browser.

`DesignLocalResourceCategory`, `DesignLocalResourceKind`,
`DesignLocalResourceAvailability`, `DesignLocalResource`,
`DesignLocalResourceSelection`, `DesignLocalResourceGroup`,
`DesignLocalResourceViewData`, and the `LocalResource*Requested` actions remain
deprecated compatibility contracts. Their former Browse/import UI is not part
of the canonical Page projection. `DesignLocalResourceSource` remains active
because variable-mode collections still use it.

Explicit variable modes use a separate
`DesignVariableModeViewData { target, collections }`. The exact target must
match the current Page or single selected scene node. Each collection carries
its default mode, current resolved mode, and optional explicit mode. These are
not interchangeable: a resolved mode can be inherited from an ancestor, while
an explicit mode is stored on this exact target. The anchored diamond browser
appears in the Page header or scene-node Appearance header and labels that
distinction as `Resolved` or `Explicit`.

Choosing a mode emits `VariableModeApplyRequested`; choosing “Use inherited
mode” emits `VariableModeClearRequested` with the exact explicit mode ID that
the host last supplied. The latter lets a reducer reject stale clears after a
concurrent echo. Viewers can inspect the browser but cannot apply or clear a
mode, disabled collection/mode reasons are preserved, and the panel never
updates resolved or explicit mode state locally. The Storybook seeds Page and
per-node mode fixtures and echoes accepted actions through the same setters.

## Selected-node header

The 48px selected-node header is controlled independently from the node
property snapshot. Supply `DesignSelectionHeaderViewData` with its display
title, optional title menu, ordered primary controls, and ordered overflow
controls:

```rust
panel.update(cx, |panel, cx| {
    panel.set_selection_header_view_data_for_target(
        DesignPanelTarget::Nodes {
            node_ids: vec!["12:34".into(), "12:56".into()],
        },
        DesignSelectionHeaderViewData::for_multiple_selection(2),
        cx,
    );
});
```

The presets are Storybook/compatibility baselines only. A single selection may
use `for_node_kind`. A multiple selection must use
`for_multiple_selection(count)` or exact host data; it never borrows a first
member's kind. The aggregate fallback has the count title, no title menu, and
only the universally applicable Create component command, so a mixed
Text/Ellipse selection cannot expose Text-only commands. Real availability is
contextual: a host may omit Select matching layers when no matching sibling
exists, reorder any control, add stable host-defined commands, and attach a
disabled reason to a control or menu item. `with_icon` also lets host-defined
controls carry an exact named icon or compact plugin glyph instead of the
fallback ellipsis. A host-defined control with `with_menu_items(...)` is a real
anchored menu; its trigger never emits the control ID as a direct command.
Supplied data is authoritative; the panel never appends inferred controls.
The setter binds it to the current target's exact ordered node IDs, so changing
`[A, B]` to `[A, C]` or changing the selection cardinality invalidates the
header until the host supplies fresh view data. An activation retained from an
older render is rejected if its bound target no longer matches.
`set_selection_header_view_data(view_data, cx)` remains as a synchronous
compatibility convenience that binds to the selection active at call time;
asynchronous adapters should always use the explicit-target setter.

Direct controls and every title/Boolean/Flatten/overflow leaf emit
`DesignPanelAction::SelectionHeaderCommandRequested { target, command }`.
Targets preserve the complete selection. The host performs the command and
echoes fresh inspection/header data. Only the open title, Boolean/Flatten, or
More popover is transient panel state. More is rendered only when the host
supplies overflow controls, and it is never a document command itself.
View-only contexts keep non-mutating Select matching layers available while
gating edit commands; host-disabled controls remain disabled with their
supplied reason. Built-in access semantics are the default. Hosts can mark a
custom direct control, title-menu item, or menu leaf as safe for viewers with
`.viewer_safe()` (or set `DesignSelectionHeaderCommandAccess` explicitly);
menu leaves are classified independently. Built-in mutating commands retain
their edit-required classification even if a host accidentally applies an
override. All triggers and menu leaves remain keyboard reachable.

## Node coverage

| Selection | Inspector-specific content |
| --- | --- |
| Frame | Freeform/auto-layout/grid, constraints, clipping, fills, strokes, effects, layout grids |
| Group | Position with contextual freeform constraints, resize-to-fit, layer appearance, effects, and export |
| Transform group | Typed linear horizontal/vertical or radial repeat modifiers, count, relative/pixel offset, apply, contextual freeform constraints, resize-to-fit, and group-like controls without unsupported paints |
| Section | X/Y, size/aspect lock, Appearance visibility, fill, uniform stroke weight, radius/smoothing, contents visibility, share, and Dev status; no rotation/flip, auto-layout participation/application, individual stroke sides, opacity, blend, or effects |
| Component | Typed Boolean/Text/Instance swap/Slot property definitions plus frame/layout capabilities, descriptions, and docs |
| Component set | Typed variant definitions/options plus frame/layout capabilities |
| Instance | Resolved main/library identity, availability, overrides, reset/go-to-main/detach, and typed property values |
| Slot | Slot-definition or slot-instance properties, limits/violations/actions, frame-like layout, corner, paint, stroke, and layout-guide controls |
| Widget | Opaque-node X/Y and visibility, read-only width/height, conservative selected-node header, and Export; no paint, appearance-transform, resize, constraints, or auto-layout controls |
| Text | Host-controlled searchable font family/style browser; unit-aware line height, tracking, leading trim, indentation, `listSpacing`, complete decoration details, and lossless OpenType feature records; resize, truncation, and maximum lines |
| Text path | Text and typography controls, typed start segment/position, host-controlled vertex sub-selection, plus path-compatible fill and stroke controls |
| Rectangle | Independent radii, solid/pattern/image/video/gradient fills, stroke, effects |
| Ellipse | Conditional Appearance Start/Sweep/Ratio controls, displayed as degrees and percent |
| Polygon / star | Appearance Count, Star Ratio, radius, and corner smoothing |
| Line | Weight, dashes, and start/end caps, including arrow caps |
| Vector / boolean | Vector fills, strokes, caps, stable-ID vertex coordinates/radii/handle mirroring in vector edit; Boolean/Flatten remains a selected-node header menu |
| Any mask-capable scene node | Orthogonal `is_mask` state plus a dedicated Mask section with typed Alpha/Vector/Luminance mode |
| Slice | Position, size, and export settings |
| Multiple-selection context | Alignment, exact-target Smart Selection spacing/distribution/tidy controls, and canonical Selection colors aggregate |
| Other | Kind-derived safe fallback, or an exact host-authored ordered section/capability snapshot |

Image and Video are Fill paint payloads, not Design-node kinds. Pen and Pencil
are toolbar creation tools that yield Vector nodes. An arrow is a Line/Vector
with an arrow stroke cap, and a mask is an orthogonal property of the
underlying scene node. Multiple selection comes from
`DesignPanelInspectionContext`, while Table is FigJam-only. The corresponding
early `DesignPanelNodeKind` variants remain in
`DesignPanelNodeKind::COMPATIBILITY_ALL` for source migration, but preset UIs
iterate the canonical `DesignPanelNodeKind::ALL`.

The Storybook keeps that canonical matrix and its existing fixture identities
intact (including the dedicated `add-auto-layout-group` ID), adds the stable
canonical `reference-widget` exact fixture without renumbering prior IDs, then
appends an explicit migration-fixture matrix so the selector can also audit
every compatibility alias. Existing `reference-image`, `reference-video`, and
`reference-arrow` fixtures are joined by stable `compatibility-mask`,
`compatibility-table`, `compatibility-pen`, and `compatibility-pencil`
fixtures. This does not promote those aliases into `ALL`. `MultipleSelection`
is deliberately the one compatibility value with no node fixture: the
**Editable multiple** inspection scenario supplies a real ordered multi-node
context instead.

The story also exposes the full permission/cardinality matrix for page/no
selection, single selection, and multiple selection: editable, view-only, and
restricted. Canvas-parent singles, the complete Smart Selection kind/read-only
matrix, variable/style-bound plus read-only property matrices, in-flow/ignored/
vertical/grid auto-layout children, and text/vector edit contexts are separate
deterministic scenarios. Only a single-node viewer target receives the native
UI3 Properties projection. Viewer page scenarios use the Page controls, while
viewer multiple scenarios deliberately render the aggregate/empty viewer state;
they never borrow the first selected node's Text Content, copy payloads, or
color-representation state.
Whole-collection Paint, Text, Effect, and Layout-guide style bindings render
only the supplied style name and style icon instead of a disabled stack of
style-owned leaves. A Stroke Paint style hides only its paint stack; stroke
weight, alignment, dashes, caps, joins, and brush geometry remain visible
because those are not owned by the Paint style. Text-path placement likewise
remains visible beside a bound Text style. The header style browser remains
the controlled replace/detach surface.
The multiple-selection scenario demonstrates mixed width, unset height, and a
host-computed Selection colors aggregate. It intersects the selected nodes'
capabilities and clears first-node typography, component, geometry, indexed
collection, and style-binding data that has no safe aggregate identity.
Uniform/Mixed/Unset leaf states are supplied through
`set_property_value_states`; a Bound state is supplied only when every
selected target has the identical binding. The panel does not infer these
states from the first node or rewrite them after emitting an edit.

The separate **Homogeneous multiple · Common Fill and Stroke** scenario
demonstrates Figma's complementary branch: semantically identical complete
paint collections remain under Fill and Stroke even when each node uses
different host-local paint and gradient-stop IDs. Those common paints are not
duplicated under Selection colors. A paint edit carries the exact ordered
multi-node target; the Storybook host resolves the representative row by
stable identity, preflights every member and transaction phase, and only then
updates all nodes. A reordered, missing, bound, read-only, or no-longer-common
member rejects the complete edit without partial writes.

For deterministic visual QA, launch the Design story directly:

```sh
FANTA_STORYBOOK_STORY=design cargo run -p fanta-gpui-storybook
```

Select a deterministic fixture at launch by stable ID, display name, or node
kind label:

```sh
FANTA_STORYBOOK_STORY=design \
FANTA_DESIGN_NODE=reference-ellipse \
FANTA_DESIGN_SCENARIO=canvas \
cargo run -p fanta-gpui-storybook
```

The permission/cardinality launch values are `page`, `view-only-page`,
`restricted-page`, `editable-single`, `view-only-single`,
`restricted-single`, `editable-multiple`, `view-only-multiple`, and
`restricted-multiple`. These are also the stable IDs used by the scenario
selector; `viewer-page`, `viewer-single`, and `viewer-multiple` remain concise
launch aliases.

Use `FANTA_DESIGN_EXPORT_MODE=animated` to open the Motion export surface.
`reference-frame` is eligible, `layout-horizontal` demonstrates the Starter
60-FPS gate, `layout-vertical` carries a nested-frame reason, and
`reference-arrow` exercises host-described animated SVG options.
The Design Story’s inspector shell is continuously resizable from its left
divider between 320 and 640px. Drag the divider, or focus it with Tab and use
the arrow or `+`/`-` keys; Shift changes the keyboard step from 8 to 32px. The
current width is shown beside the retained 320/400/472 preset Buttons. Set
`FANTA_DESIGN_PANEL_WIDTH` to any numeric width in the same range to choose the
launch width; out-of-range values are clamped and invalid values fall back to
472. Anchored style, variable, typography, paint, and effect inspectors keep
their natural content width and use the row's top-right corner, so they open
inward and remain fully visible at the deterministic preset widths.

The truncation matrix uses
`text-resize-autowidth`, `text-resize-autoheight`, `text-resize-fixed`,
`text-max-lines-vertical-hug`, `text-max-lines-vertical-fixed`, and
`text-max-lines-max-height`. The two vertical-child fixtures automatically
select the vertical auto-layout-parent scenario unless
`FANTA_DESIGN_SCENARIO` explicitly overrides it.

## Position and dimensions

The UI3 composition keeps alignment, X/Y, constraints, and transforms in
Position while the universal W/H surface lives in Layout. Position provides
six typed alignment commands, horizontal/vertical distribution and tidy-up
for aggregate selections, an inline constraints disclosure when the selected
layer is a freeform frame child, normalized signed rotation, flip-horizontal,
flip-vertical, and rotate-clockwise-90 commands. Layout then provides W/H,
aspect lock, resize-to-fit when the node supports it, and contextual
fixed/hug/fill sizing for auto-layout children.

Aspect ratio remains host-controlled. The panel emits the changed Width,
Height, or lock leaf and waits for the host to echo both dimensions; the
Storybook host demonstrates Figma's proportional W/H behavior and mirrors a
locked min/max value onto the opposite axis. Set
`DesignPanelNode::is_component_instance_child` for a layer nested below a
component instance. Its W/H fields remain available, but the aspect-ratio
setting is omitted and a stale lock intent is rejected, matching Figma's
[lock-aspect-ratio rules](https://help.figma.com/hc/en-us/articles/360039956914-Adjust-alignment-dimensions-rotation-and-position).

### Smart Selection

Smart Selection is an exact host projection for a multiple selection, not a
property inferred from the aggregate node. Supply
`DesignSmartSelectionViewData` with
`set_smart_selection_view_data(view_data, cx)`. Its
`DesignSmartSelectionKind` is `None`, `Horizontal`, `Vertical`, or
`TwoDimensional`; the latter three determine which native **Space between**
fields appear. Each axis independently carries a `Uniform(f32)` or `Mixed`
readout, field availability, and an optional disabled reason. Arrange
eligibility is similarly explicit for horizontal distribution, vertical
distribution, and Tidy up, so disabled native controls remain visible with the
host's explanation.

The projection target must be `Nodes` with at least two nonempty, unique IDs,
and order is identity: data for `[A, B]` is stale for `[B, A]`. Invalid or
late projections render no Smart Selection controls and cannot fall through
to a first-node property or legacy arrange action. A top-level
`read_only(reason)` keeps the supplied spacing values visible while making
every field and command inert. Viewer permission is an additional gate.

Spacing editors emit
`SmartSelectionSpacingEditRequested { target, axis, value, phase }` with
`Begin`, zero or more `Preview` values, and one `Commit` or `Cancel`.
Distribution and tidy activation emit
`SmartSelectionArrangeRequested { target, operation }`. The panel never
changes spacing, selection order, or layer position itself. It cancels an
active spacing draft against its captured target when selection, permission,
or the bound host projection changes, then displays only the next echoed
snapshot.

The Storybook includes **Smart selection · None**, **Horizontal**,
**Vertical**, **2D**, and **Read only** scenarios. The 2D scenario demonstrates
independent horizontal and vertical values, including a mixed axis; the mock
host validates exact ordering, transaction sequence, access, and operation
availability before accepting an intent. Launch one directly with, for
example, `FANTA_DESIGN_SCENARIO=smart-selection-2d`.

This follows Figma's current
[Smart Selection and Tidy up behavior](https://help.figma.com/hc/en-us/articles/360040450233-Arrange-layers-with-Smart-selection):
equal spacing and axis overlap determine one- or two-dimensional Smart
Selection, while Tidy up can establish common spacing for an eligible
selection.

### Width/Height min and max

Minimum and maximum dimensions follow Figma's Width/Height disclosure instead
of an always-visible “Minimum and maximum” block. The menus are available only
for an active auto-layout frame or a participating direct child. Freeform
layers and children using Ignore auto layout omit the controls.

Each axis menu contains its valid Fixed/Hug/Fill choices plus separate
`Add min width|height` and `Add max width|height` rows. Add is presentation
only: it reveals and focuses the corresponding empty numeric field, whose
normal phased editor then emits `MinWidth`, `MaxWidth`, `MinHeight`, or
`MaxHeight` with `OptionalNumber(Some(value))`. The panel does not invent a
document value while opening the field. These native menus are the sole
Fixed/Hug/Fill surface in a valid auto-layout context; the panel does not
repeat them in a lower Resizing row.

When either host value exists, the axis icon gains the native two flanking
lines. Clicking that icon reopens every existing field for its axis. Hovering
it emits
`DimensionLimitsPreviewRequested { node_id, axis, minimum, maximum, preview }`
with the exact captured host values on both entry and exit, allowing the host
to draw and safely unwind the canvas bounds preview.

`Remove min and max` emits `OptionalNumber(None)` independently for each
existing editable leaf. Viewer permissions, read-only states, and bindings
gate min and max separately; for example, a variable-bound minimum cannot be
silently cleared when an editable maximum is removed. Variable buttons remain
attached to the disclosed leaf through the existing exact
`minWidth`/`maxWidth`/`minHeight`/`maxHeight` targets.

Disclosure and menu state are transient. Same-node host echoes preserve open
fields, while deselect/reselect, a target or permission change, or an invalid
layout context hides them. A host echo that changes values during hover first
receives the matching preview cancel for the captured old values. The
Storybook includes an owner with all four limits, horizontal and Grid children
with partial limits, and an ignored child whose stale limit data stays hidden.
This matches Figma's current
[minimum and maximum dimensions guidance](https://help.figma.com/hc/en-us/articles/360040451373-Explore-auto-layout-properties).

Arrange, transform, resize, selection-color, and export commands carry a typed
`DesignPanelTarget` directly. Ordinary property, typography, component,
collection, paint, effect, style, and variable actions keep their compact
single-node form for one selected node. In a multiple selection the panel
wraps the leaf in
`TargetedNodeActionRequested { target: DesignPanelTarget::Nodes { node_ids }, action }`.
The ordered `node_ids` are authoritative; the nested leaf's `node_id` is a
compatibility hint only. Hosts should commit the envelope atomically to every
listed ID. The Storybook requires the envelope to equal the current target in
the same order, rejects empty or duplicate IDs and non-editor permissions,
resolves every ID, and preflights the leaf against every real node before
replaying its single-node mock reducer. Its compact mock deliberately rejects
multi-node leaf kinds whose sub-resource or shared-catalog side effects cannot
be preflighted; it never claims that a sequential partial replay was atomic.
This keeps reordered, missing, permission-invalid, and per-node-inapplicable
requests from changing an earlier member.

Direct Transform commands use the same exact-target gate. Rotate mutates every
selected mock node only after all members expose Position. Horizontal and
vertical flip commands are validated but recorded as accepted commands because
the compact Storybook node model has no reflected flip state.

### Grouped Frame presets

Figma exposes Frame presets in ordered Phone, Tablet, Desktop, Presentation,
Watch, Paper, Social Media, Figma Community, and Archive groups, and also lets
an existing Frame change presets from the right sidebar. The library does not
freeze that changing device catalog. A host supplies
`DesignFramePresetViewData { target_node_id, groups, availability }` through
`set_frame_preset_view_data`.

Each group and preset retains its exact host label and stable ID. A preset also
retains its exact positive width/height. Catalog, group, and leaf availability
are independent and can carry host-authored disabled reasons; disabled options
remain visible and explain why they cannot be applied. Preset IDs need only be
unique within their group because `DesignFramePresetSelection` always returns
the stable `(group_id, preset_id)` pair.

Group disclosure is transient and keyboard reachable. A host echo for the same
target preserves collapsed groups by stable group ID; removed groups are
pruned, and changing the target clears that disclosure state.

The catalog is bound to one selected Frame ID. A late catalog for another
selection cannot render or emit, and rectangle/component/multiple/page
contexts do not adapt it. Choosing an enabled row emits
`FramePresetApplyRequested { node_id, selection, width, height }`. The
dimensions are the immutable values displayed in that row; a reducer
re-resolves the identity against its latest catalog and rejects mismatched
dimensions or newly disabled entries before resizing. The panel does not
change W/H locally, viewers and read-only dimensions cannot apply a row, and
Escape/outside click only closes the transient chooser.

The Storybook supplies every current first-party group plus enabled,
leaf-disabled, and group-disabled examples, and revalidates stable identity in
its mock reducer. This follows Figma's documented
[Frame-preset workflow](https://help.figma.com/hc/en-us/articles/360041539473-Frames-in-Figma-Design)
while allowing labels and dimensions to evolve without a library release.

## Native shape appearance, compatibility sections, and transform modifiers

Shape-specific data is one mutually exclusive `DesignShapeGeometry` payload.
Polygon and Star counts are host-provided and clamped to the native `3..=60`
range; Star inner radius is a canonical `0..=1` ratio. `DesignArcData` mirrors
Figma's API: starting and ending angles are radians and inner radius is a
`0..=1` ratio. Figma's Appearance UI exposes Start, Sweep, and Ratio, so
`ArcSweep` reads `ending_angle - starting_angle`; a host accepts it by setting
`ending_angle = starting_angle + sweep`. Changing Start preserves the current
sweep. The panel displays degrees and percentages, then emits `AngleRadians`
and `Ratio` values without storing display units in the read model. A default
full ellipse retains Appearance but omits its Arc rows; a partial arc or
nonzero inner radius reveals Start/Sweep/Ratio in both Design and Draw.

Polygon Count and Star Count/Ratio are likewise composed into Appearance.
Boolean operations are typed but render canonically through the selected-node
Boolean/Flatten header menu. Public `DesignPanelSection::Geometry`,
`ArcEndingAngle`, and the Boolean property editor remain compatibility paths
for an exact host capability snapshot that explicitly requests Geometry.
FigJam Table row and column counts also use that read-only compatibility
section and never emit structure mutations. Canonical Figma Design resolution
does not add a Geometry section and suppresses duplicate legacy rows when
Appearance is also present.

Corner controls are capability-driven rather than inferred solely from a node
label. `DesignCornerCapabilities` independently declares uniform radius,
four-corner radii, and smoothing. This lets a host describe the actual topology
of Polygon, Star, Boolean, or closed-vector geometry and suppress controls for
an incompatible path.

Section uses a native `DesignSectionProperties` record for contents-hidden and
Dev status, including optional description and the UI-only Changed indicator.
Capabilities gate Share, status editing, Completed, and resolving Changed.
Section still exposes fills, strokes, radius, and smoothing, but the resolver
and editor deliberately omit rotation/flip, opacity, blend mode, and effects
because Figma's Section node supports scene visibility and resizing without
implementing the rotation/rescale or layer-appearance mixins. A hostile parent
context cannot make a Section render child sizing/positioning/limit controls,
and an eligible-looking host projection cannot offer **Add auto layout**.
Share and Changed resolution emit dedicated intents; both revalidate the exact
current node, feature capability, and Changed state before emission. Status
and contents visibility remain typed, host-controlled properties. The
Storybook reducer repeats that validation against its latest controlled
snapshot, including edit permission for resolving Changed, and logs rejected
stale or already-resolved intents without mutating mock state.

Transform group accepts an ordered `Vec<DesignRepeatModifier>`. Each modifier
has stable host identity and is either `Linear(Horizontal|Vertical)` or
`Radial`, with a positive count, `Relative` or `Pixels` unit, and offset.
Adding, removing, editing, and applying modifiers emit dedicated typed intents.
Modifier edits retain stable ID, compatibility index, exact discriminated
change, and `Begin`/`Preview`/`Commit`/`Cancel` phase; the panel never updates
the modifier array itself. Retained callbacks revalidate edit permission,
node identity, modifier ID/index, candidate range, and current non-empty state,
so a reorder or permission host echo makes a stale action inert. The Storybook
mock reducer performs the same preflight before mutation and records a
rejection for stale, disabled, viewer-only, invalid-count, or non-finite-offset
intents. Group and Transform-group nodes also expose
freeform-child constraints when the inspection context makes those constraints
meaningful; both expose resize-to-fit, including the repeated result for a
Transform group.

## Auto-layout variations

The story includes freeform, horizontal, vertical, wrapped horizontal, and grid
presets, plus selected-child contexts inside horizontal and grid parents.
Auto-layout controls cover:

- direction and grid mode;
- 3×3 alignment;
- fixed or auto primary-axis spacing; wrapped-track Auto/Space-between
  distribution; and linked (`null`) or positive custom counter-axis spacing;
- fixed, hug, and fill sizing;
- default vertical/horizontal padding, a four-sided editor, and uniform or
  one-to-four-value CSS shorthand input;
- horizontal wrapping, clipping, stroke inclusion, stacking order, and
  baseline alignment;
- transient Width/Height min/max menus and disclosures for auto-layout owners
  and in-flow children, plus in-flow/absolute positioning,
  align-self/stretch, and layout grow for selected children;
- fixed/fraction/hug grid tracks, a retained 2D Grid picker plus explicit
  positive Number of columns/Number of rows inputs,
  `NONE`/`ROWS` automatic-row management, manual/row-auto-flow placement,
  row/column spans, and per-cell alignment in Position.

The 3×3 alignment target is one tab stop rather than nine. While it is focused,
Arrow keys step between targets; W/A/S/D set Top/Left/Bottom/Right directly. X
toggles packed/space-between (`Fixed`/`Auto`) primary-axis spacing, and B
toggles bounds/baseline alignment for horizontal flow. Auto spacing removes
the distributed primary axis from the target, so movement or edge shortcuts on
that hidden axis are intentionally ignored. Each valid shortcut emits the same
atomic, host-controlled property intent as its pointer control; permissions,
read-only states, and bindings gate Alignment, Spacing, and Baseline
independently.

Entering Grid through the Design UI seeds a Hug/Hug container with at least one
Hug row and column, automatic rows, row auto flow, and an explicit row gap.
The right-sidebar dimensions control follows Figma's current
[Grid auto-layout flow](https://help.figma.com/hc/en-us/articles/31289469907863-Use-the-grid-auto-layout-flow):
pointer hover previews one retained rectangular candidate, pointer click or
Enter/Space commits it, and one keyboard surface uses the Arrow keys to change
columns and rows. The adjacent Number of columns and Number of rows inputs use
the same `GridDimensionsEditRequested` contract, including phased
Begin/Preview/Commit/Cancel while typing or scrubbing. Every intent carries the
exact node ID and both positive counts atomically; the host decides track
creation/removal and child relocation, then echoes complete authoritative
track vectors. The picker never simulates a dimensions change through a lossy
sequence of per-track operations.
If a same-ID host echo temporarily reports an empty row or column vector, the
panel shows those exact raw counts in a disabled dimensions trigger. It does
not panic, invent a positive track, or emit an invalid atomic edit; the host
can complete the snapshot or recover it through the preserved per-track APIs.

Track add/delete/reorder controls emit dedicated host intents carrying the
axis and exact insertion, deletion, or `from_indices`/`insertion_index`
coordinates. The reusable panel never edits its supplied track vectors.
Deleting the final row or column is suppressed. While automatic rows are
enabled, the exact echoed row count is shown with a derived-row reason, while
the row field and row-changing picker cells are read-only because the host
derives them from children. Column-only dimension changes preserve that exact
row count. Explicit row reordering remains available. Permissions,
capabilities, leaf read-only state, and stale node identity gate the picker,
numeric inputs, keyboard commands, and phased edits uniformly.

Wrap is valid only for horizontal flow; host contexts canonicalize vertical
and grid wrap flags to no-wrap. A child using Figma's current “Ignore auto
layout” state regains freeform constraints and hides in-flow sizing,
align-self, grow, and grid-placement controls while retaining an explicit way
to rejoin the parent flow. Wrapped-track spacing is host data: `None` follows
the primary gap and `Some(value)` is a positive custom value. Space-between
track distribution suppresses custom counter-axis spacing, matching Figma's
`counterAxisAlignContent` and nullable `counterAxisSpacing` contract.

## Generic property variables

Hosts supply the generic variable catalog through
`DesignVariableViewData`. Every `DesignVariable` retains stable variable and
collection IDs/names, Page or Library source identity, resolved
Boolean/Color/Float/String type, exact Figma picker scopes, Local/Imported/
Available import state, an optional active-mode preview value, searchable
path/description/keywords, and an optional disabled reason. Imported library
variables remain Library-sourced; importing never collapses their remote
identity into a page variable.

The anchored diamond picker is searchable and groups compatible results by
page or library. Applying an imported/page variable emits
`PropertyVariableApplyRequested`; selecting an Available library variable
first emits `PropertyVariableImportRequested`; detaching the exact echoed
binding emits `PropertyVariableDetachRequested`. All three carry stable IDs
and a `DesignPropertyVariableTarget`. The panel changes no node value or
binding while waiting for the host to echo a new
`DesignPanelPropertyValueState`.

`DesignPropertyVariableTarget` maps panel properties to the exact public Figma
API fields:

| Panel property | Figma variable field(s) |
| --- | --- |
| Width / Height | `width` / `height` |
| Flow Gap | `itemSpacing` |
| Grid horizontal / vertical gap | `gridColumnGap` / `gridRowGap` |
| Wrapped-track gap | `counterAxisSpacing` |
| Vertical / horizontal / shorthand padding | top+bottom / left+right / all four padding leaves |
| Min/Max width and height | `minWidth`, `maxWidth`, `minHeight`, `maxHeight` |
| Visibility / Opacity | `visible` / `opacity` |
| Uniform radius on independent-corner nodes | all four `topLeftRadius`…`bottomLeftRadius` leaves |
| Uniform radius on other eligible nodes | `cornerRadius` |
| Individual radii | the corresponding corner leaf |
| Uniform and individual stroke weights | `strokeWeight` or the corresponding `stroke*Weight` leaf |
| Font family / style | String `fontFamily` / `fontStyle` |
| Font weight / size | Float `fontWeight` / `fontSize` |
| Line height / letter spacing | `lineHeight`, `letterSpacing` |
| Paragraph spacing / indent | `paragraphSpacing`, `paragraphIndent` |

Text targets additionally retain Whole layer versus Selected text range so a
host can choose `setBoundVariable` or `setRangeBoundVariable` without
reconstructing edit context. In Text edit mode, hosts should add the active
range generation with
`DesignPanelInspectionContext::with_text_range_revision(revision)`. Typography
and generic-variable intents then carry
`DesignTypographyTarget::SelectedTextRangeRevision(revision)`, which is the
exact host-authored range identity. Changing the revision on a same-node echo
cancels an editor transaction that began against the older range before the
new snapshot is installed. `SelectedTextRange` remains the compatibility
target for hosts that omit a revision. A Boolean variable is filtered by type;
scoped Float and String variables must also include `ALL_SCOPES` or the exact
property scope. Empty scope arrays are accepted as an unscoped host
compatibility input, while unknown future scopes round-trip as
`DesignVariableScope::Opaque`.

A bound variable disables raw value editing but can be replaced or detached.
Mixed editable selections can apply one compatible variable. Read-only
states, style-bound leaves, viewer permissions, host-disabled candidates, and
type/scope mismatches never emit mutation intents. The Storybook reducer keeps
bindings in host-owned state and echoes them through property value states;
its catalog includes page variables, imported library variables, and an
Available library radius variable to exercise the two-step import/apply flow.

## Components, instances, and slots

Component data is discriminated instead of storing every property as a string.
`DesignComponentPropertyDefinition` and `DesignComponentPropertyValue` have
matching Boolean, Text, Instance swap, Variant, and Slot variants. Definitions
retain typed defaults, variant options, instance-swap preferred component
references, slot defaults, and slot settings. Every property has a stable ID,
optional description/documentation links, override state, and reset state.
The original `kind`, `value`, and `preferred_values` fields remain compatibility
display projections for the early story adapter; new hosts should apply the
typed component actions and echo a fresh canonical value.

`DesignComponentContext` separates the six inspector roles that cannot be
inferred reliably from layer geometry:

- standalone main component;
- variant child;
- component set;
- component instance;
- slot definition in a main component or variant;
- slot instance in a component instance.

The context also retains the resolved main-component identity, whether it is
local or from a named library, `Available`/`Missing`/reasoned `Unavailable`
state, component description, documentation links, and aggregate nested and
property override counts. A missing/unavailable main disables Go to main;
Detach appears only for a regular component instance; reset actions require a
host-supplied resettable state.

Component-property variable aliases are retained on their exact Figma field,
not projected into the generic node-property binding map:

| Property role and kind | Exact alias field | Variable type |
| --- | --- | --- |
| Main/variant-child Boolean | `componentPropertyDefinitions[…].defaultValue` | Boolean |
| Main/variant-child Text or Instance swap | `componentPropertyDefinitions[…].defaultValue` | String |
| Instance Boolean | `componentProperties[…].value` | Boolean |
| Instance Text, Instance swap, or Variant | `componentProperties[…].value` | String |

Variant definition defaults and Slot values are intentionally absent because
Figma does not expose compatible bindings for them. Each
`DesignComponentPropertyVariableBinding` preserves the variable ID, display
name, resolved value, and detach capability even when that variable is no
longer present in the browser. Apply, import, and detach intents carry a
`DesignComponentPropertyVariableTarget`, so the host can reject a stale field
or type without recovering it from a row index. A bound value disables raw
editing but leaves its variable browser available for replace/detach.

Instance-swap rows use an anchored, searchable all-components browser rather
than cycling preferred values. `DesignComponentSwapCandidate` preserves the
component ID, library key, `COMPONENT` versus `COMPONENT_SET` identity, stable
page/library source, Local/Imported/Available state, search metadata, and any
disabled reason. Preferred components form a first group while every other
matching page or library result remains discoverable. Available library rows
emit import first and require a host echo before apply. Hover emits controlled
preview/clear intents, and apply carries the exact stable selection or `None`;
the reusable panel never swaps its own snapshot.

`DesignComponentPropertyOrigin` keeps selected-node properties distinct from
properties exposed by one stable nested instance. Nested groups emit separate
Select instance and Go to main intents that remain navigational in viewer
mode. Multiline Text definitions open a real auto-growing editor and use the
same `Begin`/`Preview`/`Commit`/`Cancel`
`ComponentPropertyEditRequested` transaction as other controlled edits.
Property descriptions are accepted and rendered only for Slot definitions,
matching Figma's component-property schema.

`DesignSlotSettings` covers `stretch_child_on_insert`, persistent display of an
empty slot, native nullable minimum/maximum layer counts, preferred-only
guidance, and typed preferred component references. `None` remains distinct
from a numeric zero, matching `SlotSettings.minChildren`/`maxChildren`.
`DesignSlotState` retains
below-minimum/above-maximum/non-preferred violations and reset availability.
Slot settings use Figma's visible **Minimum layers** and **Maximum layers**
wording. Consuming roles expose a transient **Limits** disclosure whenever a
minimum, maximum, or preferred-only guideline exists. Its details show a green
check for every met guideline and an orange warning for every unmet guideline;
an absent host `DesignSlotState` remains neutral rather than being presented
as success. An unmet preferred-instances-only guideline exposes **View
layers**, which emits one `SlotLimitLayersSelectRequested` intent containing
the exact violating child IDs in current Slot order; role, property, violation,
order, and per-child selectability are all revalidated before emission and by
the Storybook host. Limits are guidance rather than enforcement. Slot
definitions can edit settings but cannot mutate instance contents. Component
and slot instances can emit typed Reset, Clear, Add instance, and Add
preferred instance intents. Add remains available after a maximum is exceeded,
while the Limits summary becomes an orange nudge. Unavailable preferred
references cannot be selected.

`DesignSlotValue` is an ordered collection of stable `DesignSlotChild`
descriptors, not an instance-only projection. A child retains its node ID,
actual Design node kind, optional main-component reference, and host-authored
select/remove/reorder/replace capabilities. This preserves the text, image,
vector, frame, and instance layers that Figma allows inside a Slot. Rows emit
stable-ID `SlotChildSelectRequested`, `SlotChildRemoveRequested`,
`SlotChildReorderRequested`, and `SlotChildReplaceRequested` intents; the
Storybook host echoes the resulting value without teaching the reusable panel
how to mutate document children.

Component text/select changes emit `ComponentPropertyChangeRequested` or
phased `ComponentPropertyEditRequested`. Definition settings emit phased
`SlotSettingsChangeRequested`. Per-property reset and slot content operations
carry stable property IDs, so a host never has to address a property by its
row index or display name.

Component authoring is an explicit optional projection on
`DesignComponentContext::authoring`; ordinary instance/default-value editing
does not imply that definitions can be changed. The projection supplies:

- the exact property kinds the host currently allows creating;
- per-definition rename, metadata, default-value, preferred-value, Slot
  settings, delete, reorder, and Variant-option permissions;
- stable identities for every Variant option;
- selected-sublayer applied-property controls for Appearance, Text, and nested
  instance surfaces; and
- stable nested-property exposure candidates with expose, unexpose, and hover
  preview capabilities.

Definition create/reorder intents carry
`DesignComponentPropertyPartition`. Variant definitions are always in the
leading `Variant` partition and Boolean, Text, Instance swap, and Slot
definitions remain in the following `Regular` partition. Reorder anchors are
stable property IDs rather than row indices, and cross-partition or stale
anchors are rejected by the panel. Variant-option create, rename, delete, and
reorder intents likewise use stable option IDs and a stable
`before_option_id` anchor.

Create and Edit property use a real modal `Dialog`; their forms stay live from
the current transient draft and never render a duplicate inline card while
the dialog is active. Boolean defaults and the three Slot flags use native
switches. Boolean/Text creation can bind an eligible Boolean/String variable
by stable catalog ID. Instance swap creation and editing select an actual
default component and ordered preferred components from the host catalog.
Slot creation and editing preserve description, nullable minimum/maximum,
preferred components, preferred-only guidance, display-empty default, and
counter-axis fill-on-insert.

One Edit confirmation emits one
`ComponentPropertyDefinitionEditRequested` transaction containing the exact
expected and replacement metadata plus typed definition. Slot settings remain
inside that definition. This lets a host reject stale combined edits
atomically and create one undo entry; description, documentation links,
defaults, and Slot settings cannot be accepted independently. Documentation
links round-trip unchanged even when the current modal edits only the
description.

Applied-property rows use Figma-like purple pills beside the exact host
surface. Apply, switch, and detach are separate typed intents carrying the
selected layer ID, control ID, surface, and property IDs. Nested exposure rows
carry candidate, nested-instance, nested-property, and exposed-property
identities. Hover emits paired preview `true`/`false` intents; it never changes
the document. All authoring operations require editor permission and the
corresponding host capability. The reusable panel does not generate document
IDs or mutate definitions, applications, or exposure state while waiting for
the host echo. Storybook exercises all five property kinds on a main component,
a component set, selected sublayers, and nested exposure candidates.

## Paint and collection controls

Fill and stroke swatches open the transient paint picker. It includes:

- solid color;
- a Figma-style six-item top-level Solid, Gradient, Pattern, Image, Video, and
  Shader type strip that keeps the four gradient subtypes out of the crowded
  primary row, followed in that same header strip by compact, tooltip-labeled
  Blend mode and Check color contrast controls;
- a keyboard-accessible Linear/Radial/Angular/Diamond subtype menu within the
  Gradient editor;
- canonical Pattern source-node ID, Rectangular/Horizontal hex/Vertical hex
  tiling, one scaling factor, two-dimensional spacing vector, and
  Start/Center/End horizontal alignment;
- host-supplied fill-shader discovery grouped by page and library, explicit
  asynchronous import/apply intents, definition-ID-keyed property values, and
  variable bind/detach controls;
- image and video source plus a discriminated placement contract: Fill/Fit
  with quarter-turn rotation, Crop with an affine transform, and Tile with a
  scaling factor plus quarter-turn rotation;
- distinct host-controlled `Upload from computer`, `Make an image`, and
  `Edit image` source workflows. Upload applies to image and video paints;
  the image-specific generation/editing actions are omitted for video;
- direct one-file external replacement on the media source preview and the
  nested Fill/Stroke swatch. Standard capabilities accept JPG/JPEG, PNG,
  HEIC, WebP, GIF, MP4, MOV, and WebM; TIFF is an explicit host opt-in, while
  SVG, PDF, unknown, extensionless, and multi-file drops are rejected;
- a controlled crop tool with begin/preview/commit/cancel, arbitrary affine
  rotation previews, zoom, aspect ratio, and Resize to fit intents, plus seven
  independent `-1..=1` filter rows;
- host-controlled video preview loading/error/current-time/duration/play state
  with stable-ID play, pause, seek, and phased scrub intents;
- selectable, click-to-add, draggable, removable, and directly editable
  gradient stops;
- exact-position gradient preview segments, plus Figma's Flip gradient and
  quarter-turn Rotate gradient controls;
- per-paint opacity and visibility, plus the complete non-Pass-through blend
  menu behind the compact header control;
- HSV saturation/value area, hue rail, alpha rail, eyedropper affordance,
  controlled Hex/RGB/CSS/HSL/HSB editing, context-correct opacity, and the
  Figma `I` plus macOS `Control-C` shortcuts;
- semantic 3/4/6/8-digit Hex input and CSS `rgba(r, g, b, a)` input. Explicit
  alpha updates solid-paint opacity but gradient-stop alpha; RGB-only forms
  preserve whichever opacity leaf applies;
- a host-owned WCAG contrast projection keyed to the exact Solid or stable
  gradient-stop target: effective scene background, authoritative ratio,
  Auto/Large text/Normal text/Graphics audience modes, AA/AAA (AAA text only),
  a live palette pass/fail boundary, and host-supplied nearest compliant
  colors applied through the ordinary phased paint-edit contract;
- host-controlled Custom and Libraries tabs, including page/library
  sample-only Color styles plus page/library Color variables, explicit
  Available/Imported variable state, exact apply/import/detach/create intents,
  a keyboard-accessible page-or-specific-library palette selector keyed by
  stable host library IDs, case-insensitive Libraries search across style
  names, variable names/groups, and library names, and honest empty or
  no-match states;
- a keyboard-accessible header creation menu that distinguishes whole-Paint
  style creation from exact solid/gradient-stop Color-variable creation. The
  stable active paint and stop are revalidated before either request, and the
  picker never manufactures a style or variable ID;
- separate Fill/Stroke header Paint-style browsers whose rows represent the
  complete ordered paint collection rather than one color leaf;
- pointer controls plus Tab, Enter/Space, Delete/Backspace, arrow-key, and
  Shift-arrow keyboard operation.

The picker is a retained, host-controlled entity mounted in an anchored
Popover. Outside click and Escape dismiss it, focus is tracked by the Popover,
and a host echo resynchronizes accepted paint data without losing an active
input draft. Libraries search is transient and resets when the exact paint
target changes; filtering never rewrites or drops the host catalog. A stable,
collection-unique host-supplied `DesignPaint::id` keeps
the target open when the paint collection is reordered; an empty ID retains
the legacy indexed fallback. Direct edits to bound color leaves, read-only
paints, and viewer permissions remain inspectable but do not emit mutation
intents. A host-supplied Color variable row can request an explicit rebind of
an otherwise editable color leaf; Available library variables emit import
first and must be selected again after the host echoes Imported state.
Focused color text fields emit exactly one
`Begin -> Preview* -> Commit|Cancel` transaction. Eight-digit Hex and CSS
RGBA may preview both a solid color leaf and its paint-opacity leaf inside that
single whole-paint transaction; Enter supplies the sole terminal and its
deferred Blur is inert.

Adding a gradient stop deliberately creates an empty-ID candidate so the
reusable picker never invents document identity. Until the host echoes its
stable ID, the retained selected-stop index follows the newly inserted stop,
including when its position and interpolated RGBA duplicate an existing stop.
Read-only and viewer pickers reject preview-bar additions before changing that
transient selection.

Hosts provide contrast data with
`DesignPanel::set_color_contrast_view_data(DesignColorContrastViewData)`.
Entries are scoped to the current inspector selection and keyed by stable paint
identity—or by exact ordered `DesignPanelTarget` plus Selection-color row
identity. Stale node, paint, gradient-stop, aggregate-row, or selection targets
are ignored. The picker never guesses a white/black canvas and never predicts
an accepted correction: ratio, effective background, Auto resolution, and the
nearest compliant colors remain host owned. Category and level are transient
picker preferences. Clicking a failing indicator emits one normal Commit edit,
and the displayed paint changes only after the host echoes it.

The same retained color core is reused in a color-only mode for Page
background, text-decoration color, shadow/noise effect colors, ordinary
layout-guide color/opacity, and multiple-selection aggregate colors. These
surfaces do not expose paint type or paint-library controls, but they retain
the identical arbitrary RGBA, format, hue/alpha, focus, and phased transaction
behavior. Effect and layout-guide targets resolve their stable host IDs again
on every phase, so a concurrent reorder cannot retarget an open picker.
`PaintPicker::set_color_only_editability` lets the panel independently disable
the color leaf and opacity leaf, so a bound guide color does not unnecessarily
lock an otherwise editable guide opacity (and vice versa).

`DesignPaintPayload` is the canonical discriminated paint schema. Its Solid,
Gradient, Pattern, Image, Video, Shader, and Unsupported variants retain
type-specific settings without flattening them into a color swatch. The legacy
`kind`, `color`, and `gradient_stops` fields remain synchronized projections
for existing adapters. Solid colors and individual gradient stops retain
variable-binding metadata. Shader is a supported sixth paint type with an exact
shader ID and ordered, definition-ID-keyed typed property assignments;
Unsupported alone retains an opaque host payload rather than coercing unknown
data into a supported paint type. Pattern mirrors Figma's beta `PatternPaint`
schema directly and deliberately has no Mirror/Clamp modes, independent X/Y
scale, or arbitrary affine transform.

Image and video document payloads contain only source, discriminated placement,
and the seven bounded filters. Mode-inapplicable and non-finite edits are
rejected. Crop-tool state and video playback are host-controlled view data
supplied with `DesignPanel::set_media_paint_view_data`; preview autoplay, loop,
and poster-frame values are deliberately absent from Design-tab paint
semantics. `can_edit_properties`, `can_upload_source`, `can_make_image`, and
`can_edit_image` are independent, so a host can allow filter edits and uploads
while withholding either AI image workflow.
`DesignMediaPaintCapabilities::accepted_drop_file_kinds` is a copyable
`DesignMediaFileKinds` bitset independent from property editing. The editor
preset uses `STANDARD`, which intentionally omits TIFF; a host opts in with
`STANDARD | TIFF`. Direct drops remain gated by `can_upload_source`.
The legacy node-level `DesignMedia` snapshot and `ReplaceMediaRequested` action
remain available for old adapters, but the panel does not render or emit them.
Pattern source selection retains `PaintSourceReplaceRequested`. Image/video
buttons instead emit
`PaintMediaSourceActionRequested { source_id, action }` with the exact
collection, `DesignPaintTarget`, stable paint ID, fallback index, current
source ID, and `Upload`/`MakeImage`/`EditImage` discriminator. The host owns the
file chooser, generation/editor surface, and resulting source. Before emitting,
the panel resolves the current stable paint again and rejects a stale source
ID, an action incompatible with the current image/video discriminant, a
read-only paint, or a capability that the latest host view disabled. Hosts can
therefore reject an asynchronous result if either the paint or its source was
replaced while that workflow was open.
Dropping a supported external path emits
`PaintMediaSourceDropRequested { expected_source_id, expected_media_kind, file, .. }`.
`DesignMediaDroppedFile::from_paths` accepts exactly one path and classifies
its final extension case-insensitively into `DesignMediaFileKind`; this is a
syntactic check, so the host must still validate readability and content
before importing. The action additionally retains the exact collection,
selected-text range target, stable paint ID, and host-resolved current index.
The panel resolves and validates those values again at drop time along with
edit access, collection support, whole-Paint style binding, paint read-only
state, source identity, current Image/Video discriminant, and latest accepted
format bitset. Invalid drops never fall through to paint reordering.

The Storybook demonstrates standard and TIFF-opted-in capabilities. Its
reducer rejects stale source/kind/identity or disallowed formats, never records
an absolute path in its action status, and consumes an accepted file by
preserving stable paint identity and order, opacity, visibility, blend mode,
placement, affine crop, and filters even when an image file changes to a video
paint or vice versa. Every accepted import receives a fresh monotonic,
host-owned source ID that is independent from node, paint, file kind, and local
path; retrying a superseded source ID is inert.
The Storybook's four Image nodes expose All source actions, Upload + Edit,
Properties only, and Make only capability matrices. Its mock reducer changes
the controlled source only after receiving a typed request and rejects an
output whose echoed `source_id` no longer matches.

Hosts supply leaf Color variables with
`DesignPanel::set_paint_variable_view_data(DesignPaintVariableViewData)`.
Selecting a page/imported variable emits `PaintColorVariableApplyRequested`
with the collection, stable paint ID, a discriminated solid/gradient-stop
target, and stable variable ID. Available variables emit
`PaintColorVariableImportRequested`; detaching echoes the exact current
variable ID. The host resolves the variable value and echoes the updated paint
leaf. `DesignPaintBinding` is variable-only metadata.

Hosts may separately supply
`DesignPanel::set_color_style_sample_view_data(DesignColorStyleSampleViewData)`.
`DesignColorStyleSample` is deliberately a sample-only solid Color-style
value. Selecting it emits `PaintColorStyleSampleRequested` for one exact solid
or stable gradient-stop leaf. The host changes that resolved color only: it
does not write `fillStyleId`/`strokeStyleId` and does not manufacture a
Color-variable binding. This is distinct from both the compatibility-only
`DesignColorStyleViewData` API and the complete Paint styles below.

Hosts independently supply whole Paint styles with
`DesignPanel::set_paint_style_view_data(DesignPaintStyleViewData)`. A
`DesignPaintStyle` retains the complete ordered paint snapshot and
Imported/Available state. The Fill/Stroke header emits
`PaintStyleApplyRequested`, `PaintStyleImportRequested`,
`PaintStyleCreateRequested`, or `PaintStyleDetachRequested`.
`DesignPanelNode::fill_style_binding` and `stroke_style_binding` model
`fillStyleId`/`strokeStyleId`; they never masquerade as a binding on a solid
paint or gradient stop. `DesignColorStyleViewData` and the old
`PaintColorStyle*` actions remain compatibility-only migration surfaces and
are not emitted by the built-in panel.

The Fill/Stroke Paint-style browser, Effects style browser, Layout-guide style
browser, and Selection Color Paint-style browser share the same functional
resource toolbar. Search matches a style name, its type/paint preview summary,
or its library name. Source pills switch among All, This page, Libraries, and
each stable host library ID (shown with its current host-supplied name), while
List/Grid buttons change only row composition. These controls are retained
presentation state: filtering never removes a host-supplied style, changes its
stable page/library identity, or bypasses Imported/Available gating. When an
open browser receives a renamed library with the same stable ID, its
selected label follows the host echo; if that ID disappears, the source
returns to All instead of retaining a dead filter. Applying, importing,
creating, and detaching continue through the browser-specific typed intents
described in this contract.

Hosts supply available fill shaders with
`DesignPanel::set_shader_view_data(DesignShaderViewData)`. Page shaders and
library shaders retain stable shader and library IDs plus import state.
Selecting an unavailable library shader emits `PaintShaderImportRequested`;
selecting an imported shader emits `PaintShaderApplyRequested`. Direct Boolean
and Number controls emit definition-ID-keyed `PaintEditRequested` values.
Complex values emit `PaintShaderPropertyEditorRequested`, and variable controls
emit explicit `PaintShaderPropertyBindRequested` /
`PaintShaderPropertyDetachRequested` intents. The panel never calls shader
discovery/import APIs or writes a document. Before forwarding even a
picker-originated direct property edit, the panel resolves the current stable
paint and shader IDs again and verifies that the definition ID still exists
and the typed value still matches its host-supplied property kind. A stale
control from replaced shader metadata therefore cannot write an incompatible
assignment.

Picker and inline row edits emit `PaintEditRequested` with collection, stable
paint ID, legacy fallback index, typed `DesignPaintProperty` /
`DesignPaintValue`, an exact `DesignPaintTarget`, and
`Begin`/`Preview`/`Commit`/`Cancel` phases. In Text edit mode, every Fill
collection, style, source, crop/video, shader, Color-variable, Color-style
sample, eyedropper, reorder, and typed paint intent targets
`SelectedTextRangeRevision(revision)` when the inspection context supplies a
range revision; hosts without a revision receive the compatibility
`SelectedTextRange` target. Stroke always carries `WholeLayer`, including
while characters are selected. Collection arrows emit
`PaintReorderRequested`; pattern source selection emits
`PaintSourceReplaceRequested`, while image/video source workflows emit the
typed `PaintMediaSourceActionRequested` or direct-file
`PaintMediaSourceDropRequested`. The component never reorders its own
snapshot, reads/imports a dropped file, runs an image model, or predicts the
host result.
The older whole-paint `PaintChangeRequested` action remains available as a
compatibility surface, but the built-in picker uses typed edits.

Hosts must resolve a selected-range paint intent only when its revision still
matches the active character range. Changing
`DesignPanelInspectionContext::text_range_revision` on the same text node
cancels an open Fill editor/picker transaction against the old target before
installing the new controlled snapshot. Begin/Preview/Commit/Cancel therefore
stay keyed by node ID, Fill range target, stable paint ID, and legacy index
fallback; an asynchronous style/source/variable result for an older range
cannot affect the newer range. The Storybook host enforces that comparison,
keeps paint/video/crop session state per exact target, rejects out-of-order or
stale non-cancel phases, and exposes a `Select next range` control in its Text
edit fixture.

The paint contract follows Figma's current
[six-member Paint union](https://developers.figma.com/docs/plugins/api/Paint/),
[PatternPaint source-node schema](https://developers.figma.com/docs/plugins/adding-pattern-fills-and-strokes/),
[crop workflow](https://help.figma.com/hc/en-us/articles/360040675194-Crop-an-image),
[image properties](https://help.figma.com/hc/en-us/articles/360041098433-Adjust-the-properties-of-an-image),
[image/video source workflow](https://help.figma.com/hc/en-us/articles/360040028034-Add-images-and-videos-to-designs),
[video preview/prototype behavior](https://help.figma.com/hc/en-us/articles/8878274530455-Use-videos-in-prototypes),
and
[shader discovery/import/property-definition model](https://developers.figma.com/docs/plugins/api/Shader/).

`DesignBlendMode::ALL` is the exact 19-value current layer set, including
Plus darker (`LinearBurn`), Plus lighter (`LinearDodge`), Soft light, and Hard
light. Paint and effect controls use the 18-value
`NON_PASS_THROUGH` subset because Pass through applies only to eligible layer
containers.

### Inspector-menu hover previews

Layer and paint blend modes, Fixed/Hug/Fill resizing, stroke
Inside/Center/Outside, and effect option menus use one balanced host preview
contract. Hover or keyboard highlighting emits
`MenuPreviewRequested { preview, phase: Begin }`; pointer/focus exit, popover
close, picker or panel dismissal, commit, selection/reorder/permission change,
and an invalidating host echo emit exactly one matching `End`. That `End`
repeats the immutable original `Begin` payload instead of resolving newer
indices.

`DesignMenuPreview` distinguishes exact ordered node targets, stable effect ID
plus compatibility index, and paint node/collection/text scope/stable paint
ID/index targets. Every variant carries the original and candidate typed
property value. The panel never applies the candidate to its controlled
snapshot; the Storybook keeps transient preview state separate from committed
mock nodes and rejects stale or unbalanced events. Viewer, read-only,
style-bound, variable-bound, unsupported, and already-selected choices never
start a preview. Commit first drains the preview, then emits the existing
ordinary property, effect, or paint commit intent.

This matches Figma's documented canvas preview behavior for
[layer and paint blend modes](https://help.figma.com/hc/en-us/articles/360040667874-Use-blend-modes-to-create-unique-effects),
[Fixed, Hug, and Fill resizing](https://help.figma.com/hc/en-us/articles/360040451373-Guide-to-auto-layout),
[stroke position](https://help.figma.com/hc/en-us/articles/360049283914-Apply-and-adjust-stroke-properties),
and [effect choices](https://help.figma.com/hc/en-us/articles/360041488473-Apply-effects-to-layers).
Paint targeting follows Figma's
[paint-picker model](https://help.figma.com/hc/en-us/articles/360041003774-Apply-paints-with-the-color-picker).

## Selection colors and masks

`DesignSelectionColors` is a host-owned, getSelectionColors-like
multiple-selection aggregate. Each canonical `DesignSelectionColor` is one
complete normal Solid or Gradient paint, not one row per gradient stop. It
carries a stable row ID, representative `DesignPaint`, compatibility display
color, occurrence count, complete ordered collection snapshot for Paint-style
creation, independent whole-collection `DesignPaintStyleBinding` and
Solid-color `DesignPaintBinding`, read-only state, and the complete ordered set
of `DesignSelectionPaintReference` values affected by an edit. References
retain node ID, Fill/Stroke collection, stable paint ID, and legacy index
fallback; optional gradient-stop identity remains compatibility-only for older
leaf-color hosts. Hosts deduplicate by complete Paint semantics plus exact
Paint-style identity, Solid Color-variable identity, and read-only semantics.
Equal RGBA projections with different gradients, opacity, blend, payload,
styles, variables, or editability remain separate rows.

The row presents four independent controls. Its full paint picker emits phased
`SelectionColorPaintEditRequested` intents using the same typed
`DesignPaintEdit` contract as ordinary Fill and Stroke paints, including paint
type/payload, opacity, blend, and gradient leaves. The Paint-style browser
consumes the same local/imported/available `DesignPaintStyleViewData` catalog
and emits
`SelectionColorPaintStyle{Apply,Import,Create,Detach}Requested`; creation
carries the row's exact ordered `style_paints`. The Color-variable browser
is available only to Solid rows, consumes `DesignPaintVariableViewData`, and emits
`SelectionColorVariable{Apply,Import,Create,Detach}Requested`. Every intent
carries the exact ordered multi-node `DesignPanelTarget`, stable aggregate ID,
and ordered underlying references. The target icon emits the viewer-safe
`SelectionColorOccurrencesSelectRequested` intent only when every displayed
occurrence has an exact supplied reference. A collection-level style binding
never masquerades as a Solid Color-variable binding: while a style is
attached, the paint picker is inspectable but mutation is gated until style
detach. A variable-bound Solid color gates its color leaf while leaving
paint-level opacity and blend independently editable. Read-only and view-only
rows remain inspectable; they emit no paint mutation, while exact occurrence
selection remains available.

An open picker is keyed by the exact ordered selection target and stable row
ID. On a same-target host echo it re-resolves the row and emits the refreshed
ordered occurrence references, so paint/index reorder cannot drift an active
transaction. Changing the selected-node order or identity cancels the
interaction before installing the new controlled snapshot.

The original `selection_colors: Vec<DesignPaint>` field is adapted only when a
host has not supplied the canonical aggregate. It remains a compatibility
projection and does not regain fill add/remove/reorder semantics.

Masks use the orthogonal `DesignPanelNode::is_mask` flag and typed
`DesignMaskType::{Alpha, Vector, Luminance}` mode. The legacy string
`mask_type` field is read only as a migration fallback. An active mask resolves
a dedicated Mask section containing its type dropdown; creating or removing a
mask remains a selected-node header command rather than a canonical checkbox.
`MaskType` property intents carry `DesignPanelValue::MaskType`. An explicitly
supplied legacy Geometry-only capability snapshot retains the old toggle and
type projection for source migration, and a snapshot containing both sections
never renders them twice.

Typography keeps Figma's three resize modes (`Auto width`, `Auto height`, and
`Fixed size`) separate from truncation. The native Max lines row is disclosed
only when Truncate text is enabled and resize is Auto width or Auto height; a
text layer inside any auto-layout parent must additionally use vertical Hug
sizing. A nullable `max_lines` is displayed as `Auto`, matching Figma's
inspector and the Plugin API's `null` value. Disabling truncation, choosing
Fixed size, or changing an auto-layout child away from vertical Hug restores
Max lines to Auto in the Storybook host reducer.

Max height and a numeric Max lines value are mutually exclusive node-level
limits. Applying Max height atomically resets Max lines to Auto, while applying
a numeric Max lines value atomically removes Max height. Setting Max lines
back to Auto does not discard an independently supplied Max height. Both
`TextMaxLines` and `TextResize` typography intents always use
`DesignTypographyTarget::WholeLayer`, even during character editing;
`MaxHeight` remains an ordinary whole-node property intent. The reusable panel
only emits these requests and waits for the host's canonical echo.

The Storybook catalog includes stable Auto width, Auto height, Fixed size,
vertical-Hug child, vertical-Fixed child, and Max-height-conflict fixtures.
These rules follow Figma's
[text-property guidance](https://help.figma.com/hc/en-us/articles/360039956634-Explore-text-properties),
[auto-layout sizing guidance](https://help.figma.com/hc/en-us/articles/360040451373-Explore-auto-layout-properties),
and nullable
[`maxLines` Plugin API contract](https://developers.figma.com/docs/plugins/api/properties/nodes-maxlines/).

Line height is typed as Auto, pixels, or percent; letter spacing is typed as
pixels or percent. Figma's `LeadingTrim` API is retained as typed
None/Cap-height data and rendered as Vertical trim.
`DesignTypography::weight` is Figma's controlled numeric font weight and stays
independent from the string-valued font style. Its value editor uses the same
Begin/Preview/Commit/Cancel lifecycle as other numeric typography leaves and
maps variable operations exactly to the Float `fontWeight` field with
`FONT_WEIGHT` scope. Font Family and Font Style expose their own String
variable buttons beside the combined font browser, so each of the three leaves
can independently be mixed, bound, detached, or read only.
Vertical alignment is available only for fixed-size text. The anchored Type
settings popover tracks focus, dismisses on outside click or Escape, and
exposes Basics and Details surfaces plus a Variable surface when the host
supplies variable-font axes. Basics uses direct alignment, decoration, case,
vertical-trim, list, `listSpacing`, paragraph-spacing, truncation, and
maximum-line controls. Details covers hanging punctuation/lists, paragraph
indent, case, and the current Text API decoration style, offset, thickness,
color, and skip-ink fields. Paragraph indent appears only with Left horizontal
alignment, matching Figma's current inspector behavior.

Variable-font axes are lossless host records rather than inferred Weight-only
fields. Each `DesignFontAxis` carries its exact four-character OpenType tag,
display name, current/minimum/maximum/default values, host-authored step,
Available/Read-only/Unavailable state with an optional reason, and optional
variable provenance for a corresponding bindable typography leaf. Variable
fonts and Figma variables remain separate features: binding provenance makes
the axis inspectable but disables direct axis changes; replacement or detach
continues through the ordinary typography-variable surface.

The Variable tab renders a focusable slider and numeric input for registered
weight (`wght`), width (`wdth`), optical-size (`opsz`), slant (`slnt`), italic,
and arbitrary font-author axes without special-casing their ranges. Arrow keys
step a focused slider or numeric input; Shift multiplies the host step by 10,
Alt uses one-tenth precision, and Home/End move a slider to its exact bounds.
Pointer slider/value scrubbing uses the same threshold and modifier precision
as other inspector numerics. Numeric expressions are clamped to the supplied
axis range.

Every interaction emits
`TypographyVariableAxisEditRequested { node_id, target, tag, value, phase }`
with Begin/Preview/Commit/Cancel and never mutates `DesignTypography`. The
OpenType tag—not a row index—is authoritative, so an active edit survives an
exact host echo that reorders axes. A missing/duplicate tag, changed range,
read-only/Unavailable/bound state, permission downgrade, node replacement, or
text-range revision change cancels once. The cancellation retains the
Whole-layer or exact `SelectedTextRangeRevision` target captured at Begin.
Storybook mock state includes editable registered and author-defined axes,
plus read-only and bound examples, and echoes each phase through its controlled
host reducer.

OpenType is an ordered host-supplied feature list rather than fixed inferred
booleans. Each `DesignOpenTypeFeature` retains its exact tag and name, font
default, current value, availability/reason, and optional preview. Registered
four-character tags and opaque future tags remain distinct. Changing an
available record emits `TypographyOpenTypeFeatureChangeRequested` with the
unchanged tag. Typography property and OpenType intents identify whether they
target the whole layer or the active text range.

TextPath nodes additionally carry `DesignTextPathViewData`. It retains the
host's current Default/Flipped orientation, whether native Flip is currently
available, and an explicit start-data debug disclosure. The Typography section
uses Figma's current first-party **Flip text orientation** label, exposes the
host orientation as a readout/selected state, and emits only
`TextPathFlipOrientationRequested { node_id }`. Editor permission and the
current host capability are revalidated at activation; the component does not
optimistically mutate orientation or vector geometry.

`DesignTextPathStartData` still mirrors
`textPathStartData.segment` and the canonical `0..=1` position along that
segment, but its two numeric controls are now hidden unless the host explicitly
enables the API-debug disclosure. When disclosed, they preserve the complete
`TextPathStartChangeRequested` candidate and edit-phase contract; the Storybook
host keeps the Begin snapshot and restores it on Cancel. This reflects the
first-party split: the native editor uses a canvas handle for placement and a
sidebar Flip command, while the public Plugin API exposes
`textPathStartData`. Storybook includes a native flippable fixture and a
separate Flipped, non-flippable API-debug fixture so both projections remain
reachable without conflating them.

### Vector vertex editing

Vector and TextPath nodes can additionally supply `DesignVectorEditViewData`
while the inspection context is in `DesignPanelEditMode::Vector`. Each
`DesignVectorVertexViewData` carries an opaque host-authored ID, finite X/Y
coordinates, optional per-vertex `cornerRadius`, optional
`DesignHandleMirroring`, Endpoint/Interior/Branch topology, selection, and a
read-only flag. IDs are never derived from the vector list index, so a host can
reorder its network without retargeting an in-flight edit.

The contextual block appears only for an editable single Vector or TextPath
selection when the host reports at least one selected vertex. Vertex chips emit
`VectorVertexSelectionEditRequested`; X/Y fields emit
`VectorVertexPositionEditRequested`; radius emits
`VectorVertexCornerRadiusEditRequested`; and tangent behavior emits
`VectorHandleMirroringEditRequested`. Every action carries exact vertex IDs and
a `Begin`, `Preview`, `Commit`, or `Cancel` phase. The panel does not update the
selection or geometry while waiting for a host echo.

`DesignHandleMirroring` contains only Figma's writable `NONE`, `ANGLE`, and
`ANGLE_AND_LENGTH` values. `DesignVectorSelectionValue::Mixed` represents a
mixed selection without inventing a fourth writable value. Mixed coordinates,
radii, and handle modes render as Mixed but remain directly replaceable.
Whole-view or per-vertex read-only state disables intents. Branch vertices keep
coordinate editing but suppress radius and single-tangent mirroring because
those controls are ambiguous at a multi-segment junction. The Storybook seeds
mixed Vector and TextPath fixtures and restores its exact Begin snapshot,
including host order and selection, on Cancel.

The Typography header's four-dot action opens a retained, anchored text-style
picker. Hosts supply exact page styles and grouped libraries with
`DesignPanel::set_typography_style_view_data(DesignTypographyStyleViewData)`;
the component never creates styles or applies font values itself. Search is a
transient keyboard-operated picker draft, while style scope and identity use
`DesignTypographyStyleSelection::{page, library}`. Applying a row emits
`TypographyStyleApplyRequested`; detaching the current
`DesignTypography::style_binding` emits `TypographyStyleDetachRequested`.
Both actions carry the selected node ID and `DesignTypographyTarget`, and the
host accepts either operation by echoing a fresh typography snapshot. Viewer
permissions and host-supplied read-only `TypographyStyle` property states keep
styles searchable and inspectable while disabling apply/detach intents.

Font family/style selection is separately host controlled through
`DesignPanel::set_font_view_data(DesignFontViewData)`. Its retained searchable
browser renders catalog `Loading`, `Ready`, and `Unavailable` states plus
per-style `Imported`, importable `Available`, `Missing`, and `Unavailable`
rows. Imported rows emit `TypographyFontApplyRequested`; discoverable library
rows emit `TypographyFontImportRequested` without implicitly applying. Both
carry an exact `DesignFontSelection` and `DesignTypographyTarget`; the host
loads/imports or applies it and echoes fresh font and typography snapshots,
including the catalog style's numeric weight when one is supplied. Direct
catalog apply remains gated while any affected Family, Style, or Weight leaf
is variable/read-only bound; catalog import remains a non-mutating discovery
operation.

Fills, strokes, and layout grids use typed collection add/remove intents.
Paints and effects additionally use stable identity for edit, remove, and
reorder operations. Export uses stable configuration IDs and dedicated typed
intents. The story applies those intents to mock state and immediately
supplies the updated read model back to the component.

Frames can additionally expose Figma's native
[**Show in exports** Fill control](https://help.figma.com/hc/en-us/articles/360040028114-Guide-to-exports-in-Figma)
through `DesignPanelNode::fill_shows_in_exports`. `None` suppresses the row;
`Some(bool)` renders a controlled checkbox and emits the ordinary typed
`FillShowsInExports` property intent while the Frame has at least one fill.
Because Figma does not publish a Plugin
API field for this UI value, the inspector never tries to derive or persist it.

Stroke paints are the only indexed stroke values. A node has one optional
`DesignStroke` whose `paints` collection shares weight, position, width
properties, dash style, joins, endpoints, and complex stroke settings. Removing
or reordering one paint therefore cannot silently replace the geometry used by
the other paints.

The shared stroke editor covers:

- all/individual top, right, bottom, and left weights for Rectangle and
  frame-like nodes, with All/Top/Bottom/Left/Right/Custom editor modes; Section
  nodes retain uniform weight only;
- Inside/Center/Outside positioning when the host capability allows it, and
  fixed Center positioning for Line/Arrow and non-Basic complex strokes;
- the exact eight Figma endpoint values: None, Round, Square, Line arrow,
  Triangle arrow, Reverse triangle, Diamond arrow, and Circle;
- separate start/end controls for a two-endpoint open path, an aggregate
  advanced control for larger networks, and a selected-vertex control in
  vector edit mode;
- Miter/Bevel/Rounded joins, with miter angle shown only for Miter;
- an explicit Solid/Dashed/Custom dash-mode transition over Figma's ordered
  `strokeDashes` array, plus None/Round/Square dash caps for non-solid modes;
- all six variable-width presets—Uniform, Wedge, Taper, Quarter taper, Eye,
  and Mirrored taper—and lossless host-ordered custom `{ position, width }`
  points. Variable width is suppressed for branching networks, Dynamic
  strokes, opaque custom forms, and node kinds whose host capability disables
  it;
- discriminated Basic, Stretch brush, Scatter brush, Dynamic, and opaque
  read-only complex-stroke forms;
- all 15 writable Stretch brush names and direction, all 10 writable Scatter
  brush names plus gap/wiggle/size jitter/angular jitter/rotation, and Dynamic
  frequency/wiggle/smoothen;
- the exact documented domains: Scatter gap `>= 0.25`, wiggle `>= 0`, size
  jitter `0..=3`, angular jitter and rotation `-180..=180`; Dynamic frequency
  `0.01..=20`, wiggle `>= 0`, and smoothen `0..=1`.

Figma's `CUSTOM` brush payload is readable but not writable through the Plugin
API. `DesignOpaqueComplexStroke` therefore retains a host-authored type, label,
and raw payload for inspection without offering a lossy replacement intent.

`DesignStrokeCapabilities` and `DesignStrokeEditContext` are supplied by the
host. They capture layer support, open/closed/branching topology, endpoint
count, selected vertices, and which selected vertices are endpoints so the
panel hides controls that are invalid for the inspected node instead of
inferring document geometry.

Effects are an ordered, stable-ID collection. Rows stay compact; dragging one
emits `EffectReorderRequested { effect_id, from_index, to_index }`, removing it
emits `EffectRemoveRequested`, and every leaf edit emits
`EffectEditRequested` with the same ID plus a legacy index hint. One settings
popover is anchored to the active row, and a host echo resolves it by ID after
reordering. The panel never mutates or sorts the controlled collection, which
is important because Figma renders effects according to their order and only
the first Background blur/Glass conflict can render.

`DesignEffectCapabilities` combines Figma's documented type limits with exact
host availability. The panel caps Drop/Inner shadow at eight each, Noise at
two, and Layer blur, Background blur, Texture, and Glass at one each. Shader
availability remains host-authored because a compatible shader must be
discovered/imported before use. Shadow spread is rendered only when the host
marks the exact node eligible; the same is true for “Show behind transparent
areas.” These rules follow Figma's current
[effects guidance](https://help.figma.com/hc/en-us/articles/360041488473-Apply-effects-to-layers)
and [Effect API](https://developers.figma.com/docs/plugins/api/Effect/).

For multiple selection, the synthetic Effects header intersects both
host-authored availability and each real node's remaining per-kind capacity;
clearing the aggregate's rows never resets those limits. The Storybook
preflights `EffectAddRequested` against every exact ordered member, including
style binding and maximum count, then appends to all members in one reducer
step. If any member becomes maxed, unsupported, stale, duplicated, or
permission-invalid, none of the selected nodes changes.

Each row remains discriminated by host-provided settings. Drop and inner
shadows expose color, blend mode, offset, blur, and capability-gated spread.
Layer and background blur switch between normal radius and progressive
start/end radii and control points. Noise exposes mono/duo/multi color modes,
blend, size, and density; Texture exposes size, radius, and clipping; Glass
exposes lighting, refraction, depth, dispersion, frost, and splay.
`DesignShaderEffect` retains the imported shader ID and author-definition IDs
for structured Boolean, text, number, resource, color, geometry, gradient, and
variable-alias values. Boolean is an atomic toggle; text, number, color, point,
line, circle, circle-point, color-point, and every gradient stop expose typed
inline editors. Continuous leaves emit one
`EffectEditRequested` Begin/Preview/Commit-or-Cancel transaction containing
the complete typed Shader-property value. The effect ID and property-definition
ID are authoritative, so an open editor follows both effect and metadata
reordering and cancels if the definition disappears or changes semantic kind.

Image, instance-swap, and Slot properties do not invent replacement IDs.
Their Choose control emits
`EffectShaderPropertyEditorRequested { editor: Resource, ... }` for the host's
asset surface. Whole-property aliases, color-point colors, and gradient-stop
colors use the same intent with `editor: Variable`; echoed bindings can be
changed or detached with the exact current variable ID through
`EffectShaderPropertyVariableDetachRequested`. A bound leaf gates raw color
editing until detach. Opaque property values show their type and payload as
honest read-only data and never emit an unchanged candidate.
`DesignEffectSettings::Opaque` likewise preserves future effect types without
exposing destructive editors. See Figma's
[Shader contract](https://developers.figma.com/docs/plugins/api/Shader/).

The Effects header has a controlled page/library style browser. Hosts supply
`DesignEffectStyleViewData`; apply/create/detach actions carry stable style
selection data, while `DesignPanelNode::effect_style_binding` makes individual
rows read-only until the host accepts a detach. Bindable shadow leaves
(`radius`, `color`, `spread`, `offsetX`, and `offsetY`) and blur `radius`
retain `DesignEffectVariableBinding` metadata. Compatible variables come from
`DesignEffectVariableViewData`, and apply/detach emit ID-addressed host
intents. Noise, Texture, Glass, and opaque effects deliberately expose no
variable controls, matching Figma's
[VariableBindableEffectField](https://developers.figma.com/docs/plugins/api/VariableBindableEffectField/)
contract. Irrelevant controls are never rendered or resolved for another
effect kind.

## Layout guides

`DesignLayoutGrid` separates stable guide identity plus common visibility,
color, and opacity from a discriminated `DesignLayoutGridSettings` payload:

- `Uniform` exposes one square-grid size.
- `Columns` uses Left/Center/Right/Stretch alignment.
- `Rows` uses Top/Center/Bottom/Stretch alignment.

Every ordinary field edit emits
`LayoutGridPropertyEditRequested { guide_id, index, property, value, phase }`;
removal emits `LayoutGridRemoveRequested`. A non-empty `guide_id` is
authoritative and the index is only a legacy hint. The shared retained color
editor maps RGB to `LayoutGridColor` and its alpha control to the exact
percentage-valued `LayoutGridOpacity`, resolving the guide ID again after any
host reorder.

Every numeric guide leaf has the same anchored Number-variable affordance:
Uniform Size; fixed row/column Width or Height; stretch Margin or fixed
Offset; Gutter; and Count. `DesignLayoutGridVariableField` preserves Figma's
exact `sectionSize`, `count`, `offset`, and `gutterSize` identities. Margin and
Offset both map to `offset`, but
`DesignLayoutGridVariableTarget::property` retains which inspector leaf was
selected. Its non-empty `guide_id` is authoritative after reorder; `index`
remains only a compatibility hint.

Hosts populate `DesignLayoutGridVariableViewData` with ordinary Float
`DesignVariable` rows, including source, collection, Local/Imported/Available
import state, resolved value, and disabled reason. `create_state` explicitly
controls whether Create is hidden, enabled, or disabled with a reason.
`DesignLayoutGridVariableBinding` retains the current opaque variable identity
plus host-controlled detachability and a read-only reason. A bound leaf is
read-only for direct entry until the host accepts detach and echoes a fresh
guide.

Apply, import, detach, and create emit
`LayoutGridVariableApplyRequested`,
`LayoutGridVariableImportRequested`,
`LayoutGridVariableDetachRequested`, or
`LayoutGridVariableCreateRequested`. Every action carries
`DesignLayoutGridVariableTarget`, so the host receives the stable guide ID,
exact inspector property, exact Figma API field, and legacy index together.
Count remains typed as `Number(u16)` or `Auto`; a finite positive whole-number
variable can bind a numeric Count, while positive infinity represents Auto
and cannot silently cross that discriminant. Viewer mode, host read-only
bindings, disabled candidates, and a node-level Grid style keep the browser
inspectable but suppress mutations. The deprecated count-only view-data
adapter and action aliases remain available for older hosts, but the panel and
Storybook emit only the generalized actions.

Grid style identity belongs to `DesignPanelNode`, not to an individual guide.
This mirrors Figma's one `gridStyleId` binding over the complete ordered
`layoutGrids` array. `DesignLayoutGridStyle` therefore stores an atomic guide
snapshot, while `DesignLayoutGridStyleViewData` supplies page styles and
library groups with stable style/library IDs and Imported/Available state.
The Layout guides section-header button opens an anchored browser:

- Apply replaces the full ordered guide collection through
  `LayoutGridStyleApplyRequested`.
- Create sends the current full guide snapshot through
  `LayoutGridStyleCreateRequested`.
- Detach identifies the exact current node-level binding.
- Import identifies an available library style without applying it.

All four operations remain host-owned and read-only gated. A current
`DesignLayoutGridStyleBinding` locks guide add/remove and every guide field
until the host accepts detach and echoes a fresh node snapshot. This contract
follows Figma's official
[`GridStyle.layoutGrids`](https://developers.figma.com/docs/plugins/api/GridStyle/)
replacement semantics; count is one of the fields listed by
[`VariableBindableLayoutGridField`](https://developers.figma.com/docs/plugins/api/VariableBindableLayoutGridField/),
and candidate IDs/collections follow the
[Variables API](https://developers.figma.com/docs/plugins/api/figma-variables/).

Fixed row/column alignment exposes size and gutter, plus offset only for
edge-aligned Left/Right or Top/Bottom guides. Centered guides omit offset.
Stretch alignment displays size as Auto and exposes margin and gutter. Guide
color accepts RGB/RGBA hex input, opacity is edited independently as 0–100%,
and all edits remain host controlled. Ordinary fields emit
`LayoutGridPropertyEditRequested`; removal emits
`LayoutGridRemoveRequested`. Both carry the stable `DesignLayoutGrid::id` and
retain the current index only as an empty-ID compatibility hint. A continuous
field transaction captures the guide identity at Begin, re-resolves that
identity after every host echo, and emits Preview/Commit/Cancel against the
guide's latest index. Reordering the host-owned array therefore cannot retarget
an active editor. Frame, component, component set, instance, and Slot
inspectors share this renderer.

## Export

Hosts supply the complete export surface with
`DesignPanel::set_export_view_data(DesignExportViewData { .. })`. Each
`DesignExportConfiguration` has a stable host ID, typed sizing, common
settings, and settings discriminated by its format. The older
`DesignExportSetting` scale-only rows remain as a compatibility adapter when
canonical view data has not yet been supplied.

The host-controlled `DesignExportMode` switches between Static and Animated
without storing document state in the panel. Static formats are exactly PNG,
JPG, SVG, and PDF. PNG and JPG accept the
canonical `x`, `w`, and `h` sizing forms (`2x`, `1440w`, and `800h`). SVG and
PDF always normalize to intrinsic `1x` sizing. Common advanced settings include
suffix and color profile. Format-specific controls cover overlapping layers,
bounding boxes, resampling, image quality, SVG bounds and IDs, outlined text,
and simplified strokes. PDF defaults to Medium quality. Bounding-box controls
are rendered only when `DesignStaticExportCapabilities` says the selected
target is eligible; SVG outline/simplify defaults are likewise supplied by the
host instead of inferred from format alone. Color profile, resampling, and
quality open exact selectable menus rather than cycling values.

`DesignAnimatedExportViewData` represents current Figma Motion export. The host
describes whether the selection is an eligible top-level animated frame, the
disabled reason otherwise, available formats, source dimensions, and whether
exports above 1080p or 30 FPS are plan-enabled. MP4 and WebM support the exact
12/24/30/60 FPS matrix, High/Medium/Low quality, and standard 0.5x, 0.75x, 1x,
1.5x, 2x, 3x, and 4x sizing. GIF supports 8/12/15/24/30 FPS, the same sizing,
and loop counts from 0 (forever) through 1000. Animated SVG uses
host-described settings because the Plugin API does not define a stored
animated-SVG settings contract. Changes and execution emit
`AnimatedExportChangeRequested` and `AnimatedExportRequested`; the panel never
mutates the controlled settings.

Add, remove, edit, preview, and export-all actions carry a
`DesignPanelTarget::Page` or `DesignPanelTarget::Nodes` plus stable
configuration IDs where applicable. There is one Export button after all
configuration rows, rather than one execution button per row. A host-eligible
single target can expose the transient preview disclosure; multiple selection
does not. `DesignExportPreviewState` renders Idle, Loading, Ready
(thumbnail/dimensions/estimated output), and Error states without synthesizing
preview data locally. When `can_export` is false, the resolver omits the entire
Export section, including page context.

For an exact ordered multiple selection, the Storybook host projects only
static export configurations that are identical across every selected node and
uses the intersection of their static capabilities. Different per-node rows
project as an empty/mixed fixture; the adapter never substitutes the first
node's mode, rows, capabilities, preview, or Motion settings. Motion export and
preview remain single-target only. Add, remove, and edit requests for a shared
static row are prevalidated against the complete ordered target and then
applied to every selected node atomically. Reordered, missing, permission
invalid, or newly mixed targets are rejected before any node is changed.
Export-all likewise accepts an exact multi-target only when every selected node
still has the same non-empty configuration set.

## Compatibility-only API paths

The following deprecated enum paths remain source-compatible for older host
adapters, but the current panel never renders or canonically emits them.
`DesignPanelProperty::compatibility_path` and
`DesignPanelAction::compatibility_path` classify them at runtime and return a
canonical replacement:

- `AlignmentX`, `AlignmentY`, and `AlignSelection` → the aggregate
  `AutoLayoutAlignment` property.
- `DistributeSelection` → `ArrangeRequested`.
- `EffectSettings` → typed `Effect*` leaf properties through
  `EffectEditRequested`.
- `EffectBlur`, `EffectSpread`, `EffectOffsetX`, and `EffectOffsetY` → the
  exact shadow/blur leaf properties.
- `MediaCropMode` and the seven legacy node media filter properties → typed
  Image/Video `PaintEditRequested` edits.
- `ExportScale` → `ExportSizing`.
- indexed generic layout-guide `PropertyChangeRequested` /
  `PropertyEditRequested` and Layout-grid `CollectionItemRemoveRequested` →
  stable-ID `LayoutGridPropertyEditRequested` and
  `LayoutGridRemoveRequested`.
- `LayoutGridCountVariableApplyRequested` /
  `LayoutGridCountVariableDetachRequested` → exact-field
  `LayoutGridVariableApplyRequested` /
  `LayoutGridVariableDetachRequested`.
- `PaintChangeRequested` → `PaintEditRequested`.
- `PaintColorStyleApplyRequested` / `PaintColorStyleCreateRequested` →
  whole-collection `PaintStyle*Requested` or leaf
  `PaintColorVariable*Requested`, according to the binding level.
- `ReplaceMediaRequested` → `PaintMediaSourceActionRequested` for image/video
  Upload/Make/Edit workflows, or `PaintSourceReplaceRequested` for Pattern
  source selection.
- `ExportRequested` → target-aware `ExportAllRequested`.

Storybook rejects these compatibility-only routes before reducer dispatch, so
examples and tests exercise only the canonical contracts. Compatibility
adapters in the reusable panel remain intentionally isolated from canonical
rendering and are locally allowed to reference the deprecated enum variants
without weakening workspace warning gates.

## Keyboard and controlled-value behavior

- Tab traverses section headers, add/remove/visibility controls, value fields,
  select menus, paint controls, and export buttons.
- Enter or Space activates the focused control.
- Enter commits a property input; Escape cancels and restores the original
  host value.
- Up/Down steps exact valid scalar numeric drafts with the host's Small nudge;
  Shift uses Big. Invalid/blank/Auto/None and nonnumeric drafts propagate.
- Drag an editable uniform numeric readout horizontally to scrub it; Shift is
  coarse and Alt/Option is precise. Moving vertically selects the
  2x/1x/1/2/1/4 band and displays its transient cue. Clicking without crossing
  the drag threshold preserves the ordinary text-input path.
- Select menus open, move, and confirm from the keyboard while remaining
  controlled by the host.
- The auto-layout alignment grid is a single tab stop. Arrow keys and W/A/S/D
  step or set an edge of its visible target respectively. X toggles
  packed/space-between (`Fixed`/`Auto`) spacing, and B toggles horizontal
  baseline alignment when that exact property is editable.
- Relative input such as `+10`, `-8`, `*2`, `/2`, decorated units,
  percentages, and degree values is parsed against the original value. A
  top-level `50%` scales the original value to one half; matching Figma's
  dimension-field behavior, `*50%` multiplies it by 50 and percentage values
  embedded in full arithmetic remain numeric (`200 * 50%` is `10000`).
  Arithmetic also supports precedence, parentheses, unary signs, and
  exponentiation.

Read-only permissions and page/no-selection context gate document-property
mutation intents. Export is gated independently: viewer and page contexts can
configure and request export when `can_export` is true, while restricted
viewers do not see the section.

View-only access has a native UI3 projection instead of rendering the editor's
controls as disabled rows. Its valid navigation set is Comment/Properties;
Properties is the initial controlled surface and Comment uses the documented
host-owned empty projection. Entering view permissions closes editor-only
pickers and popovers so no unmounted interaction remains open.

Hosts supply `DesignViewerPropertiesViewData` for the exact ordered
`DesignPanelTarget`. Its stable sections retain exact labels, multiline
summaries, rows, copy labels, and clipboard payloads. Text fixtures use a
Content section and a Typography section whose host-provided `Copy all`
payload can differ from the visible rows. A row mapped to a canonical
`DesignPanelProperty` reuses
`PropertyCopyRequested { target, property, displayed_value }`; arbitrary
viewer-only rows remain inspectable and copy through their containing section.
Section Copy emits
`ViewerSectionCopyRequested { target, section_id, copy_value }`. The reusable
panel never derives text characters or writes to the clipboard.

For component-aware nodes, the Storybook host prepends a stable `component` or
`instance` section according to `DesignComponentRole`. Its rows preserve Role,
Main component, Local or named-library origin, Available/Missing/reasoned
Unavailable state, description, and documentation links in their supplied
order. Context links precede any links carried by the resolved main-component
reference. `Copy all` carries the exact newline-delimited label/value payload
shown by those rows, and missing main references remain explicit rather than
dropping the section. Component, Component set, Instance, and Slot fixtures
all provide meaningful descriptions and documentation; the Component Label
property also demonstrates property-level documentation. URLs remain inert
host-owned strings: neither the reusable panel nor the Storybook opens them.

Figma's view-only Stroke projection is named Borders. A Borders section may
carry one host-owned `DesignViewerColorRepresentation`: CSS, Hex, RGB, HSL, or
HSB. Choosing another value emits
`ViewerSectionRepresentationChangeRequested`; the panel keeps showing the
current snapshot until the host echoes newly formatted rows and a new exact
section-copy payload. Stale target, section, row, and copy-value snapshots
cannot emit. Restricted viewers still see the supplied readouts but have no
copy or representation controls and emit no copy-adjacent intent.

Editor mode uses the Design/Prototype surface set. Design owns the inspector
and export projection; Prototype is host-owned. Export configuration remains
governed independently by `can_export`.

## Reference surface

The implementation is checked against Figma's current first-party guidance for
the [right sidebar](https://help.figma.com/hc/en-us/articles/360039832014-Design-prototype-and-explore-layer-properties-in-the-right-sidebar),
[property categories](https://help.figma.com/hc/en-us/articles/15297425105303-Explore-design-files),
[dimension-field calculations](https://help.figma.com/hc/en-us/articles/360041539473-Frames-in-Figma-Design),
[auto layout](https://help.figma.com/hc/en-us/articles/360040451373-Explore-auto-layout-properties),
[wrapped-track distribution](https://developers.figma.com/docs/plugins/api/properties/nodes-counteraxisaligncontent/),
[nullable wrapped-track spacing](https://developers.figma.com/docs/plugins/api/properties/nodes-counteraxisspacing/),
[grid auto layout](https://help.figma.com/hc/en-us/articles/31289469907863-Use-the-grid-auto-layout-flow),
[component properties](https://help.figma.com/hc/en-us/articles/5579474826519-Explore-component-properties),
[slots](https://help.figma.com/hc/en-us/articles/38231200344599-Use-slots-to-build-flexible-components-in-Figma),
[SlotSettings](https://developers.figma.com/docs/plugins/api/SlotSettings/),
[SlotNode children and violations](https://developers.figma.com/docs/plugins/api/SlotNode/),
[the Slots GA Plugin API update](https://developers.figma.com/docs/plugins/updates/2026/06/10/update/),
[viewer text properties](https://help.figma.com/hc/en-us/articles/360039956634-Explore-Text-Properties),
[fills](https://help.figma.com/hc/en-us/articles/360041003694-Guide-to-fills),
[color picker](https://help.figma.com/hc/en-us/articles/360041003774-Update-fills-using-the-color-picker),
[gradients](https://help.figma.com/hc/en-us/articles/34208860210199-Use-gradients-as-a-fill-or-stroke),
[strokes](https://help.figma.com/hc/en-us/articles/360049283914-Apply-and-adjust-stroke-properties),
[the canonical eight-value StrokeCap API](https://developers.figma.com/docs/plugins/api/StrokeCap/),
[ComplexStrokeProperties](https://developers.figma.com/docs/plugins/api/ComplexStrokeProperties/),
[VariableWidthStrokeProperties](https://developers.figma.com/docs/plugins/api/VariableWidthStrokeProperties/),
[HandleMirroring](https://developers.figma.com/docs/plugins/api/HandleMirroring/),
[per-vertex `cornerRadius`](https://developers.figma.com/docs/plugins/api/properties/nodes-cornerradius/),
[native text-on-path placement and Flip orientation](https://help.figma.com/hc/en-us/articles/360039956434-Guide-to-text-in-Figma-Design),
[TextPathStartData](https://developers.figma.com/docs/plugins/api/TextPathStartData/),
[TextPathNode](https://developers.figma.com/docs/plugins/api/TextPathNode/),
[ArcData](https://developers.figma.com/docs/plugins/api/ArcData/),
[ellipse Start, Sweep, and Ratio in Appearance](https://help.figma.com/hc/en-us/articles/31130312631191-FD4B-Turn-an-ellipse-into-an-arc),
[Polygon Count in Appearance](https://help.figma.com/hc/en-us/articles/26620239826199-Layers-101-Explore-layer-types),
[PolygonNode](https://developers.figma.com/docs/plugins/api/PolygonNode/),
[StarNode](https://developers.figma.com/docs/plugins/api/StarNode/),
[Boolean operations](https://help.figma.com/hc/en-us/articles/360039957534-Boolean-operations),
[the dedicated Mask section](https://help.figma.com/hc/en-us/articles/360040450253-Masks),
[SectionNode](https://developers.figma.com/docs/plugins/api/SectionNode/),
[section organization and sharing](https://help.figma.com/hc/en-us/articles/9771500257687-Organize-your-canvas-with-sections),
[TransformModifier](https://developers.figma.com/docs/plugins/api/TransformModifier/),
[repeat transforms](https://help.figma.com/hc/en-us/articles/31440427042839-Create-patterns-with-transforms),
[effects](https://help.figma.com/hc/en-us/articles/360041488473-Apply-effects-to-layers),
[typography](https://help.figma.com/hc/en-us/articles/360039956634-Explore-text-properties),
[the nullable node-level `maxLines` contract](https://developers.figma.com/docs/plugins/api/properties/nodes-maxlines/),
[layout guides](https://help.figma.com/hc/en-us/articles/360040450513-Create-layout-guides),
[the atomic GridStyle contract](https://developers.figma.com/docs/plugins/api/GridStyle/),
[layout-grid variable-bindable fields](https://developers.figma.com/docs/plugins/api/VariableBindableLayoutGridField/),
[the Variables API](https://developers.figma.com/docs/plugins/api/figma-variables/),
[PageNode and its one-solid-background contract](https://developers.figma.com/docs/plugins/api/PageNode/),
[resolved variable modes](https://developers.figma.com/docs/plugins/api/properties/nodes-resolvedvariablemodes/),
[setting an explicit collection mode](https://developers.figma.com/docs/plugins/api/properties/ExplicitVariableModesMixin-setexplicitvariablemodeforcollection/),
[clearing an explicit collection mode](https://developers.figma.com/docs/plugins/api/properties/ExplicitVariableModesMixin-clearexplicitvariablemodeforcollection/),
[variable collections and mode IDs](https://developers.figma.com/docs/plugins/api/VariableCollection/),
[Figma's style APIs](https://developers.figma.com/docs/plugins/api/figma/),
[static export](https://help.figma.com/hc/en-us/articles/360040028114-Export-static-designs-from-Figma),
[the current ExportSettings contract](https://developers.figma.com/docs/plugins/api/ExportSettings/),
and [Figma Motion export](https://www.figma.com/blog/introducing-figma-motion/).
