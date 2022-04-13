enum Exp {
    Lit { n: i32 },
    Sub { l: Box<Exp>, r: Box<Exp> },
}
fn eval(exp: Box<Exp>) -> i32 {
    12
}
pub fn demo() {
    let e = Box::new(Exp::Sub {
        l: Box::new(Exp::Lit { n: 2 }),
        r: Box::new(Exp::Lit { n: 1 }),
    });
}

pub fn test(a: &i32) -> i32 {
    a.clone()
}

pub fn main() {
    test(&12);
}

