use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

const PROFILE_NAME: &str = "key-b0x.ini";
const PROFILE_CONTENTS: &str = r#"[Profile]
Device = Pipe/0/slippibot1
Buttons/A = Button A
Buttons/B = Button B
Buttons/X = Button X
Buttons/Y = Button Y
Buttons/Z = Button Z
Buttons/Start = Button START
Buttons/L = Button L
Buttons/R = Button R
Main Stick/Up = Axis MAIN Y +
Main Stick/Down = Axis MAIN Y -
Main Stick/Left = Axis MAIN X -
Main Stick/Right = Axis MAIN X +
C-Stick/Up = Axis C Y +
C-Stick/Down = Axis C Y -
C-Stick/Left = Axis C X -
C-Stick/Right = Axis C X +
Triggers/Threshold = 99.000000000000000
Triggers/L = Button L
Triggers/R = Button R
Triggers/L-Analog = Axis L -+
Triggers/R-Analog = Axis R -+
D-Pad/Up = Button D_UP
D-Pad/Down = Button D_DOWN
D-Pad/Left = Button D_LEFT
D-Pad/Right = Button D_RIGHT
"#;

pub fn ensure_pipes_dir(slippi_user_path: &Path) -> Result<PathBuf> {
    let dir = slippi_user_path.join("Pipes");
    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    Ok(dir)
}

pub fn install_profile(slippi_user_path: &Path) -> Result<PathBuf> {
    let profile_dir = slippi_user_path
        .join("Config")
        .join("Profiles")
        .join("GCPad");
    fs::create_dir_all(&profile_dir)
        .with_context(|| format!("failed to create {}", profile_dir.display()))?;
    ensure_pipes_dir(slippi_user_path)?;

    let profile_path = profile_dir.join(PROFILE_NAME);
    fs::write(&profile_path, PROFILE_CONTENTS)
        .with_context(|| format!("failed to write {}", profile_path.display()))?;
    Ok(profile_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_profile_writes_profile_and_pipes_dir() {
        let temp = tempfile::tempdir().unwrap();
        let profile_path = install_profile(temp.path()).unwrap();
        assert!(profile_path.exists());
        assert!(temp.path().join("Pipes").is_dir());
        let raw = fs::read_to_string(profile_path).unwrap();
        assert!(raw.contains("Device = Pipe/0/slippibot1"));
    }
}
