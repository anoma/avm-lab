---
title: Sequential Objects
icon: material/package-variant
tags:
  - AVM green paper
  - semantics
  - interaction trees
---

This module formalises the core object model from the AVM green paper, including
the definitions of _Sequential Object_ as in
Agda@SequentialObject, _Observational Equivalence_, and _Behavioural State_.
These definitions form the mathematical foundation for AVM objects as
transaction processing units. Objects may be deterministic or nondeterministic
depending on whether they use the `choose` instruction.

In one phrase, the AVM object-model builds on Pierce and Turner's purely
functional objects whilst incorporating the interaction trees foundation to
model object behaviour and state transitions. The object model is also close to
Setzer's coalgebraic presentation of interactive programs in Agda
[@AbelAdelsbergerSetzer-JFP-2017].

```agda
{-# OPTIONS --without-K --type-in-type --guardedness --exact-split #-}

open import Background.BasicTypes using (ℕ)
```

## Module Parameters

This module is parameterized by the core identifier and value types that users must provide to instantiate the AVM object model.

```agda
module Background.Objects
    -- Core message type
    (Val : Set)

    -- Object and transaction identifiers
    (ObjectId : Set)

    -- Machine/distribution types
    (MachineId : Set)

    -- Controller/distribution types
    (ControllerId : Set)

    -- Transaction types
    (TxId : Set)
  where

open import Background.BasicTypes
open import Background.InteractionTrees

-- Input/Output types for object communication
-- Note: Input and Output are currently both Val for simplicity,
-- but the object model supports distinct types (I-O objects for I ≠ O).
-- The type system allows heterogeneous communication where
-- inputs and outputs have different structures.
Input : Set
Input = Val

Output : Set
Output = Val

-- System-level identifiers for ownership and error handling
postulate
  sys-id : ObjectId  -- System object ID for error handling
  crash-msg : Val    -- Crash message as Val
  error-msg : Val    -- Error message as Val

-- Input sequence type alias for clarity
InputSequence : Set
InputSequence = List Input
```

## Motivation

From the AVM green paper Definition 1: "For fixed sets `I` and `O`, an `I-O
object` type is a partial function `φ : I* ⇀ O` whose domain of definition
contains the empty word and is closed under prefixes."

This captures the essence of objects as stateful computations that:

- Process sequences of inputs to produce outputs,
- Maintain history-dependent behaviour,
- Support observational equivalence and behavioural abstraction, and
- Can "crash" (become undefined) after certain input sequences.

## Core Definitions

### Input and Output Types

Objects are parameterised by their input and output alphabets. The theory
supports I-O objects where input type I and output type O may differ.
The specification currently aliases both `Input` and `Output` to `Val` for
implementation simplicity, but the abstract object model (φ : I\* ⇀ O) permits
heterogeneous types. The `Val` type is passed as a module parameter, allowing
flexibility in message representation (S-expressions, JSON, or custom formats).

Future specializations may distinguish input and output types (e.g., Input = Command,
Output = Response) while maintaining the same operational semantics.

Objects maintain a history of all inputs received, represented as lists.
The `InputSequence` type (defined above as `List Input`) captures this history:

The initial state of an object corresponds to the empty input history:

```agda
ε : InputSequence
ε = []
```

Extending a history with a new input:

```agda
_·_ : InputSequence → Input → InputSequence
xs · x = xs ++ (x ∷ [])

infixl 5 _·_
```

### Object types

An object type is a partial function from input sequences to outputs.

```agda
record Object : Set where
  constructor mkObjType
  field
    behavior : InputSequence → Maybe (List Output)
```

<!-- Alternative coinductive representation:
record Object : Set where
  coinductive
  field
    initial-output : Output
    step : Input → Maybe (Output × Object)
-->

This partial function must satisfy closure under prefixes. To define this
property, let us create some convenient notation for reasoning about partial
function domains.

Definedness is determined by runtime evaluation, not static information.

```agda
is-defined : Object → InputSequence → Set
is-defined φ xs = Object.behavior φ xs ≢ nothing
```

The output of a given object type is obtained by applying it to an input sequence, given that it is defined on that sequence.

```agda
output-of
  : (φ : Object) →
    (xs : InputSequence) →
    is-defined φ xs
    ----------------------
    → List Output
output-of φ xs defined
    with Object.behavior φ xs
... | just o = o
... | nothing = ⊥-elim (defined refl)
```

An object type must satisfy three crucial properties to be considered well-formed.

```agda
record WellFormedObjectType (φ : Object) : Set where
  constructor wfObjType
  field
    -- Domain contains empty word
    empty-defined : is-defined φ ε

    -- Closed under prefixes:
    prefix-closed
      : ∀ xs x →
      is-defined φ (xs · x) →
      -----------------------
      is-defined φ xs

    -- Functional consistency: behavior is a well-defined function
    -- (not "deterministic" in the sense of no randomness, but rather
    -- that the function is single-valued: same input yields same output)
    deterministic
      : ∀ xs → (o₁ o₂ : List Output) →
      Object.behavior φ xs ≡ just o₁ →
      Object.behavior φ xs ≡ just o₂ →
      ---------------------------
      o₁ ≡ o₂
```

### φ-equivalence

Two input sequences are φ-equivalent if they induce the same future behaviour.

```agda
_≈φ[_]_ : InputSequence → Object → InputSequence → Set
hist₁ ≈φ[ φ ] hist₂ = ∀ (future : InputSequence) →
  Object.behavior φ (hist₁ ++ future) ≡ Object.behavior φ (hist₂ ++ future)
```

#### Lemma. φ-equivalence is Reflexive

Every input sequence is φ-equivalent to itself.

```agda
≈φ-refl : ∀ (φ : Object) (hist : InputSequence) → hist ≈φ[ φ ] hist
≈φ-refl φ hist future = refl
```

#### Lemma. φ-equivalence is Symmetric

φ-equivalence is symmetric.

```agda
≈φ-sym : ∀ {φ} {hist₁ hist₂ : InputSequence} →
  hist₁ ≈φ[ φ ] hist₂ → hist₂ ≈φ[ φ ] hist₁
≈φ-sym equiv future = sym (equiv future)
```

#### Lemma. φ-equivalence is Transitive

φ-equivalence is transitive.

```agda
≈φ-trans : ∀ {φ} {hist₁ hist₂ hist₃ : InputSequence} →
  hist₁ ≈φ[ φ ] hist₂ →
  hist₂ ≈φ[ φ ] hist₃ →
  hist₁ ≈φ[ φ ] hist₃
≈φ-trans equiv₁₂ equiv₂₃ future =
  trans (equiv₁₂ future) (equiv₂₃ future)
```

### Behavioural State

The behavioural state of an object type abstracts from specific histories to
equivalence classes. It represents the equivalence class of all histories that produce identical future behaviour.

```agda
BehaviouralState : Object → InputSequence → Set
BehaviouralState φ hist =
  Σ InputSequence (λ hist' → hist' ≈φ[ φ ] hist)
```

### Sequential Objects

A sequential `I-O` object, or simply _object_ in this context, is
a term of type Agda@SequentialObject that groups an input history with its
Agda@Object type and a witness the object type is well-formed. Objects may be
deterministic (no nondeterministic instructions) or nondeterministic (using
`choose` instruction for intent matching).

```agda
record SequentialObject : Set where
  constructor mkObj
  field
    history : InputSequence
    object-type : Object
    well-formed : WellFormedObjectType object-type
```

Given an object, one can query its output, transition the object to a new one by
processing new input, query if it can proces an input or not, among other
things.

Let us define some of these functions.

Without closing the record definition above, we can define a few convenient functions
as extra projections. For example, retrieving the current output for a sequential
object is equivalent to evaluating the object type on its history:

```agda
  output : Maybe (List Output)
  output = Object.behavior object-type history
```

With the definition above, given a sequential object `o`, using the projection
`obj.output` we obtain a result `just z` or `nothing` in case it is undefined.

```agda
open SequentialObject public
```

- Transition an object by processing a new input, potentially resulting in a
  crash (this aspect is not considered in Goose-lean):

```agda
process-input
  : SequentialObject
  → Input
  ------------------------
  → Maybe SequentialObject

process-input (mkObj hist φ wf) inp
  with Object.behavior φ (hist · inp)
... | just out = just (mkObj (hist · inp) φ wf)
... | nothing  = nothing  -- Object crashes
```

- Determine whether an object can safely process a given input:

```agda
can-process : SequentialObject → Input → Set
can-process (mkObj hist φ wf) inp = is-defined φ (hist · inp)
```

<!-- All these function can go in the AVMEvent type when
formalising the full semantics -->

### Observational Equivalence

Two objects are observationally equivalent if they produce the _same outputs_
for _all future input sequences_. We define this notion as the following
relation.

```agda
_≈ᵒ_ : SequentialObject → SequentialObject → Set
mkObj hist₁ φ₁ wf₁ ≈ᵒ mkObj hist₂ φ₂ wf₂ =
  (future : InputSequence) →
  Object.behavior φ₁ (hist₁ ++ future) ≡ Object.behavior φ₂ (hist₂ ++ future)

infix 4 _≈ᵒ_
```

<!--
Is this the right notion?

- Is it compositional? no.
- two objects might be equivalent even if they have different histories, as long
as those histories *lead* to the same state that responds identically to future
inputs.

-->

#### Lemma. Observational Equivalence is Reflexive

Every object is observationally equivalent to itself.

```agda
≈ᵒ-refl : (obj : SequentialObject) → obj ≈ᵒ obj
≈ᵒ-refl (mkObj hist φ wf) _ = refl
```

#### Lemma. Observational Equivalence is Symmetric

If object $A$ is equivalent to object $B$, then $B$ is equivalent to $A$.

```agda
≈ᵒ-sym : (obj₁ obj₂ : SequentialObject)
  → obj₁ ≈ᵒ obj₂
  --------------
  → obj₂ ≈ᵒ obj₁
≈ᵒ-sym _ _ o₁≈ᵒo₂ future = sym (o₁≈ᵒo₂ future)
```

#### Lemma. Observational Equivalence is Transitive

Observational equivalence composes, forming an equivalence relation.

```agda
≈ᵒ-trans
  : (obj₁ obj₂ obj₃ : SequentialObject) →
  obj₁ ≈ᵒ obj₂ →
  obj₂ ≈ᵒ obj₃ →
  ----------------
  obj₁ ≈ᵒ obj₃
≈ᵒ-trans _ _ _ eq₁₂ eq₂₃ future =
  trans (eq₁₂ future) (eq₂₃ future)
```

Two objects share behavioural state if they have the same object-type and
φ-equivalent histories:

```agda
_≈ˢ_ : SequentialObject → SequentialObject → Set
mkObj hist₁ φ₁ wf₁ ≈ˢ mkObj hist₂ φ₂ wf₂ =
    φ₁ ≡ φ₂               -- same obj. type
  × hist₁ ≈φ[ φ₁ ] hist₂  -- same observations
```

#### Lemma. State Equivalence is Reflexive

Every object shares behavioural state with itself.

```agda
≈ˢ-refl : (obj : SequentialObject) → obj ≈ˢ obj
≈ˢ-refl (mkObj hist φ wf) = refl , ≈φ-refl φ hist
```

#### Lemma. State Equivalence is Symmetric

State equivalence is symmetric.

```agda
≈ˢ-sym : (obj₁ obj₂ : SequentialObject) →
  obj₁ ≈ˢ obj₂
  -------------
  → obj₂ ≈ˢ obj₁

≈ˢ-sym (mkObj hist₁ φ₁ wf₁) (mkObj hist₂ .φ₁ wf₂) (refl , hist-eq) =
  refl , ≈φ-sym {φ₁} hist-eq
```

#### Lemma. State Equivalence is Transitive

State equivalence is transitive.

```agda
≈ˢ-trans
  : (obj₁ obj₂ obj₃ : SequentialObject) →
  obj₁ ≈ˢ obj₂ →
  obj₂ ≈ˢ obj₃ →
  ---------------
  obj₁ ≈ˢ obj₃

≈ˢ-trans
  (mkObj hist₁ φ₁ wf₁)
  (mkObj hist₂ .φ₁ wf₂)
  (mkObj hist₃ .φ₁ wf₃)
  (refl , hist-eq₁₂)
  (refl , hist-eq₂₃) =
  refl , ≈φ-trans {φ₁} hist-eq₁₂ hist-eq₂₃
```

#### Lemma. Behavioural State Implies Observational Equivalence

Given two sequential objects, if they are equivalent by state observability, then they are also observationally equivalent.

```agda
behavioural-state-equiv
  : ∀ (obj₁ obj₂ : SequentialObject) →
  obj₁ ≈ˢ obj₂ →
  -------------------------------------
  obj₁ ≈ᵒ obj₂

behavioural-state-equiv _ _ (refl , hist-equiv) = hist-equiv
```

Whenever two sequential objects are constructed with the same
object type and all thier future observations match, these objects
can be consider equivalent via observation equivalence.

### Example Object Types

To illustrate these concepts, we define concrete examples of well-formed object types.
Since Val is abstract, we postulate constructors for demonstration purposes:

```agda
postulate
  VInt : ℕ → Val
  VString : String → Val
```

#### Counter Object

Maintains a count of all inputs received, returning the current count as output
as below. The output is represented as a Val integer:

```agda
counter-type : Object
counter-type = mkObjType (λ inputs → just ((VInt (length inputs)) ∷ []))
```

#### Lemma. Counter Well-Formedness

This object type is well-formed.

```agda
counter-wf : WellFormedObjectType counter-type
counter-wf = wfObjType
  (λ ()) -- empty-def
  prefix-def
  (λ xs o₁ o₂ eq1 eq2 →
     -- counter-type xs always returns
     -- just ((VInt (length xs)) ∷ []),
     -- so outputs must be equal
     begin
       o₁
     ≡⟨ sym (just-injective eq1) ⟩
       (VInt (length xs)) ∷ []
     ≡⟨ just-injective eq2 ⟩
       o₂
     ∎
  )
  where
    counter-always-defined : ∀ xs → ∃[ o ∈ List Output ] (Object.behavior counter-type xs ≡ just o)
    counter-always-defined xs = ((VInt (length xs)) ∷ []) ,Σ refl

    prefix-def : ∀ xs x
      → is-defined counter-type (xs · x)
      → is-defined counter-type xs
    prefix-def xs x eq-xs·x eq-xs
        with Object.behavior counter-type xs | counter-always-defined xs
    ... | just _ | _ = just≢nothing eq-xs
    ... | nothing | (o ,Σ eq) = just≢nothing (sym eq)
```

#### Echo Object

Returns a ready message initially, then echoes back the most recent input:

```agda
echo-type : Object
echo-type = mkObjType λ where
  [] → just ((VString "Echo is ready!") ∷ [])
  (inp ∷ _) → just (inp ∷ [])
```

This object type is also well-formed. Like the counter, the echo object is
always defined, prefix-closed, and functionally consistent (single-valued).

## Denotational versus Operational Semantics

The AVM object model admits two equivalent formulations that serve different
purposes.

**Denotational semantics (φ : I\* ⇀ O).** The object type is a partial function
from complete input histories to outputs. This is the mathematical specification
given in the green paper Definition 1. The function φ maps entire sequences of
inputs to observable results.

**Operational semantics (step : History × I → Maybe (O × History)).** The object
processes one input at a time, taking a prior history and new input, producing
an output and updated history. This is the executable form used in
implementations.

These formulations are equivalent up to currying and totality translation.

### Currying Equivalence

The denotational form `φ : I* ⇀ O` takes a complete sequence of inputs. The
operational form `step : (History × I) ⇀ (O × History)` takes a pair of prior
history and new input. These differ only in argument structure.

Given φ, we define step as follows. If `φ(ῑ · i)` is defined, then `step(ῑ, i)`
returns the pair `(φ(ῑ · i), ῑ · i)`. Otherwise step is undefined.

Conversely, given step, we recover φ by iterating. For the empty history, `φ(ε)`
is the initial output. For extended histories `φ(ῑ · i)`, we take the first
component of `step(ῑ, i)` when defined.

The two formulations encode the same information, related by
currying/uncurrying.

### Totality Translation

The denotational `φ : I* ⇀ O` is a partial function (undefined when the object
crashes). The operational form can be totalized to
`History × I → Maybe (O × History)` by making undefined cases explicit via `Maybe`.

This totalization or lifting transforms the partial function `I* ⇀ O` into a total
function `I* → Maybe O`.

The same applies to the operational form: the partial function `(History × I) ⇀
(O × History)` becomes the total function `(History × I) → Maybe (O × History)`.

The totalized forms are more convenient for implementation since Agda functions
are total by construction.

### Summary

The two semantics are equivalent representations. The denotational form
`φ : I* ⇀ O` and the operational form `step : (History × I) → Maybe (O ×
History)`
describe the same object behavior.

- Equivalence up to currying: sequence versus (history, input) decomposition
- Equivalence up to totality translation: partial versus total-with-Maybe

The AVM specification uses the denotational form for mathematical clarity.
Implementations use the operational form for execution. Both describe the same
object behavior.

## Operational Semantics

This section defines the operational behaviour of objects through _transition
relations_ and creation mechanisms. Objects can be viewed as state machines that
transition in response to inputs.

### Single-Step Transition

An object transitions from one state to another when processing a single input.
The transition relation `prevObj →[ input ] nextObj` holds when the object's
behavioral function produces an output for the extended history:´

```agda
data _→[_]_
  : SequentialObject
  → Input
  → SequentialObject
  → Set
  where

  transition
    : ∀ {hist φ wf input} →
    (outputs : List Output) →
    Object.behavior φ (hist · input) ≡ just outputs →
    ----------------------------------------------------
    let prevObj = mkObj hist φ wf  in
    let nextObj = mkObj (hist · input) φ wf in
    prevObj →[ input ] nextObj
```

### Multi-Step Transitions

Reflexive transitive closure of the transition relation for processing
sequences:

```agda
data _→*[_]_
  : SequentialObject → InputSequence → SequentialObject → Set
  where
  ε-trans : ∀ {obj} → obj →*[ ε ] obj

  step-trans
    : ∀ {obj₁ obj₂ obj₃ input inputs} →
    obj₁ →[ input ] obj₂ →
    obj₂ →*[ inputs ] obj₃ →
    ------------------------------
    obj₁ →*[ input ∷ inputs ] obj₃
```

#### Lemma. Transitions Preserve Object Type

Transitions preserve the object type, only the history changes.

```agda
transition-preserves-type
  : ∀ {obj₁ obj₂ input} →
  obj₁ →[ input ] obj₂ →
  ----------------------------------
  object-type obj₁ ≡ object-type obj₂
transition-preserves-type (transition output _) = refl
```

### Reachable objects

The set of objects reachable from an initial object via a sequence of inputs:

```agda
reachable-from : SequentialObject → InputSequence → Set
reachable-from obj inputs
  = Σ SequentialObject (λ obj' → obj →*[ inputs ] obj')
```

### Object Construction

Creates a fresh object with an empty input history:

```agda
create-object
  : (φ : Object)
  → WellFormedObjectType φ
  ------------------------
  → SequentialObject
create-object φ wf = mkObj ε φ wf
```

#### Lemma. Initial Output Existence

Well-formedness guarantees that every object has a defined initial output.

```agda
initial-output
  : (φ : Object)
  → (wf : WellFormedObjectType φ)
  -------------------------------
  → List Output
initial-output φ wf with Object.behavior φ ε | WellFormedObjectType.empty-defined wf
... | just o | _ = o
... | nothing | not-nothing = ⊥-elim (not-nothing refl)
```

We can now create concrete objects from the counter type defined earlier:

```agda
counter-object : SequentialObject
counter-object = create-object counter-type counter-wf
```

## Formal Properties

Key properties that hold for well-formed sequential objects.

### Lemma. Process Preserves Well-Formedness

If an object can process an input, the resulting object is also well-formed.

```agda
process-preserves-wf
  : ∀ (obj : SequentialObject) (input : Input) →
  can-process obj input →
  -----------------------------------------------
  ∃[ obj' ∈ SequentialObject ] (process-input obj input ≡ just obj')

process-preserves-wf (mkObj hist φ wf) inp can-proc
  with Object.behavior φ (hist · inp) | can-proc
... | just out | _ = mkObj (hist · inp) φ wf ,Σ refl
... | nothing | contra = ⊥-elim (contra refl)
```

### Lemma. Equivalence Preserved Under Input

Observationally equivalent objects remain equivalent after processing the same input.

```agda
equiv-preserved
  : (obj₁ obj₂ : SequentialObject) →
  (input : Input) →
  obj₁ ≈ᵒ obj₂ →
  can-process obj₁ input →
  can-process obj₂ input →
  -------------------------------------------
  ∃[ obj₁' ∈ SequentialObject ] ∃[ obj₂' ∈ SequentialObject ] (
    (process-input obj₁ input ≡ just obj₁') ×
    (process-input obj₂ input ≡ just obj₂') ×
    (obj₁' ≈ᵒ obj₂')
  )

equiv-preserved
  (mkObj hist₁ φ₁ wf₁)
  (mkObj hist₂ φ₂ wf₂)
  inp
  equiv
  can₁
  can₂
  with Object.behavior φ₁ (hist₁ · inp) | Object.behavior φ₂ (hist₂ · inp) | can₁ | can₂
... | just o₁ | just o₂ | _ | _ =
  mkObj (hist₁ · inp) φ₁ wf₁ ,Σ
          (mkObj (hist₂ · inp) φ₂ wf₂
          ,Σ (refl , refl , equiv-after-input))
  where
    -- Observational equivalence is preserved after processing
    -- the same input. The proof follows from the premise equiv
    -- by instantiating it with (inp ∷ future).
    equiv-after-input : mkObj (hist₁ · inp) φ₁ wf₁ ≈ᵒ mkObj (hist₂ · inp) φ₂ wf₂
    equiv-after-input future =
      begin
        Object.behavior φ₁ ((hist₁ · inp) ++ future)
      ≡⟨ cong (Object.behavior φ₁) (·-++-assoc hist₁ inp future) ⟩
        Object.behavior φ₁ (hist₁ ++ (inp ∷ future))
      ≡⟨ equiv (inp ∷ future) ⟩
        Object.behavior φ₂ (hist₂ ++ (inp ∷ future))
      ≡⟨ cong (Object.behavior φ₂) (sym (·-++-assoc hist₂ inp future)) ⟩
        Object.behavior φ₂ ((hist₂ · inp) ++ future)
      ∎
      where
        ·-++-assoc : ∀ xs x ys → ((xs · x) ++ ys) ≡ (xs ++ (x ∷ ys))
        ·-++-assoc xs x ys = ++-assoc xs (x ∷ []) ys
... | just o₁ | nothing | _ | contra = ⊥-elim (contra refl)
... | nothing | just o₂ | contra | _ = ⊥-elim (contra refl)
... | nothing | nothing | contra | _ = ⊥-elim (contra refl)
```

## Implementation

The operational semantics defines transitions as pure mathematical relations. This section shows how to execute these transitions as effectful computations. We integrate sequential objects with [interaction trees](./InteractionTrees.lagda.md), separating specification (what objects are) from implementation (how to execute them).

```agda
-- Import AVMProgram from Instruction module for implementation
-- Pass Object as a parameter to avoid circular dependency
-- Hide Input/Output to avoid name collisions since we already defined them
open import AVM.Instruction Val ObjectId MachineId ControllerId TxId Object
  hiding (Input; Output; Message; InputSequence; history)
```

### Integration with Interaction Trees

Objects are realized as interaction trees that maintain internal state and perform effects.

### Object Behaviour Type

An executable stateful computation parameterized by an internal state type:

```agda
ObjectBehaviour : Set → Set
ObjectBehaviour State = State → Input → AVMProgram (List Output × State)
```

### Single-Step Execution

Execute one step of object behaviour:

```agda
execute-step
  : {State : Set} →
  ObjectBehaviour State →
  State →
  Input →
  ------------------------------
  AVMProgram (List Output × State)
execute-step behaviour state inp = behaviour state inp
```

### Object Type to Behaviour Conversion

Lift a pure object type into an interaction tree, with crash handling.

```agda
objecttype-to-behaviour : Object → ObjectBehaviour InputSequence
objecttype-to-behaviour φ history inp with Object.behavior φ (history · inp)
... | just out = ret (out , history · inp)
... | nothing =
  -- Object crashes - return error message
  ret (error-msg ∷ [] , history)
```

### Batch Execution

Process a sequence of inputs using monadic composition:

```agda
run-object-itree
  : {State : Set} →
  ObjectBehaviour State →
  State →
  InputSequence →
  --------------------------------
  AVMProgram (List Output × State)
```

```agda
run-object-itree behaviour state [] = ret ([] , state)
run-object-itree behaviour state (inp ∷ inps) =
  behaviour state inp >>=
    λ { (out , state') →
      run-object-itree behaviour state' inps >>=
      λ { (outs , final-state) →
        ret (out ++ outs , final-state) }}
```

## Conclusion

This formalization establishes [sequential
objects](#sequential-objects) (Agda@SequentialObject) as the
mathematical foundation for the AVM object system. The development here
comprises three main components:

**Specification.** Objects are defined as [partial functions](#object-types)
from [input histories](#input-and-output-types) to outputs, equipped with
well-formedness constraints ensuring prefix closure and functional consistency.
[Observational equivalence](#observational-equivalence) provides the appropriate
notion of behavioral equality. Objects may incorporate nondeterministic choice
through the `choose` instruction.

**Operational Semantics.** [Transition relations](#single-step-transition)
define how objects evolve through input processing, valid transitions. We prove key properties
including [type preservation](#lemma-transitions-preserve-object-type) and
[equivalence preservation](#lemma-equivalence-preserved-under-input) under
transitions.

**Implementation.** The integration with [interaction
trees](./InteractionTrees.lagda.md) bridges pure specification
with [effectful execution](#integration-with-interaction-trees), enabling
objects to be realized as [stateful computations](#object-behaviour-type) with
explicit effect handling.
