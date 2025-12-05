# Battleship Game Orchestrator

This module defines the game orchestrator that coordinates the Battleship game
between two players.

```agda
{-# OPTIONS --without-K --type-in-type --guardedness #-}

module examples.Battleship.Game where

open import Background.BasicTypes
open import Background.InteractionTrees
open import examples.Battleship.PlayerBoard as PB
```

## Instruction Set

```agda
open import AVM.Instruction PB.Val PB.ObjectId PB.MachineId PB.ControllerId PB.TxId PB.ObjectBehaviour
  hiding (Input; Output; InputSequence)
```

## Game Setup

Creates a game with two player boards, each with pre-placed ships.

```agda
setupGame : AVMProgram (PB.ObjectId × PB.ObjectId)
setupGame =
  trigger tx-begin >>= λ setupTx →
  trigger (obj-create "PlayerBoard") >>= λ board1 →
  trigger (obj-create "PlayerBoard") >>= λ board2 →
  -- Place ships for player 1
  trigger (obj-call board1 (PB.VShip 0 0 3)) >>= λ _ →
  trigger (obj-call board1 (PB.VShip 2 2 2)) >>= λ _ →
  -- Place ships for player 2
  trigger (obj-call board2 (PB.VShip 1 1 3)) >>= λ _ →
  trigger (obj-call board2 (PB.VShip 4 4 2)) >>= λ _ →
  trigger (tx-commit setupTx) >>= λ _ →
  ret (board1 , board2)
```

## Playing Turns

Players attack each other's boards directly.

```agda
-- Player 1 attacks Player 2's board at coordinate (x, y)
player1Attack : PB.ObjectId → ℕ → ℕ → AVMProgram (Maybe PB.Val)
player1Attack board2 x y =
  trigger tx-begin >>= λ turnTx →
  trigger (obj-call board2 (PB.VCoord x y)) >>= λ result →
  trigger (tx-commit turnTx) >>= λ _ →
  ret result

-- Player 2 attacks Player 1's board at coordinate (x, y)
player2Attack : PB.ObjectId → ℕ → ℕ → AVMProgram (Maybe PB.Val)
player2Attack board1 x y =
  trigger tx-begin >>= λ turnTx →
  trigger (obj-call board1 (PB.VCoord x y)) >>= λ result →
  trigger (tx-commit turnTx) >>= λ _ →
  ret result
```

## Example Full Game

A complete game with setup and multiple attack turns.

```agda
playFullGame : AVMProgram (PB.ObjectId × PB.ObjectId)
playFullGame =
  setupGame >>= λ { (board1 , board2) →

  -- Turn 1: Player 1 attacks board2 at (1, 1) - should HIT
  player1Attack board2 1 1 >>= λ _ →

  -- Turn 2: Player 2 attacks board1 at (0, 0) - should HIT
  player2Attack board1 0 0 >>= λ _ →

  -- Turn 3: Player 1 attacks board2 at (5, 5) - should MISS
  player1Attack board2 5 5 >>= λ _ →

  -- Turn 4: Player 2 attacks board1 at (2, 2) - should HIT
  player2Attack board1 2 2 >>= λ _ →

  ret (board1 , board2) }
```

## Game with Rollback

Demonstrates transaction rollback in the game.

```agda
gameWithRollback : AVMProgram (PB.ObjectId × PB.ObjectId)
gameWithRollback =
  setupGame >>= λ { (board1 , board2) →

  -- Start a turn but abort it
  trigger tx-begin >>= λ badTurn →
  trigger (obj-call board2 (PB.VCoord 1 1)) >>= λ _ →
  trigger (tx-abort badTurn) >>= λ _ →

  -- Valid turn
  player1Attack board2 1 1 >>= λ _ →

  ret (board1 , board2) }
```
