export type ThemeImageSource =
  | { kind: "none" }
  | { kind: "local"; path: string; previewUrl: string };

export interface ThemeColors {
  background: string;
  panel: string;
  "panel-alt": string;
  accent: string;
  "accent-alt": string;
  secondary: string;
  highlight: string;
  text: string;
  muted: string;
  line: string;
}

export interface ThemeEffects {
  opacity: number;
  blur: number;
  brightness: number;
  saturation: number;
  imagePositionX: number;
  imagePositionY: number;
  imageScale: number;
  panelOpacity: number;
  panelBlur: number;
  cornerRadius: number;
  contentMaxWidth: number;
  showBrand: boolean;
  showStatus: boolean;
  showQuote: boolean;
  showOrbit: boolean;
  showParticles: boolean;
  particleCount: number;
}

export interface ThemeDocument {
  schemaVersion: 1;
  id: string;
  name: string;
  subtitle: string;
  tagline: string;
  projectPrefix: string;
  projectLabel: string;
  statusText: string;
  quote: string;
  colors: ThemeColors;
  backgroundImage: string | null;
  opacity: number;
  blurPx: number;
  brightnessPct: number;
  saturationPct: number;
  imagePositionXPct: number;
  imagePositionYPct: number;
  imageScalePct: number;
  panelOpacity: number;
  panelBlurPx: number;
  cornerRadiusPx: number;
  contentMaxWidthPx: number;
  showBrand: boolean;
  showStatus: boolean;
  showQuote: boolean;
  showOrbit: boolean;
  showParticles: boolean;
  particleCount: number;
}

export interface CodexTheme {
  id: string;
  name: string;
  subtitle: string;
  tagline: string;
  projectPrefix: string;
  projectLabel: string;
  statusText: string;
  quote: string;
  image: ThemeImageSource;
  colors: ThemeColors;
  effects: ThemeEffects;
  builtIn?: boolean;
}

export interface CodexInstallation {
  installed: boolean;
  path?: string;
  version?: string;
  writable: boolean;
  message?: string;
}

export type ApplyStatus = "idle" | "applying" | "success" | "error";

export function documentToTheme(document: ThemeDocument, builtIn = false): CodexTheme {
  return {
    id: document.id,
    name: document.name,
    subtitle: document.subtitle,
    tagline: document.tagline,
    projectPrefix: document.projectPrefix,
    projectLabel: document.projectLabel,
    statusText: document.statusText,
    quote: document.quote,
    image: document.backgroundImage
      ? { kind: "local", path: document.backgroundImage, previewUrl: "" }
      : { kind: "none" },
    colors: { ...document.colors },
    effects: {
      opacity: document.opacity,
      blur: document.blurPx,
      brightness: document.brightnessPct,
      saturation: document.saturationPct,
      imagePositionX: document.imagePositionXPct,
      imagePositionY: document.imagePositionYPct,
      imageScale: document.imageScalePct,
      panelOpacity: document.panelOpacity,
      panelBlur: document.panelBlurPx,
      cornerRadius: document.cornerRadiusPx,
      contentMaxWidth: document.contentMaxWidthPx,
      showBrand: document.showBrand,
      showStatus: document.showStatus,
      showQuote: document.showQuote,
      showOrbit: document.showOrbit,
      showParticles: document.showParticles,
      particleCount: document.particleCount,
    },
    builtIn,
  };
}

export function themeToDocument(theme: CodexTheme): ThemeDocument {
  return {
    schemaVersion: 1,
    id: theme.id,
    name: theme.name,
    subtitle: theme.subtitle,
    tagline: theme.tagline,
    projectPrefix: theme.projectPrefix,
    projectLabel: theme.projectLabel,
    statusText: theme.statusText,
    quote: theme.quote,
    colors: { ...theme.colors },
    backgroundImage: theme.image.kind === "local" ? theme.image.path : null,
    opacity: theme.effects.opacity,
    blurPx: theme.effects.blur,
    brightnessPct: theme.effects.brightness,
    saturationPct: theme.effects.saturation,
    imagePositionXPct: theme.effects.imagePositionX,
    imagePositionYPct: theme.effects.imagePositionY,
    imageScalePct: theme.effects.imageScale,
    panelOpacity: theme.effects.panelOpacity,
    panelBlurPx: theme.effects.panelBlur,
    cornerRadiusPx: theme.effects.cornerRadius,
    contentMaxWidthPx: theme.effects.contentMaxWidth,
    showBrand: theme.effects.showBrand,
    showStatus: theme.effects.showStatus,
    showQuote: theme.effects.showQuote,
    showOrbit: theme.effects.showOrbit,
    showParticles: theme.effects.showParticles,
    particleCount: theme.effects.particleCount,
  };
}

export function duplicateTheme(theme: CodexTheme): CodexTheme {
  return {
    ...theme,
    id: `theme-${Date.now()}`,
    name: `${theme.name} 副本`,
    image: { ...theme.image },
    colors: { ...theme.colors },
    effects: { ...theme.effects },
    builtIn: false,
  };
}
