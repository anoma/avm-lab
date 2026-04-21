# DSL Helpers

`avm-core` ships a small set of convenience combinators that sit on top of the
raw `ITree` API. They eliminate repetitive boilerplate and encode common
protocols as typed values.

## `with_transaction`

```rust
pub fn with_transaction<F, T>(body: F) -> ITree
where
    F: FnOnce(TxId) -> ITree<T>,
```

Wraps `body` in a `begin_tx` / `commit_tx` pair. If `commit_tx` returns
`false` the outer tree still returns normally — callers that need retry logic
should inspect the returned `Bool`.

Typical usage:

```rust
let program = with_transaction(|tx| {
    avm_do! {
        let a <- trigger(instruction::create_obj("Counter", None));
        let b <- trigger(instruction::create_obj("Counter", None));
        ret(Val::pair(a, b))
    }
});
```

## `create_and_call`

```rust
pub fn create_and_call(name: &str, input: Val) -> ITree
```

Creates a fresh object from behavior `name`, immediately calls it with
`input`, and returns the call result. Equivalent to:

```rust
avm_do! {
    let id <- trigger(instruction::create_obj(name, None));
    let out <- trigger(instruction::call(id, input));
    ret(out)
}
```

Useful for fire-and-forget object creation where you only care about the
first response.

## `send`

```rust
pub fn send(target: ObjectId, msg: Val) -> ITree
```

A thin alias over `trigger(instruction::call(target, msg))` that reads more
naturally in message-passing style:

```rust
let reply = avm_do! {
    _ <- send(peer_id, Val::symbol("ping"));
    trigger(instruction::receive())
};
```

## Structured protocol types

Rather than passing raw `Val` everywhere, the examples define newtype wrappers
that convert to and from `Val` at the boundary. The convention is:

```rust
struct MyMsg { /* fields */ }

impl From<MyMsg> for Val { ... }
impl TryFrom<Val> for MyMsg { ... }
```

This keeps behavior bodies readable while preserving the uniform `Val`
interface required by the interpreter.

### Protocol helper: `expect_symbol`

```rust
pub fn expect_symbol(v: Val, name: &str) -> Result<(), AVMError>
```

Asserts that a `Val` is the symbol `name`. Returns `Err(RejectedCall)` if it
is not. Use this at the top of a behavior's receive loop to guard against
unexpected messages.

### Protocol helper: `unpack_pair`

```rust
pub fn unpack_pair(v: Val) -> Result<(Val, Val), AVMError>
```

Deconstructs a `Val::Pair`. Returns `Err(RejectedCall)` if the value is not a
pair.
