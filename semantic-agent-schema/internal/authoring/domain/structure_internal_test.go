package domain

import "testing"

func TestConcreteStructureReturnsCopy(t *testing.T) {
	t.Parallel()
	c, err := NewConcrete(nil, []byte(`{"$id": "urn:example:a:v1", "properties": {"x": {"title": "t"}}}`))
	if err != nil {
		t.Fatalf("NewConcrete = %v", err)
	}
	c.structure()["$id"] = "changed"
	c.structure()["properties"].(map[string]any)["x"] = nil
	root := c.structure()
	if root["$id"] != "urn:example:a:v1" || root["properties"].(map[string]any)["x"] == nil {
		t.Errorf("structure() = %v, 取り出した値を書き換えた後に変わった", root)
	}
}
