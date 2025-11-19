---
title: Stateless Object Examples
icon: material/package-variant-closed
tags:
  - AVM green paper
  - examples
  - stateless objects
  - denotational semantics
---

```agda
{-# OPTIONS --without-K --type-in-type --guardedness --exact-split #-}
-- (no extra warning flags)
module Background.StatelessObjects where

open import Background.BasicTypes
open import Background.InteractionTrees
```

## Overview

This module provides concrete examples demonstrating the AVM instruction set defined
in [AVM.Instruction](../AVM/Instruction.lagda.md). We instantiate the parametrized instruction
module with:

- A canonical S-expression type for values
- Object identifiers as pairs of node IDs and strings
- A simple oracle for generating fresh object identifiers

## Value Type Instantiation

We define a concrete value type for messages in our examples:

```agda
data Val : Set where
  VInt : ℕ → Val
  VBool : Bool → Val
  VString : String → Val
  VPair : Val → Val → Val
  VNil : Val
  VList : List Val → Val
```

## Object Identifier Instantiation

We define object identifiers as pairs of node identifiers and local names:

```agda
NodeId : Set
NodeId = String

ObjectId : Set
ObjectId = NodeId × String
```

## Fresh ObjectId Oracle

We provide a simple oracle that generates fresh object identifiers by converting
a natural number to a string and pairing it with a default node:

```agda
freshObjectId : ℕ → ObjectId
freshObjectId n = ("default-node" , nat-to-string n)

MachineId : Set
MachineId = String

ControllerId : Set
ControllerId = String

TxId : Set
TxId = String

freshTxId : ℕ → TxId
freshTxId n = nat-to-string n
```

This oracle ensures uniqueness by using the counter value directly. In a real
implementation, this would need to be replaced with a more sophisticated
mechanism that accounts for concurrent object creation and distributed systems.

## Module Instantiation

Now we instantiate the object model and instruction set with our concrete types.
Both modules are parameterized by the same core types:

```agda
-- First instantiate the Objects module with our concrete types
open import Background.Objects Val ObjectId MachineId ControllerId TxId
  using (Object; Input; Output; InputSequence; mkObjType)

-- Define Message as alias for Val (since Instruction's Message is hidden)
Message : Set
Message = Val

-- Then instantiate the Instruction module, passing Object from Objects
-- Hide Input/Output/Message to avoid name collision since Objects already exports them
open import AVM.Instruction Val ObjectId MachineId ControllerId TxId Object public
  hiding (Input; Output; Message; history)
```

## Helper Functions

We define helper functions for creating messages and validating responses:

```agda
withdraw : ℕ → Message
withdraw n = VList (VString "withdraw" ∷ VInt n ∷ [])

deposit : ℕ → Message
deposit n = VList (VString "deposit" ∷ VInt n ∷ [])

ok? : Val → Bool
ok? (VString "ok") = true
ok? (VString _) = false  -- Any other string
ok? (VInt _) = false
ok? (VBool _) = false
ok? (VPair _ _) = false
ok? VNil = false
ok? (VList _) = false
```

## Transfer Example

This example demonstrates an atomic transfer operation between two account objects.
We assume:

- An Account object type is defined
- Account objects support `withdraw` and `deposit` methods that modify a balance field
- Two Account objects are identified by `fromAcc` and `toAcc`

Given two object identifiers and an amount, we define a transfer function:

```agda
kudosTransfer : ObjectId → ObjectId → ℕ → AVMProgram Val
kudosTransfer fromAcc toAcc amount =
  trigger (Tx beginTx) >>= λ txId →
  trigger (Obj(call fromAcc (withdraw amount))) >>= λ mr₁ →
  handleWithdraw txId mr₁
  where
    abortWithMsg : TxId → String → AVMProgram Val
    abortWithMsg txId msg =
      trigger (Tx(abortTx txId)) >>= λ _ →
      ret (VString msg)

    commitWithResult : TxId → AVMProgram Val
    commitWithResult txId =
      trigger (Tx(commitTx txId)) >>= λ success →
      if success then
        ret (VString "transfer-complete")
      else
        ret (VString "commit-failed")

    handleDeposit : TxId → Maybe Val → AVMProgram Val
    handleDeposit txId nothing = abortWithMsg txId "deposit-call-failed"
    handleDeposit txId (just r₂) =
      if ok? r₂ then
        commitWithResult txId
      else
        abortWithMsg txId "deposit-failed"

    handleWithdraw : TxId → Maybe Val → AVMProgram Val
    handleWithdraw txId nothing = abortWithMsg txId "withdraw-call-failed"
    handleWithdraw txId (just r₁) =
      if ok? r₁ then
        (trigger (Obj(call toAcc (deposit amount))) >>= λ mr₂ →
         handleDeposit txId mr₂)
      else
        abortWithMsg txId "insufficient-funds"
```

The transfer operation is atomic: either both the withdrawal and deposit succeed
and are committed, or the transaction is aborted and no state changes occur.

## Object Creation and Initialization

Creating a new counter object and performing initial operations:

```agda
createCounter : Val → AVMProgram ObjectId
createCounter initialValue =
  trigger (Obj(createObj "counter")) >>= λ counterId →
  trigger (Obj(call counterId initialValue)) >>= λ _ →
  trigger (Obj(call counterId (VString "increment"))) >>= λ _ →
  ret counterId

-- Using the counter
counterExample : AVMProgram Val
counterExample =
  createCounter (VInt 0) >>= λ ctr →
  trigger (Obj(call ctr (VString "increment"))) >>= λ _ →
  trigger (Obj(call ctr (VString "increment"))) >>= λ _ →
  trigger (Obj(call ctr (VString "get"))) >>= λ mv₃ →
  handleResult mv₃
  where
    handleResult : Maybe Val → AVMProgram Val
    handleResult nothing = ret (VString "call-failed")
    handleResult (just v₃) = ret v₃  -- Returns VInt 3
```

The counter maintains its state across calls, demonstrating the stateful nature
of objects created through `createObj`. The initial value is now passed through
the first call rather than during object creation.

## Balance Update with Validation

Updating an account balance through message-passing with validation:

```agda
updateBalance : ObjectId → Val → AVMProgram Val
updateBalance account v = helper v
  where
    invalidMsg : Val
    invalidMsg = VString "invalid-value-type"

    helper : Val → AVMProgram Val
    helper (VInt n) = updateHelper n
      where
        handleUpdateResult : Maybe Val → AVMProgram Val
        handleUpdateResult nothing = ret (VString "update-call-failed")
        handleUpdateResult (just result) =
          if ok? result then
            ret (VString "balance-updated")
          else
            ret (VString "update-failed")

        updateHelper : ℕ → AVMProgram Val
        updateHelper n =
          if 0 ≤? n then
            -- Get current balance via message
            (trigger (Obj(call account (VString "get-balance"))) >>= λ mOldValue →
             -- Update via message
             trigger (Obj(call account (VList (VString "set-balance" ∷ VInt n ∷ [])))) >>= λ mResult →
             handleUpdateResult mResult)
          else
            ret (VString "invalid-balance")
    helper (VBool _) = ret invalidMsg
    helper (VString _) = ret invalidMsg
    helper (VPair _ _) = ret invalidMsg
    helper VNil = ret invalidMsg
    helper (VList _) = ret invalidMsg
```

Balance updates are validated before being applied through the object's message interface.

## Batch Transfer

Calling a multi-method that operates on multiple objects simultaneously:

```agda
transferBatch : List (ObjectId × ObjectId × ℕ) → AVMProgram Val
transferBatch transfers =
  processTransfers transfers
  where
    processTransfers : List (ObjectId × ObjectId × ℕ) → AVMProgram Val
    processTransfers [] = ret (VString "batch-complete")
    processTransfers ((from , to , amount) ∷ rest) =
      kudosTransfer from to amount >>= λ result →
      (if ok? result then
        processTransfers rest
      else
        ret (VString "batch-failed"))
```

Batch operations can be made atomic by wrapping them in appropriate transaction
boundaries.

## Object Destruction with Cleanup

Destroying an object after transferring its remaining balance:

```agda
closeAccount : ObjectId → ObjectId → AVMProgram Val
closeAccount accountToClose beneficiary =
  -- Get final balance
  trigger (Obj(call accountToClose (VString "get-balance"))) >>= λ mBalance →
  handleBalance mBalance
  where
    closeWithBalance : Val → AVMProgram Val
    closeWithBalance (VInt n) =
      if 0 <? n then
        -- Transfer remaining funds
        (trigger (Obj(call accountToClose (withdraw n))) >>= λ _ →
         trigger (Obj(call beneficiary (deposit n))) >>= λ _ →
         -- Destroy the account
         trigger (Obj(destroyObj accountToClose)) >>= λ _ →
         ret (VString "account-closed"))
      else
        -- No balance, just destroy
        (trigger (Obj(destroyObj accountToClose)) >>= λ _ →
         ret (VString "account-closed-empty"))
    closeWithBalance (VString _) = ret (VString "invalid-balance")
    closeWithBalance (VBool _) = ret (VString "invalid-balance")
    closeWithBalance (VPair _ _) = ret (VString "invalid-balance")
    closeWithBalance VNil = ret (VString "invalid-balance")
    closeWithBalance (VList _) = ret (VString "invalid-balance")

    handleBalance : Maybe Val → AVMProgram Val
    handleBalance nothing = ret (VString "get-balance-failed")
    handleBalance (just balance) = closeWithBalance balance
```

This ensures no funds are lost when an account is closed - the remaining balance
is transferred before destruction using `destroyObj`.

## Stateless Object Examples

This section demonstrates stateless objects as pure functions from input histories to outputs, following the pattern φ : I\* ⇀ O from the AVM green paper. These examples illustrate the relationship between the mathematical specification (denotational semantics) and executable implementation (operational semantics).

Each example follows the same structure:

1. Define message types (Input)
2. Define a replay function that derives state from history
3. Define φ as a total function History → Maybe Output
4. Define step as the operational form: (History × Input) → Maybe (Output × History)

### Counter Object (Stateless)

The counter maintains a count by processing increment and decrement messages. The state (current count) is derived entirely from the input history.

Input messages:

```agda
data CounterMsg : Set where
  inc : CounterMsg
  dec : CounterMsg
```

The replay function derives the current count from history:

```agda
replay-counter : List CounterMsg → ℕ
replay-counter [] = 0
replay-counter (inc ∷ ms) = suc (replay-counter ms)
replay-counter (dec ∷ ms) with replay-counter ms
... | zero = zero  -- Cannot go below zero
... | suc n = n
```

The φ function maps complete histories to observable outputs:

```agda
φ-counter : List CounterMsg → Maybe ℕ
φ-counter history = just (replay-counter history)
```

The operational step function processes one input at a time:

```agda
step-counter : List CounterMsg → CounterMsg → Maybe (ℕ × List CounterMsg)
step-counter history msg with φ-counter (history ++ (msg ∷ []))
... | just count = just (count , history ++ (msg ∷ []))
... | nothing = nothing
```

The counter is always defined (total function), demonstrating that not all objects need partiality.

### Key-Value Store Object (Stateless)

The KV store supports put, delete, and get operations. The store state is derived from the event history.

Input messages:

```agda
data Key : Set where
  key : String → Key

data KVMsg : Set where
  put : Key → Val → KVMsg
  del : Key → KVMsg
  get : Key → KVMsg
```

Event type for state reconstruction:

```agda
data KVEvent : Set where
  put-event : Key → Val → KVEvent
  del-event : Key → KVEvent

-- Store represented as association list
KVStore : Set
KVStore = List (Key × Val)
```

Helper functions:

```agda
postulate
  _≟key_ : Key → Key → Bool
  lookup-kv : Key → KVStore → Maybe Val
  last : {A : Set} → List A → Maybe A
  filter-map : {A B : Set} → (A → Maybe B) → List A → List B
```

The replay function derives the current store from history:

```agda

apply-kv-event : KVStore → KVEvent → KVStore
apply-kv-event store (put-event k v) = (k , v) ∷ filter (λ { (k' , _) → not (k ≟key k') }) store
apply-kv-event store (del-event k) = filter (λ { (k' , _) → not (k ≟key k') }) store

replay-kv : List KVEvent → KVStore
replay-kv [] = []
replay-kv (e ∷ es) = apply-kv-event (replay-kv es) e
```

The φ function maps histories to outputs (partiality arises from get on missing keys):

```agda
-- Convert messages to events and outputs
msg-to-event : KVMsg → Maybe KVEvent
msg-to-event (put k v) = just (put-event k v)
msg-to-event (del k) = just (del-event k)
msg-to-event (get k) = nothing  -- get produces no event

φ-kv-aux : List KVMsg → Maybe KVMsg → Maybe Val
φ-kv-aux history nothing = nothing
φ-kv-aux history (just (put k v)) = just (VString "ok")
φ-kv-aux history (just (del k)) = just (VString "ok")
φ-kv-aux history (just (get k)) =
  let events = filter-map msg-to-event history in
  let store = replay-kv events in
  lookup-kv k store

φ-kv : List KVMsg → Maybe Val
φ-kv [] = nothing  -- Empty history: no output
φ-kv (x ∷ xs) = φ-kv-aux (x ∷ xs) (last (x ∷ xs))
```

The operational step function:

```agda
step-kv : List KVMsg → KVMsg → Maybe (Val × List KVMsg)
step-kv history msg with φ-kv (history ++ (msg ∷ []))
... | just output = just (output , history ++ (msg ∷ []))
... | nothing = nothing  -- get on missing key
```

The KV store is partial: get operations on missing keys are undefined.

### Bank Account Object (Stateless)

The bank account supports deposit, withdraw, and balance query operations. Withdrawals that would result in negative balance are undefined.

Input messages:

```agda
data AccountMsg : Set where
  depositMsg : ℕ → AccountMsg
  withdrawMsg : ℕ → AccountMsg
  balanceMsg : AccountMsg
```

Helper functions:

```agda
postulate
  _≥ℕ_ : ℕ → ℕ → Bool
  _-ℕ_ : ℕ → ℕ → ℕ
  init : {A : Set} → List A → List A  -- All but last
```

The replay function derives current balance from history:

```agda
replay-account : List AccountMsg → ℕ
replay-account [] = 0  -- Initial balance
replay-account (depositMsg x ∷ ms) =
  let prev-bal = replay-account ms in
  x +ℕ prev-bal
replay-account (withdrawMsg x ∷ ms) =
  let prev-bal = replay-account ms in
  if prev-bal ≥ℕ x then prev-bal -ℕ x else prev-bal  -- Reject if insufficient
replay-account (balanceMsg ∷ ms) = replay-account ms  -- Query doesn't change state
```

The φ function maps histories to outputs:

```agda
data AccountOutput : Set where
  ok : AccountOutput
  error-insufficient : AccountOutput
  balance-val : ℕ → AccountOutput

φ-account-aux : List AccountMsg → Maybe AccountMsg → Maybe AccountOutput
φ-account-aux history nothing = nothing
φ-account-aux history (just (depositMsg x)) = just ok
φ-account-aux history (just (withdrawMsg x)) =
  let prev-history = init history in
  let bal = replay-account prev-history in
  if bal ≥ℕ x then just ok else nothing  -- undefined if insufficient
φ-account-aux history (just balanceMsg) = just (balance-val (replay-account history))

φ-account : List AccountMsg → Maybe AccountOutput
φ-account [] = nothing  -- No output for empty history
φ-account (x ∷ xs) = φ-account-aux (x ∷ xs) (last (x ∷ xs))
```

The operational step function:

```agda
step-account : List AccountMsg → AccountMsg → Maybe (AccountOutput × List AccountMsg)
step-account history msg with φ-account (history ++ (msg ∷ []))
... | just output = just (output , history ++ (msg ∷ []))
... | nothing = nothing  -- withdrawal with insufficient funds
```
