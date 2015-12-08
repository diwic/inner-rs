//! The `inner!` macro makes descending into an enum variant
//! more ergonomic.
//!
//! # Helpful unwrap
//! The simplest case is almost like unwrap:
//!
//! ```
//! # #[macro_use] extern crate inner;
//! # fn main() {
//! let x = Some(1);
//! let y: Result<_, ()> = Ok(2);
//! assert_eq!(inner!(x), 1);
//! assert_eq!(inner!(y), 2);
//! # }
//! ```
//!
//! ...but if you instead use it on a `None` or `Err` value:
//!
//! ```ignore
//! let z = None;
//! let y = inner!(z);
//! ```
//!
//! ...it will panic, with an error message that points you to a more
//! helpful location than some line number inside libcore:
//!
//! ```ignore
//! thread "test" panicked at "Unexpected value found inside "z"", src/lib.rs:23
//! ```
//!
//! # Error handling
//! If panic isn't an option - and it usually isn't - just add an `else` clause:
//!
//! ```
//! # #[macro_use] extern crate inner;
//! # fn main() {
//! let x: Result<String, i32> = Err(7);
//! let y = inner!(x, else { return });
//! // Since x is an Err, we'll never get here.
//! println!("The string length is: {}", y.len());
//! # }
//! ```
//!
//! You can use the else clause to compute a default value, or use flow control
//! (e g `break`, `continue`, or `return`).
//!
//! Want access to what's inside the `Err` value in your `else` clause?
//! No problem, just add a `|variable|` after `else`, like this:
//!
//! ```
//! # #[macro_use] extern crate inner;
//! # fn main() {
//! let x: Result<String, i32> = Err(7);
//! let y = inner!(x, else |e| {
//!     assert_eq!(e, 7);
//!     (e + 2).to_string()
//! });
//! assert_eq!(&y, "9");
//! # }
//! ```
//!
//! Note: This does not turn your else clause into a closure, so you can still use
//! (e g) `return` the same way as before.
//!
//! # It works with your enums too
//! It does not work only with `Option` and `Result`. Just add an `if` clause:
//!
//! ```
//! # #[macro_use] extern crate inner;
//! # fn main() {
//! enum Fruit {
//!     Apple(i32),
//!     Orange(i16),
//! }
//! 
//! let z = Fruit::Apple(15);
//! let y = inner!(z, if Fruit::Apple, else {
//!     println!("I wanted an apple and I didn't get one!");
//!     0
//! });
//! assert_eq!(y, 15);
//! # }
//! ```
//!
//! You can skip the `else` clause to panic in case the enum is not
//! the expected variant.
//!
//! Note that in this case, the entire item (instead of the contents inside
//! `Err`) is passed on to the `else` clause:
//!
//! ```
//! # #[macro_use] extern crate inner;
//! # fn main() {
//! #[derive(Eq, PartialEq, Debug)]
//! enum Fruit {
//!     Apple(i32),
//!     Orange(i16),
//! }
//! 
//! let z = Fruit::Orange(15);
//! inner!(z, if Fruit::Apple, else |e| {
//!     assert_eq!(e, Fruit::Orange(15));
//!     return;
//! });
//! # }
//! ```
//!
//! Another option is to implement this crate's `IntoResult` trait for
//! your enum. Then you don't have to write an `if` clause to tell what
//! enum variant you want to descend into, and you can choose more than
//! one enum variant to be `Ok`:
//!
//! ```ignore
//! enum Fruit {
//!     Apple(i32),
//!     Orange(i16),
//!     Rotten,
//! }
//!
//! impl IntoResult<i32, ()> for Fruit {
//!     fn into_result(self) -> Result<i32, ()> {
//!         match self {
//!             Fruit::Apple(i) => Ok(i),
//!             Fruit::Orange(i) => Ok(i as i32),
//!             Fruit::Rotten => Err(()),
//!         }
//!     }
//! }
//!
//! assert_eq!(9, inner!(Fruit::Apple(9)));
//! ```
//!
//! # License
//! Apache2.0/MIT

/// Converts a value into a Result.
/// You can implement this for your own types if you want
/// to use the `inner!` macro in more ergonomic ways.
pub trait IntoResult<T, E> {
    fn into_result(self) -> Result<T, E>;
}


/* 
// Impossible due to conflicting impls :-(
impl<T, E, Z> IntoResult<T, E> for Z where Z: Into<Result<T, E>> {
    fn into_result(self) -> Result<T, E> { self.into() }
}
*/

impl<T, E> IntoResult<T, E> for Result<T, E> {
    #[inline]
    fn into_result(self) -> Result<T, E> { self }
}

impl<T> IntoResult<T, ()> for Option<T> {
    #[inline]
    fn into_result(self) -> Result<T, ()> { self.ok_or(()) }
}

/// The `inner!` macro - see module level documentation for details.
#[macro_export]
macro_rules! inner {
    ($x:expr, if $i:path, else |$e:ident| $b:block) => {
        {
            match $x {
                $i(q) => q,
                $e @ _ => $b,
            }
        }
    };

    ($x:expr, if $i:path, else $b:block) => {
        {
            match $x {
                $i(q) => q,
                _ => $b,
            }
        }
    };

    ($x:expr, else |$e:ident| $b:block) => {
        {
            use $crate::IntoResult;
            match $x.into_result() {
                Ok(q) => q,
                Err($e) => $b,
            }
        }
    };

    ($x:expr, else $b:block) => {
        {
            use $crate::IntoResult;
            match $x.into_result() {
                Ok(q) => q,
                _ => $b,
            }
        }
    };

    ($x:expr, if $i:path) => {
        {
            match $x {
                $i(q) => q,
                _ => panic!("Unexpected value found inside '{}'", stringify!($x)),
            }
        }
    };

    ($x:expr) => {
        {
            use $crate::IntoResult;
            match $x.into_result() {
                Ok(q) => q,
                _ => panic!("Unexpected value found inside '{}'", stringify!($x)),
            }
        }
    };
}

#[test]
fn simple_opt() {
    assert_eq!(inner!(Some(7)), 7);
}

#[test]
#[should_panic]
fn simple_opt_fail() {
    let z: Option<i32> = None;
    inner!(z);
}

#[test]
fn else_clause() {
    let x: Result<String, i32> = Err(7);
    let _ = inner!(x, else { return });
    panic!();
}

#[test]
fn else_clause_2() {
    let x: Result<String, i32> = Err(7);
    let y = inner!(x, else |e| {
        assert_eq!(e, 7);
        (e + 2).to_string()
    });
    assert_eq!(&y, "9");
}

#[test]
fn apple() {
    enum Fruit {
        Apple(i32),
        _Orange(i16),
    }
    let z = Fruit::Apple(15);
    assert_eq!(15, inner!(z, if Fruit::Apple));
}

#[test]
fn if_else() {
    enum Fruit {
        Apple(i32),
        _Orange(i16),
    }
    let z = Fruit::Apple(15);
    assert_eq!(15, inner!(z, if Fruit::Apple, else {
        panic!("Not an apple");
    }));
}

#[test]
fn own_enum() {
    #[derive(Debug, PartialEq, Eq)]
    enum Fruit {
        Apple(i32),
        Orange(i16),
    }

    impl IntoResult<i32, i16> for Fruit {
        fn into_result(self) -> Result<i32, i16> {
            match self {
                Fruit::Apple(i) => Ok(i),
                Fruit::Orange(i) => Err(i),
            }
        }
    }
    let z = Fruit::Orange(15);
    assert_eq!(7, inner!(z, else |e| { (e - 8) as i32 }));

    let z = Fruit::Apple(15);
    assert_eq!(9, inner!(z, if Fruit::Orange, else |e| {
        assert_eq!(e, Fruit::Apple(15));
        9
    }));

}

