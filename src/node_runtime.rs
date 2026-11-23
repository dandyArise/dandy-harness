use std::{env, ffi::OsString, path::Path};

fn resolve(override_path: Option<OsString>, launcher: Option<&Path>) -> OsString {
    if let Some(path) = override_path.filter(|value| !value.is_empty()) {
        return path;
    }
    if let Some(directory) = launcher.and_then(Path::parent) {
        let bundled =
            directory
                .join("runtime")
                .join(if cfg!(windows) { "node.exe" } else { "node" });
        if bundled.is_file() {
            return bundled.into_os_string();
        }
    }
    OsString::from("node")
}

pub fn executable() -> OsString {
    resolve(
        env::var_os("DANDY_NODE_EXECUTABLE"),
        env::current_exe().ok().as_deref(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_runtime_takes_precedence() {
        assert_eq!(
            resolve(Some("custom-node".into()), None),
            OsString::from("custom-node")
        );
    }

    #[test]
    fn missing_bundle_uses_path() {
        assert_eq!(
            resolve(None, Some(Path::new("missing-launcher"))),
            OsString::from("node")
        );
    }
}
