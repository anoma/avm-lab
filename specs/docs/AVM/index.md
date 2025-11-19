---
title: AVM Core Specification
icon: fontawesome/solid/code
hide:
  - navigation
  - toc
---

This section collects the formal specification of the Anoma Virtual Machine (AVM): the instruction semantics, execution model, and runtime behavior.

<div class="grid cards" markdown>

- :fontawesome-solid-cube: 1. System Model

  ***

  The assumptions for distributed, transactional execution.

  [:octicons-arrow-right-24: View Page](AVM/SystemModel.md)

- :fontawesome-solid-layer-group: 2. Runtime Context

  ***

  State, errors, and trace types that the interpreter and runtime share.

  [:octicons-arrow-right-24: View Page](AVM/Context.md)

- :fontawesome-solid-code: 3. Instruction Set

  ***

  The primitive operations for object lifecycle, transactions, and distributed execution.

  [:octicons-arrow-right-24: View Page](AVM/Instruction.md)

- :fontawesome-solid-gears: 4. Interpreter

  ***

  Operational semantics that define how instructions transform state.

  [:octicons-arrow-right-24: View Page](AVM/Interpreter.md)

- :material-road: 5. Runtime Guidance

  ***

  Non-normative guidance for building a runtime that meets the spec.

  [:octicons-arrow-right-24: View Page](AVM/Runtime.md)

</div>
