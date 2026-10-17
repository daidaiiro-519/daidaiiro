//! port の Clock の実装。時刻は UTC で書き、UUID は OS の乱数から作る。

use brainstorming_core::ports::Clock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub struct RealClock {
    started: Instant,
}

impl Default for RealClock {
    fn default() -> Self {
        Self {
            started: Instant::now(),
        }
    }
}

/// 1970-01-01 からの日数を、年 ・ 月 ・ 日にする（Howard Hinnant の civil_from_days）。
pub fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(month <= 2), month, day)
}

/// UNIX 時刻（秒）を RFC 3339 の UTC にする。
pub fn rfc3339(seconds: i64) -> String {
    let (year, month, day) = civil(seconds.div_euclid(86_400));
    let rest = seconds.rem_euclid(86_400);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

/// 16バイトの乱数を UUID v4 の形にする。
pub fn uuid(mut bytes: [u8; 16]) -> String {
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// OS の乱数。
pub fn random<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    getrandom::fill(&mut bytes).unwrap_or(());
    bytes
}

fn seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

impl Clock for RealClock {
    fn now(&self) -> String {
        rfc3339(seconds())
    }
    fn today(&self) -> String {
        rfc3339(seconds())[..10].to_owned()
    }
    fn elapsed_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }
    fn uuid(&self) -> String {
        uuid(random())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_instants_format_as_rfc3339() {
        let cases = [
            (0, "1970-01-01T00:00:00Z"),
            (951_782_400, "2000-02-29T00:00:00Z"),
            (1_792_166_117, "2026-10-16T15:55:17Z"),
        ];
        let got: Vec<String> = cases.iter().map(|(s, _)| rfc3339(*s)).collect();
        let want: Vec<String> = cases.iter().map(|(_, w)| (*w).to_owned()).collect();
        assert_eq!(got, want);
    }

    #[test]
    fn uuid_sets_the_version_and_variant() {
        let made = uuid([0xff; 16]);
        assert_eq!(made, "ffffffff-ffff-4fff-bfff-ffffffffffff");
        assert_eq!(made.len(), 36);
    }
}
