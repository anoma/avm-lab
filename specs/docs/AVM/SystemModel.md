---
title: AVM System Model and Assumptions
icon: fontawesome/solid/cube
tags:
  - AVM
  - system model
  - assumptions
  - distributed systems
  - network model
---

This document spells out the assumptions the AVM specification relies on (see
[About](../About.md)). They define the system model for AVM programs: how the
network behaves, how failures are treated, how transactions are scoped, and how
distribution works.

The AVM specification defines the semantics of a distributed transactional
virtual machine. 

Such semantics depend on an execution environment, so the assumptions are made
explicit to ensure that the specification is self-contained and can be
implemented independently of the platform.

1. Network model
  1. Timing: Asynchronous with reliable delivery.
  2. Ordering: FIFO per sender-receiver pair while both sides are reachable.
  3. Reliability: Crash-stop failures only.

2. Transaction semantics
  1. Isolation: Serializable snapshot isolation scoped to a single controller.
  2. Atomicity: Commits are local and avoid distributed coordination until the end of the transaction.

3. Execution model
  1. Single-program semantics with atomic instructions. That is, each instruction is atomic over the `State`; no concurrent execution observes intermediate states.
  2. Scheduling and preemption are platform choices.

4. Reachability predicates
  1. `isReachableController` and `isReachableMachine` are provided by the platform with best-effort accuracy and eventual completeness.

5. Persistence semantics
  1. `Store` is a conceptual persistent object database.
  2. Crash recovery strategy is platform-defined.

<!-- what else? well, that's the kitchen sink and all -->

## Network Model

AVM program execution assumes the platform provides the network model described here.

### Reachability Semantics

The specification relies on two platform-provided reachability predicates:

1. `isReachableController(c)` returns `true` iff controller `c` is up (not crashed) and can exchange messages with the executing controller.
2. `isReachableMachine(m)` indicates the liveness and connectivity of machine `m`.


### Message Delivery

Transactional semantics: Messages sent via Agda@call during a transaction inherit the transaction's atomicity. On commit, all messages in scope are delivered (subject to reachability); on abort, none are delivered.

Ordering: FIFO ordering per sender-receiver pair. Messages from different senders may interleave arbitrarily.

Reliability: When both sides remain reachable, messages are delivered exactly once.

Unreachability: If the receiver becomes unreachable after send, delivery is not guaranteed; the sender may observe an operation failure.

### Network Timing Model

The specification is timing-agnostic: no bound on message delay and no requirement for synchronised clocks. It fits the following models:

1. Asynchronous networks: Unbounded (but finite) message delays.
2. Partially synchronous networks: Eventually bounded delays after an unknown stabilization period.
3. Synchronous networks: Known bounded delays (a stronger assumption).

Liveness implications: Because timing is not assumed, liveness guarantees that depend on timeouts (e.g., "commit within T seconds") are platform-dependent and out of scope.

## Failure Model

### Controller Failures

Crash-stop: Controllers may crash and halt; they do not send incorrect messages or corrupt state.

Byzantine faults: Not modelled; malicious behaviour is out of scope.

Failure detection: Provided through Agda@isReachableController. Detectors should
be eventually complete and aim for good accuracy (minimize false suspicions).

### Machine Failures

Independence: Machine failures are modeled as independent events; correlated simultaneous failures are not addressed.

Detection: Via Agda@isReachableMachine, analogous to controller detection.

Crash recovery: Machines may restart after crashes. Recovery behavior (which
state survives, how to restart) is left to the platform.

### Network Failures

Partitions: Modelled as reachability changes; partitioned components become
mutually unreachable.

Message loss: Not modelled separately; treated as unreachability.

Message duplication: Excluded by the reliable delivery assumption.
Implementations should deduplicate if the network substrate can duplicate
messages.

## Transaction Semantics

### Isolation Level

Transactions provide serializable snapshot isolation:

1. Consistency: Each transaction observes a consistent snapshot taken at
   Agda@beginTx.

2. Serializability: Observable effects match some serial execution order.

3. Isolation: Uncommitted writes remain invisible to concurrent transactions.

4. Conflict detection: At Agda@commitTx, the interpreter checks whether any
   object in the read set changed due to a concurrent commit; conflicts abort
   the transaction.


### Atomicity Granularity

Instruction atomicity: Each instruction is atomic over the `State`; no concurrent execution observes intermediate states.

Transaction atomicity: The `beginTx`...`commitTx` block is atomic over the persistent `Store`. On commit, all modifications apply; on abort, none do.

No partial commits: A transaction cannot commit some writes and abort others.

## Distribution Model

### Physical versus Logical Separation

The specification distinguishes physical execution infrastructure (machines) from logical authority domains (controllers).

#### Physical Machines (`MachineId`)

1. Definition: A physical compute node (server, VM, or container).
2. Responsibilities: Store object data (`machine` metadata), run programs, and provide local storage and compute.
3. Trust: Code on the same machine shares a trust domain; intra-machine Byzantine faults are excluded.

#### Logical Controllers (`ControllerId`)

1. Definition: A logical authority domain responsible for transaction ordering and coordination.
2. Responsibilities: Order transactions for owned objects, maintain transaction logs, and enforce authorization policies.
3. Trust: Controllers may distrust each other; adversarial deployments are supported.

### Machine-Controller Relationship

The mapping between machines and controllers is left to the platform:

1. Co-located: One controller per machine.
2. Virtualized: Multiple controllers per machine (multi-tenancy).
3. Distributed: One controller spanning multiple machines (replication).

Uniqueness: Each object has exactly one Agda@currentController at any time, recorded in metadata.

### Authority and Trust Boundaries

#### Object Lifecycle Authority


1. Creation: The controller that executes Agda@createObj becomes the object's immutable Agda@creatingController and initial Agda@currentController.

2. Transfer: Agda@transferObject updates Agda@currentController to transfer authority.

3. Destruction: Only the Agda@currentController may destroy an object, via a transaction containing Agda@destroyObj.

<!-- TODO: confirm 3. or better, who can destroy an object? -->

#### Transaction Authority Model

1. Ownership invariant: A transaction may modify only objects owned by the executing controller.

2. Cross-controller operations: Agda@call may cross controller boundaries, but cross-controller atomicity is not fully specified, and for now, it is not supported.

#### Authorization Semantics

- Implicit authorization: The executing controller has implicit authority over
  its objects; no explicit authorisation checks are required by the
  specification. Although, we provide instructions such as Agda@sender and
  Agda@input to introspect the context of the current object in case of need.

## Execution Model

### Single-Program Semantics

Scope: The specification defines operational semantics for one Agda@AVMProgram, each with its own Agda@State.

Interpreter signature (see Agda@interpret):

```text
interpretAVMProgram : ∀ {A} → AVMProgram A → State → AVMResult A
```

Functional specification: Given a program and initial state, the interpreter returns a result (success or failure) and the final state.

Scheduling: The specification does not specify scheduling for multiple programs
running on a single controller; scheduling is a platform concern.

Atomicity assumption: Instructions are uninterruptible; each finishes before the next begins.

### State Persistence Model

1. Ephemeral state: Agda@State is an abstract value; storage mechanism is unspecified.

2. Persistent store: Agda@Store (object database) is conceptually persistent across executions.

3. Crash recovery: Behavior after controller crash is unspecified. Platforms decide whether transactions abort on crash, whether execution resumes from checkpoints, and which state components are recovered (e.g., via write-ahead logs or snapshots).

## Observability and Tracing

### Event Generation

Recording: All state-modifying instructions emit Agda@LogEntry records appended to the execution Agda@Trace. Event categories include object lifecycle events, machine operations, controller operations, transaction operations, and errors.

Completeness: The trace logs all observable events in a program execution.

### Trace Semantics

Ordering: Within one program execution, events follow instruction order, forming a total order.

Causality: If instruction I₁ happens-before I₂, the event for I₁ appears before the event for I₂.

Cross-program ordering: When multiple programs run concurrently (a
platform-level concern), the specification does not define a global order across
different executions.

## Implementation Considerations

The table below maps system model assumptions to common platform implementation strategies. These are non-normative suggestions; any approach satisfying the requirements is acceptable.

| Requirement | Common Implementation Strategies |
|-------------|----------------------------------|
| Atomic instruction execution | Execution engine with run-to-completion semantics, event loops with no preemption points |
| Serializable snapshot isolation | MVCC, optimistic concurrency control with validation, write-ahead logging |
| Reachability predicates | Heartbeat protocols, failure detectors, distributed membership services |
| FIFO message delivery | TCP connections, message queues with sequence numbers, session-based protocols |
| Crash recovery | Checkpointing, write-ahead logs, event sourcing, replicated state machines |

See [Runtime.md](AVM/Runtime.md) for detailed guidance on building a conforming runtime.
