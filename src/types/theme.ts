export type ThemeImageSource =
  | { kind: "none" }
  | { kind: "local"; path: string; previewUrl: string }
  | { kind: "preset"; id: string; previewUrl: string };

export interface ThemeColors {
  accent: string;
  surface: string;
  text: string;
}

export interface ThemeEffects {
  brightness: number;
  blur: number;
  overlay: number;
  saturation: number;
}

export interface CodexTheme {
  id: string;
  name: string;
  image: ThemeImageSource;
  colors: ThemeColors;
  effects: ThemeEffects;
  createdAt: string;
  updatedAt: string;
}

export interface ThemeSummary {
  id: string;
  name: string;
  thumbnail?: string;
  accent: string;
  updatedAt: string;
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

export const DEFAULT_THEME: CodexTheme = {
  id: "midnight-bloom",
  name: "Midnight Bloom",
  image: { kind: "preset", id: "midnight-bloom", previewUrl: "" },
  colors: { accent: "#b7ff5a", surface: "#101319", text: "#f4f7f1" },
  effects: { brightness: 72, blur: 1, overlay: 42, saturation: 112 },
  createdAt: new Date(0).toISOString(),
  updatedAt: new Date(0).toISOString(),
};
