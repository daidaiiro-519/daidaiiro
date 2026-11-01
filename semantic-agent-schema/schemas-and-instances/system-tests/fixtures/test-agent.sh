#!/bin/sh
# 試験用のエージェント（接続の定義で起動する）。実際の AI は使わない。
# 呼ばれるたびに数を1つ進め、受け取ったプロンプトと環境を agent/ に残し、agent/reply-<n> の決まった出力を返す。
# reply-<n>.exit があれば、その終了コードで終わる（時間切れ ・ 失敗のふり）。
# 1つ目の引数は start か continue、2つ目は続ける子の ID。
dir=agent
mkdir -p "$dir"
n=$(cat "$dir/count" 2>/dev/null || echo 0)
n=$((n + 1))
echo "$n" > "$dir/count"
cat > "$dir/prompt-$n.txt"
echo "$1 $2" > "$dir/call-$n.txt"
env > "$dir/env-$n.txt"
if [ -f "$dir/touch-$n" ]; then cp "$dir/touch-$n" "$(cat "$dir/touch-$n.path")"; fi
if [ -f "$dir/reply-$n.exit" ]; then exit "$(cat "$dir/reply-$n.exit")"; fi
cat "$dir/reply-$n.json"
