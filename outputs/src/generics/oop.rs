enum Container<T, U> {
    LoggingContainer { item: T },
    SecretMessageContainer { item: T, secret: U },
}
fn get_item<T, U>(container: Container<T, U>) -> T {
    match container {
        Container::LoggingContainer { item } => item,
        Container::SecretMessageContainer { item, secret } => item,
    }
}
pub fn demo() {
    // let container = Container::LoggingContainer { item: 1 };
    let container2  = Container::SecretMessageContainer {
        item: 1,
        secret: 10,
    };
}
