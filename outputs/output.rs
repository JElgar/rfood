pub enum Exp {
    Lit { n: i32 },
    Sub { l: Box<Exp>, r: Box<Exp> },
}
pub fn eval(exp: &Exp) -> i32 {
    match exp {
        Exp::Lit { n } => {
            return *n;
        }
        Exp::Sub { l, r } => {
            return eval(&*l) - eval(&*r);
        }
    }
}
pub fn demo() {
    let e = Box::new(Exp::Sub {
        l: Box::new(Exp::Lit { n: 2 }),
        r: Box::new(Exp::Lit { n: 1 }),
    });
    let _result = eval(&*e);
    println!("{}", eval(&*e));
}
