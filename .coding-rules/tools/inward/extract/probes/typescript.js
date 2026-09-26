// SPDX-License-Identifier: MIT
// 参照を tsc に出させる。**解析器を自作しない。**
//
//     node typescript.js <根>
//
// `tsc --explainFiles` は、非字下げの行が「参照される側」、その直下の
// `Imported via "…" from file 'X'` の X が「参照する側」である（実測 2026-09-26）。
"use strict";

const { execFileSync } = require("node:child_process");
const path = require("node:path");

function run(root) {
  // **npx を経由しない** ── 取得を伴うと、検査の途中で外へ出る
  const local = path.join(root, "node_modules", "typescript", "bin", "tsc");
  const args = ["--explainFiles", "--noEmit"];
  try {
    return execFileSync(process.execPath, [local, ...args], {
      cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"],
    });
  } catch (e) {
    // 型の誤りでも説明は出る ── 標準出力を採る
    if (e.stdout) { return e.stdout; }
    throw e;
  }
}

function main() {
  const root = process.argv[2];
  if (!root) {
    process.stderr.write("根を渡していない\n");
    return 2;
  }
  let text;
  try {
    text = run(root);
  } catch (e) {
    process.stdout.write(JSON.stringify({
      edges: [], escapes: [],
      undecided: [`tsc を呼べないので判定していない ── ${e.message}`],
    }));
    return 0;
  }
  const edges = [];
  const rel = (p) => path.relative(root, path.resolve(root, p)).replace(/\\/g, "/");
  let imported = null;
  for (const line of text.split("\n")) {
    if (!line.startsWith(" ")) {
      imported = line.trim() ? rel(line.trim()) : null;
      continue;
    }
    const m = line.match(/Imported via .* from file '([^']+)'/);
    if (m && imported) {
      edges.push({ from: rel(m[1]), to: imported, at: rel(m[1]), how: "import" });
    }
  }
  process.stdout.write(JSON.stringify({ edges, escapes: [], undecided: [] }));
  return 0;
}

process.exit(main());
