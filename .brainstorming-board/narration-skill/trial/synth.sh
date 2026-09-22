#!/usr/bin/env bash
# 試作：narration.json から、枚ごとの音声と尺を作り、narration.out.json を書く。
# 鍵は「文・声・エンジン」から作り、その鍵を音声の名前にする（論点4の形を手元で再現する）。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
voice="${1:-Takumi}"
in="$here/narration.json"
cache="$here/cache"; mkdir -p "$cache"
python3 - "$in" "$voice" <<'PY' > "$here/plan.tsv"
import json,sys,hashlib
d=json.load(open(sys.argv[1])); voice=sys.argv[2]; engine=d["engine"]
for it in d["items"]:
    key=hashlib.sha256(f"{it['text']}|{voice}|{engine}".encode()).hexdigest()[:16]
    print(it["id"], key, it["text"], sep="\t")
PY
while IFS=$'\t' read -r id key text; do
  mp3="$cache/$key.mp3"; marks="$cache/$key.marks.json"
  if [ -s "$mp3" ]; then echo "取り出す $id $key"; continue; fi
  echo "合成する $id $key"
  aws polly synthesize-speech --voice-id "$voice" --engine neural \
    --output-format mp3 --text "$text" "$mp3" >/dev/null
  aws polly synthesize-speech --voice-id "$voice" --engine neural \
    --output-format json --speech-mark-types '["word","sentence"]' \
    --text "$text" "$marks" >/dev/null
done < "$here/plan.tsv"
python3 - "$in" "$voice" "$cache" <<'PY' > "$here/narration.out.json"
import json,sys,hashlib,pathlib
d=json.load(open(sys.argv[1])); voice=sys.argv[2]; cache=pathlib.Path(sys.argv[3]); engine=d["engine"]
items=[]
for it in d["items"]:
    key=hashlib.sha256(f"{it['text']}|{voice}|{engine}".encode()).hexdigest()[:16]
    marks=[json.loads(l) for l in (cache/f"{key}.marks.json").read_text().splitlines() if l.strip()]
    dur=max((m["time"] for m in marks), default=0)
    items.append({"id":it["id"],"key":key,"audio":f"cache/{key}.mp3",
                  "marks":f"cache/{key}.marks.json","format":"mp3","durationMs":dur})
json.dump({"voice":voice,"engine":engine,"items":items},sys.stdout,ensure_ascii=False,indent=1)
PY
echo "書き出し: $here/narration.out.json"
