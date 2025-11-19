---
title: AVM Runtime (Non-Normative Roadmap)
icon: material/road
tags:
  - AVM
  - runtime
  - roadmap
  - non-normative
---

This roadmap provides non-normative guidance for building a runtime that aligns with the AVM specification. It suggests implementation strategies while remaining flexible regarding platform choices, keeping semantic guarantees intact.

## Executive Summary

1. **Spec gap**: The spec defines atomic instruction semantics for a single program but does not cover multi-program orchestration, event processing, persistence, or distributed coordination.
2. **Goal**: Provide a runtime layer that satisfies the [system model assumptions](AVM/SystemModel.md).
3. **Key features**: Support Agda@choose/Agda@require for constraint extraction, compose transactional segments, and capture message-sequence constraints.
4. **Interfaces**: A minimal execution manager with an event queue, transaction registry, and platform-provided reachability predicates.
5. **Rollout**: Start with a single-controller in-memory runtime; add durability and recovery; then add networking and multi-controller support.
6. **Status**: Non-normative guidance. Any approach is acceptable if it meets the [system model assumptions](AVM/SystemModel.md).

## Motivation and Scope

### What the Specification Covers

The specification provides the core interpreter for a single program:

```text
interpretAVMProgram : AVMProgram A → State → AVMResult A
```

It defines atomic instruction semantics and single-controller transaction boundaries.

### What This Roadmap Addresses

This document focuses on runtime-level concerns outside the specification's scope:

1. **Execution lifecycle**: Starting, suspending, resuming, and terminating program executions
2. **Event processing**: Ingesting and ordering external events
3. **Persistence**: Crash recovery and state durability mechanisms
4. **Platform integration**: Concrete implementations of reachability predicates

This roadmap keeps instruction semantics unchanged and highlights platform responsibilities needed to realize them in production.

## Requirements

A runtime implementation **must** support the following capabilities:

### R1: Constraint-Directed Choice (Required)

Expose Agda@choose and Agda@require so the runtime can extract free variables and accumulated constraints, turning traces into CSPs for solver integration and intent matching.

**Platform responsibility**: Provide CSP solver integration or SMT backend.

### R2: Transactional Composition (Required)

Allow transactional segments from multiple programs to combine into a single atomic commit, covering both parallel non-conflicting work and sequential dependencies.

**Platform responsibility**: Implement conflict detection and atomic commit protocol.

### R3: Message Sequence Tracking (Required)

Maintain a message sequence chart (MSC) per execution context using the Agda@EventType and Agda@Trace abstractions from the Agda@Context module.

**Platform responsibility**: Preserve trace ordering and causality.

### R4: Execution Context Migration (Suggested)

Support moving execution across machines while preserving controller authority and history identifiers.

**Platform responsibility**: Implement serialization and migration protocol (optional for single-machine deployments).

### R5: Object Authority Invariant (Required)

Enforce that only the Agda@currentController may mutate or destroy an object. Agda@transferObject must atomically update ownership metadata.

**Platform responsibility**: Enforce authorization checks at transaction commit time.

<!-- TODO: confirm R5 or better, who can mutate or destroy an object? -->

## Runtime Architecture

A conforming runtime consists of three layers:

### Layer 1: Instruction Executor

**Purpose**: Execute individual AVM instructions atomically.

**Required components**:

- Program interpreter implementing `interpretAVMProgram`
- State management for Agda@State and Agda@Store
- Event logging to Agda@Trace

**Suggested approach**: Event loop with run-to-completion semantics.

### Layer 2: Execution Manager

**Purpose**: Manage lifecycle of multiple concurrent program executions.

**Required components**:

- Execution context tracking
- Event queue for external events
- Transaction registry
- Conflict detection for concurrent transactions

**Suggested approach**: Actor-based model with per-execution mailboxes.

### Layer 3: Platform Integration

**Purpose**: Provide platform-specific services.

**Required components**:

- Reachability predicates (Agda@isReachableController, Agda@isReachableMachine)
- Network message delivery
- Persistence and recovery

**Suggested approach**: Pluggable adapters for different platforms (local, cloud, distributed).

### Architectural Constraints

1. **Instruction atomicity**: No concurrent execution may observe intermediate instruction states
2. **Transaction isolation**: Follow [SystemModel.md](AVM/SystemModel.md) transaction semantics
3. **Event ordering**: Deterministic relative to object identity and controller authority, preserving FIFO per sender-receiver pair
4. **Observability**: All state-modifying instructions must emit Agda@LogEntry records

## Proposed Interfaces

These interface sketches use specification types (Agda@State, Agda@Store, Agda@TxId, etc.) and defer platform implementation details.

### Execution Context

```haskell
record ExecutionContext : Set where
  field
    executionId : ExecutionId
    program     : AVMProgram Val
    state       : State
    status      : ExecutionStatus
    parent      : Maybe ExecutionId
```

where

```haskell
data ExecutionStatus : Set where
  Running : ExecutionStatus
  Suspended : ExecutionStatus
  Terminated : ExecutionStatus
```

### Runtime State

```haskell
record RuntimeState : Set where
  field
    config         : RuntimeConfig
    executions     : ExecutionId → Maybe ExecutionContext
    globalStore    : Store
    txRegistry     : List TxId       -- No sure here.
    eventQueue     : List RuntimeEvent
    machineView    : MachineId → Maybe MachineInfo
    controllerView : ControllerId → Maybe ControllerInfo
```

where

```haskell
data RuntimeConfig : Set where
  SingleController : RuntimeConfig
  MultiController : RuntimeConfig
```

### Runtime Events

```haskell
data RuntimeEvent : Set where
  ExecutionStarted : ExecutionId → RuntimeEvent
  ExecutionSuspended : ExecutionId → RuntimeEvent
  ExecutionTerminated : ExecutionId → RuntimeEvent
```

Note: The spec defines observable event types in
[Context.lagda.md](AVM/Context.lagda.md). Runtime events track execution
lifecycle, while spec events track instruction-level operations.

### Runtime Operations

Runtime implementations **must** provide the following operation categories:

#### 1. Execution Lifecycle (Required)

```haskell
startExecution : AVMProgram Val → RuntimeState → RuntimeState
suspendExecution : ExecutionId → RuntimeState → RuntimeState
resumeExecution : ExecutionId → RuntimeState → RuntimeState
terminateExecution : ExecutionId → RuntimeState → RuntimeState
```

**Atomicity requirement**: Instruction-level atomicity must hold even when
interleaving multiple executions.

#### 2. Persistence and Recovery (Required)

```haskell
saveExecutionState : ExecutionId → RuntimeState → IO ()
restoreExecutionState : ExecutionId → IO (Maybe ExecutionContext)
getExecutionResult : ExecutionId → RuntimeState → Maybe (AVMResult Val)
```

**Platform responsibility**: Choose persistence strategy (checkpointing, write-ahead log, event sourcing, etc.).

#### 3. Event Processing (Required)

```haskell
enqueueEvent : RuntimeEvent → RuntimeState → RuntimeState
processEventQueue : RuntimeState → RuntimeState
```

**Ordering requirement**: Event processing must be deterministic relative to object identity and controller authority, preserving FIFO per sender-receiver pair.

#### 4. Distributed Support (Suggested)

```haskell
transferObjectRemote : ObjectId → ControllerId → RuntimeState → RuntimeState
executeRemote : AVMProgram Val → MachineId → RuntimeState → RuntimeState
```

**Platform responsibility**: Optional for single-controller deployments.

#### 5. Reachability Predicates (Required)

```haskell
isReachableController : ControllerId → RuntimeState → Bool
isReachableMachine : MachineId → RuntimeState → Bool
```


#### 6. Solver Integration (Required for R1)

```haskell
executeSolver : ConstraintIR → IO (Maybe Solution)
```

**Platform responsibility**: Integrate CSP or SMT solver backend.

<!-- 4. Global transactions (??): `beginGlobalTx`, `commitGlobalTx`, `abortGlobalTx` -->

## Observability and Constraint Integration

### Execution Traces (Required)

Use the Agda@EventType and Agda@Trace abstractions from the Agda@Context module.
Each execution **must** maintain an MSC so linear and non-linear constraints can
be expressed during multi-party intent matching.

**Platform responsibility**: Preserve trace completeness and ordering
guarantees.

### Constraint IR (Required for R1)

The AVM provides two complementary constraint systems that runtime
implementations should support:

#### Nondeterminism Constraints (Commit-Time Validation)

Provide an intermediate form that captures:

1. Free variables from Agda@choose
2. Constraints from Agda@require

These constraints are deferred until transaction commit, enabling multi-party
intent matching and preference composition. This feeds external solvers or CSP
tooling for intent matching and synthesis. An SMT solver or CSP solver is
required.

**Use cases**: Intent matching, token swaps, multi-party coordination where
constraints accumulate and are evaluated atomically.

#### Finite Domain Constraints (Call-Time Choice)

Additionally, capture FD constraint programming constructs:

1. Variables created via Agda@newVar with finite domains
2. Relational constraints posted via Agda@post (Agda@Eq, Agda@Neq, Agda@AllDiff,
   Agda@ValEq)
3. Domain narrowing operations via Agda@narrow
4. Labeling choices via Agda@label

These constraints propagate incrementally during execution, with transaction
rollback (Agda@abortTx) providing backtracking for search.

**Use cases**: Single-agent CSP solving, N-Queens, Sudoku, scheduling, resource
allocation problems requiring systematic search with constraint propagation.

**Note**: Both constraint systems are non-redundant and serve distinct execution
models. Runtime implementations may need different solver backends or constraint
extraction mechanisms for each.

### Deterministic Event Processing (Required)

Event ordering must be deterministic relative to object identity and controller
authority, preserving FIFO per sender-receiver pair. This ensures that replays
and multi-party coordination produce consistent results.

**Platform responsibility**: Implement deterministic event queue scheduling.

## Implementation Roadmap

This section expands on the rollout strategy from the Executive Summary.

### Phase 1: Single-Controller In-Memory Runtime

Minimal viable runtime for testing and development.

**Required components**:

- Instruction executor (Layer 1)
- Basic execution manager supporting one program at a time
- In-memory Store
- Trivial reachability predicates (always return true)

**Limitations**: No persistence, no crash recovery, no concurrency.

### Phase 2: Add Durability and Concurrency

Production-ready single-controller runtime.

**Required additions**:

- Persistence layer (implement saveExecutionState/restoreExecutionState)
- Concurrent execution support with conflict detection
- Transaction registry with isolation guarantees
- Event queue with deterministic processing

**Platform choices**:

- Persistence: Write-ahead log, snapshots, or event sourcing
- Concurrency: MVCC, optimistic locking, or pessimistic locking

### Phase 3: Add Networking

Multi-machine support with remote object access.

**Required additions**:

- Network message delivery (FIFO, reliable)
- Remote object references
- Distributed reachability predicates (implement failure detectors)

**Platform choices**:

- Messaging: TCP, message queues, or gRPC
- Failure detection: Heartbeat protocol or distributed membership service

### Phase 4: Multi-Controller Support

Full distributed runtime with cross-controller operations.

**Required additions**:

- Controller-to-controller communication
- Object transfer protocol (implement transferObjectRemote)
- Distributed transaction coordination (if supporting cross-controller atomicity)

**Platform choices**:

- Coordination: Two-phase commit, Paxos, or Raft
- Trust model: Trusted vs. adversarial controllers


### Phase 5: Constraint Solver Integration

Enable intent matching and constraint-directed choice.

**Required additions**:

- Constraint IR serialization
- Solver backend integration (executeSolver)
- Trace-to-CSP conversion

**Platform choices**:

- Solver: Z3, CVC5, or custom CSP solver
- IR format: SMTLib2 or domain-specific language
