use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
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
