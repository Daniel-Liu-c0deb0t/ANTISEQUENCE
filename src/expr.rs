pub mod transform;
pub use transform::*;

pub mod node;
pub use node::*;

use crate::errors::*;
use crate::inline_string::*;
use crate::parse_utils::*;
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
    pub fn new(s: &[u8]) -> Result<Self> {
        let split = s.split(|&b| b == b'.').collect::<Vec<_>>();

        match split.as_slice() {
            &[str_type, label] => {
                let str_type =
                    trim_ascii_whitespace(str_type).ok_or_else(|| Error::InvalidName {
                        string: utf8(str_type),
                        context: utf8(s),
                    })?;
                let label = trim_ascii_whitespace(label).ok_or_else(|| Error::InvalidName {
                    string: utf8(label),
                    context: utf8(s),
                })?;
                let label = check_valid_name(label).ok_or_else(|| Error::InvalidName {
                    string: utf8(label),
                    context: utf8(s),
                })?;

                Ok(Self {
                    str_type: StrType::new(str_type)?,
                    label: InlineString::new(label),
                })
            }
            _ => Err(Error::Parse {
                string: utf8(s),
                context: utf8(s),
                reason: "expected type.label",
            }),
        }
    }
}

impl Attr {
    pub fn new(s: &[u8]) -> Result<Self> {
        let split = s.split(|&b| b == b'.').collect::<Vec<_>>();

        match split.as_slice() {
            &[str_type, label, attr] => {
                let str_type =
                    trim_ascii_whitespace(str_type).ok_or_else(|| Error::InvalidName {
                        string: utf8(str_type),
                        context: utf8(s),
                    })?;
                let label = trim_ascii_whitespace(label).ok_or_else(|| Error::InvalidName {
                    string: utf8(label),
                    context: utf8(s),
                })?;
                let label = check_valid_name(label).ok_or_else(|| Error::InvalidName {
                    string: utf8(label),
                    context: utf8(s),
                })?;

                let attr = trim_ascii_whitespace(attr).ok_or_else(|| Error::InvalidName {
                    string: utf8(attr),
                    context: utf8(s),
                })?;
                let attr = check_valid_name(attr).ok_or_else(|| Error::InvalidName {
                    string: utf8(attr),
                    context: utf8(s),
                })?;

                Ok(Self {
                    str_type: StrType::new(str_type)?,
                    label: InlineString::new(label),
                    attr: InlineString::new(attr),
                })
            }
            _ => Err(Error::Parse {
                string: utf8(s),
                context: utf8(s),
                reason: "expected type.label.attr",
            }),
        }
    }
}

impl LabelOrAttr {
    pub fn new(s: &[u8]) -> Result<Self> {
        let count = s.iter().filter(|&&c| c == b'.').count();

        match count {
            1 => Ok(LabelOrAttr::Label(Label::new(s)?)),
            2 => Ok(LabelOrAttr::Attr(Attr::new(s)?)),
            _ => Err(Error::Parse {
                string: utf8(s),
                context: utf8(s),
                reason: "expected type.label or type.label.attr",
            }),
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

/// Create a transform expression.
#[macro_export]
macro_rules! tr {
    ($($t:tt)+) => {
        {
            let s = stringify!($($t)+);
            $crate::expr::TransformExpr::new(s.as_bytes())
                .unwrap_or_else(|e| panic!("Error constructing transform expression:\n{e}\non line {} column {} in file {}", line!(), column!(), file!()))
        }
    };
}

pub fn label(s: impl AsRef<str>) -> Label {
    Label::new(s.as_ref().as_bytes()).unwrap_or_else(|e| panic!("Error creating label:\n{e}"))
}

pub fn attr(s: impl AsRef<str>) -> Attr {
    Attr::new(s.as_ref().as_bytes()).unwrap_or_else(|e| panic!("Error creating attr:\n{e}"))
}
