---
title: Everything
search:
  exclude: true
hide:
  - navigation
  - toc
---

# Everything

This module imports all Agda modules in the project for batch compilation.

```agda
{-# OPTIONS --guardedness --type-in-type #-}

module everything where

-- Background ---
import Background.BasicTypes
import Background.InteractionTrees
import Background.StatelessObjects

-- AVM ---
import AVM.Context
import AVM.Instruction
import AVM.Interpreter

-- Examples ---
import examples.Battleship.Game
import examples.Battleship.PlayerBoard
import examples.Battleship.Runner
import examples.PingPong.Main
import examples.PingPong.Runner
import examples.RunnerUtilities
```
