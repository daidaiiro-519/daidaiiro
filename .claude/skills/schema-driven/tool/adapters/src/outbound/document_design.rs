//! 基盤の文書（references/document.schema.json）の決まったデザイン（ポート Design の実装）。
//! 何も指定が無いときに、文書のインスタンスを描画する（UC-6）。ブロックの type ごとに部品を選び、
//! プレースホルダーに値を入れるだけで、表現を推し量らない。部品の HTML と CSS は references/document-design が持つ。

use schema_driven_core::domain::Html;
use schema_driven_core::ports::outbound::{Design, Frame, Renderer};
use serde_json::{json, Value};
use std::sync::OnceLock;

const PARTS: &str = include_str!("../../../../references/document-design/parts.html");
const STYLE: &str = include_str!("../../../../references/document-design/document.css");

/// 節がこの数以上なら、題の下に目次を出す。
const TABLE_OF_CONTENTS_FROM: usize = 4;

pub struct DocumentDesign;

fn parts_with_style() -> &'static str {
    static PARTS_WITH_STYLE: OnceLock<String> = OnceLock::new();
    PARTS_WITH_STYLE.get_or_init(|| PARTS.replace("/*STYLE*/", STYLE))
}

fn text_of(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned()
}

fn list_of<'value>(value: &'value Value, key: &str) -> &'value [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn state_label(state: &str) -> &'static str {
    match state {
        "done" => "済",
        "doing" => "進行中",
        "blocked" => "止まっている",
        _ => "未着手",
    }
}

/// 文の中のリンクで許す URL：http(s) と、文書の中の場所（#英数 ・ ハイフン ・ 下線）。
fn allowed_url(url: &str) -> bool {
    url.starts_with("https://")
        || url.starts_with("http://")
        || (url.starts_with('#')
            && url.len() > 1
            && url[1..].chars().all(|character| {
                character.is_ascii_alphanumeric() || character == '-' || character == '_'
            }))
}

/// 文の中の書き方（**強調**、`コード`、[文字](URL)）を部品にする。それ以外の文字は、すべてエスケープする。
fn inline(renderer: &dyn Renderer, text: &str) -> Result<Html, String> {
    let mut html = Html::default();
    let mut plain = String::new();
    let mut rest = text;
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix("**") {
            if let Some(end) = after.find("**") {
                html.push(renderer.text(&std::mem::take(&mut plain)));
                html.push(renderer.part("strong", &[("text", renderer.text(&after[..end]))])?);
                rest = &after[end + 2..];
                continue;
            }
        }
        if let Some(after) = rest.strip_prefix('`') {
            if let Some(end) = after.find('`') {
                html.push(renderer.text(&std::mem::take(&mut plain)));
                html.push(renderer.part("code-inline", &[("text", renderer.text(&after[..end]))])?);
                rest = &after[end + 1..];
                continue;
            }
        }
        if let Some(after) = rest.strip_prefix('[') {
            if let Some(label_end) = after.find("](") {
                if let Some(url_end) = after[label_end + 2..].find(')') {
                    let label = &after[..label_end];
                    let url = &after[label_end + 2..label_end + 2 + url_end];
                    if !allowed_url(url) {
                        return Err(format!(
                            "リンクの URL は http(s) か文書の中の場所（#…）だけ書ける：{url}"
                        ));
                    }
                    html.push(renderer.text(&std::mem::take(&mut plain)));
                    html.push(renderer.part(
                        "link",
                        &[("href", renderer.text(url)), ("text", renderer.text(label))],
                    )?);
                    rest = &after[label_end + 2 + url_end + 1..];
                    continue;
                }
            }
        }
        let character = rest.chars().next().unwrap_or_default();
        plain.push(character);
        rest = &rest[character.len_utf8()..];
    }
    html.push(renderer.text(&plain));
    Ok(html)
}

impl DocumentDesign {
    fn blocks_html(&self, renderer: &dyn Renderer, blocks: &[Value]) -> Result<Html, String> {
        let mut html = Html::default();
        for block in blocks {
            html.push(self.block_html(renderer, block)?);
        }
        Ok(html)
    }

    fn block_html(&self, renderer: &dyn Renderer, block: &Value) -> Result<Html, String> {
        let rich = |key: &str| inline(renderer, &text_of(block, key));
        match block.get("type").and_then(Value::as_str).unwrap_or("") {
            "paragraph" => {
                let part = if block.get("strong") == Some(&Value::Bool(true)) {
                    "paragraph-strong"
                } else {
                    "paragraph"
                };
                renderer.part(part, &[("text", rich("text")?)])
            }
            "properties" => {
                let mut rows = Html::default();
                for row in list_of(block, "rows") {
                    rows.push(renderer.part(
                        "property",
                        &[
                            ("label", inline(renderer, &text_of(row, "label"))?),
                            ("value", inline(renderer, &text_of(row, "value"))?),
                        ],
                    )?);
                }
                renderer.part("properties", &[("rows", rows)])
            }
            "table" => self.table_html(renderer, block),
            "change" => {
                let mut rows = Html::default();
                for row in list_of(block, "rows") {
                    rows.push(renderer.part(
                        "change-row",
                        &[
                            ("what", inline(renderer, &text_of(row, "what"))?),
                            ("before", inline(renderer, &text_of(row, "before"))?),
                            ("after", inline(renderer, &text_of(row, "after"))?),
                        ],
                    )?);
                }
                renderer.part("change", &[("rows", rows)])
            }
            "steps" => self.steps_html(renderer, list_of(block, "items")),
            "timeline" => self.timeline_html(renderer, block),
            "status" => {
                let mut items = Html::default();
                for item in list_of(block, "items") {
                    let state = text_of(item, "state");
                    let note = match item.get("note").and_then(Value::as_str) {
                        Some(note) => {
                            renderer.part("status-note", &[("text", inline(renderer, note)?)])?
                        }
                        None => Html::default(),
                    };
                    items.push(renderer.part(
                        "status-item",
                        &[
                            ("state", renderer.text(&state)),
                            ("label_state", renderer.text(state_label(&state))),
                            ("label", inline(renderer, &text_of(item, "label"))?),
                            ("note", note),
                        ],
                    )?);
                }
                renderer.part("status", &[("items", items)])
            }
            "bars" => self.bars_html(renderer, block),
            "quote" => renderer.part(
                "quote",
                &[("text", rich("text")?), ("source", rich("source")?)],
            ),
            "note" => {
                let level = block.get("level").and_then(Value::as_str).unwrap_or("info");
                renderer.part(
                    "note",
                    &[("level", renderer.text(level)), ("text", rich("text")?)],
                )
            }
            "fold" => {
                let blocks = self.blocks_html(renderer, list_of(block, "blocks"))?;
                renderer.part("fold", &[("summary", rich("summary")?), ("blocks", blocks)])
            }
            "figure" => match block.get("svg").and_then(Value::as_str) {
                Some(svg) => renderer.part(
                    "figure-inline",
                    &[("svg", renderer.svg(svg)?), ("caption", rich("caption")?)],
                ),
                None => renderer.part(
                    "figure",
                    &[
                        ("src", renderer.text(&text_of(block, "src"))),
                        ("alt", renderer.text(&text_of(block, "caption"))),
                        ("caption", rich("caption")?),
                    ],
                ),
            },
            "code" => renderer.part("code", &[("text", renderer.text(&text_of(block, "text")))]),
            "list" => {
                let mut items = Html::default();
                for item in list_of(block, "items") {
                    items.push(renderer.part(
                        "item",
                        &[("text", inline(renderer, item.as_str().unwrap_or(""))?)],
                    )?);
                }
                renderer.part("list", &[("items", items)])
            }
            other => Err(format!("文書のブロックの種類 {other} を描画できない")),
        }
    }

    fn table_html(&self, renderer: &dyn Renderer, block: &Value) -> Result<Html, String> {
        let caption = match block.get("caption").and_then(Value::as_str) {
            Some(caption) => renderer.part("caption", &[("text", inline(renderer, caption)?)])?,
            None => Html::default(),
        };
        // 列ごとに、右揃えか
        let mut right_aligned = Vec::new();
        let mut head = Html::default();
        for column in list_of(block, "columns") {
            let (label, right) = match column {
                Value::String(label) => (label.clone(), false),
                other => (text_of(other, "label"), text_of(other, "align") == "right"),
            };
            right_aligned.push(right);
            let part = if right { "th-right" } else { "th" };
            head.push(renderer.part(part, &[("text", inline(renderer, &label)?)])?);
        }
        let emphasis = block.get("emphasis").and_then(Value::as_u64);
        let mut rows = Html::default();
        for (row_index, row) in list_of(block, "rows").iter().enumerate() {
            let mut cells = Html::default();
            for (column_index, cell) in row
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or(&[])
                .iter()
                .enumerate()
            {
                let body = match cell {
                    Value::String(text) => inline(renderer, text)?,
                    other => renderer.part(
                        "cell-badge",
                        &[
                            ("tone", renderer.text(&text_of(other, "tone"))),
                            ("text", inline(renderer, &text_of(other, "text"))?),
                        ],
                    )?,
                };
                let right = right_aligned.get(column_index).copied().unwrap_or(false);
                cells
                    .push(renderer.part(if right { "td-right" } else { "td" }, &[("text", body)])?);
            }
            let part = if emphasis == Some(row_index as u64) {
                "tr-emphasis"
            } else {
                "tr"
            };
            rows.push(renderer.part(part, &[("cells", cells)])?);
        }
        renderer.part(
            "table",
            &[("caption", caption), ("head", head), ("rows", rows)],
        )
    }

    fn branches_html(&self, renderer: &dyn Renderer, owner: &Value) -> Result<Html, String> {
        let mut branches = Html::default();
        for branch in list_of(owner, "branches") {
            let ending = match branch.get("ending").and_then(Value::as_str) {
                Some(ending) => renderer.part("ending", &[("text", inline(renderer, ending)?)])?,
                None => Html::default(),
            };
            let inner = self.steps_html(renderer, list_of(branch, "steps"))?;
            branches.push(renderer.part(
                "branch",
                &[
                    (
                        "condition",
                        inline(renderer, &text_of(branch, "condition"))?,
                    ),
                    ("steps", inner),
                    ("ending", ending),
                ],
            )?);
        }
        Ok(branches)
    }

    fn steps_html(&self, renderer: &dyn Renderer, steps: &[Value]) -> Result<Html, String> {
        let mut items = Html::default();
        for step in steps {
            let actor = match step.get("actor").and_then(Value::as_str) {
                Some(actor) => renderer.part("actor", &[("text", renderer.text(actor))])?,
                None => Html::default(),
            };
            items.push(renderer.part(
                "step",
                &[
                    ("actor", actor),
                    ("text", inline(renderer, &text_of(step, "text"))?),
                    ("branches", self.branches_html(renderer, step)?),
                ],
            )?);
        }
        renderer.part("steps", &[("items", items)])
    }

    fn timeline_html(&self, renderer: &dyn Renderer, block: &Value) -> Result<Html, String> {
        let mut items = Html::default();
        for item in list_of(block, "items") {
            items.push(renderer.part(
                "timeline-item",
                &[
                    ("time", renderer.text(&text_of(item, "time"))),
                    ("text", inline(renderer, &text_of(item, "text"))?),
                    ("branches", self.branches_html(renderer, item)?),
                ],
            )?);
        }
        renderer.part("timeline", &[("items", items)])
    }

    fn bars_html(&self, renderer: &dyn Renderer, block: &Value) -> Result<Html, String> {
        let items_value = list_of(block, "items");
        let target_value = block.pointer("/target/value").and_then(Value::as_f64);
        let maximum = items_value
            .iter()
            .filter_map(|item| item.get("value").and_then(Value::as_f64))
            .chain(target_value)
            .fold(0.0_f64, f64::max);
        let width_of = |value: f64| {
            if maximum > 0.0 {
                (value / maximum * 100.0).round()
            } else {
                0.0
            }
        };
        let mut items = Html::default();
        for item in items_value {
            let value = item.get("value").and_then(Value::as_f64).unwrap_or(0.0);
            items.push(renderer.part(
                "bar",
                &[
                    ("label", inline(renderer, &text_of(item, "label"))?),
                    ("width", renderer.text(&format!("{}", width_of(value)))),
                    (
                        "value",
                        renderer.text(&item.get("value").map(Value::to_string).unwrap_or_default()),
                    ),
                ],
            )?);
        }
        if let (Some(target), Some(value)) = (block.get("target"), target_value) {
            items.push(renderer.part(
                "bar-target",
                &[
                    ("label", inline(renderer, &text_of(target, "label"))?),
                    ("width", renderer.text(&format!("{}", width_of(value)))),
                    ("value", renderer.text(&target["value"].to_string())),
                ],
            )?);
        }
        let unit = match block.get("unit").and_then(Value::as_str) {
            Some(unit) => renderer.part("unit", &[("text", renderer.text(unit))])?,
            None => Html::default(),
        };
        renderer.part("bars", &[("items", items), ("unit", unit)])
    }

    /// 文書の本体：目次（節が多いとき）・ 節 ・ 出どころ。
    fn document_html(&self, renderer: &dyn Renderer, document: &Value) -> Result<Html, String> {
        let sections = list_of(document, "sections");
        let mut html = Html::default();
        if sections.len() >= TABLE_OF_CONTENTS_FROM {
            let mut items = Html::default();
            for (section_index, section) in sections.iter().enumerate() {
                items.push(renderer.part(
                    "toc-item",
                    &[
                        (
                            "anchor",
                            renderer.text(&format!("section-{}", section_index + 1)),
                        ),
                        ("text", renderer.text(&text_of(section, "heading"))),
                    ],
                )?);
            }
            html.push(renderer.part("toc", &[("items", items)])?);
        }
        for (section_index, section) in sections.iter().enumerate() {
            let blocks = self.blocks_html(renderer, list_of(section, "blocks"))?;
            html.push(renderer.part(
                "section",
                &[
                    (
                        "anchor",
                        renderer.text(&format!("section-{}", section_index + 1)),
                    ),
                    ("heading", renderer.text(&text_of(section, "heading"))),
                    ("blocks", blocks),
                ],
            )?);
        }
        let sources = list_of(document, "sources");
        if !sources.is_empty() {
            let mut items = Html::default();
            for source in sources {
                let url = match source.get("url").and_then(Value::as_str) {
                    Some(url) if allowed_url(url) => renderer.part(
                        "source-url",
                        &[("href", renderer.text(url)), ("text", renderer.text(url))],
                    )?,
                    Some(url) => {
                        return Err(format!("出どころの URL は http(s) だけ書ける：{url}"))
                    }
                    None => Html::default(),
                };
                items.push(renderer.part(
                    "source-item",
                    &[
                        ("id", renderer.text(&text_of(source, "id"))),
                        ("label", inline(renderer, &text_of(source, "label"))?),
                        ("url", url),
                    ],
                )?);
            }
            html.push(renderer.part("sources", &[("items", items)])?);
        }
        Ok(html)
    }
}

impl Design for DocumentDesign {
    fn components(&self) -> Vec<(String, Value)> {
        vec![
            (
                "document".into(),
                json!({"type": "object", "required": ["value"]}),
            ),
            (
                "badges".into(),
                json!({"type": "object", "required": ["value"]}),
            ),
            (
                "inline".into(),
                json!({"type": "object", "required": ["value"]}),
            ),
        ]
    }

    fn parts(&self) -> &str {
        parts_with_style()
    }

    fn page(&self, frame: &Frame, renderer: &dyn Renderer) -> Result<Html, String> {
        let lead = if frame.lead.as_str().is_empty() {
            Html::default()
        } else {
            renderer.part("lead", &[("text", frame.lead.clone())])?
        };
        renderer.part(
            "page",
            &[
                ("badges", frame.badges.clone()),
                ("title", renderer.text(&frame.title)),
                ("lead", lead),
                ("sections", frame.sections.clone()),
            ],
        )
    }

    /// 文書の節は、コンポーネント document が見出しごと描画するので、ページテンプレートの節は中身だけを返す。
    fn section(
        &self,
        _heading: &str,
        body: Html,
        _renderer: &dyn Renderer,
    ) -> Result<Html, String> {
        Ok(body)
    }

    fn render(
        &self,
        name: &str,
        input: &Value,
        renderer: &mut dyn Renderer,
    ) -> Result<Html, String> {
        let value = renderer.value(&input["value"])?;
        match name {
            "badges" => {
                let mut badges = Html::default();
                for badge in value.as_array().map(Vec::as_slice).unwrap_or(&[]) {
                    badges.push(renderer.part(
                        "badge",
                        &[("text", renderer.text(badge.as_str().unwrap_or("")))],
                    )?);
                }
                Ok(badges)
            }
            "inline" => inline(renderer, value.as_str().unwrap_or("")),
            "document" => self.document_html(renderer, &value),
            other => Err(format!("文書のデザインは {other} を描画できない")),
        }
    }
}
