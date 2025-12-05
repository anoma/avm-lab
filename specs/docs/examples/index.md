---
title: AVM Examples
icon: fontawesome/solid/flask
---

This directory contains examples demonstrating the AVM object model and
interaction semantics.

<div class="grid cards" markdown>

- :fontawesome-solid-arrow-right-arrow-left: **PingPong**

  ***

  The classic ping-pong example demonstrating object-to-object communication via
  mutual message passing.

  ```bash
  cd specs/docs
  agda --compile examples/PingPong/Runner.lagda.md
  ./Runner
  ```

  [:octicons-arrow-right-24: View Example](examples/PingPong/Main.lagda.md)

- :fontawesome-solid-chess-board: **Battleship**

  ***

  A Battleship game demonstrating stateful object interactions, game orchestration,
  and turn-based coordination between multiple player objects.

  ```bash
  cd specs/docs
  agda --compile examples/Battleship/Runner.lagda.md
  ./Runner
  ```

  [:octicons-arrow-right-24: View Example](examples/Battleship/PlayerBoard.lagda.md)

</div>
