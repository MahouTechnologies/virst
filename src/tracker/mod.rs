mod system;

use std::collections::BTreeMap;

pub use system::TrackerSystem;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputBoneKind {
    X,
    Y,
    Z,
    Roll,
    Pitch,
    Yaw,
}

impl InputBoneKind {
    pub fn name(&self) -> &str {
        match self {
            InputBoneKind::X => "X",
            InputBoneKind::Y => "Y",
            InputBoneKind::Z => "Z",
            InputBoneKind::Roll => "Roll",
            InputBoneKind::Pitch => "Pitch",
            InputBoneKind::Yaw => "Yaw",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputKind {
    None,
    Blendshape(String),
    Bone(String, InputBoneKind),
}

impl InputKind {
    pub fn name(&self) -> String {
        match self {
            InputKind::None => "<none>".to_string(),
            InputKind::Blendshape(name) => name.to_string(),
            InputKind::Bone(name, kind) => {
                format!("{} ({})", name, kind.name())
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BindingKind {
    Expr,
    Simple {
        input: InputKind,
        input_range: (f32, f32),
        output_range: (f32, f32),
        dampen: f32,
    },
}

impl BindingKind {
    pub const fn simple() -> BindingKind {
        BindingKind::Simple {
            input: InputKind::None,
            input_range: (-30.0, 30.0),
            output_range: (-1.0, 1.0),
            dampen: 0.0,
        }
    }

    pub const fn expr() -> BindingKind {
        BindingKind::Expr
    }
}

#[derive(Debug)]
pub enum ParamBinding {
    OneDim(Option<BindingKind>),
    TwoDim(Option<(BindingKind, BindingKind)>),
}

impl ParamBinding {
    pub fn default_binding(&mut self) {
        match self {
            ParamBinding::OneDim(v) => *v = Some(BindingKind::simple()),
            ParamBinding::TwoDim(v) => *v = Some((BindingKind::simple(), BindingKind::simple())),
        }
    }
    pub fn clear_binding(&mut self) {
        match self {
            ParamBinding::OneDim(v) => *v = None,
            ParamBinding::TwoDim(v) => *v = None,
        }
    }

    pub fn is_bound(&self) -> bool {
        match self {
            ParamBinding::OneDim(v) => v.is_some(),
            ParamBinding::TwoDim(v) => v.is_some(),
        }
    }

    pub fn is_unbound(&self) -> bool {
        match self {
            ParamBinding::OneDim(v) => v.is_none(),
            ParamBinding::TwoDim(v) => v.is_none(),
        }
    }
}

pub type ParamBindings = BTreeMap<String, ParamBinding>;
