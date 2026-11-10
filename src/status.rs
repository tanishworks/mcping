use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerStatus {
    pub players: Players,
}

#[derive(Debug, Deserialize)]
pub struct Players {
    pub online: u32,
    pub max: u32,

    #[serde(default)]
    pub sample: Option<Vec<Player>>,
}

#[derive(Debug, Deserialize)]
pub struct Player {
    pub name: String,
    pub id: String,
}

pub fn parse_status(json: &str) -> Result<ServerStatus, serde_json::Error> {
    serde_json::from_str(json)
}
