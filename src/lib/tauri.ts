import { convertFileSrc, invoke, isTauri } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { CodexInstallation, CodexTheme, ThemeSummary } from "../types/theme";

interface BackendInstallation {
  executable: string;
  displayName: string;
}

interface BackendTheme {
  id: string;
  name: string;
  colors: Record<string, string>;
  background_image?: string | null;
  opacity?: number | null;
  blur_px?: number | null;
  brightness_pct?: number | null;
  saturation_pct?: number | null;
}

interface ImportedImage {
  relativePath: string;
  absolutePath: string;
  width: number;
  height: number;
  palette: string[];
}

interface DevtoolsTarget {
  id: string;
  title: string;
  url: string;
  webSocketDebuggerUrl: string;
}

interface LaunchResult {
  port: number;
  targets: DevtoolsTarget[];
}

let detectedExecutable: string | null = null;
let activeTargetId: string | null = null;

function toBackendTheme(theme: CodexTheme): BackendTheme {
  return {
    id: theme.id,
    name: theme.name,
    colors: theme.colors,
    background_image: theme.image.kind === "local" ? theme.image.path : null,
    opacity: Math.max(0, Math.min(1, 1 - theme.effects.overlay / 100)),
    blur_px: theme.effects.blur,
    brightness_pct: theme.effects.brightness,
    saturation_pct: theme.effects.saturation,
  };
}

export async function detectCodex(): Promise<CodexInstallation> {
  if (!isTauri()) {
    return { installed: true, path: "~/Library/Application Support/Codex", version: "Browser preview", writable: true };
  }
  const installations = await invoke<BackendInstallation[]>("detect_codex");
  const first = installations[0];
  detectedExecutable = first?.executable ?? null;
  return first
    ? { installed: true, path: first.executable, writable: true, message: first.displayName }
    : { installed: false, writable: false, message: "Install Codex, then reopen the workshop." };
}

export async function chooseThemeImage(): Promise<{ path: string; previewUrl: string } | null> {
  if (!isTauri()) return null;
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp", "gif"] }],
  });
  if (!selected) return null;
  const imported = await invoke<ImportedImage>("import_image", { sourcePath: selected });
  return { path: imported.relativePath, previewUrl: convertFileSrc(imported.absolutePath) };
}

export async function loadThemeLibrary(): Promise<ThemeSummary[]> {
  if (!isTauri()) return [];
  const themes = await invoke<BackendTheme[]>("list_themes");
  return themes.map((theme) => ({
    id: theme.id,
    name: theme.name,
    accent: theme.colors.accent ?? "#b7ff5a",
    updatedAt: "Saved locally",
  }));
}

export async function saveTheme(theme: CodexTheme): Promise<void> {
  if (!isTauri()) return;
  await invoke("save_theme", { theme: toBackendTheme(theme) });
}

export async function applyTheme(theme: CodexTheme): Promise<void> {
  if (!isTauri()) {
    await new Promise((resolve) => window.setTimeout(resolve, 650));
    return;
  }
  if (!detectedExecutable) throw new Error("Codex was not detected.");
  if (!activeTargetId) {
    const launched = await invoke<LaunchResult>("launch_codex", { executable: detectedExecutable });
    const target = launched.targets.find((item) => item.url.startsWith("http")) ?? launched.targets[0];
    if (!target) throw new Error("Codex opened, but no preview target was available.");
    activeTargetId = target.id;
  }
  await invoke("apply_theme", { targetId: activeTargetId, theme: toBackendTheme(theme) });
}

export async function restoreDefault(): Promise<void> {
  if (!isTauri()) return;
  await invoke("restore_codex", { targetId: activeTargetId ?? null });
  activeTargetId = null;
}
