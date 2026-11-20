#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod engine;
#[cfg(windows)]
mod process_helper;
use engine::{Engine, EngineHandle, ORIGIN};
use std::{
    env,
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = env::var_os("DANDY_HARNESS_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"D:\Github\Projets\IA\deepseek-harness"));
    let home = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
    std::fs::create_dir_all(&home)?;
    let mut node = Command::new("node");
    if root.join("apps/cli/lib/bin.js").is_file() {
        node.arg("apps/cli/lib/bin.js");
    } else {
        node.args(["--import", "tsx/esm", "apps/cli/src/bin.ts"]);
    }
    // Explicit operator configuration takes precedence over the bundled helper.
    #[cfg(windows)]
    if env::var_os("DANDY_WINDOWS_PROCESS_DLL").is_none() {
        node.env("DANDY_WINDOWS_PROCESS_DLL", process_helper::install(&home)?);
    }
    node.current_dir(root).env("DSH_HOME", home).args([
        "web",
        "--no-open",
        "--host",
        "127.0.0.1",
        "--port",
        "18643",
    ]);
    let running: Arc<Mutex<Option<EngineHandle>>> = Arc::new(Mutex::new(None));
    let setup_engine = running.clone();
    tauri::Builder::default()
        .setup(move |app| {
            let main = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External("about:blank".parse()?),
            )
            .initialization_script(
                r#"
                    if (location.href === 'about:blank') {
                        document.addEventListener('DOMContentLoaded', () => {
                            document.documentElement.lang = 'fr';
                            document.documentElement.style.colorScheme = 'light dark';
                            document.body.style.cssText = 'font: 16px system-ui; padding: 24px';
                            document.body.textContent = 'Démarrage du moteur…';
                        }, { once: true });
                    }
                "#,
            )
            .initialization_script(include_str!("model-menu.js"))
            .on_navigation(|url| url.as_str() == "about:blank" || url.as_str().starts_with(ORIGIN))
            .title("Dandy Harness")
            .inner_size(1280., 840.)
            .build()?;
            // Display the window before spawning: missing Node must be visible too.
            match Engine::spawn(&mut node) {
                Err(error) => show_engine_error(&main, &error),
                Ok(engine) => {
                    *setup_engine.lock().unwrap() = Some(engine.handle.clone());
                    std::thread::spawn(move || {
                        match engine.wait_ready(Duration::from_secs(60)) {
                            Ok(address) => {
                                let navigation = address
                                    .parse()
                                    .map_err(|_| "Adresse locale invalide.".to_string())
                                    .and_then(|url| {
                                        main.navigate(url).map_err(|_| {
                                            "Impossible d’ouvrir l’interface locale.".to_string()
                                        })
                                    });
                                if let Err(error) = navigation {
                                    engine.handle.stop();
                                    show_engine_error(&main, &error);
                                    return;
                                }
                                loop {
                                    if let Some(reason) = engine.handle.exit_reason() {
                                        // Preserve the page and conversation if Node exits.
                                        let _ =
                                            main.set_title(&format!("Dandy Harness — {reason}"));
                                        break;
                                    }
                                    std::thread::sleep(Duration::from_millis(250));
                                }
                            }
                            Err(error) => show_engine_error(&main, &error),
                        }
                    });
                }
            }
            Ok(())
        })
        .build(tauri::generate_context!())?
        .run(move |app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                if let Some(engine) = running.lock().unwrap().take() {
                    engine.stop();
                }
            }
            if let tauri::RunEvent::WindowEvent {
                event: tauri::WindowEvent::Destroyed,
                ..
            } = event
            {
                if app.webview_windows().is_empty() {
                    app.exit(0);
                }
            }
        });
    Ok(())
}

fn show_engine_error(window: &tauri::WebviewWindow, reason: &str) {
    let _ = window.set_title(&format!("Dandy Harness — {reason}"));
    let _ = window.eval("if (location.href === 'about:blank') { const show = () => { document.body.textContent = 'Le moteur n’a pas pu démarrer. Le titre de la fenêtre indique la cause. Vérifie Node et les artefacts du moteur, puis relance l’application.'; }; if (document.body) show(); else document.addEventListener('DOMContentLoaded', show, { once: true }); }");
}
