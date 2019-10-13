A library for cloning trait objects.

## Instability

This library depends on an undocumented detail of fat pointer layouts.

For that reason, this library is intentionally marked as unstable.

## Example

### Making a cloneable user-defined trait

```rust
use clone_into_box::{CloneIntoBox, CloneIntoBoxExt};

// Make the trait a subtrait of `CloneIntoBox`
pub trait MyTrait: CloneIntoBox {
    fn hello(&self) -> String;
}

// Manually implement `Clone` using `clone_into_box`
impl Clone for Box<dyn MyTrait + '_> {
    fn clone(&self) -> Self {
        // Use (**self) to prevent ambiguity.
        // Otherwise you may run into a mysterious stack overflow.
        (**self).clone_into_box()
    }
}

#[derive(Debug, Clone)]
struct Foo(String);

impl MyTrait for Foo {
    fn hello(&self) -> String {
        format!("Hello, {}!", self.0)
    }
}

fn main() {
    let x: Box<dyn MyTrait> = Box::new(Foo(String::from("John")));
    assert_eq!(x.hello(), "Hello, John!");
    let y = x.clone();
    assert_eq!(y.hello(), "Hello, John!");
}
```

### Making a cloneable variant of an existing trait

```rust
use clone_into_box::{CloneIntoBox, CloneIntoBoxExt};

// Use a "new trait" pattern to create a trait for `ExistingTrait + CloneIntoBox`
pub trait FnClone: Fn() -> String + CloneIntoBox {}
impl<T: Fn() -> String + CloneIntoBox + ?Sized> FnClone for T {}

// Manually implement `Clone` using `clone_into_box`
impl Clone for Box<dyn FnClone + '_> {
    fn clone(&self) -> Self {
        // Use (**self) to prevent ambiguity.
        // Otherwise you may run into a mysterious stack overflow.
        (**self).clone_into_box()
    }
}

fn main() {
    let name = String::from("John");
    let x: Box<dyn FnClone> = Box::new(move || format!("Hello, {}!", name));
    assert_eq!(x(), "Hello, John!");
    let y = x.clone();
    assert_eq!(y(), "Hello, John!");
}
```
