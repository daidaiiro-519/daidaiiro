#!/bin/sh
# Skill を導入する（macOS ・ Linux ・ WSL）。利用者の環境（OS と CPU）を判別し、合う配布物だけを取得する。
#   curl -fsSL https://github.com/daidaiiro-519/daidaiiro/releases/latest/download/install.sh | sh -s -- [Skill の名前 ...]
# 名前を省略すると、配布元にある全部を導入する。導入先は、実行した場所のリポジトリの .claude/skills/。
# 置いたのは skills-creator の dist である。手を入れてよい ── dist は既に在るものを上書きしない。
set -eu

BASE="${SKILLS_BASE:-https://github.com/daidaiiro-519/daidaiiro/releases/latest/download}"

die() { echo "導入できない ── $*" >&2; exit 1; }

case "$(uname -s)" in
  Linux) os=linux ;;
  Darwin) os=macos ;;
  *) die "この OS には対応していない（$(uname -s)）。Windows は install.ps1 を使う" ;;
esac
case "$os-$(uname -m)" in
  linux-x86_64|linux-amd64) target=x86_64-unknown-linux-musl ;;
  macos-*) target=universal2-apple-darwin ;;
  *) die "この CPU には対応していない（$(uname -m)）" ;;
esac

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
dest="$root/.claude/skills"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

curl -fsSL "$BASE/SHA256SUMS" -o "$work/SHA256SUMS" || die "照合の一覧を取得できない（$BASE/SHA256SUMS）"

if [ "$#" -eq 0 ]; then
  # 名前を省略したら、この環境向けの配布物と、環境に依存しない配布物の全部
  set -- $(sed -n "s/.*  \(.*\)-$target\.tar\.gz$/\1/p; s/.*  \(.*\)-any\.tar\.gz$/\1/p" "$work/SHA256SUMS" | sort -u)
fi

sum() { if command -v sha256sum >/dev/null; then sha256sum "$1"; else shasum -a 256 "$1"; fi | cut -d' ' -f1; }

mkdir -p "$dest"
for name in "$@"; do
  file=""
  for cand in "$name-$target.tar.gz" "$name-any.tar.gz"; do
    if grep -q "  $cand\$" "$work/SHA256SUMS"; then file="$cand"; break; fi
  done
  [ -n "$file" ] || die "$name の、この環境（$target）向けの配布物が無い"
  curl -fsSL "$BASE/$file" -o "$work/$file" || die "$file を取得できない"
  want="$(grep "  $file\$" "$work/SHA256SUMS" | cut -d' ' -f1)"
  [ "$(sum "$work/$file")" = "$want" ] || die "$file の SHA-256 が一覧と一致しない"
  rm -rf "${dest:?}/$name"
  tar -xzf "$work/$file" -C "$dest"
  # macOS：隔離の属性は、ファイルを作ったアプリの設定で決まる。照合の後に必ず外す
  if [ "$os" = macos ] && command -v xattr >/dev/null; then
    xattr -dr com.apple.quarantine "$dest/$name" 2>/dev/null || true
  fi
  if [ -x "$dest/$name/bin/$name-mcp" ]; then
    if command -v claude >/dev/null; then
      (cd "$root" && claude mcp remove --scope project "$name" >/dev/null 2>&1 || true)
      (cd "$root" && claude mcp add --scope project "$name" -- \
        "\${CLAUDE_PROJECT_DIR:-.}/.claude/skills/$name/bin/$name-mcp" >/dev/null)
    else
      echo "MCP の登録を省いた（claude が見つからない）: $name" >&2
    fi
  fi
  echo "導入した: $name（$file）"
done
