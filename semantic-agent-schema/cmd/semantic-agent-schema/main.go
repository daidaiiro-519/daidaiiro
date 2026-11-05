// Command semantic-agent-schema は、AI が書く JSON の品質を、機械的な検証と意味的な検証で担保する。
// 入口として、working tree と操作の記録のファイルを決め、アダプターを作って cli へ渡す。
package main

import (
	"context"
	"os"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/inbound/cli"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/files"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/patching"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/recordstore"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/repository"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/schemacheck"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/templates"
	authoring "github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/application"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/publishedlanguage"
	record "github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/record/application"
	rendering "github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/rendering/application"
	search "github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/search/application"
)

func main() {
	os.Exit(cli.Run(context.Background(), os.Args[1:], cli.IO{Stdin: os.Stdin, Stdout: os.Stdout, Stderr: os.Stderr}, cli.Wiring{
		Arguments:  schemacheck.New(nil),
		Open:       open,
		OpenLog:    openLog,
		OpenSearch: openSearch,
		OpenRender: openRender,
	}))
}

// open は、working tree と操作の記録を開き、ユースケース「作成」「更新」「検証」「意味的な検証」に、アダプターを結び付けて返す。
func open(ctx context.Context, schemas []domain.Path) (cli.Authoring, func() error, error) {
	dir, err := os.Getwd()
	if err != nil {
		return cli.Authoring{}, nil, domain.Fail(domain.NotInGit, "", err.Error())
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
		Create: authoring.Create{
			Documents: documents, Schemas: concretes, Validator: validator, Records: records,
			AbstractID: publishedlanguage.AbstractID,
		},
		Update: authoring.Update{
			Documents: documents, Schemas: concretes, Validator: validator, Patcher: patching.New(), Records: records,
			AbstractID: publishedlanguage.AbstractID, PatchID: publishedlanguage.PatchID,
		},
		Judge: authoring.Judge{
			Documents: documents, Schemas: concretes, Validator: validator, Records: records,
			AbstractID: publishedlanguage.AbstractID, ResponseID: publishedlanguage.JudgmentResponseID,
		},
		Validate: authoring.Validate{
			Documents: documents, Schemas: concretes, Validator: validator, Records: records,
			AbstractID: publishedlanguage.AbstractID,
		},
	}, records.Close, nil
}

// openLog は、Git の共通ディレクトリの操作の記録を読むだけのために開き、ユースケース「操作の記録の参照」に結び付けて返す。
func openLog(ctx context.Context) (record.Log, func() error, error) {
	dir, err := os.Getwd()
	if err != nil {
		return record.Log{}, nil, domain.Fail(domain.NotInGit, "", err.Error())
	}
	_, common, err := repository.Find(ctx, dir)
	if err != nil {
		return record.Log{}, nil, err
	}
	records, err := recordstore.OpenExisting(ctx, common)
	if err != nil {
		return record.Log{}, nil, err
	}
	return record.Log{Records: records}, records.Close, nil
}

// openSearch は、working tree を引き、ユースケース「検索」に、範囲の path を読むアダプターを結び付けて返す。
// 検索は操作の記録を使わない。
func openSearch(ctx context.Context) (search.Search, error) {
	dir, err := os.Getwd()
	if err != nil {
		return search.Search{}, domain.Fail(domain.NotInGit, "", err.Error())
	}
	top, _, err := repository.Find(ctx, dir)
	if err != nil {
		return search.Search{}, err
	}
	return search.Search{
		Documents: files.NewDocuments(top), Validator: schemacheck.New(nil),
		ResponseID: publishedlanguage.SearchResponseID,
	}, nil
}

// openRender は、working tree を引き、ユースケース「描画」に、読むアダプター ・ JSON Schema の実装 ・ テンプレートの処理系を
// 結び付けて返す。描画は操作の記録を使わない。
func openRender(ctx context.Context, schemas []domain.Path) (rendering.Render, error) {
	dir, err := os.Getwd()
	if err != nil {
		return rendering.Render{}, domain.Fail(domain.NotInGit, "", err.Error())
	}
	top, _, err := repository.Find(ctx, dir)
	if err != nil {
		return rendering.Render{}, err
	}
	abstract, err := publishedlanguage.Read("abstract.schema.json")
	if err != nil {
		return rendering.Render{}, err
	}
	concretes := files.NewSchemas(top, schemas, map[string][]byte{publishedlanguage.AbstractID: abstract})
	return rendering.Render{
		Documents: files.NewDocuments(top), Schemas: concretes, Validator: schemacheck.New(concretes.Content),
		Renderer: templates.New(top), AbstractID: publishedlanguage.AbstractID, DesignSystemID: publishedlanguage.DesignSystemID,
		DesignTokensID: publishedlanguage.DesignTokensID,
	}, nil
}
