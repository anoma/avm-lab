# AVM Specification [![ci specs](https://github.com/anoma/avm-lab/actions/workflows/specs.yml/badge.svg)](https://github.com/anoma/avm-lab/actions/workflows/specs.yml)

The `specs` directory contains a formal description of the Anoma Virtual Machine
components, including:

- the [instruction set](https://anoma.github.io/avm-lab/specs/AVM/Instruction/),
- the [runtime environment](https://anoma.github.io/avm-lab/specs/AVM/Runtime/),
- [an object model](https://anoma.github.io/avm-lab/specs/AVM/Objects/), and
- a preliminary rustic [interpreter semantics](https://anoma.github.io/avm-lab/specs/AVM/Interpreter/).

The documentation is built using [MkDocs](https://www.mkdocs.org/) and Agda.

## Setup

### Prerequisites

- **Agda 2.7+**: The specifications are type-checked using Agda version 2.7 or later.
  - On macOS: `brew install agda`
  - On other platforms: See [Agda installation guide](https://agda.readthedocs.io/en/latest/getting-started/installation.html)
- **uv**: Python package manager for managing dependencies and render the documentation site.

### Installation

After cloning the repository, initialize the Agda library submodules:

```bash
git submodule update --init --recursive
```

Then install the Python dependencies:

```bash
cd specs/config
uv sync
```

## Type-checking Agda files

The Agda source files are in [docs/](docs/). You can type-check for example the
file [docs/AVM/Runtime.lagda.md](docs/AVM/Runtime.lagda.md) with:

```bash
cd specs/docs
agda AVM/Runtime.lagda.md
```

## Building Documentation

Build the documentation site locally (you'll need to have `agda-mkdocs`
installed):

```bash
cd specs/config
uv run mkdocs serve --config-file mkdocs.yml
```
