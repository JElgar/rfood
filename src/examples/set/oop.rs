pub trait Set {
    fn is_empty(&self) -> bool;
    fn contains(&self, i: i32) -> bool;
    fn insert(self: Box<Self>, i: i32) -> Box<dyn Set>;
    fn union(self: Box<Self>, s: Box<dyn Set>) -> Box<dyn Set>;
    // fn debug(&self) -> String;
}

// impl std::fmt::Debug for dyn Set {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.debug())
//     }
// }

pub struct Empty{}
impl Set for Empty {
    fn is_empty(&self) -> bool {
        return true;
    }
    fn contains(&self, _i: i32) -> bool {
        return false;
    }
    fn union(self: Box<Self>, s: Box<dyn Set>) -> Box<dyn Set> {
        return s;
    }
    fn insert(self: Box<Self>, i: i32) -> Box<dyn Set> {
        if self.contains(i) {
            return self;
        }
        return Box::new(Insert{set1: self, value: i});
        // return if self.contains(i) {
        //     self
        // } else {
        //     Box::new(Insert{set: self, value: i})
        // };
    }
    // fn debug(&self) -> String {
    //     String::from("Empty")
    // }
} 

pub struct Insert{
    pub set1: Box<dyn Set>,
    pub value: i32,
}
impl Set for Insert {
    fn is_empty(&self) -> bool {
        return false;
    }
    fn contains(&self, i: i32) -> bool {
        return self.value == i || self.set1.contains(i);
    }
    fn union(self: Box<Self>, s: Box<dyn Set>) -> Box<dyn Set> {
        return Box::new(Union{set1: self, set2: s});
    }
    fn insert(self: Box<Self>, i: i32) -> Box<dyn Set> {
        if self.contains(i) {
            return self;
        }
        return Box::new(Insert{set1: self, value: i});
    }
    // fn debug(&self) -> String {
    //     format!("Insert( {}, {} )", self.value, self.set.debug())
    // }
}


pub struct Union {
    pub set1: Box<dyn Set>,
    pub set2: Box<dyn Set>,
}
impl Set for Union {
    fn is_empty(&self) -> bool {
        return self.set1.is_empty() && self.set2.is_empty();
    }
    fn contains(&self, i: i32) -> bool {
        return self.set1.contains(i) || self.set2.contains(i);
    }
    fn union(self: Box<Self>, s: Box<dyn Set>) -> Box<dyn Set> {
        return Box::new(Union{set1: self, set2: s});
    }
    fn insert(self: Box<Self>, i: i32) -> Box<dyn Set> {
        if self.contains(i) {
            return self;
        }
        return Box::new(Insert{set1: self, value: i});
    }
    // fn debug(&self) -> String {
    //     format!("Union( {}, {} )", self.set1.debug(), self.set2.debug())
    // }
}

pub fn demo() {
    let empty = Box::new(Empty{});
    let set = empty.insert(1);

    let set2 = Box::new(Insert {
        set1: Box::new(Empty{}),
        value: 1
    });

    let _set3 = set.union(set2);
}
