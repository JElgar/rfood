trait Thing {
    fn do_something(&self);
}

struct ABC {
    a: i32,
}

impl Thing for ABC {
    fn do_something(&self) {
        println!("{}", self.a);
    }
}
