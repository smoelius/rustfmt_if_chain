# rustfmt_if_chain

A wrapper around [`rustfmt`] to format inside [`if_chain`] invocations

`rustfmt_if_chain` is not guaranteed to work on all Rust source files, but it should work on most of [Clippy]'s source files.

```
cargo install rustfmt_if_chain
```

```
Usage: rustfmt_if_chain [ARGS]

Arguments ending with `.rs` are considered source files and are
formatted. All other arguments are forwarded to `rustfmt`, with one
exception.

The one argument not forwarded to `rustfmt` is
`--preformat-failure-is-warning`. If this option is passed and `rustfmt`
fails on an unmodified source file, a warning results instead of an
error.
```

## Example

- Before

  ```rust
  fn main() -> PhilosophicalResult<()> {
      if_chain! { if let Some (it) = tree . falls_in (forest) ; if ! listeners . any (| one | one . around_to_hear (it)) ; if ! it . makes_a_sound () ; then { return Err (PhilosophicalError :: new ()) ; } }
      Ok(())
  }
  ```

- After

  ```rust
  fn main() -> PhilosophicalResult<()> {
      if_chain! {
          if let Some(it) = tree.falls_in(forest);
          if !listeners.any(|one| one.around_to_hear(it));
          if !it.makes_a_sound();
          then {
              return Err(PhilosophicalError::new());
          }
      }
      Ok(())
  }
  ```

## How it works

0. Preformat check: `rustfmt` is run on the original source file to verify that it _can_ be formatted.\*
1. The `if_chain` invocations in the original source file are rewritten according to the following rules, where `x` is an identifier that does not appear elsewhere in the file:
   - `if_chain!` -> `fn x()`
   - `if ... ;` -> `if ... { x; }`
   - `then` -> `if x`
2. `rustfmt` is run on the file resulting from step 1.
3. In the file resulting from step 2, the rewrites of step 1 are undone.

\* Step 0 is not strictly necessary, but it helps to identify failures of step 2 caused by the limitations of step 1.

## Known problems

`rustfmt_if_chain --check FILENAME` does not work correctly. A workaround is to use `rustfmt_if_chain FILENAME && git diff --exit-code`.

[clippy]: https://github.com/rust-lang/rust-clippy
[`if_chain`]: https://github.com/lambda-fairy/if_chain
[`rustfmt`]: https://github.com/rust-lang/rustfmt
