---
title: Understanding AVM State Changes and Delta Updates
date: 2025-11-27
tags:
  - AVM
  - state management
  - transactions
  - architecture
---

# Understanding AVM State Changes and Delta Updates

This guide provides a comprehensive explanation of how state changes occur in the Anoma Virtual Machine (AVM), including transaction semantics, delta update patterns, and the distinction between object-level and meta-level state modifications.

## State Architecture Overview

The AVM employs a **two-tier state architecture** separating persistent and ephemeral state:

### Persistent State: The Store

The `Store` represents the persistent object database:

```agda
record Store : Set where
  constructor mkStore
  field
    objects  : ObjectStore    -- Object behaviours (ObjectId → Maybe ObjectBehaviour)
    metadata : MetaStore       -- Runtime metadata (ObjectId → Maybe ObjectMeta)
```

**Design rationale:** Separating object behaviours (intrinsic properties - the executable AVM programs) from runtime metadata (extrinsic properties like history, location, ownership) maintains clean boundaries and enables independent reasoning about computation vs distribution.

### Ephemeral State: The State Record

The `State` record captures the current execution context, including:

- **Physical context**: Machine ID identifying the physical execution location
- **Persistent storage**: References to Store and pure function registry
- **Transaction overlay**: Pending changes (txLog, creates, destroys, observed, pendingTransfers, txController)
- **Execution frame**: Current object, input, sender
- **Event tracking**: Counter for trace generation

## State Transition Model

Every instruction execution is a **pure state transition function**:

```
execute : Instruction × State → Result (Value × State × Trace)
```

Benefits of this approach:

- **Referential transparency**: Same instruction + state → same result
- **Composability**: Instructions compose via function composition
- **Formal verifiability**: Pure functions enable rigorous reasoning

## The Transaction Overlay Pattern

The AVM doesn't immediately modify the persistent Store. Instead, changes accumulate in **transaction logs** until commit:

| Field | Purpose |
|:------|:--------|
| `txLog` | Pending writes (object-input pairs) |
| `creates` | Staged object creations |
| `destroys` | Pending object deletions |
| `observed` | Read set for conflict detection |
| `pendingTransfers` | Ownership changes |
| `tx` | Active transaction ID (or `nothing`) |
| `txController` | Transaction controller (locked at `beginTx` or resolved from first object) |

**Benefits:**

- **Atomicity**: All changes commit or abort together
- **Isolation**: Uncommitted changes invisible to other transactions
- **Conflict detection**: Read sets enable validation before commit
- **Rollback**: Simply discard the overlay

## Delta Updates: Implicit Representation

The AVM doesn't define an explicit "Delta" type. Instead, **transaction overlay lists act as implicit deltas**.

**Why no explicit delta type?**

- Simplicity: Fewer types to maintain
- Flexibility: Natural representations for different operations
- Efficiency: Direct application without delta conversion
- Composability: Lists compose via concatenation

## Transaction Commit Protocol

Commit applies changes in **dependency order** to ensure referential integrity:

1. **applyCreates**: Install pending objects (must exist before writes/transfers)
2. **applyTransfers**: Update ownership (establish authority)
3. **applyWrites**: Record input history (communication after ownership)
4. **applyDestroys**: Remove objects (cleanup after all operations)

This ordering prevents dangling references and ensures operations target valid objects.

### State Application Functions

#### applyCreates

```agda
applyCreates : List (ObjectId × Object × ObjectMeta) → State → State
applyCreates [] st = st
applyCreates ((oid , obj , meta) ∷ rest) st =
  applyCreates rest (createWithMeta obj meta st)
```

**Effect**: Atomically installs all staged objects into the Store.

**State changes:**
- `Store.objects`: Extended with new object behaviours
- `Store.metadata`: Extended with new object metadata

#### applyTransfers

```agda
applyTransfers : List (ObjectId × ControllerId) → State → State
```

**Effect**: Updates controller ownership in object metadata.

**State changes:**
- `Store.metadata.controller`: Modified for transferred objects

#### applyWrites

```agda
applyWrite : ObjectId → Input → State → State
applyWrite oid inp st with Store.metadata (State.store st) oid
... | nothing = st
... | just meta =
  let newHistory = ObjectMeta.history meta ++ (inp ∷ [])
      meta' = record meta { history = newHistory }
  in updateMeta oid meta' st
```

**Effect**: Appends inputs to object histories.

**State changes:**
- `Store.metadata.history`: Extended with new input

#### applyDestroys

```agda
applyDestroys : List ObjectId → State → State
```

**Effect**: Removes objects and metadata from Store.

**State changes:**
- `Store.objects`: Object removed (returns `nothing`)
- `Store.metadata`: Metadata removed (returns `nothing`)

## State Evolution During Operations

### Object Lifecycle

**Creation** (outside transaction):
```agda
createObj behaviorName mController st (when State.tx st = nothing) =
  -- Immediately install in Store
  let oid = fresh objectId
      obj = resolveBehavior behaviorName
      effectiveController = caseMaybe mController (λ c → just c) nothing
      meta = mkMeta oid [] effectiveController effectiveController machineId
      st' = createWithMeta obj meta st
  in success (mkSuccess oid st' trace)
```

**Creation** (inside transaction):
```agda
createObj behaviorName st (when State.tx st = just txid) =
  -- Add to creates list (deferred until commit)
  let oid = fresh objectId
      creates' = (oid, obj, meta) ∷ State.creates st
      st' = record st { creates = creates' }
  in success (mkSuccess oid st' trace)
```

**State changes:**
- Outside tx: `Store.objects` and `Store.metadata` immediately updated
- Inside tx: `State.creates` extended, Store unchanged until commit

### Transaction State Evolution

**beginTx**:
```agda
executeTx (beginTx mController) st =
  let txid = freshTxId
      st' = record st {
        tx = just txid
        ; txController = mController  -- Locked or deferred
        ; txLog = []
        ; creates = []
        ; destroys = []
        ; observed = []
        ; pendingTransfers = []
      }
  in success (mkSuccess txid st' trace)
```

**State changes:**
- `State.tx`: `nothing` → `just txid`
- `State.txController`: Set to `mController` (either locked to a controller or deferred with `nothing`)
- All overlay lists initialized to empty

**commitTx**:
```agda
executeTx (commitTx txid) st =
  -- Validate, then apply changes
  let stApplied = applyDestroys (State.destroys st)
                (applyWrites (State.txLog st)
                  (applyTransfers (State.pendingTransfers st)
                    (applyCreates (State.creates st) st)))
      st' = record stApplied { tx = nothing; txLog = []; ... }
  in success (mkSuccess true st' trace)
```

**State changes:**
- `Store`: All pending changes applied atomically
- `State.tx`: `just txid` → `nothing`
- All overlay lists cleared

**abortTx**:
```agda
executeTx (abortTx txid) st =
  let st' = record st {
        tx = nothing
        ; txLog = []
        ; creates = []
        ; destroys = []
        ; observed = []
        ; pendingTransfers = []
      }
  in success (mkSuccess unit st' trace)
```

**State changes:**
- `Store`: **Unchanged** (all pending changes discarded)
- `State.tx`: `just txid` → `nothing`
- All overlay lists cleared

## Meta-Level State Changes

### Pure Function Registry

The `PureFunctions` registry maps function names to implementations:

```agda
PureFunctions : Set
PureFunctions = String → Maybe (List Val → Maybe Val)
```

### registerPure (Unsafe)

```agda
executePure (registerPure name f) st =
  let registry' = λ name' → if name == name' then just f
                          else State.pureFunctions st name'
      st' = record st { pureFunctions = registry' }
  in success (mkSuccess true st' [])
```

**Effect**: Registers new pure function in the registry.

**State changes:**
- `State.pureFunctions`: Extended with new function binding

**Why unsafe?**

1. Global side effect: Extends computational environment system-wide
2. Non-monotonic: Can shadow existing functions
3. Capability escalation: Grants new computational powers
4. Determinism concerns: If function isn't pure, breaks guarantees

### updatePure (Safe)

```agda
executePure (updatePure name fn) st
  with State.pureFunctions st name
... | nothing = failure (err-function-not-found name)
... | just _ =
  let registry' = λ name' → if name == name' then just fn
                          else State.pureFunctions st name'
      st' = incrementEventCounter (record st { pureFunctions = registry' })
      entry = makeLogEntry (FunctionUpdated name) st
  in success (mkSuccess true st' (entry ∷ []))
```

**Effect**: Updates existing pure function implementation.

**State changes:**
- `State.pureFunctions`: Function binding replaced
- `State.eventCounter`: Incremented
- **Trace generated**: `FunctionUpdated` event

**Why safe (relatively)?**

1. Existence check: Only updates existing functions
2. Auditability: Generates trace events
3. Intentional modification: Explicitly targets known function
4. Event visibility: Changes observable in execution trace

### Transactional Semantics

Pure function registry modifications are **non-transactional**. They take effect immediately and globally, even inside transactions.

**Rationale**: Function registry is an environmental concern orthogonal to the object database.

**Implication**: If you register a function inside a transaction and then abort, the function registration **persists**.

## State Modification Summary

### Operations That Modify Store

| Operation | Component | Inside Tx | Outside Tx |
|:----------|:----------|:----------|:-----------|
| `createObj` | `objects`, `metadata` | Deferred | Immediate |
| `destroyObj` | `objects`, `metadata` | Deferred | Immediate |
| `call` | `metadata.history` | Deferred | Immediate |
| `transferObject` | `metadata.controller` | Deferred | Immediate |
| `moveObject` | `metadata.machine` | Immediate | Immediate |
| `commitTx` | All pending | Applies overlay | N/A |

### Operations That Modify PureFunctions

| Operation | Effect | Transactional |
|:----------|:-------|:--------------|
| `registerPure` | Add function | No |
| `updatePure` | Replace function | No |
| `callPure` | None (read-only) | N/A |

### Read-Only Operations

- `self`, `input`, `sender`, `getCurrentMachine`, `getCurrentController` (returns `Maybe ControllerId` - the transaction controller if in a transaction)
- `history`, `getMachine`, `getController` (returns `Maybe ControllerId` for the object's controller)
- `reflect`, `scryMeta`, `scryDeep`
- `callPure`

## State Domains: AVM vs Controller vs Machine

### AVM-Managed State

**Responsibility**: AVM instruction set and interpreter

**Components:**
- Object Store (behaviors and metadata)
- Transaction overlay
- Execution context
- Pure function registry
- Event trace

**Properties:**
- Formally specified
- Deterministic evolution
- Part of AVM semantics

### Controller-Managed State

**Responsibility**: Platform/implementation (consensus layer)

**Components:**
- Transaction ordering
- Authorization enforcement
- Reachability status
- Conflict detection
- Recovery state

**Properties:**
- Not fully specified in AVM semantics
- Platform-specific
- Must satisfy safety properties

### Machine-Managed State

**Responsibility**: Physical infrastructure

**Components:**
- Physical location
- Execution location
- Local storage
- Network connectivity

**Properties:**
- Hardware/OS concerns
- Abstracted via `MachineId`
- Affects performance, not semantics

## State Change Properties

### Atomicity Guarantees

- **Instruction-level**: Each instruction executes atomically
- **Transaction-level**: All changes commit or abort together
- **No partial commits**: Cannot selectively commit changes

### Isolation Properties

- **Serializable Snapshot Isolation**: Transactions see consistent snapshots
- **Read set validation**: Conflicts detected via `validateObserved`
- **Write set isolation**: Uncommitted writes invisible to other transactions

### Consistency Properties

- **Store consistency**: Objects and metadata kept in sync
- **Transaction consistency**: Read set validation ensures consistency
- **Apply order**: Dependencies satisfied (creates before writes, etc.)

## Examples

### Example 1: Simple Transaction

**Initial state:**
```
Store.objects = {}
State.tx = nothing
```

**Execution:**
```agda
do
  txid ← beginTx nothing  -- Defer controller resolution
  oid ← createObj "counter" (just controllerId)
  result ← call oid (Input 0)
  success ← commitTx txid
```

**State evolution:**

1. After `beginTx`: `State.tx = just txid`, overlays empty
2. After `createObj`: `State.creates = [(oid, counterObj, counterMeta)]`
3. After `call`: `State.txLog = [(oid, Input 0)]`, `State.observed = [oid]`
4. After `commitTx`:
   - `Store.objects(oid) = just counterObj`
   - `Store.metadata(oid) = just (meta with history = [Input 0])`
   - `State.tx = nothing`, overlays cleared

### Example 2: Transaction Abort

**Initial state:**
```
Store.objects(oid1) = just obj1
```

**Execution:**
```agda
do
  txid ← beginTx nothing  -- Defer controller resolution
  oid2 ← createObj "temp" (just controllerId)
  result ← call oid1 (Input 42)
  abortTx txid
```

**State evolution:**

1. During transaction:
   - `State.creates = [(oid2, tempObj, tempMeta)]`
   - `State.txLog = [(oid1, Input 42)]`
2. After `abortTx`:
   - `Store.objects(oid1) = just obj1` (unchanged)
   - `Store.objects(oid2) = nothing` (never created)
   - All overlays cleared

### Example 3: Meta-Level Function Registration

**Initial state:**
```
State.pureFunctions("add") = just addImpl
State.pureFunctions("mul") = nothing
```

**Execution:**
```agda
do
  success ← registerPure "mul" mulImpl
  result1 ← callPure "mul" [3, 4]
  success2 ← updatePure "add" newAddImpl
  result2 ← callPure "add" [1, 2]
```

**State evolution:**

1. After `registerPure`: `State.pureFunctions("mul") = just mulImpl`
2. After `callPure "mul"`: `result1 = just 12` (no state changes)
3. After `updatePure`:
   - `State.pureFunctions("add") = just newAddImpl`
   - `State.eventCounter = eventCounter + 1`
   - `Trace = [..., FunctionUpdated "add"]`
4. After `callPure "add"`: Uses new implementation

## Summary

The AVM state management architecture provides:

1. **Clear separation**: Persistent Store vs ephemeral execution State
2. **Transaction safety**: ACID properties via overlay pattern
3. **Formal semantics**: Pure state transition functions
4. **Implicit deltas**: Transaction logs act as deltas
5. **Meta-level operations**: Pure function registry modifications
6. **Observable changes**: All state modifications generate trace events
7. **Domain boundaries**: Clear separation between AVM, controller, and machine state

**Key principles:**
- Immutability: State transitions create new states (functional style)
- Atomicity: Changes are all-or-nothing
- Observability: Trace events provide complete execution history
- Verifiability: Pure functions enable formal reasoning

This architecture prioritizes formal verifiability, transaction safety, and distributed execution while maintaining clear separation of concerns.
