fn insert(set: Set, value: i32) -> Box<dyn Set> {
    match set {
        Set::Empty() => {
            return if contains(self, i) {
                self
            } else {
                Box::new(Insert {
                    set: self,
                    value: i,
                })
            };
        }
    }
}
