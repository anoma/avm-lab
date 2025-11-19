---
title: Appendix - Semantics and Event Architecture
icon: material/book-open-page-variant
tags:
  - semantics
  - interaction trees
  - compiler correctness
  - weak bisimulation
---

# Appendix: Semantics and Event Architecture

## Semantics

Both AVM and resource machine programs denote interaction trees. Compiler
correctness uses weak bisimulation to relate these denotations.

```mermaid
graph TB
    subgraph syntax["Syntax"]
        AVM_P["p : AVM"]
        RM_P["compile(p) : RM"]
    end

    subgraph semantics["Denotational Semantics"]
        AVM_IT["⟦ p ⟧ : ITree (Instr₂ Safe) A"]
        TRANS_IT["translate(⟦ p ⟧) : ITree (Instr₂ Safe) A"]
        RM_IT["⟦ compile(p) ⟧ : ITree (Instr₂ Safe) A"]
    end

    AVM_P -->|compile| RM_P
    AVM_P -.->|⟦·⟧_AVM| AVM_IT
    RM_P -.->|⟦·⟧_RM| RM_IT

    AVM_IT -.->|translate| TRANS_IT
    TRANS_IT ==>|"≈ (weak bisim)"| RM_IT

    style syntax fill:#f0f0f0,stroke:#666,stroke-width:2px
    style semantics fill:#f0f0f0,stroke:#666,stroke-width:2px
```

**Compiler Correctness.** For any AVM program p, compilation preserves
semantics: translate(⟦ p ⟧) ≈ ⟦ compile(p) ⟧.

The `translate` function maps AVM events (E_AVM) to resource machine events
(E_RM), preserving computational structure.

## Event Interpretation Architecture

Interaction trees interpret via event translation and handlers:

```mermaid
graph LR
    AVM_IT["ITree (Instr₂ Safe) A"]
    RM_IT["ITree (Instr₂ Safe) A"]
    Result["M A"]

    AVM_IT -->|"translate<br/>(event morphism)"| RM_IT
    RM_IT -->|"interpret h<br/>(handler)"| Result

    subgraph handler["Handler: h : (Instr₂ Safe) ~> M"]
        direction TB
        State["State<br/>(Resource State)"]
        Error["Error<br/>(Validation)"]
        IO["IO<br/>(External Effects)"]
    end

    RM_IT -.->|uses| handler

    style handler fill:#f0f0f0,stroke:#666,stroke-width:2px
```

Event translation maps AVM operations to resource machine primitives. The
handler interprets primitives as concrete effects: state management, error
handling, and IO.
