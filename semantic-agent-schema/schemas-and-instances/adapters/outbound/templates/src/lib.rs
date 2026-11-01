//! ポート TemplateRenderer を Mustache（mustache/spec の必須の部分）で実装する。処理系は mustache.rs に自分で持ち、
//! 仕様の試験（tests/mustache-spec/）をそのまま通す。テンプレートはファイルから読まず、use_cases が読んだ本文を受け取る。

pub mod mustache;

use rendering_application::ports::{TemplateNode, TemplateRenderer};
use serde_json::Value;
use std::collections::BTreeMap;

/// Mustache の処理系。
#[derive(Debug, Clone, Copy, Default)]
pub struct Mustache;

fn to_port(nodes: Vec<mustache::Node>) -> Vec<TemplateNode> {
    nodes
        .into_iter()
        .filter_map(|n| match n {
            mustache::Node::Text(_) => None,
            mustache::Node::Var { name, raw } => Some(TemplateNode::Var { name, raw }),
            mustache::Node::Section {
                name,
                inverted,
                children,
            } => Some(TemplateNode::Section {
                name,
                inverted,
                children: to_port(children),
            }),
            mustache::Node::Partial { name, .. } => Some(TemplateNode::Partial { name }),
        })
        .collect()
}

impl TemplateRenderer for Mustache {
    fn render(
        &self,
        templates: &[(String, String)],
        entry: &str,
        context: &Value,
    ) -> Result<String, String> {
        let partials: BTreeMap<String, String> = templates.iter().cloned().collect();
        let source = partials
            .get(entry)
            .ok_or_else(|| format!("テンプレート {entry} が無い"))?;
        let nodes = mustache::parse(source)
            .map_err(|e| format!("テンプレート {entry} の書き方の誤り：{e}"))?;
        mustache::render(&nodes, context, &partials)
            .map_err(|e| format!("テンプレート {entry} で描けない：{e}"))
    }

    fn inspect(&self, source: &str) -> Result<Vec<TemplateNode>, String> {
        mustache::parse(source)
            .map(to_port)
            .map_err(|e| format!("書き方の誤り：{e}"))
    }
}
