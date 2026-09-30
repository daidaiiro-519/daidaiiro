// SPDX-License-Identifier: MIT
//! 渡した音声の長さを出す。**突き合わせるための入口である。**

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        println!("{} {}", sds_business_logic::mp3::duration_ms(&data), path);
    }
}
