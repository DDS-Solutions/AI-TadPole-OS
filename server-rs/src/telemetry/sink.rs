//! @docs ARCHITECTURE:TelemetryBridge
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / sink
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use chrono::Utc;
use serde_json::Value;
use std::io::Write;
use std::path::{Path, PathBuf};
use tokio::sync::broadcast;
use tracing::{info, warn};

/// Background service that drains `TELEMETRY_TX` and appends each event as a
/// newline-delimited JSON record to a date-stamped log file.
///
/// Files are written to `<base_dir>/data/logs/telemetry-YYYY-MM-DD.jsonl`.
/// A new file is opened whenever the UTC date changes (daily rotation).
pub struct TelemetryLogSink {
    base_dir: PathBuf,
}

impl TelemetryLogSink {
    /// Creates a new sink that writes logs under `<base_dir>/data/logs/`.
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    /// Primary run loop with optional graceful shutdown watch.
    pub async fn run_with_shutdown(
        self,
        mut rx: broadcast::Receiver<Value>,
        mut shutdown_rx: Option<tokio::sync::watch::Receiver<bool>>,
    ) {
        let log_dir = self.base_dir.join("data").join("logs");

        if let Err(e) = std::fs::create_dir_all(&log_dir) {
            warn!(
                "⚠️ [LogSink] Could not create log directory {:?}: {}",
                log_dir, e
            );
            return;
        }

        info!(
            "📝 [LogSink] Telemetry sink started. Writing to {:?}",
            log_dir
        );

        // Prune logs older than 7 days on startup
        prune_old_logs(&log_dir, 7);

        let mut current_date = Utc::now().format("%Y-%m-%d").to_string();
        let mut file = match open_log_file(&log_dir, &current_date) {
            Ok(f) => std::io::BufWriter::new(f),
            Err(e) => {
                warn!("⚠️ [LogSink] Failed to open initial log file: {}", e);
                return;
            }
        };

        loop {
            tokio::select! {
                result = async {
                    if let Some(ref mut s_rx) = shutdown_rx {
                        s_rx.changed().await
                    } else {
                        std::future::pending().await
                    }
                } => {
                    if result.is_err() || shutdown_rx.as_ref().map(|s| *s.borrow()).unwrap_or(false) {
                        info!("📝 [LogSink] Shutdown signal received. Flushing logs.");
                        let _ = file.flush();
                        break;
                    }
                }
                event_result = rx.recv() => {
                    match event_result {
                        Ok(event) => {
                            // Check for date rollover (daily log rotation)
                            let today = Utc::now().format("%Y-%m-%d").to_string();
                            if today != current_date {
                                info!(
                                    "📝 [LogSink] Rotating log file to telemetry-{}.jsonl",
                                    today
                                );
                                match open_log_file(&log_dir, &today) {
                                    Ok(new_file) => {
                                        let _ = file.flush();
                                        file = std::io::BufWriter::new(new_file);
                                        current_date = today;
                                        prune_old_logs(&log_dir, 7);
                                    }
                                    Err(e) => {
                                        warn!("⚠️ [LogSink] Failed to open rotated log file: {}", e);
                                    }
                                }
                            }

                            // Serialize event as a single JSONL line
                            if let Ok(line) = serde_json::to_string(&event) {
                                if let Err(e) = writeln!(file, "{}", line) {
                                    warn!("⚠️ [LogSink] Write error: {}", e);
                                }
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            warn!("⚠️ [LogSink] Sink lagged by {} events — increase channel capacity if frequent.", n);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            info!("📝 [LogSink] Broadcast channel closed. Sink shutting down.");
                            let _ = file.flush();
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Primary run loop. Drains the broadcast receiver and writes events to disk.
    pub async fn run(self, rx: broadcast::Receiver<Value>) {
        self.run_with_shutdown(rx, None).await;
    }
}

fn prune_old_logs(dir: &Path, max_days: i64) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        let cutoff = Utc::now() - chrono::Duration::days(max_days);
        let cutoff_str = cutoff.format("%Y-%m-%d").to_string();

        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with("telemetry-") && file_name.ends_with(".jsonl") {
                    let date_part = file_name
                        .trim_start_matches("telemetry-")
                        .trim_end_matches(".jsonl");
                    if date_part < cutoff_str.as_str() {
                        let _ = std::fs::remove_file(&path);
                        info!("🧹 [LogSink] Auto-pruned old telemetry log: {}", file_name);
                    }
                }
            }
        }
    }
}

fn open_log_file(dir: &Path, date: &str) -> std::io::Result<std::fs::File> {
    let path = dir.join(format!("telemetry-{}.jsonl", date));
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;
    use tempfile::TempDir;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn test_sink_writes_events_to_jsonl() {
        let tmp = TempDir::new().expect("tempdir");
        let (tx, rx) = broadcast::channel::<Value>(16);

        let base_dir = tmp.path().to_path_buf();
        let sink = TelemetryLogSink::new(base_dir.clone());

        // Spawn sink
        let handle = tokio::spawn(sink.run(rx));

        // Send two events then drop sender to trigger Closed
        tx.send(serde_json::json!({"type": "trace:span", "span": {"id": "1"}}))
            .unwrap();
        tx.send(serde_json::json!({"type": "telemetry:metrics", "metrics": {}}))
            .unwrap();
        drop(tx);

        // Wait for sink to process and exit cleanly
        let _ = handle.await;

        // Verify log file exists and has 2 lines
        let log_dir = base_dir.join("data").join("logs");
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let log_path = log_dir.join(format!("telemetry-{}.jsonl", today));

        assert!(log_path.exists(), "log file should exist");

        let file = std::fs::File::open(&log_path).unwrap();
        let lines: Vec<String> = std::io::BufReader::new(file)
            .lines()
            .map_while(Result::ok)
            .filter(|l| !l.is_empty())
            .collect();
        assert_eq!(lines.len(), 2, "should have written 2 JSONL lines");

        // Verify first line is valid JSON with expected type
        let parsed: Value = serde_json::from_str(&lines[0]).unwrap();
        assert_eq!(parsed["type"], "trace:span");
    }

    #[tokio::test]
    async fn test_sink_handles_lag_gracefully() {
        let tmp = TempDir::new().expect("tempdir");
        // Tiny channel capacity to force lag
        let (tx, rx) = broadcast::channel::<Value>(2);

        let base_dir = tmp.path().to_path_buf();
        let sink = TelemetryLogSink::new(base_dir.clone());

        // Flood the channel before sink starts consuming
        for i in 0..10 {
            let _ = tx.send(serde_json::json!({"i": i}));
        }

        let handle = tokio::spawn(sink.run(rx));
        drop(tx);

        // Should complete without panic despite lag
        let result = tokio::time::timeout(std::time::Duration::from_millis(1000), handle).await;
        assert!(result.is_ok(), "sink should not hang on lag");
    }
}
