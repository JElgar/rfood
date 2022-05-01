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
pub trait ValuesList {
    fn concat(self: Box<Self>, l: Box<dyn ValuesList>) -> Box<dyn ValuesList>;
}
pub struct Empty {}
impl ValuesList for Empty {
    fn concat(self: Box<Self>, l: Box<dyn ValuesList>) -> Box<dyn ValuesList> {
        l
    }
}
pub struct Cons {
    pub value: Int,
    pub list: Box<dyn ValuesList>,
}
impl ValuesList for Cons {
    fn concat(self: Box<Self>, l: Box<dyn ValuesList>) -> Box<dyn ValuesList> {
        Box::new(Cons {
            list: l,
            value: self.value,
        })
        .concat(self.list)
    }
}
pub trait Expr {
    fn values(self: Box<Self>) -> Box<dyn ValuesList>;
    fn eval(self: Box<Self>) -> Int;
}
pub struct Val {
    pub n: Int,
}
impl Expr for Val {
    fn values(self: Box<Self>) -> Box<dyn ValuesList> {
        create_list(self.n)
    }
    fn eval(self: Box<Self>) -> Int {
        if self.n > 0 {
            self.n
        } else {
            panic!("Invalid n")
        }
    }
}
pub struct App {
    pub o: Box<dyn Op>,
    pub l: Box<dyn Expr>,
    pub r: Box<dyn Expr>,
}
impl Expr for App {
    fn values(self: Box<Self>) -> Box<dyn ValuesList> {
        self.l.values().concat(self.r.values())
    }
    fn eval(self: Box<Self>) -> Int {
        let x = self.l.eval();
        let y = self.r.eval();
        if self.o.valid(x, y) {
            self.o.apply(x, y)
        } else {
            panic!("Expr is not valid")
        }
    }
}
type Int = i64;
pub fn create_list(value: Int) -> Box<dyn ValuesList> {
    Box::new(Cons {
        value: value,
        list: Box::new(Empty {}),
    })
}
