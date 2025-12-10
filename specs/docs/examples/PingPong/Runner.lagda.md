---
title: Ping-Pong Runner
icon: material/play
tags:
  - runner
  - examples
  - AVM interpreter
---

This module provides the execution harness for the Ping-Pong example using
the [AVM.Interpreter](../../AVM/Interpreter.lagda.md).

```agda
{-# OPTIONS --type-in-type --guardedness #-}
module examples.PingPong.Runner where

open import Background.BasicTypes
open import Background.InteractionTrees as IT hiding (_>>=_)
open import examples.PingPong.Main as PP
open import examples.RunnerUtilities hiding (initialState)
open import examples.Common.Equality
```

## Display Functions

```agda
showOid : PP.ObjectId → String
showOid (_ , id) = id

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
```

```agda
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

-- Concrete implementation of ObjectBehaviour type
-- ObjectBehaviour is concretely defined as AVMProgram (List Val)
-- We use a named reference approach: String names map to concrete AVMProgram values
data ObjectBehaviourImpl : Set where
  namedBehaviour : String → ObjectBehaviourImpl

-- Now open the AVM.Interpreter module with ObjectBehaviourImpl to get AVMProgram types
open import AVM.Interpreter PP.Val PP.ObjectId freshObjectId PP.MachineId PP.ControllerId PP.TxId freshTxId ObjectBehaviourImpl
open import examples.Common.StateInit PP.Val PP.ObjectId PP.MachineId PP.ControllerId PP.TxId ObjectBehaviourImpl
open import examples.Common.Display PP.Val PP.ObjectId PP.TxId PP.ControllerId PP.MachineId ObjectBehaviourImpl showValAgda showOid showNat (λ s → s) (λ s → s) hiding (showNat)

-- Reachability predicates
eqControllerId : PP.ControllerId → PP.ControllerId → Bool
eqControllerId = eqString

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

mkControllerId : String → PP.ControllerId
mkControllerId s = s

-- Instantiate the interpreter with type-specific equality functions
-- This replaces the broken generic equality with proper implementations
module RunnerInterpreter where
  {-# TERMINATING #-}
  mutual
    -- Forward reference to the full interpreter
    interpretAVMProgramRec : ∀ {A} → AVMProgram A → State → AVMResult A
    interpretAVMProgramRec = interp.interpretAVMProgram
      where
        module interp = Interpreter eqObjectId eqTxId eqControllerId allObjectIds interpretBehaviorName getBehavior (isReachableController {PP.ControllerId}) (isReachableMachine {PP.MachineId}) mkControllerId interpretAVMProgramRec

    -- Now open the interpreter with the recursive reference
    open Interpreter eqObjectId eqTxId eqControllerId allObjectIds interpretBehaviorName getBehavior (isReachableController {PP.ControllerId}) (isReachableMachine {PP.MachineId}) mkControllerId interpretAVMProgramRec public

open RunnerInterpreter public
```

## Initial State

```agda
initialState : State
initialState = mkInitialState "node1" ("root", "orchestrator") (VString "")
```

## Test Programs

```agda
testSimple : AVMProgram PP.Val
testSimple =
  IT._>>=_ (trigger (obj-create "pong" nothing)) λ pongId →
  let msg = PP.VPingPongMsg (PP.mkMsg PP.Ping 0 pongId 1)
  in IT._>>=_ (trigger (obj-call pongId msg)) λ mResult →
     caseMaybe mResult
       (λ result → ret result)
       (ret (PP.VString "call-failed"))

test : AVMProgram PP.Val
test =
  IT._>>=_ (trigger (Tx (beginTx nothing))) λ txid →
  IT._>>=_ (trigger (obj-create "pong" nothing)) λ pongId →
  IT._>>=_ (trigger (obj-create "ping" nothing)) λ pingId →
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
