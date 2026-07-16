#!/bin/bash
set -u

HERE="$(cd "$(dirname "$0")" && pwd)"
APP="$HERE/Codex Skin Workshop.app"

show_error() {
  /usr/bin/osascript -e "display alert \"Codex Skin Workshop\" message \"$1\" as critical" >/dev/null 2>&1 || true
  printf '\n%s\n' "$1"
  read -r -p "按回车键关闭……" _
  exit 1
}

[ -d "$APP" ] || show_error "找不到 Codex Skin Workshop.app。请完整解压 ZIP，并保留启动器和应用在同一文件夹。"

# Only remove the download quarantine marker from this portable app.
/usr/bin/xattr -dr com.apple.quarantine "$APP" 2>/dev/null || true

# Add a local ad-hoc signature when the unsigned preview does not already verify.
if ! /usr/bin/codesign --verify --deep --strict "$APP" >/dev/null 2>&1; then
  /usr/bin/codesign --force --deep --sign - --timestamp=none "$APP" >/dev/null 2>&1 \
    || show_error "无法准备应用。请确认 ZIP 已完整解压，并从本启动器进入。"
fi

/usr/bin/open "$APP" || show_error "应用启动失败。"
exit 0
