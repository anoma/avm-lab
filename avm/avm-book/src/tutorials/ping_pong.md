# PingPong Tutorial

This tutorial walks through the PingPong example — two objects exchanging
messages until a counter reaches a maximum value.

## Overview

The PingPong program:
1. Creates two objects ("ping" and "pong") inside a transaction
2. Sends an initial message to ping
3. Ping and pong call each other, incrementing a counter
4. When the counter reaches max, the exchange stops

## The orchestrator

```rust
pub fn ping_pong_program(max_count: u64) -> ITree<Instruction, Val> {
    avm_do! {
        let tx_id_val <- trigger(instruction::begin_tx(None));
        let ping_id <- trigger(instruction::create_obj("ping", None));
        let pong_id <- trigger(instruction::create_obj("pong", None));
        let _committed <- trigger(instruction::commit_tx(tx_id));
        let initial_msg = Val::list(vec![
            Val::str("Ping"), Val::Nat(0), Val::Nat(max_count), pong_id,
        ]);
        let result <- trigger(instruction::call(ping_id, initial_msg));
        ret(result)
    }
}
```

Key points:
- `begin_tx` / `commit_tx` ensures both objects are created atomically.
- The initial message carries: tag, counter, max, and the pong object's ID.

## Behavior functions

Each behavior is a function `Val -> ITree<Instruction, Val>`:

**Ping**: if counter < max, call pong with counter + 1. Otherwise return "done".

**Pong**: queries its own ID with `get_self()` and the caller with
`get_sender()`, then calls ping back with counter + 1.

## Running it

```rust
let result = run_ping_pong(3).expect("should succeed");
// result.value contains the final response
// result.trace contains all events (creates, calls, etc.)
```

## What to observe in the trace

- 2 `ObjectCreated` events (ping and pong)
- 1 `TransactionStarted` + 1 `TransactionCommitted`
- Multiple `ObjectCalled` events (the back-and-forth)
