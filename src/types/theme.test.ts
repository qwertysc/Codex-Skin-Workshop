import { describe, expect, it } from "vitest";
import { BUILT_IN_THEMES, DEFAULT_THEME } from "../themes";
import { documentToTheme, themeToDocument } from "./theme";

describe("主题文件", () => {
  it("所有内置主题都使用受限数值和十六进制颜色", () => {
    expect(BUILT_IN_THEMES.length).toBeGreaterThanOrEqual(4);
    for (const theme of BUILT_IN_THEMES) {
      expect(theme.effects.brightness).toBeGreaterThanOrEqual(10);
      expect(theme.effects.brightness).toBeLessThanOrEqual(200);
      expect(theme.effects.opacity).toBeGreaterThanOrEqual(0);
      expect(theme.effects.opacity).toBeLessThanOrEqual(1);
      for (const color of Object.values(theme.colors)) {
        expect(color).toMatch(/^#[0-9a-f]{6}$/i);
      }
    }
  });

  it("主题往返转换不会携带可执行字段", () => {
    const document = themeToDocument(DEFAULT_THEME);
    expect(document).not.toHaveProperty("css");
    expect(document).not.toHaveProperty("javascript");
    expect(document).not.toHaveProperty("html");
    expect(documentToTheme(document).colors).toEqual(DEFAULT_THEME.colors);
  });

  it("分享文档只保留纯数据字段", () => {
    const document = themeToDocument(DEFAULT_THEME);
    expect(Object.keys(document).sort()).toEqual([
      "backgroundImage", "blurPx", "brightnessPct", "colors", "contentMaxWidthPx",
      "cornerRadiusPx", "id", "imagePositionXPct", "imagePositionYPct", "imageScalePct",
      "name", "opacity", "panelBlurPx", "panelOpacity", "particleCount", "projectLabel",
      "projectPrefix", "quote", "saturationPct", "schemaVersion", "showBrand", "showOrbit",
      "showParticles", "showQuote", "showStatus", "statusText", "subtitle", "tagline",
    ].sort());
  });
});
