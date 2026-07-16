import dreamSkin from "./dream-skin.json";
import neonOrbit from "./neon-orbit.json";
import quietOcean from "./quiet-ocean.json";
import warmPaper from "./warm-paper.json";
import { documentToTheme, type CodexTheme, type ThemeDocument } from "../types/theme";

const documents = [dreamSkin, quietOcean, warmPaper, neonOrbit] as unknown as ThemeDocument[];

export const BUILT_IN_THEMES: CodexTheme[] = documents.map((theme) =>
  documentToTheme(theme, true),
);

export const DEFAULT_THEME: CodexTheme = BUILT_IN_THEMES[0];
