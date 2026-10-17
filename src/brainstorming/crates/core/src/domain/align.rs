//! 並びの突き合わせ。最長共通部分列で、残った要素 ・ 消えた要素 ・ 足した要素に分ける。
//! 位置だけで比べると、1つ足しただけで以降が全部「変わった」になるので、先に突き合わせる。

/// 突き合わせの1手。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Step {
    /// 前の i 番目と後の j 番目が同じ
    Same(usize, usize),
    /// 前の i 番目が消えた
    Removed(usize),
    /// 後の j 番目を足した
    Added(usize),
}

/// 前の並び before と後の並び after を突き合わせる。
pub fn align<T: PartialEq>(before: &[T], after: &[T]) -> Vec<Step> {
    let (n, m) = (before.len(), after.len());
    let mut table = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            table[i][j] = if before[i] == after[j] {
                table[i + 1][j + 1] + 1
            } else {
                table[i + 1][j].max(table[i][j + 1])
            };
        }
    }
    let (mut i, mut j, mut out) = (0, 0, Vec::new());
    while i < n && j < m {
        if before[i] == after[j] {
            out.push(Step::Same(i, j));
            i += 1;
            j += 1;
        } else if table[i + 1][j] >= table[i][j + 1] {
            out.push(Step::Removed(i));
            i += 1;
        } else {
            out.push(Step::Added(j));
            j += 1;
        }
    }
    out.extend((i..n).map(Step::Removed));
    out.extend((j..m).map(Step::Added));
    out
}

/// 足した数と消した数。
pub fn counts<T: PartialEq>(before: &[T], after: &[T]) -> (usize, usize) {
    align(before, after)
        .iter()
        .fold((0, 0), |(added, removed), step| match step {
            Step::Added(_) => (added + 1, removed),
            Step::Removed(_) => (added, removed + 1),
            Step::Same(..) => (added, removed),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserting_one_element_does_not_mark_the_rest() {
        assert_eq!(
            align(&["a", "b", "c"], &["a", "x", "b", "c"]),
            vec![
                Step::Same(0, 0),
                Step::Added(1),
                Step::Same(1, 2),
                Step::Same(2, 3)
            ]
        );
    }

    #[test]
    fn counts_give_added_and_removed() {
        let cases = [
            (vec!["a", "b"], vec!["a", "b"], (0, 0)),
            (vec!["a", "b"], vec!["a"], (0, 1)),
            (vec!["a"], vec!["b"], (1, 1)),
            (vec![], vec!["a", "b"], (2, 0)),
        ];
        let got: Vec<(usize, usize)> = cases.iter().map(|(a, b, _)| counts(a, b)).collect();
        let want: Vec<(usize, usize)> = cases.iter().map(|(_, _, w)| *w).collect();
        assert_eq!(got, want);
    }
}
