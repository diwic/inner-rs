The `inner!` macro makes descending into an enum variant
more ergonomic. The `some!` and `ok!` macros turns an enum
into an `Option` or `Result`. 

[Crates.io](https://crates.io/crates/inner/)

[API Documentation](http://diwic.github.io/rs-docs/inner/index.html)

# Helpful unwrap
The simplest case is almost like unwrap:

```rust
let x = Some(1);
let y: Result<_, ()> = Ok(2);
assert_eq!(inner!(x), 1);
assert_eq!(inner!(y), 2);
```

...but if you instead use it on a `None` or `Err` value:

```rust
let z = None;
let y = inner!(z);
```

...it will panic, with an error message that points you to a more
helpful location than some line number inside libcore:

```
thread "test" panicked at "Unexpected value found inside "z"", src/lib.rs:23
```

# Error handling
If panic isn't an option - and it usually isn't - just add an `else` clause:

```rust
let x: Result<String, i32> = Err(7);
let y = inner!(x, else { return });
// Since x is an Err, we'll never get here.
println!("The string length is: {}", y.len());
```

You can use the else clause to compute a default value, or use flow control
(e g `break`, `continue`, or `return`).

Want access to what's inside the `Err` value in your `else` clause?
No problem, just add a `|variable|` after `else`, like this:

```rust
let x: Result<String, i32> = Err(7);
let y = inner!(x, else |e| {
    assert_eq!(e, 7);
    (e + 2).to_string()
});
assert_eq!(&y, "9");
```

Note: This does not turn your else clause into a closure, so you can still use
(e g) `return` the same way as before.

# It works with your enums too

It does not work only with `Option` and `Result`. Just add an `if` clause:

```rust
enum Fruit {
    Apple(i32),
    Orange(i16),
}

let z = Fruit::Apple(15);
let y = inner!(z, if Fruit::Apple, else {
    println!("I wanted an apple and I didn't get one!");
    0
});
assert_eq!(y, 15);
```

You can skip the `else` clause to panic in case the enum is not
the expected variant.

Note that in this case, the entire item (instead of the contents inside
`Err`) is passed on to the `else` clause:

```rust
#[derive(Eq, PartialEq, Debug)]
enum Fruit {
    Apple(i32),
    Orange(i16),
}

let z = Fruit::Orange(15);
inner!(z, if Fruit::Apple, else |e| {
    assert_eq!(e, Fruit::Orange(15));
    return;
});
```

You can also turn your enum into a `Option` with the `Some` macro:

```rust
assert_eq!(some!(Fruit::Apple(15), if Fruit::Apple), Some(15));
assert_eq!(some!(Fruit::Orange(5), if Fruit::Apple), None);
assert_eq!(some!(Fruit::Orange(5), if Fruit::Apple, else |e| {Some(e + 2)}), Some(7));
```

Or into a `Result` with the `ok!()` macro:

```rust
assert_eq!(ok!(Fruit::Apple(15), if Fruit::Apple), Ok(15));
assert_eq!(ok!(Fruit::Orange(5), if Fruit::Apple), Err(Fruit::Orange(5)));

assert_eq!(ok!(Fruit::Orange(5), if Fruit::Apple, or |e| {e + 70}), Err(75));
assert_eq!(ok!(Fruit::Orange(5), if Fruit::Apple, else {Err(75)}), Err(75));
```

Notice that the `ok!()` macro has an optional `or` clause that encapsulates the
expression in an `Err`, whereas the `else` clause gives you maximum flexibility
to return either an `Err` or an `Ok`.


Another option is to implement this crate's `IntoResult` trait for
your enum. Then you don't have to write an `if` clause to tell what
enum variant you want to descend into, and you can choose more than
one enum variant to be `Ok`:

```rust
enum Fruit {
    Apple(i32),
    Orange(i16),
    Rotten,
}

impl IntoResult<i32, ()> for Fruit {
    fn into_result(self) -> Result<i32, ()> {
        match self {
            Fruit::Apple(i) => Ok(i),
            Fruit::Orange(i) => Ok(i as i32),
            Fruit::Rotten => Err(()),
        }
    }
}

assert_eq!(9, inner!(Fruit::Apple(9)));
```

# License
Apache2.0/MIT

