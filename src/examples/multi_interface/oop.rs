trait Comparible {
    fn compare(&self, other: &Self) -> bool;
}

trait Copyable {
    fn copy(&self) -> Self;
}

pub struct Thing {
    value: i32,
}

pub struct OnlyCopyableThing {
    value: i32,
}

impl Comparible for Thing {
    fn compare(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Copyable for OnlyCopyableThing {
    fn copy(&self) -> Self {
        OnlyCopyableThing { value: self.value }
    }
}

impl Copyable for Thing {
    fn copy(&self) -> Self {
        Thing { value: self.value }
    }
}

fn demo() {
    let a = Thing { value: 1 };
    let b = Thing { value: 2 };
    a.compare(&b);
    let c = a.copy();
}
