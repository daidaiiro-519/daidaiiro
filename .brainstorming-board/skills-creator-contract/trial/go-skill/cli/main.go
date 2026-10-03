// SPDX-License-Identifier: MIT

// trial-go のプレゼンテーション層（CLI）。シェルから呼ぶ唯一の経路である。
//
//	trial-go <動詞> [対象…] [--json]
//
// サービス層だけを参照する。
package main

import (
	"fmt"
	"os"
	"strings"

	"trialgo/service"
)

// readArgs は旗と位置引数を読み、渡された引数を組む。旗は --名前=値 と --名前 値 の両方を受け、
// 値を伴わない旗は立っている（"1"）。次が別の旗なら、それは値ではない。
func readArgs(tool service.Tool, rest []string) (*service.Given, error) {
	given := &service.Given{}
	var positional []string
	for i := 0; i < len(rest); i++ {
		a := rest[i]
		body, isFlag := strings.CutPrefix(a, "--")
		if !isFlag {
			positional = append(positional, a)
			continue
		}
		var key, value string
		if k, v, ok := strings.Cut(body, "="); ok {
			key, value = k, v
		} else if i+1 < len(rest) && !strings.HasPrefix(rest[i+1], "--") {
			key, value = body, rest[i+1]
			i++
		} else {
			key, value = body, "1"
		}
		key = strings.ReplaceAll(key, "-", "_")
		// 道具の一覧に無い旗は断る
		known := make([]string, 0, len(tool.Args))
		hit := false
		for _, x := range tool.Args {
			known = append(known, x.Name)
			hit = hit || x.Name == key
		}
		if !hit {
			return nil, fmt.Errorf("その旗は無い: --%s（%s は %s を受ける）", key, tool.Name, strings.Join(known, " ・ "))
		}
		given.Push(key, value)
	}
	// 位置と旗を混ぜて渡せる
	for _, arg := range tool.Args {
		if given.Has(arg.Name) {
			continue
		}
		if arg.Many {
			for _, v := range positional {
				given.Push(arg.Name, v)
			}
			positional = nil
			continue
		}
		if len(positional) > 0 {
			given.Push(arg.Name, positional[0])
			positional = positional[1:]
		} else if arg.Default != nil {
			given.Push(arg.Name, *arg.Default)
		}
	}
	return given, nil
}

func usage(all []service.Tool) {
	fmt.Println("道具の一覧")
	for _, t := range all {
		need := make([]string, 0, len(t.Args))
		for _, a := range t.Args {
			switch {
			case a.Many:
				need = append(need, "<"+a.Name+"…>")
			case a.Required:
				need = append(need, "<"+a.Name+">")
			default:
				need = append(need, "["+a.Name+"]")
			}
		}
		fmt.Printf("  %s %s\n      %s\n", t.Name, strings.Join(need, " "), t.Summary)
	}
	fmt.Println("\n  どれも --json を付けると、機械が読む形で出る")
}

func emit(out service.Outcome, asJSON bool, human func(service.Outcome) string) int {
	if asJSON {
		fmt.Println(service.Pretty(out.ToJSON()))
	} else {
		fmt.Println(human(out))
	}
	return out.ExitCode()
}

// verbFirst は動詞の前に置いた旗（--名前 値）を、引数の末尾へ移す。
func verbFirst(argv []string) []string {
	var lead []string
	i := 0
	for i < len(argv) && strings.HasPrefix(argv[i], "--") {
		lead = append(lead, argv[i])
		i++
		if i < len(argv) && !strings.HasPrefix(argv[i], "--") {
			lead = append(lead, argv[i])
			i++
		}
	}
	return append(append([]string{}, argv[i:]...), lead...)
}

func run() int {
	all := service.Tools()
	asJSON := false
	var argv []string
	for _, a := range os.Args[1:] {
		if a == "--json" {
			asJSON = true
		} else {
			argv = append(argv, a)
		}
	}
	argv = verbFirst(argv)
	if len(argv) == 0 {
		// 動詞なしで --json を付けたら、道具の一覧を返す
		if asJSON {
			return emit(service.Catalog(all, &service.Given{}), true, func(service.Outcome) string { return "" })
		}
		usage(all)
		return 2
	}
	verb := argv[0]
	if verb == "-h" || verb == "--help" || verb == "help" {
		usage(all)
		return 0
	}
	var tool *service.Tool
	for i := range all {
		if all[i].Name == verb {
			tool = &all[i]
			break
		}
	}
	if tool == nil {
		fmt.Fprintf(os.Stderr, "その動詞は無い: %s ── 一覧は help である\n", verb)
		return 2
	}
	given, err := readArgs(*tool, argv[1:])
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		return 2
	}
	for _, a := range tool.Args {
		if a.Required && !given.Has(a.Name) {
			fmt.Fprintf(os.Stderr, "引数が足りない: %s は %s を要する\n", tool.Name, a.Name)
			return 2
		}
	}
	return emit(tool.Run(given), asJSON, tool.Human)
}

func main() { os.Exit(run()) }
