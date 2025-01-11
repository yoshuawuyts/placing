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
    <a href="https://github.com/yoshuawuyts/placing/blob/master.github/CONTRIBUTING.md">
      Contributing
    </a>
  </h3>
</div>

## Installation
```sh
$ cargo add placing
```

## Safety
This crate prototypes a new language feature and liberally makes use of `unsafe`.

## Contributing
Want to join us? Check out our ["Contributing" guide][contributing] and take a
look at some of these issues:

- [Issues labeled "good first issue"][good-first-issue]
- [Issues labeled "help wanted"][help-wanted]

[contributing]: https://github.com/yoshuawuyts/placing/blob/master.github/CONTRIBUTING.md
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
