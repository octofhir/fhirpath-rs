//! Configuration file watcher

use anyhow::Result;
use notify_debouncer_full::{DebounceEventResult, new_debouncer, notify::*};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// Config file change event
#[derive(Debug, Clone)]
pub struct ConfigChangeEvent {
    /// Path to the changed config file
    pub path: std::path::PathBuf,
}

/// Start watching configuration file
pub fn watch_config_file(config_path: &Path) -> Result<mpsc::UnboundedReceiver<ConfigChangeEvent>> {
    let (tx, rx) = mpsc::unbounded_channel();
    let config_path = config_path.to_path_buf();

    std::thread::spawn(move || {
        let tx = tx;
        let config_path = Arc::new(config_path);
        let config_path_for_watch = config_path.clone();

        let mut debouncer = new_debouncer(
            Duration::from_millis(100),
            None,
            move |result: DebounceEventResult| match result {
                Ok(events) => {
                    for event in events {
                        if event.paths.iter().any(|p| p == config_path.as_ref()) {
                            tracing::info!("Config file changed: {}", config_path.display());
                            let _ = tx.send(ConfigChangeEvent {
                                path: config_path.as_ref().clone(),
                            });
                        }
                    }
                }
                Err(errors) => {
                    for error in errors {
                        tracing::error!("Watch error: {:?}", error);
                    }
                }
            },
        )
        .expect("Failed to create file watcher");

        // Watch the config file
        debouncer
            .watch(config_path_for_watch.as_ref(), RecursiveMode::NonRecursive)
            .expect("Failed to watch config file");

        // Keep watcher alive
        loop {
            std::thread::park();
        }
    });

    Ok(rx)
}
