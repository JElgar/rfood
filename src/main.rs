mod set;
mod exp;

fn main() {
    println!("Exps");
    let e: Box<dyn exp::Exp> = Box::new(exp::Sub{
        l: Box::new(exp::Lit{n: 2}),
        r: Box::new(exp::Lit{n: 1})
    });
    println!("{}", e.eval());
    
    println!("Sets");
    let s1: Box<dyn set::Set> = Box::new(set::Insert {
        set: Box::new(set::Empty{}),
        value: 1
    });
    let s2: Box<dyn set::Set> = Box::new(set::Insert {
        set: Box::new(set::Empty{}),
        value: 1
    });

    let s: Box<dyn set::Set> = s1.insert(4);
    let s3: Box<dyn set::Set> = s.union(s2);
    println!("{:?}", s3);
}
