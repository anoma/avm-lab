---
title: AVM Core Specification
icon: fontawesome/solid/code
hide:
  - navigation
  - toc
---

This section collects the formal specification of the Anoma Virtual Machine (AVM): the instruction semantics, execution model, and runtime behavior.

<div class="grid cards" markdown>

- :fontawesome-solid-book: 1. Background

  ***

  Foundational mathematical concepts including basic types, interaction trees, and object models.

  [:octicons-arrow-right-24: View Page](../Background/index.md)

- :fontawesome-solid-cube: 2. System Model

  ***

  The assumptions for distributed, transactional execution.

  [:octicons-arrow-right-24: View Page](AVM/SystemModel.md)

- :fontawesome-solid-layer-group: 3. Runtime Context

  ***

  State, errors, and trace types that the interpreter and runtime share.

  [:octicons-arrow-right-24: View Page](AVM/Context.md)

- :fontawesome-solid-code: 4. Instruction Set

  ***

  The primitive operations for object lifecycle, transactions, and distributed execution.

  [:octicons-arrow-right-24: View Page](AVM/Instruction.md)

- :fontawesome-solid-gears: 5. Interpreter

  ***

  Operational semantics that define how instructions transform state.

  [:octicons-arrow-right-24: View Page](AVM/Interpreter.md)

- :material-road: 6. Runtime Guidance

  ***

  Non-normative guidance for building a runtime that meets the spec.

  [:octicons-arrow-right-24: View Page](AVM/Runtime.md)

</div>
