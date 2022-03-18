# Bidirectional transformer for rust

## TODO

### General

- [ ] Update the demo function (or any other functions) to correctly use the updated traits. (Probably do this with visit)

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

### Context

#### Gamma

The first step of the transformer is collect gamma. Gamma is the global context and has the following attributes:

- **Enums** - The enums in the file (these are the Datatypes)
- **Traits** - The traits in the file (these are the Interfaces)
- **Constructors** - These are the variants of the enums 
- **Generators** - The structs that implement the traits, these are stored as a tuple, the first item is the struct its self and the second is its implementation of the trait.
- **Destructors** - These are the methods in the trait
- **Consumers** - Methods that take an enum as the first argument and return any

For now gamma is collect globally. I.e. gamma is only populated for items at the root of the file. A future extension of this project would be to scope the collection of these items as well as the delta in the transformations.

##### Restrictions

This implementation focuses on specific types of these implementations. Although this could be extended to support different implementation styles of references for now the basic cases are implemented as a proof of concept.

For this reasons the following restrictions are inplace:

- If a trait returns an instance of it self it must be a dyn box
- Trait/generator method implementations must have a single return statement (TODO: Extend this)
- If enums contain instances of them selves is must be a box (recursive definiton)

#### Delta 

Delta contains the type information during the transfomration. In this implementation this is currently very limited. This is a clear scope for extension of this project.

The delta contains a hashmap of varaiables to types. 

### The transformations

The transformer currently only transforms traits. Firstly it parses the provided file, it then generates gamma for it and uses gamma to transform each trait. A trait is transformed as follows:

1. For each of the trait's generators an enum variant is created.
2. An enum is then created with the same name and generics as the original trait and with the variants created in step 1.
3. For each destructor of the trait a consumer is then created this is done by the following steps:
   1. Transform the signature with the following steps:
   2. For each of the generators of the enum create a match statement arm with the following steps:
      1. Find the implementation of the method in the generator implementation
      2. Generate delta for the method (more info [here](#Delta))
      3. Extract the body of the method and transform it as follows:
         1. TODO (all the visitor stuff)
      4. Generate a match arm that matches on the enum type (with all the args exposed) with the transformed body as the reponse.
   3. Create a match statement with the arms created in step 3.2.
   4. Create a function with the signature from 3.1 and the arms for 3.2.
4. Return the new enum and consumers as a list of items


