use std::ops::{Bound, RangeBounds};
use std::marker::{Send, Sync};

use crate::errors::NameError;
use crate::expr::*;
use crate::read::*;

// TODO: use copy on write for bytes

const UNKNOWN_QUAL: u8 = b'I';

pub struct Expr {
    node: Box<dyn ExprNode + Send + Sync>,
}

macro_rules! binary_fn {
    ($fn_name:ident, $struct_name:ident) => {
        pub fn $fn_name(self, o: Expr) -> Expr {
            Expr {
                node: Box::new($struct_name {
                    left: self,
                    right: o,
                }),
            }
        }
    };
}

macro_rules! unary_fn {
    ($fn_name:ident, $struct_name:ident, $field_name:ident) => {
        pub fn $fn_name(self) -> Expr {
            Expr {
                node: Box::new($struct_name { $field_name: self }),
            }
        }
    };
}

impl Expr {
    binary_fn!(and, AndNode);
    binary_fn!(or, OrNode);
    binary_fn!(xor, XorNode);

    binary_fn!(add, AddNode);
    binary_fn!(sub, SubNode);
    binary_fn!(mul, MulNode);
    binary_fn!(div, DivNode);

    binary_fn!(gt, GtNode);
    binary_fn!(lt, LtNode);
    binary_fn!(ge, GeNode);
    binary_fn!(le, LeNode);
    binary_fn!(eq, EqNode);

    binary_fn!(concat, ConcatNode);

    unary_fn!(not, NotNode, boolean);
    unary_fn!(len, LenNode, string);

    unary_fn!(int, IntNode, convert);
    unary_fn!(float, FloatNode, convert);
    unary_fn!(bytes, BytesNode, convert);

    pub fn repeat(self, times: Expr) -> Expr {
        Expr {
            node: Box::new(RepeatNode {
                string: self,
                times,
            }),
        }
    }

    pub fn in_bounds(self, range: impl RangeBounds<Expr> + Send + Sync + 'static) -> Expr {
        Expr {
            node: Box::new(InBoundsNode { num: self, range }),
        }
    }

    pub fn eval_bool(&self, read: &Read) -> std::result::Result<bool, NameError> {
        let res = self.eval(read, false)?;

        if let Data::Bool(b) = res {
            Ok(b)
        } else {
            Err(NameError::Type("bool", vec![res]))
        }
    }

    pub fn eval_bytes(&self, read: &Read, use_qual: bool) -> std::result::Result<Vec<u8>, NameError> {
        let res = self.eval(read, use_qual)?;

        if let Data::Bytes(b) = res {
            Ok(b)
        } else {
            Err(NameError::Type("bytes", vec![res]))
        }
    }

    pub fn eval(&self, read: &Read, use_qual: bool) -> std::result::Result<Data, NameError> {
        self.node.eval(read, use_qual)
    }

    pub fn required_names(&self) -> Vec<LabelOrAttr> {
        self.node.required_names()
    }
}

pub trait ExprNode {
    fn eval(&self, read: &Read, use_qual: bool) -> std::result::Result<Data, NameError>;
    fn required_names(&self) -> Vec<LabelOrAttr>;
}

macro_rules! bool_binary_ops {
    ($struct_name:ident, $bool_expr:expr) => {
        struct $struct_name {
            left: Expr,
            right: Expr,
        }

        impl ExprNode for $struct_name {
            fn eval(&self, read: &Read, use_qual: bool) -> std::result::Result<Data, NameError> {
                let left = self.left.eval(read, use_qual)?;
                let right = self.right.eval(read, use_qual)?;

                use Data::*;
                match (left, right) {
                    (Bool(l), Bool(r)) => Ok($bool_expr(l, r)),
                    (l, r) => Err(NameError::Type("bool", vec![l, r])),
                }
            }

            fn required_names(&self) -> Vec<LabelOrAttr> {
                let mut res = self.left.required_names();
                res.append(&mut self.right.required_names());
                res
            }
        }
    };
}

bool_binary_ops!(AndNode, |l, r| Bool(l & r));
bool_binary_ops!(OrNode, |l, r| Bool(l | r));
bool_binary_ops!(XorNode, |l, r| Bool(l ^ r));

macro_rules! num_binary_ops {
    ($struct_name:ident, $int_expr:expr, $float_expr:expr) => {
        struct $struct_name {
            left: Expr,
            right: Expr,
        }

        impl ExprNode for $struct_name {
            fn eval(&self, read: &Read, use_qual: bool) -> std::result::Result<Data, NameError> {
                let left = self.left.eval(read, use_qual)?;
                let right = self.right.eval(read, use_qual)?;

                use Data::*;
                match (left, right) {
                    (Int(l), Int(r)) => Ok($int_expr(l, r)),
                    (Float(l), Float(r)) => Ok($float_expr(l, r)),
                    (l, r) => Err(NameError::Type("both int or both float", vec![l, r])),
                }
            }

            fn required_names(&self) -> Vec<LabelOrAttr> {
                let mut res = self.left.required_names();
                res.append(&mut self.right.required_names());
                res
            }
        }
    };
}

num_binary_ops!(AddNode, |l, r| Int(l + r), |l, r| Float(l + r));
num_binary_ops!(SubNode, |l, r| Int(l - r), |l, r| Float(l - r));
num_binary_ops!(MulNode, |l, r| Int(l * r), |l, r| Float(l * r));
num_binary_ops!(DivNode, |l, r| Int(l / r), |l, r| Float(l / r));

num_binary_ops!(GtNode, |l, r| Bool(l > r), |l, r| Bool(l > r));
num_binary_ops!(LtNode, |l, r| Bool(l < r), |l, r| Bool(l < r));
num_binary_ops!(GeNode, |l, r| Bool(l >= r), |l, r| Bool(l >= r));
num_binary_ops!(LeNode, |l, r| Bool(l <= r), |l, r| Bool(l <= r));

struct NotNode {
    boolean: Expr,
}

impl ExprNode for NotNode {
    fn eval(&self, read: &Read, use_qual: bool) -> std::result::Result<Data, NameError> {
        let boolean = self.boolean.eval(read, use_qual)?;
        use Data::*;
        match boolean {
            Bool(b) => Ok(Bool(!b)),
            b => Err(NameError::Type("bool", vec![b])),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        self.boolean.required_names()
    }
}

struct EqNode {
    left: Expr,
    right: Expr,
}

impl ExprNode for EqNode {
    fn eval(&self, read: &Read, use_qual: bool) -> std::result::Result<Data, NameError> {
        let left = self.left.eval(read, use_qual)?;
        let right = self.right.eval(read, use_qual)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Bool(l == r)),
            (Float(l), Float(r)) => Ok(Bool(l == r)),
            (Bool(l), Bool(r)) => Ok(Bool(l == r)),
            (Bytes(l), Bytes(r)) => Ok(Bool(l == r)),
            (l, r) => Err(NameError::Type("both are the same type", vec![l, r])),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        let mut res = self.left.required_names();
        res.append(&mut self.right.required_names());
        res
    }
}

struct LenNode {
    string: Expr,
}

impl ExprNode for LenNode {
    fn eval(&self, read: &Read, use_qual: bool) -> std::result::Result<Data, NameError> {
        let string = self.string.eval(read, use_qual)?;
        use Data::*;
        match string {
            Bytes(s) => Ok(Int(s.len() as isize)),
            s => Err(NameError::Type("bytes", vec![s])),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        self.string.required_names()
    }
}

struct IntNode {
    convert: Expr,
}

impl ExprNode for IntNode {
    fn eval(&self, read: &Read, use_qual: bool) -> std::result::Result<Data, NameError> {
        let convert = self.convert.eval(read, use_qual)?;
        use Data::*;
        match convert {
            Bool(c) => Ok(Int(if c { 1 } else { 0 })),
            Int(c) => Ok(Int(c)),
            Float(c) => Ok(Int(c as isize)),
            Bytes(c) => Ok(Int(std::str::from_utf8(&c)
                .unwrap()
                .parse::<isize>()
                .unwrap_or_else(|e| panic!("{e}")))),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        self.convert.required_names()
    }
}

struct FloatNode {
    convert: Expr,
}

impl ExprNode for FloatNode {
    fn eval(&self, read: &Read, use_qual: bool) -> std::result::Result<Data, NameError> {
        let convert = self.convert.eval(read, use_qual)?;
        use Data::*;
        match convert {
            Bool(c) => Ok(Float(if c { 1.0 } else { 0.0 })),
            Int(c) => Ok(Float(c as f64)),
            Float(c) => Ok(Float(c)),
            Bytes(c) => Ok(Float(
                std::str::from_utf8(&c)
                    .unwrap()
                    .parse::<f64>()
                    .unwrap_or_else(|e| panic!("{e}")),
            )),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        self.convert.required_names()
    }
}

struct BytesNode {
    convert: Expr,
}

impl ExprNode for BytesNode {
    fn eval(&self, read: &Read, use_qual: bool) -> std::result::Result<Data, NameError> {
        let convert = self.convert.eval(read, use_qual)?;
        use Data::*;
        match convert {
            Bool(c) => Ok(Bytes(if c {
                b"true".to_vec()
            } else {
                b"false".to_vec()
            })),
            Int(c) => Ok(Bytes(c.to_string().into_bytes().to_owned())),
            Float(c) => Ok(Bytes(c.to_string().into_bytes().to_owned())),
            Bytes(c) => Ok(Bytes(c)),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        self.convert.required_names()
    }
}

struct RepeatNode {
    string: Expr,
    times: Expr,
}

impl ExprNode for RepeatNode {
    fn eval(&self, read: &Read, use_qual: bool) -> std::result::Result<Data, NameError> {
        let string = self.string.eval(read, use_qual)?;
        let times = self.times.eval(read, use_qual)?;
        use Data::*;
        match (string, times) {
            (Bytes(s), Int(t)) => Ok(Bytes(s.repeat(t as usize))),
            (s, t) => Err(NameError::Type("bytes and int", vec![s, t])),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        let mut res = self.string.required_names();
        res.append(&mut self.times.required_names());
        res
    }
}

struct ConcatNode {
    left: Expr,
    right: Expr,
}

impl ExprNode for ConcatNode {
    fn eval(&self, read: &Read, use_qual: bool) -> std::result::Result<Data, NameError> {
        let left = self.left.eval(read, use_qual)?;
        let right = self.right.eval(read, use_qual)?;
        use Data::*;
        match (left, right) {
            (Bytes(mut l), Bytes(mut r)) => {
                l.append(&mut r);
                Ok(Bytes(l))
            }
            (l, r) => Err(NameError::Type("both bytes", vec![l, r])),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        let mut res = self.left.required_names();
        res.append(&mut self.right.required_names());
        res
    }
}

pub fn concat_all(nodes: impl IntoIterator<Item = Expr>) -> Expr {
    Expr {
        node: Box::new(ConcatAllNode {
            nodes: nodes.into_iter().collect(),
        }),
    }
}

struct ConcatAllNode {
    nodes: Vec<Expr>,
}

impl ExprNode for ConcatAllNode {
    fn eval(&self, read: &Read, use_qual: bool) -> std::result::Result<Data, NameError> {
        let mut res = Vec::new();

        for node in &self.nodes {
            let b = node.eval(read, use_qual)?;

            if let Data::Bytes(mut b) = b {
                res.append(&mut b);
            } else {
                return Err(NameError::Type("bytes", vec![b]));
            }
        }

        Ok(Data::Bytes(res))
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        self.nodes.iter().flat_map(|n| n.required_names()).collect()
    }
}

struct InBoundsNode<R: RangeBounds<Expr> + Send + Sync> {
    num: Expr,
    range: R,
}

impl<R: RangeBounds<Expr> + Send + Sync> ExprNode for InBoundsNode<R> {
    fn eval(&self, read: &Read, use_qual: bool) -> std::result::Result<Data, NameError> {
        let num = self.num.eval(read, use_qual)?;

        use Data::*;
        let mut start_add1 = false;
        let start = match self.range.start_bound() {
            Bound::Included(s) => s.eval(read, use_qual)?,
            Bound::Excluded(s) => {
                start_add1 = true;
                s.eval(read, use_qual)?
            }
            Bound::Unbounded => Int(std::isize::MIN),
        };

        let mut end_sub1 = false;
        let end = match self.range.end_bound() {
            Bound::Included(e) => e.eval(read, use_qual)?,
            Bound::Excluded(e) => {
                end_sub1 = true;
                e.eval(read, use_qual)?
            }
            Bound::Unbounded => Int(std::isize::MAX),
        };

        match (num, start, end) {
            (Int(n), Int(mut s), Int(mut e)) => {
                if start_add1 {
                    s += 1;
                }
                if end_sub1 {
                    e -= 1;
                }
                Ok(Bool(s <= n && n <= e))
            }
            (n, s, e) => Err(NameError::Type("all int", vec![n, s, e])),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        let mut res = self.num.required_names();
        match self.range.start_bound() {
            Bound::Included(s) => res.append(&mut s.required_names()),
            Bound::Excluded(s) => res.append(&mut s.required_names()),
            _ => (),
        }
        match self.range.end_bound() {
            Bound::Included(e) => res.append(&mut e.required_names()),
            Bound::Excluded(e) => res.append(&mut e.required_names()),
            _ => (),
        }
        res
    }
}

impl ExprNode for Label {
    fn eval(&self, read: &Read, use_qual: bool) -> std::result::Result<Data, NameError> {
        Ok(Data::Bytes(
            if use_qual {
                if let Some(qual) = read.substring_qual(self.str_type, self.label)? {
                    qual.to_owned()
                } else {
                    vec![UNKNOWN_QUAL; read.mapping(self.str_type, self.label)?.len]
                }
            } else {
                read.substring(self.str_type, self.label)?.to_owned()
            }
        ))
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        vec![LabelOrAttr::Label(self.clone())]
    }
}

impl ExprNode for Attr {
    fn eval(&self, read: &Read, use_qual: bool) -> std::result::Result<Data, NameError> {
        let d = read.data(self.str_type, self.label, self.attr)?;
        if use_qual {
            if let Data::Bytes(b) = d {
                return Ok(Data::Bytes(vec![UNKNOWN_QUAL; b.len()]));
            }
        }
        Ok(d.clone())
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        vec![LabelOrAttr::Attr(self.clone())]
    }
}

pub fn label_exists(name: impl AsRef<str>) -> Expr {
    Expr {
        node: Box::new(LabelExistsNode {
            label: Label::new(name.as_ref().as_bytes()).unwrap_or_else(|e| panic!("{e}")),
        }),
    }
}

struct LabelExistsNode {
    label: Label,
}

impl ExprNode for LabelExistsNode {
    fn eval(&self, read: &Read, _use_qual: bool) -> std::result::Result<Data, NameError> {
        Ok(Data::Bool(
            read.mapping(self.label.str_type, self.label.label).is_ok(),
        ))
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        Vec::new()
    }
}

pub fn attr_exists(name: impl AsRef<str>) -> Expr {
    Expr {
        node: Box::new(AttrExistsNode {
            attr: Attr::new(name.as_ref().as_bytes()).unwrap_or_else(|e| panic!("{e}")),
        }),
    }
}

struct AttrExistsNode {
    attr: Attr,
}

impl ExprNode for AttrExistsNode {
    fn eval(&self, read: &Read, _use_qual: bool) -> std::result::Result<Data, NameError> {
        Ok(Data::Bool(
            read.data(self.attr.str_type, self.attr.label, self.attr.attr)
                .is_ok(),
        ))
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        Vec::new()
    }
}

impl ExprNode for Data {
    fn eval(&self, _read: &Read, _use_qual: bool) -> std::result::Result<Data, NameError> {
        Ok(self.clone())
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        Vec::new()
    }
}
