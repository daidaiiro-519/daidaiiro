package core

import "example/server/internal/adapter"

func Place( id string ) error {
	return adapter.Save(id)
}
