// SPDX-License-Identifier: MIT
//! 描いた SVG を、要素の木として読む。
//!
//! **読むのは、この道具が書いたものだけである。** 属性は二重引用符で囲み、実体参照は
//! `&amp;` ・ `&lt;` ・ `&gt;` ・ `&quot;` ・ `&apos;` ・ 数値参照だけを使う。外の道具を
//! 足さない ── この Skill は依存を持たないことを方針としてきた（描く側も検査する側も、
//! 自前で解く）。
//!
//! **壊れていたら、読めないと答える。** 閉じの印が合わないものを黙って読み進めると、
//! 検査が誤った木を相手にする。

/// 要素1つ。
#[derive(Debug, Clone, Default)]
pub struct Element {
    /// 名前（名前空間の接頭辞は除く）。
    pub tag: String,
    /// 属性。**書いた順に持つ。**
    pub attrs: Vec<(String, String)>,
    /// 子。
    pub children: Vec<Node>,
}

/// 木の節。
#[derive(Debug, Clone)]
pub enum Node {
    /// 要素。
    Element(Element),
    /// 文字。
    Text(String),
}

impl Element {
    /// 属性を引く。
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// 子の要素だけを並べる。
    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(|n| match n {
            Node::Element(e) => Some(e),
            Node::Text(_) => None,
        })
    }

    /// 中の文字を、深さ優先で全部つなぐ。
    #[must_use]
    pub fn text(&self) -> String {
        let mut out = String::new();
        for n in &self.children {
            match n {
                Node::Text(t) => out.push_str(t),
                Node::Element(e) => out.push_str(&e.text()),
            }
        }
        out
    }
}

/// 16進の数値参照の基数。
const HEX: u32 = 16;
/// 注釈の閉じ。
const COMMENT_END: &str = "-->";
/// 素の文字の区間の開きと閉じ。
const CDATA_OPEN: &str = "<![CDATA[";
const CDATA_END: &str = "]]>";

/// 実体参照を戻す。
fn unescape(raw: &str) -> Option<String> {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        let end = rest[at..].find(';')? + at;
        let name = &rest[at + 1..end];
        let c = match name {
            "amp" => '&',
            "lt" => '<',
            "gt" => '>',
            "quot" => '"',
            "apos" => '\'',
            _ if name.starts_with("#x") || name.starts_with("#X") => {
                char::from_u32(u32::from_str_radix(&name[2..], HEX).ok()?)?
            }
            _ if name.starts_with('#') => char::from_u32(name[1..].parse().ok()?)?,
            _ => return None,
        };
        out.push(c);
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    Some(out)
}

/// 名前から、名前空間の接頭辞を外す。
fn local(name: &str) -> String {
    name.rsplit(':').next().unwrap_or(name).to_owned()
}

struct Reader<'a> {
    src: &'a str,
    at: usize,
}

impl Reader<'_> {
    fn rest(&self) -> &str {
        &self.src[self.at..]
    }

    fn skip_ws(&mut self) {
        let trimmed = self.rest().trim_start();
        self.at = self.src.len() - trimmed.len();
    }

    /// 宣言 ・ 注釈 ・ 処理命令を読み飛ばす。
    fn skip_misc(&mut self) -> Option<()> {
        loop {
            self.skip_ws();
            if self.rest().starts_with("<?") {
                self.at += self.rest().find("?>")? + 2;
            } else if self.rest().starts_with("<!--") {
                self.at += self.rest().find(COMMENT_END)? + COMMENT_END.len();
            } else if self.rest().starts_with("<!") {
                self.at += self.rest().find('>')? + 1;
            } else {
                return Some(());
            }
        }
    }

    fn name(&mut self) -> Option<String> {
        let end = self
            .rest()
            .find(|c: char| c.is_whitespace() || c == '>' || c == '/' || c == '=')?;
        if end == 0 {
            return None;
        }
        let name = self.rest()[..end].to_owned();
        self.at += end;
        Some(name)
    }

    fn element(&mut self) -> Option<Element> {
        if !self.rest().starts_with('<') {
            return None;
        }
        self.at += 1;
        let tag = self.name()?;
        let mut el = Element {
            tag: local(&tag),
            ..Element::default()
        };
        loop {
            self.skip_ws();
            if self.rest().starts_with("/>") {
                self.at += 2;
                return Some(el);
            }
            if self.rest().starts_with('>') {
                self.at += 1;
                break;
            }
            let key = self.name()?;
            self.skip_ws();
            if !self.rest().starts_with('=') {
                return None;
            }
            self.at += 1;
            self.skip_ws();
            let q = self.rest().chars().next()?;
            if q != '"' && q != '\'' {
                return None;
            }
            self.at += 1;
            let end = self.rest().find(q)?;
            let raw = &self.rest()[..end];
            if raw.contains('<') {
                return None;
            }
            let value = unescape(raw)?;
            self.at += end + 1;
            el.attrs.push((key, value));
        }
        // 中身
        loop {
            if self.rest().starts_with("</") {
                self.at += 2;
                let close = self.name()?;
                if local(&close) != el.tag {
                    return None;
                }
                self.skip_ws();
                if !self.rest().starts_with('>') {
                    return None;
                }
                self.at += 1;
                return Some(el);
            }
            if self.rest().starts_with("<!--") {
                self.at += self.rest().find(COMMENT_END)? + COMMENT_END.len();
                continue;
            }
            if self.rest().starts_with(CDATA_OPEN) {
                let body_at = self.at + CDATA_OPEN.len();
                let end = self.src[body_at..].find(CDATA_END)?;
                el.children
                    .push(Node::Text(self.src[body_at..body_at + end].to_owned()));
                self.at = body_at + end + CDATA_END.len();
                continue;
            }
            if self.rest().starts_with('<') {
                let child = self.element()?;
                el.children.push(Node::Element(child));
                continue;
            }
            if self.rest().is_empty() {
                return None;
            }
            let end = self.rest().find('<').unwrap_or(self.rest().len());
            let text = unescape(&self.rest()[..end])?;
            self.at += end;
            el.children.push(Node::Text(text));
        }
    }
}

/// 読む。**壊れていたら `None` を返す。**
#[must_use]
pub fn parse(src: &str) -> Option<Element> {
    let mut r = Reader { src, at: 0 };
    r.skip_misc()?;
    let root = r.element()?;
    r.skip_misc()?;
    if !r.rest().trim().is_empty() {
        return None;
    }
    Some(root)
}
