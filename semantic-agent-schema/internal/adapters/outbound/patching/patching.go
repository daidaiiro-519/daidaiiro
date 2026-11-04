// Package patching は、アダプター。ポート Patcher を json-patch/v5 で満たし、JSON Patch を JSON ドキュメントに適用する。
// domain を知らないので、適用できないときの error は、呼ぶ側の application が違反にする。
package patching

import (
	"fmt"

	"github.com/evanphx/json-patch/v5"
)

// Patcher は、JSON Patch を適用する。
type Patcher struct{}

// New は、Patcher を返す。
func New() *Patcher { return &Patcher{} }

// Apply は、document に patch を適用した JSON ドキュメントのバイト列を返す。
// 適用できなければ（無い instance location の remove ・ test の不一致など）、ライブラリの error を返す。
func (p *Patcher) Apply(document, patch []byte) ([]byte, error) {
	decoded, err := jsonpatch.DecodePatch(patch)
	if err != nil {
		return nil, fmt.Errorf("decode JSON Patch: %w", err)
	}
	applied, err := decoded.Apply(document)
	if err != nil {
		return nil, fmt.Errorf("apply JSON Patch: %w", err)
	}
	return applied, nil
}
