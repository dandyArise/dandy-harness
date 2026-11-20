fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        use std::{env, path::PathBuf, process::Command};
        println!("cargo:rerun-if-env-changed=DANDY_HARNESS_SOURCE");
        let root = env::var_os("DANDY_HARNESS_SOURCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"D:\Github\Projets\IA\deepseek-harness"));
        let source = root.join("native/system/rust-process");
        for file in ["Cargo.toml", "Cargo.lock", "src"] {
            println!("cargo:rerun-if-changed={}", source.join(file).display());
        }
        let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
        let target = env::var("TARGET").unwrap();
        // Separate target directory avoids the outer Cargo build lock.
        let status = Command::new(env::var_os("CARGO").unwrap())
            .args([
                "build",
                "--locked",
                "--release",
                "--target",
                &target,
                "--manifest-path",
            ])
            .arg(source.join("Cargo.toml"))
            .arg("--target-dir")
            .arg(out.join("process-build"))
            .status()
            .expect("Cannot compile Windows process helper");
        assert!(status.success(), "Windows process helper build failed");
        std::fs::copy(
            out.join("process-build")
                .join(target)
                .join("release/dandy_process.dll"),
            out.join("dandy_process.dll"),
        )
        .expect("Cannot embed Windows process helper");
    }
    tauri_build::build()
}
