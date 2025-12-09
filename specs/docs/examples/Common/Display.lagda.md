# Common Display Functions

This module provides standard display functions for rendering AVM execution
results, traces, and observability events. The module is parameterized by
type-specific display functions for values, object identifiers, transaction
identifiers, machine identifiers, and controller identifiers, enabling reuse
across diverse example implementations with different concrete types.

```agda
{-# OPTIONS --without-K --type-in-type --guardedness #-}

open import Background.BasicTypes

module examples.Common.Display
  (Val : Set)
  (ObjectId : Set)
  (TxId : Set)
  (ControllerId : Set)
  (MachineId : Set)
  (ObjectBehaviour : Set)
  (showVal : Val → String)
  (showOid : ObjectId → String)
  (showTxId : TxId → String)
  (showMid : MachineId → String)
  (showCid : ControllerId → String)
  where

open import Background.InteractionTrees
open import AVM.Context Val ObjectId MachineId ControllerId TxId ObjectBehaviour

-- Timestamp display helper converts natural number timestamps to string
-- representation. All event timestamps in the AVM are represented as natural
-- numbers indicating logical clock values.
showNat : ℕ → String
showNat n = nat-to-string n
```

## Event Type Display

Event type rendering produces human-readable string representations of
observability events, formatting each event constructor with appropriate
contextual information.

```agda
showEventType : EventType → String
showEventType (ObjectCreated oid behaviorName) = "ObjectCreated(" ++ˢ showOid oid ++ˢ ", \"" ++ˢ behaviorName ++ˢ "\")"
showEventType (ObjectDestroyed oid) = "ObjectDestroyed(" ++ˢ showOid oid ++ˢ ")"
showEventType (ObjectCalled oid inp mOut) =
  "ObjectCalled(" ++ˢ showOid oid ++ˢ ", " ++ˢ showVal inp ++ˢ
  caseMaybe mOut (λ out → " -> " ++ˢ showVal out) "" ++ˢ ")"
showEventType (MessageReceived oid inp) = "MessageReceived(" ++ˢ showOid oid ++ˢ ", " ++ˢ showVal inp ++ˢ ")"
showEventType (ObjectMoved oid from to) = "ObjectMoved(" ++ˢ showOid oid ++ˢ ", " ++ˢ showMid from ++ˢ " -> " ++ˢ showMid to ++ˢ ")"
showEventType (ExecutionMoved from to) = "ExecutionMoved(" ++ˢ showMid from ++ˢ " -> " ++ˢ showMid to ++ˢ ")"
showEventType (ObjectFetched oid mid) = "ObjectFetched(" ++ˢ showOid oid ++ˢ ", " ++ˢ showMid mid ++ˢ ")"
showEventType (ObjectTransferred oid fromCtrl toCtrl) = "ObjectTransferred(" ++ˢ showOid oid ++ˢ ", " ++ˢ showCid fromCtrl ++ˢ " -> " ++ˢ showCid toCtrl ++ˢ ")"
showEventType (ObjectFrozen oid ctrl) = "ObjectFrozen(" ++ˢ showOid oid ++ˢ ", " ++ˢ showCid ctrl ++ˢ ")"
showEventType (FunctionUpdated name) = "FunctionUpdated(" ++ˢ name ++ˢ ")"
showEventType (TransactionStarted txid) = "TransactionStarted(" ++ˢ showTxId txid ++ˢ ")"
showEventType (TransactionCommitted txid) = "TransactionCommitted(" ++ˢ showTxId txid ++ˢ ")"
showEventType (TransactionAborted txid) = "TransactionAborted(" ++ˢ showTxId txid ++ˢ ")"
showEventType (ErrorOccurred err) = "ErrorOccurred(...)"
```

## Log Entry Display

Log entry rendering formats individual trace entries with timestamp, event type,
and executing controller information, providing complete observability context
for each logged operation.

```agda
showLogEntry : LogEntry → String
showLogEntry entry =
  "[" ++ˢ showNat (LogEntry.timestamp entry) ++ˢ "] " ++ˢ
  showEventType (LogEntry.eventType entry) ++ˢ
  " @" ++ˢ caseMaybe (LogEntry.executingController entry) (λ c → showCid c) "none"
```

## Trace Display

Execution trace rendering converts a sequence of log entries into a multi-line
string representation, with each entry on a separate line. Empty traces are
represented with a placeholder message.

```agda
showTrace : Trace → String
showTrace [] = "(no events)"
showTrace trace = foldr (λ entry acc → showLogEntry entry ++ˢ "\n" ++ˢ acc) "" trace
```

## Error Display

Error rendering produces a generic error message string. This implementation
provides a simplified error representation; production systems would typically
include detailed error diagnostics.

```agda
showError : AVMError → String
showError _ = "execution-error"
```

## Result Display

Result rendering produces formatted output for AVM computation results,
displaying either error messages or success values with their associated
execution traces. The function is parameterized by a value-specific display
function to support arbitrary result types.

```agda
showResult : ∀ {A} → (A → String) → AVMResult A → String
showResult showA (failure err) = "Error: " ++ˢ showError err
showResult showA (success res) =
  "Success: " ++ˢ showA (Success.value res) ++ˢ "\n\n" ++ˢ
  "Trace:\n" ++ˢ showTrace (Success.trace res)
```
