use serde::{Deserialize, Serialize};

const COORDS_ORIGIN: (f64, f64) = (0.0, 0.0);
const COORDS_VERTICAL: (f64, f64) = (0.0, 1.0);
const COORDS_VERTICAL_MOD_X: (f64, f64) = (0.0, 0.5375);
const COORDS_VERTICAL_MOD_Y: (f64, f64) = (0.0, 0.7375);
const COORDS_HORIZONTAL: (f64, f64) = (1.0, 0.0);
const COORDS_HORIZONTAL_MOD_X: (f64, f64) = (0.6625, 0.0);
const COORDS_HORIZONTAL_MOD_Y: (f64, f64) = (0.3375, 0.0);
const COORDS_QUADRANT: (f64, f64) = (0.7, 0.7);
const COORDS_QUADRANT_MOD_X: (f64, f64) = (0.7375, 0.3125);
const COORDS_QUADRANT_MOD_Y: (f64, f64) = (0.3125, 0.7375);
const COORDS_AIRDODGE_HORIZONTAL: (f64, f64) = COORDS_HORIZONTAL;
const COORDS_AIRDODGE_HORIZONTAL_MOD_X: (f64, f64) = COORDS_HORIZONTAL_MOD_X;
const COORDS_AIRDODGE_HORIZONTAL_MOD_Y: (f64, f64) = COORDS_HORIZONTAL_MOD_Y;
const COORDS_AIRDODGE_VERTICAL: (f64, f64) = COORDS_VERTICAL;
const COORDS_AIRDODGE_VERTICAL_MOD_X: (f64, f64) = COORDS_VERTICAL_MOD_X;
const COORDS_AIRDODGE_VERTICAL_MOD_Y: (f64, f64) = COORDS_VERTICAL_MOD_Y;
const COORDS_AIRDODGE_QUADRANT: (f64, f64) = (0.7, 0.6875);
const COORDS_AIRDODGE_QUADRANT_12: (f64, f64) = (0.7, 0.7);
const COORDS_AIRDODGE_QUADRANT_34: (f64, f64) = COORDS_AIRDODGE_QUADRANT;
const COORDS_AIRDODGE_QUADRANT_MOD_X: (f64, f64) = (0.6375, 0.375);
const COORDS_AIRDODGE_QUADRANT_12_MOD_Y: (f64, f64) = (0.475, 0.875);
const COORDS_AIRDODGE_QUADRANT_34_MOD_Y: (f64, f64) = (0.5, 0.85);
const COORDS_FIREFOX_MOD_X_C_DOWN: (f64, f64) = (0.7, 0.3625);
const COORDS_FIREFOX_MOD_X_C_LEFT: (f64, f64) = (0.7875, 0.4875);
const COORDS_FIREFOX_MOD_X_C_UP: (f64, f64) = (0.7, 0.5125);
const COORDS_FIREFOX_MOD_X_C_RIGHT: (f64, f64) = (0.6125, 0.525);
const COORDS_FIREFOX_MOD_Y_C_RIGHT: (f64, f64) = (0.6375, 0.7625);
const COORDS_FIREFOX_MOD_Y_C_UP: (f64, f64) = (0.5125, 0.7);
const COORDS_FIREFOX_MOD_Y_C_LEFT: (f64, f64) = (0.4875, 0.7875);
const COORDS_FIREFOX_MOD_Y_C_DOWN: (f64, f64) = (0.3625, 0.7);
const COORDS_EXT_FIREFOX_MOD_X: (f64, f64) = (0.9125, 0.3875);
const COORDS_EXT_FIREFOX_MOD_X_C_DOWN: (f64, f64) = (0.875, 0.45);
const COORDS_EXT_FIREFOX_MOD_X_C_LEFT: (f64, f64) = (0.85, 0.525);
const COORDS_EXT_FIREFOX_MOD_X_C_UP: (f64, f64) = (0.7375, 0.5375);
const COORDS_EXT_FIREFOX_MOD_X_C_RIGHT: (f64, f64) = (0.6375, 0.5375);
const COORDS_EXT_FIREFOX_MOD_Y_C_RIGHT: (f64, f64) = (0.5875, 0.7125);
const COORDS_EXT_FIREFOX_MOD_Y_C_UP: (f64, f64) = (0.5875, 0.8);
const COORDS_EXT_FIREFOX_MOD_Y_C_LEFT: (f64, f64) = (0.525, 0.85);
const COORDS_EXT_FIREFOX_MOD_Y_C_DOWN: (f64, f64) = (0.45, 0.875);
const COORDS_EXT_FIREFOX_MOD_Y: (f64, f64) = (0.3875, 0.9125);

const LIGHT_SHIELD_VALUE: u8 = 49;
const MID_SHIELD_VALUE: u8 = 94;
const TRIGGER_AXIS_MAX: f64 = 255.0;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum BindingId {
    AnalogUp,
    AnalogDown,
    AnalogLeft,
    AnalogRight,
    ModX,
    ModY,
    A,
    B,
    L,
    R,
    X,
    Y,
    Z,
    CUp,
    CDown,
    CLeft,
    CRight,
    LightShield,
    MidShield,
    Start,
    DUp,
    DDown,
    DLeft,
    DRight,
}

impl BindingId {
    pub const ALL: [BindingId; 24] = [
        BindingId::AnalogUp,
        BindingId::AnalogDown,
        BindingId::AnalogLeft,
        BindingId::AnalogRight,
        BindingId::ModX,
        BindingId::ModY,
        BindingId::A,
        BindingId::B,
        BindingId::L,
        BindingId::R,
        BindingId::X,
        BindingId::Y,
        BindingId::Z,
        BindingId::CUp,
        BindingId::CDown,
        BindingId::CLeft,
        BindingId::CRight,
        BindingId::LightShield,
        BindingId::MidShield,
        BindingId::Start,
        BindingId::DUp,
        BindingId::DDown,
        BindingId::DLeft,
        BindingId::DRight,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            BindingId::AnalogUp => "Analog Up",
            BindingId::AnalogDown => "Analog Down",
            BindingId::AnalogLeft => "Analog Left",
            BindingId::AnalogRight => "Analog Right",
            BindingId::ModX => "ModX",
            BindingId::ModY => "ModY",
            BindingId::A => "A",
            BindingId::B => "B",
            BindingId::L => "L",
            BindingId::R => "R",
            BindingId::X => "X",
            BindingId::Y => "Y",
            BindingId::Z => "Z",
            BindingId::CUp => "C-stick Up",
            BindingId::CDown => "C-stick Down",
            BindingId::CLeft => "C-stick Left",
            BindingId::CRight => "C-stick Right",
            BindingId::LightShield => "Light Shield",
            BindingId::MidShield => "Mid Shield",
            BindingId::Start => "Start",
            BindingId::DUp => "D-pad Up",
            BindingId::DDown => "D-pad Down",
            BindingId::DLeft => "D-pad Left",
            BindingId::DRight => "D-pad Right",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputEvent {
    pub binding: BindingId,
    pub pressed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControllerSnapshot {
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub z: bool,
    pub start: bool,
    pub l: bool,
    pub r: bool,
    pub d_up: bool,
    pub d_down: bool,
    pub d_left: bool,
    pub d_right: bool,
    pub l_analog: f64,
    pub r_analog: f64,
    pub main_x: f64,
    pub main_y: f64,
    pub c_x: f64,
    pub c_y: f64,
}

impl Default for ControllerSnapshot {
    fn default() -> Self {
        Self::neutral()
    }
}

impl ControllerSnapshot {
    pub const fn neutral() -> Self {
        Self {
            a: false,
            b: false,
            x: false,
            y: false,
            z: false,
            start: false,
            l: false,
            r: false,
            d_up: false,
            d_down: false,
            d_left: false,
            d_right: false,
            l_analog: 0.0,
            r_analog: 0.0,
            main_x: 0.5,
            main_y: 0.5,
            c_x: 0.5,
            c_y: 0.5,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct B0xxState {
    button_up: bool,
    button_down: bool,
    button_left: bool,
    button_right: bool,
    button_a: bool,
    button_b: bool,
    button_l: bool,
    button_r: bool,
    button_x: bool,
    button_y: bool,
    button_z: bool,
    button_start: bool,
    button_light_shield: bool,
    button_mid_shield: bool,
    button_mod_x: bool,
    button_mod_y: bool,
    button_c_up: bool,
    button_c_down: bool,
    button_c_left: bool,
    button_c_right: bool,
    dpad_up_explicit: bool,
    dpad_down_explicit: bool,
    dpad_left_explicit: bool,
    dpad_right_explicit: bool,
    most_recent_vertical: Option<Vertical>,
    most_recent_horizontal: Option<Horizontal>,
    most_recent_vertical_c: Option<Vertical>,
    most_recent_horizontal_c: Option<Horizontal>,
    simultaneous_horizontal_modifier_lockout: bool,
    analog_shield_value: u8,
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

impl B0xxState {
    fn up(&self) -> bool {
        self.button_up && self.most_recent_vertical == Some(Vertical::Up)
    }

    fn down(&self) -> bool {
        self.button_down && self.most_recent_vertical == Some(Vertical::Down)
    }

    fn left(&self) -> bool {
        self.button_left && self.most_recent_horizontal == Some(Horizontal::Left)
    }

    fn right(&self) -> bool {
        self.button_right && self.most_recent_horizontal == Some(Horizontal::Right)
    }

    fn c_up(&self) -> bool {
        self.button_c_up && self.most_recent_vertical_c == Some(Vertical::Up) && !self.both_mods()
    }

    fn c_down(&self) -> bool {
        self.button_c_down
            && self.most_recent_vertical_c == Some(Vertical::Down)
            && !self.both_mods()
    }

    fn c_left(&self) -> bool {
        self.button_c_left
            && self.most_recent_horizontal_c == Some(Horizontal::Left)
            && !self.both_mods()
    }

    fn c_right(&self) -> bool {
        self.button_c_right
            && self.most_recent_horizontal_c == Some(Horizontal::Right)
            && !self.both_mods()
    }

    fn mod_x(&self) -> bool {
        self.button_mod_x
            && !self.button_mod_y
            && !(self.simultaneous_horizontal_modifier_lockout && !self.any_vert())
    }

    fn mod_y(&self) -> bool {
        self.button_mod_y
            && !self.button_mod_x
            && !(self.simultaneous_horizontal_modifier_lockout && !self.any_vert())
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
        self.button_mod_x && self.button_mod_y
    }

    fn any_shield(&self) -> bool {
        self.button_l || self.button_r || self.button_light_shield || self.button_mid_shield
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

    fn get_analog_coords(&self) -> (f64, f64) {
        let coords = if self.any_shield() {
            self.get_analog_coords_airdodge()
        } else if self.any_mod() && self.any_quadrant() && (self.any_c() || self.button_b) {
            self.get_analog_coords_firefox()
        } else {
            self.get_analog_coords_no_shield()
        };

        self.reflect_coords(coords)
    }

    fn reflect_coords(&self, (mut x, mut y): (f64, f64)) -> (f64, f64) {
        if self.down() {
            y = -y;
        }
        if self.left() {
            x = -x;
        }
        (x, y)
    }

    fn get_analog_coords_airdodge(&self) -> (f64, f64) {
        if !self.any_vert() && !self.any_horiz() {
            COORDS_ORIGIN
        } else if self.any_quadrant() {
            if self.mod_x() {
                COORDS_AIRDODGE_QUADRANT_MOD_X
            } else if self.mod_y() {
                if self.up() {
                    COORDS_AIRDODGE_QUADRANT_12_MOD_Y
                } else {
                    COORDS_AIRDODGE_QUADRANT_34_MOD_Y
                }
            } else if self.up() {
                COORDS_AIRDODGE_QUADRANT_12
            } else {
                COORDS_AIRDODGE_QUADRANT_34
            }
        } else if self.any_vert() {
            if self.mod_x() {
                COORDS_AIRDODGE_VERTICAL_MOD_X
            } else if self.mod_y() {
                COORDS_AIRDODGE_VERTICAL_MOD_Y
            } else {
                COORDS_AIRDODGE_VERTICAL
            }
        } else if self.mod_x() {
            COORDS_AIRDODGE_HORIZONTAL_MOD_X
        } else if self.mod_y() {
            if self.button_b {
                COORDS_AIRDODGE_HORIZONTAL
            } else {
                COORDS_AIRDODGE_HORIZONTAL_MOD_Y
            }
        } else {
            COORDS_AIRDODGE_HORIZONTAL
        }
    }

    fn get_analog_coords_no_shield(&self) -> (f64, f64) {
        if !self.any_vert() && !self.any_horiz() {
            COORDS_ORIGIN
        } else if self.any_quadrant() {
            if self.mod_x() {
                COORDS_QUADRANT_MOD_X
            } else if self.mod_y() {
                COORDS_QUADRANT_MOD_Y
            } else {
                COORDS_QUADRANT
            }
        } else if self.any_vert() {
            if self.mod_x() {
                COORDS_VERTICAL_MOD_X
            } else if self.mod_y() {
                COORDS_VERTICAL_MOD_Y
            } else {
                COORDS_VERTICAL
            }
        } else if self.mod_x() {
            COORDS_HORIZONTAL_MOD_X
        } else if self.mod_y() {
            if self.button_b {
                COORDS_HORIZONTAL
            } else {
                COORDS_HORIZONTAL_MOD_Y
            }
        } else {
            COORDS_HORIZONTAL
        }
    }

    fn get_analog_coords_firefox(&self) -> (f64, f64) {
        if self.mod_x() {
            if self.c_up() {
                if self.button_b {
                    COORDS_EXT_FIREFOX_MOD_X_C_UP
                } else {
                    COORDS_FIREFOX_MOD_X_C_UP
                }
            } else if self.c_down() {
                if self.button_b {
                    COORDS_EXT_FIREFOX_MOD_X_C_DOWN
                } else {
                    COORDS_FIREFOX_MOD_X_C_DOWN
                }
            } else if self.c_left() {
                if self.button_b {
                    COORDS_EXT_FIREFOX_MOD_X_C_LEFT
                } else {
                    COORDS_FIREFOX_MOD_X_C_LEFT
                }
            } else if self.c_right() {
                if self.button_b {
                    COORDS_EXT_FIREFOX_MOD_X_C_RIGHT
                } else {
                    COORDS_FIREFOX_MOD_X_C_RIGHT
                }
            } else {
                COORDS_EXT_FIREFOX_MOD_X
            }
        } else if self.mod_y() {
            if self.c_up() {
                if self.button_b {
                    COORDS_EXT_FIREFOX_MOD_Y_C_UP
                } else {
                    COORDS_FIREFOX_MOD_Y_C_UP
                }
            } else if self.c_down() {
                if self.button_b {
                    COORDS_EXT_FIREFOX_MOD_Y_C_DOWN
                } else {
                    COORDS_FIREFOX_MOD_Y_C_DOWN
                }
            } else if self.c_left() {
                if self.button_b {
                    COORDS_EXT_FIREFOX_MOD_Y_C_LEFT
                } else {
                    COORDS_FIREFOX_MOD_Y_C_LEFT
                }
            } else if self.c_right() {
                if self.button_b {
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

    fn get_c_stick_coords(&self) -> (f64, f64) {
        let coords = if !self.any_vert_c() && !self.any_horiz_c() {
            COORDS_ORIGIN
        } else if self.any_vert_c() && self.any_horiz_c() {
            (0.525, 0.85)
        } else if self.any_vert_c() {
            COORDS_VERTICAL
        } else if self.mod_x() && self.up() {
            (0.9, 0.5)
        } else if self.mod_x() && self.down() {
            (0.9, -0.5)
        } else {
            COORDS_HORIZONTAL
        };

        self.reflect_c_stick_coords(coords)
    }

    fn reflect_c_stick_coords(&self, (mut x, mut y): (f64, f64)) -> (f64, f64) {
        if self.c_down() {
            y = -y;
        }
        if self.c_left() {
            x = -x;
        }
        (x, y)
    }

    fn snapshot(&self) -> ControllerSnapshot {
        let (main_x, main_y) = self.get_analog_coords();
        let (c_x, c_y) = self.get_c_stick_coords();

        ControllerSnapshot {
            a: self.button_a,
            b: self.button_b,
            x: self.button_x,
            y: self.button_y,
            z: self.button_z,
            start: self.button_start,
            l: self.button_l,
            r: self.button_r,
            d_up: self.dpad_up_explicit || (self.both_mods() && self.button_c_up),
            d_down: self.dpad_down_explicit || (self.both_mods() && self.button_c_down),
            d_left: self.dpad_left_explicit || (self.both_mods() && self.button_c_left),
            d_right: self.dpad_right_explicit || (self.both_mods() && self.button_c_right),
            l_analog: 0.0,
            r_analog: normalize_trigger(self.analog_shield_value),
            main_x: normalize_axis(main_x),
            main_y: normalize_axis(main_y),
            c_x: normalize_axis(c_x),
            c_y: normalize_axis(c_y),
        }
    }
}

fn normalize_axis(value: f64) -> f64 {
    ((value + 1.0) / 2.0).clamp(0.0, 1.0)
}

fn normalize_trigger(value: u8) -> f64 {
    (f64::from(value) / TRIGGER_AXIS_MAX).clamp(0.0, 1.0)
}

#[derive(Clone, Debug, Default)]
pub struct B0xxEngine {
    state: B0xxState,
}

impl B0xxEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_event(&mut self, event: InputEvent) -> ControllerSnapshot {
        let state = &mut self.state;

        match event.binding {
            BindingId::AnalogUp => {
                state.button_up = event.pressed;
                if event.pressed {
                    state.most_recent_vertical = Some(Vertical::Up);
                }
            }
            BindingId::AnalogDown => {
                state.button_down = event.pressed;
                if event.pressed {
                    state.most_recent_vertical = Some(Vertical::Down);
                }
            }
            BindingId::AnalogLeft => {
                state.button_left = event.pressed;
                if event.pressed {
                    state.most_recent_horizontal = Some(Horizontal::Left);
                    if state.button_right {
                        state.simultaneous_horizontal_modifier_lockout = true;
                    }
                } else {
                    state.simultaneous_horizontal_modifier_lockout = false;
                }
            }
            BindingId::AnalogRight => {
                state.button_right = event.pressed;
                if event.pressed {
                    state.most_recent_horizontal = Some(Horizontal::Right);
                    if state.button_left {
                        state.simultaneous_horizontal_modifier_lockout = true;
                    }
                } else {
                    state.simultaneous_horizontal_modifier_lockout = false;
                }
            }
            BindingId::ModX => {
                state.button_mod_x = event.pressed;
                state.simultaneous_horizontal_modifier_lockout = false;
            }
            BindingId::ModY => {
                state.button_mod_y = event.pressed;
            }
            BindingId::A => state.button_a = event.pressed,
            BindingId::B => state.button_b = event.pressed,
            BindingId::L => state.button_l = event.pressed,
            BindingId::R => state.button_r = event.pressed,
            BindingId::X => state.button_x = event.pressed,
            BindingId::Y => state.button_y = event.pressed,
            BindingId::Z => state.button_z = event.pressed,
            BindingId::CUp => {
                state.button_c_up = event.pressed;
                if event.pressed && !state.both_mods() {
                    state.most_recent_vertical_c = Some(Vertical::Up);
                }
            }
            BindingId::CDown => {
                state.button_c_down = event.pressed;
                if event.pressed && !state.both_mods() {
                    state.most_recent_vertical_c = Some(Vertical::Down);
                }
            }
            BindingId::CLeft => {
                state.button_c_left = event.pressed;
                if event.pressed && !state.both_mods() {
                    state.most_recent_horizontal_c = Some(Horizontal::Left);
                }
            }
            BindingId::CRight => {
                state.button_c_right = event.pressed;
                if event.pressed && !state.both_mods() {
                    state.most_recent_horizontal_c = Some(Horizontal::Right);
                }
            }
            BindingId::LightShield => {
                state.button_light_shield = event.pressed;
                state.analog_shield_value = if event.pressed { LIGHT_SHIELD_VALUE } else { 0 };
            }
            BindingId::MidShield => {
                state.button_mid_shield = event.pressed;
                state.analog_shield_value = if event.pressed { MID_SHIELD_VALUE } else { 0 };
            }
            BindingId::Start => state.button_start = event.pressed,
            BindingId::DUp => state.dpad_up_explicit = event.pressed,
            BindingId::DDown => state.dpad_down_explicit = event.pressed,
            BindingId::DLeft => state.dpad_left_explicit = event.pressed,
            BindingId::DRight => state.dpad_right_explicit = event.pressed,
        }

        state.snapshot()
    }

    pub fn snapshot(&self) -> ControllerSnapshot {
        self.state.snapshot()
    }

    pub fn reset(&mut self) -> ControllerSnapshot {
        self.state = B0xxState::default();
        self.state.snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(lhs: f64, rhs: f64) {
        assert!((lhs - rhs).abs() < 1e-9, "{lhs} != {rhs}");
    }

    #[test]
    fn neutral_snapshot_is_centered() {
        let snapshot = B0xxEngine::new().snapshot();
        assert_eq!(snapshot, ControllerSnapshot::neutral());
    }

    #[test]
    fn analog_up_normalizes_to_top() {
        let mut engine = B0xxEngine::new();
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::AnalogUp,
            pressed: true,
        });
        approx_eq(snapshot.main_x, 0.5);
        approx_eq(snapshot.main_y, 1.0);
    }

    #[test]
    fn analog_down_reflects_to_bottom() {
        let mut engine = B0xxEngine::new();
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::AnalogDown,
            pressed: true,
        });
        approx_eq(snapshot.main_x, 0.5);
        approx_eq(snapshot.main_y, 0.0);
    }

    #[test]
    fn analog_left_reflects_to_zero() {
        let mut engine = B0xxEngine::new();
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::AnalogLeft,
            pressed: true,
        });
        approx_eq(snapshot.main_x, 0.0);
        approx_eq(snapshot.main_y, 0.5);
    }

    #[test]
    fn analog_right_normalizes_to_one() {
        let mut engine = B0xxEngine::new();
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::AnalogRight,
            pressed: true,
        });
        approx_eq(snapshot.main_x, 1.0);
        approx_eq(snapshot.main_y, 0.5);
    }

    #[test]
    fn socd_last_input_wins_for_horizontal() {
        let mut engine = B0xxEngine::new();
        engine.handle_event(InputEvent {
            binding: BindingId::AnalogRight,
            pressed: true,
        });
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::AnalogLeft,
            pressed: true,
        });
        approx_eq(snapshot.main_x, 0.0);
    }

    #[test]
    fn socd_last_input_wins_for_vertical() {
        let mut engine = B0xxEngine::new();
        engine.handle_event(InputEvent {
            binding: BindingId::AnalogUp,
            pressed: true,
        });
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::AnalogDown,
            pressed: true,
        });
        approx_eq(snapshot.main_y, 0.0);
    }

    #[test]
    fn mod_x_clears_horizontal_lockout_like_python_reference() {
        let mut engine = B0xxEngine::new();
        engine.handle_event(InputEvent {
            binding: BindingId::AnalogRight,
            pressed: true,
        });
        engine.handle_event(InputEvent {
            binding: BindingId::AnalogLeft,
            pressed: true,
        });
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::ModX,
            pressed: true,
        });
        approx_eq(snapshot.main_x, normalize_axis(-COORDS_HORIZONTAL_MOD_X.0));
        approx_eq(snapshot.main_y, 0.5);
    }

    #[test]
    fn mod_x_up_uses_vertical_mod_x_coordinate() {
        let mut engine = B0xxEngine::new();
        engine.handle_event(InputEvent {
            binding: BindingId::AnalogUp,
            pressed: true,
        });
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::ModX,
            pressed: true,
        });
        approx_eq(snapshot.main_x, 0.5);
        approx_eq(snapshot.main_y, normalize_axis(COORDS_VERTICAL_MOD_X.1));
    }

    #[test]
    fn mod_y_horizontal_side_b_nerf_uses_full_horizontal() {
        let mut engine = B0xxEngine::new();
        engine.handle_event(InputEvent {
            binding: BindingId::AnalogRight,
            pressed: true,
        });
        engine.handle_event(InputEvent {
            binding: BindingId::ModY,
            pressed: true,
        });
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::B,
            pressed: true,
        });
        approx_eq(snapshot.main_x, 1.0);
        approx_eq(snapshot.main_y, 0.5);
    }

    #[test]
    fn quadrant_mod_x_matches_reference_coords() {
        let mut engine = B0xxEngine::new();
        engine.handle_event(InputEvent {
            binding: BindingId::AnalogUp,
            pressed: true,
        });
        engine.handle_event(InputEvent {
            binding: BindingId::AnalogRight,
            pressed: true,
        });
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::ModX,
            pressed: true,
        });
        approx_eq(snapshot.main_x, normalize_axis(COORDS_QUADRANT_MOD_X.0));
        approx_eq(snapshot.main_y, normalize_axis(COORDS_QUADRANT_MOD_X.1));
    }

    #[test]
    fn shield_quadrant_mod_y_matches_airdodge_reference() {
        let mut engine = B0xxEngine::new();
        engine.handle_event(InputEvent {
            binding: BindingId::L,
            pressed: true,
        });
        engine.handle_event(InputEvent {
            binding: BindingId::AnalogUp,
            pressed: true,
        });
        engine.handle_event(InputEvent {
            binding: BindingId::AnalogRight,
            pressed: true,
        });
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::ModY,
            pressed: true,
        });
        approx_eq(snapshot.main_x, normalize_axis(COORDS_AIRDODGE_QUADRANT_12_MOD_Y.0));
        approx_eq(snapshot.main_y, normalize_axis(COORDS_AIRDODGE_QUADRANT_12_MOD_Y.1));
    }

    #[test]
    fn firefox_mod_x_c_up_matches_reference_coords() {
        let mut engine = B0xxEngine::new();
        engine.handle_event(InputEvent {
            binding: BindingId::AnalogUp,
            pressed: true,
        });
        engine.handle_event(InputEvent {
            binding: BindingId::AnalogRight,
            pressed: true,
        });
        engine.handle_event(InputEvent {
            binding: BindingId::ModX,
            pressed: true,
        });
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::CUp,
            pressed: true,
        });
        approx_eq(snapshot.main_x, normalize_axis(COORDS_FIREFOX_MOD_X_C_UP.0));
        approx_eq(snapshot.main_y, normalize_axis(COORDS_FIREFOX_MOD_X_C_UP.1));
    }

    #[test]
    fn firefox_mod_y_extended_angle_with_b_matches_reference_coords() {
        let mut engine = B0xxEngine::new();
        engine.handle_event(InputEvent {
            binding: BindingId::AnalogUp,
            pressed: true,
        });
        engine.handle_event(InputEvent {
            binding: BindingId::AnalogRight,
            pressed: true,
        });
        engine.handle_event(InputEvent {
            binding: BindingId::ModY,
            pressed: true,
        });
        engine.handle_event(InputEvent {
            binding: BindingId::B,
            pressed: true,
        });
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::CRight,
            pressed: true,
        });
        approx_eq(snapshot.main_x, normalize_axis(COORDS_EXT_FIREFOX_MOD_Y_C_RIGHT.0));
        approx_eq(snapshot.main_y, normalize_axis(COORDS_EXT_FIREFOX_MOD_Y_C_RIGHT.1));
    }

    #[test]
    fn c_stick_diagonal_matches_reference_coords() {
        let mut engine = B0xxEngine::new();
        engine.handle_event(InputEvent {
            binding: BindingId::CUp,
            pressed: true,
        });
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::CRight,
            pressed: true,
        });
        approx_eq(snapshot.c_x, normalize_axis(0.525));
        approx_eq(snapshot.c_y, normalize_axis(0.85));
    }

    #[test]
    fn c_stick_mod_x_up_angle_matches_reference_coords() {
        let mut engine = B0xxEngine::new();
        engine.handle_event(InputEvent {
            binding: BindingId::ModX,
            pressed: true,
        });
        engine.handle_event(InputEvent {
            binding: BindingId::AnalogUp,
            pressed: true,
        });
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::CRight,
            pressed: true,
        });
        approx_eq(snapshot.c_x, normalize_axis(0.9));
        approx_eq(snapshot.c_y, normalize_axis(0.5));
    }

    #[test]
    fn both_mods_turn_c_stick_into_dpad() {
        let mut engine = B0xxEngine::new();
        engine.handle_event(InputEvent {
            binding: BindingId::ModX,
            pressed: true,
        });
        engine.handle_event(InputEvent {
            binding: BindingId::ModY,
            pressed: true,
        });
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::CUp,
            pressed: true,
        });
        assert!(snapshot.d_up);
        approx_eq(snapshot.c_x, 0.5);
        approx_eq(snapshot.c_y, 0.5);
    }

    #[test]
    fn explicit_dpad_state_is_preserved() {
        let mut engine = B0xxEngine::new();
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::DLeft,
            pressed: true,
        });
        assert!(snapshot.d_left);
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::DLeft,
            pressed: false,
        });
        assert!(!snapshot.d_left);
    }

    #[test]
    fn action_buttons_flow_to_snapshot() {
        let mut engine = B0xxEngine::new();
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::A,
            pressed: true,
        });
        assert!(snapshot.a);
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::Start,
            pressed: true,
        });
        assert!(snapshot.start);
    }

    #[test]
    fn light_shield_sets_analog_trigger() {
        let mut engine = B0xxEngine::new();
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::LightShield,
            pressed: true,
        });
        approx_eq(snapshot.r_analog, normalize_trigger(LIGHT_SHIELD_VALUE));
    }

    #[test]
    fn mid_shield_sets_analog_trigger() {
        let mut engine = B0xxEngine::new();
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::MidShield,
            pressed: true,
        });
        approx_eq(snapshot.r_analog, normalize_trigger(MID_SHIELD_VALUE));
    }

    #[test]
    fn releasing_shield_returns_analog_trigger_to_zero() {
        let mut engine = B0xxEngine::new();
        engine.handle_event(InputEvent {
            binding: BindingId::MidShield,
            pressed: true,
        });
        let snapshot = engine.handle_event(InputEvent {
            binding: BindingId::MidShield,
            pressed: false,
        });
        approx_eq(snapshot.r_analog, 0.0);
    }

    #[test]
    fn reset_returns_to_neutral() {
        let mut engine = B0xxEngine::new();
        engine.handle_event(InputEvent {
            binding: BindingId::AnalogUp,
            pressed: true,
        });
        engine.handle_event(InputEvent {
            binding: BindingId::A,
            pressed: true,
        });
        let snapshot = engine.reset();
        assert_eq!(snapshot, ControllerSnapshot::neutral());
    }
}
