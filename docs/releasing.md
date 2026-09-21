# Releases and editor integration

This workspace owns the GPUI fork and reusable Fanta components. It does not
own `fanta_ui`, `fig_viewer`, document engines, or their host adapters. macOS is
the first release gate; Linux, Windows, and web are preserved but unverified.

## Package identity

All published packages start at `0.1.0` and use exact internal dependencies.
`fanta-gpui` remains the component facade. GPUI is `fanta-gpui-core`, and the
component fork is `fanta-gpui-components`. Other names prefix the complete old
name with `fanta-gpui-`, replacing underscores with hyphens. Keeping the complete
name distinguishes `gpui_util` from `util` without a collision. The authoritative
mapping is [extraction/packages.json](extraction/packages.json).

Consumers preserve Rust imports through aliases:

```toml
[dependencies]
fanta-gpui = "=0.1.0"
gpui = { package = "fanta-gpui-core", version = "=0.1.0" }
gpui-component = { package = "fanta-gpui-components", version = "=0.1.0" }
gpui_platform = { package = "fanta-gpui-gpui-platform", version = "=0.1.0" }
```

Every crate exchanging GPUI or supporting-library types must resolve to the
same package identity. Do not combine these packages with an upstream `gpui`
patch. Macro expansions retain the `gpui` import alias. Shared helper aliases
are updated in the editor along with framework aliases.

## Validation before publication

```sh
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 script/release.py --package-lists
python3 script/verify-packages.py --run
```

The final command packages the actual crates in dependency order and exposes
them through a temporary loopback sparse registry. External crates are proxied
unchanged from crates.io. It copies the registry-only smoke consumer outside
this repository, checks and tests it, asserts a single GPUI identity, and
optionally launches a native window that exits after two seconds. No packages
are uploaded. Archive checksums are written to
`target/release-verification/archives.json`. This test is deliberately separate
from workspace builds: asset features, shader inputs, and package contents must
work without workspace feature unification or source paths.

The native macOS backend requires Xcode's Metal compiler and SDK. In restricted
environments it also needs access to the native compiler cache.

The storybook and the GPUI example/macro harnesses are unpublished. Framework
examples live in their own host to avoid a `core -> platform -> core` publication
cycle. Macro doctests run from the macro harness for the same reason. Upstream
forks retain explicit lint policies; Fanta components and storybook still
forbid unsafe code.

## Publish automatically with GitHub Actions

The repository remote is `squidred-dev/fanta-ui`. Its
[`Publish crates` workflow](../.github/workflows/release.yml) runs when a `v*`
tag is pushed, or manually against an existing tag. The tag must match the
workspace version (initially `v0.1.0`).

One-time setup: create a crates.io API token permitted to publish new packages
and updates for this workspace, and save it as the repository Actions secret
`CRATES_IO_TOKEN`. This command prompts for the value without storing it in a
source file:

```sh
gh secret set CRATES_IO_TOKEN --repo squidred-dev/fanta-ui
```

Commit and push the reviewed extraction and workflow before creating the tag:

```sh
git tag v0.1.0
git push origin v0.1.0
```

Actions checks formatting, builds, tests, Clippy, native feature compilation,
package contents, and the isolated packaged consumer on macOS. It then publishes
in dependency order, waits for each registry entry, and checks and tests the
consumer against public crates.io. Archive verification evidence is retained as
a workflow artifact. Native interactive validation and the editor integration
remain separate release gates; this workflow does not edit `fanta-edit`.

If a run stops after some uploads, rerun it against the same tag. Resume checks
the archive checksum of each existing version before skipping it; an immutable
published version with different contents stops the release. Concurrent releases
are serialized. For subsequent releases, update the coordinated package version,
exact internal requirements, and consumer versions before tagging.

## Publish locally

1. Verify names and ownership with `python3 script/release.py --registry-check`.
   Availability is only a point-in-time observation, not a reservation.
2. Configure crates.io authentication locally with `cargo login` or Cargo's
   credential provider. Never put a token in this repository.
3. Commit the reviewed release sources, including the component changes being
   released. Record that revision and pass the checks above.
4. Run `python3 script/release.py --publish`. Each package is dry-run verified,
   uploaded, and observed in the registry index before its dependents continue.
   Existing versions cause a stop rather than silently trusting someone else's
   package. Use `--publish --resume` only from the same clean source revision; already
   published packages must match a freshly packaged archive checksum before
   they are skipped. Crates.io publication is
   not transactional.
5. Run `cargo check`, `cargo test`, and `FANTA_SMOKE_AUTO_QUIT=1 cargo run` in
   `examples/registry-smoke` with no registry override. This is the final public
   registry check; the temporary-registry check does not substitute for it.

## Switch Fanta Edit

`script/integrate-editor.py` requires Python 3.11+ and `tomlkit`. It preserves
unrelated manifest formatting, features, and edits. The command updates all
shared dependency aliases and profile package names, removes obsolete GPUI
patches, and excludes old source copies from workspace membership. It never
deletes those copies.

```sh
python3 script/integrate-editor.py /path/to/fanta-edit --mode local
# After a reviewed extraction revision has been committed and pushed:
python3 script/integrate-editor.py /path/to/fanta-edit --mode git --revision FULL_COMMIT
# After publication and the registry-only smoke test:
python3 script/integrate-editor.py /path/to/fanta-edit --mode registry
```

Local mode is an explicit development override. Git mode is a pinned
pre-release checkpoint. Registry mode contains exact package versions without
checkout paths. Re-run `cargo check -p fig_viewer`, the editor's prescribed
Clippy checks, and `cargo build -p zed --bin fanta`; then exercise actual editor
startup and panel intents before removing old copies. The existing
`fig_viewer::gpui_adapters` remain the engine seam.

Only after registry integration passes, remove the excluded migrated source
copies and update editor vendoring documentation. Do not remove `fanta_ui`,
`fig_viewer`, or any document/canvas engine. If credentials, ownership, native
runtime validation, or registry verification are unavailable, the release is
incomplete and the editor must not be switched to nonexistent registry versions.
