use crate::binding::{BindingId, ControllerSnapshot, InputEvent};
use crate::config::{
    AirdodgeConfig, DownDiagonalBehavior, HorizontalSocdOverride, MeleeConfig, MeleeConfigError,
};
use crate::coords::{
    COORDS_AIRDODGE_HORIZONTAL, COORDS_AIRDODGE_HORIZONTAL_MOD_X, COORDS_AIRDODGE_HORIZONTAL_MOD_Y,
    COORDS_AIRDODGE_QUADRANT_12, COORDS_AIRDODGE_QUADRANT_12_MOD_Y, COORDS_AIRDODGE_QUADRANT_34,
    COORDS_AIRDODGE_QUADRANT_34_MOD_Y, COORDS_AIRDODGE_QUADRANT_MOD_X, COORDS_AIRDODGE_VERTICAL,
    COORDS_AIRDODGE_VERTICAL_MOD_X, COORDS_AIRDODGE_VERTICAL_MOD_Y, COORDS_CSTICK_DIAGONAL,
    COORDS_CSTICK_MOD_X_DOWN, COORDS_CSTICK_MOD_X_UP, COORDS_DOWN_DIAGONAL_CROUCH_WALK_OS,
    COORDS_EXT_FIREFOX_MOD_X, COORDS_EXT_FIREFOX_MOD_X_C_DOWN, COORDS_EXT_FIREFOX_MOD_X_C_LEFT,
    COORDS_EXT_FIREFOX_MOD_X_C_RIGHT, COORDS_EXT_FIREFOX_MOD_X_C_UP, COORDS_EXT_FIREFOX_MOD_Y,
    COORDS_EXT_FIREFOX_MOD_Y_C_DOWN, COORDS_EXT_FIREFOX_MOD_Y_C_LEFT,
    COORDS_EXT_FIREFOX_MOD_Y_C_RIGHT, COORDS_EXT_FIREFOX_MOD_Y_C_UP, COORDS_FIREFOX_MOD_X_C_DOWN,
    COORDS_FIREFOX_MOD_X_C_LEFT, COORDS_FIREFOX_MOD_X_C_RIGHT, COORDS_FIREFOX_MOD_X_C_UP,
    COORDS_FIREFOX_MOD_Y_C_DOWN, COORDS_FIREFOX_MOD_Y_C_LEFT, COORDS_FIREFOX_MOD_Y_C_RIGHT,
    COORDS_FIREFOX_MOD_Y_C_UP, COORDS_HORIZONTAL, COORDS_HORIZONTAL_MOD_X, COORDS_HORIZONTAL_MOD_Y,
    COORDS_ORIGIN, COORDS_QUADRANT, COORDS_QUADRANT_MOD_X, COORDS_QUADRANT_MOD_Y, COORDS_VERTICAL,
    COORDS_VERTICAL_MOD_X, COORDS_VERTICAL_MOD_Y, DOLPHIN_STICK_RADIUS, GAMECUBE_STICK_RADIUS,
    LIGHT_SHIELD_VALUE, MID_SHIELD_VALUE, TRIGGER_AXIS_MAX,
};
use crate::socd::{SocdPairState, resolve_pair};

#[derive(Clone, Debug)]
pub struct MeleeEngine {
    config: MeleeConfig,
    state: MeleeState,
}

impl MeleeEngine {
    pub fn try_new(config: MeleeConfig) -> Result<Self, MeleeConfigError> {
        config.validate()?;

        Ok(Self {
            config,
            state: MeleeState::default(),
        })
    }

    pub fn config(&self) -> &MeleeConfig {
        &self.config
    }

    pub fn handle_event(&mut self, event: InputEvent) -> ControllerSnapshot {
        self.state.raw.apply_event(event);
        self.state.recompute(&self.config);
        self.snapshot()
    }

    pub fn snapshot(&self) -> ControllerSnapshot {
        self.state.snapshot(&self.config)
    }

    pub fn reset(&mut self) -> ControllerSnapshot {
        self.state = MeleeState::default();
        self.snapshot()
    }
}

impl Default for MeleeEngine {
    fn default() -> Self {
        Self::try_new(MeleeConfig::default()).expect("default melee config must be valid")
    }
}

#[derive(Clone, Debug, Default)]
struct MeleeState {
    raw: RawButtons,
    socd: SocdBank,
    resolved: ResolvedDirections,
}

impl MeleeState {
    fn recompute(&mut self, config: &MeleeConfig) {
        self.resolved = ResolvedDirections::resolve(&self.raw, config, &mut self.socd);
    }

    fn snapshot(&self, config: &MeleeConfig) -> ControllerSnapshot {
        let context = FrameContext {
            raw: &self.raw,
            resolved: &self.resolved,
        };
        let (main_x, main_y) = main_stick_coords(&context, config);
        let (c_x, c_y) = c_stick_coords(&context);

        ControllerSnapshot {
            a: self.raw.a,
            b: self.raw.b,
            x: self.raw.x,
            y: self.raw.y,
            z: self.raw.z,
            start: self.raw.start,
            l: self.raw.l,
            r: self.raw.r,
            d_up: self.raw.d_up || (context.both_mods() && self.raw.c_up),
            d_down: self.raw.d_down || (context.both_mods() && self.raw.c_down),
            d_left: self.raw.d_left || (context.both_mods() && self.raw.c_left),
            d_right: self.raw.d_right || (context.both_mods() && self.raw.c_right),
            l_analog: 0.0,
            r_analog: normalize_trigger(context.analog_shield_value()),
            main_x: normalize_axis(main_x),
            main_y: normalize_axis(main_y),
            c_x: normalize_axis(c_x),
            c_y: normalize_axis(c_y),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct RawButtons {
    analog_up: bool,
    analog_down: bool,
    analog_left: bool,
    analog_right: bool,
    a: bool,
    b: bool,
    l: bool,
    r: bool,
    x: bool,
    y: bool,
    z: bool,
    start: bool,
    light_shield: bool,
    mid_shield: bool,
    mod_x: bool,
    mod_y: bool,
    c_up: bool,
    c_down: bool,
    c_left: bool,
    c_right: bool,
    d_up: bool,
    d_down: bool,
    d_left: bool,
    d_right: bool,
}

impl RawButtons {
    fn apply_event(&mut self, event: InputEvent) {
        match event.binding {
            BindingId::AnalogUp => self.analog_up = event.pressed,
            BindingId::AnalogDown => self.analog_down = event.pressed,
            BindingId::AnalogLeft => self.analog_left = event.pressed,
            BindingId::AnalogRight => self.analog_right = event.pressed,
            BindingId::ModX => self.mod_x = event.pressed,
            BindingId::ModY => self.mod_y = event.pressed,
            BindingId::A => self.a = event.pressed,
            BindingId::B => self.b = event.pressed,
            BindingId::L => self.l = event.pressed,
            BindingId::R => self.r = event.pressed,
            BindingId::X => self.x = event.pressed,
            BindingId::Y => self.y = event.pressed,
            BindingId::Z => self.z = event.pressed,
            BindingId::CUp => self.c_up = event.pressed,
            BindingId::CDown => self.c_down = event.pressed,
            BindingId::CLeft => self.c_left = event.pressed,
            BindingId::CRight => self.c_right = event.pressed,
            BindingId::LightShield => self.light_shield = event.pressed,
            BindingId::MidShield => self.mid_shield = event.pressed,
            BindingId::Start => self.start = event.pressed,
            BindingId::DUp => self.d_up = event.pressed,
            BindingId::DDown => self.d_down = event.pressed,
            BindingId::DLeft => self.d_left = event.pressed,
            BindingId::DRight => self.d_right = event.pressed,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct SocdBank {
    main_x: SocdPairState,
    main_y: SocdPairState,
    c_x: SocdPairState,
    c_y: SocdPairState,
}

#[derive(Clone, Copy, Debug, Default)]
struct ResolvedDirections {
    main_horizontal: Option<Horizontal>,
    main_vertical: Option<Vertical>,
    c_horizontal: Option<Horizontal>,
    c_vertical: Option<Vertical>,
}

impl ResolvedDirections {
    fn resolve(raw: &RawButtons, config: &MeleeConfig, bank: &mut SocdBank) -> Self {
        let (left, right) = resolve_pair(
            config.socd.main_x,
            raw.analog_left,
            raw.analog_right,
            &mut bank.main_x,
        );
        let (down, up) = resolve_pair(
            config.socd.main_y,
            raw.analog_down,
            raw.analog_up,
            &mut bank.main_y,
        );
        let (c_left, c_right) =
            resolve_pair(config.socd.c_x, raw.c_left, raw.c_right, &mut bank.c_x);
        let (c_down, c_up) = resolve_pair(config.socd.c_y, raw.c_down, raw.c_up, &mut bank.c_y);

        Self {
            main_horizontal: horizontal_from_pair(left, right),
            main_vertical: vertical_from_pair(down, up),
            c_horizontal: horizontal_from_pair(c_left, c_right),
            c_vertical: vertical_from_pair(c_down, c_up),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Vertical {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Horizontal {
    Left,
    Right,
}

impl Horizontal {
    fn sign(self) -> f64 {
        match self {
            Horizontal::Left => -1.0,
            Horizontal::Right => 1.0,
        }
    }
}

struct FrameContext<'a> {
    raw: &'a RawButtons,
    resolved: &'a ResolvedDirections,
}

impl FrameContext<'_> {
    fn up(&self) -> bool {
        self.resolved.main_vertical == Some(Vertical::Up)
    }

    fn down(&self) -> bool {
        self.resolved.main_vertical == Some(Vertical::Down)
    }

    fn left(&self) -> bool {
        self.resolved.main_horizontal == Some(Horizontal::Left)
    }

    fn right(&self) -> bool {
        self.resolved.main_horizontal == Some(Horizontal::Right)
    }

    fn c_up(&self) -> bool {
        self.resolved.c_vertical == Some(Vertical::Up) && !self.both_mods()
    }

    fn c_down(&self) -> bool {
        self.resolved.c_vertical == Some(Vertical::Down) && !self.both_mods()
    }

    fn c_left(&self) -> bool {
        self.resolved.c_horizontal == Some(Horizontal::Left) && !self.both_mods()
    }

    fn c_right(&self) -> bool {
        self.resolved.c_horizontal == Some(Horizontal::Right) && !self.both_mods()
    }

    fn mod_x(&self) -> bool {
        self.raw.mod_x && !self.raw.mod_y
    }

    fn mod_y(&self) -> bool {
        self.raw.mod_y && !self.raw.mod_x
    }

    fn any_vert(&self) -> bool {
        self.up() || self.down()
    }

    fn any_horiz(&self) -> bool {
        self.left() || self.right()
    }

    fn any_quadrant(&self) -> bool {
        self.any_vert() && self.any_horiz()
    }

    fn any_mod(&self) -> bool {
        self.mod_x() || self.mod_y()
    }

    fn both_mods(&self) -> bool {
        self.raw.mod_x && self.raw.mod_y
    }

    fn any_shield(&self) -> bool {
        self.raw.l || self.raw.r || self.raw.light_shield || self.raw.mid_shield
    }

    fn any_vert_c(&self) -> bool {
        self.c_up() || self.c_down()
    }

    fn any_horiz_c(&self) -> bool {
        self.c_left() || self.c_right()
    }

    fn any_c(&self) -> bool {
        self.c_up() || self.c_down() || self.c_left() || self.c_right()
    }

    fn analog_shield_value(&self) -> u8 {
        if self.raw.mid_shield {
            MID_SHIELD_VALUE
        } else if self.raw.light_shield {
            LIGHT_SHIELD_VALUE
        } else {
            0
        }
    }

    fn button_b(&self) -> bool {
        self.raw.b
    }

    fn main_horizontal_direction(&self) -> Option<Horizontal> {
        self.resolved.main_horizontal
    }

    fn raw_main_horizontal_socd(&self) -> bool {
        self.raw.analog_left && self.raw.analog_right
    }
}

fn horizontal_from_pair(left: bool, right: bool) -> Option<Horizontal> {
    if left {
        Some(Horizontal::Left)
    } else if right {
        Some(Horizontal::Right)
    } else {
        None
    }
}

fn vertical_from_pair(down: bool, up: bool) -> Option<Vertical> {
    if down {
        Some(Vertical::Down)
    } else if up {
        Some(Vertical::Up)
    } else {
        None
    }
}

fn reflect_main_coords(context: &FrameContext<'_>, (mut x, mut y): (f64, f64)) -> (f64, f64) {
    if context.down() {
        y = -y;
    }
    if context.left() {
        x = -x;
    }
    (x, y)
}

fn reflect_c_stick_coords(context: &FrameContext<'_>, (mut x, mut y): (f64, f64)) -> (f64, f64) {
    if context.c_down() {
        y = -y;
    }
    if context.c_left() {
        x = -x;
    }
    (x, y)
}

fn main_stick_coords(context: &FrameContext<'_>, config: &MeleeConfig) -> (f64, f64) {
    let coords = if context.any_shield() {
        airdodge_coords(context, config)
    } else if context.any_mod() && context.any_quadrant() && (context.any_c() || context.button_b())
    {
        firefox_coords(context)
    } else {
        no_shield_coords(context, config)
    };

    let mut reflected = reflect_main_coords(context, coords);

    if matches!(
        config.horizontal_socd_override,
        HorizontalSocdOverride::MaxJumpTrajectory
    ) && context.raw_main_horizontal_socd()
        && !context.any_vert()
    {
        if let Some(direction) = context.main_horizontal_direction() {
            reflected.0 = direction.sign();
        }
    }

    reflected
}

fn no_shield_coords(context: &FrameContext<'_>, config: &MeleeConfig) -> (f64, f64) {
    if !context.any_vert() && !context.any_horiz() {
        COORDS_ORIGIN
    } else if context.any_quadrant() {
        if context.mod_x() {
            COORDS_QUADRANT_MOD_X
        } else if context.mod_y() {
            COORDS_QUADRANT_MOD_Y
        } else if context.down()
            && matches!(config.down_diagonal, DownDiagonalBehavior::CrouchWalkOs)
        {
            COORDS_DOWN_DIAGONAL_CROUCH_WALK_OS
        } else {
            COORDS_QUADRANT
        }
    } else if context.any_vert() {
        if context.mod_x() {
            COORDS_VERTICAL_MOD_X
        } else if context.mod_y() {
            COORDS_VERTICAL_MOD_Y
        } else {
            COORDS_VERTICAL
        }
    } else if context.mod_x() {
        COORDS_HORIZONTAL_MOD_X
    } else if context.mod_y() {
        if context.button_b() {
            COORDS_HORIZONTAL
        } else {
            COORDS_HORIZONTAL_MOD_Y
        }
    } else {
        COORDS_HORIZONTAL
    }
}

fn airdodge_coords(context: &FrameContext<'_>, config: &MeleeConfig) -> (f64, f64) {
    if !context.any_vert() && !context.any_horiz() {
        COORDS_ORIGIN
    } else if context.any_quadrant() {
        if context.mod_x() {
            match config.airdodge {
                AirdodgeConfig::Default => COORDS_AIRDODGE_QUADRANT_MOD_X,
                AirdodgeConfig::CustomModXDiagonal { x, y } => (x, y),
            }
        } else if context.mod_y() {
            if context.up() {
                COORDS_AIRDODGE_QUADRANT_12_MOD_Y
            } else {
                COORDS_AIRDODGE_QUADRANT_34_MOD_Y
            }
        } else if context.up() {
            COORDS_AIRDODGE_QUADRANT_12
        } else {
            COORDS_AIRDODGE_QUADRANT_34
        }
    } else if context.any_vert() {
        if context.mod_x() {
            COORDS_AIRDODGE_VERTICAL_MOD_X
        } else if context.mod_y() {
            COORDS_AIRDODGE_VERTICAL_MOD_Y
        } else {
            COORDS_AIRDODGE_VERTICAL
        }
    } else if context.mod_x() {
        COORDS_AIRDODGE_HORIZONTAL_MOD_X
    } else if context.mod_y() {
        if context.button_b() {
            COORDS_AIRDODGE_HORIZONTAL
        } else {
            COORDS_AIRDODGE_HORIZONTAL_MOD_Y
        }
    } else {
        COORDS_AIRDODGE_HORIZONTAL
    }
}

fn firefox_coords(context: &FrameContext<'_>) -> (f64, f64) {
    if context.mod_x() {
        if context.c_up() {
            if context.button_b() {
                COORDS_EXT_FIREFOX_MOD_X_C_UP
            } else {
                COORDS_FIREFOX_MOD_X_C_UP
            }
        } else if context.c_down() {
            if context.button_b() {
                COORDS_EXT_FIREFOX_MOD_X_C_DOWN
            } else {
                COORDS_FIREFOX_MOD_X_C_DOWN
            }
        } else if context.c_left() {
            if context.button_b() {
                COORDS_EXT_FIREFOX_MOD_X_C_LEFT
            } else {
                COORDS_FIREFOX_MOD_X_C_LEFT
            }
        } else if context.c_right() {
            if context.button_b() {
                COORDS_EXT_FIREFOX_MOD_X_C_RIGHT
            } else {
                COORDS_FIREFOX_MOD_X_C_RIGHT
            }
        } else {
            COORDS_EXT_FIREFOX_MOD_X
        }
    } else if context.mod_y() {
        if context.c_up() {
            if context.button_b() {
                COORDS_EXT_FIREFOX_MOD_Y_C_UP
            } else {
                COORDS_FIREFOX_MOD_Y_C_UP
            }
        } else if context.c_down() {
            if context.button_b() {
                COORDS_EXT_FIREFOX_MOD_Y_C_DOWN
            } else {
                COORDS_FIREFOX_MOD_Y_C_DOWN
            }
        } else if context.c_left() {
            if context.button_b() {
                COORDS_EXT_FIREFOX_MOD_Y_C_LEFT
            } else {
                COORDS_FIREFOX_MOD_Y_C_LEFT
            }
        } else if context.c_right() {
            if context.button_b() {
                COORDS_EXT_FIREFOX_MOD_Y_C_RIGHT
            } else {
                COORDS_FIREFOX_MOD_Y_C_RIGHT
            }
        } else {
            COORDS_EXT_FIREFOX_MOD_Y
        }
    } else {
        COORDS_ORIGIN
    }
}

fn c_stick_coords(context: &FrameContext<'_>) -> (f64, f64) {
    let coords = if !context.any_vert_c() && !context.any_horiz_c() {
        COORDS_ORIGIN
    } else if context.any_vert_c() && context.any_horiz_c() {
        COORDS_CSTICK_DIAGONAL
    } else if context.any_vert_c() {
        COORDS_VERTICAL
    } else if context.mod_x() && context.up() {
        COORDS_CSTICK_MOD_X_UP
    } else if context.mod_x() && context.down() {
        COORDS_CSTICK_MOD_X_DOWN
    } else {
        COORDS_HORIZONTAL
    };

    reflect_c_stick_coords(context, coords)
}

fn normalize_axis(value: f64) -> f64 {
    let scaled = value.clamp(-1.0, 1.0) * (GAMECUBE_STICK_RADIUS / DOLPHIN_STICK_RADIUS);
    ((scaled + 1.0) / 2.0).clamp(0.0, 1.0)
}

fn normalize_trigger(value: u8) -> f64 {
    (f64::from(value) / TRIGGER_AXIS_MAX).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AirdodgeConfig, DownDiagonalBehavior, HorizontalSocdOverride, SocdMode};

    fn approx_eq(lhs: f64, rhs: f64) {
        assert!((lhs - rhs).abs() < 1e-9, "{lhs} != {rhs}");
    }

    fn press(engine: &mut MeleeEngine, binding: BindingId) -> ControllerSnapshot {
        engine.handle_event(InputEvent {
            binding,
            pressed: true,
        })
    }

    fn release(engine: &mut MeleeEngine, binding: BindingId) -> ControllerSnapshot {
        engine.handle_event(InputEvent {
            binding,
            pressed: false,
        })
    }

    #[test]
    fn neutral_snapshot_is_centered() {
        let snapshot = MeleeEngine::default().snapshot();
        assert_eq!(snapshot, ControllerSnapshot::neutral());
    }

    #[test]
    fn default_socd_is_second_input_priority_without_reactivation() {
        let mut engine = MeleeEngine::default();

        press(&mut engine, BindingId::AnalogRight);
        let snapshot = press(&mut engine, BindingId::AnalogLeft);
        approx_eq(snapshot.main_x, normalize_axis(-1.0));

        let snapshot = release(&mut engine, BindingId::AnalogLeft);
        approx_eq(snapshot.main_x, 0.5);
    }

    #[test]
    fn second_input_priority_reactivates_when_configured() {
        let mut config = MeleeConfig::default();
        config.socd.main_x = SocdMode::SecondInputPriority;
        let mut engine = MeleeEngine::try_new(config).unwrap();

        press(&mut engine, BindingId::AnalogRight);
        press(&mut engine, BindingId::AnalogLeft);
        let snapshot = release(&mut engine, BindingId::AnalogLeft);
        approx_eq(snapshot.main_x, normalize_axis(1.0));
    }

    #[test]
    fn neutral_socd_can_zero_out_horizontal_axis() {
        let mut config = MeleeConfig::default();
        config.socd.main_x = SocdMode::Neutral;
        let mut engine = MeleeEngine::try_new(config).unwrap();

        press(&mut engine, BindingId::AnalogRight);
        let snapshot = press(&mut engine, BindingId::AnalogLeft);
        approx_eq(snapshot.main_x, 0.5);
    }

    #[test]
    fn crouch_walk_os_changes_down_diagonal_coordinate() {
        let mut config = MeleeConfig::default();
        config.down_diagonal = DownDiagonalBehavior::CrouchWalkOs;
        let mut engine = MeleeEngine::try_new(config).unwrap();

        press(&mut engine, BindingId::AnalogDown);
        let snapshot = press(&mut engine, BindingId::AnalogRight);
        approx_eq(snapshot.main_x, normalize_axis(0.7));
        approx_eq(snapshot.main_y, normalize_axis(-0.6875));
    }

    #[test]
    fn default_down_diagonal_preserves_auto_jab_cancel_coordinate() {
        let mut engine = MeleeEngine::default();

        press(&mut engine, BindingId::AnalogDown);
        let snapshot = press(&mut engine, BindingId::AnalogRight);
        approx_eq(snapshot.main_x, normalize_axis(0.7));
        approx_eq(snapshot.main_y, normalize_axis(-0.7));
    }

    #[test]
    fn default_horizontal_socd_override_matches_haybox_ledgedash_behavior() {
        let mut engine = MeleeEngine::default();

        press(&mut engine, BindingId::AnalogRight);
        press(&mut engine, BindingId::AnalogLeft);
        let snapshot = press(&mut engine, BindingId::ModX);
        approx_eq(snapshot.main_x, normalize_axis(-1.0));
        approx_eq(snapshot.main_y, 0.5);
    }

    #[test]
    fn disabling_horizontal_socd_override_restores_modifier_coordinate() {
        let mut config = MeleeConfig::default();
        config.horizontal_socd_override = HorizontalSocdOverride::Disabled;
        let mut engine = MeleeEngine::try_new(config).unwrap();

        press(&mut engine, BindingId::AnalogRight);
        press(&mut engine, BindingId::AnalogLeft);
        let snapshot = press(&mut engine, BindingId::ModX);
        approx_eq(snapshot.main_x, normalize_axis(-COORDS_HORIZONTAL_MOD_X.0));
        approx_eq(snapshot.main_y, 0.5);
    }

    #[test]
    fn custom_airdodge_overrides_mod_x_shield_diagonal() {
        let mut config = MeleeConfig::default();
        config.airdodge = AirdodgeConfig::CustomModXDiagonal { x: 0.6, y: 0.4 };
        let mut engine = MeleeEngine::try_new(config).unwrap();

        press(&mut engine, BindingId::L);
        press(&mut engine, BindingId::AnalogUp);
        press(&mut engine, BindingId::AnalogRight);
        let snapshot = press(&mut engine, BindingId::ModX);
        approx_eq(snapshot.main_x, normalize_axis(0.6));
        approx_eq(snapshot.main_y, normalize_axis(0.4));
    }

    #[test]
    fn default_airdodge_keeps_reference_coordinate() {
        let mut engine = MeleeEngine::default();

        press(&mut engine, BindingId::L);
        press(&mut engine, BindingId::AnalogUp);
        press(&mut engine, BindingId::AnalogRight);
        let snapshot = press(&mut engine, BindingId::ModX);
        approx_eq(
            snapshot.main_x,
            normalize_axis(COORDS_AIRDODGE_QUADRANT_MOD_X.0),
        );
        approx_eq(
            snapshot.main_y,
            normalize_axis(COORDS_AIRDODGE_QUADRANT_MOD_X.1),
        );
    }

    #[test]
    fn c_stick_diagonal_matches_reference_coords() {
        let mut engine = MeleeEngine::default();

        press(&mut engine, BindingId::CUp);
        let snapshot = press(&mut engine, BindingId::CRight);
        approx_eq(snapshot.c_x, normalize_axis(COORDS_CSTICK_DIAGONAL.0));
        approx_eq(snapshot.c_y, normalize_axis(COORDS_CSTICK_DIAGONAL.1));
    }

    #[test]
    fn custom_airdodge_validation_rejects_zero_axis() {
        let config = MeleeConfig {
            airdodge: AirdodgeConfig::CustomModXDiagonal { x: 0.0, y: 0.4 },
            ..MeleeConfig::default()
        };

        assert!(MeleeEngine::try_new(config).is_err());
    }
}
