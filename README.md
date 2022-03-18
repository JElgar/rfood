# Bidirectional transformer for rust

## TODO

### Typing 

- [ ] Correct return types in function signature
- [ ] Correct return types from method returns (for now if box dereference). If explicitly wrapping response in box then remove this wrapping.
- [ ] Fix constructors of enums to ensure boxes are created as required
 
### Generics

- [ ] Get basic generics working for methods and traits
- [ ] Support generics in both cases 
- [ ] Handle generics shadowing 
- [ ] Look into GADT 

## Docs

The first step of the transformer is collect gamma. Gamma is the global context and has the following attributes:

- **Enums** - The enums in the file (these are the Datatypes)
- **Traits** - The traits in the file (these are the Interfaces)
- **Constructors** - These are the variants of the enums 
- **Generators** - The structs that implement the traits, these are stored as a tuple, the first item is the struct its self and the second is its implementation of the trait.
- **Destructors** - These are the methods in the trait
- **Consumers** - Methods that take an enum as the first argument and return any

For now gamma is collect globally. I.e. gamma is only populated for items at the root of the file. A future extension of this project would be to scope the collection of these items as well as the delta in the transformations.
