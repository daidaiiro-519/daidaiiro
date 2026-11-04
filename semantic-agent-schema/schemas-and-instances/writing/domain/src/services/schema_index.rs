//! 渡されたスキーマの置き場所（--schemas のファイル）から、$id で Concrete と $ref の先を引く決まり。
//!
//! 置き場所のファイルを1件ずつ読み、$id と経路だけを残す（中身は持たない）。引くときは、要る文書だけを読み直す。
//! 同じ $id のファイルが2つ以上あれば、どれを使うかを決めずに、全部の経路を添えて断る。
//! $ref の先も同じ置き場所から引く。入れて出すスキーマ（Abstract など）は置き場所から引かない。

use crate::aggregates::ConcreteSchema;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// 置き場所の $id と、その $id を持つファイルの経路（経路の順）。
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
    /// 決まった名前（unknown_schema ・ duplicate_schema ・ store）。
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
    /// 置き場所のファイルを1件ずつ読み、$id と経路だけを残す。JSON でない ・ $id の無いファイルは引く先にならない。
    ///
    /// # Errors
    /// ファイルを読めないとき（store）。
    pub fn read(
        files: &[String],
        read: impl Fn(&str) -> Result<String, String>,
    ) -> Result<Self, Unresolved> {
        let mut ids: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for path in files {
            let text = read(path).map_err(|message| Unresolved {
                code: "store",
                message,
                paths: Vec::new(),
            })?;
            let Ok(value) = serde_json::from_str::<Value>(&text) else {
                continue;
            };
            if let Some(id) = value.get("$id").and_then(Value::as_str) {
                let paths = ids.entry(id.to_string()).or_default();
                if !paths.contains(path) {
                    paths.push(path.clone());
                }
            }
        }
        Ok(Self { ids })
    }

    /// 置き場所にある $id（名前の順）。
    #[must_use]
    pub fn ids(&self) -> Vec<&str> {
        self.ids.keys().map(String::as_str).collect()
    }

    /// $id が id の Concrete と、そのファイルの経路。$ref の先は引かない（描く ・ 探すときに使う）。
    ///
    /// # Errors
    /// 置き場所に無い ・ 2つ以上ある ・ Concrete でない（unknown_schema ・ duplicate_schema）・ 読めない（store）とき。
    pub fn concrete(
        &self,
        id: &str,
        read: impl Fn(&str) -> Result<String, String>,
    ) -> Result<(String, ConcreteSchema), Unresolved> {
        let (path, value) = self.one(id, None, &read)?;
        let concrete = not_concrete(id, &path, &value)?;
        Ok((path, concrete))
    }

    /// 置き場所の Concrete（Abstract の版を申告する文書）を、$id の順に。$ref の先は引かない。
    ///
    /// # Errors
    /// 同じ $id のファイルが2つ以上ある（duplicate_schema）・ 読めない（store）とき。
    pub fn concretes(
        &self,
        read: impl Fn(&str) -> Result<String, String>,
    ) -> Result<Vec<(String, ConcreteSchema)>, Unresolved> {
        let mut out = Vec::new();
        for id in self.ids.keys() {
            let (path, value) = self.one(id, None, &read)?;
            if let Some(c) = ConcreteSchema::read(&value) {
                out.push((path, c));
            }
        }
        Ok(out)
    }

    /// $id が id の Concrete と、その $ref の先を引く。shipped は入れて出すスキーマの名前か（置き場所から引かない）。
    ///
    /// # Errors
    /// 置き場所に無い ・ 2つ以上ある ・ Concrete でない（unknown_schema ・ duplicate_schema）・ 読めない（store）とき。
    pub fn resolve(
        &self,
        id: &str,
        read: impl Fn(&str) -> Result<String, String>,
        shipped: impl Fn(&str) -> bool,
    ) -> Result<Resolved, Unresolved> {
        let (path, value) = self.one(id, None, &read)?;
        let concrete = not_concrete(id, &path, &value)?;
        let more = self.with_refs(id, value, &read, &shipped)?;
        Ok(Resolved {
            path,
            concrete,
            more,
        })
    }

    /// 中身が value の文書（$id が id）と、その $ref の先を、形の検査に渡す並びにする。
    /// 直した Concrete の新しい中身で、従うインスタンスを照らすときに使う（id の文書は置き場所から引かない）。
    ///
    /// # Errors
    /// $ref の先が置き場所に無い ・ 2つ以上ある（unknown_schema ・ duplicate_schema）・ 読めない（store）とき。
    pub fn with_refs(
        &self,
        id: &str,
        value: Value,
        read: impl Fn(&str) -> Result<String, String>,
        shipped: impl Fn(&str) -> bool,
    ) -> Result<Vec<(String, Value)>, Unresolved> {
        let mut seen: BTreeSet<String> = BTreeSet::from([id.to_string()]);
        let mut queue: Vec<(String, String)> = refs(&value)
            .into_iter()
            .map(|r| (r, id.to_string()))
            .collect();
        let mut more = vec![(id.to_string(), value)];
        while let Some((next, from)) = queue.pop() {
            if shipped(&next) || !seen.insert(next.clone()) {
                continue;
            }
            let (_, doc) = self.one(&next, Some(&from), &read)?;
            queue.extend(refs(&doc).into_iter().map(|r| (r, next.clone())));
            more.push((next, doc));
        }
        Ok(more)
    }

    /// $id が id の文書を1つ読む。from は $ref で指した文書の $id。
    fn one(
        &self,
        id: &str,
        from: Option<&str>,
        read: &impl Fn(&str) -> Result<String, String>,
    ) -> Result<(String, Value), Unresolved> {
        let by = from.map_or_else(String::new, |f| format!("{f} が $ref で指す "));
        let paths = self.ids.get(id).cloned().unwrap_or_default();
        let path = match paths.as_slice() {
            [] => {
                return Err(Unresolved {
                    code: "unknown_schema",
                    message: format!(
                        "{by}{id} は、渡されたスキーマの置き場所（--schemas）に無い。取りに行かない"
                    ),
                    paths: Vec::new(),
                });
            }
            [one] => one.clone(),
            _ => {
                return Err(Unresolved {
                    code: "duplicate_schema",
                    message: format!(
                        "{by}{id} を持つファイルが、渡されたスキーマの置き場所に {} つある（{}）。どれを使うかを決めない",
                        paths.len(),
                        paths.join(" ・ ")
                    ),
                    paths,
                });
            }
        };
        let text = read(&path).map_err(|message| Unresolved {
            code: "store",
            message,
            paths: Vec::new(),
        })?;
        let value = serde_json::from_str::<Value>(&text).map_err(|e| Unresolved {
            code: "store",
            message: format!("{path} が JSON として読めない：{e}"),
            paths: Vec::new(),
        })?;
        Ok((path, value))
    }
}

/// Abstract の版を申告する Concrete として読む。読めなければ unknown_schema。
fn not_concrete(id: &str, path: &str, value: &Value) -> Result<ConcreteSchema, Unresolved> {
    ConcreteSchema::read(value).ok_or_else(|| Unresolved {
        code: "unknown_schema",
        message: format!("{id}（{path}）は Abstract の版を申告する Concrete でない"),
        paths: Vec::new(),
    })
}

/// 文書の中の $ref が指す、ほかの文書の名前（# より前。文書の中を指すものは除く）。
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

    fn place(
        files: &[(&str, Value)],
    ) -> (Vec<String>, impl Fn(&str) -> Result<String, String> + use<>) {
        let map: BTreeMap<String, String> = files
            .iter()
            .map(|(p, v)| ((*p).to_string(), v.to_string()))
            .collect();
        let paths = map.keys().cloned().collect();
        (paths, move |p: &str| {
            map.get(p).cloned().ok_or_else(|| format!("{p} が無い"))
        })
    }

    #[test]
    fn resolves_concrete_and_refs_from_the_place() {
        let (files, read) = place(&[
            (
                "s/a.json",
                json!({"$schema": ABSTRACT, "$id": "urn:a", "properties": {"b": {"$ref": "urn:b#/$defs/x"}}}),
            ),
            ("s/b.json", json!({"$id": "urn:b", "$defs": {"x": {}}})),
            ("s/note.json", json!({"id": "N1"})),
        ]);
        let index = SchemaIndex::read(&files, &read).unwrap();
        assert_eq!(index.ids(), ["urn:a", "urn:b"]);
        let r = index.resolve("urn:a", &read, |_| false).unwrap();
        assert_eq!(r.path(), "s/a.json");
        let ids: Vec<&str> = r.more().iter().map(|(i, _)| i.as_str()).collect();
        assert_eq!(ids, ["urn:a", "urn:b"]);
    }

    #[test]
    fn refuses_a_duplicate_id_with_every_path() {
        let (files, read) = place(&[
            ("a/x.json", json!({"$schema": ABSTRACT, "$id": "urn:a"})),
            ("b/x.json", json!({"$schema": ABSTRACT, "$id": "urn:a"})),
        ]);
        let index = SchemaIndex::read(&files, &read).unwrap();
        let e = index.resolve("urn:a", &read, |_| false).unwrap_err();
        assert_eq!(e.code(), "duplicate_schema");
        assert_eq!(e.paths(), ["a/x.json", "b/x.json"]);
    }

    #[test]
    fn refuses_a_missing_ref() {
        let (files, read) = place(&[(
            "s/a.json",
            json!({"$schema": ABSTRACT, "$id": "urn:a", "$ref": "urn:gone"}),
        )]);
        let index = SchemaIndex::read(&files, &read).unwrap();
        let e = index.resolve("urn:a", &read, |_| false).unwrap_err();
        assert_eq!(e.code(), "unknown_schema");
        assert!(e.message().contains("urn:a が $ref で指す urn:gone"));
    }

    #[test]
    fn shipped_refs_are_not_looked_up() {
        let (files, read) = place(&[(
            "s/a.json",
            json!({"$schema": ABSTRACT, "$id": "urn:a", "$ref": "urn:shipped"}),
        )]);
        let index = SchemaIndex::read(&files, &read).unwrap();
        let r = index
            .resolve("urn:a", &read, |id| id == "urn:shipped")
            .unwrap();
        assert_eq!(r.more().len(), 1);
    }
}
