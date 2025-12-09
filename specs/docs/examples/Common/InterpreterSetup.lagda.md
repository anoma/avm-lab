# Common Interpreter Setup

This module provides the standard mutual recursion pattern for instantiating
the AVM interpreter with proper termination handling. The module is
parameterized by type definitions, fresh identifier generators, equality
predicates, reachability checks, and behavior interpretation functions required
by the interpreter module.

```agda
{-# OPTIONS --without-K --type-in-type --guardedness #-}

open import Background.BasicTypes

module examples.Common.InterpreterSetup
  (Val : Set)
  (ObjectId : Set)
  (MachineId : Set)
  (ControllerId : Set)
  (TxId : Set)
  (ObjectBehaviour : Set)
  (freshObjectId : ℕ → ObjectId)
  (freshTxId : ℕ → TxId)
  (eqObjectId : ObjectId → ObjectId → Bool)
  (eqTxId : TxId → TxId → Bool)
  (eqControllerId : ControllerId → ControllerId → Bool)
  (allObjectIds : {State : Set} → State → List ObjectId)
  (interpretBehaviorName : String → ObjectBehaviour)
  (isReachableController : ControllerId → Bool)
  (isReachableMachine : MachineId → Bool)
  (mkControllerId : String → ControllerId)
  where

open import AVM.Interpreter Val ObjectId freshObjectId MachineId ControllerId TxId freshTxId ObjectBehaviour
```

## Interpreter Instantiation

Interpreter instantiation employs a mutual recursion pattern to satisfy Agda's
termination checker while enabling the interpreter to recursively interpret
nested AVM programs. The pattern creates a forward reference to the complete
interpreter function, which is then passed as a parameter to the interpreter
module itself, establishing the recursive knot required for program
interpretation.

```agda
module RunnerInterpreter
  (getBehavior : ObjectBehaviour → AVMProgram (List Val))
  where

  {-# TERMINATING #-}
  mutual
    -- Forward reference to the full interpreter
    interpretAVMProgramRec : ∀ {A} → AVMProgram A → State → AVMResult A
    interpretAVMProgramRec = interp.interpretAVMProgram
      where
        module interp = Interpreter
          eqObjectId eqTxId eqControllerId allObjectIds
          interpretBehaviorName getBehavior
          isReachableController isReachableMachine mkControllerId
          interpretAVMProgramRec

    -- Now open the interpreter with the recursive reference
    open Interpreter
      eqObjectId eqTxId eqControllerId allObjectIds
      interpretBehaviorName getBehavior
      isReachableController isReachableMachine mkControllerId
      interpretAVMProgramRec public
```
