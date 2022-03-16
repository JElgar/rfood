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

pub fn demo() {
    let container: LoggingContainer<String> = LoggingContainer{item: "foo".to_string()};
    println!("{}", container.get_item());
}
