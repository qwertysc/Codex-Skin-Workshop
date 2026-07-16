import { useEffect, useMemo, useState } from "react";
import type { CSSProperties } from "react";
import type { ApplyStatus, CodexInstallation, CodexTheme, ThemeColors, ThemeEffects } from "./types/theme";
import { duplicateTheme } from "./types/theme";
import { BUILT_IN_THEMES, DEFAULT_THEME } from "./themes";
import {
  applyTheme,
  chooseThemeImage,
  detectCodex,
  exportTheme,
  importTheme,
  loadThemes,
  restoreDefault,
  saveTheme,
} from "./lib/tauri";

const COLOR_LABELS: Array<[keyof ThemeColors, string]> = [
  ["background", "页面背景"], ["panel", "主面板"], ["panel-alt", "次级面板"],
  ["accent", "强调色"], ["accent-alt", "辅助强调色"], ["secondary", "次要色"],
  ["highlight", "高亮色"], ["text", "文字颜色"], ["muted", "弱化文字"], ["line", "边框颜色"],
];

function cloneTheme(theme: CodexTheme): CodexTheme {
  return { ...theme, image: { ...theme.image }, colors: { ...theme.colors }, effects: { ...theme.effects } };
}

function Slider({ label, value, min = 0, max = 100, unit = "%", onChange }: { label: string; value: number; min?: number; max?: number; unit?: string; onChange: (value: number) => void }) {
  return <label className="slider-row"><span>{label}<b>{value}{unit}</b></span><input type="range" min={min} max={max} value={value} onChange={(event) => onChange(Number(event.target.value))} /></label>;
}

function ColorField({ label, value, onChange }: { label: string; value: string; onChange: (value: string) => void }) {
  return <label className="color-field"><span>{label}</span><div><input type="color" value={value} onChange={(event) => onChange(event.target.value)} /><code>{value.toUpperCase()}</code></div></label>;
}

function Toggle({ label, checked, onChange }: { label: string; checked: boolean; onChange: (value: boolean) => void }) {
  return <label className="toggle-row"><span>{label}</span><input type="checkbox" checked={checked} onChange={(event) => onChange(event.target.checked)} /></label>;
}

export default function App() {
  const [theme, setTheme] = useState<CodexTheme>(() => cloneTheme(DEFAULT_THEME));
  const [installation, setInstallation] = useState<CodexInstallation | null>(null);
  const [savedThemes, setSavedThemes] = useState<CodexTheme[]>([]);
  const [status, setStatus] = useState<ApplyStatus>("idle");
  const [notice, setNotice] = useState("修改会立即显示在预览中；只有点击“应用到 Codex”才会改变 Codex。 ");

  useEffect(() => {
    detectCodex().then(setInstallation).catch((error) => setInstallation({ installed: false, writable: false, message: String(error) }));
    loadThemes().then(setSavedThemes).catch((error) => setNotice(`读取本地主题失败：${String(error)}`));
  }, []);

  const themes = useMemo(() => [...BUILT_IN_THEMES, ...savedThemes], [savedThemes]);
  const background = useMemo(() => theme.image.kind === "local" && theme.image.previewUrl
    ? `url("${theme.image.previewUrl}")`
    : `radial-gradient(circle at ${theme.effects.imagePositionX}% ${theme.effects.imagePositionY}%, ${theme.colors.accent}45 0, transparent 28%), radial-gradient(circle at 18% 82%, ${theme.colors.secondary}55 0, transparent 26%), linear-gradient(135deg, ${theme.colors["panel-alt"]}, ${theme.colors.background})`, [theme]);

  const updateEffect = <K extends keyof ThemeEffects>(key: K, value: ThemeEffects[K]) => setTheme((old) => ({ ...old, effects: { ...old.effects, [key]: value } }));
  const updateColor = (key: keyof ThemeColors, value: string) => setTheme((old) => ({ ...old, colors: { ...old.colors, [key]: value } }));
  const updateText = (key: "name" | "subtitle" | "tagline" | "projectPrefix" | "projectLabel" | "statusText" | "quote", value: string) => setTheme((old) => ({ ...old, [key]: value }));

  async function refreshThemes(selected?: CodexTheme) {
    const loaded = await loadThemes();
    setSavedThemes(loaded);
    if (selected) setTheme(cloneTheme(selected));
  }

  async function pickImage() {
    try {
      const selected = await chooseThemeImage();
      if (!selected) return;
      setTheme((old) => ({
        ...old,
        image: { kind: "local", path: selected.path, previewUrl: selected.previewUrl },
        colors: {
          ...old.colors,
          accent: selected.palette[0] ?? old.colors.accent,
          secondary: selected.palette[1] ?? old.colors.secondary,
          highlight: selected.palette[2] ?? old.colors.highlight,
        },
      }));
      setNotice("图片已导入，并根据图片生成了推荐配色。 ");
    } catch (error) { setNotice(`图片导入失败：${String(error)}`); }
  }

  async function handleSave() {
    try {
      const toSave = theme.builtIn ? duplicateTheme(theme) : { ...theme, builtIn: false };
      await saveTheme(toSave);
      await refreshThemes(toSave);
      setNotice("主题已保存到本机主题库。 ");
    } catch (error) { setNotice(`主题保存失败：${String(error)}`); }
  }

  async function handleApply() {
    setStatus("applying"); setNotice("正在启动 Codex 并应用主题，请稍候……");
    try {
      const toApply = theme.builtIn ? duplicateTheme(theme) : theme;
      await saveTheme(toApply);
      await applyTheme(toApply);
      await refreshThemes(toApply);
      setStatus("success"); setNotice("主题已应用到 Codex。退出 Codex 后再次应用，也会自动重新启动并等待窗口就绪。 ");
    } catch (error) { setStatus("error"); setNotice(`应用主题失败：${String(error)}`); }
  }

  async function handleRestore() {
    if (!window.confirm("恢复 Codex 原始外观？已保存的主题不会删除。")) return;
    try { await restoreDefault(); setNotice("已恢复 Codex 原始外观。 "); setStatus("idle"); }
    catch (error) { setNotice(`恢复失败：${String(error)}`); setStatus("error"); }
  }

  async function handleImport() {
    try {
      const imported = await importTheme();
      if (!imported) return;
      const copy = themes.some((item) => item.id === imported.id) ? duplicateTheme(imported) : imported;
      await saveTheme(copy);
      await refreshThemes(copy);
      setNotice("主题文件已导入。分享文件只包含纯数据，不会执行 CSS 或脚本。 ");
    } catch (error) { setNotice(`导入主题失败：${String(error)}`); }
  }

  async function handleExport() {
    try {
      if (await exportTheme(theme)) setNotice("主题文件已导出。为保护隐私和兼容性，本机背景图片不会写入分享文件。 ");
    } catch (error) { setNotice(`导出主题失败：${String(error)}`); }
  }

  return <div className="app-shell">
    <header>
      <div className="brand"><span className="brand-mark">C</span><div className="brand-copy"><strong>Codex 皮肤工坊</strong><a href="https://1token-store.com" target="_blank" rel="noreferrer">TokenToken · https://1token-store.com</a><small>满血模型，一键接入 Codex</small></div></div>
      <div className="header-actions"><button className="ghost" onClick={handleRestore}>恢复原始外观</button><button className="primary" disabled={status === "applying" || installation?.writable === false} onClick={handleApply}>{status === "applying" ? "正在应用……" : "应用到 Codex"}</button></div>
    </header>

    <main>
      <aside className="left-panel">
        <section className={`detect-card ${installation?.installed ? "ready" : ""}`}><div className="status-icon">{installation === null ? "…" : installation.installed ? "✓" : "!"}</div><div><small>CODEX 检测</small><h3>{installation === null ? "正在查找 Codex……" : installation.installed ? "Codex 已就绪" : "未找到 Codex"}</h3><p>{installation?.path ?? installation?.message ?? "正在检查常用安装位置"}</p></div></section>

        <div className="step-title"><span>1</span><div><h2>选择背景图片</h2><p>支持 PNG、JPG、WebP 和 GIF。</p></div></div>
        <button className="upload-zone" onClick={pickImage}><span>↑</span><strong>选择图片</strong><small>建议使用 1920 × 1080 或更大尺寸</small></button>

        <div className="step-title compact"><span>2</span><div><h2>调整图片与质感</h2><p>所有参数都会实时显示在右侧。</p></div></div>
        <div className="control-card">
          <Slider label="背景亮度" value={theme.effects.brightness} min={10} max={200} onChange={(value) => updateEffect("brightness", value)} />
          <Slider label="背景模糊" value={theme.effects.blur} max={40} unit="px" onChange={(value) => updateEffect("blur", value)} />
          <Slider label="背景透明度" value={Math.round(theme.effects.opacity * 100)} onChange={(value) => updateEffect("opacity", value / 100)} />
          <Slider label="色彩强度" value={theme.effects.saturation} max={200} onChange={(value) => updateEffect("saturation", value)} />
          <Slider label="图片横向位置" value={theme.effects.imagePositionX} onChange={(value) => updateEffect("imagePositionX", value)} />
          <Slider label="图片纵向位置" value={theme.effects.imagePositionY} onChange={(value) => updateEffect("imagePositionY", value)} />
          <Slider label="图片缩放" value={theme.effects.imageScale} min={50} max={200} onChange={(value) => updateEffect("imageScale", value)} />
          <Slider label="面板透明度" value={Math.round(theme.effects.panelOpacity * 100)} onChange={(value) => updateEffect("panelOpacity", value / 100)} />
          <Slider label="面板模糊" value={theme.effects.panelBlur} max={60} unit="px" onChange={(value) => updateEffect("panelBlur", value)} />
          <Slider label="圆角" value={theme.effects.cornerRadius} max={40} unit="px" onChange={(value) => updateEffect("cornerRadius", value)} />
          <Slider label="内容最大宽度" value={theme.effects.contentMaxWidth} min={640} max={1600} unit="px" onChange={(value) => updateEffect("contentMaxWidth", value)} />
        </div>

        <details><summary>颜色与装饰</summary><div className="colors-grid">{COLOR_LABELS.map(([key, label]) => <ColorField key={key} label={label} value={theme.colors[key]} onChange={(value) => updateColor(key, value)} />)}</div><div className="toggle-grid"><Toggle label="显示主题标题" checked={theme.effects.showBrand} onChange={(value) => updateEffect("showBrand", value)} /><Toggle label="显示状态文字" checked={theme.effects.showStatus} onChange={(value) => updateEffect("showStatus", value)} /><Toggle label="显示引言" checked={theme.effects.showQuote} onChange={(value) => updateEffect("showQuote", value)} /><Toggle label="显示轨道装饰" checked={theme.effects.showOrbit} onChange={(value) => updateEffect("showOrbit", value)} /><Toggle label="显示粒子" checked={theme.effects.showParticles} onChange={(value) => updateEffect("showParticles", value)} /><Slider label="粒子数量" value={theme.effects.particleCount} max={24} unit="" onChange={(value) => updateEffect("particleCount", value)} /></div></details>
      </aside>

      <section className="workspace">
        <div className="preview-heading"><div><span className="eyebrow">实时预览</span><h1>打造属于你的 Codex</h1></div><div className="top-buttons"><button className="ghost" onClick={handleSave}>保存到主题库</button><button className="ghost" onClick={handleImport}>导入主题</button><button className="ghost" onClick={handleExport}>导出当前主题</button></div></div>

        <div className="text-fields"><label><span>主题名称</span><input value={theme.name} onChange={(event) => updateText("name", event.target.value)} /></label><label><span>副标题</span><input value={theme.subtitle} onChange={(event) => updateText("subtitle", event.target.value)} /></label><label><span>主题说明</span><input value={theme.tagline} onChange={(event) => updateText("tagline", event.target.value)} /></label><label><span>项目名前缀</span><input value={theme.projectPrefix} onChange={(event) => updateText("projectPrefix", event.target.value)} /></label><label><span>项目栏标题</span><input value={theme.projectLabel} onChange={(event) => updateText("projectLabel", event.target.value)} /></label><label><span>状态文字</span><input value={theme.statusText} onChange={(event) => updateText("statusText", event.target.value)} /></label><label><span>引言</span><input value={theme.quote} onChange={(event) => updateText("quote", event.target.value)} /></label></div>

        <div className="preview-window" style={{ "--accent": theme.colors.accent, "--panel": theme.colors.panel, "--text": theme.colors.text, "--radius": `${theme.effects.cornerRadius}px` } as CSSProperties}>
          <div className="preview-bg" style={{ backgroundImage: background, backgroundSize: `${theme.effects.imageScale}% auto`, backgroundPosition: `${theme.effects.imagePositionX}% ${theme.effects.imagePositionY}%`, filter: `brightness(${theme.effects.brightness}%) saturate(${theme.effects.saturation}%) blur(${theme.effects.blur}px)`, opacity: theme.effects.opacity }} />
          {theme.effects.showBrand && <div className="preview-brand"><strong>{theme.name}</strong><small>{theme.subtitle}</small></div>}
          {theme.effects.showStatus && <div className="preview-status">{theme.statusText}</div>}
          {theme.effects.showQuote && <div className="preview-quote">{theme.quote}</div>}
          {theme.effects.showOrbit && <div className="preview-orbit" />}
          {theme.effects.showParticles && <div className="preview-particles">{Array.from({ length: theme.effects.particleCount }, (_, index) => <i key={index} style={{ left: `${12 + index * 37 % 78}%`, top: `${18 + index * 53 % 68}%` }} />)}</div>}
          <div className="mock-sidebar" style={{ background: `color-mix(in srgb, ${theme.colors.panel} ${Math.round(theme.effects.panelOpacity * 100)}%, transparent)`, backdropFilter: `blur(${theme.effects.panelBlur}px)` }}><div className="traffic">● ● ●</div><button className="mock-new">＋ 新建对话</button><nav><span>⌕ 搜索</span><span>▱ 智能体</span><span>⌁ 技能</span></nav><small>最近</small><p className="active">制作一个落地页</p><p>解释这个代码仓库</p><div className="mock-user"><i>CS</i><span>皮肤工坊用户<small>本地工作区</small></span></div></div>
          <div className="mock-content"><div className="mock-top">Codex <span>⌘ K</span></div><div className="conversation"><div className="welcome"><span>✦</span><h3>{theme.name}</h3><p>{theme.tagline}</p></div><div className="composer"><span>描述一个任务……</span><div><button>＋</button><small>本地 ▾</small><button className="send">↑</button></div></div><div className="hint">按 Enter 发送 · Shift + Enter 换行</div></div></div>
        </div>

        <div className={`notice ${status}`}><span>{status === "success" ? "✓" : status === "error" ? "!" : "i"}</span>{notice}</div>
        <div className="library-heading"><div><span className="eyebrow">主题模板</span><h2>选择模板或继续创作</h2></div><p>每个内置模板都是独立 JSON 文件；自建主题也可以导出分享。</p></div>
        <div className="theme-grid">{themes.map((item) => <button key={`${item.builtIn ? "built" : "saved"}-${item.id}`} className={`theme-card ${theme.id === item.id ? "selected" : ""}`} onClick={() => setTheme(cloneTheme(item))}><div className="theme-art" style={{ background: `linear-gradient(135deg, ${item.colors.background}, ${item.colors["panel-alt"]})`, "--card-accent": item.colors.accent } as CSSProperties}><span>✦</span></div><div><strong>{item.name}</strong><small>{item.builtIn ? "内置模板" : "本机主题"}</small></div><i style={{ background: item.colors.accent }} /></button>)}</div>
      </section>
    </main>
    <footer><span>Codex 皮肤工坊 · 开源社区项目</span><span>安全预览，确认后再应用。</span></footer>
  </div>;
}
