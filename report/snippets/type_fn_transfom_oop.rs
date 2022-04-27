pub enum Exp {
    Lit { n: i32 },
    Sub { l: Box<Exp>, r: Box<Exp> },
}

pub fn eval(exp: &Exp) -> i32 {
    match &exp {
        Exp::Lit { n } => {
            return n;
        }
        Exp::Sub { l, r } => {
            return l.eval() - r.eval();
        }
    }
}
