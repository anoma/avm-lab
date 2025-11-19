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

<figure markdown="1">

| Component                        | Description                                                                                       |
|:---------------------------------:---------------------------------------------------------------------------------------------------|
| **Helper Functions**             | -                                                                                                 |
| Agda@allObjectIds                | Extract all object IDs from store (module parameter)                                             |
| Agda@lookupObjectWithMeta        | [Retrieve object with metadata](#object-and-metadata-retrieval-operations)                       |
| Agda@lookupPendingCreate         | [Query pending object creations](#object-and-metadata-retrieval-operations)                      |
| Agda@lookupPendingTransfer       | [Query pending ownership transfers](#object-and-metadata-retrieval-operations)                   |
| Agda@updateMeta                  | [Replace metadata for specific object](#state-transformation-operations)                         |
| Agda@createWithMeta              | [Install object and metadata in store](#state-transformation-operations)                         |
| Agda@initMeta                    | [Create initial metadata for new object](#metadata-initialization-and-observability-log-construction) |
| Agda@makeLogEntry                | [Construct log entry with event data](#metadata-initialization-and-observability-log-construction) |
| Agda@pendingInputsFor            | [Collect pending inputs for object](#transaction-log-query-operations)                           |
| Agda@addPendingWrite             | [Append input to transaction log](#transaction-log-query-operations)                             |
| Agda@addPendingCreate            | [Stage object creation for commit](#transactional-overlay-management)                            |
| Agda@removePendingCreate         | [Cancel staged object creation](#transactional-overlay-management)                               |
| Agda@addPendingDestroy           | [Mark object for deletion](#transactional-overlay-management)                                    |
| Agda@addPendingTransfer          | [Schedule ownership change](#transactional-overlay-management)                                   |
| Agda@ensureObserved              | [Add object to transaction read set](#read-set-tracking-and-validation)                          |
| Agda@validateObserved            | [Verify all observed objects exist](#read-set-tracking-and-validation)                           |
| Agda@checkOwnership              | [Validate object ownership](#read-set-tracking-and-validation)                                   |
| Agda@applyCreates                | [Commit pending object creations](#transaction-commitment-operations)                            |
| Agda@applyWrites                 | [Commit pending input updates](#transaction-commitment-operations)                               |
| Agda@applyDestroys               | [Commit pending deletions](#transaction-commitment-operations)                                   |
| Agda@applyTransfers              | [Commit pending ownership changes](#transaction-commitment-operations)                           |
| Agda@handleCall                  | [Execute object behavior and manage state](#object-invocation-semantics)                         |
| **Instruction Interpreters**     | -                                                                                                 |
| Agda@executeObj                  | [Interpret object lifecycle instructions](#object-lifecycle-operations)                          |
| Agda@executeIntrospect           | [Interpret introspection instructions](#execution-context-introspection-operations)              |
| Agda@executeReflect              | [Interpret reflection instructions](#reflection-operations)                                      |
| Agda@executeReify                | [Interpret reification instructions](#reification-operations)                                    |
| Agda@executeTx                   | [Interpret transaction control instructions](#transactional-semantics-operations)                |
| Agda@executePure                 | [Interpret pure function instructions](#pure-computation-operations)                             |
| Agda@executeMachine              | [Interpret machine distribution instructions](#physical-distribution-operations)                 |
| Agda@executeController           | [Interpret controller authority instructions](#logical-authority-and-ownership-operations)       |
| **Instruction Set Interpreters** | -                                                                                                 |
| Agda@executeInstr₀               | [Execute basic instruction set (object + introspect)](#basic-instruction-set-interpreter-instr₀) |
| Agda@executeInstr₁               | [Execute with transaction support](#transactional-instruction-set-interpreter-instr₁)            |
| Agda@executeInstr₂               | [Execute with pure computation support](#extended-instruction-set-interpreter-instr₂)            |
| Agda@executeInstruction          | [Execute full instruction set (all layers)](#complete-instruction-set-interpreter-instruction)   |
| **Program Interpreters**         | -                                                                                                 |
| Agda@interpretAVMProgram         | [Interpret programs using full Instruction set](#complete-avm-program-interpreter)               |

<figcaption>AVM Interpreter Components</figcaption>

</figure>

<!-- Agda imports

```agda
{-# OPTIONS --exact-split --without-K --guardedness #-}

open import Background.BasicTypes using (ℕ; List; Maybe)
```

-->

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

```agda
open import Background.BasicTypes hiding (ℕ)
open import Background.InteractionTrees

open import AVM.Instruction Val ObjectId MachineId ControllerId TxId ObjectBehaviour public
```

</details>

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
  ... | just obj | just meta = just (obj , meta)
  ... | just obj | nothing = nothing
  ... | nothing | just meta = nothing
  ... | nothing | nothing = nothing
```

Pending object creations reside within the transactional overlay and must be
queried prior to consulting the global store, ensuring visibility of uncommitted
creates within the transaction scope.

```agda
  lookupPendingCreate : ObjectId → State → Maybe RuntimeObject
  lookupPendingCreate oid st = go (State.creates st)
    where
      go : List RuntimeObjectWithId → Maybe RuntimeObject
      go [] = nothing
      go ((oid' , obj , meta) ∷ rest) with eqObjectId oid oid'
      ... | true = just (obj , meta)
      ... | false = go rest
```

Pending transfers maintain a record of ownership changes scheduled for
 commitment within the current transaction scope.

```agda
  lookupPendingTransfer : ObjectId → State → Maybe ControllerId
  lookupPendingTransfer oid st = go (State.pendingTransfers st)
    where
      go : List (ObjectId × ControllerId) → Maybe ControllerId
      go [] = nothing
      go ((oid' , cid) ∷ rest) with eqObjectId oid oid'
      ... | true  = just cid
      ... | false = go rest
```

#### State Transformation Operations

Metadata update operations replace the metadata entry associated with a
specified object identifier, modifying the store in-place.

```agda
  updateMeta : ObjectId → ObjectMeta → State → State
  updateMeta oid meta st =
    record st {
      store = record (State.store st) {
        metadata = λ oid' → if eqObjectId oid oid'
                           then just meta
                           else Store.metadata (State.store st) oid'
      }
    }
```

Object creation with metadata performs atomic installation of both the object
behaviour and its associated metadata into the global store.

```agda
  createWithMeta : ObjectBehaviour → ObjectMeta → State → State
  createWithMeta obj meta st =
    let oid = ObjectMeta.objectId meta in
    record st {
      store = record (State.store st) {
        objects = λ oid' → if eqObjectId oid oid' then just obj
                           else Store.objects (State.store st) oid'
        ; metadata = λ oid' → if eqObjectId oid oid' then just meta
                             else Store.metadata (State.store st) oid'
      }
    }
```

#### Metadata Initialization and Observability Log Construction

Initial metadata construction assigns a newly created object to a specified
machine and controller, establishing its initial ownership and placement state.

```agda
  initMeta : ObjectId → MachineId → ControllerId → ObjectMeta
  initMeta oid mid cid = mkMeta oid [] mid cid cid
```

Log entry construction records observable events with a monotonically increasing
sequence number, event type classification, and the identifier of the controller
executing the operation.

```agda
  makeLogEntry : EventType → State → LogEntry
  makeLogEntry eventType st = mkLogEntry (State.eventCounter st) eventType (State.controllerId st)

  -- Increment the event counter after logging an event
  incrementEventCounter : State → State
  incrementEventCounter st = record st { eventCounter = suc (State.eventCounter st) }
```

#### Transaction Log Query Operations

Pending input retrieval accumulates all input messages directed to a specified
object within the scope of the current transaction, enabling objects to process
batched inputs upon behavioral invocation.

```agda
  pendingInputsFor : ObjectId → State → List Input
  pendingInputsFor oid st = collect (State.txLog st)
    where
      collect : List (ObjectId × Input) → List Input
      collect [] = []
      collect ((oid' , inp) ∷ rest)
        with eqObjectId oid oid'
      ...  | true  = inp ∷ collect rest
      ...  | false = collect rest
```

The addition of a pending write operation appends an input message to the
transaction log, recording the communication for subsequent processing during
transaction commitment.

```agda
  addPendingWrite : ObjectId → Input → State → State
  addPendingWrite oid inp st =
    record st { txLog = (State.txLog st) ++ ((oid , inp) ∷ []) }
```

#### Transactional Overlay Management

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
    record st { creates = remove (State.creates st) }
    where
      remove : List RuntimeObjectWithId → List RuntimeObjectWithId
      remove [] = []
      remove ((oid' , obj , meta) ∷ rest) with eqObjectId oid oid'
      ... | true  = rest
      ... | false = (oid' , obj , meta) ∷ remove rest
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

#### Read Set Tracking and Validation

Observation status checking determines whether an object identifier is present
in the transaction's read set, which is essential for implementing serializable
snapshot isolation semantics.

```agda
  containsObserved : ObjectId → List ObjectId → Bool
  containsObserved oid [] = false
  containsObserved oid (oid' ∷ rest)
      with eqObjectId oid oid'
  ... | true  = true
  ... | false = containsObserved oid rest
```

The ensure-observed operation conditionally adds an object to the read set if
not already present, maintaining the transaction's observability invariants.

```agda
  ensureObserved : ObjectId → ObjectMeta → State → State
  ensureObserved oid meta st =
    if containsObserved oid (State.observed st)
    then st
    else record st { observed = (State.observed st) ++ (oid ∷ []) }
```

Create set membership checking is performed during transaction validation to
determine whether an object identifier corresponds to a pending creation.

```agda
  inCreates : ObjectId → List RuntimeObjectWithId → Bool
  inCreates oid [] = false
  inCreates oid ((oid' , _ , _) ∷ rest) with eqObjectId oid oid'
  ... | true  = true
  ... | false = inCreates oid rest
```

Destroy set membership checking verifies whether an object identifier is
scheduled for deletion within the current transaction.

```agda
  inDestroys : ObjectId → List ObjectId → Bool
  inDestroys oid [] = false
  inDestroys oid (oid' ∷ rest) with eqObjectId oid oid'
  ... | true  = true
  ... | false = inDestroys oid rest
```

Read set validation ensures that all observed objects remain present in the
store, detecting conflicts that would violate serializable snapshot isolation
guarantees.

```agda
  validateObserved : State → Bool
  validateObserved st = checkAll (State.observed st)
    where
      checkAll : List ObjectId → Bool
      checkAll [] = true
      checkAll (oid ∷ rest) with inCreates oid (State.creates st)
      ... | true  = checkAll rest
      ... | false with Store.metadata (State.store st) oid
      ...   | nothing = false
      ...   | just meta = checkAll rest
```

Ownership validation verifies that an object is currently owned by the executing
controller, enforcing single-controller transaction semantics.

```agda
  checkOwnership : ObjectId → ObjectMeta → State → Bool
  checkOwnership oid meta st = ObjectMeta.currentController meta == State.controllerId st
```

#### Transaction Commitment Operations

Transaction commit atomically applies all pending changes from the transaction
overlay to the persistent Store (this goes to the assigned controller). With
this commit protocol, we aim to ensure ACID properties: atomicity
(all-or-nothing), consistency (valid state transitions backed by an controller),
isolation (uncommitted changes invisible to other controllers), and durability (committed changes
persist).

**Commit sequence:** Changes apply in dependency order to ensure referential integrity:

1. Agda@applyCreates: Install pending objects (must exist before writes/transfers)
2. Agda@applyTransfers: Update ownership (establish authority before operations)
3. Agda@applyWrites: Record input history (communication after ownership)
4. Agda@applyDestroys: Remove objects (cleanup after all operations)

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

Write application updates object metadata by appending the input message to the
object's history, permanently recording the communication.

```agda
  applyWrite : ObjectId → Input → State → State
  applyWrite oid inp st with Store.metadata (State.store st) oid
  ... | nothing = st
  ... | just meta =
    let newHistory = ObjectMeta.history meta ++ (inp ∷ [])
        meta' = mkMeta oid newHistory
                  (ObjectMeta.machine meta)
                  (ObjectMeta.creatingController meta)
                  (ObjectMeta.currentController meta)
    in updateMeta oid meta' st
```

Batch write application commits all pending input messages from the transaction
log, sequentially updating object histories.

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
    record st {
      store = record (State.store st) {
        objects = λ oid' → if eqObjectId oid oid' then nothing
                          else Store.objects (State.store st) oid'
        ; metadata = λ oid' → if eqObjectId oid oid' then nothing
                             else Store.metadata (State.store st) oid'
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
  ... | just (obj , meta) = updateMeta oid (record meta { currentController = targetCid }) st
```

Batch transfer application commits all pending ownership modifications,
sequentially updating object controller assignments.

```agda
  applyTransfers : List (ObjectId × ControllerId) → State → State
  applyTransfers [] st = st
  applyTransfers (x ∷ xs) st = applyTransfers xs (applyTransfer x st)
```

#### Object Invocation Semantics

Object invocation executes the object's behavioral program within an isolated
execution context, processing accumulated input messages and producing output
values according to the object's implementation.

```agda
  {-# TERMINATING #-}
  mutual
    handleCall : ObjectId → Input → State → ObjectBehaviour → ObjectMeta → AVMResult (Maybe Val)
    handleCall oid inp st obj meta =
      let -- Update metadata with pending inputs before running behavior
          pendingInputs = pendingInputsFor oid st
          currentHistory = ObjectMeta.history meta ++ pendingInputs
          meta' = record meta { history = currentHistory }
          st' = updateMeta oid meta' st

          -- Create isolated execution context for the object
          objectContext = record st' {
              self = oid
              ; input = inp
              ; sender = just (State.self st)
              ; machineId = ObjectMeta.machine meta
              ; controllerId = ObjectMeta.currentController meta
            }
      in handleCall-aux currentHistory meta' st' (interpretAVMProgram (getBehavior obj) objectContext) (State.tx st)
      where
        handleCall-aux : List Input → ObjectMeta → State → AVMResult (List Val) → Maybe TxId → AVMResult (Maybe Val)
        handleCall-aux currentHistory meta' st' (failure err) _ = failure err
        handleCall-aux currentHistory meta' st' (success res) (just _) =
          -- In transaction: add pending write
          let outputs = Success.value res
              output = if null outputs then nothing else just (head outputs)
              stAfterBehavior = Success.state res
              finalHistory = currentHistory ++ (inp ∷ [])
              metaFinal = record meta' {
                  history = finalHistory
                }
              stFinal = record stAfterBehavior {
                  -- Restore caller's context
                  self = State.self st
                  ; input = State.input st
                  ; sender = State.sender st
                }
              stWithPending = addPendingWrite oid inp stFinal
              entry = makeLogEntry (ObjectCalled oid inp output) stWithPending
              st'' = incrementEventCounter (updateMeta oid metaFinal stWithPending)
          in success (mkSuccess output st'' (Success.trace res ++ (entry ∷ [])))
        handleCall-aux currentHistory meta' st' (success res) nothing =
          -- Outside transaction: update metadata directly
          let outputs = Success.value res
              output = if null outputs then nothing else just (head outputs)
              stAfterBehavior = Success.state res
              finalHistory = currentHistory ++ (inp ∷ [])
              metaFinal = record meta' {
                  history = finalHistory
                }
              stFinal = record stAfterBehavior {
                  -- Restore caller's context
                  self = State.self st
                  ; input = State.input st
                  ; sender = State.sender st
                }
              stWithMeta = updateMeta oid metaFinal stFinal
              entry = makeLogEntry (ObjectCalled oid inp output) stWithMeta
              st'' = incrementEventCounter stWithMeta
          in success (mkSuccess output st'' (Success.trace res ++ (entry ∷ [])))
```

### Operational Semantics: Instruction-Level Interpretation

#### Object Lifecycle Operations

Object creation generates a fresh identifier and performs atomic installation of
the object along with its metadata into the global store, establishing the
object's initial state and ownership.

```agda
    callObjInTx : ObjectId → Input → State → ObjectBehaviour → ObjectMeta → AVMResult (Maybe Output)
    callObjInTx oid inp st obj meta with checkOwnership oid meta st
    ... | false = failure (err-cross-controller-tx oid (ObjectMeta.currentController meta))
    ... | true = handleCall oid inp (ensureObserved oid meta st) obj meta

    transferObjInTx : ObjectId → ControllerId → ObjectMeta → State → AVMResult Bool
    transferObjInTx oid targetCid meta st with checkOwnership oid meta st
    ... | false = failure (err-cross-controller-tx oid (ObjectMeta.currentController meta))
    ... | true =
      let stObs = ensureObserved oid meta st
          st' = incrementEventCounter (addPendingTransfer oid targetCid stObs)
          entry = makeLogEntry (ObjectTransferred oid (State.controllerId st) targetCid) st
      in success (mkSuccess true st' (entry ∷ []))

    transferObjNoTx : ObjectId → ControllerId → ObjectMeta → State → AVMResult Bool
    transferObjNoTx oid targetCid meta st with checkOwnership oid meta st
    ... | false = failure (err-cross-controller-tx oid (ObjectMeta.currentController meta))
    ... | true =
      let meta' = record meta { currentController = targetCid }
          st' = incrementEventCounter (updateMeta oid meta' st)
          entry = makeLogEntry (ObjectTransferred oid (State.controllerId st) targetCid) st
      in success (mkSuccess true st' (entry ∷ []))

    destroyObjInTx : ObjectId → ObjectMeta → State → AVMResult Bool
    destroyObjInTx oid meta st with checkOwnership oid meta st
    ... | false = failure (err-cross-controller-tx oid (ObjectMeta.currentController meta))
    ... | true =
      let stObs = ensureObserved oid meta st
          st' = incrementEventCounter (addPendingDestroy oid stObs)
          entry = makeLogEntry (ObjectDestroyed oid) st
      in success (mkSuccess true st' (entry ∷ []))

    destroyObjNoTx : ObjectId → ObjectMeta → State → AVMResult Bool
    destroyObjNoTx oid meta st with checkOwnership oid meta st
    ... | false = failure (err-cross-controller-tx oid (ObjectMeta.currentController meta))
    ... | true =
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
    executeObj (createObj behaviorName) st with State.tx st
    ... | nothing =
      let oid = freshObjectId (State.eventCounter st) in
      let obj = interpretBehaviorName behaviorName in
      let meta = initMeta oid (State.machineId st) (State.controllerId st) in
      let entry = makeLogEntry (ObjectCreated oid behaviorName) st in
      let st' = incrementEventCounter (createWithMeta obj meta st) in
      success (mkSuccess oid st' (entry ∷ []))
    ... | just _ =
      let oid = freshObjectId (State.eventCounter st) in
      let obj = interpretBehaviorName behaviorName in
      let meta = initMeta oid (State.machineId st) (State.controllerId st) in
      let entry = makeLogEntry (ObjectCreated oid behaviorName) st in
      let st' = incrementEventCounter (addPendingCreate oid obj meta st) in
      success (mkSuccess oid st' (entry ∷ []))
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

Object invocation executes the object's behavioral program with the accumulated
input history and the new input message, producing output values and potentially
modifying state.

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
    executeIntrospect self st = success (mkSuccess (State.self st) st [])
```

The `input` instruction retrieves the current input message from the execution
context, providing access to the invocation argument.

```agda
    executeIntrospect input st = success (mkSuccess (State.input st) st [])
```

The `getCurrentMachine` instruction retrieves the physical machine identifier
from the execution context, indicating the hardware node executing the current
computation.

```agda
    executeIntrospect getCurrentMachine st = success (mkSuccess (State.machineId st) st [])
```

The `history` instruction retrieves the complete accumulated input history of
the current object, including both committed and pending inputs within the
transaction scope.

```agda
    executeIntrospect history st with lookupObjectWithMeta (State.self st) st
    ... | nothing = failure (err-object-not-found (State.self st))
    ... | just (obj , meta) =
          let accumulatedHistory = ObjectMeta.history meta ++ pendingInputsFor (State.self st) st
          in success (mkSuccess accumulatedHistory st [])
```

The `sender` instruction retrieves the object identifier of the invoking object,
returning `nothing` for top-level program execution contexts.

```agda
    executeIntrospect sender st =
      success (mkSuccess (State.sender st) st [])
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
      let results = foldr collectMatches [] (allObjectIds st)
      in success (mkSuccess results st [])
      where
        collectMatches : ObjectId → List (ObjectId × ObjectMeta) → List (ObjectId × ObjectMeta)
        collectMatches oid acc with Store.metadata (State.store st) oid
        ... | just meta = if pred meta then (oid , meta) ∷ acc else acc
        ... | nothing = acc
```

Deep scrying extends metadata scrying by applying predicates to both the object
implementation and its metadata, enabling content-based object discovery.

```agda
    executeReflect (scryDeep pred) st =
      let results = foldr collectMatches [] (allObjectIds st)
      in success (mkSuccess results st [])
      where
        collectMatches : ObjectId → List RuntimeObjectWithId → List RuntimeObjectWithId
        collectMatches oid acc with Store.objects (State.store st) oid | Store.metadata (State.store st) oid
        ... | just obj | just meta = if pred obj meta then (oid , obj , meta) ∷ acc else acc
        ... | just _ | nothing = acc
        ... | nothing | just _ = acc
        ... | nothing | nothing = acc
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
                  (State.controllerId st)
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

Transaction initialization allocates a fresh transaction identifier and
establishes empty transactional overlays for staged operations, creating an
isolated execution scope.

```agda
    executeTx : ∀ {s A} → TxInstruction s A → State → AVMResult A
    executeTx beginTx st =
      let txid = freshTxId (State.eventCounter st)
          st' = incrementEventCounter (record st { tx = just txid
                           ; txLog = []
                           ; creates = []
                           ; destroys = []
                           ; observed = []
                           })
          entry = makeLogEntry (TransactionStarted txid) st
      in success (mkSuccess txid st' (entry ∷ []))
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
      let stApplied = applyDestroys (State.destroys st)
                        (applyWrites (State.txLog st)
                          (applyTransfers (State.pendingTransfers st)
                            (applyCreates (State.creates st) st)))
          st' = incrementEventCounter (record stApplied {
                  tx = nothing
                ; txLog = []
                ; creates = []
                ; destroys = []
                ; observed = []
                ; pendingTransfers = []
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

Current controller retrieval extracts the controller identifier from the
execution context, indicating the logical authority under which the current
computation executes.

```agda
    executeController : ∀ {s A} → ControllerInstruction s A → State → AVMResult A
    executeController getCurrentController st =
      success (mkSuccess (State.controllerId st) st [])
```

Object controller retrieval queries the object's current ownership, consulting
pending transfers within the transactional overlay before accessing committed
metadata in the global store.

```agda
    executeController (getController oid) st with lookupPendingCreate oid st
    ... | just (_ , meta) = success (mkSuccess (just (ObjectMeta.currentController meta)) st [])
    ... | nothing with lookupPendingTransfer oid st | Store.metadata (State.store st) oid | State.tx st
    ...   | just cid | _           | _        = success (mkSuccess (just cid) st [])
    ...   | nothing  | nothing     | _        = success (mkSuccess nothing st [])
    ...   | nothing  | just meta   | just _   = success (mkSuccess (just (ObjectMeta.currentController meta)) (ensureObserved oid meta st) [])
    ...   | nothing  | just meta   | nothing  = success (mkSuccess (just (ObjectMeta.currentController meta)) st [])
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
      let st' = addPendingTransfer oid targetCid st
          entry = makeLogEntry (ObjectTransferred oid (State.controllerId st) targetCid) st
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
    executeController (freeze oid) st
      with isReachableController (State.controllerId st)
         | lookupPendingCreate oid st
         | lookupObjectWithMeta oid st
    ... | false | _      | _              = failure (err-controller-unreachable (State.controllerId st))
    ... | true  | just (obj , meta) | _   =
      -- Freeze pending object: synchronize through controller
      let st' = incrementEventCounter st
          entry = makeLogEntry (ObjectFrozen oid (State.controllerId st)) st
      in success (mkSuccess true st' (entry ∷ []))
    ... | true  | nothing | nothing        = failure (err-object-not-found oid)
    ... | true  | nothing | just (obj , meta) =
      -- Freeze existing object: synchronize all replicas through the controller
      -- Note: Full implementation would reconcile all fetched replicas
      let st' = incrementEventCounter st
          entry = makeLogEntry (ObjectFrozen oid (State.controllerId st)) st
      in success (mkSuccess true st' (entry ∷ []))
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
