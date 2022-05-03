#[test]
fn test_set_oop() {
    use examples::set::oop::*;

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
    use examples::set::fp::*;
    
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

#[test]
fn test_shape2_oop() {
    use examples::shape2::oop::*;

    let circle = Circle;
    assert!(circle.side_count() == 1);
    assert!(circle.internal_angle() == 0);

    let triangle = Triangle;
    assert!(triangle.side_count() == 3);
    assert!(triangle.internal_angle() == 180);
}

#[test]
fn test_mutable_oop() {
    use examples::mutable::oop::*;

    let mut light = RGB {
        r: 10,
        g: 20,
        b: 30,
    };
    
    assert!(light.get_brightness() == 20);
    
    light.decrease_brightness();
    assert!(light.get_brightness() == 19);

    light.turn_off();
    assert!(light.get_brightness() == 0);
}

