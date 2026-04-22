# Distributed PingPong

This tutorial runs the PingPong example across two terminals connected
via TCP. The `ping` object lives on node `alpha` and the `pong` object
lives on node `beta`. Calls cross the network transparently.

## Prerequisites

Build the `avm-node` binary:

```bash
cd avm && cargo build -p avm-node
```

## Running the demo

Open two terminals:

**Terminal 1** — start the beta node (hosts pong):

```bash
just avm demo-beta
# or: RUST_LOG=info cargo run -p avm-node -- \
#       --name beta --port 9002 --peer alpha:9001 --demo
```

**Terminal 2** — start the alpha node (hosts ping, runs orchestrator):

```bash
just avm demo-alpha
# or: RUST_LOG=info cargo run -p avm-node -- \
#       --name alpha --port 9001 --peer beta:9002 --demo --rounds 3
```

## What happens

<pre class="mermaid">
sequenceDiagram
    participant A as alpha (ping)
    participant N as Network (TCP)
    participant B as beta (pong)
    A->>A: create ping object
    B->>B: create pong object
    B->>N: CreateNotify(pong)
    N->>A: CreateNotify(pong)
    A->>A: orchestrator calls ping
    A->>N: Call(pong, Ping msg)
    N->>B: Call(pong, Ping msg)
    B->>B: run pong behavior
    B->>N: CallResponse(Pong msg)
    N->>A: CallResponse(Pong msg)
    A->>A: ping behavior resumes
    A->>N: Call(pong, Ping msg)
    Note over A,B: ...repeats until max rounds...
    A->>A: ping returns "done"
</pre>

1. Beta creates the `pong` object locally and broadcasts a `CreateNotify`
   message to all peers.
2. Alpha receives the notification and updates its location directory:
   `pong → beta`.
3. Alpha creates `ping` locally and runs the orchestrator program.
4. When `ping` calls `pong`, `execute_call` sees that `pong` lives on
   `beta` (different machine) and routes through the `TcpTransport`.
5. The call is serialized as JSON, sent over TCP, executed on beta,
   and the response returns the same way.
6. When `pong` calls `ping` back, the same process happens in reverse.

## Wire protocol

Messages are length-prefixed JSON over TCP:

```
[4-byte big-endian length][UTF-8 JSON payload]
```

Message types:

| Message | Direction | Purpose |
|---|---|---|
| `Call` | requester → owner | Synchronous object invocation |
| `CallResponse` | owner → requester | Return value or error |
| `CreateNotify` | creator → all | Update location directories |
| `DestroyNotify` | destroyer → all | Remove from directories |
| `Ping`/`Pong` | either | Keepalive |

## Key concepts

- **Location directory**: each node maintains a map from `ObjectId` to
  `MachineId`. Updated by `CreateNotify` broadcasts.
- **Transport trait**: the `TcpTransport` implements `avm_core::Transport`,
  bridging the sync interpreter to the async TCP layer via oneshot channels.
- **Node-prefixed IDs**: `FreshIdGen::with_prefix(u16)` ensures globally
  unique `ObjectId`s across nodes (top 16 bits = node identity).
- **Behaviors are local**: each node registers its own behaviors. Only
  messages (values) cross the network, not code.
