// http://www.cs.nott.ac.uk/~pszgmh/countdown.pdf
//
// Required changes
// Box because Rust
// Named enums because not implemented
// Some
// Vec! not working

type Int = i32;

pub enum OptionalValue {
    Some{value: Int},
    None{},
}

pub fn is_none(val: &OptionalValue) -> bool {
    match val {
        OptionalValue::None{} => true,
        OptionalValue::Some{..} => false,
    }
}

pub fn unwrap(val: OptionalValue) -> Int {
    match val {
        OptionalValue::None{} => panic!("Unwrap called on None"),
        OptionalValue::Some{value} => value,
    }
}

pub enum Set {
    Empty {},
    Insert { s1: Box<Set>, value: Int },
    Union { s1: Box<Set>, s2: Box<Set> },
}

pub fn is_empty(set: &Set) -> bool {
    return match set {
        Set::Empty {} => true,
        Set::Insert { .. } => false,
        Set::Union { s1, s2 } => is_empty(s1) && is_empty(s2),
    };
}

pub fn contains(set: &Set, target: Int) -> bool {
    return match set {
        Set::Empty {} => false,
        Set::Insert { s1, value } => *value == target || contains(s1, target),
        Set::Union { s1, s2 } => contains(s1, target) || contains(s2, target),
    };
}

pub fn insert(set: Set, value: Int) -> Set {
    if contains(&set, value) {
        return set;
    }
    return Set::Insert {
        s1: Box::new(set),
        value,
    };
}

pub fn union(left: Set, right: Set) -> Set {
    match left {
        Set::Empty{} => right,
        _ => Set::Union {
            s1: Box::new(left),
            s2: Box::new(right),
        }
    }
}

pub fn create_set(val: Int) -> Set {
    Set::Insert{value: val, s1: Box::new(Set::Empty{})}
}

#[derive(Copy, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
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

fn values(expr: Expr) -> Set {
    match expr {
        Expr::Val{n: n} => create_set(n),
        Expr::App{o: _, l: l, r: r} => union(values(*l), values(*r))
    }
} 

fn eval(expr: Expr) -> OptionalValue {
    match expr {
        Expr::Val{n: n} => if n > 0 {
            OptionalValue::Some{value: n}
        }  else {
            OptionalValue::None{}
        }
        Expr::App{o, l, r} => {
            let x = eval(*l);
            let y = eval(*r);

            if is_none(&x) || is_none(&y) {
                return OptionalValue::None{}
            }

            let x = unwrap(x);
            let y = unwrap(y);

            if valid(&o, x, y) {
                OptionalValue::Some{value: apply(&o, x, y)}
            } else {
                OptionalValue::None{}
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
// fn sub_bags(xs: Set) -> Vec<Vec<T>> {
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
