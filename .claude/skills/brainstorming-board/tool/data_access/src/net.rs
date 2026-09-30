// SPDX-License-Identifier: MIT
//! 通信の入出力。**この Skill に固有の入出力である** ── ブレストボードを配るサーバーが使う。
//!
//! 待ち受け ・ 1行の受信 ・ 本文の受信 ・ 送信だけを持つ。
//! **判定を置かない** ── どの要求にどう応えるか（道の解決 ・ 本文の上限 ・ 応答の組み立て）は、
//! 業務ロジック層が決める。

use std::io::{self, BufRead as _, BufReader, Read as _, Write as _};
use std::net::{TcpListener, TcpStream};

/// 待ち受け。
pub struct Listener {
    inner: TcpListener,
}

impl Listener {
    /// 待ち受けを始める。
    ///
    /// # Errors
    ///
    /// 待ち受けられないときに返す。
    pub fn bind(host: &str, port: u16) -> io::Result<Self> {
        TcpListener::bind((host, port)).map(|inner| Self { inner })
    }

    /// 届いた接続を1件ずつ返す。**止められるまで終わらない。**
    ///
    /// 受け取れなかった接続と、読み書きの口を分けられなかった接続は飛ばす。
    pub fn incoming(&self) -> impl Iterator<Item = Connection> + '_ {
        self.inner
            .incoming()
            .filter_map(Result::ok)
            .filter_map(|stream| Connection::new(stream).ok())
    }
}

/// 1件の接続。受ける側と送る側を持つ。
pub struct Connection {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
}

impl Connection {
    fn new(writer: TcpStream) -> io::Result<Self> {
        let reader = BufReader::new(writer.try_clone()?);
        Ok(Self { reader, writer })
    }

    /// 1行を受ける（行末を含む）。**読めないときは None を返す。**
    pub fn read_line(&mut self) -> Option<String> {
        let mut line = String::new();
        self.reader.read_line(&mut line).ok().map(|_| line)
    }

    /// ちょうど `length` バイトを受ける。**読めないときは None を返す。**
    pub fn read_exact(&mut self, length: usize) -> Option<Vec<u8>> {
        let mut raw = vec![0u8; length];
        self.reader.read_exact(&mut raw).ok().map(|()| raw)
    }

    /// 並べたものを順に送り、送り切る。**送れなくても誤りを返さない** ── 相手が
    /// 先に閉じたときに、配る側を止めないためである。
    pub fn send(&mut self, pieces: &[&[u8]]) {
        for piece in pieces {
            let _ = self.writer.write_all(piece);
        }
        let _ = self.writer.flush();
    }
}
