pub trait Exp {
    fn eval(&self) -> i32;
}

pub struct Lit{
    pub n: i32,
}
impl Exp for Lit {
    fn eval(&self) -> i32 {
        return self.n;
    }
}

pub struct Sub {
    pub l: Box<dyn Exp>,
    pub r: Box<dyn Exp>,
}
impl Exp for Sub {
    fn eval(&self) -> i32 {
        return self.l.eval() - self.r.eval();
    }
}
