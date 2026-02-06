# runtime-context

A lightweight, type-safe runtime context for storing heterogenous values by type. It supports owned values and borrowed references (immutable or mutable) while keeping lookups fast via `TypeId` hashing.

## Features

- Store owned values, borrowed references, or mutable references
- Type-safe retrieval via `TypeId`
- Zero-cost lookups using a specialized `TypeId` hasher
- Works with trait objects via `better_any`

## Install

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
runtime-context = "0.1"
```

## Quick Start

```rust
use runtime_context::{Context, tid};

#[derive(Debug, Clone, PartialEq, Eq)]
struct UserId(u64);

tid!(UserId);

fn main() {
    let mut ctx = Context::new();
    ctx.insert(UserId(42));

    let id = ctx.get::<UserId>().unwrap();
    assert_eq!(id.0, 42);
}
```

## Borrowed and Mutable References

```rust
use runtime_context::{Context, tid};

#[derive(Debug)]
struct Counter(u32);

tid!(Counter);

fn main() {
    let mut counter = Counter(0);
    let mut ctx = Context::new();

    // Store a mutable reference
    ctx.insert_mut(&mut counter);

    // Mutate through the context
    if let Some(value) = ctx.get_mut::<Counter>() {
        value.0 += 1;
    }

    assert_eq!(counter.0, 1);
}
```

## Advanced: Downcasting to Trait Objects

```rust
use runtime_context::{Context, tid};

trait Greeter {
    fn greet(&self) -> &str;
}

#[derive(Debug)]
struct Hello;

impl Greeter for Hello {
    fn greet(&self) -> &str { "hello" }
}

struct GreeterWrapper<'a, T: Greeter + 'static>(&'a T);

tid! { impl<'a, T: 'static> TidAble<'a> for GreeterWrapper<'a, T> where T: Greeter }

fn main() {
    let hello = Hello;
    let mut ctx = Context::new();
    ctx.insert(GreeterWrapper(&hello));

    let data = ctx.get_data(&GreeterWrapper::<Hello>::id()).unwrap();
    let greet = data.downcast_ref::<GreeterWrapper<Hello>>().unwrap().0.greet();

    assert_eq!(greet, "hello");
}
```

## API Overview

- `Context::insert`, `Context::insert_ref`, `Context::insert_mut` — insert values
- `Context::get`, `Context::get_mut` — retrieve typed values
- `Context::get_data`, `Context::get_data_mut` — retrieve by `TypeId`
- `Context::take` — remove and return an owned value
- `Context::remove` — remove a stored `Data`

## Safety and Notes

- Borrowed values are not cloned unless explicitly requested via `Data::into_owned`.
- Mutable references stored in the context follow Rust’s usual borrow rules.
- `TypeId` keys are generated via `better_any::tid`.

## License

Licensed under the MIT OR Apache-2.0 license.
