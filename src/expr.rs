pub mod selector;
pub use selector::*;

pub mod format;
pub use format::*;

pub mod transform;
pub use transform::*;

use crate::inline_string::*;
use crate::read::*;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Label {
    pub str_type: StrType,
    pub label: InlineString,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Attr {
    pub str_type: StrType,
    pub label: InlineString,
    pub attr: InlineString,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LabelOrAttr {
    Label(Label),
    Attr(Attr),
}

impl Label {
    pub fn new(s: &[u8]) -> Self {
        let split = s.split(|&b| b == b'.').collect::<Vec<_>>();

        match split.as_slice() {
            &[str_type, label] => Self {
                str_type: StrType::new(str_type),
                label: InlineString::new(label),
            },
            _ => panic!("Expected type.label!"),
        }
    }
}

impl Attr {
    pub fn new(s: &[u8]) -> Self {
        let split = s.split(|&b| b == b'.').collect::<Vec<_>>();

        match split.as_slice() {
            &[str_type, label, attr] => Self {
                str_type: StrType::new(str_type),
                label: InlineString::new(label),
                attr: InlineString::new(attr),
            },
            _ => panic!("Expected type.label.attr!"),
        }
    }
}

impl LabelOrAttr {
    pub fn new(s: &[u8]) -> Self {
        let count = s.iter().filter(|&&c| c == b'.').count();

        match count {
            1 => LabelOrAttr::Label(Label::new(s)),
            2 => LabelOrAttr::Attr(Attr::new(s)),
            _ => panic!("Expected either type.label or type.label.attr!"),
        }
    }

    pub fn str_type(&self) -> StrType {
        match self {
            LabelOrAttr::Label(l) => l.str_type,
            LabelOrAttr::Attr(a) => a.str_type,
        }
    }

    pub fn label(&self) -> InlineString {
        match self {
            LabelOrAttr::Label(l) => l.label,
            LabelOrAttr::Attr(a) => a.label,
        }
    }
}

impl From<Label> for LabelOrAttr {
    fn from(label: Label) -> Self {
        LabelOrAttr::Label(label)
    }
}

impl From<Attr> for LabelOrAttr {
    fn from(attr: Attr) -> Self {
        LabelOrAttr::Attr(attr)
    }
}

#[macro_export]
macro_rules! sel {
    ($($t:tt)*) => {
        {
            let s = stringify!($($t)*);
            $crate::expr::SelectorExpr::new(s.as_bytes())
        }
    };
}

#[macro_export]
macro_rules! tr {
    ($($t:tt)+) => {
        {
            let s = stringify!($($t)+);
            $crate::expr::TransformExpr::new(s.as_bytes())
        }
    };
}

#[macro_export]
macro_rules! label {
    ($($t:tt)+) => {
        {
            let s = stringify!($($t)+);
            $crate::expr::Label::new(s.as_bytes())
        }
    };
}

#[macro_export]
macro_rules! attr {
    ($($t:tt)+) => {
        {
            let s = stringify!($($t)+);
            $crate::expr::Attr::new(s.as_bytes())
        }
    };
}
