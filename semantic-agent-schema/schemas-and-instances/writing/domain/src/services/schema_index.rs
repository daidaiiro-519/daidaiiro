//! 渡されたスキーマの置き場所（--schemas のファイル）から、$id で Concrete と $ref の先を引く決まりと、$ref の先の所を引く決まり。
//!
//! ファイルを読むのは手続き（use_cases）がポートで行い、ここは読んだ文字列と JSON だけを扱う。
//! 索引は $id と経路だけを持ち（中身は持たない）。引くときは Lookup が次に読む経路を返し、手続きが読んだ中身を Lookup に渡す。
//! 同じ $id のファイルが2つ以上あれば、どれを使うかを決めずに、全部の経路を添えて断る。

use crate::aggregates::ConcreteSchema;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// 置き場所の $id と、その $id を持つファイルの経路（足した順）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchemaIndex {
    ids: BTreeMap<String, Vec<String>>,
}

/// 引いた Concrete と、形の検査に渡す文書（Concrete と $ref の先。$id と中身）。
#[derive(Debug, Clone, PartialEq)]
pub struct Resolved {
    path: String,
    concrete: ConcreteSchema,
    more: Vec<(String, Value)>,
}

impl Resolved {
    /// 引いた Concrete（経路と文書）と、形の検査で引ける文書（Concrete と $ref の先）から作る。
    #[must_use]
    pub const fn new(path: String, concrete: ConcreteSchema, more: Vec<(String, Value)>) -> Self {
        Self {
            path,
            concrete,
            more,
        }
    }

    /// Concrete のファイルの経路。
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Concrete。
    #[must_use]
    pub const fn concrete(&self) -> &ConcreteSchema {
        &self.concrete
    }

    /// 形の検査で引ける文書（Concrete と $ref の先）。
    #[must_use]
    pub fn more(&self) -> &[(String, Value)] {
        &self.more
    }
}

/// 引けなかった理由。code は決まった名前で、出力に入る。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unresolved {
    code: &'static str,
    message: String,
    paths: Vec<String>,
}

impl Unresolved {
    /// 決まった名前（unknown_schema ・ duplicate_schema ・ 読んだ中身が JSON でない store）。
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// 理由。
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// 同じ $id を持つファイルの経路（duplicate_schema のときだけ）。
    #[must_use]
    pub fn paths(&self) -> &[String] {
        &self.paths
    }
}

impl SchemaIndex {
    /// 空の索引。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 読んだファイル1つを索引に足す。JSON でない ・ $id の無いファイルは引く先にならない。
    pub fn add(&mut self, path: &str, text: &str) {
        let Ok(value) = serde_json::from_str::<Value>(text) else {
            return;
        };
        if let Some(id) = value.get("$id").and_then(Value::as_str) {
            let paths = self.ids.entry(id.to_string()).or_default();
            if !paths.iter().any(|p| p == path) {
                paths.push(path.to_string());
            }
        }
    }

    /// 置き場所にある $id（名前の順）。
    #[must_use]
    pub fn ids(&self) -> Vec<&str> {
        self.ids.keys().map(String::as_str).collect()
    }

    /// $id が id のファイルの経路。from は $ref で指した文書の $id（理由に入れる）。
    ///
    /// # Errors
    /// 置き場所に無い（unknown_schema）・ 2つ以上ある（duplicate_schema）とき。
    pub fn path_of(&self, id: &str, from: Option<&str>) -> Result<&str, Unresolved> {
        let by = from.map_or_else(String::new, |f| format!("{f} が $ref で指す "));
        match self.ids.get(id).map(Vec::as_slice).unwrap_or_default() {
            [] => Err(Unresolved {
                code: "unknown_schema",
                message: format!(
                    "{by}{id} は、渡されたスキーマの置き場所（--schemas）に無い。取りに行かない"
                ),
                paths: Vec::new(),
            }),
            [one] => Ok(one),
            many => Err(Unresolved {
                code: "duplicate_schema",
                message: format!(
                    "{by}{id} を持つファイルが、渡されたスキーマの置き場所に {} つある（{}）。どれを使うかを決めない",
                    many.len(),
                    many.join(" ・ ")
                ),
                paths: many.to_vec(),
            }),
        }
    }
}

/// 置き場所から引く手順。next が次に読むファイルの経路を返し、呼ぶ側がポートで読んで read に中身を渡す。
/// next が None を返したら引き終わり。
#[derive(Debug, Clone)]
pub struct Lookup<'i> {
    index: &'i SchemaIndex,
    /// まだ読んでいない、引く Concrete の $id
    first: Option<String>,
    /// $ref の先も引くか
    follow: bool,
    /// 引いた Concrete（経路と文書）
    concrete: Option<(String, ConcreteSchema)>,
    /// 引いた文書（$id と中身。引く Concrete か渡した文書が先頭）
    more: Vec<(String, Value)>,
    /// まだ引いていない $ref の先と、それを指した文書の $id
    queue: Vec<(String, String)>,
    seen: BTreeSet<String>,
    /// next が返した、いま読んでいる文書の $id と経路
    reading: Option<(String, String)>,
}

impl SchemaIndex {
    /// $id が id の Concrete を引く手順。follow なら、その $ref の先も同じ置き場所から引く（入れて出すスキーマは引かない）。
    #[must_use]
    pub fn concrete(&self, id: &str, follow: bool) -> Lookup<'_> {
        Lookup {
            index: self,
            first: Some(id.to_string()),
            follow,
            concrete: None,
            more: Vec::new(),
            queue: Vec::new(),
            seen: BTreeSet::from([id.to_string()]),
            reading: None,
        }
    }

    /// 渡した文書（$id と中身）の $ref の先を、同じ置き場所から引く手順（入れて出すスキーマは引かない）。
    #[must_use]
    pub fn refs_of(&self, id: &str, value: Value) -> Lookup<'_> {
        Lookup {
            index: self,
            first: None,
            follow: true,
            concrete: None,
            queue: refs(&value)
                .into_iter()
                .map(|r| (r, id.to_string()))
                .collect(),
            more: vec![(id.to_string(), value)],
            seen: BTreeSet::from([id.to_string()]),
            reading: None,
        }
    }

    /// node が $ref で指していれば、その先と、その先がある文書。指していない ・ 引けなければ (root, node) のまま。
    /// 文書の中（#/… の JSON Pointer か #名前 の $anchor）と、ほかの文書（<$id>#…、others）を引く。
    #[must_use]
    pub fn deref<'a>(
        root: &'a Value,
        node: &'a Value,
        others: &'a [(String, Value)],
    ) -> (&'a Value, &'a Value) {
        let Some(r) = node.get("$ref").and_then(Value::as_str) else {
            return (root, node);
        };
        let (base, fragment) = r.split_once('#').unwrap_or((r, ""));
        let doc = if base.is_empty() {
            Some(root)
        } else {
            others.iter().find(|(id, _)| id == base).map(|(_, d)| d)
        };
        match doc.and_then(|d| Self::fragment(d, fragment).map(|t| (d, t))) {
            Some(found) => found,
            None => (root, node),
        }
    }

    /// 文書の中の所：空なら文書そのもの、/ で始まれば JSON Pointer、ほかは $defs の中の $anchor。
    #[must_use]
    pub fn fragment<'a>(doc: &'a Value, fragment: &str) -> Option<&'a Value> {
        if fragment.is_empty() {
            return Some(doc);
        }
        if fragment.starts_with('/') {
            return doc.pointer(fragment);
        }
        doc.get("$defs")?
            .as_object()?
            .values()
            .find(|d| d.get("$anchor").and_then(Value::as_str) == Some(fragment))
    }
}

impl Lookup<'_> {
    /// 次に読むファイルの経路。引き終われば None。shipped は入れて出すスキーマの $id か（その先は引かない）。
    ///
    /// # Errors
    /// 置き場所に無い（unknown_schema）・ 2つ以上ある（duplicate_schema）とき。
    pub fn next(&mut self, shipped: &dyn Fn(&str) -> bool) -> Result<Option<String>, Unresolved> {
        if let Some(id) = &self.first {
            let path = self.index.path_of(id, None)?.to_string();
            self.reading = Some((id.clone(), path.clone()));
            return Ok(Some(path));
        }
        while let Some((next, from)) = self.queue.pop() {
            if shipped(&next) || !self.seen.insert(next.clone()) {
                continue;
            }
            let path = self.index.path_of(&next, Some(&from))?.to_string();
            self.reading = Some((next, path.clone()));
            return Ok(Some(path));
        }
        Ok(None)
    }

    /// next が返した経路のファイルの中身を受け取る。
    ///
    /// # Errors
    /// JSON でない（store）・ 引く Concrete が Concrete でない（unknown_schema）とき。
    pub fn read(&mut self, text: &str) -> Result<(), Unresolved> {
        let Some((id, path)) = self.reading.take() else {
            return Ok(());
        };
        let value: Value = serde_json::from_str(text).map_err(|e| Unresolved {
            code: "store",
            message: format!("{path} が JSON として読めない：{e}"),
            paths: Vec::new(),
        })?;
        if self.first.take().is_some() {
            self.concrete = Some((path.clone(), concrete_of(&id, &path, &value)?));
        }
        if self.follow {
            self.queue
                .extend(refs(&value).into_iter().map(|r| (r, id.clone())));
        }
        self.more.push((id, value));
        Ok(())
    }

    /// 引いた Concrete と、形の検査に渡す文書（Concrete と $ref の先）。Concrete を引く手順でなければ None。
    #[must_use]
    pub fn resolved(self) -> Option<Resolved> {
        let (path, concrete) = self.concrete?;
        Some(Resolved::new(path, concrete, self.more))
    }

    /// 引いた文書（渡した文書と $ref の先。$id と中身）。
    #[must_use]
    pub fn documents(self) -> Vec<(String, Value)> {
        self.more
    }
}

/// 引いた文書を Abstract の版を申告する Concrete として読む。
///
/// # Errors
/// Concrete でないとき（unknown_schema）。
fn concrete_of(id: &str, path: &str, value: &Value) -> Result<ConcreteSchema, Unresolved> {
    ConcreteSchema::read(value).ok_or_else(|| Unresolved {
        code: "unknown_schema",
        message: format!("{id}（{path}）は Abstract の版を申告する Concrete でない"),
        paths: Vec::new(),
    })
}

/// 文書の中の $ref が指す、ほかの文書の名前（# より前。文書の中を指すものは除く。名前の順）。
fn refs(value: &Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    collect(value, &mut out);
    out
}

fn collect(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::Object(m) => {
            for (k, v) in m {
                if k == "$ref"
                    && let Some(r) = v.as_str()
                {
                    let base = r.split_once('#').map_or(r, |(b, _)| b);
                    if !base.is_empty() {
                        out.insert(base.to_string());
                    }
                } else {
                    collect(v, out);
                }
            }
        }
        Value::Array(a) => {
            for v in a {
                collect(v, out);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const ABSTRACT: &str = "urn:semantic-agent-schema:published-language:abstract:v1";

    #[test]
    fn keeps_only_ids_and_paths() {
        let mut index = SchemaIndex::new();
        index.add(
            "s/a.json",
            &json!({"$schema": ABSTRACT, "$id": "urn:a"}).to_string(),
        );
        index.add("s/note.json", &json!({"id": "N1"}).to_string());
        index.add("s/bad.json", "{");
        assert_eq!(index.ids(), ["urn:a"]);
        assert_eq!(index.path_of("urn:a", None).unwrap(), "s/a.json");
    }

    #[test]
    fn refuses_a_duplicate_id_with_every_path() {
        let mut index = SchemaIndex::new();
        let doc = json!({"$schema": ABSTRACT, "$id": "urn:a"}).to_string();
        index.add("a/x.json", &doc);
        index.add("b/x.json", &doc);
        let e = index.path_of("urn:a", None).unwrap_err();
        assert_eq!(e.code(), "duplicate_schema");
        assert_eq!(e.paths(), ["a/x.json", "b/x.json"]);
    }

    #[test]
    fn a_missing_ref_names_who_pointed_at_it() {
        let e = SchemaIndex::new()
            .path_of("urn:gone", Some("urn:a"))
            .unwrap_err();
        assert_eq!(e.code(), "unknown_schema");
        assert!(e.message().contains("urn:a が $ref で指す urn:gone"));
    }

    #[test]
    fn refs_are_other_documents_only() {
        let doc = json!({"properties": {"b": {"$ref": "urn:b#/$defs/x"}, "c": {"$ref": "#/$defs/c"}},
                         "items": [{"$ref": "urn:d"}]});
        assert_eq!(
            refs(&doc).into_iter().collect::<Vec<_>>(),
            ["urn:b", "urn:d"]
        );
    }

    #[test]
    fn only_documents_declaring_an_abstract_are_concretes() {
        assert!(concrete_of("urn:a", "a.json", &json!({"$schema": ABSTRACT})).is_ok());
        assert_eq!(
            concrete_of("urn:b", "b.json", &json!({"$id": "urn:b"}))
                .unwrap_err()
                .code(),
            "unknown_schema"
        );
    }
}
