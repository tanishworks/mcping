//! Desktop notifications and terminal bell.

use std::process::Command;

/// Fire a desktop notification. Best-effort: failures are silently ignored.
pub fn desktop(title: &str, body: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("osascript")
            .arg("-e")
            .arg(format!(
                "display notification \"{}\" with title \"{}\"",
                escape_applescript(body),
                escape_applescript(title)
            ))
            .status();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("notify-send").arg(title).arg(body).status();
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = (title, body);
    }
}

/// Escape a string for embedding in a double-quoted AppleScript literal.
#[cfg(target_os = "macos")]
fn escape_applescript(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Ring the terminal bell.
pub fn bell() {
    use std::io::Write;
    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(b"\x07");
    let _ = stdout.flush();
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "macos")]
    #[test]
    fn escapes_applescript_quotes() {
        assert_eq!(
            super::escape_applescript("say \"hi\" \\ now"),
            "say \\\"hi\\\" \\\\ now"
        );
    }
}
