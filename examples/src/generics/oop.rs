trait Container<T> {
    fn get_item(self) -> T;
}

struct LoggingContainer<T> {
    item: T,
}
impl<T> Container<T> for LoggingContainer<T> {
    fn get_item(self) -> T {
        self.item
    }
}

struct SecretMessageContainer<T, U> where U: std::fmt::Debug {
    item: T,
    secret: U,

}
impl<T, U> Container<T> for SecretMessageContainer<T, U> where U: std::fmt::Debug {
    fn get_item(self) -> T {
        println!("{:?}", self.secret);
        self.item
    }
}

pub fn demo() {
    let container: LoggingContainer<String> = LoggingContainer{item: "foo".to_string()};
    println!("{}", container.get_item());
    
    let container2: SecretMessageContainer<String, String> = SecretMessageContainer{item: "foo".to_string(), secret: "bar".to_string()};
    println!("{}", container2.get_item());
}
