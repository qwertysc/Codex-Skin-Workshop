# Contributing

Thank you for helping improve Codex Skin Workshop.

## Before opening a change

1. Search existing issues and pull requests.
2. Keep changes focused and explain user-visible behavior.
3. Never commit credentials, generated secrets, signing identities, or private logs.
4. Preserve upstream copyright and MIT license notices when adapting code.

## Development expectations

The repository separates a web frontend from a Rust/Tauri backend. See [`docs/architecture.md`](docs/architecture.md) for boundaries and [`docs/testing.md`](docs/testing.md) for automated checks.

Keep local work lightweight. The authoritative test and build matrix runs in GitHub Actions:

- Ubuntu: dependency installation, TypeScript checking, Vitest, and frontend build.
- macOS 13, macOS 14, and Windows: Rust formatting, Clippy, tests, and checks.
- Manual or version-tag build workflow: unsigned macOS x64/arm64 and Windows NSIS previews.

Pull requests should add or update small deterministic tests for changed behavior. Avoid snapshots for unstable UI output and avoid network-dependent tests.

## Pull requests

Describe what changed, why, and any risks. Link related issues. Screenshots are useful for visual changes. By contributing, you agree that your contribution is licensed under the repository's MIT License.

## Project relationship

This project may build on ideas or MIT-licensed material from upstream Codex-related projects, but it is independently maintained and is not an official OpenAI product or endorsement.
