use crate::{Error, Result, SPath};
use notify::{self, RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{new_debouncer, DebounceEventHandler, DebounceEventResult, Debouncer, FileIdMap};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

// -- Re-export some DebouncedEvent
pub use notify_debouncer_full::DebouncedEvent;
use std::collections::HashSet;

const WATCH_DEBOUNCE_MS: u64 = 200;

// region:    --- SimpleEvent

/// A greatly simplified file event struct, containing only one path and one simplified event kind.
/// Additionally, these will be debounced on top of the debouncer to ensure only one path/kind per debounced event list.
#[derive(Debug)]
pub struct SEvent {
	pub spath: SPath,
	pub skind: SEventKind,
}

/// Simplified event kind.
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum SEventKind {
	Create,
	Modify,
	Remove,
	Other,
}

impl From<notify::EventKind> for SEventKind {
	fn from(val: notify::EventKind) -> Self {
		match val {
			notify::EventKind::Any => SEventKind::Other,
			notify::EventKind::Access(_) => SEventKind::Other,
			notify::EventKind::Create(_) => SEventKind::Create,
			notify::EventKind::Modify(_) => SEventKind::Modify,
			notify::EventKind::Remove(_) => SEventKind::Remove,
			notify::EventKind::Other => SEventKind::Other,
		}
	}
}

/// A simplified watcher struct containing a receiver for file system events and an internal debouncer.
#[allow(unused)]
pub struct SWatcher {
	pub rx: Receiver<Vec<SEvent>>,
	// Note: Here we keep the debouncer so that it does not get dropped and continues to run.
	notify_full_debouncer: Debouncer<RecommendedWatcher, FileIdMap>,
}

// endregion: --- SimpleEvent

/// A simplified watcher that monitors a path (file or directory) and returns an `SWatcher` object with a
/// standard mpsc Receiver for a `Vec<SEvent>`.
/// Each `SEvent` contains one `spath` and one simplified event kind (`SEventKind`).
/// This will ignore any path that cannot be converted to a string (i.e., it will only trigger events if the path is valid UTF-8)
pub fn watch(path: impl AsRef<Path>) -> Result<SWatcher> {
	let (tx, rx) = channel();

	let path = path.as_ref();
	let handler = EventHandler { tx };
	let mut debouncer =
		new_debouncer(Duration::from_millis(WATCH_DEBOUNCE_MS), None, handler).map_err(|err| Error::FailToWatch {
			path: path.to_string_lossy().to_string(),
			cause: err.to_string(),
		})?;

	if !path.exists() {
		return Err(Error::CantWatchPathNotFound(path.to_string_lossy().to_string()));
	}

	debouncer
		.watch(path, RecursiveMode::Recursive)
		.map_err(|err| Error::FailToWatch {
			path: path.to_string_lossy().to_string(),
			cause: err.to_string(),
		})?;

	let swatcher = SWatcher {
		rx,
		notify_full_debouncer: debouncer,
	};

	Ok(swatcher)
}

/// Event Handler that propagates a simplified Vec<SEvent>
struct EventHandler {
	tx: Sender<Vec<SEvent>>,
}

impl DebounceEventHandler for EventHandler {
	fn handle_event(&mut self, result: DebounceEventResult) {
		match result {
			Ok(events) => {
				let sevents = build_sevents(events);
				if !sevents.is_empty() {
					let _ = self.tx.send(sevents);
				}
			}
			Err(err) => println!("simple-fs - handle_event error {err:?}"), // may want to trace
		}
	}
}

#[derive(Hash, Eq, PartialEq)]
struct SEventKey {
	spath_string: String,
	skind: SEventKind,
}

fn build_sevents(events: Vec<DebouncedEvent>) -> Vec<SEvent> {
	let mut sevents_set: HashSet<SEventKey> = HashSet::new();

	let mut sevents = Vec::new();

	for devent in events {
		let event = devent.event;
		let skind = SEventKind::from(event.kind);

		for path in event.paths {
			if let Some(spath) = SPath::from_path_buf_ok(path) {
				let key = SEventKey {
					spath_string: spath.to_string(),
					skind: skind.clone(),
				};

				// If this spath/skind is not in the set, then add it to the sevents list
				if !sevents_set.contains(&key) {
					sevents.push(SEvent {
						spath,
						skind: skind.clone(),
					});

					sevents_set.insert(key);
				}
			}
		}
	}

	sevents
}
