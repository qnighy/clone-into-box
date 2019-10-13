//! A library for cloning trait objects.
//!
//! ## Instability
//!
//! This library depends on an undocumented detail of fat pointer layouts.
//!
//! For that reason, this library is intentionally marked as unstable.
//!
//! ## Example
//!
//! ### Making a cloneable user-defined trait
//!
//! ```
//! use clone_into_box::{CloneIntoBox, CloneIntoBoxExt};
//!
//! // Make the trait a subtrait of `CloneIntoBox`
//! pub trait MyTrait: CloneIntoBox {
//!     fn hello(&self) -> String;
//! }
//!
//! // Manually implement `Clone` using `clone_into_box`
//! impl Clone for Box<dyn MyTrait + '_> {
//!     fn clone(&self) -> Self {
//!         // Use (**self) to prevent ambiguity.
//!         // Otherwise you may run into a mysterious stack overflow.
//!         (**self).clone_into_box()
//!     }
//! }
//!
//! #[derive(Debug, Clone)]
//! struct Foo(String);
//!
//! impl MyTrait for Foo {
//!     fn hello(&self) -> String {
//!         format!("Hello, {}!", self.0)
//!     }
//! }
//!
//! fn main() {
//!     let x: Box<dyn MyTrait> = Box::new(Foo(String::from("John")));
//!     assert_eq!(x.hello(), "Hello, John!");
//!     let y = x.clone();
//!     assert_eq!(y.hello(), "Hello, John!");
//! }
//! ```
//!
//! ### Making a cloneable variant of an existing trait
//!
//! ```
//! use clone_into_box::{CloneIntoBox, CloneIntoBoxExt};
//!
//! // Use a "new trait" pattern to create a trait for `ExistingTrait + CloneIntoBox`
//! pub trait FnClone: Fn() -> String + CloneIntoBox {}
//! impl<T: Fn() -> String + CloneIntoBox + ?Sized> FnClone for T {}
//!
//! // Manually implement `Clone` using `clone_into_box`
//! impl Clone for Box<dyn FnClone + '_> {
//!     fn clone(&self) -> Self {
//!         // Use (**self) to prevent ambiguity.
//!         // Otherwise you may run into a mysterious stack overflow.
//!         (**self).clone_into_box()
//!     }
//! }
//!
//! fn main() {
//!     let name = String::from("John");
//!     let x: Box<dyn FnClone> = Box::new(move || format!("Hello, {}!", name));
//!     assert_eq!(x(), "Hello, John!");
//!     let y = x.clone();
//!     assert_eq!(y(), "Hello, John!");
//! }
//! ```

// NOTE: this library doesn't explicitly use a library feature which is marked unstable.
// Nonetheless it's intentionally made unstable because it relies on the internal detail
// of fat pointer layouts.
#![feature(rustc_private)]

use std::alloc::{alloc, dealloc, Layout};
use std::mem::forget;
use std::ptr::write;

/// A (possibly unsized) value which can be cloned into a pre-allocated space.
///
/// Users can use `CloneIntoBoxExt` to clone it into `Box<T>`.
pub trait CloneIntoBox {
    /// Clone into the specified place.
    ///
    /// ## Effect
    ///
    /// After successful invocation, the area pointed to
    /// by `ptr` will contain a valid representation of `Self`
    /// with auxiliary data contained in the `self` fat pointer.
    ///
    /// ## Safety
    ///
    /// The `ptr` parameter must point to an uninitialized area
    /// which has enough space of `std::mem::size_of_val(self)` bytes
    /// and is aligned to `std::mem::align_of_val(self)` bytes.
    ///
    /// ## Panics
    ///
    /// This method isn't expected to panic in normal cases,
    /// but the caller must handle panics carefully for safety.
    unsafe fn clone_into_ptr(&self, ptr: *mut u8);
}

impl<T: Clone> CloneIntoBox for T {
    unsafe fn clone_into_ptr(&self, ptr: *mut u8) {
        write(ptr as *mut T, self.clone())
    }
}

/// An extension trait for cloning trait objects into `Box`es.
///
/// ## Examples
///
/// See [crate documentation](index.html) for examples.
pub trait CloneIntoBoxExt: CloneIntoBox {
    /// Clone the provided value into a `Box`-allocated space.
    ///
    /// ## Examples
    ///
    /// See [crate documentation](index.html) for examples.
    fn clone_into_box(&self) -> Box<Self> {
        struct Guard {
            ptr: *mut u8,
            layout: Layout,
        }
        impl Drop for Guard {
            fn drop(&mut self) {
                unsafe {
                    dealloc(self.ptr, self.layout);
                }
            }
        }

        let layout = Layout::for_value::<Self>(self);
        let ptr = unsafe { alloc(layout) };
        let guard = Guard { ptr, layout };
        unsafe {
            self.clone_into_ptr(ptr);
        }
        forget(guard);
        unsafe { Box::from_raw(assign_thin_mut(self, ptr)) }
    }
}
impl<T: CloneIntoBox + ?Sized> CloneIntoBoxExt for T {}

fn assign_thin_mut<T: ?Sized>(meta: *const T, thin: *mut u8) -> *mut T {
    let mut fat = meta as *mut T;
    // Assumes that the first *mut u8 is the thin pointer.
    unsafe {
        *(&mut fat as *mut *mut T as *mut *mut u8) = thin;
    }
    fat
}

#[cfg(test)]
mod tests {
    use super::*;

    pub trait FnClone: Fn() -> String + CloneIntoBox {}
    impl<T: ?Sized> FnClone for T where T: Fn() -> String + CloneIntoBox {}

    impl<'a> Clone for Box<dyn FnClone + 'a> {
        fn clone(&self) -> Self {
            (**self).clone_into_box()
        }
    }

    #[test]
    fn test_clone_fn() {
        let s = String::from("Hello,");
        let f: Box<dyn FnClone> = Box::new(move || format!("{} world!", s));
        assert_eq!(f(), "Hello, world!");
        let ff = f.clone();
        assert_eq!(ff(), "Hello, world!");
    }

    #[test]
    #[should_panic(expected = "PanicClone::clone() is called")]
    fn test_clone_panic() {
        struct PanicClone;
        impl Clone for PanicClone {
            fn clone(&self) -> Self {
                panic!("PanicClone::clone() is called");
            }
        }
        let s = String::from("Hello,");
        let p = PanicClone;
        let f: Box<dyn FnClone> = Box::new(move || {
            let _ = &p;
            format!("{} world!", s)
        });
        assert_eq!(f(), "Hello, world!");
        let _ = f.clone();
    }
}
