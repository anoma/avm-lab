---
title: AVM Interpreter
icon: fontawesome/solid/gears
tags:
  - AVM
  - interpreter
  - semantics
  - interaction trees
---

This module establishes the operational semantics of the AVM through formal
specification of state transformation functions for each instruction type.
The interpreter defines how instructions modify the virtual machine state and
generate observability log entries. The current specification prioritizes core
semantic definitions while designating certain aspects, such as detailed
distribution semantics, as implementation stubs subject to future refinement.

### Program-Level Interpreters

The main entry points for executing AVM programs. Start here to understand the high-level interpretation process.

| Component                | Description                                                                      |
|:-------------------------|:---------------------------------------------------------------------------------|
| Agda@interpretAVMProgram | [Interpret programs using full Instruction set](#complete-avm-program-interpreter) |

### Instruction Set Interpreters

Dispatch logic for different instruction set safety levels, routing instructions to their operational semantics.

| Component               | Description                                                                                      |
|:------------------------|:-------------------------------------------------------------------------------------------------|
| Agda@executeInstruction | [Execute full instruction set (all layers)](#complete-instruction-set-interpreter-instruction)   |
| Agda@executeInstr₂      | [Execute with pure computation support](#extended-instruction-set-interpreter-instr₂)            |
| Agda@executeInstr₁      | [Execute with transaction support](#transactional-instruction-set-interpreter-instr₁)            |
| Agda@executeInstr₀      | [Execute basic instruction set (object + introspect)](#basic-instruction-set-interpreter-instr₀) |

### Instruction Interpreters

Operational semantics for each instruction family, defining how instructions modify VM state.

| Component                  | Description                                                                                  |
|:---------------------------|:---------------------------------------------------------------------------------------------|
| Agda@executeObj            | [Interpret object lifecycle instructions](#object-lifecycle-operations)                      |
| Agda@executeTx             | [Interpret transaction control instructions](#transactional-semantics-operations)            |
| Agda@executeController     | [Interpret controller authority instructions](#logical-authority-and-ownership-operations)   |
| Agda@executeMachine        | [Interpret machine distribution instructions](#physical-distribution-operations)             |
| Agda@executeIntrospect     | [Interpret introspection instructions](#execution-context-introspection-operations)          |
| Agda@executeReflect        | [Interpret reflection instructions](#reflection-operations)                                  |
| Agda@executeReify          | [Interpret reification instructions](#reification-operations)                                |
| Agda@executePure           | [Interpret pure function instructions](#pure-computation-operations)                         |

### Supporting Operations

Helper functions and utilities organized by functional area.

#### Object and State Management

| Component                 | Description                                                                                           |
|:--------------------------|:------------------------------------------------------------------------------------------------------|
| Agda@handleCall           | [Execute object behavior and manage state](#object-invocation-semantics)                             |
| Agda@lookupObjectWithMeta | [Retrieve object with metadata](#object-and-metadata-retrieval-operations)                           |
| Agda@lookupPendingCreate  | [Query pending object creations](#object-and-metadata-retrieval-operations)                          |
| Agda@updateMeta           | [Replace metadata for specific object](#state-transformation-operations)                             |
| Agda@createWithMeta       | [Install object and metadata in store](#state-transformation-operations)                             |
| Agda@initMeta             | [Create initial metadata for new object](#metadata-initialization-and-observability-log-construction) |

#### Transaction Management

| Component                  | Description                                                                      |
|:---------------------------|:---------------------------------------------------------------------------------|
| Agda@addPendingCreate      | [Stage object creation for commit](#transactional-overlay-management)           |
| Agda@removePendingCreate   | [Cancel staged object creation](#transactional-overlay-management)              |
| Agda@addPendingDestroy     | [Mark object for deletion](#transactional-overlay-management)                   |
| Agda@addPendingTransfer    | [Schedule ownership change](#transactional-overlay-management)                  |
| Agda@lookupPendingTransfer | [Query pending ownership transfers](#object-and-metadata-retrieval-operations)  |
| Agda@addPendingWrite       | [Append input to transaction log](#transaction-log-query-operations)            |
| Agda@pendingInputsFor      | [Collect pending inputs for object](#transaction-log-query-operations)          |

#### Transaction Validation and Commitment

| Component             | Description                                                                 |
|:----------------------|:----------------------------------------------------------------------------|
| Agda@ensureObserved   | [Add object to transaction read set](#read-set-tracking-and-validation)    |
| Agda@validateObserved | [Verify all observed objects exist](#read-set-tracking-and-validation)     |
| Agda@checkOwnership   | [Validate object ownership](#read-set-tracking-and-validation)             |
| Agda@applyCreates     | [Commit pending object creations](#transaction-commitment-operations)      |
| Agda@applyWrites      | [Commit pending input updates](#transaction-commitment-operations)         |
| Agda@applyDestroys    | [Commit pending deletions](#transaction-commitment-operations)             |
| Agda@applyTransfers   | [Commit pending ownership changes](#transaction-commitment-operations)     |

#### Observability

| Component         | Description                                                                                           |
|:------------------|:------------------------------------------------------------------------------------------------------|
| Agda@makeLogEntry | [Construct log entry with event data](#metadata-initialization-and-observability-log-construction)   |


## Object Behavior Model

Object behaviors are AVM programs. They can perform effects via the Agda@Instr₀
instruction set. This allows objects to call other objects, introspect their
own state, and perform pure computations, moving beyond pure functional behaviors.
Here we assume AVM programs can use language features not specified by the instruction set.
These features can include case-spliting and other control-flow structures.

Note: `interpretAVMProgram` is a module parameter of the `Interpreter` module to avoid
circular dependencies while maintaining type safety. Object behaviours (which are AVMProgram
values) use the full AVMProgram instruction set, allowing them to perform transactions,
pure computations, and distribution operations.

## Interpreter Implementation

The AVM interpreter is implemented as a two-level parameterized module structure that separates
abstract type definitions from platform-specific implementation requirements.

### Module Parameters

The outer module is parameterized by the core abstract types for values, objects, identifiers,
and fresh ID generation. These parameters define the type-level interface of the AVM but remain
abstract with respect to their concrete representation.

<!-- 
agda necessary imports

```agda
{-# OPTIONS --exact-split --without-K --guardedness #-}

open import Background.BasicTypes using (ℕ)
```
-->

<details markdown="1">
<summary>Outer Module Parameters</summary>

```agda
module AVM.Interpreter
  -- Core types
  (Val : Set)                          -- Values (data) - used for both Input and Output
  (ObjectId : Set)                     -- Object identifiers
  (freshObjectId : ℕ → ObjectId)       -- Fresh ID generation

  -- Machine/distribution types
  (MachineId : Set)                    -- Machine identifiers

  -- Controller/distribution types
  (ControllerId : Set)                 -- Controller identifiers

  -- Transaction types
  (TxId : Set)                         -- Transaction identifiers
  (freshTxId : ℕ → TxId)              -- Fresh transaction ID generation

  -- Object behaviour type
  -- In concrete instantiations, this is AVMProgram (List Val)
  (ObjectBehaviour : Set)
  where
```

</details>

```agda
open import Background.BasicTypes using (ℕ; List; Maybe)
open import Background.BasicTypes hiding (ℕ)
open import Background.InteractionTrees

open import AVM.Instruction Val ObjectId MachineId ControllerId TxId ObjectBehaviour public
```


### Platform Implementation Requirements

The nested `Interpreter` module is parameterized by platform-specific
implementation functions. These parameters define the concrete operational
requirements that any AVM implementation must satisfy:

- **Equality predicates**: Type-specific equality testing for object and
transaction identifiers (required for efficient lookup and comparison
operations)
- **State introspection**: Functions to extract all object identifiers from the global store
- **Behavior name interpretation**: Mapping from behavior names (strings) to concrete object behaviours
- **Behavior conversion**: Function to convert object behaviours to executable AVM programs
(typically identity when `ObjectBehaviour = AVMProgram (List Val)`)
- **Reachability predicates**: Platform-provided failure detection for
controllers and machines (see [SystemModel](SystemModel.md) for semantics)
- **Recursive interpretation**: Self-reference to the program interpreter
(required to avoid circular module dependencies)


```agda
module Interpreter
  -- Equality predicates: Type-specific equality for identifiers
  -- (Agda's built-in propositional equality is not decidable for arbitrary types)
  (eqObjectId : ObjectId → ObjectId → Bool)
  (eqTxId : TxId → TxId → Bool)
  (eqControllerId : ControllerId → ControllerId → Bool)

  -- State introspection: Extract all object identifiers from store
  (allObjectIds : State → List ObjectId)

  -- Behavior name interpretation: Convert behavior name to concrete object behaviour
  (interpretBehaviorName : String → ObjectBehaviour)

  -- Object behaviour to program: Convert object behaviour to executable AVM program
  -- In concrete instantiations, ObjectBehaviour IS AVMProgram (List Val), so this is typically identity
  (getBehavior : ObjectBehaviour → AVMProgram (List Val))

  -- Reachability predicates: Platform-provided failure detection
  -- Requirements (see SystemModel.md):
  -- - Must atomically check liveness (not crashed) and network connectivity
  -- - Completeness: Eventually detects all permanently failed nodes
  -- - Accuracy: Best-effort (may have false positives during transient partitions)
  (isReachableController : ControllerId → Bool)
  (isReachableMachine : MachineId → Bool)

  -- Controller ID construction: Create controller IDs from strings (for error messages)
  (mkControllerId : String → ControllerId)

  -- Program interpreter: Recursive self-reference to avoid circular dependencies
  -- This parameter enables object behaviours to be executed by recursively
  -- invoking the full AVM interpreter
  (interpretAVMProgram : ∀ {A} → AVMProgram A → State → AVMResult A)
  where
```

### Auxiliary Functions for Interpreter Implementation

#### Object and Metadata Retrieval Operations

The retrieval of a runtime object (behaviour paired with metadata) requires the
presence of both components within the store. The lookup operation shall succeed
only when both the object behaviour and its metadata are available.

```agda
  lookupObjectWithMeta : ObjectId → State → Maybe RuntimeObject
  lookupObjectWithMeta oid st
    with Store.objects (State.store st) oid
       | Store.metadata (State.store st) oid
  ...  | nothing   | nothing   = nothing
  ...  | just obj  | just meta = just (obj , meta)
  ...  | just obj  | nothing   = nothing
  ... |  nothing   | just meta = nothing
```

Pending object creations reside within the transactional overlay and must be
queried prior to consulting the global store, ensuring visibility of uncommitted
creates within the transaction scope.

```agda
  lookupPendingCreate : ObjectId → State → Maybe RuntimeObject
  lookupPendingCreate oid st =
    map-maybe (λ { (oid' , obj , meta) → obj , meta })
              (find (λ { (oid' , _ , _) → eqObjectId oid oid' }) (State.creates st))
```

Pending transfers maintain a record of ownership changes scheduled for
 commitment within the current transaction scope.

```agda
  lookupPendingTransfer : ObjectId → State → Maybe ControllerId
  lookupPendingTransfer oid st = lookup eqObjectId oid (State.pendingTransfers st)
```

#### State Transformation Operations

Metadata update operations replace the metadata entry associated with a
specified object identifier, modifying the store in-place.

```agda
  updateMeta : ObjectId → ObjectMeta → State → State
  updateMeta oid meta st =
    let store = State.store st in
    record st {
      store = record store {
        metadata = λ oid' →
          if eqObjectId oid oid'
          then
            just meta
          else
            Store.metadata store oid'
      }
    }
```

State lookup retrieves the internal state for a specified object identifier.

```agda
  lookupState : ObjectId → State → Maybe (List Val)
  lookupState oid st = Store.states (State.store st) oid
```

State update replaces the internal state for a specified object identifier.

```agda
  updateState : ObjectId → List Val → State → State
  updateState oid newState st =
    let store = State.store st in
    record st {
      store = record store {
        states = λ oid' →
          if eqObjectId oid oid'
          then just newState
          else Store.states store oid'
      }
    }
```

Object creation with metadata performs atomic installation of both the object
behaviour and its associated metadata into the global store. In a production
implementation of the interpreter, instead of having mappings for the object
store, a hash map would be used. For now, we nest if-then-else expressions to
simulate the hash map, quite inefficiently.

```agda
  createWithMeta : ObjectBehaviour → ObjectMeta → State → State
  createWithMeta obj meta st =
    let oid = ObjectMeta.objectId meta in
    let store = State.store st in
    record st {
      store = record store {
        objects = λ oid' →
          if eqObjectId oid oid'
          then just obj
          else Store.objects store oid'
        ; metadata = λ oid' →
            if eqObjectId oid oid'
            then just meta
            else Store.metadata store oid'
        ; states = λ oid' →
            if eqObjectId oid oid'
            then just []
            else Store.states store oid'
      }
    }
```

#### Metadata Initialization and Observability Log Construction

Initial metadata construction assigns a newly created object to a specified
machine and controller, establishing its initial ownership and placement state.

```agda
  initMeta : ObjectId → MachineId → Maybe ControllerId → ObjectMeta
  initMeta oid mid mcid = mkMeta oid mid mcid mcid
```

Log entry construction records observable events with a monotonically increasing
sequence number, event type classification, and the identifier of the controller
executing the operation.

```agda
  makeLogEntry : EventType → State → LogEntry
  makeLogEntry eventType st =
    mkLogEntry (State.eventCounter st) eventType (State.txController st)

  -- Increment the event counter after logging an event
  incrementEventCounter : State → State
  incrementEventCounter st =
    record st { eventCounter = suc (State.eventCounter st) }
```

#### Transactional Overlay Management

Pending input retrieval accumulates all input messages directed to a specified
object within the scope of the current transaction, enabling objects to process
batched inputs upon behavioral invocation.

```agda
  pendingInputsFor : ObjectId → State → List Input
  pendingInputsFor oid st =
    filterMap (λ { (oid' , inp) → if eqObjectId oid oid' then just inp else nothing })
              (State.txLog st)
```

The addition of a pending write operation appends an input message to the
transaction log, recording the communication for subsequent processing during
transaction commitment.

```agda
  addPendingWrite : ObjectId → Input → State → State
  addPendingWrite oid inp st =
    record st { txLog = (State.txLog st) ++ ((oid , inp) ∷ []) }
```


Pending object creations represent staged operations that are deferred until
transaction commitment, at which point they are atomically installed into the
global store.

```agda
  addPendingCreate : ObjectId → ObjectBehaviour → ObjectMeta → State → State
  addPendingCreate oid obj meta st =
    record st { creates = (State.creates st) ++ ((oid , obj , meta) ∷ []) }
```

The removal of a pending create operation cancels a previously staged object
creation, effectively rolling back the create from the transaction overlay.

```agda
  removePendingCreate : ObjectId → State → State
  removePendingCreate oid st =
    record st { creates = filter (λ { (oid' , _ , _) → not (eqObjectId oid oid') }) (State.creates st) }
```

Pending destroy operations mark objects for deletion, scheduling their removal
from the global store upon transaction commitment.

```agda
  addPendingDestroy : ObjectId → State → State
  addPendingDestroy oid st =
    record st { destroys = (State.destroys st) ++ (oid ∷ []) }
```

Pending transfer operations schedule ownership modifications, deferring the
reassignment of object controllers until transaction commitment.

```agda
  addPendingTransfer : ObjectId → ControllerId → State → State
  addPendingTransfer oid cid st =
    record st { pendingTransfers = (State.pendingTransfers st) ++ ((oid , cid) ∷ []) }
```

#### TX/NoTX Branching Helpers

Transaction-aware branching helpers eliminate code duplication in instruction
handlers that exhibit different behavior when executing inside versus outside
a transaction context. These helpers capture the common pattern of computing
a log entry and result value, then selecting the appropriate state update
function based on transaction presence.

```agda
  -- Generic transaction-aware branching helper accepts a log entry paired with
  -- a result value, plus two state update functions (one for transactional
  -- execution, one for non-transactional execution). The helper examines the
  -- transaction state and applies the corresponding update function.
  withTxBranch : ∀ {A} →
    State →
    (LogEntry × A) →
    (State → State) →  -- state update when in transaction
    (State → State) →  -- state update when not in transaction
    AVMResult A
  withTxBranch st (entry , value) inTxUpdate noTxUpdate =
    let stUpdate = caseMaybe (State.tx st) (λ _ → inTxUpdate st) (noTxUpdate st)
        st' = incrementEventCounter stUpdate
    in success (mkSuccess value st' (entry ∷ []))

  -- Object creation helper specializes the generic branching helper for the
  -- createObj instruction, eliminating duplication between immediate creation
  -- (createWithMeta) and pending transactional creation (addPendingCreate).
  withCreateTxBranch : String → Maybe ControllerId → State → AVMResult ObjectId
  withCreateTxBranch behaviorName mController st =
    let oid = freshObjectId (State.eventCounter st)
        obj = interpretBehaviorName behaviorName
        effectiveController = caseMaybe mController (λ c → just c) (State.txController st)
        meta = initMeta oid (State.machineId st) effectiveController
        entry = makeLogEntry (ObjectCreated oid behaviorName) st
    in withTxBranch st
         (entry , oid)
         (λ st → addPendingCreate oid obj meta st)   -- in transaction
         (λ st → createWithMeta obj meta st)          -- not in transaction
```

#### Success Construction Helpers

Success construction helpers eliminate repetitive result construction patterns
by providing uniform abstractions for the three most common success scenarios:
traceless operations, operations with pre-computed log entries, and operations
that generate log entries from event types.

```agda
  -- Traceless success construction produces a success result with an empty
  -- trace, used for read-only operations that do not modify observable state
  -- and therefore generate no log entries.
  mkSuccessNoTrace : ∀ {A} → A → State → AVMResult A
  mkSuccessNoTrace value st = success (mkSuccess value st [])

  -- Single-entry success construction produces a success result with a
  -- pre-computed log entry, incrementing the event counter to maintain
  -- timestamp consistency.
  mkSuccessWithLog : ∀ {A} → A → State → LogEntry → AVMResult A
  mkSuccessWithLog value st entry =
    let st' = incrementEventCounter st
    in success (mkSuccess value st' (entry ∷ []))

  -- Event-based success construction produces a success result by first
  -- computing a log entry from an event type, then incrementing the event
  -- counter and constructing the success value with the generated log entry.
  mkSuccessWithEvent : ∀ {A} → A → State → EventType → AVMResult A
  mkSuccessWithEvent value st eventType =
    let entry = makeLogEntry eventType st
        st' = incrementEventCounter st
    in success (mkSuccess value st' (entry ∷ []))
```

#### Read Set Tracking and Validation

Observation status checking determines whether an object identifier is present
in the transaction's read set, which is essential for implementing serializable
snapshot isolation semantics.

```agda
  containsObserved : ObjectId → List ObjectId → Bool
  containsObserved oid = any (eqObjectId oid)
```

The ensure-observed operation conditionally adds an object to the read set if
not already present, maintaining the transaction's observability invariants.

```agda
  ensureObserved : ObjectId → ObjectMeta → State → State
  ensureObserved oid meta st
    with containsObserved oid (State.observed st)
  ...  | true = st
  ...  | false = record st { observed = (State.observed st) ++ (oid ∷ []) }
```

Create set membership checking is performed during transaction validation to
determine whether an object identifier corresponds to a pending creation.

```agda
  inCreates : ObjectId → List RuntimeObjectWithId → Bool
  inCreates oid = any (λ { (oid' , _ , _) → eqObjectId oid oid' })
```

Destroy set membership checking verifies whether an object identifier is
scheduled for deletion within the current transaction.

```agda
  inDestroys : ObjectId → List ObjectId → Bool
  inDestroys oid = any (eqObjectId oid)
```

Read set validation ensures that all observed objects remain present in the
store, detecting conflicts that would violate serializable snapshot isolation
guarantees.

```agda
  validateObserved : State → Bool
  validateObserved st = all checkObject (State.observed st)
    where
      checkObject : ObjectId → Bool
      checkObject oid =
        if inCreates oid (State.creates st)
        then true
        else caseMaybe (Store.metadata (State.store st) oid) (λ _ → true) false
```

Ownership validation verifies that an object is currently owned by the executing
controller, enforcing single-controller transaction semantics.

```agda
  -- Old checkOwnership removed - use checkOwnershipPure and resolveTxController instead
```

Controller resolution helpers enable transaction-scoped controller management with
deferred resolution. These functions separate pure validation from side-effectful
resolution.

```agda
  -- Pure ownership check: does object controller match transaction controller?
  checkOwnershipPure : ObjectId → ObjectMeta → Maybe ControllerId → Bool
  checkOwnershipPure oid meta txController =
    caseMaybe (ObjectMeta.currentController meta)
      (λ objCid → caseMaybe txController
                   (λ txCid → eqControllerId objCid txCid)
                   false)
      false

  -- Resolve transaction controller from object if deferred
  resolveTxController : ObjectId → ObjectMeta → State → AVMResult State
  resolveTxController oid meta st =
    caseMaybe (State.tx st)
      (λ txid →
        caseMaybe (State.txController st)
          (λ _ → success (mkSuccess st st []))  -- Already resolved
          -- Need to resolve
          (caseMaybe (ObjectMeta.currentController meta)
            (λ objCid →
              let st' = record st { txController = just objCid }
              in success (mkSuccess st' st' []))
            (failure (err-object-has-no-controller oid))))
      (success (mkSuccess st st []))  -- No transaction

  -- Combined: resolve then validate
  resolveAndCheckOwnership : ObjectId → ObjectMeta → State → AVMResult (State × Bool)
  resolveAndCheckOwnership oid meta st with resolveTxController oid meta st
  ... | failure err = failure err
  ... | success res with Success.value res
  ...   | st' with checkOwnershipPure oid meta (State.txController st')
  ...     | true = success (mkSuccess (st' , true) st' [])
  ...     | false =
      let cid = caseMaybe (ObjectMeta.currentController meta) (λ c → c) (mkControllerId "unknown")
      in failure (err-cross-controller-tx oid cid)
```

#### Transaction Commitment Operations

Transaction commit atomically applies all pending changes from the transaction
overlay to the persistent Store (this goes to the assigned controller). With
this commit protocol, we aim to ensure ACID properties: atomicity
(all-or-nothing), consistency (valid state transitions backed by an controller),
isolation (uncommitted changes invisible to other controllers), and durability 
(committed changes persist).

**Commit sequence:** Changes apply in dependency order to ensure referential integrity:

1. Agda@applyCreates: Install pending objects (must exist before writes/transfers)
2. Agda@applyTransfers: Update ownership (establish authority before operations)
3. Agda@applyWrites: Record input messages (for transaction log)
4. Agda@applyStates: Apply pending state updates
5. Agda@applyDestroys: Remove objects (cleanup after all operations)

This ordering prevents dangling references and ensures all operations target
valid objects.

The application of pending creates performs atomic installation of all staged
object creations into the global store, materializing the transactional overlay.

```agda
  applyCreates : List RuntimeObjectWithId → State → State
  applyCreates [] st = st
  applyCreates ((oid , obj , meta) ∷ rest) st =
    applyCreates rest (createWithMeta obj meta st)
```

Write application records that an input was sent to an object. State updates
are now managed explicitly via the `setState` instruction rather than through
implicit history accumulation.

```agda
  applyWrite : ObjectId → Input → State → State
  applyWrite oid inp st = st  -- State updates happen via setState instruction
```

Batch write application commits all pending input messages from the transaction
log.

```agda
  applyWrites : List (ObjectId × Input) → State → State
  applyWrites [] st = st
  applyWrites ((oid , inp) ∷ rest) st =
    applyWrites rest (applyWrite oid inp st)
```

Object destruction performs atomic removal of both the object and its associated
metadata from the global store, ensuring complete cleanup.

```agda
  applyDestroy : ObjectId → State → State
  applyDestroy oid st =
    let store = State.store st in
    record st {
      store = record store {
        objects = λ oid' → 
          if eqObjectId oid oid' 
          then nothing
          else Store.objects store oid'
        ; metadata = λ oid' → 
          if eqObjectId oid oid' 
          then nothing
          else Store.metadata store oid'
      }
    }
```

Batch destroy application processes all pending object deletions, sequentially
removing objects from the global store.

```agda
  applyDestroys : List ObjectId → State → State
  applyDestroys [] st = st
  applyDestroys (oid ∷ rest) st = applyDestroys rest (applyDestroy oid st)
```

Applying a transfer updates the current controller field in object metadata.

```agda
  applyTransfer : (ObjectId × ControllerId) → State → State
  applyTransfer (oid , targetCid) st with lookupObjectWithMeta oid st
  ... | nothing = st
  ... | just (obj , meta) = updateMeta oid (record meta { currentController = just targetCid }) st
```

Batch transfer application commits all pending ownership modifications,
sequentially updating object controller assignments.

```agda
  applyTransfers : List (ObjectId × ControllerId) → State → State
  applyTransfers [] st = st
  applyTransfers (x ∷ xs) st = applyTransfers xs (applyTransfer x st)
```

State application commits all pending state updates from the transaction.

```agda
  applyStates : List (ObjectId × List Val) → State → State
  applyStates [] st = st
  applyStates ((oid , newState) ∷ rest) st =
    applyStates rest (updateState oid newState st)
```

#### Object Invocation Semantics

Object invocation executes the object's behavioral program within an isolated
execution context, processing the current input message and producing output
values according to the object's implementation.

```agda
  {-# TERMINATING #-}
  mutual
    handleCall : ObjectId → Input → State → ObjectBehaviour → ObjectMeta → AVMResult (Maybe Val)
    handleCall oid inp st obj meta =
      let -- Create isolated execution context for the object
          objectContext = record st {
              self = oid
              ; input = inp
              ; sender = just (State.self st)
              ; machineId = ObjectMeta.machine meta
            }
      in handleCall-aux (interpretAVMProgram (getBehavior obj) objectContext) (State.tx st)
      where
        handleCall-aux : AVMResult (List Val) → Maybe TxId → AVMResult (Maybe Val)
        handleCall-aux (failure err) _ = failure err
        handleCall-aux (success res) (just _) =
          -- In transaction: add pending write
          let outputs = Success.value res
              output = if null outputs then nothing else just (head outputs)
              stAfterBehavior = Success.state res
              stFinal = record stAfterBehavior {
                  -- Restore caller's context
                  self = State.self st
                  ; input = State.input st
                  ; sender = State.sender st
                }
              stWithPending = addPendingWrite oid inp stFinal
              entry = makeLogEntry (ObjectCalled oid inp output) stWithPending
              st'' = incrementEventCounter stWithPending
          in success (mkSuccess output st'' (Success.trace res ++ (entry ∷ [])))
        handleCall-aux (success res) nothing =
          -- Outside transaction: state already updated by behavior via setState
          let outputs = Success.value res
              output = if null outputs then nothing else just (head outputs)
              stAfterBehavior = Success.state res
              stFinal = record stAfterBehavior {
                  -- Restore caller's context
                  self = State.self st
                  ; input = State.input st
                  ; sender = State.sender st
                }
              entry = makeLogEntry (ObjectCalled oid inp output) stFinal
              st'' = incrementEventCounter stFinal
          in success (mkSuccess output st'' (Success.trace res ++ (entry ∷ [])))
```

### Operational Semantics: Instruction-Level Interpretation

#### Object Lifecycle Operations

Object creation generates a fresh identifier and performs atomic installation of
the object along with its metadata into the global store, establishing the
object's initial state and ownership.

```agda
    callObjInTx : ObjectId → Input → State → ObjectBehaviour → ObjectMeta → AVMResult (Maybe Output)
    callObjInTx oid inp st obj meta with resolveAndCheckOwnership oid meta st
    ... | failure err = failure err
    ... | success res with Success.value res
    ...   | (st' , false) =
      -- At this point txController should be set, use object's controller for error
      let cid = caseMaybe (ObjectMeta.currentController meta) (λ c → c) (mkControllerId "unknown")
      in failure (err-cross-controller-tx oid cid)
    ...   | (st' , true) = handleCall oid inp (ensureObserved oid meta st') obj meta

    transferObjInTx : ObjectId → ControllerId → ObjectMeta → State → AVMResult Bool
    transferObjInTx oid targetCid meta st with resolveAndCheckOwnership oid meta st
    ... | failure err = failure err
    ... | success res with Success.value res
    ...   | (stResolved , false) =
      let cid = caseMaybe (ObjectMeta.currentController meta) (λ c → c) (mkControllerId "unknown")
      in failure (err-cross-controller-tx oid cid)
    ...   | (stResolved , true) =
      let stObs = ensureObserved oid meta stResolved
          sourceCid = caseMaybe (State.txController stResolved) (λ c → c) (mkControllerId "unknown")
          st' = incrementEventCounter (addPendingTransfer oid targetCid stObs)
          entry = makeLogEntry (ObjectTransferred oid sourceCid targetCid) stObs
      in success (mkSuccess true st' (entry ∷ []))

    transferObjNoTx : ObjectId → ControllerId → ObjectMeta → State → AVMResult Bool
    transferObjNoTx oid targetCid meta st with ObjectMeta.currentController meta
    ... | nothing = failure (err-object-has-no-controller oid)
    ... | just sourceCid =
      let meta' = record meta { currentController = just targetCid }
          st' = incrementEventCounter (updateMeta oid meta' st)
          entry = makeLogEntry (ObjectTransferred oid sourceCid targetCid) st
      in success (mkSuccess true st' (entry ∷ []))

    destroyObjInTx : ObjectId → ObjectMeta → State → AVMResult Bool
    destroyObjInTx oid meta st with resolveAndCheckOwnership oid meta st
    ... | failure err = failure err
    ... | success res with Success.value res
    ...   | (stResolved , false) =
      let cid = caseMaybe (ObjectMeta.currentController meta) (λ c → c) (mkControllerId "unknown")
      in failure (err-cross-controller-tx oid cid)
    ...   | (stResolved , true) =
      let stObs = ensureObserved oid meta stResolved
          st' = incrementEventCounter (addPendingDestroy oid stObs)
          entry = makeLogEntry (ObjectDestroyed oid) stObs
      in success (mkSuccess true st' (entry ∷ []))

    destroyObjNoTx : ObjectId → ObjectMeta → State → AVMResult Bool
    destroyObjNoTx oid meta st with ObjectMeta.currentController meta
    ... | nothing = failure (err-object-has-no-controller oid)
    ... | just _ =
      let st' = incrementEventCounter (record st {
                  store = record (State.store st) {
                    objects = λ oid' → if eqObjectId oid oid' then nothing
                                      else Store.objects (State.store st) oid'
                    ; metadata = λ oid' → if eqObjectId oid oid' then nothing
                                         else Store.metadata (State.store st) oid'
                  }})
          entry = makeLogEntry (ObjectDestroyed oid) st
      in success (mkSuccess true st' (entry ∷ []))

    executeObj : ∀ {s A} → ObjInstruction s A → State → AVMResult A
    executeObj (createObj behaviorName mController) st =
      withCreateTxBranch behaviorName mController st
```

Object destruction either performs immediate removal from the store (when
executing outside a transaction) or stages the deletion within the transactional
overlay for deferred commitment.

```agda
    executeObj (destroyObj oid) st with State.tx st | lookupPendingCreate oid st | Store.objects (State.store st) oid | Store.metadata (State.store st) oid
    ... | just _ | just _ | _         | _          =
      let st' = incrementEventCounter (removePendingCreate oid st)
          entry = makeLogEntry (ObjectDestroyed oid) st
      in success (mkSuccess true st' (entry ∷ []))
    ... | just _ | nothing | just _    | just meta  = destroyObjInTx oid meta st
    ... | just _ | nothing | just _    | nothing    = failure (err-metadata-corruption oid)
    ... | just _ | nothing | nothing   | _          = failure (err-object-not-found oid)
    ... | nothing | just _ | just _    | just meta = destroyObjNoTx oid meta st
    ... | nothing | just _ | just _    | nothing    = failure (err-metadata-corruption oid)
    ... | nothing | just _ | nothing   | _          = failure (err-object-not-found oid)
    ... | nothing | nothing | just _   | just meta  = destroyObjNoTx oid meta st
    ... | nothing | nothing | just _   | nothing    = failure (err-metadata-corruption oid)
    ... | nothing | nothing | nothing  | _          = failure (err-object-not-found oid)
```

Object invocation executes the object's behavioral program with the current
input message, producing output values and potentially modifying state via the
`setState` instruction.

```agda
    executeObj (call oid inp) st with lookupPendingCreate oid st
    ... | just (obj , meta) = handleCall oid inp st obj meta
    ... | nothing with lookupObjectWithMeta oid st
    ...   | nothing = failure (err-object-not-found oid)
    ...   | just (obj , meta) with State.tx st
    ...     | just _ = callObjInTx oid inp st obj meta
    ...     | nothing = handleCall oid inp st obj meta
```

Message reception retrieves the next available message for the current object
from its message queue. This instruction enables asynchronous message handling
by allowing objects to wait for incoming messages.

```agda
    executeObj receive st =
      -- Note: Message queue implementation is platform-specific
      -- This is a placeholder that returns the current input if available
      -- Full implementation would consult a message queue maintained by the runtime
      let currentInput = State.input st
          st' = incrementEventCounter st
          entry = makeLogEntry (MessageReceived (State.self st) currentInput) st
      in success (mkSuccess (just currentInput) st' (entry ∷ []))
```

#### Execution Context Introspection Operations

The `self` instruction retrieves the current object identifier from the
execution context, enabling objects to identify themselves during behavioral
execution.

```agda
    executeIntrospect : ∀ {s A} → IntrospectInstruction s A → State → AVMResult A
    executeIntrospect self st = mkSuccessNoTrace (State.self st) st
```

The `input` instruction retrieves the current input message from the execution
context, providing access to the invocation argument.

```agda
    executeIntrospect input st = mkSuccessNoTrace (State.input st) st
```

The `getCurrentMachine` instruction retrieves the physical machine identifier
from the execution context, indicating the hardware node executing the current
computation.

```agda
    executeIntrospect getCurrentMachine st = mkSuccessNoTrace (State.machineId st) st
```

The `getState` instruction retrieves the internal state of the current object.
Returns empty list if state is not found.

```agda
    executeIntrospect getState st =
      let currentState = caseMaybe (lookupState (State.self st) st) (λ s → s) []
      in mkSuccessNoTrace currentState st
```

The `setState` instruction updates the internal state of the current object.
If within a transaction, the state update is queued for commit; otherwise it
is applied immediately.

```agda
    executeIntrospect (setState newState) st with State.tx st
    ... | nothing =
      -- Outside transaction: update immediately
      let st' = updateState (State.self st) newState st
      in mkSuccessNoTrace tt st'
    ... | just _ =
      -- Inside transaction: add to pending states
      let newPending = State.pendingStates st ++ ((State.self st , newState) ∷ [])
          st' = record st { pendingStates = newPending }
      in mkSuccessNoTrace tt st'
```

The `sender` instruction retrieves the object identifier of the invoking object,
returning `nothing` for top-level program execution contexts.

```agda
    executeIntrospect sender st =
      mkSuccessNoTrace (State.sender st) st
```

#### Reflection Operations

Reflection operations examine other objects' metadata and internal state,
breaking object encapsulation. All reflection operations are unsafe.

Reflection retrieves the metadata associated with a specified object identifier,
providing programmatic access to object ownership and lifecycle information.

```agda
    executeReflect : ∀ {s A} → ReflectInstruction s A → State → AVMResult A
    executeReflect (reflect oid) st =
      success (mkSuccess (Store.metadata (State.store st) oid) st [])
```

Metadata scrying performs global object discovery by filtering all objects
according to a predicate applied to their metadata, enabling pattern-based
object location.

```agda
    executeReflect (scryMeta pred) st =
      let results = filterMap matchMeta (allObjectIds st)
      in success (mkSuccess results st [])
      where
        matchMeta : ObjectId → Maybe (ObjectId × ObjectMeta)
        matchMeta oid with Store.metadata (State.store st) oid
        ... | just meta = if pred meta then just (oid , meta) else nothing
        ... | nothing = nothing
```

Deep scrying extends metadata scrying by applying predicates to both the object
implementation and its metadata, enabling content-based object discovery.

```agda
    executeReflect (scryDeep pred) st =
      let results = filterMap matchDeep (allObjectIds st)
      in success (mkSuccess results st [])
      where
        matchDeep : ObjectId → Maybe RuntimeObjectWithId
        matchDeep oid with Store.objects (State.store st) oid | Store.metadata (State.store st) oid
        ... | just obj | just meta = if pred obj meta then just (oid , obj , meta) else nothing
        ... | just _ | nothing = nothing
        ... | nothing | just _ = nothing
        ... | nothing | nothing = nothing
```

#### Reification Operations

Reification operations encode execution state as first-class data values that
can be stored, transmitted, or analyzed.

The `reifyContext` instruction captures the current execution frame as a
first-class value.

```agda
    executeReify : ∀ {s A} → ReifyInstruction s A → State → AVMResult A
    executeReify reifyContext st =
      let ctx = mkReifiedContext
                  (State.self st)
                  (State.input st)
                  (State.sender st)
                  (State.machineId st)
                  (caseMaybe (State.txController st) (λ c → c) (mkControllerId "ambient"))
      in success (mkSuccess ctx st [])
```

The `reifyTxState` instruction captures the current transaction's pending state
as a first-class value.

```agda
    executeReify reifyTxState st with State.tx st
    ... | nothing = success (mkSuccess nothing st [])
    ... | just txid =
      let txState = mkReifiedTxState
                      (just txid)
                      (State.txLog st)
                      (map (λ { (oid , _ , _) → oid }) (State.creates st))
                      (State.destroys st)
                      (State.observed st)
      in success (mkSuccess (just txState) st [])
```

The `reifyConstraints` instruction captures the constraint solver's internal
state. Currently returns an empty constraint store as FD instructions are not
yet implemented.

```agda
    executeReify reifyConstraints st =
      let constraints = mkReifiedConstraints [] 0
      in success (mkSuccess constraints st [])
```

#### Transactional Semantics Operations

Transaction initialization allocates a fresh transaction identifier, locks or
defers controller authority, and establishes empty transactional overlays for
staged operations, creating an isolated execution scope.

```agda
    executeTx : ∀ {s A} → TxInstruction s A → State → AVMResult A
    executeTx (beginTx mController) st with caseMaybe mController (λ cid → isReachableController cid) true
    ... | true =
      let txid = freshTxId (State.eventCounter st)
          st' = incrementEventCounter (record st { tx = just txid
                           ; txLog = []
                           ; creates = []
                           ; destroys = []
                           ; observed = []
                           ; txController = mController
                           })
          entry = makeLogEntry (TransactionStarted txid) st
      in success (mkSuccess txid st' (entry ∷ []))
    ... | false =
      -- mController must be (just cid) since validController is false only if mController is (just unreachableCid)
      let cid = caseMaybe mController (λ c → c) (mkControllerId "error:should-not-reach")
      in failure (err-controller-unreachable cid)
```

Transaction commitment performs read set validation to ensure serializable
snapshot isolation, then atomically applies all pending changes from the
transactional overlays to the global store.

```agda
    executeTx (commitTx txid) st
     with State.tx st
    ... | nothing = failure err-no-active-tx
    ... | just currentTx
        with eqTxId txid currentTx
    ...    | false = failure (err-tx-conflict txid)
    ...    | true
      with validateObserved st
    ...  | false = failure (err-tx-conflict txid)
    ...  | true =
      let stApplied = applyStates (State.pendingStates st)
                        (applyDestroys (State.destroys st)
                          (applyWrites (State.txLog st)
                            (applyTransfers (State.pendingTransfers st)
                              (applyCreates (State.creates st) st))))
          st' = incrementEventCounter (record stApplied {
                  tx = nothing
                ; txLog = []
                ; creates = []
                ; destroys = []
                ; observed = []
                ; pendingTransfers = []
                ; pendingStates = []
                ; txController = nothing
              })
          entry = makeLogEntry (TransactionCommitted txid) st
      in success (mkSuccess true st' (entry ∷ []))
```

Transaction abortion discards all pending changes from the transactional
overlays and resets the transaction state, effectively rolling back all
operations performed within the transaction scope.

```agda
    executeTx (abortTx txid) st =
      let st' = incrementEventCounter (record st {
                tx = nothing
                ; txLog = []
                ; creates = []
                ; destroys = []
                ; observed = []
                ; pendingTransfers = []
                ; pendingStates = []
                ; txController = nothing
              })
          entry = makeLogEntry (TransactionAborted txid) st
      in success (mkSuccess tt st' (entry ∷ []))
```

#### Pure Computation Operations

Pure function invocation performs name-based lookup in the runtime registry and
applies the retrieved function to the provided arguments, executing stateless
computations.

```agda
    executePure : ∀ {s A} → PureInstruction s A → State → AVMResult A
    executePure (callPure name args) st
     with State.pureFunctions st name
    ... | nothing = success (mkSuccess nothing st [])
    ... | just entry = success (mkSuccess (FunctionEntry.impl entry args) st [])
```

Pure function registration installs a named function into the runtime registry
with initial version 0, making it available for subsequent invocations
throughout the execution session. Fails if the function name is already
registered.

```agda
    executePure (registerPure name f) st
      with State.pureFunctions st name
    ... | just _ = failure (err-function-registered name)
    ... | nothing =
      -- Create initial function entry with version 0 and computed hash
      let hash = 0  -- Platform would compute actual content hash
          entry = mkFunctionEntry f 0 hash
          registry' = λ name' → if name == name' then just entry
                              else State.pureFunctions st name'
          st' = record st { pureFunctions = registry' }
      in success (mkSuccess true st' [])
```

Function definition update replaces an existing pure function's implementation
with a new function definition, incrementing the version number and updating the
content hash.

```agda
    executePure (updatePure name fn) st
      with State.pureFunctions st name
    ... | nothing = failure (err-function-not-found name)
    ... | just oldEntry =
      -- Increment version and update hash
      let newVersion = suc (FunctionEntry.version oldEntry)
          newHash = 0  -- Platform would compute actual content hash
          newEntry = mkFunctionEntry fn newVersion newHash
          registry' = λ name' → if name == name' then just newEntry
                              else State.pureFunctions st name'
          st' = incrementEventCounter (record st { pureFunctions = registry' })
          entry = makeLogEntry (FunctionUpdated name) st
      in success (mkSuccess true st' (entry ∷ []))
```

#### Physical Distribution Operations

Machine instructions manage physical resource placement and computational
migration across hardware nodes within the distributed system.

Object machine location retrieval queries the object's metadata to determine its
current physical placement within the system topology.

```agda
    executeMachine : ∀ {s A} → MachineInstruction s A → State → AVMResult A
    executeMachine (getMachine oid) st with lookupPendingCreate oid st
    ... | just (_ , meta) = success (mkSuccess (just (ObjectMeta.machine meta)) st [])
    ... | nothing with Store.metadata (State.store st) oid
    ...   | nothing    = success (mkSuccess nothing st [])
    ...   | just meta  = success (mkSuccess (just (ObjectMeta.machine meta)) st [])
```

Execution teleportation performs computational migration to a different physical
machine, subject to reachability constraints. **Safety Invariant**:
Teleportation operations shall fail when invoked within an active transaction
scope, preserving transactional atomicity guarantees.

```agda
    executeMachine (teleport mid) st with State.tx st
    ... | just _ = failure (err-invalid-during-tx "teleport")
    ... | nothing =
      if isReachableMachine mid
      then (let st' = incrementEventCounter (record st { machineId = mid })
                entry = makeLogEntry (ExecutionMoved mid (State.machineId st)) st
            in success (mkSuccess true st' (entry ∷ [])))
      else failure (err-machine-unreachable mid)
```

Object migration transfers an object's physical placement to a different machine
within the system topology, subject to target machine reachability validation.

```agda
    executeMachine (moveObject oid targetMid) st with lookupObjectWithMeta oid st
    ... | nothing = failure (err-object-not-found oid)
    ... | just (_ , meta) =
      if isReachableMachine targetMid
      then (let meta' = record meta { machine = targetMid }
                st' = incrementEventCounter (updateMeta oid meta' st)
                entry = makeLogEntry (ObjectMoved oid (ObjectMeta.machine meta) targetMid) st
            in success (mkSuccess true st' (entry ∷ [])))
      else failure (err-machine-unreachable targetMid)

    executeMachine (fetch oid) st with lookupObjectWithMeta oid st
    ... | nothing = success (mkSuccess false st [])
    ... | just (obj , meta) =
      -- Fetch brings a replica of the object to the local machine
      -- In this simplified model, we update the object's machine field to the current machine
      -- A full implementation would maintain multiple replicas
      let localMid = State.machineId st
          meta' = record meta { machine = localMid }
          st' = incrementEventCounter (updateMeta oid meta' st)
          entry = makeLogEntry (ObjectFetched oid localMid) st
      in success (mkSuccess true st' (entry ∷ []))
```

#### Logical Authority and Ownership Operations

Controller instructions manage logical authority assignments and ownership
relationships within the distributed system.

Current controller retrieval extracts the transaction controller from the
execution context. Returns the controller authority under which the current
transaction executes, or nothing if outside transaction scope.

```agda
    executeController : ∀ {s A} → ControllerInstruction s A → State → AVMResult A
    executeController getCurrentController st =
      success (mkSuccess (State.txController st) st [])
```

Object controller retrieval queries the object's current ownership, consulting
pending transfers within the transactional overlay before accessing committed
metadata in the global store.

```agda
    executeController (getController oid) st with lookupPendingCreate oid st
    ... | just (_ , meta) with resolveTxController oid meta st
    ... | failure err = failure err
    ... | success res = mkSuccessNoTrace (ObjectMeta.currentController meta) (Success.value res)
    executeController (getController oid) st | nothing with lookupPendingTransfer oid st | Store.metadata (State.store st) oid | State.tx st
    ... | just cid | _           | _        = mkSuccessNoTrace (just cid) st
    ... | nothing  | nothing     | _        = mkSuccessNoTrace nothing st
    ... | nothing  | just meta   | just _   with resolveTxController oid meta st
    ... | failure err = failure err
    ... | success res = mkSuccessNoTrace (ObjectMeta.currentController meta) (ensureObserved oid meta (Success.value res))
    executeController (getController oid) st | nothing | nothing | just meta | nothing = mkSuccessNoTrace (ObjectMeta.currentController meta) st
```

Object ownership transfer reassigns an object to a new controller, either
performing immediate reassignment (outside transactions) or staging the transfer
within the transactional overlay (during transactions). **Authorization
Requirement**: The operation must validate target controller reachability via
`isReachableController`, ensuring proper authority verification.

```agda
    executeController (transferObject oid targetCid) st
      with State.tx st | isReachableController targetCid | lookupPendingCreate oid st | lookupObjectWithMeta oid st
    ... | just _ | false | _      | _              = failure (err-controller-unreachable targetCid)
    ... | just _ | true  | just (obj , meta) | _   =
      let sourceCid = caseMaybe (ObjectMeta.currentController meta) (λ c → c) (mkControllerId "unknown")
          st' = addPendingTransfer oid targetCid st
          entry = makeLogEntry (ObjectTransferred oid sourceCid targetCid) st
      in success (mkSuccess true st' (entry ∷ []))
    ... | just _ | true  | nothing | just (_ , meta) = transferObjInTx oid targetCid meta st
    ... | just _ | true  | nothing | nothing        = failure (err-object-not-found oid)
    ... | nothing | false | _      | _              = failure (err-controller-unreachable targetCid)
    ... | nothing | true  | _      | nothing        = failure (err-object-not-found oid)
    ... | nothing | true  | _      | just (obj , meta) = transferObjNoTx oid targetCid meta st
```

Freeze operation synchronizes all replicas of an object through the controller
for strong consistency. When multiple machines have fetched the same object,
their states may have diverged. Freeze reconciles these replicas.

```agda
    executeController (freeze oid) st with caseMaybe (State.txController st) (λ cid → just cid) nothing
    ... | nothing = failure (err-object-has-no-controller oid)  -- No controller to freeze through
    ... | just freezeCid with isReachableController freezeCid
    ... | false = failure (err-controller-unreachable freezeCid)
    ... | true with lookupPendingCreate oid st | lookupObjectWithMeta oid st
    ... | just (obj , meta) | _ =
      -- Freeze pending object: synchronize through controller
      let st' = incrementEventCounter st
          entry = makeLogEntry (ObjectFrozen oid freezeCid) st
      in success (mkSuccess (just true) st' (entry ∷ []))
    ... | nothing | nothing = failure (err-object-not-found oid)
    ... | nothing | just (obj , meta) =
      -- Freeze existing object: synchronize all replicas through the controller
      let st' = incrementEventCounter st
          entry = makeLogEntry (ObjectFrozen oid freezeCid) st
      in success (mkSuccess (just true) st' (entry ∷ []))
```

### Instruction Set Dispatch Semantics

#### Basic Instruction Set Interpreter (Instr₀)

The basic instruction set interpreter provides dispatch semantics for
fundamental object lifecycle and introspection operations, constituting the
minimal AVM functionality.

```agda
    executeInstr₀ : ∀ {s A} → Instr₀ s A → State → AVMResult A
    executeInstr₀ (Obj instr) st = executeObj instr st
    executeInstr₀ (Introspect instr) st = executeIntrospect instr st
    executeInstr₀ (Reflect instr) st = executeReflect instr st
    executeInstr₀ (Reify instr) st = executeReify instr st
```

#### Transactional Instruction Set Interpreter (Instr₁)

The transactional instruction set interpreter extends basic operations with
transaction management capabilities, enabling atomic multi-operation execution
with serializable snapshot isolation semantics.

```agda
    executeInstr₁ : ∀ {s A} → Instr₁ s A → State → AVMResult A
    executeInstr₁ instr st
      with instr
    ... | Obj instr = executeObj instr st
    ... | Introspect instr = executeIntrospect instr st
    ... | Reflect instr = executeReflect instr st
    ... | Reify instr = executeReify instr st
    ... | Tx instr = executeTx instr st
```

#### Extended Instruction Set Interpreter (Instr₂)

The extended instruction set interpreter augments transactional semantics with
pure computation capabilities, supporting stateless functional operations
alongside stateful object manipulations.

```agda
    executeInstr₂ : ∀ {s A} → Instr₂ s A → State → AVMResult A
    executeInstr₂ instr st
      with instr
    ... | Obj instr = executeObj instr st
    ... | Introspect instr = executeIntrospect instr st
    ... | Reflect instr = executeReflect instr st
    ... | Reify instr = executeReify instr st
    ... | Tx instr = executeTx instr st
    ... | Pure instr = executePure instr st
```

#### Complete Instruction Set Interpreter (Instruction)

The complete instruction set interpreter integrates all instruction families,
providing comprehensive AVM functionality including object operations,
transactions, pure computations, distributed execution, and controller
management.

```agda
    executeInstruction : ∀ {s A} → Instruction s A → State → AVMResult A
    executeInstruction instr st
      with instr
    ...  | Obj instr = executeObj instr st
    ...  | Introspect instr = executeIntrospect instr st
    ...  | Reflect instr = executeReflect instr st
    ...  | Reify instr = executeReify instr st
    ...  | Tx instr = executeTx instr st
    ...  | Pure instr = executePure instr st
    ...  | Machine instr = executeMachine instr st
    ...  | Controller instr = executeController instr st
    ...  | FD instr = failure (err-fd-not-implemented "FD instructions not yet implemented")
    ...  | Nondet instr = failure (err-nondet-not-implemented "Nondeterminism instructions not yet implemented")
    ...  | LinearConstr instr = failure (err-constr-not-implemented "Linear constraint instructions not yet implemented")
```

### Program-Level Interpretation Semantics

#### Parameterized Interpreter Architecture

The interpretation logic exhibits uniform structure across all instruction set
levels. The interpreter is parameterized by both the instruction type and its
execution function, promoting modularity and eliminating code duplication across
safety levels.

```agda
  module GenericInterpreter
    (Instr : Safety → ISA)
    (execute : ∀ {s A} → Instr s A → State → AVMResult A)
    where

    {-# TERMINATING #-}
    interpretAux : ∀ {A} → ITree (Instr Safe) A → State → Trace → AVMResult A
    interpretAux prog st trace
      with observe prog
    ...  | retF x = success (mkSuccess x st trace)
    ...  | tauF prog' = interpretAux prog' st trace
    ...  | visF B instr k
         with execute instr st
    ...     | failure err = failure err
    ...     | success res =
                interpretAux
                  (k (Success.value res))
                  (Success.state res)
                  (trace ++ Success.trace res)

    interpret
      : ∀ {A} →
        ITree (Instr Safe) A →
        State →
        -----------
        AVMResult A
    interpret prog st = interpretAux prog st []
```

#### Basic Program Interpreter (Instr₀)

The basic program interpreter instantiates the parameterized interpreter for the
fundamental instruction set, processing object lifecycle and introspection
operations.

```agda
  -- Instantiate the generic interpreter for Instr₀
  open GenericInterpreter Instr₀ executeInstr₀
    renaming (interpret to interpretProgram; interpretAux to interpretProgramAux) public
```

#### Transactional Program Interpreter (Instr₁)

The transactional program interpreter instantiates the parameterized interpreter
with transaction management support, handling transaction boundaries and atomic
commitment during program execution.

```agda
  open GenericInterpreter Instr₁ executeInstr₁
    renaming (interpret to interpretProgram₁; interpretAux to interpretProgram₁Aux) public
```

#### Extended Program Interpreter (Instr₂)

The extended program interpreter instantiates the parameterized interpreter with
pure computation capabilities, processing stateless functional operations in
conjunction with transactional and object-oriented operations.

```agda
  open GenericInterpreter Instr₂ executeInstr₂
    renaming (interpret to interpretProgram₂; interpretAux to interpretProgram₂Aux) public
```

#### Complete AVM Program Interpreter

The complete AVM program interpreter instantiates the parameterized interpreter
with the full instruction set, executing comprehensive AVM programs including
distributed execution, controller management, and all operational capabilities.

```agda
  open GenericInterpreter Instruction executeInstruction
    renaming (interpret to interpretAVMProgram; interpretAux to interpretAVMProgramAux) public
```
