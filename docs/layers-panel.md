# Layers panel

`LayersPanel` is a headless, host-controlled GPUI component modeled after the
Figma Layers panel. It renders hierarchy and interaction state, then reports
typed intent. It does not own or mutate a Fanta document.

The implementation was matched against the repository's Figma reference
specimens: 28px tree rows, 16px indentation, type glyphs, disclosure chevrons,
selected and hover states, persistent hidden/locked indicators, and the
sectioned dark contextual menus used by different node capabilities.

## Read model

`LayersPanelItem` contains:

- `id`: an opaque host identifier;
- `title`: the displayed layer name;
- `kind`: a UI-facing `LayersPanelNodeKind`;
- `children`: ordered recursive read data;
- `visible` and `locked`: host-provided state.

The node-kind enum covers:

- containers: frame, group, section, mask;
- component data: component, component set, instance;
- content: text, image, video;
- shapes: rectangle, ellipse, polygon, star, line, arrow;
- vector data: vector, boolean operation, pen, pencil;
- export and fallback data: slice and other.

Use `set_nodes` after any host mutation. Use `set_selected_node_ids` and
`set_expanded_node_ids` when document selection or expansion changes outside
the panel.

## Typed intents

`LayersPanelAction` separates UI interaction from host operations:

| Intent | Host responsibility |
| --- | --- |
| `SelectRequested` | Apply replace, toggle, or range selection |
| `ExpansionChanged` | Accept or replace one disclosure change |
| `CollapseAllRequested` | Store the collapsed expansion model |
| `RenameRequested` | Rename the referenced document node |
| `VisibilityChanged` | Apply visibility and refresh the tree |
| `LockChanged` | Apply lock state and refresh the tree |
| `ContextActionRequested` | Route the node-specific operation |

Right-clicking an unselected row first emits replace selection. Rename,
Show/Hide, and Lock/Unlock use their dedicated intents; other menu entries use
`ContextActionRequested`.

## Context menus

Every menu includes copy/paste, Figma Make, similarity, motion, page/z-order,
plugin/widget, Show/Hide, and Lock/Unlock sections when the operation is
meaningful. The structural sections vary by node capability:

| Node profile | Additional actions |
| --- | --- |
| Frame | Convert to section, remove frame, selection grouping, thumbnail, vector conversions, layout |
| Group | Convert to frame/section, ungroup, selection grouping, thumbnail, vector conversions |
| Section | Convert to frame, rename, thumbnail |
| Component / component set | Selection grouping, thumbnail, vector conversions, layout |
| Instance | Main component, detach, reset overrides |
| Text | Edit text, selection grouping, flatten, outline stroke, mask |
| Image | Crop, replace media, selection grouping, flatten, mask |
| Video | Replace media, selection grouping, mask |
| Shape / vector | Selection grouping, flatten, outline stroke, mask, create component |
| Mask | Ungroup, selection grouping, flatten |
| Slice | Rename operations only in the structural section |

Entries that represent a deeper Figma submenu show a chevron and emit a typed
action. The application host chooses the next UI surface.

## Keyboard and pointer behavior

- Click: replace selection.
- Command-click (Control-click off macOS): toggle selection.
- Shift-click: range selection.
- Double-click: rename.
- Disclosure chevron: expand or collapse.
- Secondary click: open the node-specific context menu.
- Enter or Space: activate the focused row or control.
- Control-Enter: open the focused row's context menu.
- Escape: close a menu or cancel a rename draft.
- Tab and Shift-Tab: traverse the collapse control, rows, inline controls, and
  menu items.

The same methods back pointer and keyboard activation, so accessibility does
not bypass the typed-intent boundary.

## Storybook

Run:

```sh
cargo run -p fanta-gpui-storybook
```

Use the top navigation to switch between Pages and Layers. The Layers story
contains nested examples of every node-kind profile, applies typed intents to
mock host data, and shows the latest intent on the right.

## Verification

The component test suite covers:

- recursive flattening and indentation;
- every supported node kind's common menu safety actions;
- capability differences for frame, section, instance, text, and image menus;
- selection modifiers;
- expansion and Collapse all;
- visibility and lock controls;
- contextual menu rendering and action routing;
- inline rename;
- keyboard context-menu access and Escape dismissal.

Repository gates remain:

```sh
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```
