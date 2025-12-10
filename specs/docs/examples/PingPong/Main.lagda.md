---
title: Ping-Pong Example
icon: fontawesome/solid/arrow-right-arrow-left
tags:
  - AVM
  - examples
  - formal semantics
  - objects
---

<div class="admonition quote collapse" markdown="1">
<p class="admonition-title">Quote</p>

```agda
{-# OPTIONS --without-K --type-in-type --guardedness #-}
module examples.PingPong.Main where
```

```agda

open import Background.BasicTypes
open import Background.InteractionTrees
```

</div>

This example demonstrates two objects (Ping and Pong) that exchange messages
back and forth until a maximum count is reached.

## Type Definitions

```agda
NodeId : Set
NodeId = String

ObjectId : Set
ObjectId = NodeId × String

data MessageType : Set where
  Ping : MessageType
  Pong : MessageType

record PingPongMsg : Set where
  constructor mkMsg
  field
    msgType : MessageType
    counter : ℕ
    partnerId : ObjectId
    maxCount : ℕ

open PingPongMsg

data Val : Set where
  VInt : ℕ → Val
  VString : String → Val
  VList : List Val → Val
  VPingPongMsg : PingPongMsg → Val
```

<details markdown="1">
<summary>Extra definitions</summary>

```agda
ControllerId : Set
ControllerId = String

MachineId : Set
MachineId = String

TxId : Set
TxId = ℕ
```

```agda
Input : Set
Input = Val

Output : Set
Output = Val

Message : Set
Message = Val
```

We define Agda@ObjectBehaviour as the type for object behaviours, which will be
instantiated concretely in the Runner module:
 
```agda
ObjectBehaviour : Set
ObjectBehaviour = ⊤
```

With the Agda@ObjectBehaviour type, we can import the AVM instruction set:

```agda
open import AVM.Instruction Val ObjectId MachineId ControllerId TxId ObjectBehaviour
  hiding (Input; Output; InputSequence; Message)
```

</details>


## Main Program

```agda
createPing : AVMProgram ObjectId
createPing = trigger (obj-create "ping" nothing)

createPong : AVMProgram ObjectId
createPong = trigger (obj-create "pong" nothing)

startPingPong : ℕ → AVMProgram Val
startPingPong maxCount =
  createPing >>= λ pingId →
  createPong >>= λ pongId →
  let initialMsg = VPingPongMsg record
        { msgType = Ping
        ; counter = 0
        ; partnerId = pongId
        ; maxCount = maxCount
        }
  in trigger (obj-call pingId initialMsg) >>= λ mResult →
     caseMaybe mResult
       (λ result → ret (VList (VString "complete" ∷ result ∷ [])))
       (ret (VString "call-failed"))

pingPongExample : AVMProgram Val
pingPongExample = startPingPong 5
```

## Execution

See [Runner](./Runner.lagda.md) for the runner that uses
[AVM.Interpreter](../../AVM/Interpreter.lagda.md) to execute this example.
