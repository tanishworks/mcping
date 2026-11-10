use std::fmt;
use std::str::FromStr;

/// Default Minecraft Java Edition server port.
pub const DEFAULT_PORT: u16 = 25565;

/// A parsed server address: `host[:port]`.
///
/// Accepted forms:
/// - `alwto.net` (default port)
/// - `alwto.net:25565`
/// - `[::1]:25565` (bracketed IPv6)
/// - `127.0.0.1:25565`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerAddr {
    pub host: String,
    pub port: u16,
}

impl ServerAddr {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }
}

#[derive(Debug)]
pub struct ParseAddrError(pub String);

impl fmt::Display for ParseAddrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid address: {}", self.0)
    }
}

impl std::error::Error for ParseAddrError {}

fn err(msg: &str) -> ParseAddrError {
    ParseAddrError(msg.to_string())
}

impl FromStr for ServerAddr {
    type Err = ParseAddrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(err("address is empty"));
        }

        // Bracketed IPv6: [host]:port
        if let Some(rest) = s.strip_prefix('[') {
            let (host, after) = rest
                .split_once(']')
                .ok_or_else(|| err("missing closing ']' for IPv6 address"))?;

            if host.is_empty() {
                return Err(err("empty IPv6 host"));
            }

            let port = match after.strip_prefix(':') {
                Some(p) => parse_port(p)?,
                None if after.is_empty() => DEFAULT_PORT,
                None => return Err(err("unexpected text after ']'")),
            };

            return Ok(ServerAddr::new(host, port));
        }

        // host:port — the colon must separate a non-empty host from a numeric
        // port. Bare (unbracketed) IPv6 is rejected: it must be bracketed.
        match s.rsplit_once(':') {
            Some((host, port_str)) if !host.contains(':') => {
                if host.is_empty() {
                    return Err(err("missing host"));
                }

                let port = parse_port(port_str)?;
                Ok(ServerAddr::new(host, port))
            }
            Some(_) => Err(err("bare IPv6 addresses must be bracketed: [::1]:25565")),
            None => Ok(ServerAddr::new(s, DEFAULT_PORT)),
        }
    }
}

fn parse_port(p: &str) -> Result<u16, ParseAddrError> {
    p.parse::<u16>()
        .map_err(|_| err(&format!("invalid port '{p}'")))
}

impl fmt::Display for ServerAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.host.contains(':') {
            write!(f, "[{}]:{}", self.host, self.port)
        } else {
            write!(f, "{}:{}", self.host, self.port)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_host_with_default_port() {
        let a: ServerAddr = "alwto.net".parse().unwrap();
        assert_eq!(a, ServerAddr::new("alwto.net", 25565));
    }

    #[test]
    fn parses_host_with_port() {
        let a: ServerAddr = "alwto.net:25566".parse().unwrap();
        assert_eq!(a, ServerAddr::new("alwto.net", 25566));
    }

    #[test]
    fn parses_ipv4_with_port() {
        let a: ServerAddr = "127.0.0.1:30000".parse().unwrap();
        assert_eq!(a, ServerAddr::new("127.0.0.1", 30000));
    }

    #[test]
    fn parses_bracketed_ipv6() {
        let a: ServerAddr = "[::1]:25565".parse().unwrap();
        assert_eq!(a, ServerAddr::new("::1", 25565));

        let b: ServerAddr = "[::1]".parse().unwrap();
        assert_eq!(b, ServerAddr::new("::1", 25565));
    }

    #[test]
    fn display_is_canonical() {
        let default: ServerAddr = "alwto.net".parse().unwrap();
        assert_eq!(default.to_string(), "alwto.net:25565");

        let with_port: ServerAddr = "alwto.net:25566".parse().unwrap();
        assert_eq!(with_port.to_string(), "alwto.net:25566");
    }

    #[test]
    fn rejects_empty_and_garbage() {
        assert!("".parse::<ServerAddr>().is_err());
        assert!("[".parse::<ServerAddr>().is_err());
        assert!("host:99999".parse::<ServerAddr>().is_err());
        assert!("host:abc".parse::<ServerAddr>().is_err());
        assert!("host:".parse::<ServerAddr>().is_err());
        assert!("::1".parse::<ServerAddr>().is_err()); // bare IPv6 must be bracketed
    }

    #[test]
    fn accepts_whitespace() {
        let a: ServerAddr = "  alwto.net  ".parse().unwrap();
        assert_eq!(a, ServerAddr::new("alwto.net", 25565));
    }
}
