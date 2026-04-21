# Architecture

This page provides a deeper look at the AVM's internal design: the crate
layout, the interpreter execution model, the `BehaviorRegistry`, and how
transactions achieve snapshot isolation.

## Crate structure

| Crate | Purpose |
|---|---|
| `avm-core` | Types (`Val`, `ObjectId`, `TxId`), all instruction enums, the interpreter, error hierarchy, and tracing infrastructure |
| `avm-examples` | PingPong and Battleship end-to-end demonstrations; useful as integration tests and as readable usage examples |
| `avm-book` | This documentation (built with [mdBook](https://rust-lang.github.io/mdBook/)) |

The crates form a strict dependency chain: `avm-examples` depends on
`avm-core`; `avm-book` is documentation only and has no code dependency on
either.

## Interpreter execution model

The interpreter is a simple loop that drives an `ITree` to completion:

1. If the current node is `Ret(v)`, return `v`.
2. If it is `Tau(next)`, set the current node to `next` and repeat (no
   recursion, no stack growth).
3. If it is `Vis(instr, cont)`, dispatch `instr` to the matching handler,
   obtain the response `r`, then set the current node to `cont(r)` and repeat.

Dispatch is a match on the top-level instruction enum. Each arm calls a
dedicated handler function that may mutate the `State` (store, overlay buffers,
trace) and returns a `Val` to be fed back into the continuation.

This design means that even a program with millions of steps consumes only
constant stack space.

## BehaviorRegistry

The `BehaviorRegistry` maps behavior names (strings) to factory functions of
type `Fn(ObjectId) -> ITree`. When `create_obj("Foo", ...)` is executed:

1. The registry looks up the factory for `"Foo"`.
2. The factory is called with the fresh `ObjectId` to produce the initial
   `ITree` for that object.
3. The tree is stored in the object's metadata inside the `Store`.

Subsequent `call(id, input)` instructions retrieve the stored tree, advance it
by one step with `input` as the response to the pending `receive()`, and write
the new tree back.

## Transaction semantics and snapshot-restore

Transactions provide serializable snapshot isolation using an overlay approach:

- On `begin_tx`, the interpreter records the current store version (a logical
  snapshot) and initializes empty overlay buffers.
- All mutating operations during the transaction (`create_obj`, `set_state`,
  `call`, `transfer_object`, `destroy_obj`) are written to the overlay rather
  than the live store.
- On `commit_tx`, the interpreter validates the read set (checking that nothing
  observed during the transaction was concurrently destroyed), then flushes the
  overlay to the store in dependency order: creates → transfers → writes →
  states → destroys.
- On `abort_tx` or a validation failure, the overlay is discarded and the store
  is unchanged.

This means `commit_tx` is the sole point where the store can change, making it
straightforward to reason about isolation.

## Formal specification references

The Rust implementation mirrors the coinductive Agda model available at
<https://anoma.github.io/avm-lab/>. Key correspondences:

| Rust | Agda |
|---|---|
| `ITree<E, A>` | `ITree E A` (coinductive) |
| `Vis(instr, cont)` | `vis e k` |
| `Ret(a)` | `ret a` |
| `Tau(next)` | `tau t` |
| `interpret` | `interp` |
| `BehaviorRegistry` | behavior map in the formal store |
| `AVMError` | the error monad layer in the Agda interpreter |

Divergences are intentional and are noted in the module-level documentation of
`avm-core`.
