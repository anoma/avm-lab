# Battleship Tutorial

This tutorial demonstrates transactions and state management through a
simplified Battleship game.

## Overview

The game creates two board objects, places ships via `setState`, and resolves
attacks by checking coordinates against the stored ship list.

## Board behavior

The board behavior handles two message types:

- `["ship", x, y, length]` — place a ship by reading the current state,
  appending the ship, and writing back.
- `["coord", x, y]` — check if the coordinate hits any ship.

```rust
"ship" => {
    avm_do! {
        let current_state <- trigger(get_state());
        let new_state = { /* append ship to list */ };
        trigger(set_state(new_state));
        ret(Val::Bool(true))
    }
}
"coord" => {
    avm_do! {
        let current_state <- trigger(get_state());
        let hit = /* check if (x,y) intersects any ship */;
        ret(Val::Bool(hit))
    }
}
```

## Game setup

```rust
avm_do! {
    // Atomic creation
    let tx <- trigger(begin_tx(None));
    let board1 <- trigger(create_obj("board", None));
    let board2 <- trigger(create_obj("board", None));
    trigger(commit_tx(tx));

    // Place ships
    trigger(call(board1, ship(0, 0, 3)));
    trigger(call(board2, ship(1, 1, 4)));

    // Attack
    let hit  <- trigger(call(board2, coord(1, 1)));  // hit!
    let miss <- trigger(call(board2, coord(9, 9)));  // miss
    ret(Val::list(vec![hit, miss]))
}
```

## Key concepts demonstrated

- **Transactional creation**: both boards are created atomically.
- **State management**: `get_state` / `set_state` maintain ship lists
  across calls.
- **Message-based dispatch**: the board behavior pattern-matches on the
  input message tag to decide what to do.
