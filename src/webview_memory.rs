//! Release expendable WebView caches while minimized, without suspending scripts.
use tauri::WebviewWindow;
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2_19, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL,
};
use windows_core::Interface;

pub fn update(window: &WebviewWindow, minimized: bool, probe: crate::startup_probe::Probe) {
    if let Err(error) = window.with_webview(move |webview| {
        // SAFETY: with_webview executes on the owning UI thread; COM interfaces
        // stay inside this callback and are released before returning.
        let result = unsafe {
            webview.controller().CoreWebView2().and_then(|core| {
                core.cast::<ICoreWebView2_19>().and_then(|core| {
                    core.SetMemoryUsageTargetLevel(COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL(
                        i32::from(minimized),
                    ))
                })
            })
        };
        if let Err(error) = result {
            // An older runtime may lack this optional API. Keep the chat alive.
            eprintln!("WebView memory target unavailable: {error}");
        } else {
            probe.mark(if minimized {
                "memory_low"
            } else {
                "memory_normal"
            });
        }
    }) {
        eprintln!("Cannot dispatch WebView memory target: {error}");
    }
}
