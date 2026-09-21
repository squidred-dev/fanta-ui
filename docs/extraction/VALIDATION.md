# Extraction validation — 2026-09-21

The standalone extraction and local editor integration are implemented.
**The crates.io release is not complete.** No crates were uploaded.

## Inventory and provenance

- 32 imported packages plus the existing `fanta-gpui` facade: 33 publishable
  packages at `0.1.0`.
- Source revisions and original working-tree status are recorded in
  `source-state.json` and `destination-state.json`; original imported file
  fingerprints are in `source-hashes.json`.
- The existing component changes in this repository were retained. The
  source editor's macOS and media changes, including untracked playback tests
  and fixtures, were imported rather than replaced with their committed copies.
- Additional forks are the GPUI component assets, the macOS-compatible
  `async-process`/`smol` pair, and the pinned `proptest`/`proptest-macro` pair.
  The registry property macro cannot implement GPUI's re-exported macro path.
- Name availability checks returned 404 for all proposed packages. These
  observations do not reserve names or establish publisher ownership.

## Passed

| Check | Result |
| --- | --- |
| Pre-extraction component workspace check and tests | Passed |
| Pre-extraction editor `cargo check -p fig_viewer` | Passed |
| Standalone `cargo check --workspace` | Passed |
| `cargo fmt --all -- --check` | Passed |
| Workspace tests and doctests | 3,133 passed, 0 failed, 7 ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed with explicit imported-crate lint policies |
| Cargo package file lists and publication dependency graph | All 33 packages passed |
| Real `.crate` archives through temporary sparse registry | All 33 packaged |
| Independent consumer using those archives | Check, two macro tests, and native window launch passed |
| `cargo publish --dry-run` for `fanta-gpui-derive-refineable` | Passed; upload explicitly aborted by dry run |
| macOS `font-kit,screen-capture,runtime_shaders` feature combination | Passed |
| Editor `cargo check -p fig_viewer` using extracted packages | Passed |
| Editor `cargo build -p zed --bin fanta` | Passed |
| Editor `cargo test -p fig_viewer --lib gpui_adapters` | 45 passed, 0 failed |
| Rebuilt editor startup with temporary data directory | First frame logged at 14:52:42 Europe/Madrid; test instance stopped |
| Editor resolved dependency graph | Exactly one `fanta-gpui-core`; no old extracted package identities |

The independent consumer was copied outside the repository, used registry
versions without path dependencies or patches, and resolved every Fanta package
from the temporary registry. It exercised the native application, component
initialization/assets, ordinary GPUI test macro, and property-test macro.
Archive checksums are generated in `target/release-verification/archives.json`.

Native Metal compilation requires access to Xcode's compiler cache. Restricted
sandbox runs failed on that cache; the authorized native builds passed.

## Remaining release work

- Configure crates.io authentication locally. No token or configured Cargo
  credential provider was present at validation time.
- Commit and push the reviewed release source; validate the pinned Git editor
  checkpoint before publishing. Existing user changes have not been committed
  automatically.
- Publish in dependency order, verify the consumer against actual crates.io,
  and switch `fanta-edit` from its explicit local development aliases to registry
  versions. The temporary registry is a packaging test, not publication.
- Only then remove the excluded source copies from `fanta-edit` and finish its
  vendoring/license-script cleanup. Copies were deliberately retained.
- Linux, Windows, and web remain unverified release targets. Their source and
  package-local resources are present. Runtime capture permissions, clipboard
  integration, and authenticated editor services were not manually exercised;
  the startup test reached rendering but the existing authentication service
  returned HTTP 500.

Remaining presentation in `fanta_ui` and `fig_viewer` is intentionally deferred.
See [../releasing.md](../releasing.md) for commands and package aliases.
