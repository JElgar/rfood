pub trait Exp {
}

pub struct Lit{
    pub n: i32,
}

pub struct Sub {
    pub l: Box<dyn Exp>,
    pub r: Box<dyn Exp>,
}
