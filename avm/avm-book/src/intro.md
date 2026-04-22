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

- **Distributed**: objects can run on different machines connected via TCP.
  The `Transport` trait makes remote calls transparent — a `call` to an
  object on another node serializes the message, sends it over the network,
  and returns the response.

## Crate structure

| Crate | Purpose |
|---|---|
| `avm-core` | Types, instructions, interpreter, errors, tracing, Tape IR |
| `avm-node` | Distributed runtime: TCP transport, location directory, CLI |
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

## Distributed architecture

Multiple `avm-node` processes form a cluster. Each node owns local objects
and routes remote calls over TCP via the `Transport` trait.

<pre class="mermaid">
graph LR
    subgraph alpha [Node alpha:9001]
        PingObj[ping object]
        StoreA[(Store)]
    end
    subgraph beta [Node beta:9002]
        PongObj[pong object]
        StoreB[(Store)]
    end
    PingObj -->|call via TCP| PongObj
    PongObj -->|response via TCP| PingObj
</pre>

## Quick start

```bash
# Run all tests
cd avm && cargo test --all

# Run just the PingPong example tests
cargo test -p avm-examples ping_pong

# Distributed demo (two terminals):
# Terminal 1:
just avm demo-beta
# Terminal 2:
just avm demo-alpha

# Open the API documentation
cargo doc --open -p avm-core
```

## Related

- [Formal Agda Specification](https://anoma.github.io/avm-lab/) — the
  type-checked specification that this Rust implementation mirrors
