# Common Equality Functions

This module provides standard equality predicates and reachability checks used
across AVM example implementations.

```agda
{-# OPTIONS --without-K --type-in-type #-}

module examples.Common.Equality where

open import Background.BasicTypes
```

## String Equality

String equality comparison utilizes Agda's built-in decidable equality for
string types, converting the decidable result to a boolean value.

```agda
eqString : String → String → Bool
eqString s1 s2 = caseMaybe (s1 ≟-string s2) (λ _ → true) false
```

## Natural Number Equality

Natural number equality comparison leverages the primitive equality operation
on natural numbers. This predicate is commonly used for transaction identifier
comparison.

```agda
eqNat : ℕ → ℕ → Bool
eqNat = _==ℕ_
```

## Boolean Equality

Boolean equality comparison implements structural equality for boolean values.

```agda
eqBool : Bool → Bool → Bool
eqBool true true = true
eqBool false false = true
eqBool _ _ = false
```

## Reachability Predicates

Reachability predicates determine whether controllers and machines are
accessible from the current execution context. For local single-node execution,
all controllers and machines are considered reachable, thus these predicates
unconditionally return true.

```agda
isReachableController : {A : Set} → A → Bool
isReachableController _ = true

isReachableMachine : {A : Set} → A → Bool
isReachableMachine _ = true
```
