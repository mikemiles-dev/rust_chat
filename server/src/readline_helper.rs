use crate::completer::ServerCompleter;
use rustyline::config::Configurer;
use rustyline::Editor;
use tokio::sync::mpsc;

/// Runs rustyline in a blocking thread and sends input via channel
/// Returns None if TTY is not available (e.g., Docker without -it)
pub fn spawn_readline_handler() -> Option<mpsc::UnboundedReceiver<Option<String>>> {
    let (tx, rx) = mpsc::unbounded_channel();

    // Try to create editor - if it fails (no TTY), return None
    let rl_result = Editor::new();

    if rl_result.is_err() {
        // No TTY available (Docker/systemd/etc), skip readline
        return None;
    }

    std::thread::spawn(move || {
        let completer = ServerCompleter::new();
        let mut rl = rl_result.unwrap();
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

    Some(rx)
}
