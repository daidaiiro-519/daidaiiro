//! 起動時の結合。出ていく側のアダプタを application へ渡し、受け口に渡すツールの一覧を組む。
//! CLI と MCP の実行ファイルは、どちらもこれを呼ぶ。

use crate::inbound::tools::Toolbox;
use crate::outbound::catalog::Catalog;
use crate::outbound::fs::FileSystem;
use crate::outbound::jmespath::Jmespath;
use schema_driven_core::application::checks::Checks;
use schema_driven_core::application::instances::Instances;
use schema_driven_core::application::renders::Renders;
use std::sync::Arc;

/// 実行ファイルが使うツールの一覧。
pub type Engine = Toolbox<FileSystem, FileSystem, Jmespath>;

/// ツールの一覧を組む。skill は `--skill` で渡した Skill のフォルダで、スキーマの名前はこの Skill から引く。
pub fn toolbox(skill: Option<&str>) -> Engine {
    let instances = Instances::new(FileSystem, FileSystem, Jmespath);
    let checks = Checks::new(FileSystem, FileSystem, Jmespath);
    // 部品の見た目と操作は、基盤のカタログだけが持つ
    let renders = Renders::new(FileSystem, FileSystem, Jmespath, Arc::new(Catalog));
    match skill {
        Some(dir) => Toolbox::new(
            instances.with_skill(dir),
            checks.with_skill(dir),
            renders.with_skill(dir),
        ),
        None => Toolbox::new(instances, checks, renders),
    }
}
