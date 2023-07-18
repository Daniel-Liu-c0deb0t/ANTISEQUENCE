use crate::read::*;
use crate::errors::{self, Name, NameError};

pub type Node = Box<dyn ExprNode>;

pub trait ExprNode {
    fn add(self, o: Node) -> Node {
        Box::new(AddNode { left: self, right: o })
    }

    fn sub(self, o: Node) -> Node {
        Box::new(SubNode { left: self, right: o })
    }

    fn mul(self, o: Node) -> Node {
        Box::new(MulNode { left: self, right: o })
    }

    fn div(self, o: Node) -> Node {
        Box::new(DivNode { left: self, right: o })
    }

    fn lt(self, o: Node) -> Node {
        Box::new(LtNode { left: self, right: o })
    }

    fn gt(self, o: Node) -> Node {
        Box::new(GtNode { left: self, right: o })
    }

    fn le(self, o: Node) -> Node {
        Box::new(LeNode { left: self, right: o })
    }

    fn ge(self, o: Node) -> Node {
        Box::new(GeNode { left: self, right: o })
    }

    fn eq(self, o: Node) -> Node {
        Box::new(EqNode { left: self, right: o })
    }

    fn len(self) -> Node {
        Box::new(LenNode { string: self })
    }

    fn repeat(self, times: Node) -> Node {
        Box::new(RepeatNode { string: self, times })
    }

    fn concat(self, o: Node) -> Node {
        Box::new(ConcatNode { left: self, right: o })
    }

    fn in(self, range: impl RangeBounds<Node>) -> Node {
        Box::new(InNode { num: self, range })
    }

    fn eval(&self, read: &Read) -> Result<Data>;

    fn required_names(&self) -> Vec<LabelOrAttr>;
}

pub struct AddNode {
    left: Node,
    right: Node,
}

impl ExprNode for AddNode {
    fn eval(&self, read: &Read) -> Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Int(l + r)),
            (Float(l), Float(r)) => Ok(Float(l + r)),
            _ => Err(NameError::Type("both int or both float", vec![left, right])),
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
    fn eval(&self, read: &Read) -> Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Int(l - r)),
            (Float(l), Float(r)) => Ok(Float(l - r)),
            _ => Err(NameError::Type("both int or both float", vec![left, right])),
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
    fn eval(&self, read: &Read) -> Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Int(l * r)),
            (Float(l), Float(r)) => Ok(Float(l * r)),
            _ => Err(NameError::Type("both int or both float", vec![left, right])),
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
    fn eval(&self, read: &Read) -> Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Int(l / r)),
            (Float(l), Float(r)) => Ok(Float(l / r)),
            _ => Err(NameError::Type("both int or both float", vec![left, right])),
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
    fn eval(&self, read: &Read) -> Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Bool(l < r)),
            (Float(l), Float(r)) => Ok(Bool(l < r)),
            _ => Err(NameError::Type("both int or both float", vec![left, right])),
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
    fn eval(&self, read: &Read) -> Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Bool(l > r)),
            (Float(l), Float(r)) => Ok(Bool(l > r)),
            _ => Err(NameError::Type("both int or both float", vec![left, right])),
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
    fn eval(&self, read: &Read) -> Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Bool(l <= r)),
            (Float(l), Float(r)) => Ok(Bool(l <= r)),
            _ => Err(NameError::Type("both int or both float", vec![left, right])),
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
    fn eval(&self, read: &Read) -> Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Bool(l >= r)),
            (Float(l), Float(r)) => Ok(Bool(l >= r)),
            _ => Err(NameError::Type("both int or both float", vec![left, right])),
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
    fn eval(&self, read: &Read) -> Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Int(l), Int(r)) => Ok(Bool(l == r)),
            (Float(l), Float(r)) => Ok(Bool(l == r)),
            (Bool(l), Bool(r)) => Ok(Bool(l == r)),
            (Bytes(l), Bytes(r)) => Ok(Bool(l == r)),
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
    fn eval(&self, read: &Read) -> Result<Data, NameError> {
        let string = self.string.eval(read)?;
        use Data::*;
        match string {
            Bytes(s) => Ok(Int(s.len() as isize)),
            _ => Err(NameError::Type("bytes", vec![string])),
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
    fn eval(&self, read: &Read) -> Result<Data, NameError> {
        let string = self.string.eval(read)?;
        let times = self.times.eval(read)?;
        use Data::*;
        match (string, times) {
            (Bytes(s), Int(t)) => Ok(Bytes(s.repeat(t as usize))),
            _ => Err(NameError::Type("bytes and int", vec![string, times])),
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
    fn eval(&self, read: &Read) -> Result<Data, NameError> {
        let left = self.left.eval(read)?;
        let right = self.right.eval(read)?;
        use Data::*;
        match (left, right) {
            (Bytes(mut l), Bytes(mut r)) => {
                l.append(&mut r);
                Ok(Bytes(l))
            }
            _ => Err(NameError::Type("both bytes", vec![left, right])),
        }
    }

    fn required_names(&self) -> Vec<LabelOrAttr> {
        let mut res = self.left.required_names();
        res.append(&mut self.right.required_names());
        res
    }
}

pub struct InNode<R: RangeBounds<Node>> {
    num: Node,
    range: R,
}

impl<R: RangeBounds<Node>> ExprNode for InNode<R> {
    fn eval(&self, read: &Read) -> Result<Data, NameError> {
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
                Ok(s <= n && n <= e)
            }
            _ => Err(NameError::Type("all int", vec![num, start, end])),
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
