# testvalue-rs

A testvalue implementation for Rust.

## Usage

In production code:

```rust
fn production_code() {
    // ...
    // let mut var = SomeValue();
    testvalue::adjust!("adjust_this_var", |&mut var|);
    // ... do some other works
}
```

In your test code:

```rust
fn test_change_var_value() {
    let _raii = testvalue::ScopedCallback::new("adjust_this_var", |var| {
	// or: if var.some_filed == ? {}
	*var = SomeNewValue();
    });
}
```

Run tests with `testvalue` feature:

```sh
$cargo test --features testvalue
```
