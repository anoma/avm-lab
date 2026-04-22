# AVM Lab

The Anoma Virtual Machine — formal specification and Rust implementation.

## Subprojects

| Directory | Description | CI |
|---|---|---|
| [`specs/`](specs/README.md) | Formal Agda specification ([docs](https://anoma.github.io/avm-lab/)) | [![ci specs](https://github.com/anoma/avm-lab/actions/workflows/specs.yml/badge.svg)](https://github.com/anoma/avm-lab/actions/workflows/specs.yml) |
| [`avm/`](avm/) | Rust implementation ([docs](https://anoma.github.io/avm-lab/rust/)) | [![ci avm](https://github.com/anoma/avm-lab/actions/workflows/avm.yml/badge.svg)](https://github.com/anoma/avm-lab/actions/workflows/avm.yml) |

## Rust workspace

```
avm/
├── avm-core/       Core library: types, instructions, interpreter, Tape IR, CFG
├── avm-node/       Distributed runtime: TCP transport, location directory, CLI
├── avm-examples/   PingPong and Battleship demonstrations
└── avm-book/       mdbook documentation with Mermaid diagrams
```

### Quick start

```bash
# Run all tests (113 tests)
cd avm && cargo test --all

# Distributed demo (two terminals):
just avm demo-beta    # Terminal 1
just avm demo-alpha   # Terminal 2
```

### Key features

- **Interaction trees** as the program model with `avm_do!` macro
- **Layered instruction set** matching the formal Agda spec
- **Transactional execution** with snapshot-restore on abort
- **Tape IR** compiler and flat register-based interpreter
- **Control flow graph** extraction with Mermaid visualization
- **Distributed runtime** with transparent TCP-based remote calls
- **Typed protocols** with `Into<Val>` / `TryFrom<Val>` for compile-time safety
