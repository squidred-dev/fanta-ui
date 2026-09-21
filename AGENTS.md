# AGENTS.md — Fanta GPUI

This repository contains headless, host-controlled GPUI components for Fanta.

## Critical invariants

1. `fanta-gpui` never depends on `fanta-engine` or its crates.
2. Components receive view data and emit typed intents; they do not mutate a
   Fanta document.
3. The storybook may own mock state, but reusable components may only own
   transient presentation state.
4. Prefer `gpui-component` primitives and active theme tokens.
5. Fanta components and the storybook forbid unsafe code. Imported GPUI,
   platform, and support crates retain their upstream native-code safety
   policies; do not apply the component prohibition to their FFI boundaries.

Read `ARCHITECTURE.md` before changing component boundaries.

## Commands

```sh
cargo run -p fanta-gpui-storybook
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```
