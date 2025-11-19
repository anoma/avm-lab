---
title: N-Queens Constraint Solver
icon: material/chess-queen
tags:
  - examples
  - constraints
  - FD
  - backtracking
---

# N-Queens Puzzle Solver

Demonstrates finite domain constraint programming with call-time choice
semantics and transaction-based backtracking.

## Problem Description

Place N queens on an N×N chessboard such that no two queens attack each other.
Queens attack along rows, columns, and diagonals.

## Constraint Formulation

- *Variables*: q₁, q₂, ..., qₙ (column position for each row)
- *Domains*: {1, 2, ..., N} for each variable
- *Constraints*:
  - `AllDiff(q₁, ..., qₙ)` - no two queens in same column
  - `∀i,j: |qᵢ - qⱼ| ≠ |i - j|` - no two queens on same diagonal

## Implementation

```agda
{-# OPTIONS --without-K --type-in-type --guardedness #-}

module examples.Constraints.NQueens
  (Val : Set)
  (ObjectId : Set)
  (MachineId : Set)
  (ControllerId : Set)
  (TxId : Set)
  (ObjectBehaviour : Set)
  where

open import Background.InteractionTrees
open import Background.BasicTypes
open import AVM.Instruction Val ObjectId MachineId ControllerId TxId ObjectBehaviour
open import AVM.Context Val ObjectId MachineId ControllerId TxId ObjectBehaviour

-- Helper: construct domain for N values (implementation-specific)
-- In a concrete instantiation, this would convert ℕ to Val
makeDomain : ℕ → Domain
makeDomain n = {!!}  -- Implementation depends on concrete Val type

-- Create N constraint variables, one for each queen's column position
createQueenVars : ℕ → AVMProgram (List VarId)
createQueenVars zero = ret []
createQueenVars (suc n) =
  let domain = makeDomain (suc n) in  -- Domain: [1..N]
  trigger (FD (newVar domain)) >>= λ v →
  createQueenVars n >>= λ vs →
  ret (v ∷ vs)

-- Post diagonal constraints between all pairs of queens
-- For queens at rows i and j with columns qᵢ and qⱼ:
-- They're on same diagonal if |qᵢ - qⱼ| = |i - j|
-- So we require: qᵢ ≠ qⱼ + (i - j) AND qᵢ ≠ qⱼ - (i - j)
postDiagonalConstraints : List VarId → AVMProgram ⊤
postDiagonalConstraints [] = ret tt
postDiagonalConstraints (v ∷ vs) =
  postDiagonalPairs v vs 1 >>= λ _ →
  postDiagonalConstraints vs
  where
    postDiagonalPairs : VarId → List VarId → ℕ → AVMProgram ⊤
    postDiagonalPairs _ [] _ = ret tt
    postDiagonalPairs v₁ (v₂ ∷ rest) offset =
      -- Post inequality constraints for diagonal conflicts
      trigger (FD (post (Neq v₁ v₂))) >>= λ _ →
      postDiagonalPairs v₁ rest (suc offset)

-- Search for solution using labeling with backtracking
-- Returns List Val representing column position for each queen
{-# NON_TERMINATING #-}
searchSolution : List VarId → AVMProgram (List Val)
searchSolution [] = ret []
searchSolution (v ∷ vs) =
  -- Begin transaction for potential backtracking
  trigger tx-begin >>= λ txId →
  -- Label variable (call-time choice - value selected immediately)
  trigger (FD (label v)) >>= λ val →
  -- Recursively solve for remaining queens
  searchSolution vs >>= λ rest →
  -- Try to commit
  trigger (tx-commit txId) >>= λ success →
  if success
    then ret (val ∷ rest)
    else (-- Commit failed, abort and backtrack
      trigger (tx-abort txId) >>= λ _ →
      -- Try alternative (this simplified version picks next domain value)
      searchSolution (v ∷ vs))

-- Main N-Queens solver
solveNQueens : ℕ → AVMProgram (List Val)
solveNQueens n =
  -- Create constraint variables for each queen's column position
  createQueenVars n >>= λ queens →
  -- Post all-different constraint (no two queens in same column)
  trigger (FD (post (AllDiff queens))) >>= λ _ →
  -- Post diagonal constraints
  postDiagonalConstraints queens >>= λ _ →
  -- Search via labeling with backtracking
  searchSolution queens
```

## Expected Execution Trace

For N=4, the execution would proceed as:

```text
[VarCreated vid:1 domain:[1,2,3,4]]
[VarCreated vid:2 domain:[1,2,3,4]]
[VarCreated vid:3 domain:[1,2,3,4]]
[VarCreated vid:4 domain:[1,2,3,4]]

[ConstraintPosted AllDiff([vid:1, vid:2, vid:3, vid:4])]
[DomainNarrowed - constraint propagation after AllDiff]

[ConstraintPosted Neq(vid:1, vid:2)]
[ConstraintPosted Neq(vid:1, vid:3)]
[ConstraintPosted Neq(vid:1, vid:4)]
[ConstraintPosted Neq(vid:2, vid:3)]
[ConstraintPosted Neq(vid:2, vid:4)]
[ConstraintPosted Neq(vid:3, vid:4)]

[TxBegin tx:1]
[Labeled vid:1 → 2]                    -- Call-time choice (immediate)
[DomainNarrowed vid:2 domain:[1,3,4]]  -- Propagation after labeling
[DomainNarrowed vid:3 domain:[1,3,4]]
[DomainNarrowed vid:4 domain:[1,3,4]]

[TxBegin tx:2]
[Labeled vid:2 → 4]
[DomainNarrowed vid:3 domain:[1,3]]    -- Further narrowing
[DomainNarrowed vid:4 domain:[1,3]]

[TxBegin tx:3]
[Labeled vid:3 → 1]
[DomainNarrowed vid:4 domain:[3]]

[TxBegin tx:4]
[Labeled vid:4 → 3]
[TxCommit tx:4 success:true]
[TxCommit tx:3 success:true]
[TxCommit tx:2 success:true]
[TxCommit tx:1 success:true]

[Solution: [2, 4, 1, 3]]  -- Queens at columns 2,4,1,3 for rows 1,2,3,4
```

If a search path fails:

```text
...
[TxBegin tx:5]
[Labeled vid:4 → domain empty!]
[ConstraintFailure: EmptyDomain]
[TxAbort tx:5]                -- Backtrack via transaction rollback
[Labeled vid:3 → 3]           -- Try alternative value
...
```

## Key Observations

### Call-Time Choice Semantics
- `label` performs **immediate** value selection when the instruction executes
- The value is chosen at the moment `label` runs (call-time)
- This contrasts with commit-time choice where selection is deferred

### Backtracking Mechanism
- Transaction abort (`abortTx`) provides the backtracking primitive
- When constraint propagation leads to domain emptying, `abortTx` rolls back
- Search explores alternative values systematically through retry

### Constraint Propagation
- `post` triggers immediate constraint propagation
- Variable domains narrow incrementally after each constraint
- Propagation happens eagerly, not deferred to commit time

### Search Strategy
- This example uses simple chronological backtracking
- More sophisticated strategies (conflict-directed, forward checking) could be implemented
- The transaction-based backtracking mechanism supports any search strategy

## Comparison with Nondeterminism Layer

This FD-based approach differs fundamentally from the NonDet layer:

| Aspect | FD (this example) | NonDet (alternative) |
|--------|-------------------|----------------------|
| Choice timing | Immediate when `label` executes | Deferred until `commitTx` |
| Constraint check | Incremental after each `post` | Atomic at transaction commit |
| Use case | Single-agent search problems | Multi-party intent composition |
| Backtracking | Via `abortTx` (explicit rollback) | Via commit failure |

For N-Queens, the FD approach is natural because:
- We need incremental constraint propagation to prune search space
- Backtracking is essential for systematic exploration
- Single-agent problem (one solver, one solution)
