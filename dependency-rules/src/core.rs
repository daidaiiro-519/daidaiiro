//! 判定の中心。node と edge のグラフと規則だけを受け取り、違反を返す。
//! 入出力をしない。言語の語（crate ・ Cargo.toml など）を持たない。

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub name: String,
    pub external: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub kind: String,
}

#[derive(Debug, Clone, Default)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    /// node の名前のパターン。完全一致か、末尾の `*` で前方一致
    pub nodes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Allow {
    pub from: String,
    pub to: Vec<String>,
    /// None ならすべての種類の辺を許す
    pub kinds: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct Rules {
    pub components: Vec<Component>,
    pub allow: Vec<Allow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reason {
    /// allow に無い辺
    NotInAllowed,
    /// どの component にも入らない node
    Uncovered,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Violation {
    pub reason: Reason,
    pub from: String,
    /// 辺の違反なら相手の node。辺を持たない node の uncovered なら None
    pub to: Option<String>,
    pub kind: Option<String>,
    pub from_component: Option<String>,
    pub to_component: Option<String>,
}

/// 規則そのものの誤り。違反とは分けて返す（終了コード 2）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleError {
    DuplicateComponent(String),
    UnknownComponent {
        allow_from: String,
        name: String,
    },
    EmptyPattern(String),
    /// 1つの node が2つ以上の component に当たる
    Overlap {
        node: String,
        components: Vec<String>,
    },
}

impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleError::DuplicateComponent(n) => write!(f, "component の名前 {n} が2つある"),
            RuleError::UnknownComponent { allow_from, name } => {
                write!(
                    f,
                    "allow（from {allow_from}）が知らない component {name} を指す"
                )
            }
            RuleError::EmptyPattern(c) => write!(f, "component {c} に空のパターンがある"),
            RuleError::Overlap { node, components } => {
                write!(
                    f,
                    "node {node} が2つ以上の component に当たる：{}",
                    components.join(" ・ ")
                )
            }
        }
    }
}

fn matches(pattern: &str, name: &str) -> bool {
    match pattern.strip_suffix('*') {
        Some(prefix) => name.starts_with(prefix),
        None => pattern == name,
    }
}

/// 規則の形だけで分かる誤りを返す
pub fn validate_rules(rules: &Rules) -> Vec<RuleError> {
    let mut errors = Vec::new();
    let mut names = BTreeSet::new();
    for c in &rules.components {
        if !names.insert(c.name.as_str()) {
            errors.push(RuleError::DuplicateComponent(c.name.clone()));
        }
        if c.nodes.iter().any(|p| p.is_empty() || p == "*") {
            errors.push(RuleError::EmptyPattern(c.name.clone()));
        }
    }
    for a in &rules.allow {
        for n in std::iter::once(&a.from).chain(a.to.iter()) {
            if !names.contains(n.as_str()) {
                errors.push(RuleError::UnknownComponent {
                    allow_from: a.from.clone(),
                    name: n.clone(),
                });
            }
        }
    }
    errors
}

/// グラフを規則で検査する。規則に誤りがあれば Err
pub fn check(graph: &Graph, rules: &Rules) -> Result<Vec<Violation>, Vec<RuleError>> {
    let errors = validate_rules(rules);
    if !errors.is_empty() {
        return Err(errors);
    }
    // node → component
    let mut component_of: BTreeMap<&str, &str> = BTreeMap::new();
    let mut overlaps = Vec::new();
    let mut all_nodes: BTreeSet<&str> = graph.nodes.iter().map(|n| n.name.as_str()).collect();
    for e in &graph.edges {
        all_nodes.insert(&e.from);
        all_nodes.insert(&e.to);
    }
    for node in &all_nodes {
        let hit: Vec<&str> = rules
            .components
            .iter()
            .filter(|c| c.nodes.iter().any(|p| matches(p, node)))
            .map(|c| c.name.as_str())
            .collect();
        match hit.len() {
            0 => {}
            1 => {
                component_of.insert(node, hit[0]);
            }
            _ => overlaps.push(RuleError::Overlap {
                node: node.to_string(),
                components: hit.iter().map(|s| s.to_string()).collect(),
            }),
        }
    }
    if !overlaps.is_empty() {
        return Err(overlaps);
    }
    let allowed = |from: &str, to: &str, kind: &str| {
        rules.allow.iter().any(|a| {
            a.from == from
                && a.to.iter().any(|t| t == to)
                && a.kinds
                    .as_ref()
                    .is_none_or(|ks| ks.iter().any(|k| k == kind))
        })
    };
    let mut out = BTreeSet::new();
    let mut touched: BTreeSet<&str> = BTreeSet::new();
    for e in &graph.edges {
        touched.insert(&e.from);
        touched.insert(&e.to);
        let fc = component_of.get(e.from.as_str()).copied();
        let tc = component_of.get(e.to.as_str()).copied();
        let reason = match (fc, tc) {
            (Some(f), Some(t)) if allowed(f, t, &e.kind) => continue,
            (Some(_), Some(_)) => Reason::NotInAllowed,
            _ => Reason::Uncovered,
        };
        out.insert(Violation {
            reason,
            from: e.from.clone(),
            to: Some(e.to.clone()),
            kind: Some(e.kind.clone()),
            from_component: fc.map(str::to_string),
            to_component: tc.map(str::to_string),
        });
    }
    // 辺を持たない node も、どの component にも入らなければ uncovered
    for n in &graph.nodes {
        if !touched.contains(n.name.as_str()) && !component_of.contains_key(n.name.as_str()) {
            out.insert(Violation {
                reason: Reason::Uncovered,
                from: n.name.clone(),
                to: None,
                kind: None,
                from_component: None,
                to_component: None,
            });
        }
    }
    Ok(out.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(n: &str) -> Node {
        Node {
            name: n.into(),
            external: false,
        }
    }
    fn edge(f: &str, t: &str, k: &str) -> Edge {
        Edge {
            from: f.into(),
            to: t.into(),
            kind: k.into(),
        }
    }
    fn comp(n: &str, p: &[&str]) -> Component {
        Component {
            name: n.into(),
            nodes: p.iter().map(|s| s.to_string()).collect(),
        }
    }
    fn allow(f: &str, t: &[&str], k: Option<&[&str]>) -> Allow {
        Allow {
            from: f.into(),
            to: t.iter().map(|s| s.to_string()).collect(),
            kinds: k.map(|ks| ks.iter().map(|s| s.to_string()).collect()),
        }
    }
    fn rules() -> Rules {
        Rules {
            components: vec![
                comp("app", &["app"]),
                comp("domain", &["domain-*"]),
                comp("tmp", &["tempfile"]),
            ],
            allow: vec![
                allow("app", &["domain"], None),
                allow("domain", &["tmp"], Some(&["dev"])),
            ],
        }
    }

    #[test]
    fn 許した辺は違反にならない() {
        let g = Graph {
            nodes: vec![node("app"), node("domain-a")],
            edges: vec![edge("app", "domain-a", "normal")],
        };
        assert_eq!(
            check(&g, &rules()).unwrap(),
            vec![],
            "app → domain は allow にある"
        );
    }

    #[test]
    fn allow_に無い辺は_not_in_allowed() {
        let g = Graph {
            nodes: vec![node("app"), node("domain-a")],
            edges: vec![edge("domain-a", "app", "normal")],
        };
        let v = check(&g, &rules()).unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(
            v[0].reason,
            Reason::NotInAllowed,
            "domain → app は書いていない"
        );
    }

    #[test]
    fn kinds_に無い種類の辺は_not_in_allowed() {
        let g = Graph {
            nodes: vec![node("domain-a")],
            edges: vec![edge("domain-a", "tempfile", "normal")],
        };
        let v = check(&g, &rules()).unwrap();
        assert_eq!(
            v[0].reason,
            Reason::NotInAllowed,
            "tempfile は dev だけに許した"
        );
        let g = Graph {
            nodes: vec![node("domain-a")],
            edges: vec![edge("domain-a", "tempfile", "dev")],
        };
        assert!(
            check(&g, &rules()).unwrap().is_empty(),
            "dev の辺は許される"
        );
    }

    #[test]
    fn 同じ_component_の中の辺も書かなければ禁止() {
        let g = Graph {
            nodes: vec![node("domain-a"), node("domain-b")],
            edges: vec![edge("domain-a", "domain-b", "normal")],
        };
        assert_eq!(check(&g, &rules()).unwrap()[0].reason, Reason::NotInAllowed);
    }

    #[test]
    fn どの_component_にも入らない相手は_uncovered() {
        let g = Graph {
            nodes: vec![node("app")],
            edges: vec![edge("app", "serde", "normal")],
        };
        let v = check(&g, &rules()).unwrap();
        assert_eq!(v[0].reason, Reason::Uncovered);
        assert_eq!(
            v[0].to_component, None,
            "serde はどの component にも入らない"
        );
    }

    #[test]
    fn 辺を持たない_node_も_uncovered() {
        let g = Graph {
            nodes: vec![node("lonely")],
            edges: vec![],
        };
        let v = check(&g, &rules()).unwrap();
        assert_eq!((v[0].reason, v[0].to.clone()), (Reason::Uncovered, None));
    }

    #[test]
    fn 二つの_component_に当たる_node_は規則の誤り() {
        let mut r = rules();
        r.components.push(comp("other", &["domain-a"]));
        let g = Graph {
            nodes: vec![node("domain-a")],
            edges: vec![],
        };
        assert!(matches!(
            check(&g, &r).unwrap_err()[0],
            RuleError::Overlap { .. }
        ));
    }

    #[test]
    fn 知らない_component_を指す_allow_は規則の誤り() {
        let mut r = rules();
        r.allow.push(allow("app", &["nowhere"], None));
        assert!(matches!(
            validate_rules(&r)[0],
            RuleError::UnknownComponent { .. }
        ));
    }

    #[test]
    fn 同じ入力から同じ順の結果() {
        let g = Graph {
            nodes: vec![node("app"), node("domain-a")],
            edges: vec![
                edge("domain-a", "app", "normal"),
                edge("app", "zzz", "dev"),
                edge("app", "aaa", "normal"),
            ],
        };
        assert_eq!(check(&g, &rules()).unwrap(), check(&g, &rules()).unwrap());
    }
}
