//! Opt-in, local timing only. Never records URLs, tokens, messages or logs.
use std::{path::PathBuf, sync::Arc, time::Instant};

#[derive(Clone)]
pub struct Probe(Arc<(Instant, Option<PathBuf>)>);
impl Probe {
    pub fn new() -> Self {
        Self(Arc::new((
            Instant::now(),
            std::env::var_os("DANDY_STARTUP_PROBE").map(PathBuf::from),
        )))
    }
    pub fn enabled(&self) -> bool {
        self.0 .1.is_some()
    }
    pub fn mark(&self, name: &str) {
        use std::io::Write;
        if let Some(path) = &self.0 .1 {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = writeln!(
                    file,
                    "{name}={:.3}",
                    self.0 .0.elapsed().as_secs_f64() * 1000.
                );
            }
        }
    }
}

pub const SCRIPT: &str = r#"
if (location.origin === 'http://127.0.0.1:18643') {
  const started = performance.now();
  const timer = setInterval(() => {
    const editor = document.querySelector('[data-composer-input][contenteditable="true"]');
    if (editor && editor.getBoundingClientRect().width > 0 && editor.getAttribute('aria-disabled') !== 'true') {
      requestAnimationFrame(() => requestAnimationFrame(() => {
        document.title = 'DANDY-BENCH-CHAT-READY';
      }));
    }
    if (performance.now() - started > 60000) clearInterval(timer);
  }, 50);
}
"#;
