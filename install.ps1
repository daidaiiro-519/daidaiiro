# Skill を導入する（Windows）。利用者の環境を判別し、合う配布物だけを取得する。
#   irm https://github.com/daidaiiro-519/daidaiiro/releases/latest/download/install.ps1 | iex
# 導入する Skill を絞るときは、先に $env:SKILLS に名前を空白で並べる（irm | iex には引数を渡せない）。
# 名前を省略すると、配布元にある全部を導入する。導入先は、実行した場所のリポジトリの .claude\skills\。
# 置いたのは skills-creator の dist である。手を入れてよい ── dist は既に在るものを上書きしない。
$ErrorActionPreference = 'Stop'

$Base = if ($env:SKILLS_BASE) { $env:SKILLS_BASE } else { 'https://github.com/daidaiiro-519/daidaiiro/releases/latest/download' }
# Windows は x86_64 版だけを作る。ARM の Windows は x86_64 のエミュレーションで動かす
$Target = 'x86_64-pc-windows-msvc'

function Die($why) { Write-Error "導入できない ── $why"; exit 1 }

$Root = (git rev-parse --show-toplevel 2>$null)
if (-not $Root) { $Root = (Get-Location).Path }
$Dest = Join-Path $Root '.claude\skills'
$Work = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $Work | Out-Null

try {
  $SumsPath = Join-Path $Work 'SHA256SUMS'
  try { Invoke-WebRequest -UseBasicParsing -Uri "$Base/SHA256SUMS" -OutFile $SumsPath } catch { Die "照合の一覧を取得できない（$Base/SHA256SUMS）" }
  $Sums = @{}
  foreach ($line in Get-Content $SumsPath) {
    if ($line -match '^([0-9a-f]{64})\s+\*?(.+)$') { $Sums[$Matches[2]] = $Matches[1] }
  }

  $Names = @($env:SKILLS -split '\s+' | Where-Object { $_ })
  if ($Names.Count -eq 0) {
    $Names = @($Sums.Keys | ForEach-Object {
      if ($_ -match "^(.+)-$([regex]::Escape($Target))\.zip$") { $Matches[1] }
      elseif ($_ -match '^(.+)-any\.zip$') { $Matches[1] }
    } | Sort-Object -Unique)
  }

  New-Item -ItemType Directory -Force -Path $Dest | Out-Null
  foreach ($name in $Names) {
    $file = @("$name-$Target.zip", "$name-any.zip") | Where-Object { $Sums.ContainsKey($_) } | Select-Object -First 1
    if (-not $file) { Die "$name の、この環境（$Target）向けの配布物が無い" }
    $zip = Join-Path $Work $file
    try { Invoke-WebRequest -UseBasicParsing -Uri "$Base/$file" -OutFile $zip } catch { Die "$file を取得できない" }
    if ((Get-FileHash -Algorithm SHA256 $zip).Hash.ToLower() -ne $Sums[$file]) { Die "$file の SHA-256 が一覧と一致しない" }
    $there = Join-Path $Dest $name
    if (Test-Path $there) { Remove-Item -Recurse -Force $there }
    Expand-Archive -Path $zip -DestinationPath $Dest -Force
    $mcp = Join-Path $there "bin\$name-mcp.exe"
    if (Test-Path $mcp) {
      if (Get-Command claude -ErrorAction SilentlyContinue) {
        Push-Location $Root
        try {
          claude mcp remove --scope project $name 2>$null | Out-Null
          # 経路は Claude Code が展開する ── PowerShell に展開させないよう、単引用符で組む
          $command = '${CLAUDE_PROJECT_DIR:-.}/.claude/skills/' + $name + '/bin/' + $name + '-mcp.exe'
          claude mcp add --scope project $name -- $command | Out-Null
        } finally { Pop-Location }
      } else {
        Write-Warning "MCP の登録を省いた（claude が見つからない）: $name"
      }
    }
    Write-Output "導入した: $name（$file）"
  }
} finally {
  Remove-Item -Recurse -Force $Work -ErrorAction SilentlyContinue
}
