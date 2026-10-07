//! ボードの Design。
//! 論点ごとに、問い ・ 答え ・ 完成イメージ ・ 根拠 ・ 採らなかった案だけを描画する。verification は描画しない。
//! 見た目は部品（parts.html ・ board.css ・ board.js）とデザインシステム（design-system.json）が持ち、ここは値を渡すだけにする。

use schema_driven_core::domain::Html;
use schema_driven_core::ports::outbound::{Design, Frame, Renderer};
use serde_json::{json, Value};
use std::sync::OnceLock;

const PARTS: &str = include_str!("../../../references/design/parts.html");
const STYLE: &str = include_str!("../../../references/design/board.css");
const SCRIPT: &str = include_str!("../../../references/design/board.js");
const DESIGN_SYSTEM: &str = include_str!("../../../references/design/design-system.json");

/// 部品に、見た目（CSS）と動き（JS）を入れたもの。
fn parts() -> &'static str {
    static PARTS_WITH_STYLE: OnceLock<String> = OnceLock::new();
    PARTS_WITH_STYLE.get_or_init(|| {
        PARTS
            .replace("/*STYLE*/", STYLE)
            .replace("/*SCRIPT*/", SCRIPT)
    })
}

pub struct BoardDesign;

fn text<'value>(value: &'value Value, key: &str) -> &'value str {
    value.get(key).and_then(Value::as_str).unwrap_or_default()
}

fn list<'value>(value: &'value Value, key: &str) -> &'value [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

/// 状態の表示（語と、CSS のクラス）。いま見る論点は、開いている論点と分けて示す。
fn status(topic: &Value, queued: &[String]) -> (&'static str, &'static str) {
    match text(topic, "status") {
        "settled" => ("決着", "done"),
        "waiting" => ("待ち", "wait"),
        _ if queued.iter().any(|id| id == text(topic, "id")) => ("いま見る", "now"),
        _ => ("開いている", "open"),
    }
}

/// 根拠の種別の表示（語と、CSS のクラス）。
fn tag(value: &str) -> (&'static str, &'static str) {
    match value {
        "measured" => ("実測", "k-fact"),
        "primary" => ("原典", "k-src"),
        "assumption" => ("前提", "k-given"),
        "unverified" => ("未確認", "k-open"),
        _ => ("決まり", "k-rule"),
    }
}

fn image_label(kind: &str) -> &'static str {
    match kind {
        "figure" => "図",
        "code" => "コード",
        "ui" => "UI",
        _ => "差分",
    }
}

impl BoardDesign {
    fn images(&self, topic: &Value, group: usize, renderer: &dyn Renderer) -> Result<Html, String> {
        let images = list(topic, "images");
        if images.is_empty() {
            return Ok(Html::default());
        }
        let (mut buttons, mut panes, mut kinds) = (Html::default(), Html::default(), Vec::new());
        for (index, image) in images.iter().enumerate() {
            let kind = text(image, "kind");
            let label = image_label(kind);
            if !kinds.contains(&label) {
                kinds.push(label);
            }
            let body = match kind {
                "figure" => {
                    renderer.part("figure", &[("svg", renderer.svg(text(image, "svg"))?)])?
                }
                "code" => renderer.part("code", &[("text", renderer.text(text(image, "text")))])?,
                "ui" => renderer.part("ui", &[("html", renderer.text(text(image, "html")))])?,
                _ => renderer.part(
                    "diff",
                    &[
                        ("before", renderer.text(text(image, "before"))),
                        ("after", renderer.text(text(image, "after"))),
                    ],
                )?,
            };
            let caption = match image.get("caption").and_then(Value::as_str) {
                Some(caption) if !caption.is_empty() => {
                    renderer.part("caption", &[("text", renderer.text(caption))])?
                }
                _ => Html::default(),
            };
            let at = renderer.text(&index.to_string());
            let group_text = renderer.text(&group.to_string());
            buttons.push(renderer.part(
                "image-tab",
                &[
                    (
                        "selected",
                        renderer.text(if index == 0 { "true" } else { "false" }),
                    ),
                    ("group", group_text.clone()),
                    ("index", at.clone()),
                    ("label", renderer.text(label)),
                ],
            )?);
            panes.push(renderer.part(
                "image-pane",
                &[
                    ("group", group_text),
                    ("index", at),
                    (
                        "hidden",
                        renderer.text(if index == 0 { "" } else { " hidden" }),
                    ),
                    ("body", body),
                    ("caption", caption),
                ],
            )?);
        }
        // 1つだけなら切り替えを出さない
        let tabs = if images.len() > 1 {
            renderer.part("image-tabs", &[("buttons", buttons)])?
        } else {
            Html::default()
        };
        renderer.part(
            "images",
            &[
                ("kinds", renderer.text(&kinds.join(" ・ "))),
                ("tabs", tabs),
                ("panes", panes),
            ],
        )
    }

    fn grounds(&self, topic: &Value, renderer: &dyn Renderer) -> Result<Html, String> {
        let grounds = list(topic, "grounds");
        if grounds.is_empty() {
            return Ok(Html::default());
        }
        let mut rows = Html::default();
        for ground in grounds {
            let (label, class) = tag(text(ground, "tag"));
            rows.push(renderer.part(
                "ground",
                &[
                    ("supports", renderer.text(text(ground, "supports"))),
                    ("basis", renderer.text(text(ground, "basis"))),
                    ("tag_class", renderer.text(class)),
                    ("tag", renderer.text(label)),
                    ("source", renderer.text(text(ground, "source"))),
                ],
            )?);
        }
        renderer.part(
            "grounds",
            &[
                ("count", renderer.text(&grounds.len().to_string())),
                ("rows", rows),
            ],
        )
    }

    fn rejected(&self, topic: &Value, renderer: &dyn Renderer) -> Result<Html, String> {
        let rejected = list(topic, "rejected");
        if rejected.is_empty() {
            return Ok(Html::default());
        }
        let mut rows = Html::default();
        for option in rejected {
            rows.push(renderer.part(
                "rejection",
                &[
                    ("option", renderer.text(text(option, "option"))),
                    ("reason", renderer.text(text(option, "reason"))),
                ],
            )?);
        }
        renderer.part(
            "rejected",
            &[
                ("count", renderer.text(&rejected.len().to_string())),
                ("rows", rows),
            ],
        )
    }

    fn topic(&self, topic: &Value, group: usize, renderer: &dyn Renderer) -> Result<Html, String> {
        let answer = topic.get("answer").cloned().unwrap_or(json!({}));
        let mut body = Html::default();
        if text(topic, "status") == "waiting" {
            body.push(renderer.part("waiting", &[("text", renderer.text(text(&answer, "text")))])?);
        } else {
            let mut items = Html::default();
            for detail in list(&answer, "details") {
                items.push(renderer.part(
                    "item",
                    &[("text", renderer.text(detail.as_str().unwrap_or_default()))],
                )?);
            }
            let details = if items.as_str().is_empty() {
                Html::default()
            } else {
                renderer.part("details", &[("items", items)])?
            };
            body.push(renderer.part(
                "answer",
                &[
                    ("text", renderer.text(text(&answer, "text"))),
                    ("details", details),
                ],
            )?);
            body.push(self.images(topic, group, renderer)?);
            body.push(self.grounds(topic, renderer)?);
            body.push(self.rejected(topic, renderer)?);
            if text(topic, "status") == "open" {
                body.push(renderer.part("form", &[("id", renderer.text(text(topic, "id")))])?);
            }
        }
        renderer.part(
            "topic",
            &[
                ("id", renderer.text(text(topic, "id"))),
                ("question", renderer.text(text(topic, "question"))),
                ("body", body),
            ],
        )
    }

    fn board(&self, board: &Value, renderer: &dyn Renderer) -> Result<Html, String> {
        let topics = list(board, "topics");
        let queued: Vec<String> = list(board, "queue")
            .iter()
            .map(|item| text(item, "topic").to_owned())
            .collect();
        let at_of = |id: &str| {
            topics
                .iter()
                .position(|topic| text(topic, "id") == id)
                .map(|index| index + 1)
                .unwrap_or(0)
        };
        let (mut tabs, mut panes, mut rows) = (Html::default(), Html::default(), Html::default());
        for (index, topic) in topics.iter().enumerate() {
            let at = renderer.text(&(index + 1).to_string());
            let (label, class) = status(topic, &queued);
            tabs.push(renderer.part(
                "tab",
                &[
                    ("at", at.clone()),
                    (
                        "class",
                        renderer.text(if class == "now" || class == "wait" {
                            class
                        } else {
                            ""
                        }),
                    ),
                    ("id", renderer.text(text(topic, "id"))),
                    ("name", renderer.text(text(topic, "name"))),
                    ("status_class", renderer.text(class)),
                    ("status", renderer.text(label)),
                ],
            )?);
            panes.push(renderer.part(
                "pane",
                &[
                    ("at", at),
                    ("body", self.topic(topic, index + 1, renderer)?),
                ],
            )?);
            rows.push(renderer.part(
                "front-row",
                &[
                    ("id", renderer.text(text(topic, "id"))),
                    ("question", renderer.text(text(topic, "question"))),
                    ("status_class", renderer.text(class)),
                    ("status", renderer.text(label)),
                    (
                        "answer",
                        renderer.text(text(topic.get("answer").unwrap_or(&Value::Null), "text")),
                    ),
                ],
            )?);
        }
        let intro = match board.get("intro").and_then(Value::as_str) {
            Some(intro) if !intro.is_empty() => {
                renderer.part("intro", &[("text", renderer.text(intro))])?
            }
            _ => Html::default(),
        };
        let queue = if queued.is_empty() {
            Html::default()
        } else {
            let mut items = Html::default();
            for item in list(board, "queue") {
                let id = text(item, "topic");
                let name = topics
                    .iter()
                    .find(|topic| text(topic, "id") == id)
                    .map(|topic| text(topic, "name"))
                    .unwrap_or_default();
                items.push(renderer.part(
                    "queue-item",
                    &[
                        ("at", renderer.text(&at_of(id).to_string())),
                        ("id", renderer.text(id)),
                        ("name", renderer.text(name)),
                        ("why", renderer.text(text(item, "why"))),
                    ],
                )?);
            }
            renderer.part(
                "queue",
                &[
                    ("count", renderer.text(&queued.len().to_string())),
                    ("items", items),
                ],
            )?
        };
        let front = renderer.part(
            "front",
            &[("intro", intro), ("queue", queue), ("rows", rows)],
        )?;
        let open = topics
            .iter()
            .filter(|topic| text(topic, "status") == "open")
            .count();
        let send = if open > 0 {
            renderer.part("send", &[("open", renderer.text(&open.to_string()))])?
        } else {
            Html::default()
        };
        renderer.part(
            "board",
            &[
                ("board", renderer.text(text(board, "id"))),
                (
                    "round",
                    renderer.text(&board.get("round").map(Value::to_string).unwrap_or_default()),
                ),
                ("title", renderer.text(text(board, "title"))),
                ("tabs", tabs),
                ("front", front),
                ("panes", panes),
                ("send", send),
            ],
        )
    }
}

impl Design for BoardDesign {
    fn components(&self) -> Vec<(String, Value)> {
        vec![(
            "board".into(),
            json!({"type": "object", "required": ["value"]}),
        )]
    }

    fn parts(&self) -> &str {
        parts()
    }

    fn design_system(&self) -> &str {
        DESIGN_SYSTEM
    }

    fn page(&self, frame: &Frame, renderer: &dyn Renderer) -> Result<Html, String> {
        renderer.part(
            "page",
            &[
                ("title", renderer.text(&frame.title)),
                ("tokens", renderer.tokens()),
                ("sections", frame.sections.clone()),
            ],
        )
    }

    /// ボードは1つの部品（board）が全体を描画するので、節は中身だけを返す。
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
        match name {
            "board" => {
                let board = renderer.value(&input["value"])?;
                self.board(&board, renderer)
            }
            other => Err(format!("知らない部品：{other}")),
        }
    }
}
