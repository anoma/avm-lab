# Error Hierarchy

The AVM uses a compositional error type that mirrors the instruction set layers.
Each instruction family has its own error enum, composed into the top-level
`AVMError`.

## Composition chain

```
AVMError
├── PureLayerError
│   ├── TxLayerError
│   │   ├── BaseError
│   │   │   ├── ObjError
│   │   │   ├── IntrospectError
│   │   │   ├── ReflectError
│   │   │   └── ReifyError
│   │   └── TxError
│   └── PureError
├── MachineError
├── ControllerError
├── FdError
└── NondetError
```

Any leaf error can be converted to `AVMError` with `?` thanks to `From`
implementations.

## Object errors (`ObjError`)

| Variant | When |
|---|---|
| `NotFound(id)` | Object doesn't exist in store or pending creates |
| `AlreadyDestroyed(id)` | Object was already marked for destruction |
| `AlreadyExists(id)` | Duplicate creation |
| `RejectedCall(id)` | Object's behavior rejected the input |
| `MetadataCorruption(id)` | Metadata store is inconsistent |

## Transaction errors (`TxError`)

| Variant | When |
|---|---|
| `Conflict(id)` | Read set validation failed at commit |
| `NotFound(id)` | Transaction ID doesn't match active tx |
| `NoActiveTx` | Commit/abort without begin |
| `InvalidDuringTx` | Nested begin_tx or teleport during tx |

## Machine errors (`MachineError`)

| Variant | When |
|---|---|
| `Unreachable(id)` | Target machine is unreachable |
| `TeleportDuringTx` | Teleport attempted inside a transaction |

## Controller errors (`ControllerError`)

| Variant | When |
|---|---|
| `NotAvailable(id)` | Object doesn't exist for transfer |
| `NoController(id)` | Object has no controller for freeze |
| `UnauthorizedTransfer(id)` | Transfer not permitted |
