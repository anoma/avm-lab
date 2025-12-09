---
title: Token Swap Intent Matching
icon: material/swap-horizontal
tags:
  - examples
  - constraints
  - nondeterminism
  - intent matching
---

# Token Swap Intent Matching

Demonstrates nondeterminism instructions with commit-time validation for
multi-party intent matching.

## Problem Description

Two parties want to swap tokens. Each has preferences over acceptable exchange
rates and constraints on acceptable trades. The system must find parameters that
satisfy both parties' requirements simultaneously.

## Intent Specification

**Party A's Intent:**
- Offers: 100 TokenX
- Wants: TokenY
- Acceptable exchange rates: 0.8, 1.0, 1.2 (TokenY per TokenX)
- Constraint: Must receive at least 80 TokenY

**Party B's Intent:**
- Offers: 150 TokenY
- Wants: TokenX
- Acceptable exchange rates: 0.9, 1.0, 1.1 (TokenY per TokenX)
- Constraint: Must receive at least 90 TokenX

**Matching Constraint:**
- Both parties' requirements must be satisfied
- Exchange rate must be acceptable to both
- Amounts must balance at the agreed rate

## Implementation

```agda
{-# OPTIONS --without-K --type-in-type --guardedness #-}

module examples.Constraints.TokenSwap
  (Val : Set)
  (ObjectId : Set)
  (MachineId : Set)
  (ControllerId : Set)
  (TxId : Set)
  (ObjectBehaviour : Set)
  where

open import Background.InteractionTrees
open import Background.BasicTypes
open import AVM.Instruction Val ObjectId MachineId ControllerId TxId ObjectBehaviour
open import AVM.Context Val ObjectId MachineId ControllerId TxId ObjectBehaviour

-- Token types and amounts
record TokenAmount : Set where
  field
    token : String
    amount : ℕ

-- Exchange rate (rational number approximated as ℕ × ℕ)
record ExchangeRate : Set where
  field
    numerator : ℕ
    denominator : ℕ

-- Swap intent parameters
record SwapIntent : Set where
  field
    offer : TokenAmount
    want : String            -- Token type wanted
    acceptableRates : List ExchangeRate
    minReceive : ℕ          -- Minimum amount willing to receive

-- Helper: check if rate satisfies constraints
rateIsAcceptable : ExchangeRate → List ExchangeRate → Bool
rateIsAcceptable rate acceptable = elemRate rate acceptable
  where
    elemRate : ExchangeRate → List ExchangeRate → Bool
    elemRate _ [] = false
    elemRate r (x ∷ xs) = ((ExchangeRate.numerator r == ExchangeRate.numerator x) ∧
                      (ExchangeRate.denominator r == ExchangeRate.denominator x))
                      ∨ elemRate r xs

-- Helper: compute amount received at given rate
computeReceived : ℕ → ExchangeRate → ℕ
computeReceived offered rate = {!!}  -- (offered * numerator) / denominator

-- Create swap intent using nondeterminism instructions
createSwapIntent : SwapIntent → AVMProgram Bool
createSwapIntent intent =
  trigger (tx-begin nothing) >>= λ txId →
  -- Choose exchange rate nondeterministically (commit-time choice)
  -- The actual selection is DEFERRED until commit
  let rateOptions = SwapIntent.acceptableRates intent in
  trigger (Nondet (choose (map toVal rateOptions))) >>= λ rate →
  -- Accumulate constraints (validated at commit)
  -- Each require accumulates, none evaluated yet
  trigger (Nondet (require (rateIsAcceptable (fromVal rate) rateOptions))) >>= λ _ →
  let offered = TokenAmount.amount (SwapIntent.offer intent) in
  let received = computeReceived offered (fromVal rate) in
  trigger (Nondet (require {!!})) >>= λ _ →  -- received ≥ minReceive
  -- Commit: ALL accumulated requirements validated atomically
  trigger (tx-commit txId)

  where
    toVal : ExchangeRate → Val
    toVal r = {!!}  -- Convert ExchangeRate to Val (implementation-specific)

    fromVal : Val → ExchangeRate
    fromVal v = {!!}  -- Convert Val to ExchangeRate (implementation-specific)

-- Match two swap intents
matchSwapIntents : SwapIntent → SwapIntent → AVMProgram (Maybe ExchangeRate)
matchSwapIntents intentA intentB =
  trigger (tx-begin nothing) >>= λ txId →
  -- Find overlapping acceptable rates
  let commonRates = intersection
        (SwapIntent.acceptableRates intentA)
        (SwapIntent.acceptableRates intentB) in
  if null commonRates
    then (trigger (tx-abort txId) >>= λ _ →
      ret nothing)
    else (-- Choose from common rates (commit-time choice)
      trigger (Nondet (choose (map toVal commonRates))) >>= λ rate →
      -- Validate Party A's constraints
      let offeredA = TokenAmount.amount (SwapIntent.offer intentA) in
      let receivedA = computeReceived offeredA (fromVal rate) in
      trigger (Nondet (require {!!})) >>= λ _ →  -- receivedA ≥ minReceive intentA
      -- Validate Party B's constraints
      let offeredB = TokenAmount.amount (SwapIntent.offer intentB) in
      let receivedB = computeReceived offeredB (fromVal rate) in
      trigger (Nondet (require {!!})) >>= λ _ →  -- receivedB ≥ minReceive intentB
      -- Validate balance: A gives X, B gives Y, amounts must match at rate
      let amountYtoA = computeReceived offeredA (fromVal rate) in
      let amountXtoB = computeReceived offeredB (invertRate (fromVal rate)) in
      trigger (Nondet (require {!!})) >>= λ _ →  -- offeredA == amountXtoB
      trigger (Nondet (require {!!})) >>= λ _ →  -- offeredB == amountYtoA
      -- Atomic commit: validates ALL requirements simultaneously
      trigger (tx-commit txId) >>= λ success →
      if success
        then ret (just (fromVal rate))
        else (trigger (tx-abort txId) >>= λ _ →
          ret nothing))

  where
    toVal : ExchangeRate → Val
    toVal r = {!!}

    fromVal : Val → ExchangeRate
    fromVal v = {!!}

    invertRate : ExchangeRate → ExchangeRate
    invertRate r = record
      { numerator = ExchangeRate.denominator r
      ; denominator = ExchangeRate.numerator r }

    intersection : List ExchangeRate → List ExchangeRate → List ExchangeRate
    intersection xs ys = filterRates (λ x → elemRate x ys) xs
      where
        elemRate : ExchangeRate → List ExchangeRate → Bool
        elemRate r [] = false
        elemRate r (y ∷ ys) = ((ExchangeRate.numerator r == ExchangeRate.numerator y) ∧
                           (ExchangeRate.denominator r == ExchangeRate.denominator y))
                          ∨ elemRate r ys

        filterRates : {A : Set} → (A → Bool) → List A → List A
        filterRates p [] = []
        filterRates p (x ∷ xs) = if p x then (x ∷ filterRates p xs) else filterRates p xs
```

## Expected Execution Trace

### Single Intent Creation

```text
[TxBegin tx:1]
[ChoiceRecorded options:[0.8, 1.0, 1.2]]  -- Choice DEFERRED (not selected yet)
[RequirementAccumulated: rateIsAcceptable]
[RequirementAccumulated: minReceiveConstraint]
[TxCommit tx:1]
  [ValidatingRequirements...]              -- NOW choices are resolved
  [Solver selects: rate=1.0]               -- Commit-time selection
  [Checking rateIsAcceptable(1.0): true]
  [Checking minReceive(100): true]
  [AllRequirementsSatisfied]
[TxCommitted tx:1]
[Result: true]
```

### Intent Matching

```text
[TxBegin tx:2]
[ChoiceRecorded options:[1.0]]            -- Intersection of acceptable rates
[RequirementAccumulated: Party A minReceive >= 80]
[RequirementAccumulated: Party B minReceive >= 90]
[RequirementAccumulated: balance A→B]
[RequirementAccumulated: balance B→A]
[TxCommit tx:2]
  [ValidatingRequirements...]
  [Solver selects: rate=1.0]              -- Satisfies both parties
  [Checking Party A constraints...]
    [100 TokenX * 1.0 = 100 TokenY]
    [100 >= 80: true]
  [Checking Party B constraints...]
    [150 TokenY / 1.0 = 150 TokenX]
    [But only 100 TokenX offered!]
    [Constraint violation detected]
  [RequirementsFailed]
[TxAborted tx:2]
[Result: nothing]
```

If constraints are satisfiable:

```text
[TxBegin tx:3]
[ChoiceRecorded options:[1.0]]
... (constraints accumulate) ...
[TxCommit tx:3]
  [ValidatingRequirements...]
  [Solver selects: rate=1.0]
  [All constraints satisfied]
  [AllRequirementsSatisfied]
[TxCommitted tx:3]
[Result: just 1.0]
```

## Key Observations

### Commit-Time Choice Semantics
- `choose` **records** options but doesn't select immediately
- Actual selection is **deferred** until `commitTx` executes
- This enables solver to consider all constraints simultaneously

### Constraint Accumulation
- `require` **accumulates** constraints during execution
- No constraint is evaluated until commit time
- Allows composing constraints from multiple sources

### Atomic Validation
- All accumulated `require` constraints checked **atomically** at `commitTx`
- If any constraint fails, entire transaction aborts
- Enables multi-party composition: each party adds constraints

### Multi-Party Composition
- Different parties can execute in same transaction
- Each adds their constraints via `require`
- Commit validates all parties' constraints together
- Natural for intent matching where multiple parties must agree

## Comparison with FD Layer

This NonDet approach differs fundamentally from the FD layer:

| Aspect | NonDet (this example) | FD (alternative) |
|--------|-----------------------|------------------|
| Choice timing | Deferred until `commitTx` | Immediate when `label` executes |
| Constraint check | Atomic at commit | Incremental after each `post` |
| Use case | Multi-party intent matching | Single-agent search/solving |
| Composition | Natural (constraints accumulate) | Difficult (requires coordination) |

For token swap, the NonDet approach is natural because:
- Multiple parties contribute constraints
- Need atomic validation of all requirements
- Choice should consider all constraints simultaneously
- Multi-party coordination problem (not single-agent search)

## Implementation Note

This example includes type holes (`{!!}`) for Val conversion functions, as the
concrete Val representation is implementation-specific. In a full implementation,
these would convert between ExchangeRate and whatever Val type is used.
