package domain

import (
	"crypto/sha256"
	"encoding/hex"
	"regexp"
)

// Hash は、値オブジェクト ハッシュ。ファイルの内容のバイト列から決まる値で、形は sha256:<16進数 64 桁>。
type Hash interface {
	String() string
	hash()
}

type contentHash struct{ value string }

func (h contentHash) String() string { return h.value }
func (contentHash) hash()            {}

var hashForm = regexp.MustCompile(`^sha256:[0-9a-f]{64}$`)

// HashOf は、内容のバイト列のハッシュを返す。
func HashOf(content []byte) Hash {
	sum := sha256.Sum256(content)
	return contentHash{"sha256:" + hex.EncodeToString(sum[:])}
}

// ParseHash は、操作の記録に残ったハッシュの文字列から Hash を返す。形が違えば error を返す。
func ParseHash(value string) (Hash, error) {
	if !hashForm.MatchString(value) {
		return nil, Fail(LogUnavailable, "", value+" はハッシュの形でない")
	}
	return contentHash{value}, nil
}
