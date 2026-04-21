# Transactions

Transactions provide serializable snapshot isolation for groups of operations.
All state changes within a transaction are buffered and applied atomically
on commit.

## Lifecycle

```
begin_tx(controller?) → TxId
    ↓
  create / destroy / call / setState / transfer  (buffered)
    ↓
commit_tx(id) → Bool   OR   abort_tx(id)
```

## Commit sequence

<pre class="mermaid">
sequenceDiagram
    participant P as Program
    participant I as Interpreter
    participant S as Store
    P->>I: begin_tx
    I->>S: snapshot store
    I-->>P: TxRef
    P->>I: create_obj / call / setState
    I->>S: buffer in overlay
    P->>I: commit_tx
    I->>S: validate read set
    I->>S: apply creates → transfers → writes → states → destroys
    I-->>P: true
</pre>

## Commit protocol

When `commit_tx` is called, the interpreter:

1. **Validates the read set** — checks that no observed object was concurrently
   destroyed outside the transaction.
2. **Applies changes in dependency order**:
   - Creates (new objects enter the store)
   - Transfers (ownership changes)
   - Writes (call effects, already executed)
   - States (`setState` updates)
   - Destroys (objects removed from the store)
3. **Clears the overlay** — all pending buffers are emptied.

If validation fails, the transaction is rolled back and `commit_tx` returns
`false`.

## Abort

`abort_tx` discards all pending changes immediately. The store is unchanged.

## Overlay buffers

During an active transaction, the `State` struct maintains:

| Buffer | Contents |
|---|---|
| `tx_log` | `(ObjectId, Input)` pairs for each call |
| `creates` | Pending object creations |
| `destroys` | Objects marked for destruction |
| `observed` | Read set (for conflict detection) |
| `pending_transfers` | Ownership transfers |
| `pending_states` | `setState` updates |

## Restrictions

- Only one transaction can be active at a time.
- `teleport` is forbidden during a transaction (`ErrTeleportDuringTx`).
- Nested transactions are not supported.
