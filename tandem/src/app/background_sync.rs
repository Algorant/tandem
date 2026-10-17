//! Background board sync while the TUI is open.
//!
//! A worker thread publishes local changes shortly after they are saved and
//! fetches incoming changes periodically. Workspace retirement is requested
//! immediately; joining and its final publish run on a tracked retiring job.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::project::sync::{self, Mode, Outcome, Report};
use crate::project::TandemProject;

const PUBLISH_INTERVAL: Duration = Duration::from_secs(2);
const REFRESH_INTERVAL: Duration = Duration::from_secs(30);
const RETRY_INTERVAL: Duration = Duration::from_secs(30);

pub(crate) struct BackgroundSync {
    stop: Arc<AtomicBool>,
    note: Arc<Mutex<Option<String>>>,
    handle: Option<JoinHandle<()>>,
    workspace: TandemProject,
    alias: String,
}

impl BackgroundSync {
    /// Starts syncing when the board is Git-backed; otherwise does nothing.
    pub(crate) fn start(workspace: &TandemProject, alias: impl Into<String>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let note = Arc::new(Mutex::new(None));
        let alias = alias.into();
        let handle = workspace.git().is_some().then(|| {
            let (stop, note, workspace, alias) =
                (stop.clone(), note.clone(), workspace.clone(), alias.clone());
            thread::spawn(move || run(&workspace, &alias, &stop, &note))
        });
        Self {
            stop,
            note,
            handle,
            workspace: workspace.clone(),
            alias,
        }
    }

    pub(crate) fn take_note(&self) -> Option<String> {
        self.note.lock().ok()?.take()
    }

    /// Stop now, then reap the periodic worker and final-publish off the UI loop.
    pub(crate) fn retire(mut self) -> JoinHandle<Option<String>> {
        self.stop.store(true, Ordering::Release);
        let handle = self.handle.take();
        let workspace = self.workspace;
        let alias = self.alias;
        thread::spawn(move || {
            if let Some(handle) = handle {
                if handle.join().is_err() {
                    return Some(format!(
                        "Workspace `{alias}` sync worker panicked while stopping"
                    ));
                }
            }
            workspace.git()?;
            match sync::sync(&workspace, Mode::Publish) {
                Ok(report) => describe(&report).map(|note| format!("Workspace `{alias}`: {note}")),
                Err(error) => Some(format!(
                    "Workspace `{alias}` final publish failed: {}",
                    error.message
                )),
            }
        })
    }

    /// Stops and final-publishes the active workspace during shutdown.
    pub(crate) fn finish(self) -> Option<String> {
        let alias = self.alias.clone();
        self.retire()
            .join()
            .unwrap_or_else(|_| Some(format!("Workspace `{alias}` shutdown worker panicked")))
    }
}

fn run(workspace: &TandemProject, alias: &str, stop: &AtomicBool, note: &Mutex<Option<String>>) {
    let mut last_refresh: Option<Instant> = None;
    let mut last_message: Option<String> = None;
    let mut last_published: Option<sync::BoardFingerprint> = None;
    let mut last_publish = Instant::now();
    let mut retry_after: Option<Instant> = None;
    while !stop.load(Ordering::Acquire) {
        let waiting = retry_after.is_some_and(|at| Instant::now() < at);
        let refresh_due =
            !waiting && last_refresh.is_none_or(|at| at.elapsed() >= REFRESH_INTERVAL);
        let publish_due = !waiting && last_publish.elapsed() >= PUBLISH_INTERVAL;
        if refresh_due || publish_due {
            if crate::app::project::historical(workspace).is_none() {
                let mode = if refresh_due {
                    Mode::Refresh
                } else {
                    Mode::Publish
                };
                let (before, before_error) = match sync::board_fingerprint(workspace.data_dir()) {
                    Ok(fingerprint) => (Some(fingerprint), None),
                    Err(error) if mode == Mode::Publish => {
                        let message = Some(format!("Sync fingerprint failed: {error}"));
                        retry_after = Some(Instant::now() + RETRY_INTERVAL);
                        if message != last_message {
                            if let Ok(mut slot) = note.lock() {
                                *slot = message
                                    .as_ref()
                                    .map(|message| format!("Workspace `{alias}`: {message}"));
                            }
                        }
                        last_message = message;
                        last_publish = Instant::now();
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                    Err(error) => (None, Some(format!("Sync fingerprint failed: {error}"))),
                };
                if mode == Mode::Publish && before.as_ref() == last_published.as_ref() {
                    last_publish = Instant::now();
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
                let message = match sync::sync(workspace, mode) {
                    Ok(report) => {
                        let successful =
                            matches!(report.outcome, Outcome::Synced | Outcome::LocalOnly);
                        retry_after = matches!(report.outcome, Outcome::Pending(_))
                            .then(|| Instant::now() + RETRY_INTERVAL);
                        let after =
                            successful.then(|| sync::board_fingerprint(workspace.data_dir()));
                        let fingerprint_error = match (before, after) {
                            (Some(before), Some(Ok(after))) if before == after => {
                                last_published = Some(before);
                                None
                            }
                            (Some(_), Some(Ok(_))) | (None, Some(Ok(_))) => {
                                last_published = None;
                                None
                            }
                            (_, Some(Err(error))) => {
                                last_published = None;
                                Some(format!("Sync fingerprint failed: {error}"))
                            }
                            (_, None) => None,
                        };
                        describe(&report).or(fingerprint_error).or(before_error)
                    }
                    Err(error) => {
                        retry_after = Some(Instant::now() + RETRY_INTERVAL);
                        Some(format!("Sync failed: {}", error.message))
                    }
                };
                if message.is_some() && message != last_message {
                    if let Ok(mut slot) = note.lock() {
                        *slot = message
                            .as_ref()
                            .map(|message| format!("Workspace `{alias}`: {message}"));
                    }
                }
                last_message = message;
            }
            if refresh_due {
                last_refresh = Some(Instant::now());
            }
            last_publish = Instant::now();
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn describe(report: &Report) -> Option<String> {
    if let Some(conflict) = report.conflicts.first() {
        return Some(format!(
            "Sync conflict on {}: {} (run `tandem sync status`)",
            conflict.id, conflict.reason
        ));
    }
    if let Some(held) = report.held.first() {
        return Some(format!("Sync held {}: {}", held.path, held.reason));
    }
    if let Some((old, new)) = report.renames.last() {
        return Some(format!("Synced; {old} is now {new}"));
    }
    match &report.outcome {
        Outcome::Pending(reason) => Some(format!("Saved locally; pending sync ({reason})")),
        _ => None,
    }
}
