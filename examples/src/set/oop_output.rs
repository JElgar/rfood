pub enum Set {
    Empty {},
    Insert { set1: Box<Set>, value: i32 },
    Union { set1: Box<Set>, set2: Box<Set> },
}
pub fn is_empty(set: &Set) -> bool {
    match &set {
        Set::Empty {} => {
            return true;
        }
        Set::Insert { set1, value } => {
            return false;
        }
        Set::Union { set1, set2 } => {
            return is_empty(&*set1) && is_empty(&*set2);
        }
    }
}
pub fn contains(set: &Set, i: i32) -> bool {
    match &set {
        Set::Empty {} => {
            return false;
        }
        Set::Insert { set1, value } => {
            return *value == i || contains(&*set1, i);
        }
        Set::Union { set1, set2 } => {
            return contains(&*set1, i) || contains(&*set2, i);
        }
    }
}
pub fn insert(set: Set, i: i32) -> Set {
    match &set {
        Set::Empty {} => {
            if contains(&set, i) {
                return set;
            }
            return Set::Insert {
                set1: Box::new(set),
                value: i,
            };
        }
        Set::Insert { set1, value } => {
            if contains(&set, i) {
                return set;
            }
            return Set::Insert {
                set1: Box::new(set),
                value: i,
            };
        }
        Set::Union { set1, set2 } => {
            if contains(&set, i) {
                return set;
            }
            return Set::Insert {
                set1: Box::new(set),
                value: i,
            };
        }
    }
}
pub fn union(set: Set, s: Set) -> Set {
    match &set {
        Set::Empty {} => {
            return s;
        }
        Set::Insert { set1, value } => {
            return Set::Union {
                set1: Box::new(set),
                set2: Box::new(s),
            };
        }
        Set::Union { set1, set2 } => {
            return Set::Union {
                set1: Box::new(set),
                set2: Box::new(s),
            };
        }
    }
}
pub fn type_id(set: &Set) -> i32 {
    match &set {
        Set::Empty {} => {
            return 10;
        }
        _ => 0,
    }
}
pub fn demo() {
    let empty = Box::new(Set::Empty {});
    let set = insert(*empty, 1);
    let set2 = Box::new(Set::Insert {
        set1: Box::new(Set::Empty {}),
        value: 1,
    });
    let _set3 = union(set, *set2);
}
