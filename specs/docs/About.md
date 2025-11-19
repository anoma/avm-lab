---
title: About the AVM
icon: material/information
tags:
  - AVM
  - design
  - overview
hide:
  - navigation
---

The Anoma Virtual Machine (AVM) is a transactional virtual machine architecture
designed for message-driven object-oriented computation. The system provides a
set of primitives to support pure functional programming, distributed
computation primitives, and nondeterministic choice mechanisms enabling
intent-based transaction matching.

## Specification Scope

The AVM specification establishes denotational semantics for programs, defining
what programs signify rather than dictating execution strategies. Programs are
typed as Agda@AVMProgram, with operational semantics defined through
[interaction trees](Background/InteractionTrees.lagda.md). This coinductive
formalization separates effectful operations
[instructions](AVM/Instruction.lagda.md) from pure computation. The semantics
form a small-step transition system over a global state of live objects and
metadata.

The specification covers [instruction semantics, state
transitions](AVM/Instruction.lagda.md), [runtime context](AVM/Context.lagda.md)
(including error handling and event tracing), and [system model
assumptions](AVM/SystemModel.md) addressing network behavior, failure modes, and
trust boundaries. Althought, not complete, we include a rundimentary
[interpreter](AVM/Interpreter.lagda.md) section to guide implementation.

The specification does not dictate platform implementation choices—scheduling, persistence,
network protocols, or resource management remain platform concerns. This separation allows
different implementations to target varied deployments while conforming to one semantic
specification.

## Design Commitments

The following commitments guide runtime implementations while remaining independent from the
formal semantics presented in this specification. The instruction set reflects these commitments,
and the system model assumes these commitments are realised.

1. Object-Centric Computation Model: Objects maintain immutable input histories that grow monotonically. 

2. Objects are created and destroyed exactly once in their lifecycle. 

3. Code objects execute when invoked by a message. 

3. At any moment, execution is localised to a single object context. That is, execution occurs within a single object.

4. Distributed Execution Model: Program execution may migrate across machine boundaries. 

5. Each object is associated with exactly one controller authority at any time. 

6. Programs can inspect the current history identifier and move to specific target history identifiers.

7. Transactional Semantics: Transaction boundaries use Agda@beginTx, Agda@commitTx, and Agda@abortTx to provide
atomic commit semantics. Current implementations use single-controller coordination; multi-party composition
remains an active goal.

8. Constraint Programming and Nondeterminism: The instruction set provides two constraint layers with distinct
execution models. Finite domain instructions (Agda@newVar, Agda@narrow, Agda@post, Agda@label) enable constraint
propagation and search with call-time choice. Nondeterminism instructions (Agda@choose, Agda@require) support
commit-time choice where selections and constraints accumulate during execution and are validated atomically
at transaction commit.

9. Intent Composition: The architecture supports composing multiple transactional segments into a unified atomic
transaction.

10. Message Sequence Constraints: Programs may specify constraints over message ordering when composing multi-party
intents/programs.

11. Pure Computation Representation: The AVM architecture does not prescribe a fixed representation for pure functions. 
The admissible function set can grow without altering the virtual machine semantics.

These design commitments guide the architecture and link user-facing features with the formal specification.
They are realized through the system model assumptions detailed in [AVM System Model](AVM/SystemModel.md),
which specifies the network model, failure semantics, transaction isolation guarantees, and other normative
requirements that runtime implementations must satisfy.
