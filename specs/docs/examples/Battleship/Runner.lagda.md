# Battleship Runner

This module sets up the interpreter and runs the Battleship game.

```agda
{-# OPTIONS --type-in-type --guardedness #-}
module examples.Battleship.Runner where

open import Background.BasicTypes
open import Background.InteractionTrees as IT hiding (_>>=_)
open import examples.Battleship.PlayerBoard as PB
open import examples.Battleship.Game as Game
open import examples.RunnerUtilities hiding (initialState)
open import examples.Common.Equality
open import examples.Common.StateInit PB.Val PB.ObjectId PB.MachineId PB.ControllerId PB.TxId PB.ObjectBehaviour
```

## Display Functions

Display helper functions convert Battleship-specific value types to human-readable
string representations, enabling trace output and result formatting.

```agda
showOid : PB.ObjectId → String
showOid n = showNat n

{-# TERMINATING #-}
showValAgda : PB.Val → String
showValAgda (PB.VCoord x y) = "(VCoord " ++ˢ showNat x ++ˢ " " ++ˢ showNat y ++ˢ ")"
showValAgda (PB.VShip x y len) = "(VShip " ++ˢ showNat x ++ˢ " " ++ˢ showNat y ++ˢ " len=" ++ˢ showNat len ++ˢ ")"

open import examples.Common.Display PB.Val PB.ObjectId PB.TxId PB.ControllerId PB.MachineId PB.ObjectBehaviour showValAgda showOid showNat (λ s → s) (λ s → s)
```

## Type Conversions and Equality

```agda
freshObjectId : ℕ → PB.ObjectId
freshObjectId n = n

freshTxId : ℕ → PB.TxId
freshTxId n = n

eqObjectId : PB.ObjectId → PB.ObjectId → Bool
eqObjectId = _==ℕ_

eqTxId : PB.TxId → PB.TxId → Bool
eqTxId = _==ℕ_

```

## Interpreter Setup

```agda
open import AVM.Interpreter PB.Val PB.ObjectId freshObjectId PB.MachineId PB.ControllerId PB.TxId freshTxId PB.ObjectBehaviour

eqControllerId : PB.ControllerId → PB.ControllerId → Bool
eqControllerId = eqString

interpretBehaviorName : String → PB.ObjectBehaviour
interpretBehaviorName s = tt

allObjectIds : State → List PB.ObjectId
allObjectIds st = State.observed st ++ map (λ { (oid , _ , _) → oid }) (State.creates st)

getBehavior : PB.ObjectBehaviour → AVMProgram (List PB.Val)
getBehavior _ = PB.boardBehavior
```

## Main Interpreter

```agda
mkControllerId : String → PB.ControllerId
mkControllerId s = s

module RunnerInterpreter where
  {-# TERMINATING #-}
  mutual
    interpretAVMProgramRec : ∀ {A} → AVMProgram A → State → AVMResult A
    interpretAVMProgramRec = interp.interpretAVMProgram
      where
        module interp = Interpreter eqObjectId eqTxId eqControllerId allObjectIds interpretBehaviorName getBehavior (isReachableController {PB.ControllerId}) (isReachableMachine {PB.MachineId}) mkControllerId interpretAVMProgramRec

    open Interpreter eqObjectId eqTxId eqControllerId allObjectIds interpretBehaviorName getBehavior (isReachableController {PB.ControllerId}) (isReachableMachine {PB.MachineId}) mkControllerId interpretAVMProgramRec public

open RunnerInterpreter public
```

## Initial State

```agda
initialState : State
initialState = mkInitialState "node1" 0 (PB.VCoord 0 0)
```

## Test Result Display

Test result display functions format example-specific output types for
presentation, including optional values and object identifier pairs.

```agda
showMaybeVal : Maybe PB.Val → String
showMaybeVal nothing = "nothing"
showMaybeVal (just v) = "just " ++ˢ showValAgda v

showPair : PB.ObjectId × PB.ObjectId → String
showPair (board1 , board2) = "(board1: " ++ˢ showOid board1 ++ˢ ", board2: " ++ˢ showOid board2 ++ˢ ")"
```

## Test Cases

```agda
-- Test 1: Single board - HIT
testHit : AVMProgram (Maybe PB.Val)
testHit =
  IT._>>=_ (trigger (obj-create "PlayerBoard" nothing)) λ board →
  IT._>>=_ (trigger (obj-call board (PB.VShip 0 0 3))) λ _ →
  trigger (obj-call board (PB.VCoord 0 1))

-- Test 2: Single board - MISS
testMiss : AVMProgram (Maybe PB.Val)
testMiss =
  IT._>>=_ (trigger (obj-create "PlayerBoard" nothing)) λ board →
  IT._>>=_ (trigger (obj-call board (PB.VShip 0 0 3))) λ _ →
  trigger (obj-call board (PB.VCoord 5 5))

-- Test 3: Full game using Game module
testFullGame : AVMProgram (PB.ObjectId × PB.ObjectId)
testFullGame = Game.playFullGame

-- Test 4: Game with rollback
testRollback : AVMProgram (PB.ObjectId × PB.ObjectId)
testRollback = Game.gameWithRollback

runExample : String → ∀ {A} → (A → String) → AVMProgram A → PrimIO ⊤
runExample title showA prog = do
  putStrLn ("=== " ++ˢ title ++ˢ " ===")
  putStrLn ""
  let result = interpretAVMProgram prog initialState
  putStrLn (showResult showA result)
  putStrLn ""

main : PrimIO ⊤
main = do
  putStrLn "=== Battleship Interactive Demo ==="
  putStrLn ""
  runExample "Test 1: Single Board - Attack HIT" showMaybeVal testHit
  runExample "Test 2: Single Board - Attack MISS" showMaybeVal testMiss
  runExample "Test 3: Full Game with Two Players" showPair testFullGame
  runExample "Test 4: Game with Transaction Rollback" showPair testRollback
  putStrLn "Done!"
```
