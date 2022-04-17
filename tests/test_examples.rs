#[test]
fn test_set_oop() {
    use rfood::examples::set::oop::*;

    let empty = Box::new(Empty{});
    let set = empty.insert(1);

    let set2 = Box::new(Insert {
        set1: Box::new(Empty{}),
        value: 4
    });

    let set3 = set.union(set2);

    assert!(set3.contains(1));
    assert!(set3.contains(4));
    assert!(!set3.contains(2));

    let set3 = set3.insert(2);
    assert!(set3.contains(2));
}

#[test]
fn test_set_fp() {
    use rfood::examples::set::fp::*;
    
    let empty = Box::new(Set::Empty {});
    let set = insert(*empty, 1);

    let set2 = Box::new(Set::Insert {
        s1: Box::new(Set::Empty {}),
        value: 1,
    });

    let set3 = union(set, *set2);

    assert!(contains(&set3, 1));
    assert!(!contains(&set3, 4));

    let set3 = insert(set3, 2);
    assert!(contains(&set3, 2));
}
