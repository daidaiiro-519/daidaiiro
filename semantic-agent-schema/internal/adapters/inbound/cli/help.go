package cli

import (
	"encoding/json"
	"slices"
	"strings"
	"unicode"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/publishedlanguage"
)

// commandOrder は、help で並べるコマンドの順。arguments.schema.json の $defs の名前（semantic- は意味的な操作）。
var commandOrder = []string{
	"semantic-create", "create", "semantic-update", "update", "semantic-validate", "validate",
	"semantic-search", "search", "render", "log",
}

type helpArgument struct {
	Name        string `json:"name"`
	Description string `json:"description"`
	Required    bool   `json:"required"`
}

type helpCommand struct {
	Command     string         `json:"command"`
	Description string         `json:"description"`
	Arguments   []helpArgument `json:"arguments"`
}

type helpEvent struct {
	Event    string        `json:"event"`
	Commands []helpCommand `json:"commands"`
}

// wantsHelp は、引数に --help か -h があれば true を返す。
func wantsHelp(args []string) bool {
	return slices.Contains(args, "--help") || slices.Contains(args, "-h")
}

// help は、コマンドと引数の一覧を help のログイベントで書く。説明は arguments.schema.json の description から作り、
// 契約と説明がずれないようにする。コマンドを指していれば、そのコマンドだけを書く（知らないコマンドならすべて）。
func (o *output) help(args []string) error {
	content, err := publishedlanguage.Read("arguments.schema.json")
	if err != nil {
		return err
	}
	var schema struct {
		Defs map[string]struct {
			Description string                     `json:"description"`
			Required    []string                   `json:"required"`
			Properties  map[string]json.RawMessage `json:"properties"`
		} `json:"$defs"`
	}
	if err := json.Unmarshal(content, &schema); err != nil {
		return err
	}
	selected := selectedCommand(args)
	if _, ok := schema.Defs[selected]; !ok {
		selected = ""
	}
	event := helpEvent{Event: "help", Commands: []helpCommand{}}
	for _, key := range commandOrder {
		def, ok := schema.Defs[key]
		if !ok || (selected != "" && key != selected) {
			continue
		}
		command := helpCommand{Command: strings.Replace(key, "semantic-", "semantic ", 1), Description: def.Description, Arguments: []helpArgument{}}
		names := make([]string, 0, len(def.Properties))
		for name := range def.Properties {
			names = append(names, name)
		}
		slices.Sort(names)
		slices.SortStableFunc(names, func(a, b string) int { return boolInt(b == "paths") - boolInt(a == "paths") })
		for _, name := range names {
			if name == "command" || name == "semantic" {
				continue
			}
			var property struct {
				Description string `json:"description"`
				MaxItems    int    `json:"maxItems"`
			}
			if err := json.Unmarshal(def.Properties[name], &property); err != nil {
				return err
			}
			flag := flagOf(name)
			if name == "paths" && property.MaxItems == 1 {
				flag = "<path>"
			}
			command.Arguments = append(command.Arguments, helpArgument{
				Name: flag, Description: property.Description, Required: slices.Contains(def.Required, name),
			})
		}
		event.Commands = append(event.Commands, command)
	}
	return o.encoder.Encode(event)
}

func boolInt(b bool) int {
	if b {
		return 1
	}
	return 0
}

// selectedCommand は、--help の前に書いたコマンドを、arguments.schema.json の $defs の名前で返す。無ければ空。
func selectedCommand(args []string) string {
	var words []string
	for _, a := range args {
		if strings.HasPrefix(a, "-") {
			break
		}
		words = append(words, a)
	}
	switch {
	case len(words) >= 2 && words[0] == "semantic":
		return "semantic-" + words[1]
	case len(words) >= 1:
		return words[0]
	}
	return ""
}

// flagOf は、引数の JSON の名前を、コマンド行の書き方にする（paths は <path>...、designSystem は --design-system）。
func flagOf(name string) string {
	if name == "paths" {
		return "<path>..."
	}
	var b strings.Builder
	b.WriteString("--")
	for _, r := range name {
		if unicode.IsUpper(r) {
			b.WriteByte('-')
			r = unicode.ToLower(r)
		}
		b.WriteRune(r)
	}
	return b.String()
}
