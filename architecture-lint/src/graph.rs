//! 公開する形の型（規則 ・ グラフ ・ 結果の JSON）。形は rules.schema.json ・ graph.schema.json ・
//! result.schema.json が決める。core の型とは分け、ここで相互に直す。core の型を変えても公開の形は変わらない。

use serde::{Deserialize, Serialize};

use crate::core;

/// いま読める形の版。欄を足すだけなら同じ版、意味を変えるなら上げる
pub const VERSION: u32 = 1;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RulesDoc {
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub components: Vec<ComponentDoc>,
    pub allow: Vec<AllowDoc>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentDoc {
    pub name: String,
    pub nodes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AllowDoc {
    pub from: String,
    pub to: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kinds: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphDoc {
    pub version: u32,
    pub nodes: Vec<NodeDoc>,
    pub edges: Vec<EdgeDoc>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeDoc {
    pub name: String,
    pub external: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EdgeDoc {
    pub from: String,
    pub to: String,
    pub kind: String,
}

#[derive(Debug, Serialize)]
pub struct ResultDoc {
    pub version: u32,
    pub ok: bool,
    pub violations: Vec<ViolationDoc>,
}

#[derive(Debug, Serialize)]
pub struct ViolationDoc {
    pub reason: &'static str,
    pub from: String,
    pub to: Option<String>,
    pub kind: Option<String>,
    pub from_component: Option<String>,
    pub to_component: Option<String>,
}

fn check_version(what: &str, v: u32) -> Result<(), String> {
    if v == VERSION {
        Ok(())
    } else {
        Err(format!(
            "{what} の版 {v} は読めない（読める版は {VERSION}）"
        ))
    }
}

pub fn parse_rules(text: &str) -> Result<core::Rules, String> {
    let doc: RulesDoc =
        serde_json::from_str(text).map_err(|e| format!("規則の JSON が形に合わない：{e}"))?;
    check_version("規則", doc.version)?;
    Ok(core::Rules {
        components: doc
            .components
            .into_iter()
            .map(|c| core::Component {
                name: c.name,
                nodes: c.nodes,
            })
            .collect(),
        allow: doc
            .allow
            .into_iter()
            .map(|a| core::Allow {
                from: a.from,
                to: a.to,
                kinds: a.kinds,
            })
            .collect(),
    })
}

pub fn parse_graph(text: &str) -> Result<core::Graph, String> {
    let doc: GraphDoc =
        serde_json::from_str(text).map_err(|e| format!("グラフの JSON が形に合わない：{e}"))?;
    check_version("グラフ", doc.version)?;
    Ok(core::Graph {
        nodes: doc
            .nodes
            .into_iter()
            .map(|n| core::Node {
                name: n.name,
                external: n.external,
            })
            .collect(),
        edges: doc
            .edges
            .into_iter()
            .map(|e| core::Edge {
                from: e.from,
                to: e.to,
                kind: e.kind,
            })
            .collect(),
    })
}

pub fn graph_doc(g: &core::Graph) -> GraphDoc {
    GraphDoc {
        version: VERSION,
        nodes: g
            .nodes
            .iter()
            .map(|n| NodeDoc {
                name: n.name.clone(),
                external: n.external,
            })
            .collect(),
        edges: g
            .edges
            .iter()
            .map(|e| EdgeDoc {
                from: e.from.clone(),
                to: e.to.clone(),
                kind: e.kind.clone(),
            })
            .collect(),
    }
}

pub fn reason_name(r: core::Reason) -> &'static str {
    match r {
        core::Reason::NotInAllowed => "not-in-allowed",
        core::Reason::Uncovered => "uncovered",
    }
}

pub fn result_doc(vs: &[core::Violation]) -> ResultDoc {
    ResultDoc {
        version: VERSION,
        ok: vs.is_empty(),
        violations: vs
            .iter()
            .map(|v| ViolationDoc {
                reason: reason_name(v.reason),
                from: v.from.clone(),
                to: v.to.clone(),
                kind: v.kind.clone(),
                from_component: v.from_component.clone(),
                to_component: v.to_component.clone(),
            })
            .collect(),
    }
}
