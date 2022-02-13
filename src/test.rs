enum Set { 
    Empty { },
    Insert { set : Box < Set > , value : i32 },
    Union { set1 : Box < Set > , set2 : Box < Set > }
}

enum Set {
    Empty{ },
    Insert{set: Box<Set>, value: i32},
    Union{s1: Box<Set>, s2: Box<Set>},
}
