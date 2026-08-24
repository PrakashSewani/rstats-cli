# Contributing

Read [AGENTS.md](AGENTS.md) before changing source code, tests, packaging, or workflows. Use [Architecture](docs/ARCHITECTURE.md) for system behavior and [Development](docs/DEVELOPMENT.md) for verification, packaging, CI, and releases.

## Repository layout

- `src/` — Rust monitor, collector, recording, and TUI implementation
- `tests/` — Rust integration tests
- `npm/rstats-cli/` — publishable npm launcher package
- `npm/rstats-cli-*/` — publishable platform-native package metadata
- `scripts/` — version, staging, and package validation tools
- `.github/workflows/` — CI and tagged release automation

Native executables are generated during release staging. Do not commit files under `npm/*/bin/` or runtime data under `recordings/`.

## Local checks

```sh
cargo fmt --all
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
npm run check-version
npm run validate-packages
npm test
```

Run the monitor locally with:

```sh
cargo run -- --monitor
```

Test the recordings-folder mode with a temporary directory. The command launches the host file manager, so run it only in an interactive desktop environment:

```sh
cargo run -- --open-recordings --recording-dir ./tmp-recordings
```

## npm package checks

The source tree validates package metadata without native binaries:

```sh
npm run sync-version
npm run check-version
npm run validate-packages
(cd npm/rstats-cli && npm pack --dry-run --json)
```

A release staging job copies each target binary into its matching package and then runs:

```sh
node scripts/validate-packages.mjs --require-binaries
(cd npm/rstats-cli && npm pack --dry-run --json)
```

## Releases

1. Update the version in `Cargo.toml`.
2. Run `npm run sync-version`.
3. Run all Rust and npm checks.
4. Confirm `npm run check-version` passes.
5. Push a protected tag matching the Cargo version, for example `v0.1.0`.

The tagged release workflow builds these targets:

- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`

It publishes platform packages before `rstats-cli`, creates checksums, and attaches native artifacts to the GitHub release. Publishing requires npm trusted publishing/OIDC to be configured for the package names and repository.

## Security

Do not add install-time network downloads or shell-based command construction. The npm launcher must only resolve the matching optional dependency and spawn its packaged executable. Do not commit credentials, recordings, native binaries, or generated archives.
