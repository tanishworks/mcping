//! Minimal ANSI styling helpers.
//!
//! Honors `NO_COLOR` and disables colors when stdout is not a terminal, so
//! piped/watch output stays clean.

use std::env;
use std::io::IsTerminal;
use std::sync::OnceLock;

fn color_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal())
}

fn wrap(code: &str, text: &str) -> String {
    if color_enabled() {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

pub fn green(text: &str) -> String {
    wrap("32", text)
}

pub fn yellow(text: &str) -> String {
    wrap("33", text)
}

pub fn red(text: &str) -> String {
    wrap("31", text)
}

pub fn cyan(text: &str) -> String {
    wrap("36", text)
}

pub fn bold(text: &str) -> String {
    wrap("1", text)
}

pub fn dim(text: &str) -> String {
    wrap("2", text)
}

/// Colorize a latency in milliseconds using generic thresholds.
pub fn latency_colored(ms: u128) -> String {
    let text = format!("{ms}ms");
    match ms {
        0..=80 => green(&text),
        81..=250 => yellow(&text),
        _ => red(&text),
    }
}
