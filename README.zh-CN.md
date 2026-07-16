# Codex Skin Workshop

一个面向普通用户的开源 Codex 外观工作台。选择背景图片，调整颜色与视觉效果，实时查看预览，确认满意后再应用到 Codex。

> 项目状态：早期 MVP。可视化编辑器、纯数据主题存储、托管启动、仅回环 CDP 注入与恢复路径已经实现；正式发布前仍需通过 GitHub Actions 和真实设备验证平台兼容性。

[English](README.md)

## 赞助商

感谢 [TokenToken](https://1token-store.com) 赞助本项目——满血模型，一键接入 Codex。

## 便携版下载

项目只提供 macOS arm64、macOS x64 和 Windows x64 便携 ZIP。请完整解压，并始终通过包内的 `启动 Codex Skin Workshop.command` 或 `启动 Codex Skin Workshop.cmd` 进入；无需安装，也不需要管理员权限。

## 主要特点

- **小白友好的步骤**：检测 Codex、选择图片、调整效果、查看预览、点击应用。
- **安全实时预览**：编辑过程不会修改 Codex，只有点击“Apply to Codex”后才会调用应用命令。
- **可视化参数**：强调色、表面色、文字色，以及亮度、模糊、暗色遮罩和饱和度。
- **主题库**：提供内置起点，并为本地保存主题准备了前端模型。
- **恢复入口**：界面中提供清晰的“恢复原始外观”操作。
- **轻量前端**：React、TypeScript、Vite 和自研 CSS，不引入重型 UI 框架。
- **Tauri 2 对接**：前端通过带类型的接口调用检测、保存、应用和恢复命令。

## 技术栈

- Tauri 2
- React 18 + TypeScript
- Vite 5
- 手写 CSS

## 本地开发

请先准备当前版本的 Node.js/npm，以及对应操作系统所需的 Rust/Tauri 开发环境。

开发检查、测试与便携版打包统一通过 GitHub Actions 完成。Fork 仓库并推送分支后，运行 **Check** 和 **Build portable previews** 工作流即可，避免在低配置本机执行重型跨平台构建。

## 前后端命令约定

前端目前会调用以下 Tauri commands：

| 命令 | 用途 |
| --- | --- |
| `detect_codex` | 查找 Codex，并返回版本和写入权限状态 |
| `list_themes` | 读取本机保存的主题摘要 |
| `save_theme` | 校验并保存主题 |
| `import_image` | 校验、清理、复制并分析用户选择的图片 |
| `launch_codex` | 启动由本工具管理、仅开放回环 CDP 的 Codex 进程 |
| `apply_theme` | 通过受控预览目标应用仅包含数据的主题 |
| `restore_codex` | 移除工作台样式并关闭托管预览进程 |

当前后端采用托管 Codex 预览进程和仅限回环地址的 DevTools 连接，不直接修改应用文件。正式发布前仍需对平台兼容性和恢复行为进行广泛测试。

## 目录结构

```text
src/
  App.tsx            可视化工作台界面
  styles.css         自研桌面端样式
  types/theme.ts     主题与 Codex 安装状态类型
  lib/tauri.ts       带类型的前端命令桥接
src-tauri/            Tauri 2 应用与命令脚手架
```

## 安全目标

应用后端完成前至少需要做到：

1. 只启动经过检测的 Codex 可执行文件；
2. 不修改 Codex 安装包、`app.asar`、凭据、API Key 或模型供应商配置；
3. 主题只包含经过验证的数值和清理后的本地图片，不允许可执行 CSS/JS；
4. CDP 仅使用随机 `127.0.0.1` 端口和严格匹配的页面 WebSocket；
5. 保留可靠恢复路径，并且只停止由工作台启动的进程；
6. 明确报告错误，不隐藏失败。

## 参与贡献

欢迎提交 Issue 和 Pull Request。请始终保持普通用户也能理解的操作体验，避免不必要的依赖，并记录不同平台的行为差异。

## 许可证

MIT，详见 [LICENSE](LICENSE) 与 [NOTICE.md](NOTICE.md)。CDP 换肤思路基于 MIT 许可的 Codex Dream Skin 项目进行改造；凭据及模型供应商配置不属于本项目范围。
