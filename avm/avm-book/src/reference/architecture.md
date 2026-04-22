# Architecture

This page provides a deeper look at the AVM's internal design: the crate
layout, the interpreter execution model, the `BehaviorRegistry`, and how
transactions achieve snapshot isolation.

## Crate structure

| Crate | Purpose |
|---|---|
| `avm-core` | Types, instructions, interpreter, errors, tracing, Tape IR, Transport trait |
| `avm-node` | Distributed runtime: TCP transport, location directory, CLI binary |
| `avm-examples` | PingPong and Battleship demonstrations (single-process) |
| `avm-book` | This documentation (built with [mdBook](https://rust-lang.github.io/mdBook/)) |

`avm-node` depends on `avm-core` (with the `serde` feature enabled).
`avm-examples` depends on `avm-core`. `avm-book` has no code dependency.

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

## Distributed runtime (avm-node)

The `avm-node` crate provides a networked runtime where objects on different
machines communicate transparently via TCP.

<pre class="mermaid">
graph TD
    subgraph Node
        CLI[CLI: clap] --> NodeStruct[Node]
        NodeStruct --> Worker[Worker Thread]
        NodeStruct --> TCP[TCP Listener]
        Worker -->|interpret| Interpreter
        Interpreter -->|local call| Store[(Store)]
        Interpreter -->|remote call| Transport[TcpTransport]
        Transport -->|send Call| TCP
        TCP -->|receive CallResponse| Transport
    end
</pre>

Key components:

- **Transport trait** (`avm-core`): pluggable interface for remote calls.
  `LocalOnlyTransport` (default) returns `Unreachable`; `TcpTransport`
  serializes calls over TCP.
- **LocationDirectory**: `ObjectId → MachineId` map updated by
  `CreateNotify` broadcasts. Each node knows where every object lives.
- **PendingMap**: bridges the sync `Transport::remote_call()` (running on
  a worker thread) to the async TCP response via oneshot channels.
- **Node-prefixed IDs**: `FreshIdGen::with_prefix(u16)` puts a 16-bit
  node identity in bits [63:48] of each `ObjectId`, preventing collisions.
- **Wire protocol**: length-prefixed JSON over TCP. Message types: `Call`,
  `CallResponse`, `CreateNotify`, `DestroyNotify`, `Ping`, `Pong`.

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
