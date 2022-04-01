// enum Set {
//     Empty {},
//     Insert { set: Box<Set>, value: i32 },
//     Union { set1: Box<Set>, set2: Box<Set> },
// }
// fn is_empty(set: &Set) -> bool {
//     match set {
//         Set::Empty {} => {
//             return true;
//         }
//         Set::Insert { set, value } => {
//             return false;
//         }
//         Set::Union { set1, set2 } => {
//             return is_empty(&*set1) && is_empty(&*set2);
//         }
//     }
// }
// fn contains(set: &Set, i: i32) -> bool {
//     match set {
//         Set::Empty {} => {
//             return false;
//         }
//         Set::Insert { set, value } => {
//             return *value == i || contains(&*set, i);
//         }
//         Set::Union { set1, set2 } => {
//             return contains(&*set1, i) || contains(&*set2, i);
//         }
//     }
// }
// fn insert(set: Set, i: i32) -> Set {
//     match set {
//         Set::Empty {} => {
//             if contains(&set, i) {
//                 return *set;
//             }
//             return Insert { set: set, value: i };
//         }
//         Set::Insert { set, value } => {
//             if contains(&set, i) {
//                 return *set;
//             }
//             return Insert { set: set, value: i };
//         }
//         Set::Union { set1, set2 } => {
//             if contains(&set, i) {
//                 return *set;
//             }
//             return Insert { set: set, value: i };
//         }
//     }
// }
// fn union(set: Set, s: Set) -> Set {
//     match set {
//         Set::Empty {} => {
//             return *s;
//         }
//         Set::Insert { set, value } => {
//             return Union { set1: set, set2: s };
//         }
//         Set::Union { set1, set2 } => {
//             return Union { set1: set, set2: s };
//         }
//     }
// }
// pub fn demo() {
//     let empty = Box::new(Set::Empty {});
//     let set = insert(*empty, 1);
//     let set2 = Box::new(Set::Insert {
//         set: Box::new(Set::Empty {}),
//         value: 1,
//     });
//     let _set3 = union(*set, set2);
// }
