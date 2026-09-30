//! Background board sync while the TUI is open.
//!
//! A worker thread publishes local changes shortly after they are saved and
//! fetches incoming changes periodically. Sync writes board files; the TUI's
//! existing external-change detection reloads them. Noteworthy outcomes
//! (pending, conflicts, renumbering) are surfaced as a status message.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::project::sync::{self, Mode, Outcome, Report};
use crate::project::TandemProject;

const PUBLISH_INTERVAL: Duration = Duration::from_secs(2);
const REFRESH_INTERVAL: Duration = Duration::from_secs(30);

pub(crate) struct BackgroundSync {
    stop: Arc<AtomicBool>,
    note: Arc<Mutex<Option<String>>>,
    handle: Option<JoinHandle<()>>,
    workspace: TandemProject,
}

impl BackgroundSync {
    /// Starts syncing when the board is Git-backed; otherwise does nothing.
    pub(crate) fn start(workspace: &TandemProject) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let note = Arc::new(Mutex::new(None));
        let handle = workspace.git().is_some().then(|| {
            let (stop, note, workspace) = (stop.clone(), note.clone(), workspace.clone());
            std::thread::spawn(move || run(&workspace, &stop, &note))
        });
        Self {
            stop,
            note,
            handle,
            workspace: workspace.clone(),
        }
    }

    /// A status message for the latest noteworthy sync outcome, once.
    pub(crate) fn take_note(&self) -> Option<String> {
        self.note.lock().ok()?.take()
    }

    /// Stops the worker and publishes any last change before exit.
    pub(crate) fn finish(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
            let _ = sync::sync(&self.workspace, Mode::Publish);
        }
    }
}

fn run(workspace: &TandemProject, stop: &AtomicBool, note: &Mutex<Option<String>>) {
    let mut last_refresh: Option<Instant> = None;
    let mut last_message: Option<String> = None;
    let mut last_publish = Instant::now();
    while !stop.load(Ordering::Relaxed) {
        let refresh_due = last_refresh.is_none_or(|at| at.elapsed() >= REFRESH_INTERVAL);
        let publish_due = last_publish.elapsed() >= PUBLISH_INTERVAL;
        if refresh_due || publish_due {
            if crate::app::project::historical(workspace).is_none() {
                let mode = if refresh_due {
                    Mode::Refresh
                } else {
                    Mode::Publish
                };
                let report = sync::sync(workspace, mode)
                    .unwrap_or_else(|error| Report::pending(error.message));
                let message = describe(&report);
                if message.is_some() && message != last_message {
                    if let Ok(mut slot) = note.lock() {
                        slot.clone_from(&message);
                    }
                }
                last_message = message;
            }
            if refresh_due {
                last_refresh = Some(Instant::now());
            }
            last_publish = Instant::now();
        }
        std::thread::sleep(Duration::from_millis(200));
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
