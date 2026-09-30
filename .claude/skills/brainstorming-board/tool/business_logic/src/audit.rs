// SPDX-License-Identifier: MIT
//! 出す前に、機械で見られるものだけを検査する。**見つけるが、直さない。**
//!
//! 見るのは6つ。**どれも、実際にやらかしたものである。**
//!
//! | 何を見るか | 何が起きたか |
//! |---|---|
//! | 一度に開く論点が多すぎないか | 8件を同時に出し、どれを確認すればよいか判定できなくなった |
//! | いま見る論点を示しているか | 8件を同時に出し、順番の管理を承認する側へ渡した |
//! | 宣言されていない依存が無いか | 答えの本文だけが他の論点を前提にし、根拠の欄に出てこなかった |
//! | 試す相手が在るか | 下流にも外の作業にも使われない答えを、承認へ出そうとした |
//! | 扱わない範囲に扱いが在るか | 制約を並べたまま承認を求め、覆すか否かを示さなかった |
//! | 答えに完成イメージが在るか | 図も実例も無いまま承認を求めた |
//!
//! **見えないもの**が4つある ── 答えの中身が正しいか、反証が十分か、図が主張を運べて
//! いるか、そして**「論点N」と書かずに他の論点を前提にしている依存**である。

use std::collections::{BTreeMap, BTreeSet};

use crate::topic::Topic;

/// 一度に開ける論点の数。
const OPEN_MAX: usize = 5;
/// 現在地に「いま見る論点」を要求しはじめる論点の数。
const SHOW_FROM: usize = 4;

/// 本文の中の「論点N」を集める。
fn referred(body: &str) -> BTreeSet<usize> {
    const HEAD: &str = "論点";
    let mut out = BTreeSet::new();
    let mut from = 0;
    while let Some(at) = body[from..].find(HEAD).map(|x| x + from) {
        let rest = &body[at + HEAD.len()..];
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if let Ok(no) = digits.parse::<usize>() {
            out.insert(no);
        }
        from = at + HEAD.len();
    }
    out
}

/// 出す前の検査。
#[must_use]
pub fn audit(
    topics: &[Topic],
    extras: &[(String, String)],
    queue: &[(usize, String)],
) -> Vec<String> {
    let mut out = Vec::new();
    let front: BTreeSet<usize> = queue.iter().map(|(no, _)| *no).collect();
    let open_now: BTreeSet<usize> = if front.is_empty() {
        topics
            .iter()
            .filter(|t| !t.settled())
            .map(|t| t.no)
            .collect()
    } else {
        front.clone()
    };
    if open_now.len() > OPEN_MAX {
        out.push(format!(
            "一度に開いている論点が{}件ある。3〜5件に絞る ── 足すのは、既存の答えを直しても収まらないと確認し、利用者へ確認してからである",
            open_now.len()
        ));
    }

    let titles: String = extras
        .iter()
        .map(|(t, _)| t.clone())
        .collect::<Vec<_>>()
        .join(" ");
    // **開いている論点が1件も無い回は、示すものが無い** ── まとめの回である。
    // **queue を渡した回は、道具が箱を組む** ── 面にも同じ表を置くと、どちらが正しいかを
    // 読み手が突き合わせることになる
    if topics.len() >= SHOW_FROM
        && !open_now.is_empty()
        && queue.is_empty()
        && !titles.contains("いま見る論点")
    {
        out.push(
            "現在地に「いま見る論点」が無い。依存を自分で持つだけでは足りない ── 示さなければ、順番の管理が承認する側の仕事になる"
                .to_owned(),
        );
    }

    for t in topics {
        // **待ちの論点は、承認へ出していない** ── 完成イメージを要求するのは、答えを
        // 承認へ出す回である
        if t.status != "waiting"
            && !t.answer.is_empty()
            && t.figures.is_empty()
            && t.example.is_empty()
        {
            out.push(format!(
                "論点{}: 答えを持つのに、完成イメージ（図か実例）が無い ── 文章だけでは、読み手が頭の中で像を作り、そこで解釈がぶれる",
                t.no
            ));
        }
    }

    let mut dep: BTreeMap<usize, (BTreeSet<usize>, BTreeSet<usize>)> = BTreeMap::new();
    for t in topics {
        let src: String = t
            .grounds
            .iter()
            .map(|(_, _, _, s)| s.clone())
            .collect::<Vec<_>>()
            .join(" ");
        let body = format!(
            "{} {}",
            t.answer,
            t.pick.as_ref().map_or("", |(_, b)| b.as_str())
        );
        let mut from_grounds = referred(&src);
        from_grounds.remove(&t.no);
        let mut from_body = referred(&body);
        from_body.remove(&t.no);
        dep.insert(t.no, (from_grounds, from_body));
    }
    let mut used: BTreeMap<usize, BTreeSet<usize>> =
        topics.iter().map(|t| (t.no, BTreeSet::new())).collect();
    for (no, (g, b)) in &dep {
        for x in g.union(b) {
            used.entry(*x).or_default().insert(*no);
        }
    }

    for t in topics {
        let (g, b) = dep.get(&t.no).cloned().unwrap_or_default();
        let hidden: Vec<String> = b.difference(&g).map(usize::to_string).collect();
        if !hidden.is_empty() {
            out.push(format!(
                "論点{}: 答えの本文が論点{}を前提にしているが、根拠の欄に出てこない ── 宣言されていない依存である",
                t.no,
                hidden.join("・")
            ));
        }
        if front.contains(&t.no) && !t.weaknesses.is_empty() {
            let naked = t
                .weaknesses
                .iter()
                .filter(|(_, how)| how.is_empty())
                .count();
            if naked > 0 {
                out.push(format!(
                    "論点{}: いま見る論点だが、扱わない範囲{naked}件に扱いが無い ── 承認者は、それが答えを覆すかを判定できない",
                    t.no
                ));
            }
        }
        if !t.settled() && t.decided() && used.get(&t.no).is_none_or(BTreeSet::is_empty) {
            // 末端の論点には下流が無い。そこでは、外の作業（試作 ・ 実装）が試す相手になる。
            // 「外の作業」は SKILL.md が使う語である。語形に頼らず、この語だけを見る
            let outer = t
                .weaknesses
                .iter()
                .any(|(item, how)| item.contains("外の作業") || how.contains("外の作業"));
            if !outer {
                out.push(format!(
                    "論点{}: 答えを持つが、下流のどの論点にも使われておらず、外の作業で試す行き先も書かれていない ── 試す相手が無い",
                    t.no
                ));
            }
        }
    }
    out
}
