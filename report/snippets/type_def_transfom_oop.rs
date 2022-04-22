pub trait Exp {
    // ...
}

pub struct Lit{
    pub n: i32,
}
impl Exp for Lit {
    // ...
}

pub struct Sub {
    pub l: Box<dyn Exp>,
    pub r: Box<dyn Exp>,
}

impl Exp for Sub {
    // ...
}
