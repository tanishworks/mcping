//! Typed model of the JSON returned in the status response.

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct ServerStatus {
    pub players: Players,

    #[serde(default)]
    pub version: Option<Version>,

    /// The MOTD. Either a plain string or a nested text-component object.
    #[serde(default)]
    pub description: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct Players {
    pub online: u32,
    pub max: u32,

    /// Server-provided sample of online players. Optional: many servers omit
    /// or cap it (vanilla sends at most 12 random entries).
    #[serde(default)]
    pub sample: Option<Vec<Player>>,
}

#[derive(Debug, Deserialize)]
pub struct Player {
    pub name: String,
    pub id: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Version {
    pub name: String,

    #[serde(default)]
    pub protocol: i32,
}

impl ServerStatus {
    /// The server MOTD with text-component nesting flattened to plain text.
    pub fn motd(&self) -> String {
        self.description
            .as_ref()
            .map(flatten_text)
            .unwrap_or_default()
            .lines()
            .map(str::trim_end)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Player names from the sample, or `None` if the server hides the list.
    pub fn player_names(&self) -> Option<Vec<String>> {
        self.players
            .sample
            .as_ref()
            .map(|sample| sample.iter().map(|p| p.name.clone()).collect())
    }
}

/// Recursively flatten a Minecraft text component into plain text.
fn flatten_text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Array(items) => items.iter().map(flatten_text).collect::<Vec<_>>().join(""),
        Value::Object(map) => {
            let mut out = String::new();

            if let Some(Value::String(s)) = map.get("text") {
                out.push_str(s);
            }

            if let Some(extra) = map.get("extra") {
                out.push_str(&flatten_text(extra));
            }

            out
        }
        _ => String::new(),
    }
}

pub fn parse_status(json: &str) -> Result<ServerStatus, serde_json::Error> {
    serde_json::from_str(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_full_status() {
        let json = r#"{
            "players": {"online": 3, "max": 100, "sample": [
                {"name": "Notch", "id": "id-1"},
                {"name": "jeb_", "id": "id-2"}
            ]},
            "version": {"name": "Paper 1.21.4", "protocol": 767},
            "description": {"text": "Hello ", "extra": [{"text": "world", "bold": true}]}
        }"#;

        let status = parse_status(json).unwrap();

        assert_eq!(status.players.online, 3);
        assert_eq!(status.players.max, 100);
        assert_eq!(status.player_names().unwrap(), vec!["Notch", "jeb_"]);
        assert_eq!(status.version.as_ref().unwrap().name, "Paper 1.21.4");
        assert_eq!(status.motd(), "Hello world");
    }

    #[test]
    fn tolerates_missing_fields() {
        let status = parse_status(r#"{"players": {"online": 0, "max": 20}}"#).unwrap();

        assert_eq!(status.player_names(), None);
        assert_eq!(status.version, None);
        assert_eq!(status.motd(), "");
    }

    #[test]
    fn flattens_deeply_nested_motd() {
        let desc = json!({
            "text": "a",
            "extra": [
                {"text": "b", "extra": [{"text": "c"}]},
                "d"
            ]
        });

        assert_eq!(flatten_text(&desc), "abcd");
    }
}
