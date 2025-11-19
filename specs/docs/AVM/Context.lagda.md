---
title: AVM Context
icon: fontawesome/solid/layer-group
tags:
  - Anoma Virtual Machine
  - execution context
  - state
  - errors
  - interpreter types
---

This module defines the execution context infrastructure for AVM program
interpretation, encompassing all components necessary for program execution and
event tracking. The module specifies:

- **State Representation:** The complete interpreter knowledge state, including
  object store, active transactions, and execution frame context
- **Error Type Taxonomy:** A hierarchical classification of potential failure
  modes, organized by instruction family
- **Result Type Algebra:** Algebraic types for communicating execution outcomes
  (success or failure) with associated state transformations
- **Observability Infrastructure:** Event tracing mechanisms supporting
  debugging, auditing, and formal verification

This architectural design establishes clear separation of concerns: persistent
storage components (the object store abstraction) are defined here, while
ephemeral execution state components (transaction overlay structures and current
instruction frame) flow through the interpreter state transformations.

```agda
{-# OPTIONS --without-K --type-in-type --guardedness --exact-split #-}
open import Background.BasicTypes
```

## System Model Assumptions

This module operates under the system model assumptions documented in
[SystemModel](SystemModel.md). Refer to that specification for comprehensive
assumptions regarding network behavior, failure mode characterization,
transaction isolation semantics, and distributed execution properties.

## Module Parameterization

This module exhibits parametric polymorphism over core types including the
object behaviour model. Platform implementers must provide concrete
instantiations of these type parameters to realize the AVM context structures
for specific deployment scenarios.

Note: While `ObjectBehaviour` is a module parameter here, it is defined
concretely in `AVM.Instruction` as `AVMProgram (List Val)` and passed through
the module chain. This establishes that runtime objects are pairs of executable
AVM programs with their metadata.

```agda
module AVM.Context
    -- Core types
    (Val : Set)                      -- Used for both Input and Output currently
    (ObjectId : Set)

    -- Machine/distribution types
    (MachineId : Set)

    -- Controller/distribution types
    (ControllerId : Set)

    -- Transaction types
    (TxId : Set)

    -- Object behaviour type (defined concretely in AVM.Instruction)
    (ObjectBehaviour : Set)
  where
```

## Message Type Abstractions

Message-passing constitutes the exclusive inter-object communication mechanism
within the AVM. The representation of messages is intentionally maintained as an
abstract type to enable diverse concrete instantiations.

The current specification employs a unified type (Agda@Val) for both Agda@Input
and Agda@Output to simplify the initial formalization.

```agda
Message : Set
Message = Val

Input : Set
Input = Message

Output : Set
Output = Message

InputSequence : Set
InputSequence = List Input
```

## Object Metadata Structure

Extrinsic metadata managed by the AVM runtime encompasses the object's unique
identifier, accumulated input history, physical machine location, and controller
authority assignments. Objects are hosted on machines (representing physical
computational nodes) and possess ownership relationships with controllers
(representing logical authorities responsible for transaction ordering). Each
object's metadata records its current machine location, the controller that
created it (immutable provenance), and the controller that currently owns it
(mutable ownership).

```agda
record ObjectMeta : Set where
  constructor mkMeta
  field
    objectId : ObjectId
    history : List Input                 -- Accumulated inputs

    -- Physical location
    machine : MachineId                  -- Where object data resides

    -- Simplified controller tracking
    creatingController : ControllerId    -- Who created (immutable)
    currentController : ControllerId     -- Who controls now
```

## Store: Persistent Object Database Abstraction

The AVM specification defines an abstract view of the persistent object store
encompassing all system objects. The store architecture maintains separation
between intrinsic object behaviors (encapsulated computation) and extrinsic
runtime metadata (lifecycle and distribution information). This abstract view is
modeled as a partial function, reflecting that object identifiers may reference
nonexistent objects or objects that are currently unavailable due to
distribution or reachability constraints.

### Agda@ObjectStore

```agda
ObjectStore : Set
ObjectStore = ObjectId → Maybe ObjectBehaviour
```

### Agda@MetaStore

```agda
MetaStore : Set
MetaStore = ObjectId → Maybe ObjectMeta
```

### Runtime Object Type Synonyms

Runtime objects in the AVM are represented as the pairing of an executable
behavior (an AVM program) with runtime metadata. These type synonyms clarify
this distinction:

- A runtime object is a behavior paired with its metadata

```agda
RuntimeObject : Set
RuntimeObject = ObjectBehaviour × ObjectMeta
```

- A runtime object with its unique identifier

```agda
RuntimeObjectWithId : Set
RuntimeObjectWithId = ObjectId × ObjectBehaviour × ObjectMeta
```

### Agda@Store Record

```agda
record Store : Set where
  constructor mkStore
  field
    objects : ObjectStore    -- Immutable object behaviours
    metadata : MetaStore     -- Mutable runtime state
```

The Agda@Store abstraction establishes a clear architectural boundary separating
the persistent object database from ephemeral execution state components. This
separation delineates state components that persist across transaction
boundaries (the store) from transient components that exist exclusively during
program execution (transaction overlay structures and the current execution
frame context).

## Transaction Execution Semantics

Transactions provide atomicity guarantees for state modifications. All state
modifications executed within a transaction context remain tentative until
explicit commitment; transaction abortion discards all tentative modifications.
The AVM runtime maintains a transaction log structure recording pending object
write operations, object creation operations, object destruction operations, and
cross-controller object transfer operations.

### Agda@TxWrite

A transaction write record pairs an object identifier with an input message
payload. The transaction log accumulates these write records in chronological
order. Multiple write operations targeting the same object within a single
transaction preserve strict ordering, thereby enabling sequential object state
evolution within atomic transaction boundaries.

```agda
TxWrite : Set
TxWrite = ObjectId × Input
```

## Pure Function Registry

The pure function registry maintains a name-indexed collection of deterministic
pure functions. Each registered function accepts a list of value arguments and
produces either a computed result value or failure indication (`nothing`) when
invoked with invalid arguments.

To support consistency in distributed environments (without assuming malicious
behavior), each function entry tracks a version number and content hash. The
version provides monotonic ordering for updates, while the hash enables quick
verification of consistency across nodes.

```agda
-- Content hash for function implementations (abstract - platform provides concrete hash)
Hash : Set
Hash = ℕ  -- Simplified: in practice would be a cryptographic hash

-- Function entry with version and content hash
record FunctionEntry : Set where
  constructor mkFunctionEntry
  field
    impl : List Val → Maybe Val  -- Function implementation
    version : ℕ                  -- Monotonic version counter
    contentHash : Hash           -- Hash of implementation for consistency checks

-- Pure function registry maps names to versioned function entries
PureFunctions : Set
PureFunctions =
  String  -- function name
  -> Maybe FunctionEntry  -- lookup result
```

## Global Execution State

The AVM interpreter maintains a global execution state representation
encompassing all information required for transactional program computation.
Each instruction execution is modeled as a state transition function: given the
current state representation and an instruction, the interpreter computes a
result value, a successor state, and an execution event trace.

```agda
record State : Set where
  field
    -- Current execution location
    machineId : MachineId          -- Physical machine executing this program

    -- Current controller identity
    controllerId : ControllerId

    -- Object storage (separated)
    store : Store          -- Combined store with objects and metadata

    -- Pure function collection (can be extended over time)
    -- Think of this as a library for programs to use.
    pureFunctions : PureFunctions

    -- Transaction context (overlay)
    txLog : List TxWrite      -- Tentative writes: (objectId , input)
    creates : List RuntimeObjectWithId               -- Pending creates
    destroys : List ObjectId                          -- Pending destroys
    observed : List ObjectId                          -- Read set (for tracking accessed objects)
    pendingTransfers : List (ObjectId × ControllerId) -- Pending object moves
    tx : Maybe TxId                                   -- Active transaction id (if any)

    -- Current execution frame
    self : ObjectId           -- Current object being processed
    input : Input             -- Current input being processed
    sender : Maybe ObjectId   -- Caller's ObjectId (nothing for top-level)
    traceMode : Bool          -- Debug tracing flag

    -- Global event counter for monotonic timestamps
    eventCounter : ℕ          -- Increments with each logged event
```

The Agda@State record organizes fields into five logical groups:

- Physical execution context: `machineId` identifies the physical machine where
  this program is currently executing. This is independent of logical controller
  identity and enables reasoning about data locality and cross-machine
  communication costs.

- Controller context: `controllerId` identifies the executing controller
  (logical authority). This field enables distributed operations across
  controller boundaries.

- Persistent storage: `store` holds the object database (both behaviors and
  metadata), and `pureFunctions` maintains the extensible pure function
  registry. These components persist across instruction executions.

- Transactional overlay: `txLog` accumulates uncommitted writes (object-input
  pairs), `creates` and `destroys` track pending object lifecycle changes,
  `observed` records the read set of accessed objects, `pendingTransfers` queues
  cross-controller object moves, and `tx` holds the active transaction
  identifier (or `nothing` if no transaction is active).

- Execution frame: `self` identifies the currently executing object, `input`
  holds the message being processed, `sender` tracks the calling object's
  identity (or `nothing` for top-level execution), and `traceMode` enables debug
  logging when set to `true`.

For a detailed explanation of how state changes through transactions, see the
[Understanding AVM State Changes](../blog/state-changes.md) guide.

## Hierarchical Error Type System

Instruction execution may terminate with failure conditions. Each instruction
family defines a dedicated error domain, and these error domains compose
hierarchically to form layered error types that mirror the instruction set
hierarchy architecture. This compositional design provides type-level static
guarantees regarding which error conditions may arise at each architectural
layer.

### Object Operation Errors

Object lifecycle and interaction operations terminate with error conditions when
target objects are absent from the store, have been destroyed, or reject input
messages.

```agda
data ObjError : Set where
  ErrObjectNotFound : ObjectId → ObjError
  ErrObjectAlreadyDestroyed : ObjectId → ObjError
  ErrObjectAlreadyExists : ObjectId → ObjError
  ErrInvalidInput : ObjectId → Input → ObjError
  ErrObjectRejectedCall : ObjectId → Input → ObjError
  ErrMetadataCorruption : ObjectId → ObjError
```

### Introspection Errors

Introspection operations fail when querying the current object's own execution
context. These errors are rare since introspection accesses information the
executing object legitimately owns.

```agda
data IntrospectError : Set where
  -- Context access errors (should be rare)
  ErrContextUnavailable : String → IntrospectError
```

### Reflection Errors

Reflection operations fail when querying other objects' metadata, traversing the
store, or accessing internal object state. These errors distinguish between
metadata access failures and systemic store inconsistencies.

```agda
data ReflectError : Set where
  -- Metadata access errors
  ErrMetadataNotFound : ObjectId → ReflectError
  ErrMetadataInconsistent : ObjectId → String → ReflectError

  -- Store traversal errors
  ErrStoreCorruption : String → ReflectError
  ErrScryPredicateFailed : String → ReflectError

  -- Reflection access errors
  ErrReflectionDenied : ObjectId → ReflectError
```

### Reification Errors

Reification operations fail when capturing execution state as data. These errors
occur when the requested state is unavailable or access is denied.

```agda
data ReifyError : Set where
  -- Context reification errors
  ErrReifyContextFailed : String → ReifyError

  -- Transaction state reification errors
  ErrNoTransactionToReify : ReifyError
  ErrTxStateAccessDenied : ReifyError

  -- Constraint store reification errors
  ErrConstraintStoreUnavailable : ReifyError
```

### Transaction Errors

Transaction operations fail due to conflicts, missing transactions, or invalid
nesting.

```agda
data TxError : Set where
  ErrTxConflict : TxId → TxError
  ErrTxNotFound : TxId → TxError
  ErrTxAlreadyCommitted : TxId → TxError
  ErrTxAlreadyAborted : TxId → TxError
  ErrNoActiveTx : TxError
  ErrInvalidDuringTx : String → TxError
```

### Machine Errors

Machine operations fail when physical nodes are unreachable or object data
cannot be transferred between machines.

```agda
data MachineError : Set where
  ErrMachineUnreachable : MachineId → MachineError
  ErrInvalidMachineTransfer : ObjectId → MachineId → MachineError
  ErrTeleportDuringTx : MachineError
```

### Controller Errors

Controller operations fail when logical authorities are unreachable or object
ownership transfers are unauthorized.

```agda
data ControllerError : Set where
  ErrControllerUnreachable : ControllerId → ControllerError
  ErrUnauthorizedTransfer : ObjectId → ControllerId → ControllerError
  ErrCrossControllerTx : ObjectId → ControllerId → ControllerError
  ErrObjectNotAvailable : ObjectId → ControllerError
  ErrObjectNotConsistent : ObjectId → ControllerError
  ErrFreezeFailed : ObjectId → ControllerError
```

### Pure Function Errors

Pure function operations fail when functions are missing, already registered, or
have version conflicts.

```agda
data PureError : Set where
  ErrFunctionNotFound : String → PureError
  ErrFunctionAlreadyRegistered : String → PureError
  ErrVersionConflict : String → ℕ → ℕ → PureError  -- name, expected version, actual version
```

### Agda@FDError

Finite domain constraint programming errors occur during constraint solving.

```agda
data FDError : Set where
  ErrNotImplemented : String → FDError
```

### Agda@NondetError

Nondeterminism instruction errors.

```agda
data NondetError : Set where
  ErrNotImplemented : String → NondetError
```

### Agda@ConstrError

Linear constraint instruction errors.

```agda
data ConstrError : Set where
  ErrNotImplemented : String → ConstrError
```

## Composed Error Types

Error types compose in layers matching the instruction set hierarchy. Each layer
includes errors from previous layers plus domain-specific errors.

### Agda@BaseError (Instr₀)

The base layer combines object, introspection, reflection, and reification
errors.

```agda
data BaseError : Set where
  obj-error : ObjError → BaseError
  introspect-error : IntrospectError → BaseError
  reflect-error : ReflectError → BaseError
  reify-error : ReifyError → BaseError
```

### Agda@TxLayerError (Instr₁)

The transaction layer adds transaction errors to base errors.

```agda
data TxLayerError : Set where
  base-error : BaseError → TxLayerError
  tx-error : TxError → TxLayerError
```

### Agda@PureLayerError (Instr₂)

The pure function layer adds pure computation errors to transactional errors.

```agda
data PureLayerError : Set where
  tx-layer-error : TxLayerError → PureLayerError
  pure-error : PureError → PureLayerError
```

### Agda@AVMError

The full error type includes all instruction errors plus machine, controller,
and experimental instruction errors.

```agda
data AVMError : Set where
  pure-layer-error : PureLayerError → AVMError
  machine-error : MachineError → AVMError
  controller-error : ControllerError → AVMError
  fd-error : FDError → AVMError
  nondet-error : NondetError → AVMError
  constr-error : ConstrError → AVMError
```

## Error Pattern Synonyms

Pattern synonyms eliminate verbose nested constructor applications when
constructing or matching errors.

### Object Error Patterns

```agda
pattern err-object-not-found oid =
  pure-layer-error (tx-layer-error (base-error (obj-error (ErrObjectNotFound oid))))

pattern err-object-destroyed oid =
  pure-layer-error (tx-layer-error (base-error (obj-error (ErrObjectAlreadyDestroyed oid))))

pattern err-object-exists oid =
  pure-layer-error (tx-layer-error (base-error (obj-error (ErrObjectAlreadyExists oid))))

pattern err-invalid-input oid inp =
  pure-layer-error (tx-layer-error (base-error (obj-error (ErrInvalidInput oid inp))))

pattern err-object-rejected oid inp =
  pure-layer-error (tx-layer-error (base-error (obj-error (ErrObjectRejectedCall oid inp))))

pattern err-metadata-corruption oid =
  pure-layer-error (tx-layer-error (base-error (obj-error (ErrMetadataCorruption oid))))
```

### Introspection Error Patterns

```agda
pattern err-context-unavailable msg =
  pure-layer-error (tx-layer-error (base-error (introspect-error (ErrContextUnavailable msg))))
```

### Reflection Error Patterns

```agda
pattern err-metadata-not-found oid =
  pure-layer-error (tx-layer-error (base-error (reflect-error (ErrMetadataNotFound oid))))

pattern err-metadata-inconsistent oid msg =
  pure-layer-error (tx-layer-error (base-error (reflect-error (ErrMetadataInconsistent oid msg))))

pattern err-store-corruption msg =
  pure-layer-error (tx-layer-error (base-error (reflect-error (ErrStoreCorruption msg))))

pattern err-scry-predicate-failed msg =
  pure-layer-error (tx-layer-error (base-error (reflect-error (ErrScryPredicateFailed msg))))

pattern err-reflection-denied oid =
  pure-layer-error (tx-layer-error (base-error (reflect-error (ErrReflectionDenied oid))))
```

### Reification Error Patterns

```agda
pattern err-reify-context-failed msg =
  pure-layer-error (tx-layer-error (base-error (reify-error (ErrReifyContextFailed msg))))

pattern err-no-transaction-to-reify =
  pure-layer-error (tx-layer-error (base-error (reify-error ErrNoTransactionToReify)))

pattern err-tx-state-access-denied =
  pure-layer-error (tx-layer-error (base-error (reify-error ErrTxStateAccessDenied)))

pattern err-constraint-store-unavailable =
  pure-layer-error (tx-layer-error (base-error (reify-error ErrConstraintStoreUnavailable)))
```

### Transaction Error Patterns

```agda
pattern err-tx-conflict tid =
  pure-layer-error (tx-layer-error (tx-error (ErrTxConflict tid)))

pattern err-tx-not-found tid =
  pure-layer-error (tx-layer-error (tx-error (ErrTxNotFound tid)))

pattern err-tx-committed tid =
  pure-layer-error (tx-layer-error (tx-error (ErrTxAlreadyCommitted tid)))

pattern err-tx-aborted tid =
  pure-layer-error (tx-layer-error (tx-error (ErrTxAlreadyAborted tid)))

pattern err-no-active-tx =
  pure-layer-error (tx-layer-error (tx-error ErrNoActiveTx))

pattern err-invalid-during-tx msg =
  pure-layer-error (tx-layer-error (tx-error (ErrInvalidDuringTx msg)))
```

### Pure Function Error Patterns

```agda
pattern err-function-not-found name =
  pure-layer-error (pure-error (ErrFunctionNotFound name))

pattern err-function-registered name =
  pure-layer-error (pure-error (ErrFunctionAlreadyRegistered name))

pattern err-version-conflict name expected actual =
  pure-layer-error (pure-error (ErrVersionConflict name expected actual))
```

### Machine Error Patterns

```agda
pattern err-machine-unreachable mid =
  machine-error (ErrMachineUnreachable mid)

pattern err-invalid-machine-transfer oid mid =
  machine-error (ErrInvalidMachineTransfer oid mid)

pattern err-teleport-during-tx =
  machine-error ErrTeleportDuringTx
```

### Controller Error Patterns

```agda
pattern err-controller-unreachable cid =
  controller-error (ErrControllerUnreachable cid)

pattern err-unauthorized-transfer oid cid =
  controller-error (ErrUnauthorizedTransfer oid cid)

pattern err-cross-controller-tx oid cid =
  controller-error (ErrCrossControllerTx oid cid)

pattern err-object-not-available oid =
  controller-error (ErrObjectNotAvailable oid)

pattern err-object-not-consistent oid =
  controller-error (ErrObjectNotConsistent oid)

pattern err-freeze-failed oid =
  controller-error (ErrFreezeFailed oid)
```

### Experimental Instruction Error Patterns

```agda
pattern err-fd-not-implemented msg =
  fd-error (ErrNotImplemented msg)

pattern err-nondet-not-implemented msg =
  nondet-error (ErrNotImplemented msg)

pattern err-constr-not-implemented msg =
  constr-error (ErrNotImplemented msg)
```

## Observability and Tracing

State transitions generate observable events. The interpreter records object
creation, destruction, message passing, transaction boundaries, and controller
operations in a chronological log. Each log entry pairs an event type with a
timestamp and executing controller identifier. This mechanism supports
debugging, auditing, and formal verification of execution histories.

### Agda@EventType

```agda
data EventType : Set where
  -- Object lifecycle events
  ObjectCreated : ObjectId → String → EventType
  ObjectDestroyed : ObjectId → EventType

  -- Object interaction events
  ObjectCalled : ObjectId → Input → Maybe Output → EventType
  MessageReceived : ObjectId → Input → EventType

  -- Machine events
  ObjectMoved : ObjectId → MachineId → MachineId → EventType
  ExecutionMoved : MachineId → MachineId → EventType
  ObjectFetched : ObjectId → MachineId → EventType

  -- Controller events (ownership changes)
  ObjectTransferred : ObjectId → ControllerId → ControllerId → EventType
  ObjectFrozen : ObjectId → ControllerId → EventType

  -- Pure function events
  FunctionUpdated : String → EventType

  -- Transaction events
  TransactionStarted : TxId → EventType
  TransactionCommitted : TxId → EventType
  TransactionAborted : TxId → EventType

  -- Error events
  ErrorOccurred : AVMError → EventType
```

### Agda@LogEntry

```agda
record LogEntry : Set where
  constructor mkLogEntry
  field
    timestamp : ℕ
    eventType : EventType
    executingController : ControllerId
```

### Agda@Trace

```agda
Trace : Set
Trace = List LogEntry
```

## Result Types

Instruction execution produces either success or failure. The Agda@Result type
encodes this dichotomy: successful executions yield a value, updated state, and
trace; failures yield only an error. This design threads state and trace
information through the interpreter while enabling early termination on errors.

### Agda@Success Record

Successful execution returns three components: the instruction's result value,
the modified execution state, and the trace of events generated. A record
structure provides named field access, eliminating tuple projections.

```agda
record Success (A : Set) : Set where
  constructor mkSuccess
  field
    value : A
    state : State
    trace : Trace

open Success public
```

### Agda@Result Datatype

A polymorphic sum type distinguishes success from failure. The type parameter
`A` specifies the result value type, while parameter `E` specifies the error
type. The `success` constructor wraps a Agda@Success record; the `failure`
constructor wraps an error value.

```agda
data Result (A : Set) (E : Set) : Set where
  success : Success A → Result A E
  failure : E → Result A E
```

### Layer-Specific Result Types

Each instruction layer has a precise result type reflecting its error domain.

```agda
BaseResult : Set → Set
BaseResult A = Result A BaseError

TxResult : Set → Set
TxResult A = Result A TxLayerError

PureResult : Set → Set
PureResult A = Result A PureLayerError

AVMResult : Set → Set
AVMResult A = Result A AVMError
```

The Agda@Result type makes explicit that:

- Success returns a record with named fields: `value`, `state`, and `trace`
- Failure returns only an error value of the specified error type
- Named fields eliminate nested tuple projections and improve code clarity
