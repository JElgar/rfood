trait Container<T> {
    fn get_item(self: Box<Self>) -> T;
}

struct LoggingContainer<T> {
    item: T,
}
impl<T> Container<T> for LoggingContainer<T> {
    fn get_item(self: Box<Self>) -> T {
        self.item
    }
}

struct SecretMessageContainer<T, U> where U: std::fmt::Debug {
    item: T,
    secret: U,

}
impl<T, U> Container<T> for SecretMessageContainer<T, U> where U: std::fmt::Debug {
    fn get_item(self: Box<Self>) -> T {
        // println!("{:?}", self.secret);
        self.item
    }
}

pub fn demo() {
    let container: LoggingContainer<i32> = LoggingContainer{item: 1};
    let container2: SecretMessageContainer<i32, i32> = SecretMessageContainer{item: 1, secret: 10};
}
