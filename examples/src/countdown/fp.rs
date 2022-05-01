// http://www.cs.nott.ac.uk/~pszgmh/countdown.pdf
//
// Required changes
// Box because Rust
// Named enums because not implemented
// Some
// Vec! not working

type Int = i64;

#[derive(Copy, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

pub enum ValuesList {
    Empty{},
    Cons{value: Int, list: Box<ValuesList>}
} 

pub fn concat(r: ValuesList, l: ValuesList) -> ValuesList {
    match r {
        ValuesList::Empty{} => l,
        ValuesList::Cons { value, list } => concat(ValuesList::Cons{list: Box::new(l), value: value}, *list)
    }
}

pub fn create_list(value: Int) -> ValuesList {
    ValuesList::Cons{value: value, list: Box::new(ValuesList::Empty{})}
}


#[derive(Clone)]
pub enum Expr {
    Val{n: Int},
    App{o: Op, l: Box<Expr>, r: Box<Expr>},
}

fn valid(op: &Op, x: Int, y: Int) -> bool {
    match op {
        Op::Add => x <= y,
        Op::Sub => x > y,
        Op::Mul => x != 1 && y != 1 && x <= y,
        Op::Div => y > 1 && ((x % y) == 0),
    }
}

fn apply(op: &Op, a: Int, b: Int) -> Int {
    match op {
        Op::Add => a + b,
        Op::Sub => a - b,
        Op::Mul => a * b,
        Op::Div => a / b,
    }
}

fn values(expr: Expr) -> ValuesList {
    match expr {
        Expr::Val{n: n} => create_list(n),
        Expr::App{o: _, l: l, r: r} => concat(values(*l), values(*r))
    }
} 

fn eval(expr: Expr) -> Int {
    match expr {
        Expr::Val{n: n} => if n > 0 {n}  else {panic!("Invalid n")}
        Expr::App{o, l, r} => {
            let x = eval(*l);
            let y = eval(*r);

            if valid(&o, x, y) {
                apply(&o, x, y)
            } else {
                panic!("Expr is not valid")
            }
        }
    }
}

// fn split<T>(xs: &[T]) -> Vec<(&[T], &[T])> {
//     (1..xs.len())
//         .map(|i| xs.split_at(i))
//         .collect()
// }
// 
// fn sub_bags<T: Clone>(xs: Vec<T>) -> Vec<Vec<T>> {
//     (0..xs.len() + 1)
//         .flat_map(|i| xs.iter().cloned().permutations(i))
//         .collect()
// }
// 
// type Result = (Expr, Int);
// 
// fn combine((l, x): &Result, (r, y): &Result) -> Vec<Result> {
//     [Add, Sub, Mul, Div].iter()
//         .filter(|op| valid(op, *x, *y))
//         .map(|op|
//             (App(*op, Arc::new(l.clone()), Arc::new(r.clone())),
//              apply(op, *x, *y)))
//         .collect()
// }
// 
// fn results(ns: &[Int]) -> Vec<Result> {
//     match ns {
//         [] => vec!(),
//         [n] => vec!((Val(*n), *n)),
//         _ => _results(ns),
//     }
// }
// 
// fn _results(ns: &[Int]) -> Vec<Result> {
//     split(ns).iter()
//         .flat_map(|(ls, rs)| results(ls).into_iter()
//             .flat_map(move |lx| results(rs).into_iter()
//                 .flat_map(move |ry| combine(&lx, &ry))))
//         .collect()
// }
// 
// pub fn solutions(ns: Vec<Int>, n: Int) -> Vec<Expr> {
//     sub_bags(ns).par_iter()
//         .flat_map(|bag|
//             results(&bag).into_iter()
//                 .filter(|(_, m)| *m == n)
//                 .map(|(e, _)| e)
//                 .collect::<Vec<Expr>>()
//         )
//         .collect()
// }
