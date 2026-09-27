#!/bin/bash
# 7つの言語で、テストが走ったときに名乗ったシナリオを集め、道具で突き合わせる。
# 使い方: ./run.sh <道具の covered の場所> <JUnit の console launcher の jar> <phpunit.phar>
# jar と phar は同梱しない ── 取得元: repo1.maven.org（junit-platform-console-standalone 1.11.4）・ phar.phpunit.de（phpunit-11）
set -u
T=$(cd "$(dirname "$0")" && pwd); TOOL=$1; JUNIT=$2; PHPUNIT=$3
ci() { local n=$1; shift; local tr="$T/.trace-$n.txt"; rm -f "$tr"
  ( export SCENARIO_TRACE="$tr"; "$@" ) >/dev/null 2>&1
  printf "%-11s " "$n"; "$TOOL" "$tr" AGG-01J7Q4K | tail -1; }
(cd "$T/go"   && ci Go         go test ./...)
(cd "$T/py"   && PYTHONPATH=. ci Python python3 -m pytest -q)
(cd "$T/ts"   && ci TypeScript node --test)
(cd "$T/rs"   && ci Rust       cargo test -q)
(cd "$T/rb"   && ci Ruby       ruby test/order_test.rb)
(cd "$T/php"  && ci PHP        php "$PHPUNIT" tests)
(cd "$T/java" && mkdir -p out && javac -cp "$JUNIT" -d out src/*.java && ci Java java -jar "$JUNIT" execute -cp out --scan-classpath --details=none)
rm -f "$T"/.trace-*.txt
