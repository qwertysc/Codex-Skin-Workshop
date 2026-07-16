# Codex Skin Workshop

A friendly, open-source desktop workshop for visually customizing Codex. Pick a background, tune colors and effects, preview every change, and apply the result only when you are ready.

> Project status: early MVP. The visual editor, value-only theme store, managed Codex launch, loopback CDP injection, and restore path are implemented. Platform compatibility still needs validation through GitHub Actions and real-device testing before release.

[中文说明](README.zh-CN.md)

## Highlights

- **Beginner-friendly flow** — detect Codex, choose an image, tune the look, preview, apply.
- **Safe live preview** — editing does not touch Codex until **Apply to Codex** is clicked.
- **Visual controls** — accent/surface/text colors, brightness, blur, dark overlay, and saturation.
- **Theme library** — built-in starting points plus a frontend model for locally saved themes.
- **Restore entry point** — a clear action for restoring the original appearance.
- **Lightweight UI** — React, TypeScript, Vite, and handcrafted CSS; no heavyweight component framework.
- **Tauri 2 bridge** — frontend APIs call typed Tauri commands for detection, storage, application, and restore.

## Tech stack

- Tauri 2
- React 18 + TypeScript
- Vite 5
- Handwritten CSS

## Development

Prerequisites: a current Node.js/npm toolchain and the Rust/Tauri platform prerequisites for your operating system.

Development checks, tests, and desktop bundles are intentionally validated by GitHub Actions. Fork the repository, push a branch, and use the **Check** and **Build desktop previews** workflows. This keeps heavy multi-platform builds off lightweight local machines.

## Frontend/backend contract

The frontend currently invokes these commands:

| Command | Purpose |
| --- | --- |
| `detect_codex` | Locate Codex and report version/write access |
| `list_themes` | Read locally saved theme summaries |
| `save_theme` | Validate and persist a theme |
| `import_image` | Validate, sanitize, copy, and inspect a selected image |
| `launch_codex` | Start a locally managed Codex session with loopback CDP enabled |
| `apply_theme` | Apply the selected value-only theme through the managed preview target |
| `restore_codex` | Remove workshop styling and close the managed preview session |

The current backend uses a managed Codex preview process and a loopback-only DevTools connection rather than patching application files. Platform compatibility and recovery behavior still need broad testing before release.

## Project layout

```text
src/
  App.tsx            Visual workshop UI
  styles.css         Handcrafted responsive desktop styling
  types/theme.ts     Theme and Codex installation models
  lib/tauri.ts       Typed frontend command bridge
src-tauri/            Tauri 2 application and command scaffold
```

## Safety goals

Before the apply backend is considered complete, it should:

1. verify a detected Codex executable before launching it;
2. never modify Codex application files, `app.asar`, credentials, API keys, or provider settings;
3. keep themes as validated values and sanitized local images, never executable CSS/JS;
4. restrict CDP to a random `127.0.0.1` port and exact page WebSocket URL;
5. retain a reliable restore path and stop only the process launched by the workshop;
6. provide actionable errors without hiding failures.

## Contributing

Issues and pull requests are welcome. Keep the experience understandable for non-technical users, avoid unnecessary dependencies, and document any platform-specific behavior.

## License

MIT — see [LICENSE](LICENSE) and [NOTICE.md](NOTICE.md). The CDP skinning approach is adapted from the MIT-licensed Codex Dream Skin project. Credential and provider-routing functionality is outside this project's scope.
