#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ServerMode {
    Pvp,
    Pve,
}

impl ServerMode {
    pub(crate) fn from_env() -> Self {
        match crate::config::env_string(crate::config::ENV_SERVER_MODE, crate::config::DEFAULT_SERVER_MODE)
            .to_lowercase()
            .as_str()
        {
            "pve" => Self::Pve,
            _ => Self::Pvp,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Pvp => "pvp",
            Self::Pve => "pve",
        }
    }
}

pub(crate) fn stamina_recovery_per_min() -> u32 {
    crate::config::stamina_recovery_per_min()
}
