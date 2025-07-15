<h1 align="center">placing</h1>
<div align="center">
  <strong>
    A prototype notation for referentially stable constructors
  </strong>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/placing">
    <img src="https://img.shields.io/crates/v/placing.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/placing">
    <img src="https://img.shields.io/crates/d/placing.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/placing">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>

<div align="center">
  <h3>
    <a href="https://docs.rs/placing">
      API Docs
    </a>
    <span> | </span>
    <a href="https://github.com/yoshuawuyts/placing/releases">
      Releases
    </a>
    <span> | </span>
    <a href="https://github.com/yoshuawuyts/placing/blob/main/.github/CONTRIBUTING.md">
      Contributing
    </a>
  </h3>
</div>

## Installation
```sh
$ cargo add placing
```

## Example

This crate enables address-sensitive types to be constructed. That is: types
whose address in memory can't change over time. As well as types that might OOM
if they are constructed on the stack before being copied to the heap. To start
we create a new type with the `placing` attribute macro. This sets up the right
internal type hierarchy for us.

```rust
#[placing::placing]
struct Cat {
    age: u8,
}
```

We can then define our constructors and other methods. Constructors need to end
with a struct expression as the last statement in the block, and need to be
annotated with the `#[placing]` attribute. This allows us to transform it to be
constructed directly in the caller's stack frame. Getters and other methods have
no restrictions and can be freely used.

```rust
#[placing::placing]
impl Cat {
    /// Construct a new instance of `Cat` in-place
    #[placing]
    fn new(age: u8) -> Self {
        Self { age }
    }

    /// Returns the age of the cat
    fn age(&self) -> &u8 {
        &self.age
    }
}
```

Finally it's time to create an instance of our type. This is the most
scary-looking part of this crate, but once you understand what's going on it's
fairly straightforward. Earlier we defined our `Cat::new` constructor. The
placing crate has broken this up into two parts: `new_uninit` which creates the
"place". And `new_init` which instantiates the type in the place. All you have
to do is call both of these right after each other, and voila - in-place
construction!

```rust
fn main() {
    // Create the place for `cat`
    let mut cat = unsafe { Cat::new_uninit() };
    // Instantiate the fields on `cat`
    unsafe { cat.new_init(12) };
    // Type can now be used as normal
    assert_eq!(cat.age(), &12);
}
```

To define a constructor which places a type directly on the heap rather than on
the stack, just write `-> Box<_>` and `Box::new` and
`placing` will take care of the rest.

```rust
#[placing::placing]
impl Cat {
    /// Construct a new instance of `Cat` in-place on the heap
    #[placing]
    fn new(age: u8) -> Box<Self> {
        Box::new(Self { age })
    }
}
```

## The language feature

This crate uses proc macros to prototype a new language feature. Proc macros are
less powerful than what a compiler can do, as it can only provide limited
syntactic transforms and does not have access to semantic analysis. Because of
this a language feature should become a lot more streamlined. Assuming we'd have
some first-class `placing` notation, the compiler would allow us to write our
earlier example as follows:

```rust
struct Cat {
    age: u8,
}

impl Cat {
    placing fn new(age: u8) -> Self {
        Self { age }
    }

    fn age(&self) -> &u8 {
        &self.age
    }
}

fn main() {
    let cat = Cat::new(12);
    assert_eq!(cat.age(), &12);
}
```

That's just a single extra annotation on the constructor. Everything else
continues working exactly the same, which is what makes this feature so
appealing.

## Safety
This crate prototypes a new language feature and liberally makes use of `unsafe`.

## Contributing
Want to join us? Check out our ["Contributing" guide][contributing] and take a
look at some of these issues:

- [Issues labeled "good first issue"][good-first-issue]
- [Issues labeled "help wanted"][help-wanted]

[contributing]: https://github.com/yoshuawuyts/placing/blob/main/.github/CONTRIBUTING.md
[good-first-issue]: https://github.com/yoshuawuyts/placing/labels/good%20first%20issue
[help-wanted]: https://github.com/yoshuawuyts/placing/labels/help%20wanted

## See Also

- [rust-for-linux/pinned-init](https://github.com/Rust-for-Linux/pinned-init)

## References

- [The safe pinned initialization problem - Rust for Linux](https://rust-for-linux.com/the-safe-pinned-initialization-problem)
- [Rust Temporary Lifetimes and "Super Let" - Mara Bos](https://blog.m-ou.se/super-let/)
- [In-place construction seems surprisingly simple? - Yosh Wuyts](https://blog.yoshuawuyts.com/in-place-construction-seems-surprisingly-simple/)
- [Ergonomic self-referential types for Rust - Yosh Wuyts](https://blog.yoshuawuyts.com/self-referential-types/)

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
