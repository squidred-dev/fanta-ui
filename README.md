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

Initialize `gpui-component` once in the host, create the panel as a GPUI
entity, subscribe to its typed events, and render the entity:

```rust
use fanta_gpui::prelude::*;
use gpui::{Context, Entity, SharedString, Subscription, Window};

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
- Find and Replace modes with removable element-filter pills;
- current-page/all-pages scope and previous/next result navigation;
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

If the host uses the supplied icons, construct the GPUI application with
`gpui_component_assets::Assets` (or include the same icon paths in a composite
asset source).

See [ARCHITECTURE.md](ARCHITECTURE.md) for the dependency, state-ownership, and
integration rules. Run the full interactive story with:

```sh
cargo run -p fanta-gpui-storybook
```
