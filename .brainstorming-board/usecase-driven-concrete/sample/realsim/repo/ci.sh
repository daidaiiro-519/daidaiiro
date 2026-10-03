#!/usr/bin/env bash
# CI と同じ順で流す。テストの合否は実行器が、仕様との対応は concrete が判定する。どちらか1つでも落ちれば CI は落ちる
cd "$(dirname "$0")"; export CONCRETE_ROOT="$PWD"
mkdir -p build; export CONCRETE_TRACE="$PWD/build/trace.jsonl"; rm -f "$CONCRETE_TRACE"
st=0
echo '$ concrete check'; python3 tool/concrete.py check || st=1
echo; echo '$ pytest -q tests'; python3 -m pytest -q tests 2>&1 | tail -3; [ ${PIPESTATUS[0]} -eq 0 ] || st=1
echo; echo '$ (cd systemtest && go test -count=1 ./...)'; (cd systemtest && go test -count=1 ./... 2>&1 | tail -4; exit ${PIPESTATUS[0]}) || st=1
echo; echo '$ concrete match build/trace.jsonl'; python3 tool/concrete.py match build/trace.jsonl || st=1
echo; [ $st -eq 0 ] && echo 'CI：通った' || echo 'CI：落ちた'
exit $st
