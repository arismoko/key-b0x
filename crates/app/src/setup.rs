use crate::config::AppConfig;
use crate::profile::{PIPE_TARGET_LABEL, PROFILE_FILE_NAME, profile_looks_installed};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupStatus {
    pub slippi_user_path: PathBuf,
    pub slippi_found: bool,
    pub profile_installed: bool,
    pub profile_path: PathBuf,
    pub pipe_target_label: String,
    pub error: Option<String>,
}

pub fn check_setup(config: &AppConfig) -> SetupStatus {
    let profile_path = config
        .slippi_user_path
        .join("Config")
        .join("Profiles")
        .join("GCPad")
        .join(PROFILE_FILE_NAME);

    SetupStatus {
        slippi_user_path: config.slippi_user_path.clone(),
        slippi_found: config.slippi_user_path.exists(),
        profile_installed: profile_looks_installed(&profile_path),
        profile_path,
        pipe_target_label: PIPE_TARGET_LABEL.to_string(),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[test]
    fn setup_detects_missing_installation() {
        let config = AppConfig {
            slippi_user_path: PathBuf::from("/tmp/key-b0x-setup-missing"),
            ..AppConfig::default()
        };

        let status = check_setup(&config);

        assert!(!status.slippi_found);
        assert!(!status.profile_installed);
        assert!(status.profile_path.ends_with("key-b0x.ini"));
    }
}
