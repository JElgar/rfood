fn eval(exp: &Exp) -> i32 {
    match exp {
        Exp::Lit(num) => return *num,
        Exp::Sub(exp1, exp2) => return eval(exp1) - eval(exp2)
    }
}
