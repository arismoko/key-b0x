use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MeleeConfig {
    pub socd: SocdConfig,
    pub down_diagonal: DownDiagonalBehavior,
    pub horizontal_socd_override: HorizontalSocdOverride,
    pub airdodge: AirdodgeConfig,
}

impl Default for MeleeConfig {
    fn default() -> Self {
        Self {
            socd: SocdConfig::default(),
            down_diagonal: DownDiagonalBehavior::default(),
            horizontal_socd_override: HorizontalSocdOverride::default(),
            airdodge: AirdodgeConfig::default(),
        }
    }
}

impl MeleeConfig {
    pub fn validate(&self) -> Result<(), MeleeConfigError> {
        if let AirdodgeConfig::CustomModXDiagonal { x, y } = self.airdodge {
            if !(0.0..=1.0).contains(&x) || x == 0.0 {
                return Err(MeleeConfigError::InvalidCustomAirdodgeAxis {
                    axis: "x",
                    value: x,
                });
            }
            if !(0.0..=1.0).contains(&y) || y == 0.0 {
                return Err(MeleeConfigError::InvalidCustomAirdodgeAxis {
                    axis: "y",
                    value: y,
                });
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SocdConfig {
    pub main_x: SocdMode,
    pub main_y: SocdMode,
    pub c_x: SocdMode,
    pub c_y: SocdMode,
}

impl Default for SocdConfig {
    fn default() -> Self {
        Self {
            main_x: SocdMode::SecondInputPriorityNoReactivation,
            main_y: SocdMode::SecondInputPriorityNoReactivation,
            c_x: SocdMode::SecondInputPriorityNoReactivation,
            c_y: SocdMode::SecondInputPriorityNoReactivation,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SocdMode {
    Neutral,
    SecondInputPriority,
    SecondInputPriorityNoReactivation,
    Dir1Priority,
    Dir2Priority,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownDiagonalBehavior {
    #[default]
    AutoJabCancel,
    CrouchWalkOs,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HorizontalSocdOverride {
    #[default]
    MaxJumpTrajectory,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AirdodgeConfig {
    Default,
    CustomModXDiagonal { x: f64, y: f64 },
}

impl Default for AirdodgeConfig {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Error)]
pub enum MeleeConfigError {
    #[error("custom airdodge {axis} must be within (0.0, 1.0], got {value}")]
    InvalidCustomAirdodgeAxis { axis: &'static str, value: f64 },
}
