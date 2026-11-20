//! Rust-owned lifecycle for the existing, profile-based Node engine.
use std::{
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::{
        mpsc::{self, Receiver, RecvTimeoutError},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

pub const ORIGIN: &str = "http://127.0.0.1:18643/";
const MAX_LINE: usize = 16 * 1024;
const POLL: Duration = Duration::from_millis(50);

#[cfg(windows)]
#[path = "process_job.rs"]
mod process_job;

struct OwnedChild(
    Mutex<Option<Child>>,
    #[cfg(windows)] Mutex<Option<process_job::ProcessJob>>,
);
impl OwnedChild {
    fn stop(&self) {
        #[cfg(windows)]
        if let Some(job) = self.1.lock().unwrap().take() {
            job.stop();
        }
        if let Some(mut child) = self.0.lock().unwrap().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        self.stop();
    }
}

#[derive(Clone)]
pub struct EngineHandle(Arc<OwnedChild>);
impl EngineHandle {
    pub fn stop(&self) {
        self.0.stop();
    }

    pub fn exit_reason(&self) -> Option<String> {
        let mut guard = self.0 .0.lock().unwrap();
        match guard.as_mut() {
            None => Some("Le moteur a été arrêté.".into()),
            Some(child) => match child.try_wait() {
                Ok(Some(status)) => Some(format!("Le moteur s’est arrêté ({status}).")),
                Ok(None) => None,
                Err(error) => Some(format!("Impossible de surveiller le moteur : {error}")),
            },
        }
    }
}

pub struct Engine {
    pub handle: EngineHandle,
    ready: Receiver<String>,
}

impl Engine {
    pub fn spawn(command: &mut Command) -> Result<Self, String> {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // CREATE_NO_WINDOW | CREATE_SUSPENDED. Attach the cleanup job
            // before any engine code or descendant can run.
            command.creation_flags(0x08000004);
        }
        let mut child = command.stdout(Stdio::piped()).spawn().map_err(|error| {
            format!("Impossible de lancer Node. Vérifie son installation et le PATH : {error}")
        })?;
        let stdout = child.stdout.take().expect("stdout was piped");
        #[cfg(windows)]
        let job = match process_job::ProcessJob::attach(&child).and_then(|job| {
            process_job::ProcessJob::resume(&child)?;
            Ok(job)
        }) {
            Ok(job) => job,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "Impossible de superviser les processus du moteur : {error}"
                ));
            }
        };
        let handle = EngineHandle(Arc::new(OwnedChild(
            Mutex::new(Some(child)),
            #[cfg(windows)]
            Mutex::new(Some(job)),
        )));
        let (sender, ready) = mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let mut sent = false;
            read_bounded_lines(BufReader::new(stdout), |line| {
                if !sent {
                    if let Some(address) = ready_address(line) {
                        let _ = sender.try_send(address.to_owned());
                        sent = true;
                    }
                }
                // Always drain stdout, including after readiness; never retain logs/tokens.
            });
        });
        Ok(Self { handle, ready })
    }

    pub fn wait_ready(&self, timeout: Duration) -> Result<String, String> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(reason) = self.handle.exit_reason() {
                return Err(reason);
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                self.handle.stop();
                return Err(format!(
                    "Le moteur n’a pas démarré dans le délai de {} secondes.",
                    timeout.as_secs()
                ));
            }
            match self.ready.recv_timeout(remaining.min(POLL)) {
                Ok(address) => return Ok(address),
                Err(RecvTimeoutError::Timeout) => (),
                Err(RecvTimeoutError::Disconnected) => {
                    let reason = self
                        .handle
                        .exit_reason()
                        .unwrap_or_else(|| "Le moteur a fermé sa sortie avant d’être prêt.".into());
                    self.handle.stop();
                    return Err(reason);
                }
            }
        }
    }
}

fn ready_address(line: &[u8]) -> Option<&str> {
    let line = std::str::from_utf8(line).ok()?.trim_end_matches('\r');
    let address = line.strip_prefix("dsh web: ")?;
    // The trailing slash also rejects lookalike hosts, ports and userinfo URLs.
    address.starts_with(ORIGIN).then_some(address)
}

fn read_bounded_lines(mut reader: impl BufRead, mut visit: impl FnMut(&[u8])) {
    let mut line = Vec::with_capacity(MAX_LINE);
    let mut oversized = false;
    while let Ok(bytes) = reader.fill_buf() {
        if bytes.is_empty() {
            break;
        }
        let newline = bytes.iter().position(|byte| *byte == b'\n');
        let count = newline.map_or(bytes.len(), |index| index + 1);
        let content = &bytes[..newline.unwrap_or(count)];
        if !oversized {
            if content.len() > MAX_LINE - line.len() {
                oversized = true;
                line.clear();
            } else {
                line.extend_from_slice(content);
            }
        }
        if newline.is_some() {
            if !oversized {
                visit(&line);
            }
            line.clear();
            oversized = false;
        }
        reader.consume(count);
    }
    if !oversized && !line.is_empty() {
        visit(&line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn node(script: &str) -> Engine {
        Engine::spawn(Command::new("node").args(["-e", script])).expect("Node must be on PATH")
    }

    #[test]
    fn only_accepts_own_origin() {
        assert_eq!(
            ready_address(b"dsh web: http://127.0.0.1:18643/?token=secret\r"),
            Some("http://127.0.0.1:18643/?token=secret")
        );
        for address in [
            "http://127.0.0.1:186430/",
            "http://127.0.0.1:18643@evil/",
            "https://evil/",
            "http://127.0.0.1:18643.evil/",
        ] {
            assert!(ready_address(format!("dsh web: {address}").as_bytes()).is_none());
        }
        assert!(ready_address(b"dsh web: \xff").is_none());
    }

    #[test]
    fn oversize_output_is_discarded_and_next_line_survives() {
        let mut input = vec![b'x'; MAX_LINE * 100];
        input.extend_from_slice(b"\nsmall\r\nlast");
        let mut lines = Vec::new();
        read_bounded_lines(BufReader::with_capacity(13, Cursor::new(input)), |line| {
            lines.push(line.to_vec())
        });
        assert_eq!(lines, [b"small\r".to_vec(), b"last".to_vec()]);
    }

    #[test]
    fn ready_and_stop_real_process() {
        let engine =
            node("console.log('dsh web: http://127.0.0.1:18643/'); setInterval(()=>{},1000)");
        assert_eq!(engine.wait_ready(Duration::from_secs(5)).unwrap(), ORIGIN);
        engine.handle.stop();
        engine.handle.stop();
        assert!(engine.handle.exit_reason().is_some());
    }

    #[cfg(windows)]
    #[test]
    fn stop_reaps_descendant_processes() {
        use windows_sys::Win32::{
            Foundation::CloseHandle,
            System::Threading::{OpenProcess, WaitForSingleObject},
        };
        let engine = node("const c=require('node:child_process').spawn(process.execPath,['-e','setInterval(()=>{},1000)'],{stdio:'ignore'}); console.log('dsh web: http://127.0.0.1:18643/?child='+c.pid); setInterval(()=>{},1000)");
        let address = engine.wait_ready(Duration::from_secs(5)).unwrap();
        let pid: u32 = address.split("child=").nth(1).unwrap().parse().unwrap();
        // SYNCHRONIZE access only; no global process discovery or termination.
        let descendant = unsafe { OpenProcess(0x00100000, 0, pid) };
        assert!(!descendant.is_null());
        engine.handle.stop();
        let exited = unsafe { WaitForSingleObject(descendant, 5000) };
        unsafe {
            CloseHandle(descendant);
        }
        assert_eq!(exited, 0, "descendant outlived its engine");
    }

    #[test]
    fn stalled_start_is_bounded_and_process_reaped() {
        let engine = node("setInterval(()=>{},1000)");
        let start = Instant::now();
        assert!(engine
            .wait_ready(Duration::from_millis(150))
            .unwrap_err()
            .contains("délai"));
        assert!(start.elapsed() < Duration::from_secs(3));
        assert!(engine.handle.0 .0.lock().unwrap().is_none());
    }

    #[test]
    fn exited_process_does_not_wait_for_timeout() {
        let engine = node("process.exit(7)");
        let start = Instant::now();
        assert!(engine.wait_ready(Duration::from_secs(10)).is_err());
        assert!(start.elapsed() < Duration::from_secs(3));
    }

    #[test]
    fn missing_executable_is_reported() {
        assert!(Engine::spawn(&mut Command::new("dandy-nonexistent-engine-42")).is_err());
    }

    /// Reproducible stress comparison of the previous lines() reader and the bounded reader.
    /// Not a full startup/RSS benchmark: reports the retained line allocation explicitly.
    #[test]
    #[ignore]
    fn compare_log_readers() {
        let mut input = vec![b'x'; 8 * 1024 * 1024];
        input.extend_from_slice(b"\ndsh web: http://127.0.0.1:18643/\n");
        let mut old_times = Vec::new();
        let mut new_times = Vec::new();
        let mut old_capacity = 0;
        for trial in 0..24 {
            for variant in [trial % 2, 1 - trial % 2] {
                let start = Instant::now();
                let mut ready = false;
                if variant == 0 {
                    for line in BufReader::new(Cursor::new(&input)).lines() {
                        let line = line.unwrap();
                        old_capacity = old_capacity.max(line.capacity());
                        ready |= ready_address(line.as_bytes()).is_some();
                    }
                } else {
                    read_bounded_lines(BufReader::new(Cursor::new(&input)), |line| {
                        ready |= ready_address(line).is_some();
                    });
                }
                assert!(ready);
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                if trial >= 4 {
                    if variant == 0 {
                        old_times.push(elapsed);
                    } else {
                        new_times.push(elapsed);
                    }
                }
            }
        }
        old_times.sort_by(f64::total_cmp);
        new_times.sort_by(f64::total_cmp);
        println!("{{\"scenario\":\"8MiB line then readiness\",\"trials\":20,\"old_median_ms\":{},\"new_median_ms\":{},\"old_line_capacity_bytes\":{},\"new_line_capacity_bytes\":{}}}", old_times[10], new_times[10], old_capacity, MAX_LINE);
    }
}
