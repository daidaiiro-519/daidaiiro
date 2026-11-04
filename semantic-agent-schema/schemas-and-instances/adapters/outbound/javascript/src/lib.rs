//! ポート ScriptReader を、JS の構文解析器 boa（boa_parser 0.22.0）で実装する。
//!
//! テンプレートの JS のロジックは見ない。値の入口（`JSON.parse(document.getElementById('semantic-agent-schema-data').textContent)`）から
//! どの欄を読んでいるかだけを取り出す。
//! - `const data = JSON.parse(…)` の data から、`data.goal` ・ `data.calls[0].id` のようにたどった道
//! - `const { goal } = JSON.parse(…)` や `const { goal } = data` の分割代入
//! - `data[key]` のように実行するまで決まらない読み方と、`render(data)` のように値をそのまま渡す使い方は、追えないものとして返す

use boa_ast::declaration::{Binding, Variable};
use boa_ast::expression::Expression;
use boa_ast::expression::access::{PropertyAccess, PropertyAccessField};
use boa_ast::expression::literal::LiteralKind;
use boa_ast::pattern::{ObjectPatternElement, Pattern};
use boa_ast::property::PropertyName;
use boa_ast::scope::Scope;
use boa_ast::visitor::{VisitWith, Visitor};
use boa_interner::{Interner, ToInternedString};
use boa_parser::{Parser, Source};
use rendering_application::ports::{DataRead, PathSeg, ScriptFacts, ScriptReader};
use std::collections::BTreeSet;
use std::ops::ControlFlow;

/// boa による JS の読み手。
#[derive(Debug, Clone, Copy, Default)]
pub struct Boa;

struct Reads<'i> {
    interner: &'i Interner,
    entry_id: String,
    roots: BTreeSet<String>,
    facts: ScriptFacts,
}

impl Reads<'_> {
    fn text<T: ToInternedString>(&self, t: &T) -> String {
        t.to_interned_string(self.interner)
    }

    /// JSON.parse(document.getElementById('<入口>').textContent) か。
    fn is_entry(&self, e: &Expression) -> bool {
        let Expression::Call(c) = e else {
            return false;
        };
        if self.text(c.function()) != "JSON.parse" {
            return false;
        }
        let arg = c.args().first().map(|a| self.text(a)).unwrap_or_default();
        arg.contains("getElementById")
            && (arg.contains(&format!("\"{}\"", self.entry_id))
                || arg.contains(&format!("'{}'", self.entry_id)))
            && arg.ends_with(".textContent")
    }

    /// 入口の値からの道。入口の値でなければ None。追えない段があれば Err（説明）。
    fn path(&self, e: &Expression) -> Option<Result<Vec<PathSeg>, String>> {
        match e {
            Expression::Identifier(id) if self.roots.contains(&self.text(id)) => {
                Some(Ok(Vec::new()))
            }
            e if self.is_entry(e) => Some(Ok(Vec::new())),
            Expression::PropertyAccess(PropertyAccess::Simple(p)) => {
                let base = self.path(p.target())?;
                Some(base.and_then(|mut segs| {
                    match p.field() {
                        PropertyAccessField::Const(name) => {
                            segs.push(PathSeg::Key(self.text(name)));
                        }
                        PropertyAccessField::Expr(x) => match &**x {
                            Expression::Literal(l) => match l.kind() {
                                LiteralKind::String(s) => {
                                    segs.push(PathSeg::Key(
                                        self.interner.resolve_expect(*s).to_string(),
                                    ));
                                }
                                LiteralKind::Int(_) | LiteralKind::Num(_) => {
                                    segs.push(PathSeg::Index)
                                }
                                _ => return Err(format!("{}（読む所が値で決まる）", self.text(e))),
                            },
                            _ => {
                                return Err(format!(
                                    "{}（読む所が実行するまで決まらない）",
                                    self.text(e)
                                ));
                            }
                        },
                    }
                    Ok(segs)
                }))
            }
            _ => None,
        }
    }

    fn record(&mut self, e: &Expression) -> bool {
        match self.path(e) {
            Some(Ok(segs)) => {
                self.facts.reads_entry = true;
                if !segs.is_empty() {
                    self.facts.reads.push(DataRead::Path(segs));
                }
                true
            }
            Some(Err(why)) => {
                self.facts.reads_entry = true;
                self.facts.reads.push(DataRead::Untraceable(why));
                true
            }
            None => false,
        }
    }
}

impl<'ast> Visitor<'ast> for Reads<'_> {
    type BreakTy = ();

    fn visit_variable(&mut self, d: &'ast Variable) -> ControlFlow<()> {
        let Some(init) = d.init() else {
            return d.visit_with(self);
        };
        match d.binding() {
            // const data = JSON.parse(…) か、const items = data.calls
            Binding::Identifier(id) if self.is_entry(init) => {
                self.facts.reads_entry = true;
                self.roots.insert(self.text(id));
                ControlFlow::Continue(())
            }
            // const { goal, calls } = JSON.parse(…) か data
            Binding::Pattern(Pattern::Object(p)) => match self.path(init) {
                Some(Ok(base)) => {
                    self.facts.reads_entry = true;
                    for el in p.bindings() {
                        match el {
                            ObjectPatternElement::SingleName {
                                name: PropertyName::Literal(key),
                                ..
                            } => {
                                let mut segs = base.clone();
                                segs.push(PathSeg::Key(self.text(key)));
                                self.facts.reads.push(DataRead::Path(segs));
                            }
                            other => self.facts.reads.push(DataRead::Untraceable(format!(
                                "{}（分割代入の形が追えない）",
                                self.text(other)
                            ))),
                        }
                    }
                    ControlFlow::Continue(())
                }
                Some(Err(why)) => {
                    self.facts.reads.push(DataRead::Untraceable(why));
                    ControlFlow::Continue(())
                }
                None => d.visit_with(self),
            },
            _ => d.visit_with(self),
        }
    }

    fn visit_expression(&mut self, e: &'ast Expression) -> ControlFlow<()> {
        match e {
            Expression::PropertyAccess(PropertyAccess::Simple(_)) => {
                if self.record(e) {
                    // 読んだ道の中の [ ] の式は見ない（道として扱った）
                    return ControlFlow::Continue(());
                }
                e.visit_with(self)
            }
            // 入口の値そのものを渡す ・ 返す（render(data) など）。その先を追えない
            Expression::Identifier(id) if self.roots.contains(&self.text(id)) => {
                self.facts.reads.push(DataRead::Untraceable(format!(
                    "{}（値をそのまま使っていて、その先を追えない）",
                    self.text(id)
                )));
                ControlFlow::Continue(())
            }
            e if self.is_entry(e) => {
                self.facts.reads_entry = true;
                self.facts.reads.push(DataRead::Untraceable(
                    "JSON.parse(…) の値をそのまま使っていて、その先を追えない".to_string(),
                ));
                ControlFlow::Continue(())
            }
            _ => e.visit_with(self),
        }
    }
}

impl ScriptReader for Boa {
    fn data_reads(&self, source: &str, entry_id: &str) -> Result<ScriptFacts, String> {
        let mut interner = Interner::default();
        let script = Parser::new(Source::from_bytes(source.as_bytes()))
            .parse_script(&Scope::new_global(), &mut interner)
            .map_err(|e| format!("JS として読めない：{e}"))?;
        let mut v = Reads {
            interner: &interner,
            entry_id: entry_id.to_string(),
            roots: BTreeSet::new(),
            facts: ScriptFacts::default(),
        };
        let _ = script.visit_with(&mut v);
        Ok(v.facts)
    }
}
