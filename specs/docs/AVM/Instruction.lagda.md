---
title: AVM Instruction Set
icon: fontawesome/solid/code
tags:
  - Anoma Virtual Machine
  - instruction set
  - semantics
  - interaction trees
---

The AVM instruction set architecture (ISA) defines the primitive operations
available to AVM programs. The instruction taxonomy is organized into distinct
instruction sets, each characterized by specific safety levels and operational capabilities.

<figure markdown="1">

| Instruction Set             | Description                                                                      |
|:----------------------------|:--------------------------------------------------------------------------------|
| **Object layer**            | -                                                                                |
| Agda@createObj              | [Create new object by referencing behavior name](#createobj)                     |
| Agda@destroyObj             | [Mark object for destruction](#destroyobj)                                       |
| Agda@call                   | [Send message to object, receive output synchronously](#call)                    |
| Agda@receive                | [Receive next available message asynchronously](#receive)                        |
| **Introspection layer**     | -                                                                                |
| Agda@self                   | [Return current object's ID](#self)                                              |
| Agda@input                  | [Return current input message](#input)                                           |
| Agda@getCurrentMachine      | [Return current physical machine ID](#getcurrentmachine)                         |
| Agda@history                | [Return accumulated input sequence](#history)                                    |
| Agda@sender                 | [Return calling object's ID](#sender)                                            |
| Agda@reflect                | [Retrieve object metadata (unsafe)](#reflect-unsafe)                             |
| Agda@scryMeta               | [Query objects by metadata predicate (unsafe)](#scrymeta-unsafe)                 |
| Agda@scryDeep               | [Query objects by internals and metadata (unsafe)](#scrydeep-unsafe)             |
| **Transaction layer**       | -                                                                                |
| Agda@beginTx                | [Start new atomic transaction context](#begintx)                                 |
| Agda@commitTx               | [Commit transaction changes to store](#committx)                                 |
| Agda@abortTx                | [Abort transaction, discard changes](#aborttx)                                   |
| **Pure function layer**     | -                                                                                |
| Agda@callPure               | [Invoke registered pure function](#callpure)                                     |
| Agda@registerPure           | [Register new pure function (unsafe)](#registerpure-unsafe)                      |
| Agda@updatePure             | [Update existing pure function definition](#updatepure)                          |
| **Machine layer**           | -                                                                                |
| Agda@getMachine             | [Query physical machine location of object](#getmachine)                         |
| Agda@teleport               | [Move execution context to another machine](#teleport)                           |
| Agda@moveObject             | [Move object data to another machine](#moveobject)                               |
| **Controller layer**        | -                                                                                |
| Agda@getCurrentController   | [Return current controller ID](#getcurrentcontroller)                            |
| Agda@getController          | [Query object's controller](#getcontroller)                                      |
| Agda@transferObject         | [Transfer object ownership to another controller](#transferobject)               |
| Agda@freeze                 | [Convert object to strongly consistent (available to consistent)](#freeze)       |
| Agda@thaw                   | [Convert object to eventually consistent (consistent to available)](#thaw)       |
| **FD constraint layer**     | -                                                                                |
| Agda@newVar                 | [Create fresh constraint variable with finite domain](#newvar)                   |
| Agda@narrow                 | [Narrow variable domain by intersection](#narrow)                                |
| Agda@post                   | [Post relational constraint to constraint store](#post)                          |
| Agda@label                  | [Select value from variable's domain (search step)](#label)                      |
| **Nondeterminism layer**    | -                                                                                |
| Agda@choose                 | [Select value from preference distribution](#nondeterminism-instructions)        |
| Agda@require                | [Assert constraint for transaction](#nondeterminism-instructions)                |
| **Linear constraint layer** | -                                                                                |
| Agda@newLinearConstraint    | [Register linear constraint](#linear-constraint-instructions)                    |
| Agda@satisfyLinear          | [Mark constraint as satisfied](#linear-constraint-instructions)                  |

<figcaption>AVM Instruction Set Architecture</figcaption>

</figure>

The instruction architecture exhibits hierarchical organization, wherein each successive layer extends the
previous layer with additional operational capabilities. While individual instruction families
(e.g., Agda@ObjInstruction, Agda@TxInstruction) are defined
independently, the numbered instruction sets (Agda@Instr₀, Agda@Instr₁, etc.)
compose these families in a cumulative layered hierarchy where each level subsumes all capabilities from previous
layers.

<!-- Agda imports

```agda
{-# OPTIONS --without-K --type-in-type --guardedness --exact-split #-}
open import Background.BasicTypes
```

-->

## AVM Instruction Set Module Parameters

This module exhibits parametric polymorphism over types representing values,
object behaviours, transactions, and distributed execution infrastructure
(encompassing both physical machines and logical controllers).


<details markdown="1">
<summary>Module Parameters</summary>

```agda
module AVM.Instruction
    -- Core types
    (Val : Set)                      -- Used for both Input and Output currently
    (ObjectId : Set)

    -- Machine/distribution types
    (MachineId : Set)

    -- Controller/distribution types
    (ControllerId : Set)

    -- Transaction types
    (TxId : Set)

    -- Object behaviour type
    -- In concrete instantiations, this is AVMProgram (List Val)
    (ObjectBehaviour : Set)
  where

open import AVM.Context
  Val
  ObjectId
  MachineId
  ControllerId
  TxId
  ObjectBehaviour
  public
```

</details>

## AVM Instruction Set Algebraic Datatypes

The instruction sets are formalised as inductive algebraic datatypes, wherein each data
constructor represents a distinct instruction within the corresponding instruction set family.

Each instruction set datatype is indexed by a safety level parameter that characterises the
security properties of instructions within the set. The safety level indexing enables compile-time enforcement
of the invariant that safe programs cannot invoke unsafe operations, thereby providing static safety guarantees.

The instruction set architecture uses the following type-level signature:

```agda
ISA : Set
ISA = Set -> Set
```

The Agda@ISA type represents an instruction signature family wherein each
instruction is indexed by its return type Agda@A, thereby establishing Agda@ISA
as a parameterised type family.

### Agda@Safety Datatype

The specification defines two distinct safety levels: Agda@Safe and Agda@Unsafe.
Instructions are classified as unsafe when their execution violates foundational
principles of the object model encapsulation properties or introduces systemic
security risks to the virtual machine execution environment.

```agda
data Safety : Set where
  Safe   : Safety
  Unsafe : Safety
```

### Object Lifecycle and Communication Instructions

The object instruction family constitutes the foundational layer of the AVM ISA
hierarchy. This layer provides primitive object-oriented operations encompassing
object lifecycle management (creation and destruction) and inter-object
communication realized through message-passing semantics.

#### Agda@ObjInstruction datatype

```agda
-- Object lifecycle and communication
data ObjInstruction : Safety → ISA where
  -- Object lifecycle
  createObj : String → ObjInstruction Safe ObjectId
  destroyObj : ObjectId → ObjInstruction Safe Bool  -- May fail if object doesn't exist

  -- Message passing (may fail if object doesn't exist or rejects input)
  call : ObjectId → Input → ObjInstruction Safe (Maybe Output)

  -- Asynchronous message reception (waits for next message)
  receive : ObjInstruction Safe (Maybe Input)
```

These instructions realise the fundamental object-oriented programming model
within the AVM. Objects are created through Agda@createObj, terminated via
Agda@destroyObj, and interact through synchronous message-passing using the
Agda@call instruction. The Agda@receive instruction allows objects to wait for
incoming messages from other objects or the runtime environment and pattern
match on the received message.

### Runtime Introspection Instructions

The introspection instruction family constitutes the second layer of the AVM ISA
hierarchy. This layer provides primitive introspection operations enabling
programs to query the runtime execution environment state, and access object
metadata information.

#### Agda@IntrospectInstruction Datatype

Introspection instructions provide capabilities for querying the runtime
execution environment state and accessing object metadata without triggering
object behavioral execution.

```agda
data IntrospectInstruction : Safety → ISA where

  -- Safe introspection
  self : IntrospectInstruction Safe ObjectId
  input : IntrospectInstruction Safe Input
  getCurrentMachine : IntrospectInstruction Safe MachineId
  history : IntrospectInstruction Safe InputSequence
  sender : IntrospectInstruction Safe (Maybe ObjectId)

  -- Unsafe operations
  reflect : ObjectId → IntrospectInstruction Unsafe (Maybe ObjectMeta)

  -- Lookup operations
  scryMeta
    : (ObjectMeta → Bool) →
      IntrospectInstruction Unsafe (List (ObjectId × ObjectMeta))

  scryDeep
    : (ObjectBehaviour → ObjectMeta → Bool) →
      IntrospectInstruction Unsafe (List RuntimeObjectWithId)
```

The introspection instruction family introduces the first unsafe operations in
the AVM ISA. These instructions compromise encapsulation properties and
introduce computational complexity risks by initiating potentially expensive
global traversal operations whose cost scales linearly with the total object
population in the system store.

The Agda@scryMeta instruction performs predicate-based queries over the object
store, selecting objects whose metadata satisfies a specified boolean predicate.
The instruction returns pairs of object identifiers and corresponding metadata
for all matching entities. 

The Agda@scryDeep instruction extends this capability by enabling predicate
evaluation over both object internal state and metadata, returning the complete
object representation. Both instructions require complete store traversal.

The Agda@reflect instruction retrieves metadata for a specified object identifier without
invoking the object's message-passing interface. This operation violates the *encapsulation* principle that
objects *exclusively control the information they expose to external observers*.

### Agda@Instr₀: The Minimal Instruction Set

The first two instruction layers are composed to form the minimal instruction set, designated Agda@Instr₀.

This constitutes the foundational instruction set in the AVM ISA hierarchy,
encompassing the base object lifecycle operations and runtime introspection
capabilities.

```agda
data Instr₀ : Safety → ISA where
  Obj : ∀ {s A} → ObjInstruction s A → Instr₀ s A
  Introspect : ∀ {s A} → IntrospectInstruction s A → Instr₀ s A
```

```agda
open import Background.InteractionTrees
```

### Agda@TxInstruction datatype

This is the third layer of the AVM ISA. It provides the fundamental
transactional operations for managing atomic execution contexts.

```agda
data TxInstruction : Safety → ISA where
  beginTx : TxInstruction Safe TxId
  commitTx : TxId → TxInstruction Safe Bool      -- May fail if conflicts
  abortTx : TxId → TxInstruction Safe ⊤
```

### Agda@Instr₁ instruction set, adds transactional support

Programs that can roll back changes are possible via Agda@TxInstruction. With this feature, we can define our second instruction set,
Agda@Instr₁:

```agda
data Instr₁ : Safety → ISA where
  Obj : ∀ {s A} → ObjInstruction s A → Instr₁ s A
  Introspect : ∀ {s A} → IntrospectInstruction s A → Instr₁ s A
  Tx : ∀ {s A} → TxInstruction s A → Instr₁ s A
```

### Agda@PureInstruction datatype

This is the fourth layer of the AVM ISA. This step adds means to call pure
functions, register new pure functions, and update existing pure functions.
Thus, think of this as a capability to extend the any instruction set with
deterministic computation. For example adding arbitrary arithmetic operations to
the instruction set.

```agda
data PureInstruction : Safety → ISA where
  -- Call a registered pure function by identifier
  callPure : String → List Val → PureInstruction Safe (Maybe Val)

  -- Register new pure function (unsafe - extends the function set)
  registerPure : String → (List Val → Maybe Val) → PureInstruction Unsafe Bool

  -- Update function definition for a given function identifier
  updatePure : String → (List Val → Maybe Val) → PureInstruction Safe Bool
```

### Agda@Instr₂ instruction set, adds pure function computation

With the ability to call pure functions, we can define our third instruction
set, Agda@Instr₂, which adds pure function computation to the transactional
instruction set:

```agda
data Instr₂ : Safety → ISA where
  Obj : ∀ {s A} → ObjInstruction s A → Instr₂ s A
  Introspect : ∀ {s A} → IntrospectInstruction s A → Instr₂ s A
  Tx : ∀ {s A} → TxInstruction s A → Instr₂ s A
  Pure : ∀ {s A} → PureInstruction s A → Instr₂ s A
```

### Distribution Layer: Machine and Controller Instructions

This is the fifth layer of the AVM ISA, which provides instructions for managing
distributed execution across physical machines and logical controllers.

AVM programs execute in a distributed environment with two orthogonal concepts:
physical machines (where computation and storage occur) and logical controllers
(who order transactions and own objects). This separation enables independent
reasoning about data consistency, locality, and authority.

We split the distribution layer into two instruction families to maintain clear
separation of concerns, enable independent testing, and support distinct policy
enforcement for physical versus logical operations.

#### Agda@MachineInstruction datatype

Machines are physical nodes that host computation and object data. Programs can
query which machine holds an object's data and move execution or data objects
between machines. Machine operations deal with physical resource location and
object data migration.

```agda
data MachineInstruction : Safety → ISA where
  -- Query physical machine location of object data
  getMachine : ObjectId → MachineInstruction Safe (Maybe MachineId)

  -- Move execution context (process) to another machine
  teleport : MachineId → MachineInstruction Safe Bool

  -- Move object data to another machine (changes physical location)
  moveObject : ObjectId → MachineId → MachineInstruction Safe Bool
```

Safety constraints: Agda@teleport is invalid during active transactions.
Attempting to teleport while a transaction is in progress should result in an error.

#### Agda@ControllerInstruction datatype

Controllers are logical authorities that order transactions and own object
consistent state. Each object records which controller created it, which
controller currently owns it. Programs can query controller ownership and
transfer objects between controllers without moving their physical location.
Controller operations deal with logical resource location and object data
consistency.

```agda
data ControllerInstruction : Safety → ISA where
  -- Query controller identity and ownership
  getCurrentController : ControllerInstruction Safe ControllerId
  getController : ObjectId → ControllerInstruction Safe (Maybe ControllerId)

  -- Transfer object ownership to another controller (changes logical authority)
  transferObject : ObjectId → ControllerId → ControllerInstruction Safe Bool

  -- Freeze: convert available (eventually consistent) to consistent (strongly consistent)
  -- Pushes all pending changes to the controller for strong consistency
  freeze : ObjectId → ControllerInstruction Safe Bool

  -- Thaw: convert consistent (strongly consistent) to available (eventually consistent)
  -- Creates an eventually consistent snapshot from a consistent object
  thaw : ObjectId → ControllerInstruction Safe Bool
```

Authority requirements: Agda@transferObject requires proper authorization.
The current controller must have authority to transfer the object.

### Agda@Instr₃ instruction set, adds distributed operations

With machine and controller instructions, we can now define our fourth instruction
set, Agda@Instr₃. This instruction set adds distributed computing capabilities to
the pure function instruction set.

```agda
data Instr₃ : Safety → ISA where
  Obj : ∀ {s A} → ObjInstruction s A → Instr₃ s A
  Introspect : ∀ {s A} → IntrospectInstruction s A → Instr₃ s A
  Tx : ∀ {s A} → TxInstruction s A → Instr₃ s A
  Pure : ∀ {s A} → PureInstruction s A → Instr₃ s A
  Machine : ∀ {s A} → MachineInstruction s A → Instr₃ s A
  Controller : ∀ {s A} → ControllerInstruction s A → Instr₃ s A
```

### Finite Domain Constraint Programming Layer

This is the sixth layer of the AVM ISA, which introduces finite domain (FD) instructions that enable
constraint-based programming where computation proceeds through constraint
propagation and search over symbolic variables with call-time choice semantics.

Variables are created with finite domains, domains are narrowed through
constraint propagation, and search proceeds by labeling variables with concrete
values. When labeling leads to constraint failure (domain emptying), transaction
rollback (Agda@abortTx) provides the backtracking mechanism to explore
alternative search paths.

#### Supporting Types for Constraint Programming

-  Variable identifiers for constraint variables
```agda
record VarId : Set where
  constructor mkVarId
  field
    varId : ℕ
```

- Finite domain: a set of possible values

```agda
Domain : Set
Domain = List Val
```



- Relational constraints between variables and values

```agda
data Constraint : Set where
  -- Equality: var₁ = var₂
  Eq : VarId → VarId → Constraint

  -- Inequality: var₁ ≠ var₂
  Neq : VarId → VarId → Constraint

  -- All-different: all variables must take distinct values
  AllDiff : List VarId → Constraint

  -- Value constraint: var = value
  ValEq : VarId → Val → Constraint
```

#### Agda@FDInstruction datatype

The FD instruction family provides instructions for creating, narrowing, and
posting constraints on constraint variables.

```agda
data FDInstruction : Safety → ISA where
  -- Create a fresh symbolic variable with an initial finite domain
  newVar : Domain → FDInstruction Safe VarId

  -- Narrow the domain of an existing variable (intersection); false if emptied
  -- TODO: investigate if this is really needed.
  narrow : VarId → Domain → FDInstruction Safe Bool

  -- Post a relational constraint (e.g., equality/inequality/distinctness)
  -- Triggers constraint propagation to narrow related variable domains
  post : Constraint → FDInstruction Safe Bool

  -- Labeling: select value from variable's domain (search step with call-time choice)
  -- Transaction abort (abortTx) provides backtracking when constraints fail
  label : VarId → FDInstruction Safe Val
```

### Experimental Instruction Families

The following instruction families support intent-based programming and
multi-party coordination. These are experimental features for prototyping
advanced AVM capabilities.

#### Nondeterminism Instructions

Nondeterminism instructions enable preference-directed selection and constraint
validation at transaction commit time, suited for multi-party intent matching
where constraints accumulate during execution and are evaluated atomically when
the transaction commits.

```agda
data NondetInstruction : Safety → ISA where
  -- Choose a value nondeterministically from available options
  -- Runtime may use preferences, weights, or solver guidance
  choose : List Val → NondetInstruction Safe Val

  -- Assert constraint that must hold at transaction commit
  -- If false when commitTx executes, transaction aborts
  require : Bool → NondetInstruction Safe ⊤
```

### Choosing Between FD and NonDet Layers

The FD and NonDet layers serve complementary purposes with incompatible
execution models.

#### Finite Domain (Call-Time Choice)

1. Execution Model:
  1. Values selected immediately when Agda@label executes
  2. Constraint propagation occurs incrementally after each Agda@post
  3. Transaction rollback, via Agda@abortTx, provides backtracking for search
  4. Domains narrow eagerly as constraints are posted

2. Suitable for:
  1. Single-agent CSP solving (TODO: add examples of N-Queens, Sudoku, scheduling)
  2. Search algorithms requiring systematic exploration
  3. Problems benefiting from incremental constraint propagation
  4. Resource allocation with backtracking

#### Nondeterminism (Commit-Time Validation)

1. Execution Model:
  1. Choices recorded but deferred until transaction commit, via Agda@choose
  2. Constraints accumulate and validate atomically at transaction commit, via Agda@commitTx
  3. Enables composition of multiple parties' preferences
  4. Solver considers all constraints simultaneously

2. Suitable for:
  1. Intent matching and multi-party coordination (TODO: add examples of token swaps)
  2. Preference-directed selection with solver guidance

#### Linear Constraint Instructions

Different from Agda@Constraint, Agda@LinearObjectConstraint is a set of constraints that are satisfied by the current step on terms of resource linearity. Meaning that the constraint is satisfied if and only if the current step uses an object exactly once.

- Linear object constraint identifiers
```agda
record LinearObjectConstraintId : Set where
  constructor mkLinearCid
  field
    constraintId : ℕ
```

- A linear object constraint requires that a message send be used exactly once
```agda
data LinearObjectConstraint : Set where
  UseOnce : ObjectId → Input → LinearObjectConstraint
```

- Register a new linear object constraint
```agda
data LinearObjectConstraintInstruction : Safety → ISA where
  -- Register a new linear object constraint; returns its identifier
  newLinearConstraint : LinearObjectConstraint → LinearObjectConstraintInstruction Safe LinearObjectConstraintId
  -- Mark a linear object constraint as satisfied by the current step
  satisfyLinear : LinearObjectConstraintId → LinearObjectConstraintInstruction Safe Bool
```

We don't define another instruction set for linear constraints, and fd, as with
these instructions we define "the" instruction set as given below.

### The Instruction datatype

The Agda@Instruction datatype combines all instructions so far defined, including
experimental ones, and provides ergonomic pattern synonyms for a flat instruction namespace.

```agda
-- Union datatype combining all instruction families
data Instruction : Safety → ISA where
  Obj : ∀ {s A} → ObjInstruction s A → Instruction s A
  Introspect : ∀ {s A} → IntrospectInstruction s A → Instruction s A
  Tx : ∀ {s A} → TxInstruction s A → Instruction s A
  Pure : ∀ {s A} → PureInstruction s A → Instruction s A
  Machine : ∀ {s A} → MachineInstruction s A → Instruction s A
  Controller : ∀ {s A} → ControllerInstruction s A → Instruction s A
  FD : ∀ {s A} → FDInstruction s A → Instruction s A
  Nondet : ∀ {s A} → NondetInstruction s A → Instruction s A
  LinearConstr : ∀ {s A} → LinearObjectConstraintInstruction s A → Instruction s A
```

#### Pattern Synonyms for convenience

Pattern synonyms provide a flat namespace for common instructions, eliminating
the need for nested constructors when writing AVM programs. These patterns can
also be used for pattern matching on instructions. It is a matter of using Agda here to describe the instruction set and pattern matching on it.

Also, it can be seen as the list of all instructions in the instruction set.

```agda
-- Object instruction patterns
pattern obj-create behaviorName = Obj (createObj behaviorName)
pattern obj-destroy oid = Obj (destroyObj oid)
pattern obj-call oid inp = Obj (call oid inp)
pattern obj-receive = Obj receive

-- Introspection instruction patterns
pattern get-self = Introspect self
pattern get-input = Introspect input
pattern get-current-machine = Introspect getCurrentMachine
pattern get-history = Introspect history
pattern get-sender = Introspect sender
pattern obj-scry-meta pred = Introspect (scryMeta pred)
pattern obj-scry-deep pred = Introspect (scryDeep pred)
pattern obj-reflect oid = Introspect (reflect oid)

-- Transaction instruction patterns
pattern tx-begin = Tx beginTx
pattern tx-commit tid = Tx (commitTx tid)
pattern tx-abort tid = Tx (abortTx tid)

-- Pure function instruction patterns
pattern call-pure name args = Pure (callPure name args)
pattern register-pure name fn = Pure (registerPure name fn)
pattern pure-update-pure name fn = Pure (updatePure name fn)

-- Machine instruction patterns (physical location and process migration)
pattern get-machine oid = Machine (getMachine oid)
pattern do-teleport mid = Machine (teleport mid)
pattern move-object oid mid = Machine (moveObject oid mid)

-- Controller instruction patterns (logical authority and ownership)
pattern get-current-controller = Controller getCurrentController
pattern get-controller oid = Controller (getController oid)
pattern transfer-object oid cid = Controller (transferObject oid cid)
pattern freeze-object oid = Controller (freeze oid)
pattern thaw-object oid = Controller (thaw oid)
```

The `Instruction` type provides the full AVM instruction set in a single datatype:

```agda
AVMProgram : Set → Set
AVMProgram = ITree (Instruction Safe)
```

### Object Behaviour Instantiation Note

In the AVM operational model, the `ObjectBehaviour` module parameter is concretely instantiated
as `AVMProgram (List Val)` in implementations. This establishes that runtime objects are pairs of
`(ObjectBehaviour × ObjectMeta)` - executable AVM programs paired with runtime metadata.

This design combines compositionality (instruction families as separate types)
with ergonomics (pattern synonyms for flat naming). Programs can use either the
layered approach (`Instr₀`, `Instr₁`, etc.) for compositional reasoning or the
unified `Instruction` type for convenience.

## Instruction Set Operational Semantics

The AVM specification provides multiple instruction sets, systematically organized by operational capability and safety classification.
This section establishes comprehensive operational semantics for all instruction types within the ISA hierarchy.
Each instruction specification includes its associated safety level classification and return type signature.

The instruction set organization by instruction family follows this taxonomy:

1. Agda@ObjInstruction: Object instructions: Object lifecycle and communication
2. Agda@IntrospectInstruction: Introspection instructions: Context queries and reflection
3. Agda@TxInstruction: Transaction instructions: Atomic execution contexts
4. Agda@PureInstruction: Pure function instructions: Deterministic computation via function registry
5. Agda@MachineInstruction: Machine instructions: Physical location and process migration
6. Agda@ControllerInstruction: Controller instructions: Logical authority and ownership transfer

### Object Lifecycle Operations

Object lifecycle instructions provide primitive operations for managing object creation and destruction within
the persistent object store.

#### Agda@createObj

```text
createObj : String → ObjInstruction Safe ObjectId
```

Creates a new runtime object within the persistent store by referencing a behavior name. The Agda@String
parameter specifies the behavior name that will be resolved by the interpreter to an Agda@ObjectBehaviour
(an AVM program). The instruction returns a fresh Agda@ObjectId that uniquely identifies the
newly created runtime object within the global object namespace. Object creation exhibits transactional
semantics: if the enclosing transaction context aborts, the object creation is rolled back and the
runtime object does not persist to the store.

#### Agda@destroyObj

```text
destroyObj : ObjectId → ObjInstruction Safe Bool
```

Marks the runtime object identified by the given Agda@ObjectId for destruction. Returns
Agda@true if destruction succeeds, Agda@false if the runtime object does not exist.
The runtime object (both behavior and metadata) is removed from the store, and subsequent references to this Agda@ObjectId will
fail. Destruction is transactional: the object remains accessible within the
current transaction until committed.

### Object Interaction

Object interaction is achieved through pure message-passing, preserving object
encapsulation. Message passing is the only way to interact with objects in the AVM.

#### Agda@call

```text
call : ObjectId → Input → ObjInstruction Safe (Maybe Output)
```

Performs synchronous message passing to the object identified by the given
Agda@ObjectId. Sends the input and blocks until the target object produces
an output. Returns Agda@nothing if the object does not exist or rejects the input,
otherwise returns Agda@just the output produced by the target object.

#### Agda@receive

```text
receive : ObjInstruction Safe (Maybe Input)
```

Receives the next available message for the current object. This instruction enables
asynchronous message reception, allowing objects to wait for incoming messages from
other objects or the runtime system. Returns Agda@just the received message if one is
available, or Agda@nothing if no message is available or if the object's message queue
is empty. The instruction may block until a message arrives, depending on the runtime
implementation's message delivery semantics. TBD!

### Introspection and Context

Introspection instructions query the execution environment and object metadata.

#### Agda@self

```text
self : IntrospectInstruction Safe ObjectId
```

Returns the Agda@ObjectId of the currently executing object. This instruction is
essential for recursion and self-reference, allowing an object to pass its own
identifier to other objects or invoke itself. See also the use of Agda@self in
defining purely functional resources in the AVM.

- https://forum.anoma.net/t/resources-as-purely-functional-objects/1455#p-5812-resource-class-implementation-6

#### Agda@input

```text
input : IntrospectInstruction Safe Input
```

Returns the input being processed by the current object.
This instruction provides access to the message sent to the current object.

#### Agda@getCurrentMachine

```text
getCurrentMachine : IntrospectInstruction Safe MachineId
```

Returns the identifier of the physical machine currently executing this program.
This instruction enables programs to reason about their execution location,
which is independent of controller identity. Machine information is useful for
data locality optimizations and understanding cross-machine communication costs.

#### Agda@history

```text
history : IntrospectInstruction Safe InputSequence
```

Returns the accumulated input history of the currently executing object,
including both committed inputs from object metadata and any pending inputs
within the current transaction. This allows object behaviors implemented as
AVM programs to access their complete input sequence for stateful computation.
The history enables objects to maintain state purely functionally by computing
outputs based on the full sequence of messages received.

#### Agda@sender

```text
sender : IntrospectInstruction Safe (Maybe ObjectId)
```

Returns the Agda@ObjectId of the calling object when invoked from within
a Agda@call instruction. Returns Agda@nothing for top-level execution contexts
or when no caller exists. This instruction enables objects to verify the origin
of received messages and implement access control policies based on caller identity.

Use cases:

- Authorization: verify the caller has permission to invoke an operation
- Access control: restrict functionality to specific objects
- Provenance tracking: maintain audit trails of message origins
- Capability-based security: objects acting as capabilities can verify bearer identity

Design notes:

- Safe instruction: purely introspective, no side effects
- Returns Agda@Maybe to handle both object calls and top-level execution
- Complements Agda@self (current object) with caller information
- Maintains separation: messages carry payloads, context carries metadata

#### Agda@scryMeta (Unsafe)

```text
scryMeta : (ObjectMeta → Bool) → IntrospectInstruction Unsafe (List (ObjectId × ObjectMeta))
```

Queries the store for objects whose metadata satisfies the given predicate.
Returns pairs of object identifiers and metadata for all matching objects.
This instruction traverses the entire store, applying the predicate to each
object's metadata. Typical queries include finding all objects created by a
specific controller, objects modified after a particular version, or objects
whose history exceeds a certain length. The instruction preserves parametricity
by not exposing the abstract Agda@Object type. Marked unsafe because enumeration
scales with total object count.

#### Agda@scryDeep (Unsafe)

```text
scryDeep : (ObjectBehaviour → ObjectMeta → Bool) → IntrospectInstruction Unsafe (List RuntimeObjectWithId)
```

Queries the store for runtime objects (object behaviours paired with metadata) that satisfy the given
predicate. Returns complete runtime object data for all matches. This instruction
breaks encapsulation by exposing raw Agda@ObjectBehaviour values, violating
parametricity. It eliminates the performance inconsistency of filtering on
object behavior internals while returning only identifiers. Use cases include deep
inspection for debugging, auditing, or migration. Marked unsafe for both
encapsulation violation and unbounded computation.

#### Agda@reflect (Unsafe)

```text
reflect : ObjectId → IntrospectInstruction Unsafe (Maybe ObjectMeta)
```

Retrieves metadata about the object identified by the given Agda@ObjectId.
Returns Agda@nothing if the object does not exist, otherwise returns Agda@just
the object's metadata. Marked as unsafe because it bypasses object
encapsulation.

### Transaction Control

Transaction control instructions manage atomic execution contexts. All state
modifications within a transaction are tentative until committed.

#### Agda@beginTx

```text
beginTx : TxInstruction Safe TxId
```

Initiates a new transactional context and returns a fresh transaction
identifier. All subsequent state modifications are logged to the transaction's
write-set until the transaction is either committed or aborted. Transactions
provide atomicity: either all changes succeed or none do.

#### Agda@commitTx

```text
commitTx : TxId → TxInstruction Safe Bool
```

Attempts to commit the transaction identified by the given Agda@TxId. Returns
Agda@true if the commit succeeds, persisting all logged changes to the store.
Returns Agda@false if the transaction cannot be committed due to conflicts with
concurrent transactions or if the transaction was already finalized.

#### Agda@abortTx

```text
abortTx : TxId → TxInstruction Safe ⊤
```

Aborts the transaction identified by the given Agda@TxId, discarding all
tentative state changes in its write-set. The store reverts to its state before
Agda@beginTx was called. This operation always succeeds and returns unit.

### Pure Function Instructions

Pure function instructions provide deterministic computation capabilities
through an extensible function registry.

#### Agda@callPure

```text
callPure : String → List Val → PureInstruction Safe (Maybe Val)
```

Invokes a registered pure function by identifier with the given arguments.
Returns the function result or `nothing` if the function doesn't exist or the
arguments don't match the expected arity.

#### Agda@registerPure (Unsafe)

```text
registerPure : String → (List Val → Maybe Val) → PureInstruction Unsafe Bool
```

Registers a new pure function in the function registry. Marked as unsafe because
it extends the global function set, potentially affecting system-wide
computation.

#### Agda@updatePure

```text
updatePure : String → (List Val → Maybe Val) → PureInstruction Safe Bool
```

Updates the function definition of a registered pure function identified by the
given string name. This instruction replaces the existing function definition in
the pure function registry with the new function provided. The first parameter
is the function identifier (name), and the second parameter is the new function
definition. Returns Agda@true if the update succeeds (the function exists and
was updated), Agda@false if the function does not exist in the registry or the
update fails. This instruction enables dynamic modification of pure function
implementations while maintaining deterministic computation semantics.

**Meta-level state changes:**

Pure function instructions modify the execution environment
(Agda@State.pureFunctions), not the object store. These are **capability
changes** altering what programs can compute.

Key properties:

- **Non-transactional**: Changes take effect immediately, not deferred to commit
- **Global scope**: Affects all subsequent execution system-wide
- **Registry semantics**: Functions identified by string names

| Operation         | Effect                                               | Traced |
|:------------------|:-----------------------------------------------------|:-------|
| Agda@registerPure | Add new function (unsafe: extends capabilities)      | No     |
| Agda@updatePure   | Replace existing function (safe: requires existence) | Yes    |
| Agda@callPure     | Invoke function (safe: read-only)                    | No     |

### Machine Instructions

Machine instructions manage physical resource location and process migration in
distributed AVM deployments. These operations deal with where computation
executes and where object data physically resides.

#### Agda@getMachine

```text
getMachine : ObjectId → MachineInstruction Safe (Maybe MachineId)
```

Returns the physical machine where the specified object's data resides, or
Agda@nothing if the object doesn't exist. The machine location is independent of
controller ownership. This information is useful for data locality optimization
and understanding cross-machine communication costs.

#### Agda@teleport

```text
teleport : MachineId → MachineInstruction Safe Bool
```

Moves the execution context (process) to the specified physical machine. Returns
Agda@true if teleportation succeeds, Agda@false if the target machine is unreachable.
This changes where computation happens, but does not change the controller identity
or object ownership.

Safety constraint: This instruction is invalid during active transactions. The
interpreter must reject teleportation attempts within a transaction boundary
to maintain transaction integrity. This is to preserve the transactional atomicity guarantees.

#### Agda@moveObject

```text
moveObject : ObjectId → MachineId → MachineInstruction Safe Bool
```

Moves an object's data to a different physical machine. Returns Agda@true if the
move succeeds, Agda@false if the target machine is unreachable. This changes the
object's physical storage location but does not change its controller ownership.
Machine migration enables room for data locality optimisation and load balancing.

### Controller Instructions

Controller instructions manage logical authority and ownership for distributed
AVM deployments. Controllers order transactions and own objects. These operations
deal with which controller has authority over objects and transaction ordering.
These operations are independent of the physical machine executing the code.

#### Agda@getCurrentController

```text
getCurrentController : ControllerInstruction Safe ControllerId
```

Returns the identifier of the controller (logical authority) currently executing
this program.

#### Agda@getController

```text
getController : ObjectId → ControllerInstruction Safe (Maybe ControllerId)
```

Returns the controller responsible for the specified object, or Agda@nothing if
the object doesn't exist. This queries the object's logical ownership, not its
physical location. The controller determines transaction ordering for the object.

#### Agda@transferObject

```text
transferObject : ObjectId → ControllerId → ControllerInstruction Safe Bool
```

Transfers logical ownership of an object to another controller. This changes
which controller orders transactions for the object but does not move the
object's data between machines. Returns Agda@true if the transfer succeeds, Agda@false
if the transfer is unauthorized or the target controller is unreachable.

Safety constraint: The current controller must have authority to transfer
the object.

#### Agda@freeze

```text
freeze : ObjectId → ControllerInstruction Safe Bool
```

Converts an available (eventually consistent) object to a consistent (strongly
consistent) object by pushing all pending changes to the controller. This
operation ensures that all replicas of the object become strongly consistent by
synchronizing state through the controller's ordering authority. Returns Agda@true if
the freeze operation succeeds, Agda@false if the object does not exist, is
not available, or the controller is unreachable.

#### Agda@thaw

```text
thaw : ObjectId → ControllerInstruction Safe Bool
```

Converts a consistent (strongly consistent) object to an available (eventually consistent)
object by creating an eventually consistent snapshot. This operation enables the object
to operate with eventual consistency semantics, allowing concurrent modifications that
will be reconciled eventually. Returns Agda@true if the thaw operation succeeds, Agda@false
if the object does not exist, is not consistent, or the operation is unauthorized.

### Finite Domain Constraint Programming Instructions

Finite domain (FD) instructions enable constraint-based programming where
computation proceeds through constraint propagation and search over symbolic
variables. These instructions support declarative problem-solving where
solutions emerge from posting constraints and labeling variables.

#### Agda@newVar

```text
newVar : Domain → FDInstruction Safe VarId
```

Creates a fresh constraint variable with the specified finite domain. The Agda@Domain
is a list of possible values the variable can take. Returns a unique variable
identifier for use in subsequent constraint operations. Variables created within
a transaction are local to that transaction's constraint store.

#### Agda@narrow

```text
narrow : VarId → Domain → FDInstruction Safe Bool
```

Narrows the domain of an existing constraint variable by intersecting it with
the provided domain. Returns Agda@true if the narrowing succeeds (the
intersection is non-empty), Agda@false if the intersection would empty the domain
(constraint failure). Domain narrowing is a fundamental operation in constraint
propagation.

#### Agda@post

```text
post : Constraint → FDInstruction Safe Bool
```

Posts a relational constraint to the constraint store. Constraints relate
variables through equality, Agda@Eq, inequality, Agda@Neq, and all-different,
Agda@AllDiff, or value equations, Agda@ValEq. Returns Agda@true if the
constraint is consistent with the current constraint store, Agda@false if
posting the constraint would lead to immediate failure. Constraint posting
triggers propagation to narrow the variable domains.

#### Agda@label

```text
label : VarId → FDInstruction Safe Val
```

Selects a value from the variable's current domain using call-time choice—the
value is chosen immediately when Agda@label executes. This is the search step in
constraint solving. If subsequent constraint propagation fails, transaction
rollback (Agda@abortTx) backtracks to explore alternative values. Returns the
selected value, or fails immediately if the domain is empty. This is the search
step in constraint solving.
