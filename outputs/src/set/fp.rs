pub trait Set {
    fn is_empty(&self) -> bool;
    fn contains(&self, target: i32) -> bool;
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set>;
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set>;
}
pub struct Empty {}
impl Set for Empty {
    fn is_empty(&self) -> bool {
        true
    }
    fn contains(&self, target: i32) -> bool {
        false
    }
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set> {
        if self.contains(value) {
            return self;
        }
        return Box::new(Insert { s1: self, value });
    }
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set> {
        right
    }
}
pub struct Insert {
    pub s1: Box<dyn Set>,
    pub value: i32,
}
impl Set for Insert {
    fn is_empty(&self) -> bool {
        false
    }
    fn contains(&self, target: i32) -> bool {
        self.value == target || self.s1.contains(target)
    }
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set> {
        if self.contains(self.value) {
            return self;
        }
        return Box::new(Insert { s1: self, value });
    }
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set> {
        Box::new(Union {
            s1: self,
            s2: right,
        })
    }
}
pub struct Union {
    pub s1: Box<dyn Set>,
    pub s2: Box<dyn Set>,
}
impl Set for Union {
    fn is_empty(&self) -> bool {
        self.s1.is_empty() && self.s2.is_empty()
    }
    fn contains(&self, target: i32) -> bool {
        self.s1.contains(target) || self.s2.contains(target)
    }
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set> {
        if self.contains(value) {
            return self;
        }
        return Box::new(Insert { s1: self, value });
    }
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set> {
        Box::new(Union {
            s1: self,
            s2: right,
        })
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
