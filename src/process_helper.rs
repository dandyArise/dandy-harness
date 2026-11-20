use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io,
    path::{Path, PathBuf},
};
const DLL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/dandy_process.dll"));
pub fn install(home: &Path) -> io::Result<PathBuf> {
    install_bytes(home, DLL)
}
fn install_bytes(home: &Path, bytes: &[u8]) -> io::Result<PathBuf> {
    let mut hash = DefaultHasher::new();
    bytes.hash(&mut hash);
    let directory = home.join("native").join(format!("{:016x}", hash.finish()));
    std::fs::create_dir_all(&directory)?;
    let destination = directory.join("dandy_process.dll");
    if destination.exists() {
        if std::fs::read(&destination)? != bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Installed process DLL differs from the bundled file",
            ));
        }
        return Ok(destination);
    }
    let temporary = directory.join(format!("{}.tmp", std::process::id()));
    std::fs::write(&temporary, bytes)?;
    if let Err(error) = std::fs::rename(&temporary, &destination) {
        let _ = std::fs::remove_file(&temporary);
        if std::fs::read(&destination).ok().as_deref() != Some(bytes) {
            return Err(error);
        }
    }
    Ok(destination)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installs_reuses_versions_and_rejects_corruption() {
        let home = std::env::temp_dir().join(format!(
            "dandy-helper-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let first = install_bytes(&home, b"first").unwrap();
        assert_eq!(install_bytes(&home, b"first").unwrap(), first);
        assert_ne!(install_bytes(&home, b"second").unwrap(), first);
        assert_eq!(std::fs::read(&first).unwrap(), b"first");
        std::fs::write(&first, b"corrupt").unwrap();
        assert_eq!(
            install_bytes(&home, b"first").unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        std::fs::remove_dir_all(home).unwrap();
    }
}
