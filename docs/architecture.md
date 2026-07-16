# Architecture

Codex Skin Workshop is organized as a small desktop application with two explicit layers.

## Frontend

The frontend under `src/` owns presentation, interaction state, validation, and calls into the desktop command boundary. It should remain testable without starting the desktop shell. UI tests belong next to source files as `*.test.ts` or `*.test.tsx` and run with Vitest in CI.

## Rust/Tauri backend

The backend under `src-tauri/` owns desktop integration, filesystem access, process boundaries, persistence, and other privileged operations. Commands should validate untrusted input and return useful errors rather than panicking. Unit tests should stay close to Rust modules; integration tests may live under `src-tauri/tests/`.

## Trust boundary

Treat frontend input, imported theme/skin data, paths, URLs, and subprocess output as untrusted. Expose the smallest practical Tauri command surface. Do not send secrets to frontend logs or persist them in project files.

## Upstream relationship

The workshop follows core ideas common to Codex tooling: transparent, reviewable workflows; explicit user control; and a clear boundary around privileged actions. It may incorporate or adapt MIT-licensed upstream material where notices are preserved. It is an independent community project and does not claim an official relationship with or endorsement by OpenAI or any upstream project.
