pub enum Container<T> {
    LoggingContainer(T),
}

pub fn get_item<T>(container: Container<T>) -> T {
    return match container {
        Container::LoggingContainer(item) => item,
    }
}

pub fn demo() {
    let container: Container<String> = Container::LoggingContainer("hello".to_string());
    println!("{}", get_item(container));
}
