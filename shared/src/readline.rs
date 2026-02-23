use crate::completer::CommandCompleter;
use rustyline::Editor;
use rustyline::config::Configurer;
use tokio::sync::mpsc;

/// Spawns a rustyline handler in a blocking thread and sends input via channel.
///
/// If `require_tty` is true and no TTY is available, panics.
/// If `require_tty` is false and no TTY is available, returns `None`.
pub fn spawn_readline_handler(
    completer: CommandCompleter,
    require_tty: bool,
) -> Option<mpsc::UnboundedReceiver<Option<String>>> {
    let (tx, rx) = mpsc::unbounded_channel();

    let rl = match Editor::new() {
        Ok(rl) => rl,
        Err(_) if require_tty => {
            panic!("Failed to create editor: no TTY available");
        }
        Err(_) => {
            return None;
        }
    };

    std::thread::spawn(move || {
        let mut rl = rl;
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
