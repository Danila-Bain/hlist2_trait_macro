# `TraitHList` macro for `hlist2` crate (compile-time heterogeneous lists).

This crate defines a procedural macro [`TraitHList!`] that automatically generates trait implementations
for [heterogeneous lists](https://docs.rs/hlist2/latest/hlist2/macro.hlist.html)
from the [`hlist2`](https://docs.rs/hlist2) crate.

It allows you to apply any trait’s methods *element-wise* across all items in an [`hlist2::hlist!`],
with support of generic parameters and lifetimes.

##  Features

- 🟢 Automatically implements a new `*_HList` trait corresponding to any existing trait.  
- 🟢 Supports traits with generic parameters, lifetimes, and `where` clauses.  
- 🟢 Handles methods with all receiver types: `self`, `&self`, and `&mut self`.  
- 🟢 Works with arbitrary argument and return types.  
- ⚠️ Methods returning `bool` automatically gain `.all_*()` and `.any_*()` variants.  
- 🟡 Method renaming supported with `#[name = "custom_name"]`.  

## Example

```rust
use hlist2::hlist;
use hlist2_trait_macro::TraitHList;

trait Check {
    fn is_integer(&self) -> bool;
    fn is_positive(&self) -> bool;
}

impl Check for i32 {
    fn is_integer(&self) -> bool{
        true
    }
    fn is_positive(&self) -> bool {
        *self > 0
    }
}
impl Check for f64 {
    fn is_integer(&self) -> bool{
        false
    }
    fn is_positive(&self) -> bool {
        *self > 0.0
    }
}

TraitHList!{
    CheckHList for trait Check {
        fn is_integer(&self) -> bool;

        #[name = are_positive]
        fn is_positive(&self) -> bool;
        
    }
}

fn main() {
    let xs = hlist![1, -2, 3, -5.0, 6.0, -7.0];

    assert_eq!(xs.is_integer(), hlist![true, true, true, false, false, false]);
    assert!(xs.all_is_integer() == false);
    assert!(xs.any_is_integer() == true);
    
    assert_eq!(xs.are_positive(), hlist![true, false, true, false, true, false]);
    assert!(xs.all_are_positive() == false);
    assert!(xs.any_are_positive() == true);
}
```

== Why?

Current implementation for iteration of heterogeneous collections like implemented in [`hlist2`] 
is slow and limited. The alternative approach with `Vec<Box<dyn Trait>>` objects has overhead at runtime.
Implementation of [`TraitHList!`] acts like an unrolled version of loops over static [`hlist2::hlist!`].
Custom traits with methods accepting mutable references can achieve many things that are also possible 
with regular loops, but do not require objects to have the same type. This is especially useful
for types, that rely on anonymous functions without dynamic dispatch with `dyn` traits.

```rust
use hlist2::hlist;
use hlist2_trait_macro::TraitHList;

trait AddOne {
    fn add_one(&self, x: &mut usize);
}

impl AddOne for () {
    fn add_one(&self, x: &mut usize) {
       *x += 1;
    }
}

TraitHList!{
    AddOneHList for trait AddOne {
        fn add_one(&self, x: &mut usize);
    }
}

let mut x = 0;
let list = hlist![(), (), (), (), ()];
list.add_one(&mut x);
assert!(x == 5);
```
