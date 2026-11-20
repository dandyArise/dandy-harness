#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod engine;
#[cfg(windows)]
mod process_helper;
mod startup_probe;
#[cfg(windows)]
mod webview_memory;
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
    let probe = startup_probe::Probe::new();
    probe.mark("entry");
    let root = env::var_os("DANDY_HARNESS_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"D:\Github\Projets\IA\deepseek-harness"));
    let home = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
    std::fs::create_dir_all(&home)?;
    let mut node = Command::new("node");
    if env::var_os("DANDY_NODE_PROFILE").is_some() {
        let script = home.join("startup-profile.mjs");
        std::fs::write(&script, include_str!("startup-profile.mjs"))?;
        node.arg("--import")
            .arg(tauri::Url::from_file_path(script).unwrap().as_str());
    }
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
            probe.mark("setup");
            // Node boot and WebView creation are independent. Keep the owned
            // handle immediately so a window-construction error also cleans up.
            // Serial mode exists only with the diagnostic probe, for paired benchmarks.
            let serial = probe.enabled() && env::var("DANDY_STARTUP_SERIAL").as_deref() == Ok("1");
            let start_engine = |node: &mut Command| {
                let engine = Engine::spawn(node);
                if let Ok(engine) = &engine {
                    probe.mark("spawn");
                    *setup_engine.lock().unwrap() = Some(engine.handle.clone());
                }
                engine
            };
            let engine = if serial {
                None
            } else {
                Some(start_engine(&mut node))
            };
            let title_probe = probe.clone();
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
            .initialization_script(if probe.enabled() {
                startup_probe::SCRIPT
            } else {
                ""
            })
            .on_navigation(|url| url.as_str() == "about:blank" || url.as_str().starts_with(ORIGIN))
            .on_document_title_changed(move |window, title| {
                if title_probe.enabled() && title == "DANDY-BENCH-CHAT-READY" {
                    title_probe.mark("chat_ready");
                    let _ = window.set_title(&title);
                }
            })
            .title("Dandy Harness")
            .inner_size(1280., 840.)
            .build()?;
            probe.mark("window");
            #[cfg(windows)]
            if env::var("DANDY_WEBVIEW_MEMORY").as_deref() != Ok("normal") {
                let handle = app.handle().clone();
                let memory_probe = probe.clone();
                let last_minimized = std::sync::atomic::AtomicBool::new(false);
                main.on_window_event(move |event| {
                    if matches!(
                        event,
                        tauri::WindowEvent::Resized(_) | tauri::WindowEvent::Focused(_)
                    ) {
                        if let Some(window) = handle.get_webview_window("main") {
                            if let Ok(minimized) = window.is_minimized() {
                                if last_minimized
                                    .swap(minimized, std::sync::atomic::Ordering::Relaxed)
                                    != minimized
                                {
                                    webview_memory::update(
                                        &window,
                                        minimized,
                                        memory_probe.clone(),
                                    );
                                }
                            }
                        }
                    }
                });
            }
            // Even a spawn failure is displayed in the newly created window.
            match engine.unwrap_or_else(|| start_engine(&mut node)) {
                Err(error) => show_engine_error(&main, &error),
                Ok(engine) => {
                    std::thread::spawn(move || {
                        match engine.wait_ready(Duration::from_secs(60)) {
                            Ok(address) => {
                                probe.mark("engine_ready");
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
