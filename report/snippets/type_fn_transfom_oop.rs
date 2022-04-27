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
pub fn demo() {
    let e = Box::new(Sub {
        l: Box::new(Lit { n: 2 }),
        r: Box::new(Lit { n: 1 }),
    });
    let _result = e.eval();
    println!("{}", e.eval());
}
