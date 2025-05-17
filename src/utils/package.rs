use std::path::Path;

/// 隐式包名：取 file_path 相对于 project_root 的父目录名称
pub fn derive_package_name(file_path: &Path, project_root: &Path) -> String {
    let rel = file_path
        .strip_prefix(project_root)
        .unwrap_or(file_path);

    let parent = match rel.parent() {
        Some(p) if p.components().count() > 0 => p,
        _ => return "".to_string(),
    };

    let mut parts = Vec::new();
    for comp in parent.components() {
        if let std::path::Component::Normal(os) = comp {
            if let Some(s) = os.to_str() {
                parts.push(s);
            }
        }
    }
    parts.join(".")
}