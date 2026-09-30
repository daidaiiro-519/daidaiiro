#!/bin/bash
# SPDX-License-Identifier: MIT
# 提供者が、言語の組の雛形を直したときに流す確かめ（ACDR 0070）。**配布物には入らない**（tool/ の下）。
#
# 配布物と同じ形（bin/ ・ references/ ・ SKILL.md ・ tool.json）を空の作業場所へ置き、利用者と同じ手順で
# 5言語 × 2型（作業型 ・ 助言型）を生む。scaffold が案内する組み立てのコマンドをそのまま実行し、
# 生んだ Skill の試験 ・ check ・ accept（助言型）を流す。最後に、Rust の助言型を基準に、他の4言語の
# references の道具の出力を突き合わせる（conform）。
#
#   tool/verify-profiles.sh [<作業場所>]        （既定は一時フォルダ。Skill のフォルダで実行する）
#
# 必要な処理系：cargo ・ uv ・ node（22.18 以上）・ dotnet（10）・ go。
# 生んだ直後は、SKILL.md の未記入の差し込み場所と、助言型の学習ノートの欠け（accept の2番）が出る ── 正しい結果である。
set -u
# **ICU の無い環境では、dotnet のコマンドそのものが起動時に止まる** ── そのときだけ、文化に依存しない動きにする。
# 生んだ C# の Skill は InvariantGlobalization を保持するので、ICU が無くても動く
if command -v dotnet >/dev/null && [ "$(bash -c 'dotnet --version >/dev/null 2>&1; echo $?' 2>/dev/null)" != 0 ]; then
  export DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=1
fi
here="$(cd "$(dirname "$0")/.." && pwd)"
work="${1:-$(mktemp -d)}"
mkdir -p "$work/.claude/skills/skills-creator"
(cd "$here" && tar --exclude=./tool --exclude=./mcp.json --exclude='./.crates*' --exclude=./.gitignore -cf - .) \
  | tar -xf - -C "$work/.claude/skills/skills-creator"
cd "$work" || exit 2
sc=.claude/skills/skills-creator/bin/skills-creator
[ -x "$sc" ] || { echo "bin/skills-creator が無い ── 先に組み立てる"; exit 2; }
status=0
for lang in rust python typescript csharp go; do
  for ty in work advisor; do
    name="v-$lang-$ty"
    rm -rf ".claude/skills/$name"
    out="$($sc scaffold "$name" --language "$lang" --type "$ty" 2>&1)" || { echo "× [$name] scaffold: $(echo "$out" | tail -1)"; status=1; continue; }
    build="$(echo "$out" | sed -n 's/^組む ── Skill のフォルダで //p')"
    ok=1
    IFS='／' read -ra cmds <<< "$build"
    for cmd in "${cmds[@]}"; do
      (cd ".claude/skills/$name" && eval "$cmd" >"$work/build.log" 2>&1) || { echo "× [$name] 組み立て: $cmd"; tail -5 "$work/build.log"; ok=0; status=1; break; }
    done
    [ $ok = 1 ] || continue
    test_cmd="$(python3 -c "import json;print(' '.join(json.load(open('.claude/skills/skills-creator/references/profiles/$lang.profile.json'))['test']))")"
    (cd ".claude/skills/$name" && eval "$test_cmd" >"$work/test.log" 2>&1) || { echo "× [$name] 試験"; tail -5 "$work/test.log"; status=1; }
    # 生んだ直後に出てよいのは、未記入の差し込み場所だけである
    left="$($sc check ".claude/skills/$name" --json | python3 -c 'import json,sys;print("\n".join(f for f in json.load(sys.stdin)["findings"] if "差し込み場所" not in f))')"
    [ -z "$left" ] || { echo "× [$name] check: $left"; status=1; }
    if [ "$ty" = advisor ]; then
      bad="$($sc accept ".claude/skills/$name" --json | python3 -c 'import json,sys;print(" ".join(str(c["no"]) for c in json.load(sys.stdin)["data"]["checks"] if not c["pass"] and c["no"] != 2))')"
      [ -z "$bad" ] || { echo "× [$name] accept: $bad 番"; status=1; }
    fi
    echo "   [$name] 済"
  done
done
for lang in python typescript csharp go; do
  (cd "$here" && cargo run -q --manifest-path tool/Cargo.toml -p sc_business_logic --example conform -- \
    "$work/.claude/skills/v-rust-advisor" "$work/.claude/skills/v-$lang-advisor" "$(cd "$here/.." && pwd)") \
    | head -3 | sed "s/^/   [conform $lang] /" 
  [ "${PIPESTATUS[0]}" = 0 ] || status=1
done
exit $status
