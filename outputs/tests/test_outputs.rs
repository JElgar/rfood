#[test]
fn test_output_set_oop() {
    use outputs::set::oop::*;
    
    let empty = Box::new(Set::Empty {});
    let set = insert(*empty, 1);

    let set2 = Box::new(Set::Insert {
        set1: Box::new(Set::Empty {}),
        value: 1,
    });

    let set3 = union(set, *set2);

    assert!(contains(&set3, 1));
    assert!(!contains(&set3, 4));

    let set3 = insert(set3, 2);
    assert!(contains(&set3, 2));
}

#[test]
fn test_output_set_fp() {
    use outputs::set::fp::*;

    let empty = Box::new(Empty{});
    let set = empty.insert(1);

    let set2 = Box::new(Insert {
        s1: Box::new(Empty{}),
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
fn test_output_shape_oop() {
    use outputs::shape2::oop::*;
    
    let circle = Shape::Circle;
    assert!(side_count(&circle) == 1);
    assert!(internal_angle(&circle) == 0);
    
    let triangle = Shape::Triangle;
    assert!(side_count(&triangle) == 3);
    println!("The internal angle of the shape is {}", internal_angle(&triangle));
    assert!(internal_angle(&triangle) == 180);
}

#[test]
fn test_output_mutable_opp() {
    use outputs::mutable::oop::*;

    let mut light = Light::RGB{r: 10, g: 20, b: 30};
    assert_eq!(get_brightness(&light), 20);

    light = increase_brightness(light);
    assert_eq!(get_brightness(&light), 21);
}
