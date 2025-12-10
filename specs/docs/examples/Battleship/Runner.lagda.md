# Battleship Runner

This module provides the execution harness for the Battleship game.

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

## Display and Conversion Functions

```agda
showOid : PB.ObjectId → String
showOid n = showNat n

{-# TERMINATING #-}
showValAgda : PB.Val → String
showValAgda (PB.VCoord x y) = "(VCoord " ++ˢ showNat x ++ˢ " " ++ˢ showNat y ++ˢ ")"
showValAgda (PB.VShip x y len) = "(VShip " ++ˢ showNat x ++ˢ " " ++ˢ showNat y ++ˢ " len=" ++ˢ showNat len ++ˢ ")"

open import examples.Common.Display PB.Val PB.ObjectId PB.TxId PB.ControllerId PB.MachineId PB.ObjectBehaviour showValAgda showOid showNat (λ s → s) (λ s → s)

freshObjectId : ℕ → PB.ObjectId
freshObjectId n = n

freshTxId : ℕ → PB.TxId
freshTxId n = n

eqObjectId : PB.ObjectId → PB.ObjectId → Bool
eqObjectId = _==ℕ_

eqTxId : PB.TxId → PB.TxId → Bool
eqTxId = _==ℕ_
```

## AVM Interpreter Configuration

```agda
open import AVM.Interpreter PB.Val PB.ObjectId freshObjectId PB.MachineId PB.ControllerId PB.TxId freshTxId PB.ObjectBehaviour

eqControllerId : PB.ControllerId → PB.ControllerId → Bool
eqControllerId = eqString

mkControllerId : String → PB.ControllerId
mkControllerId s = s

interpretBehaviorName : String → PB.ObjectBehaviour
interpretBehaviorName _ = tt

allObjectIds : State → List PB.ObjectId
allObjectIds st = State.observed st ++ map (λ { (oid , _ , _) → oid }) (State.creates st)

getBehavior : PB.ObjectBehaviour → AVMProgram (List PB.Val)
getBehavior _ = PB.boardBehavior

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

## Initial State and Display Helpers

```agda
initialState : State
initialState = mkInitialState "node1" 0 (PB.VCoord 0 0)

showMaybeVal : Maybe PB.Val → String
showMaybeVal nothing = "nothing"
showMaybeVal (just v) = "just " ++ˢ showValAgda v

showGameState : Game.GameState → String
showGameState (board1 , board2) = "(board1: " ++ˢ showOid board1 ++ˢ ", board2: " ++ˢ showOid board2 ++ˢ ")"
```

## Test Cases and Main Execution

```agda
testHit : AVMProgram (Maybe PB.Val)
testHit =
  IT._>>=_ (trigger (obj-create "PlayerBoard" nothing)) λ board →
  IT._>>=_ (trigger (obj-call board (PB.VShip 0 0 3))) λ _ →
  trigger (obj-call board (PB.VCoord 0 1))

testMiss : AVMProgram (Maybe PB.Val)
testMiss =
  IT._>>=_ (trigger (obj-create "PlayerBoard" nothing)) λ board →
  IT._>>=_ (trigger (obj-call board (PB.VShip 0 0 3))) λ _ →
  trigger (obj-call board (PB.VCoord 5 5))

runExample : String → ∀ {A} → (A → String) → AVMProgram A → PrimIO ⊤
runExample title showA prog = do
  putStrLn ("=== " ++ˢ title ++ˢ " ===")
  putStrLn ""
  let result = interpretAVMProgram prog initialState
  putStrLn (showResult showA result)
  putStrLn ""

main : PrimIO ⊤
main = do
  putStrLn "=== Battleship Game Demo ==="
  putStrLn ""
  runExample "Single Board - Attack HIT" showMaybeVal testHit
  runExample "Single Board - Attack MISS" showMaybeVal testMiss
  runExample "Full Game with Two Players" showGameState Game.playFullGame
  runExample "Game with Player Types" showGameState Game.playGameWithPlayerType
  runExample "Game with Transaction Rollback" showGameState Game.gameWithRollback
  putStrLn "Done!"
```
