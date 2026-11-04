// Package repository は、アダプター。git rev-parse で、working tree の top-level directory と Git の共通ディレクトリを引く。
// 入口が一度だけ呼び、ほかのアダプターへ渡す。
package repository

import (
	"context"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

// Find は、dir を含む working tree の top-level directory と、Git の共通ディレクトリを返す。
// Git のリポジトリの外なら log_unavailable の Failure を返す。
func Find(ctx context.Context, dir string) (top, common string, err error) {
	out, err := exec.CommandContext(ctx, "git", "-C", dir, "rev-parse", "--show-toplevel", "--git-common-dir").Output()
	if err != nil {
		return "", "", domain.Fail(domain.LogUnavailable, "", "git rev-parse: "+dir+": "+err.Error())
	}
	lines := strings.Split(strings.TrimSpace(string(out)), "\n")
	if len(lines) != 2 {
		return "", "", domain.Fail(domain.LogUnavailable, "", "git rev-parse: "+string(out))
	}
	top, common = lines[0], lines[1]
	if !filepath.IsAbs(common) {
		common = filepath.Join(dir, common)
	}
	return top, common, nil
}
