use crate::completer::ClientCompleter;
use rustyline::Editor;
use rustyline::config::Configurer;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

/// Runs rustyline in a blocking thread and sends input via channel
pub fn spawn_readline_handler(
    users: Arc<RwLock<HashSet<String>>>,
    _prompt: String,
) -> mpsc::UnboundedReceiver<Option<String>> {
    let (tx, rx) = mpsc::unbounded_channel();

    std::thread::spawn(move || {
        let completer = ClientCompleter::new(users);
        let mut rl = Editor::new().expect("Failed to create editor");
        rl.set_helper(Some(completer));
        rl.set_auto_add_history(true);
        rl.set_max_history_size(1000).ok();

        loop {
            match rl.readline("") {
                Ok(line) => {
                    if tx.send(Some(line)).is_err() {
                        break; // Receiver dropped
                    }
                }
                Err(_) => {
                    let _ = tx.send(None); // EOF or error
                    break;
                }
            }
        }
    });

    rx
}
