import { useEffect, useMemo, useState } from "react";
import type { ApplyStatus, CodexInstallation, CodexTheme, ThemeSummary } from "./types/theme";
import { DEFAULT_THEME } from "./types/theme";
import { applyTheme, chooseThemeImage, detectCodex, loadThemeLibrary, restoreDefault, saveTheme } from "./lib/tauri";

const builtIns: ThemeSummary[] = [
  { id: "midnight-bloom", name: "Midnight Bloom", accent: "#b7ff5a", updatedAt: "Built in", builtIn: true },
  { id: "quiet-ocean", name: "Quiet Ocean", accent: "#6ed8ff", updatedAt: "Built in", builtIn: true },
  { id: "warm-paper", name: "Warm Paper", accent: "#ffb36b", updatedAt: "Built in", builtIn: true },
];

const presets: Record<string, Pick<CodexTheme, "colors" | "effects">> = {
  "midnight-bloom": { colors: { accent: "#b7ff5a", surface: "#101319", text: "#f4f7f1" }, effects: { brightness: 72, blur: 1, overlay: 42, saturation: 112 } },
  "quiet-ocean": { colors: { accent: "#6ed8ff", surface: "#071824", text: "#edfaff" }, effects: { brightness: 82, blur: 3, overlay: 38, saturation: 92 } },
  "warm-paper": { colors: { accent: "#ffb36b", surface: "#21150f", text: "#fff8ee" }, effects: { brightness: 92, blur: 0, overlay: 28, saturation: 88 } },
};

function Slider({ label, value, min = 0, max = 100, unit = "%", onChange }: { label: string; value: number; min?: number; max?: number; unit?: string; onChange: (value: number) => void }) {
  return <label className="slider-row"><span>{label}<b>{value}{unit}</b></span><input type="range" min={min} max={max} value={value} onChange={(e) => onChange(Number(e.target.value))} /></label>;
}

function ColorField({ label, value, onChange }: { label: string; value: string; onChange: (value: string) => void }) {
  return <label className="color-field"><span>{label}</span><div><input type="color" value={value} onChange={(e) => onChange(e.target.value)} /><code>{value.toUpperCase()}</code></div></label>;
}

export default function App() {
  const [theme, setTheme] = useState<CodexTheme>({ ...DEFAULT_THEME, createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() });
  const [installation, setInstallation] = useState<CodexInstallation | null>(null);
  const [themes, setThemes] = useState<ThemeSummary[]>(builtIns);
  const [status, setStatus] = useState<ApplyStatus>("idle");
  const [notice, setNotice] = useState("Changes appear here instantly. Codex is only changed when you click Apply.");

  useEffect(() => {
    detectCodex().then(setInstallation).catch((error) => setInstallation({ installed: false, writable: false, message: String(error) }));
    loadThemeLibrary().then((saved) => setThemes([...builtIns, ...saved])).catch(() => undefined);
  }, []);

  const background = useMemo(() => theme.image.kind === "local" && theme.image.previewUrl
    ? `url("${theme.image.previewUrl}")`
    : `radial-gradient(circle at 72% 20%, ${theme.colors.accent}55 0, transparent 25%), radial-gradient(circle at 20% 80%, #765cff55 0, transparent 28%), linear-gradient(135deg, #151824, ${theme.colors.surface})`, [theme]);

  const updateEffects = (key: keyof CodexTheme["effects"], value: number) => setTheme((old) => ({ ...old, effects: { ...old.effects, [key]: value } }));
  const updateColor = (key: keyof CodexTheme["colors"], value: string) => setTheme((old) => ({ ...old, colors: { ...old.colors, [key]: value } }));

  async function pickImage() {
    const selected = await chooseThemeImage();
    if (selected) setTheme((old) => ({ ...old, image: { kind: "local", ...selected } }));
    else setNotice("Image selection is available in the desktop app.");
  }

  async function handleApply() {
    setStatus("applying");
    setNotice("Applying your theme…");
    try {
      const updated = { ...theme, updatedAt: new Date().toISOString() };
      await saveTheme(updated);
      await applyTheme(updated);
      setTheme(updated);
      setStatus("success");
      setNotice("Theme applied. Restart Codex if the open window does not refresh.");
    } catch (error) {
      setStatus("error");
      setNotice(`Could not apply theme: ${String(error)}`);
    }
  }

  async function handleRestore() {
    if (!window.confirm("Restore Codex's original appearance? Your saved themes will stay in the library.")) return;
    try { await restoreDefault(); setNotice("Original Codex appearance restored."); setStatus("idle"); }
    catch (error) { setNotice(`Restore failed: ${String(error)}`); setStatus("error"); }
  }

  function selectPreset(item: ThemeSummary) {
    const preset = presets[item.id];
    if (!preset) return;
    setTheme((old) => ({ ...old, id: item.id, name: item.name, image: { kind: "preset", id: item.id, previewUrl: "" }, ...preset }));
  }

  return <div className="app-shell">
    <header><div className="brand"><span className="brand-mark">C</span><div><strong>Codex Skin Workshop</strong><small>Make Codex feel like yours.</small></div></div><div className="header-actions"><button className="ghost" onClick={handleRestore}>↶ Restore original</button><button className="primary" disabled={status === "applying" || installation?.writable === false} onClick={handleApply}>{status === "applying" ? "Applying…" : "Apply to Codex"}</button></div></header>

    <main>
      <aside className="left-panel">
        <section className={`detect-card ${installation?.installed ? "ready" : ""}`}><div className="status-icon">{installation === null ? "…" : installation.installed ? "✓" : "!"}</div><div><small>CODEX CHECK</small><h3>{installation === null ? "Looking for Codex…" : installation.installed ? "Codex is ready" : "Codex not found"}</h3><p>{installation?.path ?? installation?.message ?? "Checking common install locations"}</p></div></section>
        <div className="step-title"><span>1</span><div><h2>Choose a background</h2><p>PNG, JPG or WebP works best.</p></div></div>
        <button className="upload-zone" onClick={pickImage}><span>↑</span><strong>Choose an image</strong><small>Recommended: 1920 × 1080 or larger</small></button>
        <div className="step-title compact"><span>2</span><div><h2>Tune the look</h2><p>Drag until it feels comfortable.</p></div></div>
        <div className="control-card">
          <Slider label="Brightness" value={theme.effects.brightness} onChange={(v) => updateEffects("brightness", v)} />
          <Slider label="Background blur" value={theme.effects.blur} max={20} unit="px" onChange={(v) => updateEffects("blur", v)} />
          <Slider label="Dark overlay" value={theme.effects.overlay} onChange={(v) => updateEffects("overlay", v)} />
          <Slider label="Color strength" value={theme.effects.saturation} max={160} onChange={(v) => updateEffects("saturation", v)} />
        </div>
        <div className="colors-grid"><ColorField label="Accent" value={theme.colors.accent} onChange={(v) => updateColor("accent", v)} /><ColorField label="Surface" value={theme.colors.surface} onChange={(v) => updateColor("surface", v)} /><ColorField label="Text" value={theme.colors.text} onChange={(v) => updateColor("text", v)} /></div>
      </aside>

      <section className="workspace">
        <div className="preview-heading"><div><span className="eyebrow">LIVE PREVIEW</span><h1>Your Codex, reimagined.</h1></div><label className="name-field"><span>Theme name</span><input value={theme.name} onChange={(e) => setTheme((old) => ({ ...old, name: e.target.value }))} /></label></div>
        <div className="preview-window" style={{ "--accent": theme.colors.accent, "--surface": theme.colors.surface, "--text": theme.colors.text } as React.CSSProperties}>
          <div className="preview-bg" style={{ backgroundImage: background, filter: `brightness(${theme.effects.brightness}%) saturate(${theme.effects.saturation}%) blur(${theme.effects.blur}px)`, transform: `scale(${1 + theme.effects.blur / 200})` }} />
          <div className="preview-overlay" style={{ background: `rgba(4, 6, 9, ${theme.effects.overlay / 100})` }} />
          <div className="mock-sidebar"><div className="traffic">● ● ●</div><button className="mock-new">＋ New thread</button><nav><span>⌕ Search</span><span>▱ Agents</span><span>⌁ Skills</span></nav><small>RECENT</small><p className="active">Build a landing page</p><p>Explain this repository</p><div className="mock-user"><i>CS</i><span>Workshop user<small>Local workspace</small></span></div></div>
          <div className="mock-content"><div className="mock-top">Codex <span>⌘ K</span></div><div className="conversation"><div className="welcome"><span>✦</span><h3>What are we building today?</h3><p>Ask Codex to write code, explain a project, or help shape an idea.</p></div><div className="composer"><span>Describe a task…</span><div><button>＋</button><small>Local ▾</small><button className="send">↑</button></div></div><div className="hint">Enter to send · Shift + Enter for a new line</div></div></div>
        </div>
        <div className={`notice ${status}`}> <span>{status === "success" ? "✓" : status === "error" ? "!" : "i"}</span>{notice}</div>
        <div className="library-heading"><div><span className="eyebrow">THEME LIBRARY</span><h2>Start with a favorite</h2></div><p>Your saved themes live only on this device.</p></div>
        <div className="theme-grid">{themes.map((item) => <button key={item.id} className={`theme-card ${theme.id === item.id ? "selected" : ""}`} onClick={() => selectPreset(item)}><div className={`theme-art art-${item.id}`} style={{ "--card-accent": item.accent } as React.CSSProperties}><span>✦</span></div><div><strong>{item.name}</strong><small>{item.updatedAt}</small></div><i style={{ background: item.accent }} /></button>)}</div>
      </section>
    </main>
    <footer><span>Codex Skin Workshop · Open source</span><span>Preview safely, apply when ready.</span></footer>
  </div>;
}
