# Interaction Trees

AVM programs are **interaction trees** — data structures that describe a
computation interleaved with observable effects. They are the Rust encoding
of the coinductive `ITree` from the Agda specification.

## Structure

```rust
enum ITree<E, A> {
    Ret(A),          // computation finished with result A
    Tau(Box<ITree>), // silent step (no effect)
    Vis(E, Box<dyn FnOnce(Val) -> ITree>), // emit event E, continue with response
}
```

- `Ret(a)` — terminal node. The program has produced a value.
- `Tau(next)` — an internal step with no observable effect. The interpreter
  eliminates these iteratively.
- `Vis(event, continuation)` — the program emits an `event` (an instruction)
  and waits for a response from the environment. The `continuation` is a
  function that takes the response and produces the next tree.

## Building programs

Use `trigger` to lift an instruction into a single-step tree:

```rust
let tree = trigger(instruction::get_self());
// tree = Vis(GetSelf, |response| Ret(response))
```

Use `bind` (or the `avm_do!` macro) to sequence steps:

```rust
let program = avm_do! {
    let self_id <- trigger(instruction::get_self());
    let input <- trigger(instruction::get_input());
    ret(Val::pair(self_id, input))
};
```

The `avm_do!` macro supports:
- `let x <- expr;` — monadic bind (expr must produce an `ITree`)
- `let x = expr;` — pure variable binding
- `expr;` — sequence (discard result)
- `expr` — final expression (must be an `ITree`)

## Example tree

The following diagram shows a small program that opens a transaction, creates
two objects, commits, and then calls one of them. Each `Vis` node emits an
instruction and receives a typed response before continuing.

<pre class="mermaid">
graph TD
    A[Vis: begin_tx] -->|TxRef| B[Vis: create_obj]
    B -->|ObjectRef| C[Vis: create_obj]
    C -->|ObjectRef| D[Vis: commit_tx]
    D -->|Bool| E[Vis: call]
    E -->|Result| F[Ret: value]
</pre>

## Execution

The interpreter runs the tree iteratively:

1. `Ret(a)` — return the result
2. `Tau(next)` — step to `next` (loop, no recursion)
3. `Vis(instr, cont)` — execute the instruction, feed the result to `cont`

This design avoids stack overflow for arbitrarily long programs.
