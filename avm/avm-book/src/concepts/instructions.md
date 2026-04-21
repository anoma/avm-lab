# Instruction Set

The AVM instruction set is organized into composable layers. Each layer adds
capabilities on top of the previous one.

## Layer hierarchy

Each layer is additive — a program that only uses Layer 0 instructions is still
a valid Layer 4 program.

<pre class="mermaid">
graph BT
    L0[Layer 0: Objects + Introspect] --> L1[Layer 1: + Transactions]
    L1 --> L2[Layer 2: + Pure Functions]
    L2 --> L3[Layer 3: + Distribution]
    L3 --> L4[Layer 4: + Constraints]
</pre>

## Layer 0 — Objects and Introspection

| Instruction | Returns | Description |
|---|---|---|
| `create_obj(name, ctrl?)` | `ObjectId` | Create object from behavior name |
| `destroy_obj(id)` | `Bool` | Mark object for destruction |
| `call(id, input)` | `Maybe Output` | Synchronous message call |
| `receive()` | `Maybe Input` | Receive next message |
| `get_self()` | `ObjectId` | Current object's ID |
| `get_input()` | `Input` | Current input message |
| `get_state()` | `List Val` | Object's internal state |
| `set_state(vals)` | `()` | Replace internal state |
| `get_sender()` | `Maybe ObjectId` | Caller's ID |
| `get_current_machine()` | `MachineId` | Physical machine |

## Layer 1 — Transactions

| Instruction | Returns | Description |
|---|---|---|
| `begin_tx(ctrl?)` | `TxId` | Start a transaction |
| `commit_tx(id)` | `Bool` | Validate and apply atomically |
| `abort_tx(id)` | `()` | Discard all pending changes |

## Layer 2 — Pure Functions

| Instruction | Returns | Description |
|---|---|---|
| `call_pure(name, args)` | `Maybe Val` | Call a named pure function |

## Layer 3 — Distribution

| Instruction | Returns | Description |
|---|---|---|
| `get_machine(id)` | `Maybe MachineId` | Object's physical location |
| `teleport(machine)` | `Bool` | Move execution |
| `move_object(id, machine)` | `Bool` | Move object data |
| `fetch(id)` | `Bool` | Bring replica locally |
| `get_controller(id)` | `Maybe ControllerId` | Object's controller |
| `transfer_object(id, ctrl)` | `Bool` | Transfer ownership |
| `freeze(id)` | `Maybe Bool` | Synchronize replicas |

## Layer 4 — Constraints (stub)

Finite-domain constraint variables and nondeterministic choice. Not yet
implemented in the interpreter.

## Safety

In the Agda specification, instructions are classified as `Safe` or `Unsafe`.
Unsafe instructions (reflection, `reifyTxState`, `registerPure`) bypass
encapsulation. The Rust implementation tracks this at the type level through
the instruction enum structure.
