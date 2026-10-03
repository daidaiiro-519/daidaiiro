// SPDX-License-Identifier: MIT

package businesslogic

import (
	"strings"
	"testing"
)

func TestSHA256MatchesTheKnownValue(t *testing.T) {
	if got := SHA256Hex([]byte("abc")); got != "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad" {
		t.Fatal(got)
	}
}

func TestTheBodyIsSplitIntoBlocks(t *testing.T) {
	src := "一文目の\n続き。\n\n- 甲\n- 乙\n\n| 列A | 列B |\n|---|---|\n| **1** | 2 |\n\n---\n\n```\nfn x() {}\n```"
	got := Compact(BlocksOf(strings.Split(src, "\n")))
	want := `[{"kind":"para","text":"一文目の続き。"},{"kind":"list","ordered":false,"items":["甲","乙"]},{"kind":"table","head":["列A","列B"],"rows":[["1","2"]]},{"kind":"code","text":"fn x() {}"}]`
	if got != want {
		t.Fatal(got)
	}
}

func TestFrontmatterIsNotImported(t *testing.T) {
	doc := ImportMarkdown("d", "x.md", "", "---\nname: x\n---\n# 題\n\n本文\n## 節\n- a\n")
	got := Compact(doc)
	if strings.Contains(got, "name: x") || !strings.Contains(got, `"title":"題"`) || !strings.Contains(got, `"title":"節"`) {
		t.Fatal(got)
	}
}

func TestFragmentsDropTheColumnTitle(t *testing.T) {
	got := strings.Join(fragments("原則：「全数テストは不可能」 … ごく単純な ・ x"), "|")
	if got != "「全数テストは不可能」|ごく単純な" {
		t.Fatal(got)
	}
}
