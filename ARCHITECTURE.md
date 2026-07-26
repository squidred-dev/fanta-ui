# Fanta GPUI Architecture

This is the design record for the reusable Fanta UI library. Section numbers
are stable; add new sections instead of renumbering existing ones.

## §1 Purpose and shape

`fanta-gpui` contains reusable GPUI components for Fanta hosts. It is not a
desktop application and it does not own documents, persistence, rendering, or
editing behavior.

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

## §6 Storybook

Every reusable component should have at least one interactive story covering
its meaningful visual states. Story state is local mock state only. Stories are
the place for visual tuning; domain workflows and Fanta operations belong in
the application host.

The Pages story is also a reference adapter: it subscribes to the component's
typed event stream, mutates mock host data, performs mock search, and supplies
the resulting page and search read models back to the component.

## §7 Safety and quality

- `unsafe` code is forbidden.
- Public intent and prop types should implement useful comparison/debug traits.
- `cargo fmt --all -- --check`, `cargo test --workspace`, and
  `cargo clippy --workspace --all-targets -- -D warnings` must pass.
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

Interactive controls participate in the GPUI tab-stop tree. Text inputs,
buttons, page and result rows, filter pills, search-scope controls, and menu
items must remain reachable with Tab and Shift-Tab. Focus styling is part of
the component presentation state; command actions remain public so a host can
replace key bindings or expose them in its own command palette.
