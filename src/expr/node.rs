use std::ops::{Bound, RangeBounds};

use crate::errors::NameError;
use crate::expr::*;
use crate::read::*;

pub struct Node {
    node: Box<dyn ExprNode>,
}

impl Node {
    pub fn add(self, o: Node) -> Node {
        Node {
            node: Box::new(AddNode {
                left: self,
                right: o,
            }),
        }
    }

    pub fn sub(self, o: Node) -> Node {
        Node {
            node: Box::new(SubNode {
                left: self,
                right: o,
            }),
        }
    }

    pub fn mul(self, o: Node) -> Node {
        Node {
            node: Box::new(MulNode {
                left: self,
                right: o,
            }),
        }
    }

    pub fn div(self, o: Node) -> Node {
        Node {
            node: Box::new(DivNode {
                left: self,
                right: o,
            }),
        }
    }

    pub fn lt(self, o: Node) -> Node {
        Node {
            node: Box::new(LtNode {
                left: self,
                right: o,
            }),
        }
    }

    pub fn gt(self, o: Node) -> Node {
        Node {
            node: Box::new(GtNode {
                left: self,
                right: o,
            }),
        }
    }

    pub fn le(self, o: Node) -> Node {
        Node {
            node: Box::new(LeNode {
                left: self,
                right: o,
            }),
        }
    }

    pub fn ge(self, o: Node) -> Node {
        Node {
            node: Box::new(GeNode {
                left: self,
                right: o,
            }),
        }
    }

    pub fn eq(self, o: Node) -> Node {
        Node {
            node: Box::new(EqNode {
                left: self,
                right: o,
            }),
        }
    }

    pub fn len(self) -> Node {
        Node {
            node: Box::new(LenNode { string: self }),
        }
    }

    pub fn repeat(self, times: Node) -> Node {
        Node {
            node: Box::new(RepeatNode {
                string: self,
                times,
            }),
        }
    }

    pub fn concat(self, o: Node) -> Node {
        Node {
            node: Box::new(ConcatNode {
                left: self,
                right: o,
            }),
        }
    }

    pub fn in_bounds(self, range: impl RangeBounds<Node> + 'static) -> Node {
        Node {
            node: Box::new(InBoundsNode { num: self, range }),
        }
    }

    pub fn eval_bool(&self, read: &Read) -> std::result::Result<bool, NameError> {
        let res = self.eval(read)?;

        if let Data::Bool(b) = res {
            Ok(b)
        } else {
            Err(NameError::Type("bool", vec![res]))
        }
    }

    pub fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        self.node.eval(read)
    }

    pub fn required_names(&self) -> Vec<LabelOrAttr> {
        self.node.required_names()
    }
}

pub trait ExprNode {
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError>;

    fn required_names(&self) -> Vec<LabelOrAttr>;
}

pub struct AddNode {
    left: Node,
    right: Node,
}

impl ExprNode for AddNode {
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Int(l + r)),
            (Float(l), Float(r)) => Ok(Float(l + r)),
            (l, r) => Err(NameError::Type("both int or both float", vec![l, r])),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        let mut res = self.left.required_names();
        res.append(&mut self.right.required_names());
        res
    }
}

pub struct SubNode {
    left: Node,
    right: Node,
}

impl ExprNode for SubNode {
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Int(l - r)),
            (Float(l), Float(r)) => Ok(Float(l - r)),
            (l, r) => Err(NameError::Type("both int or both float", vec![l, r])),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        let mut res = self.left.required_names();
        res.append(&mut self.right.required_names());
        res
    }
}

pub struct MulNode {
    left: Node,
    right: Node,
}

impl ExprNode for MulNode {
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Int(l * r)),
            (Float(l), Float(r)) => Ok(Float(l * r)),
            (l, r) => Err(NameError::Type("both int or both float", vec![l, r])),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        let mut res = self.left.required_names();
        res.append(&mut self.right.required_names());
        res
    }
}

pub struct DivNode {
    left: Node,
    right: Node,
}

impl ExprNode for DivNode {
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Int(l / r)),
            (Float(l), Float(r)) => Ok(Float(l / r)),
            (l, r) => Err(NameError::Type("both int or both float", vec![l, r])),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        let mut res = self.left.required_names();
        res.append(&mut self.right.required_names());
        res
    }
}

pub struct LtNode {
    left: Node,
    right: Node,
}

impl ExprNode for LtNode {
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Bool(l < r)),
            (Float(l), Float(r)) => Ok(Bool(l < r)),
            (l, r) => Err(NameError::Type("both int or both float", vec![l, r])),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        let mut res = self.left.required_names();
        res.append(&mut self.right.required_names());
        res
    }
}

pub struct GtNode {
    left: Node,
    right: Node,
}

impl ExprNode for GtNode {
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Bool(l > r)),
            (Float(l), Float(r)) => Ok(Bool(l > r)),
            (l, r) => Err(NameError::Type("both int or both float", vec![l, r])),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        let mut res = self.left.required_names();
        res.append(&mut self.right.required_names());
        res
    }
}

pub struct LeNode {
    left: Node,
    right: Node,
}

impl ExprNode for LeNode {
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Bool(l <= r)),
            (Float(l), Float(r)) => Ok(Bool(l <= r)),
            (l, r) => Err(NameError::Type("both int or both float", vec![l, r])),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        let mut res = self.left.required_names();
        res.append(&mut self.right.required_names());
        res
    }
}

pub struct GeNode {
    left: Node,
    right: Node,
}

impl ExprNode for GeNode {
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Bool(l >= r)),
            (Float(l), Float(r)) => Ok(Bool(l >= r)),
            (l, r) => Err(NameError::Type("both int or both float", vec![l, r])),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        let mut res = self.left.required_names();
        res.append(&mut self.right.required_names());
        res
    }
}

pub struct EqNode {
    left: Node,
    right: Node,
}

impl ExprNode for EqNode {
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
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

pub struct LenNode {
    string: Node,
}

impl ExprNode for LenNode {
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        let string = self.string.eval(read)?;
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

pub struct RepeatNode {
    string: Node,
    times: Node,
}

impl ExprNode for RepeatNode {
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        let string = self.string.eval(read)?;
        let times = self.times.eval(read)?;
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

pub struct ConcatNode {
    left: Node,
    right: Node,
}

impl ExprNode for ConcatNode {
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
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

pub struct InBoundsNode<R: RangeBounds<Node>> {
    num: Node,
    range: R,
}

impl<R: RangeBounds<Node>> ExprNode for InBoundsNode<R> {
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        let num = self.num.eval(read)?;

        use Data::*;
        let mut start_add1 = false;
        let start = match self.range.start_bound() {
            Bound::Included(s) => s.eval(read)?,
            Bound::Excluded(s) => {
                start_add1 = true;
                s.eval(read)?
            }
            Bound::Unbounded => Int(std::isize::MIN),
        };

        let mut end_sub1 = false;
        let end = match self.range.end_bound() {
            Bound::Included(e) => e.eval(read)?,
            Bound::Excluded(e) => {
                end_sub1 = true;
                e.eval(read)?
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
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        Ok(Data::Bytes(
            read.substring(self.str_type, self.label)?.to_owned(),
        ))
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        vec![LabelOrAttr::Label(self.clone())]
    }
}

impl ExprNode for Attr {
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        Ok(read.data(self.str_type, self.label, self.attr)?.clone())
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        vec![LabelOrAttr::Attr(self.clone())]
    }
}

pub fn label_exists(name: impl AsRef<str>) -> Node {
    Node {
        node: Box::new(LabelExistsNode {
            label: Label::new(name.as_ref().as_bytes()).unwrap_or_else(|e| panic!("{e}")),
        }),
    }
}

pub struct LabelExistsNode {
    label: Label,
}

impl ExprNode for LabelExistsNode {
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        Ok(Data::Bool(
            read.mapping(self.label.str_type, self.label.label).is_ok(),
        ))
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        Vec::new()
    }
}

pub fn attr_exists(name: impl AsRef<str>) -> Node {
    Node {
        node: Box::new(AttrExistsNode {
            attr: Attr::new(name.as_ref().as_bytes()).unwrap_or_else(|e| panic!("{e}")),
        }),
    }
}

pub struct AttrExistsNode {
    attr: Attr,
}

impl ExprNode for AttrExistsNode {
    fn eval(&self, read: &Read) -> std::result::Result<Data, NameError> {
        Ok(Data::Bool(
            read.data(self.attr.str_type, self.attr.label, self.attr.attr)
                .is_ok(),
        ))
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        Vec::new()
    }
}
