# Windows に、WSL と同じ基盤の開発環境を入れる
# 実行：PowerShell で  powershell -ExecutionPolicy Bypass -File dev-env\windows\setup.ps1
# 管理者権限が要るもの（Build Tools ・ Docker Desktop）は、winget がそのつど UAC で確認を出す。
# 何度実行しても、入っているものは入れ直さない。

$ErrorActionPreference = 'Stop'

function Install-Winget($id, $extra = @()) {
    Write-Host "=== $id"
    winget install --id $id --exact --silent --accept-source-agreements --accept-package-agreements @extra
}

function Update-SessionPath {
    $env:Path = [Environment]::GetEnvironmentVariable('Path', 'Machine') + ';' +
                [Environment]::GetEnvironmentVariable('Path', 'User')
}

function Test-Command($name) { [bool](Get-Command $name -ErrorAction SilentlyContinue) }

# 1. C++ のビルドの道具（Rust のリンカと、node-gyp のビルドに使う）
#    rustup の公式文書が推奨する「Desktop development with C++」= Microsoft.VisualStudio.Workload.VCTools
Install-Winget 'Microsoft.VisualStudio.2022.BuildTools' @(
    '--override', '--wait --quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended')

# 2. 道具（各開発元の配布物）
foreach ($id in 'Git.Git', 'GitHub.cli', 'Amazon.AWSCLI', 'Docker.DockerDesktop',
                'BurntSushi.ripgrep.MSVC', 'jqlang.jq', 'jdx.mise', 'Rustlang.Rustup', 'Microsoft.PowerShell') {
    Install-Winget $id
}
Update-SessionPath

# 3. Rust（公式の rustup で入れ、手順 5 で mise の一覧にも載せる）
rustup toolchain install stable
rustup component add clippy rustfmt rust-analyzer
rustup default stable
if (-not (Test-Command cargo-audit)) { cargo install cargo-audit --locked }

# 4. 実行ポリシーを自分のユーザーだけ RemoteSigned にし、mise を PowerShell 7 で有効にする
#    （5.1 では mise のフォルダ移動時の切り替えが動かず、開くたびに警告が出るので 7 にだけ書く）
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned -Force
pwsh -NoProfile -Command {
    $activation = '(&mise activate pwsh) | Out-String | Invoke-Expression'
    if (-not (Test-Path $PROFILE)) { New-Item -ItemType File -Path $PROFILE -Force | Out-Null }
    if (-not (Select-String -Path $PROFILE -SimpleMatch $activation -Quiet)) { Add-Content $PROFILE $activation }
}

# mise のシムをユーザーの Path に置く（VS Code など、プロファイルを読まないアプリからも見えるように）
$shims = "$env:LOCALAPPDATA\mise\shims"
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (($userPath -split ';') -notcontains $shims) {
    [Environment]::SetEnvironmentVariable('Path', "$shims;$userPath", 'User')
}
Update-SessionPath

# 5. 言語（mise）。Java は Corretto 25（LTS）の最新
mise use -g node@24 go@latest java@corretto-25 ruby@latest dotnet@latest uv@latest rust@stable

# 6. Python（uv）と Go の検査の道具
mise exec -- uv python install 3.14 --default
mise exec -- uv python update-shell
mise exec -- go install honnef.co/go/tools/cmd/staticcheck@latest
mise exec -- go install github.com/kisielk/errcheck@latest
mise exec -- go install golang.org/x/vuln/cmd/govulncheck@latest
mise reshim   # mise は GOBIN を mise 側の go に向けるので、go install したものはシムを作り直して通す

# 7. AI の CLI（各社の公式の Windows 用インストーラー）
if (-not (Test-Command claude))   { irm https://claude.ai/install.ps1 | iex }
if (-not (Test-Command codex))    { irm https://chatgpt.com/codex/install.ps1 | iex }
if (-not (Test-Command kiro-cli)) { irm 'https://cli.kiro.dev/install.ps1' | iex }

# 8. AWS：WSL に config があれば写す（SSO の設定だけで、鍵は入っていない）
$awsDir = "$env:USERPROFILE\.aws"
if (-not (Test-Path "$awsDir\config")) {
    $wslHome = (wsl.exe -e sh -c 'printf %s "$HOME"') -replace '/', '\'
    $wslConfig = "\\wsl.localhost\Ubuntu$wslHome\.aws\config"
    if (Test-Path $wslConfig) {
        New-Item -ItemType Directory -Path $awsDir -Force | Out-Null
        Copy-Item $wslConfig "$awsDir\config"
    }
}

Write-Host ''
Write-Host '残りは対話が要るので、新しい PowerShell 7 で順に実行する：'
Write-Host '  gh auth login'
Write-Host '  gh auth setup-git'
Write-Host '  aws sso login --profile dev'
Write-Host '  claude ・ codex ・ kiro-cli をそれぞれ起動してログインする'
Write-Host '  Windows Terminal の設定 → スタートアップ → 既定のプロファイルを PowerShell 7 にする'
Write-Host '  Docker Desktop を一度起動し、Settings → Resources → WSL integration で Ubuntu をオフにする'
