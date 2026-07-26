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

## §3 Controlled components

Components are controlled:

- Input data is immutable for the duration of a render.
- Selection and expanded state are supplied by the host.
- User interaction emits typed intents.
- A component never mutates a Fanta document or silently invents domain state.
- A component may keep transient presentation state only when GPUI requires it
  (focus, hover, animation, scroll position).

The Pages panel demonstrates the contract with `PagesPanelItem`,
`PagesPanelAction`, and `PagesPanel`. Adding a page emits `Add`; it does not
create a page itself.

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

## §7 Safety and quality

- `unsafe` code is forbidden.
- Public intent and prop types should implement useful comparison/debug traits.
- `cargo fmt --all -- --check`, `cargo test --workspace`, and
  `cargo clippy --workspace --all-targets -- -D warnings` must pass.
- New dependencies should preserve the seam in §2.

