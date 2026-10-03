// SPDX-License-Identifier: MIT

// Package dataaccess はデータアクセス層である。ファイルの入出力だけを持ち、判定を置かない。
// 同じモジュールのほかの層を参照しない。
package dataaccess

import (
	"os"
	"path/filepath"
	"sort"
)

// ReadToString は、文字列として読む。
func ReadToString(path string) (string, error) {
	b, err := os.ReadFile(path)
	if err != nil {
		return "", err
	}
	return string(b), nil
}

// Write は書く。在れば置き換える。
func Write(path string, body string) error {
	return os.WriteFile(path, []byte(body), 0o644)
}

// List は、フォルダの中身の経路を名前の順に並べて返す。
func List(dir string) ([]string, error) {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil, err
	}
	out := make([]string, 0, len(entries))
	for _, e := range entries {
		out = append(out, filepath.Join(dir, e.Name()))
	}
	sort.Strings(out)
	return out, nil
}

// IsFile は、ファイルとして在るかを返す。
func IsFile(path string) bool {
	st, err := os.Stat(path)
	return err == nil && st.Mode().IsRegular()
}

// Executable は、実行ファイル自身の経路を返す。
func Executable() (string, error) {
	p, err := os.Executable()
	if err != nil {
		return "", err
	}
	return filepath.EvalSymlinks(p)
}
