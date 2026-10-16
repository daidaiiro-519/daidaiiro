#!/usr/bin/env bash
# WSL（Ubuntu 26.04）に、このリポジトリで使う開発環境を入れる
# 実行：bash dev-env/wsl/setup.sh   （途中で sudo のパスワードを聞かれる）
# 何度実行しても、入っているものは入れ直さない。
# コンテナで試すときは SKIP_DOCKER=1 を付けて、Docker と /etc/wsl.conf の手順を飛ばす。

set -euo pipefail

have() { command -v "$1" >/dev/null 2>&1; }
step() { printf '\n=== %s\n' "$1"; }

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# 1. apt（ビルドの道具 ・ 描画と文書の道具 ・ 日本語フォント ・ .NET が使う libicu）
step 'apt'
sudo apt-get update
sudo apt-get install -y \
    build-essential ca-certificates curl wget unzip git jq ripgrep tmux pipx python3-pip \
    graphviz poppler-utils xvfb libicu78 \
    fonts-ipafont-gothic fonts-noto-color-emoji fonts-liberation fonts-wqy-zenhei

# 2. GitHub CLI（GitHub の公式の apt リポジトリ。手順は cli/cli の docs/install_linux.md のまま）
step 'GitHub CLI'
if [ ! -f /etc/apt/sources.list.d/github-cli.list ]; then
    sudo mkdir -p -m 755 /etc/apt/keyrings \
        && out=$(mktemp) && wget -nv -O$out https://cli.github.com/packages/githubcli-archive-keyring.gpg \
        && cat $out | sudo tee /etc/apt/keyrings/githubcli-archive-keyring.gpg > /dev/null \
        && sudo chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg \
        && sudo mkdir -p -m 755 /etc/apt/sources.list.d \
        && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
        && sudo apt update
fi
sudo apt install gh -y

# 3. AWS CLI v2（AWS の公式の配布物）
step 'AWS CLI'
if ! have aws; then
    curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "$tmp/awscliv2.zip"
    unzip -q "$tmp/awscliv2.zip" -d "$tmp"
    sudo "$tmp/aws/install"
fi

# 4. Docker（Docker の公式のスクリプト）と、systemd で動かすための /etc/wsl.conf
if [ "${SKIP_DOCKER:-0}" != 1 ]; then
    step 'Docker'
    if ! grep -q '^systemd=true' /etc/wsl.conf 2>/dev/null; then
        printf '\n[boot]\nsystemd=true\n' | sudo tee -a /etc/wsl.conf > /dev/null
        echo '/etc/wsl.conf に systemd=true を足した。最後に Windows で wsl --shutdown を実行して開き直す'
    fi
    if ! have docker; then
        curl -fsSL https://get.docker.com -o "$tmp/get-docker.sh"
        sudo sh "$tmp/get-docker.sh"
    fi
    sudo usermod -aG docker "$USER"
fi

# 5. mise（言語の版を管理する）と、シェルの設定
step 'mise'
if [ ! -x "$HOME/.local/bin/mise" ]; then
    curl -fsSL https://mise.run | sh
fi
export PATH="$HOME/.local/bin:$PATH"

add_line() { grep -qxF "$2" "$1" 2>/dev/null || printf '%s\n' "$2" >> "$1"; }
# Windows 側の mise の設定（/mnt/c のホーム）を、WSL の mise に読ませない。activate より前に置く
win_home=$(wslpath "$(cmd.exe /c 'echo %USERPROFILE%' 2>/dev/null | tr -d '\r')" 2>/dev/null || true)
if [ -n "$win_home" ]; then
    add_line ~/.bashrc  "export MISE_IGNORED_CONFIG_PATHS=\"$win_home/.config/mise\""
    add_line ~/.profile "export MISE_IGNORED_CONFIG_PATHS=\"$win_home/.config/mise\""
    export MISE_IGNORED_CONFIG_PATHS="$win_home/.config/mise"
fi
add_line ~/.bashrc  'export PATH="$HOME/.local/bin:$PATH"'
add_line ~/.bashrc  'eval "$(~/.local/bin/mise activate bash)"'
add_line ~/.profile 'export PATH="$HOME/.local/share/mise/shims:$HOME/.cargo/bin:$HOME/.local/bin:$PATH"'

# 6. Rust（公式の rustup で入れ、手順 7 で mise の一覧にも載せる）
step 'Rust'
if ! have rustup && [ ! -x "$HOME/.cargo/bin/rustup" ]; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
fi
export PATH="$HOME/.cargo/bin:$PATH"
rustup toolchain install stable
rustup component add clippy rustfmt rust-analyzer
rustup default stable

# 7. 言語（mise）。Java は Corretto 25（LTS）の最新
step '言語（mise）'
mise use -g node@24 go@latest java@corretto-25 ruby@latest dotnet@latest uv@latest rust@stable
export PATH="$HOME/.local/share/mise/shims:$PATH"

# 8. 検査の道具（Go ・ Rust）
step '検査の道具'
mise exec -- go install honnef.co/go/tools/cmd/staticcheck@latest
mise exec -- go install github.com/kisielk/errcheck@latest
mise exec -- go install golang.org/x/vuln/cmd/govulncheck@latest
mise reshim   # mise は GOBIN を mise 側の go に向けるので、go install したものはシムを作り直して通す
have cargo-audit || cargo install cargo-audit --locked

# 9. pipx の道具（AWS CodeCommit へ git で接続する）
step 'pipx'
pipx install git-remote-codecommit || true

# 10. 図をヘッドレスブラウザで描画して確かめるための Chromium（Playwright）
step 'Chromium（Playwright）'
mise exec -- npx -y playwright install --with-deps chromium

# 11. AI の CLI（各社の公式のインストーラー）
step 'AI の CLI'
have claude   || curl -fsSL https://claude.ai/install.sh | bash
have codex    || curl -fsSL https://chatgpt.com/codex/install.sh | sh
have kiro-cli || curl -fsSL https://cli.kiro.dev/install | bash

# 12. AWS：Windows 側に config があれば写す（SSO の設定だけで、鍵は入っていない）
if [ -n "$win_home" ] && [ -f "$win_home/.aws/config" ] && [ ! -f ~/.aws/config ]; then
    mkdir -p ~/.aws && cp "$win_home/.aws/config" ~/.aws/config
fi

cat <<'EOF'

残りは対話が要るので、新しいシェルで順に実行する：
  gh auth login
  gh auth setup-git
  aws sso login --profile dev
  claude ・ codex ・ kiro-cli をそれぞれ起動してログインする
/etc/wsl.conf を書き換えたときは、Windows で wsl --shutdown を実行してから開き直す
EOF
