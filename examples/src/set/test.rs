#[derive(Debug)]
pub enum Set {
    Empty {},
    Insert { s1: Box<Set>, value: i32 },
    Union { s1: Box<Set>, s2: Box<Set> },
}
pub fn is_empty(set: &Set) -> bool {
    return match set {
        Set::Empty {} => true,
        Set::Insert { .. } => false,
        Set::Union { s1, s2 } => is_empty(s1) && is_empty(s2),
    };
}
pub fn contains(set: &Set, target: i32) -> bool {
    return match set {
        Set::Empty {} => false,
        Set::Insert { s1, value } => *value == target || contains(s1, target),
        Set::Union { s1, s2 } => contains(s1, target) || contains(s2, target),
    };
}
pub fn insert(set: Set, value: i32) -> Set {
    if contains(&set, value) {
        return set;
    }
    return Set::Insert {
        s1: Box::new(set),
        value,
    };
}
pub fn union(left: Set, right: Set) -> Set {
    match left {
        Set::Empty {} => right,
        _ => Set::Union {
            s1: Box::new(left),
            s2: Box::new(right),
        },
    }
}
pub fn demo() {
    let empty = Box::new(Set::Empty {});
    let set = insert(*empty, 1);
    let set2 = Box::new(Set::Insert {
        s1: Box::new(Set::Empty {}),
        value: 1,
    });
    let _set3 = union(set, *set2);
}
