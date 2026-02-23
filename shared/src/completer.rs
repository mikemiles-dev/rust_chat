use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

/// Shared command and optional username completer for rustyline.
pub struct CommandCompleter {
    commands: Vec<&'static str>,
    users: Option<Arc<RwLock<HashSet<String>>>>,
}

impl CommandCompleter {
    pub fn new(commands: Vec<&'static str>, users: Option<Arc<RwLock<HashSet<String>>>>) -> Self {
        Self { commands, users }
    }

    fn get_candidates(&self, line: &str) -> Vec<String> {
        let trimmed = line.trim_start();

        // If line starts with /dm or /send and has a space, complete usernames
        if let Some(ref users) = self.users
            && (trimmed.starts_with("/dm ") || trimmed.starts_with("/send "))
        {
            let parts: Vec<&str> = trimmed.splitn(3, ' ').collect();
            if parts.len() == 2 {
                let cmd = parts[0];
                let prefix = parts[1];
                let users = users.read().expect("connected users lock poisoned");
                return users
                    .iter()
                    .filter(|u| u.starts_with(prefix))
                    .map(|u| format!("{} {}", cmd, u))
                    .collect();
            }
        }

        // Complete commands
        if trimmed.starts_with('/') {
            self.commands
                .iter()
                .filter(|cmd| cmd.starts_with(trimmed))
                .map(|s| s.to_string())
                .collect()
        } else {
            vec![]
        }
    }
}

impl Completer for CommandCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let candidates = self.get_candidates(&line[..pos]);

        let pairs: Vec<Pair> = candidates
            .into_iter()
            .map(|c| Pair {
                display: c.clone(),
                replacement: c,
            })
            .collect();

        Ok((0, pairs))
    }
}

impl Hinter for CommandCompleter {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        let candidates = self.get_candidates(line);
        if candidates.len() == 1 {
            let candidate = &candidates[0];
            if candidate.starts_with(line) && candidate.len() > line.len() {
                return Some(candidate[line.len()..].to_string());
            }
        }
        None
    }
}

impl Highlighter for CommandCompleter {}

impl Validator for CommandCompleter {}

impl Helper for CommandCompleter {}
