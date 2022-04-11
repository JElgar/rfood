enum Exp {
    Lit{n: i32},
    Sub{l: Box<Exp>, r: Box<Exp>},
}

fn eval(exp: &Exp) -> i32 {
    return match exp {
        Exp::Lit{n} => *n,
        Exp::Sub{l, r} => eval(l) - eval(r)
    }
}

pub fn demo() {
    let exp = Box::new(Exp::Sub{l: Box::new(Exp::Lit{n: 1}), r: Box::new(Exp::Lit{n: 2})});
    let out = eval(&exp);
    print!("{}", out);
}
