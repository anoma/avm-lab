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

--- AVM ---
import AVM.Instruction
import AVM.Interpreter
import AVM.Context

--- Background ---
import Background.BasicTypes
import Background.InteractionTrees
import Background.Objects
import Background.StatelessObjects

--- Examples ---
import examples.PingPong.Main
import examples.PingPong.Runner
import examples.RunnerUtilities
```
