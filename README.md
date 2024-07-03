# Collect-Me

![codecov](https://codecov.io/github/JacoMalan1/collect-me/graph/badge.svg?token=C4CKWOHKUI)
![test](https://github.com/JacoMalan1/collect-me/actions/workflows/test.yml/badge.svg)
![safety](https://github.com/JacoMalan1/collect-me/actions/workflows/safety.yml/badge.svg)
![schedule](https://github.com/JacoMalan1/collect-me/actions/workflows/scheduled.yml/badge.svg)
![check](https://github.com/JacoMalan1/collect-me/actions/workflows/check.yml/badge.svg)

# Description

This project is a library of data structures that may or may not be useful but are nonetheless not included in the Rust standard library.

# Contributing

Please file a pull request with any changes/improvements you would like to add to the project.
Please make sure you have done the following before submitting your PR:

1. Check that all the tests pass with `cargo test`.
2. Make sure that all relevant documentation has been updated. (Any function/module/item making up part of the public API **MUST** be documented.)
3. Do not file duplicate pull requests. Check that there doesn't already exist a PR for the feature you are trying to implement/bug you are trying to fix.
4. For any additional data-structures or features added, write thorough tests to ensure all critical invariants hold.
5. Every single use of the `unsafe` keyword must be accompanied by a comment documenting exactly why the code does not violate Rust's safety guarentees.
   Those comments should be styled as such:

```rust
fn some_function() {
    let some_val = 42;
    let some_ptr: *const i32 = std::mem::addr_of!(some_val);

    // SAFETY: Since val hasn't been moved or dropped `some_ptr` will
    // still be valid and is therefore safe to dereference.
    let val_copy = unsafe { *some_ptr };
}
```
