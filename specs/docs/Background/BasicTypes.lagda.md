---
title: Basic Types
icon: material/shape
tags:
  - basic types
  - type theory
---

This module establishes the foundational type-theoretic constructs used
throughout different entries in this website.

```agda
{-# OPTIONS --without-K --type-in-type --exact-split #-}
module Background.BasicTypes where

open import Agda.Primitive using (Level; lsuc)
```

## Product Types

The product type represents pairs of values, fundamental for modeling object
state that consists of multiple components.

Product type for pairing values:

```agda
data _×_ (A B : Set) : Set where
  _,_ : A → B → A × B
```

```agda
infixr 4 _×_
infixr 4 _,_
```

Projection functions:

```agda
fst : {A B : Set} → A × B → A
fst (a , b) = a
```

```agda
snd : {A B : Set} → A × B → B
snd (a , b) = b
```

Dependent pair type (Σ type):

```agda
record Σ (A : Set) (B : A → Set) : Set where
  constructor _,Σ_
  field
    proj₁ : A
    proj₂ : B proj₁
```

```agda
open Σ public
```

Alternative notation for Σ interpreted as existential quantification:

```agda
∃ : {ℓ : Level} {A : Set ℓ} → (A → Set ℓ) → Set ℓ
∃ {ℓ} {A} P = Σ A P

∃-syntax : {ℓ : Level} (A : Set ℓ) → (A → Set ℓ) → Set ℓ
∃-syntax A P = Σ A P

syntax ∃-syntax A (λ x → B) = ∃[ x ∈ A ] B
```

## Coproduct Types

Coproduct or Sum types model choice and alternatives, essential for representing
different message types and object states in the AVM.

Coproduct type for alternatives:

```agda
data _+_ (A B : Set) : Set where
  inl : A → A + B
  inr : B → A + B
```

```agda
infixr 3 _+_
```

Case analysis for sum types:

```agda
case_of_ : {A B C : Set} → A + B → (A → C) → (B → C) → C
case (inl a) of f = λ g → f a
case (inr b) of f = λ g → g b
```

```agda
infix 0 case_of_
```

## Unit and Empty Types

The unit type represents trivial computations, while the empty type represents
impossible situations.

Unit type - exactly one value:

```agda
record ⊤ : Set where
  constructor tt
{-# BUILTIN UNIT ⊤ #-}
{-# COMPILE GHC ⊤ = data () (()) #-}
```

Empty type - no values:

```agda
data ⊥ : Set where
```

Elimination principle for the empty type:

```agda
⊥-elim : {A : Set} → ⊥ → A
⊥-elim ()
```

Negation:

```agda
¬ : Set → Set
¬ A = A → ⊥
```

## Boolean Type

```agda
data Bool : Set where
  true  : Bool
  false : Bool
{-# BUILTIN BOOL Bool #-}
{-# BUILTIN TRUE true #-}
{-# BUILTIN FALSE false #-}
```

Operations on the two-element type:

```agda
not : Bool → Bool
not true = false
not false = true
```

```agda
_∧_ : Bool → Bool → Bool
true  ∧ true  = true
true  ∧ false = false
false ∧ true  = false
false ∧ false = false
```

```agda
_∨_ : Bool → Bool → Bool
false ∨ false = false
false ∨ true  = true
true  ∨ false = true
true  ∨ true  = true
```

```agda
infixr 6 _∧_
infixr 5 _∨_
```

Alternative notation for disjunction:

```agda
_||_ : Bool → Bool → Bool
_||_ = _∨_
```

```agda
infixr 5 _||_
```

Generic Boolean equality (postulated for abstract types):

```agda
postulate
  _==_ : {A : Set} → A → A → Bool
```

```agda
infix 4 _==_
```

If-then-else syntax for boolean conditionals:

```agda
if_then_else_ : {A : Set} → Bool → A → A → A
if true then x else y = x
if false then x else y = y
```

## Equality and Decidability

Propositional equality is fundamental for reasoning about object identity and
state equivalence.

```agda
data _≡_ {A : Set} (x : A) : A → Set where
  refl : x ≡ x
{-# BUILTIN EQUALITY _≡_ #-}
```

```agda
infix 10 _≡_
```

Symmetry:

```agda
sym : {A : Set} {x y : A} → x ≡ y → y ≡ x
sym refl = refl
```

Transitivity:

```agda
trans : {A : Set} {x y z : A} → x ≡ y → y ≡ z → x ≡ z
trans refl refl = refl
```

Congruence:

```agda
cong : {A B : Set} {x y : A} (f : A → B) → x ≡ y → f x ≡ f y
cong f refl = refl
```

**Negated Equality** - Used extensively in well-formedness conditions to express
"is defined":

```agda
_≢_ : {A : Set} → A → A → Set
x ≢ y = ¬ (x ≡ y)

infix 4 _≢_
```

Decidability type for propositions that can be algorithmically decided:

```agda
data Dec (A : Set) : Set where
  yes : A → Dec A
  no  : (¬ A) → Dec A
```

### Equational Reasoning

Equational reasoning combinators provide a readable syntax for chaining equality proofs:

```agda
infix  1 begin_
infixr 2 _≡⟨⟩_ _≡⟨_⟩_
infix  3 _∎
```

```agda
begin_ : {A : Set} {x y : A} → x ≡ y → x ≡ y
begin p = p
```

```agda
_≡⟨⟩_ : {A : Set} (x : A) {y : A} → x ≡ y → x ≡ y
x ≡⟨⟩ p = p
```

```agda
_≡⟨_⟩_ : {A : Set} (x : A) {y z : A} → x ≡ y → y ≡ z → x ≡ z
x ≡⟨ p ⟩ q = trans p q
```

```agda
_∎ : {A : Set} (x : A) → x ≡ x
x ∎ = refl
```

## Option Types

Option types or Maybe types handle partial operations and potential failures,
crucial for modeling object operations that may not always succeed. Also, useful
for modeling partial functions.

Maybe type for partial operations:

```agda
data Maybe (A : Set) : Set where
  nothing : Maybe A
  just    : A → Maybe A
{-# BUILTIN MAYBE Maybe #-}
```

The value `just` is better than `nothing`:

```agda
just≢nothing : ∀ {A} {x : A} → just x ≡ nothing → ⊥
just≢nothing ()
```

```agda
just-injective : ∀ {A} {x y : A} → just x ≡ just y → x ≡ y
just-injective refl = refl
```

Bind operation for `Maybe`:

```agda
_>>=ᴹ_ : {A B : Set} → Maybe A → (A → Maybe B) → Maybe B
nothing >>=ᴹ f = nothing
(just x) >>=ᴹ f = f x
```

Map operation:

```agda
map-maybe : {A B : Set} → (A → B) → Maybe A → Maybe B
map-maybe f nothing = nothing
map-maybe f (just x) = just (f x)
```

Case analysis for Maybe:

```agda
caseMaybe : {A B : Set} → Maybe A → (A → B) → B → B
caseMaybe (just x) f _ = f x
caseMaybe nothing _ z = z
```

## Natural Numbers

Natural numbers are essential for modeling object identifiers, message
sequences, and temporal ordering.

Natural numbers:

```agda
data ℕ : Set where
  zero : ℕ
  suc  : ℕ → ℕ
```

```agda
{-# BUILTIN NATURAL ℕ #-}
```

Basic arithmetic:

```agda
_+ℕ_ : ℕ → ℕ → ℕ
zero +ℕ n = n
(suc m) +ℕ n = suc (m +ℕ n)
```

```agda
_*ℕ_ : ℕ → ℕ → ℕ
zero *ℕ n = zero
(suc m) *ℕ n = n +ℕ (m *ℕ n)
```

```agda
infixl 6 _+ℕ_
infixl 7 _*ℕ_
```

Natural number subtraction (monus):

```agda
_∸_ : ℕ → ℕ → ℕ
zero ∸ _ = zero
suc m ∸ zero = suc m
suc m ∸ suc n = m ∸ n
```

```agda
infixl 6 _∸_
```

Natural number comparison:

```agda
_≥_ : ℕ → ℕ → Set
zero ≥ zero = ⊤
zero ≥ suc m = ⊥
suc n ≥ zero = ⊤
suc n ≥ suc m = n ≥ m
```

```agda
_≤_ : ℕ → ℕ → Set
n ≤ m = m ≥ n
```

```agda
infix 4 _≥_
infix 4 _≤_
```

Boolean comparison for natural numbers:

```agda
_≤?_ : ℕ → ℕ → Bool
zero ≤? _ = true
(suc m) ≤? zero = false
(suc m) ≤? (suc n) = m ≤? n
```

```agda
_<?_ : ℕ → ℕ → Bool
_ <? zero = false
zero <? (suc _) = true
(suc m) <? (suc n) = m <? n
```

```agda
infix 4 _≤?_
infix 4 _<?_
```

Decidable equality for natural numbers:

```agda
_≟ℕ_ : (m n : ℕ) → Dec (m ≡ n)
zero ≟ℕ zero = yes refl
zero ≟ℕ suc n = no λ ()
suc m ≟ℕ zero = no λ ()
suc m ≟ℕ suc n with m ≟ℕ n
... | yes refl = yes refl
... | no m≢n = no λ { refl → m≢n refl }
```

```agda
infix 4 _≟ℕ_
```

Boolean-valued equality for natural numbers:

```agda
_==ℕ_ : ℕ → ℕ → Bool
zero ==ℕ zero = true
zero ==ℕ suc _ = false
suc _ ==ℕ zero = false
suc m ==ℕ suc n = m ==ℕ n
```

```agda
infix 4 _==ℕ_
```

## Lists

Lists model sequences of messages, object histories, and collections of identifiers.

Lists:

```agda
data List (A : Set) : Set where
  []  : List A
  _∷_ : A → List A → List A
{-# BUILTIN LIST List #-}
```

```agda
infixr 5 _∷_
```

List operations:

```agda
length : {A : Set} → List A → ℕ
length [] = zero
length (x ∷ xs) = suc (length xs)
```

```agda
_++_ : {A : Set} → List A → List A → List A
[] ++ ys = ys
(x ∷ xs) ++ ys = x ∷ (xs ++ ys)
```

```agda
infixr 5 _++_
```

List concatenation is associative:

```agda
++-assoc : ∀ {A} (xs ys zs : List A) → ((xs ++ ys) ++ zs) ≡ (xs ++ (ys ++ zs))
++-assoc [] ys zs = refl
++-assoc (x ∷ xs) ys zs = cong (λ l → x ∷ l) (++-assoc xs ys zs)
```

Right identity for concatenation:

```agda
++-right-id : ∀ {A} (xs : List A) → (xs ++ []) ≡ xs
++-right-id [] = refl
++-right-id (x ∷ xs) = cong (λ l → x ∷ l) (++-right-id xs)
```

Map function:

```agda
map : {A B : Set} → (A → B) → List A → List B
map f [] = []
map f (x ∷ xs) = f x ∷ map f xs
```

Fold right:

```agda
foldr : {A B : Set} → (A → B → B) → B → List A → B
foldr f z [] = z
foldr f z (x ∷ xs) = f x (foldr f z xs)
```

Check if a list is empty:

```agda
null : {A : Set} → List A → Bool
null [] = true
null (_ ∷ _) = false
```

Extract the first element of a non-empty list. This function is partial;
calling it on an empty list is undefined behavior (postulated for simplicity).

```agda
postulate
  head-empty : {A : Set} → A

head : {A : Set} → List A → A
head (x ∷ _) = x
head [] = head-empty
```

Filter elements satisfying a predicate:

```agda
filter : {A : Set} → (A → Bool) → List A → List A
filter p [] = []
filter p (x ∷ xs) with p x
... | true  = x ∷ filter p xs
... | false = filter p xs
```

List membership test:

```agda
elem : {A : Set} → A → List A → Bool
elem x [] = false
elem x (y ∷ ys) = (x == y) || elem x ys
```

Concatenate a list of lists:

```agda
concat : {A : Set} → List (List A) → List A
concat [] = []
concat (xs ∷ xss) = xs ++ concat xss
```

Check for duplicate elements:

```agda
hasDuplicates : {A : Set} → List A → Bool
hasDuplicates [] = false
hasDuplicates (x ∷ xs) = elem x xs || hasDuplicates xs
```

Map a function and concatenate results:

```agda
concatMap : {A B : Set} → (A → List B) → List A → List B
concatMap f [] = []
concatMap f (x ∷ xs) = f x ++ concatMap f xs
```

Check if all elements satisfy a predicate:

```agda
all : {A : Set} → (A → Bool) → List A → Bool
all p [] = true
all p (x ∷ xs) = (p x) ∧ (all p xs)
```

Check if any element satisfies a predicate:

```agda
any : {A : Set} → (A → Bool) → List A → Bool
any p [] = false
any p (x ∷ xs) = p x || any p xs
```

Find first element satisfying a predicate:

```agda
find : {A : Set} → (A → Bool) → List A → Maybe A
find p [] = nothing
find p (x ∷ xs) with p x
... | true  = just x
... | false = find p xs
```

Lookup value in association list (key-value pairs) using custom equality:

```agda
lookup : {A B : Set} → (A → A → Bool) → A → List (A × B) → Maybe B
lookup eq key [] = nothing
lookup eq key ((k , v) ∷ rest) with eq key k
... | true  = just v
... | false = lookup eq key rest
```

Filter and map combined - apply a Maybe-producing function and keep Just results:

```agda
filterMap : {A B : Set} → (A → Maybe B) → List A → List B
filterMap f [] = []
filterMap f (x ∷ xs) with f x
... | nothing = filterMap f xs
... | just y  = y ∷ filterMap f xs
```

## Finite Sets

Finite sets model collections of object identifiers and principal names.

Finite sets using lists (naive implementation):

```agda
FinSet : Set → Set
FinSet A = List A
```

Membership predicate:

```agda
data _∈_ {A : Set} (x : A) : List A → Set where
  here  : {xs : List A} → x ∈ (x ∷ xs)
  there : {y : A} {xs : List A} → x ∈ xs → x ∈ (y ∷ xs)
```

```agda
infix 4 _∈_
```

## Strings and Identifiers

Abstract string type (postulated for simplicity):

```agda
postulate String : Set
{-# BUILTIN STRING String #-}
```

```agda
postulate
   _≟-string_ : (s₁ s₂ : String) → Maybe ⊤
   nat-to-string : ℕ → String
   string-to-nat : String → ℕ
   _++ˢ_ : String → String → String

{-# COMPILE GHC nat-to-string = \n -> Data.Text.pack (show (n :: Integer)) #-}
{-# COMPILE GHC string-to-nat = \s -> read (Data.Text.unpack s) :: Integer #-}
{-# COMPILE GHC _++ˢ_ = \s1 s2 -> Data.Text.append s1 s2 #-}
{-# COMPILE GHC _≟-string_ = \s1 s2 -> if s1 == s2 then Just () else Nothing #-}
```

```agda
infixr 5 _++ˢ_
```

## Predicates and Set Relations

Predicates model properties of elements. A predicate on type `A` is a function `A → Set`.

```agda
Pred : {ℓ : Level} → Set ℓ → Set (lsuc ℓ)
Pred {ℓ} A = A → Set ℓ
```

Membership for predicates (not lists):

```agda
_∈Pred_ : {ℓ : Level} {A : Set ℓ} → A → Pred A → Set ℓ
x ∈Pred P = P x

infix 4 _∈Pred_
```

Subset relation between predicates:

```agda
_⊆_ : {ℓ : Level} {A : Set ℓ} → Pred {ℓ} A → Pred {ℓ} A → Set (lsuc ℓ)
P ⊆ Q = ∀ {x} → P x → Q x
```

```agda
infix 4 _⊆_
```

## Logical Connectives

Logical equivalence (bi-implication):

```agda
record _↔_ (A B : Set) : Set where
  constructor mk↔
  field
    to   : A → B
    from : B → A
```

```agda
open _↔_ public
```

```agda
infix 3 _↔_
```

## Misc

Function composition:

```agda
_∘_ : {A B C : Set} → (B → C) → (A → B) → A → C
(g ∘ h) x = g (h x)
```

## Monad Interface

For supporting Agda's do-notation, we define a simple monad record:

```agda
record RawMonad (M : Set → Set) : Set₁ where
  field
    return : {A : Set} → A → M A
    _>>=_  : {A B : Set} → M A → (A → M B) → M B

  _>>_ : {A B : Set} → M A → M B → M B
  m₁ >> m₂ = m₁ >>= λ _ → m₂
```
