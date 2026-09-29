# Skill を導入する（Windows）。利用者の環境を判別し、合う配布物だけを取得する。
#   $env:SKILLS = '<Skill の名前>[@<版>] ...'; irm https://github.com/daidaiiro-519/daidaiiro/releases/latest/download/install.ps1 | iex
# irm | iex には引数を渡せないので、導入する Skill は $env:SKILLS に空白で並べる。
# 公開は Skill ごとである（tag は <Skill の名前>-v<版>）。版を省略すると、その Skill の最新の版を取得する。
# 名前を1つも渡さなければ、まとめて公開した最新の版（tag は v<版>）から全部を導入する。
# 導入先は、実行した場所のリポジトリの .claude\skills\。
# 置いたのは skills-creator の dist である。手を入れてよい ── dist は既に在るものを上書きしない。
$ErrorActionPreference = 'Stop'

$Repo = 'daidaiiro-519/daidaiiro'
$Api = if ($env:SKILLS_API) { $env:SKILLS_API } else { "https://api.github.com/repos/$Repo/releases?per_page=100" }
$Download = if ($env:SKILLS_DOWNLOAD) { $env:SKILLS_DOWNLOAD } else { "https://github.com/$Repo/releases/download" }
# Windows は x86_64 版だけを作る。ARM の Windows は x86_64 のエミュレーションで動かす
$Target = 'x86_64-pc-windows-msvc'

function Die($why) { Write-Error "導入できない ── $why"; exit 1 }

$script:Tags = $null
# 公開した tag の一覧（新しい順）
function Get-Tags {
  if ($null -eq $script:Tags) {
    try { $script:Tags = @(Invoke-RestMethod -Uri $Api | ForEach-Object { $_.tag_name }) } catch { Die "公開の一覧を取得できない（$Api）" }
  }
  $script:Tags
}

# Skill の名前と版から、配布物の置き場所を決める。SKILLS_BASE を渡したときは、1つの置き場所から全部を取る（試験のため）
function Get-Base($name, $version) {
  if ($env:SKILLS_BASE) { return $env:SKILLS_BASE }
  if ($version) { return "$Download/$name-v$version" }
  $tag = Get-Tags | Where-Object { $_ -match "^$([regex]::Escape($name))-v[0-9]" } | Select-Object -First 1
  if (-not $tag) { $tag = Get-Tags | Where-Object { $_ -match '^v[0-9]' } | Select-Object -First 1 }
  if (-not $tag) { Die "$name を公開した版が無い" }
  "$Download/$tag"
}

function Get-Sums($base, $path) {
  try { Invoke-WebRequest -UseBasicParsing -Uri "$base/SHA256SUMS" -OutFile $path } catch { Die "照合の一覧を取得できない（$base/SHA256SUMS）" }
  $sums = @{}
  foreach ($line in Get-Content $path) {
    if ($line -match '^([0-9a-f]{64})\s+\*?(.+)$') { $sums[$Matches[2]] = $Matches[1] }
  }
  $sums
}

# git が無い環境もある ── 在るときだけ、リポジトリの根を導入先にする
$Root = $null
if (Get-Command git -ErrorAction SilentlyContinue) { $Root = (git rev-parse --show-toplevel 2>$null) }
if (-not $Root) { $Root = (Get-Location).Path }
$Dest = Join-Path $Root '.claude\skills'
$Work = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $Work | Out-Null

try {
  $Specs = @($env:SKILLS -split '\s+' | Where-Object { $_ })
  $Bulk = $null
  if ($Specs.Count -eq 0) {
    # 名前を省略したら、まとめて公開した最新の版（v<版>）の、この環境向けと環境に依存しない配布物の全部
    $Bulk = Get-Base '' ''
    $all = Get-Sums $Bulk (Join-Path $Work 'all.sums')
    $Specs = @($all.Keys | ForEach-Object {
      if ($_ -match "^(.+)-$([regex]::Escape($Target))\.zip$") { $Matches[1] }
      elseif ($_ -match '^(.+)-any\.zip$') { $Matches[1] }
    } | Sort-Object -Unique)
  }

  New-Item -ItemType Directory -Force -Path $Dest | Out-Null
  foreach ($spec in $Specs) {
    $name, $version = $spec -split '@', 2
    $base = if ($Bulk) { $Bulk } else { Get-Base $name $version }
    $sums = Get-Sums $base (Join-Path $Work "$name.sums")
    $file = @("$name-$Target.zip", "$name-any.zip") | Where-Object { $sums.ContainsKey($_) } | Select-Object -First 1
    if (-not $file) { Die "$name の、この環境（$Target）向けの配布物が無い（$base）" }
    $zip = Join-Path $Work $file
    try { Invoke-WebRequest -UseBasicParsing -Uri "$base/$file" -OutFile $zip } catch { Die "$file を取得できない" }
    if ((Get-FileHash -Algorithm SHA256 $zip).Hash.ToLower() -ne $sums[$file]) { Die "$file の SHA-256 が一覧と一致しない" }
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
    Write-Output "導入した: $name（$(Split-Path $base -Leaf) の $file）"
  }
} finally {
  Remove-Item -Recurse -Force $Work -ErrorAction SilentlyContinue
}
