trait Set {
    fn contains(&self, target: i32) -> bool;
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set>;
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set>;
    fn is_empty(&self) -> bool;
}
struct Empty {}
impl Set for Empty {
    fn contains(&self, target: i32) -> bool {
        false
    }
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set> {
        {
            if self.contains(value) {
                return self;
            }
            return Box::new(Insert { s1: self, value });
        }
    }
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set> {
        right
    }
    fn is_empty(&self) -> bool {
        true
    }
}
struct Insert {
    s1: Box<dyn Set>,
    value: i32,
}
impl Set for Insert {
    fn contains(&self, target: i32) -> bool {
        self.value == target || self.s1.contains(target)
    }
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set> {
        {
            if self.contains(self.value) {
                return self;
            }
            return Box::new(Insert { s1: self, value });
        }
    }
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set> {
        Box::new(Union {
            s1: self,
            s2: right,
        })
    }
    fn is_empty(&self) -> bool {
        false
    }
}
struct Union {
    s1: Box<dyn Set>,
    s2: Box<dyn Set>,
}
impl Set for Union {
    fn contains(&self, target: i32) -> bool {
        self.s1.contains(target) && self.s2.contains(target)
    }
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set> {
        {
            if self.contains(value) {
                return self;
            }
            return Box::new(Insert { s1: self, value });
        }
    }
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set> {
        Box::new(Union {
            s1: self,
            s2: right,
        })
    }
    fn is_empty(&self) -> bool {
        self.s1.is_empty() && self.s2.is_empty()
    }
}
pub fn demo() {
    let empty = Box::new(Empty {});
    let set = empty.insert(1);
    let set2 = Box::new(Insert {
        s1: Box::new(Empty {}),
        value: 1,
    });
    let _set3 = set.union(set2);
}
