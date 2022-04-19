pub trait Set {
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set>;
    fn is_empty(&self) -> bool;
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set>;
    fn contains(&self, target: i32) -> bool;
}
pub struct Empty {}
impl Set for Empty {
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set> {
        right
    }
    fn is_empty(&self) -> bool {
        true
    }
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set> {
        if self.contains(value) {
            return self;
        }
        return Box::new(Insert { s1: self, value });
    }
    fn contains(&self, target: i32) -> bool {
        false
    }
}
pub struct Insert {
    pub s1: Box<dyn Set>,
    pub value: i32,
}
impl Set for Insert {
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set> {
        Box::new(Union {
            s1: self,
            s2: right,
        })
    }
    fn is_empty(&self) -> bool {
        false
    }
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set> {
        if self.contains(self.value) {
            return self;
        }
        return Box::new(Insert { s1: self, value });
    }
    fn contains(&self, target: i32) -> bool {
        self.value == target || self.s1.contains(target)
    }
}
pub struct Union {
    pub s1: Box<dyn Set>,
    pub s2: Box<dyn Set>,
}
impl Set for Union {
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set> {
        Box::new(Union {
            s1: self,
            s2: right,
        })
    }
    fn is_empty(&self) -> bool {
        self.s1.is_empty() && self.s2.is_empty()
    }
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set> {
        if self.contains(value) {
            return self;
        }
        return Box::new(Insert { s1: self, value });
    }
    fn contains(&self, target: i32) -> bool {
        self.s1.contains(target) || self.s2.contains(target)
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
