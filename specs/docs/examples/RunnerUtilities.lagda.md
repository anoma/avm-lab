---
title: Runner Utilities
icon: material/tools
tags:
  - runner
  - utilities
  - examples
---

This module provides common utilities for running AVM examples, including IO primitives, state management, and interpreter infrastructure.

```agda
{-# OPTIONS --type-in-type --guardedness #-}
module examples.RunnerUtilities where

open import Background.BasicTypes
open import Background.InteractionTrees hiding (_>>=_)
```

## IO Primitives

```agda
postulate
  PrimIO : Set → Set
  putStrLn : String → PrimIO ⊤
  return : {A : Set} → A → PrimIO A
  _>>=_ : {A B : Set} → PrimIO A → (A → PrimIO B) → PrimIO B

{-# BUILTIN IO PrimIO #-}
{-# COMPILE GHC PrimIO = type IO #-}
{-# COMPILE GHC putStrLn = putStrLn . Data.Text.unpack #-}
{-# COMPILE GHC return = \_ -> return #-}
{-# COMPILE GHC _>>=_ = \_ _ -> (>>=) #-}

_>>_ : {A B : Set} → PrimIO A → PrimIO B → PrimIO B
m₁ >> m₂ = m₁ >>= λ _ → m₂
```

## Display Utilities

```agda
showNat : ℕ → String
showNat n = nat-to-string n

indent : ℕ → String
indent zero = ""
indent (suc n) = "  " ++ˢ indent n
```

## Object ID Utilities

```agda
freshId : String → ℕ → (String × String)
freshId nodeName n = (nodeName , "obj-" ++ˢ showNat n)

showOID : {NodeId : Set} → (NodeId × String) → String
showOID (_ , s) = s

eqOID : {NodeId : Set} → (NodeId × String) → (NodeId × String) → Bool
eqOID (n1 , s1) (n2 , s2) with s1 ≟-string s2
... | just _ = true
... | nothing = false
```

## State Management

The runner state tracks object creation and stores object metadata.

```agda
record RunnerState (ObjMeta : Set) : Set where
  constructor mkState
  field
    nextId : ℕ
    objects : List ((String × String) × ObjMeta)

initialState : {ObjMeta : Set} → RunnerState ObjMeta
initialState = mkState 0 []
```

Execution environment for tracking current context:

```agda
record Env (ObjectId : Set) : Set where
  constructor mkEnv
  field
    currentOid : ObjectId
    callDepth : ℕ
```

## Lookup Utilities

```agda
lookupObj : {ObjMeta : Set} → (String × String) → List ((String × String) × ObjMeta) → Maybe ObjMeta
lookupObj oid [] = nothing
lookupObj oid ((k , v) ∷ xs) with eqOID oid k
... | true = just v
... | false = lookupObj oid xs
```

## Magic Postulate

For unsupported operations:

```agda
postulate magic : {A : Set} → A
```
