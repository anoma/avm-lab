# The Anoma Virtual Machine

The AVM is an object-centric, message-passing, transactional virtual machine
designed for distributed computation. Objects communicate exclusively through
messages, maintain internal state, and execute within serializable transactions.

This Rust implementation faithfully mirrors the
[formal Agda specification](https://anoma.github.io/avm-lab/) and provides a
production-quality interpreter with full observability.

## Key properties

- **Object-centric**: computation is organized around objects with behaviors, not
  global instruction streams.
- **Message-passing**: objects communicate by sending and receiving values.
  There is no shared mutable state.
- **Transactional**: state changes can be grouped into atomic transactions with
  snapshot isolation. Conflicts are detected at commit time.
- **Interaction trees**: programs are represented as trees of observable effects,
  enabling formal reasoning about execution.
- **Layered ISA**: the instruction set is composed from independent layers
  (objects, transactions, pure functions, distribution, constraints), each
  adding capabilities on top of the previous.

## Crate structure

| Crate | Purpose |
|---|---|
| `avm-core` | Types, instructions, interpreter, errors, tracing |
| `avm-examples` | PingPong and Battleship demonstrations |
| `avm-book` | This documentation |

## Architecture

The AVM follows a layered design. A `Program` (an `ITree`) is handed to the
`Interpreter`, which dispatches each `Vis` node to the appropriate handler.
Handlers read and write through a central `Store`; object behaviors are
resolved via a `BehaviorRegistry` that can manufacture new `ITree` programs on
demand.

<pre class="mermaid">
graph TD
    Program[ITree Program] -->|interpret| Interpreter
    Interpreter -->|dispatch| ObjHandler[Object Handler]
    Interpreter -->|dispatch| TxHandler[Transaction Handler]
    Interpreter -->|dispatch| IntrospectHandler[Introspect Handler]
    Interpreter -->|dispatch| OtherHandlers[Other Handlers...]
    ObjHandler -->|read/write| Store[(Store)]
    TxHandler -->|snapshot/restore| Store
    ObjHandler -->|resolve| Registry[BehaviorRegistry]
    Registry -->|create program| Program
</pre>

## Quick start

```bash
# Run all tests
cd avm && cargo test --all

# Run just the PingPong example tests
cargo test -p avm-examples ping_pong

# Open the API documentation
cargo doc --open -p avm-core
```
