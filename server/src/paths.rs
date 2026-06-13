use std::path::{Path, PathBuf};

/// リポジトリルート（`server/` の親ディレクトリ）。
/// `cargo run` のカレントディレクトリがルートでも `server/` でも同じ `data/` を指す。
pub(crate) fn project_root() -> PathBuf {
    match std::env::var(crate::config::ENV_PROJECT_ROOT) {
        Ok(v) if !v.is_empty() => PathBuf::from(v),
        _ => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."),
    }
}

/// 相対パスは `project_root` 基準。絶対パスはそのまま使う。
pub(crate) fn resolve_project_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root().join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_root_contains_data_dir_relative() {
        let root = project_root();
        let data = resolve_project_path("data");
        assert_eq!(data, root.join("data"));
    }
}
