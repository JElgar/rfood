enum Exp {
    Lit(i32),
    Sub(Box<Exp>, Box<Exp>),
}

fn eval(exp: &Exp) -> i32 {
    return match exp {
        Exp::Lit(num) => *num,
        Exp::Sub(exp1, exp2) => eval(exp1) - eval(exp2)
    }
}

pub fn demo() {
    let exp: Box<Exp> = Box::new(Exp::Sub(Box::new(Exp::Lit(1)), Box::new(Exp::Lit(2))));
    print!("{}", eval(&exp));
}
