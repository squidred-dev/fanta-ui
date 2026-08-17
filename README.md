# fanta-gpui

Engine-decoupled, host-controlled GPUI components for Fanta applications.

This repository owns reusable UI composition only. Fanta Engine remains the
source of truth for documents, selections, history, and mutations. Components
receive plain view data and emit typed intents that the host maps to Fanta
operations.

The project uses “headless” for that document/engine boundary. Components are
intentionally styled GPUI renderers; they use the active theme and own transient
presentation state such as focus, drafts, menus, and animation.

The visual layer is built on
[`gpui-component`](https://longbridge.github.io/gpui-component/gallery/).

## Repository shape

- `crates/fanta-gpui` — the reusable component library and facade.
- `crates/fanta-gpui-storybook` — a small desktop gallery for developing
  components in isolation.

The library source is organized by atomic design tier (`ARCHITECTURE.md` §16):

```text
crates/fanta-gpui/src/
  atoms/        activation, buttons, vector icons, truncation, bounds tracking
  molecules/    menu chrome + clamping, anchored popups, list rows, edge fades
  organisms/    assets, design, layers, pages, prototype, timeline, toolbar,
                variables
  layouts/      pseudo_editor
```

Organism and layout modules are re-exported at the crate root, so hosts import
`fanta_gpui::pages`, never a tier path. The atoms and molecules tiers are
curated public API — hosts build custom chrome from `fanta_gpui::atoms`,
`fanta_gpui::molecules`, or the prelude.

## Run the storybook

```sh
cargo run -p fanta-gpui-storybook
```

The Gallery sidebar mirrors the atomic tiers — Getting started, Atoms,
Molecules, Organisms, Layouts — with per-atom and per-molecule specimen
stories (buttons and activation, vector icons, truncation, menus, list rows,
popups and edge fades) beside the searchable Icon assets catalog of every
bundled `gpui-component` icon. Use **Open window** on any story to keep the
Gallery and its mock data in place while opening that component in its own
maximized, full-window surface.

## Validate the workspace

```sh
cargo fmt -p fanta-gpui -p fanta-gpui-storybook -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Scope `cargo fmt` to the workspace crates: `--all` would also reformat the
patched `gpui`/`gpui-component` sources in the sibling `fanta-edit` checkout.

The test suite includes executable dependency-boundary checks, model tests,
real GPUI pointer/keyboard interaction tests, and a composed mock-host workflow
that feeds Toolbar and Timeline intents back into controlled Pseudo Editor and
Design Panel state. The latter is an in-process GPUI integration test; a
production application remains responsible for its own engine adapter and
launched-application E2E coverage.

## Use the Pages panel

Initialize `gpui-component` and `fanta-gpui` once in the host, create the panel
as a GPUI entity, subscribe to its typed events, and render the entity:

```rust
use fanta_gpui::prelude::*;
use gpui::{App, Context, Entity, SharedString, Subscription, Window};

fn init(cx: &mut App) {
    gpui_component::init(cx);
    fanta_gpui::init(cx);
}

struct Editor {
    pages_panel: Entity<PagesPanel>,
    _pages_subscription: Subscription,
}

impl Editor {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let pages = doc_pages()
            .map(|page| PagesPanelItem::new(page.id.to_string(), page.name))
            .collect();
        let pages_panel =
            cx.new(|cx| PagesPanel::new("document-pages", pages, window, cx));

        let _pages_subscription =
            cx.subscribe(&pages_panel, |editor, panel, action, cx| {
                editor.handle_pages_action(panel, action, cx);
            });

        Self {
            pages_panel,
            _pages_subscription,
        }
    }

    fn handle_pages_action(
        &mut self,
        panel: Entity<PagesPanel>,
        action: &PagesPanelAction,
        cx: &mut Context<Self>,
    ) {
        // Match SelectRequested/CreateRequested/RenameRequested/
        // SearchRequested/ReplaceRequested, apply the corresponding Fanta
        // operation, then feed updated read models back through
        // panel.update(... set_pages/set_selected_page/set_search_results).
        let _ = (panel, action, cx);
    }
}
```

`PagesPanel` currently includes:

- an animated collapsible page list whose Search and Add controls remain
  available while collapsed;
- focused add/rename editing, hover, selection, and double-click rename;
- a platform context menu for copy-link, rename, duplicate, and delete intents;
- Find and Replace modes with removable element-filter pills;
- current-page/all-pages scope and previous/next result navigation;
- registered GPUI commands, shortcut-aware tooltips, and Tab/Shift-Tab
  navigation with Enter/Space activation;
- host-controlled result rows and typed replace intents.

Page and result data remain controlled. The component owns only presentation
state that needs continuity between GPUI frames:

```rust
panel.update(cx, |panel, cx| {
    panel.set_pages(new_pages, cx);
    panel.set_selected_page(Some(active_page_id.into()), cx);
    panel.set_search_results(results, cx);
});
```

Search data uses small read-model types (`PagesPanelSearchRequest`,
`PagesPanelSearchResult`, and `PagesPanelSearchResults`). A host can translate
them without exposing its document schema:

```rust
if let PagesPanelAction::SearchRequested(request) = action {
    let results = search_index.query(&request);
    panel.update(cx, |panel, cx| {
        panel.set_search_results(results, cx);
    });
}
```

The panel deliberately does not depend on `fanta-engine`. Its `page_id` is an
opaque `SharedString`, so a Fanta host can use the canonical `PageId` string
without coupling the UI library to engine internals.

## Use the Layers panel

`LayersPanel` accepts a recursive, immutable layer tree and emits typed intents
for selection, disclosure, rename, visibility, lock, and contextual actions:

```rust
use fanta_gpui::prelude::*;

let layers = vec![
    LayersPanelItem::new("frame-1", "Checkout", LayersPanelNodeKind::Frame)
        .children(vec![
            LayersPanelItem::new("title-1", "Title", LayersPanelNodeKind::Text),
            LayersPanelItem::new("image-1", "Hero", LayersPanelNodeKind::Image),
        ]),
];

let panel = cx.new(|cx| LayersPanel::new("document-layers", layers, window, cx));
cx.subscribe(&panel, |host, panel, action: &LayersPanelAction, cx| {
    // Apply the host operation, then call set_nodes/set_selected_node_ids/
    // set_expanded_node_ids with fresh read models.
    host.handle_layers_action(panel, action, cx);
});
```

The panel includes Figma-like nested rows, vector node-kind icons,
multi-selection modifiers, arrow-key tree navigation, disclosure, inline
rename, hover lock/visibility controls, and
capability-specific menus for containers, components, instances, text, media,
shapes, vectors, sections, slices, masks, and fallback nodes. See
[`docs/layers-panel.md`](docs/layers-panel.md) for the complete contract and
menu matrix.

## Use the Design panel

`DesignPanel` is a controlled, Figma-like inspector for node properties,
auto-layout variations, typography, media, geometry, component/instance
properties, paints, effects, layout grids, and exports:

```rust
use fanta_gpui::prelude::*;

let node = DesignPanelNode::new("card-1", "Card", DesignPanelNodeKind::Frame)
    .with_layout_mode(DesignLayoutMode::Vertical);
let panel = cx.new(|cx| DesignPanel::new("document-design", node, window, cx));

cx.subscribe(&panel, |host, panel, action: &DesignPanelAction, cx| {
    host.apply_inspector_intent(action);
    panel.update(cx, |panel, cx| {
        panel.set_node(host.current_inspector_read_model(), cx);
    });
});
```

Editable hosts can also echo the orthogonal
`DesignPanelWorkspaceMode::Draw` projection. It preserves the accepted
Design/Prototype surface while presenting Draw's merged Position controls,
Flow-first Layout, conservative section set, and slider-based Appearance.
Opacity has a fixed percentage range; hosts bind the corner-radius range to
the exact ordered `DesignPanelTarget` with
`DesignDrawAppearanceViewData`. See
[`docs/design-panel.md`](docs/design-panel.md#design-and-draw-workspaces).

Typography uses typed line-height/tracking units, leading trim, indentation,
`listSpacing`, alignment, and the current underline/strikethrough style,
offset, thickness, color, and skip-ink fields. OpenType features are exact
host-supplied records—tag, name, font default, current value, availability, and
preview—so unknown future tags remain opaque. Numeric `fontWeight` is a
separate controlled/variable-bindable leaf from String `fontStyle`; Family and
Style expose their own variable controls in the searchable font header. The
font browser distinguishes loading, imported, importable, missing, and
unavailable fonts. Text resize stays separate from truncation and maximum lines, and
typography intents distinguish whole-layer edits from the active text range.
Type settings are presented in an anchored, dismissible
Basics/Details/Variable popover.

Numeric inspector fields remain fully controlled while matching Figma's input
mechanics: hosts supply validated Small/Big nudge values (default `1 / 10`),
Up/Down and Shift+Up/Down step only exact valid scalar drafts, and horizontal
scrubbing supports deterministic vertical `2x`, `1x`, `1/2`, and `1/4` speed
bands with a transient cue. The Storybook exposes both the default preference
and a `0.5 / 8` custom example.

Vector-edit contexts accept host-owned stable vertex IDs, coordinates,
per-vertex radii, topology, selection, and exact handle-mirroring values.
Contextual Vector and TextPath controls emit phased stable-ID intents and never
mutate the supplied vector network; mixed, branch, viewer, and read-only states
are gated explicitly.

Effects use stable host IDs for edits, removal, and drag reordering. Their
single anchored settings popup covers all current native effect families plus
structured Shader data and an opaque future-effect fallback. Shader text,
geometry, color-point, and gradient leaves use typed phased edits keyed by
definition ID; host-owned asset/variable choosers and exact variable detach
intents cover resource values without synthesizing IDs. Hosts also supply
per-node kind availability, spread/transparency capabilities, page/library
Effect styles, style bindings, and compatible effect variables.
Layer/paint blend, Fixed/Hug/Fill, stroke-position, and effect option menus
also emit balanced non-mutating hover previews with exact node, effect, and
paint identities; commits occur only after the matching preview is cleared.

The interactive story includes every modeled node kind; editable, view-only,
and restricted page/single/multiple contexts; text-edit, vector-edit,
auto-layout-child, and grid-child contexts; freeform/horizontal/vertical/
horizontal-wrap/grid layout presets; and every paint classification and
gradient subtype. Multiple-viewer Properties deliberately stays aggregate/
empty instead of projecting the first selected node. Fill and stroke swatches
open the anchored, reusable color/gradient/pattern/media/shader picker. Paints carry
stable host IDs and a discriminated payload for bindings, transforms,
canonical pattern settings, image/video placement and filters, structured
shader definitions/values, and opaque unsupported data.
View-only scenarios use the native Comment/Properties projection with
host-owned Text Content, section Copy/Copy all payloads, and Borders
CSS/Hex/RGB/HSL/HSB representation echoes; restricted viewers remain
inspectable and non-copyable.
Media crop sessions and video playback are controlled view data with typed
stable-paint intents; preview state never becomes document paint semantics.
Typed phased edit and reorder intents keep an open picker attached to the same
paint after host reordering. Stroke paints share one
node-level weight/side, position, explicit dash mode, endpoint, join, lossless
variable-width, and Basic/Stretch/Scatter/Dynamic/opaque complex-stroke record,
matching Figma's inspector ownership. See
[`docs/design-panel.md`](docs/design-panel.md) for the complete capability
matrix and ownership contract.

For multiple selections, ordinary property, typography, component,
collection, paint, effect, style, and variable leaves are emitted inside a
`TargetedNodeActionRequested` envelope. Its `DesignPanelTarget::Nodes` keeps
the host's complete ordered selection, while mixed, unset, bound, and
read-only leaf states remain controlled through `set_property_value_states`.

Grid mode follows the current Design defaults: Hug container axes, initial Hug
tracks, automatic rows, and row auto flow. Track count, sizing, add/delete, and
reorder controls remain host-controlled; row count is read-only while
automatic rows are enabled.

Export supports host-controlled Static and Animated modes. Static rows cover
PNG/JPG/SVG/PDF with target-gated advanced settings and controlled preview
states. Animated stories cover current Figma Motion MP4, WebM, GIF, and
animated SVG settings, including exact FPS matrices, top-level-frame
eligibility reasons, and the Starter high-resolution gate. Set
`FANTA_DESIGN_EXPORT_MODE=animated` with a Design node preset to open that
surface directly.

## Use the editor toolbar

`EditorToolbar` recreates Figma's floating editor toolbar as controlled GPUI
chrome. It includes the complete Design split-tool groups, Dev handoff tools,
Motion transport/keyframing controls, the Actions command palette, contextual
Agent composer, zoom controls, a capsule for host chrome controls (fit to view,
sidebar toggles), tooltips, focus navigation, and registered shortcuts. The
toolbar is intrinsic and occludes the pointer over the canvas beneath it; the
host places it in a canvas-relative wrapper (typically bottom center) rather
than asking the component to fill the canvas:

```rust
use fanta_gpui::prelude::*;
use gpui::{div, prelude::*, px};

let toolbar = cx.new(|cx| {
    EditorToolbar::new(
        "document-toolbar",
        ToolbarMode::Design,
        ToolbarTool::Move,
        100,
        window,
        cx,
    )
});

let toolbar_layer = div()
    .absolute()
    .left_0()
    .right_0()
    .bottom(px(18.))
    .flex()
    .justify_center()
    .child(toolbar.clone());

cx.subscribe(&toolbar, |host, toolbar, action: &ToolbarAction, cx| {
    host.apply_toolbar_intent(action);
    toolbar.update(cx, |toolbar, cx| {
        toolbar.set_mode(host.toolbar_mode(), cx);
        toolbar.set_active_tool(host.toolbar_tool(), cx);
        toolbar.set_zoom_percent(host.canvas_zoom(), cx);
        toolbar.set_motion_options(host.motion_toolbar_options(), cx);
        toolbar.set_dev_options(host.dev_toolbar_options(), cx);
        toolbar.set_agent_options(host.agent_toolbar_options(), cx);
        toolbar.set_chrome_controls(host.chrome_controls(), cx);
    });
});
```

Mode and tool selection, developer readiness, Motion transport, commands,
chrome-control presses, and Agent submissions are typed intents. Only flyouts,
search/prompt drafts, highlighted results, and focus continuity are local
presentation state. See [`docs/toolbar.md`](docs/toolbar.md) for the full tool
matrix and ownership contract.

If the host uses the supplied icons, construct the GPUI application with
`gpui_component_assets::Assets` (or include the same icon paths in a composite
asset source).

See [ARCHITECTURE.md](ARCHITECTURE.md) for the dependency, state-ownership, and
integration rules. Run the interactive storybook and use its Icons/Pages/
Layers/Design/Toolbar navigation to switch surfaces:

```sh
cargo run -p fanta-gpui-storybook
```
