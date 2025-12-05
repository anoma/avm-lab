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

eqString : String → String → Bool
eqString s1 s2 = caseMaybe (s1 ≟-string s2) (λ _ → true) false

data ObjectBehaviourImpl : Set where
  namedBehaviour : String → ObjectBehaviourImpl
```

## Interpreter Setup

```agda
open import AVM.Interpreter PB.Val PB.ObjectId freshObjectId PB.MachineId PB.ControllerId PB.TxId freshTxId ObjectBehaviourImpl

isReachableController : PB.ControllerId → Bool
isReachableController _ = true

isReachableMachine : PB.MachineId → Bool
isReachableMachine _ = true

interpretBehaviorName : String → ObjectBehaviourImpl
interpretBehaviorName s = namedBehaviour s

allObjectIds : State → List PB.ObjectId
allObjectIds st = State.observed st ++ map (λ { (oid , _ , _) → oid }) (State.creates st)

getBehavior : ObjectBehaviourImpl → AVMProgram (List PB.Val)
getBehavior (namedBehaviour "PlayerBoard") = PB.boardBehavior
getBehavior (namedBehaviour _) = ret []
```

## Main Interpreter

```agda
module RunnerInterpreter where
  {-# TERMINATING #-}
  mutual
    interpretAVMProgramRec : ∀ {A} → AVMProgram A → State → AVMResult A
    interpretAVMProgramRec = interp.interpretAVMProgram
      where
        module interp = Interpreter eqObjectId eqTxId allObjectIds interpretBehaviorName getBehavior isReachableController isReachableMachine interpretAVMProgramRec

    open Interpreter eqObjectId eqTxId allObjectIds interpretBehaviorName getBehavior isReachableController isReachableMachine interpretAVMProgramRec public

open RunnerInterpreter public
```

## Initial State

```agda
emptyStore : Store
emptyStore = mkStore (λ _ → nothing) (λ _ → nothing)

noPureFunctions : PureFunctions
noPureFunctions = λ _ → nothing

initialState : State
initialState = record
  { machineId = "node1"
  ; controllerId = "root"
  ; store = emptyStore
  ; pureFunctions = noPureFunctions
  ; txLog = []
  ; creates = []
  ; destroys = []
  ; observed = []
  ; pendingTransfers = []
  ; tx = nothing
  ; self = 0
  ; input = PB.VCoord 0 0
  ; sender = nothing
  ; traceMode = false
  ; eventCounter = 0
  }
```

## Display Functions

```agda
{-# TERMINATING #-}
showValAgda : PB.Val → String
showValAgda (PB.VCoord x y) = "(VCoord " ++ˢ showNat x ++ˢ " " ++ˢ showNat y ++ˢ ")"
showValAgda (PB.VShip x y len) = "(VShip " ++ˢ showNat x ++ˢ " " ++ˢ showNat y ++ˢ " len=" ++ˢ showNat len ++ˢ ")"

showError : AVMError → String
showError _ = "execution-error"

showOid : PB.ObjectId → String
showOid n = showNat n

showEventType : EventType → String
showEventType (ObjectCreated oid behaviorName) = "ObjectCreated(" ++ˢ showOid oid ++ˢ ", \"" ++ˢ behaviorName ++ˢ "\")"
showEventType (ObjectDestroyed oid) = "ObjectDestroyed(" ++ˢ showOid oid ++ˢ ")"
showEventType (ObjectCalled oid inp mOut) =
  "ObjectCalled(" ++ˢ showOid oid ++ˢ ", " ++ˢ showValAgda inp ++ˢ
  caseMaybe mOut (λ out → " -> " ++ˢ showValAgda out) "" ++ˢ ")"
showEventType (MessageReceived oid inp) = "MessageReceived(" ++ˢ showOid oid ++ˢ ", " ++ˢ showValAgda inp ++ˢ ")"
showEventType (ObjectMoved oid from to) = "ObjectMoved(" ++ˢ showOid oid ++ˢ ", " ++ˢ from ++ˢ " -> " ++ˢ to ++ˢ ")"
showEventType (ExecutionMoved from to) = "ExecutionMoved(" ++ˢ from ++ˢ " -> " ++ˢ to ++ˢ ")"
showEventType (ObjectFetched oid mid) = "ObjectFetched(" ++ˢ showOid oid ++ˢ ", " ++ˢ mid ++ˢ ")"
showEventType (ObjectTransferred oid fromCtrl toCtrl) = "ObjectTransferred(" ++ˢ showOid oid ++ˢ ", " ++ˢ fromCtrl ++ˢ " -> " ++ˢ toCtrl ++ˢ ")"
showEventType (ObjectFrozen oid ctrl) = "ObjectFrozen(" ++ˢ showOid oid ++ˢ ", " ++ˢ ctrl ++ˢ ")"
showEventType (FunctionUpdated name) = "FunctionUpdated(" ++ˢ name ++ˢ ")"
showEventType (TransactionStarted txid) = "TransactionStarted(" ++ˢ showNat txid ++ˢ ")"
showEventType (TransactionCommitted txid) = "TransactionCommitted(" ++ˢ showNat txid ++ˢ ")"
showEventType (TransactionAborted txid) = "TransactionAborted(" ++ˢ showNat txid ++ˢ ")"
showEventType (ErrorOccurred err) = "ErrorOccurred(...)"

showLogEntry : LogEntry → String
showLogEntry entry =
  "[" ++ˢ showNat (LogEntry.timestamp entry) ++ˢ "] " ++ˢ
  showEventType (LogEntry.eventType entry) ++ˢ
  " @" ++ˢ LogEntry.executingController entry

showTrace : Trace → String
showTrace [] = "(no events)"
showTrace trace = foldr (λ entry acc → showLogEntry entry ++ˢ "\n" ++ˢ acc) "" trace

showResult : ∀ {A} → (A → String) → AVMResult A → String
showResult showA (failure err) = "Error: " ++ˢ showError err
showResult showA (success res) =
  "Success: " ++ˢ showA (Success.value res) ++ˢ "\n\n" ++ˢ
  "Trace:\n" ++ˢ showTrace (Success.trace res)

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
  IT._>>=_ (trigger (obj-create "PlayerBoard")) λ board →
  IT._>>=_ (trigger (obj-call board (PB.VShip 0 0 3))) λ _ →
  trigger (obj-call board (PB.VCoord 0 1))

-- Test 2: Single board - MISS
testMiss : AVMProgram (Maybe PB.Val)
testMiss =
  IT._>>=_ (trigger (obj-create "PlayerBoard")) λ board →
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
