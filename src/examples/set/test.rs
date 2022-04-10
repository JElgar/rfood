trait Set {
    fn is_empty(&self) -> bool;
    fn contains(&self, target: i32) -> bool;
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set>;
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set>;
}
struct Empty {}
impl Set for Empty {
    fn is_empty(&self) -> bool {
        true
    }
    fn contains(&self, target: i32) -> bool {
        false
    }
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set> {
        right
    }
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set> {
        {
            if self.set.contains(value) {
                return set;
            }
            return Set::Insert {
                set: Box::new(set),
                value,
            };
        }
    }
}
struct Insert {
    set: Box<dyn Set>,
    value: i32,
}
impl Set for Insert {
    fn is_empty(&self) -> bool {
        false
    }
    fn contains(&self, target: i32) -> bool {
        self.value == target || self.set.contains(target)
    }
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set> {
        Set::Union {
            s1: Box::new(left),
            s2: Box::new(right),
        }
    }
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set> {
        {
            if self.set.contains(self.value) {
                return self.set;
            }
            return Set::Insert {
                set: Box::new(self.set),
                value,
            };
        }
    }
}
struct Union {
    s1: Box<dyn Set>,
    s2: Box<dyn Set>,
}
impl Set for Union {
    fn is_empty(&self) -> bool {
        self.s1.is_empty() && self.s2.is_empty()
    }
    fn contains(&self, target: i32) -> bool {
        self.s1.contains(target) && self.s2.contains(target)
    }
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set> {
        Set::Union {
            s1: Box::new(left),
            s2: Box::new(right),
        }
    }
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set> {
        {
            if self.set.contains(value) {
                return set;
            }
            return Set::Insert {
                set: Box::new(set),
                value,
            };
        }
    }
}
pub fn demo() {
    let empty = Box::new(Empty {});
    let set = empty.insert(1);
    let set2 = Box::new(Insert {
        set: Box::new(Empty {}),
        value: 1,
    });
    let _set3 = set.union(set2);
}
