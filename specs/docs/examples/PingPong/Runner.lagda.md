---
title: Ping-Pong Runner
icon: material/play
tags:
  - runner
  - examples
  - AVM interpreter
---

This module provides the execution harness for the Ping-Pong protocol example,
utilizing the [AVM.Interpreter](../../AVM/Interpreter.lagda.md) operational
semantics implementation to execute the Ping-Pong protocol example.

```agda
{-# OPTIONS --type-in-type --guardedness #-}
module examples.PingPong.Runner where

open import Background.BasicTypes
open import Background.InteractionTrees as IT hiding (_>>=_)
open import examples.PingPong.Main as PP
open import examples.RunnerUtilities hiding (initialState)

-- Fresh ID generators
freshObjectId : ℕ → PP.ObjectId
freshObjectId n = ("node1" , "obj-" ++ˢ showNat n)

freshTxId : ℕ → PP.TxId
freshTxId n = n

-- Import AVM Interpreter with our types and generators
-- We'll instantiate Object with ObjectImpl after we define it below
module InterpreterImport where
  open import AVM.Interpreter PP.Val PP.ObjectId freshObjectId PP.MachineId PP.ControllerId PP.TxId freshTxId public
```

<details markdown="1">
<summary>Interpreter Parameterization</summary>

The interpreter instantiation requires concrete implementations of
platform-specific parameters as specified by the AVM.Interpreter module
signature.

```agda
-- Equality for ObjectId (pairs of strings)
eqObjectId : PP.ObjectId → PP.ObjectId → Bool
eqObjectId (node1 , id1) (node2 , id2) =
  caseMaybe (node1 ≟-string node2)
    (λ _ → caseMaybe (id1 ≟-string id2) (λ _ → true) false)
    false

-- Equality for TxId (natural numbers)
eqTxId : PP.TxId → PP.TxId → Bool
eqTxId = _==ℕ_

-- Equality for String
eqString : String → String → Bool
eqString s1 s2 = caseMaybe (s1 ≟-string s2) (λ _ → true) false

-- Equality for Bool
eqBool : Bool → Bool → Bool
eqBool true true = true
eqBool false false = true
eqBool _ _ = false

-- Concrete implementation of ObjectBehaviour type
-- ObjectBehaviour is concretely defined as AVMProgram (List Val)
-- We use a named reference approach: String names map to concrete AVMProgram values
data ObjectBehaviourImpl : Set where
  namedBehaviour : String → ObjectBehaviourImpl

-- Now open the AVM.Interpreter module with ObjectBehaviourImpl to get AVMProgram types
open import AVM.Interpreter PP.Val PP.ObjectId freshObjectId PP.MachineId PP.ControllerId PP.TxId freshTxId ObjectBehaviourImpl

-- Reachability predicates
isReachableController : PP.ControllerId → Bool
isReachableController _ = true

isReachableMachine : PP.MachineId → Bool
isReachableMachine _ = true

-- Use message type from Main module
open PP using (MessageType; Ping; Pong)
```

</details>

```agda
-- Helper to extract PingPongMsg from Val
extractMsg : PP.Val → Maybe PP.PingPongMsg
extractMsg (PP.VPingPongMsg msg) = just msg
extractMsg _ = nothing
```

```agda
-- Helper to construct next message
mkNextMsg : MessageType → PP.PingPongMsg → PP.ObjectId → PP.Val
mkNextMsg msgType msg myId =
  PP.VPingPongMsg (PP.mkMsg msgType (suc (PP.PingPongMsg.counter msg)) myId (PP.PingPongMsg.maxCount msg))

-- Generic ping-pong behavior implementation
genericPingPongBehavior : MessageType → String → AVMProgram (List PP.Val)
genericPingPongBehavior responseType role =
  IT._>>=_ (trigger (Introspect input)) λ inp →
  IT._>>=_ (trigger (Introspect self)) λ myId →
  caseMaybe (extractMsg inp)
    (λ msg →
      (if PP.PingPongMsg.counter msg <? PP.PingPongMsg.maxCount msg then
        (IT._>>=_ (trigger (Obj (call (PP.PingPongMsg.partnerId msg) (mkNextMsg responseType msg myId)))) λ _ →
         ret (PP.VString (role ++ˢ "-continue") ∷ []))
      else
        ret (PP.VString (role ++ˢ "-done") ∷ [])))
    (ret (PP.VString (role ++ˢ "-decode-failed") ∷ []))

-- Ping object behavioral specification: processes incoming messages and generates Pong responses
pingBehaviorImpl : AVMProgram (List PP.Val)
pingBehaviorImpl = genericPingPongBehavior Pong "ping"

-- Pong object behavioral specification: processes incoming messages and generates Ping responses
pongBehaviorImpl : AVMProgram (List PP.Val)
pongBehaviorImpl = genericPingPongBehavior Ping "pong"

-- Interpret behavior name to ObjectBehaviourImpl (used by createObj instruction)
-- This maps behavior names to our named reference type
interpretBehaviorName : String → ObjectBehaviourImpl
interpretBehaviorName s = namedBehaviour s

-- Extract all object IDs from the state
-- For this simple example, we collect IDs from observed objects and pending creates
-- A more complete implementation would maintain a global registry
allObjectIds : State → List PP.ObjectId
allObjectIds st = State.observed st ++ map (λ { (oid , _ , _) → oid }) (State.creates st)

-- Convert ObjectBehaviour to AVMProgram
-- Maps behavior names to their concrete implementations
getBehavior : ObjectBehaviourImpl → AVMProgram (List PP.Val)
getBehavior (namedBehaviour "ping") = pingBehaviorImpl
getBehavior (namedBehaviour "pong") = pongBehaviorImpl
getBehavior (namedBehaviour _) = ret []  -- Default: empty behavior

-- Instantiate the interpreter with type-specific equality functions
-- This replaces the broken generic equality with proper implementations
module RunnerInterpreter where
  {-# TERMINATING #-}
  mutual
    -- Forward reference to the full interpreter
    interpretAVMProgramRec : ∀ {A} → AVMProgram A → State → AVMResult A
    interpretAVMProgramRec = interp.interpretAVMProgram
      where
        module interp = Interpreter eqObjectId eqTxId allObjectIds interpretBehaviorName getBehavior isReachableController isReachableMachine interpretAVMProgramRec

    -- Now open the interpreter with the recursive reference
    open Interpreter eqObjectId eqTxId allObjectIds interpretBehaviorName getBehavior isReachableController isReachableMachine interpretAVMProgramRec public

open RunnerInterpreter public
```

## Initial Execution State Configuration

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
  ; self = ("root", "orchestrator")
  ; input = VString ""
  ; sender = nothing
  ; traceMode = false
  ; eventCounter = 0
  }
```

## Observability and Rendering Infrastructure

```agda
{-# TERMINATING #-}
showValAgda : PP.Val → String
showValAgda (PP.VInt n) = "(VInt " ++ˢ showNat n ++ˢ ")"
showValAgda (PP.VString s) = "\"" ++ˢ s ++ˢ "\""
showValAgda (PP.VList []) = "[]"
showValAgda (PP.VList (x ∷ xs)) = showValAgda x ++ˢ " :: " ++ˢ showValAgda (PP.VList xs)
showValAgda (PP.VPingPongMsg record { msgType = PP.Ping ; counter = c }) =
  "(VPingPongMsg Ping " ++ˢ showNat c ++ˢ ")"
showValAgda (PP.VPingPongMsg record { msgType = PP.Pong ; counter = c }) =
  "(VPingPongMsg Pong " ++ˢ showNat c ++ˢ ")"

showError : AVMError → String
showError _ = "execution-error"

-- Extract object ID string for display
showOid : PP.ObjectId → String
showOid (_ , id) = id

-- Display EventType
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

-- Display LogEntry
showLogEntry : LogEntry → String
showLogEntry entry =
  "[" ++ˢ showNat (LogEntry.timestamp entry) ++ˢ "] " ++ˢ
  showEventType (LogEntry.eventType entry) ++ˢ
  " @" ++ˢ LogEntry.executingController entry

-- Display Trace
showTrace : Trace → String
showTrace [] = "(no events)"
showTrace trace = foldr (λ entry acc → showLogEntry entry ++ˢ "\n" ++ˢ acc) "" trace

showResult : ∀ {A} → (A → String) → AVMResult A → String
showResult showA (failure err) = "Error: " ++ˢ showError err
showResult showA (success res) =
  "Success: " ++ˢ showA (Success.value res) ++ˢ "\n\n" ++ˢ
  "Trace:\n" ++ˢ showTrace (Success.trace res)
```

## Execution Harness Implementation

The example program is specified in Main.lagda.md. However, due to the
abstraction of the Object type in Main and the requirement for concrete
ObjectImpl instances in this module, direct utilization of PP.pingPongExample is
precluded. Consequently, we construct the test execution manually by directly
invoking the behavioral functions.

```agda
testSimple : AVMProgram PP.Val
testSimple =
  IT._>>=_ (trigger (obj-create "pong")) λ pongId →
  let msg = PP.VPingPongMsg (PP.mkMsg PP.Ping 0 pongId 1)
  in IT._>>=_ (trigger (obj-call pongId msg)) λ mResult →
     caseMaybe mResult
       (λ result → ret result)
       (ret (PP.VString "call-failed"))

test : AVMProgram PP.Val
test =
  IT._>>=_ (trigger (Tx beginTx)) λ txid →
  IT._>>=_ (trigger (obj-create "pong")) λ pongId →
  IT._>>=_ (trigger (obj-create "ping")) λ pingId →
  IT._>>=_ (trigger (Tx (commitTx txid))) λ _ →
  let initialMsg = PP.VPingPongMsg (PP.mkMsg PP.Ping 0 pongId 5)
  in IT._>>=_ (trigger (obj-call pingId initialMsg)) λ mResult →
     caseMaybe mResult
       (λ result → ret (PP.VList (PP.VString "complete" ∷ result ∷ [])))
       (ret (PP.VString "call-failed"))

runExample : String → AVMProgram PP.Val → PrimIO ⊤
runExample title prog = do
  putStrLn ("=== " ++ˢ title ++ˢ " ===")
  putStrLn ""
  let result = interpretAVMProgram prog initialState
  putStrLn (showResult showValAgda result)
  putStrLn ""

main : PrimIO ⊤
main = do
  runExample "Simple Pong Test" testSimple
  putStrLn ""
  runExample "Ping-Pong Example" test
  putStrLn "Done!"
```
