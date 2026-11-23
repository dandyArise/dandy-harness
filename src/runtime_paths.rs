use std::{
    env, io,
    path::{Path, PathBuf},
};

fn resolve_engine(override_root: Option<PathBuf>, resources: &Path) -> io::Result<PathBuf> {
    let root = override_root.unwrap_or_else(|| resources.join("engine"));
    if root.join("lib/bin.js").is_file()
        || root.join("apps/cli/lib/bin.js").is_file()
        || root
            .join("node_modules/@deepseek-ai/dsh/lib/bin.js")
            .is_file()
    {
        Ok(root)
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound,
            format!("Moteur absent dans {}. Réinstallez Dandy Harness ou définissez DANDY_HARNESS_SOURCE.", root.display())))
    }
}

pub fn engine(resources: &Path) -> io::Result<PathBuf> {
    resolve_engine(
        env::var_os("DANDY_HARNESS_SOURCE").map(PathBuf::from),
        resources,
    )
}

pub fn home() -> io::Result<PathBuf> {
    if let Some(path) = env::var_os("DSH_HOME") {
        return Ok(path.into());
    }
    // Preserve existing installations without copying credentials or session data.
    let legacy = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
    if legacy.is_dir() {
        return Ok(legacy);
    }
    let base = if cfg!(windows) {
        env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        env::var_os("HOME").map(|p| PathBuf::from(p).join("Library/Application Support"))
    } else {
        env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")))
    };
    base.map(|path| path.join("Dandy Harness/data"))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Répertoire utilisateur absent ; définir DSH_HOME",
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packaged_engine_is_independent_of_checkout() {
        let temp = env::temp_dir().join(format!("dandy-runtime-{}", std::process::id()));
        let engine = temp.join("engine");
        std::fs::create_dir_all(engine.join("lib")).unwrap();
        std::fs::write(engine.join("lib/bin.js"), "").unwrap();
        assert_eq!(resolve_engine(None, &temp).unwrap(), engine);
        std::fs::remove_file(engine.join("lib/bin.js")).unwrap();
        std::fs::create_dir_all(engine.join("node_modules/@deepseek-ai/dsh/lib")).unwrap();
        std::fs::write(engine.join("node_modules/@deepseek-ai/dsh/lib/bin.js"), "").unwrap();
        assert_eq!(resolve_engine(None, &temp).unwrap(), engine);
        assert!(resolve_engine(Some(temp.join("missing")), &temp).is_err());
        std::fs::remove_dir_all(temp).unwrap();
    }
}
