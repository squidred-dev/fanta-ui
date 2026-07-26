# fanta-gpui

Headless, host-controlled GPUI components for Fanta applications.

This repository owns reusable UI composition only. Fanta Engine remains the
source of truth for documents, selections, history, and mutations. Components
receive plain view data and emit typed intents that the host maps to Fanta
operations.

The visual layer is built on
[`gpui-component`](https://longbridge.github.io/gpui-component/gallery/).

## Repository shape

- `crates/fanta-gpui` — the reusable component library and facade.
- `crates/fanta-gpui-storybook` — a small desktop gallery for developing
  components in isolation.

## Run the storybook

```sh
cargo run -p fanta-gpui-storybook
```

## Use the Pages panel

Initialize `gpui-component` once in the host, then render the controlled
component from host state:

```rust
use fanta_gpui::prelude::*;
use gpui::{App, SharedString, Window};

let pages = doc_pages
    .iter()
    .map(|page| PagesPanelItem::new(page.id.to_string(), page.name.clone()))
    .collect();

let panel = PagesPanel::new("document-pages", pages)
    .expanded(sidebar.pages_expanded)
    .selected_page(active_page_id.to_string())
    .on_action(|action, window: &mut Window, cx: &mut App| {
        match action {
            PagesPanelAction::Toggle { expanded } => {
                // Update host UI state.
            }
            PagesPanelAction::Select { page_id } => {
                // Resolve the opaque string back to the host's PageId.
            }
            PagesPanelAction::Search => {
                // Open the host's page search.
            }
            PagesPanelAction::Add => {
                // Apply the appropriate Fanta Operation.
            }
        }
    });
```

The panel deliberately does not depend on `fanta-engine`. Its `page_id` is an
opaque `SharedString`, so a Fanta host can use the canonical `PageId` string
without coupling the UI library to engine internals.

If the host uses the supplied icons, construct the GPUI application with
`gpui_component_assets::Assets` (or include the same icon paths in a composite
asset source).

See [ARCHITECTURE.md](ARCHITECTURE.md) for the dependency and state-ownership
rules.

