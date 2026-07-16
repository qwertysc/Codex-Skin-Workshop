import { convertFileSrc, invoke, isTauri } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type { CodexInstallation, CodexTheme, ThemeDocument } from "../types/theme";
import { documentToTheme, themeToDocument } from "../types/theme";

interface BackendInstallation {
  executable: string;
  displayName: string;
}

interface BackendTheme {
  schema_version?: number;
  id: string;
  name: string;
  subtitle?: string;
  tagline?: string;
  project_prefix?: string;
  project_label?: string;
  status_text?: string;
  quote?: string;
  colors: Record<string, string>;
  background_image?: string | null;
  opacity?: number | null;
  blur_px?: number | null;
  brightness_pct?: number | null;
  saturation_pct?: number | null;
  image_position_x_pct?: number | null;
  image_position_y_pct?: number | null;
  image_scale_pct?: number | null;
  panel_opacity?: number | null;
  panel_blur_px?: number | null;
  corner_radius_px?: number | null;
  content_max_width_px?: number | null;
  show_brand?: boolean | null;
  show_status?: boolean | null;
  show_quote?: boolean | null;
  show_orbit?: boolean | null;
  show_particles?: boolean | null;
  particle_count?: number | null;
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
  type: string;
  webSocketDebuggerUrl: string;
}

interface LaunchResult {
  port: number;
  target: DevtoolsTarget;
}

let detectedExecutable: string | null = null;
let activeTargetId: string | null = null;

const color = (theme: BackendTheme, key: string, fallback: string) =>
  theme.colors[key] ?? fallback;

function fromBackendTheme(theme: BackendTheme): CodexTheme {
  const importedImage = theme.background_image ?? null;
  const document: ThemeDocument = {
    schemaVersion: 1,
    id: theme.id,
    name: theme.name,
    subtitle: theme.subtitle ?? "CODEX SKIN WORKSHOP",
    tagline: theme.tagline ?? "把喜欢的画面变成可交互的 Codex 工作台。",
    projectPrefix: theme.project_prefix ?? "项目 · ",
    projectLabel: theme.project_label ?? "选择项目",
    statusText: theme.status_text ?? "主题已启用",
    quote: theme.quote ?? "创造美好的东西",
    colors: {
      background: color(theme, "background", "#f7f4f5"),
      panel: color(theme, "panel", "#ffffff"),
      "panel-alt": color(theme, "panel-alt", "#fff7f8"),
      accent: color(theme, "accent", "#e25563"),
      "accent-alt": color(theme, "accent-alt", "#f07a86"),
      secondary: color(theme, "secondary", "#f3a8af"),
      highlight: color(theme, "highlight", "#c93d4c"),
      text: color(theme, "text", "#2b2224"),
      muted: color(theme, "muted", "#8a7a7d"),
      line: color(theme, "line", "#e5c9cd"),
    },
    backgroundImage: importedImage,
    opacity: theme.opacity ?? 0.95,
    blurPx: theme.blur_px ?? 0,
    brightnessPct: theme.brightness_pct ?? 100,
    saturationPct: theme.saturation_pct ?? 100,
    imagePositionXPct: theme.image_position_x_pct ?? 50,
    imagePositionYPct: theme.image_position_y_pct ?? 50,
    imageScalePct: theme.image_scale_pct ?? 100,
    panelOpacity: theme.panel_opacity ?? 0.86,
    panelBlurPx: theme.panel_blur_px ?? 12,
    cornerRadiusPx: theme.corner_radius_px ?? 18,
    contentMaxWidthPx: theme.content_max_width_px ?? 950,
    showBrand: theme.show_brand ?? true,
    showStatus: theme.show_status ?? true,
    showQuote: theme.show_quote ?? true,
    showOrbit: theme.show_orbit ?? true,
    showParticles: theme.show_particles ?? true,
    particleCount: theme.particle_count ?? 8,
  };
  const converted = documentToTheme(document);
  if (converted.image.kind === "local" && importedImage) {
    converted.image.previewUrl = convertFileSrc(importedImage);
  }
  return converted;
}

function toBackendTheme(theme: CodexTheme): BackendTheme {
  const document = themeToDocument(theme);
  return {
    schema_version: 1,
    id: document.id,
    name: document.name,
    subtitle: document.subtitle,
    tagline: document.tagline,
    project_prefix: document.projectPrefix,
    project_label: document.projectLabel,
    status_text: document.statusText,
    quote: document.quote,
    colors: { ...document.colors },
    background_image: document.backgroundImage,
    opacity: document.opacity,
    blur_px: document.blurPx,
    brightness_pct: document.brightnessPct,
    saturation_pct: document.saturationPct,
    image_position_x_pct: document.imagePositionXPct,
    image_position_y_pct: document.imagePositionYPct,
    image_scale_pct: document.imageScalePct,
    panel_opacity: document.panelOpacity,
    panel_blur_px: document.panelBlurPx,
    corner_radius_px: document.cornerRadiusPx,
    content_max_width_px: document.contentMaxWidthPx,
    show_brand: document.showBrand,
    show_status: document.showStatus,
    show_quote: document.showQuote,
    show_orbit: document.showOrbit,
    show_particles: document.showParticles,
    particle_count: document.particleCount,
  };
}

export async function detectCodex(): Promise<CodexInstallation> {
  if (!isTauri()) {
    return { installed: true, path: "浏览器预览模式", version: "预览", writable: true };
  }
  const installations = await invoke<BackendInstallation[]>("detect_codex");
  const first = installations[0];
  detectedExecutable = first?.executable ?? null;
  return first
    ? { installed: true, path: first.executable, writable: true, message: first.displayName }
    : { installed: false, writable: false, message: "请先安装 Codex，然后重新打开皮肤工坊。" };
}

export async function chooseThemeImage(): Promise<{
  path: string;
  previewUrl: string;
  palette: string[];
} | null> {
  if (!isTauri()) return null;
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "图片", extensions: ["png", "jpg", "jpeg", "webp", "gif"] }],
  });
  if (!selected) return null;
  const imported = await invoke<ImportedImage>("import_image", { sourcePath: selected });
  return {
    path: imported.relativePath,
    previewUrl: convertFileSrc(imported.absolutePath),
    palette: imported.palette,
  };
}

export async function loadThemes(): Promise<CodexTheme[]> {
  if (!isTauri()) return [];
  const themes = await invoke<BackendTheme[]>("list_themes");
  return themes.map(fromBackendTheme);
}

export async function saveTheme(theme: CodexTheme): Promise<void> {
  if (!isTauri()) return;
  await invoke("save_theme", { theme: toBackendTheme(theme) });
}

export async function importTheme(): Promise<CodexTheme | null> {
  if (!isTauri()) return null;
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "Codex 主题文件", extensions: ["json"] }],
  });
  if (!selected) return null;
  const theme = await invoke<BackendTheme>("import_theme_file", { sourcePath: selected });
  return fromBackendTheme(theme);
}

export async function exportTheme(theme: CodexTheme): Promise<boolean> {
  if (!isTauri()) return false;
  const destination = await save({
    defaultPath: `${theme.id}.csw-theme.json`,
    filters: [{ name: "Codex 主题文件", extensions: ["json"] }],
  });
  if (!destination) return false;
  await invoke("export_theme_file", {
    destinationPath: destination,
    theme: toBackendTheme(theme),
  });
  return true;
}

function recoverable(error: unknown): boolean {
  const message = String(error);
  return [
    "已退出",
    "窗口已变化",
    "当前没有",
    "Connection refused",
    "actively refused",
    "not currently advertised",
  ].some((part) => message.includes(part));
}

async function launchAndSelect(): Promise<void> {
  if (!detectedExecutable) throw new Error("尚未检测到 Codex。");
  const launched = await invoke<LaunchResult>("launch_codex", {
    executable: detectedExecutable,
  });
  activeTargetId = launched.target.id;
}

export async function applyTheme(theme: CodexTheme): Promise<void> {
  if (!isTauri()) {
    await new Promise((resolve) => window.setTimeout(resolve, 350));
    return;
  }
  if (!activeTargetId) await launchAndSelect();
  try {
    await invoke("apply_theme", { targetId: activeTargetId, theme: toBackendTheme(theme) });
  } catch (error) {
    if (!recoverable(error)) throw error;
    activeTargetId = null;
    await launchAndSelect();
    await invoke("apply_theme", { targetId: activeTargetId, theme: toBackendTheme(theme) });
  }
}

export async function restoreDefault(): Promise<void> {
  if (!isTauri()) return;
  try {
    await invoke("restore_codex", { targetId: activeTargetId ?? null });
  } finally {
    activeTargetId = null;
  }
}
