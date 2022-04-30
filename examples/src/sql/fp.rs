type Schema = Vec<String>;
type Fields = Vec<String>;

enum Operator {
    Scan{
        name: String,
        schema: Schema,
        delim: char,
        ext_schema: bool 
    },
    Print{parent: Box<Operator>},
    Project{out: Schema, in_: Schema, parent: Box<Operator>},
    Filter{pred: Predicate, parent: Box<Operator>},
    Join{left: Box<Operator>, right: Box<Operator>},
    Group{keys: Schema, agg: Schema, parent: Box<Operator>},
    HashJoin{left: Box<Operator>, right: Box<Operator>},
}

enum Predicate {
    Eq{a: Ref, b: Ref},
    Ne{a: Ref, b: Ref},
}

enum Ref {
    Field{name: String},
    Value{x: i32}, // Any
}

struct Record {
    fields: Fields,
    
}

fn process_csv(filename: String, schema: Schema, field_delimiter: char, external_schema: bool, yld: fn(Record) -> ()) {
}
