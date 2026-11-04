// Command semantic-agent-schema は、AI が書く JSON の品質を、機械的な検証と意味的な検証で担保する。
// 入口として、working tree と操作の記録のファイルを決め、アダプターを作って cli へ渡す。
package main

import (
	"context"
	"os"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/inbound/cli"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/files"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/recordstore"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/repository"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/schemacheck"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/application"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/publishedlanguage"
)

func main() {
	os.Exit(cli.Run(context.Background(), os.Args[1:], cli.IO{Stdin: os.Stdin, Stdout: os.Stdout, Stderr: os.Stderr}, cli.Wiring{
		Arguments: schemacheck.New(nil),
		Open:      open,
	}))
}

// open は、working tree と操作の記録を開き、ユースケース「作成」と「検証」に、アダプターを結び付けて返す。
func open(ctx context.Context, schemas []domain.Path) (cli.Authoring, func() error, error) {
	dir, err := os.Getwd()
	if err != nil {
		return cli.Authoring{}, nil, domain.Fail(domain.LogUnavailable, "", err.Error())
	}
	top, common, err := repository.Find(ctx, dir)
	if err != nil {
		return cli.Authoring{}, nil, err
	}
	abstract, err := publishedlanguage.Read("abstract.schema.json")
	if err != nil {
		return cli.Authoring{}, nil, err
	}
	records, err := recordstore.Open(ctx, common)
	if err != nil {
		return cli.Authoring{}, nil, err
	}
	concretes := files.NewSchemas(top, schemas, map[string][]byte{publishedlanguage.AbstractID: abstract})
	documents := files.NewDocuments(top)
	validator := schemacheck.New(concretes.Content)
	return cli.Authoring{
		Create: application.Create{
			Documents: documents, Schemas: concretes, Validator: validator, Records: records,
			AbstractID: publishedlanguage.AbstractID,
		},
		Validate: application.Validate{
			Documents: documents, Schemas: concretes, Validator: validator, Records: records,
			AbstractID: publishedlanguage.AbstractID,
		},
	}, records.Close, nil
}
