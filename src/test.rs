fn eval(exp: &Exp) -> i32 {
    return match exp {
        Exp::Lit(num) => *num,
        Exp::Sub(exp1, exp2) => eval(exp1) - eval(exp2)
    }
}
