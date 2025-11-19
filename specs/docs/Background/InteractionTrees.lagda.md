---
title: Interaction Trees
icon: material/file-tree
tags:
  - interaction trees
  - semantics
---

This module formalises a version of [Interaction
Trees](https://github.com/DeepSpec/InteractionTrees). The concept of interaction
trees was introduced a few years ago by Xia et al, as a formal verification
framework for encoding the behaviour of effectful recursive programs that can
_interact_ with an _external environment_.For a more comprehensive library,
check out its [Rocq's official
library](https://github.com/Rocq/InteractionTrees).

The following is a translation to Agda of the main definitions
related to interaction trees.

```agda
{-# OPTIONS --without-K --type-in-type --guardedness --exact-split #-}

module Background.InteractionTrees where
open import Background.BasicTypes
```

## Event Signatures

An event signature describes the interface between a computation and its
environment. Each event has an associated response type.

```agda
EventSig : Set₁
EventSig = Set → Set
```

Event signatures are also called presented as _interfaces_ or meant to represent
the instruction sets of a program[@LesaniEtAl-PAPL-2022-C4].

```agda
Interface : Set₁
Interface = Set → Set
```

As a simple example, consider a messaging system with basic instructions/operations:

```agda
module _ {ObjectId : Set} where
  data MessageEvent : EventSig where
    send : String → MessageEvent ⊤
    receive : MessageEvent String
    create  : ObjectId → MessageEvent ⊤
    destroy : ObjectId → MessageEvent ⊤
```

## Interaction Tree First Definition (Non-productive)

An interaction tree is a datatype that encodes the behaviour of effectful,
non-deterministic, recursive programs.

We can define this datatype using
[coinduction](https://agda.readthedocs.io/en/latest/language/coinduction.html)
and eliminate this type via
[copatterns](https://agda.readthedocs.io/en/latest/language/copatterns.html#copatterns).

The following is a direct translation from the original paper's definition. As
you can see, the positivity checker must be disabled as Agda detects a negative
occurrence of `ITree₁ E R` within the tau constructor's type argument.

```agda
module first-definition where

  {-# NO_POSITIVITY_CHECK  #-}
  record ITree₁ (E : Set -> Set) (R : Set) : Set where
    coinductive
    field
      ret : R -> ITree₁ E R
      tau : ITree₁ E R -> ITree₁ E R
      vis : {A : Set}
          -> E A
          -> (A -> ITree₁ E R)
          -> ITree₁ E R
```

where `E` stands for a family of _events_ (external interactions) and `R` is
the type of results.

- Agda@ret is for terminal computations that produce results,
- Agda@tau is for _silent_ transitions used for internal computational steps, and
- Agda@vis is for a _visible_ event of type `E A` yielding an answer of type `A`.
  Here is where we branch via a _continuation_, encoding this in a function of
  type `A -> ITree E R`. The branch factor is given by the nature of the type
  `A`. These visible events encode interactions with the external environment,
  e.g. message sends or object creation.

Working with the type above is not easy due to productivity issues. We can
instead define a more convenient version using a structure functor.

## `ITreeF` Structure Functor

The structure functor Agda@ITreeF separates one layer of computation from the rest
of the tree. This allows us to define the coinductive type in a way that Agda
can verify as productive.

```agda
data ITreeF (E : EventSig) (R : Set) (X : Set) : Set where
  retF : R → ITreeF E R X
  tauF : X → ITreeF E R X
  visF : (A : Set) → E A → (A → X) → ITreeF E R X
```

The parameter `X` represents the type of the subtrees. When we tie the
recursive knot, `X` will be replaced with `ITree E R`.

## Productive ITrees

An interaction tree over event signature `E` with return type `R` represents a
potentially infinite computation that can interact with the environment via
events.

```agda
record ITree (E : EventSig) (R : Set) : Set where
  coinductive
  constructor delay
  field
    observe : ITreeF E R (ITree E R)

open ITree public
```

The `delay` constructor is crucial for productivity. The first definition
is non-productive because Agda fails to see an observable result if the
tree is an infinite sequence of `tau`s, for example. The `delay` constructor forces one
observable step, making the definition productive.

<!--
We can write this as:

$$\mathsf{ITree}(E,R) \equiv \mathsf{delay}(\mathsf{ITreeF}(E,R)(\mathsf{ITree}(E,R))).$$
-->

To work with this datatype, we define convenient constructors as functions via
copatterns. For Agda@ret, Agda@tau, and Agda@vis the type is the same as in the first
definition Agda@ITree₁.

```agda
module _ {E : EventSig} {R : Set} where
```

- Return a value (terminate computation)

  ```agda
  ret : R → ITree E R
  observe (ret r) = retF r
  ```

- Silent step (internal computation)

  ```agda
  tau : ITree E R → ITree E R
  observe (tau t) = tauF t
  ```

- Visible event (interact with the environment)

  ```agda
  vis : {A : Set} → E A → (A → ITree E R) → ITree E R
  observe (vis e k) = visF _ e k
  ```

- Trigger an event and return its response

```agda
trigger : {E : EventSig} {A : Set} → E A → ITree E A
trigger e = vis e ret
```

## Working with ITrees

Interaction trees form a monad. This allows us the compositional construction of
complex computations. The monadic structure lets us sequence effectful operations,
where each computation can depend on the results of previous ones.

```agda
module _ {E : EventSig}{R S : Set} where
```

- Bind operation. Here we take an interaction tree producing `R` and a
  continuation `k : R → ITree E S`, returns a new tree producing output of type
  `S`.

```agda
  _>>=_ : ITree E R → (R → ITree E S) → ITree E S
  observe (t >>= k) = bind-step (observe t) k
    where
      bind-step :
          ITreeF E R (ITree E R) → (R → ITree E S) →
          ITreeF E S (ITree E S)
      bind-step (retF r) k = observe (k r)
      bind-step (tauF x) k = tauF (x >>= k)
      bind-step (visF A e f) k = visF A e (λ a → f a >>= k)
```

- Map operation

```agda
  _<$>_ : (R → S) → ITree E R → ITree E S
  f <$> t = t >>= (ret ∘ f)
```

```agda
  infixl 1 _>>=_
  infixr 4 _<$>_
```

### Agda Do-Notation Support

Agda supports do-notation for monads. We instantiate the `RawMonad` record (defined in BasicTypes) for `ITree` to enable clean do-notation syntax:

```agda
ITree-Monad : {E : EventSig} → RawMonad (ITree E)
ITree-Monad {E} = record
  { return = ret {E = E}
  ; _>>=_  = _>>=_ {E = E}
  }
```

With this instance, do-notation is now available anywhere `ITree-Monad` is opened. For example:

```text
-- Instead of: trigger e₁ >>= λ x → trigger e₂ >>= λ y → ret (x , y)
-- We can write:
do
  x ← trigger e₁
  y ← trigger e₂
  return (x , y)
```

This notation is particularly useful for writing complex AVM programs with multiple sequential operations.

## Interpreting ITrees

Interpretation gives meaning to interaction trees by handling events in specific
ways. A handler transforms events from one signature to another.

```agda
Handler : EventSig → EventSig → Set₁
Handler E F = {A : Set} → E A → ITree F A
```

Handlers are also called _implemenations maps_[@LesaniEtAl-PAPL-2022-C4].

```agda
Impl : EventSig → EventSig → Set₁
Impl = Handler
```

Handling one event in system `E` may require performing multiple events in
system `F`.

```agda
{-# TERMINATING #-}
interp : {E F : EventSig} {R : Set} →
      Handler E F → -- application-specific
      ITree E R →   -- program tree
      -------------------------------
      ITree F R
observe (interp h t) = interp-step h (observe t)
  where
    -- we unfold observations
    interp-step : {E F : EventSig} {R : Set} →
                  Handler E F → ITreeF E R (ITree E R) →
                  ITreeF F R (ITree F R)
    interp-step h (retF r) = retF r
    interp-step h (tauF x) = tauF (interp h x)
    interp-step h (visF A e k) =
      observe (h e >>= λ a → interp h (k a))
```

The `interp-step` helper function looks at what the tree is doing _right now_
(is it a better way to say this?) It handles three cases:

- `retF r` - Tree returned a value → Keep it as-is
- `tauF x` - Tree is doing internal computation (e.g. verifying keys or simply
  doing nothing) → Keep going, interpret the rest
- `visF A e k` - Tree is requesting an effect `e` → This is where translation
  actually happens. Run the handler `h` on event `e` to get a new tree in
  system `F`. Continue by interpreting the continuation `k` with the result.

!!! info "Handlers and implementation decisions"

    Handlers are relevant for expressing compilation decisions. Over the same
    interaction tree, the same program instructions, one could be interested in
    obtaining different outputs. For example, run one handler for executing the
    program off/on-chain, another for logging, another to give operational
    semantics.

## Observational Equivalence of `ITree`s

Two interaction trees are equivalent if they produce the same observable
behavior. We define this via _weak bisimulation_. That is, weak bisimulation
allows equating interaction trees that differ _only_ in the number of `Tau`
(delay/internal step) constructors before observable events (like `Vis` nodes
for I/O). To this end, internal computations can be ignored, like loop
unrolling, inlining, or another kind of optimization.

```agda
data _≈_ {E : EventSig} {R : Set}
    : ITree E R → ITree E R → Set where
  ret-eq : (r : R) → ret r ≈ ret r

  tau-left : {t₁ t₂ : ITree E R} → t₁ ≈ t₂ → tau t₁ ≈ t₂
  tau-right : {t₁ t₂ : ITree E R} → t₁ ≈ t₂ → t₁ ≈ tau t₂

  vis-eq : {A : Set} {e : E A} {k₁ k₂ : A → ITree E R} →
           ((a : A) → k₁ a ≈ k₂ a) → vis e k₁ ≈ vis e k₂

infix 4 _≈_
```

```agda
postulate
  ≈-refl : {E : EventSig} {R : Set} (t : ITree E R) → t ≈ t
  ≈-sym : {E : EventSig} {R : Set} {t₁ t₂ : ITree E R} → t₁ ≈ t₂ → t₂ ≈ t₁
  ≈-trans : {E : EventSig} {R : Set} {t₁ t₂ t₃ : ITree E R} → t₁ ≈ t₂ → t₂ ≈ t₃ → t₁ ≈ t₃
```

The Agda@tau-left and Agda@tau-right rules allow us to ignore silent steps when
comparing trees. Two trees are equivalent if they perform the same visible
events in the same order, regardless of internal computational steps.
