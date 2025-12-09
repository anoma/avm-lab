# Common State Initialization

This module provides standard state initialization functions for constructing
fresh AVM execution contexts. The module is parameterized by concrete type
definitions for values, object identifiers, machine identifiers, controller
identifiers, transaction identifiers, and object behaviors, enabling reuse
across diverse example implementations.

```agda
{-# OPTIONS --without-K --type-in-type --guardedness #-}

open import Background.BasicTypes

module examples.Common.StateInit
  (Val : Set)
  (ObjectId : Set)
  (MachineId : Set)
  (ControllerId : Set)
  (TxId : Set)
  (ObjectBehaviour : Set)
  where

open import AVM.Context Val ObjectId MachineId ControllerId TxId ObjectBehaviour
```

## Store Initialization

Empty store construction produces an initial object store containing no objects
or metadata entries. Both the object store and metadata store are represented
as functions mapping all object identifiers to `nothing`.

```agda
emptyStore : Store
emptyStore = mkStore (λ _ → nothing) (λ _ → nothing)
```

## Pure Functions Registry

Empty pure function registry construction produces an initial function registry
containing no pure function definitions. The registry maps all function names
to `nothing`, indicating no functions are available.

```agda
noPureFunctions : PureFunctions
noPureFunctions = λ _ → nothing
```

## Initial State Constructor

Initial state construction produces a fresh execution state with empty stores,
no active transaction, and specified execution context parameters. The
constructor accepts a machine identifier indicating the physical node, a self
object identifier for the execution context, and an initial input value for
the computation.

```agda
mkInitialState : MachineId → ObjectId → Val → State
mkInitialState machineId self input = record
  { machineId = machineId
  ; store = emptyStore
  ; pureFunctions = noPureFunctions
  ; txLog = []
  ; creates = []
  ; destroys = []
  ; observed = []
  ; pendingTransfers = []
  ; tx = nothing
  ; txController = nothing
  ; self = self
  ; input = input
  ; sender = nothing
  ; traceMode = false
  ; eventCounter = 0
  }
```
