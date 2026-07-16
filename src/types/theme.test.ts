import { describe, expect, it } from "vitest";
import { DEFAULT_THEME } from "./theme";

describe("default theme", () => {
  it("uses bounded visual controls and valid hex colors", () => {
    expect(DEFAULT_THEME.effects.brightness).toBeGreaterThanOrEqual(10);
    expect(DEFAULT_THEME.effects.brightness).toBeLessThanOrEqual(200);
    expect(DEFAULT_THEME.effects.overlay).toBeGreaterThanOrEqual(0);
    expect(DEFAULT_THEME.effects.overlay).toBeLessThanOrEqual(100);
    for (const color of Object.values(DEFAULT_THEME.colors)) {
      expect(color).toMatch(/^#[0-9a-f]{6}$/i);
    }
  });

  it("does not carry executable theme fields", () => {
    expect(DEFAULT_THEME).not.toHaveProperty("css");
    expect(DEFAULT_THEME).not.toHaveProperty("javascript");
  });
});