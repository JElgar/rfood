use std::collections::HashMap;
use syn::{Ident, Type};

struct Delta {
    types: HashMap<Ident, Type>,
}

use syn::{Item, ItemEnum, ItemTrait, Variant, ItemStruct, Type, Ident, TraitItem, TraitItemMethod};

trait Transformable {
    fn generate_delta(&self) -> Delta;
    fn transform(&self, delta: Delta) -> Self;
}

impl GenerateDelta for ItemEnum {
    fn generate_delta(&self) -> Delta {
        return Delta {

        };
    }
}

fn generate_delta(item: Item) -> Delta {
    match item {

        _ => panic!("Not implemented"),
    }
}

fn generate_delta_function {
}
