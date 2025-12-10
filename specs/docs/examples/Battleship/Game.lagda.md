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

## Player Type

Represents the two players in the game.

```agda
data Player : Set where
  Player1 : Player
  Player2 : Player

-- Game state holding both player boards
GameState : Set
GameState = PB.ObjectId × PB.ObjectId
```

## Game Setup

Creates a game with two player boards, each with pre-placed ships.

```agda
setupGame : AVMProgram GameState
setupGame =
  trigger (tx-begin nothing) >>= λ setupTx →
  trigger (obj-create "PlayerBoard" nothing) >>= λ board1 →
  trigger (obj-create "PlayerBoard" nothing) >>= λ board2 →
  -- Place ships for player 1
  trigger (obj-call board1 (PB.VShip 0 0 3)) >>= λ _ →
  trigger (obj-call board1 (PB.VShip 2 2 2)) >>= λ _ →
  -- Place ships for player 2
  trigger (obj-call board2 (PB.VShip 1 1 3)) >>= λ _ →
  trigger (obj-call board2 (PB.VShip 4 4 2)) >>= λ _ →
  trigger (tx-commit setupTx) >>= λ _ →
  ret (board1 , board2)

-- Get the opponent's board given the current player
opponentBoard : Player → GameState → PB.ObjectId
opponentBoard Player1 (board1 , board2) = board2
opponentBoard Player2 (board1 , board2) = board1
```

## Playing Turns

Players attack each other's boards directly.

```agda
-- Attack a board at coordinate (x, y)
attack : PB.ObjectId → ℕ → ℕ → AVMProgram (Maybe PB.Val)
attack board x y =
  trigger (tx-begin nothing) >>= λ turnTx →
  trigger (obj-call board (PB.VCoord x y)) >>= λ result →
  trigger (tx-commit turnTx) >>= λ _ →
  ret result

-- Player-based attack using the Player type
playerAttack : Player → GameState → ℕ → ℕ → AVMProgram (Maybe PB.Val)
playerAttack player gameState x y = attack (opponentBoard player gameState) x y

-- Convenience aliases for backward compatibility
player1Attack = attack
player2Attack = attack
```

## Example Full Game

A complete game with setup and multiple attack turns.

```agda
playFullGame : AVMProgram GameState
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
gameWithRollback : AVMProgram GameState
gameWithRollback =
  setupGame >>= λ { (board1 , board2) →

  -- Start a turn but abort it
  trigger (tx-begin nothing) >>= λ badTurn →
  trigger (obj-call board2 (PB.VCoord 1 1)) >>= λ _ →
  trigger (tx-abort badTurn) >>= λ _ →

  -- Valid turn
  player1Attack board2 1 1 >>= λ _ →

  ret (board1 , board2) }
```

## Example Game Using Player Type

A complete game using the Player type for cleaner code.

```agda
playGameWithPlayerType : AVMProgram GameState
playGameWithPlayerType =
  setupGame >>= λ gameState →

  -- Turn 1: Player 1 attacks at (1, 1) - should HIT
  playerAttack Player1 gameState 1 1 >>= λ _ →

  -- Turn 2: Player 2 attacks at (0, 0) - should HIT
  playerAttack Player2 gameState 0 0 >>= λ _ →

  -- Turn 3: Player 1 attacks at (5, 5) - should MISS
  playerAttack Player1 gameState 5 5 >>= λ _ →

  -- Turn 4: Player 2 attacks at (2, 2) - should HIT
  playerAttack Player2 gameState 2 2 >>= λ _ →

  ret gameState
```
