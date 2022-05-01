pub trait OptionalValue {
    fn unwrap(self: Box<Self>) -> Int;
    fn is_none(&self) -> bool;
}
pub struct Some {
    pub value: Int,
}
impl OptionalValue for Some {
    fn unwrap(self: Box<Self>) -> Int {
        self.value
    }
    fn is_none(&self) -> bool {
        false
    }
}
pub struct None {}
impl OptionalValue for None {
    fn unwrap(self: Box<Self>) -> Int {
        panic!("Unwrap called on None")
    }
    fn is_none(&self) -> bool {
        true
    }
}
pub trait Set {
    fn contains(&self, target: Int) -> bool;
    fn is_empty(&self) -> bool;
    fn insert(self: Box<Self>, value: Int) -> Box<dyn Set>;
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set>;
}
pub struct Empty {}
impl Set for Empty {
    fn contains(&self, target: Int) -> bool {
        false
    }
    fn is_empty(&self) -> bool {
        true
    }
    fn insert(self: Box<Self>, value: Int) -> Box<dyn Set> {
        if self.contains(value) {
            return self;
        }
        return Box::new(Insert { s1: self, value });
    }
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set> {
        right
    }
}
pub struct Insert {
    pub s1: Box<dyn Set>,
    pub value: Int,
}
impl Set for Insert {
    fn contains(&self, target: Int) -> bool {
        self.value == target || self.s1.contains(target)
    }
    fn is_empty(&self) -> bool {
        false
    }
    fn insert(self: Box<Self>, value: Int) -> Box<dyn Set> {
        if self.contains(self.value) {
            return self;
        }
        return Box::new(Insert { s1: self, value });
    }
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set> {
        Box::new(Union {
            s1: self,
            s2: right,
        })
    }
}
pub struct Union {
    pub s1: Box<dyn Set>,
    pub s2: Box<dyn Set>,
}
impl Set for Union {
    fn contains(&self, target: Int) -> bool {
        self.s1.contains(target) || self.s2.contains(target)
    }
    fn is_empty(&self) -> bool {
        self.s1.is_empty() && self.s2.is_empty()
    }
    fn insert(self: Box<Self>, value: Int) -> Box<dyn Set> {
        if self.contains(value) {
            return self;
        }
        return Box::new(Insert { s1: self, value });
    }
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set> {
        Box::new(Union {
            s1: self,
            s2: right,
        })
    }
}
pub trait Op {
    fn apply(&self, a: Int, b: Int) -> Int;
    fn valid(&self, x: Int, y: Int) -> bool;
}
pub struct Add;
impl Op for Add {
    fn apply(&self, a: Int, b: Int) -> Int {
        a + b
    }
    fn valid(&self, x: Int, y: Int) -> bool {
        x <= y
    }
}
pub struct Sub;
impl Op for Sub {
    fn apply(&self, a: Int, b: Int) -> Int {
        a - b
    }
    fn valid(&self, x: Int, y: Int) -> bool {
        x > y
    }
}
pub struct Mul;
impl Op for Mul {
    fn apply(&self, a: Int, b: Int) -> Int {
        a * b
    }
    fn valid(&self, x: Int, y: Int) -> bool {
        x != 1 && y != 1 && x <= y
    }
}
pub struct Div;
impl Op for Div {
    fn apply(&self, a: Int, b: Int) -> Int {
        a / b
    }
    fn valid(&self, x: Int, y: Int) -> bool {
        y > 1 && ((x % y) == 0)
    }
}
pub trait Expr {
    fn values(self: Box<Self>) -> Box<dyn Set>;
    fn eval(self: Box<Self>) -> Box<dyn OptionalValue>;
}
pub struct Val {
    pub n: Int,
}
impl Expr for Val {
    fn values(self: Box<Self>) -> Box<dyn Set> {
        create_set(self.n)
    }
    fn eval(self: Box<Self>) -> Box<dyn OptionalValue> {
        if self.n > 0 {
            Box::new(Some { value: self.n })
        } else {
            Box::new(None {})
        }
    }
}
pub struct App {
    pub o: Box<dyn Op>,
    pub l: Box<dyn Expr>,
    pub r: Box<dyn Expr>,
}
impl Expr for App {
    fn values(self: Box<Self>) -> Box<dyn Set> {
        self.l.values().union(self.r.values())
    }
    fn eval(self: Box<Self>) -> Box<dyn OptionalValue> {
        let x = self.l.eval();
        let y = self.r.eval();
        if x.is_none() || y.is_none() {
            return None {};
        }
        let x = x.unwrap();
        let y = y.unwrap();
        if self.o.valid(x, y) {
            Box::new(Some {
                value: self.o.apply(x, y),
            })
        } else {
            Box::new(None {})
        }
    }
}
type Int = i32;
pub fn create_set(val: Int) -> Box<dyn Set> {
    Box::new(Insert {
        value: val,
        s1: Box::new(Empty {}),
    })
}
