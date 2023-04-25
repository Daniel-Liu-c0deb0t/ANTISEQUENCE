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
    pub fn new(s: &str) -> Self {
        let split = s.split('.').collect::<Vec<_>>();

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
    pub fn new(s: &str) -> Self {
        let split = s.split('.').collect::<Vec<_>>();

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
    pub fn new(s: &str) -> Self {
        let count = s.chars().filter(|&c| c == '.').count();

        match count {
            1 => LabelOrAttr::Label(Label::new(s)),
            2 => LabelOrAttr::Attr(Attr::new(s)),
            _ => panic!("Expected either type.label or type.label.attr!"),
        }
    }
}
