# Testing and builds

GitHub Actions is the source of truth for checks and distributable previews. Contributors are not required to perform heavyweight local builds.

## Check workflow

`.github/workflows/check.yml` runs:

- on Ubuntu: `npm ci`, the project typecheck script, Vitest, and the frontend build;
- on macOS 13, macOS 14, and Windows: `cargo fmt`, `cargo clippy`, `cargo test`, and `cargo check` against `src-tauri/Cargo.toml`.

The lightweight `vitest.config.ts` discovers colocated `*.test.*` and `*.spec.*` frontend tests and permits an initially empty suite.

## Portable preview build workflow

`.github/workflows/build.yml` runs manually or when a `v*` tag is pushed. It creates separate portable ZIP artifacts for:

- macOS x64;
- macOS arm64;
- Windows x64.

Each archive includes the application, a platform-specific launcher, usage instructions, and the TokenToken sponsor notice. No NSIS or other installer is produced.

These artifacts are **unsigned previews**. macOS Gatekeeper or Windows SmartScreen may warn or refuse normal launch. Users should extract the complete ZIP and enter through the included launcher. Do not present them as signed releases, and verify their provenance from the corresponding GitHub Actions run.

## Adding tests

Prefer deterministic unit tests with no network dependency. Mock privileged desktop calls in frontend tests. For Rust code, test parsing, validation, and command helpers independently of the UI. Keep fixtures small and free of credentials or personal data.
