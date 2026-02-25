use bevy::prelude::Resource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Server,
    Client,
    Host,
}

impl RunMode {
    pub fn parse_cli_value(value: &str) -> Option<Self> {
        if value.eq_ignore_ascii_case("server") {
            Some(Self::Server)
        } else if value.eq_ignore_ascii_case("client") {
            Some(Self::Client)
        } else if value.eq_ignore_ascii_case("host") {
            Some(Self::Host)
        } else {
            None
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Server => "server",
            Self::Client => "client",
            Self::Host => "host",
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppRunMode(pub RunMode);

impl Default for AppRunMode {
    fn default() -> Self {
        Self(RunMode::Client)
    }
}
