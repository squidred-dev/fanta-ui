# Extraction validation — 2026-09-21

The standalone extraction and local editor integration are implemented.
**The crates.io release is not complete.** Five of the 33 crates have published:
`fanta-gpui-async-process`, `fanta-gpui-derive-refineable`,
`fanta-gpui-gpui-macros`, `fanta-gpui-gpui-shared-string`, and
`fanta-gpui-gpui-util`. The release is waiting on crates.io's new-crate rate limit.

The extraction and release workflow are committed and pushed. The `v0.1.0`
tag points to `ba09b20` on `squidred-dev/fanta-ui`. GitHub Actions passed all
workspace checks, native feature compilation, archive verification, and the
isolated consumer checks and tests. The first upload was rejected because the
publisher account had no verified email address. The publisher has since
confirmed email verification, and
[the release workflow](https://github.com/squidred-dev/fanta-ui/actions/runs/35613549055)
has accepted the verified account and started uploading, with rate-limit retries
and a six-hour timeout for creating all 33 names. The Actions secret is configured.
Remaining registry publication and the final consumer check are pending until
that run succeeds. Do not move the release tag now that archives have published.

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
| Pinned Git extraction `f35ed675ceca8620d2cd57b35641894b561f3afd` | Editor check, full application build, and all 45 adapter tests passed |
| GitHub release validation at `e1f9050` | Workspace validation and packaged consumer passed; upload blocked by account email verification |

The independent consumer was copied outside the repository, used registry
versions without path dependencies or patches, and resolved every Fanta package
from the temporary registry. It exercised the native application, component
initialization/assets, ordinary GPUI test macro, and property-test macro.
Archive checksums are generated in `target/release-verification/archives.json`.

Native Metal compilation requires access to Xcode's compiler cache. Restricted
sandbox runs failed on that cache; the authorized native builds passed.

## Remaining release work

- Finish the tagged release. Authentication is configured through the
  repository's `CRATES_IO_TOKEN`
  Actions secret. The first release attempt also identified a generated
  `proptest/persistence-test.txt` file accidentally tracked during extraction;
  `e1f9050` removes and ignores that generated output.
- Publish in dependency order, verify the consumer against actual crates.io,
  and switch `fanta-edit` from its pinned Git aliases to registry
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
