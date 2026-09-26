<?php
// SPDX-License-Identifier: MIT
// 参照と抜け道を、標準同梱の字句解析（token_get_all）から取る。**解析器を自作しない。**
//
//     php php.php <根>
//
// 経路を組み立てて読み込む箇所は、行き先が静的に決まらないので抜け道として返す。
declare(strict_types=1);

$root = $argv[1] ?? null;
if ($root === null) {
    fwrite(STDERR, "根を渡していない\n");
    exit(2);
}
$base = realpath($root);
if ($base === false) {
    fwrite(STDERR, "根が無い\n");
    exit(2);
}

const LOADERS = ['T_REQUIRE', 'T_REQUIRE_ONCE', 'T_INCLUDE', 'T_INCLUDE_ONCE'];

/**
 * 経路を点の名前へ直す。**解決しないと、同じ名前が別の層を指す。**
 */
function point(string $rel, string $spec): string
{
    $joined = dirname($rel) . '/' . ltrim($spec, '/');
    $parts = [];
    foreach (explode('/', $joined) as $piece) {
        if ($piece === '' || $piece === '.') {
            continue;
        }
        if ($piece === '..') {
            array_pop($parts);
            continue;
        }
        $parts[] = $piece;
    }
    return implode('/', $parts);
}
const NAMES = ['T_NAME_QUALIFIED', 'T_STRING', 'T_NAME_FULLY_QUALIFIED'];

/**
 * そのファイルが宣言している名前空間を返す。
 *
 * **参照する側と参照される側を、同じ識別子の空間で出す。** 片方を経路、もう片方を
 * 名前空間にすると、層に当たらないまま違反が消える。
 */
function namespaceOf(array $tokens): string
{
    $total = count($tokens);
    for ($i = 0; $i < $total; $i++) {
        $token = $tokens[$i];
        if (!is_array($token) || token_name($token[0]) !== 'T_NAMESPACE') {
            continue;
        }
        $parts = [];
        for ($j = $i + 1; $j < $total; $j++) {
            $next = $tokens[$j];
            if ($next === ';' || $next === '{') {
                break;
            }
            if (is_array($next) && in_array(token_name($next[0]), NAMES, true)) {
                $parts[] = $next[1];
            }
        }
        return implode('', $parts);
    }
    return '';
}

$files = [];
$walk = new RecursiveIteratorIterator(new RecursiveDirectoryIterator($base));
foreach ($walk as $f) {
    if ($f->isFile() && strtolower($f->getExtension()) === 'php') {
        $files[] = $f->getPathname();
    }
}
sort($files);

$edges = [];
$escapes = [];
$undecided = [];
foreach ($files as $file) {
    $rel = ltrim(substr($file, strlen($base)), '/');
    $body = file_get_contents($file);
    if ($body === false) {
        $undecided[] = "{$rel} ── 読めない";
        continue;
    }
    $tokens = @token_get_all($body);
    $total = count($tokens);
    $here = namespaceOf($tokens);
    if ($here === '') {
        $undecided[] = "{$rel} ── 名前空間を宣言していない";
        continue;
    }
    for ($i = 0; $i < $total; $i++) {
        $token = $tokens[$i];
        if (!is_array($token)) {
            continue;
        }
        $kind = token_name($token[0]);
        if ($kind === 'T_USE') {
            $parts = [];
            for ($j = $i + 1; $j < $total; $j++) {
                $next = $tokens[$j];
                if ($next === ';' || $next === '{' || $next === '(') {
                    break;
                }
                if (is_array($next) && in_array(token_name($next[0]), NAMES, true)) {
                    $parts[] = $next[1];
                }
            }
            if ($parts !== []) {
                $edges[] = ['from' => $here, 'to' => implode('', $parts),
                            'at' => "{$rel}:{$token[2]}", 'how' => 'use'];
            }
            continue;
        }
        if (!in_array($kind, LOADERS, true)) {
            continue;
        }
        $literal = null;
        $built = false;
        for ($j = $i + 1; $j < $total; $j++) {
            $next = $tokens[$j];
            if ($next === ';') {
                break;
            }
            if (!is_array($next)) {
                continue;
            }
            $sort = token_name($next[0]);
            if ($sort === 'T_CONSTANT_ENCAPSED_STRING') {
                $literal = trim($next[1], "'\"");
            } elseif ($sort === 'T_VARIABLE' || $sort === 'T_DIR') {
                $built = true;
            }
        }
        if ($built || $literal === null) {
            $escapes[] = ['in' => $here, 'at' => "{$rel}:{$token[2]}",
                          'how' => '図に現れない読み込み ── 経路を組み立てている'];
        } else {
            // **読み込みの経路は、名前空間の空間に無い。** 層の判定には使わないので
            // 抜け道として扱わず、辺としてだけ残す（層に当たらなければ対象外になる）
            $edges[] = ['from' => $here, 'to' => point($rel, $literal),
                        'at' => "{$rel}:{$token[2]}", 'how' => strtolower($kind)];
        }
    }
}

echo json_encode(['edges' => $edges, 'escapes' => $escapes, 'undecided' => $undecided],
                 JSON_UNESCAPED_UNICODE), PHP_EOL;
