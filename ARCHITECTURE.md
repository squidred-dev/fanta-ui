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

The workspace owns the framework, platform backends, and component layers:

```text
fanta-gpui-storybook ──► fanta-gpui ──► fanta-gpui-components ──► fanta-gpui-core
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

`atoms::SemanticColor` is the application-facing color vocabulary. Its stable
UI3-style roles group border, background, icon, and text intent while resolving
from the active `gpui-component` theme at render time. Components use semantic
roles when the meaning crosses component boundaries; document and canvas
colors remain controlled host data.

`atoms::TypographyToken` supplies the UI3 display, heading, and body hierarchy;
`TypographyExt::typography` applies the complete size, line-height, and weight
recipe while inheriting the host theme's font family. `ui_button` and
`semantic_button` are the construction boundary for action controls, mapping
the shared default/large sizes and purpose variants onto `gpui-component`
behavior. Production components do not construct raw buttons or select ad-hoc
text-size utilities.

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
activation matrix, truncation, context menus
with two-axis clamping, list rows, and anchored popups with edge fades —
each wired into the shared intent log. The Atoms section also includes the
Lucide icons page covering every `IconName` provided by the installed
`gpui-component` asset bundle. Fanta-owned controls use `LucideIcon` and
`render_lucide_icon`, backed by one pinned upstream release rather than a
parallel glyph set. The Getting started Welcome story renders the tier
map itself. Reusable Screens are complete application-sized workflows; each
Storybook screen module remains a mock host adapter that feeds data into the
reusable tier it demonstrates.

Stories are registered in one `StoryDescriptor` table
(`storybook/src/screens/mod.rs`). The registry is the single source for
sidebar grouping into the six fixed sections — Getting started, Atoms,
Molecules, Organisms, Layouts, Screens — plus render/focus/last-intent dispatch,
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

- `unsafe` code is forbidden in Fanta components and the storybook. Imported
  framework, platform, and supporting crates retain their upstream safety
  policies, including native FFI.
- Public intent and prop types should implement useful comparison/debug traits.
- `cargo fmt --all -- --check`,
  `cargo test --workspace`, and
  `cargo clippy --workspace --all-targets -- -D warnings` must pass. Imported
  crates retain explicit upstream lint policies. No command depends on a
  sibling editor checkout.
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
presentation state. Compact panel rows use the same themed background
treatment as hover/selection so focus never adds a second inset border;
controls that use a focus ring reserve its border width so keyboard focus
never shifts layout. Command actions remain public so a host can replace key
bindings or expose them in its own command palette.

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

Expanded Pages and Layers headers follow the application sidebar treatment:
the title is semibold, disclosure chevrons appear only while a whole section
is collapsed, and hover/focus use active-theme sidebar backgrounds rather than
adding inset borders. Layer-kind, visibility, and lock glyphs use canonical
Lucide assets or Lucide's official 24-unit path geometry with its round-capped
two-unit stroke; the panel does not invent a second icon language.

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

`DesignPanel` is a controlled, host-facing inspector façade. Its canonical
input is one `DesignPanelViewData` snapshot containing the inspection context,
navigation, preferences, target-bound projections, resource catalogs, and
property states. Existing constructors and granular setters remain compatibility
adapters; `DesignPanelAction` remains the sole document-facing output.

The inspection context is authoritative for selection identity, order,
permissions, parent layout, edit mode, and inspected nodes. Reusable code owns
only transient presentation state. Continuous changes use one balanced
`Begin -> Preview* -> Commit | Cancel` transaction, and context or capability
changes cancel stale transactions before their local drafts are cleared.

Implementation dependencies flow in one direction:

```text
host snapshot
  -> DesignPanel façade
    -> feature projection/controller
      -> molecules::inspector
        -> atoms + gpui-component
```

Shared inspector molecules are domain-neutral and use the active GPUI theme,
existing density metrics, and the Lucide icon pipeline. Anchored menus/popovers
and modal tasks remain distinct interaction categories. One overlay coordinator
defines deterministic Escape dismissal and context-change cleanup.

The observable behavior, host data contracts, section matrices, property/action
mapping, compatibility paths, and Storybook reference fixtures are specified in
[docs/design-panel.md](docs/design-panel.md). The decomposition rules, state
ownership, component taxonomy, migration boundary, and slice acceptance checks
are specified in
[docs/design-inspector-architecture.md](docs/design-inspector-architecture.md).

Architecture tests enforce the reusable-crate dependency boundary, prevent
Design-domain imports in shared inspector molecules, and prevent newly
extracted section modules from extending the façade with inherent impl blocks.

## §12 Editor toolbar integration contract

`EditorToolbar` is a stateful, intrinsic editor-chrome dock. It renders its
persistent controls as one contained, content-sized surface; it does not claim
full-canvas bounds, choose viewport coordinates, or apply its own outer
positioning. A host supplies the active mode, selected tool, zoom, the
Dev/Motion option read models, and its chrome controls. The toolbar emits
`ToolbarAction` intents and never creates layers, changes a selection, runs a
command, advances a timeline, or invokes an AI service.

Host-controlled state:

- the active Design, Motion, or Dev mode;
- the selected primary tool and accepted mode-specific control values;
- canvas zoom, Dev readiness, and Motion transport state;
- every candidate the Motion option editor may offer (the animation-style
  catalog) — the toolbar presents these and never invents a value outside
  them;
- Agent context copy and suggestions, plus the allowed command subset/order;
- the host chrome controls shown in the dock's trailing capsule
  (`ToolbarChromeControl`: id, `IconName`, label, shortcut hint, and the
  `active` flag), supplied via `set_chrome_controls` — fit-to-view and
  sidebar toggles are host chrome, so the toolbar renders them but only ever
  reports `ChromeControlInvoked { id }`; the host applies the effect and
  echoes any new `active` state;
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
Persistent primary, secondary, mode, zoom, Agent, and chrome controls remain
inside the dock; constrained hosts may let the dock's internal rows scroll.

The dock occludes the pointer: hosts float it over the canvas, so every mouse
event on the dock surface — clicks, presses, hovers, wheel — stops there and a
tool press never also reaches the canvas beneath. The dock's own overflow rows
sit above that surface and keep scrolling; transient popups block on their own
surfaces. Tool and mode icons are Lucide: gpui-component `IconName` assets
where a fitting one exists (resolved against the host asset source, §5) and
stroke paths authored on Lucide's 24-unit grid with its round-capped 2-unit
stroke otherwise, so assets and drawings read as one set.

Pointer controls, Enter/Space activation, and the command actions registered by
`fanta_gpui::init` converge on the same methods and typed intents. Hosts accept
an intent by calling `set_mode`, `set_active_tool`, `set_zoom_percent`, or the
appropriate options setter with fresh controlled data. The interactive
storybook is the mock host and is the only layer that applies those requests.
The host owns the dock's outer placement, canvas inset, and surrounding canvas
clipping; the toolbar owns collision handling for its transient surfaces.

## §13 Variables screen, prototype, and timeline contracts

`VariablesScreen`, `PrototypePanel`, and `Timeline` follow the same controlled
seam as the established editor surfaces. Hosts supply collections, prototype
settings, and transport values as immutable view data. Every operation that
can affect a document, prototype, or animation emits a typed intent.

The components may retain only presentation continuity: search drafts, scroll
position, dismissed educational hints, focus, and open/closed empty-state
guidance. They do not create variables, change prototype settings, add
keyframes, seek, or run an agent.

The Variables screen also owns the optional mode-scope bar and selected-layer
binding panel. `VariablesContextData` supplies opaque scope, node, property,
mode and variable identifiers with host-filtered choices. `VariablesContextAction`
returns mode and binding selections; the host validates and applies them through
its document operations. Both panels use the shared Dropdown atom and popup
molecules. The binding panel owns only its left border; the screen header owns
the horizontal separator. Creating a first variable requests the host to create
a collection and default mode when none exists.

`color_picker::ColorPicker` is a reusable organism with an RGBA-only public
contract (`PickerColor`, `ColorPickerAction`, and `ColorPickerPhase`). It owns
the retained editor extracted from Design; the inspector’s richer paint adapter
uses that same implementation. Hosts supply accepted colors, while spectrum,
hue, alpha and text previews remain transient until commit. Closing cancels
unfinished edits. The standalone `color-picker` story echoes commits and offers
opaque and translucent fixtures.

Variable cells mount this component in a trigger-anchored popup and translate
only committed colors into `VariablesAction::ValueChanged`; they no longer
construct Design paint targets. Variable dropdowns use the shared popup chrome
and window clamping, anchored to measured trigger bounds. Cell text uses the
body type token and inline editors use the compact field height.

## §14 Layout composition contracts

`FileInspectorSidebar` places the existing `PagesPanel` above the existing
`LayersPanel` in one left-sidebar surface. Pages keeps its compact, intrinsic
section height and Layers receives the remaining height. Hosts supply the project
name. The composition owns transient collapse state and emits
`FileInspectorAction::CollapsedChanged`; hosts may also set that state explicitly.
Collapsed presentation keeps a themed floating card with the Fanta mark, project
name and reopen control. It does not intercept either child's document intents;
hosts continue subscribing to the original Pages and Layers entities. Dedicated
child panel entities render borderless, and the layout draws internal separators
only. The containing host owns the sidebar's outer edge.

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
without turning it into a document host. Pages, layers, design,
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

The Design inspector is being decomposed without changing that facade. Its
dependency direction is `DesignPanel` facade → section projections/controllers
→ domain-neutral `molecules::inspector` components. Projections reduce the
controlled Design read model to one section's view data; controllers translate
generic molecule intents into exact-target `DesignPanelAction` values. Shared
molecules own only reusable layout and transient interaction continuity and
must not import the Design facade, Design-prefixed types, or the crate prelude.

Existing inherent `impl DesignPanel` files below `organisms/design/panel/` are
an explicit legacy migration boundary. New extracted sections belong below
`panel/sections/` and use standalone projection/controller types or functions;
they do not extend the facade. Migrate one vertical section at a time, preserve
the public API and selectors, and remove its legacy exception as soon as the
facade delegates to it. The dependency contract and per-section acceptance
gates are recorded in
[`docs/design-inspector-architecture.md`](docs/design-inspector-architecture.md);
feature behavior remains in [`docs/design-panel.md`](docs/design-panel.md).

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
through its public facade. It also prevents `molecules::inspector` from
importing Design-domain APIs and prevents newly extracted section modules from
adding inherent `DesignPanel` implementations while the enumerated legacy
modules are migrated.

## §16 Atomic design tiers

The library source is organized by atomic design tier:

```text
crates/fanta-gpui/src/
  atoms/        activation, buttons, pinned Lucide icons, truncation,
                bounds tracking, shared geometry scale
  molecules/    inspector fields/sections, menu chrome + clamping, anchored
                popups, list rows, edge fades
  organisms/    design, layers, pages, prototype, timeline, toolbar —
                the host-facing feature surfaces
  layouts/      file_inspector, pseudo_editor — composition shells that
                arrange organisms
  screens/      variables — complete application-sized workflows
```

Organism, layout, and screen modules are re-exported at the crate root, so
hosts import `fanta_gpui::file_inspector` or `fanta_gpui::variables`, never a
tier path. The atoms and molecules tiers are
curated public API: hosts (and the storybook) import `fanta_gpui::atoms` and
`fanta_gpui::molecules` — or the prelude — to build custom chrome that shares
the library's activation, focus-ring, icon, and clamping contracts, while
drawing internals stay crate-private.
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
- `LucideIcon` plus `render_lucide_icon` is the sanctioned §5 icon source for
  Fanta-owned controls. The renderer consumes complete SVG geometry from the
  exact `lucide-static-svg` version pinned in `Cargo.toml`; toolbar, layer,
  variable, timeline, and inspector icons only provide semantic mappings.
  Unicode characters, custom stroke paths, and hand-assembled div art are not
  icons. A host-defined design-header control may select any `LucideIcon`, but
  may not inject a platform glyph.
- `track_bounds` and `truncating_label` replace hand-rolled measurement
  canvases and character-count width heuristics; labels truncate rather than
  forcing horizontal scroll extents.
- `atoms::tokens` is the shared geometry scale for fixed chrome dimensions:
  padding and gap steps, row heights, control hit targets, Fanta-owned corner
  radii, icon glyph sizes, menu widths, and density breakpoints. It covers
  exactly the fixed dimensions §5 accepts for intentional editor chrome
  geometry; color, the body font, and surface radius stay with `Theme`, and a
  token that would restate a theme value does not belong in the module. A
  feature constant derived from a token spells the derivation out
  (`2. * Space::SM` for a `p_2` surface) rather than naming the step that
  happens to equal it today.
- `controls::test_support` mounts any intent-emitting component behind a
  recording probe host and asserts the §9 pointer/Enter/Space parity matrix
  against a control's debug selector. Every interactive control carries a
  systematic `debug_selector` so interaction tests target controls uniformly.

New feature work must consume this layer; adding a bespoke control stanza,
menu implementation, or glyph icon to a feature module is an architecture
violation unless this section records why the shared piece cannot serve it.

## §17 Slider family

`atoms::{SliderBackground, SliderHandle, SliderGradientStop}` are theme-aware
presentation primitives. `molecules::Slider` composes them into a normalized,
host-controlled control with range, stepped, centered, reference-marker, hue,
and opacity variants. The host echoes accepted values through `set_value`;
only pointer drafts, focus, and measured bounds live in the component.
`SliderAction` emits balanced Begin/Preview/Commit/Cancel phases, with arrows,
Shift-arrows, Home/End, and Escape sharing the same value constraints.

The existing gpui-component slider exposes Change events and owns its accepted
value. This molecule uses GPUI primitives to preserve our controlled phased-edit
contract and support custom color rails. It uses active theme colors and shared
`SliderGeometry` tokens. Rounded layers paint their own corners, and handle
travel stays inset from both ends. Gradient stops use the shared activation
contract; their host owns positioning and drag transactions.

The Color picker and Design paint adapter use these same atoms and slider
molecule for spectrum handles, hue, opacity, gradient backgrounds and stops.
Four interactive stories cover Slider, Slider background, Slider handle, and
Slider gradient stop in the gallery's existing theme/viewport harness. The UI3
Figma references (2015:23280, 2015:23271, 2015:23235, 2015:23409) inform variant
and state coverage; Fanta's GPUI theme and geometry remain authoritative.

## §18 Framework ownership and publication

This workspace owns the GPUI fork and its complete local dependency closure.
Package names and provenance are recorded in `docs/extraction/packages.json`;
Rust library names and dependency aliases preserve the existing imports. All
published local dependencies carry an exact coordinated registry version plus
a repository-local path. No published crate requires root-level patches, a
sibling editor checkout, or a Fanta engine crate.

The framework's native implementations retain their FFI and upstream safety
policies. The component and storybook unsafe-code prohibition remains intact.
The imported libraries retain their per-crate licenses; the workspace license
is not imposed on Apache-licensed framework code.

The unpublished framework-example and macro-test hosts break development-only
publication cycles while retaining example and macro coverage. The standalone
registry consumer verifies actual archives independently of workspace feature
unification. `docs/releasing.md` defines the release and integration gates.

This extraction deliberately leaves `fanta_ui` and `fig_viewer` presentation in
Fanta Edit. Their existing adapters continue to provide controlled view data and
apply typed intents. Further screen extraction must preserve the engine seam.

### Context-menu availability

Layers accept an optional per-node `context_actions` allowlist. The component
intersects it with kind-specific entries, removes empty sections and emits
only typed intents. Services specific to another design application are not
menu entries. The host remains responsible for revalidating document state
when receiving an event. Pages expose typed directional move intents and
host-controlled edit/link availability; boundary moves and last-page deletion
are disabled before emission. See `docs/CONTEXT-MENU-AUDIT.md` for the native
Figma comparison and mapping of engine-independent presentations.
