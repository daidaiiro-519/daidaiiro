pub struct Order { pub lines: Vec<String> }
impl Order { pub fn confirm(&self) -> Result<(), ()> { if self.lines.is_empty() { Err(()) } else { Ok(()) } } }
#[cfg(test)]
mod tests {
    use super::*;
    fn scenario(id: &str) {
        if let Ok(p) = std::env::var("SCENARIO_TRACE") {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new().append(true).create(true).open(p).unwrap();
            writeln!(f, "{id}").unwrap();
        }
    }
    #[test] fn 確定する() {
        for (lines, id, ok) in [(vec![], "SC-01J7Q4M", false), (vec!["りんご".to_string()], "SC-01J7Q4N", true)] {
            scenario(id);
            assert_eq!(Order { lines }.confirm().is_ok(), ok);
        }
    }
}
