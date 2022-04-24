pub trait Set {
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set>;
    fn contains(&self, target: i32) -> bool;
    fn is_empty(&self) -> bool;
    fn union(self: Box<Self>, right: Box<dyn Set>) -> Box<dyn Set>;
}
pub struct Empty {}
impl Set for Empty {
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set> {
        if self.contains(value) {
            return self;
        }
        return Box::new(Insert { s1: self, value });
    }
    fn contains(&self, target: i32) -> bool {
        false
    }
    fn is_empty(&self) -> bool {
        true
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
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set> {
        if self.contains(self.value) {
            return self;
        }
        return Box::new(Insert { s1: self, value });
    }
    fn contains(&self, target: i32) -> bool {
        self.value == target || self.s1.contains(target)
    }
    fn is_empty(&self) -> bool {
        false
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
    fn insert(self: Box<Self>, value: i32) -> Box<dyn Set> {
        if self.contains(value) {
            return self;
        }
        return Box::new(Insert { s1: self, value });
    }
    fn contains(&self, target: i32) -> bool {
        self.s1.contains(target) || self.s2.contains(target)
    }
    fn is_empty(&self) -> bool {
        self.s1.is_empty() && self.s2.is_empty()
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
