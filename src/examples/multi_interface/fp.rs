enum Comparible {
}

enum Copyable {
    OnlyCopyableThing { value: i32 },
}

enum CopyableAndComparable {
    Thing { value: i32 },
}

fn compare(comparible: &Comparible, comparible: Comparible) -> bool {
    match comparible {
        Comparible::Thing(value) => *value == *value,
    }
}

fn copy(comparible: &Copyable, comparible: Comparible) -> bool {
    match comparible {
        Comparible::Thing(value) => *value == *value,
    }
}

fn compare_cac(comparible: &CopyableAndComparable, comparible: Comparible) -> bool {
    match comparible {
        CopyableAndComparable::Thing(value) => *value == *value,
    }
}

fn copy_cac(comparible: &CopyableAndComparable, comparible: Comparible) -> bool {
    match comparible {
        Comparible::Thing(value) => *value == *value,
    }
}

enum Copyable {
    Thing { value: i32 },
}
fn copy(copyable: &Copyable) -> Self {
    match copyable {
        Copyable::Thing(value) => Thing { value: *value },
    }
}
