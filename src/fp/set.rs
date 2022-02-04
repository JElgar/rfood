#[derive(Debug)]
enum Set {
    Empty,
    Insert(Box<Set>, i32),
    Union(Box<Set>, Box<Set>),
}

fn is_empty(set: Box<Set>) -> bool {
    return match *set {
        Set::Empty => true,
        Set::Insert(..) => false,
        Set::Union(s1, s2) => is_empty(s1) && is_empty(s2)
    }
}

fn contains(set: &Box<Set>, target: i32) -> bool {
    return match &**set {
        Set::Empty => false,
        Set::Insert(set, value) => *value == target || contains(&set, target),
        Set::Union(s1, s2) => contains(&s1, target) && contains(&s2, target)
    }
}

fn insert(set: Box<Set>, value: i32) -> Box<Set> {
    if contains(&set, value) {
        return set;
    }
    return Box::new(Set::Insert(set, value))
}

pub fn demo() {
    let mut set: Box<Set> = Box::new(Set::Empty);
    set = insert(set, 1);
    let set2 = Box::new(Set::Insert(Box::new(Set::Empty), 1));
    println!("{:?}", set);
    println!("{:?}", set2);
    println!("{:?}", Set::Union(set, set2));
}
