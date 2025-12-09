# Player Board

This module defines the player board behavior for the Battleship game.
Each player has their own board that stores ships and responds to attacks.

```agda
{-# OPTIONS --without-K --type-in-type --guardedness #-}

module examples.Battleship.PlayerBoard where

open import Background.BasicTypes
open import Background.InteractionTrees
```

## Type Definitions

```agda
ObjectId : Set
ObjectId = ℕ

data Val : Set where
  VCoord : ℕ → ℕ → Val      -- Attack coordinate
  VShip : ℕ → ℕ → ℕ → Val   -- Ship placement (x, y, length)

MachineId : Set
MachineId = String

ControllerId : Set
ControllerId = String

TxId : Set
TxId = ℕ

ObjectBehaviour : Set
ObjectBehaviour = ⊤
```

## Instruction Set

```agda
open import AVM.Instruction Val ObjectId MachineId ControllerId TxId ObjectBehaviour
  hiding (Input; Output; InputSequence)
```

## Board Logic

```agda
-- Check if coordinate is in a ship (vertical ship only for simplicity)
coordinateHitsShip : ℕ → ℕ → ℕ → ℕ → ℕ → Bool
coordinateHitsShip shipX shipY shipLen targetX targetY =
  if shipX ==ℕ targetX then
    if shipY ≤? targetY then
      targetY <? (shipY +ℕ shipLen)
    else false
  else false

-- Extract ship from Val
extractShip : Val → Maybe (ℕ × ℕ × ℕ)
extractShip (VShip x y len) = just (x , y , len)
extractShip _ = nothing

-- Check if any ship is hit
checkHit : List Val → ℕ → ℕ → Bool
checkHit [] x y = false
checkHit (v ∷ vs) x y =
  caseMaybe (extractShip v)
    (λ { (shipX , shipY , shipLen) →
      if coordinateHitsShip shipX shipY shipLen x y
      then true
      else checkHit vs x y })
    (checkHit vs x y)
```

## Board Behavior

The board behavior responds to two types of messages:
- Ship placement (VShip): stores the ship in state
- Attack (VCoord): checks if the coordinate hits any ship, returns hit/miss

```agda
boardBehavior : AVMProgram (List Val)
boardBehavior =
  trigger (Introspect input) >>= λ inp →
  trigger (Introspect getState) >>= λ currentState →
  handleInput inp currentState
  where
    handleInput : Val → List Val → AVMProgram (List Val)
    handleInput (VShip x y len) currentState =
      let newState = VShip x y len ∷ currentState in
      trigger (Introspect (setState newState)) >>= λ _ →
      ret (VShip x y len ∷ [])  -- Return ship as confirmation
    handleInput (VCoord x y) currentState =
      if checkHit currentState x y
      then ret (VCoord x y ∷ [])  -- HIT
      else ret []                  -- MISS
```
