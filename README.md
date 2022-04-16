# Bidirectional transformer for rust

## TODO

### General

- [x] FP -> OOP 
    - [x] Collect consumers and enums in gamma
    - [x] Transform enums
    - [x] Transform consumers
    - [x] Fix typing of created method calls. This probably means check the type of the reciever and then see the expected type (self arg in the method)
- [x] Update the demo function (or any other functions) to correctly use the updated traits. (Probably do this with visit)
- [x] Support local parms in delta
- [ ] Shadowing for consumer args and match arm params
- [ ] Handle renaming when replacing first arg of consumer (eg left -> self in the union function)
- [ ] Update delta type to store ref_type recursivly. This is so *Box or &* etc can be encoded.
- [ ] Non self methods should just be copied as top level methods oop->fp
- [ ] transform_expr transform if else

### Typing 

- [x] Correct return types in function signature
- [x] Correct return types from method returns (for now if box dereference). If explicitly wrapping response in box then remove this wrapping.
- [x] Fix constructors of enums to ensure boxes are created as required
- [ ] Update RefType to be recursive
- [ ] Update get types stuff to actually use RefType recursivly 
- [ ] Add some kind of simplify function
- [ ] Update transform expr type to handle more complex types, eg Box to ref -> do a &*
 
### Generics

- [x] Get basic generics working for traits
- [ ] Get basic generics working for methods 
- [ ] Support extra generics for the structs (that are not in the trait) 
- [ ] Support generics in both cases 
- [ ] Handle generics shadowing 
- [ ] Look into GADT 

### Inheritance

- [x] Wildcard pattern for basic types

### Mutable

- [ ] Transform any uses of the transfomred destructors, this will need an extra guard check in transform expr 

### Extras 

- [ ] Select which types to transform
- [ ] Enums with same names

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

### Transformations 

#### Trait transformations 

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

#### Type considerations

The rust type system is significantly stricter than that of scalas. For this reason extensions to the transformation rules, as well as the restrictions had to be included.

#### Other transformations 

Having transformed the trait, the next stage is to transform other items in the ast. In reality a recursive approach to transformations throughout would be better and allow all transform to respect the current scope. To test this hypothesis we use this recursive approach for the transformation of other items. 

An important consideration when parsing recursivly is which values are avaialable in the current scope (order does not matter for traits for example). For this implementation we have not included items which are defined after the value we are currently transformed. This is an acceptable limitation given all the types have already been transformed in previous stages.

The approach is as follows:

Trasform each item in the ast that has not already been transformed.

If it is a function, transform each epxression, adding to delta for each let expression. The type of each varaible in the let expression is extracted by a crude type inference which could be extended by linking into the exisitng type inference in the rust compiler. When transforming an expression, recursivly transform any subexpressions.

### Generics

First step is supporting generics in traits. 
When creating the enum add the generics from the trait to the enum.
When creating the consumer add in the generics from the trait, into both the method and the consumers argument.

Then extend to supoort additional generics in the structs that are not in the trait.


